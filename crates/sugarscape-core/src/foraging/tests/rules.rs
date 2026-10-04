use crate::foraging::{
    informed_variation, poisson_cdf, uninformed_variation, waypoint_strength, CpfaParameters,
};
use std::f64::consts::PI;

fn parameters() -> CpfaParameters {
    CpfaParameters {
        p_search: 0.0,
        p_return: 1.0,
        omega: 4.0 * PI,
        lambda_informed: 0.0,
        lambda_fidelity: 256.0,
        lambda_publish: 256.0,
        lambda_waypoint: f64::MAX,
    }
}

#[test]
fn parameters_accept_closed_endpoints_and_uncapped_decay() {
    parameters().validate().unwrap();
    CpfaParameters {
        p_search: 1.0,
        p_return: 0.0,
        omega: 0.0,
        lambda_informed: f64::MAX,
        lambda_fidelity: 0.0,
        lambda_publish: 0.0,
        lambda_waypoint: 0.0,
    }
    .validate()
    .unwrap();
}

#[test]
fn parameters_aggregate_all_errors_in_declaration_order() {
    let p = CpfaParameters {
        p_search: -1.0,
        p_return: 2.0,
        omega: 13.0,
        lambda_informed: -1.0,
        lambda_fidelity: 257.0,
        lambda_publish: 257.0,
        lambda_waypoint: -1.0,
    };
    let errors = p.validate().unwrap_err();
    assert_eq!(
        errors.iter().map(|e| e.field.as_str()).collect::<Vec<_>>(),
        [
            "p_search",
            "p_return",
            "omega",
            "lambda_informed",
            "lambda_fidelity",
            "lambda_publish",
            "lambda_waypoint"
        ]
    );
    assert!(errors.iter().all(|e| !e.message.is_empty()));
}

#[test]
fn parameters_reject_nonfinite_values_in_every_field() {
    for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        let p = CpfaParameters {
            p_search: bad,
            p_return: bad,
            omega: bad,
            lambda_informed: bad,
            lambda_fidelity: bad,
            lambda_publish: bad,
            lambda_waypoint: bad,
        };
        assert_eq!(p.validate().unwrap_err().len(), 7);
    }
}

#[test]
fn rules_reject_invalid_original_arguments_with_context() {
    for bad in [-1.0, f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert_eq!(uninformed_variation(bad).unwrap_err()[0].field, "omega");
        assert_eq!(
            informed_variation(0.0, bad, 0.0).unwrap_err()[0].field,
            "rate"
        );
        assert_eq!(
            informed_variation(0.0, 0.0, bad).unwrap_err()[0].field,
            "age"
        );
        assert_eq!(waypoint_strength(bad, 0.0).unwrap_err()[0].field, "rate");
        assert_eq!(waypoint_strength(0.0, bad).unwrap_err()[0].field, "age");
        assert_eq!(poisson_cdf(0, bad).unwrap_err()[0].field, "rate");
    }
    assert!(uninformed_variation(13.0).is_err());
    assert!(informed_variation(13.0, 0.0, 0.0).is_err());
    assert_eq!(informed_variation(-1.0, -1.0, -1.0).unwrap_err().len(), 3);
    assert_eq!(waypoint_strength(-1.0, -1.0).unwrap_err().len(), 2);
    assert_eq!(
        poisson_cdf(257, 257.0)
            .unwrap_err()
            .iter()
            .map(|e| e.field.as_str())
            .collect::<Vec<_>>(),
        ["count", "rate"]
    );
    assert!(poisson_cdf(u32::MAX, 1.0).is_err());
}

#[test]
fn poisson_reference_includes_the_count_endpoint() {
    let actual = poisson_cdf(1, 1.0).unwrap();
    assert!((actual - 0.7357588823428846).abs() <= 1e-12);
}

#[test]
fn poisson_matches_independent_decimal_fixtures() {
    for (count, rate, expected) in [
        (0, 1.0, 0.3678794411714423),
        (4, 2.0, 0.9473469826562888),
        (128, 128.0, 0.5234844486527008),
        (256, 256.0, 0.5166143010472157),
    ] {
        assert!((poisson_cdf(count, rate).unwrap() - expected).abs() <= 1e-12);
    }
}

#[test]
fn poisson_zero_rate_is_exactly_one_for_every_count() {
    for count in 0..=256 {
        assert_eq!(poisson_cdf(count, 0.0).unwrap(), 1.0);
    }
}

#[test]
fn poisson_is_monotone_in_count_and_bounded() {
    for rate in [0.0, 1.0, 20.0, 128.0, 256.0] {
        let mut previous = 0.0;
        for count in 0..=256 {
            let actual = poisson_cdf(count, rate).unwrap();
            assert!((0.0..=1.0).contains(&actual));
            assert!(actual + 1e-14 >= previous);
            previous = actual;
        }
    }
}

#[test]
fn poisson_is_inverse_monotone_in_rate() {
    for count in [0, 1, 64, 256] {
        let mut previous = 1.0;
        for rate in 0..=256 {
            let actual = poisson_cdf(count, f64::from(rate)).unwrap();
            assert!(actual <= previous + 1e-14);
            previous = actual;
        }
    }
}

#[test]
fn uninformed_variation_returns_checked_omega() {
    for omega in [0.0, 1.0, 4.0 * PI] {
        assert_eq!(uninformed_variation(omega).unwrap(), omega);
    }
}

#[test]
fn informed_variation_preserves_boundary_cases() {
    assert_eq!(informed_variation(1.0, f64::MAX, 0.0).unwrap(), 4.0 * PI);
    assert_eq!(informed_variation(1.0, 0.0, f64::MAX).unwrap(), 4.0 * PI);
    assert_eq!(
        informed_variation(4.0 * PI, f64::MAX, f64::MAX).unwrap(),
        4.0 * PI
    );
    assert_eq!(informed_variation(1.0, f64::MAX, f64::MAX).unwrap(), 1.0);
    assert!(
        (informed_variation(0.0, 1.0, std::f64::consts::LN_2).unwrap() - 2.0 * PI).abs() <= 1e-12
    );
}

#[test]
fn informed_variation_decreases_toward_omega() {
    let mut previous = 4.0 * PI;
    for age in [0.0, 0.1, 1.0, 2.0, 20.0, 1000.0] {
        let actual = informed_variation(1.0, 1.0, age).unwrap();
        assert!((1.0..=4.0 * PI).contains(&actual));
        assert!(actual <= previous + 1e-14);
        previous = actual;
    }
    assert_eq!(previous, 1.0);
}

#[test]
fn waypoint_decay_has_exact_zero_factor_and_expiration_cases() {
    assert_eq!(waypoint_strength(0.0, f64::MAX).unwrap(), 1.0);
    assert_eq!(waypoint_strength(f64::MAX, 0.0).unwrap(), 1.0);
    assert!((waypoint_strength(1.0, std::f64::consts::LN_2).unwrap() - 0.5).abs() <= 1e-12);
    assert_eq!(waypoint_strength(1.0, 1000.0).unwrap(), 0.0);
}

#[test]
fn finite_decay_product_overflow_expires_instead_of_nan() {
    assert_eq!(waypoint_strength(f64::MAX, f64::MAX).unwrap(), 0.0);
}
