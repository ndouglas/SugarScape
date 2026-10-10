use super::config::{EngagementConfig, Geometry, Side};
use super::reference::{invariant_sign, reference_at, ReferenceRegime};
use std::cmp::Ordering;

fn config(geometry: Geometry, blue: u32, red: u32, b: f64, r: f64) -> EngagementConfig {
    EngagementConfig {
        blue,
        red,
        blue_rate: b,
        red_rate: r,
        dt: 0.01,
        max_steps: 1000,
        geometry,
    }
}
fn close(actual: f64, expected: f64) {
    assert!((actual - expected).abs() < 2e-12, "{actual} != {expected}");
}

// Each fixture catches an incorrect analytic branch, coefficient, or crossing restart.
#[test]
fn closed_form_fixtures() {
    use Geometry::{AimedFire as A, DuelContact as D};
    use ReferenceRegime::{Asymptotic, FiniteExtinction};
    let l2 = libm::log(2.0);
    for (g, blue, red, b, r, t, want, regime) in [
        (A, 1, 1, 1., 1., l2, [0.5, 0.5], Asymptotic),
        (A, 1, 2, 4., 1., l2 / 2., [0.5, 1.], Asymptotic),
        (
            A,
            3,
            4,
            1.,
            1.,
            libm::log(7.) / 2.,
            [0., libm::sqrt(7.)],
            FiniteExtinction,
        ),
        (
            A,
            1,
            1,
            4.,
            1.,
            libm::log(3.) / 4.,
            [libm::sqrt(3.) / 2., 0.],
            FiniteExtinction,
        ),
        (A, 1, 3, 2., 0., 1.5, [1., 0.], FiniteExtinction),
        (D, 1, 1, 2., 1., l2 / 2., [0.75, 0.5], Asymptotic),
        (D, 1, 3, 2., 1., l2, [0.5, 2.], Asymptotic),
        (D, 1, 2, 2., 1., l2, [0.5, 1.], Asymptotic),
        (D, 1, 3, 2., 0., 0.5, [1., 2.], Asymptotic),
        (D, 1, 2, 3., 1., l2 + l2 / 3., [5. / 12., 0.25], Asymptotic),
        (D, 1, 3, 2., 0., 1. + l2 / 2., [1., 0.5], Asymptotic),
    ] {
        let c = config(g, blue, red, b, r);
        let p = reference_at(&c, t).unwrap();
        close(p.forces[0], want[0]);
        close(p.forces[1], want[1]);
        assert_eq!(p.regime, regime);
    }
}

#[test]
fn zero_time_preserves_counts_for_both_laws_and_rate_orderings() {
    for g in [Geometry::AimedFire, Geometry::DuelContact] {
        for (blue, red, b, r) in [
            (1, 1, 2., 1.),
            (1, 1, 1., 2.),
            (3, 4, 1., 1.),
            (1, 2, 4., 1.),
        ] {
            let p = reference_at(&config(g, blue, red, b, r), 0.).unwrap();
            assert_eq!(p.forces, [blue as f64, red as f64]);
            assert!(!p.at_boundary);
            assert_eq!(p.survivor, None);
        }
    }
}

#[test]
fn initial_extinction_stops_at_zero_including_both_empty() {
    for g in [Geometry::AimedFire, Geometry::DuelContact] {
        for (blue, red, survivor) in [
            (0, 0, None),
            (0, 3, Some(Side::Red)),
            (3, 0, Some(Side::Blue)),
        ] {
            let p = reference_at(&config(g, blue, red, 0., 0.), 2.).unwrap();
            assert_eq!(p.regime, ReferenceRegime::InitialExtinction);
            assert_eq!(p.forces, [blue as f64, red as f64]);
            assert_eq!(p.requested_time, 2.);
            assert_eq!(p.evaluated_time, 0.);
            assert_eq!(p.extinction_time, Some(0.));
            assert!(p.at_boundary);
            assert_eq!(p.survivor, survivor);
        }
    }
}

#[test]
fn both_zero_rates_are_constant_without_boundary() {
    for g in [Geometry::AimedFire, Geometry::DuelContact] {
        let p = reference_at(&config(g, 3, 4, 0., 0.), 10.).unwrap();
        assert_eq!(p.regime, ReferenceRegime::RateZero);
        assert_eq!(p.forces, [3., 4.]);
        assert_eq!(p.evaluated_time, 10.);
        assert_eq!(p.extinction_time, None);
        assert!(!p.at_boundary);
        assert_eq!(p.survivor, None);
    }
}

