mod archive;
mod experiment_design;
#[cfg(test)]
mod tests;

use super::*;

const DEPTH: u8 = 2;
const MAX_PROPOSALS: usize = 12;
const MAX_REFERENCES: usize = 6;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct ComparisonScope {
    task: String,
    case_sha256: String,
    profile: String,
    size: u16,
    metric: String,
    treatment: String,
    control: String,
    budget: u16,
}

impl ComparisonScope {
    fn valid(&self) -> bool {
        valid_hash(&self.case_sha256)
            && (1..=1024).contains(&self.size)
            && (1..=96).contains(&self.budget)
            && self.treatment != self.control
            && [
                &self.task,
                &self.profile,
                &self.metric,
                &self.treatment,
                &self.control,
            ]
            .into_iter()
            .all(|text| !text.is_empty() && text.len() <= 64 && !text.chars().any(char::is_control))
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Comparison {
    scope: ComparisonScope,
    treatment_gap: u64,
    control_gap: u64,
}

impl Comparison {
    fn direction(&self) -> std::cmp::Ordering {
        self.control_gap.cmp(&self.treatment_gap)
    }
}

#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "snake_case")]
enum Generator {
    Contradiction,
    EvidenceGap,
    Boundary,
    NegativeResult,
    MethodComposition,
    CrossDomain,
}

#[derive(Clone, Debug, Serialize)]
struct Reference<'a> {
    node: &'a str,
    revision_sha256: &'a str,
    kind: Kind,
    status: Status,
    source: &'a Source,
    source_group: &'a str,
    statement: &'a str,
}

#[derive(Debug, Serialize)]
struct Ranking {
    testability: i16,
    evidence_quality: i16,
    utility: i16,
    uncertainty: i16,
    estimated_cost: i16,
    score: i16,
}

#[derive(Debug, Serialize)]
struct Resources {
    unit: &'static str,
    conditions: u16,
    repetitions: u16,
    evaluations_per_condition: u16,
    total_evaluations: u32,
    note: &'static str,
}

#[derive(Clone, Copy)]
struct Design {
    generator: Generator,
    proposition: &'static str,
    alternative_explanations: &'static [&'static str],
    assumptions: &'static [&'static str],
    limitations: &'static [&'static str],
    falsification: &'static str,
    metrics: &'static [&'static str],
    resources: (u16, u16, u16),
    testability: i16,
    utility: i16,
    uncertainty: i16,
    unit: &'static str,
}

#[derive(Debug, Serialize)]
struct Proposal<'a> {
    id: String,
    generator: Generator,
    status: &'static str,
    proposition: &'static str,
    supports: Vec<Reference<'a>>,
    contradictory_evidence: Vec<Reference<'a>>,
    context: Vec<Reference<'a>>,
    alternative_explanations: &'static [&'static str],
    assumptions: &'static [&'static str],
    limitations: &'static [&'static str],
    falsification: &'static str,
    metrics: &'static [&'static str],
    resources: Resources,
    ranking: Ranking,
}

#[derive(Default, Debug, Serialize)]
struct Diagnostics {
    unsupported_claims: usize,
    unqualified_conflicts: usize,
    comparable_conflicts: usize,
    annotated_conflicts: usize,
    duplicate_candidates: usize,
    controlled_wins: usize,
    controlled_ties: usize,
    controlled_losses: usize,
}

struct Discovery<'a> {
    graph: &'a GraphSlice,
    nodes: BTreeMap<&'a str, &'a Node>,
    revisions: BTreeMap<(&'a str, &'a str), &'a Revision>,
    proposals: BTreeMap<String, Proposal<'a>>,
    diagnostics: Diagnostics,
}

fn quality(reference: &Reference<'_>) -> i16 {
    match reference.source {
        Source::Passage { .. } | Source::Archive { .. } => 2,
        Source::Campaign { .. } if reference.status == Status::IndependentlyVerified => 3,
        Source::Campaign { .. } => 1,
        Source::Metadata { .. } | Source::Operator => 0,
    }
}

