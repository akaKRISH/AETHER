use std::collections::BTreeMap;
use std::fmt;

use serde::{Deserialize, Serialize};

use crate::ActionKind;

/// Cumulative resource dimensions. Instantaneous memory and concurrency limits
/// belong to execution policy and have no cumulative debit here.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResourceVec {
    pub wall_ms: u64,
    pub model_tokens: u64,
    pub attempts: u64,
    pub output_bytes: u64,
    pub paid_microusd: u64,
}

impl ResourceVec {
    pub(crate) fn units(self) -> [u64; 5] {
        [
            self.wall_ms,
            self.model_tokens,
            self.attempts,
            self.output_bytes,
            self.paid_microusd,
        ]
    }

    fn zip(self, other: Self, op: fn(u64, u64) -> Option<u64>) -> Option<Self> {
        Some(Self {
            wall_ms: op(self.wall_ms, other.wall_ms)?,
            model_tokens: op(self.model_tokens, other.model_tokens)?,
            attempts: op(self.attempts, other.attempts)?,
            output_bytes: op(self.output_bytes, other.output_bytes)?,
            paid_microusd: op(self.paid_microusd, other.paid_microusd)?,
        })
    }

    fn checked_add(self, other: Self) -> Option<Self> {
        self.zip(other, u64::checked_add)
    }

    fn checked_sub(self, other: Self) -> Option<Self> {
        self.zip(other, u64::checked_sub)
    }

    fn saturating_sub(self, other: Self) -> Self {
        Self {
            wall_ms: self.wall_ms.saturating_sub(other.wall_ms),
            model_tokens: self.model_tokens.saturating_sub(other.model_tokens),
            attempts: self.attempts.saturating_sub(other.attempts),
            output_bytes: self.output_bytes.saturating_sub(other.output_bytes),
            paid_microusd: self.paid_microusd.saturating_sub(other.paid_microusd),
        }
    }

    fn fits(self, other: Self) -> bool {
        self.units()
            .into_iter()
            .zip(other.units())
            .all(|(requested, available)| requested <= available)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReservationOutcome {
    Created,
    AlreadyRecorded,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BudgetError {
    ReserveExceedsCaps,
    InvalidAttempt,
    Unaffordable,
    AttemptConflict,
    UnknownAttempt,
    ActualExceedsReservation,
    Overflow,
}

impl fmt::Display for BudgetError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::ReserveExceedsCaps => "verification reserve exceeds an approved resource cap",
            Self::InvalidAttempt => "attempt requires a nonempty identity and exactly one unit",
            Self::Unaffordable => "estimated use exceeds spendable resources",
            Self::AttemptConflict => "attempt identity is bound to another reservation or outcome",
            Self::UnknownAttempt => "no reservation exists for this attempt",
            Self::ActualExceedsReservation => {
                "actual resource use exceeds the reserved upper bound"
            }
            Self::Overflow => "resource ledger arithmetic overflow or underflow",
        })
    }
}

impl std::error::Error for BudgetError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ReservationState {
    Pending,
    Settled(ResourceVec),
    Released,
}

#[derive(Debug, Clone, Copy)]
struct Reservation {
    kind: ActionKind,
    upper_bound: ResourceVec,
    state: ReservationState,
}

#[derive(Debug, Clone)]
pub struct Budget {
    caps: ResourceVec,
    verification_reserve: ResourceVec,
    used: ResourceVec,
    reserved: ResourceVec,
    verification_committed: ResourceVec,
    attempts: BTreeMap<String, Reservation>,
}

impl Budget {
    pub fn new(caps: ResourceVec, verification_reserve: ResourceVec) -> Result<Self, BudgetError> {
        if !verification_reserve.fits(caps) {
            return Err(BudgetError::ReserveExceedsCaps);
        }
        Ok(Self::from_valid_caps(caps, verification_reserve))
    }

    pub fn with_quarter_reserve(caps: ResourceVec) -> Self {
        Self::from_valid_caps(
            caps,
            ResourceVec {
                wall_ms: caps.wall_ms.div_ceil(4),
                model_tokens: caps.model_tokens.div_ceil(4),
                ..ResourceVec::default()
            },
        )
    }

    fn from_valid_caps(caps: ResourceVec, verification_reserve: ResourceVec) -> Self {
        Self {
            caps,
            verification_reserve,
            used: ResourceVec::default(),
            reserved: ResourceVec::default(),
            verification_committed: ResourceVec::default(),
            attempts: BTreeMap::new(),
        }
    }

    pub fn caps(&self) -> ResourceVec {
        self.caps
    }

    pub fn used(&self) -> ResourceVec {
        self.used
    }

    pub fn reserved(&self) -> ResourceVec {
        self.reserved
    }

    pub fn remaining(&self) -> ResourceVec {
        self.caps
            .saturating_sub(self.used)
            .saturating_sub(self.reserved)
    }

    pub fn verification_reserve(&self) -> ResourceVec {
        self.verification_reserve
    }

    pub fn unconsumed_verification_reserve(&self) -> ResourceVec {
        self.verification_reserve
            .saturating_sub(self.verification_committed)
    }

    pub fn can_reserve(&self, kind: ActionKind, upper_bound: ResourceVec) -> bool {
        let mut available = self.remaining();
        if kind == ActionKind::Explore {
            available = available.saturating_sub(self.unconsumed_verification_reserve());
        }
        upper_bound.attempts == 1 && upper_bound.fits(available)
    }

