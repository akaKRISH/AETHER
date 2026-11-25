use std::cmp::Ordering;
use std::fmt;

use serde::{Deserialize, Serialize};

use crate::{Budget, ResourceVec};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ActionKind {
    Explore,
    Replay,
    Challenge,
}

impl ActionKind {
    pub fn is_verification(self) -> bool {
        matches!(self, Self::Replay | Self::Challenge)
    }
}

/// The controller supplies candidates from the reviewed task registry. The
/// eligible flag is its trusted result of checking approval and prerequisites.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Action {
    pub id: String,
    pub kind: ActionKind,
    pub estimated_cost: ResourceVec,
    pub expected_gain: u16,
    pub uncertainty_reduction: u16,
    pub claim_importance: u16,
    pub eligible: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ScoreWeights {
    pub lambda: u16,
    pub mu: u16,
}

impl Default for ScoreWeights {
    fn default() -> Self {
        Self { lambda: 1, mu: 1 }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct ScheduleContext {
    pub cancelled: bool,
    pub objective_met: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum StopReason {
    Cancelled,
    ObjectiveMet,
    AttemptLimit,
    BudgetExhausted,
    NoFeasibleAction,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "decision", content = "detail", rename_all = "snake_case")]
pub enum Decision {
    Run { action_id: String, kind: ActionKind },
    Stop(StopReason),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ScheduleTrace {
    pub decision: Decision,
    pub candidates: Vec<CandidateTrace>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CandidateTrace {
    pub action_id: String,
    pub kind: ActionKind,
    pub evaluation: CandidateEvaluation,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum CandidateEvaluation {
    Ineligible,
    Unaffordable,
    Ranked {
        numerator: String,
        denominator: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScheduleError {
    InvalidAction(String),
    DuplicateActionId(String),
}

impl fmt::Display for ScheduleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidAction(id) => {
                write!(f, "invalid action identity or attempt estimate: {id:?}")
            }
            Self::DuplicateActionId(id) => write!(f, "duplicate action identity: {id:?}"),
        }
    }
}

impl std::error::Error for ScheduleError {}

pub fn schedule(
    budget: &Budget,
    actions: &[Action],
    weights: ScoreWeights,
    context: ScheduleContext,
) -> Result<Decision, ScheduleError> {
    Ok(schedule_with_trace(budget, actions, weights, context)?.decision)
}

pub fn schedule_with_trace(
    budget: &Budget,
    actions: &[Action],
    weights: ScoreWeights,
    context: ScheduleContext,
) -> Result<ScheduleTrace, ScheduleError> {
    let remaining = budget.remaining();
    let early_stop = if context.cancelled {
        Some(StopReason::Cancelled)
    } else if context.objective_met {
        Some(StopReason::ObjectiveMet)
    } else if remaining.attempts == 0 {
        Some(StopReason::AttemptLimit)
    } else {
        None
    };
    if let Some(reason) = early_stop {
        return Ok(ScheduleTrace {
            decision: Decision::Stop(reason),
            candidates: Vec::new(),
        });
    }

    let mut ordered: Vec<&Action> = actions.iter().collect();
    ordered.sort_unstable_by(|left, right| left.id.cmp(&right.id));
    let mut best: Option<(&Action, Ratio)> = None;
    let mut candidates = Vec::with_capacity(ordered.len());
    let mut previous_id: Option<&str> = None;
    let mut budget_denied = false;
    for action in ordered {
        if action.id.trim().is_empty() || action.estimated_cost.attempts != 1 {
            return Err(ScheduleError::InvalidAction(action.id.clone()));
        }
        if previous_id == Some(action.id.as_str()) {
            return Err(ScheduleError::DuplicateActionId(action.id.clone()));
        }
        previous_id = Some(&action.id);
        let evaluation = if !action.eligible {
            CandidateEvaluation::Ineligible
        } else if budget.can_reserve(action.kind, action.estimated_cost) {
            let score = score(action, weights, remaining);
            if best.is_none_or(|(_, current)| score.compare(current).is_gt()) {
                best = Some((action, score));
            }
            CandidateEvaluation::Ranked {
                numerator: score.numerator.to_string(),
                denominator: score.denominator.to_string(),
            }
        } else {
            budget_denied = true;
            CandidateEvaluation::Unaffordable
        };
        candidates.push(CandidateTrace {
            action_id: action.id.clone(),
            kind: action.kind,
            evaluation,
        });
    }
    let decision = if let Some((action, _)) = best {
        Decision::Run {
            action_id: action.id.clone(),
            kind: action.kind,
        }
    } else if budget_denied {
        Decision::Stop(StopReason::BudgetExhausted)
    } else {
        Decision::Stop(StopReason::NoFeasibleAction)
    };
    Ok(ScheduleTrace {
        decision,
        candidates,
    })
}

#[derive(Debug, Clone, Copy)]
struct Ratio {
    numerator: u128,
    denominator: u128,
}

impl Ratio {
    fn compare(self, other: Self) -> Ordering {
        let mut a = self;
        let mut b = other;
        let mut reversed = false;
        loop {
            let whole = (a.numerator / a.denominator).cmp(&(b.numerator / b.denominator));
            let a_rem = a.numerator % a.denominator;
            let b_rem = b.numerator % b.denominator;
            let order = if whole.is_ne() {
                whole
            } else if a_rem == 0 || b_rem == 0 {
                a_rem.cmp(&b_rem)
            } else {
                // Reciprocal comparison of the fractional remainders gives
                // the exact opposite order without a cross product.
                a = Self {
                    numerator: a.denominator,
                    denominator: a_rem,
                };
                b = Self {
                    numerator: b.denominator,
                    denominator: b_rem,
                };
                reversed = !reversed;
                continue;
            };
            return if reversed { order.reverse() } else { order };
        }
    }
}

fn score(action: &Action, weights: ScoreWeights, remaining: ResourceVec) -> Ratio {
    // The 16-bit inputs bound this sum below 2^33. Multiplying by one
    // 64-bit cap below fits in 97 bits.
    let gain = u128::from(action.expected_gain)
        + u128::from(weights.lambda) * u128::from(action.uncertainty_reduction)
        + u128::from(weights.mu) * u128::from(action.claim_importance);
    let mut cost = Ratio {
        numerator: 1,
        denominator: 1_000_000,
    };
    for (estimated, available) in action
        .estimated_cost
        .units()
        .into_iter()
        .zip(remaining.units())
    {
        if available > 0 {
            let ratio = Ratio {
                numerator: u128::from(estimated),
                denominator: u128::from(available),
            };
            if ratio.compare(cost).is_gt() {
                cost = ratio;
            }
        }
    }
    Ratio {
        numerator: gain * cost.denominator,
        denominator: cost.numerator,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn candidate(id: &str, cost: u64, score: u16) -> Action {
        Action {
            id: id.to_owned(),
            kind: ActionKind::Replay,
            estimated_cost: ResourceVec {
                wall_ms: cost,
                attempts: 1,
                ..ResourceVec::default()
            },
            expected_gain: score,
            uncertainty_reduction: 0,
            claim_importance: 0,
            eligible: true,
        }
    }

    #[test]
    fn ranking_is_exact_and_ties_are_stable() {
        let budget = Budget::with_quarter_reserve(ResourceVec {
            wall_ms: u64::MAX,
            attempts: 20,
            ..ResourceVec::default()
        });
        let actions = [
            candidate("b", u64::MAX - 1, u16::MAX),
            candidate("a", u64::MAX, u16::MAX),
            candidate("c", u64::MAX - 1, u16::MAX),
        ];
        let selected = Decision::Run {
            action_id: "b".to_owned(),
            kind: ActionKind::Replay,
        };
        let weights = ScoreWeights::default();
        let context = ScheduleContext::default();
        assert_eq!(
            schedule(&budget, &actions, weights, context).unwrap(),
            selected
        );
        let reversed: Vec<_> = actions.into_iter().rev().collect();
        assert_eq!(
            schedule(&budget, &reversed, weights, context).unwrap(),
            selected
        );
    }

    #[test]
    fn exact_ratio_order_agrees_with_small_cross_products() {
        let mut seed = 19_u64;
        let mut next = || {
            seed = seed.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
            u128::from((seed >> 32) % 10_000 + 1)
        };
        for _ in 0..512 {
            let a = Ratio {
                numerator: next(),
                denominator: next(),
            };
            let b = Ratio {
                numerator: next(),
                denominator: next(),
            };
            assert_eq!(
                a.compare(b),
                (a.numerator * b.denominator).cmp(&(b.numerator * a.denominator))
            );
        }
    }

    #[test]
    fn cancellation_precedes_objective_and_budget() {
        let budget = Budget::with_quarter_reserve(ResourceVec::default());
        let context = ScheduleContext {
            cancelled: true,
            objective_met: true,
        };
        assert_eq!(
            schedule(&budget, &[], ScoreWeights::default(), context).unwrap(),
            Decision::Stop(StopReason::Cancelled)
        );
    }
}
