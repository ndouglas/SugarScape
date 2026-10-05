use super::super::*;
use crate::deduction::testimony_game;
fn rules() -> Config {
    Config::standard(
        Probability {
            numerator: 4,
            denominator: 5,
        },
        Probability {
            numerator: 3,
            denominator: 4,
        },
        UtilityTable::opposed(),
    )
}
#[test]
fn validates_permissions_and_independent_utility() {
    let c = rules();
    assert!(c.validate().is_ok());
    for index in 0..3 {
        for field in 0..4 {
            let mut bad = c.clone();
            let p = &mut bad.permissions[index];
            match field {
                0 => p.receive_signal = !p.receive_signal,
                1 => p.report = !p.report,
                2 => p.observe_verification = !p.observe_verification,
                _ => p.decide = !p.decide,
            };
            assert!(bad.validate().is_err());
        }
    }
    assert_eq!(
        UtilityTable::aligned().utility(false, DecisionAction::Intervene),
        -1
    );
    assert_eq!(
        UtilityTable::opposed().utility(false, DecisionAction::Intervene),
        1
    );
    for truth in [false, true] {
        for action in [DecisionAction::Intervene, DecisionAction::Abstain] {
            assert_eq!(
                UtilityTable::aligned().utility(truth, action),
                decision_payoff(truth, action)
            );
        }
    }
}
#[test]
fn rejects_invalid_configuration() {
    for n in 0..11 {
        let mut c = rules();
        match n {
            0 => {
                c.permissions.pop();
            }
            1 => c.permissions.push(c.permissions[0].clone()),
            2 => c.permissions[1].agent = c.strategic,
            3 => c.fixed = c.strategic,
            4 => c.version = 2,
            5 => c.accuracy.numerator = 6,
            6 => c.accuracy.denominator = 0,
            7 => c.fixed_copy_prior.denominator = 17,
            8 => c.strategic_utility.intervene_true = 2,
            9 => c.strategic_utility.abstain_false = -2,
            _ => c.permissions[0].agent = 99,
        };
        assert!(c.validate().is_err(), "case {n}");
    }
}
#[test]
fn canonical_rows_and_controls() {
    assert!(Policy::new(1 << 18).is_err());
    for row in 0..16 {
        let p = Policy::new(1 << (2 + row)).unwrap();
        for input in 0..16 {
            assert_eq!(
                p.live(
                    input & 8 != 0,
                    input & 4 != 0,
                    input & 2 != 0,
                    input & 1 != 0
                ),
                row == input
            );
        }
    }
    for signal in [false, true] {
        assert!(Policy::new(1 << u32::from(signal))
            .unwrap()
            .calibration(signal));
        for c in 0..8 {
            assert_eq!(
                Policy::copy().live(c & 4 != 0, c & 2 != 0, c & 1 != 0, signal),
                signal
            );
            assert_eq!(
                Policy::invert().live(c & 4 != 0, c & 2 != 0, c & 1 != 0, signal),
                !signal
            );
            assert!(Policy::positive().live(c & 4 != 0, c & 2 != 0, c & 1 != 0, signal));
            assert!(!Policy::negative().live(c & 4 != 0, c & 2 != 0, c & 1 != 0, signal));
            assert_eq!(
                Policy::calibration_copy_live_invert().live(
                    c & 4 != 0,
                    c & 2 != 0,
                    c & 1 != 0,
                    signal
                ),
                !signal
            );
        }
        assert_eq!(Policy::copy().calibration(signal), signal);
        assert_eq!(Policy::invert().calibration(signal), !signal);
        assert_eq!(
            Policy::calibration_copy_live_invert().calibration(signal),
            signal
        );
    }
}
#[test]
fn partial_observations_rejected_and_other_report_ignored() {
    let p = Policy::copy();
    for mask in 0..8 {
        let mut v = StrategicObservation {
            rules: rules(),
            signal: true,
            calibration_signal: None,
            calibration_reports: None,
            calibration_truth: None,
        };
        if mask & 1 != 0 {
            v.calibration_signal = Some(false);
        }
        if mask & 2 != 0 {
            v.calibration_reports = Some([false, false]);
        }
        if mask & 4 != 0 {
            v.calibration_truth = Some(true);
        }
        assert_eq!(p.report(&v).is_ok(), mask == 0 || mask == 7);
        if mask == 7 {
            let before = p.report(&v).unwrap();
            v.calibration_reports = Some([false, true]);
            assert_eq!(p.report(&v).unwrap(), before);
        }
    }
}
#[test]
fn adapter_matches_legacy_for_all_histories_and_endpoints() {
    let algorithms = [
        Listener::Bayesian,
        Listener::Credulous,
        Listener::Skeptical,
        Listener::DirectEvidence,
        Listener::Passive,
        Listener::Evolved(Genome {
            b: -3,
            u: 0,
            d: 2,
            k: 0,
        }),
    ];
    for q in [0, 1, 2] {
        for rho in [0, 1, 2] {
            for bits in 0..32 {
                for algorithm in &algorithms {
                    let mut c = rules();
                    c.accuracy = Probability {
                        numerator: q,
                        denominator: 2,
                    };
                    c.strategic = 7;
                    c.fixed = 9;
                    c.decider = 11;
                    for (p, id) in c.permissions.iter_mut().zip([7, 9, 11]) {
                        p.agent = id;
                    }
                    let v = DecisionObservation {
                        rules: c.clone(),
                        calibration_reports: [bits & 1 != 0, bits & 2 != 0],
                        calibration_truth: bits & 4 != 0,
                        live_reports: [bits & 8 != 0, bits & 16 != 0],
                    };
                    let prior = Probability {
                        numerator: rho,
                        denominator: 2,
                    };
                    let mut old =
                        testimony_game::Config::standard(c.accuracy.clone(), prior.clone());
                    old.reporters = [7, 9];
                    old.decider = 11;
                    old.permissions = c.permissions;
                    let legacy = testimony_game::DecisionObservation {
                        rules: old,
                        calibration_reports: v.calibration_reports,
                        calibration_truth: v.calibration_truth,
                        live_reports: v.live_reports,
                    };
                    let expected = algorithm.decide(&legacy).map(|d| d.action);
                    let actual = FrozenListener {
                        algorithm: algorithm.clone(),
                        assumed_copy_prior: prior,
                    }
                    .decide(&v);
                    assert_eq!(actual, expected.map_err(Error::LegacyListener));
                }
            }
        }
    }
}
#[test]
fn strict_decoding() {
    fn unknown<T: serde::Serialize + serde::de::DeserializeOwned>(value: T) {
        let mut json = serde_json::to_value(value).unwrap();
        json.as_object_mut()
            .unwrap()
            .insert("unknown".into(), true.into());
        assert!(serde_json::from_value::<T>(json).is_err());
    }
    unknown(rules());
    unknown(UtilityTable::opposed());
    unknown(Policy::copy());
    unknown(FrozenListener {
        algorithm: Listener::Bayesian,
        assumed_copy_prior: Probability {
            numerator: 3,
            denominator: 4,
        },
    });
    unknown(StrategicObservation {
        rules: rules(),
        signal: false,
        calibration_signal: None,
        calibration_reports: None,
        calibration_truth: None,
    });
    unknown(FixedObservation {
        rules: rules(),
        signal: false,
        profile: Profile::Copy,
        calibration_reports: None,
        calibration_truth: None,
    });
    unknown(DecisionObservation {
        rules: rules(),
        calibration_reports: [false; 2],
        calibration_truth: false,
        live_reports: [false; 2],
    });
}
