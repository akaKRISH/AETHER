use crate::*;

const MAX_SNAPSHOT_ROWS_PER_KIND: usize = 4096;

const CAMPAIGN_COLUMNS: &str =
    "SELECT c.id, c.title, c.objective, c.protocol_hash, c.policy_hash, c.evaluator_hash,
        c.requires_replay, b.caps, c.state
     FROM campaign c JOIN budget b ON b.campaign_id = c.id";

const ATTEMPT_COLUMNS: &str =
    "SELECT a.id, a.run_id, a.lease_expires_at_ms, a.estimate, a.outcome, a.usage, a.manifest_sha256, a.reason
     FROM attempt a";

impl Store {
    pub fn campaign(&self, id: &str) -> Result<Option<Campaign>> {
        validate_id(id)?;
        let sql = format!("{CAMPAIGN_COLUMNS} WHERE c.id = ?");
        Ok(self.conn.query_row(&sql, [id], map_campaign).optional()?)
    }

    pub fn campaigns(&self) -> Result<Vec<Campaign>> {
        self.campaigns_page(None, MAX_PAGE_SIZE)
    }

    pub fn campaigns_page(&self, after: Option<&str>, limit: usize) -> Result<Vec<Campaign>> {
        if let Some(id) = after {
            validate_id(id)?;
        }
        let limit = validate_page_size(limit)?;
        let sql = format!("{CAMPAIGN_COLUMNS} WHERE c.id > ? ORDER BY c.id LIMIT ?");
        let mut query = self.conn.prepare(&sql)?;
        Ok(query
            .query_map(params![after.unwrap_or(""), limit], map_campaign)?
            .collect::<rusqlite::Result<_>>()?)
    }

    pub fn run(&self, id: &str) -> Result<Option<NewRun>> {
        validate_id(id)?;
        Ok(self.conn.query_row(
            "SELECT id, experiment_id, parent_run_id, action, seed, input_hash FROM run WHERE id = ?",
            [id], map_run,
        ).optional()?)
    }

    pub fn attempt(&self, id: &str) -> Result<Option<Attempt>> {
        validate_id(id)?;
        let sql = format!("{ATTEMPT_COLUMNS} WHERE a.id = ?");
        Ok(self.conn.query_row(&sql, [id], map_attempt).optional()?)
    }

    pub fn claim(&self, id: &str) -> Result<Option<Claim>> {
        validate_id(id)?;
        Ok(self
            .conn
            .query_row(
                "SELECT id, campaign_id, statement, state FROM claim WHERE id = ?",
                [id],
                map_claim,
            )
            .optional()?)
    }

    pub fn events(&self, campaign_id: &str) -> Result<Vec<Event>> {
        self.events_page(campaign_id, 0, MAX_PAGE_SIZE)
    }

    pub fn events_page(&self, campaign_id: &str, after: u64, limit: usize) -> Result<Vec<Event>> {
        validate_id(campaign_id)?;
        let limit = validate_page_size(limit)?;
        let after = sqlite_integer(after)?;
        campaign_state(&self.conn, campaign_id)?;
        let mut query = self.conn.prepare(
            "SELECT campaign_id, sequence, kind, occurred_at_ms, detail FROM event
             WHERE campaign_id = ? AND sequence > ? ORDER BY sequence LIMIT ?",
        )?;
        Ok(query
            .query_map(params![campaign_id, after, limit], |row| {
                Ok(Event {
                    campaign_id: row.get(0)?,
                    sequence: row_u64(row, 1)?,
                    kind: row.get(2)?,
                    occurred_at_ms: row_u64(row, 3)?,
                    detail: row_json(row, 4)?,
                })
            })?
            .collect::<rusqlite::Result<_>>()?)
    }