fn unique_sources<'a>(references: Vec<Reference<'a>>) -> Vec<Reference<'a>> {
    let mut kept = Vec::new();
    let mut ids = BTreeSet::new();
    for reference in references {
        if ids.insert((reference.node, reference.revision_sha256)) {
            kept.push(reference);
        }
        if kept.len() == MAX_REFERENCES {
            break;
        }
    }
    kept
}

fn evidence_quality(references: &[Reference<'_>]) -> i16 {
    let mut grades = BTreeMap::new();
    let mut passages = BTreeSet::new();
    for reference in references {
        let grade = quality(reference);
        if grade == 0 {
            continue;
        }
        if matches!(reference.source, Source::Passage { .. })
            && !passages.insert(reference.statement)
        {
            continue;
        }
        grades
            .entry(reference.source_group)
            .and_modify(|g: &mut i16| *g = (*g).max(grade))
            .or_insert(grade);
    }
    grades.values().copied().max().unwrap_or(0) + i16::from(grades.len() >= 2)
}

impl<'a> Discovery<'a> {
    fn new(graph: &'a GraphSlice) -> Self {
        let nodes = graph
            .nodes
            .iter()
            .map(|node| (node.id.as_str(), node))
            .collect();
        let revisions = graph
            .nodes
            .iter()
            .map(|node| {
                (
                    (node.id.as_str(), node.current.sha256.as_str()),
                    &node.current,
                )
            })
            .chain(
                graph
                    .root_history
                    .iter()
                    .map(|revision| ((graph.root.as_str(), revision.sha256.as_str()), revision)),
            )
            .chain(graph.linked_history.iter().map(|history| {
                (
                    (history.node.as_str(), history.revision.sha256.as_str()),
                    &history.revision,
                )
            }))
            .collect();
        Self {
            graph,
            nodes,
            revisions,
            proposals: BTreeMap::new(),
            diagnostics: Diagnostics::default(),
        }
    }

    fn reference(&self, node: &str, sha256: &str) -> Result<Reference<'a>, LiteratureError> {
        let node = self
            .nodes
            .get(node)
            .ok_or(LiteratureError::MissingGraphNode)?;
        let rev = self.revisions.get(&(node.id.as_str(), sha256)).ok_or(
            LiteratureError::InvalidGraph("graph export omitted an endpoint revision"),
        )?;
        Ok(Reference {
            node: &node.id,
            revision_sha256: &rev.sha256,
            kind: node.kind,
            status: rev.observation.status,
            source: &rev.observation.source,
            source_group: &rev.source_group,
            statement: &rev.observation.statement,
        })
    }

    fn insert(
        &mut self,
        design: Design,
        identity: impl Serialize,
        supports: Vec<Reference<'a>>,
        contradictory_evidence: Vec<Reference<'a>>,
        context: Vec<Reference<'a>>,
    ) -> Result<(), LiteratureError> {
        let supports = unique_sources(
            supports
                .into_iter()
                .filter(|reference| {
                    quality(reference) > 0 && !reference.statement.trim().is_empty()
                })
                .collect(),
        );
        if supports.is_empty() {
            return Ok(());
        }
        let id = digest(&("aether.hypothesis/1", design.generator, identity))?;
        if self.proposals.contains_key(&id) {
            self.diagnostics.duplicate_candidates += 1;
            return Ok(());
        }
        let evidence_quality = evidence_quality(&supports);
        let (conditions, repetitions, evaluations_per_condition) = design.resources;
        let total_evaluations = u32::from(conditions)
            .checked_mul(repetitions.into())
            .and_then(|total| total.checked_mul(evaluations_per_condition.into()))
            .ok_or(LiteratureError::InvalidGraph(
                "proposal resource estimate overflow",
            ))?;
        let estimated_cost = match total_evaluations {
            0..=12 => 1,
            13..=64 => 2,
            65..=320 => 3,
            _ => 4,
        };
        let uncertainty = (design.uncertainty
            + i16::from(evidence_quality < 3)
            + i16::from(self.graph.truncated))
        .min(4);
        let ranking = Ranking {
            testability: design.testability,
            evidence_quality,
            utility: design.utility,
            uncertainty,
            estimated_cost,
            score: 3 * design.testability + 3 * evidence_quality + 2 * design.utility
                - 2 * uncertainty
                - estimated_cost,
        };
        self.proposals.insert(id.clone(), Proposal {
            id, generator: design.generator, status: "unverified", proposition: design.proposition,
            supports, contradictory_evidence: unique_sources(contradictory_evidence), context: unique_sources(context),
            alternative_explanations: design.alternative_explanations, assumptions: design.assumptions,
            limitations: design.limitations, falsification: design.falsification,
            metrics: design.metrics,
            resources: Resources { unit: design.unit, conditions, repetitions, evaluations_per_condition, total_evaluations,
                note: "Planning estimate. Runtime and memory require a separate trial and review." },
            ranking,
        });
        Ok(())
    }

    fn claims(&mut self) -> Result<(), LiteratureError> {
        for node in &self.graph.nodes {
            if !matches!(node.kind, Kind::Claim | Kind::Hypothesis) {
                continue;
            }
            let mut supports = Vec::new();
            let mut objections = Vec::new();
            let mut independently_checked = false;
            for edge in self.graph.edges.iter().filter(|edge| {
                edge.target == node.id && edge.target_revision == node.current.sha256
            }) {
                match edge.relation {
                    Relation::Supports => {
                        supports.push(self.reference(&edge.origin, &edge.origin_revision)?)
                    }
                    Relation::Contradicts => {
                        objections.push(self.reference(&edge.origin, &edge.origin_revision)?)
                    }
                    Relation::Evaluates => {
                        let origin = self.reference(&edge.origin, &edge.origin_revision)?;
                        // A receipt superseded by another capture cannot close a current gap.
                        independently_checked |= origin.status == Status::IndependentlyVerified
                            && self.nodes[origin.node].current.sha256 == origin.revision_sha256;
                    }
                    _ => {}
                }
            }
            if !objections.is_empty() {
                // A persuasive label or an asserted edge does not establish matching test conditions.
                self.diagnostics.unqualified_conflicts += 1;
            }
            let claim = self.reference(&node.id, &node.current.sha256)?;
            if quality(&claim) > 0 && !claim.statement.trim().is_empty() {
                supports.insert(0, claim.clone());
            }
            if !supports
                .iter()
                .any(|reference| quality(reference) > 0 && !reference.statement.trim().is_empty())
            {
                self.diagnostics.unsupported_claims += 1;
            } else if !independently_checked && !claim.statement.trim().is_empty() {
                self.insert(
                    GAP,
                    (&node.id, &node.current.sha256),
                    supports,
                    objections,
                    vec![claim],
                )?;
            }
        }
        Ok(())
    }

    fn controlled_comparisons(&mut self) -> Result<(), LiteratureError> {
        use std::cmp::Ordering::{Equal, Greater, Less};
        let mut observed = Vec::new();
        for node in self.graph.nodes.iter().filter(|node| {
            node.kind == Kind::Metric
                && matches!(node.current.observation.source, Source::Archive { .. })
        }) {
            let comparison: Comparison = serde_json::from_str(&node.current.observation.statement)?;
            let reference = self.reference(&node.id, &node.current.sha256)?;
            observed.push((comparison, reference));
        }
        // Compare only archived designed contrasts. Matching text and observational links
        // cannot enter this path. One source may report many non-independent cases.
        let mut unique = BTreeSet::new();
        observed.retain(|(comparison, reference)| {
            unique.insert((
                reference.source_group,
                comparison.scope.case_sha256.clone(),
                comparison.scope.metric.clone(),
                comparison.scope.budget,
                comparison.scope.treatment.clone(),
                comparison.scope.control.clone(),
                comparison.treatment_gap,
                comparison.control_gap,
            ))
        });
        for (comparison, _) in &observed {
            match comparison.direction() {
                Greater => self.diagnostics.controlled_wins += 1,
                Equal => self.diagnostics.controlled_ties += 1,
                Less => self.diagnostics.controlled_losses += 1,
            }
        }
        let mut qualified = BTreeSet::new();
        for (win, positive) in observed
            .iter()
            .filter(|(comparison, _)| comparison.direction() == Greater)
        {
            for (other, counter) in observed
                .iter()
                .filter(|(comparison, _)| comparison.direction() != Greater)
            {
                if win.scope == other.scope
                    && other.direction() == Less
                    && positive.source_group != counter.source_group
                {
                    if qualified.insert(digest(&win.scope)?) {
                        self.diagnostics.comparable_conflicts += 1;
                    }
                    let design = Design {
                        resources: (4, 3, win.scope.budget),
                        ..CONTRADICTION
                    };
                    self.insert(
                        design,
                        &win.scope,
                        vec![positive.clone()],
                        vec![counter.clone()],
                        vec![],
                    )?;
                }
            }
        }
        type DesignKey<'a> = (&'a str, &'a str, &'a str, &'a str, u16);
        let mut regimes: BTreeMap<DesignKey<'_>, Vec<_>> = BTreeMap::new();
        let mut profiles = BTreeMap::<_, BTreeMap<_, Vec<_>>>::new();
        for (comparison, reference) in &observed {
            let scope = &comparison.scope;
            let key: DesignKey<'_> = (
                &scope.task,
                &scope.metric,
                &scope.treatment,
                &scope.control,
                scope.budget,
            );
            let point = (comparison.direction(), reference);
            regimes.entry(key).or_default().push(point);
            profiles
                .entry((key, scope.size))
                .or_default()
                .entry(&scope.profile)
                .or_default()
                .push(point);
        }
        for ((key, size), profiles) in profiles {
            let profiles: Vec<_> = profiles
                .into_iter()
                .map(|(profile, points)| {
                    let wins = points.iter().filter(|(order, _)| *order == Greater).count();
                    (profile, points, wins)
                })
                .collect();
            for (index, (first, a, aw)) in profiles.iter().enumerate() {
                for (second, b, bw) in &profiles[index + 1..] {
                    let left = aw * b.len();
                    let right = bw * a.len();
                    if left.abs_diff(right) * 10 < a.len() * b.len() {
                        continue;
                    }
                    let (favored, higher, contrast, lower) = if left > right {
                        (first, a, second, b)
                    } else {
                        (second, b, first, a)
                    };
                    let supports = higher
                        .iter()
                        .filter(|(order, _)| *order == Greater)
                        .map(|(_, reference)| (*reference).clone())
                        .collect();
                    let contrary = higher
                        .iter()
                        .filter(|(order, _)| *order != Greater)
                        .chain(lower.iter().filter(|(order, _)| *order == Greater))
                        .map(|(_, reference)| (*reference).clone())
                        .collect();
                    let context = lower
                        .iter()
                        .map(|(_, reference)| (*reference).clone())
                        .collect();
                    let design = Design {
                        resources: (4, 12, key.4),
                        ..BOUNDARY
                    };
                    self.insert(
                        design,
                        (key, size, favored, contrast),
                        supports,
                        contrary,
                        context,
                    )?;
                }
            }
        }
        for (key, observations) in regimes {
            let wins = observations
                .iter()
                .filter(|(direction, _)| *direction == Greater)
                .count();
            if observations.len() >= 4 && wins * 2 < observations.len() {
                let mut negative: Vec<_> = observations
                    .iter()
                    .filter(|(direction, _)| *direction != Greater)
                    .collect();
                // Retain strict losses as well as ties in the display limit.
                negative.sort_by_key(|(direction, _)| *direction);
                let supports = negative
                    .into_iter()
                    .map(|(_, reference)| (*reference).clone())
                    .collect();
                let objections = observations
                    .iter()
                    .filter(|(direction, _)| *direction == Greater)
                    .map(|(_, reference)| (*reference).clone())
                    .collect();
                let design = Design {
                    resources: (2, 20, key.4),
                    ..NEGATIVE
                };
                self.insert(design, key, supports, objections, vec![])?;
            }
        }
        Ok(())
    }

    fn reported_comparisons(&mut self) -> Result<(), LiteratureError> {
        use std::cmp::Ordering::{Greater, Less};
        let mut candidates = Vec::new();
        let mut seen = BTreeSet::new();
        for edge in self
            .graph
            .edges
            .iter()
            .filter(|edge| edge.relation == Relation::DerivedFrom)
        {
            let annotation = self.reference(&edge.origin, &edge.origin_revision)?;
            if !matches!(annotation.source, Source::Operator)
                || self.nodes[annotation.node].current.sha256 != annotation.revision_sha256
            {
                continue;
            }
            let Ok(comparison) = serde_json::from_str::<Comparison>(annotation.statement) else {
                continue;
            };
            let source = self.reference(&edge.target, &edge.target_revision)?;
            if comparison.scope.valid()
                && quality(&source) > 0
                && !source.statement.trim().is_empty()
                && seen.insert((annotation.node, source.source_group))
            {
                candidates.push((comparison, source, annotation));
            }
        }
        let mut qualified = BTreeSet::new();
        for (positive, source, annotation) in
            candidates.iter().filter(|(c, ..)| c.direction() == Greater)
        {
            for (negative, counter, other_annotation) in
                candidates.iter().filter(|(c, ..)| c.direction() == Less)
            {
                if positive.scope != negative.scope
                    || source.source_group == counter.source_group
                    || source.statement == counter.statement
                {
                    continue;
                }
                if qualified.insert(digest(&positive.scope)?) {
                    self.diagnostics.annotated_conflicts += 1;
                }
                let design = Design {
                    resources: (4, 3, positive.scope.budget),
                    uncertainty: 4,
                    assumptions: &[
                        "Independently check each annotated value and scope against the exact source before planning a rerun.",
                    ],
                    limitations: &[
                        "The matching scope and numeric values were declared by an operator. The engine did not verify those annotations.",
                    ],
                    ..CONTRADICTION
                };
                self.insert(
                    design,
                    &positive.scope,
                    vec![source.clone()],
                    vec![counter.clone()],
                    vec![annotation.clone(), other_annotation.clone()],
                )?;
            }
        }
        Ok(())
    }

    fn methods(&mut self) -> Result<(), LiteratureError> {
        let mut uses: BTreeMap<&str, BTreeMap<&str, Reference<'a>>> = BTreeMap::new();
        let mut domains = BTreeMap::new();
        for edge in &self.graph.edges {
            let origin = self.reference(&edge.origin, &edge.origin_revision)?;
            // Revisions with newer labels or statements need an explicit fresh link.
            if self.nodes[origin.node].current.sha256 != origin.revision_sha256 {
                continue;
            }
            let target = self.reference(&edge.target, &edge.target_revision)?;
            if self.nodes[target.node].current.sha256 != target.revision_sha256 {
                continue;
            }
            match edge.relation {
                Relation::UsesMethod | Relation::Reports
                    if target.kind == Kind::Method && quality(&target) > 0 =>
                {
                    uses.entry(origin.node)
                        .or_default()
                        .entry(target.node)
                        .or_insert(target);
                }
                Relation::UsesDataset if let Some(domain) = declared_domain(target.statement) => {
                    domains.entry(origin.node).or_insert((domain, target));
                }
                _ => {}
            }
        }
        let mut transfers: BTreeMap<&str, BTreeMap<String, Reference<'a>>> = BTreeMap::new();
        for (&anchor, methods) in &uses {
            let context = self.reference(anchor, &self.nodes[anchor].current.sha256)?;
            let methods: Vec<_> = methods.values().collect();
            // One factorial design per anchor keeps this a bounded comparison.
            if let [a, b, ..] = methods.as_slice() {
                self.insert(
                    COMPOSITION,
                    [a.node, b.node],
                    vec![(*a).clone(), (*b).clone()],
                    vec![],
                    vec![context],
                )?;
            }
            if let Some((domain, dataset)) = domains.get(anchor) {
                for method in methods {
                    transfers
                        .entry(method.node)
                        .or_default()
                        .insert(domain.clone(), dataset.clone());
                }
            }
        }
        for (method, domains) in transfers {
            if let [a, b, ..] = domains.values().collect::<Vec<_>>().as_slice()
                && a.node != b.node
            {
                let source = self.reference(method, &self.nodes[method].current.sha256)?;
                self.insert(
                    TRANSFER,
                    [method, a.node, b.node],
                    vec![source],
                    vec![],
                    vec![(*a).clone(), (*b).clone()],
                )?;
            }
        }
        Ok(())
    }
}

