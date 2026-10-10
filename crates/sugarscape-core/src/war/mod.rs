//! Native, opt-in reciprocal engagement benchmark.

pub mod checkpoint;
pub mod config;
pub mod engine;
pub mod math;
pub mod records;
pub mod reference;
pub mod runner;

pub use checkpoint::Checkpoint;
pub use config::StudyInput;
pub use engine::Engagement;
pub use records::{
    BookDeath, BookFrame, BookKill, Capture, CapturedFrame, EqualityBasis, ObservedFrame,
    RunFailure, RunHeader, RunPayload, RunRecord, RunSummary, UnavailableObservation,
};
pub use records::{Casualty, EndReason, Ending, Exposure, Frame, StepFailure};
pub use reference::{reference_at, ReferenceFailure, ReferencePoint, ReferenceRegime};
pub use runner::run_to;

#[cfg(test)]
mod config_tests;
#[cfg(test)]
mod math_tests;

#[cfg(test)]
mod checkpoint_tests;
#[cfg(test)]
mod engine_tests;

#[cfg(test)]
mod reference_tests;

#[cfg(test)]
mod runner_tests;
