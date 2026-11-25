use std::fmt;

use serde::{Deserialize, Serialize};

use crate::{
    Action, ActionKind, Budget, BudgetError, Decision, ReservationOutcome, ResourceVec,
    ScheduleContext, ScheduleError, ScoreWeights, StopReason, schedule,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum StudyMode {
    A,
    B,
    C,
    D,
    E,
}

impl StudyMode {
    fn adaptive(self) -> bool {
        matches!(self, Self::C | Self::D)
    }

    fn graph_aware(self) -> bool {
        matches!(self, Self::D | Self::E)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct StudyConfig {
    pub seeds: Vec<u64>,
    pub caps: ResourceVec,
    pub modes: Vec<StudyMode>,
}

impl Default for StudyConfig {
    fn default() -> Self {
        Self {
            seeds: vec![11, 23, 37],
            caps: ResourceVec {
                wall_ms: 120,
                attempts: 12,
                ..ResourceVec::default()
            },
            modes: vec![
                StudyMode::A,
                StudyMode::B,
                StudyMode::C,
                StudyMode::D,
                StudyMode::E,
            ],
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct StudyReport {
    pub protocol: &'static str,
    pub evidence_scope: &'static str,
    pub caps: ResourceVec,
    pub verification_reserve: ResourceVec,
    pub weights: ScoreWeights,
    pub campaigns: Vec<SimulatedCampaign>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SimulatedCampaign {
    pub task: &'static str,
    pub seed: u64,
    pub mode: StudyMode,
    pub usage: ResourceVec,
    pub verified_utility_points: u16,
    pub maximum_utility_points: u16,
    pub verified_claims: u8,
    pub rejected_claims: u8,
    pub inconclusive_claims: u8,
    pub stop_reason: StopReason,
    pub steps: Vec<SimulatedStep>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SimulatedStep {
    pub action_id: String,
    pub kind: ActionKind,
    pub charged: ResourceVec,
    pub check_passed: Option<bool>,
}

#[derive(Debug)]
pub enum SimulationError {
    InvalidConfig(&'static str),
    Budget(BudgetError),
    Schedule(ScheduleError),
    InvalidSelection,
}

impl fmt::Display for SimulationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidConfig(message) => f.write_str(message),
            Self::Budget(error) => write!(f, "simulation budget: {error}"),
            Self::Schedule(error) => write!(f, "simulation schedule: {error}"),
            Self::InvalidSelection => f.write_str("selected simulation action was unavailable"),
        }
    }
}

impl std::error::Error for SimulationError {}

#[derive(Debug, Clone, Copy)]
struct Task {
    id: &'static str,
    keyed_salt: u64,
    dependent: bool,
    challenge: bool,
}

const TASKS: [Task; 3] = [
    Task {
        id: "flat",
        keyed_salt: 0,
        dependent: false,
        challenge: false,
    },
    Task {
        id: "shared-prerequisite",
        keyed_salt: 1,
        dependent: true,
        challenge: false,
    },
    Task {
        id: "challenged-prerequisite",
        keyed_salt: 2,
        dependent: true,
        challenge: true,
    },
];
const POINTS: [u16; 4] = [0, 39, 37, 24];
const TOTAL_POINTS: u16 = 100;

impl Task {
    fn depends_on_root(self, index: usize) -> bool {
        self.dependent && (index == 1 || index == 2)
    }

    fn requires_replay(self, index: usize) -> bool {
        self.depends_on_root(index) || index == 0
    }

    fn requires_challenge(self, index: usize) -> bool {
        self.challenge && index == 0
    }
}

#[derive(Debug, Default, Clone, Copy)]
struct Observation {
    explored: bool,
    replay_passed: Option<bool>,
    challenge_passed: Option<bool>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Verdict {
    Verified,
    Rejected,
    Inconclusive,
}

fn independent_verdict(task: Task, index: usize, observations: &[Observation; 4]) -> Verdict {
    let observation = observations[index];
    if observation.replay_passed == Some(false) || observation.challenge_passed == Some(false) {
        return Verdict::Rejected;
    }
    if !observation.explored
        || (task.requires_replay(index) && observation.replay_passed.is_none())
        || (task.requires_challenge(index) && observation.challenge_passed.is_none())
        || (task.depends_on_root(index)
            && independent_verdict(task, 0, observations) != Verdict::Verified)
    {
        return Verdict::Inconclusive;
    }
    Verdict::Verified
}

/// Run the same bounded synthetic tasks and seeds through all selected policies.
/// Fixtures and costs exercise mechanics and carry no empirical inference.
pub fn simulate(config: &StudyConfig) -> Result<StudyReport, SimulationError> {
    if config.seeds.is_empty()
        || config.seeds.len() > 64
        || config.modes.is_empty()
        || config.modes.len() > 5
    {
        return Err(SimulationError::InvalidConfig(
            "choose 1–64 seeds and 1–5 comparison modes",
        ));
    }
    let mut seeds = config.seeds.clone();
    seeds.sort_unstable();
    let mut modes = config.modes.clone();
    modes.sort_unstable();
    if seeds.windows(2).any(|pair| pair[0] == pair[1])
        || modes.windows(2).any(|pair| pair[0] == pair[1])
    {
        return Err(SimulationError::InvalidConfig(
            "seeds and comparison modes must be unique",
        ));
    }
    let budget = Budget::with_quarter_reserve(config.caps);
    let weights = ScoreWeights::default();
    let mut campaigns = Vec::new();
    for task in TASKS {
        for &seed in &seeds {
            for &mode in &modes {
                campaigns.push(simulate_campaign(
                    task,
                    seed,
                    mode,
                    budget.clone(),
                    weights,
                )?);
            }
        }
    }
    Ok(StudyReport {
        protocol: "ecar-synthetic-v1",
        evidence_scope: "synthetic policy simulation only; no model, physical measurements, or scientific findings",
        caps: config.caps,
        verification_reserve: budget.verification_reserve(),
        weights,
        campaigns,
    })
}

fn simulate_campaign(
    task: Task,
    seed: u64,
    mode: StudyMode,
    mut budget: Budget,
    weights: ScoreWeights,
) -> Result<SimulatedCampaign, SimulationError> {
    let mut observations = [Observation::default(); 4];
    let mut steps = Vec::new();
    let fixed = budget.verification_reserve();
    let mut remaining_fixed_wall = fixed.wall_ms;
    let stop_reason = loop {
        let objective_met = POINTS.iter().enumerate().all(|(index, &points)| {
            points == 0 || independent_verdict(task, index, &observations) == Verdict::Verified
        });
        let context = ScheduleContext {
            objective_met,
            cancelled: false,
        };
        if mode == StudyMode::A && !steps.is_empty() {
            break if objective_met {
                StopReason::ObjectiveMet
            } else {
                StopReason::NoFeasibleAction
            };
        }
        let mut actions = candidate_actions(task, mode, &observations);
        if !mode.adaptive() && mode != StudyMode::A {
            for action in &mut actions {
                if action.kind.is_verification()
                    && action.estimated_cost.wall_ms > remaining_fixed_wall
                {
                    action.eligible = false;
                }
            }
            // Fixed allocation explores before spending its distinct replay window.
            let can_explore = actions.iter().any(|action| {
                action.eligible
                    && action.kind == ActionKind::Explore
                    && budget.can_reserve(action.kind, action.estimated_cost)
            });
            if can_explore {
                for action in &mut actions {
                    action.eligible &= action.kind == ActionKind::Explore;
                }
            }
        }
        let action_id = match schedule(&budget, &actions, weights, context)
            .map_err(SimulationError::Schedule)?
        {
            Decision::Run { action_id, .. } => action_id,
            Decision::Stop(reason) => break reason,
        };
        let (index, action) = actions
            .iter()
            .enumerate()
            .find(|(_, action)| action.id == action_id)
            .map(|(position, action)| (position / 3, action))
            .ok_or(SimulationError::InvalidSelection)?;
        if budget
            .try_reserve(&action_id, action.kind, action.estimated_cost)
            .map_err(SimulationError::Budget)?
            != ReservationOutcome::Created
        {
            return Err(SimulationError::InvalidSelection);
        }
        let outcome = perform(task, seed, index, action.kind, &mut observations[index]);
        budget
            .settle(&action_id, action.estimated_cost)
            .map_err(SimulationError::Budget)?;
        if action.kind.is_verification() {
            remaining_fixed_wall =
                remaining_fixed_wall.saturating_sub(action.estimated_cost.wall_ms);
        }
        steps.push(SimulatedStep {
            action_id,
            kind: action.kind,
            charged: action.estimated_cost,
            check_passed: outcome,
        });
    };
    let mut verified_utility_points = 0;
    let mut verified_claims = 0;
    let mut rejected_claims = 0;
    let mut inconclusive_claims = 0;
    for (index, points) in POINTS.into_iter().enumerate() {
        match independent_verdict(task, index, &observations) {
            Verdict::Verified => {
                verified_utility_points += points;
                verified_claims += 1;
            }
            Verdict::Rejected => rejected_claims += 1,
            Verdict::Inconclusive => inconclusive_claims += 1,
        }
    }
    Ok(SimulatedCampaign {
        task: task.id,
        seed,
        mode,
        usage: budget.used(),
        verified_utility_points,
        maximum_utility_points: TOTAL_POINTS,
        verified_claims,
        rejected_claims,
        inconclusive_claims,
        stop_reason,
        steps,
    })
}

fn candidate_actions(task: Task, mode: StudyMode, observations: &[Observation; 4]) -> Vec<Action> {
    let mut actions = Vec::new();
    for (index, &points) in POINTS.iter().enumerate() {
        let obs = observations[index];
        let root_has_evidence = independent_verdict(task, 0, observations) == Verdict::Verified;
        let root_blocks_claim = task.depends_on_root(index) && !root_has_evidence;
        let downstream_value = if index == 0 && task.dependent && mode.graph_aware() {
            POINTS[1] + POINTS[2]
        } else {
            0
        };
        for (kind, name, cost, eligible) in [
            (
                ActionKind::Explore,
                "explore",
                9 + u64::from(points / 4),
                !obs.explored && !root_blocks_claim,
            ),
            (
                ActionKind::Replay,
                "replay",
                18,
                obs.explored && task.requires_replay(index) && obs.replay_passed.is_none(),
            ),
            (
                ActionKind::Challenge,
                "challenge",
                12,
                obs.replay_passed == Some(true)
                    && task.requires_challenge(index)
                    && obs.challenge_passed.is_none(),
            ),
        ] {
            let expected_gain = if kind == ActionKind::Explore {
                points
            } else {
                0
            };
            actions.push(Action {
                id: format!("{index:02}-{name}"),
                kind,
                estimated_cost: ResourceVec {
                    wall_ms: cost,
                    attempts: 1,
                    ..ResourceVec::default()
                },
                expected_gain,
                uncertainty_reduction: if kind.is_verification() { 20 } else { 0 },
                claim_importance: points + downstream_value,
                eligible: eligible && (mode != StudyMode::A || kind == ActionKind::Explore),
            });
        }
    }
    actions
}

fn perform(
    task: Task,
    seed: u64,
    claim: usize,
    kind: ActionKind,
    observation: &mut Observation,
) -> Option<bool> {
    // Key outcomes to task, seed, claim and check. Different policy histories
    // therefore observe the same simulated result for the same action.
    let check = match kind {
        ActionKind::Explore => {
            observation.explored = true;
            return None;
        }
        ActionKind::Replay => &mut observation.replay_passed,
        ActionKind::Challenge => &mut observation.challenge_passed,
    };
    let kind_salt = if kind == ActionKind::Replay { 3 } else { 7 };
    let mut key = seed ^ (task.keyed_salt << 12) ^ ((claim as u64) << 6) ^ kind_salt;
    key = (key ^ (key >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    key = (key ^ (key >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    let passed = !(key ^ (key >> 31)).is_multiple_of(5);
    *check = Some(passed);
    Some(passed)
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::*;

    #[test]
    fn matched_synthetic_campaigns_are_deterministic_and_bounded() {
        let config = StudyConfig::default();
        let report = simulate(&config).unwrap();
        let mut reordered = config.clone();
        reordered.seeds.reverse();
        reordered.modes.reverse();
        assert_eq!(report, simulate(&reordered).unwrap());
        assert_eq!(
            report.campaigns.len(),
            TASKS.len() * config.seeds.len() * config.modes.len()
        );
        let mut observations = BTreeMap::new();
        for campaign in &report.campaigns {
            assert!(campaign.usage.wall_ms <= config.caps.wall_ms);
            assert!(campaign.usage.attempts <= config.caps.attempts);
            assert!(campaign.verified_utility_points <= TOTAL_POINTS);
            if campaign.mode == StudyMode::A {
                assert!(campaign.steps.len() <= 1);
            }
            if matches!(campaign.mode, StudyMode::B | StudyMode::E) {
                let verification_wall: u64 = campaign
                    .steps
                    .iter()
                    .filter(|step| step.kind.is_verification())
                    .map(|step| step.charged.wall_ms)
                    .sum();
                assert!(verification_wall <= report.verification_reserve.wall_ms);
            }
            for step in &campaign.steps {
                let key = (campaign.task, campaign.seed, step.action_id.as_str());
                assert_eq!(
                    *observations.entry(key).or_insert(step.check_passed),
                    step.check_passed
                );
            }
        }
        assert!(
            report
                .campaigns
                .iter()
                .any(|campaign| !campaign.steps.is_empty())
        );
    }
}
