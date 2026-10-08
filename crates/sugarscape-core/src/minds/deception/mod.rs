//! Checked P4 construction and public evidence for supplied caching gestures.
//! This model makes no claim of learned deceptive intent or animal cognition.
pub mod accounting;
pub mod controller;
pub mod lab;
pub mod observation;
pub mod records;
pub mod runner;
pub mod state;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod runner_tests;

pub const SCHEMA: &str = "minds-deception-measured-v1";
pub use lab::{condition_id, conditions};
pub use records::{EpisodeFailure, EpisodeRecord, FrameRecord};
pub use runner::{run_episode, run_episode_with_sink};
