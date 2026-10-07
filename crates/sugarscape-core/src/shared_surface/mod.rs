//! Paid finite surface transitions, local observations, and exact supplied protocols.

mod controller;
mod diagnostics;
mod evaluation;
mod inference;
mod protocol;
mod types;
mod world;

pub use controller::{decide, predict};
pub use diagnostics::*;
pub use evaluation::*;
pub use inference::Ensemble;
pub use protocol::{action_cost, role_at, validate_action};
pub use types::*;
pub use world::World;

pub const INITIAL_CREDITS: u8 = 48;
pub const TRIALS: u8 = 4;
pub const MAX_CANDIDATE_WORLDS: usize = 1_024;

#[cfg(test)]
mod tests {
    mod diagnostics;
    mod evaluation;
    mod inference;
    mod world;
}