fn declared_domain(statement: &str) -> Option<String> {
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Domain {
        domain: String,
    }
    let domain = serde_json::from_str::<Domain>(statement).ok()?.domain;
    (!domain.is_empty()
        && domain.len() <= 64
        && domain
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-'))
    .then_some(domain)
}

const GAP: Design = Design {
    generator: Generator::EvidenceGap,
    proposition: "Two reviewers working independently will agree on a measurable endpoint and a disconfirming observation for this pinned claim.",
    alternative_explanations: &[
        "Scope, population, comparator or outcome may be ambiguous.",
        "An independent check may exist outside the scanned graph.",
    ],
    assumptions: &["Reviewers receive the same exact passages and a predefined coding rubric."],
    limitations: &[
        "Agreement measures operational clarity. Retrieve any missing methods and check prior work before designing replication.",
    ],
    falsification: "The reviewers disagree on either the endpoint or the disconfirming condition, or either identifies insufficient source detail.",
    metrics: &[
        "Endpoint agreement",
        "Falsification-condition agreement",
        "Unresolved rubric items",
    ],
    resources: (1, 2, 1),
    testability: 3,
    utility: 2,
    uncertainty: 3,
    unit: "reviewer assessments",
};
const CONTRADICTION: Design = Design {
    generator: Generator::Contradiction,
    proposition: "Rerunning the two recorded implementations under the same pinned conditions will reproduce opposite signs of the paired treatment effect.",
    alternative_explanations: &[
        "Version drift, a calculation error, or random variation could explain the disagreement.",
    ],
    assumptions: &[
        "Fix the case, comparator, endpoint, evaluation budget and seed schedule before the rerun.",
    ],
    limitations: &[
        "Matching scope fields do not establish that both sources used identical executable builds.",
    ],
    falsification: "Both arms agree on the sign or either paired effect is zero on the predeclared reruns.",
    metrics: &[
        "Sign of the paired difference",
        "Reproducibility of the archived effect",
    ],
    resources: (4, 3, 8),
    testability: 4,
    utility: 4,
    uncertainty: 2,
    unit: "candidate evaluations",
};
const BOUNDARY: Design = Design {
    generator: Generator::Boundary,
    proposition: "Across fresh matched cases of the same size and budget, the treatment wins at least 10 percentage points more often in the supporting profile than in the comparison profile.",
    alternative_explanations: &[
        "Case construction or seed selection may account for the difference.",
        "The profile label may proxy for several interacting factors.",
    ],
    assumptions: &[
        "Prespecify profile assignment. Sample cases without access to treatment outcomes and keep the comparator and budget fixed.",
    ],
    limitations: &[
        "The archived profile association motivates an interaction test. It does not identify the cause of the difference.",
    ],
    falsification: "The supporting profile's observed winning fraction exceeds the comparison profile's by less than 10 percentage points.",
    metrics: &[
        "Strict treatment winning fraction by profile",
        "Uncertainty interval for the difference in winning fractions",
    ],
    resources: (4, 12, 8),
    testability: 4,
    utility: 3,
    uncertainty: 3,
    unit: "candidate evaluations",
};
const NEGATIVE: Design = Design {
    generator: Generator::NegativeResult,
    proposition: "At the pinned evaluation budget, the treatment will strictly outperform the control on fewer than half of 20 fresh paired cases sampled from the recorded instance mix.",
    alternative_explanations: &[
        "The archived mix may be easier than the target population.",
        "A larger budget could change the winning fraction.",
    ],
    assumptions: &["Match both methods on the same cases, scoring rule, budget and seed schedule."],
    limitations: &[
        "A tie is not evidence of equivalence. Cases from the same archive are one provenance group.",
    ],
    falsification: "At least 10 of 20 fresh matched cases show a strictly smaller optimality gap under the treatment.",
    metrics: &[
        "Strict treatment winning fraction",
        "Treatment ties and losses",
        "Paired optimality-gap difference",
    ],
    resources: (2, 20, 8),
    testability: 4,
    utility: 3,
    uncertainty: 2,
    unit: "candidate evaluations",
};
const COMPOSITION: Design = Design {
    generator: Generator::MethodComposition,
    proposition: "On the same benchmark and budget, using both referenced methods improves the primary endpoint over the better single method by at least one preregistered meaningful unit.",
    alternative_explanations: &[
        "Additional compute, tuning, or a shared component may account for an apparent combination gain.",
    ],
    assumptions: &[
        "Define the measurable endpoint and meaningful margin before data collection. Match costs across baseline, each method alone, and both together.",
    ],
    limitations: &[
        "Shared use is a basis for candidate selection. Component compatibility and interaction effects need testing.",
    ],
    falsification: "The joint arm misses the preregistered margin against the better single-method arm under matched cost.",
    metrics: &[
        "Primary endpoint by arm",
        "Factorial interaction contrast",
        "Actual resource use",
    ],
    resources: (4, 12, 1),
    testability: 3,
    utility: 3,
    uncertainty: 3,
    unit: "benchmark trials",
};
const TRANSFER: Design = Design {
    generator: Generator::CrossDomain,
    proposition: "With matched resources, the shared method improves a preregistered primary endpoint over the target-domain baseline after transfer.",
    alternative_explanations: &[
        "Dataset overlap, leakage or domain-label error may explain the apparent affinity.",
        "The method may require domain-specific tuning.",
    ],
    assumptions: &[
        "Confirm the operator-declared domains, screen for overlap, and fix the target split and comparator in advance.",
    ],
    limitations: &[
        "The graph supplies a candidate connection. Transfer success and domain independence require new evidence.",
    ],
    falsification: "The paired primary endpoint shows no positive improvement in the target domain under matched resources.",
    metrics: &[
        "Paired target-domain metric difference",
        "Dataset overlap audit",
        "Trial cost",
    ],
    resources: (2, 12, 1),
    testability: 3,
    utility: 3,
    uncertainty: 3,
    unit: "benchmark trials",
};

#[derive(Debug, Serialize)]
pub struct HypothesisReport<'a> {
    schema: &'static str,
    root: &'a str,
    graph_sha256: &'a str,
    graph_truncated: bool,
    coverage: &'static str,
    ranking_formula: &'static str,
    scanned_nodes: usize,
    scanned_edges: usize,
    diagnostics: Diagnostics,
    eligible_proposals: usize,
    proposals: Vec<Proposal<'a>>,
}

