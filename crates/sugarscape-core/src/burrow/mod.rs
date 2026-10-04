//! A bounded excavation lab with explicit, conserved spoil.

mod actions;
mod config;
mod controller;
pub mod fixtures;
mod observation;
mod state;

pub use actions::{Action, ActionEvent, Outcome};
pub use config::{Cue, Fixture, LabConfig, Pile, Side, Transport};
pub use controller::{Decision, WorkerView};
pub use observation::{ExitNeighbor, Observation, ObservedCell};
pub use state::{InitialSpoil, Inventory, Pos, Setup, World};

#[cfg(test)]
mod tests;
