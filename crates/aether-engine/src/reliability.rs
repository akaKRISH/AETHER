//! Experimental action priorities from observed evidence. The scheduler still controls admission.

use crate::Action;

const UNIT: u64 = 32;

#[derive(Clone, Copy, Debug, Default)]
pub struct ClaimEvidence {
    pub parent: Option<usize>,
    pub weight: u16,
    pub support: u8,
    pub contradiction: u8,
    // Set only by the controller after a trusted rejection.
    pub rejected: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DownstreamWeight {
    DirectOnly,
    Full,
    PriorMean,
    PosteriorMean,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Priority {
    pub blocked: bool,
    importance: u16,
    verification_information: u16,
}

impl Priority {
    pub fn apply(self, action: &mut Action) -> Result<(), &'static str> {
        let scaled = |value: u16| {
            value
                .checked_mul(UNIT as u16)
                .ok_or("experimental score exceeds bound")
        };
        let expected_gain = scaled(action.expected_gain)?;
        let uncertainty_reduction = if action.kind.is_verification() {
            self.verification_information
        } else {
            scaled(action.uncertainty_reduction)?
        };
        action.expected_gain = expected_gain;
        action.uncertainty_reduction = uncertainty_reduction;
        action.claim_importance = self.importance;
        action.eligible &= !self.blocked;
        Ok(())
    }
}

/// The Beta(1,1) model estimates support on the next usable evidence receipt.
/// A failed execution contributes no receipt. Ranks cannot change a verdict.
pub fn priorities(
    claims: &[ClaimEvidence],
    weighting: DownstreamWeight,
) -> Result<Vec<Priority>, &'static str> {
    controlled_priorities(claims, weighting, 1, 1)
}

