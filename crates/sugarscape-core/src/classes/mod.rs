//! The Emergence of Classes (milestone 15): Axtell, Epstein and Young, "The
//! Emergence of Classes in a Multi-Agent Bargaining Model" (2000), with Poza
//! et al.'s replication (2011) departures as named switches. See
//! docs/superpowers/specs/2026-09-25-classes-design.md.

mod config;
mod presets;
mod simplex;
mod stats;
mod world;

pub use config::{
    schema, ClassesConfig, Decision, Interaction, LatticeConfig, Layout, Neighborhood, Start,
    TagMemory,
};
pub use presets::presets;
pub use simplex::{mix, point, reply, GAP, SIDE, TALL};
pub use stats::{
    equity, expected_best, mode_best, regime, ClassesSnapshot, View, CLASSES, DIVIDED_BELOW,
    EQUITY, EQUITY_BETWEEN, FRACTIOUS, H, L, M, MIXED, SERIES,
};
pub use world::{
    letter, Bargainer, BargainerView, ClassesCell, ClassesInspection, ClassesMode, ClassesWorld,
    Memory,
};
