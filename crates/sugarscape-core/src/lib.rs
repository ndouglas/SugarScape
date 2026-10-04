//! Sugarscape, after Epstein & Axtell, *Growing Artificial Societies* (1996).
//!
//! Rule semantics follow Appendix B ("Summary of Rule Notation") and the
//! rule statements in Chapters II–III.

pub mod agent;
pub mod agreement;
pub mod anasazi;
pub mod ants;
pub mod auctions;
pub mod bali;
pub mod bits;
pub mod burrow;
pub mod civil;
pub mod classes;
pub mod collusion;
pub mod config;
pub mod culture;
pub mod dpd;
pub mod econ;
pub mod edit;
pub mod ethno;
pub mod export;
pub mod farol;
pub mod firms;
pub mod frames;
pub mod geometry;
pub mod graph;
pub mod hoard;
pub mod image;
pub mod landscape;
mod legacy;
pub mod line;
pub mod minds;
pub mod model;
pub mod network;
pub mod norms;
pub mod opinions;
pub mod polarity;
pub mod portable;
pub mod presets;
pub mod punishment;
pub mod render;
pub mod retirement;
pub mod ring;
pub mod rng;
pub mod rules;
pub mod schelling;
pub mod schema;
pub mod social;
pub mod spatial;
pub mod stats;
pub mod structure;
pub mod sweep;
pub mod tags;
pub mod thresholds;
pub mod tipping;
pub mod titles;
pub mod world;
pub mod zi;

#[cfg(test)]
pub(crate) mod testkit;

pub mod democratic_peace;
pub mod geosim;