    /// Read a consistent bounded view of one campaign. Audit events are paged separately.
    pub fn snapshot(&self, campaign_id: &str) -> Result<Snapshot> {
        validate_id(campaign_id)?;
        let tx = self.conn.unchecked_transaction()?;
        let campaign_sql = format!("{CAMPAIGN_COLUMNS} WHERE c.id = ?");
        let campaign = tx
            .query_row(&campaign_sql, [campaign_id], map_campaign)
            .optional()?
            .ok_or(StoreError::NotFound("campaign"))?;
        let budget = Self::read_budget(&tx, campaign_id)?;
        let hypotheses = bounded(
            &tx,
            campaign_id,
            "SELECT id, campaign_id, statement FROM hypothesis WHERE campaign_id = ? ORDER BY id LIMIT ?",
            |row| {
                Ok(NewHypothesis {
                    id: row.get(0)?,
                    campaign_id: row.get(1)?,
                    statement: row.get(2)?,
                })
            },
        )?;
        let experiments = bounded(
            &tx,
            campaign_id,
            "SELECT id, campaign_id, hypothesis_id, task_key, definition_hash
             FROM experiment WHERE campaign_id = ? ORDER BY id LIMIT ?",
            |row| {
                Ok(NewExperiment {
                    id: row.get(0)?,
                    campaign_id: row.get(1)?,
                    hypothesis_id: row.get(2)?,
                    task_key: row.get(3)?,
                    definition_hash: row.get(4)?,
                })
            },
        )?;
        let runs = bounded(
            &tx,
            campaign_id,
            "SELECT r.id, r.experiment_id, r.parent_run_id, r.action, r.seed, r.input_hash
             FROM run r JOIN experiment e ON e.id = r.experiment_id
             WHERE e.campaign_id = ? ORDER BY r.id LIMIT ?",
            map_run,
        )?;
        let attempt_sql = format!("{ATTEMPT_COLUMNS} JOIN run r ON r.id = a.run_id
             JOIN experiment e ON e.id = r.experiment_id WHERE e.campaign_id = ? ORDER BY a.id LIMIT ?");
        let attempts = bounded(&tx, campaign_id, &attempt_sql, map_attempt)?;
        let metrics = bounded(&tx, campaign_id,
            "SELECT m.attempt_id, m.id, m.name, m.unit, m.evaluator_hash, m.split_hash, m.value
             FROM metric m JOIN attempt a ON a.id = m.attempt_id JOIN run r ON r.id = a.run_id
             JOIN experiment e ON e.id = r.experiment_id WHERE e.campaign_id = ? ORDER BY m.id LIMIT ?",
            |row| Ok(Metric {
                attempt_id: row.get(0)?, data: MetricInput {
                    id: row.get(1)?, name: row.get(2)?, unit: row.get(3)?,
                    evaluator_hash: row.get(4)?, split_hash: row.get(5)?, value: row.get(6)?,
                },
            }),
        )?;
        let artifacts = bounded(
            &tx,
            campaign_id,
            "SELECT aa.attempt_id, aa.name, aa.media_type, aa.sha256, length(b.content)
             FROM attempt_artifact aa JOIN blob b ON b.sha256 = aa.sha256
             JOIN attempt a ON a.id = aa.attempt_id JOIN run r ON r.id = a.run_id
             JOIN experiment e ON e.id = r.experiment_id WHERE e.campaign_id = ?
             ORDER BY aa.attempt_id, aa.name LIMIT ?",
            |row| {
                Ok(Artifact {
                    attempt_id: row.get(0)?,
                    name: row.get(1)?,
                    media_type: row.get(2)?,
                    sha256: row.get(3)?,
                    size_bytes: row_u64(row, 4)?,
                })
            },
        )?;
        let claims = bounded(
            &tx,
            campaign_id,
            "SELECT id, campaign_id, statement, state FROM claim WHERE campaign_id = ? ORDER BY id LIMIT ?",
            map_claim,
        )?;
        let evidence = bounded(
            &tx,
            campaign_id,
            "SELECT e.id, e.claim_id, e.metric_id, e.artifact_attempt_id, e.artifact_sha256,
                    e.relation, e.invalidated
             FROM evidence e JOIN claim c ON c.id = e.claim_id
             WHERE c.campaign_id = ? ORDER BY e.id LIMIT ?",
            |row| {
                let target = if let Some(metric_id) = row.get(2)? {
                    EvidenceTarget::Metric { metric_id }
                } else {
                    EvidenceTarget::Artifact {
                        attempt_id: row.get(3)?,
                        sha256: row.get(4)?,
                    }
                };
                Ok(Evidence {
                    data: NewEvidence {
                        id: row.get(0)?,
                        claim_id: row.get(1)?,
                        target,
                        relation: row.get(5)?,
                    },
                    invalidated: row.get(6)?,
                })
            },
        )?;
        let dependencies = bounded(&tx, campaign_id,
            "SELECT d.claim_id, d.prerequisite_id FROM claim_dependency d JOIN claim c ON c.id = d.claim_id
             WHERE c.campaign_id = ? ORDER BY d.claim_id, d.prerequisite_id LIMIT ?",
            |row| Ok(Dependency { claim_id: row.get(0)?, prerequisite_id: row.get(1)? }),
        )?;
        let verifications = bounded(&tx, campaign_id,
            "SELECT v.id, v.claim_id, v.verdict, v.rule_hash, v.evaluator_hash, v.replay_attempt_id, v.reason
             FROM verification v JOIN claim c ON c.id = v.claim_id
             WHERE c.campaign_id = ? ORDER BY v.id LIMIT ?",
            |row| Ok(NewVerification {
                id: row.get(0)?, claim_id: row.get(1)?, verdict: row.get(2)?, rule_hash: row.get(3)?,
                evaluator_hash: row.get(4)?, replay_attempt_id: row.get(5)?, reason: row.get(6)?,
            }),
        )?;
        let contradictions = bounded(
            &tx,
            campaign_id,
            "SELECT x.id, x.claim_id, x.evidence_id, x.reason, x.invalidates
             FROM contradiction x JOIN claim c ON c.id = x.claim_id
             WHERE c.campaign_id = ? ORDER BY x.id LIMIT ?",
            |row| {
                Ok(NewContradiction {
                    id: row.get(0)?,
                    claim_id: row.get(1)?,
                    evidence_id: row.get(2)?,
                    reason: row.get(3)?,
                    invalidates: row.get(4)?,
                })
            },
        )?;
        let replications = bounded(&tx, campaign_id,
            "SELECT rp.id, rp.original_attempt_id, rp.replay_attempt_id, rp.passed, rp.reason
             FROM replication rp JOIN attempt a ON a.id = rp.original_attempt_id JOIN run r ON r.id = a.run_id
             JOIN experiment e ON e.id = r.experiment_id
             WHERE e.campaign_id = ? ORDER BY rp.id LIMIT ?",
            |row| Ok(NewReplication {
                id: row.get(0)?, original_attempt_id: row.get(1)?, replay_attempt_id: row.get(2)?,
                passed: row.get(3)?, reason: row.get(4)?,
            }),
        )?;
        tx.commit()?;
        Ok(Snapshot {
            campaign,
            budget,
            hypotheses,
            experiments,
            runs,
            attempts,
            metrics,
            artifacts,
            claims,
            evidence,
            dependencies,
            verifications,
            contradictions,
            replications,
        })
    }
}

fn bounded<T>(
    conn: &Connection,
    campaign_id: &str,
    sql: &str,
    map: impl FnMut(&Row<'_>) -> rusqlite::Result<T>,
) -> Result<Vec<T>> {
    let mut query = conn.prepare(sql)?;
    let rows = query
        .query_map(
            params![campaign_id, (MAX_SNAPSHOT_ROWS_PER_KIND + 1) as i64],
            map,
        )?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    if rows.len() > MAX_SNAPSHOT_ROWS_PER_KIND {
        return Err(StoreError::Conflict("snapshot exceeds local report bounds"));
    }
    Ok(rows)
}

fn validate_page_size(limit: usize) -> Result<i64> {
    if !(1..=MAX_PAGE_SIZE).contains(&limit) {
        return Err(StoreError::Invalid("page limit out of range"));
    }
    Ok(limit as i64)
}

fn map_campaign(row: &Row<'_>) -> rusqlite::Result<Campaign> {
    Ok(Campaign {
        spec: NewCampaign {
            id: row.get(0)?,
            title: row.get(1)?,
            objective: row.get(2)?,
            protocol_hash: row.get(3)?,
            policy_hash: row.get(4)?,
            evaluator_hash: row.get(5)?,
            requires_replay: row.get(6)?,
            caps: row_json(row, 7)?,
        },
        state: row.get(8)?,
    })
}

fn map_run(row: &Row<'_>) -> rusqlite::Result<NewRun> {
    Ok(NewRun {
        id: row.get(0)?,
        experiment_id: row.get(1)?,
        parent_run_id: row.get(2)?,
        action: row.get(3)?,
        seed: row_u64(row, 4)?,
        input_hash: row.get(5)?,
    })
}

fn map_attempt(row: &Row<'_>) -> rusqlite::Result<Attempt> {
    let usage = if row.get_ref(5)?.data_type() == Type::Null {
        None
    } else {
        Some(row_json(row, 5)?)
    };
    Ok(Attempt {
        id: row.get(0)?,
        run_id: row.get(1)?,
        lease_expires_at_ms: row_u64(row, 2)?,
        estimate: row_json(row, 3)?,
        outcome: row.get(4)?,
        usage,
        manifest_sha256: row.get(6)?,
        reason: row.get(7)?,
    })
}

fn map_claim(row: &Row<'_>) -> rusqlite::Result<Claim> {
    Ok(Claim {
        spec: NewClaim {
            id: row.get(0)?,
            campaign_id: row.get(1)?,
            statement: row.get(2)?,
        },
        state: row.get(3)?,
    })
}