/// Sensitivity controls for the development policy. Prior strength is per Beta side.
pub fn controlled_priorities(
    claims: &[ClaimEvidence],
    weighting: DownstreamWeight,
    prior: u8,
    information_multiplier: u8,
) -> Result<Vec<Priority>, &'static str> {
    if !(1..=12).contains(&prior) || information_multiplier > 2 {
        return Err("prior or information multiplier outside development bounds");
    }
    if !(1..=16).contains(&claims.len()) {
        return Err("expected 1–16 claim evidence records");
    }
    let mut blocked = [false; 16];
    let mut mass = [0; 16];
    for (index, claim) in claims.iter().enumerate() {
        if !(1..=100).contains(&claim.weight) || claim.parent.is_some_and(|p| p >= index) {
            return Err("invalid claim weight or dependency order");
        }
        blocked[index] = claim.rejected || claim.parent.is_some_and(|p| blocked[p]);
        mass[index] = if blocked[index] {
            0
        } else {
            u64::from(claim.weight)
        };
    }
    for (index, claim) in claims.iter().enumerate().rev() {
        if let Some(parent) = claim.parent {
            mass[parent] += mass[index];
        }
    }
    claims
        .iter()
        .enumerate()
        .map(|(index, claim)| {
            if blocked[index] {
                return Ok(Priority {
                    blocked: true,
                    importance: 0,
                    verification_information: 0,
                });
            }
            let a = u64::from(prior) + u64::from(claim.support);
            let b = u64::from(prior) + u64::from(claim.contradiction);
            let n = a + b;
            let direct = u64::from(claim.weight);
            let downstream = (mass[index] - direct) * UNIT;
            let weighted = match weighting {
                DownstreamWeight::DirectOnly => 0,
                DownstreamWeight::Full => downstream,
                DownstreamWeight::PriorMean => downstream / 2,
                DownstreamWeight::PosteriorMean => downstream * a / n,
            };
            // Var(E[theta | next usable receipt]) is the expected variance reduction.
            // Multiply by 36 so cold-start information has the same scale as claim mass.
            let information = mass[index] * UNIT * 36 * u64::from(information_multiplier) * a * b
                / (n * n * (n + 1) * (n + 1));
            Ok(Priority {
                blocked: false,
                importance: (direct * UNIT + weighted) as u16,
                verification_information: u16::try_from(information)
                    .map_err(|_| "information score exceeds bound")?,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        ActionKind, Budget, Decision, ResourceVec, ScheduleContext, ScoreWeights, schedule,
    };

    fn claim(weight: u16, parent: Option<usize>) -> ClaimEvidence {
        ClaimEvidence {
            weight,
            parent,
            ..ClaimEvidence::default()
        }
    }

    #[test]
    fn provisional_contradiction_retains_information_and_trusted_rejection_blocks_descendants() {
        let mut claims = [
            claim(2, None),
            claim(40, Some(0)),
            claim(30, Some(1)),
            claim(8, None),
        ];
        let prior = priorities(&claims, DownstreamWeight::PosteriorMean).unwrap();
        assert_eq!(prior[0].verification_information, 72 * 32);
        claims[0].contradiction = 2;
        let weighted = priorities(&claims, DownstreamWeight::PosteriorMean).unwrap();
        let unweighted = priorities(&claims, DownstreamWeight::Full).unwrap();
        let prior_only = priorities(&claims, DownstreamWeight::PriorMean).unwrap();
        assert_eq!(weighted[0].importance, (2 * 32 + 70 * 8));
        assert_eq!(prior[0].importance, prior_only[0].importance);
        assert_eq!(
            weighted[0].verification_information,
            unweighted[0].verification_information
        );
        assert!(weighted[0].verification_information > weighted[3].verification_information);
        assert!(weighted.iter().all(|p| !p.blocked));
        claims[0].rejected = true;
        let rejected = priorities(&claims, DownstreamWeight::PosteriorMean).unwrap();
        assert!(rejected[..3].iter().all(|p| p.blocked));
        assert!(!rejected[3].blocked);
        claims[2].parent = Some(3);
        assert!(priorities(&claims, DownstreamWeight::Full).is_err());
    }

    #[test]
    fn priorities_preserve_reserve_cost_ranking_ties_and_existing_denial() {
        let action = |id: &str, kind, cost, eligible| Action {
            id: id.into(),
            kind,
            estimated_cost: ResourceVec {
                wall_ms: cost,
                attempts: 1,
                ..ResourceVec::default()
            },
            expected_gain: 10,
            uncertainty_reduction: 10,
            claim_importance: 0,
            eligible,
        };
        let budget = Budget::with_quarter_reserve(ResourceVec {
            wall_ms: 12,
            attempts: 10,
            ..ResourceVec::default()
        });
        let mut actions = [
            action("b", ActionKind::Replay, 4, true),
            action("a", ActionKind::Replay, 4, true),
            action("c", ActionKind::Replay, 8, true),
            action("explore", ActionKind::Explore, 10, true),
            action("denied", ActionKind::Replay, 1, false),
        ];
        let priority = priorities(&[claim(10, None)], DownstreamWeight::PosteriorMean).unwrap()[0];
        for action in &mut actions {
            priority.apply(action).unwrap();
        }
        assert_eq!(
            schedule(
                &budget,
                &actions,
                ScoreWeights::default(),
                ScheduleContext::default()
            )
            .unwrap(),
            Decision::Run {
                action_id: "a".into(),
                kind: ActionKind::Replay
            }
        );
    }

    #[test]
    fn controls_change_only_the_intended_terms() {
        let mut claims = [claim(2, None), claim(12, Some(0))];
        claims[0].support = 8;
        let score =
            |weight, prior, bonus| controlled_priorities(&claims, weight, prior, bonus).unwrap()[0];
        use DownstreamWeight::*;
        assert_eq!(score(DirectOnly, 1, 1).importance, 2 * 32);
        assert_eq!(score(PriorMean, 1, 1).importance, 8 * 32);
        assert_eq!(score(Full, 1, 1).importance, 14 * 32);
        assert!(score(PosteriorMean, 1, 1).importance > score(PosteriorMean, 12, 1).importance);
        assert_eq!(score(PosteriorMean, 1, 0).verification_information, 0);
        assert_eq!(
            score(PosteriorMean, 1, 0).importance,
            score(PosteriorMean, 1, 2).importance
        );
        assert!(
            score(PriorMean, 1, 2).verification_information
                > score(PriorMean, 1, 1).verification_information
        );
    }
}
