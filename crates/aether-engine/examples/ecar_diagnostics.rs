//! Synthetic ECAR diagnostic with separate truth and noisy observation streams.

use aether_engine::{
    Action, ActionKind, Budget, Decision, ResourceVec, ScheduleContext, ScoreWeights,
    schedule_with_trace,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use std::error::Error;
use std::io::Read;
use std::path::Path;
use std::time::Instant;

const PROTOCOL: &[u8] = include_bytes!("../../../docs/research/ATH_02B_ECAR_PROTOCOL.md");
const ATTEMPT_CAP: u64 = 48;
const ARCHIVED_PILOT_SHA256: &str =
    "76c9b935fb52b3b84eddc4afdb350fbe103e479ccf37747b9f69196e2434675b";

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
enum Phase {
    Development,
    Heldout,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Config {
    phase: Phase,
    version: Option<String>,
    seed_key: Option<String>,
    first_seed: u32,
    seed_count: u32,
    caps: Vec<u64>,
    scenarios: Vec<Scenario>,
}

impl Config {
    fn validate(&self, phase: Phase, versioned: bool) -> Result<Draws, Box<dyn Error>> {
        let minimum_seeds = match (phase, versioned) {
            (Phase::Heldout, _) => 30,
            (_, true) => 2,
            (_, false) => 1,
        };
        if self.phase != phase
            || !(minimum_seeds..=256).contains(&self.seed_count)
            || self.first_seed.checked_add(self.seed_count).is_none()
            || !(1..=6).contains(&self.caps.len())
            || self.caps.iter().any(|cap| !(8..=240).contains(cap))
            || self.caps.iter().collect::<BTreeSet<_>>().len() != self.caps.len()
            || !(1..=8).contains(&self.scenarios.len())
            || !self.scenarios.iter().all(Scenario::valid)
            || self
                .scenarios
                .iter()
                .map(|s| &s.id)
                .collect::<BTreeSet<_>>()
                .len()
                != self.scenarios.len()
        {
            return Err("invalid diagnostic specification or phase".into());
        }
        match (versioned, self.version.as_deref(), self.seed_key.as_deref()) {
            (false, None, None) => Ok(Draws::Legacy),
            (true, Some("aether.ecar-config/2"), Some(key)) => {
                let mut bytes = [0; 32];
                if key.len() != 64 || !key.bytes().all(|byte| byte.is_ascii_hexdigit()) {
                    return Err("seed_key must contain 64 hexadecimal characters".into());
                }
                for (index, byte) in bytes.iter_mut().enumerate() {
                    *byte = u8::from_str_radix(&key[index * 2..index * 2 + 2], 16)?;
                }
                Ok(Draws::Separated(phase, bytes))
            }
            _ => Err("invalid diagnostic configuration version or seed key".into()),
        }
    }
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
        !self.id.is_empty()
            && (1..=16).contains(&n)
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

#[derive(Clone, Copy)]
enum Draws {
    Legacy,
    Separated(Phase, [u8; 32]),
}

fn draw_key(
    phase: Phase,
    secret: &[u8; 32],
    seed: u32,
    family: u8,
    claim: u8,
    stream: u8,
) -> [u8; 40] {
    let mut key = [0; 40];
    key[0] = u8::from(phase == Phase::Heldout);
    key[1..33].copy_from_slice(secret);
    key[33..37].copy_from_slice(&seed.to_be_bytes());
    key[37..].copy_from_slice(&[family, claim, stream]);
    key
}

fn draw(draws: Draws, seed: u32, family: usize, claim: usize, stream: u8) -> u8 {
    if let Draws::Separated(phase, secret) = draws {
        // Configuration limits family and claim indices to at most 7 and 15.
        let key = draw_key(phase, &secret, seed, family as u8, claim as u8, stream);
        let hash = Sha256::new()
            .chain_update(b"aether.ecar.draw/2\0")
            .chain_update(key)
            .finalize();
        return (u64::from_be_bytes(hash[..8].try_into().expect("SHA-256 prefix")) % 100) as u8;
    }
    let mut word =
        u64::from(seed) ^ ((family as u64) << 40) ^ ((claim as u64) << 24) ^ u64::from(stream);
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
    draws: Draws,
    scheduler_ns: &mut u128,
) -> Result<Outcome<'a>, Box<dyn Error>> {
    let mut states = vec![Status::New; scenario.weights.len()];
    let mut actual = Vec::with_capacity(states.len());
    for (index, parent) in scenario.parents.iter().enumerate() {
        actual.push(
            parent.is_none_or(|p| actual[p])
                && draw(draws, seed, family, index, 1) < scenario.truth[index],
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
        attempts: ATTEMPT_CAP,
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
        states[index] = if draw(draws, seed, family, index, marker) < scenario.failure {
            failed_attempts += 1;
            Status::Failed
        } else {
            let observed =
                actual[index] ^ (draw(draws, seed, family, index, marker + 1) < scenario.noise);
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

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Approval {
    version: String,
    protocol_sha256: String,
    runner_sha256: String,
    config_sha256: String,
    analysis_sha256: String,
}

fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn read_input(path: &Path) -> Result<Vec<u8>, Box<dyn Error>> {
    const LIMIT: u64 = 64 * 1024;
    let mut bytes = Vec::new();
    std::fs::File::open(path)?
        .take(LIMIT + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() as u64 > LIMIT {
        return Err("study input exceeds 64 KiB".into());
    }
    Ok(bytes)
}

fn authorize(bytes: &[u8], digest: &str, runner: &str, config: &str) -> Result<(), Box<dyn Error>> {
    if sha256(bytes) != digest {
        return Err("approval manifest digest mismatch".into());
    }
    let approval: Approval = serde_json::from_slice(bytes)?;
    if approval.version != "aether.ecar-approval/1"
        || approval.analysis_sha256.len() != 64
        || !approval
            .analysis_sha256
            .bytes()
            .all(|byte| matches!(byte, b'0'..=b'9' | b'a'..=b'f'))
    {
        return Err("invalid approval manifest".into());
    }
    if approval.protocol_sha256 != sha256(PROTOCOL) {
        return Err("protocol digest mismatch".into());
    }
    if approval.runner_sha256 != runner {
        return Err("runner digest mismatch".into());
    }
    if approval.config_sha256 != config {
        return Err("configuration digest mismatch".into());
    }
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    let (path, versioned, phase, approval) = match args.as_slice() {
        [path] => (path, false, Phase::Development, None),
        [flag, path] if flag == "--development-v2" => (path, true, Phase::Development, None),
        [flag, path, manifest, digest] if flag == "--heldout" => (
            path, true, Phase::Heldout,
            Some((manifest, digest.to_str().ok_or("invalid approval digest")?)),
        ),
        _ => return Err("usage: <config> | --development-v2 <config> | --heldout <config> <manifest> <approved-digest>".into()),
    };
    let input = read_input(Path::new(path))?;
    let config_sha256 = sha256(&input);
    let runner_sha256 = if versioned {
        Some(sha256(&std::fs::read(std::env::current_exe()?)?))
    } else {
        None
    };
    if let Some((manifest, digest)) = approval {
        authorize(
            &read_input(Path::new(manifest))?,
            digest,
            runner_sha256
                .as_deref()
                .ok_or("required runner pin missing")?,
            &config_sha256,
        )?;
    }
    let config: Config = serde_json::from_slice(&input)?;
    let draws = config.validate(phase, versioned)?;
    if !versioned && config_sha256 != ARCHIVED_PILOT_SHA256 {
        return Err("version 1 accepts only the exact archived pilot configuration".into());
    }
    let end_seed = config.first_seed + config.seed_count;
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
                    outcomes.push(run(
                        scenario,
                        family,
                        seed,
                        cap,
                        mode,
                        draws,
                        &mut scheduler_ns,
                    )?);
                }
            }
        }
    }
    let mut report = serde_json::json!({
        "version": "aether.ecar-diagnostics/1",
        "scope": "synthetic development",
        "wall_us": started.elapsed().as_micros(),
        "scheduler_ns": scheduler_ns,
        "outcomes": outcomes,
    });
    if versioned {
        report["version"] = "aether.ecar-diagnostics/2".into();
        report["scope"] = match phase {
            Phase::Development => "synthetic development",
            Phase::Heldout => "synthetic heldout",
        }
        .into();
        report["phase"] = serde_json::to_value(phase)?;
        report["protocol_sha256"] = sha256(PROTOCOL).into();
        report["runner_sha256"] = serde_json::to_value(runner_sha256)?;
        report["config_sha256"] = config_sha256.into();
        report["approval_sha256"] = serde_json::to_value(approval.map(|(_, digest)| digest))?;
    }
    serde_json::to_writer(std::io::stdout().lock(), &report)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn draw_keys_are_unique_across_the_reviewed_coordinates() {
        let mut keys = BTreeSet::new();
        for phase in [Phase::Development, Phase::Heldout] {
            for seed in 8193..8223 {
                for family in 0..8 {
                    for claim in 0..16 {
                        for stream in [1, 11, 12, 21, 22, 31, 32] {
                            assert!(
                                keys.insert(draw_key(phase, &[0; 32], seed, family, claim, stream))
                            );
                        }
                    }
                }
            }
        }
        assert_eq!(keys.len(), 2 * 30 * 8 * 16 * 7);
    }

    #[test]
    fn seed_stream_and_split_separation_break_the_old_alias() {
        assert_eq!(8193 ^ 1, 8203 ^ 11);
        let key = [42; 32];
        let truth = draw_key(Phase::Heldout, &key, 8193, 0, 0, 1);
        assert_ne!(truth, draw_key(Phase::Heldout, &key, 8203, 0, 0, 11));
        assert_ne!(truth, draw_key(Phase::Heldout, &key, 8194, 0, 0, 1));
        assert_ne!(truth, draw_key(Phase::Development, &key, 8193, 0, 0, 1));
        assert_ne!(truth, draw_key(Phase::Heldout, &[43; 32], 8193, 0, 0, 1));
    }

    #[test]
    fn separated_draws_reproduce_a_fixed_vector() {
        let draws = Draws::Separated(Phase::Development, [42; 32]);
        let values = [1, 11, 21, 22, 31, 32].map(|stream| draw(draws, 201, 2, 3, stream));
        assert_eq!(values, [76, 81, 53, 95, 15, 68]);
    }
}
