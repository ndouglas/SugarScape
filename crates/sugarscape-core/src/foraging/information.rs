//! Stateless information decisions with explicit paper and later-source variants.

use std::collections::BTreeSet;

use crate::config::FieldError;

use super::config::{checked, closed_interval, resource_count, uniform_draw};
use super::{poisson_cdf, CpfaParameters, WAYPOINT_THRESHOLD};

/// Caller-supplied valid observation; site identifiers have no sentinel values.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FindRecord {
    pub site: u64,
    pub count: u32,
}

/// Caller-supplied snapshot record; strength is current, not an age to expire.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Waypoint {
    pub id: u64,
    pub site: u64,
    pub strength: f64,
}

/// The paper does not fully specify selection among server records.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WaypointSelection {
    /// Supplied comparison rule with equal intervals among active records.
    UniformComparison,
    /// Strength-weighted selection from the inspected later ARGoS source.
    LaterArgosStrengthWeighted,
}

/// Explicit equality conventions for the source expiration threshold.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WaypointThreshold {
    /// Paper wording expires strength below 0.001, retaining equality.
    PaperBelow,
    /// Inspected later ARGoS source requires strength strictly above 0.001.
    LaterArgosStrict,
}

/// Next-trip information choice, without movement or resource-truth inspection.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Departure {
    Uninformed,
    SiteFidelity { site: u64 },
    Recruitment { waypoint: u64, site: u64 },
}

/// Request publication using the displayed lower-tail CDF and strict comparison.
/// The independent supplied draw is validated even without a find. A returned
/// site is a request, not an insertion into a server.
pub fn publication(
    parameters: &CpfaParameters,
    find: Option<FindRecord>,
    draw: f64,
) -> Result<Option<u64>, Vec<FieldError>> {
    let mut errors = parameters.validate().err().unwrap_or_default();
    if let Some(find) = find {
        resource_count(&mut errors, "find.count", find.count);
    }
    uniform_draw(&mut errors, "draw", draw);
    checked(errors)?;
    match find {
        Some(find) if poisson_cdf(find.count, parameters.lambda_publish)? > draw => {
            Ok(Some(find.site))
        }
        _ => Ok(None),
    }
}

/// Validate every original input, then prefer private fidelity over recruitment.
/// Records retain caller order; inactive records are still validated. Neither
/// memory nor snapshots are mutated, and no hidden random draws are consumed.
pub fn departure(
    parameters: &CpfaParameters,
    memory: Option<FindRecord>,
    waypoints: &[Waypoint],
    selection: WaypointSelection,
    threshold: WaypointThreshold,
    fidelity_draw: f64,
    recruitment_draw: f64,
) -> Result<Departure, Vec<FieldError>> {
    let mut errors = parameters.validate().err().unwrap_or_default();
    if let Some(memory) = memory {
        resource_count(&mut errors, "memory.count", memory.count);
    }
    uniform_draw(&mut errors, "fidelity_draw", fidelity_draw);
    uniform_draw(&mut errors, "recruitment_draw", recruitment_draw);
    let mut ids = BTreeSet::new();
    for (index, waypoint) in waypoints.iter().enumerate() {
        if !ids.insert(waypoint.id) {
            errors.push(FieldError::new(
                format!("waypoints[{index}].id"),
                format!("duplicate waypoint id {}", waypoint.id),
            ));
        }
        closed_interval(
            &mut errors,
            &format!("waypoints[{index}].strength"),
            waypoint.strength,
            0.0,
            1.0,
        );
    }
    checked(errors)?;
    if let Some(memory) = memory {
        if poisson_cdf(memory.count, parameters.lambda_fidelity)? > fidelity_draw {
            return Ok(Departure::SiteFidelity { site: memory.site });
        }
    }
    let active: Vec<_> = waypoints
        .iter()
        .filter(|waypoint| match threshold {
            WaypointThreshold::PaperBelow => waypoint.strength >= WAYPOINT_THRESHOLD,
            WaypointThreshold::LaterArgosStrict => waypoint.strength > WAYPOINT_THRESHOLD,
        })
        .collect();
    if active.is_empty() {
        return Ok(Departure::Uninformed);
    }
    let total: f64 = active
        .iter()
        .map(|waypoint| match selection {
            WaypointSelection::UniformComparison => 1.0,
            WaypointSelection::LaterArgosStrengthWeighted => waypoint.strength,
        })
        .sum();
    let mut cumulative = 0.0;
    for waypoint in active {
        cumulative += match selection {
            WaypointSelection::UniformComparison => 1.0,
            WaypointSelection::LaterArgosStrengthWeighted => waypoint.strength,
        };
        // Identical ordered sums give a final upper boundary of exactly one.
        // Equality advances to the next half-open interval.
        if recruitment_draw < cumulative / total {
            return Ok(Departure::Recruitment {
                waypoint: waypoint.id,
                site: waypoint.site,
            });
        }
    }
    unreachable!("validated draw below one falls in an active interval")
}
