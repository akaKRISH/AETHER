use super::*;

fn fixture() -> (CitationCatalog, String) {
    let mut catalog = CitationCatalog::open(":memory:").unwrap();
    let indexed = catalog.index_hypothesis_archive().unwrap();
    (catalog, indexed["root"].as_str().unwrap().into())
}

#[test]
fn complete_designs_reproduce_and_reserve_the_matched_grid() {
    let (catalog, root) = fixture();
    let first = catalog.design_experiments(&root).unwrap();
    let (fresh, fresh_root) = fixture();
    assert_eq!(first, fresh.design_experiments(&fresh_root).unwrap());
    let (_, drafts) = catalog.design_drafts(&root).unwrap();
    assert_eq!(drafts.len(), 3);
    let mut seeds = BTreeSet::new();
    for draft in drafts {
        let protocol = &draft.protocol;
        let counts = match protocol.recipe {
            Recipe::NegativeResult => (20, 11, 320, 100, 70_000),
            Recipe::Boundary => (24, 12, 384, 120, 84_000),
            Recipe::MethodComposition => (12, 6, 384, 108, 72_000),
        };
        assert_eq!(
            (
                protocol.manifest.development_cases.len(),
                protocol.analysis.seed_blocks,
                protocol.resources.candidate_evaluations,
                protocol.resources.cap.attempts,
                protocol.resources.cap.wall_ms
            ),
            counts
        );
        assert_eq!(draft.approval_status, Approval::Unapproved);
        assert!(!protocol.supports.is_empty());
        assert!(protocol.manifest.held_out_commitment.is_none());
        assert!(protocol.analysis.power_estimate.is_none());
        for seed in protocol
            .manifest
            .development_cases
            .iter()
            .map(|case| case.seed)
            .collect::<BTreeSet<_>>()
        {
            assert!(seeds.insert(seed));
        }
        let receipt = catalog.validate_experiment_design(json!(draft)).unwrap();
        assert_eq!(receipt["approval_status"], "unapproved");
    }
}

#[test]
fn invalid_or_leaky_designs_are_rejected() {
    let (catalog, root) = fixture();
    let prepare = || catalog.design_drafts(&root).unwrap().1.remove(0).protocol;
    for edit in [
        (|p: &mut Protocol| {
            p.arms[0] = Cell::Registered {
                arm: Arm::Greedy,
                candidate_evaluations: 9,
            };
        }) as fn(&mut Protocol),
        |p| p.controls.oracle_phase = OraclePhase::DuringSearch,
        |p| p.controls.candidate_visible_oracle = true,
        |p| p.manifest.exclude_archive_input_bytes = false,
        |p| p.manifest.development_cases[0].seed = SEEDS[0],
        |p| p.manifest.development_cases.truncate(1),
        |p| p.manifest.held_out_commitment = Some("a".repeat(64)),
        |p| p.mode = StudyMode::Confirmatory,
        |p| p.analysis.power_estimate = Some(80),
        |p| p.manifest.execution_task = Some(aether_runner::Task::Classification {}),
        |p| p.resources.cap.attempts -= 1,
        |p| p.resources.verification_reserve.wall_ms -= 1,
    ] {
        let mut draft = prepare();
        edit(&mut draft);
        assert!(
            draft.validate().is_err(),
            "accepted invalid {:?}",
            draft.recipe
        );
    }
    let mut draft = json!(prepare());
    draft["controls"]["dependent_variable"] = json!("oracle_labels");
    assert!(serde_json::from_value::<Protocol>(draft).is_err());
    let mut draft = json!(prepare());
    draft["manifest"]["execution_task"] = json!({"kind": "paired_knapsack"});
    assert!(serde_json::from_value::<Protocol>(draft).is_err());
}

#[test]
fn budget_arithmetic_fails_closed_on_overflow() {
    assert!(resources(u64::MAX, 2, PER_ARM).is_err());
    assert!(resources(2, 1, u64::MAX).is_err());
    assert!(resources(u64::MAX / 4096 + 1, 1, 1).is_err());
}

#[test]
fn hashes_do_not_bless_mutation_or_promote_drafts() {
    let (catalog, root) = fixture();
    let mut draft = catalog.design_drafts(&root).unwrap().1.remove(0);
    draft.protocol.analysis.falsification.clear();
    let failed = catalog
        .validate_experiment_design(json!(draft))
        .unwrap_err();
    assert!(failed.to_string().contains("digest mismatch"));
    draft.protocol_sha256 = digest(&draft.protocol).unwrap();
    let failed = catalog
        .validate_experiment_design(json!(draft))
        .unwrap_err();
    assert!(failed.to_string().contains("no longer matches"));
    let mut forged = catalog.design_experiments(&root).unwrap()["designs"][0].clone();
    forged["approval_status"] = json!("approved");
    assert!(catalog.validate_experiment_design(forged).is_err());
}
