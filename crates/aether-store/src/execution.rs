use crate::*;

const MAX_MANIFEST_BYTES: usize = 256 * 1024;
const MAX_ATTEMPT_ARTIFACT_BYTES: usize = 8 * 1024 * 1024;
const MAX_ATTEMPT_ARTIFACTS: usize = 32;
const MAX_ATTEMPT_METRICS: usize = 64;

pub(crate) struct RunContext {
    pub campaign_id: String,
    pub experiment_id: String,
    pub parent_run_id: Option<String>,
    pub action: Action,
    pub seed: u64,
    pub input_hash: String,
}

pub(crate) struct Lease {
    pub run_id: String,
    token: String,
    expires_at_ms: u64,
    estimate: BudgetUnits,
    pub outcome: Option<AttemptOutcome>,
    pub manifest_sha256: Option<String>,
    completion_sha256: Option<String>,
}

impl Store {
    pub fn add_run(&mut self, run: &NewRun) -> Result<()> {
        validate_id(&run.id)?;
        validate_id(&run.experiment_id)?;
        validate_hash(&run.input_hash)?;
        let seed = sqlite_integer(run.seed)?;
        self.write(|tx| {
            let campaign_id: String = tx
                .query_row(
                    "SELECT campaign_id FROM experiment WHERE id = ?",
                    [&run.experiment_id],
                    |row| row.get(0),
                )
                .optional()?
                .ok_or(StoreError::NotFound("experiment"))?;
            ensure_campaign_open(tx, &campaign_id)?;
            if let Some(parent_id) = run.parent_run_id.as_deref() {
                validate_id(parent_id)?;
                let parent = run_context(tx, parent_id)?;
                require_campaign(&campaign_id, &parent.campaign_id)?;
                if run.action == Action::Replay
                    && (run.experiment_id != parent.experiment_id
                        || run.seed != parent.seed
                        || run.input_hash != parent.input_hash)
                {
                    return Err(StoreError::Conflict(
                        "replay must retain the parent input, experiment, and seed",
                    ));
                }
            } else if matches!(run.action, Action::Replay | Action::Challenge) {
                return Err(StoreError::Invalid(
                    "replay and challenge require a parent run",
                ));
            }
            tx.execute(
                "INSERT INTO run VALUES (?, ?, ?, ?, ?, ?)",
                params![
                    run.id,
                    run.experiment_id,
                    run.parent_run_id,
                    run.action,
                    seed,
                    run.input_hash
                ],
            )?;
            append(
                tx,
                &campaign_id,
                "run.created",
                json!({"id": run.id, "action": run.action}),
            )
        })
    }

    pub fn start_attempt(&mut self, attempt: &NewAttempt) -> Result<()> {
        self.start_attempt_with_reserve(attempt, BudgetUnits::default())
    }

    /// Atomically admit a trusted estimate while leaving the caller's protected budget unused.
    pub fn start_attempt_with_reserve(
        &mut self,
        attempt: &NewAttempt,
        protected: BudgetUnits,
    ) -> Result<()> {
        validate_id(&attempt.id)?;
        validate_id(&attempt.run_id)?;
        validate_id(&attempt.lease_token)?;
        let expiry = sqlite_integer(attempt.lease_expires_at_ms)?;
        if attempt.estimate.attempts != 1 {
            return Err(StoreError::Invalid("attempt reservation or lease expiry"));
        }
        self.write(|tx| {
            if attempt.lease_expires_at_ms <= now_ms()? {
                return Err(StoreError::Invalid("lease expiry must be in the future"));
            }
            let run = run_context(tx, &attempt.run_id)?;
            if campaign_state(tx, &run.campaign_id)? != CampaignState::Running {
                return Err(StoreError::Conflict("attempt requires a running campaign"));
            }
            let run_closed: bool = tx.query_row(
                "SELECT EXISTS(SELECT 1 FROM attempt WHERE run_id = ? AND outcome IN ('succeeded', 'cancelled'))",
                [&attempt.run_id], |row| row.get(0),
            )?;
            if run_closed {
                return Err(StoreError::Conflict("run already has a final attempt"));
            }
            let mut budget = Self::read_budget(tx, &run.campaign_id)?;
            let reserved = budget
                .reserved
                .checked_add(attempt.estimate)
                .ok_or(StoreError::BudgetExceeded)?;
            let admission = budget
                .used
                .checked_add(reserved)
                .and_then(|committed| committed.checked_add(protected))
                .ok_or(StoreError::BudgetExceeded)?;
            if budget.halted || !admission.fits(budget.caps) {
                return Err(StoreError::BudgetExceeded);
            }
            budget.reserved = reserved;
            tx.execute(
                "INSERT INTO attempt (id, run_id, lease_token, lease_expires_at_ms, estimate)
                 VALUES (?, ?, ?, ?, ?)",
                params![attempt.id, attempt.run_id, attempt.lease_token, expiry, object_json(attempt.estimate)?],
            )?;
            save_budget(tx, &run.campaign_id, budget)?;
            append(tx, &run.campaign_id, "budget.reserved", json!({"attempt_id": attempt.id, "estimate": attempt.estimate}))?;
            append(tx, &run.campaign_id, "run.started", json!({"attempt_id": attempt.id, "run_id": attempt.run_id}))
        })
    }

