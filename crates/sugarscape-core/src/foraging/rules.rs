use std::f64::consts::PI;

use crate::config::FieldError;

use super::config::{checked, closed_interval, nonnegative, resource_count};
use super::MAX_INFORMATION_RATE;

/// Hecker and Moses (2015) uninformed angular standard deviation, in radians.
/// Requires finite `omega` in `[0,4*pi]`; no heading is sampled.
pub fn uninformed_variation(omega: f64) -> Result<f64, Vec<FieldError>> {
    let mut errors = Vec::new();
    closed_interval(&mut errors, "omega", omega, 0.0, 4.0 * PI);
    checked(errors)?;
    Ok(omega)
}

/// Hecker and Moses (2015) informed angular standard deviation:
/// `omega + (4*pi - omega)*exp(-rate*age)`.
///
/// `omega` is finite in `[0,4*pi]`; rate and age are finite and nonnegative.
/// Uses standard-library `exp`, without a cross-platform bit identity claim.
pub fn informed_variation(omega: f64, rate: f64, age: f64) -> Result<f64, Vec<FieldError>> {
    let mut errors = Vec::new();
    closed_interval(&mut errors, "omega", omega, 0.0, 4.0 * PI);
    nonnegative(&mut errors, "rate", rate);
    nonnegative(&mut errors, "age", age);
    checked(errors)?;
    Ok(omega + (4.0 * PI - omega) * exponential_decay(rate, age))
}

/// Lower-tail Poisson CDF, including `count`, as displayed in Hecker and
/// Moses (2015). The source prose describes a conflicting tail; this API
/// follows the displayed equation.
///
/// Counts are integers in `[0,256]` and rates finite in `[0,256]`. These
/// supplied engineering bounds keep `exp(-rate)` representable and the
/// recurrence bounded. Endpoint excursions of at most `1e-12` are corrected;
/// larger or nonfinite results produce a contextual error.
pub fn poisson_cdf(count: u32, rate: f64) -> Result<f64, Vec<FieldError>> {
    let mut errors = Vec::new();
    resource_count(&mut errors, "count", count);
    closed_interval(&mut errors, "rate", rate, 0.0, MAX_INFORMATION_RATE);
    checked(errors)?;
    let mut term = (-rate).exp();
    let mut total = term;
    for k in 1..=count {
        term *= rate / f64::from(k);
        total += term;
    }
    if !total.is_finite() || !(-1e-12..=1.0 + 1e-12).contains(&total) {
        return Err(vec![FieldError::new(
            "rate",
            format!("invalid probability {total} for count={count}, rate={rate}"),
        )]);
    }
    Ok(total.clamp(0.0, 1.0))
}

/// Hecker and Moses (2015) waypoint strength `exp(-rate*age)`.
/// Rate and age must be finite and nonnegative; zero factors yield one,
/// while positive product overflow yields zero (expired information).
pub fn waypoint_strength(rate: f64, age: f64) -> Result<f64, Vec<FieldError>> {
    let mut errors = Vec::new();
    nonnegative(&mut errors, "rate", rate);
    nonnegative(&mut errors, "age", age);
    checked(errors)?;
    Ok(exponential_decay(rate, age))
}

// Inputs have already been checked as finite and nonnegative.
fn exponential_decay(rate: f64, age: f64) -> f64 {
    if rate == 0.0 || age == 0.0 {
        return 1.0;
    }
    let product = rate * age;
    if product.is_infinite() {
        0.0
    } else {
        (-product).exp()
    }
}
