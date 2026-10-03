//! Cederman's emergent polarity, with named reconstruction choices.
mod config;
pub use config::*;
mod alliance;
mod analysis;
mod combat;
mod decision;
mod resources;
mod territory;
pub use analysis::{Category, Episode, Event, Ledger, Outcome};
mod presets;
mod stats;
mod world;
pub use presets::presets;
pub use stats::{series_names, PolaritySnapshot, SERIES};
pub use world::PolarityWorld;
mod view;

#[cfg(test)]
mod tests;
