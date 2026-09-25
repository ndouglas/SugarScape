//! Epstein's civil violence model (milestone 11): "Modeling civil
//! violence: An agent-based computational approach", PNAS 99 suppl. 3
//! (2002). Model I — agents rebel against a central authority whose cops
//! arrest them — and Model II — two ethnic groups whose active members
//! kill each other, with cloning and death by age — as one model kind,
//! with NetLogo *Rebellion*'s departures as named switches. See
//! docs/superpowers/specs/2026-09-25-civil-violence-design.md.

mod config;
mod presets;
mod stats;
mod world;

pub use crate::portable::exp_neg;
pub use config::{
    is_live, schema, CivilConfig, Jail, Quirks, Ramp, Variant, Vision, LIVE, RAMPABLE,
};
pub use presets::presets;
pub use stats::{CivilSnapshot, Outbursts, SERIES};
pub use world::{
    sight, Citizen, CitizenView, CivilInspection, CivilMode, CivilWorld, Cop, CopView, SiteXy,
    Term, COP, GREEN,
};