    pub fn complete_attempt(&mut self, completion: &AttemptCompletion) -> Result<()> {
        validate_id(&completion.id)?;
        validate_id(&completion.lease_token)?;
        if completion.usage.attempts != 1 || completion.outcome == AttemptOutcome::Expired {
            return Err(StoreError::Invalid("completion usage or outcome"));
        }
        if completion.manifest.len() > MAX_MANIFEST_BYTES
            || !serde_json::from_slice::<Value>(&completion.manifest)?.is_object()
        {
            return Err(StoreError::Invalid(
                "manifest must be a bounded JSON object",
            ));
        }
        if let Some(reason) = completion.reason.as_deref() {
            validate_text(reason)?;
        } else if completion.outcome != AttemptOutcome::Succeeded {
            return Err(StoreError::Invalid(
                "failed or cancelled attempt requires a reason",
            ));
        }
        if completion.metrics.len() > MAX_ATTEMPT_METRICS {
            return Err(StoreError::Invalid("attempt has too many metrics"));
        }
        if completion.outcome != AttemptOutcome::Succeeded && !completion.metrics.is_empty() {
            return Err(StoreError::Invalid(
                "only successful attempts carry authoritative metrics",
            ));
        }
        for metric in &completion.metrics {
            validate_id(&metric.id)?;
            validate_id(&metric.name)?;
            validate_text(&metric.unit)?;
            validate_hash(&metric.evaluator_hash)?;
            validate_hash(&metric.split_hash)?;
            if !metric.value.is_finite() {
                return Err(StoreError::Invalid("metric must be finite"));
            }
        }
        let artifacts = artifact_metadata(completion)?;
        let manifest_hash = artifact_sha256(&completion.manifest);
        let completion_hash = artifact_sha256(&serde_json::to_vec(&(
            completion.outcome,
            &manifest_hash,
            completion.usage,
            &completion.metrics,
            &artifacts,
            &completion.reason,
        ))?);
        self.write(|tx| {
            let lease = attempt_lease(tx, &completion.id)?;
            if lease.token != completion.lease_token {
                return Err(StoreError::Conflict("attempt lease does not match"));
            }
            if let Some(previous) = lease.completion_sha256.as_deref() {
                if previous == completion_hash {
                    return Ok(());
                }
                return Err(StoreError::Conflict("attempt already settled differently"));
            }
            if now_ms()? >= lease.expires_at_ms {
                return Err(StoreError::Conflict("attempt lease expired"));
            }
            let run = run_context(tx, &lease.run_id)?;
            let evaluator_hash: String = tx.query_row(
                "SELECT evaluator_hash FROM campaign WHERE id = ?", [&run.campaign_id],
                |row| row.get(0),
            )?;
            for metric in &completion.metrics {
                if metric.evaluator_hash != evaluator_hash {
                    return Err(StoreError::Conflict("metric evaluator does not match the frozen campaign"));
                }
                tx.execute(
                    "INSERT INTO metric VALUES (?, ?, ?, ?, ?, ?, ?)",
                    params![metric.id, completion.id, metric.name, metric.unit, metric.evaluator_hash, metric.split_hash, metric.value],
                )?;
            }
            insert_blob(tx, &manifest_hash, &completion.manifest)?;
            for (meta, input) in artifacts.iter().zip(&completion.artifacts) {
                insert_blob(tx, &meta.sha256, &input.bytes)?;
                tx.execute(
                    "INSERT INTO attempt_artifact VALUES (?, ?, ?, ?)",
                    params![completion.id, meta.name, meta.media_type, meta.sha256],
                )?;
            }
            tx.execute(
                "UPDATE attempt SET outcome = ?, usage = ?, manifest_sha256 = ?, reason = ?, completion_sha256 = ?
                 WHERE id = ?",
                params![completion.outcome, object_json(completion.usage)?, manifest_hash,
                    completion.reason, completion_hash, completion.id],
            )?;
            settle(tx, &run.campaign_id, &completion.id, lease.estimate, completion.usage)?;
            append(
                tx, &run.campaign_id, "run.completed",
                json!({"attempt_id": completion.id, "outcome": completion.outcome, "manifest_sha256": manifest_hash, "reason": completion.reason}),
            )
        })
    }

