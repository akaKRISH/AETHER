use crate::execution::{attempt_lease, read_blob, run_context};
use crate::*;

impl Store {
    pub fn add_claim(&mut self, claim: &NewClaim) -> Result<()> {
        validate_id(&claim.id)?;
        validate_id(&claim.campaign_id)?;
        validate_text(&claim.statement)?;
        self.write(|tx| {
            ensure_campaign_open(tx, &claim.campaign_id)?;
            let count: i64 = tx.query_row(
                "SELECT COUNT(*) FROM claim WHERE campaign_id = ?",
                [&claim.campaign_id],
                |row| row.get(0),
            )?;
            if count >= MAX_PAGE_SIZE as i64 {
                return Err(StoreError::Conflict(
                    "local campaign claim limit is reached",
                ));
            }
            tx.execute(
                "INSERT INTO claim VALUES (?, ?, ?, 'proposed')",
                params![claim.id, claim.campaign_id, claim.statement],
            )?;
            append(
                tx,
                &claim.campaign_id,
                "claim.proposed",
                json!({"id": claim.id}),
            )
        })
    }

    pub fn add_dependency(&mut self, claim_id: &str, prerequisite_id: &str) -> Result<()> {
        validate_id(claim_id)?;
        validate_id(prerequisite_id)?;
        self.write(|tx| {
            let (campaign_id, _) = claim_context(tx, claim_id)?;
            let (prerequisite_campaign, prerequisite_state) = claim_context(tx, prerequisite_id)?;
            require_campaign(&campaign_id, &prerequisite_campaign)?;
            let creates_cycle: bool = tx.query_row(
                "WITH RECURSIVE upstream(id) AS (
                    SELECT ?1 UNION SELECT d.prerequisite_id FROM claim_dependency d JOIN upstream u ON d.claim_id = u.id
                 ) SELECT EXISTS(SELECT 1 FROM upstream WHERE id = ?2)",
                params![prerequisite_id, claim_id], |row| row.get(0),
            )?;
            if creates_cycle {
                return Err(StoreError::Conflict("claim dependency cycle"));
            }
            tx.execute("INSERT INTO claim_dependency VALUES (?, ?)", params![claim_id, prerequisite_id])?;
            append(
                tx, &campaign_id, "claim.dependency_added",
                json!({"claim_id": claim_id, "prerequisite_id": prerequisite_id, "relation": "depends_on"}),
            )?;
            if prerequisite_state != ClaimState::Verified {
                revoke(tx, &campaign_id, claim_id, "new prerequisite is unverified")?;
            }
            Ok(())
        })
    }

    pub fn link_evidence(&mut self, evidence: &NewEvidence) -> Result<()> {
        validate_id(&evidence.id)?;
        validate_id(&evidence.claim_id)?;
        self.write(|tx| {
            let (campaign_id, state) = claim_context(tx, &evidence.claim_id)?;
            let (metric_id, artifact_attempt_id, artifact_sha256, attempt_id) = match &evidence.target {
                EvidenceTarget::Metric { metric_id } => {
                    validate_id(metric_id)?;
                    let attempt: String = tx.query_row(
                        "SELECT attempt_id FROM metric WHERE id = ?", [metric_id], |row| row.get(0),
                    ).optional()?.ok_or(StoreError::NotFound("metric"))?;
                    (Some(metric_id), None, None, attempt)
                }
                EvidenceTarget::Artifact { attempt_id, sha256 } => {
                    validate_id(attempt_id)?;
                    validate_hash(sha256)?;
                    let exists: bool = tx.query_row(
                        "SELECT EXISTS(SELECT 1 FROM attempt_artifact WHERE attempt_id = ? AND sha256 = ?)",
                        params![attempt_id, sha256], |row| row.get(0),
                    )?;
                    if !exists {
                        return Err(StoreError::NotFound("attempt artifact"));
                    }
                    (None, Some(attempt_id), Some(sha256), attempt_id.clone())
                }
            };
            let lease = attempt_lease(tx, &attempt_id)?;
            require_campaign(&campaign_id, &run_context(tx, &lease.run_id)?.campaign_id)?;
            if lease.outcome.is_none() {
                return Err(StoreError::Conflict("evidence attempt must be complete"));
            }
            tx.execute(
                "INSERT INTO evidence (id, claim_id, metric_id, artifact_attempt_id, artifact_sha256, relation)
                 VALUES (?, ?, ?, ?, ?, ?)",
                params![evidence.id, evidence.claim_id, metric_id, artifact_attempt_id, artifact_sha256, evidence.relation],
            )?;
            if state == ClaimState::Proposed && evidence.relation == EvidenceRelation::Supports {
                set_claim_state(tx, &evidence.claim_id, ClaimState::Supported)?;
            }
            append(tx, &campaign_id, "claim.evidence_linked", json!(evidence))
        })
    }

    /// Record a trusted evaluator's comparison of distinct original and replay attempts.
    pub fn record_replication(&mut self, replication: &NewReplication) -> Result<()> {
        validate_id(&replication.id)?;
        validate_id(&replication.original_attempt_id)?;
        validate_id(&replication.replay_attempt_id)?;
        validate_text(&replication.reason)?;
        self.write(|tx| {
            let original = attempt_lease(tx, &replication.original_attempt_id)?;
            let replay = attempt_lease(tx, &replication.replay_attempt_id)?;
            let original_run = run_context(tx, &original.run_id)?;
            let replay_run = run_context(tx, &replay.run_id)?;
            require_campaign(&original_run.campaign_id, &replay_run.campaign_id)?;
            if replay_run.action != Action::Replay
                || replay_run.parent_run_id.as_deref() != Some(original.run_id.as_str())
                || original.run_id == replay.run_id
                || original.outcome != Some(AttemptOutcome::Succeeded)
                || replay.outcome.is_none()
                || (replication.passed && replay.outcome != Some(AttemptOutcome::Succeeded))
            {
                return Err(StoreError::Conflict(
                    "replication requires a completed fresh replay of a successful parent",
                ));
            }
            check_attempt_blobs(tx, &replication.original_attempt_id)?;
            check_attempt_blobs(tx, &replication.replay_attempt_id)?;
            tx.execute(
                "INSERT INTO replication VALUES (?, ?, ?, ?, ?)",
                params![
                    replication.id,
                    replication.original_attempt_id,
                    replication.replay_attempt_id,
                    replication.passed,
                    replication.reason
                ],
            )?;
            append(
                tx,
                &original_run.campaign_id,
                "run.replicated",
                json!(replication),
            )
        })
    }

    /// Record an authoritative rule outcome while checking frozen policy and required independent evidence.
    pub fn record_verification(&mut self, verification: &NewVerification) -> Result<()> {
        validate_id(&verification.id)?;
        validate_id(&verification.claim_id)?;
        validate_hash(&verification.rule_hash)?;
        validate_hash(&verification.evaluator_hash)?;
        validate_text(&verification.reason)?;
        self.write(|tx| {
            let (campaign_id, previous) = claim_context(tx, &verification.claim_id)?;
            let (protocol, evaluator, requires_replay): (String, String, bool) = tx.query_row(
                "SELECT protocol_hash, evaluator_hash, requires_replay FROM campaign WHERE id = ?",
                [&campaign_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )?;
            if verification.rule_hash != protocol || verification.evaluator_hash != evaluator {
                return Err(StoreError::Conflict(
                    "verification must match the frozen protocol and evaluator",
                ));
            }
            if let Some(replay_id) = verification.replay_attempt_id.as_deref() {
                validate_id(replay_id)?;
                let replay = attempt_lease(tx, replay_id)?;
                let run = run_context(tx, &replay.run_id)?;
                require_campaign(&campaign_id, &run.campaign_id)?;
                if run.action != Action::Replay || replay.outcome.is_none() {
                    return Err(StoreError::Conflict("verification replay must be complete"));
                }
            }
            let mut support = None;
            let state = match verification.verdict {
                VerificationVerdict::Verified => {
                    support = Some(eligible_support(tx, verification, requires_replay)?);
                    ClaimState::Verified
                }
                VerificationVerdict::Rejected => ClaimState::Rejected,
                VerificationVerdict::Inconclusive => ClaimState::Inconclusive,
            };
            if previous == ClaimState::Verified && state != ClaimState::Verified {
                revoke(
                    tx,
                    &campaign_id,
                    &verification.claim_id,
                    &verification.reason,
                )?;
            }
            tx.execute(
                "INSERT INTO verification VALUES (?, ?, ?, ?, ?, ?, ?)",
                params![
                    verification.id,
                    verification.claim_id,
                    verification.verdict,
                    verification.rule_hash,
                    verification.evaluator_hash,
                    verification.replay_attempt_id,
                    verification.reason,
                ],
            )?;
            set_claim_state(tx, &verification.claim_id, state)?;
            append(
                tx,
                &campaign_id,
                "claim.verdict_recorded",
                json!({
                    "record": verification,
                    "support_evidence_id": support,
                }),
            )
        })
    }

    pub fn record_contradiction(&mut self, contradiction: &NewContradiction) -> Result<()> {
        validate_id(&contradiction.id)?;
        validate_id(&contradiction.claim_id)?;
        validate_id(&contradiction.evidence_id)?;
        validate_text(&contradiction.reason)?;
        self.write(|tx| {
            let (campaign_id, _) = claim_context(tx, &contradiction.claim_id)?;
            let (claim_id, relation, invalidated): (String, EvidenceRelation, bool) = tx
                .query_row(
                    "SELECT claim_id, relation, invalidated FROM evidence WHERE id = ?",
                    [&contradiction.evidence_id],
                    |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
                )
                .optional()?
                .ok_or(StoreError::NotFound("evidence"))?;
            if claim_id != contradiction.claim_id
                || invalidated
                || !matches!(
                    relation,
                    EvidenceRelation::Contradicts | EvidenceRelation::Invalidates
                )
            {
                return Err(StoreError::Conflict(
                    "contradiction requires active negative evidence for this claim",
                ));
            }
            tx.execute(
                "INSERT INTO contradiction VALUES (?, ?, ?, ?, ?)",
                params![
                    contradiction.id,
                    contradiction.claim_id,
                    contradiction.evidence_id,
                    contradiction.reason,
                    contradiction.invalidates
                ],
            )?;
            append(
                tx,
                &campaign_id,
                "claim.contradiction_recorded",
                json!(contradiction),
            )?;
            if contradiction.invalidates {
                revoke(
                    tx,
                    &campaign_id,
                    &contradiction.claim_id,
                    &contradiction.reason,
                )?;
            }
            Ok(())
        })
    }

    pub fn invalidate_evidence(&mut self, evidence_id: &str, reason: &str) -> Result<()> {
        validate_id(evidence_id)?;
        validate_text(reason)?;
        self.write(|tx| {
            let (claim_id, relation, invalidated): (String, EvidenceRelation, bool) = tx.query_row(
                "SELECT claim_id, relation, invalidated FROM evidence WHERE id = ?", [evidence_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            ).optional()?.ok_or(StoreError::NotFound("evidence"))?;
            if invalidated {
                return Ok(());
            }
            let (campaign_id, state) = claim_context(tx, &claim_id)?;
            tx.execute("UPDATE evidence SET invalidated = 1 WHERE id = ?", [evidence_id])?;
            append(tx, &campaign_id, "evidence.invalidated", json!({"id": evidence_id, "claim_id": claim_id, "reason": reason}))?;
            if matches!(relation, EvidenceRelation::Supports | EvidenceRelation::DerivedFrom | EvidenceRelation::Reproduces) {
                revoke(tx, &campaign_id, &claim_id, reason)?;
            }
            if state == ClaimState::Supported && relation == EvidenceRelation::Supports {
                let has_support: bool = tx.query_row(
                    "SELECT EXISTS(SELECT 1 FROM evidence WHERE claim_id = ? AND relation = 'supports' AND invalidated = 0)",
                    [&claim_id], |row| row.get(0),
                )?;
                if !has_support {
                    set_claim_state(tx, &claim_id, ClaimState::Proposed)?;
                }
            }
            Ok(())
        })
    }
}

fn eligible_support(
    tx: &Transaction<'_>,
    verification: &NewVerification,
    requires_replay: bool,
) -> Result<String> {
    let unresolved: bool = tx.query_row(
        "SELECT EXISTS(
            SELECT 1 FROM claim_dependency d JOIN claim required ON required.id = d.prerequisite_id
                WHERE d.claim_id = ?1 AND required.state != 'verified'
            UNION ALL
            SELECT 1 FROM contradiction c JOIN evidence e ON e.id = c.evidence_id
                WHERE c.claim_id = ?1 AND c.invalidates = 1 AND e.invalidated = 0
        )",
        [&verification.claim_id],
        |row| row.get(0),
    )?;
    if unresolved {
        return Err(StoreError::Conflict(
            "unresolved prerequisite or invalidating contradiction",
        ));
    }
    if requires_replay && verification.replay_attempt_id.is_none() {
        return Err(StoreError::Conflict(
            "frozen protocol requires an independent replay",
        ));
    }
    let eligible: (String, String) = tx.query_row(
        "SELECT e.id, m.attempt_id
         FROM evidence e JOIN metric m ON m.id = e.metric_id JOIN attempt a ON a.id = m.attempt_id
         WHERE e.claim_id = ?1 AND m.evaluator_hash = ?2 AND e.relation = 'supports'
             AND e.invalidated = 0 AND a.outcome = 'succeeded'
             AND (?3 IS NULL OR (EXISTS (
                 SELECT 1 FROM replication r JOIN metric replay ON replay.attempt_id = r.replay_attempt_id
                 WHERE r.original_attempt_id = m.attempt_id AND r.replay_attempt_id = ?3
                     AND r.passed = 1 AND replay.name = m.name AND replay.unit = m.unit
                     AND replay.evaluator_hash = m.evaluator_hash AND replay.split_hash = m.split_hash
             ) AND NOT EXISTS (
                 SELECT 1 FROM evidence obsolete LEFT JOIN metric referenced ON referenced.id = obsolete.metric_id
                 WHERE obsolete.claim_id = ?1 AND obsolete.relation = 'reproduces' AND obsolete.invalidated = 1
                   AND coalesce(obsolete.artifact_attempt_id, referenced.attempt_id) = ?3
             )))
         ORDER BY e.id LIMIT 1",
        params![verification.claim_id, verification.evaluator_hash, verification.replay_attempt_id],
        |row| Ok((row.get(0)?, row.get(1)?)),
    ).optional()?.ok_or(StoreError::Conflict("verification requires eligible trusted metric and replay evidence"))?;
    check_attempt_blobs(tx, &eligible.1)?;
    if let Some(replay) = verification.replay_attempt_id.as_deref() {
        check_attempt_blobs(tx, replay)?;
    }
    Ok(eligible.0)
}

