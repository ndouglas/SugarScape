//! Ethnocentrism (milestone 16): Hammond & Axelrod, "The Evolution of
//! Ethnocentrism", J. Conflict Resolution 50 (2006), with its appendix's
//! and its archived code's departures, and the variants of Hartshorn,
//! Kaznatcheev & Shultz (JASSS 2013) and Jansson (JASSS 2013), as named
//! switches. See docs/superpowers/specs/2026-09-25-ethnocentrism-design.md.

mod config;
mod presets;
mod stats;
mod world;

pub use config::{
    schema, Discrimination, EthnoConfig, KinBasis, Offspring, PairPlay, Start, Strategy, LIVE,
};
pub use presets::presets;
pub use stats::{EthnoSnapshot, SERIES};
pub use world::{
    lineage_color, tag_color, Agent, AgentView, EthnoInspection, EthnoMode, EthnoWorld,
    NeighborView, Site, ETHNOCENTRIC, HUMANITARIAN, KIN, MIXED, NONKIN, SELFISH, TRAITOROUS,
};
