//! Cederman's GeoSim war-size reconstruction with explicit source readings.
mod config;
mod resources;
#[cfg(test)]
mod tests;
pub use config::*;
mod wars;
pub use wars::{Merge, Participant, StateId, War, WarTracker};
mod territory;
pub use territory::{Cell, State};
mod analysis;
pub use analysis::{Event, Ledger, Outcome, ResourceUpdate};
mod world;
pub use world::GeosimWorld;
mod stats;
pub use stats::{series_names, GeosimSnapshot, SERIES};
mod presets;
mod view;
pub use presets::presets;
pub use world::Front;
mod combat;
pub use combat::{
    contest_probability, losses, losses_with_incidence, resolve_victory, victory_probabilities,
    Victory,
};
mod claims;
mod decision;
