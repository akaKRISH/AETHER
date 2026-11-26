use serde_json::{Value, json};

use super::*;
use crate::literature::knowledge::projection::edge;

pub(super) const ARTIFACT: &str = "dae-hybrid-development";
pub(super) const PIN: &str = "bc3654d2eb279884cca74f9a5d7f4fafde60e10930bc6cd06338eec256736ced";
pub(super) const ROOT_IDENTITY: &str = "archive:dae-hybrid-development:budget-8";
const BYTES: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../research/results/dae-hybrid-development.json"
));
const BUDGET: u16 = 8;

#[derive(Deserialize)]
struct Archive {
    scope: String,
    evidence: Vec<Case>,
}

#[derive(Deserialize)]
struct Case {
    case: Value,
    records: Vec<Arm>,
}

#[derive(Deserialize)]
struct Arm {
    arm: String,
    evaluation_budget: u16,
    optimality_gap: u64,
}

impl Case {
    fn arm(&self, name: &str) -> Result<(usize, &Arm), LiteratureError> {
        let mut matches = self
            .records
            .iter()
            .enumerate()
            .filter(|(_, record)| record.arm == name && record.evaluation_budget == BUDGET);
        let found = matches.next().ok_or(LiteratureError::InvalidResponse)?;
        if matches.next().is_some() {
            return Err(LiteratureError::InvalidResponse);
        }
        Ok(found)
    }
}

impl CitationCatalog {
    /// Materialize only the reviewed development evidence. Graph import never starts a trial.
    pub fn index_hypothesis_archive(&mut self) -> Result<Value, LiteratureError> {
        if sha256(BYTES) != PIN {
            return Err(LiteratureError::InvalidResponse);
        }
        let archive: Archive = serde_json::from_slice(BYTES)?;
        if archive.scope != "aether.knapsack-hybrid-development/1" || archive.evidence.len() != 20 {
            return Err(LiteratureError::InvalidResponse);
        }
        let source = |pointer: String| Source::Archive {
            artifact: ARTIFACT.into(),
            artifact_sha256: PIN.into(),
            pointer,
        };
        let observation = |label: String, statement: String, pointer: String| Observation {
            label,
            statement,
            status: Status::Observed,
            source: source(pointer),
            confidence_bps: None,
        };
        let tx = self
            .db
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let root = save_node(
            &tx,
            Kind::Experiment,
            ROOT_IDENTITY,
            observation(
                "DAE-04 matched hybrid search (8 evaluations)".into(),
                archive.scope,
                "/scope".into(),
            ),
        )?;
        let mut cases = BTreeSet::new();
        for (index, entry) in archive.evidence.iter().enumerate() {
            let case_sha256 = digest(&entry.case)?;
            if !cases.insert(case_sha256.clone()) {
                return Err(LiteratureError::InvalidResponse);
            }
            let profile = entry.case["difficulty"]
                .as_str()
                .ok_or(LiteratureError::InvalidResponse)?;
            let size = entry.case["size"]
                .as_u64()
                .and_then(|size| u16::try_from(size).ok())
                .ok_or(LiteratureError::InvalidResponse)?;
            let (_, treatment) = entry.arm("adaptive_hybrid")?;
            let (_, control) = entry.arm("greedy")?;
            let comparison = Comparison {
                scope: ComparisonScope {
                    task: "development knapsack".into(),
                    case_sha256,
                    profile: profile.into(),
                    size,
                    metric: "optimality_gap".into(),
                    treatment: treatment.arm.clone(),
                    control: control.arm.clone(),
                    budget: BUDGET,
                },
                treatment_gap: treatment.optimality_gap,
                control_gap: control.optimality_gap,
            };
            let pointer = format!("/evidence/{index}");
            let metric = save_node(
                &tx,
                Kind::Metric,
                &format!("archive:{ARTIFACT}:{index}:budget-8"),
                observation(
                    format!("DAE-04 pair {index} ({profile}, {size} items)"),
                    serde_json::to_string(&comparison)?,
                    pointer.clone(),
                ),
            )?;
            save_edge(
                &tx,
                edge(
                    &root,
                    &metric,
                    Relation::Measures,
                    Status::Observed,
                    source(pointer),
                    String::new(),
                ),
            )?;
        }
        for name in ["seeded_local", "seeded_repair"] {
            let (index, arm) = archive.evidence[0].arm(name)?;
            let pointer = format!("/evidence/0/records/{index}");
            let method = save_node(
                &tx,
                Kind::Method,
                &format!("archive:{ARTIFACT}:method:{name}"),
                observation(
                    name.into(),
                    format!("Archived named arm {} with separate evaluation", arm.arm),
                    pointer.clone(),
                ),
            )?;
            save_edge(
                &tx,
                edge(
                    &root,
                    &method,
                    Relation::UsesMethod,
                    Status::Observed,
                    source(pointer),
                    String::new(),
                ),
            )?;
        }
        enforce_graph_bounds(&tx)?;
        tx.commit()?;
        Ok(
            json!({"root": root, "artifact": ARTIFACT, "artifact_sha256": PIN, "indexed_pairs": cases.len()}),
        )
    }
}
