//! Ants and Recruitment (milestone 24): Kirman, "Ants, Rationality, and
//! Recruitment" (QJE 1993), with the extensions he names (Becker's majority
//! pull, more sources, meetings over a network) and Alfarano and Milaković's
//! (2007) agent rule and network critique as presets and switches. See
//! docs/superpowers/specs/2026-09-27-ants-design.md.

mod config;
mod presets;
mod stats;
mod theory;
mod view;
mod world;

pub use config::{schema, AntsConfig, Conversion, Network, Rule, Start, MAX_ANTS, MAX_SOURCES};
pub use presets::presets;
pub use stats::{AntsSnapshot, Regimes, Running, HOLD, SERIES};
pub use theory::{alfarano, join, kirman, kirman_alpha, kirman_rates, stationary, variance};
pub use view::{grid, row, row_of, GRID_X, HIST_X, SHOWN, TALL, TIME_W};
pub use world::{AntView, AntsCell, AntsInspection, AntsMode, AntsWorld};
