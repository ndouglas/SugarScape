//! Bounded Confidence (milestone 16): Hegselmann and Krause, "Opinion
//! Dynamics and Bounded Confidence: Models, Analysis, and Simulation" (JASSS
//! 2002), with its unfigured claims (serial updating, local neighborhoods) as
//! named switches. See docs/superpowers/specs/2026-09-25-bounded-confidence-design.md.

mod config;
mod presets;
mod stats;
mod view;
mod world;

pub use config::{
    schema, Confidence, Interaction, LatticeConfig, Neighborhood, OpinionsConfig, Start, Updating,
};
pub use presets::presets;
pub use stats::{clusters, median, splits, OpinionsSnapshot, SAME, SERIES, STILL};
pub use view::{hue, opinion_at, row, GAP, HISTORY, LATTICE_X, STEP, TALL, WIDE};
pub use world::{OpinionAgent, OpinionsCell, OpinionsInspection, OpinionsMode, OpinionsWorld};
