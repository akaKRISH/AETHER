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
const STAGNATION_SLOTS: u16 = 24;
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
    InvalidCandidate,
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
            Self::InvalidCandidate => formatter.write_str("knapsack mask exceeds 12 items"),
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(transparent)]
pub struct KnapsackSelection(u16);

impl TryFrom<u16> for KnapsackSelection {
    type Error = EvolutionError;

    fn try_from(mask: u16) -> Result<Self, Self::Error> {
        if mask >= GENOTYPES {
            return Err(EvolutionError::InvalidCandidate);
        }
        Ok(Self(mask))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
struct Candidate {
    slot: u16,
    genotype: KnapsackSelection,
    fitness: Fitness,
}

impl Candidate {
    fn rank(&self) -> (u16, Reverse<u16>, Reverse<KnapsackSelection>) {
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

#[derive(Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
enum DiscoveryStop {
    BudgetExhausted,
    Stagnation,
}

#[derive(Debug, PartialEq, Eq, Serialize)]
pub struct DiscoveryReport {
    scope: &'static str,
    seed: u32,
    items: [Item; ITEM_COUNT],
    capacity: u16,
    evaluation_slots: u16,
    stagnation_slots: u16,
    stop: DiscoveryStop,
    oracle_optimal_value: u16,
    oracle_lightest_optimum_weight: u16,
    search: Arm,
}

/// Run a bounded, single-arm discovery on the reviewed synthetic knapsack fixture.
pub fn discover(seed: u32, evaluations: u16) -> Result<DiscoveryReport, EvolutionError> {
    let (items, capacity) = fixture(seed);
    let (value, Reverse(weight)) = exhaustive_optimum(&items, capacity);
    let search = search(
        Mode::Evolution,
        seed,
        &items,
        capacity,
        evaluations,
        value,
        Some(STAGNATION_SLOTS),
    )?;
    if !search.verified {
        return Err(EvolutionError::InvalidTransition);
    }
    Ok(DiscoveryReport {
        scope: "aether.knapsack-discovery/1",
        seed,
        items,
        capacity,
        evaluation_slots: evaluations,
        stagnation_slots: STAGNATION_SLOTS,
        stop: if search.proposal_count + search.verification_count == evaluations {
            DiscoveryStop::BudgetExhausted
        } else {
            DiscoveryStop::Stagnation
        },
        oracle_optimal_value: value,
        oracle_lightest_optimum_weight: weight,
        search,
    })
}

/// Compare four reviewed search schedules on the same seeded development fixture.
pub fn evolve(seed: u32, evaluations: u16) -> Result<EvolutionReport, EvolutionError> {
    if !(8..=96).contains(&evaluations) {
        return Err(EvolutionError::EvaluationCap);
    }
    let (items, capacity) = fixture(seed);
    let (value, Reverse(weight)) = exhaustive_optimum(&items, capacity);
    let run = |mode| search(mode, seed, &items, capacity, evaluations, value, None);
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

fn fixture(mut seed: u32) -> ([Item; ITEM_COUNT], u16) {
    let items = std::array::from_fn(|_| Item {
        weight: 2 + (draw(&mut seed) % 24) as u16,
        value: 5 + (draw(&mut seed) % 81) as u16,
    });
    let capacity = items.iter().map(|item| item.weight).sum::<u16>() / 3;
    (items, capacity)
}

fn search(
    mode: Mode,
    seed: u32,
    items: &[Item; ITEM_COUNT],
    capacity: u16,
    evaluations: u16,
    reference: u16,
    stagnation_slots: Option<u16>,
) -> Result<Arm, EvolutionError> {
    if !(8..=96).contains(&evaluations) {
        return Err(EvolutionError::EvaluationCap);
    }
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
        let final_slot = slot == evaluations
            || stagnation_slots
                .is_some_and(|limit| best.is_some_and(|candidate| slot - candidate.slot > limit));
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
                    parent.genotype.0 ^ (1 << (draw(&mut random) as usize % ITEM_COUNT))
                }
                (_, None) => (draw(&mut random) % u32::from(GENOTYPES)) as u16,
            };
            let genotype = KnapsackSelection::try_from(genotype)?;
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
        if final_slot {
            break;
        }
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
        verification_count: last_verification_slot - proposals,
        archive,
        ecar_decisions,
    })
}

fn draw(state: &mut u32) -> u32 {
    *state = state.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
    *state ^ (*state >> 16)
}

fn score(items: &[Item], capacity: u16, genotype: KnapsackSelection) -> Fitness {
    let (weight, value) = items
        .iter()
        .enumerate()
        .filter(|(position, _)| genotype.0 & (1 << position) != 0)
        .fold((0, 0), |(weight, value), (_, item)| {
            (weight + item.weight, value + item.value)
        });
    Fitness {
        weight,
        value,
        feasible: weight <= capacity,
    }
}

fn check(items: &[Item], capacity: u16, selection: KnapsackSelection) -> Fitness {
    let mut genotype = selection.0;
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
        let selection = KnapsackSelection::try_from(0b110).unwrap();
        assert_eq!(score(&items, 6, selection), check(&items, 6, selection));
        assert!(!check(&items, 6, KnapsackSelection::try_from(0b111).unwrap()).feasible);
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
                    assert_eq!((ancestor.genotype.0 ^ candidate.genotype.0).count_ones(), 1);
                }
            }
        }
        assert_eq!(study.arms[0].verification_count, 1);
        assert_eq!(study.arms[1].verification_count, 1);
        assert_eq!(study.arms[2].verification_count, 12);
        assert_eq!(study.arms[3].ecar_decisions.len(), 48);
        assert!(study.arms[3].verification_count > 1);
    }

    #[test]
    fn discovery_rejects_invalid_masks_and_caps() {
        assert!(KnapsackSelection::try_from(GENOTYPES - 1).is_ok());
        assert!(matches!(
            KnapsackSelection::try_from(GENOTYPES),
            Err(EvolutionError::InvalidCandidate)
        ));
        for cap in [0, 7, 97, u16::MAX] {
            assert!(matches!(
                discover(19, cap),
                Err(EvolutionError::EvaluationCap)
            ));
        }
    }

    #[test]
    fn discovery_reserves_final_check_at_either_stop() {
        let capped = discover(19, 8).unwrap();
        assert_eq!(capped.stop, DiscoveryStop::BudgetExhausted);
        assert_eq!(capped.search.archive.len(), 8);
        let stalled = discover(19, 96).unwrap();
        assert_eq!(stalled.stop, DiscoveryStop::Stagnation);
        assert_eq!(
            stalled.search.proposal_count - stalled.search.best.slot,
            STAGNATION_SLOTS
        );
        for report in [capped, stalled] {
            let arm = report.search;
            assert_eq!(arm.verification_count, 1);
            assert_eq!(arm.archive.len(), usize::from(arm.proposal_count + 1));
            assert!(arm.archive.len() <= usize::from(report.evaluation_slots));
            assert!(
                matches!(arm.archive.last(), Some(Observation::Verification {
                slot, candidate_slot, independently_scored, passed: true,
            }) if *slot == arm.proposal_count + 1 && *candidate_slot == arm.best.slot
                && *independently_scored == arm.best.fitness)
            );
        }
    }

    #[test]
    fn discovery_replays_distinct_candidates_and_valid_lineage() {
        let report = discover(19, 96).unwrap();
        assert_eq!(report, discover(19, 96).unwrap());
        assert_ne!(report.items, discover(281, 96).unwrap().items);
        let mut seen = BTreeSet::new();
        let mut mutations = 0;
        for (i, row) in report.search.archive.iter().enumerate() {
            let Observation::Proposal {
                candidate,
                parent_slot,
                disposition,
                ..
            } = row
            else {
                continue;
            };
            assert_eq!(usize::from(candidate.slot), i + 1);
            assert_eq!(
                candidate.fitness,
                check(&report.items, report.capacity, candidate.genotype)
            );
            assert_eq!(
                seen.insert(candidate.genotype),
                *disposition != Disposition::Duplicate
            );
            if let Some(parent) = parent_slot {
                assert!(*parent < candidate.slot);
                let Observation::Proposal {
                    candidate: ancestor,
                    disposition: Disposition::Feasible,
                    ..
                } = report.search.archive[usize::from(*parent - 1)]
                else {
                    panic!("invalid parent")
                };
                assert_eq!((candidate.genotype.0 ^ ancestor.genotype.0).count_ones(), 1);
                mutations += 1;
            }
        }
        assert!(mutations > 0);
        assert_eq!(seen.len(), report.search.unique_genotypes);
        assert!(seen.len() > 8);
        assert_eq!(
            report.search.best.fitness.value + report.search.value_regret,
            report.oracle_optimal_value
        );
    }
}
