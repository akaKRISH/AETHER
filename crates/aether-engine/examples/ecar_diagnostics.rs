//! Synthetic ECAR diagnostic with separate truth and noisy observation streams.

use aether_engine::{
    Action, ActionKind, Budget, Decision, ResourceVec, ScheduleContext, ScoreWeights,
    schedule_with_trace,
};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::time::Instant;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Config {
    phase: String,
    first_seed: u32,
    seed_count: u32,
    caps: Vec<u64>,
    scenarios: Vec<Scenario>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Scenario {
    id: String,
    parents: Vec<Option<usize>>,
    weights: Vec<u16>,
    truth: Vec<u8>,
    replay_cost: u64,
    challenge_cost: u64,
    noise: u8,
    failure: u8,
}

impl Scenario {
    fn valid(&self) -> bool {
        let n = self.weights.len();
        (1..=16).contains(&n)
            && self.parents.len() == n
            && self.truth.len() == n
            && self.weights.iter().all(|value| (1..=100).contains(value))
            && self.truth.iter().all(|value| *value <= 100)
            && self
                .parents
                .iter()
                .enumerate()
                .all(|(row, parent)| parent.is_none_or(|p| p < row))
            && (1..=100).contains(&self.replay_cost)
            && (1..=100).contains(&self.challenge_cost)
            && self.noise <= 100
            && self.failure <= 100
    }
}

#[derive(Clone, Copy, Serialize)]
enum Mode {
    FixedFlat,
    AdaptiveFlat,
    FixedGraph,
    Ecar,
}

impl Mode {
    fn fixed(self) -> bool {
        matches!(self, Self::FixedFlat | Self::FixedGraph)
    }

    fn graph(self) -> bool {
        matches!(self, Self::FixedGraph | Self::Ecar)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Status {
    New,
    Explored,
    Replayed,
    Verified,
    Rejected,
    Failed,
}

#[derive(Serialize)]
struct Outcome<'a> {
    family: &'a str,
    seed: u32,
    cap: u64,
    mode: Mode,
    total_weight: u16,
    verified_utility: u16,
    verified: usize,
    false_verified: usize,
    rejected: usize,
    unresolved_high_impact: usize,
    replays: usize,
    replay_passes: usize,
    failed_attempts: usize,
    total_claims: usize,
    used: ResourceVec,
    trace: Vec<String>,
}

fn draw(seed: u32, family: usize, claim: usize, stream: u64) -> u8 {
    let mut word = u64::from(seed) ^ ((family as u64) << 40) ^ ((claim as u64) << 24) ^ stream;
    word = (word ^ (word >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    word = (word ^ (word >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    ((word ^ (word >> 31)) % 100) as u8
}

fn run<'a>(
    scenario: &'a Scenario,
    family: usize,
    seed: u32,
    cap: u64,
    mode: Mode,
    scheduler_ns: &mut u128,
) -> Result<Outcome<'a>, Box<dyn Error>> {
    let mut states = vec![Status::New; scenario.weights.len()];
    let mut actual = Vec::with_capacity(states.len());
    for (index, parent) in scenario.parents.iter().enumerate() {
        actual.push(
            parent.is_none_or(|p| actual[p])
                && draw(seed, family, index, 1) < scenario.truth[index],
        );
    }
    let mut mass = scenario.weights.clone();
    if mode.graph() {
        for (index, parent) in scenario.parents.iter().enumerate().rev() {
            if let Some(parent) = parent {
                mass[*parent] = mass[*parent]
                    .checked_add(mass[index])
                    .ok_or("claim weight overflow")?;
            }
        }
    }
    let mut budget = Budget::with_quarter_reserve(ResourceVec {
        wall_ms: cap,
        attempts: 48,
        ..ResourceVec::default()
    });
    let mut verification_spent = 0;
    let mut trace = Vec::new();
    let (mut replays, mut replay_passes, mut failed_attempts) = (0, 0, 0);
    loop {
        let mut actions: Vec<Action> = states
            .iter()
            .enumerate()
            .filter_map(|(index, state)| {
                let (kind, cost, gain) = match state {
                    Status::New => (ActionKind::Explore, 3, 4),
                    Status::Explored => (ActionKind::Replay, scenario.replay_cost, 10),
                    Status::Replayed => (ActionKind::Challenge, scenario.challenge_cost, 14),
                    _ => return None,
                };
                let parent_ready =
                    scenario.parents[index].is_none_or(|p| states[p] == Status::Verified);
                Some(Action {
                    id: format!("{index:02}-{kind:?}"),
                    kind,
                    estimated_cost: ResourceVec {
                        wall_ms: cost,
                        attempts: 1,
                        ..ResourceVec::default()
                    },
                    expected_gain: gain,
                    uncertainty_reduction: gain,
                    claim_importance: mass[index],
                    eligible: (!kind.is_verification() || parent_ready)
                        && (!mode.fixed()
                            || !kind.is_verification()
                            || verification_spent + cost <= cap / 2),
                })
            })
            .collect();
        if mode.fixed()
            && actions.iter().any(|action| {
                action.kind == ActionKind::Explore
                    && action.eligible
                    && budget.can_reserve(action.kind, action.estimated_cost)
            })
        {
            for action in &mut actions {
                action.eligible &= action.kind == ActionKind::Explore;
            }
        }
        let start = Instant::now();
        let result = schedule_with_trace(
            &budget,
            &actions,
            ScoreWeights::default(),
            ScheduleContext::default(),
        )?;
        *scheduler_ns += start.elapsed().as_nanos();
        let Decision::Run { action_id, kind } = result.decision else {
            break;
        };
        let action = actions
            .iter()
            .find(|action| action.id == action_id)
            .ok_or("unknown action")?;
        let index: usize = action_id
            .split_once('-')
            .ok_or("invalid action ID")?
            .0
            .parse()?;
        let cost = action.estimated_cost;
        budget.try_reserve(&action_id, kind, cost)?;
        budget.settle(&action_id, cost)?;
        if kind.is_verification() {
            verification_spent += cost.wall_ms;
        }
        let marker = match kind {
            ActionKind::Explore => 11,
            ActionKind::Replay => 21,
            ActionKind::Challenge => 31,
        };
        if kind == ActionKind::Replay {
            replays += 1;
        }
        states[index] = if draw(seed, family, index, marker) < scenario.failure {
            failed_attempts += 1;
            Status::Failed
        } else {
            let observed = actual[index] ^ (draw(seed, family, index, marker + 1) < scenario.noise);
            match kind {
                ActionKind::Explore => Status::Explored,
                ActionKind::Replay if observed => {
                    replay_passes += 1;
                    Status::Replayed
                }
                ActionKind::Challenge if observed => Status::Verified,
                _ => Status::Rejected,
            }
        };
        trace.push(action_id);
    }
    let verified = states
        .iter()
        .filter(|state| **state == Status::Verified)
        .count();
    let false_verified = states
        .iter()
        .zip(&actual)
        .filter(|(state, truth)| **state == Status::Verified && !**truth)
        .count();
    let verified_utility = states
        .iter()
        .zip(&actual)
        .zip(&scenario.weights)
        .filter_map(|((&state, truth), &weight)| {
            (state == Status::Verified && *truth).then_some(weight)
        })
        .sum();
    Ok(Outcome {
        family: &scenario.id,
        seed,
        cap,
        mode,
        total_weight: scenario.weights.iter().sum(),
        verified_utility,
        verified,
        false_verified,
        rejected: states
            .iter()
            .filter(|state| **state == Status::Rejected)
            .count(),
        unresolved_high_impact: states
            .iter()
            .zip(&scenario.weights)
            .filter(|(state, weight)| {
                **weight >= 9 && !matches!(state, Status::Verified | Status::Rejected)
            })
            .count(),
        replays,
        replay_passes,
        failed_attempts,
        total_claims: states.len(),
        used: budget.used(),
        trace,
    })
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = std::env::args_os().skip(1);
    let path = args.next().ok_or("supply one frozen study configuration")?;
    if args.next().is_some() {
        return Err("supply one frozen study configuration".into());
    }
    let config: Config = serde_json::from_slice(&std::fs::read(path)?)?;
    if config.phase != "development"
        || !(1..=256).contains(&config.seed_count)
        || !(1..=6).contains(&config.caps.len())
        || config.caps.iter().any(|cap| !(8..=240).contains(cap))
        || !(1..=8).contains(&config.scenarios.len())
        || !config.scenarios.iter().all(Scenario::valid)
    {
        return Err("invalid diagnostic development specification".into());
    }
    let end_seed = config
        .first_seed
        .checked_add(config.seed_count)
        .ok_or("seed overflow")?;
    let started = Instant::now();
    let mut scheduler_ns = 0;
    let mut outcomes = Vec::new();
    for (family, scenario) in config.scenarios.iter().enumerate() {
        for seed in config.first_seed..end_seed {
            for &cap in &config.caps {
                for mode in [
                    Mode::FixedFlat,
                    Mode::AdaptiveFlat,
                    Mode::FixedGraph,
                    Mode::Ecar,
                ] {
                    outcomes.push(run(scenario, family, seed, cap, mode, &mut scheduler_ns)?);
                }
            }
        }
    }
    let report = serde_json::json!({
        "version": "aether.ecar-diagnostics/1",
        "scope": "synthetic development",
        "wall_us": started.elapsed().as_micros(),
        "scheduler_ns": scheduler_ns,
        "outcomes": outcomes,
    });
    serde_json::to_writer(std::io::stdout().lock(), &report)?;
    Ok(())
}
