//! Image scoring (milestone 21): Nowak & Sigmund, "Evolution of indirect
//! reciprocity by image scoring" (Nature 393, 1998) and its Methods, with
//! Leimar & Hammerstein's (Proc. R. Soc. B 268, 2001) island model, errors,
//! own-score, standing and q strategies, and each unstated choice as a named
//! switch. See docs/superpowers/specs/2026-09-26-image-scoring-design.md.

pub mod analytic;
mod config;
mod presets;
mod stats;
mod strategy;
mod world;

pub use config::{
    schema, ImageConfig, Information, Initial, Offset, Records, RoundsKind, Seeded, LIVE,
};
pub use presets::presets;
pub use stats::{ImageSnapshot, SERIES};
pub use strategy::{Class, Situation, Strategy, Tallies, K_MAX, K_MIN};
pub use world::{Agent, AgentView, Cell, ImageInspection, ImageMode, ImageWorld};
