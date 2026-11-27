use std::collections::BTreeSet;

use aether_engine::quality::{
    self as evaluator, ArtifactKind, ArtifactRef, DimensionId, DimensionRulePin, EpistemicStatus,
    EvaluatorIdentity, EvidenceOrigin, EvidenceRelation, GraphEdgeFact, GraphNodeFact, Outcome,
    Report, SourceIdentityStatus, StructuralInput, SubjectFacts, SubjectKind, SubjectRef,
};
use aether_runner::sha256;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::lab::{Error as LabError, Lab};

type QualityResult<T> = std::result::Result<T, LabError>;

const BENCHMARK: &[u8] = include_bytes!("../../../../contracts/apo11/benchmark.json");
const BENCHMARK_DETAIL_FIXTURE: &[u8] =
    include_bytes!("../../../../contracts/apo11/fixtures/benchmark-detail.json");
const BENCHMARK_LIST_FIXTURE: &[u8] =
    include_bytes!("../../../../contracts/apo11/fixtures/benchmark-catalog.json");
const SNAPSHOT_SCHEMA: &str = "aether.quality-snapshot/1";
const SNAPSHOT_RECEIPT_SCHEMA: &str = "aether.quality-snapshot-receipt/1";
const REQUEST_SCHEMA: &str = "aether.quality-snapshot-request/1";
const RUN_SCHEMA: &str = "aether.quality-run-request/1";
const REPORT_ENVELOPE_SCHEMA: &str = "aether.quality-report-envelope/1";
const MAX_ARTIFACT_BYTES: usize = 2 * 1024 * 1024;
const MAX_SNAPSHOT_BYTES: usize = 4 * 1024 * 1024;
const MAX_REPORT_BYTES: usize = 1024 * 1024;
const MAX_REPORT_LIST_BYTES: usize = 256 * 1024;
const MAX_SUBJECTS: usize = 8;
const MAX_PAGE: usize = 32;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Benchmark {
    schema: String,
    id: String,
    version: String,
    scope: String,
    evaluator_version: String,
    dimension_rules: Vec<RulePin>,
    limits: Limits,
    excluded_metrics: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct RulePin {
    dimension: DimensionId,
    rule_id: String,
    version: String,
    title: String,
    scope: String,
    cannot_establish: String,
    empty_collection_policy: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Limits {
    request_bytes: usize,
    graph_depth: u8,
    graph_nodes: usize,
    graph_edges: usize,
    subjects: usize,
    artifact_bytes: usize,
    snapshot_bytes: usize,
    report_bytes: usize,
    report_list_bytes: usize,
    page_size: usize,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct SnapshotRequest {
    schema: String,
    graph_root_id: String,
    graph_depth: u8,
    subjects: Vec<SubjectRef>,
    generate_hypotheses: bool,
    generate_experiment_designs: bool,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RunRequest {
    schema: String,
    benchmark_sha256: String,
    snapshot_sha256: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct SnapshotDocument {
    schema: String,
    input: StructuralInput,
    source_text_included: bool,
    source_replay: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct ReportEnvelope {
    schema: String,
    report_sha256: String,
    sequence: i64,
    report: Report,
}

#[derive(Clone, Debug, Serialize)]
#[serde(deny_unknown_fields)]
struct DimensionSummary {
    dimension: DimensionId,
    outcome: Outcome,
}

#[derive(Clone, Debug, Serialize)]
#[serde(deny_unknown_fields)]
struct ReportSummary {
    report_sha256: String,
    sequence: i64,
    benchmark_sha256: String,
    snapshot_sha256: String,
    execution_outcome: String,
    dimensions: Vec<DimensionSummary>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(deny_unknown_fields)]
struct ReportList {
    schema: String,
    reports: Vec<ReportSummary>,
    truncated: bool,
    next_before_sequence: Option<i64>,
}

pub(crate) fn benchmarks(lab: &Lab) -> QualityResult<Value> {
    let expected = sha256(BENCHMARK);
    let parsed: Value = serde_json::from_slice(BENCHMARK_LIST_FIXTURE)?;
    // Keep the digest derived from the actual pinned spec, not from display metadata.
    let mut result = parsed;
    result["benchmarks"][0]["benchmark_sha256"] = json!(expected);
    let _ = lab;
    Ok(result)
}

pub(crate) fn benchmark(lab: &Lab, digest: &str) -> QualityResult<Value> {
    let expected = sha256(BENCHMARK);
    if !evaluator::valid_digest(digest) {
        return Err(LabError::Input(
            "benchmark SHA-256 must be lowercase hex".into(),
        ));
    }
    if digest != expected {
        return Err(LabError::Missing);
    }
    let parsed: Value = serde_json::from_slice(BENCHMARK)?;
    let fixture: Value = serde_json::from_slice(BENCHMARK_DETAIL_FIXTURE)?;
    if fixture["benchmark"] != parsed {
        return Err(LabError::Integrity(
            "benchmark detail golden fixture is stale".into(),
        ));
    }
    let spec: Benchmark = serde_json::from_value(parsed.clone())?;
    validate_benchmark(&spec)?;
    let _ = lab;
    Ok(
        json!({"schema":"aether.quality-benchmark-detail/1","benchmark_sha256":expected,"benchmark":parsed}),
    )
}

fn validate_benchmark(spec: &Benchmark) -> QualityResult<()> {
    if spec.schema != "aether.quality-benchmark/1"
        || spec.id != "aether-research-quality-structural"
        || spec.version != "1.0.0"
        || spec.scope != "structural_checks_only"
        || spec.evaluator_version != evaluator::EVALUATOR_VERSION
        || spec.dimension_rules.len() != 8
        || spec
            .dimension_rules
            .iter()
            .map(|rule| rule.dimension)
            .collect::<Vec<_>>()
            != DimensionId::ALL
        || spec.dimension_rules.iter().any(|rule| {
            rule.version != "1"
                || rule.title.is_empty()
                || rule.scope.is_empty()
                || rule.cannot_establish.is_empty()
                || rule.rule_id.is_empty()
        })
        || spec.excluded_metrics.len() < 6
        || spec.limits.request_bytes != 8192
        || spec.limits.graph_depth != 3
        || spec.limits.graph_nodes != evaluator::MAX_NODES
        || spec.limits.graph_edges != evaluator::MAX_EDGES
        || spec.limits.subjects != MAX_SUBJECTS
        || spec.limits.artifact_bytes != MAX_ARTIFACT_BYTES
        || spec.limits.snapshot_bytes != MAX_SNAPSHOT_BYTES
        || spec.limits.report_bytes != MAX_REPORT_BYTES
        || spec.limits.report_list_bytes != MAX_REPORT_LIST_BYTES
        || spec.limits.page_size != MAX_PAGE
    {
        return Err(LabError::Integrity(
            "benchmark schema or limits differ from the evaluator".into(),
        ));
    }
    Ok(())
}

pub(crate) fn snapshot(lab: &Lab, request: SnapshotRequest) -> QualityResult<(Value, bool)> {
    if request.schema != REQUEST_SCHEMA
        || !evaluator::valid_digest(&request.graph_root_id)
        || request.graph_depth > 3
        || request.subjects.len() > MAX_SUBJECTS
    {
        return Err(LabError::Input(
            "invalid snapshot request or fixed resource limit exceeded".into(),
        ));
    }
    let mut unique = BTreeSet::new();
    if request.subjects.iter().any(|reference| {
        !evaluator::valid_digest(&reference.artifact.sha256)
            || !reference.pointer.starts_with('/')
            || reference.pointer.len() > 512
            || !unique.insert((
                reference.kind,
                reference.artifact.sha256.clone(),
                reference.pointer.clone(),
            ))
    }) {
        return Err(LabError::Input(
            "invalid or duplicate typed subject reference".into(),
        ));
    }

    let catalog = lab.catalog()?;
    let first = catalog
        .graph_export(&request.graph_root_id, request.graph_depth)
        .map_err(|error| LabError::Input(error.to_string()))?;
    if !first.digest_valid() {
        return Err(LabError::Integrity(
            "knowledge graph export digest mismatch".into(),
        ));
    }
    let mut references = request.subjects.clone();
    if request.generate_hypotheses || request.generate_experiment_designs {
        if request.graph_depth != 2 {
            return Err(LabError::Input(
                "generated APO-03/APO-04 artifacts require graph depth two".into(),
            ));
        }
        if request.generate_hypotheses {
            let mut generated = catalog
                .discover_hypotheses(&request.graph_root_id)
                .map_err(|error| LabError::Input(error.to_string()))?;
            if generated["graph_sha256"] != first.graph_sha256() {
                return Err(LabError::Conflict(
                    "graph changed while generating hypotheses".into(),
                ));
            }
            let raw_digest = sha256(&serde_json::to_vec(&generated)?);
            remove_source_statements(&mut generated);
            generated["source_report_sha256"] = json!(raw_digest);
            let digest = persist_projection(lab, &generated)?;
            for index in 0..generated["proposals"].as_array().map_or(0, Vec::len) {
                references.push(SubjectRef {
                    kind: SubjectKind::Hypothesis,
                    artifact: ArtifactRef {
                        kind: ArtifactKind::HypothesisReport,
                        sha256: digest.clone(),
                    },
                    pointer: format!("/proposals/{index}"),
                });
            }
        }
        if request.generate_experiment_designs {
            let mut generated = catalog
                .design_experiments(&request.graph_root_id)
                .map_err(|error| LabError::Input(error.to_string()))?;
            if generated["graph_sha256"] != first.graph_sha256() {
                return Err(LabError::Conflict(
                    "graph changed while generating experiment designs".into(),
                ));
            }
            let raw_digest = sha256(&serde_json::to_vec(&generated)?);
            remove_source_statements(&mut generated);
            generated["source_report_sha256"] = json!(raw_digest);
            let digest = persist_projection(lab, &generated)?;
            for index in 0..generated["designs"].as_array().map_or(0, Vec::len) {
                references.push(SubjectRef {
                    kind: SubjectKind::ExperimentDesign,
                    artifact: ArtifactRef {
                        kind: ArtifactKind::ExperimentDesignReport,
                        sha256: digest.clone(),
                    },
                    pointer: format!("/designs/{index}"),
                });
            }
        }
    }
    if references.len() > MAX_SUBJECTS {
        return Err(LabError::Input(
            "combined snapshot subjects exceed eight".into(),
        ));
    }
    let input = project_graph(&first, &references, lab)?;
    let second = catalog
        .graph_export(&request.graph_root_id, request.graph_depth)
        .map_err(|error| LabError::Input(error.to_string()))?;
    if !second.digest_valid() {
        return Err(LabError::Integrity(
            "knowledge graph export digest mismatch".into(),
        ));
    }
    check_capture_revisions(first.graph_sha256(), second.graph_sha256())?;

    let document = SnapshotDocument {
        schema: SNAPSHOT_SCHEMA.into(),
        input,
        source_text_included: false,
        source_replay: "unavailable".into(),
    };
    let bytes = serde_json::to_vec(&document)?;
    if bytes.len() > MAX_SNAPSHOT_BYTES {
        return Err(LabError::Input("snapshot exceeds 4 MiB".into()));
    }
    let digest = sha256(&bytes);
    let was_present = match lab.artifact(&digest) {
        Ok(existing) => existing == bytes,
        Err(LabError::Missing) => false,
        Err(error) => return Err(error),
    };
    if lab.record_development_bytes(&bytes)? != digest || lab.artifact(&digest)? != bytes {
        return Err(LabError::Integrity(
            "stored snapshot did not match its raw SHA-256".into(),
        ));
    }
    let receipt = json!({
        "schema":SNAPSHOT_RECEIPT_SCHEMA,
        "snapshot_sha256":digest,
        "graph_root_id":document.input.graph_root_id,
        "graph_sha256":document.input.graph_sha256,
        "source_text_included":false,
        "source_replay":"unavailable"
    });
    Ok((receipt, !was_present))
}

fn persist_projection(lab: &Lab, value: &Value) -> QualityResult<String> {
    let bytes = serde_json::to_vec(value)?;
    if bytes.len() > MAX_ARTIFACT_BYTES {
        return Err(LabError::Input(
            "generated structural projection exceeds 2 MiB".into(),
        ));
    }
    let digest = sha256(&bytes);
    if lab.record_development_bytes(&bytes)? != digest || lab.artifact(&digest)? != bytes {
        return Err(LabError::Integrity(
            "generated projection failed its content digest".into(),
        ));
    }
    Ok(digest)
}

fn check_capture_revisions(before: &str, after: &str) -> QualityResult<()> {
    if before == after {
        Ok(())
    } else {
        Err(LabError::Conflict(
            "knowledge graph changed during snapshot capture".into(),
        ))
    }
}

fn remove_source_statements(value: &mut Value) {
    match value {
        Value::Object(object) => {
            object.remove("statement");
            for child in object.values_mut() {
                remove_source_statements(child);
            }
        }
        Value::Array(items) => items.iter_mut().for_each(remove_source_statements),
        _ => {}
    }
}

fn project_graph(
    export: &crate::literature::GraphExport,
    references: &[SubjectRef],
    lab: &Lab,
) -> QualityResult<StructuralInput> {
    let value = serde_json::to_value(export)?;
    let graph = &value["graph"];
    let mut nodes = Vec::new();
    for node in graph["nodes"]
        .as_array()
        .ok_or_else(|| LabError::Integrity("graph snapshot has no node array".into()))?
    {
        let current = &node["current"];
        let source = parse_optional::<EvidenceOrigin>(&current["source"])?;
        let source_identity_status = source.as_ref().map(|origin| {
            if matches!(origin, EvidenceOrigin::Operator) {
                SourceIdentityStatus::NotApplicable
            } else {
                SourceIdentityStatus::Unverified
            }
        });
        let epistemic_status = parse_optional::<EpistemicStatus>(&current["status"])?;
        let statement = current["statement"].as_str().unwrap_or("");
        nodes.push(GraphNodeFact {
            id: required_string(node, "id")?.into(),
            kind: serde_json::from_value(node["kind"].clone())
                .map_err(|_| LabError::Input("unknown graph entity kind".into()))?,
            revision_sha256: required_string(current, "sha256")?.into(),
            statement_sha256: (!statement.is_empty()).then(|| sha256(statement.as_bytes())),
            statement_bytes: statement.len() as u64,
            source,
            source_identity_status,
            epistemic_status,
        });
    }
    let mut edges = Vec::new();
    for edge in graph["edges"]
        .as_array()
        .ok_or_else(|| LabError::Integrity("graph snapshot has no edge array".into()))?
    {
        let relation = match edge["relation"].as_str() {
            Some("supports") => EvidenceRelation::Supports,
            Some("contradicts") => EvidenceRelation::Contradicts,
            Some(_) => EvidenceRelation::Other,
            None => return Err(LabError::Integrity("graph edge has no relation".into())),
        };
        edges.push(GraphEdgeFact {
            id: required_string(edge, "id")?.into(),
            origin: required_string(edge, "origin")?.into(),
            target: required_string(edge, "target")?.into(),
            relation,
        });
    }
    nodes.sort_by(|a, b| a.id.cmp(&b.id));
    edges.sort_by(|a, b| a.id.cmp(&b.id));
    if nodes.len() > evaluator::MAX_NODES || edges.len() > evaluator::MAX_EDGES {
        return Err(LabError::Input(
            "graph export exceeds fixed structural evaluator limits".into(),
        ));
    }
    let subjects = references
        .iter()
        .map(|reference| project_subject(reference, lab))
        .collect::<QualityResult<Vec<_>>>()?;
    Ok(StructuralInput {
        graph_root_id: required_string(graph, "root")?.into(),
        graph_sha256: required_string(&value, "graph_sha256")?.into(),
        graph_depth: graph["depth"].as_u64().unwrap_or_default() as u8,
        graph_truncated: graph["truncated"].as_bool().unwrap_or(true),
        nodes,
        edges,
        subjects,
        snapshot_digest_verified: true,
        graph_digest_verified: export.digest_valid(),
        // We retain locators and digests, never source text whose local-storage rights are unknown.
        source_replay_available: false,
    })
}

fn project_subject(reference: &SubjectRef, lab: &Lab) -> QualityResult<SubjectFacts> {
    let expected_artifact_kind = match reference.kind {
        SubjectKind::Hypothesis => ArtifactKind::HypothesisReport,
        SubjectKind::ExperimentDesign => ArtifactKind::ExperimentDesignReport,
    };
    if reference.artifact.kind != expected_artifact_kind {
        return Err(LabError::Input(
            "subject and artifact kinds do not match".into(),
        ));
    }
    let bytes = lab.artifact(&reference.artifact.sha256)?;
    if bytes.len() > MAX_ARTIFACT_BYTES || sha256(&bytes) != reference.artifact.sha256 {
        return Err(LabError::Integrity(
            "subject artifact size or digest mismatch".into(),
        ));
    }
    let value: Value = serde_json::from_slice(&bytes)
        .map_err(|_| LabError::Input("subject artifact is not valid JSON".into()))?;
    let subject = value
        .pointer(&reference.pointer)
        .ok_or_else(|| LabError::Input("subject JSON pointer does not resolve".into()))?;
    match reference.kind {
        SubjectKind::Hypothesis => project_hypothesis(reference, subject),
        SubjectKind::ExperimentDesign => project_design(reference, subject),
    }
}

fn project_hypothesis(reference: &SubjectRef, value: &Value) -> QualityResult<SubjectFacts> {
    let status = optional_epistemic(value.get("epistemic_status"))?;
    // Ranking.uncertainty is a prioritization heuristic, not a declared uncertainty estimate.
    let uncertainty = value
        .get("uncertainty_grade")
        .and_then(Value::as_u64)
        .map(|grade| grade.min(u64::from(u8::MAX)) as u8);
    Ok(SubjectFacts::Hypothesis {
        reference: reference.clone(),
        proposition_present: nonempty(value.get("proposition")),
        support_count: len(value.get("supports")),
        counterevidence_count: len(value.get("contradictory_evidence")),
        alternative_count: len(value.get("alternative_explanations")),
        assumption_count: len(value.get("assumptions")),
        limitation_count: len(value.get("limitations")),
        falsification_present: nonempty(value.get("falsification")),
        metric_count: len(value.get("metrics")),
        resources_present: value.get("resources").is_some_and(Value::is_object),
        uncertainty_grade: uncertainty,
        epistemic_status: status,
    })
}

fn project_design(reference: &SubjectRef, value: &Value) -> QualityResult<SubjectFacts> {
    // APO-04 stores a Draft{protocol}; callers may also pin the protocol directly.
    let protocol = value.get("protocol").unwrap_or(value);
    let controls = &protocol["controls"];
    let analysis = &protocol["analysis"];
    let manifest = &protocol["manifest"];
    let resources = &protocol["resources"];
    let cap = &resources["cap"];
    let reserve = &resources["verification_reserve"];
    let held_out_pin = manifest["held_out_commitment"]
        .as_str()
        .is_some_and(evaluator::valid_digest)
        && manifest["held_out_custody"] == "independent_custodian";
    Ok(SubjectFacts::ExperimentDesign {
        reference: reference.clone(),
        independent_variable_count: len(controls.get("independent_variables")),
        dependent_variable_present: nonempty(controls.get("dependent_variable")),
        control_count: len(controls.get("held_constant")),
        baseline_present: nonempty(controls.get("baseline_rationale")),
        confounder_analysis_present: len(controls.get("confounders")) > 0,
        development_and_test_separated: len(manifest.get("development_cases")) > 0 && held_out_pin,
        sample_size_basis_present: nonempty(analysis.get("sample_size_basis")),
        statistical_plan_present: nonempty(analysis.get("statistical_plan")),
        metric_count: usize::from(nonempty(analysis.get("primary")))
            + len(analysis.get("secondary")),
        falsification_present: nonempty(analysis.get("falsification")),
        termination_present: nonempty(analysis.get("termination")),
        resource_cap_attempts: number(cap.get("attempts")),
        search_attempts: number(resources.get("search_attempts")),
        verification_attempts: number(resources.get("verification_attempts")),
        oracle_attempts: number(resources.get("oracle_attempts")),
        reserve_attempts: number(reserve.get("attempts")),
        resource_cap_wall_ms: number(cap.get("wall_ms")),
        declared_cpu_ms_ceiling: number(resources.get("cpu_ms_ceiling")),
    })
}

fn parse_optional<T: for<'de> Deserialize<'de>>(value: &Value) -> QualityResult<Option<T>> {
    if value.is_null() {
        Ok(None)
    } else {
        serde_json::from_value(value.clone())
            .map(Some)
            .map_err(|_| LabError::Input("invalid closed enum or provenance record".into()))
    }
}

fn optional_epistemic(value: Option<&Value>) -> QualityResult<Option<EpistemicStatus>> {
    value
        .map(|value| {
            serde_json::from_value(value.clone())
                .map_err(|_| LabError::Input("invalid explicit epistemic status".into()))
        })
        .transpose()
}

fn required_string<'a>(value: &'a Value, key: &str) -> QualityResult<&'a str> {
    value
        .get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| LabError::Integrity("stored graph export is malformed".into()))
}

fn nonempty(value: Option<&Value>) -> bool {
    value
        .and_then(Value::as_str)
        .is_some_and(|text| !text.trim().is_empty())
}
fn len(value: Option<&Value>) -> usize {
    value.and_then(Value::as_array).map_or(0, Vec::len)
}
fn number(value: Option<&Value>) -> u64 {
    value.and_then(Value::as_u64).unwrap_or_default()
}

pub(crate) fn run(lab: &Lab, request: RunRequest) -> QualityResult<(Value, bool)> {
    if request.schema != RUN_SCHEMA
        || !evaluator::valid_digest(&request.benchmark_sha256)
        || !evaluator::valid_digest(&request.snapshot_sha256)
    {
        return Err(LabError::Input("invalid evaluation run request".into()));
    }
    let benchmark_sha256 = sha256(BENCHMARK);
    if request.benchmark_sha256 != benchmark_sha256 {
        return Err(LabError::Missing);
    }
    let raw = lab.artifact(&request.snapshot_sha256)?;
    if raw.len() > MAX_SNAPSHOT_BYTES || sha256(&raw) != request.snapshot_sha256 {
        return Err(LabError::Integrity("snapshot raw SHA-256 mismatch".into()));
    }
    let snapshot: SnapshotDocument = serde_json::from_slice(&raw)
        .map_err(|_| LabError::Integrity("stored snapshot schema is invalid".into()))?;
    if snapshot.schema != SNAPSHOT_SCHEMA
        || snapshot.source_text_included
        || snapshot.source_replay != "unavailable"
    {
        return Err(LabError::Integrity(
            "stored snapshot contract mismatch".into(),
        ));
    }
    let report = evaluator::evaluate(
        &snapshot.input,
        &benchmark_sha256,
        &request.snapshot_sha256,
        &serde_json::from_slice::<Benchmark>(BENCHMARK)?
            .dimension_rules
            .into_iter()
            .map(|rule| DimensionRulePin {
                dimension: rule.dimension,
                rule_id: rule.rule_id,
                version: rule.version,
            })
            .collect::<Vec<_>>(),
        EvaluatorIdentity {
            version: evaluator::EVALUATOR_VERSION.into(),
            executable_sha256: Some(lab.evaluator_executable_sha256().into()),
            build_attestation: "not_provided".into(),
        },
    )
    .map_err(|_| LabError::Input("snapshot is outside evaluator limits".into()))?;
    let report_bytes = serde_json::to_vec(&report)?;
    if report_bytes.len() > MAX_REPORT_BYTES {
        return Err(LabError::Storage("report exceeds output ceiling".into()));
    }
    let report_sha256 = sha256(&report_bytes);
    if lab.record_development_bytes(&report_bytes)? != report_sha256
        || lab.artifact(&report_sha256)? != report_bytes
    {
        return Err(LabError::Integrity(
            "stored report raw SHA-256 mismatch".into(),
        ));
    }
    let mut registry = lab.model_registry()?;
    let (sequence, inserted) = registry
        .index_quality_report(&report_sha256, &benchmark_sha256, &request.snapshot_sha256)
        .map_err(|_| LabError::Storage("quality report index write failed".into()))?;
    let envelope = ReportEnvelope {
        schema: REPORT_ENVELOPE_SCHEMA.into(),
        report_sha256,
        sequence,
        report,
    };
    Ok((serde_json::to_value(envelope)?, inserted))
}

pub(crate) fn reports(
    lab: &Lab,
    limit: usize,
    before_sequence: Option<i64>,
) -> QualityResult<Value> {
    if !(1..=MAX_PAGE).contains(&limit) || before_sequence.is_some_and(|value| value <= 0) {
        return Err(LabError::Input(
            "report page limit or cursor is invalid".into(),
        ));
    }
    let registry = lab.model_registry()?;
    let rows = registry
        .quality_report_page(before_sequence, limit + 1)
        .map_err(|_| LabError::Storage("quality report listing failed".into()))?;
    let truncated = rows.len() > limit;
    let mut summaries = Vec::with_capacity(rows.len().min(limit));
    for (sequence, digest) in rows.into_iter().take(limit) {
        let (envelope, _) = inspect_with_sequence(lab, &digest, Some(sequence))?;
        summaries.push(ReportSummary {
            report_sha256: envelope.report_sha256,
            sequence: envelope.sequence,
            benchmark_sha256: envelope.report.benchmark_sha256,
            snapshot_sha256: envelope.report.snapshot_sha256,
            execution_outcome: envelope.report.execution_outcome,
            dimensions: envelope
                .report
                .dimensions
                .into_iter()
                .map(|row| DimensionSummary {
                    dimension: row.dimension,
                    outcome: row.outcome,
                })
                .collect(),
        });
    }
    let next = if truncated {
        summaries.last().map(|value| value.sequence)
    } else {
        None
    };
    let response = ReportList {
        schema: "aether.quality-reports/1".into(),
        reports: summaries,
        truncated,
        next_before_sequence: next,
    };
    let bytes = serde_json::to_vec(&response)?;
    if bytes.len() > MAX_REPORT_LIST_BYTES {
        return Err(LabError::Storage(
            "report listing exceeds output ceiling".into(),
        ));
    }
    Ok(serde_json::from_slice(&bytes)?)
}

pub(crate) fn inspect(lab: &Lab, digest: &str) -> QualityResult<Value> {
    if !evaluator::valid_digest(digest) {
        return Err(LabError::Input(
            "report SHA-256 must be lowercase hex".into(),
        ));
    }
    let (envelope, _) = inspect_with_sequence(lab, digest, None)?;
    Ok(serde_json::to_value(envelope)?)
}

fn inspect_with_sequence(
    lab: &Lab,
    digest: &str,
    expected_sequence: Option<i64>,
) -> QualityResult<(ReportEnvelope, bool)> {
    let registry = lab.model_registry()?;
    let Some((sequence, benchmark_sha256, snapshot_sha256)) = registry
        .quality_report_sequence(digest)
        .map_err(|_| LabError::Storage("quality report lookup failed".into()))?
    else {
        return Err(LabError::Missing);
    };
    if expected_sequence.is_some_and(|expected| expected != sequence) {
        return Err(LabError::Integrity("report sequence index changed".into()));
    }
    let bytes = lab
        .artifact(digest)
        .map_err(|_| LabError::Integrity("indexed report blob is missing".into()))?;
    if bytes.len() > MAX_REPORT_BYTES || sha256(&bytes) != digest {
        return Err(LabError::Integrity("stored report digest mismatch".into()));
    }
    let report: Report = serde_json::from_slice(&bytes)
        .map_err(|_| LabError::Integrity("stored report schema is invalid".into()))?;
    if report.schema != evaluator::REPORT_SCHEMA
        || report.benchmark_sha256 != benchmark_sha256
        || report.snapshot_sha256 != snapshot_sha256
        || !valid_report(&report)
    {
        return Err(LabError::Integrity(
            "stored report contract mismatch".into(),
        ));
    }
    Ok((
        ReportEnvelope {
            schema: REPORT_ENVELOPE_SCHEMA.into(),
            report_sha256: digest.into(),
            sequence,
            report,
        },
        false,
    ))
}

fn valid_report(report: &Report) -> bool {
    if report.execution_outcome != "completed"
        || report.semantic_assessment != "not_evaluated"
        || report.independent_scientific_validation != "not_performed"
        || report.evaluator.version != evaluator::EVALUATOR_VERSION
        || report.evaluator.build_attestation != "not_provided"
        || report.dimensions.len() != 8
        || report.dimension_rule_pins.len() != 8
        || report
            .dimensions
            .iter()
            .map(|dimension| dimension.dimension)
            .collect::<Vec<_>>()
            != DimensionId::ALL
        || report
            .dimension_rule_pins
            .iter()
            .map(|pin| pin.dimension)
            .collect::<Vec<_>>()
            != DimensionId::ALL
    {
        return false;
    }
    report
        .dimensions
        .iter()
        .zip(&report.dimension_rule_pins)
        .all(|(dimension, pin)| {
            let rolled_up = if dimension.checks.is_empty() {
                return false;
            } else if dimension
                .checks
                .iter()
                .any(|check| check.outcome == Outcome::Fail)
            {
                Outcome::Fail
            } else if dimension
                .checks
                .iter()
                .all(|check| check.outcome == Outcome::Pass)
            {
                Outcome::Pass
            } else {
                Outcome::Unavailable
            };
            dimension.outcome == rolled_up
                && dimension
                    .checks
                    .iter()
                    .all(|check| check.rule_id == pin.rule_id)
                && pin.version == "1"
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Registry;
    use rusqlite::Connection;

    #[test]
    fn benchmark_pins_match_the_engine_and_golden_bytes_are_exact() {
        let spec: Benchmark = serde_json::from_slice(BENCHMARK).unwrap();
        validate_benchmark(&spec).unwrap();
        let detail: Value = serde_json::from_slice(BENCHMARK_DETAIL_FIXTURE).unwrap();
        let spec: Value = serde_json::from_slice(BENCHMARK).unwrap();
        assert_eq!(detail["benchmark"], spec);
        let list: Value = serde_json::from_slice(BENCHMARK_LIST_FIXTURE).unwrap();
        assert_eq!(
            list["benchmarks"][0]["dimensions"]
                .as_array()
                .unwrap()
                .len(),
            8
        );
        assert_eq!(sha256(BENCHMARK), sha256(BENCHMARK));
    }

    #[test]
    fn report_sequence_is_idempotent_and_vacuum_stable() {
        let mut registry = Registry(Connection::open_in_memory().unwrap());
        registry.0.execute_batch("CREATE TABLE IF NOT EXISTS model_registry(kind TEXT NOT NULL,key TEXT NOT NULL,sha256 TEXT NOT NULL,record TEXT NOT NULL,PRIMARY KEY(kind,key,sha256)) STRICT; CREATE TABLE IF NOT EXISTS quality_report_sequence(sequence INTEGER PRIMARY KEY AUTOINCREMENT,report_sha256 TEXT NOT NULL UNIQUE,benchmark_sha256 TEXT NOT NULL,snapshot_sha256 TEXT NOT NULL) STRICT;").unwrap();
        let one = "1".repeat(64);
        let two = "2".repeat(64);
        let three = "3".repeat(64);
        assert_eq!(
            registry.index_quality_report(&one, &two, &three).unwrap(),
            (1, true)
        );
        assert_eq!(
            registry.index_quality_report(&one, &two, &three).unwrap(),
            (1, false)
        );
        assert_eq!(
            registry
                .index_quality_report(&"4".repeat(64), &two, &three)
                .unwrap(),
            (2, true)
        );
        registry.0.execute_batch("VACUUM").unwrap();
        assert_eq!(
            registry.quality_report_page(None, 4).unwrap(),
            vec![(2, "4".repeat(64)), (1, one)]
        );
    }

    #[test]
    fn graph_revision_must_stay_pinned_during_capture() {
        assert!(check_capture_revisions(&"a".repeat(64), &"a".repeat(64)).is_ok());
        assert!(matches!(
            check_capture_revisions(&"a".repeat(64), &"b".repeat(64)),
            Err(LabError::Conflict(_))
        ));
    }

    #[test]
    fn report_golden_bytes_round_trip_and_structure_rollup_is_checked() {
        let bytes = include_bytes!("../../../../contracts/apo11/fixtures/report-envelope.json");
        let envelope: ReportEnvelope = serde_json::from_slice(bytes).unwrap();
        assert_eq!(serde_json::to_vec(&envelope).unwrap(), bytes);
        assert_eq!(
            sha256(&serde_json::to_vec(&envelope.report).unwrap()),
            envelope.report_sha256
        );
        assert!(valid_report(&envelope.report));
    }
}
