use super::*;

const CAMPAIGN: &str = "campaign-1";
const EXPERIMENT: &str = "experiment-1";

fn units(wall_ms: u64, attempts: u64) -> BudgetUnits {
    BudgetUnits {
        wall_ms,
        attempts,
        ..BudgetUnits::default()
    }
}

fn spec(id: &str) -> NewCampaign {
    NewCampaign {
        id: id.into(),
        title: "Classification fixture".into(),
        objective: "Observe repeatability".into(),
        protocol_hash: artifact_sha256(b"frozen protocol"),
        policy_hash: artifact_sha256(b"local policy"),
        evaluator_hash: artifact_sha256(b"trusted evaluator"),
        requires_replay: true,
        caps: units(5_000, 4),
    }
}

fn store_running() -> Store {
    let mut store = Store::open_in_memory().unwrap();
    let campaign = spec(CAMPAIGN);
    store.create_campaign(&campaign).unwrap();
    assert!(
        store
            .approve_campaign(CAMPAIGN, &artifact_sha256(b"different protocol"))
            .is_err()
    );
    store
        .approve_campaign(CAMPAIGN, &campaign.protocol_hash)
        .unwrap();
    store
        .transition_campaign(CAMPAIGN, CampaignState::Running)
        .unwrap();
    assert!(
        store
            .transition_campaign(CAMPAIGN, CampaignState::Running)
            .is_err()
    );
    store
        .add_hypothesis(&NewHypothesis {
            id: "hypothesis-1".into(),
            campaign_id: CAMPAIGN.into(),
            statement: "Fixture is repeatable".into(),
        })
        .unwrap();
    store
        .add_experiment(&NewExperiment {
            id: EXPERIMENT.into(),
            campaign_id: CAMPAIGN.into(),
            hypothesis_id: Some("hypothesis-1".into()),
            task_key: "fixed-classification".into(),
            definition_hash: artifact_sha256(b"fixture definition"),
        })
        .unwrap();
    store
}

fn run(store: &mut Store, id: &str, parent: Option<&str>) {
    store
        .add_run(&NewRun {
            id: id.into(),
            experiment_id: EXPERIMENT.into(),
            parent_run_id: parent.map(String::from),
            action: if parent.is_some() {
                Action::Replay
            } else {
                Action::Explore
            },
            seed: 42,
            input_hash: artifact_sha256(b"fixed inputs"),
        })
        .unwrap();
}

fn attempt(store: &mut Store, id: &str, run_id: &str) -> AttemptCompletion {
    store
        .start_attempt(&NewAttempt {
            id: id.into(),
            run_id: run_id.into(),
            lease_token: format!("lease-{id}"),
            lease_expires_at_ms: now_ms().unwrap() + 60_000,
            estimate: units(100, 1),
        })
        .unwrap();
    AttemptCompletion {
        id: id.into(),
        lease_token: format!("lease-{id}"),
        outcome: AttemptOutcome::Succeeded,
        manifest: serde_json::to_vec(&json!({"attempt_id": id, "fixture_version": 1})).unwrap(),
        usage: units(30, 1),
        metrics: vec![MetricInput {
            id: format!("metric-{id}"),
            name: "accuracy".into(),
            unit: "ratio".into(),
            evaluator_hash: spec(CAMPAIGN).evaluator_hash,
            split_hash: artifact_sha256(b"fixed labels"),
            value: 0.75,
        }],
        artifacts: vec![ArtifactInput {
            name: "result".into(),
            media_type: "application/json".into(),
            bytes: b"{\"correct\":3}".to_vec(),
        }],
        reason: None,
    }
}

fn verify(claim: &str, replay: Option<&str>) -> NewVerification {
    let campaign = spec(CAMPAIGN);
    NewVerification {
        id: format!("verdict-{claim}"),
        claim_id: claim.into(),
        verdict: VerificationVerdict::Verified,
        rule_hash: campaign.protocol_hash,
        evaluator_hash: campaign.evaluator_hash,
        replay_attempt_id: replay.map(String::from),
        reason: "replayed trusted classification metric".into(),
    }
}

#[test]
fn failed_write_rolls_back_artifacts_metrics_budget_and_audit() {
    let mut store = store_running();
    run(&mut store, "baseline", None);
    let completion = attempt(&mut store, "original", "baseline");
    let before = store.events(CAMPAIGN).unwrap();
    let mut conflicting = completion.clone();
    conflicting.artifacts.push(conflicting.artifacts[0].clone());
    assert!(store.complete_attempt(&conflicting).is_err());
    let failed = store.snapshot(CAMPAIGN).unwrap();
    assert_eq!(failed.budget.reserved, units(100, 1));
    assert_eq!(failed.budget.used, BudgetUnits::default());
    assert!(failed.attempts[0].outcome.is_none());
    assert!(failed.metrics.is_empty());
    assert!(failed.artifacts.is_empty());
    assert!(
        store
            .artifact(&artifact_sha256(&completion.manifest))
            .unwrap()
            .is_none()
    );
    assert_eq!(store.events(CAMPAIGN).unwrap(), before);

    store.complete_attempt(&completion).unwrap();
    let accepted = store.events(CAMPAIGN).unwrap();
    store.complete_attempt(&completion).unwrap();
    assert_eq!(store.events(CAMPAIGN).unwrap(), accepted);
    assert_eq!(store.budget(CAMPAIGN).unwrap().used, units(30, 1));
    assert_eq!(
        store.budget(CAMPAIGN).unwrap().reserved,
        BudgetUnits::default()
    );
    assert_eq!(
        store
            .artifact(&artifact_sha256(&completion.manifest))
            .unwrap(),
        Some(completion.manifest)
    );
    assert_eq!(
        store
            .events_page(CAMPAIGN, accepted[0].sequence, 1)
            .unwrap(),
        accepted[1..2]
    );
}

