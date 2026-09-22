//! Sugarscape, after Epstein & Axtell, *Growing Artificial Societies* (1996).
//!
//! Rule semantics follow Appendix B ("Summary of Rule Notation") and the
//! rule statements in Chapters II–III.

pub mod agent;
pub mod config;
pub mod geometry;
pub mod landscape;
pub mod rng;
pub mod rules;
pub mod world;

#[cfg(test)]
pub(crate) mod testkit;