    /// Only a newly created reservation may authorize dispatch. Identities stay
    /// bound to their original reservation for the lifetime of the ledger.
    pub fn try_reserve(
        &mut self,
        id: &str,
        kind: ActionKind,
        upper_bound: ResourceVec,
    ) -> Result<ReservationOutcome, BudgetError> {
        if id.trim().is_empty() || upper_bound.attempts != 1 {
            return Err(BudgetError::InvalidAttempt);
        }
        if let Some(existing) = self.attempts.get(id) {
            return if existing.kind == kind && existing.upper_bound == upper_bound {
                Ok(ReservationOutcome::AlreadyRecorded)
            } else {
                Err(BudgetError::AttemptConflict)
            };
        }
        if !self.can_reserve(kind, upper_bound) {
            return Err(BudgetError::Unaffordable);
        }
        let next_reserved = self
            .reserved
            .checked_add(upper_bound)
            .ok_or(BudgetError::Overflow)?;
        let next_verification = if kind.is_verification() {
            self.verification_committed
                .checked_add(upper_bound)
                .ok_or(BudgetError::Overflow)?
        } else {
            self.verification_committed
        };
        self.attempts.insert(
            id.to_owned(),
            Reservation {
                kind,
                upper_bound,
                state: ReservationState::Pending,
            },
        );
        self.reserved = next_reserved;
        self.verification_committed = next_verification;
        Ok(ReservationOutcome::Created)
    }

    /// A launched attempt consumes one attempt even when execution failed. An
    /// overrun leaves the reservation held for the caller's stop and audit path.
    pub fn settle(&mut self, id: &str, actual: ResourceVec) -> Result<(), BudgetError> {
        if actual.attempts != 1 {
            return Err(BudgetError::InvalidAttempt);
        }
        self.finalize(id, ReservationState::Settled(actual), actual)
    }

    /// Release only an attempt that did not execute and consumed no resources.
    pub fn release(&mut self, id: &str) -> Result<(), BudgetError> {
        self.finalize(id, ReservationState::Released, ResourceVec::default())
    }

    fn finalize(
        &mut self,
        id: &str,
        outcome: ReservationState,
        actual: ResourceVec,
    ) -> Result<(), BudgetError> {
        let entry = self
            .attempts
            .get_mut(id)
            .ok_or(BudgetError::UnknownAttempt)?;
        match entry.state {
            existing if existing == outcome => return Ok(()),
            ReservationState::Pending => {}
            _ => return Err(BudgetError::AttemptConflict),
        }
        let refund = entry
            .upper_bound
            .checked_sub(actual)
            .ok_or(BudgetError::ActualExceedsReservation)?;
        let next_used = self.used.checked_add(actual).ok_or(BudgetError::Overflow)?;
        let next_reserved = self
            .reserved
            .checked_sub(entry.upper_bound)
            .ok_or(BudgetError::Overflow)?;
        let next_verification = if entry.kind.is_verification() {
            self.verification_committed
                .checked_sub(refund)
                .ok_or(BudgetError::Overflow)?
        } else {
            self.verification_committed
        };
        entry.state = outcome;
        self.used = next_used;
        self.reserved = next_reserved;
        self.verification_committed = next_verification;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn resource(wall_ms: u64, attempts: u64) -> ResourceVec {
        ResourceVec {
            wall_ms,
            attempts,
            ..ResourceVec::default()
        }
    }

    #[test]
    fn verification_refunds_restore_the_floor_and_retry_is_idempotent() {
        let mut budget = Budget::with_quarter_reserve(resource(100, 6));
        assert!(!budget.can_reserve(ActionKind::Explore, resource(76, 1)));
        assert_eq!(
            budget.try_reserve("replay", ActionKind::Replay, resource(25, 1)),
            Ok(ReservationOutcome::Created)
        );
        budget.settle("replay", resource(10, 1)).unwrap();
        budget.settle("replay", resource(10, 1)).unwrap();
        assert_eq!(
            budget.try_reserve("replay", ActionKind::Replay, resource(25, 1)),
            Ok(ReservationOutcome::AlreadyRecorded)
        );
        assert_eq!(budget.used(), resource(10, 1));
        assert_eq!(budget.unconsumed_verification_reserve().wall_ms, 15);
        assert!(budget.can_reserve(ActionKind::Explore, resource(75, 1)));
        assert!(!budget.can_reserve(ActionKind::Explore, resource(76, 1)));
        assert_eq!(budget.release("replay"), Err(BudgetError::AttemptConflict));
        assert_eq!(budget.reserved(), ResourceVec::default());
    }

    #[test]
    fn cap_invariants_hold_across_a_fixed_replay_of_mixed_outcomes() {
        let caps = resource(u64::MAX, 100);
        let mut budget = Budget::with_quarter_reserve(caps);
        for i in 0..100 {
            let identity = i.to_string();
            let estimate = resource(100 + i, 1);
            budget
                .try_reserve(&identity, ActionKind::Challenge, estimate)
                .unwrap();
            assert_eq!(
                budget.settle(&identity, resource(estimate.wall_ms + 1, 1)),
                Err(BudgetError::ActualExceedsReservation)
            );
            if i % 3 == 0 {
                budget.release(&identity).unwrap();
                budget.release(&identity).unwrap();
            } else {
                budget.settle(&identity, resource(i, 1)).unwrap();
            }
            assert_eq!(budget.used().checked_add(budget.remaining()), Some(caps));
        }
    }
}
