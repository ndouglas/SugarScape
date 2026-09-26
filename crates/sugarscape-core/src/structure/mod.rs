//! Social Structure (milestone 18): Cohen, Riolo and Axelrod, "The Role of
//! Social Structure in the Maintenance of Cooperative Regimes" (Rationality
//! and Society 2001), with the points the paper leaves unstated or states
//! twice as named switches. See docs/superpowers/specs/2026-09-26-social-structure-design.md.

mod config;
mod graph;
mod presets;
mod stats;
mod view;
mod world;

pub use config::{schema, square_side, NoiseOn, Start, Structure, StructureConfig};
pub use graph::{fanout, Graph};
pub use presets::presets;
pub use stats::{payoff, regression, StructureSnapshot, SERIES};
pub use view::{block_side, cell, class, frame, plane_cell, plane_x, PLANE, TRAIL};
pub use world::{
    AgentView, PartnerView, Strategy, StructureCell, StructureInspection, StructureMode,
    StructureWorld,
};
