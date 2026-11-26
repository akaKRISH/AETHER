use std::{
    collections::{BTreeMap, BTreeSet},
    sync::atomic::{AtomicBool, Ordering},
};

use aether_runner::sha256;
use rusqlite::OptionalExtension;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use super::{
    Registry,
    skills::{self, JsonShape, WorkflowSkillContract},
};
use crate::{
    Result,
    lab::{Lab, valid_digest, valid_id},
};

const DEFINITION_SCHEMA: &str = "aether.research-workflow/1";
const RECORD_SCHEMA: &str = "aether.workflow-registry-record/1";
const APPROVAL_SCHEMA: &str = "aether.workflow-approval/1";
const REVOCATION_SCHEMA: &str = "aether.workflow-revocation/1";
const EXECUTION_SCHEMA: &str = "aether.workflow-execution/1";
const EXECUTOR_VERSION: &str = "aether-workflow-orchestrator/1.0.0";
const MAX_WORKFLOWS: usize = 64;
const MAX_NODES: usize = 8;
const MAX_DEPTH: u8 = 8;
const MAX_INPUT_BYTES: u64 = 64 * 1024;
const MAX_OUTPUT_BYTES: u64 = 2 * 1024 * 1024;
const MAX_NODE_OUTPUT_BYTES: u64 = 256 * 1024;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WorkflowPin {
    pub id: String,
    pub version: String,
    pub sha256: String,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WorkflowBudget {
    pub operations: u8,
    pub input_bytes: u64,
    pub output_bytes: u64,
    pub dependency_depth: u8,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WorkflowRequest {
    workflow: WorkflowPin,
    input: Value,
    budget: WorkflowBudget,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "source", rename_all = "snake_case", deny_unknown_fields)]
enum Binding {
    WorkflowInput { path: Vec<String> },
    NodeOutput { node_id: String, path: Vec<String> },
    Literal { value: Value },
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct WorkflowNode {
    id: String,
    skill: WorkflowPin,
    depends_on: Vec<String>,
    inputs: BTreeMap<String, Binding>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct WorkflowDefinition {
    schema: String,
    id: String,
    version: String,
    description: String,
    input_schema: JsonShape,
    output_schema: JsonShape,
    nodes: Vec<WorkflowNode>,
    outputs: BTreeMap<String, Binding>,
    maximum_budget: WorkflowBudget,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct WorkflowRecord {
    schema: String,
    definition: WorkflowDefinition,
    sha256: String,
    artifact_sha256: String,
    approval_receipt_sha256: Option<String>,
    revocation_receipt_sha256: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct SkillApprovalPin {
    id: String,
    version: String,
    sha256: String,
    approval_receipt_sha256: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct ApprovalReceipt {
    schema: String,
    workflow_sha256: String,
    skill_approvals: Vec<SkillApprovalPin>,
    action: String,
    reviewer: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct RevocationReceipt {
    schema: String,
    workflow_sha256: String,
    approval_receipt_sha256: String,
    action: String,
    reviewer: String,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
enum Outcome {
    Completed,
    PartiallyCompleted,
    Failed,
    Cancelled,
    Abstained,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
enum NodeState {
    Completed,
    Abstained,
    Failed,
    Cancelled,
    Skipped,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
enum Failure {
    UnknownWorkflow,
    InvalidWorkflow,
    ApprovalRequired,
    StaleApproval,
    RevokedWorkflow,
    SkillApprovalRequired,
    SkillRevoked,
    UntrustedSkill,
    InvalidInput,
    BudgetExceeded,
    MappingUnavailable,
    DependencyFailed,
    SkillFailed,
    Abstained,
    Cancelled,
    InputTooLarge,
    ReceiptIntegrityError,
    OutputSchemaMismatch,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct NodeReceipt {
    node_id: String,
    skill: WorkflowPin,
    state: NodeState,
    execution_receipt_sha256: Option<String>,
    input_sha256: Option<String>,
    output_sha256: Option<String>,
    effective_capabilities: BTreeSet<String>,
    failure: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct ConsumedBudget {
    operations: u8,
    input_bytes: u64,
    output_bytes: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct ExecutionReceipt {
    schema: String,
    executor_version: String,
    workflow: WorkflowPin,
    workflow_approval_receipt_sha256: Option<String>,
    skill_approvals: Vec<SkillApprovalPin>,
    input_sha256: String,
    output_sha256: Option<String>,
    output_artifact_sha256: Option<String>,
    execution_order: Vec<String>,
    nodes: Vec<NodeReceipt>,
    requested_budget: WorkflowBudget,
    consumed_budget: ConsumedBudget,
    outcome: Outcome,
    failure: Option<Failure>,
    evidence_status: String,
}

struct RunContext<'a> {
    lab: &'a Lab,
    record: &'a WorkflowRecord,
    request: &'a WorkflowRequest,
    skill_approvals: Vec<SkillApprovalPin>,
    order: &'a [String],
    contracts: &'a BTreeMap<String, WorkflowSkillContract>,
    cancel: &'a AtomicBool,
}

fn version_key(id: &str, version: &str) -> String {
    format!("{id}@{version}")
}

fn valid_version(version: &str) -> bool {
    let parts = version.split('.').collect::<Vec<_>>();
    parts.len() == 3
        && parts.iter().all(|part| {
            !part.is_empty()
                && part.bytes().all(|byte| byte.is_ascii_digit())
                && (part == &"0" || !part.starts_with('0'))
                && part.parse::<u32>().is_ok()
        })
}

fn valid_text(value: &str, max: usize) -> bool {
    !value.trim().is_empty() && value.len() <= max && !value.chars().any(char::is_control)
}

fn object(required: &[(&str, JsonShape)], optional: &[(&str, JsonShape)]) -> JsonShape {
    JsonShape::Object {
        required: required
            .iter()
            .map(|(key, shape)| ((*key).into(), shape.clone()))
            .collect(),
        optional: optional
            .iter()
            .map(|(key, shape)| ((*key).into(), shape.clone()))
            .collect(),
        additional_properties: false,
    }
}

fn string_list() -> JsonShape {
    JsonShape::Array {
        items: Box::new(JsonShape::String),
    }
}

fn workflow_input_shape(include_literature: bool) -> JsonShape {
    let mut required = vec![("root_id", JsonShape::String)];
    let mut optional = vec![];
    if include_literature {
        required.push(("question", JsonShape::String));
        optional.push(("source_ids", string_list()));
    }
    object(&required, &optional)
}

fn workflow_output_shape(nodes: &[WorkflowNode], outputs: &BTreeMap<String, Binding>) -> JsonShape {
    let states = JsonShape::StringEnum {
        values: ["completed", "abstained", "failed", "cancelled", "skipped"]
            .into_iter()
            .map(str::to_owned)
            .collect(),
    };
    object(
        &[
            (
                "values",
                JsonShape::Object {
                    required: BTreeMap::new(),
                    optional: outputs
                        .keys()
                        .map(|name| (name.clone(), JsonShape::ObjectValue))
                        .collect(),
                    additional_properties: false,
                },
            ),
            (
                "node_states",
                JsonShape::Object {
                    required: nodes
                        .iter()
                        .map(|node| (node.id.clone(), states.clone()))
                        .collect(),
                    optional: BTreeMap::new(),
                    additional_properties: false,
                },
            ),
        ],
        &[],
    )
}

fn pin_from_registry(rows: &[Value], id: &str) -> Result<WorkflowPin> {
    let row = rows
        .iter()
        .find(|row| row["id"] == id)
        .ok_or("built-in workflow skill is missing")?;
    let pin = WorkflowPin {
        id: row["id"]
            .as_str()
            .ok_or("invalid built-in skill id")?
            .into(),
        version: row["version"]
            .as_str()
            .ok_or("invalid built-in skill version")?
            .into(),
        sha256: row["sha256"]
            .as_str()
            .ok_or("invalid built-in skill digest")?
            .into(),
    };
    if !valid_digest(&pin.sha256) {
        return Err("built-in workflow skill digest is invalid".into());
    }
    Ok(pin)
}

fn node(
    id: &str,
    skill: WorkflowPin,
    depends_on: &[&str],
    inputs: &[(&str, Binding)],
) -> WorkflowNode {
    WorkflowNode {
        id: id.into(),
        skill,
        depends_on: depends_on.iter().map(|item| (*item).into()).collect(),
        inputs: inputs
            .iter()
            .map(|(key, value)| ((*key).into(), value.clone()))
            .collect(),
    }
}

fn input(path: &str) -> Binding {
    Binding::WorkflowInput {
        path: vec![path.into()],
    }
}

fn output(node_id: &str) -> Binding {
    Binding::NodeOutput {
        node_id: node_id.into(),
        path: vec!["result".into()],
    }
}

fn workflow(
    id: &str,
    description: &str,
    input_schema: JsonShape,
    nodes: Vec<WorkflowNode>,
    outputs: &[(&str, Binding)],
) -> WorkflowDefinition {
    let outputs = outputs
        .iter()
        .map(|(key, binding)| ((*key).into(), binding.clone()))
        .collect::<BTreeMap<_, _>>();
    WorkflowDefinition {
        schema: DEFINITION_SCHEMA.into(),
        id: id.into(),
        version: "1.0.0".into(),
        description: description.into(),
        input_schema,
        output_schema: workflow_output_shape(&nodes, &outputs),
        maximum_budget: WorkflowBudget {
            operations: nodes.len() as u8,
            input_bytes: MAX_INPUT_BYTES,
            output_bytes: (nodes.len() as u64 * MAX_NODE_OUTPUT_BYTES * 2).min(MAX_OUTPUT_BYTES),
            dependency_depth: MAX_DEPTH,
        },
        nodes,
        outputs,
    }
}

fn builtins(lab: &Lab) -> Result<Vec<WorkflowDefinition>> {
    let listed = skills::list(lab)?;
    let rows = listed["skills"]
        .as_array()
        .ok_or("skill registry response malformed")?;
    let literature = pin_from_registry(rows, "literature-review")?;
    let hypothesis = pin_from_registry(rows, "hypothesis-criticism")?;
    let design = pin_from_registry(rows, "experimental-design")?;
    let lit_hyp = workflow(
        "literature-hypothesis",
        "Fixture workflow: local literature lookup followed by hypothesis criticism.",
        workflow_input_shape(true),
        vec![
            node(
                "literature",
                literature.clone(),
                &[],
                &[
                    ("question", input("question")),
                    ("source_ids", input("source_ids")),
                ],
            ),
            node(
                "hypothesis",
                hypothesis.clone(),
                &["literature"],
                &[("root_id", input("root_id"))],
            ),
        ],
        &[
            ("literature", output("literature")),
            ("hypothesis", output("hypothesis")),
        ],
    );
    let hyp_design = workflow(
        "hypothesis-design",
        "Fixture workflow: hypothesis criticism followed by exploratory design lookup.",
        workflow_input_shape(false),
        vec![
            node(
                "criticism",
                hypothesis.clone(),
                &[],
                &[("root_id", input("root_id"))],
            ),
            node(
                "design",
                design.clone(),
                &["criticism"],
                &[("root_id", input("root_id"))],
            ),
        ],
        &[
            ("criticism", output("criticism")),
            ("design", output("design")),
        ],
    );
    let lit_hyp_design = workflow(
        "literature-hypothesis-design",
        "Fixture workflow: local literature lookup, hypothesis criticism and exploratory design lookup.",
        workflow_input_shape(true),
        vec![
            node(
                "literature",
                literature,
                &[],
                &[
                    ("question", input("question")),
                    ("source_ids", input("source_ids")),
                ],
            ),
            node(
                "hypothesis",
                hypothesis,
                &["literature"],
                &[("root_id", input("root_id"))],
            ),
            node(
                "design",
                design,
                &["hypothesis"],
                &[("root_id", input("root_id"))],
            ),
        ],
        &[
            ("literature", output("literature")),
            ("hypothesis", output("hypothesis")),
            ("design", output("design")),
        ],
    );
    Ok(vec![lit_hyp, hyp_design, lit_hyp_design])
}

fn pins(definition: &WorkflowDefinition) -> Vec<(String, String, String)> {
    definition
        .nodes
        .iter()
        .map(|node| {
            (
                node.skill.id.clone(),
                node.skill.version.clone(),
                node.skill.sha256.clone(),
            )
        })
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn topology(definition: &WorkflowDefinition) -> Result<(Vec<String>, u8)> {
    let nodes = definition
        .nodes
        .iter()
        .map(|node| (node.id.as_str(), node))
        .collect::<BTreeMap<_, _>>();
    let mut remaining = BTreeMap::new();
    let mut dependents: BTreeMap<&str, BTreeSet<&str>> = BTreeMap::new();
    for node in &definition.nodes {
        let dependencies = node
            .depends_on
            .iter()
            .map(String::as_str)
            .collect::<BTreeSet<_>>();
        if dependencies.len() != node.depends_on.len() || dependencies.contains(node.id.as_str()) {
            return Err("workflow has duplicate or self dependencies".into());
        }
        for dependency in &dependencies {
            if !nodes.contains_key(dependency) {
                return Err("workflow references a missing node dependency".into());
            }
            dependents.entry(dependency).or_default().insert(&node.id);
        }
        remaining.insert(node.id.as_str(), dependencies.len());
    }
    let mut ready = remaining
        .iter()
        .filter_map(|(id, count)| (*count == 0).then_some(*id))
        .collect::<BTreeSet<_>>();
    let mut order = Vec::with_capacity(nodes.len());
    let mut depth: BTreeMap<&str, u8> = BTreeMap::new();
    let mut maximum = 0;
    while let Some(id) = ready.pop_first() {
        let node = nodes[id];
        let node_depth = node
            .depends_on
            .iter()
            .map(|parent| depth[parent.as_str()].saturating_add(1))
            .max()
            .unwrap_or(0);
        if node_depth > MAX_DEPTH {
            return Err("workflow dependency depth exceeds the limit".into());
        }
        maximum = maximum.max(node_depth);
        depth.insert(id, node_depth);
        order.push(id.to_owned());
        for child in dependents.get(id).into_iter().flatten() {
            let count = remaining.get_mut(child).expect("known dependent node");
            *count -= 1;
            if *count == 0 {
                ready.insert(child);
            }
        }
    }
    if order.len() != nodes.len() {
        return Err("workflow dependency graph contains a cycle".into());
    }
    Ok((order, maximum))
}

fn field_shape<'a>(shape: &'a JsonShape, field: &str) -> Option<(&'a JsonShape, bool)> {
    match shape {
        JsonShape::Object {
            required, optional, ..
        } => required
            .get(field)
            .map(|shape| (shape, true))
            .or_else(|| optional.get(field).map(|shape| (shape, false))),
        _ => None,
    }
}

fn schema_at_path<'a>(shape: &'a JsonShape, path: &[String]) -> Result<&'a JsonShape> {
    if path.is_empty() || path.len() > 12 {
        return Err("workflow field path is empty or too deep".into());
    }
    let mut current = shape;
    for field in path {
        if !valid_text(field, 64) {
            return Err("workflow field path contains an invalid key".into());
        }
        current = field_shape(current, field)
            .map(|(shape, _)| shape)
            .ok_or("workflow field path is not declared by its source schema")?;
    }
    Ok(current)
}

fn compatible(source: &JsonShape, target: &JsonShape) -> bool {
    match (source, target) {
        (left, right) if left == right => true,
        (JsonShape::StringEnum { .. }, JsonShape::String) => true,
        (JsonShape::StringEnum { values: source }, JsonShape::StringEnum { values: target }) => {
            source.is_subset(target)
        }
        (JsonShape::Array { items: source }, JsonShape::Array { items: target }) => {
            compatible(source, target)
        }
        _ => false,
    }
}

fn ancestors(
    node: &WorkflowNode,
    nodes: &BTreeMap<&str, &WorkflowNode>,
) -> Result<BTreeSet<String>> {
    let mut result = BTreeSet::new();
    let mut pending = node.depends_on.clone();
    while let Some(id) = pending.pop() {
        if result.insert(id.clone()) {
            pending.extend(
                nodes
                    .get(id.as_str())
                    .ok_or("workflow references a missing dependency")?
                    .depends_on
                    .iter()
                    .cloned(),
            );
        }
    }
    Ok(result)
}

fn source_shape<'a>(
    binding: &'a Binding,
    workflow_input: &'a JsonShape,
    current: &WorkflowNode,
    nodes: &BTreeMap<&str, &WorkflowNode>,
    node_contracts: &BTreeMap<&str, &'a WorkflowSkillContract>,
) -> Result<Option<&'a JsonShape>> {
    match binding {
        Binding::Literal { .. } => Ok(None),
        Binding::WorkflowInput { path } => Ok(Some(schema_at_path(workflow_input, path)?)),
        Binding::NodeOutput { node_id, path } => {
            if !ancestors(current, nodes)?.contains(node_id) {
                return Err("workflow output mapping lacks an explicit dependency path".into());
            }
            let source = node_contracts
                .get(node_id.as_str())
                .ok_or("workflow output mapping references an unknown node")?;
            Ok(Some(schema_at_path(&source.output_schema, path)?))
        }
    }
}

fn validate_definition(lab: &Lab, definition: &WorkflowDefinition) -> Result<Vec<String>> {
    if definition.schema != DEFINITION_SCHEMA
        || !valid_id(&definition.id)
        || !valid_version(&definition.version)
        || !valid_text(&definition.description, 240)
        || !skills::valid_workflow_shape(&definition.input_schema)
        || !matches!(
            definition.input_schema,
            JsonShape::Object {
                additional_properties: false,
                ..
            }
        )
        || definition.output_schema != workflow_output_shape(&definition.nodes, &definition.outputs)
        || definition.nodes.is_empty()
        || definition.nodes.len() > MAX_NODES
        || definition.outputs.is_empty()
        || definition.maximum_budget.operations as usize > MAX_NODES
        || (definition.maximum_budget.operations as usize) < definition.nodes.len()
        || definition.maximum_budget.input_bytes == 0
        || definition.maximum_budget.input_bytes > MAX_INPUT_BYTES
        || definition.maximum_budget.output_bytes == 0
        || definition.maximum_budget.output_bytes > MAX_OUTPUT_BYTES
        || definition.maximum_budget.dependency_depth > MAX_DEPTH
    {
        return Err("invalid or unsupported workflow definition".into());
    }
    let (order, maximum_depth) = topology(definition)?;
    if maximum_depth > definition.maximum_budget.dependency_depth {
        return Err("workflow dependency depth exceeds its declared budget".into());
    }
    let nodes = definition
        .nodes
        .iter()
        .map(|node| (node.id.as_str(), node))
        .collect::<BTreeMap<_, _>>();
    if nodes.len() != definition.nodes.len()
        || nodes.keys().any(|id| !valid_id(id))
        || definition
            .nodes
            .iter()
            .any(|node| node.inputs.len() > 32 || node.skill.sha256.len() != 64)
    {
        return Err("workflow contains duplicate or invalid nodes".into());
    }
    let contracts = skills::workflow_skill_closure(lab, &pins(definition))?
        .into_iter()
        .map(|contract| (contract.sha256.clone(), contract))
        .collect::<BTreeMap<_, _>>();
    let node_contracts = definition
        .nodes
        .iter()
        .map(|node| {
            let contract = contracts
                .get(&node.skill.sha256)
                .filter(|contract| {
                    contract.id == node.skill.id && contract.version == node.skill.version
                })
                .ok_or("workflow skill pin does not resolve exactly")?;
            if !contract.trusted || !contract.executable {
                return Err("workflow may reference only reviewed executable skills".into());
            }
            Ok((node.id.as_str(), contract))
        })
        .collect::<Result<BTreeMap<_, _>>>()?;
    for node in &definition.nodes {
        let contract = node_contracts[node.id.as_str()];
        let JsonShape::Object {
            required,
            optional,
            additional_properties: false,
        } = &contract.input_schema
        else {
            return Err("workflow skill input schema must be a strict object".into());
        };
        if required
            .keys()
            .any(|field| !node.inputs.contains_key(field))
            || node
                .inputs
                .keys()
                .any(|field| !required.contains_key(field) && !optional.contains_key(field))
        {
            return Err("workflow node bindings do not match the skill input schema".into());
        }
        for (target, binding) in &node.inputs {
            let (target_shape, target_required) = field_shape(&contract.input_schema, target)
                .ok_or("workflow input target is not declared by the skill")?;
            if let Some(source_shape) = source_shape(
                binding,
                &definition.input_schema,
                node,
                &nodes,
                &node_contracts,
            )? {
                if !compatible(source_shape, target_shape) {
                    return Err("workflow input/output mapping has incompatible types".into());
                }
                if let Binding::WorkflowInput { path } = binding
                    && !field_shape(&definition.input_schema, &path[0])
                        .is_some_and(|(_, required)| required)
                    && target_required
                {
                    return Err("optional workflow input cannot fill a required skill field".into());
                }
            } else if let Binding::Literal { value } = binding
                && !skills::matches_workflow_shape(target_shape, value)
            {
                return Err("workflow literal does not match the skill input type".into());
            }
        }
    }
    for (name, binding) in &definition.outputs {
        if !valid_id(name) {
            return Err("workflow output name is invalid".into());
        }
        let Binding::NodeOutput { node_id, path } = binding else {
            return Err("workflow outputs must map from an executed node output".into());
        };
        let contract = node_contracts
            .get(node_id.as_str())
            .ok_or("workflow output references an unknown node")?;
        let source = schema_at_path(&contract.output_schema, path)?;
        let values = field_shape(&definition.output_schema, "values")
            .map(|(shape, _)| shape)
            .ok_or("workflow result values schema is missing")?;
        let target = field_shape(values, name)
            .map(|(shape, _)| shape)
            .ok_or("workflow output name is not declared")?;
        if !compatible(source, target) {
            return Err("workflow output mapping has an incompatible type".into());
        }
    }
    Ok(order)
}

fn definition_bytes(definition: &WorkflowDefinition) -> Result<Vec<u8>> {
    Ok(serde_json::to_vec(definition)?)
}

fn definition_digest(definition: &WorkflowDefinition) -> Result<String> {
    Ok(sha256(&definition_bytes(definition)?))
}

fn expected_skill_pins(
    lab: &Lab,
    definition: &WorkflowDefinition,
) -> Result<Vec<(String, String, String)>> {
    Ok(skills::workflow_skill_closure(lab, &pins(definition))?
        .into_iter()
        .map(|contract| (contract.id, contract.version, contract.sha256))
        .collect())
}

fn artifact_receipt<T: for<'de> Deserialize<'de>>(lab: &Lab, digest: &str) -> Result<T> {
    if !valid_digest(digest) {
        return Err("workflow receipt digest is invalid".into());
    }
    let bytes = lab.artifact(digest)?;
    if sha256(&bytes) != digest {
        return Err("workflow receipt artifact failed its SHA-256 check".into());
    }
    Ok(serde_json::from_slice(&bytes)?)
}

fn validate_record(lab: &Lab, record: &WorkflowRecord) -> Result<()> {
    let bytes = definition_bytes(&record.definition)?;
    if record.schema != RECORD_SCHEMA
        || sha256(&bytes) != record.sha256
        || record.artifact_sha256 != record.sha256
        || lab.artifact(&record.sha256)? != bytes
    {
        return Err("workflow definition failed its content integrity check".into());
    }
    validate_definition(lab, &record.definition)?;
    let expected = expected_skill_pins(lab, &record.definition)?;
    if let Some(pin) = &record.approval_receipt_sha256 {
        let receipt: ApprovalReceipt = artifact_receipt(lab, pin)?;
        let approved: Vec<_> = receipt
            .skill_approvals
            .iter()
            .map(|skill| {
                (
                    skill.id.clone(),
                    skill.version.clone(),
                    skill.sha256.clone(),
                )
            })
            .collect();
        if receipt.schema != APPROVAL_SCHEMA
            || receipt.workflow_sha256 != record.sha256
            || receipt.action != "approved_exact_workflow_and_skills"
            || receipt.reviewer != "local_operator"
            || approved != expected
        {
            return Err("workflow approval receipt does not match its pinned definition".into());
        }
    }
    if let Some(pin) = &record.revocation_receipt_sha256 {
        let receipt: RevocationReceipt = artifact_receipt(lab, pin)?;
        if receipt.schema != REVOCATION_SCHEMA
            || receipt.workflow_sha256 != record.sha256
            || Some(receipt.approval_receipt_sha256.as_str())
                != record.approval_receipt_sha256.as_deref()
            || receipt.action != "revoked_workflow_approval"
            || receipt.reviewer != "local_operator"
        {
            return Err("workflow revocation receipt does not match its approval".into());
        }
    }
    Ok(())
}

fn workflow_records(lab: &Lab) -> Result<Vec<WorkflowRecord>> {
    let registry = lab.model_registry()?;
    let mut statement = registry.0.prepare(
        "SELECT record FROM model_registry WHERE kind='research-workflow' ORDER BY key, sha256 LIMIT ?1",
    )?;
    let records = statement
        .query_map([MAX_WORKFLOWS as i64 + 1], |row| row.get::<_, String>(0))?
        .map(|row| {
            let record: WorkflowRecord = serde_json::from_str(&row?)?;
            validate_record(lab, &record)?;
            Ok(record)
        })
        .collect::<Result<Vec<_>>>()?;
    if records.len() > MAX_WORKFLOWS {
        return Err("workflow registry capacity exceeded".into());
    }
    Ok(records)
}

fn register(lab: &Lab, definition: WorkflowDefinition) -> Result<WorkflowRecord> {
    validate_definition(lab, &definition)?;
    let digest = definition_digest(&definition)?;
    let key = version_key(&definition.id, &definition.version);
    let registry = lab.model_registry()?;
    let existing = registry
        .0
        .query_row(
            "SELECT record FROM model_registry WHERE kind='research-workflow' AND key=?1 LIMIT 1",
            [&key],
            |row| row.get::<_, String>(0),
        )
        .optional()?;
    if let Some(existing) = existing {
        let record: WorkflowRecord = serde_json::from_str(&existing)?;
        validate_record(lab, &record)?;
        if record.sha256 != digest {
            return Err(
                "workflow versions are immutable; publish changed content under a new version"
                    .into(),
            );
        }
        return Ok(record);
    }
    let bytes = definition_bytes(&definition)?;
    let artifact = lab.record_development_bytes(&bytes)?;
    if artifact != digest {
        return Err("workflow definition artifact digest mismatch".into());
    }
    let record = WorkflowRecord {
        schema: RECORD_SCHEMA.into(),
        definition,
        sha256: digest.clone(),
        artifact_sha256: artifact,
        approval_receipt_sha256: None,
        revocation_receipt_sha256: None,
    };
    Registry::save(&registry.0, "research-workflow", &key, &digest, &record)?;
    Ok(record)
}

fn seed(lab: &Lab) -> Result<Vec<WorkflowRecord>> {
    for definition in builtins(lab)? {
        register(lab, definition)?;
    }
    workflow_records(lab)
}

fn approval_status(record: &WorkflowRecord) -> &'static str {
    if record.revocation_receipt_sha256.is_some() {
        "revoked"
    } else if record.approval_receipt_sha256.is_some() {
        "approved"
    } else {
        "unapproved"
    }
}

pub(crate) fn list(lab: &Lab) -> Result<Value> {
    let records = seed(lab)?;
    let workflows = records
        .iter()
        .map(|record| {
            json!({
                "id":record.definition.id,
                "version":record.definition.version,
                "sha256":record.sha256,
                "description":record.definition.description,
                "approval":approval_status(record),
                "nodes":record.definition.nodes,
                "maximum_budget":record.definition.maximum_budget,
                "execution":"sequential_via_approved_skill_kernel"
            })
        })
        .collect::<Vec<_>>();
    Ok(json!({"schema":"aether.workflow-registry/1","workflows":workflows,"truncated":false}))
}

pub(crate) fn inspect(lab: &Lab, digest: &str) -> Result<Value> {
    let record = seed(lab)?
        .into_iter()
        .find(|record| record.sha256 == digest)
        .ok_or("unknown workflow digest")?;
    Ok(
        json!({"workflow":record,"approval":approval_status(&record),
        "execution":"sequential_via_approved_skill_kernel"}),
    )
}

fn save_approval(lab: &Lab, receipt: &ApprovalReceipt) -> Result<String> {
    let bytes = serde_json::to_vec(receipt)?;
    let digest = lab.record_development_bytes(&bytes)?;
    if sha256(&bytes) != digest {
        return Err("workflow approval receipt digest mismatch".into());
    }
    let registry = lab.model_registry()?;
    Registry::save(
        &registry.0,
        "workflow-approval",
        &receipt.workflow_sha256,
        &digest,
        receipt,
    )?;
    Ok(digest)
}

pub(crate) fn approve(lab: &Lab, digest: &str) -> Result<Value> {
    let mut record = seed(lab)?
        .into_iter()
        .find(|record| record.sha256 == digest)
        .ok_or("unknown workflow digest")?;
    if record.revocation_receipt_sha256.is_some() {
        return Err(
            "revoked workflow approvals are terminal; publish a new workflow version".into(),
        );
    }
    let contracts = skills::workflow_skill_closure(lab, &pins(&record.definition))?;
    if contracts.iter().any(|contract| !contract.trusted) {
        return Err("workflow references an untrusted skill".into());
    }
    if contracts.iter().any(|contract| contract.revoked) {
        return Err("workflow references a revoked skill approval".into());
    }
    if contracts.iter().any(|contract| !contract.approved) {
        return Err(
            "approve every exact referenced skill version before approving the workflow".into(),
        );
    }
    let skill_approvals = contracts
        .into_iter()
        .map(|contract| SkillApprovalPin {
            id: contract.id,
            version: contract.version,
            sha256: contract.sha256,
            approval_receipt_sha256: contract
                .approval_receipt_sha256
                .expect("approved contract has receipt"),
        })
        .collect();
    let receipt = ApprovalReceipt {
        schema: APPROVAL_SCHEMA.into(),
        workflow_sha256: record.sha256.clone(),
        skill_approvals,
        action: "approved_exact_workflow_and_skills".into(),
        reviewer: "local_operator".into(),
    };
    record.approval_receipt_sha256 = Some(save_approval(lab, &receipt)?);
    let registry = lab.model_registry()?;
    Registry::save(
        &registry.0,
        "research-workflow",
        &version_key(&record.definition.id, &record.definition.version),
        &record.sha256,
        &record,
    )?;
    Ok(
        json!({"workflow_sha256":record.sha256,"approval":"approved",
        "approval_receipt_sha256":record.approval_receipt_sha256,"execution":"not_performed"}),
    )
}

pub(crate) fn revoke(lab: &Lab, digest: &str) -> Result<Value> {
    let mut record = seed(lab)?
        .into_iter()
        .find(|record| record.sha256 == digest)
        .ok_or("unknown workflow digest")?;
    let approval = record
        .approval_receipt_sha256
        .clone()
        .ok_or("workflow has no active approval to revoke")?;
    if record.revocation_receipt_sha256.is_some() {
        return Err("workflow approval is already revoked".into());
    }
    let receipt = RevocationReceipt {
        schema: REVOCATION_SCHEMA.into(),
        workflow_sha256: record.sha256.clone(),
        approval_receipt_sha256: approval,
        action: "revoked_workflow_approval".into(),
        reviewer: "local_operator".into(),
    };
    let bytes = serde_json::to_vec(&receipt)?;
    let pin = lab.record_development_bytes(&bytes)?;
    let registry = lab.model_registry()?;
    Registry::save(&registry.0, "workflow-revocation", digest, &pin, &receipt)?;
    record.revocation_receipt_sha256 = Some(pin.clone());
    Registry::save(
        &registry.0,
        "research-workflow",
        &version_key(&record.definition.id, &record.definition.version),
        &record.sha256,
        &record,
    )?;
    Ok(json!({"workflow_sha256":record.sha256,"approval":"revoked",
        "revocation_receipt_sha256":pin,"execution":"not_performed"}))
}

fn resolve_path(value: &Value, path: &[String]) -> Option<Value> {
    let mut current = value;
    for field in path {
        current = current.as_object()?.get(field)?;
    }
    Some(current.clone())
}

fn resolve_binding(
    binding: &Binding,
    workflow_input: &Value,
    node_outputs: &BTreeMap<String, Value>,
) -> Result<Option<Value>> {
    match binding {
        Binding::WorkflowInput { path } => Ok(resolve_path(workflow_input, path)),
        Binding::NodeOutput { node_id, path } => node_outputs
            .get(node_id)
            .and_then(|output| resolve_path(output, path))
            .map(Some)
            .ok_or_else(|| "workflow mapping source is unavailable".into()),
        Binding::Literal { value } => Ok(Some(value.clone())),
    }
}

fn node_input(
    node: &WorkflowNode,
    input: &Value,
    outputs: &BTreeMap<String, Value>,
    contract: &WorkflowSkillContract,
) -> Result<Value> {
    let mut values = serde_json::Map::new();
    for (target, binding) in &node.inputs {
        if let Some(value) = resolve_binding(binding, input, outputs)? {
            values.insert(target.clone(), value);
        }
    }
    let value = Value::Object(values);
    if !skills::matches_workflow_shape(&contract.input_schema, &value) {
        return Err("resolved workflow input failed the pinned skill schema".into());
    }
    Ok(value)
}

fn output_values(
    definition: &WorkflowDefinition,
    node_outputs: &BTreeMap<String, Value>,
    node_states: &BTreeMap<String, NodeState>,
) -> Result<Value> {
    let mut values = serde_json::Map::new();
    for (name, binding) in &definition.outputs {
        if matches!(binding, Binding::NodeOutput { node_id, .. } if !node_outputs.contains_key(node_id))
        {
            continue;
        }
        if let Some(value) = resolve_binding(binding, &Value::Null, node_outputs)? {
            values.insert(name.clone(), value);
        }
    }
    let states = node_states
        .iter()
        .map(|(id, state)| (id.clone(), json!(state)))
        .collect::<serde_json::Map<_, _>>();
    let result = json!({"values":values,"node_states":states});
    if !skills::matches_workflow_shape(&definition.output_schema, &result) {
        return Err("workflow result failed its declared output schema".into());
    }
    Ok(result)
}

fn save_execution(lab: &Lab, receipt: &ExecutionReceipt) -> Result<String> {
    let bytes = serde_json::to_vec(receipt)?;
    let digest = lab.record_development_bytes(&bytes)?;
    if sha256(&bytes) != digest {
        return Err("workflow execution receipt digest mismatch".into());
    }
    let registry = lab.model_registry()?;
    Registry::save(&registry.0, "workflow-execution", &digest, &digest, receipt)?;
    Ok(digest)
}

fn failed_receipt(
    lab: &Lab,
    request: &WorkflowRequest,
    failure: Failure,
    outcome: Outcome,
) -> Result<Value> {
    let receipt = ExecutionReceipt {
        schema: EXECUTION_SCHEMA.into(),
        executor_version: EXECUTOR_VERSION.into(),
        workflow: request.workflow.clone(),
        workflow_approval_receipt_sha256: None,
        skill_approvals: Vec::new(),
        input_sha256: sha256(&serde_json::to_vec(&request.input)?),
        output_sha256: None,
        output_artifact_sha256: None,
        execution_order: Vec::new(),
        nodes: Vec::new(),
        requested_budget: request.budget,
        consumed_budget: ConsumedBudget {
            operations: 0,
            input_bytes: 0,
            output_bytes: 0,
        },
        outcome,
        failure: Some(failure),
        evidence_status: "unverified".into(),
    };
    let digest = save_execution(lab, &receipt)?;
    Ok(json!({"workflow_execution_sha256":digest,"receipt":receipt,"output":Value::Null}))
}

fn approval_for(
    lab: &Lab,
    record: &WorkflowRecord,
) -> Result<(ApprovalReceipt, Vec<SkillApprovalPin>)> {
    if record.revocation_receipt_sha256.is_some() {
        return Err("workflow approval is revoked".into());
    }
    let approval_pin = record
        .approval_receipt_sha256
        .as_deref()
        .ok_or("workflow approval is required")?;
    let receipt: ApprovalReceipt = artifact_receipt(lab, approval_pin)?;
    let contracts = skills::workflow_skill_closure(lab, &pins(&record.definition))?;
    if contracts.iter().any(|contract| !contract.trusted) {
        return Err("workflow references an untrusted skill".into());
    }
    if contracts.iter().any(|contract| contract.revoked) {
        return Err("workflow references a revoked skill".into());
    }
    if contracts.iter().any(|contract| !contract.approved) {
        return Err("workflow references an unapproved skill".into());
    }
    let current = contracts
        .into_iter()
        .map(|contract| SkillApprovalPin {
            id: contract.id,
            version: contract.version,
            sha256: contract.sha256,
            approval_receipt_sha256: contract
                .approval_receipt_sha256
                .expect("approved contract has receipt"),
        })
        .collect::<Vec<_>>();
    if receipt.skill_approvals != current {
        return Err("workflow approval is stale for its current skill approvals".into());
    }
    Ok((receipt, current))
}

fn budget_fits(requested: WorkflowBudget, maximum: WorkflowBudget, graph_depth: u8) -> bool {
    requested.operations > 0
        && requested.operations <= maximum.operations
        && requested.input_bytes > 0
        && requested.input_bytes <= maximum.input_bytes
        && requested.output_bytes > 0
        && requested.output_bytes <= maximum.output_bytes
        && requested.dependency_depth >= graph_depth
        && requested.dependency_depth <= maximum.dependency_depth
        && requested
            .input_bytes
            .checked_add(requested.output_bytes)
            .is_some()
}

fn output_artifact(lab: &Lab, output: &Value) -> Result<(String, u64)> {
    let bytes = serde_json::to_vec(output)?;
    let digest = lab.record_development_bytes(&bytes)?;
    if sha256(&bytes) != digest {
        return Err("workflow output artifact digest mismatch".into());
    }
    Ok((digest, bytes.len() as u64))
}

fn node_failure_state(result: &Value) -> (NodeState, Option<String>) {
    let state = match result["receipt"]["state"].as_str() {
        Some("completed") => NodeState::Completed,
        Some("abstained") => NodeState::Abstained,
        Some("cancelled") => NodeState::Cancelled,
        _ => NodeState::Failed,
    };
    (
        state,
        result["receipt"]["failure"].as_str().map(str::to_owned),
    )
}

fn run_nodes<F>(context: RunContext<'_>, mut run_kernel: F) -> Result<Value>
where
    F: FnMut(&WorkflowSkillContract, Value, u8, u64, &AtomicBool) -> Result<Value>,
{
    let RunContext {
        lab,
        record,
        request,
        skill_approvals,
        order,
        contracts,
        cancel,
    } = context;
    let by_id = record
        .definition
        .nodes
        .iter()
        .map(|node| (node.id.as_str(), node))
        .collect::<BTreeMap<_, _>>();
    let mut outputs = BTreeMap::new();
    let mut states = BTreeMap::new();
    let mut nodes = Vec::new();
    let mut execution_order = Vec::new();
    let mut consumed = ConsumedBudget {
        operations: 0,
        input_bytes: 0,
        output_bytes: 0,
    };
    let mut failure = None;
    let mut outcome = Outcome::Completed;
    for (position, id) in order.iter().enumerate() {
        let node = by_id[id.as_str()];
        if node
            .depends_on
            .iter()
            .any(|dependency| states.get(dependency) != Some(&NodeState::Completed))
        {
            failure = Some(Failure::DependencyFailed);
            states.insert(id.clone(), NodeState::Skipped);
            nodes.push(NodeReceipt {
                node_id: id.clone(),
                skill: node.skill.clone(),
                state: NodeState::Skipped,
                execution_receipt_sha256: None,
                input_sha256: None,
                output_sha256: None,
                effective_capabilities: BTreeSet::new(),
                failure: Some("dependency_failed".into()),
            });
            break;
        }
        if consumed.operations >= request.budget.operations {
            failure = Some(Failure::BudgetExceeded);
            states.insert(id.clone(), NodeState::Skipped);
            nodes.push(NodeReceipt {
                node_id: id.clone(),
                skill: node.skill.clone(),
                state: NodeState::Skipped,
                execution_receipt_sha256: None,
                input_sha256: None,
                output_sha256: None,
                effective_capabilities: BTreeSet::new(),
                failure: Some("aggregate_operation_budget_exhausted".into()),
            });
            break;
        }
        let contract = &contracts[&node.skill.sha256];
        let input = match node_input(node, &request.input, &outputs, contract) {
            Ok(input) => input,
            Err(_) => {
                failure = Some(Failure::MappingUnavailable);
                states.insert(id.clone(), NodeState::Failed);
                nodes.push(NodeReceipt {
                    node_id: id.clone(),
                    skill: node.skill.clone(),
                    state: NodeState::Failed,
                    execution_receipt_sha256: None,
                    input_sha256: None,
                    output_sha256: None,
                    effective_capabilities: BTreeSet::new(),
                    failure: Some("input_mapping_unavailable".into()),
                });
                break;
            }
        };
        let input_bytes = serde_json::to_vec(&input)?;
        let input_length = input_bytes.len() as u64;
        let Some(next_input) = consumed.input_bytes.checked_add(input_length) else {
            failure = Some(Failure::BudgetExceeded);
            break;
        };
        if next_input > request.budget.input_bytes {
            failure = Some(Failure::BudgetExceeded);
            states.insert(id.clone(), NodeState::Skipped);
            nodes.push(NodeReceipt {
                node_id: id.clone(),
                skill: node.skill.clone(),
                state: NodeState::Skipped,
                execution_receipt_sha256: None,
                input_sha256: Some(sha256(&input_bytes)),
                output_sha256: None,
                effective_capabilities: BTreeSet::new(),
                failure: Some("aggregate_input_budget_exhausted".into()),
            });
            break;
        }
        let remaining_output = request
            .budget
            .output_bytes
            .saturating_sub(consumed.output_bytes);
        if remaining_output == 0 {
            failure = Some(Failure::BudgetExceeded);
            states.insert(id.clone(), NodeState::Skipped);
            nodes.push(NodeReceipt {
                node_id: id.clone(),
                skill: node.skill.clone(),
                state: NodeState::Skipped,
                execution_receipt_sha256: None,
                input_sha256: Some(sha256(&input_bytes)),
                output_sha256: None,
                effective_capabilities: BTreeSet::new(),
                failure: Some("aggregate_output_budget_exhausted".into()),
            });
            break;
        }
        consumed.operations += 1;
        consumed.input_bytes = next_input;
        execution_order.push(id.clone());
        let result = run_kernel(
            contract,
            input,
            request.budget.dependency_depth,
            remaining_output.min(MAX_NODE_OUTPUT_BYTES),
            cancel,
        )?;
        let Some(kernel_digest) = result["execution_sha256"].as_str() else {
            failure = Some(Failure::ReceiptIntegrityError);
            break;
        };
        let kernel_receipt = skills::execution_receipt(lab, kernel_digest)?;
        if result["receipt"] != kernel_receipt {
            failure = Some(Failure::ReceiptIntegrityError);
            break;
        }
        let effective = kernel_receipt["effective_capabilities"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
            .map(str::to_owned)
            .collect::<BTreeSet<_>>();
        let allowed = serde_json::to_value(&contract.declared_capabilities)?
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
            .map(str::to_owned)
            .collect::<BTreeSet<_>>();
        let required = serde_json::to_value(&contract.required_capabilities)?
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
            .map(str::to_owned)
            .collect::<BTreeSet<_>>();
        if !required.is_subset(&effective) || !effective.is_subset(&allowed) {
            failure = Some(Failure::ReceiptIntegrityError);
            break;
        }
        let (state, kernel_failure) = node_failure_state(&result);
        let mut output_digest = None;
        if !result["output"].is_null() {
            let output_bytes = serde_json::to_vec(&result["output"])?;
            let output_length = output_bytes.len() as u64;
            let Some(next_output) = consumed.output_bytes.checked_add(output_length) else {
                failure = Some(Failure::BudgetExceeded);
                break;
            };
            if next_output > request.budget.output_bytes
                || kernel_receipt["output_sha256"].as_str() != Some(sha256(&output_bytes).as_str())
            {
                failure = Some(Failure::ReceiptIntegrityError);
                break;
            }
            let artifact = lab.record_development_bytes(&output_bytes)?;
            if artifact != sha256(&output_bytes) {
                failure = Some(Failure::ReceiptIntegrityError);
                break;
            }
            consumed.output_bytes = next_output;
            output_digest = Some(artifact);
            outputs.insert(id.clone(), result["output"].clone());
        }
        states.insert(id.clone(), state);
        nodes.push(NodeReceipt {
            node_id: id.clone(),
            skill: node.skill.clone(),
            state,
            execution_receipt_sha256: Some(kernel_digest.into()),
            input_sha256: Some(sha256(&input_bytes)),
            output_sha256: output_digest,
            effective_capabilities: effective,
            failure: kernel_failure,
        });
        if cancel.load(Ordering::Acquire) || state == NodeState::Cancelled {
            failure = Some(Failure::Cancelled);
            outcome = Outcome::Cancelled;
            break;
        }
        if state == NodeState::Abstained {
            failure = Some(Failure::Abstained);
            outcome = if nodes.iter().any(|node| node.state == NodeState::Completed) {
                Outcome::PartiallyCompleted
            } else {
                Outcome::Abstained
            };
            break;
        }
        if state != NodeState::Completed {
            failure = Some(Failure::SkillFailed);
            outcome = if nodes.iter().any(|node| node.state == NodeState::Completed) {
                Outcome::PartiallyCompleted
            } else {
                Outcome::Failed
            };
            break;
        }
        if position + 1 == order.len() {
            outcome = Outcome::Completed;
        }
    }
    if failure.is_some() && outcome == Outcome::Completed {
        outcome = if failure == Some(Failure::Cancelled) {
            Outcome::Cancelled
        } else if nodes.iter().any(|node| node.state == NodeState::Completed) {
            Outcome::PartiallyCompleted
        } else {
            Outcome::Failed
        };
    }
    for id in order.iter().skip(nodes.len()) {
        let node = by_id[id.as_str()];
        states.entry(id.clone()).or_insert(NodeState::Skipped);
        nodes.push(NodeReceipt {
            node_id: id.clone(),
            skill: node.skill.clone(),
            state: NodeState::Skipped,
            execution_receipt_sha256: None,
            input_sha256: None,
            output_sha256: None,
            effective_capabilities: BTreeSet::new(),
            failure: Some("workflow_stopped".into()),
        });
    }
    let output = match output_values(&record.definition, &outputs, &states) {
        Ok(output) => Some(output),
        Err(_) => {
            failure = Some(Failure::OutputSchemaMismatch);
            outcome = if consumed.operations > 0 {
                Outcome::PartiallyCompleted
            } else {
                Outcome::Failed
            };
            None
        }
    };
    let mut output_digest = None;
    if let Some(value) = output.as_ref() {
        let bytes = serde_json::to_vec(value)?;
        match consumed.output_bytes.checked_add(bytes.len() as u64) {
            Some(next_output) if next_output <= request.budget.output_bytes => {
                let (digest, length) = output_artifact(lab, value)?;
                consumed.output_bytes = consumed
                    .output_bytes
                    .checked_add(length)
                    .ok_or("workflow output accounting overflow")?;
                output_digest = Some(digest);
            }
            Some(_) | None => {
                failure = Some(Failure::BudgetExceeded);
                outcome = if consumed.operations > 0 {
                    Outcome::PartiallyCompleted
                } else {
                    Outcome::Failed
                };
            }
        }
    }
    let receipt = ExecutionReceipt {
        schema: EXECUTION_SCHEMA.into(),
        executor_version: EXECUTOR_VERSION.into(),
        workflow: request.workflow.clone(),
        workflow_approval_receipt_sha256: Some(
            record
                .approval_receipt_sha256
                .clone()
                .expect("approved workflow has receipt"),
        ),
        skill_approvals,
        input_sha256: sha256(&serde_json::to_vec(&request.input)?),
        output_sha256: output_digest.clone(),
        output_artifact_sha256: output_digest,
        execution_order,
        nodes,
        requested_budget: request.budget,
        consumed_budget: consumed,
        outcome,
        failure,
        evidence_status: "unverified".into(),
    };
    let digest = save_execution(lab, &receipt)?;
    Ok(
        json!({"workflow_execution_sha256":digest,"receipt":receipt,"output":if receipt.output_sha256.is_some() { output.unwrap_or(Value::Null) } else { Value::Null }}),
    )
}

pub(crate) fn execute(lab: &Lab, request: WorkflowRequest, cancel: &AtomicBool) -> Result<Value> {
    let records = match seed(lab) {
        Ok(records) => records,
        Err(_) => return failed_receipt(lab, &request, Failure::InvalidWorkflow, Outcome::Failed),
    };
    let Some(record) = records.into_iter().find(|record| {
        record.sha256 == request.workflow.sha256
            && record.definition.id == request.workflow.id
            && record.definition.version == request.workflow.version
    }) else {
        return failed_receipt(lab, &request, Failure::UnknownWorkflow, Outcome::Failed);
    };
    match serde_json::to_vec(&request.input) {
        Ok(bytes) if bytes.len() as u64 <= MAX_INPUT_BYTES => {}
        Ok(_) => return failed_receipt(lab, &request, Failure::InputTooLarge, Outcome::Failed),
        Err(_) => return failed_receipt(lab, &request, Failure::InvalidInput, Outcome::Failed),
    }
    let order = match validate_definition(lab, &record.definition) {
        Ok(order) => order,
        Err(_) => return failed_receipt(lab, &request, Failure::InvalidWorkflow, Outcome::Failed),
    };
    if !skills::matches_workflow_shape(&record.definition.input_schema, &request.input) {
        return failed_receipt(lab, &request, Failure::InvalidInput, Outcome::Failed);
    }
    let graph_depth = topology(&record.definition)?.1;
    if !budget_fits(
        request.budget,
        record.definition.maximum_budget,
        graph_depth,
    ) {
        return failed_receipt(lab, &request, Failure::BudgetExceeded, Outcome::Failed);
    }
    if cancel.load(Ordering::Acquire) {
        return failed_receipt(lab, &request, Failure::Cancelled, Outcome::Cancelled);
    }
    let (_, skill_approvals) = match approval_for(lab, &record) {
        Ok(approval) => approval,
        Err(error) => {
            let text = error.to_string();
            let failure =
                if text.contains("revoked workflow") || text == "workflow approval is revoked" {
                    Failure::RevokedWorkflow
                } else if text.contains("revoked skill") {
                    Failure::SkillRevoked
                } else if text.contains("stale") {
                    Failure::StaleApproval
                } else if text.contains("untrusted") {
                    Failure::UntrustedSkill
                } else if text == "workflow approval is required" {
                    Failure::ApprovalRequired
                } else if text.contains("unapproved") || text.contains("approval") {
                    Failure::SkillApprovalRequired
                } else {
                    Failure::InvalidWorkflow
                };
            return failed_receipt(lab, &request, failure, Outcome::Failed);
        }
    };
    let contracts = skills::workflow_skill_closure(lab, &pins(&record.definition))?
        .into_iter()
        .map(|contract| (contract.sha256.clone(), contract))
        .collect::<BTreeMap<_, _>>();
    run_nodes(
        RunContext {
            lab,
            record: &record,
            request: &request,
            skill_approvals,
            order: &order,
            contracts: &contracts,
            cancel,
        },
        |skill, input, depth, output_bytes, cancel| {
            skills::execute_workflow_node(
                lab,
                skills::WorkflowNodeRequest {
                    skill_id: skill.id.clone(),
                    skill_version: skill.version.clone(),
                    skill_sha256: skill.sha256.clone(),
                    input,
                    dependency_depth: depth,
                    output_bytes,
                },
                cancel,
            )
        },
    )
}

pub(crate) fn executions(lab: &Lab) -> Result<Value> {
    let registry = lab.model_registry()?;
    let mut statement = registry.0.prepare(
        "SELECT sha256, record FROM model_registry WHERE kind='workflow-execution' ORDER BY rowid DESC LIMIT 33",
    )?;
    let mut rows = statement.query([])?;
    let mut executions = Vec::new();
    while let Some(row) = rows.next()? {
        if executions.len() == 32 {
            return Ok(
                json!({"schema":"aether.workflow-executions/1","executions":executions,"truncated":true}),
            );
        }
        let digest: String = row.get(0)?;
        let raw: String = row.get(1)?;
        if !valid_digest(&digest)
            || sha256(raw.as_bytes()) != digest
            || lab.artifact(&digest)? != raw.as_bytes()
        {
            return Err("workflow execution receipt failed its content integrity check".into());
        }
        let receipt: ExecutionReceipt = serde_json::from_str(&raw)?;
        let output = match receipt.output_artifact_sha256.as_deref() {
            Some(pin) if valid_digest(pin) => {
                let bytes = lab.artifact(pin)?;
                if sha256(&bytes) != pin || Some(pin) != receipt.output_sha256.as_deref() {
                    return Err("workflow output artifact failed its integrity check".into());
                }
                Some(serde_json::from_slice::<Value>(&bytes)?)
            }
            Some(_) => return Err("workflow output digest is invalid".into()),
            None => None,
        };
        executions
            .push(json!({"workflow_execution_sha256":digest,"receipt":receipt,"output":output}));
    }
    Ok(json!({"schema":"aether.workflow-executions/1","executions":executions,"truncated":false}))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        fs,
        path::PathBuf,
        sync::atomic::{AtomicU64, Ordering},
    };

    static SEQUENCE: AtomicU64 = AtomicU64::new(0);

    struct TestLab {
        root: PathBuf,
        lab: Lab,
    }

    impl TestLab {
        fn new() -> Self {
            let root = std::env::temp_dir().join(format!(
                "aether-workflows-{}-{}",
                std::process::id(),
                SEQUENCE.fetch_add(1, Ordering::Relaxed)
            ));
            let lab = Lab::open(root.clone()).expect("temporary lab");
            Self { root, lab }
        }

        fn root_id(&self) -> String {
            self.lab
                .catalog()
                .unwrap()
                .index_hypothesis_archive()
                .unwrap()["root"]
                .as_str()
                .unwrap()
                .to_owned()
        }
    }

    impl Drop for TestLab {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.root);
        }
    }

    fn workflow_record(lab: &Lab, id: &str) -> WorkflowRecord {
        seed(lab)
            .unwrap()
            .into_iter()
            .find(|record| record.definition.id == id)
            .expect("fixture workflow")
    }

    fn request(record: &WorkflowRecord, input: Value) -> WorkflowRequest {
        WorkflowRequest {
            workflow: WorkflowPin {
                id: record.definition.id.clone(),
                version: record.definition.version.clone(),
                sha256: record.sha256.clone(),
            },
            input,
            budget: record.definition.maximum_budget,
        }
    }

    fn sample_input(record: &WorkflowRecord, root: &str) -> Value {
        let mut input = json!({"root_id":root});
        if record.definition.id.starts_with("literature-") {
            input["question"] = json!("CAVEAT reusable evaluation");
        }
        input
    }

    fn approve_workflow(lab: &Lab, record: &WorkflowRecord) {
        for skill in skills::workflow_skill_closure(lab, &pins(&record.definition)).unwrap() {
            if !skill.approved {
                skills::approve(lab, &skill.sha256, skill.required_capabilities)
                    .expect("approve exact built-in skill scope");
            }
        }
        approve(lab, &record.sha256).expect("approve exact workflow and skill pins");
    }

    #[test]
    fn built_in_workflows_are_immutable_dags_with_reproducible_digests() {
        let temp = TestLab::new();
        let first = seed(&temp.lab).unwrap();
        let second = seed(&temp.lab).unwrap();
        assert_eq!(first.len(), 3);
        assert_eq!(
            first
                .iter()
                .map(|record| &record.sha256)
                .collect::<Vec<_>>(),
            second
                .iter()
                .map(|record| &record.sha256)
                .collect::<Vec<_>>()
        );
        assert_eq!(
            first
                .iter()
                .map(|record| record.definition.id.as_str())
                .collect::<Vec<_>>(),
            [
                "hypothesis-design",
                "literature-hypothesis-design",
                "literature-hypothesis"
            ]
        );
        for record in &first {
            assert_eq!(
                definition_digest(&record.definition).unwrap(),
                record.sha256
            );
            let order = topology(&record.definition).unwrap().0;
            assert_eq!(order, topology(&record.definition).unwrap().0);
            assert!(validate_definition(&temp.lab, &record.definition).is_ok());
        }
        let mut changed = first[0].definition.clone();
        changed.description.push_str(" altered");
        assert!(
            register(&temp.lab, changed)
                .unwrap_err()
                .to_string()
                .contains("immutable")
        );
        assert_eq!(
            topology(&workflow_record(&temp.lab, "literature-hypothesis-design").definition)
                .unwrap()
                .0,
            ["literature", "hypothesis", "design"]
        );
        assert_eq!(
            list(&temp.lab).unwrap()["workflows"]
                .as_array()
                .unwrap()
                .len(),
            3
        );
    }

    #[test]
    fn graph_validation_rejects_cycles_missing_nodes_duplicates_and_excessive_depth() {
        let temp = TestLab::new();
        let original = workflow_record(&temp.lab, "hypothesis-design").definition;

        let mut cycle = original.clone();
        cycle.nodes[0].depends_on = vec![cycle.nodes[1].id.clone()];
        cycle.nodes[1].depends_on = vec![cycle.nodes[0].id.clone()];
        assert!(topology(&cycle).unwrap_err().to_string().contains("cycle"));

        let mut missing = original.clone();
        missing.nodes[1].depends_on = vec!["missing-node".into()];
        assert!(
            topology(&missing)
                .unwrap_err()
                .to_string()
                .contains("missing")
        );

        let mut duplicate = original.clone();
        duplicate.nodes[1].id = duplicate.nodes[0].id.clone();
        assert!(validate_definition(&temp.lab, &duplicate).is_err());

        let base = original.nodes[0].clone();
        let chain = (0..10)
            .map(|index| {
                let mut node = base.clone();
                node.id = format!("node-{index}");
                node.depends_on = (index > 0)
                    .then(|| format!("node-{}", index - 1))
                    .into_iter()
                    .collect();
                node
            })
            .collect();
        let mut deep = original;
        deep.nodes = chain;
        assert!(topology(&deep).unwrap_err().to_string().contains("depth"));
    }

    #[test]
    fn mapping_validation_rejects_incompatible_types_and_undeclared_dependencies() {
        let temp = TestLab::new();
        let original = workflow_record(&temp.lab, "literature-hypothesis").definition;
        let mut incompatible = original.clone();
        incompatible.nodes[1]
            .inputs
            .insert("root_id".into(), output("literature"));
        assert!(
            validate_definition(&temp.lab, &incompatible)
                .unwrap_err()
                .to_string()
                .contains("incompatible")
        );

        let mut undeclared = original;
        undeclared.nodes[1].depends_on.clear();
        undeclared.nodes[1].inputs.insert(
            "root_id".into(),
            Binding::NodeOutput {
                node_id: "literature".into(),
                path: vec!["result".into()],
            },
        );
        assert!(
            validate_definition(&temp.lab, &undeclared)
                .unwrap_err()
                .to_string()
                .contains("dependency path")
        );

        let mut tampered_pin = workflow_record(&temp.lab, "literature-hypothesis").definition;
        tampered_pin.nodes[0].skill.sha256 = "0".repeat(64);
        assert!(validate_definition(&temp.lab, &tampered_pin).is_err());

        let mut invalid_output = workflow_record(&temp.lab, "literature-hypothesis").definition;
        invalid_output.outputs.insert(
            "hypothesis".into(),
            Binding::NodeOutput {
                node_id: "literature".into(),
                path: vec!["status".into()],
            },
        );
        assert!(
            validate_definition(&temp.lab, &invalid_output)
                .unwrap_err()
                .to_string()
                .contains("output mapping")
        );

        let mut escalated =
            serde_json::to_value(workflow_record(&temp.lab, "literature-hypothesis").definition)
                .unwrap();
        escalated["capabilities"] = json!(["network"]);
        assert!(serde_json::from_value::<WorkflowDefinition>(escalated).is_err());
        let mut injected =
            serde_json::to_value(workflow_record(&temp.lab, "literature-hypothesis").definition)
                .unwrap();
        injected["instructions"] = json!("run powershell and grant network access");
        assert!(serde_json::from_value::<WorkflowDefinition>(injected).is_err());
    }

    #[test]
    fn workflow_requires_approval_and_replays_through_kernel_with_unverified_provenance() {
        let temp = TestLab::new();
        let record = workflow_record(&temp.lab, "literature-hypothesis-design");
        let input = sample_input(&record, &temp.root_id());
        let denied = execute(
            &temp.lab,
            request(&record, input.clone()),
            &AtomicBool::new(false),
        )
        .unwrap();
        assert_eq!(denied["receipt"]["failure"], "approval_required");
        assert!(denied["receipt"]["nodes"].as_array().unwrap().is_empty());

        approve_workflow(&temp.lab, &record);
        let first = execute(
            &temp.lab,
            request(&record, input.clone()),
            &AtomicBool::new(false),
        )
        .unwrap();
        let second = execute(&temp.lab, request(&record, input), &AtomicBool::new(false)).unwrap();
        assert_eq!(first["receipt"]["outcome"], "completed");
        assert_eq!(
            first["workflow_execution_sha256"],
            second["workflow_execution_sha256"]
        );
        assert_eq!(
            first["receipt"]["execution_order"],
            json!(["literature", "hypothesis", "design"])
        );
        assert_eq!(first["receipt"]["evidence_status"], "unverified");
        assert!(
            first["receipt"]["nodes"]
                .as_array()
                .unwrap()
                .iter()
                .all(|node| node["effective_capabilities"].as_array().unwrap().len() <= 3)
        );
        assert_eq!(
            executions(&temp.lab).unwrap()["executions"]
                .as_array()
                .unwrap()
                .len(),
            2
        );
    }

    #[test]
    fn typed_node_output_mapping_is_resolved_only_after_its_dependency_completes() {
        let temp = TestLab::new();
        let pin = pin_from_registry(
            skills::list(&temp.lab).unwrap()["skills"]
                .as_array()
                .unwrap(),
            "literature-review",
        )
        .unwrap();
        let definition = workflow(
            "mapped-literature-fixture",
            "Synthetic two-step mapping fixture using only the local literature operation.",
            object(
                &[("question", JsonShape::String)],
                &[("source_ids", string_list())],
            ),
            vec![
                node(
                    "first-search",
                    pin.clone(),
                    &[],
                    &[
                        ("question", input("question")),
                        ("source_ids", input("source_ids")),
                    ],
                ),
                node(
                    "second-search",
                    pin,
                    &["first-search"],
                    &[
                        (
                            "question",
                            Binding::NodeOutput {
                                node_id: "first-search".into(),
                                path: vec!["summary".into()],
                            },
                        ),
                        ("source_ids", input("source_ids")),
                    ],
                ),
            ],
            &[
                ("first", output("first-search")),
                ("second", output("second-search")),
            ],
        );
        let record = register(&temp.lab, definition).unwrap();
        approve_workflow(&temp.lab, &record);
        let result = execute(
            &temp.lab,
            request(&record, json!({"question":"CAVEAT"})),
            &AtomicBool::new(false),
        )
        .unwrap();
        assert_eq!(result["receipt"]["outcome"], "completed");
        assert_eq!(
            result["receipt"]["execution_order"],
            json!(["first-search", "second-search"])
        );
        assert_eq!(
            result["receipt"]["nodes"][1]["input_sha256"],
            sha256(
                br#"{"question":"Completed by the registered deterministic native operation."}"#
            )
        );
    }

    #[test]
    fn aggregate_budget_and_failed_dependencies_preserve_partial_and_skipped_states() {
        let temp = TestLab::new();
        let record = workflow_record(&temp.lab, "hypothesis-design");
        approve_workflow(&temp.lab, &record);
        let root = temp.root_id();
        let mut bounded = request(&record, sample_input(&record, &root));
        bounded.budget.operations = 1;
        let partial = execute(&temp.lab, bounded, &AtomicBool::new(false)).unwrap();
        assert_eq!(partial["receipt"]["outcome"], "partially_completed");
        assert_eq!(partial["receipt"]["consumed_budget"]["operations"], 1);
        assert_eq!(partial["receipt"]["nodes"][1]["state"], "skipped");

        let mut insufficient_depth = request(&record, sample_input(&record, &root));
        insufficient_depth.budget.dependency_depth = 0;
        let rejected = execute(&temp.lab, insufficient_depth, &AtomicBool::new(false)).unwrap();
        assert_eq!(rejected["receipt"]["failure"], "budget_exceeded");
        assert_eq!(rejected["receipt"]["consumed_budget"]["operations"], 0);

        let failed = execute(
            &temp.lab,
            request(&record, sample_input(&record, &"f".repeat(64))),
            &AtomicBool::new(false),
        )
        .unwrap();
        assert_eq!(failed["receipt"]["nodes"][0]["state"], "failed");
        assert_eq!(failed["receipt"]["nodes"][1]["state"], "skipped");
        assert_eq!(failed["receipt"]["failure"], "skill_failed");
    }

    #[test]
    fn missing_local_model_report_yields_an_abstained_workflow_receipt() {
        let temp = TestLab::new();
        let listed = skills::list(&temp.lab).unwrap();
        let pin =
            pin_from_registry(listed["skills"].as_array().unwrap(), "model-evaluation").unwrap();
        let input_schema = object(
            &[
                ("model_sha256", JsonShape::String),
                ("suite_sha256", JsonShape::String),
            ],
            &[],
        );
        let definition = workflow(
            "model-evaluation-abstain",
            "A synthetic workflow fixture for the explicit no-local-report abstention path.",
            input_schema,
            vec![node(
                "evaluation",
                pin,
                &[],
                &[
                    ("model_sha256", input("model_sha256")),
                    ("suite_sha256", input("suite_sha256")),
                ],
            )],
            &[("evaluation", output("evaluation"))],
        );
        let record = register(&temp.lab, definition).unwrap();
        approve_workflow(&temp.lab, &record);
        let result = execute(
            &temp.lab,
            request(
                &record,
                json!({"model_sha256":"a".repeat(64),"suite_sha256":"b".repeat(64)}),
            ),
            &AtomicBool::new(false),
        )
        .unwrap();
        assert_eq!(result["receipt"]["outcome"], "abstained");
        assert_eq!(result["receipt"]["nodes"][0]["state"], "abstained");
        assert_eq!(result["output"]["node_states"]["evaluation"], "abstained");
        assert_eq!(result["receipt"]["evidence_status"], "unverified");
    }

    #[test]
    fn revoked_workflow_and_skill_approvals_fail_closed_and_cancellation_is_cooperative() {
        let temp = TestLab::new();
        let record = workflow_record(&temp.lab, "hypothesis-design");
        approve_workflow(&temp.lab, &record);
        let contracts =
            skills::workflow_skill_closure(&temp.lab, &pins(&record.definition)).unwrap();
        skills::revoke(&temp.lab, &contracts[0].sha256).unwrap();
        let revoked_skill = execute(
            &temp.lab,
            request(&record, sample_input(&record, &temp.root_id())),
            &AtomicBool::new(false),
        )
        .unwrap();
        assert_eq!(revoked_skill["receipt"]["failure"], "skill_revoked");

        let another = TestLab::new();
        let record = workflow_record(&another.lab, "hypothesis-design");
        approve_workflow(&another.lab, &record);
        revoke(&another.lab, &record.sha256).unwrap();
        let revoked = execute(
            &another.lab,
            request(&record, sample_input(&record, &another.root_id())),
            &AtomicBool::new(false),
        )
        .unwrap();
        assert_eq!(revoked["receipt"]["failure"], "revoked_workflow");

        let stale_lab = TestLab::new();
        let initial = workflow_record(&stale_lab.lab, "hypothesis-design");
        approve_workflow(&stale_lab.lab, &initial);
        let mut stale_record = workflow_record(&stale_lab.lab, "hypothesis-design");
        let mut approval: ApprovalReceipt = artifact_receipt(
            &stale_lab.lab,
            stale_record.approval_receipt_sha256.as_deref().unwrap(),
        )
        .unwrap();
        approval.skill_approvals[0].approval_receipt_sha256 = "f".repeat(64);
        let bytes = serde_json::to_vec(&approval).unwrap();
        let approval_pin = stale_lab.lab.record_development_bytes(&bytes).unwrap();
        stale_record.approval_receipt_sha256 = Some(approval_pin);
        let registry = stale_lab.lab.model_registry().unwrap();
        Registry::save(
            &registry.0,
            "research-workflow",
            &version_key(
                &stale_record.definition.id,
                &stale_record.definition.version,
            ),
            &stale_record.sha256,
            &stale_record,
        )
        .unwrap();
        let stale = execute(
            &stale_lab.lab,
            request(
                &stale_record,
                sample_input(&stale_record, &stale_lab.root_id()),
            ),
            &AtomicBool::new(false),
        )
        .unwrap();
        assert_eq!(stale["receipt"]["failure"], "stale_approval");

        let cancellation = TestLab::new();
        let initial = workflow_record(&cancellation.lab, "hypothesis-design");
        approve_workflow(&cancellation.lab, &initial);
        let record = workflow_record(&cancellation.lab, "hypothesis-design");
        let order = validate_definition(&cancellation.lab, &record.definition).unwrap();
        let (_, skill_approvals) = approval_for(&cancellation.lab, &record).unwrap();
        let contracts =
            skills::workflow_skill_closure(&cancellation.lab, &pins(&record.definition))
                .unwrap()
                .into_iter()
                .map(|contract| (contract.sha256.clone(), contract))
                .collect();
        let cancel = AtomicBool::new(false);
        let result = run_nodes(
            RunContext {
                lab: &cancellation.lab,
                record: &record,
                request: &request(&record, sample_input(&record, &cancellation.root_id())),
                skill_approvals,
                order: &order,
                contracts: &contracts,
                cancel: &cancel,
            },
            |skill, input, depth, output_bytes, cancel| {
                let result = skills::execute_workflow_node(
                    &cancellation.lab,
                    skills::WorkflowNodeRequest {
                        skill_id: skill.id.clone(),
                        skill_version: skill.version.clone(),
                        skill_sha256: skill.sha256.clone(),
                        input,
                        dependency_depth: depth,
                        output_bytes,
                    },
                    cancel,
                );
                cancel.store(true, Ordering::Release);
                result
            },
        )
        .unwrap();
        assert_eq!(result["receipt"]["outcome"], "cancelled");
        assert_eq!(
            result["receipt"]["execution_order"]
                .as_array()
                .unwrap()
                .len(),
            1
        );
        assert_eq!(result["receipt"]["nodes"][1]["state"], "skipped");
    }

    #[test]
    fn oversized_inputs_modified_pins_and_tampered_execution_receipts_are_rejected() {
        let temp = TestLab::new();
        let record = workflow_record(&temp.lab, "literature-hypothesis");
        approve_workflow(&temp.lab, &record);
        let mut too_large = sample_input(&record, &temp.root_id());
        too_large["question"] = json!("x".repeat(MAX_INPUT_BYTES as usize + 1));
        let oversized = execute(
            &temp.lab,
            request(&record, too_large),
            &AtomicBool::new(false),
        )
        .unwrap();
        assert_eq!(oversized["receipt"]["failure"], "input_too_large");

        let mut changed = request(&record, sample_input(&record, &temp.root_id()));
        changed.workflow.sha256 = "0".repeat(64);
        let tampered = execute(&temp.lab, changed, &AtomicBool::new(false)).unwrap();
        assert_eq!(tampered["receipt"]["failure"], "unknown_workflow");

        let done = execute(
            &temp.lab,
            request(&record, sample_input(&record, &temp.root_id())),
            &AtomicBool::new(false),
        )
        .unwrap();
        let registry = temp.lab.model_registry().unwrap();
        registry
            .0
            .execute(
                "UPDATE model_registry SET record='{}' WHERE kind='workflow-execution' AND sha256=?1",
                [done["workflow_execution_sha256"].as_str().unwrap()],
            )
            .unwrap();
        assert!(
            executions(&temp.lab)
                .unwrap_err()
                .to_string()
                .contains("integrity")
        );
    }
}
