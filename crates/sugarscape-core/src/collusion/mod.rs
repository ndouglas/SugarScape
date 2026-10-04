//! Algorithmic Collusion: Calvano, Calzolari, Denicolò and Pastorello,
//! "Artificial Intelligence, Algorithmic Pricing, and Collusion" (AER 2020),
//! checked against the authors' own code, with the critics' tests — Asker,
//! Fershtman and Pakes; Lambin; Epivent and Lambin; den Boer, Meylahn and
//! Schinkel; Eschenbaum, Mellgren and Zahn — as named switches.
//! See docs/superpowers/specs/2026-10-01-algorithmic-collusion-design.md.

pub mod analysis;
mod config;
pub mod demand;
pub mod learner;
mod presets;
pub mod ran2;
mod stats;
mod view;
mod world;

pub use config::{
    schema, BestResponseTo, CollusionConfig, EquilibriumCheck, Exploration, Grid, Impulse, QInit,
    RngKind, Ties, Update, BELOW_BOTTOM, Q_BUDGET,
};
pub use presets::{as_coded, presets};
pub use stats::{CollusionSnapshot, SERIES};
pub use world::{CollusionInspection, CollusionMode, CollusionWorld, Outcome, StateView};
