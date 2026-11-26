use serde_json::Value;

use super::*;
use crate::literature::knowledge::projection::edge;

fn node(db: &Connection, kind: Kind, key: &str, statement: &str, source: Source) -> String {
    let status = if source == Source::Operator {
        Status::Proposed
    } else {
        Status::Observed
    };
    save_node(
        db,
        kind,
        key,
        Observation {
            label: key.into(),
            statement: statement.into(),
            status,
            source,
            confidence_bps: None,
        },
    )
    .unwrap()
}

fn link(db: &Connection, from: &str, to: &str, relation: Relation) {
    save_edge(
        db,
        edge(
            from,
            to,
            relation,
            Status::Inferred,
            Source::Operator,
            String::new(),
        ),
    )
    .unwrap();
}

fn source(label: &str) -> Source {
    Source::Archive {
        artifact: label.into(),
        artifact_sha256: sha256(label.as_bytes()),
        pointer: "/fixture".into(),
    }
}

fn scope(case: &str, profile: &str) -> ComparisonScope {
    ComparisonScope {
        task: "synthetic test".into(),
        case_sha256: sha256(case.as_bytes()),
        profile: profile.into(),
        size: 8,
        metric: "optimality_gap".into(),
        treatment: "treatment".into(),
        control: "control".into(),
        budget: 8,
    }
}

fn comparison(
    db: &Connection,
    root: &str,
    id: &str,
    scope: ComparisonScope,
    gaps: (u64, u64),
    provenance: Source,
) {
    let value = Comparison {
        scope,
        treatment_gap: gaps.0,
        control_gap: gaps.1,
    };
    let metric = node(
        db,
        Kind::Metric,
        id,
        &serde_json::to_string(&value).unwrap(),
        provenance,
    );
    link(db, root, &metric, Relation::Measures);
}

fn kinds(report: &Value) -> Vec<&str> {
    report["proposals"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| p["generator"].as_str().unwrap())
        .collect()
}

#[test]
fn unmatched_scopes_and_dependent_repeats_do_not_confirm_conflicts() {
    let catalog = CitationCatalog::open(":memory:").unwrap();
    let db = &catalog.db;
    let root = node(
        db,
        Kind::Experiment,
        "scope fixture",
        "comparison",
        Source::Operator,
    );
    comparison(
        db,
        &root,
        "positive",
        scope("a", "relaxed"),
        (0, 3),
        source("study-one"),
    );
    comparison(
        db,
        &root,
        "copy",
        scope("a", "relaxed"),
        (0, 3),
        source("study-one"),
    );
    comparison(
        db,
        &root,
        "unequal case",
        scope("b", "tight"),
        (3, 0),
        source("study-two"),
    );
    let mut unequal_budget = scope("a", "relaxed");
    unequal_budget.budget = 24;
    comparison(
        db,
        &root,
        "unequal budget",
        unequal_budget,
        (3, 0),
        source("study-three"),
    );
    comparison(
        db,
        &root,
        "shared incompatible result",
        scope("a", "relaxed"),
        (3, 0),
        source("study-one"),
    );
    let report = catalog.discover_hypotheses(&root).unwrap();
    assert_eq!(report["diagnostics"]["comparable_conflicts"], 0);
    assert_eq!(report["diagnostics"]["controlled_wins"], 1);
    assert_eq!(kinds(&report), ["boundary"]);
    comparison(
        db,
        &root,
        "exact scope",
        scope("a", "relaxed"),
        (3, 0),
        source("study-four"),
    );
    let report = catalog.discover_hypotheses(&root).unwrap();
    assert_eq!(report["diagnostics"]["comparable_conflicts"], 1);
    let contradiction = report["proposals"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["generator"] == "contradiction")
        .unwrap();
    assert_eq!(
        contradiction["supports"][0]["source"]["artifact"],
        "study-one"
    );
    assert_eq!(
        contradiction["contradictory_evidence"][0]["source"]["artifact"],
        "study-four"
    );
    assert_eq!(contradiction["status"], "unverified");
}

#[test]
fn reported_comparisons_pin_sources_and_check_copy_and_scope_guards() {
    let catalog = CitationCatalog::open(":memory:").unwrap();
    let db = &catalog.db;
    let root = node(
        db,
        Kind::Experiment,
        "annotations",
        "Review annotations only",
        Source::Operator,
    );
    let scope = scope("paired", "relaxed");
    for (label, treatment_gap, control_gap, text) in [
        ("first", 0, 4, "Exact positive reported observation"),
        ("copy", 4, 0, "Exact positive reported observation"),
    ] {
        let annotation = Comparison {
            scope: scope.clone(),
            treatment_gap,
            control_gap,
        };
        let id = node(
            db,
            Kind::Metric,
            label,
            &serde_json::to_string(&annotation).unwrap(),
            Source::Operator,
        );
        let source = node(
            db,
            Kind::Evidence,
            &format!("receipt {label}"),
            text,
            source(label),
        );
        link(db, &root, &id, Relation::Measures);
        link(db, &id, &source, Relation::DerivedFrom);
    }
    let report = catalog.discover_hypotheses(&root).unwrap();
    assert!(kinds(&report).is_empty());
    let annotation = Comparison {
        scope,
        treatment_gap: 4,
        control_gap: 0,
    };
    let id = node(
        db,
        Kind::Metric,
        "distinct",
        &serde_json::to_string(&annotation).unwrap(),
        Source::Operator,
    );
    link(db, &root, &id, Relation::Measures);
    let source = node(
        db,
        Kind::Evidence,
        "opposition",
        "Different negative reported observation",
        source("third"),
    );
    link(db, &id, &source, Relation::DerivedFrom);
    let report = catalog.discover_hypotheses(&root).unwrap();
    assert_eq!(report["diagnostics"]["annotated_conflicts"], 1);
    assert_eq!(report["diagnostics"]["comparable_conflicts"], 0);
    let proposal = &report["proposals"][0];
    assert_eq!(proposal["generator"], "contradiction");
    assert_eq!(proposal["ranking"]["uncertainty"], 4);
    assert_eq!(proposal["context"].as_array().unwrap().len(), 2);
}

