//! Conditional commitments and source-direction probability, independent of GeoSim.
use super::*;
pub(crate) fn probability(
    n: f64,
    d: f64,
    t: f64,
    c: u32,
    direction: ProbabilityDirection,
    zero: ZeroRatio,
) -> Result<f64, String> {
    if !n.is_finite() || !d.is_finite() || n < 0. || d < 0. {
        return Err("invalid resources in probability".into());
    }
    if d == 0. {
        if zero == ZeroRatio::RejectZeroDenominator {
            return Err("zero probability denominator".into());
        }
        if n == 0. {
            return Ok(0.5);
        }
        return Ok(if direction == ProbabilityDirection::PrintedDecreasing {
            0.
        } else {
            1.
        });
    }
    if n == 0. {
        return Ok(if direction == ProbabilityDirection::PrintedDecreasing {
            1.
        } else {
            0.
        });
    }
    if !(n / d).is_finite() {
        return Err("nonfinite probability balance with positive denominator".into());
    }
    let z = (libm::log(n) - libm::log(d) - libm::log(t)) * f64::from(c);
    let z = if direction == ProbabilityDirection::PrintedDecreasing {
        z
    } else {
        -z
    };
    let p = if z >= 0. {
        let e = libm::exp(-z);
        e / (1. + e)
    } else {
        1. / (1. + libm::exp(z))
    };
    if p.is_finite() {
        Ok(p)
    } else {
        Err("nonfinite probability".into())
    }
}
pub(crate) fn commitment(
    r: f64,
    mobile: f64,
    n: usize,
    opposing: f64,
    enemy: f64,
    active: bool,
    inactive_term: f64,
) -> Result<f64, String> {
    if [r, opposing, enemy, inactive_term]
        .iter()
        .any(|v| !v.is_finite() || *v < 0.)
    {
        return Err("invalid allocation input".into());
    }
    if n == 0 {
        return Ok(0.);
    }
    let fixed = (1. - mobile) * r / n as f64;
    let fraction = if enemy == 0. {
        1.
    } else if active {
        opposing / enemy
    } else {
        let d = enemy + inactive_term;
        if !d.is_finite() || d <= 0. {
            return Err("invalid inactive allocation denominator".into());
        }
        opposing / d
    };
    let value = fixed + mobile * r * fraction;
    if value.is_finite() {
        Ok(value)
    } else {
        Err("nonfinite commitment".into())
    }
}
pub(crate) fn threat_ratio(n: f64, d: f64, zero: ZeroRatio) -> Result<f64, String> {
    if !n.is_finite() || !d.is_finite() || n < 0. || d < 0. {
        return Err("invalid threat resources".into());
    }
    if d == 0. {
        if zero == ZeroRatio::RejectZeroDenominator {
            return Err("zero threat denominator".into());
        }
        return Ok(if n == 0. { 1. } else { f64::INFINITY });
    }
    let ratio = n / d;
    if ratio.is_finite() {
        Ok(ratio)
    } else {
        Err("nonfinite threat ratio with positive denominator".into())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn printed_probability_declines_with_superiority() {
        assert!(
            probability(
                4.4,
                1.0,
                2.2,
                30,
                ProbabilityDirection::PrintedDecreasing,
                ZeroRatio::EqualZeroNeutral
            )
            .unwrap()
                < 0.5
        );
    }
    #[test]
    fn prose_probability_increases_with_superiority() {
        assert!(
            probability(
                4.4,
                1.0,
                2.2,
                30,
                ProbabilityDirection::ProseIncreasing,
                ZeroRatio::EqualZeroNeutral
            )
            .unwrap()
                > 0.5
        );
    }
    #[test]
    fn zero_limits_are_direction_sensitive() {
        assert_eq!(
            probability(
                0.,
                1.,
                2.2,
                30,
                ProbabilityDirection::PrintedDecreasing,
                ZeroRatio::EqualZeroNeutral
            )
            .unwrap(),
            1.0
        );
    }
    #[test]
    fn strict_zero_policy_rejects_denominator() {
        assert!(probability(
            1.,
            0.,
            2.2,
            30,
            ProbabilityDirection::PrintedDecreasing,
            ZeroRatio::RejectZeroDenominator
        )
        .is_err());
    }
    #[test]
    fn fixed_allocation_uses_distinct_eligible_fronts() {
        assert_eq!(
            commitment(100., 0.5, 5, 15., 40., true, 15.).unwrap(),
            28.75
        );
    }
    #[test]
    fn inactive_front_uses_own_opponent() {
        assert!(
            (commitment(100., 0.5, 5, 20., 40., false, 20.).unwrap() - 26.666666666666667).abs()
                < 1e-12
        );
    }
    #[test]
    fn zero_fronts_never_divide() {
        assert_eq!(commitment(100., 0.5, 0, 0., 0., false, 0.).unwrap(), 0.);
    }
}

#[cfg(test)]
mod overflow_tests {
    use super::*;
    #[test]
    fn finite_denominator_overflow_is_invalid() {
        assert!(threat_ratio(10., 1e-320, ZeroRatio::EqualZeroNeutral).is_err());
    }
}
