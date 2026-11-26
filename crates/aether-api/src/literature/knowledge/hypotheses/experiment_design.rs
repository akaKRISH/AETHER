#[cfg(test)]
mod tests;

use aether_engine::evolution::comparison::{HybridArm, SEEDS, SearchArm};
use aether_engine::{ActionKind, Budget, ResourceVec};
use serde_json::{Value, json};

use super::archive::{ARTIFACT, PIN, ROOT_IDENTITY};
use super::*;

const SCHEMA: &str = "aether.experiment-design/1";
const PER_ARM: u64 = 8;
const MEMORY_MIB: u64 = 64;
const SUMMARY_BYTES: u64 = aether_runner::MAX_OUTPUT;
const SEARCH_MS: u64 = 1_000;
const CHECK_MS: u64 = 250;
const ORACLE_MS: u64 = 1_000;
const MAX_CASES: usize = 24;

fn invalid(reason: &'static str) -> LiteratureError {
    LiteratureError::InvalidGraph(reason)
}

macro_rules! names {
    ($name:ident { $($member:ident),+ $(,)? }) => {
        #[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
        #[serde(rename_all = "snake_case")]
        enum $name { $($member),+ }
    }
}

names!(Recipe {
    NegativeResult,
    Boundary,
    MethodComposition
});
names!(StudyMode {
    Exploratory,
    Confirmatory
});
names!(Approval { Unapproved });
names!(Metric { OptimalityGap });
names!(OraclePhase {
    AfterAllCandidatesFrozen,
    DuringSearch
});
names!(Profile {
    Relaxed,
    TightCorrelated,
    DensityFriendlyAnalog,
    DensityTrapAnalog
});

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum Arm {
    Greedy,
    AdaptiveHybrid,
    Seeded,
    SeededLocal,
    SeededRepair,
}