#[test]
fn verified_claims_need_fresh_replay_and_revoke_transitively() {
    let mut store = store_running();
    run(&mut store, "baseline", None);
    let original = attempt(&mut store, "original", "baseline");
    store.complete_attempt(&original).unwrap();
    for id in ["root", "leaf", "dependent"] {
        store
            .add_claim(&NewClaim {
                id: id.into(),
                campaign_id: CAMPAIGN.into(),
                statement: "3 of 4 correct".into(),
            })
            .unwrap();
        store
            .link_evidence(&NewEvidence {
                id: format!("e-{id}"),
                claim_id: id.into(),
                target: EvidenceTarget::Metric {
                    metric_id: original.metrics[0].id.clone(),
                },
                relation: EvidenceRelation::Supports,
            })
            .unwrap();
    }
    store.add_dependency("leaf", "root").unwrap();
    store.add_dependency("dependent", "leaf").unwrap();
    assert!(store.add_dependency("root", "dependent").is_err());
    assert!(store.record_verification(&verify("root", None)).is_err());
    assert!(
        store
            .campaigns()
            .unwrap()
            .iter()
            .all(|c| c.spec.id == CAMPAIGN)
    );

    run(&mut store, "replay", Some("baseline"));
    let replay = attempt(&mut store, "replayed", "replay");
    store.complete_attempt(&replay).unwrap();
    let replay_verdict = verify("root", Some("replayed"));
    assert!(store.record_verification(&replay_verdict).is_err());
    store
        .record_replication(&NewReplication {
            id: "replication-1".into(),
            original_attempt_id: "original".into(),
            replay_attempt_id: "replayed".into(),
            passed: true,
            reason: "trusted evaluator matched frozen inputs".into(),
        })
        .unwrap();
    assert!(
        store
            .record_verification(&verify("leaf", Some("replayed")))
            .is_err()
    );
    store.record_verification(&replay_verdict).unwrap();
    store
        .record_verification(&verify("leaf", Some("replayed")))
        .unwrap();
    store
        .record_verification(&verify("dependent", Some("replayed")))
        .unwrap();

    store
        .invalidate_evidence(
            "e-root",
            "Protocol audit invalidated the evidence provenance",
        )
        .unwrap();
    assert!(
        store
            .link_evidence(&NewEvidence {
                id: "duplicate-e-root".into(),
                claim_id: "root".into(),
                target: EvidenceTarget::Metric {
                    metric_id: original.metrics[0].id.clone(),
                },
                relation: EvidenceRelation::Supports,
            })
            .is_err()
    );
    let snapshot = store.snapshot(CAMPAIGN).unwrap();
    assert!(
        snapshot
            .claims
            .iter()
            .all(|claim| claim.state == ClaimState::Revoked)
    );
    assert_eq!(snapshot.verifications.len(), 3);
    assert!(
        snapshot
            .verifications
            .iter()
            .all(|record| record.verdict == VerificationVerdict::Verified)
    );
    assert_eq!(
        store
            .events(CAMPAIGN)
            .unwrap()
            .iter()
            .filter(|event| event.kind == "claim.revoked")
            .count(),
        3
    );
    assert!(
        store
            .record_verification(&verify("root", Some("replayed")))
            .is_err()
    );
}

#[test]
fn recovery_charges_the_estimate_and_fences_stale_completions() {
    let mut store = store_running();
    run(&mut store, "baseline", None);
    let completion = attempt(&mut store, "original", "baseline");
    // Move the live lease deadline to a known past instant without waiting on the system clock.
    store
        .conn
        .execute(
            "UPDATE attempt SET lease_expires_at_ms = 1 WHERE id = 'original'",
            [],
        )
        .unwrap();
    assert!(store.complete_attempt(&completion).is_err());
    assert_eq!(store.recover_expired().unwrap(), 1);
    assert_eq!(store.recover_expired().unwrap(), 0);
    assert_eq!(store.budget(CAMPAIGN).unwrap().used, units(100, 1));
    assert_eq!(
        store.attempt("original").unwrap().unwrap().outcome,
        Some(AttemptOutcome::Expired)
    );
    assert!(store.complete_attempt(&completion).is_err());
    assert!(
        store
            .conn
            .execute("UPDATE event SET kind = 'edited'", [])
            .is_err()
    );
}

