//! The Emergence of Firms (milestone 30): Axtell, "The Emergence of Firms in a
//! Population of Agents" (Brookings CSED Working Paper 3, 1999), with his
//! 2013 parameterization ("Endogenous Dynamics of Firms and Labor with Large
//! Numbers of Simple Agents"). See
//! docs/superpowers/specs/2026-09-30-emergence-of-firms-design.md.

mod config;
pub mod effort;
pub mod fit;
mod presets;
mod stats;
mod view;
mod world;

pub use config::{
    schema, Activation, AdjustScope, BasePay, CesSign, EffortSearch, FirmsConfig, Initial, Network,
    OthersEffort, Pay, Preferences, RandomBehavior, SeniorityOrder,
};
pub use presets::presets;
pub use stats::{FirmsSnapshot, SERIES};
pub use view::{FIRMS_H, FIRMS_W, PLOT_X, TALL, WIDE};
pub use world::{
    Agent, AgentView, Firm, FirmView, FirmsCell, FirmsInspection, FirmsMode, FirmsWorld,
};
