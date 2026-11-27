#![forbid(unsafe_code)]

pub mod budget;
pub mod evolution;
pub mod quality;
pub mod reliability;
pub mod scheduler;
pub mod simulation;

pub use budget::{Budget, BudgetError, ReservationOutcome, ResourceVec};
pub use evolution::{DiscoveryReport, EvolutionError, EvolutionReport, discover, evolve};
pub use scheduler::{
    Action, ActionKind, Decision, ScheduleContext, ScheduleError, ScheduleTrace, ScoreWeights,
    StopReason, schedule, schedule_with_trace,
};
pub use simulation::{SimulationError, StudyConfig, StudyMode, StudyReport, simulate};
