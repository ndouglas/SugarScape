//! Norms and Metanorms (milestone 20): Axelrod, "An Evolutionary Approach to
//! Norms" (APSR 1986), with Galán and Izquierdo's re-implementation (JASSS
//! 2005) — its departures and its readings of what Axelrod left unstated — as
//! named switches. See docs/superpowers/specs/2026-09-26-norms-design.md.

mod config;
mod presets;
mod stats;
mod view;
mod world;

pub use config::{schema, AllEqual, GroupsConfig, NormsConfig, Refill, Selection};
pub use presets::presets;
pub use stats::{collapsed, established, NormsSnapshot, SERIES};
pub use view::{frame, level_at, level_cell, row_top, LEVEL, PLANE, STRIP, STRIP_X, TRAIL};
pub use world::{Agent, NormAgentView, NormsCell, NormsInspection, NormsMode, NormsWorld};