impl CitationCatalog {
    pub fn discover_hypotheses(&self, root: &str) -> Result<serde_json::Value, LiteratureError> {
        let export = self.graph_export(root, DEPTH)?;
        Ok(serde_json::to_value(discover(&export)?)?)
    }
}

fn discover(export: &GraphExport) -> Result<HypothesisReport<'_>, LiteratureError> {
    let graph = &export.graph;
    let mut engine = Discovery::new(graph);
    engine.claims()?;
    engine.controlled_comparisons()?;
    engine.reported_comparisons()?;
    engine.methods()?;
    let mut proposals: Vec<_> = engine.proposals.into_values().collect();
    proposals.sort_unstable_by(|a, b| b.ranking.score.cmp(&a.ranking.score).then(a.id.cmp(&b.id)));
    let eligible_proposals = proposals.len();
    proposals.truncate(MAX_PROPOSALS);
    Ok(HypothesisReport {
        schema: "aether.hypotheses/1",
        root: &graph.root,
        graph_sha256: &export.graph_sha256,
        graph_truncated: graph.truncated,
        coverage: "Only this two-hop local graph slice was examined. Missing records cannot establish novelty, absence of prior evidence, or a verified result.",
        ranking_formula: "3×testability + 3×evidence_quality + 2×utility − 2×uncertainty − estimated_cost. Grades are heuristic, not probabilities.",
        scanned_nodes: graph.nodes.len(),
        scanned_edges: graph.edges.len(),
        diagnostics: engine.diagnostics,
        eligible_proposals,
        proposals,
    })
}
