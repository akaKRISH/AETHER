use std::cmp::Reverse;
use std::collections::BTreeSet;
use std::fmt;

use serde::Serialize;

use crate::{
    Action, ActionKind, Budget, BudgetError, Decision, ResourceVec, ScheduleContext, ScheduleError,
    ScheduleTrace, ScoreWeights, schedule_with_trace,
};

const ITEM_COUNT: usize = 12;
const GENOTYPES: u16 = 1 << ITEM_COUNT;
const EVALUATION: ResourceVec = ResourceVec {
    attempts: 1,
    wall_ms: 0,
    model_tokens: 0,
    output_bytes: 0,
    paid_microusd: 0,
};

#[derive(Debug)]
pub enum EvolutionError {
    EvaluationCap,
    InvalidTransition,
    Budget(BudgetError),
    Schedule(ScheduleError),
}

impl fmt::Display for EvolutionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EvaluationCap => {
                formatter.write_str("knapsack search requires 8–96 evaluation slots")
            }
            Self::InvalidTransition => {
                formatter.write_str("knapsack search has no eligible action or incumbent")
            }
            Self::Budget(error) => error.fmt(formatter),
            Self::Schedule(error) => error.fmt(formatter),
        }
    }
}

impl std::error::Error for EvolutionError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
struct Item {
    weight: u16,
    value: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
struct Fitness {
    weight: u16,
    value: u16,
    feasible: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
enum Mode {
    Random,
    Evolution,
    FixedVerification,
    EcarGuided,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
struct Candidate {
    slot: u16,
    genotype: u16,
    fitness: Fitness,
}

impl Candidate {
    fn rank(&self) -> (u16, Reverse<u16>, Reverse<u16>) {
        (
            self.fitness.value,
            Reverse(self.fitness.weight),
            Reverse(self.genotype),
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
enum Disposition {
    Infeasible,
    Duplicate,
    Feasible,
}

#[derive(Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum Observation {
    Proposal {
        candidate: Candidate,
        parent_slot: Option<u16>,
        disposition: Disposition,
        improves_incumbent: bool,
    },
    Verification {
        slot: u16,
        candidate_slot: u16,
        independently_scored: Fitness,
        passed: bool,
    },
}

#[derive(Debug, PartialEq, Eq, Serialize)]
struct Arm {
    mode: Mode,
    best: Candidate,
    verified: bool,
    value_regret: u16,
    unique_genotypes: usize,
    proposal_count: u16,
    verification_count: u16,
    archive: Vec<Observation>,
    ecar_decisions: Vec<ScheduleTrace>,
}

#[derive(Debug, PartialEq, Eq, Serialize)]
pub struct EvolutionReport {
    scope: &'static str,
    seed: u32,
    items: [Item; ITEM_COUNT],
    capacity: u16,
    evaluation_slots_per_arm: u16,
    oracle_optimal_value: u16,
    oracle_lightest_optimum_weight: u16,
    arms: [Arm; 4],
}

/// Compare four reviewed search schedules on the same seeded development fixture.
pub fn evolve(seed: u32, evaluations: u16) -> Result<EvolutionReport, EvolutionError> {
    if !(8..=96).contains(&evaluations) {
        return Err(EvolutionError::EvaluationCap);
    }
    let mut state = seed;
    let items = std::array::from_fn(|_| Item {
        weight: 2 + (draw(&mut state) % 24) as u16,
        value: 5 + (draw(&mut state) % 81) as u16,
    });
    let capacity = items.iter().map(|item| item.weight).sum::<u16>() / 3;
    let (value, Reverse(weight)) = exhaustive_optimum(&items, capacity);
    let run = |mode| search(mode, seed, &items, capacity, evaluations, value);
    Ok(EvolutionReport {
        scope: "aether.knapsack-development/1",
        seed,
        items,
        capacity,
        evaluation_slots_per_arm: evaluations,
        oracle_optimal_value: value,
        oracle_lightest_optimum_weight: weight,
        arms: [
            run(Mode::Random)?,
            run(Mode::Evolution)?,
            run(Mode::FixedVerification)?,
            run(Mode::EcarGuided)?,
        ],
    })
}

fn search(
    mode: Mode,
    seed: u32,
    items: &[Item; ITEM_COUNT],
    capacity: u16,
    evaluations: u16,
    reference: u16,
) -> Result<Arm, EvolutionError> {
    let cap = ResourceVec {
        attempts: u64::from(evaluations),
        ..EVALUATION
    };
    let mut budget = Budget::new(cap, EVALUATION).map_err(EvolutionError::Budget)?;
    let mut archive = Vec::with_capacity(usize::from(evaluations));
    let mut ecar_decisions = Vec::new();
    let mut unique = BTreeSet::new();
    let mut pool: Vec<Candidate> = Vec::new();
    let mut best: Option<Candidate> = None;
    let mut last_verified_candidate_slot = None;
    let mut last_verification_slot = 0;
    let mut proposals: u16 = 0;
    let mut verified = false;
    let mut random = seed ^ 0x321f_194b;

    for slot in 1..=evaluations {
        let final_slot = slot == evaluations;
        let action = if mode == Mode::EcarGuided {
            let changed_candidate =
                best.is_some_and(|candidate| Some(candidate.slot) != last_verified_candidate_slot);
            let elapsed = slot - last_verification_slot;
            let trace = schedule_with_trace(
                &budget,
                &[
                    Action {
                        id: "propose".into(),
                        kind: ActionKind::Explore,
                        estimated_cost: EVALUATION,
                        expected_gain: 8,
                        uncertainty_reduction: 0,
                        claim_importance: 0,
                        eligible: !final_slot,
                    },
                    Action {
                        id: "verify-incumbent".into(),
                        kind: ActionKind::Replay,
                        estimated_cost: EVALUATION,
                        expected_gain: 0,
                        uncertainty_reduction: elapsed.min(10),
                        claim_importance: if changed_candidate { 4 } else { 0 },
                        eligible: final_slot || changed_candidate,
                    },
                ],
                ScoreWeights::default(),
                ScheduleContext::default(),
            )
            .map_err(EvolutionError::Schedule)?;
            let Decision::Run { kind, .. } = trace.decision else {
                return Err(EvolutionError::InvalidTransition);
            };
            ecar_decisions.push(trace);
            kind
        } else if final_slot || (mode == Mode::FixedVerification && slot.is_multiple_of(4)) {
            ActionKind::Replay
        } else {
            ActionKind::Explore
        };

        let slot_id = slot.to_string();
        budget
            .try_reserve(&slot_id, action, EVALUATION)
            .map_err(EvolutionError::Budget)?;
        let observation = if action.is_verification() {
            let candidate = best.ok_or(EvolutionError::InvalidTransition)?;
            let checked = check(items, capacity, candidate.genotype);
            verified = checked == candidate.fitness && checked.feasible;
            last_verified_candidate_slot = Some(candidate.slot);
            last_verification_slot = slot;
            Observation::Verification {
                slot,
                candidate_slot: candidate.slot,
                independently_scored: checked,
                passed: verified,
            }
        } else {
            let parent = if mode != Mode::Random && proposals > 2 && !proposals.is_multiple_of(4) {
                pool.get(draw(&mut random) as usize % pool.len().max(1))
                    .copied()
            } else {
                None
            };
            let genotype = match (proposals, parent) {
                (0, _) => 0,
                (_, Some(parent)) => {
                    parent.genotype ^ (1 << (draw(&mut random) as usize % ITEM_COUNT))
                }
                (_, None) => (draw(&mut random) % u32::from(GENOTYPES)) as u16,
            };
            let fitness = score(items, capacity, genotype);
            let candidate = Candidate {
                slot,
                genotype,
                fitness,
            };
            let fresh = unique.insert(genotype);
            let improves_incumbent =
                fitness.feasible && best.is_none_or(|old| candidate.rank() > old.rank());
            if improves_incumbent {
                best = Some(candidate);
            }
            let disposition = if !fresh {
                Disposition::Duplicate
            } else if !fitness.feasible {
                Disposition::Infeasible
            } else {
                // ponytail: retain eight distinct feasible parents with periodic
                // random immigration. Revisit diversity if the search space grows.
                pool.push(candidate);
                pool.sort_unstable_by_key(|parent| Reverse(parent.rank()));
                pool.truncate(8);
                Disposition::Feasible
            };
            proposals += 1;
            Observation::Proposal {
                candidate,
                parent_slot: parent.map(|candidate| candidate.slot),
                disposition,
                improves_incumbent,
            }
        };
        budget
            .settle(&slot_id, EVALUATION)
            .map_err(EvolutionError::Budget)?;
        archive.push(observation);
    }

    let best = best.ok_or(EvolutionError::InvalidTransition)?;
    Ok(Arm {
        mode,
        best,
        verified,
        value_regret: reference
            .checked_sub(best.fitness.value)
            .ok_or(EvolutionError::InvalidTransition)?,
        unique_genotypes: unique.len(),
        proposal_count: proposals,
        verification_count: evaluations - proposals,
        archive,
        ecar_decisions,
    })
}

fn draw(state: &mut u32) -> u32 {
    *state = state.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
    *state ^ (*state >> 16)
}

fn score(items: &[Item], capacity: u16, genotype: u16) -> Fitness {
    let (weight, value) = items
        .iter()
        .enumerate()
        .filter(|(position, _)| genotype & (1 << position) != 0)
        .fold((0, 0), |(weight, value), (_, item)| {
            (weight + item.weight, value + item.value)
        });
    Fitness {
        weight,
        value,
        feasible: weight <= capacity,
    }
}

fn check(items: &[Item], capacity: u16, mut genotype: u16) -> Fitness {
    let mut weight = 0;
    let mut value = 0;
    while genotype != 0 {
        let item = items[genotype.trailing_zeros() as usize];
        weight += item.weight;
        value += item.value;
        genotype &= genotype - 1;
    }
    Fitness {
        weight,
        value,
        feasible: weight <= capacity,
    }
}

fn exhaustive_optimum(items: &[Item], capacity: u16) -> (u16, Reverse<u16>) {
    let Some((item, rest)) = items.split_first() else {
        return (0, Reverse(0));
    };
    let excluded = exhaustive_optimum(rest, capacity);
    if item.weight > capacity {
        return excluded;
    }
    let (value, Reverse(weight)) = exhaustive_optimum(rest, capacity - item.weight);
    excluded.max((value + item.value, Reverse(weight + item.weight)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exhaustive_oracle_checks_a_known_case() {
        let items = [
            Item {
                weight: 3,
                value: 7,
            },
            Item {
                weight: 4,
                value: 8,
            },
            Item {
                weight: 2,
                value: 4,
            },
        ];
        assert_eq!(exhaustive_optimum(&items, 6), (12, Reverse(6)));
        assert_eq!(score(&items, 6, 0b110), check(&items, 6, 0b110));
        assert!(!check(&items, 6, 0b111).feasible);
    }

    #[test]
    fn matched_arms_preserve_lineage_and_final_checks() {
        assert!(matches!(evolve(0, 7), Err(EvolutionError::EvaluationCap)));
        assert!(matches!(evolve(0, 97), Err(EvolutionError::EvaluationCap)));
        let study = evolve(19, 48).unwrap();
        assert_eq!(study, evolve(19, 48).unwrap());
        assert_ne!(study.items, evolve(281, 48).unwrap().items);
        for arm in &study.arms {
            assert_eq!(arm.archive.len(), 48);
            assert_eq!(arm.proposal_count + arm.verification_count, 48);
            assert!(arm.verified);
            assert!(arm.best.fitness.feasible);
            assert_eq!(
                arm.best.fitness.value + arm.value_regret,
                study.oracle_optimal_value
            );
            assert!(matches!(
                arm.archive.last(),
                Some(Observation::Verification {
                    slot: 48,
                    passed: true,
                    ..
                })
            ));
            for record in &arm.archive {
                if let Observation::Proposal {
                    candidate,
                    parent_slot: Some(parent),
                    ..
                } = record
                {
                    assert!(parent < &candidate.slot);
                    let Observation::Proposal {
                        candidate: ancestor,
                        ..
                    } = arm.archive[usize::from(*parent - 1)]
                    else {
                        panic!("parent slot must hold a candidate")
                    };
                    assert_eq!((ancestor.genotype ^ candidate.genotype).count_ones(), 1);
                }
            }
        }
        assert_eq!(study.arms[0].verification_count, 1);
        assert_eq!(study.arms[1].verification_count, 1);
        assert_eq!(study.arms[2].verification_count, 12);
        assert_eq!(study.arms[3].ecar_decisions.len(), 48);
        assert!(study.arms[3].verification_count > 1);
    }
}
