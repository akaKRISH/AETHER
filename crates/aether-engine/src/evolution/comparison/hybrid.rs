use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum HybridArm {
    Seeded,
    SeededFeasibleFlip,
    SeededRepair,
    SeededSwap,
    SeededLocal,
    FixedHybrid,
    AdaptiveHybrid,
}

impl HybridArm {
    pub const ALL: [Self; 7] = [
        Self::Seeded,
        Self::SeededFeasibleFlip,
        Self::SeededRepair,
        Self::SeededSwap,
        Self::SeededLocal,
        Self::FixedHybrid,
        Self::AdaptiveHybrid,
    ];

    fn repairs(self) -> bool {
        matches!(
            self,
            Self::SeededRepair | Self::FixedHybrid | Self::AdaptiveHybrid
        )
    }

    fn constrains_flip(self) -> bool {
        matches!(
            self,
            Self::SeededFeasibleFlip | Self::FixedHybrid | Self::AdaptiveHybrid
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
enum Operator {
    Empty,
    Seed,
    Immigrant,
    Flip,
    Swap,
    Local,
}

const OPERATORS: [Operator; 3] = [Operator::Flip, Operator::Swap, Operator::Local];

#[derive(Debug, PartialEq, Eq, Serialize)]
struct Operation {
    operator: Operator,
    #[serde(skip_serializing_if = "Option::is_none")]
    before_repair: Option<(KnapsackSelection, u16)>,
    seed_visits: u16,
    constraint_visits: u16,
    repair_visits: u16,
    repair_drops: u16,
    local_visits: u16,
    adaptive_comparisons: u16,
}

#[derive(Debug, PartialEq, Eq, Serialize)]
pub struct HybridTrace {
    density_comparisons: u16,
    operations: Vec<Operation>,
}

#[derive(Default)]
struct Allocation {
    uses: [u16; 3],
    gains: [u16; 3],
}

impl Allocation {
    fn adaptive(&self) -> usize {
        let explore = self.uses.contains(&0) || self.uses.iter().sum::<u16>().is_multiple_of(4);
        (0..OPERATORS.len())
            .max_by(|&a, &b| {
                let rate = |x: usize, y: usize| u32::from(self.gains[x]) * u32::from(self.uses[y]);
                let order = if explore {
                    std::cmp::Ordering::Equal
                } else {
                    rate(a, b).cmp(&rate(b, a))
                };
                order.then(self.uses[b].cmp(&self.uses[a])).then(b.cmp(&a))
            })
            .expect("three operators")
    }
}

fn pick_bit(mut mask: u16, draw: u64) -> usize {
    for _ in 0..draw % u64::from(mask.count_ones()) {
        mask &= mask - 1;
    }
    mask.trailing_zeros() as usize
}

impl Case {
    pub fn hybrid_grid() -> impl Iterator<Item = Self> {
        Self::development_grid().chain(
            [(901, "density_friendly", 15), (902, "density_trap", 10)].map(
                |(seed, label, decoy)| {
                    let items = [
                        (4, decoy),
                        (3, 7),
                        (3, 7),
                        (3, 7),
                        (5, 1),
                        (7, 2),
                        (8, 1),
                        (11, 1),
                    ];
                    Self {
                        seed,
                        size: 8,
                        difficulty: Difficulty::TightCorrelated,
                        items: items.map(|(weight, value)| Item { weight, value }).into(),
                        capacity: 9,
                        fixture: Some(label),
                    }
                },
            ),
        )
    }

    pub fn search_hybrid(
        &self,
        arm: HybridArm,
        evaluations: u16,
    ) -> Result<(Trial, HybridTrace), EvolutionError> {
        if !(8..=96).contains(&evaluations) {
            return Err(EvolutionError::EvaluationCap);
        }
        let cap = ResourceVec {
            attempts: u64::from(evaluations),
            ..EVALUATION
        };
        let mut budget =
            Budget::new(cap, ResourceVec::default()).map_err(EvolutionError::Budget)?;
        let mut pool = CandidatePool::default();
        let mut allocation = Allocation::default();
        let mut archive = Vec::with_capacity(usize::from(evaluations));
        let mut trace = HybridTrace {
            density_comparisons: 0,
            operations: Vec::with_capacity(usize::from(evaluations)),
        };
        let density = self.density_order(|| trace.density_comparisons += 1);
        let draw = |stream, slot| draw_for(self.seed, self.size, stream, slot);
        let mut feasible_evaluations = 0;
        let mut evaluations_to_best_value = 1;
        let mask = ((1u32 << self.size) - 1) as u16;
        for slot in 1..=evaluations {
            let id = slot.to_string();
            budget
                .try_reserve(&id, ActionKind::Explore, EVALUATION)
                .map_err(EvolutionError::Budget)?;
            let mut parent = pool.parent(slot - 1, |len| {
                (draw(Stream::Parent, slot) % len as u64) as usize
            });
            let chosen = parent.map(|_| match arm {
                HybridArm::Seeded | HybridArm::SeededFeasibleFlip | HybridArm::SeededRepair => 0,
                HybridArm::SeededSwap => 1,
                HybridArm::SeededLocal => 2,
                HybridArm::FixedHybrid => {
                    usize::from(allocation.uses.iter().sum::<u16>()) % OPERATORS.len()
                }
                HybridArm::AdaptiveHybrid => allocation.adaptive(),
            });
            let operator = match slot {
                1 => Operator::Empty,
                2 => Operator::Seed,
                _ => chosen.map_or(Operator::Immigrant, |index| OPERATORS[index]),
            };
            let mut work = Operation {
                operator,
                before_repair: None,
                seed_visits: 0,
                constraint_visits: 0,
                repair_visits: 0,
                repair_drops: 0,
                local_visits: 0,
                adaptive_comparisons: 0,
            };
            if chosen.is_some() && arm == HybridArm::AdaptiveHybrid {
                work.adaptive_comparisons = 2;
            }
            if operator == Operator::Local {
                parent = pool.best;
            }
            let mut genotype = match (operator, parent) {
                (Operator::Empty, _) => KnapsackSelection(0),
                (Operator::Seed, _) => {
                    work.seed_visits = u16::from(self.size);
                    self.greedy(&density, None)
                }
                (Operator::Immigrant, _) => {
                    KnapsackSelection(draw(Stream::SharedMask, slot) as u16 & mask)
                }
                (Operator::Flip | Operator::Swap, Some(parent)) => {
                    let selected = parent.genotype;
                    if operator == Operator::Swap && selected.0 != 0 && selected.0 != mask {
                        selected
                            .flip(pick_bit(selected.0, draw(Stream::SwapOut, slot)))
                            .flip(pick_bit(mask ^ selected.0, draw(Stream::SwapIn, slot)))
                    } else {
                        let start = (draw(Stream::Mutation, slot) % u64::from(self.size)) as usize;
                        if arm.constrains_flip() {
                            self.feasible_flip(parent, start, &mut work)
                        } else {
                            selected.flip(start)
                        }
                    }
                }
                (Operator::Local, Some(parent)) => self.local(
                    parent,
                    &density,
                    draw(Stream::LocalDrop, 0).wrapping_add(u64::from(allocation.uses[2])),
                    &mut work,
                ),
                _ => return Err(EvolutionError::InvalidTransition),
            };
            if arm.repairs()
                && matches!(
                    operator,
                    Operator::Immigrant | Operator::Flip | Operator::Swap
                )
            {
                genotype = self.repair(genotype, &density, &mut work);
            }
            let fitness = score(&self.items, self.capacity, genotype);
            feasible_evaluations += u16::from(fitness.feasible);
            let gain = if fitness.feasible {
                pool.best
                    .map_or(0, |best| fitness.value.saturating_sub(best.fitness.value))
            } else {
                0
            };
            if gain > 0 {
                evaluations_to_best_value = slot;
            }
            if let Some(index) = chosen {
                allocation.uses[index] += 1;
                allocation.gains[index] += gain;
            }
            archive.push(pool.record(
                Candidate {
                    slot,
                    genotype,
                    fitness,
                },
                parent,
            ));
            trace.operations.push(work);
            budget
                .settle(&id, EVALUATION)
                .map_err(EvolutionError::Budget)?;
        }
        Ok((
            Trial {
                best: pool.best.ok_or(EvolutionError::InvalidTransition)?,
                evaluations_to_best_value,
                candidate_evaluations: evaluations,
                feasible_evaluations,
                unique_genotypes: pool.unique.len(),
                archive,
            },
            trace,
        ))
    }

    fn feasible_flip(
        &self,
        parent: Candidate,
        start: usize,
        work: &mut Operation,
    ) -> KnapsackSelection {
        let remaining = self.capacity - parent.fitness.weight;
        for offset in 0..self.items.len() {
            let i = (start + offset) % self.items.len();
            work.constraint_visits += 1;
            if parent.genotype.0 & (1 << i) != 0 || self.items[i].weight <= remaining {
                return parent.genotype.flip(i);
            }
        }
        parent.genotype
    }

    fn repair(
        &self,
        mut selection: KnapsackSelection,
        density: &[usize],
        work: &mut Operation,
    ) -> KnapsackSelection {
        let mut weight = 0;
        for i in 0..self.items.len() {
            work.repair_visits += 1;
            if selection.0 & (1 << i) != 0 {
                weight += self.items[i].weight;
            }
        }
        if weight > self.capacity {
            work.before_repair = Some((selection, weight));
            for &i in density.iter().rev() {
                work.repair_visits += 1;
                if selection.0 & (1 << i) != 0 {
                    selection = selection.flip(i);
                    weight -= self.items[i].weight;
                    work.repair_drops += 1;
                    if weight <= self.capacity {
                        break;
                    }
                }
            }
        }
        selection
    }

    fn local(
        &self,
        parent: Candidate,
        density: &[usize],
        draw: u64,
        work: &mut Operation,
    ) -> KnapsackSelection {
        let mut selection = parent.genotype;
        let mut remaining = self.capacity - parent.fitness.weight;
        let dropped = (selection.0 != 0).then(|| pick_bit(selection.0, draw));
        if let Some(i) = dropped {
            selection = selection.flip(i);
            remaining += self.items[i].weight;
            work.local_visits += 1;
        }
        for &i in density {
            work.local_visits += 1;
            let weight = self.items[i].weight;
            if Some(i) != dropped && selection.0 & (1 << i) == 0 && weight <= remaining {
                selection = selection.flip(i);
                remaining -= weight;
            }
        }
        selection
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allocation_explores_and_uses_observed_gain_rates() {
        assert_eq!(Allocation::default().adaptive(), 0);
        assert_eq!(
            Allocation {
                uses: [1, 0, 0],
                gains: [20, 0, 0]
            }
            .adaptive(),
            1
        );
        assert_eq!(
            Allocation {
                uses: [2, 1, 2],
                gains: [5, 3, 5]
            }
            .adaptive(),
            1
        );
        assert_eq!(
            Allocation {
                uses: [4, 2, 2],
                gains: [50, 0, 0]
            }
            .adaptive(),
            1
        );
    }

    #[test]
    fn constructed_profiles_exercise_both_greedy_outcomes() {
        let cases: Vec<_> = Case::hybrid_grid()
            .filter(|case| case.fixture.is_some())
            .collect();
        for (case, expected) in cases.iter().zip([0, 4]) {
            let greedy = case.search(SearchArm::Greedy, 24).unwrap();
            assert_eq!(case.oracle().optimal_value - greedy.best_value(), expected);
            let (local, _) = case.search_hybrid(HybridArm::SeededLocal, 24).unwrap();
            assert_eq!(local.best_value(), case.oracle().optimal_value);
        }
    }

    #[test]
    fn operators_preserve_lineage_budget_and_repair_inputs() {
        let mut repairs = 0;
        let mut swaps = 0;
        for case in Case::hybrid_grid() {
            for arm in HybridArm::ALL {
                let (trial, trace) = case.search_hybrid(arm, 24).unwrap();
                let (prefix, _) = case.search_hybrid(arm, 8).unwrap();
                assert_eq!(prefix.archive, trial.archive[..8]);
                assert_eq!(trial.candidate_evaluations, 24);
                assert_eq!(trace.operations.len(), 24);
                assert_eq!(trace.operations[1].seed_visits, u16::from(case.size));
                assert!(case.verify(&trial).unwrap().passed);
                for (row, operation) in trial.archive.iter().zip(trace.operations) {
                    let Observation::Proposal {
                        candidate,
                        parent_slot,
                        ..
                    } = row
                    else {
                        panic!("expected proposal")
                    };
                    assert_eq!(
                        check(&case.items, case.capacity, candidate.genotype),
                        candidate.fitness
                    );
                    if operation.constraint_visits > 0 {
                        assert!(candidate.fitness.feasible && operation.before_repair.is_none());
                    }
                    if let Some((raw, weight)) = operation.before_repair {
                        assert!(weight > case.capacity && candidate.fitness.feasible);
                        assert_eq!(check(&case.items, case.capacity, raw).weight, weight);
                        assert_eq!(
                            (raw.0 ^ candidate.genotype.0).count_ones(),
                            u32::from(operation.repair_drops)
                        );
                        repairs += 1;
                    }
                    if let Some(slot) = parent_slot {
                        assert!(*slot < candidate.slot);
                        let Observation::Proposal {
                            candidate: parent, ..
                        } = &trial.archive[usize::from(*slot - 1)]
                        else {
                            panic!("invalid parent")
                        };
                        let raw = operation.before_repair.map_or(candidate.genotype, |x| x.0);
                        if operation.operator == Operator::Swap && parent.genotype.0 != 0 {
                            assert_eq!((raw.0 ^ parent.genotype.0).count_ones(), 2);
                            swaps += 1;
                        }
                    }
                }
            }
        }
        assert!(repairs > 0 && swaps > 0);
    }
}
