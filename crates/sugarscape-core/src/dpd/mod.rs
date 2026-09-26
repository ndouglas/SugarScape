//! The demographic Prisoner's Dilemma (milestone 17): Epstein, "Zones of
//! Cooperation in Demographic Prisoner's Dilemma" (SFI WP 97-12-094;
//! Complexity 4(2), 1998; Generative Social Science, 2006, ch. 9 and its
//! appendix), with the working paper's rule, Radax & Rengs' (2009) unstated
//! timing choices, soup, metabolism and the coordination game as named
//! switches. See docs/superpowers/specs/2026-09-26-demographic-pd-design.md.

mod config;
mod presets;
mod stats;
mod world;

pub use config::{
    schema, DeathTiming, DpdConfig, EndowmentFrom, MetabolismPer, NewbornAge, NewbornsAct, Pairing,
    Play, Removal, Shuffle, Updating, LIVE,
};
pub use presets::presets;
pub use stats::{DpdSnapshot, SERIES};
pub use world::{
    Agent, AgentView, DpdInspection, DpdMode, DpdWorld, NeighborView, Site, COOPERATOR, DEFECTOR,
};
