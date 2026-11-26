//! Development-only adaptive inference over typed maximum-subarray answers.
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use aether_engine::{ActionKind, Budget, ResourceVec};
use aether_runner::{Interval, Output, Task};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::planning::{OllamaModel, PlanningError};
use crate::{Result, lab::Lab};

pub(crate) const OUTPUT_TOKENS: u64 = 64;
const PROMPT_TOKENS: u64 = 2048;
const MODEL: ResourceVec = units(35_000, PROMPT_TOKENS + OUTPUT_TOKENS, 256);
const VERIFY: ResourceVec = units(100, 0, 0);
const EXECUTE: ResourceVec = units(1_000, 0, aether_runner::MAX_OUTPUT);
const CALLS: u64 = 3;

const fn units(wall_ms: u64, model_tokens: u64, output_bytes: u64) -> ResourceVec {
    ResourceVec {
        wall_ms,
        model_tokens,
        output_bytes,
        attempts: 1,
        paid_microusd: 0,
    }
}

fn elapsed_ms(start: Instant) -> u64 {
    start.elapsed().as_micros().div_ceil(1000) as u64
}

fn ledger(per_attempt: ResourceVec, attempts: u64) -> Result<Budget> {
    Ok(Budget::new(
        ResourceVec {
            wall_ms: per_attempt
                .wall_ms
                .checked_mul(attempts)
                .ok_or("wall budget overflow")?,
            model_tokens: per_attempt
                .model_tokens
                .checked_mul(attempts)
                .ok_or("token budget overflow")?,
            output_bytes: per_attempt
                .output_bytes
                .checked_mul(attempts)
                .ok_or("byte budget overflow")?,
            attempts,
            paid_microusd: 0,
        },
        ResourceVec::default(),
    )?)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum Action {
    Generate,
    Refine,
    Critique,
    Verify,
    Stop,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
enum Strategy {
    Single,
    BestOfN,
    FixedRefinement,
    Adaptive,
}

const STRATEGIES: [Strategy; 4] = [
    Strategy::Single,
    Strategy::BestOfN,
    Strategy::FixedRefinement,
    Strategy::Adaptive,
];

#[derive(Clone, Copy, Debug, Serialize)]
pub(crate) struct Usage {
    pub prompt_tokens: u64,
    pub output_tokens: u64,
    /// A reported subset of output tokens. None means the provider has no split.
    pub reasoning_tokens: Option<u64>,
}

impl Usage {
    fn tokens(self) -> Option<u64> {
        (self.prompt_tokens <= PROMPT_TOKENS
            && self.output_tokens <= OUTPUT_TOKENS
            && self
                .reasoning_tokens
                .is_none_or(|n| n <= self.output_tokens))
        .then(|| self.prompt_tokens + self.output_tokens)
    }
}

pub(crate) struct ModelReply {
    pub text: Option<String>,
    pub usage: Option<Usage>,
    pub model_digest: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Bounds {
    start: usize,
    end_exclusive: usize,
}

impl Bounds {
    fn interval(self, values: &[i16]) -> Option<Interval> {
        let slice = values.get(self.start..self.end_exclusive)?;
        (!slice.is_empty()).then(|| Interval {
            start: self.start,
            end_exclusive: self.end_exclusive,
            sum: slice.iter().map(|n| i32::from(*n)).sum(),
        })
    }
}

struct Request<'a> {
    action: Action,
    values: &'a [i16],
    parent: Option<Bounds>,
    turn: u64,
    seed: u32,
}

impl Request<'_> {
    fn prompt(&self) -> Result<String> {
        let instruction = match self.action {
            Action::Generate => "Propose an interval from scratch.",
            Action::Refine => "Try to improve the previous interval using the numbers.",
            Action::Critique => {
                "Reconsider the previous interval and return the best interval you can justify from the numbers."
            }
            _ => return Err("invalid inference action".into()),
        };
        Ok(format!(
            "Choose a nonempty contiguous subarray with maximum sum. {instruction} Return JSON with only integer start and end_exclusive (zero-based). Data: {}",
            json!({"values": self.values,"previous": self.parent,"sample":self.turn})
        ))
    }
}

fn fixture(request: &Request<'_>) -> ModelReply {
    let singleton = |start| Bounds {
        start,
        end_exclusive: start + 1,
    };
    let largest = request
        .values
        .iter()
        .enumerate()
        .max_by_key(|(i, n)| (*n, std::cmp::Reverse(*i)))
        .expect("reviewed nonempty input")
        .0;
    let bounds = match (request.action, request.turn, request.parent) {
        (Action::Generate, 1, _) => singleton(0),
        (Action::Generate, 2, _) => singleton(largest),
        (Action::Refine, _, Some(b)) => Bounds {
            start: b.start,
            end_exclusive: (b.end_exclusive + 1).min(request.values.len()),
        },
        (Action::Critique, _, _) => singleton(largest),
        _ => Bounds {
            start: 0,
            end_exclusive: request.values.len(),
        },
    };
    ModelReply {
        text: Some(serde_json::to_string(&bounds).expect("bounded JSON")),
        usage: Some(Usage {
            prompt_tokens: 80,
            output_tokens: 24,
            reasoning_tokens: Some(0),
        }),
        model_digest: None,
    }
}

#[derive(Serialize)]
struct Candidate {
    bounds: Bounds,
    interval: Interval,
    parent: Option<usize>,
    checked: bool,
}

#[derive(Serialize)]
struct Limits {
    model: ResourceVec,
    verifier: ResourceVec,
    execution: ResourceVec,
}

#[derive(Serialize)]
pub(crate) struct Report {
    schema: &'static str,
    seed: u32,
    values: Vec<i16>,
    strategy: Strategy,
    provider: &'static str,
    model_digest: Option<String>,
    limits: Limits,
    model_used: ResourceVec,
    verifier_used: ResourceVec,
    execution_used: ResourceVec,
    steps: Vec<Value>,
    candidates: Vec<Candidate>,
    verified_candidate: Option<usize>,
    stop: &'static str,
}

fn run(
    seed: u32,
    strategy: Strategy,
    provider: &'static str,
    ask: impl Fn(&Request<'_>, &AtomicBool) -> Result<ModelReply>,
    cancelled: &AtomicBool,
) -> Result<Report> {
    let task = Task::MaxSubarray { seed, items: 6 };
    let mut model = ledger(MODEL, CALLS)?;
    let mut verifier = ledger(VERIFY, CALLS)?;
    let mut execution = ledger(EXECUTE, 1)?;
    let mut report = Report {
        schema: "aether.inference-development/1",
        seed,
        values: aether_runner::optimization_input(seed, 6)?,
        strategy,
        provider,
        model_digest: None,
        limits: Limits {
            model: model.caps(),
            verifier: verifier.caps(),
            execution: execution.caps(),
        },
        model_used: ResourceVec::default(),
        verifier_used: ResourceVec::default(),
        execution_used: ResourceVec::default(),
        steps: Vec::new(),
        candidates: Vec::new(),
        verified_candidate: None,
        stop: "budget_exhausted",
    };
    let adapt = strategy == Strategy::Adaptive;
    let mut stagnant = 0;
    let mut baseline = None;
    let mut latest = None;
    loop {
        if cancelled.load(Ordering::Acquire) {
            report.stop = "cancelled";
            break;
        }
        let calls = model.used().attempts;
        let pending = report
            .candidates
            .iter()
            .enumerate()
            .filter(|(_, c)| !c.checked)
            .max_by_key(|(i, c)| (c.interval.sum, std::cmp::Reverse(*i)))
            .map(|(i, _)| i);
        let infer = (strategy != Strategy::Single || calls == 0)
            && (!adapt || stagnant < 2)
            && model.can_reserve(ActionKind::Explore, MODEL);
        if let Some(index) = pending
            .filter(|_| (adapt || !infer) && verifier.can_reserve(ActionKind::Replay, VERIFY))
        {
            if baseline.is_none() {
                if !execution.can_reserve(ActionKind::Replay, EXECUTE) {
                    break;
                }
                execution.try_reserve("native-reference", ActionKind::Replay, EXECUTE)?;
                let start = Instant::now();
                let result = aether_runner::supervise(
                    &std::env::current_exe()?,
                    &task,
                    Duration::from_millis(EXECUTE.wall_ms),
                    cancelled,
                );
                let wall_ms = elapsed_ms(start);
                if let Ok(receipt) = result {
                    let actual = units(wall_ms, 0, receipt.output_bytes);
                    let fit = actual.wall_ms <= EXECUTE.wall_ms
                        && actual.output_bytes <= EXECUTE.output_bytes;
                    execution.settle("native-reference", if fit { actual } else { EXECUTE })?;
                    report.steps.push(json!({"action":Action::Verify,"phase":"native_preparation","wall_ms":wall_ms,"artifact_sha256":receipt.artifact_sha256,"within_limits":fit}));
                    if let Output::MaxSubarray {
                        baseline: worker_baseline,
                        ..
                    } = receipt.output
                        && fit
                    {
                        baseline = Some(worker_baseline);
                    }
                } else {
                    execution.settle("native-reference", EXECUTE)?;
                    report.steps.push(json!({"action":Action::Verify,"phase":"native_preparation","wall_ms":wall_ms,"outcome":"execution_failed"}));
                }
            }
            if cancelled.load(Ordering::Acquire) {
                report.stop = "cancelled";
                break;
            }
            let Some(reference_baseline) = &baseline else {
                report.stop = "execution_failed";
                break;
            };
            let id = format!("verify-{index}");
            verifier.try_reserve(&id, ActionKind::Replay, VERIFY)?;
            let start = Instant::now();
            let candidate = &mut report.candidates[index];
            let valid = aether_runner::evaluate(
                &task,
                &Output::MaxSubarray {
                    baseline: reference_baseline.clone(),
                    candidate: candidate.interval.clone(),
                },
            )
            .is_ok();
            let wall_ms = elapsed_ms(start);
            verifier.settle(&id, units(wall_ms.min(VERIFY.wall_ms), 0, 0))?;
            candidate.checked = true;
            let approved = valid && wall_ms <= VERIFY.wall_ms && !cancelled.load(Ordering::Acquire);
            report.steps.push(json!({"action":Action::Verify,"phase":"authoritative_check","candidate":index,"passed":approved,"wall_ms":wall_ms}));
            if approved {
                report.verified_candidate = Some(index);
                report.stop = "verified";
                break;
            }
            if !adapt || wall_ms > VERIFY.wall_ms {
                report.stop = "verification_rejected";
                break;
            }
            continue;
        }
        if !infer {
            report.stop = if adapt && stagnant >= 2 {
                "repetition"
            } else if calls == 0 {
                "budget_exhausted"
            } else {
                "abstained"
            };
            break;
        }
        let action = match (strategy, calls) {
            (Strategy::BestOfN, _) | (_, 0) => Action::Generate,
            (Strategy::Adaptive, 1) => Action::Critique,
            _ => Action::Refine,
        };
        let parent = (action != Action::Generate).then_some(latest).flatten();
        let request = Request {
            action,
            values: &report.values,
            parent: parent.map(|i: usize| report.candidates[i].bounds),
            turn: calls + 1,
            seed,
        };
        let prompt_sha256 = aether_runner::sha256(request.prompt()?.as_bytes());
        let id = format!("infer-{calls}");
        model.try_reserve(&id, ActionKind::Explore, MODEL)?;
        let start = Instant::now();
        let reply = ask(&request, cancelled);
        let wall_ms = elapsed_ms(start);
        let mut step = json!({"action":action,"parent":parent,"wall_ms":wall_ms,"prompt_sha256":prompt_sha256});
        let mut accounted = MODEL;
        let mut overrun = wall_ms > MODEL.wall_ms;
        let mut added = false;
        match reply {
            Ok(reply) => {
                let bytes = reply.text.as_ref().map_or(0, |text| text.len() as u64);
                let tokens = reply.usage.and_then(Usage::tokens);
                overrun |= bytes > MODEL.output_bytes || reply.usage.is_some() && tokens.is_none();
                if let Some(tokens) = tokens.filter(|_| !overrun) {
                    accounted = units(wall_ms, tokens, bytes);
                }
                step["usage"] = json!(reply.usage);
                step["outcome"] = json!("invalid_or_unmetered_reply");
                report.model_digest = reply.model_digest.or(report.model_digest.take());
                if !overrun && tokens.is_some() && !cancelled.load(Ordering::Acquire) {
                    let candidate = reply
                        .text
                        .as_deref()
                        .and_then(|text| serde_json::from_str::<Bounds>(text).ok())
                        .and_then(|bounds| Some((bounds, bounds.interval(&report.values)?)));
                    if let Some((bounds, interval)) = candidate {
                        if let Some(index) =
                            report.candidates.iter().position(|c| c.bounds == bounds)
                        {
                            step["outcome"] = json!("duplicate");
                            step["candidate"] = json!(index);
                        } else {
                            latest = Some(report.candidates.len());
                            report.candidates.push(Candidate {
                                bounds,
                                interval,
                                parent,
                                checked: false,
                            });
                            step["outcome"] = json!("candidate");
                            step["candidate"] = json!(latest);
                            added = true;
                        }
                    }
                }
            }
            Err(error) => {
                step["outcome"] = json!("provider_failed");
                step["error"] = json!(error.to_string());
            }
        }
        model.settle(&id, accounted)?;
        step["charged"] = json!(accounted);
        report.steps.push(step);
        if overrun {
            report.stop = "provider_limit_exceeded";
            break;
        }
        stagnant = if added { 0 } else { stagnant + 1 };
    }
    report.model_used = model.used();
    report.verifier_used = verifier.used();
    report.execution_used = execution.used();
    report
        .steps
        .push(json!({"action":Action::Stop,"reason":report.stop}));
    Ok(report)
}

pub(crate) fn compare(lab: &Lab, live: bool, cancelled: &AtomicBool) -> Result<Value> {
    let local = live.then(|| OllamaModel::new("qwen3.5:4b")).transpose()?;
    let seeds = if live { &[19][..] } else { &[19, 281, 617][..] };
    let mut trials = Vec::new();
    for &seed in seeds {
        for strategy in STRATEGIES {
            let provider = if live {
                "ollama-local"
            } else {
                "scripted-fixture"
            };
            let report = run(
                seed,
                strategy,
                provider,
                |request, flag| {
                    if let Some(adapter) = &local {
                        if flag.load(Ordering::Acquire) {
                            return Err(PlanningError::Cancelled.into());
                        }
                        Ok(adapter.interval(
                            &request.prompt()?,
                            request.seed ^ (request.turn as u32),
                            flag,
                        )?)
                    } else {
                        Ok(fixture(request))
                    }
                },
                cancelled,
            )?;
            let terminal = report.stop;
            let digest = lab.record_inference(&report)?;
            trials.push(json!({"artifact_sha256":digest,"report":report}));
            if terminal == "cancelled"
                || live
                    && (terminal == "provider_limit_exceeded"
                        || trials
                            .last()
                            .is_some_and(|t| t["report"]["model_digest"].is_null()))
            {
                return Ok(json!({"scope":"development", "complete":false, "trials":trials}));
            }
        }
    }
    Ok(json!({"scope":"development", "complete":true,"trials":trials}))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn failures_are_charged_and_repetition_stops_before_the_call_limit() {
        let report = run(
            19,
            Strategy::Adaptive,
            "test",
            |request, _| {
                if request.turn == 1 {
                    return Err(PlanningError::Timeout.into());
                }
                Ok(ModelReply {
                    text: Some(r#"{"start":0,"end_exclusive":8,"tools":[]}"#.into()),
                    usage: Some(Usage {
                        prompt_tokens: 40,
                        output_tokens: 10,
                        reasoning_tokens: None,
                    }),
                    model_digest: None,
                })
            },
            &AtomicBool::new(false),
        )
        .unwrap();
        assert_eq!(report.stop, "repetition");
        assert_eq!(report.model_used.attempts, 2);
        assert_eq!(report.model_used.model_tokens, MODEL.model_tokens + 50);
        assert!(report.candidates.is_empty());
        assert_eq!(report.verifier_used.attempts, 0);
    }

    #[test]
    fn cancellation_and_provider_overrun_abstain_without_execution() {
        let flag = AtomicBool::new(false);
        let cancelled = run(
            19,
            Strategy::Single,
            "test",
            |request, flag| {
                flag.store(true, Ordering::Release);
                Ok(fixture(request))
            },
            &flag,
        )
        .unwrap();
        assert_eq!(cancelled.stop, "cancelled");
        assert_eq!(cancelled.model_used.model_tokens, 104);
        let overrun = run(
            19,
            Strategy::Single,
            "test",
            |request, _| {
                let mut reply = fixture(request);
                reply.usage.as_mut().unwrap().output_tokens = OUTPUT_TOKENS + 1;
                Ok(reply)
            },
            &AtomicBool::new(false),
        )
        .unwrap();
        assert_eq!(overrun.stop, "provider_limit_exceeded");
        assert_eq!(overrun.model_used.model_tokens, MODEL.model_tokens);
        for report in [cancelled, overrun] {
            assert_eq!(report.execution_used.attempts, 0);
            assert!(report.verified_candidate.is_none() && report.candidates.is_empty());
        }
    }
}
