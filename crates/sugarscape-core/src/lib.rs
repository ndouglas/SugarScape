//! Sugarscape, after Epstein & Axtell, *Growing Artificial Societies* (1996).
//!
//! Rule semantics follow Appendix B ("Summary of Rule Notation") and the
//! rule statements in Chapters II–III.

pub mod agent;
pub mod config;
pub mod edit;
pub mod export;
pub mod geometry;
pub mod landscape;
pub mod presets;
pub mod render;
pub mod rng;
pub mod rules;
pub mod stats;
pub mod world;

#[cfg(test)]
pub(crate) mod testkit;
