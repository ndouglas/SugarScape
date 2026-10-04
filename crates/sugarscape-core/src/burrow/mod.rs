//! A bounded excavation lab with explicit, conserved spoil.

mod access;
mod actions;
mod config;
mod controller;
pub mod fixtures;
mod ledger;
mod observation;
mod runner;
mod state;
mod validation;
mod view;
pub use validation::validate_episode;

pub use access::{
    run_access_episode, AccessConfig, AccessDiagnostics, AccessEpisode, AccessMilestone,
    AccessObjective, AccessSummary, AccessTask, GoalObservation,
};
pub use actions::{Action, ActionEvent, Outcome};
pub use config::{Cue, Fixture, LabConfig, Pile, Side, Transport};
pub use controller::{Decision, WorkerView};
pub use ledger::Delivery;
pub use observation::{ExitNeighbor, Observation, ObservedCell};
pub use runner::{
    run_episode, ChoiceEvent, DigDistance, Episode, Frame, Labels, LoadedTravel, Rates, RunOptions,
    Snapshot, SpatialWork, Storage, WorkerWork,
};
pub use state::{InitialSpoil, Inventory, Pos, Setup, World};

#[cfg(test)]
mod tests;
