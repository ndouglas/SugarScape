//! Epstein's civil violence model (milestone 11). See
//! docs/superpowers/specs/2026-09-25-civil-violence-design.md.

mod math;
mod stats;

pub use math::exp_neg;
pub use stats::{CivilSnapshot, Outbursts, SERIES};