fn claim_context(conn: &Connection, id: &str) -> Result<(String, ClaimState)> {
    conn.query_row(
        "SELECT campaign_id, state FROM claim WHERE id = ?",
        [id],
        |row| Ok((row.get(0)?, row.get(1)?)),
    )
    .optional()?
    .ok_or(StoreError::NotFound("claim"))
}

fn set_claim_state(conn: &Connection, id: &str, state: ClaimState) -> Result<()> {
    conn.execute(
        "UPDATE claim SET state = ? WHERE id = ?",
        params![state, id],
    )?;
    Ok(())
}

fn revoke(tx: &Transaction<'_>, campaign_id: &str, source_claim: &str, reason: &str) -> Result<()> {
    let mut query = tx.prepare(
        "WITH RECURSIVE affected(id) AS (
            SELECT ?1 UNION SELECT d.claim_id FROM claim_dependency d JOIN affected a ON d.prerequisite_id = a.id
         )
         SELECT c.id FROM claim c JOIN affected a ON c.id = a.id
         WHERE c.campaign_id = ?2 AND c.state = 'verified'
         ORDER BY (c.id != ?1), c.id",
    )?;
    let ids: Vec<String> = query
        .query_map(params![source_claim, campaign_id], |row| row.get(0))?
        .collect::<rusqlite::Result<_>>()?;
    for id in ids {
        set_claim_state(tx, &id, ClaimState::Revoked)?;
        append(
            tx,
            campaign_id,
            "claim.revoked",
            json!({"claim_id": id, "source_claim_id": source_claim, "reason": reason}),
        )?;
    }
    Ok(())
}

fn check_attempt_blobs(conn: &Connection, attempt: &str) -> Result<()> {
    let lease = attempt_lease(conn, attempt)?;
    let manifest = lease
        .manifest_sha256
        .ok_or(StoreError::Integrity("attempt has no immutable manifest"))?;
    if read_blob(conn, &manifest)?.is_none() {
        return Err(StoreError::Integrity("attempt manifest is missing"));
    }
    let mut query = conn.prepare(
        "SELECT DISTINCT sha256 FROM attempt_artifact WHERE attempt_id = ? ORDER BY sha256",
    )?;
    let hashes = query.query_map([attempt], |row| row.get::<_, String>(0))?;
    for hash in hashes {
        if read_blob(conn, &hash?)?.is_none() {
            return Err(StoreError::Integrity("attempt artifact is missing"));
        }
    }
    Ok(())
}
