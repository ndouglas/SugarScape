//! Axelrod's culture model (milestone 14): "The Dissemination of Culture: A
//! Model with Local Convergence and Global Polarization", Journal of
//! Conflict Resolution 41 (1997), with Axtell, Axelrod, Epstein & Cohen's
//! docking departures (1996) and later literature's as named switches. See
//! docs/superpowers/specs/2026-09-25-culture-design.md.

mod config;
mod presets;
mod stats;
mod world;

pub use config::{schema, Activation, Changes, CultureConfig, Edges, Neighborhood};
pub use presets::presets;
pub use stats::{settle, CultureSnapshot, Sets, SERIES};
pub use world::{
    culture_color, hue_color, offsets, CultureCell, CultureInspection, CultureMode, CultureWorld,
    Lattice, NeighborView, SiteView,
};
