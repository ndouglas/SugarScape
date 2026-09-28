//! The Timing of Retirement (milestone 26): Axtell and Epstein, "Coordination
//! in Transient Social Networks: An Agent-Based Computational Model of the
//! Timing of Retirement" (Brookings CSED Working Paper 1, 1999), with the
//! revised text in Epstein, *Generative Social Science* (2006), chapter 7.
//! See docs/superpowers/specs/2026-09-27-retirement-design.md.

mod config;
mod presets;
mod stats;
mod view;
mod world;

pub use config::{
    schema, Counts, Groups, InitialDeaths, Order, Policy, Renewal, RetirementConfig, Size, COHORTS,
    OLDEST, YOUNGEST,
};
pub use presets::presets;
pub use stats::{RetirementSnapshot, AGES_WINDOW, SERIES};
pub use view::{population, row, AGES_W, GAP, ROW, SHOWN, TALL, TIME_W};
pub use world::{
    AgentView, Kind, RetirementCell, RetirementInspection, RetirementMode, RetirementWorld,
};
