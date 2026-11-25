//! Fixed development grid for a paired knapsack proposal comparison.
use super::*;

mod hybrid;
pub use hybrid::{HybridArm, HybridTrace};

pub const SIZES: [u8; 3] = [8, 12, 16];
pub const SEEDS: [u32; 3] = [19, 281, 617];
pub const CAPS: [u16; 2] = [8, 24];

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Difficulty {
    Relaxed,
    TightCorrelated,
}

impl Difficulty {
    pub const ALL: [Self; 2] = [Self::Relaxed, Self::TightCorrelated];
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SearchArm {
    Random,
    Greedy,
    Evolution,
}

impl SearchArm {
    pub const ALL: [Self; 3] = [Self::Random, Self::Greedy, Self::Evolution];
}

#[derive(Debug, Serialize)]
pub struct Case {
    seed: u32,
    size: u8,
    difficulty: Difficulty,
    items: Vec<Item>,
    capacity: u16,
    #[serde(skip_serializing_if = "Option::is_none")]
    fixture: Option<&'static str>,
}

#[derive(Debug, PartialEq, Eq, Serialize)]
pub struct Trial {
    best: Candidate,
    evaluations_to_best_value: u16,
    candidate_evaluations: u16,
    feasible_evaluations: u16,
    unique_genotypes: usize,
    archive: Vec<Observation>,
}

#[derive(Debug, PartialEq, Eq, Serialize)]
pub struct Verification {
    candidate_slot: u16,
    independently_scored: Fitness,
    pub passed: bool,
}

#[derive(Debug, PartialEq, Eq, Serialize)]
pub struct Oracle {
    pub optimal_value: u16,
    lightest_optimum_weight: u16,
}

// The fixed-width key makes distinct (seed, size, domain, index) tuples distinct.
// A bijective mixer scrambles the full word. Bounded draws can still repeat.
#[derive(Clone, Copy)]
#[repr(u8)]
enum Stream {
    Weight = 1,
    Value,
    SharedMask,
    Parent,
    Mutation,
    GreedyStart,
    SwapOut,
    SwapIn,
    LocalDrop,
}

fn draw_for(seed: u32, size: u8, stream: Stream, index: u16) -> u64 {
    let key = (u64::from(seed) << 32)
        | (u64::from(size) << 24)
        | (u64::from(stream as u8) << 16)
        | u64::from(index);
    let z = (key ^ (key >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    let z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    z ^ (z >> 31)
}

impl Case {
    /// Each capacity profile shares underlying weight and value draws.
    pub fn development_grid() -> impl Iterator<Item = Self> {
        SIZES.into_iter().flat_map(|size| {
            SEEDS.into_iter().flat_map(move |seed| {
                Difficulty::ALL.map(|difficulty| Self::new(seed, size, difficulty))
            })
        })
    }

    fn new(seed: u32, size: u8, difficulty: Difficulty) -> Self {
        let items: Vec<_> = (0..u16::from(size))
            .map(|i| {
                let weight = 2 + (draw_for(seed, size, Stream::Weight, i) % 24) as u16;
                let value = draw_for(seed, size, Stream::Value, i);
                let value = match difficulty {
                    Difficulty::Relaxed => 5 + (value % 81) as u16,
                    Difficulty::TightCorrelated => 3 * weight + (value % 5) as u16,
                };
                Item { weight, value }
            })
            .collect();
        let total = items.iter().map(|item| item.weight).sum::<u16>();
        let capacity = match difficulty {
            Difficulty::Relaxed => total / 2,
            Difficulty::TightCorrelated => total / 4,
        };
        Self {
            seed,
            size,
            difficulty,
            items,
            capacity,
            fixture: None,
        }
    }

    pub fn search(&self, arm: SearchArm, evaluations: u16) -> Result<Trial, EvolutionError> {
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
        let mut archive = Vec::with_capacity(usize::from(evaluations));
        let mut feasible_evaluations = 0;
        let mut evaluations_to_best_value = 1;
        let density = if arm == SearchArm::Greedy {
            self.density_order(|| {})
        } else {
            Vec::new()
        };
        let draw = |stream, slot| draw_for(self.seed, self.size, stream, slot);
        for slot in 1..=evaluations {
            let id = slot.to_string();
            budget
                .try_reserve(&id, ActionKind::Explore, EVALUATION)
                .map_err(EvolutionError::Budget)?;
            let parent = (arm == SearchArm::Evolution)
                .then(|| {
                    pool.parent(slot - 1, |len| {
                        (draw(Stream::Parent, slot) % len as u64) as usize
                    })
                })
                .flatten();
            let genotype = match (slot, arm, parent) {
                (1, _, _) => KnapsackSelection(0),
                (_, SearchArm::Greedy, _) => {
                    let start = (slot > 2).then(|| {
                        let offset = draw(Stream::GreedyStart, 0) % u64::from(self.size);
                        density[((offset + u64::from(slot - 3)) % u64::from(self.size)) as usize]
                    });
                    self.greedy(&density, start)
                }
                (_, _, Some(parent)) => parent
                    .genotype
                    .flip((draw(Stream::Mutation, slot) % u64::from(self.size)) as usize),
                _ => KnapsackSelection(
                    (draw(Stream::SharedMask, slot) & ((1 << self.size) - 1)) as u16,
                ),
            };
            let fitness = score(&self.items, self.capacity, genotype);
            feasible_evaluations += u16::from(fitness.feasible);
            if fitness.feasible
                && pool
                    .best
                    .is_some_and(|best| fitness.value > best.fitness.value)
            {
                evaluations_to_best_value = slot;
            }
            archive.push(pool.record(
                Candidate {
                    slot,
                    genotype,
                    fitness,
                },
                parent,
            ));
            budget
                .settle(&id, EVALUATION)
                .map_err(EvolutionError::Budget)?;
        }
        Ok(Trial {
            best: pool.best.ok_or(EvolutionError::InvalidTransition)?,
            evaluations_to_best_value,
            candidate_evaluations: evaluations,
            feasible_evaluations,
            unique_genotypes: pool.unique.len(),
            archive,
        })
    }

    fn density_order(&self, mut comparison: impl FnMut()) -> Vec<usize> {
        let mut density: Vec<_> = (0..self.items.len()).collect();
        density.sort_unstable_by(|&a, &b| {
            comparison();
            let (a_item, b_item) = (self.items[a], self.items[b]);
            (b_item.value * a_item.weight)
                .cmp(&(a_item.value * b_item.weight))
                .then(a.cmp(&b))
        });
        density
    }

    fn greedy(&self, density: &[usize], start: Option<usize>) -> KnapsackSelection {
        let mut selection = KnapsackSelection(0);
        let mut remaining = self.capacity;
        for i in start.into_iter().chain(density.iter().copied()) {
            let weight = self.items[i].weight;
            if selection.0 & (1 << i) == 0 && weight <= remaining {
                selection = selection.flip(i);
                remaining -= weight;
            }
        }
        selection
    }

    pub fn verify(&self, trial: &Trial) -> Result<Verification, EvolutionError> {
        let best = trial.best;
        let selection = KnapsackSelection::for_items(best.genotype.0, self.items.len())?;
        let independently_scored = check(&self.items, self.capacity, selection);
        Ok(Verification {
            candidate_slot: best.slot,
            independently_scored,
            passed: independently_scored == best.fitness && independently_scored.feasible,
        })
    }

    pub fn oracle(&self) -> Oracle {
        let (optimal_value, Reverse(lightest_optimum_weight)) =
            exhaustive_optimum(&self.items, self.capacity);
        Oracle {
            optimal_value,
            lightest_optimum_weight,
        }
    }
}

impl Trial {
    pub fn best_value(&self) -> u16 {
        self.best.fitness.value
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn full_draws_are_separated_by_seed_size_stream_and_index() {
        let mut draws = BTreeSet::new();
        for seed in SEEDS {
            for size in SIZES {
                for stream in [
                    Stream::Weight,
                    Stream::Value,
                    Stream::SharedMask,
                    Stream::Parent,
                    Stream::Mutation,
                    Stream::GreedyStart,
                ] {
                    for slot in 0..=96 {
                        assert!(draws.insert(draw_for(seed, size, stream, slot)));
                    }
                }
            }
        }
    }

    #[test]
    fn matched_caps_keep_prefixes_and_share_only_random_proposals() {
        for case in Case::development_grid() {
            let [random, _, evolved] = SearchArm::ALL.map(|arm| {
                let lower = case.search(arm, 8).unwrap();
                let upper = case.search(arm, 24).unwrap();
                assert_eq!(lower.archive, upper.archive[..8]);
                assert_eq!(upper.archive.len(), 24);
                assert!(case.verify(&upper).unwrap().passed);
                assert_eq!(upper.candidate_evaluations, 24);
                assert!(upper.feasible_evaluations <= 24);
                assert!(upper.unique_genotypes <= 24);
                upper
            });
            let mut mutations = 0;
            for (row, random_row) in evolved.archive.iter().zip(&random.archive) {
                let Observation::Proposal {
                    candidate,
                    parent_slot,
                    ..
                } = row
                else {
                    panic!("expected proposal")
                };
                assert_eq!(
                    candidate.fitness,
                    check(&case.items, case.capacity, candidate.genotype)
                );
                match parent_slot {
                    None => {
                        let Observation::Proposal {
                            candidate: random_candidate,
                            ..
                        } = random_row
                        else {
                            panic!("expected random proposal")
                        };
                        assert_eq!(candidate.genotype, random_candidate.genotype);
                    }
                    Some(slot) => {
                        assert!(*slot < candidate.slot);
                        let Observation::Proposal {
                            candidate: parent,
                            disposition: Disposition::Feasible,
                            ..
                        } = evolved.archive[usize::from(*slot - 1)]
                        else {
                            panic!("invalid parent")
                        };
                        assert_eq!((candidate.genotype.0 ^ parent.genotype.0).count_ones(), 1);
                        mutations += 1;
                    }
                }
            }
            assert!(mutations > 0);
        }
    }
}
