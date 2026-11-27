use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::PathBuf,
    process::Command,
    sync::atomic::{AtomicBool, AtomicU64, Ordering},
};

use aether_runner::sha256;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use super::{Registry, skills, workflows};
use crate::{Result, lab::Lab};

const SPEC_ID: &str = "aether-workflow-contract";
const SPEC_VERSION: &str = "1.0.0";
const REPORT_SCHEMA: &str = "aether.workflow-benchmark-report/1";
const WORKER_SCHEMA: &str = "aether.workflow-benchmark-worker/1";
const RUNNER_VERSION: &str = "aether-workflow-evaluator/1.0.0";
const WORKER_ARGUMENT: &str = "--workflow-benchmark-fixture-runner";
const ROOT_ALIAS: &str = "$dae_hybrid_archive_root";
const EMPTY_ROOT_ALIAS: &str = "$apo10_empty_evidence_root";
const MAX_WORKER_OUTPUT: usize = 8 * 1024 * 1024;
static TEMP_SEQUENCE: AtomicU64 = AtomicU64::new(0);

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct FixtureSpec {
    id: String,
    workflow_id: String,
    workflow_version: String,
    input: Value,
    expected_outcome: String,
    expected_failure: Option<String>,
    expected_order: Vec<String>,
    expected_node_states: BTreeMap<String, String>,
    operation_limit: Option<u8>,
    cancel_before_start: bool,
    assertion: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct BenchmarkSpec {
    schema: String,
    id: String,
    version: String,
    scope: String,
    execution_policy: String,
    fixture_source: String,
    fixtures: Vec<FixtureSpec>,
    adversarial_fixtures: Vec<String>,
    system_metrics: Vec<String>,
    unavailable_scientific_metrics: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct FixtureResult {
    fixture_id: String,
    fixture_sha256: String,
    resolved_input_sha256: String,
    workflow_sha256: String,
    receipt_sha256: String,
    outcome: String,
    failure: Option<String>,
    execution_order: Vec<String>,
    node_states: BTreeMap<String, String>,
    consumed_budget: Value,
    budget_compliant: bool,
    input_contract_valid: bool,
    expected_input_valid: bool,
    output_contract_valid: Option<bool>,
    kernel_evidence_unverified: bool,
    assertion_passed: bool,
    passed: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct WorkerReport {
    schema: String,
    runner_version: String,
    executable_sha256: String,
    specification_sha256: String,
    fixture_digests: Vec<String>,
    fixtures: Vec<FixtureResult>,
    adversarial_fixtures: Vec<Value>,
    system_metrics: Value,
    scientific_quality_metrics: Value,
    failures: Vec<Value>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct WorkerEnvelope {
    worker_sha256: String,
    worker: WorkerReport,
}

struct TempLab {
    root: PathBuf,
    lab: Lab,
}

impl TempLab {
    fn new() -> Result<Self> {
        let root = std::env::temp_dir().join(format!(
            "aether-apo10-{}-{}",
            std::process::id(),
            TEMP_SEQUENCE.fetch_add(1, Ordering::Relaxed)
        ));
        let lab = Lab::open(root.clone())?;
        Ok(Self { root, lab })
    }
}

impl Drop for TempLab {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn fixture(
    id: &str,
    workflow_id: &str,
    input: Value,
    expected: (&str, Option<&str>),
    topology: (&[&str], &[(&str, &str)]),
    limits: (Option<u8>, bool),
    assertion: &str,
) -> FixtureSpec {
    let (expected_order, expected_states) = topology;
    let (operation_limit, cancel_before_start) = limits;
    FixtureSpec {
        id: id.into(),
        workflow_id: workflow_id.into(),
        workflow_version: "1.0.0".into(),
        input,
        expected_outcome: expected.0.into(),
        expected_failure: expected.1.map(str::to_owned),
        expected_order: expected_order.iter().map(|value| (*value).into()).collect(),
        expected_node_states: expected_states
            .iter()
            .map(|(node, state)| ((*node).into(), (*state).into()))
            .collect(),
        operation_limit,
        cancel_before_start,
        assertion: assertion.into(),
    }
}

fn specification() -> BenchmarkSpec {
    let root = json!(ROOT_ALIAS);
    let empty_root = json!(EMPTY_ROOT_ALIAS);
    let regular = json!({
        "root_id":root,
        "question":"CAVEAT",
        "source_ids":["10.21203/rs.3.rs-11035327/v1"]
    });
    BenchmarkSpec {
        schema: "aether.workflow-benchmark-spec/1".into(),
        id: SPEC_ID.into(),
        version: SPEC_VERSION.into(),
        scope: "controlled local development fixtures; no scientific discovery claim".into(),
        execution_policy: "sequential through the existing APO-09 kernel; no retries".into(),
        fixture_source: "bundled citation catalog and pinned DAE-04 development archive".into(),
        fixtures: vec![
            fixture(
                "literature_hypothesis",
                "literature-hypothesis",
                regular.clone(),
                ("completed", None),
                (
                    &["literature", "hypothesis"],
                    &[("literature", "completed"), ("hypothesis", "completed")],
                ),
                (None, false),
                "both pinned native operations complete in declared order",
            ),
            fixture(
                "hypothesis_design_conflicting_archive_context",
                "hypothesis-design",
                json!({"root_id":root}),
                ("completed", None),
                (
                    &["criticism", "design"],
                    &[("criticism", "completed"), ("design", "completed")],
                ),
                (None, false),
                "both pinned operations complete and expose advisory proposals over the mixed-result development archive",
            ),
            fixture(
                "complete_three_stage",
                "literature-hypothesis-design",
                regular,
                ("completed", None),
                (
                    &["literature", "hypothesis", "design"],
                    &[
                        ("literature", "completed"),
                        ("hypothesis", "completed"),
                        ("design", "completed"),
                    ],
                ),
                (None, false),
                "all three pinned native operations complete in declared order",
            ),
            fixture(
                "missing_literature_match",
                "literature-hypothesis",
                json!({"root_id":root,"question":"CAVEAT","source_ids":["10.9999/aether-apo10-absent"]}),
                ("completed", None),
                (&["literature", "hypothesis"], &[]),
                (None, false),
                "empty literature matches remain explicit; graph proposals remain unverified",
            ),
            fixture(
                "invalid_workflow_input",
                "literature-hypothesis",
                json!({"root_id":root,"question":17,"source_ids":[]}),
                ("failed", Some("invalid_input")),
                (&[], &[]),
                (None, false),
                "wrongly typed workflow input is rejected before any kernel operation",
            ),
            fixture(
                "operation_failure_skips_dependent",
                "literature-hypothesis",
                json!({"root_id":root,"question":"\n","source_ids":[]}),
                ("failed", Some("skill_failed")),
                (
                    &["literature"],
                    &[("literature", "failed"), ("hypothesis", "skipped")],
                ),
                (None, false),
                "kernel operation failure is recorded and its dependent node is skipped",
            ),
            fixture(
                "missing_graph_root_abstains",
                "hypothesis-design",
                json!({"root_id":empty_root}),
                ("abstained", Some("abstained")),
                (
                    &["criticism"],
                    &[("criticism", "abstained"), ("design", "skipped")],
                ),
                (None, false),
                "missing local evidence yields abstention and skips dependent design",
            ),
            fixture(
                "aggregate_budget_exhaustion",
                "literature-hypothesis",
                json!({"root_id":root,"question":"CAVEAT","source_ids":[]}),
                ("partially_completed", Some("budget_exceeded")),
                (
                    &["literature"],
                    &[("literature", "completed"), ("hypothesis", "skipped")],
                ),
                (Some(1), false),
                "the workflow aggregate operation cap is respected",
            ),
            fixture(
                "cancel_before_start",
                "hypothesis-design",
                json!({"root_id":root}),
                ("cancelled", Some("cancelled")),
                (&[], &[]),
                (None, true),
                "pre-start cooperative cancellation prevents kernel operations",
            ),
        ],
        adversarial_fixtures: vec![
            "workflow-definition-digest-mutation".into(),
            "workflow-and-skill-pin-tampering".into(),
            "permission-escalation-field-injection".into(),
            "missing-workflow-dependency".into(),
            "malformed-evidence-reference".into(),
            "workflow-receipt-artifact-tampering".into(),
        ],
        system_metrics: vec![
            "fixture outcome and execution-order correctness".into(),
            "typed input/output contract enforcement".into(),
            "aggregate budget compliance".into(),
            "failure propagation and dependent-node skipping".into(),
            "receipt integrity, fresh-process replay and evidence status preservation".into(),
        ],
        unavailable_scientific_metrics: vec![
            "scientific validity".into(),
            "novelty".into(),
            "hypothesis truth".into(),
            "experiment quality".into(),
            "discovery rate".into(),
        ],
    }
}

fn canonical<T: Serialize>(value: &T) -> Result<Vec<u8>> {
    Ok(serde_json::to_vec(value)?)
}

fn digest<T: Serialize>(value: &T) -> Result<String> {
    Ok(sha256(&canonical(value)?))
}

fn executable_identity() -> Result<String> {
    Ok(sha256(&fs::read(std::env::current_exe()?)?))
}

fn resolve_input(input: &Value, root: &str, empty_root: &str) -> Value {
    match input {
        Value::String(value) if value == ROOT_ALIAS => json!(root),
        Value::String(value) if value == EMPTY_ROOT_ALIAS => json!(empty_root),
        Value::Array(values) => Value::Array(
            values
                .iter()
                .map(|value| resolve_input(value, root, empty_root))
                .collect(),
        ),
        Value::Object(values) => Value::Object(
            values
                .iter()
                .map(|(key, value)| (key.clone(), resolve_input(value, root, empty_root)))
                .collect(),
        ),
        value => value.clone(),
    }
}

fn state_map(receipt: &Value) -> BTreeMap<String, String> {
    receipt["nodes"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|node| {
            Some((
                node["node_id"].as_str()?.to_owned(),
                node["state"].as_str()?.to_owned(),
            ))
        })
        .collect()
}

fn budget_compliant(receipt: &Value) -> bool {
    ["operations", "input_bytes", "output_bytes"]
        .into_iter()
        .all(|field| {
            receipt["consumed_budget"][field]
                .as_u64()
                .zip(receipt["requested_budget"][field].as_u64())
                .is_some_and(|(used, maximum)| used <= maximum)
        })
}

fn kernel_receipts_unverified(lab: &Lab, receipt: &Value) -> Result<bool> {
    for node in receipt["nodes"].as_array().into_iter().flatten() {
        let Some(digest) = node["execution_receipt_sha256"].as_str() else {
            continue;
        };
        let kernel = skills::execution_receipt(lab, digest)?;
        if kernel["evidence_status"] != "unverified" {
            return Ok(false);
        }
    }
    Ok(receipt["evidence_status"] == "unverified")
}

fn evaluate_fixture(
    lab: &Lab,
    root: &str,
    empty_root: &str,
    case: &FixtureSpec,
) -> Result<FixtureResult> {
    let input = resolve_input(&case.input, root, empty_root);
    let input_sha256 = sha256(&canonical(&input)?);
    let cancel = AtomicBool::new(case.cancel_before_start);
    let result = workflows::execute_benchmark_case(
        lab,
        &case.workflow_id,
        &case.workflow_version,
        input.clone(),
        case.operation_limit,
        &cancel,
    )?;
    let receipt = &result["receipt"];
    let output = &result["output"];
    let outcome = receipt["outcome"].as_str().unwrap_or("invalid").to_owned();
    let failure = receipt["failure"].as_str().map(str::to_owned);
    let order = receipt["execution_order"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .map(str::to_owned)
        .collect::<Vec<_>>();
    let states = state_map(receipt);
    let workflow_sha256 = result["benchmark_workflow_sha256"]
        .as_str()
        .ok_or("workflow fixture has no digest")?
        .to_owned();
    let input_contract_valid = result["benchmark_input_valid"] == true;
    let expected_input_valid = case.id != "invalid_workflow_input";
    let output_contract_valid = if receipt["output_sha256"].is_string() {
        Some(result["benchmark_output_valid"] == true)
    } else {
        None
    };
    let expected_state_match = case
        .expected_node_states
        .iter()
        .all(|(node, state)| states.get(node) == Some(state));
    let output_assertion = match case.id.as_str() {
        "missing_literature_match" => output["values"]["literature"]["papers"]
            .as_array()
            .is_some_and(Vec::is_empty),
        "hypothesis_design_conflicting_archive_context" => {
            output["values"]["criticism"]["proposals"]
                .as_array()
                .is_some_and(|rows| {
                    !rows.is_empty() && rows.iter().all(|row| row["status"] == "unverified")
                })
        }
        _ => true,
    };
    let assertion_passed = output_assertion;
    let kernel_unverified = kernel_receipts_unverified(lab, receipt)?;
    let passed = outcome == case.expected_outcome
        && failure.as_deref() == case.expected_failure.as_deref()
        && order == case.expected_order
        && expected_state_match
        && input_contract_valid == expected_input_valid
        && output_contract_valid != Some(false)
        && budget_compliant(receipt)
        && kernel_unverified
        && assertion_passed;
    Ok(FixtureResult {
        fixture_id: case.id.clone(),
        fixture_sha256: digest(case)?,
        resolved_input_sha256: input_sha256,
        workflow_sha256,
        receipt_sha256: result["workflow_execution_sha256"]
            .as_str()
            .ok_or("workflow fixture returned no receipt digest")?
            .to_owned(),
        outcome,
        failure,
        execution_order: order,
        node_states: states,
        consumed_budget: receipt["consumed_budget"].clone(),
        budget_compliant: budget_compliant(receipt),
        input_contract_valid,
        expected_input_valid,
        output_contract_valid,
        kernel_evidence_unverified: kernel_unverified,
        assertion_passed,
        passed,
    })
}

fn mutate_receipt(lab: &Lab, receipt_sha256: &str) -> Result<bool> {
    let registry = lab.model_registry()?;
    registry.0.execute(
        "UPDATE model_registry SET record='{}' WHERE kind='workflow-execution' AND sha256=?1",
        [receipt_sha256],
    )?;
    Ok(workflows::executions(lab).is_err())
}

fn worker_report() -> Result<WorkerReport> {
    let temp = TempLab::new()?;
    let lab = &temp.lab;
    let archive = lab.catalog()?.index_hypothesis_archive()?;
    let root = archive["root"]
        .as_str()
        .ok_or("development archive has no root")?;
    let empty_root = lab
        .catalog()?
        .graph_apply(serde_json::from_value(json!({
            "nodes":[{"kind":"experiment","key":"apo10-no-evidence-root","label":"APO-10 abstention fixture"}]
        }))?)?["node_ids"][0]
        .as_str()
        .ok_or("abstention fixture has no graph root")?
        .to_owned();
    workflows::authorize_benchmark_fixture_lab(lab)?;
    let spec = specification();
    let spec_sha256 = digest(&spec)?;
    let fixtures = spec
        .fixtures
        .iter()
        .map(|case| evaluate_fixture(lab, root, &empty_root, case))
        .collect::<Result<Vec<_>>>()?;

    let mut adversarial = workflows::adversarial_definition_fixtures(lab)?;
    let malformed = skills::malformed_evidence_fixture(lab)?;
    adversarial.push(json!({
        "id":"malformed_evidence_reference",
        "passed":malformed["state"] == "denied"
            && malformed["failure"] == "invalid_evidence"
            && malformed["evidence_status"] == "unverified",
        "observed":malformed
    }));

    let stored = workflows::executions(lab)?;
    let stored_receipts = stored["executions"]
        .as_array()
        .ok_or("workflow receipt listing malformed")?;
    let receipt_sha_set = stored_receipts
        .iter()
        .filter_map(|entry| entry["workflow_execution_sha256"].as_str())
        .collect::<BTreeSet<_>>();
    let receipt_integrity_passed = fixtures
        .iter()
        .all(|fixture| receipt_sha_set.contains(fixture.receipt_sha256.as_str()));
    let tamper_passed = fixtures
        .first()
        .is_some_and(|fixture| mutate_receipt(lab, &fixture.receipt_sha256).unwrap_or(false));
    adversarial.push(json!({
        "id":"workflow_receipt_artifact_tampering",
        "passed":tamper_passed,
        "expected":"receipt listing rejects modified registry bytes"
    }));

    let passed = fixtures.iter().filter(|fixture| fixture.passed).count();
    let output_cases = fixtures
        .iter()
        .filter(|fixture| fixture.output_contract_valid.is_some())
        .collect::<Vec<_>>();
    let input_cases = fixtures
        .iter()
        .filter(|fixture| fixture.input_contract_valid == fixture.expected_input_valid)
        .count();
    let budget_cases = fixtures
        .iter()
        .filter(|fixture| fixture.budget_compliant)
        .count();
    let unverified_cases = fixtures
        .iter()
        .filter(|fixture| fixture.kernel_evidence_unverified)
        .count();
    let system_metrics = json!({
        "workflow_fixtures":fixtures.len(),
        "workflow_fixtures_passed":passed,
        "expected_outcome_and_order_correct":fixtures.iter().filter(|fixture| {
            fixture.outcome == spec.fixtures.iter().find(|case| case.id == fixture.fixture_id).unwrap().expected_outcome
                && fixture.execution_order == spec.fixtures.iter().find(|case| case.id == fixture.fixture_id).unwrap().expected_order
        }).count(),
        "typed_input_contracts_passed":input_cases,
        "typed_input_contracts_total":fixtures.len(),
        "typed_output_contracts_passed":output_cases.iter().filter(|fixture| fixture.output_contract_valid == Some(true)).count(),
        "typed_output_contracts_total":output_cases.len(),
        "aggregate_budgets_compliant":budget_cases,
        "aggregate_budget_cases":fixtures.len(),
        "kernel_evidence_unverified":unverified_cases,
        "evidence_cases":fixtures.len(),
        "workflow_receipts_integrity_checked":stored_receipts.len(),
        "fixture_receipts_found_and_valid":receipt_integrity_passed,
        "downstream_failure_skip_observed":fixtures.iter().any(|fixture| fixture.fixture_id == "operation_failure_skips_dependent"
            && fixture.node_states.get("hypothesis").is_some_and(|state| state == "skipped")),
        "adversarial_fixtures_passed":adversarial.iter().filter(|item| item["passed"] == true).count(),
        "adversarial_fixtures_total":adversarial.len()
    });
    let failures = fixtures
        .iter()
        .filter(|fixture| !fixture.passed)
        .map(|fixture| json!({"fixture_id":fixture.fixture_id,"expected":"pass","observed_outcome":fixture.outcome,"failure":fixture.failure}))
        .chain((!receipt_integrity_passed).then_some(json!({"workflow_receipt_integrity":"failed"})))
        .chain(
            adversarial
                .iter()
                .filter(|fixture| fixture["passed"] != true)
                .map(|fixture| json!({"adversarial_fixture":fixture["id"],"expected":"rejected","observed":fixture})),
        )
        .collect::<Vec<_>>();
    let mut all_fixture_digests = spec
        .fixtures
        .iter()
        .map(digest)
        .collect::<Result<Vec<_>>>()?;
    all_fixture_digests.extend(adversarial.iter().map(digest).collect::<Result<Vec<_>>>()?);
    Ok(WorkerReport {
        schema: WORKER_SCHEMA.into(),
        runner_version: RUNNER_VERSION.into(),
        executable_sha256: executable_identity()?,
        specification_sha256: spec_sha256,
        fixture_digests: all_fixture_digests,
        fixtures,
        adversarial_fixtures: adversarial,
        system_metrics,
        scientific_quality_metrics: json!({
            "status":"unavailable_not_evaluated",
            "scientific_validity":Value::Null,
            "novelty":Value::Null,
            "hypothesis_truth":Value::Null,
            "experiment_quality":Value::Null,
            "discovery_rate":Value::Null
        }),
        failures,
    })
}

pub(crate) fn worker() -> Result<Value> {
    let report = worker_report()?;
    let envelope = WorkerEnvelope {
        worker_sha256: digest(&report)?,
        worker: report,
    };
    Ok(serde_json::to_value(envelope)?)
}

fn run_worker_process() -> Result<WorkerEnvelope> {
    let output = Command::new(std::env::current_exe()?)
        .arg(WORKER_ARGUMENT)
        .output()?;
    if !output.status.success() {
        return Err(format!(
            "pinned benchmark worker failed: {}",
            String::from_utf8_lossy(&output.stderr)
                .chars()
                .take(512)
                .collect::<String>()
        )
        .into());
    }
    if output.stdout.len() > MAX_WORKER_OUTPUT {
        return Err("benchmark worker output exceeds its fixed 8 MiB limit".into());
    }
    let envelope: WorkerEnvelope = serde_json::from_slice(&output.stdout)?;
    if digest(&envelope.worker)? != envelope.worker_sha256
        || envelope.worker.schema != WORKER_SCHEMA
        || envelope.worker.runner_version != RUNNER_VERSION
    {
        return Err("fresh-process worker report failed its integrity check".into());
    }
    Ok(envelope)
}

fn register_spec(lab: &Lab, spec: &BenchmarkSpec) -> Result<String> {
    let bytes = canonical(spec)?;
    let sha = sha256(&bytes);
    if lab.record_development_bytes(&bytes)? != sha {
        return Err("benchmark specification artifact digest mismatch".into());
    }
    let key = format!("{SPEC_ID}@{SPEC_VERSION}");
    let registry = lab.model_registry()?;
    let mut statement = registry.0.prepare(
        "SELECT sha256 FROM model_registry WHERE kind='workflow-benchmark-spec' AND key=?1",
    )?;
    let existing = statement
        .query_map([&key], |row| row.get::<_, String>(0))?
        .collect::<std::result::Result<Vec<_>, _>>()?;
    if existing.iter().any(|digest| digest != &sha) {
        return Err("benchmark specification versions are immutable; publish a new version".into());
    }
    Registry::save(&registry.0, "workflow-benchmark-spec", &key, &sha, spec)?;
    Ok(sha)
}

pub(crate) fn specification_record(lab: &Lab) -> Result<Value> {
    let spec = specification();
    let digest = register_spec(lab, &spec)?;
    Ok(json!({"specification_sha256":digest,"specification":spec}))
}

pub(crate) fn list(lab: &Lab) -> Result<Value> {
    let spec = specification_record(lab)?;
    let registry = lab.model_registry()?;
    let mut statement = registry.0.prepare(
        "SELECT sha256, record FROM model_registry WHERE kind='workflow-benchmark-report' ORDER BY rowid DESC LIMIT 33",
    )?;
    let mut rows = statement.query([])?;
    let mut reports = Vec::new();
    let mut truncated = false;
    while let Some(row) = rows.next()? {
        if reports.len() == 32 {
            truncated = true;
            break;
        }
        let sha: String = row.get(0)?;
        let raw: String = row.get(1)?;
        if sha256(raw.as_bytes()) != sha || lab.artifact(&sha)? != raw.as_bytes() {
            return Err("workflow benchmark report failed its content integrity check".into());
        }
        reports.push(json!({"report_sha256":sha,"report":serde_json::from_str::<Value>(&raw)?}));
    }
    Ok(json!({
        "schema":"aether.workflow-benchmarks/1",
        "specification":spec,
        "reports":reports,
        "truncated":truncated
    }))
}

pub(crate) fn inspect(lab: &Lab, digest: &str) -> Result<Value> {
    let registry = lab.model_registry()?;
    let raw: String = registry.0.query_row(
        "SELECT record FROM model_registry WHERE kind='workflow-benchmark-report' AND sha256=?1",
        [digest],
        |row| row.get(0),
    )?;
    if sha256(raw.as_bytes()) != digest || lab.artifact(digest)? != raw.as_bytes() {
        return Err("workflow benchmark report failed its content integrity check".into());
    }
    Ok(json!({"report_sha256":digest,"report":serde_json::from_str::<Value>(&raw)?}))
}

pub(crate) fn run(lab: &Lab) -> Result<Value> {
    let spec = specification();
    let spec_sha256 = register_spec(lab, &spec)?;
    let first = run_worker_process()?;
    let second = run_worker_process()?;
    if first.worker.executable_sha256 != second.worker.executable_sha256
        || first.worker.specification_sha256 != spec_sha256
        || second.worker.specification_sha256 != spec_sha256
    {
        return Err("fresh-process benchmark configuration identities do not match".into());
    }
    let first_worker_sha = first.worker_sha256;
    let second_worker_sha = second.worker_sha256;
    let first_receipts = first
        .worker
        .fixtures
        .iter()
        .map(|fixture| fixture.receipt_sha256.clone())
        .collect::<Vec<_>>();
    let second_receipts = second
        .worker
        .fixtures
        .iter()
        .map(|fixture| fixture.receipt_sha256.clone())
        .collect::<Vec<_>>();
    let receipt_sets_match = first_receipts == second_receipts;
    let workers_match = first_worker_sha == second_worker_sha && receipt_sets_match;
    let executable_sha256 = first.worker.executable_sha256.clone();
    let runner_config = json!({
        "runner_version":RUNNER_VERSION,
        "executable_sha256":executable_sha256,
        "specification_sha256":spec_sha256,
        "fresh_process_repetitions":2,
        "execution":"same pinned AETHER executable; fixed private mode; no shell"
    });
    let configuration_sha256 = digest(&runner_config)?;
    let all_checks_passed = first.worker.failures.is_empty()
        && first
            .worker
            .adversarial_fixtures
            .iter()
            .all(|check| check["passed"] == true)
        && first.worker.fixtures.iter().all(|fixture| fixture.passed)
        && receipt_sets_match;
    let report = json!({
        "schema":REPORT_SCHEMA,
        "benchmark":{"id":SPEC_ID,"version":SPEC_VERSION,"specification_sha256":spec_sha256},
        "configuration":{"sha256":configuration_sha256,"identity":runner_config},
        "fixture_count":spec.fixtures.len(),
        "adversarial_fixture_count":first.worker.adversarial_fixtures.len(),
        "fixture_digests":first.worker.fixture_digests,
        "system_metrics":first.worker.system_metrics,
        "scientific_quality_metrics":first.worker.scientific_quality_metrics,
        "fixtures":first.worker.fixtures,
        "adversarial_fixtures":first.worker.adversarial_fixtures,
        "fresh_process_replay":{
            "process_runs":2,
            "worker_report_sha256s":[first_worker_sha,second_worker_sha],
            "workflow_receipt_sha256s":[first_receipts,second_receipts],
            "worker_reports_match":workers_match,
            "workflow_receipt_digests_match":receipt_sets_match
        },
        "passed":all_checks_passed && workers_match,
        "failures":first.worker.failures,
        "limitations":[
            "Synthetic fixture approvals exist only inside each disposable evaluation database.",
            "Evidence remains unverified; benchmark completion is not research correctness.",
            "No scientific quality, novelty, hypothesis truth, or discovery-rate score is measured.",
            "Independent human review and a separately custodied held-out evaluation remain required.",
            "Operations are synchronous; cancellation remains cooperative and has no hard preemption or OS isolation claim."
        ]
    });
    let bytes = canonical(&report)?;
    let report_sha256 = sha256(&bytes);
    if lab.record_development_bytes(&bytes)? != report_sha256 {
        return Err("workflow benchmark report artifact digest mismatch".into());
    }
    let registry = lab.model_registry()?;
    Registry::save(
        &registry.0,
        "workflow-benchmark-report",
        &report_sha256,
        &report_sha256,
        &report,
    )?;
    Ok(json!({"report_sha256":report_sha256,"report":report}))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn specification_is_versioned_with_fixed_scenarios_and_metrics() {
        let spec = specification();
        assert_eq!(spec.version, "1.0.0");
        assert_eq!(spec.fixtures.len(), 9);
        assert!(spec.adversarial_fixtures.len() >= 5);
        assert!(!spec.unavailable_scientific_metrics.is_empty());
        assert_eq!(digest(&spec).unwrap(), digest(&spec).unwrap());
    }
}