    /// Reconcile expired metadata in a bounded batch after the controller has stopped its workers.
    pub fn recover_expired(&mut self) -> Result<usize> {
        self.recover_expired_matching(None)
    }

    pub fn recover_expired_in(&mut self, campaign_id: &str) -> Result<usize> {
        validate_id(campaign_id)?;
        self.recover_expired_matching(Some(campaign_id))
    }

    fn recover_expired_matching(&mut self, campaign_id: Option<&str>) -> Result<usize> {
        let mut recovered = 0;
        self.write(|tx| {
            let recovered_at = now_ms()?;
            let ids: Vec<String> = {
                let mut query = tx.prepare(
                    "SELECT a.id FROM attempt a JOIN run r ON a.run_id = r.id
                       JOIN experiment e ON r.experiment_id = e.id
                     WHERE a.outcome IS NULL AND a.lease_expires_at_ms <= ?1
                       AND (?2 IS NULL OR e.campaign_id = ?2)
                     ORDER BY a.lease_expires_at_ms, a.id LIMIT 512",
                )?;
                query.query_map(params![sqlite_integer(recovered_at)?, campaign_id], |row| row.get(0))?
                    .collect::<rusqlite::Result<_>>()?
            };
            for id in ids {
                let lease = attempt_lease(tx, &id)?;
                let run = run_context(tx, &lease.run_id)?;
                let manifest = serde_json::to_vec(&json!({"attempt_id": id, "outcome": "expired", "observed_at_ms": recovered_at}))?;
                let digest = artifact_sha256(&manifest);
                insert_blob(tx, &digest, &manifest)?;
                tx.execute(
                    "UPDATE attempt SET outcome = 'expired', usage = estimate, manifest_sha256 = ?1,
                     reason = 'lease expired; estimate charged', completion_sha256 = ?1 WHERE id = ?2",
                    params![digest, id],
                )?;
                settle(tx, &run.campaign_id, &id, lease.estimate, lease.estimate)?;
                append(tx, &run.campaign_id, "run.expired", json!({"attempt_id": id}))?;
                recovered += 1;
            }
            Ok(())
        })?;
        Ok(recovered)
    }

    pub fn budget(&self, campaign_id: &str) -> Result<Budget> {
        validate_id(campaign_id)?;
        Self::read_budget(&self.conn, campaign_id)
    }

    pub(crate) fn read_budget(conn: &Connection, campaign_id: &str) -> Result<Budget> {
        conn.query_row(
            "SELECT caps, used, reserved, halted FROM budget WHERE campaign_id = ?",
            [campaign_id],
            |row| {
                Ok(Budget {
                    caps: row_json(row, 0)?,
                    used: row_json(row, 1)?,
                    reserved: row_json(row, 2)?,
                    halted: row.get(3)?,
                })
            },
        )
        .optional()?
        .ok_or(StoreError::NotFound("budget"))
    }

    pub fn artifact(&self, sha256: &str) -> Result<Option<Vec<u8>>> {
        validate_hash(sha256)?;
        read_blob(&self.conn, sha256)
    }

    /// Store a local discovery report without attaching it to a campaign claim.
    pub fn record_discovery_artifact(&mut self, bytes: &[u8]) -> Result<String> {
        if bytes.is_empty() || bytes.len() > MAX_ATTEMPT_ARTIFACT_BYTES {
            return Err(StoreError::Invalid(
                "discovery artifact size outside limits",
            ));
        }
        let digest = artifact_sha256(bytes);
        self.write(|tx| insert_blob(tx, &digest, bytes))?;
        Ok(digest)
    }
}

fn artifact_metadata(completion: &AttemptCompletion) -> Result<Vec<Artifact>> {
    if completion.artifacts.len() > MAX_ATTEMPT_ARTIFACTS {
        return Err(StoreError::Invalid("attempt has too many artifacts"));
    }
    let mut total_bytes = 0_usize;
    let mut artifacts = Vec::with_capacity(completion.artifacts.len());
    for artifact in &completion.artifacts {
        validate_id(&artifact.name)?;
        if artifact.media_type.len() > 127
            || !artifact.media_type.contains('/')
            || !artifact
                .media_type
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b"-+./".contains(&b))
        {
            return Err(StoreError::Invalid("artifact media type"));
        }
        total_bytes = total_bytes
            .checked_add(artifact.bytes.len())
            .ok_or(StoreError::Invalid("attempt artifact size exceeds limit"))?;
        if total_bytes > MAX_ATTEMPT_ARTIFACT_BYTES {
            return Err(StoreError::Invalid("attempt artifact size exceeds limit"));
        }
        artifacts.push(Artifact {
            attempt_id: completion.id.clone(),
            name: artifact.name.clone(),
            media_type: artifact.media_type.clone(),
            sha256: artifact_sha256(&artifact.bytes),
            size_bytes: artifact.bytes.len() as u64,
        });
    }
    Ok(artifacts)
}

