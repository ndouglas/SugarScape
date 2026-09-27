//! Relative Agreement (milestone 22): Deffuant, Amblard, Weisbuch and Faure,
//! "How Can Extremism Prevail?" (JASSS 2002), with the pairwise bounded
//! confidence of Deffuant, Neau, Amblard and Weisbuch (2000), the networks of
//! Amblard and Deffuant (2004) and Weisbuch (2004), and Meadows and Cliff's
//! (2012) and the authors' (2013) readings as named switches. See
//! docs/superpowers/specs/2026-09-26-relative-agreement-design.md.

mod config;
mod network;
mod presets;
mod stats;
mod view;
mod world;

pub use crate::graph::Graph;
pub use config::{
    schema, AgreementConfig, LatticeConfig, Network, PairUpdate, Pairing, Placement, Rule,
    ScaleFreeConfig, SmallWorldConfig, Substrate, Window, MAX_AGENTS, MAX_SIDE,
};
pub use presets::presets;
pub use stats::{grouping, AgreementSnapshot, Outcome, GAP, SERIES, STILL};
pub use view::{row, scatter_col, COLUMNS, KEPT, SCATTER_X, TALL, TORUS_X};
pub use world::{
    extremist_counts, influence, AgreementAgent, AgreementCell, AgreementInspection, AgreementMode,
    AgreementWorld, Role,
};
