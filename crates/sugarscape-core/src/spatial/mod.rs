//! Spatial games (milestone 12): the spatial Prisoner's Dilemma of Nowak &
//! May, "Evolutionary games and spatial chaos", Nature 359 (1992), with the
//! asynchronous updating of Huberman & Glance (PNAS 90, 1993) and the
//! probabilistic winning, continuous time, irregular arrays and cubes of
//! Nowak, Bonhoeffer & May (PNAS 91, 1994). See
//! docs/superpowers/specs/2026-09-25-spatial-games-design.md.

mod config;
mod geometry;
mod presets;
mod stats;
mod world;

pub use config::{
    schema, Boundary, Lattice, Neighborhood, SpatialConfig, Start, Update, Winning, LIVE,
};
pub use geometry::{offsets, Geometry, EMPTY};
pub use presets::presets;
pub use stats::{SpatialSnapshot, SERIES};
pub use world::{
    Candidate, CellXyz, PlayerView, SpatialInspection, SpatialMode, SpatialWorld, C_AFTER_C,
    C_AFTER_D, D_AFTER_C, D_AFTER_D,
};
