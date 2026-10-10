//! Analytic continuous-force references, independent of the finite kernel and RNG.
//!
//! An unavailable floating-point evaluation is an error, never an extinction event.
use super::config::{EngagementConfig, Geometry, Side};
use serde::Serialize;
use std::cmp::Ordering;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ReferenceRegime {
    InitialExtinction,
    FiniteExtinction,
    Asymptotic,
    RateZero,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ReferencePoint {
    pub requested_time: f64,
    pub evaluated_time: f64,
    pub forces: [f64; 2],
    pub regime: ReferenceRegime,
    pub extinction_time: Option<f64>,
    pub at_boundary: bool,
    pub survivor: Option<Side>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ReferenceFailure {
    pub field: String,
    pub detail: String,
}

fn failure(field: &str, detail: &str) -> ReferenceFailure {
    ReferenceFailure {
        field: field.into(),
        detail: detail.into(),
    }
}

fn positive(value: f64, field: &str) -> Result<f64, ReferenceFailure> {
    if value.is_finite() && value > 0.0 {
        Ok(value)
    } else {
        Err(failure(
            field,
            "required positive value is nonfinite, underflowed, or nonpositive",
        ))
    }
}

fn product(a: f64, b: f64, field: &str) -> Result<f64, ReferenceFailure> {
    if a == 0.0 || b == 0.0 {
        Ok(0.0)
    } else {
        positive(a * b, field)
    }
}

fn difference(a: f64, b: f64, field: &str) -> Result<f64, ReferenceFailure> {
    let gap = positive(a - b, field)?;
    if gap <= 8.0 * f64::EPSILON * (a + b) {
        return Err(failure(
            field,
            "positive difference is unresolved at binary64 precision",
        ));
    }
    Ok(gap)
}

fn decay(rate: f64, time: f64) -> Result<f64, ReferenceFailure> {
    let u = product(rate, time, "rate * time")?;
    positive(libm::exp(-u), "exponential decay")
}

/// Evaluate within the configured horizon, stopping at the first finite extinction.
/// A failure means the reference is unavailable; it says nothing about kernel validity.
pub fn reference_at(
    config: &EngagementConfig,
    time: f64,
) -> Result<ReferencePoint, ReferenceFailure> {
    config
        .validate()
        .map_err(|errors| failure("config", &format!("invalid engagement config: {errors:?}")))?;
    if !time.is_finite() || time < 0.0 || time > config.max_steps as f64 * config.dt {
        return Err(failure(
            "time",
            "must be finite, nonnegative, and within max_steps * dt",
        ));
    }
    let mut point = ReferencePoint {
        requested_time: time,
        evaluated_time: time,
        forces: [f64::from(config.blue), f64::from(config.red)],
        regime: ReferenceRegime::Asymptotic,
        extinction_time: None,
        at_boundary: false,
        survivor: None,
    };
    if config.blue == 0 || config.red == 0 {
        point.regime = ReferenceRegime::InitialExtinction;
        point.evaluated_time = 0.0;
        point.extinction_time = Some(0.0);
        point.at_boundary = true;
        point.survivor = match (config.blue, config.red) {
            (0, 0) => None,
            (0, _) => Some(Side::Red),
            _ => Some(Side::Blue),
        };
    } else if config.blue_rate == 0.0 && config.red_rate == 0.0 {
        point.regime = ReferenceRegime::RateZero;
    } else {
        match config.geometry {
            Geometry::AimedFire => aimed(config, &mut point)?,
            Geometry::DuelContact => matched(config, &mut point)?,
        }
    }
    Ok(point)
}

// Binary64 rate = integer coefficient * 2^exponent. Integer populations keep
// the largest coefficient below 2^77, even for the squared-force invariant.
fn weighted_rate(rate: f64, count: u32, squared: bool) -> (u128, i32) {
    let bits = rate.to_bits();
    let fraction = bits & ((1_u64 << 52) - 1);
    let exponent = ((bits >> 52) & 0x7ff) as i32;
    let (mantissa, exponent) = if exponent == 0 {
        (fraction, -1074)
    } else {
        (fraction | (1_u64 << 52), exponent - 1075)
    };
    let count = u128::from(count);
    (
        u128::from(mantissa) * count * if squared { count } else { 1 },
        exponent,
    )
}

/// Exact sign for the original dyadic rates and integer populations.
pub(super) fn invariant_sign(config: &EngagementConfig) -> Result<Ordering, ReferenceFailure> {
    config
        .validate()
        .map_err(|errors| failure("config", &format!("invalid engagement config: {errors:?}")))?;
    let squared = config.geometry == Geometry::AimedFire;
    let (a, ae) = weighted_rate(config.blue_rate, config.blue, squared);
    let (b, be) = weighted_rate(config.red_rate, config.red, squared);
    if a == 0 || b == 0 {
        return Ok(a.cmp(&b));
    }
    let abits = 128 - a.leading_zeros();
    let bbits = 128 - b.leading_zeros();
    let leading = (ae + abits as i32).cmp(&(be + bbits as i32));
    if leading != Ordering::Equal {
        return Ok(leading);
    }
    // Equal leading exponents imply alignment needs at most the coefficient
    // bit-length difference; no huge shift from the raw exponent gap occurs.
    Ok(if abits < bbits {
        (a << (bbits - abits)).cmp(&b)
    } else {
        a.cmp(&(b << (abits - bbits)))
    })
}

fn boundary(point: &mut ReferencePoint, time: f64, survivor: usize, force: f64) {
    point.regime = ReferenceRegime::FiniteExtinction;
    point.extinction_time = Some(time);
    if point.requested_time >= time {
        point.evaluated_time = time;
        point.forces = [0.0; 2];
        point.forces[survivor] = force;
        point.at_boundary = true;
        point.survivor = Some(if survivor == 0 { Side::Blue } else { Side::Red });
    }
}

fn aimed(config: &EngagementConfig, point: &mut ReferencePoint) -> Result<(), ReferenceFailure> {
    let b = config.blue_rate;
    let r = config.red_rate;
    if b == 0.0 || r == 0.0 {
        let (winner, rate) = if r == 0.0 { (0, b) } else { (1, r) };
        let loser = 1 - winner;
        let initial = point.forces;
        let end = positive((initial[loser] / initial[winner]) / rate, "extinction time")?;
        boundary(point, end, winner, initial[winner]);
        if !point.at_boundary && point.requested_time > 0.0 {
            let loss = product(
                product(rate, point.requested_time, "rate * time")?,
                initial[winner],
                "linear loss",
            )?;
            point.forces[loser] = difference(initial[loser], loss, "linear remaining force")?;
        }
        return Ok(());
    }
    let sign = invariant_sign(config)?;
    let t = point.requested_time;
    if sign == Ordering::Equal {
        let z = product(
            libm::sqrt(product(b, t, "blue rate * time")?),
            libm::sqrt(product(r, t, "red rate * time")?),
            "aimed exponent",
        )?;
        let factor = positive(libm::exp(-z), "exponential decay")?;
        for force in &mut point.forces {
            *force = product(*force, factor, "balanced force")?;
        }
        return Ok(());
    }
    let m = b.max(r);
    let weights = [
        libm::sqrt(positive(b / m, "blue normalized rate")?),
        libm::sqrt(positive(r / m, "red normalized rate")?),
    ];
    let x = product(weights[0], point.forces[0], "weighted blue force")?;
    let y = product(weights[1], point.forces[1], "weighted red force")?;
    let sum = x + y;
    let gap = x - y;
    if gap == 0.0 || gap.total_cmp(&0.0) != sign || gap.abs() <= 8.0 * f64::EPSILON * sum {
        return Err(failure(
            "aimed invariant",
            "exact nonzero imbalance has unresolved normalized magnitude",
        ));
    }
    let z_end = positive(
        libm::log(positive(sum / gap.abs(), "aimed boundary ratio")?) / 2.0,
        "aimed boundary exponent",
    )?;
    let frequency = product(libm::sqrt(b), libm::sqrt(r), "aimed frequency")?;
    let end = positive(z_end / frequency, "extinction time")?;
    let winner = if sign == Ordering::Greater { 0 } else { 1 };
    let remaining = positive(
        libm::sqrt(gap.abs() * sum) / weights[winner],
        "boundary survivor",
    )?;
    boundary(point, end, winner, remaining);
    if point.at_boundary || t == 0.0 {
        return Ok(());
    }
    let z = product(
        libm::sqrt(product(b, t, "blue rate * time")?),
        libm::sqrt(product(r, t, "red rate * time")?),
        "aimed exponent",
    )?;
    let falling = product(
        sum,
        positive(libm::exp(-z), "falling exponential")?,
        "falling term",
    )?;
    let rising = product(
        gap.abs(),
        positive(libm::exp(z), "rising exponential")?,
        "rising term",
    )?;
    point.forces[winner] = positive(
        (falling + rising) / (2.0 * weights[winner]),
        "winning force",
    )?;
    point.forces[1 - winner] = positive(
        difference(falling, rising, "losing weighted force")? / (2.0 * weights[1 - winner]),
        "losing force",
    )?;
    Ok(())
}

// Solve one interval with a fixed smaller side. alpha removes the smaller
// force; beta removes the larger. phi avoids subtracting nearly equal exp values.
fn matched_segment(
    small: f64,
    large: f64,
    alpha: f64,
    beta: f64,
    time: f64,
) -> Result<[f64; 2], ReferenceFailure> {
    if time == 0.0 {
        return Ok([small, large]);
    }
    let u = product(alpha, time, "matched exponent")?;
    let factor = positive(libm::exp(-u), "exponential decay")?;
    let phi = if u == 0.0 {
        1.0
    } else {
        positive(-libm::expm1(-u) / u, "matched phi")?
    };
    let loss = product(
        product(
            product(beta, time, "rate * time")?,
            small,
            "matched loss budget",
        )?,
        phi,
        "matched loss",
    )?;
    Ok([
        product(small, factor, "matched smaller force")?,
        difference(large, loss, "matched larger force")?,
    ])
}

fn matched(config: &EngagementConfig, point: &mut ReferencePoint) -> Result<(), ReferenceFailure> {
    let t = point.requested_time;
    if t == 0.0 {
        return Ok(());
    }
    let b = config.blue_rate;
    let r = config.red_rate;
    // At equality the larger opposing rate determines the future smaller side.
    let small_index = if config.blue < config.red || (config.blue == config.red && r >= b) {
        0
    } else {
        1
    };
    let large_index = 1 - small_index;
    let (alpha, beta) = if small_index == 0 { (r, b) } else { (b, r) };
    let small = point.forces[small_index];
    let large = point.forces[large_index];
    let mut sign = invariant_sign(config)?;
    if small_index == 1 {
        sign = sign.reverse();
    }
    let forces = if sign == Ordering::Equal {
        let factor = decay(alpha, t)?;
        [
            product(small, factor, "balanced smaller force")?,
            product(large, factor, "balanced larger force")?,
        ]
    } else if sign == Ordering::Greater {
        // The initially larger force becomes smaller exactly once. When alpha
        // is zero the first interval is linear and the second exponential.
        let crossing = if alpha == 0.0 {
            positive(((large - small) / small) / beta, "crossing time")?
        } else {
            let fraction = product(
                positive(
                    alpha / positive(beta - alpha, "crossing rate difference")?,
                    "crossing rate ratio",
                )?,
                positive((large - small) / small, "crossing force ratio")?,
                "crossing fraction",
            )?;
            if fraction >= 1.0 {
                return Err(failure(
                    "crossing fraction",
                    "must be strictly between zero and one",
                ));
            }
            positive(-libm::log1p(-fraction) / alpha, "crossing time")?
        };
        if t < crossing {
            matched_segment(small, large, alpha, beta, t)?
        } else {
            let equal = product(small, decay(alpha, crossing)?, "crossing force")?;
            let after = matched_segment(equal, equal, beta, alpha, t - crossing)?;
            [after[1], after[0]]
        }
    } else {
        matched_segment(small, large, alpha, beta, t)?
    };
    point.forces[small_index] = forces[0];
    point.forces[large_index] = forces[1];
    Ok(())
}