impl Arm {
    fn registered(self) -> Result<bool, LiteratureError> {
        let label = serde_json::to_value(self)?;
        let registered = match self {
            Self::Greedy => serde_json::to_value(SearchArm::ALL)?,
            _ => serde_json::to_value(HybridArm::ALL)?,
        };
        Ok(registered
            .as_array()
            .is_some_and(|names| names.contains(&label)))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
enum Cell {
    Registered {
        arm: Arm,
        candidate_evaluations: u64,
    },
    // A required control cell has no exact equivalent in the current registry.
    MissingJointCell {
        candidate_evaluations: u64,
    },
}

impl Cell {
    fn cap(self) -> u64 {
        match self {
            Self::Registered {
                candidate_evaluations,
                ..
            }
            | Self::MissingJointCell {
                candidate_evaluations,
            } => candidate_evaluations,
        }
    }
}

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct EvidenceRef {
    node: String,
    revision_sha256: String,
    source: Source,
}

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Case {
    block: u16,
    seed: u32,
    size: u16,
    profile: Profile,
}

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct SourcePin {
    path: String,
    sha256: String,
}

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Manifest {
    // These are public development seed assignments, not materialized inputs.
    seed_policy: String,
    case_schedule_sha256: String,
    development_cases: Vec<Case>,
    excluded_archive_seeds: Vec<u32>,
    exclude_archive_input_bytes: bool,
    held_out_commitment: Option<String>,
    held_out_custody: String,
    evaluator_contract: String,
    reference_sources: Vec<SourcePin>,
    execution_task: Option<aether_runner::Task>,
}

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Controls {
    independent_variables: Vec<String>,
    dependent_variable: Metric,
    held_constant: Vec<String>,
    baseline_rationale: String,
    order_policy: String,
    confounders: Vec<String>,
    oracle_phase: OraclePhase,
    candidate_visible_oracle: bool,
}

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Analysis {
    sampling_unit: String,
    sample_size_basis: String,
    seed_blocks: usize,
    power_estimate: Option<u16>,
    primary: String,
    secondary: Vec<String>,
    statistical_plan: String,
    falsification: String,
    termination: String,
}

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Resources {
    candidate_evaluations: u64,
    search_attempts: u64,
    verification_attempts: u64,
    oracle_attempts: u64,
    peak_memory_mib: u64,
    cpu_ms_ceiling: u64,
    concurrency: u8,
    cap: ResourceVec,
    verification_reserve: ResourceVec,
}

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Protocol {
    schema: String,
    recipe: Recipe,
    mode: StudyMode,
    root: String,
    graph_sha256: String,
    hypothesis_id: String,
    proposition: String,
    supports: Vec<EvidenceRef>,
    counterevidence: Vec<EvidenceRef>,
    context: Vec<EvidenceRef>,
    assumptions: Vec<String>,
    arms: Vec<Cell>,
    controls: Controls,
    analysis: Analysis,
    manifest: Manifest,
    resources: Resources,
    criticism: Vec<String>,
    blockers: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Draft {
    protocol_sha256: String,
    approval_status: Approval,
    protocol: Protocol,
}

fn amount(attempts: u64, time: u64) -> Result<ResourceVec, LiteratureError> {
    Ok(ResourceVec {
        attempts,
        wall_ms: attempts
            .checked_mul(time)
            .ok_or(invalid("wall budget overflow"))?,
        output_bytes: attempts
            .checked_mul(SUMMARY_BYTES)
            .ok_or(invalid("output budget overflow"))?,
        ..ResourceVec::default()
    })
}

fn add(a: ResourceVec, b: ResourceVec) -> Result<ResourceVec, LiteratureError> {
    let sum = |a: u64, b: u64| a.checked_add(b).ok_or(invalid("resource budget overflow"));
    Ok(ResourceVec {
        attempts: sum(a.attempts, b.attempts)?,
        wall_ms: sum(a.wall_ms, b.wall_ms)?,
        output_bytes: sum(a.output_bytes, b.output_bytes)?,
        model_tokens: sum(a.model_tokens, b.model_tokens)?,
        paid_microusd: sum(a.paid_microusd, b.paid_microusd)?,
    })
}

fn resources(cases: u64, arms: u64, per_arm: u64) -> Result<Resources, LiteratureError> {
    let searches = cases
        .checked_mul(arms)
        .ok_or(invalid("trial count overflow"))?;
    let evaluations = searches
        .checked_mul(per_arm)
        .ok_or(invalid("candidate budget overflow"))?;
    let reserve = add(amount(searches, CHECK_MS)?, amount(cases, ORACLE_MS)?)?;
    let cap = add(amount(searches, SEARCH_MS)?, reserve)?;
    Ok(Resources {
        candidate_evaluations: evaluations,
        search_attempts: searches,
        verification_attempts: searches,
        oracle_attempts: cases,
        peak_memory_mib: MEMORY_MIB,
        cpu_ms_ceiling: cap.wall_ms,
        concurrency: 1,
        cap,
        verification_reserve: reserve,
    })
}

fn check_budget(estimate: &Resources, expected: &Resources) -> Result<(), LiteratureError> {
    if estimate != expected {
        return Err(invalid(
            "resource limits do not cover the matched grid and independent evaluation",
        ));
    }
    let mut budget = Budget::new(estimate.cap, estimate.verification_reserve)
        .map_err(|_| invalid("ECAR verification reserve exceeds its cap"))?;
    for (label, kind, count, limit) in [
        (
            "search",
            ActionKind::Explore,
            estimate.search_attempts,
            SEARCH_MS,
        ),
        (
            "check",
            ActionKind::Replay,
            estimate.verification_attempts,
            CHECK_MS,
        ),
        (
            "oracle",
            ActionKind::Challenge,
            estimate.oracle_attempts,
            ORACLE_MS,
        ),
    ] {
        let unit = amount(1, limit)?;
        for slot in 0..count {
            budget
                .try_reserve(&format!("{label}-{slot}"), kind, unit)
                .map_err(|_| invalid("ECAR budget cannot reserve the planned grid"))?;
        }
    }
    Ok(())
}

fn paired_schedule(recipe: Recipe) -> Result<Vec<Case>, LiteratureError> {
    use Profile::*;
    let distinct = match recipe {
        Recipe::NegativeResult => {
            let mut cells = Vec::new();
            for size in [8, 12, 16] {
                for _ in 0..3 {
                    cells.push((size, vec![Relaxed, TightCorrelated]));
                }
            }
            cells.extend([
                (8, vec![DensityFriendlyAnalog]),
                (8, vec![DensityTrapAnalog]),
            ]);
            cells
        }
        Recipe::Boundary => vec![(8, vec![Relaxed, TightCorrelated]); 12],
        Recipe::MethodComposition => vec![(8, vec![Relaxed, TightCorrelated]); 6],
    };
    let tag = serde_json::to_string(&recipe)?;
    let mut cases = Vec::new();
    let mut used = BTreeSet::from_iter(SEEDS.into_iter().chain([901, 902]));
    for (index, (size, profiles)) in distinct.into_iter().enumerate() {
        let block = index as u16;
        let hash = sha256(format!("aether.apollo.development-seed/1/{tag}/{block}").as_bytes());
        let seed =
            u32::from_str_radix(&hash[..8], 16).map_err(|_| invalid("invalid seed digest"))?;
        if !used.insert(seed) {
            return Err(invalid(
                "development seed collides with an archive or another block",
            ));
        }
        cases.extend(profiles.into_iter().map(|profile| Case {
            block,
            seed,
            size,
            profile,
        }));
    }
    Ok(cases)
}

fn strings(items: &[&str]) -> Vec<String> {
    items.iter().map(|item| (*item).into()).collect()
}

fn registered(arm: Arm) -> Cell {
    Cell::Registered {
        arm,
        candidate_evaluations: PER_ARM,
    }
}

impl Recipe {
    fn arms(self) -> Vec<Cell> {
        match self {
            Self::NegativeResult | Self::Boundary => {
                vec![registered(Arm::Greedy), registered(Arm::AdaptiveHybrid)]
            }
            Self::MethodComposition => vec![
                registered(Arm::Seeded),
                registered(Arm::SeededLocal),
                registered(Arm::SeededRepair),
                Cell::MissingJointCell {
                    candidate_evaluations: PER_ARM,
                },
            ],
        }
    }

    fn analysis(self, blocks: usize, falsification: &str) -> Analysis {
        let (basis, primary, statistical_plan) = match self {
            Self::NegativeResult => (
                "20 development cases across 11 seed blocks. Three blocks per size share the two difficulty profiles. The two analog families have one case each. This is a budgeted design, with no power estimate.",
                "Strict adaptive-hybrid wins out of 20 paired cases using optimality gap.",
                "Report paired gap differences and win/tie/loss counts overall and by stratum. Show seed-block labels. The singleton analog strata lack variance estimates, so use descriptive summaries only.",
            ),
            Self::Boundary => (
                "12 development seed blocks crossed with both profiles. This matches the APO-03 design estimate. Effect-size plausibility and variance are unqualified.",
                "Relaxed minus tight-correlated strict win rate, in percentage points.",
                "Compute the paired difference of the two win indicators within each seed block. Report all block differences and their mean. An optional descriptive block bootstrap requires a reviewed resampling method and independent-block justification.",
            ),
            Self::MethodComposition => (
                "12 development cases from six seed blocks crossed with two profiles. Four factorial cells give 48 search trials. The exact joint cell is pending registration.",
                "Mean paired reduction in optimality gap for the joint arm versus each single arm. Both reductions must reach one objective point.",
                "Give each seed block equal weight, averaging its two profiles. Report both paired reductions and the factorial contrast joint minus local minus repair plus baseline on the gap scale. Summarize by profile. Use no confirmatory significance claim.",
            ),
        };
        Analysis {
            sampling_unit: "Seed block. Arms and profiles sharing the block are repeated measurements.".into(),
            sample_size_basis: basis.into(), seed_blocks: blocks, power_estimate: None, primary: primary.into(),
            secondary: strings(&["Paired optimality gap", "Feasibility", "Operator work and actual resource use"]),
            statistical_plan: statistical_plan.into(), falsification: falsification.into(),
            termination: "Stop at the prespecified grid or the first cap, invalid result, cancellation, input overlap or integrity failure. Retain failure rows and mark an incomplete grid inconclusive. Use no outcome-dependent early stopping.".into(),
        }
    }

    fn controls(self) -> Controls {
        let independent_variables = match self {
            Self::NegativeResult => &["Search method"][..],
            Self::Boundary => &["Search method", "Difficulty profile"][..],
            Self::MethodComposition => &["Local improvement enabled", "Repair enabled"][..],
        };
        let baseline = match self {
            Self::NegativeResult | Self::Boundary => {
                "Use density greedy, the existing matched comparator in DAE-04."
            }
            Self::MethodComposition => {
                "Use seeded search with neither factor and both single-factor arms. Fixed/adaptive hybrid also change operator allocation or constraints and cannot stand in for the joint arm."
            }
        };
        Controls {
            independent_variables: strings(independent_variables), dependent_variable: Metric::OptimalityGap,
            held_constant: strings(&["Exact input and cap shared by all arms in a case", "Same search budget and verifier", "Same seed-stream policy, with no cross-arm state", "Count failed and duplicate candidate proposals", "Optimality gap is exact optimal value less independently scored feasible value. An invalid final result makes the grid inconclusive."]),
            baseline_rationale: baseline.into(),
            order_policy: "Rotate arm order by case index modulo the number of arms. Freeze all results before running any exact oracle on the case.".into(),
            confounders: strings(&["Difficulty combines capacity and value structure", "Profile outcomes share seed blocks", "Candidate counts do not equal operator work", "Cache and process order may affect timing"]),
            oracle_phase: OraclePhase::AfterAllCandidatesFrozen, candidate_visible_oracle: false,
        }
    }
}

impl Protocol {
    fn validate(&self) -> Result<(), LiteratureError> {
        if self.schema != SCHEMA
            || self.mode != StudyMode::Exploratory
            || self.analysis.power_estimate.is_some()
            || self.manifest.held_out_commitment.is_some()
            || self.manifest.held_out_custody != "independent_custody_pending"
        {
            return Err(invalid(
                "confirmatory validation requires separately justified power and independent custody",
            ));
        }
        if self.manifest.execution_task.is_some() {
            return Err(invalid(
                "no registered campaign execution task implements the paired knapsack design",
            ));
        }
        if self.controls.oracle_phase != OraclePhase::AfterAllCandidatesFrozen
            || self.controls.candidate_visible_oracle
            || !self.manifest.exclude_archive_input_bytes
        {
            return Err(invalid(
                "oracle, held-out or archive-overlap control failed",
            ));
        }
        if self.arms != self.recipe.arms() || self.arms.iter().any(|cell| cell.cap() != PER_ARM) {
            return Err(invalid("required arms or matched per-case budgets changed"));
        }
        for cell in &self.arms {
            if let Cell::Registered { arm, .. } = cell
                && !arm.registered()?
            {
                return Err(invalid("unregistered development benchmark arm"));
            }
        }
        let cases = &self.manifest.development_cases;
        if cases.is_empty()
            || cases.len() > MAX_CASES
            || *cases != paired_schedule(self.recipe)?
            || digest(cases)? != self.manifest.case_schedule_sha256
            || cases
                .iter()
                .any(|case| self.manifest.excluded_archive_seeds.contains(&case.seed))
        {
            return Err(invalid(
                "development seed schedule or disjointness check failed",
            ));
        }
        let expected = resources(cases.len() as u64, self.arms.len() as u64, PER_ARM)?;
        check_budget(&self.resources, &expected)?;
        Ok(())
    }
}

fn evidence(references: &[Reference<'_>]) -> Result<Vec<EvidenceRef>, LiteratureError> {
    references
        .iter()
        .map(|reference| match reference.source {
            Source::Archive {
                artifact,
                artifact_sha256,
                ..
            } if artifact == ARTIFACT && artifact_sha256 == PIN => Ok(EvidenceRef {
                node: reference.node.into(),
                revision_sha256: reference.revision_sha256.into(),
                source: reference.source.clone(),
            }),
            _ => Err(invalid("source lacks the reviewed archive pin")),
        })
        .collect()
}

fn reference_sources() -> Vec<SourcePin> {
    [
        (
            "evolution.rs",
            include_str!("../../../../../aether-engine/src/evolution.rs"),
        ),
        (
            "evolution/comparison.rs",
            include_str!("../../../../../aether-engine/src/evolution/comparison.rs"),
        ),
        (
            "evolution/comparison/hybrid.rs",
            include_str!("../../../../../aether-engine/src/evolution/comparison/hybrid.rs"),
        ),
    ]
    .into_iter()
    .map(|(name, source)| SourcePin {
        path: format!("crates/aether-engine/src/{name}"),
        sha256: sha256(source.replace("\r\n", "\n").as_bytes()),
    })
    .collect()
}

fn draft(
    proposal: &Proposal<'_>,
    report: &HypothesisReport<'_>,
    recipe: Recipe,
) -> Result<Draft, LiteratureError> {
    let cases = paired_schedule(recipe)?;
    let arms = recipe.arms();
    let blocks = cases
        .iter()
        .map(|case| case.block)
        .collect::<BTreeSet<_>>()
        .len();
    let mut blockers = strings(&[
        "Campaign task registry has no paired knapsack worker.",
        "Fresh case generation and exact archive-input exclusion must be qualified before review.",
        "The planning ceilings are unmeasured. The matched grid exceeds a single existing campaign's attempt limit.",
        "Confirmatory study requires independent custody, justified power assumptions and separate approval.",
    ]);
    if recipe == Recipe::MethodComposition {
        blockers.push("The exact local-plus-repair factorial cell is unregistered.".into());
    }
    let mut criticism = strings(proposal.alternative_explanations);
    criticism.extend(strings(proposal.limitations));
    criticism.push("The optimality gap depends on an isolated oracle. Charge its work separately and evaluate only frozen outputs.".into());
    if recipe == Recipe::MethodComposition {
        criticism.push("Before registration, verify that all factorial arms share one policy and vary only the two declared factors.".into());
    }
    let protocol = Protocol {
        schema: SCHEMA.into(), recipe, mode: StudyMode::Exploratory, root: report.root.into(),
        graph_sha256: report.graph_sha256.into(), hypothesis_id: proposal.id.clone(), proposition: proposal.proposition.into(),
        supports: evidence(&proposal.supports)?, counterevidence: evidence(&proposal.contradictory_evidence)?, context: evidence(&proposal.context)?,
        assumptions: strings(proposal.assumptions),
        resources: resources(cases.len() as u64, arms.len() as u64, PER_ARM)?, arms,
        controls: recipe.controls(), analysis: recipe.analysis(blocks, proposal.falsification),
        manifest: Manifest {
            seed_policy: "SHA-256 of aether.apollo.development-seed/1/{JSON recipe}/{block}; first four bytes big-endian. Shared within a block, including paired arms. Public development only.".into(),
            case_schedule_sha256: digest(&cases)?, development_cases: cases,
            excluded_archive_seeds: SEEDS.into_iter().chain([901, 902]).collect(),
            exclude_archive_input_bytes: true,
            held_out_commitment: None, held_out_custody: "independent_custody_pending".into(),
            evaluator_contract: "Existing development knapsack independent score and post-search exhaustive oracle. Reference source pins use LF-normalized text. Qualify a fresh input adapter before review.".into(),
            reference_sources: reference_sources(),
            execution_task: None,
        }, criticism, blockers,
    };
    protocol.validate()?;
    Ok(Draft {
        protocol_sha256: digest(&protocol)?,
        approval_status: Approval::Unapproved,
        protocol,
    })
}

impl CitationCatalog {
    fn design_drafts(&self, root: &str) -> Result<(String, Vec<Draft>), LiteratureError> {
        if root != node_id(Kind::Experiment, ROOT_IDENTITY) {
            return Err(invalid(
                "no registered experimental design for this hypothesis source",
            ));
        }
        let exported = self.graph_export(root, super::DEPTH)?;
        let report = discover(&exported)?;
        let designs = report
            .proposals
            .iter()
            .filter_map(|proposal| {
                let recipe = match proposal.generator {
                    Generator::NegativeResult => Recipe::NegativeResult,
                    Generator::Boundary => Recipe::Boundary,
                    Generator::MethodComposition => Recipe::MethodComposition,
                    _ => return None,
                };
                Some(draft(proposal, &report, recipe))
            })
            .collect::<Result<Vec<_>, _>>()?;
        if designs.len() != 3 || report.graph_truncated {
            return Err(invalid("expected the three complete archive design inputs"));
        }
        Ok((report.graph_sha256.into(), designs))
    }

    pub fn design_experiments(&self, root: &str) -> Result<Value, LiteratureError> {
        let (graph_sha256, designs) = self.design_drafts(root)?;
        Ok(
            json!({"schema": "aether.design-review/1", "graph_sha256": graph_sha256, "designs": designs}),
        )
    }

    pub fn validate_experiment_design(&self, record: Value) -> Result<Value, LiteratureError> {
        let draft: Draft = serde_json::from_value(record)?;
        if draft.protocol_sha256 != digest(&draft.protocol)? {
            return Err(invalid("canonical protocol digest mismatch"));
        }
        draft.protocol.validate()?;
        let (_, designs) = self.design_drafts(&draft.protocol.root)?;
        if !designs
            .iter()
            .any(|expected| expected.protocol == draft.protocol)
        {
            return Err(invalid(
                "design no longer matches the pinned graph and registered template",
            ));
        }
        Ok(
            json!({"protocol_sha256": draft.protocol_sha256, "valid_review_draft": true,
            "approval_status": draft.approval_status}),
        )
    }
}
