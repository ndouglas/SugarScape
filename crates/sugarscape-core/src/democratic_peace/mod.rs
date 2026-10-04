//! Cederman2001 selection with independently specified dynamics.
mod config;
pub use config::*;
mod resources;
mod types;
pub use types::*;
mod world;
pub use world::DemocraticPeaceWorld;
mod stats;
pub use stats::{series_names, DemocraticPeaceSnapshot, SERIES};
mod presets;
mod view;
pub use presets::presets;
mod alliances;
mod claims;
mod combat;
mod decisions;
mod territory;
#[cfg(test)]
mod tests;
