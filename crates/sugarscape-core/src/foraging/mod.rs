//! CPFA stateless rules and fixed-world reconstruction after Hecker and Moses (2015).
//!
//! The top-level API calculates source rules from explicit inputs. The
//! [`fixed`] submodule provides the separate fixed-world reconstruction.

//! The caller owns observations, publication order, retained memory and random
//! variates. Publication and fidelity use independent draws; recruitment uses
//! a third draw. These example values are supplied, not evolved defaults.
//!
//! ```
//! use sugarscape_core::foraging::{
//!     departure, publication, CpfaParameters, Departure, FindRecord,
//!     Waypoint, WaypointSelection, WaypointThreshold,
//! };
//! let parameters = CpfaParameters {
//!     p_search: 0.5, p_return: 0.5, omega: 1.0,
//!     lambda_informed: 1.0, lambda_fidelity: 1.0,
//!     lambda_publish: 1.0, lambda_waypoint: 1.0,
//! };
//! let find = Some(FindRecord { site: 77, count: 1 });
//! let snapshot = [Waypoint { id: 9, site: 88, strength: 0.8 }];
//! assert_eq!(publication(&parameters, find, 0.8).unwrap(), None);
//! assert_eq!(departure(&parameters, find, &snapshot,
//!     WaypointSelection::LaterArgosStrengthWeighted,
//!     WaypointThreshold::LaterArgosStrict, 0.7, 0.0).unwrap(),
//!     Departure::SiteFidelity { site: 77 });
//! assert_eq!(publication(&parameters, find, 0.7).unwrap(), Some(77));
//! assert_eq!(departure(&parameters, find, &snapshot,
//!     WaypointSelection::LaterArgosStrengthWeighted,
//!     WaypointThreshold::LaterArgosStrict, 0.8, 0.0).unwrap(),
//!     Departure::Recruitment { waypoint: 9, site: 88 });
//! ```

pub mod fixed;
pub mod passage;

mod config;
mod information;
mod rules;

pub use config::CpfaParameters;
pub use information::{
    departure, publication, Departure, FindRecord, Waypoint, WaypointSelection, WaypointThreshold,
};
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