#[test]
fn finite_boundary_distinguishes_requested_and_evaluated_time() {
    let c = config(Geometry::AimedFire, 3, 4, 1., 1.);
    let boundary = libm::log(7.) / 2.;
    let before = reference_at(&c, boundary - 1e-6).unwrap();
    assert!(before.forces[0] > 0.);
    assert!(!before.at_boundary);
    assert_eq!(before.survivor, None);
    for t in [boundary, boundary + 1e-6, 5.] {
        let p = reference_at(&c, t).unwrap();
        assert_eq!(p.requested_time, t);
        close(p.evaluated_time, boundary);
        close(p.extinction_time.unwrap(), boundary);
        assert!(p.at_boundary);
        assert_eq!(p.survivor, Some(Side::Red));
        assert_eq!(p.forces[0], 0.);
        close(p.forces[1], libm::sqrt(7.));
    }
}

#[test]
fn side_exchange_mirrors_all_nontrivial_branches() {
    for g in [Geometry::AimedFire, Geometry::DuelContact] {
        for (blue, red, b, r, t) in [
            (1, 2, 3., 1., 1.),
            (3, 4, 1., 1., 3.),
            (1, 3, 2., 0., 2.),
            (3, 1, 2., 0., 0.4),
            (1, 2, 4., 1., 0.1),
        ] {
            let p = reference_at(&config(g, blue, red, b, r), t).unwrap();
            let q = reference_at(&config(g, red, blue, r, b), t).unwrap();
            close(p.forces[0], q.forces[1]);
            close(p.forces[1], q.forces[0]);
            assert_eq!(p.regime, q.regime);
            assert_eq!(p.at_boundary, q.at_boundary);
            assert_eq!(p.extinction_time, q.extinction_time);
            assert_eq!(
                p.survivor.map(|s| if s == Side::Blue {
                    Side::Red
                } else {
                    Side::Blue
                }),
                q.survivor
            );
        }
    }
}

#[test]
fn matched_crossing_is_continuous_and_never_a_finite_extinction() {
    let c = config(Geometry::DuelContact, 1, 2, 3., 1.);
    let crossing = libm::log(2.);
    let p = reference_at(&c, crossing).unwrap();
    close(p.forces[0], 0.5);
    close(p.forces[1], 0.5);
    let before = reference_at(&c, crossing - 1e-6).unwrap();
    let after = reference_at(&c, crossing + 1e-6).unwrap();
    assert!(before.forces[0] < before.forces[1]);
    assert!(after.forces[0] > after.forces[1]);
    assert_eq!(after.extinction_time, None);
    assert!(!after.at_boundary);
}

#[test]
fn exact_dyadic_sign_handles_weighted_balance_and_adjacent_rates() {
    for (g, red, b) in [
        (Geometry::AimedFire, 2, 4.0_f64),
        (Geometry::DuelContact, 4, 4.0),
    ] {
        for (rate, want) in [
            (f64::from_bits(b.to_bits() - 1), Ordering::Less),
            (b, Ordering::Equal),
            (f64::from_bits(b.to_bits() + 1), Ordering::Greater),
        ] {
            assert_eq!(invariant_sign(&config(g, 1, red, rate, 1.)).unwrap(), want);
        }
    }
}

#[test]
fn exact_dyadic_sign_handles_subnormals_large_coefficients_and_exponent_gaps() {
    for g in [Geometry::AimedFire, Geometry::DuelContact] {
        for (blue, red, b, r, want) in [
            (4096, 4095, f64::MAX, f64::MAX, Ordering::Greater),
            (4095, 4096, f64::MAX, f64::MAX, Ordering::Less),
            (1, 4096, f64::MAX, f64::from_bits(1), Ordering::Greater),
            (4096, 1, f64::from_bits(1), f64::MAX, Ordering::Less),
            (1, 1, f64::from_bits(1), f64::from_bits(2), Ordering::Less),
            (0, 0, 1., 2., Ordering::Equal),
            (4096, 4096, f64::MAX, f64::MAX, Ordering::Equal),
            (1, 1, 0., 1., Ordering::Less),
            (1, 1, 1., 0., Ordering::Greater),
        ] {
            let mut c = config(g, blue, red, b, r);
            c.dt = f64::from_bits(1);
            assert_eq!(invariant_sign(&c).unwrap(), want);
        }
    }
}

