use super::math::{chance, derive_seed, probability, uniform53, NumericIssue, Probability};

#[test]
fn valid_tiny_hazard_retains_probability() {
    let p = probability(1e-12, 1.0).unwrap();
    assert!((p.ideal - 9.999_999_999_995e-13).abs() < 1e-27);
}

#[test]
fn stable_helper_retains_hazard_below_sampler_resolution() {
    assert_eq!(-libm::expm1(-1e-18), 1e-18);
}

#[test]
fn below_sampler_resolution_is_contextual_error() {
    let error = probability(1e-18, 1.0).unwrap_err();
    assert_eq!(error.field, "probability");
    assert!(error.detail.contains("53-bit"));
}

#[test]
fn maximum_uniform_is_less_than_one() {
    assert_eq!(uniform53(u64::MAX), 1.0 - 2.0_f64.powi(-53));
}

#[test]
fn uniform_uses_high_53_bits() {
    for (word, expected) in [
        (0, 0.0),
        (2047, 0.0),
        (2048, 1.110_223_024_625_156_5e-16),
        (1 << 63, 0.5),
    ] {
        assert_eq!(uniform53(word), expected);
    }
}

#[test]
fn probability_rejects_invalid_operands() {
    for hazard in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY, -1.0] {
        assert_eq!(probability(hazard, 1.0).unwrap_err().field, "hazard");
    }
    for dt in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY, -1.0, 0.0, -0.0] {
        assert_eq!(probability(0.0, dt).unwrap_err().field, "dt");
    }
}

#[test]
fn dose_overflow_is_contextual_error() {
    let error = probability(f64::MAX, 2.0).unwrap_err();
    assert_eq!(error.field, "dose");
    assert!(error.detail.contains("hazard") && error.detail.contains("dt"));
}

#[test]
fn positive_product_underflow_is_rejected() {
    assert_eq!(
        probability(f64::from_bits(1), 0.5).unwrap_err().field,
        "dose"
    );
}

#[test]
fn zero_hazard_has_positive_zero_probability() {
    for hazard in [0.0, -0.0] {
        let p = probability(hazard, 1.0).unwrap();
        for value in [p.hazard, p.dose, p.ideal, p.realized] {
            assert_eq!(value.to_bits(), 0);
        }
        assert!(!p.saturated);
        assert!(!chance(0, &p));
        assert!(!chance(u64::MAX, &p));
    }
}

#[test]
fn high_finite_dose_reports_saturation() {
    let p = probability(400.0, 1.0).unwrap();
    assert_eq!(p.dose, 400.0);
    assert_eq!(p.ideal, 1.0);
    assert_eq!(p.realized, 1.0);
    assert!(p.saturated);
    assert!(chance(u64::MAX, &p));
}

#[test]
fn realized_probability_rounds_up_within_one_resolution_unit() {
    for hazard in [1e-12, 0.1, 1.0, 10.0] {
        let p = probability(hazard, 1.0).unwrap();
        assert!(p.realized >= p.ideal);
        assert!(p.realized - p.ideal < 1.110_223_024_625_156_5e-16);
        assert_eq!((p.realized * 9_007_199_254_740_992.0).fract(), 0.0);
        assert!(!p.saturated);
    }
}

#[test]
fn exact_resolution_probability_is_accepted() {
    let p = probability(1.110_223_024_625_156_5e-16, 1.0).unwrap();
    assert_eq!(p.ideal, 1.110_223_024_625_156_5e-16);
    assert_eq!(p.realized, p.ideal);
}

#[test]
fn chance_uses_strict_ideal_probability_threshold() {
    let p = Probability {
        hazard: 1.0,
        dose: 1.0,
        ideal: 0.5,
        realized: 0.5,
        saturated: false,
    };
    assert!(chance(0, &p));
    assert!(chance((1 << 63) - 2048, &p));
    assert!(!chance(1 << 63, &p));
    assert!(!chance(u64::MAX, &p));
    // An ideal p between consecutive uniforms rounds up to q. Comparing to q
    // inclusively would accept the next grid point incorrectly.
    let p = Probability {
        ideal: 1.5 / 9_007_199_254_740_992.0,
        realized: 2.0 / 9_007_199_254_740_992.0,
        ..p
    };
    assert!(chance(2048, &p));
    assert!(!chance(4096, &p));
}

#[test]
fn independently_derived_stream_seed_vectors_match_protocol() {
    assert_eq!(
        derive_seed(7, b"war1-contact-v1"),
        16_934_061_643_845_198_696
    );
    assert_eq!(
        derive_seed(7, b"war1-casualty-v1"),
        14_728_233_557_300_059_986
    );
}

#[test]
fn nonfinite_operands_produce_serializable_error_receipts() {
    let error = probability(f64::NAN, 1.0).unwrap_err();
    let encoded = serde_json::to_string(&error).unwrap();
    assert_eq!(
        serde_json::from_str::<NumericIssue>(&encoded).unwrap(),
        error
    );
    assert!(error.detail.contains("NaN"));
}
