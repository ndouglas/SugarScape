use std::f64::consts::PI;

use crate::config::FieldError;

use super::{MAX_INFORMATION_RATE, MAX_RESOURCE_COUNT};

/// Seven explicit CPFA parameters from Hecker and Moses (2015).
///
/// No evolved defaults are supplied. Probabilities are in `[0,1]`, angular
/// variation in `[0,4*pi]` radians, and decay rates are nonnegative and finite.
/// Fidelity/publication rates additionally use the reference-utility bound
/// `[0,256]`, not the paper's evolutionary initialization range `[0,20]`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CpfaParameters {
    pub p_search: f64,
    pub p_return: f64,
    pub omega: f64,
    pub lambda_informed: f64,
    pub lambda_fidelity: f64,
    pub lambda_publish: f64,
    pub lambda_waypoint: f64,
}

impl CpfaParameters {
    /// Validate original inputs, accumulating errors in declaration order.
    pub fn validate(&self) -> Result<(), Vec<FieldError>> {
        let mut errors = Vec::new();
        closed_interval(&mut errors, "p_search", self.p_search, 0.0, 1.0);
        closed_interval(&mut errors, "p_return", self.p_return, 0.0, 1.0);
        closed_interval(&mut errors, "omega", self.omega, 0.0, 4.0 * PI);
        nonnegative(&mut errors, "lambda_informed", self.lambda_informed);
        closed_interval(
            &mut errors,
            "lambda_fidelity",
            self.lambda_fidelity,
            0.0,
            MAX_INFORMATION_RATE,
        );
        closed_interval(
            &mut errors,
            "lambda_publish",
            self.lambda_publish,
            0.0,
            MAX_INFORMATION_RATE,
        );
        nonnegative(&mut errors, "lambda_waypoint", self.lambda_waypoint);
        checked(errors)
    }
}

pub(super) fn closed_interval(
    errors: &mut Vec<FieldError>,
    field: &str,
    value: f64,
    min: f64,
    max: f64,
) {
    if !value.is_finite() || !(min..=max).contains(&value) {
        errors.push(FieldError::new(
            field,
            format!("must be finite and in [{min},{max}]"),
        ));
    }
}

pub(super) fn nonnegative(errors: &mut Vec<FieldError>, field: &str, value: f64) {
    if !value.is_finite() || value < 0.0 {
        errors.push(FieldError::new(field, "must be finite and nonnegative"));
    }
}

pub(super) fn resource_count(errors: &mut Vec<FieldError>, field: &str, count: u32) {
    if count > MAX_RESOURCE_COUNT {
        errors.push(FieldError::new(
            field,
            format!("must be at most {MAX_RESOURCE_COUNT}"),
        ));
    }
}

pub(super) fn checked(errors: Vec<FieldError>) -> Result<(), Vec<FieldError>> {
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

pub(super) fn uniform_draw(errors: &mut Vec<FieldError>, field: &str, value: f64) {
    if !value.is_finite() || !(0.0..1.0).contains(&value) {
        errors.push(FieldError::new(field, "must be finite and in [0,1)"));
    }
}
