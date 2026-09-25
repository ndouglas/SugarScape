//! Sugarscape, after Epstein & Axtell, *Growing Artificial Societies* (1996).
//!
//! Rule semantics follow Appendix B ("Summary of Rule Notation") and the
//! rule statements in Chapters II–III.

pub mod agent;
pub mod bits;
pub mod config;
pub mod econ;
pub mod edit;
pub mod export;
pub mod geometry;
pub mod landscape;
mod legacy;
pub mod model;
pub mod network;
pub mod presets;
pub mod render;
pub mod rng;
pub mod rules;
pub mod social;
pub mod stats;
pub mod sweep;
pub mod world;

#[cfg(test)]
pub(crate) mod testkit;
