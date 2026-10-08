//! Paid active experiments composed with the frozen shared surface physics.

mod belief;
mod kernel;
mod planner;
mod types;

pub use kernel::{advance_one, apply_choice, select_choice, EpisodeState, Step};
pub use planner::{predict_target, ChoiceValue, CompiledPolicy, Decision, SearchStats};
pub use types::*;

#[cfg(test)]
mod tests;
