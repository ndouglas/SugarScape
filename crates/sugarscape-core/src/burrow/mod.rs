//! A bounded excavation lab with explicit, conserved spoil.

mod actions;
mod config;
pub mod fixtures;
mod state;

pub use actions::{Action, ActionEvent, Outcome};
pub use config::{Cue, Fixture, LabConfig, Pile, Side, Transport};
pub use state::{InitialSpoil, Inventory, Pos, Setup, World};

#[cfg(test)]
mod tests;