pub(crate) fn run_context(conn: &Connection, id: &str) -> Result<RunContext> {
    conn.query_row(
        "SELECT e.campaign_id, r.experiment_id, r.parent_run_id, r.action, r.seed, r.input_hash
         FROM run r JOIN experiment e ON e.id = r.experiment_id WHERE r.id = ?",
        [id],
        |row| {
            Ok(RunContext {
                campaign_id: row.get(0)?,
                experiment_id: row.get(1)?,
                parent_run_id: row.get(2)?,
                action: row.get(3)?,
                seed: row_u64(row, 4)?,
                input_hash: row.get(5)?,
            })
        },
    )
    .optional()?
    .ok_or(StoreError::NotFound("run"))
}

pub(crate) fn attempt_lease(conn: &Connection, id: &str) -> Result<Lease> {
    conn.query_row(
        "SELECT run_id, lease_token, lease_expires_at_ms, estimate, outcome, manifest_sha256, completion_sha256
         FROM attempt WHERE id = ?",
        [id],
        |row| Ok(Lease {
            run_id: row.get(0)?, token: row.get(1)?, expires_at_ms: row_u64(row, 2)?, estimate: row_json(row, 3)?,
            outcome: row.get(4)?, manifest_sha256: row.get(5)?, completion_sha256: row.get(6)?,
        }),
    )
    .optional()?
    .ok_or(StoreError::NotFound("attempt"))
}

fn insert_blob(tx: &Transaction<'_>, hash: &str, bytes: &[u8]) -> Result<()> {
    let inserted = tx.execute(
        "INSERT INTO blob VALUES (?, ?) ON CONFLICT(sha256) DO NOTHING",
        params![hash, bytes],
    )?;
    if inserted == 0 {
        let same_bytes: bool = tx.query_row(
            "SELECT content = ? FROM blob WHERE sha256 = ?",
            params![bytes, hash],
            |row| row.get(0),
        )?;
        if !same_bytes {
            return Err(StoreError::Integrity(
                "stored bytes disagree with their content digest",
            ));
        }
    }
    Ok(())
}

pub(crate) fn read_blob(conn: &Connection, sha256: &str) -> Result<Option<Vec<u8>>> {
    let content: Option<Option<Vec<u8>>> = conn
        .query_row(
            "SELECT CASE WHEN length(content) <= ?2 THEN content END FROM blob WHERE sha256 = ?1",
            params![sha256, MAX_ATTEMPT_ARTIFACT_BYTES as i64],
            |row| row.get(0),
        )
        .optional()?;
    match content {
        None => Ok(None),
        Some(None) => Err(StoreError::Integrity(
            "stored artifact exceeds its byte limit",
        )),
        Some(Some(bytes)) if artifact_sha256(&bytes) != sha256 => {
            Err(StoreError::Integrity("artifact SHA-256 mismatch"))
        }
        Some(Some(bytes)) => Ok(Some(bytes)),
    }
}

fn save_budget(tx: &Transaction<'_>, campaign_id: &str, budget: Budget) -> Result<()> {
    tx.execute(
        "UPDATE budget SET used = ?, reserved = ?, halted = ? WHERE campaign_id = ?",
        params![
            object_json(budget.used)?,
            object_json(budget.reserved)?,
            budget.halted,
            campaign_id
        ],
    )?;
    Ok(())
}

fn settle(
    tx: &Transaction<'_>,
    campaign_id: &str,
    attempt_id: &str,
    estimate: BudgetUnits,
    usage: BudgetUnits,
) -> Result<()> {
    let mut budget = Store::read_budget(tx, campaign_id)?;
    budget.reserved = budget
        .reserved
        .checked_sub(estimate)
        .ok_or(StoreError::Integrity(
            "live attempt has no matching budget reservation",
        ))?;
    budget.used = budget.used.checked_add(usage).ok_or(StoreError::Invalid(
        "usage accumulation exceeds integer range",
    ))?;
    if !budget
        .used
        .checked_add(budget.reserved)
        .is_some_and(|total| total.fits(budget.caps))
    {
        budget.halted = true;
        append(
            tx,
            campaign_id,
            "budget.overrun",
            json!({"attempt_id": attempt_id, "usage": usage}),
        )?;
    }
    save_budget(tx, campaign_id, budget)?;
    append(
        tx,
        campaign_id,
        "budget.settled",
        json!({"attempt_id": attempt_id, "usage": usage}),
    )
}
