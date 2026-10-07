//! Shared-worker excavation and food return.
mod draws;
mod knowledge;
mod metrics;
mod navigation;
mod observation;
mod setup;
mod terrain;
pub use crate::foraging::passage::{Parameters, Pos, Resource};
pub use knowledge::{CellKnowledge, KnownCell};
pub use setup::Setup;
pub use terrain::TerrainInventory;
type Checked<T> = Result<T, Vec<crate::config::FieldError>>;
#[cfg(test)]
mod tests;

mod access;
mod food;
mod server;
mod spoil;
mod state;
pub use access::{EventContext, EventMilestone, Milestones};
pub use food::{FoodInventory, FoodState, FoodView};
pub use spoil::{SpoilInventory, SpoilState, SpoilView};
pub use state::{Cargo, FoodPhase, WorkCounts};
