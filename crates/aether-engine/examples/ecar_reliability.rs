//! Development-only comparison. The frozen diagnostic remains the baseline reference.
use std::error::Error;
use std::time::Instant;

use aether_engine::reliability::{ClaimEvidence, DownstreamWeight, priorities};
use aether_engine::{
    Action, ActionKind, Budget, Decision, ResourceVec, ScheduleContext, ScoreWeights,
    schedule_with_trace,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

const INPUT: &[u8] = include_bytes!("../../../benchmarks/dae-ecar-reliability-development.json");

#[derive(Deserialize)]
struct Config {
    phase: String,
    version: String,
    seed_key: String,
    first_seed: u32,
    seed_count: u32,
    caps: Vec<u64>,
    scenarios: Vec<Scenario>,
}

#[derive(Deserialize)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
enum Mode {
    FixedFlat,
    AdaptiveFlat,
    Ecar,
    EvidenceUnweighted,
    EvidencePriorOnly,
    EvidenceWeighted,
}

const ARMS: [Mode; 6] = [
    Mode::FixedFlat,
    Mode::AdaptiveFlat,
    Mode::Ecar,
    Mode::EvidenceUnweighted,
    Mode::EvidencePriorOnly,
    Mode::EvidenceWeighted,
];

impl Mode {
    fn weighting(self) -> Option<DownstreamWeight> {
        match self {
            Self::EvidenceUnweighted => Some(DownstreamWeight::Full),
            Self::EvidencePriorOnly => Some(DownstreamWeight::PriorMean),
            Self::EvidenceWeighted => Some(DownstreamWeight::PosteriorMean),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum Status {
    #[default]
    New,
    Explored,
    Replayed,
    Verified,
    Rejected,
    Failed,
}

impl Status {
    fn proposal(self, replay_cost: u64, challenge_cost: u64) -> Option<(ActionKind, u64, u16)> {
        match self {
            Self::New => Some((ActionKind::Explore, 3, 4)),
            Self::Explored => Some((ActionKind::Replay, replay_cost, 10)),
            Self::Replayed => Some((ActionKind::Challenge, challenge_cost, 14)),
            _ => None,
        }
    }
}

#[derive(Default, Serialize)]
struct Timing {
    scheduler_calls: u64,
    scheduler_ns: u128,
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
    verification_wall_ms: u64,
    evidence_receipts: usize,
    blocked_descendants: usize,
}

// Same encoding as the original v2 development runner for matched baseline replay.
fn draw(seed: u32, family: u8, claim: u8, stream: u8) -> u8 {
    let hash = Sha256::new()
        .chain_update(b"aether.ecar.draw/2\0")
        .chain_update([0])
        .chain_update([0x51; 32])
        .chain_update(seed.to_be_bytes())
        .chain_update([family, claim, stream])
        .finalize();
    (u64::from_be_bytes(hash[..8].try_into().expect("SHA-256 prefix")) % 100) as u8
}

fn run<'a>(
    scenario: &'a Scenario,
    family: u8,
    seed: u32,
    cap: u64,
    mode: Mode,
    timing: &mut Timing,
) -> Result<Outcome<'a>, Box<dyn Error>> {
    let mut evidence: Vec<_> = scenario
        .weights
        .iter()
        .zip(&scenario.parents)
        .map(|(&weight, &parent)| ClaimEvidence {
            weight,
            parent,
            ..ClaimEvidence::default()
        })
        .collect();
    let mut states = vec![Status::New; evidence.len()];
    let mut truth = Vec::with_capacity(states.len());
    for (index, claim) in evidence.iter().enumerate() {
        truth.push(
            claim.parent.is_none_or(|p| truth[p])
                && draw(seed, family, index as u8, 1) < scenario.truth[index],
        );
    }
    let mut budget = Budget::with_quarter_reserve(ResourceVec {
        wall_ms: cap,
        attempts: 48,
        ..ResourceVec::default()
    });
    let fixed = mode == Mode::FixedFlat;
    let mut outcome = Outcome {
        family: &scenario.id,
        seed,
        cap,
        mode,
        total_weight: scenario.weights.iter().sum(),
        verified_utility: 0,
        verified: 0,
        false_verified: 0,
        rejected: 0,
        unresolved_high_impact: 0,
        replays: 0,
        replay_passes: 0,
        failed_attempts: 0,
        total_claims: states.len(),
        used: ResourceVec::default(),
        trace: Vec::new(),
        verification_wall_ms: 0,
        evidence_receipts: 0,
        blocked_descendants: 0,
    };
    let prepare = Instant::now();
    let mut mass = scenario.weights.clone();
    if mode == Mode::Ecar {
        for (index, claim) in evidence.iter().enumerate().rev() {
            if let Some(parent) = claim.parent {
                mass[parent] = mass[parent]
                    .checked_add(mass[index])
                    .ok_or("claim mass overflow")?;
            }
        }
    }
    timing.scheduler_ns += prepare.elapsed().as_nanos();
    loop {
        let started = Instant::now();
        let ranks = mode
            .weighting()
            .map(|weighting| priorities(&evidence, weighting))
            .transpose()?;
        let mut actions = Vec::new();
        for (index, state) in states.iter().enumerate() {
            let Some((kind, wall_ms, gain)) =
                state.proposal(scenario.replay_cost, scenario.challenge_cost)
            else {
                continue;
            };
            let parent_ready = evidence[index]
                .parent
                .is_none_or(|p| states[p] == Status::Verified);
            let is_check = kind.is_verification();
            let mut action = Action {
                id: format!("{index:02}-{kind:?}"),
                kind,
                expected_gain: gain,
                uncertainty_reduction: gain,
                claim_importance: mass[index],
                estimated_cost: ResourceVec {
                    wall_ms,
                    attempts: 1,
                    ..ResourceVec::default()
                },
                eligible: !is_check
                    || parent_ready
                        && (!fixed || outcome.verification_wall_ms + wall_ms <= cap / 2),
            };
            if let Some(ranks) = &ranks {
                ranks[index].apply(&mut action)?;
            }
            actions.push(action);
        }
        if fixed
            && actions.iter().any(|a| {
                a.kind == ActionKind::Explore
                    && a.eligible
                    && budget.can_reserve(a.kind, a.estimated_cost)
            })
        {
            for action in &mut actions {
                action.eligible &= action.kind == ActionKind::Explore;
            }
        }
        let decision = schedule_with_trace(
            &budget,
            &actions,
            ScoreWeights::default(),
            ScheduleContext::default(),
        )?
        .decision;
        timing.scheduler_calls += 1;
        timing.scheduler_ns += started.elapsed().as_nanos();
        let Decision::Run { action_id, kind } = decision else {
            if let Some(ranks) = ranks {
                outcome.blocked_descendants = ranks
                    .iter()
                    .zip(&evidence)
                    .filter(|(rank, claim)| rank.blocked && !claim.rejected)
                    .count();
            }
            break;
        };
        let cost = actions
            .iter()
            .find(|a| a.id == action_id)
            .ok_or("unknown action")?
            .estimated_cost;
        let index: usize = action_id
            .split_once('-')
            .ok_or("invalid action ID")?
            .0
            .parse()?;
        budget.try_reserve(&action_id, kind, cost)?;
        budget.settle(&action_id, cost)?;
        outcome.trace.push(action_id);
        if kind.is_verification() {
            outcome.verification_wall_ms += cost.wall_ms;
        }
        if kind == ActionKind::Replay {
            outcome.replays += 1;
        }
        let stream = match kind {
            ActionKind::Explore => 11,
            ActionKind::Replay => 21,
            ActionKind::Challenge => 31,
        };
        if draw(seed, family, index as u8, stream) < scenario.failure {
            states[index] = Status::Failed;
            outcome.failed_attempts += 1;
            continue;
        }
        let receipt = truth[index] ^ (draw(seed, family, index as u8, stream + 1) < scenario.noise);
        outcome.evidence_receipts += 1;
        if receipt {
            evidence[index].support += 1;
        } else {
            evidence[index].contradiction += 1;
        }
        // Independent simulated evaluator. The experimental priority cannot amend these rules.
        states[index] = match kind {
            ActionKind::Explore => Status::Explored,
            ActionKind::Replay if receipt => {
                outcome.replay_passes += 1;
                Status::Replayed
            }
            ActionKind::Challenge if receipt => Status::Verified,
            _ => Status::Rejected,
        };
        evidence[index].rejected = states[index] == Status::Rejected;
    }
    for ((&state, &actual), &weight) in states.iter().zip(&truth).zip(&scenario.weights) {
        if state == Status::Verified {
            outcome.verified += 1;
            if actual {
                outcome.verified_utility += weight;
            } else {
                outcome.false_verified += 1;
            }
        }
        outcome.rejected += usize::from(state == Status::Rejected);
        outcome.unresolved_high_impact +=
            usize::from(weight >= 9 && !matches!(state, Status::Verified | Status::Rejected));
    }
    outcome.used = budget.used();
    Ok(outcome)
}

fn config() -> Result<Config, Box<dyn Error>> {
    let config: Config = serde_json::from_slice(INPUT)?;
    if config.phase != "development"
        || config.version != "aether.ecar-config/2"
        || config.seed_key != "51".repeat(32)
    {
        return Err("invalid embedded development scope or seed key".into());
    }
    Ok(config)
}

fn main() -> Result<(), Box<dyn Error>> {
    if std::env::args_os().len() != 1 {
        return Err("uses only the embedded development grid".into());
    }
    let config = config()?;
    let mut timings: [Timing; ARMS.len()] = std::array::from_fn(|_| Timing::default());
    let mut outcomes = Vec::new();
    let start = Instant::now();
    for (family, scenario) in config.scenarios.iter().enumerate() {
        for seed in config.first_seed..config.first_seed + config.seed_count {
            for &cap in &config.caps {
                for (index, mode) in ARMS.into_iter().enumerate() {
                    outcomes.push(run(
                        scenario,
                        family as u8,
                        seed,
                        cap,
                        mode,
                        &mut timings[index],
                    )?);
                }
            }
        }
    }
    let report = serde_json::json!({
        "scope": "aether.ecar-reliability-development/1", "config_sha256": format!("{:x}", Sha256::digest(INPUT)),
        "wall_ns": start.elapsed().as_nanos(), "timing_arms": ARMS, "timing": timings, "outcomes": outcomes,
    });
    serde_json::to_writer(std::io::stdout().lock(), &report)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn failure_produces_no_evidence_and_replay_uses_only_observed_receipts() {
        let mut scenario = config().unwrap().scenarios.remove(1);
        scenario.failure = 100;
        let failed = run(
            &scenario,
            1,
            301,
            24,
            Mode::EvidenceWeighted,
            &mut Timing::default(),
        )
        .unwrap();
        assert_eq!(failed.evidence_receipts, 0);
        assert_eq!(failed.failed_attempts, failed.trace.len());
        assert_eq!(
            (failed.verified, failed.rejected, failed.blocked_descendants),
            (0, 0, 0)
        );
        let trace = failed.trace;
        scenario.truth.fill(0);
        let changed = run(
            &scenario,
            1,
            301,
            24,
            Mode::EvidenceWeighted,
            &mut Timing::default(),
        )
        .unwrap();
        assert_eq!(trace, changed.trace);
    }

    #[test]
    fn matched_grid_obeys_budget_and_is_deterministic() {
        for (family, scenario) in config().unwrap().scenarios.iter().enumerate() {
            for cap in [12, 84] {
                for mode in ARMS {
                    let run = || {
                        run(
                            scenario,
                            family as u8,
                            301,
                            cap,
                            mode,
                            &mut Timing::default(),
                        )
                        .unwrap()
                    };
                    let outcome = run();
                    assert!(outcome.used.wall_ms <= cap && outcome.used.attempts <= 48);
                    assert_eq!(
                        outcome.evidence_receipts + outcome.failed_attempts,
                        outcome.trace.len()
                    );
                    assert_eq!(
                        serde_json::to_vec(&outcome).unwrap(),
                        serde_json::to_vec(&run()).unwrap()
                    );
                }
            }
        }
    }
}