#[test]
fn unresolved_aimed_imbalance_is_unavailable_instead_of_balanced() {
    let c = config(
        Geometry::AimedFire,
        1,
        2,
        f64::from_bits(4.0_f64.to_bits() + 1),
        1.,
    );
    assert!(reference_at(&c, 0.5).is_err());
}

#[test]
fn asymptotic_force_underflow_is_unavailable_not_extinction() {
    for g in [Geometry::AimedFire, Geometry::DuelContact] {
        let mut c = config(g, 1, 1, 1., 1.);
        c.max_steps = 100_000;
        assert!(reference_at(&c, 1000.).is_err());
    }
}

#[test]
fn unrepresentable_positive_products_and_normalization_are_unavailable() {
    for g in [Geometry::AimedFire, Geometry::DuelContact] {
        let c = config(g, 1, 1, f64::from_bits(1), f64::from_bits(1));
        assert!(reference_at(&c, 0.01).is_err());
    }
    let mut c = config(Geometry::AimedFire, 1, 2, f64::MAX, f64::from_bits(1));
    c.dt = f64::from_bits(1);
    assert!(reference_at(&c, c.dt).is_err());
}

#[test]
fn time_must_be_finite_nonnegative_and_within_declared_horizon() {
    let c = config(Geometry::DuelContact, 1, 1, 1., 1.);
    for t in [-1., f64::NAN, f64::INFINITY, 10.0001] {
        let failure = reference_at(&c, t).unwrap_err();
        assert_eq!(failure.field, "time");
        assert!(!failure.detail.is_empty());
    }
    assert!(reference_at(&c, 10.).is_ok());
}

#[test]
fn invalid_config_returns_reference_failure() {
    let mut c = config(Geometry::AimedFire, 5000, 1, 1., 1.);
    assert!(reference_at(&c, 0.).is_err());
    c.blue = 1;
    c.red_rate = -1.;
    assert!(reference_at(&c, 0.).is_err());
}

#[test]
fn independent_fixed_time_differences_follow_both_odes() {
    for g in [Geometry::AimedFire, Geometry::DuelContact] {
        for (blue, red, b, r, t) in [
            (3, 4, 1., 1., 0.2),
            (1, 2, 3., 1., 0.3),
            (1, 2, 3., 1., 1.),
            (1, 3, 2., 0., 1.2),
        ] {
            let c = config(g, blue, red, b, r);
            let t = if g == Geometry::AimedFire && b == 3. && t == 1. {
                0.6
            } else {
                t
            };
            let f = reference_at(&c, t).unwrap().forces;
            let h = 1e-5;
            let before = reference_at(&c, t - h).unwrap().forces;
            let after = reference_at(&c, t + h).unwrap().forces;
            let want = match g {
                Geometry::AimedFire => [-r * f[1], -b * f[0]],
                Geometry::DuelContact => [-r * f[0].min(f[1]), -b * f[0].min(f[1])],
            };
            for i in 0..2 {
                assert!(((after[i] - before[i]) / (2. * h) - want[i]).abs() < 1e-8);
            }
        }
    }
}

#[test]
fn weighted_invariants_hold_before_and_after_crossings_or_extinction() {
    for g in [Geometry::AimedFire, Geometry::DuelContact] {
        for (blue, red, b, r) in [
            (3, 4, 1., 1.),
            (1, 2, 3., 1.),
            (1, 3, 2., 0.),
            (2, 1, 1., 4.),
        ] {
            let c = config(g, blue, red, b, r);
            let power = if g == Geometry::AimedFire { 2 } else { 1 };
            let initial = b * (blue as f64).powi(power) - r * (red as f64).powi(power);
            for t in [0.1, 0.5, 1., 2.] {
                let f = reference_at(&c, t).unwrap().forces;
                let current = b * f[0].powi(power) - r * f[1].powi(power);
                assert!((current - initial).abs() < 2e-12 * (1. + initial.abs()));
            }
        }
    }
}
