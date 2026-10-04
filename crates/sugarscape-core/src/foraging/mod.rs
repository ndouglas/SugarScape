//! Stateless CPFA rule reference after Hecker and Moses (2015).
//!
//! This module calculates source rules from explicit inputs; it does not
//! simulate movement, resources, or the waypoint server.

mod config;
mod rules;

pub use config::CpfaParameters;
pub use rules::{informed_variation, poisson_cdf, uninformed_variation, waypoint_strength};

/// Supplied reference-utility count bound, not a biological limit.
pub const MAX_RESOURCE_COUNT: u32 = 256;
/// Supplied reference-utility fidelity/publication rate bound.
pub const MAX_INFORMATION_RATE: f64 = 256.0;
/// Source waypoint expiration threshold; equality conventions are explicit
/// in the information decision API.
pub const WAYPOINT_THRESHOLD: f64 = 0.001;

#[cfg(test)]
mod tests;