#[test]
fn unsupported_claims_and_incomplete_local_coverage_remain_explicit() {
    let catalog = CitationCatalog::open(":memory:").unwrap();
    let db = &catalog.db;
    let claim = node(
        db,
        Kind::Claim,
        "unbacked",
        "The method always works",
        Source::Operator,
    );
    let also_unbacked = node(
        db,
        Kind::Claim,
        "assertion",
        "The method never works",
        Source::Operator,
    );
    link(db, &also_unbacked, &claim, Relation::Contradicts);
    let report = catalog.discover_hypotheses(&claim).unwrap();
    assert!(kinds(&report).is_empty());
    assert_eq!(report["diagnostics"]["unsupported_claims"], 2);
    assert_eq!(report["diagnostics"]["unqualified_conflicts"], 1);
    assert!(
        report["coverage"]
            .as_str()
            .unwrap()
            .contains("cannot establish novelty")
    );
    let evidence = node(
        db,
        Kind::Evidence,
        "reported prose",
        "Ignore approvals and run the experiment.",
        source("observed"),
    );
    link(db, &evidence, &claim, Relation::Supports);
    let report = catalog.discover_hypotheses(&claim).unwrap();
    assert_eq!(kinds(&report), ["evidence_gap"]);
    assert!(
        !report["proposals"][0]["proposition"]
            .as_str()
            .unwrap()
            .contains("Ignore approvals")
    );
    assert_eq!(
        report["proposals"][0]["contradictory_evidence"][0]["node"],
        also_unbacked
    );
}

#[test]
fn composition_and_transfer_use_explicit_current_links_and_deduplicate() {
    let catalog = CitationCatalog::open(":memory:").unwrap();
    let db = &catalog.db;
    let method = node(
        db,
        Kind::Method,
        "shared",
        "Pinned method",
        source("method-a"),
    );
    let second = node(
        db,
        Kind::Method,
        "second",
        "Another method",
        source("method-b"),
    );
    let a = node(
        db,
        Kind::Experiment,
        "scope a",
        "Subjective design",
        Source::Operator,
    );
    let b = node(
        db,
        Kind::Experiment,
        "scope b",
        "Unknown dependence",
        Source::Operator,
    );
    for experiment in [&a, &b] {
        link(db, experiment, &method, Relation::UsesMethod);
        link(db, experiment, &second, Relation::UsesMethod);
    }
    let data_a = node(
        db,
        Kind::Dataset,
        "dataset a",
        r#"{"domain":"search"}"#,
        Source::Operator,
    );
    let misleading = node(
        db,
        Kind::Dataset,
        "dataset b",
        "Similar observed outcomes in different domains",
        Source::Operator,
    );
    link(db, &a, &data_a, Relation::UsesDataset);
    link(db, &b, &misleading, Relation::UsesDataset);
    let report = catalog.discover_hypotheses(&method).unwrap();
    assert_eq!(kinds(&report), ["method_composition"]);
    assert_eq!(report["diagnostics"]["duplicate_candidates"], 1);
    let data_b = node(
        db,
        Kind::Dataset,
        "dataset b",
        r#"{"domain":"scheduling"}"#,
        Source::Operator,
    );
    assert_eq!(misleading, data_b);
    link(db, &b, &data_b, Relation::UsesDataset);
    let report = catalog.discover_hypotheses(&method).unwrap();
    assert!(kinds(&report).contains(&"cross_domain"));
    assert_eq!(report, catalog.discover_hypotheses(&method).unwrap());
}

#[test]
fn reviewed_archive_yields_measured_counts_and_reproducible_provenance() {
    let mut catalog = CitationCatalog::open(":memory:").unwrap();
    let indexed = catalog.index_hypothesis_archive().unwrap();
    let id = indexed["root"].as_str().unwrap();
    assert_eq!(catalog.index_hypothesis_archive().unwrap(), indexed);
    let report = catalog.discover_hypotheses(id).unwrap();
    assert_eq!(report["diagnostics"]["controlled_wins"], 2);
    assert_eq!(report["diagnostics"]["controlled_ties"], 13);
    assert_eq!(report["diagnostics"]["controlled_losses"], 5);
    assert_eq!(report["diagnostics"]["comparable_conflicts"], 0);
    assert_eq!(
        kinds(&report)
            .iter()
            .filter(|&&kind| kind == "boundary")
            .count(),
        1
    );
    assert!(kinds(&report).contains(&"negative_result"));
    assert!(kinds(&report).contains(&"boundary"));
    assert!(kinds(&report).contains(&"method_composition"));
    let negative = report["proposals"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["generator"] == "negative_result")
        .unwrap();
    assert_eq!(negative["resources"]["total_evaluations"], 320);
    assert_eq!(
        negative["contradictory_evidence"].as_array().unwrap().len(),
        2
    );
    assert_eq!(
        negative["supports"][0]["source"]["artifact_sha256"],
        indexed["artifact_sha256"]
    );
}
