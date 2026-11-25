use super::*;
use aether_engine::ScheduleTrace;
use aether_engine::reliability::{Priority, controlled_priorities};
use aether_engine::scheduler::CandidateEvaluation;

const SEEDS: u32 = 6;
const CAPS: [u64; 2] = [24, 84];
const HISTORIES: [(u8, u8); 4] = [(0, 1), (2, 1), (8, 1), (8, 4)];
const NOISE: [u8; 2] = [10, 30];

#[derive(Clone, Copy, Serialize)]
struct Controls {
    weight: DownstreamWeight,
    prior: u8,
    bonus: u8,
}

// Full factorial for the weak prior. Stronger priors compare fixed and updated
// means with the information term on and off. Keep the entire grid.
fn controls() -> Vec<Controls> {
    use DownstreamWeight::*;
    let mut values = Vec::new();
    for (prior, weights, bonuses) in [
        (
            1,
            &[DirectOnly, Full, PriorMean, PosteriorMean][..],
            &[0, 1, 2][..],
        ),
        (4, &[PriorMean, PosteriorMean][..], &[0, 1][..]),
        (12, &[PriorMean, PosteriorMean][..], &[0, 1][..]),
    ] {
        for &bonus in bonuses {
            values.extend(weights.iter().map(|&weight| Controls {
                weight,
                prior,
                bonus,
            }));
        }
    }
    values
}

#[derive(Clone, Copy, Serialize)]
pub(super) struct Condition {
    controls: Controls,
    groups: u8,
    copies_per_group: u8,
    pub noise: u8,
}

#[derive(Serialize)]
pub(super) struct AblationTrace {
    condition: Condition,
    initial_votes: Vec<(u8, u8)>,
    final_votes: Vec<(u8, u8)>,
    affordable_rankings: Vec<String>,
    probe_support_mask: u16,
}

// A group yields one vote. Mixed reports inside a group are ambiguous and abstain.
fn group_votes(receipts: impl IntoIterator<Item = (u8, bool)>) -> (u8, u8) {
    let (mut support, mut contradiction) = (0u8, 0u8);
    for (group, positive) in receipts {
        if positive {
            support |= 1 << group;
        } else {
            contradiction |= 1 << group;
        }
    }
    (
        (support & !contradiction).count_ones() as u8,
        (contradiction & !support).count_ones() as u8,
    )
}

impl Condition {
    pub fn initialize(
        self,
        seed: u32,
        family: u8,
        truth: &[bool],
        evidence: &mut [ClaimEvidence],
    ) -> AblationTrace {
        let initial_votes = truth
            .iter()
            .zip(evidence)
            .enumerate()
            .map(|(index, (&actual, claim))| {
                let history = (0..self.groups).flat_map(|group| {
                    let sign = actual ^ (draw(seed, family, index as u8, 61 + group) < self.noise);
                    std::iter::repeat_n((group, sign), usize::from(self.copies_per_group))
                });
                let counts = group_votes(history);
                (claim.support, claim.contradiction) = counts;
                counts
            })
            .collect();
        AblationTrace {
            condition: self,
            initial_votes,
            final_votes: Vec::new(),
            affordable_rankings: Vec::new(),
            probe_support_mask: 0,
        }
    }

    pub fn priorities(self, evidence: &[ClaimEvidence]) -> Result<Vec<Priority>, &'static str> {
        controlled_priorities(
            evidence,
            self.controls.weight,
            self.controls.prior,
            self.controls.bonus,
        )
    }
}

impl AblationTrace {
    pub fn record_order(&mut self, trace: &ScheduleTrace) -> Result<(), Box<dyn Error>> {
        let mut rows = Vec::new();
        for row in &trace.candidates {
            if let CandidateEvaluation::Ranked {
                numerator,
                denominator,
            } = &row.evaluation
            {
                let index: u8 = row
                    .action_id
                    .split_once('-')
                    .ok_or("invalid ranked ID")?
                    .0
                    .parse()?;
                rows.push((
                    index,
                    u128::from(numerator.parse::<u64>()?),
                    u128::from(denominator.parse::<u64>()?),
                ));
            }
        }
        // The exact scheduler ratios fit in u64 on this bounded grid. Equal scores retain ID order.
        rows.sort_unstable_by(|a, b| (b.1 * a.2).cmp(&(a.1 * b.2)).then(a.0.cmp(&b.0)));
        self.affordable_rankings
            .push(rows.iter().map(|row| char::from(b'a' + row.0)).collect());
        Ok(())
    }

    pub fn finish(&mut self, seed: u32, family: u8, truth: &[bool], evidence: &[ClaimEvidence]) {
        self.final_votes = evidence
            .iter()
            .map(|claim| (claim.support, claim.contradiction))
            .collect();
        for (index, &actual) in truth.iter().enumerate() {
            let probe = actual ^ (draw(seed, family, index as u8, 79) < self.condition.noise);
            self.probe_support_mask |= u16::from(probe) << index;
        }
    }
}

pub(super) fn run_grid(config: Config) -> Result<(), Box<dyn Error>> {
    let controls = controls();
    let mut timings: Vec<_> = controls.iter().map(|_| Timing::default()).collect();
    let mut outcomes = Vec::new();
    let start = Instant::now();
    for (family, scenario) in config.scenarios.iter().enumerate() {
        for seed in config.first_seed..config.first_seed + SEEDS {
            for cap in CAPS {
                for noise in NOISE {
                    for (groups, copies_per_group) in HISTORIES {
                        for (&controls, timing) in controls.iter().zip(&mut timings) {
                            let condition = Condition {
                                controls,
                                groups,
                                copies_per_group,
                                noise,
                            };
                            outcomes.push(run(
                                scenario,
                                family as u8,
                                seed,
                                cap,
                                Mode::EvidenceWeighted,
                                Some(condition),
                                timing,
                            )?);
                        }
                    }
                }
            }
        }
    }
    serde_json::to_writer(
        std::io::stdout().lock(),
        &serde_json::json!({
            "scope": "aether.ecar-reliability-ablation-development/1", "wall_ns": start.elapsed().as_nanos(),
            "timing_controls": controls, "timing": timings, "outcomes": outcomes,
        }),
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn correlated_copies_do_not_create_independent_votes() {
        assert_eq!(
            group_votes([(0, true), (0, true), (1, false), (1, false)]),
            (1, 1)
        );
        assert_eq!(group_votes([(0, true), (0, false), (1, true)]), (1, 0));
        let scenario = &config().unwrap().scenarios[1];
        let rich = Condition {
            controls: Controls {
                weight: DownstreamWeight::PosteriorMean,
                prior: 1,
                bonus: 1,
            },
            groups: 8,
            copies_per_group: 1,
            noise: 30,
        };
        let duplicated = Condition {
            copies_per_group: 4,
            ..rich
        };
        let replay = |condition| {
            run(
                scenario,
                1,
                301,
                84,
                Mode::EvidenceWeighted,
                Some(condition),
                &mut Timing::default(),
            )
            .unwrap()
        };
        let (clean, copies) = (replay(rich), replay(duplicated));
        assert_eq!(clean.trace, copies.trace);
        assert_eq!(clean.verified_utility, copies.verified_utility);
        let (clean, copies) = (clean.ablation.unwrap(), copies.ablation.unwrap());
        assert_eq!(clean.initial_votes, copies.initial_votes);
        assert_eq!(clean.final_votes, copies.final_votes);
        assert_eq!(clean.affordable_rankings, copies.affordable_rankings);
        assert_eq!(clean.probe_support_mask, copies.probe_support_mask);
    }
}