#[test]
fn verification_rejects_cross_campaign_references() {
    let mut store = store_running();
    run(&mut store, "baseline", None);
    let completion = attempt(&mut store, "original", "baseline");
    store.complete_attempt(&completion).unwrap();
    store.create_campaign(&spec("elsewhere")).unwrap();
    store
        .add_claim(&NewClaim {
            id: "other-claim".into(),
            campaign_id: "elsewhere".into(),
            statement: "Separate evidence only".into(),
        })
        .unwrap();
    assert!(
        store
            .link_evidence(&NewEvidence {
                id: "cross".into(),
                claim_id: "other-claim".into(),
                target: EvidenceTarget::Metric {
                    metric_id: completion.metrics[0].id.clone()
                },
                relation: EvidenceRelation::Supports,
            })
            .is_err()
    );
    assert!(store.snapshot("elsewhere").unwrap().evidence.is_empty());
}

#[test]
fn protected_reservation_and_overrun_are_enforced() {
    let mut store = store_running();
    run(&mut store, "baseline", None);
    run(&mut store, "next", None);
    let next = NewAttempt {
        id: "second".into(),
        run_id: "next".into(),
        lease_token: "lease-second".into(),
        lease_expires_at_ms: now_ms().unwrap() + 60_000,
        estimate: units(100, 1),
    };
    assert!(matches!(
        store.start_attempt_with_reserve(&next, units(4_901, 0)),
        Err(StoreError::BudgetExceeded)
    ));
    let mut completion = attempt(&mut store, "original", "baseline");
    completion.usage = units(5_001, 1);
    store.complete_attempt(&completion).unwrap();
    assert!(store.budget(CAMPAIGN).unwrap().halted);
    assert_eq!(store.budget(CAMPAIGN).unwrap().used, units(5_001, 1));
    assert!(matches!(
        store.start_attempt(&next),
        Err(StoreError::BudgetExceeded)
    ));
}

#[test]
fn file_reopen_preserves_schema_state_and_audit() {
    let path = std::env::temp_dir().join(format!(
        "aether-store-{}-{}.sqlite",
        std::process::id(),
        now_ms().unwrap()
    ));
    {
        let mut store = Store::open(&path).unwrap();
        store.create_campaign(&spec(CAMPAIGN)).unwrap();
    }
    {
        let reopened = Store::open(&path).unwrap();
        let journal_mode: String = reopened
            .conn
            .pragma_query_value(None, "journal_mode", |row| row.get(0))
            .unwrap();
        assert_eq!(journal_mode, "delete");
        assert_eq!(
            reopened.snapshot(CAMPAIGN).unwrap().campaign.spec,
            spec(CAMPAIGN)
        );
        assert_eq!(
            reopened.events(CAMPAIGN).unwrap()[0].kind,
            "campaign.created"
        );
    }
    std::fs::remove_file(path).unwrap();
}

#[test]
fn opening_an_existing_store_reads_through_an_uncommitted_write() {
    let path = std::env::temp_dir().join(format!(
        "aether-store-reader-{}-{}.sqlite",
        std::process::id(),
        now_ms().unwrap()
    ));
    {
        let mut writer = Store::open(&path).unwrap();
        writer.create_campaign(&spec(CAMPAIGN)).unwrap();
        let tx = writer
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .unwrap();
        tx.execute(
            "UPDATE campaign SET title = 'Pending title' WHERE id = ?",
            [CAMPAIGN],
        )
        .unwrap();
        append(&tx, CAMPAIGN, "test.title_updated", json!({})).unwrap();

        let reader = Store::open(&path).unwrap();
        let existing = reader.snapshot(CAMPAIGN).unwrap();
        assert_eq!(existing.campaign.spec.title, spec(CAMPAIGN).title);
        assert_eq!(existing.events.len(), 1);
        tx.commit().unwrap();
        let committed = reader.snapshot(CAMPAIGN).unwrap();
        assert_eq!(committed.campaign.spec.title, "Pending title");
        assert_eq!(committed.events[1].kind, "test.title_updated");
    }
    std::fs::remove_file(path).unwrap();
}

#[test]
fn corrupted_content_is_rejected_as_evidence() {
    let mut store = store_running();
    run(&mut store, "baseline", None);
    let original = attempt(&mut store, "original", "baseline");
    store.complete_attempt(&original).unwrap();
    let sha256 = artifact_sha256(&original.artifacts[0].bytes);

    // Simulate a trusted local corruption event outside the Store API.
    store
        .conn
        .execute_batch("DROP TRIGGER blob_immutable_update")
        .unwrap();
    store
        .conn
        .execute(
            "UPDATE blob SET content = ? WHERE sha256 = ?",
            params![b"corrupted test content".as_slice(), sha256],
        )
        .unwrap();
    assert!(matches!(
        store.artifact(&sha256),
        Err(StoreError::Integrity(_))
    ));

    run(&mut store, "replay", Some("baseline"));
    let replay = attempt(&mut store, "replayed", "replay");
    assert!(matches!(
        store.complete_attempt(&replay),
        Err(StoreError::Integrity(_))
    ));
    assert!(
        store
            .attempt("replayed")
            .unwrap()
            .unwrap()
            .outcome
            .is_none()
    );
}
