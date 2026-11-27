//! Deterministic structural checks for pinned research snapshots.
//! These checks do not establish relevance, entailment, novelty, calibration, or truth.

use serde::{Deserialize, Serialize};

pub const REPORT_SCHEMA: &str = "aether.quality-report/1";
pub const EVALUATOR_VERSION: &str = "aether-structural-evaluator/1.0.0";
pub const MAX_NODES: usize = 64;
pub const MAX_EDGES: usize = 192;
pub const MAX_SUBJECTS: usize = 8;

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DimensionId {
    LiteratureScopeInventoryIntegrity,
    ClaimSourceLocatorIntegrity,
    EvidenceProvenanceFieldIntegrity,
    CounterevidenceEdgeIntegrity,
    HypothesisStructureCompleteness,
    ExperimentDesignArithmeticIntegrity,
    UncertaintyFieldWellformedness,
    ArtifactSnapshotReplayIntegrity,
}

impl DimensionId {
    pub const ALL: [Self; 8] = [
        Self::LiteratureScopeInventoryIntegrity,
        Self::ClaimSourceLocatorIntegrity,
        Self::EvidenceProvenanceFieldIntegrity,
        Self::CounterevidenceEdgeIntegrity,
        Self::HypothesisStructureCompleteness,
        Self::ExperimentDesignArithmeticIntegrity,
        Self::UncertaintyFieldWellformedness,
        Self::ArtifactSnapshotReplayIntegrity,
    ];
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Outcome {
    Pass,
    Fail,
    Unavailable,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EpistemicStatus {
    Proposed,
    Reported,
    Extracted,
    Inferred,
    Observed,
    IndependentlyVerified,
    Unknown,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceIdentityStatus {
    Verified,
    Unverified,
    NotApplicable,
}

/// Provenance is an explicit tagged union. Optional legacy fields remain absent, not fabricated.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum EvidenceOrigin {
    Passage {
        paper: Option<String>,
        paper_version: Option<String>,
        document: Option<String>,
        source_sha256: Option<String>,
        source_url: Option<String>,
        license: Option<String>,
        start_byte: Option<u64>,
        end_byte: Option<u64>,
        section: Option<String>,
    },
    Metadata {
        paper: Option<String>,
        snapshot_sha256: Option<String>,
    },
    Archive {
        artifact: Option<String>,
        artifact_sha256: Option<String>,
        pointer: Option<String>,
    },
    Campaign {
        campaign: Option<String>,
        protocol_sha256: Option<String>,
        record: Option<String>,
    },
    Operator,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum GraphKind {
    Paper,
    Claim,
    Method,
    Dataset,
    Metric,
    Experiment,
    Model,
    Evidence,
    Limitation,
    Hypothesis,
    ResearchQuestion,
    Campaign,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceRelation {
    Supports,
    Contradicts,
    Other,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct GraphNodeFact {
    pub id: String,
    pub kind: GraphKind,
    pub revision_sha256: String,
    pub statement_sha256: Option<String>,
    pub statement_bytes: u64,
    pub source: Option<EvidenceOrigin>,
    pub source_identity_status: Option<SourceIdentityStatus>,
    pub epistemic_status: Option<EpistemicStatus>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct GraphEdgeFact {
    pub id: String,
    pub origin: String,
    pub target: String,
    pub relation: EvidenceRelation,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SubjectKind {
    Hypothesis,
    ExperimentDesign,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ArtifactKind {
    HypothesisReport,
    ExperimentDesignReport,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ArtifactRef {
    pub kind: ArtifactKind,
    pub sha256: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SubjectRef {
    pub kind: SubjectKind,
    pub artifact: ArtifactRef,
    pub pointer: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum SubjectFacts {
    Hypothesis {
        reference: SubjectRef,
        proposition_present: bool,
        support_count: usize,
        counterevidence_count: usize,
        alternative_count: usize,
        assumption_count: usize,
        limitation_count: usize,
        falsification_present: bool,
        metric_count: usize,
        resources_present: bool,
        uncertainty_grade: Option<u8>,
        epistemic_status: Option<EpistemicStatus>,
    },
    ExperimentDesign {
        reference: SubjectRef,
        independent_variable_count: usize,
        dependent_variable_present: bool,
        control_count: usize,
        baseline_present: bool,
        confounder_analysis_present: bool,
        development_and_test_separated: bool,
        sample_size_basis_present: bool,
        statistical_plan_present: bool,
        metric_count: usize,
        falsification_present: bool,
        termination_present: bool,
        resource_cap_attempts: u64,
        search_attempts: u64,
        verification_attempts: u64,
        oracle_attempts: u64,
        reserve_attempts: u64,
        resource_cap_wall_ms: u64,
        declared_cpu_ms_ceiling: u64,
    },
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct StructuralInput {
    pub graph_root_id: String,
    pub graph_sha256: String,
    pub graph_depth: u8,
    pub graph_truncated: bool,
    pub nodes: Vec<GraphNodeFact>,
    pub edges: Vec<GraphEdgeFact>,
    pub subjects: Vec<SubjectFacts>,
    pub snapshot_digest_verified: bool,
    pub graph_digest_verified: bool,
    pub source_replay_available: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct StructuralCheck {
    pub rule_id: String,
    pub outcome: Outcome,
    pub reason_codes: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DimensionResult {
    pub dimension: DimensionId,
    pub outcome: Outcome,
    pub checks: Vec<StructuralCheck>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct EvaluatorIdentity {
    pub version: String,
    pub executable_sha256: Option<String>,
    pub build_attestation: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DimensionRulePin {
    pub dimension: DimensionId,
    pub rule_id: String,
    pub version: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Report {
    pub schema: String,
    pub benchmark_sha256: String,
    pub dimension_rule_pins: Vec<DimensionRulePin>,
    pub evaluator: EvaluatorIdentity,
    pub snapshot_sha256: String,
    pub graph_root_id: String,
    pub graph_sha256: String,
    pub execution_outcome: String,
    pub dimensions: Vec<DimensionResult>,
    pub semantic_assessment: String,
    pub independent_scientific_validation: String,
    pub source_replay: String,
    pub limitations: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EvaluationError {
    InvalidInput(&'static str),
}

pub fn valid_digest(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

pub fn evaluate(
    input: &StructuralInput,
    benchmark_sha256: &str,
    snapshot_sha256: &str,
    rule_pins: &[DimensionRulePin],
    evaluator: EvaluatorIdentity,
) -> Result<Report, EvaluationError> {
    if !valid_digest(benchmark_sha256)
        || !valid_digest(snapshot_sha256)
        || !valid_digest(&input.graph_root_id)
        || !valid_digest(&input.graph_sha256)
        || input.graph_depth > 3
        || input.nodes.len() > MAX_NODES
        || input.edges.len() > MAX_EDGES
        || input.subjects.len() > MAX_SUBJECTS
        || rule_pins.len() != 8
        || rule_pins
            .iter()
            .map(|pin| pin.dimension)
            .collect::<Vec<_>>()
            != DimensionId::ALL
        || rule_pins
            .iter()
            .any(|pin| pin.version != "1" || pin.rule_id != expected_rule_id(pin.dimension))
    {
        return Err(EvaluationError::InvalidInput(
            "evaluation input is outside fixed limits",
        ));
    }
    if input.nodes.iter().any(|node| {
        !valid_digest(&node.id)
            || !valid_digest(&node.revision_sha256)
            || node
                .statement_sha256
                .as_deref()
                .is_some_and(|hash| !valid_digest(hash))
    }) || input.edges.iter().any(|edge| !valid_digest(&edge.id))
        || input.subjects.iter().any(|subject| {
            let reference = match subject {
                SubjectFacts::Hypothesis { reference, .. }
                | SubjectFacts::ExperimentDesign { reference, .. } => reference,
            };
            !valid_digest(&reference.artifact.sha256)
                || matches!(
                    (reference.kind, reference.artifact.kind),
                    (
                        SubjectKind::Hypothesis,
                        ArtifactKind::ExperimentDesignReport
                    ) | (
                        SubjectKind::ExperimentDesign,
                        ArtifactKind::HypothesisReport
                    )
                )
                || reference.pointer.len() > 512
                || !reference.pointer.starts_with('/')
        })
    {
        return Err(EvaluationError::InvalidInput(
            "invalid typed artifact or graph reference",
        ));
    }

    let nodes_by_id = input
        .nodes
        .iter()
        .map(|node| (node.id.as_str(), node))
        .collect::<std::collections::BTreeMap<_, _>>();
    let papers = input
        .nodes
        .iter()
        .filter(|node| node.kind == GraphKind::Paper)
        .count();
    let claims = input
        .nodes
        .iter()
        .filter(|node| node.kind == GraphKind::Claim)
        .collect::<Vec<_>>();
    let evidence = input
        .nodes
        .iter()
        .filter(|node| node.kind == GraphKind::Evidence)
        .collect::<Vec<_>>();
    let hypotheses = input
        .subjects
        .iter()
        .filter(|subject| matches!(subject, SubjectFacts::Hypothesis { .. }))
        .collect::<Vec<_>>();
    let designs = input
        .subjects
        .iter()
        .filter(|subject| matches!(subject, SubjectFacts::ExperimentDesign { .. }))
        .collect::<Vec<_>>();

    let dimensions = vec![
        one(
            DimensionId::LiteratureScopeInventoryIntegrity,
            "literature_inventory_present",
            if input.graph_truncated {
                Outcome::Unavailable
            } else if papers == 0 {
                Outcome::Fail
            } else {
                Outcome::Pass
            },
            if input.graph_truncated {
                "graph_truncated"
            } else if papers == 0 {
                "required_collection_empty"
            } else {
                ""
            },
        ),
        one(
            DimensionId::ClaimSourceLocatorIntegrity,
            "claim_source_locators_resolve",
            claim_locator_outcome(&claims, &input.edges, &nodes_by_id),
            claim_locator_reason(&claims, &input.edges, &nodes_by_id),
        ),
        one(
            DimensionId::EvidenceProvenanceFieldIntegrity,
            "evidence_provenance_fields_present",
            provenance_outcome(&evidence),
            provenance_reason(&evidence),
        ),
        one(
            DimensionId::CounterevidenceEdgeIntegrity,
            "counterevidence_edges_preserve_relation",
            contradiction_outcome(input, &nodes_by_id),
            contradiction_reason(input, &nodes_by_id),
        ),
        one(
            DimensionId::HypothesisStructureCompleteness,
            "hypothesis_required_fields_present",
            hypothesis_outcome(&hypotheses),
            hypothesis_reason(&hypotheses),
        ),
        one(
            DimensionId::ExperimentDesignArithmeticIntegrity,
            "experiment_design_budget_arithmetic",
            design_outcome(&designs),
            design_reason(&designs),
        ),
        one(
            DimensionId::UncertaintyFieldWellformedness,
            "uncertainty_fields_explicit_and_bounded",
            uncertainty_outcome(&hypotheses),
            uncertainty_reason(&hypotheses),
        ),
        DimensionResult {
            dimension: DimensionId::ArtifactSnapshotReplayIntegrity,
            outcome: if !input.snapshot_digest_verified || !input.graph_digest_verified {
                Outcome::Fail
            } else if input.graph_truncated || !input.source_replay_available {
                Outcome::Unavailable
            } else {
                Outcome::Pass
            },
            checks: vec![StructuralCheck {
                rule_id: "snapshot_and_source_replay_integrity".into(),
                outcome: if !input.snapshot_digest_verified || !input.graph_digest_verified {
                    Outcome::Fail
                } else if input.graph_truncated || !input.source_replay_available {
                    Outcome::Unavailable
                } else {
                    Outcome::Pass
                },
                reason_codes: if !input.snapshot_digest_verified || !input.graph_digest_verified {
                    vec!["digest_mismatch".into()]
                } else if input.graph_truncated {
                    vec!["graph_truncated".into()]
                } else if !input.source_replay_available {
                    vec!["source_replay_unavailable_rights_unknown".into()]
                } else {
                    vec![]
                },
            }],
        },
    ];

    Ok(Report {
        schema: REPORT_SCHEMA.into(),
        benchmark_sha256: benchmark_sha256.into(),
        dimension_rule_pins: rule_pins.to_vec(),
        evaluator,
        snapshot_sha256: snapshot_sha256.into(),
        graph_root_id: input.graph_root_id.clone(),
        graph_sha256: input.graph_sha256.clone(),
        execution_outcome: "completed".into(),
        dimensions,
        semantic_assessment: "not_evaluated".into(),
        independent_scientific_validation: "not_performed".into(),
        source_replay: if input.source_replay_available { "available" } else { "unavailable" }.into(),
        limitations: vec![
            "Structural checks do not assess relevance, entailment, novelty, calibration, or scientific truth.".into(),
            "Source identity and evidence truth remain unverified unless a separate trusted process establishes them.".into(),
            "Structural results must not be used as generator optimization or selection objectives.".into(),
        ],
    })
}

fn expected_rule_id(dimension: DimensionId) -> &'static str {
    match dimension {
        DimensionId::LiteratureScopeInventoryIntegrity => "literature_inventory_present",
        DimensionId::ClaimSourceLocatorIntegrity => "claim_source_locators_resolve",
        DimensionId::EvidenceProvenanceFieldIntegrity => "evidence_provenance_fields_present",
        DimensionId::CounterevidenceEdgeIntegrity => "counterevidence_edges_preserve_relation",
        DimensionId::HypothesisStructureCompleteness => "hypothesis_required_fields_present",
        DimensionId::ExperimentDesignArithmeticIntegrity => "experiment_design_budget_arithmetic",
        DimensionId::UncertaintyFieldWellformedness => "uncertainty_fields_explicit_and_bounded",
        DimensionId::ArtifactSnapshotReplayIntegrity => "snapshot_and_source_replay_integrity",
    }
}

fn one(dimension: DimensionId, rule: &str, outcome: Outcome, reason: &str) -> DimensionResult {
    DimensionResult {
        dimension,
        outcome,
        checks: vec![StructuralCheck {
            rule_id: rule.into(),
            outcome,
            reason_codes: if reason.is_empty() {
                vec![]
            } else {
                vec![reason.into()]
            },
        }],
    }
}

fn claim_locator_outcome(
    claims: &[&GraphNodeFact],
    edges: &[GraphEdgeFact],
    nodes: &std::collections::BTreeMap<&str, &GraphNodeFact>,
) -> Outcome {
    if claims.is_empty() {
        return Outcome::Fail;
    }
    let mut seen = false;
    let mut missing = false;
    for claim in claims {
        let refs = edges
            .iter()
            .filter(|edge| {
                edge.target == claim.id
                    && matches!(
                        edge.relation,
                        EvidenceRelation::Supports | EvidenceRelation::Contradicts
                    )
                    && nodes
                        .get(edge.origin.as_str())
                        .is_some_and(|node| node.kind == GraphKind::Evidence)
            })
            .collect::<Vec<_>>();
        if refs.is_empty() {
            missing = true;
            continue;
        }
        for edge in refs {
            seen = true;
            match nodes
                .get(edge.origin.as_str())
                .and_then(|node| node.source.as_ref())
                .map(locator_status)
            {
                Some(Some(true)) => (),
                Some(Some(false)) => return Outcome::Fail,
                Some(None) | None => missing = true,
            }
        }
    }
    if !seen || missing {
        Outcome::Unavailable
    } else {
        Outcome::Pass
    }
}

fn claim_locator_reason(
    claims: &[&GraphNodeFact],
    edges: &[GraphEdgeFact],
    nodes: &std::collections::BTreeMap<&str, &GraphNodeFact>,
) -> &'static str {
    if claims.is_empty() {
        return "required_collection_empty";
    }
    if claim_locator_outcome(claims, edges, nodes) == Outcome::Unavailable {
        "legacy_locator_unavailable"
    } else if claim_locator_outcome(claims, edges, nodes) == Outcome::Fail {
        "invalid_source_locator"
    } else {
        ""
    }
}

fn locator_status(origin: &EvidenceOrigin) -> Option<bool> {
    match origin {
        EvidenceOrigin::Passage {
            paper,
            document,
            source_sha256,
            start_byte,
            end_byte,
            ..
        } => {
            let (Some(paper), Some(document), Some(source_sha256), Some(start), Some(end)) = (
                paper.as_deref(),
                document.as_deref(),
                source_sha256.as_deref(),
                *start_byte,
                *end_byte,
            ) else {
                return None;
            };
            Some(
                !paper.trim().is_empty()
                    && !document.trim().is_empty()
                    && valid_digest(source_sha256)
                    && start < end,
            )
        }
        // A paper metadata record lacks a passage locator. Preserve it, but do not treat it as one.
        EvidenceOrigin::Metadata { .. } | EvidenceOrigin::Operator => None,
        EvidenceOrigin::Archive {
            artifact,
            artifact_sha256,
            pointer,
        } => {
            let (Some(artifact), Some(artifact_sha256), Some(pointer)) = (
                artifact.as_deref(),
                artifact_sha256.as_deref(),
                pointer.as_deref(),
            ) else {
                return None;
            };
            Some(
                !artifact.trim().is_empty()
                    && valid_digest(artifact_sha256)
                    && pointer.starts_with('/'),
            )
        }
        EvidenceOrigin::Campaign {
            campaign,
            protocol_sha256,
            record,
        } => {
            let (Some(campaign), Some(protocol_sha256), Some(record)) = (
                campaign.as_deref(),
                protocol_sha256.as_deref(),
                record.as_deref(),
            ) else {
                return None;
            };
            Some(
                !campaign.trim().is_empty()
                    && valid_digest(protocol_sha256)
                    && !record.trim().is_empty(),
            )
        }
    }
}

fn provenance_outcome(evidence: &[&GraphNodeFact]) -> Outcome {
    if evidence.is_empty() {
        return Outcome::Unavailable;
    }
    if evidence.iter().any(|node| {
        node.source.is_none()
            || node
                .source
                .as_ref()
                .is_some_and(|source| !provenance_valid(source))
    }) {
        return Outcome::Fail;
    }
    if evidence
        .iter()
        .any(|node| node.source_identity_status.is_none() || node.epistemic_status.is_none())
    {
        Outcome::Unavailable
    } else {
        Outcome::Pass
    }
}

fn provenance_reason(evidence: &[&GraphNodeFact]) -> &'static str {
    if evidence.is_empty() {
        "evidence_collection_empty"
    } else if provenance_outcome(evidence) == Outcome::Fail {
        "provenance_fields_invalid"
    } else if provenance_outcome(evidence) == Outcome::Unavailable {
        "epistemic_or_identity_status_missing"
    } else {
        ""
    }
}

fn provenance_valid(source: &EvidenceOrigin) -> bool {
    match source {
        EvidenceOrigin::Passage {
            paper,
            paper_version,
            document,
            source_sha256,
            source_url,
            license,
            start_byte,
            end_byte,
            section,
        } => {
            [paper, paper_version, document, source_url, license, section]
                .into_iter()
                .all(|value| value.as_ref().is_some_and(|text| !text.trim().is_empty()))
                && source_sha256.as_deref().is_some_and(valid_digest)
                && (*start_byte)
                    .zip(*end_byte)
                    .is_some_and(|(start, end)| start < end)
        }
        EvidenceOrigin::Metadata {
            paper,
            snapshot_sha256,
        } => paper.is_some() && snapshot_sha256.as_deref().is_some_and(valid_digest),
        EvidenceOrigin::Archive {
            artifact,
            artifact_sha256,
            pointer,
        } => {
            artifact.is_some()
                && artifact_sha256.as_deref().is_some_and(valid_digest)
                && pointer.is_some()
        }
        EvidenceOrigin::Campaign {
            campaign,
            protocol_sha256,
            record,
        } => {
            campaign.is_some()
                && protocol_sha256.as_deref().is_some_and(valid_digest)
                && record.is_some()
        }
        EvidenceOrigin::Operator => true,
    }
}

fn contradiction_outcome(
    input: &StructuralInput,
    nodes: &std::collections::BTreeMap<&str, &GraphNodeFact>,
) -> Outcome {
    let edges = input
        .edges
        .iter()
        .filter(|edge| edge.relation == EvidenceRelation::Contradicts)
        .collect::<Vec<_>>();
    if edges.is_empty() {
        return Outcome::Unavailable;
    }
    if edges.iter().any(|edge| {
        !nodes.contains_key(edge.origin.as_str())
            || !nodes.contains_key(edge.target.as_str())
            || edge.origin == edge.target
    }) {
        Outcome::Fail
    } else {
        Outcome::Pass
    }
}

fn contradiction_reason(
    input: &StructuralInput,
    nodes: &std::collections::BTreeMap<&str, &GraphNodeFact>,
) -> &'static str {
    if contradiction_outcome(input, nodes) == Outcome::Unavailable {
        "counterevidence_collection_empty"
    } else if contradiction_outcome(input, nodes) == Outcome::Fail {
        "invalid_counterevidence_reference"
    } else {
        ""
    }
}

fn hypothesis_outcome(items: &[&SubjectFacts]) -> Outcome {
    if items.is_empty() {
        return Outcome::Fail;
    }
    for item in items {
        let SubjectFacts::Hypothesis {
            proposition_present,
            support_count,
            alternative_count,
            assumption_count,
            limitation_count,
            falsification_present,
            metric_count,
            resources_present,
            ..
        } = item
        else {
            continue;
        };
        if !proposition_present
            || support_count == &0
            || alternative_count == &0
            || assumption_count == &0
            || limitation_count == &0
            || !falsification_present
            || metric_count == &0
            || !resources_present
        {
            return Outcome::Fail;
        }
    }
    Outcome::Pass
}

fn hypothesis_reason(items: &[&SubjectFacts]) -> &'static str {
    if items.is_empty() {
        "required_hypothesis_missing"
    } else if hypothesis_outcome(items) == Outcome::Fail {
        "hypothesis_required_field_missing_or_empty"
    } else {
        ""
    }
}

fn design_outcome(items: &[&SubjectFacts]) -> Outcome {
    if items.is_empty() {
        return Outcome::Fail;
    }
    for item in items {
        let SubjectFacts::ExperimentDesign {
            independent_variable_count,
            dependent_variable_present,
            control_count,
            baseline_present,
            confounder_analysis_present,
            development_and_test_separated,
            sample_size_basis_present,
            statistical_plan_present,
            metric_count,
            falsification_present,
            termination_present,
            resource_cap_attempts,
            search_attempts,
            verification_attempts,
            oracle_attempts,
            reserve_attempts,
            resource_cap_wall_ms,
            declared_cpu_ms_ceiling,
            ..
        } = item
        else {
            continue;
        };
        let Some(attempts) = search_attempts
            .checked_add(*verification_attempts)
            .and_then(|sum| sum.checked_add(*oracle_attempts))
        else {
            return Outcome::Fail;
        };
        let Some(reserve) = verification_attempts.checked_add(*oracle_attempts) else {
            return Outcome::Fail;
        };
        if *independent_variable_count == 0
            || !dependent_variable_present
            || *control_count == 0
            || !baseline_present
            || !confounder_analysis_present
            || !development_and_test_separated
            || !sample_size_basis_present
            || !statistical_plan_present
            || *metric_count == 0
            || !falsification_present
            || !termination_present
            || *resource_cap_attempts != attempts
            || *reserve_attempts != reserve
            || *resource_cap_wall_ms == 0
            || resource_cap_wall_ms != declared_cpu_ms_ceiling
        {
            return Outcome::Fail;
        }
    }
    Outcome::Pass
}

fn design_reason(items: &[&SubjectFacts]) -> &'static str {
    if items.is_empty() {
        "required_experiment_design_missing"
    } else if design_outcome(items) == Outcome::Fail {
        "design_fields_or_budget_arithmetic_invalid"
    } else {
        ""
    }
}

fn uncertainty_outcome(items: &[&SubjectFacts]) -> Outcome {
    if items.is_empty() {
        return Outcome::Fail;
    }
    if items.iter().any(|item| matches!(item, SubjectFacts::Hypothesis { uncertainty_grade: Some(value), .. } if *value > 4)) { return Outcome::Fail; }
    if items.iter().any(|item| {
        matches!(
            item,
            SubjectFacts::Hypothesis {
                uncertainty_grade: None,
                ..
            } | SubjectFacts::Hypothesis {
                epistemic_status: None,
                ..
            }
        )
    }) {
        Outcome::Unavailable
    } else {
        Outcome::Pass
    }
}

fn uncertainty_reason(items: &[&SubjectFacts]) -> &'static str {
    if items.is_empty() {
        "required_hypothesis_missing"
    } else if uncertainty_outcome(items) == Outcome::Fail {
        "uncertainty_out_of_range"
    } else if uncertainty_outcome(items) == Outcome::Unavailable {
        "uncertainty_or_epistemic_status_missing"
    } else {
        ""
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hash(byte: u8) -> String {
        std::iter::repeat_n(format!("{byte:02x}"), 32).collect()
    }
    fn minimal() -> StructuralInput {
        StructuralInput {
            graph_root_id: hash(1),
            graph_sha256: hash(2),
            graph_depth: 1,
            graph_truncated: false,
            nodes: vec![],
            edges: vec![],
            subjects: vec![],
            snapshot_digest_verified: true,
            graph_digest_verified: true,
            source_replay_available: false,
        }
    }
    fn dimension(report: &Report, id: DimensionId) -> Outcome {
        report
            .dimensions
            .iter()
            .find(|d| d.dimension == id)
            .unwrap()
            .outcome
    }
    fn rule_pins() -> Vec<DimensionRulePin> {
        DimensionId::ALL
            .into_iter()
            .map(|dimension| DimensionRulePin {
                dimension,
                rule_id: expected_rule_id(dimension).into(),
                version: "1".into(),
            })
            .collect()
    }
    fn report(input: &StructuralInput) -> Report {
        evaluate(
            input,
            &hash(3),
            &hash(4),
            &rule_pins(),
            EvaluatorIdentity {
                version: EVALUATOR_VERSION.into(),
                executable_sha256: None,
                build_attestation: "not_provided".into(),
            },
        )
        .unwrap()
    }

    #[test]
    fn fixed_dimensions_fail_closed_on_empty_collections_and_report_completed_execution() {
        let report = report(&minimal());
        assert_eq!(report.dimensions.len(), 8);
        assert_eq!(
            dimension(&report, DimensionId::LiteratureScopeInventoryIntegrity),
            Outcome::Fail
        );
        assert_eq!(
            dimension(&report, DimensionId::CounterevidenceEdgeIntegrity),
            Outcome::Unavailable
        );
        assert_eq!(
            dimension(&report, DimensionId::HypothesisStructureCompleteness),
            Outcome::Fail
        );
        assert_eq!(report.execution_outcome, "completed");
        assert_eq!(report.independent_scientific_validation, "not_performed");
    }

    #[test]
    fn missing_and_explicit_unknown_epistemic_status_are_distinct() {
        let mut input = minimal();
        input.subjects.push(SubjectFacts::Hypothesis {
            reference: SubjectRef {
                kind: SubjectKind::Hypothesis,
                artifact: ArtifactRef {
                    kind: ArtifactKind::HypothesisReport,
                    sha256: hash(5),
                },
                pointer: "/proposals/0".into(),
            },
            proposition_present: true,
            support_count: 1,
            counterevidence_count: 0,
            alternative_count: 1,
            assumption_count: 1,
            limitation_count: 1,
            falsification_present: true,
            metric_count: 1,
            resources_present: true,
            uncertainty_grade: Some(2),
            epistemic_status: None,
        });
        assert_eq!(
            dimension(&report(&input), DimensionId::UncertaintyFieldWellformedness),
            Outcome::Unavailable
        );
        if let SubjectFacts::Hypothesis {
            epistemic_status, ..
        } = &mut input.subjects[0]
        {
            *epistemic_status = Some(EpistemicStatus::Unknown);
        }
        assert_eq!(
            dimension(&report(&input), DimensionId::UncertaintyFieldWellformedness),
            Outcome::Pass
        );
    }

    #[test]
    fn absent_counterevidence_never_vacuously_passes_and_edge_polarity_is_preserved() {
        let mut input = minimal();
        assert_eq!(
            dimension(&report(&input), DimensionId::CounterevidenceEdgeIntegrity),
            Outcome::Unavailable
        );
        input.nodes = vec![
            GraphNodeFact {
                id: hash(6),
                kind: GraphKind::Evidence,
                revision_sha256: hash(7),
                statement_sha256: None,
                statement_bytes: 0,
                source: None,
                source_identity_status: Some(SourceIdentityStatus::Unverified),
                epistemic_status: Some(EpistemicStatus::Unknown),
            },
            GraphNodeFact {
                id: hash(8),
                kind: GraphKind::Claim,
                revision_sha256: hash(9),
                statement_sha256: None,
                statement_bytes: 0,
                source: None,
                source_identity_status: None,
                epistemic_status: Some(EpistemicStatus::Reported),
            },
        ];
        input.edges.push(GraphEdgeFact {
            id: hash(10),
            origin: hash(6),
            target: hash(8),
            relation: EvidenceRelation::Contradicts,
        });
        assert_eq!(
            dimension(&report(&input), DimensionId::CounterevidenceEdgeIntegrity),
            Outcome::Pass
        );
        assert_eq!(input.edges[0].origin, hash(6));
    }

    #[test]
    fn all_eight_dimensions_evaluate_with_structural_only_semantics() {
        let mut input = minimal();
        let evidence_id = hash(6);
        let claim_id = hash(7);
        input.nodes = vec![
            GraphNodeFact {
                id: hash(5),
                kind: GraphKind::Paper,
                revision_sha256: hash(11),
                statement_sha256: None,
                statement_bytes: 0,
                source: None,
                source_identity_status: None,
                epistemic_status: None,
            },
            GraphNodeFact {
                id: claim_id.clone(),
                kind: GraphKind::Claim,
                revision_sha256: hash(12),
                statement_sha256: Some(hash(13)),
                statement_bytes: 3,
                source: None,
                source_identity_status: None,
                epistemic_status: Some(EpistemicStatus::Reported),
            },
            GraphNodeFact {
                id: evidence_id.clone(),
                kind: GraphKind::Evidence,
                revision_sha256: hash(14),
                statement_sha256: Some(hash(15)),
                statement_bytes: 4,
                source: Some(EvidenceOrigin::Passage {
                    paper: Some("doi:10.1/example".into()),
                    paper_version: Some("v1".into()),
                    document: Some("doc-1".into()),
                    source_sha256: Some(hash(16)),
                    source_url: Some("https://example.invalid/paper".into()),
                    license: Some("CC0-1.0".into()),
                    start_byte: Some(1),
                    end_byte: Some(5),
                    section: Some("results".into()),
                }),
                source_identity_status: Some(SourceIdentityStatus::Unverified),
                epistemic_status: Some(EpistemicStatus::Unknown),
            },
        ];
        input.edges = vec![
            GraphEdgeFact {
                id: hash(17),
                origin: evidence_id.clone(),
                target: claim_id.clone(),
                relation: EvidenceRelation::Supports,
            },
            GraphEdgeFact {
                id: hash(18),
                origin: evidence_id,
                target: claim_id,
                relation: EvidenceRelation::Contradicts,
            },
        ];
        input.subjects = vec![
            SubjectFacts::Hypothesis {
                reference: SubjectRef {
                    kind: SubjectKind::Hypothesis,
                    artifact: ArtifactRef {
                        kind: ArtifactKind::HypothesisReport,
                        sha256: hash(19),
                    },
                    pointer: "/proposals/0".into(),
                },
                proposition_present: true,
                support_count: 1,
                counterevidence_count: 1,
                alternative_count: 1,
                assumption_count: 1,
                limitation_count: 1,
                falsification_present: true,
                metric_count: 1,
                resources_present: true,
                uncertainty_grade: Some(2),
                epistemic_status: Some(EpistemicStatus::Unknown),
            },
            SubjectFacts::ExperimentDesign {
                reference: SubjectRef {
                    kind: SubjectKind::ExperimentDesign,
                    artifact: ArtifactRef {
                        kind: ArtifactKind::ExperimentDesignReport,
                        sha256: hash(20),
                    },
                    pointer: "/designs/0".into(),
                },
                independent_variable_count: 1,
                dependent_variable_present: true,
                control_count: 1,
                baseline_present: true,
                confounder_analysis_present: true,
                development_and_test_separated: true,
                sample_size_basis_present: true,
                statistical_plan_present: true,
                metric_count: 1,
                falsification_present: true,
                termination_present: true,
                resource_cap_attempts: 5,
                search_attempts: 3,
                verification_attempts: 1,
                oracle_attempts: 1,
                reserve_attempts: 2,
                resource_cap_wall_ms: 100,
                declared_cpu_ms_ceiling: 100,
            },
        ];
        let report = report(&input);
        for id in DimensionId::ALL.iter().take(7) {
            assert_eq!(dimension(&report, *id), Outcome::Pass, "{id:?}");
        }
        assert_eq!(
            dimension(&report, DimensionId::ArtifactSnapshotReplayIntegrity),
            Outcome::Unavailable
        );
        assert_eq!(report.semantic_assessment, "not_evaluated");
        assert_eq!(
            report
                .dimensions
                .iter()
                .map(|row| row.dimension)
                .collect::<Vec<_>>(),
            DimensionId::ALL
        );
    }

    #[test]
    fn empty_and_placeholder_hypothesis_or_design_fields_fail() {
        let mut input = minimal();
        input.subjects.push(SubjectFacts::Hypothesis {
            reference: SubjectRef {
                kind: SubjectKind::Hypothesis,
                artifact: ArtifactRef {
                    kind: ArtifactKind::HypothesisReport,
                    sha256: hash(21),
                },
                pointer: "/proposals/0".into(),
            },
            proposition_present: false,
            support_count: 0,
            counterevidence_count: 0,
            alternative_count: 0,
            assumption_count: 0,
            limitation_count: 0,
            falsification_present: false,
            metric_count: 0,
            resources_present: false,
            uncertainty_grade: None,
            epistemic_status: None,
        });
        assert_eq!(
            dimension(
                &report(&input),
                DimensionId::HypothesisStructureCompleteness
            ),
            Outcome::Fail
        );
        input.subjects.clear();
        input.subjects.push(SubjectFacts::ExperimentDesign {
            reference: SubjectRef {
                kind: SubjectKind::ExperimentDesign,
                artifact: ArtifactRef {
                    kind: ArtifactKind::ExperimentDesignReport,
                    sha256: hash(22),
                },
                pointer: "/designs/0".into(),
            },
            independent_variable_count: 0,
            dependent_variable_present: false,
            control_count: 0,
            baseline_present: false,
            confounder_analysis_present: false,
            development_and_test_separated: false,
            sample_size_basis_present: false,
            statistical_plan_present: false,
            metric_count: 0,
            falsification_present: false,
            termination_present: false,
            resource_cap_attempts: 0,
            search_attempts: 0,
            verification_attempts: 0,
            oracle_attempts: 0,
            reserve_attempts: 0,
            resource_cap_wall_ms: 0,
            declared_cpu_ms_ceiling: 0,
        });
        assert_eq!(
            dimension(
                &report(&input),
                DimensionId::ExperimentDesignArithmeticIntegrity
            ),
            Outcome::Fail
        );
    }

    #[test]
    fn legacy_metadata_without_a_passage_locator_is_unavailable() {
        let mut input = minimal();
        let claim_id = hash(23);
        input.nodes = vec![
            GraphNodeFact {
                id: claim_id.clone(),
                kind: GraphKind::Claim,
                revision_sha256: hash(24),
                statement_sha256: None,
                statement_bytes: 0,
                source: None,
                source_identity_status: None,
                epistemic_status: None,
            },
            GraphNodeFact {
                id: hash(25),
                kind: GraphKind::Evidence,
                revision_sha256: hash(26),
                statement_sha256: None,
                statement_bytes: 0,
                source: Some(EvidenceOrigin::Metadata {
                    paper: Some("doi:10.1/legacy".into()),
                    snapshot_sha256: Some(hash(27)),
                }),
                source_identity_status: Some(SourceIdentityStatus::Unverified),
                epistemic_status: Some(EpistemicStatus::Reported),
            },
        ];
        input.edges.push(GraphEdgeFact {
            id: hash(28),
            origin: hash(25),
            target: claim_id,
            relation: EvidenceRelation::Supports,
        });
        assert_eq!(
            dimension(&report(&input), DimensionId::ClaimSourceLocatorIntegrity),
            Outcome::Unavailable
        );
    }

    #[test]
    fn invalid_refs_unknown_enums_and_unknown_fields_are_rejected() {
        let mut input = minimal();
        input.graph_root_id = "not-a-digest".into();
        assert!(
            evaluate(
                &input,
                &hash(3),
                &hash(4),
                &rule_pins(),
                EvaluatorIdentity {
                    version: "v".into(),
                    executable_sha256: None,
                    build_attestation: "not_provided".into()
                }
            )
            .is_err()
        );
        assert!(serde_json::from_str::<EpistemicStatus>("\"verified_by_magic\"").is_err());
        assert!(serde_json::from_str::<SubjectRef>(r#"{"kind":"hypothesis","artifact":{"kind":"hypothesis_report","sha256":"0000000000000000000000000000000000000000000000000000000000000000"},"pointer":"/x","extra":true}"#).is_err());
    }
}
