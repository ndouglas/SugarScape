//! Paid active experiments composed with the frozen shared surface physics.

mod kernel;
mod types;

pub use kernel::{advance_one, apply_choice, select_choice, EpisodeState, Step};
pub use types::*;

#[cfg(test)]
mod tests;
