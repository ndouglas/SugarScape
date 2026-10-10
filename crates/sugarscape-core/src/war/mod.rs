//! Native, opt-in reciprocal engagement benchmark.

pub mod checkpoint;
pub mod config;
pub mod engine;
pub mod math;
pub mod records;

pub use checkpoint::Checkpoint;
pub use engine::Engagement;
pub use records::{Casualty, EndReason, Ending, Exposure, Frame, StepFailure};

#[cfg(test)]
mod config_tests;
#[cfg(test)]
mod math_tests;

#[cfg(test)]
mod checkpoint_tests;
#[cfg(test)]
mod engine_tests;
