//! Fixed passage foraging with immediate observations and private topology.
mod draws;
mod knowledge;
mod metrics;
mod navigation;
mod observation;
mod setup;
pub use knowledge::{CellKnowledge, KnownCell};
pub use setup::{Parameters, Pos, Resource, Setup};
type Checked<T> = Result<T, Vec<crate::config::FieldError>>;
mod ledger;
mod server;
mod state;
#[cfg(test)]
mod tests;
pub use ledger::{Inventory, ResourceState, ResourceView};
pub use server::ServerRecordView;
pub use state::{Phase, WorkCounts};
mod actions;
mod controller;
mod decision;
mod world;
pub use world::World;
