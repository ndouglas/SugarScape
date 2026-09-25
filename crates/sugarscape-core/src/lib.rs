//! Sugarscape, after Epstein & Axtell, *Growing Artificial Societies* (1996).
//!
//! Rule semantics follow Appendix B ("Summary of Rule Notation") and the
//! rule statements in Chapters II–III.

pub mod agent;
pub mod anasazi;
pub mod bits;
pub mod civil;
pub mod config;
pub mod culture;
pub mod econ;
pub mod edit;
pub mod export;
pub mod geometry;
pub mod landscape;
mod legacy;
pub mod model;
pub mod network;
pub mod portable;
pub mod presets;
pub mod render;
pub mod ring;
pub mod rng;
pub mod rules;
pub mod schelling;
pub mod schema;
pub mod social;
pub mod spatial;
pub mod stats;
pub mod sweep;
pub mod tags;
pub mod world;

#[cfg(test)]
pub(crate) mod testkit;
