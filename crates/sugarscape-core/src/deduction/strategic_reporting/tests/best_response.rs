use super::super::*;
fn config() -> Config {
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
fn panel(c: &Config) -> TrainingPanel {
    TrainingPanel::new(
        c,
        vec![
            FrozenListener {
                algorithm: Listener::Bayesian,
                assumed_copy_prior: c.fixed_copy_prior.clone(),
            },
            FrozenListener {
                algorithm: Listener::Credulous,
                assumed_copy_prior: c.fixed_copy_prior.clone(),
            },
        ],
    )
    .unwrap()
}
#[test]
fn panel_requires_one_to_sixteen_listeners() {
    let c = config();
    assert!(TrainingPanel::new(&c, vec![]).is_err());
    assert!(TrainingPanel::new(
        &c,
        vec![
            FrozenListener {
                algorithm: Listener::Passive,
                assumed_copy_prior: c.fixed_copy_prior.clone()
            };
            17
        ]
    )
    .is_err());
}
#[test]
fn independently_verified_canonical_training_optimum() {
    let p = panel(&config());
    let best = exact_best_response(&p).unwrap();
    assert_eq!(best.policy.bits, 81942);
    assert_eq!(best.fitness_numerator, 1260);
    assert_eq!(best.denominator, 20000);
    for (policy, fitness) in [
        (Policy::copy(), -4770),
        (Policy::invert(), -810),
        (Policy::positive(), -3150),
        (Policy::negative(), -900),
        (Policy::calibration_copy_live_invert(), 1260),
    ] {
        assert_eq!(p.fitness(&policy).unwrap(), fitness);
        assert!(best.fitness_numerator >= fitness);
    }
}
#[test]
fn unreachable_rows_preserve_fitness() {
    let p = panel(&config());
    for calibration in 0..4 {
        let base = Policy::new(calibration).unwrap();
        for row in 0..16 {
            let signal = row & 8 != 0;
            let own = row & 4 != 0;
            if base.calibration(signal) != own {
                let changed = Policy::new(calibration | (1 << (2 + row))).unwrap();
                assert_eq!(p.fitness(&base).unwrap(), p.fitness(&changed).unwrap());
            }
        }
    }
}
#[test]
fn aligned_utility_and_equal_weight_normalization() {
    let mut c = config();
    let opposed = panel(&c);
    c.strategic_utility = UtilityTable::aligned();
    let aligned = panel(&c);
    for bits in [0, 81942, 174762, 262143] {
        let policy = Policy::new(bits).unwrap();
        assert_eq!(
            opposed.fitness(&policy).unwrap(),
            -aligned.fitness(&policy).unwrap()
        );
    }
    let repeated = TrainingPanel::new(
        &c,
        vec![
            FrozenListener {
                algorithm: Listener::Credulous,
                assumed_copy_prior: c.fixed_copy_prior.clone()
            };
            2
        ],
    )
    .unwrap();
    let single = TrainingPanel::new(
        &c,
        vec![FrozenListener {
            algorithm: Listener::Credulous,
            assumed_copy_prior: c.fixed_copy_prior.clone(),
        }],
    )
    .unwrap();
    assert_eq!(repeated.denominator(), 2 * single.denominator());
    assert_eq!(
        repeated.fitness(&Policy::copy()).unwrap(),
        2 * single.fitness(&Policy::copy()).unwrap()
    );
}
#[test]
fn passive_ties_select_smallest_encoding() {
    let c = config();
    let p = TrainingPanel::new(
        &c,
        vec![FrozenListener {
            algorithm: Listener::Passive,
            assumed_copy_prior: c.fixed_copy_prior.clone(),
        }],
    )
    .unwrap();
    let b = exact_best_response(&p).unwrap();
    assert_eq!(b.policy.bits, 0);
    assert_eq!(b.fitness_numerator, 0);
}
#[test]
fn renaming_is_not_information() {
    let c = config();
    let original = panel(&c);
    let mut renamed = c;
    renamed.strategic = 9;
    renamed.fixed = 21;
    renamed.decider = 33;
    for (p, id) in renamed.permissions.iter_mut().zip([9, 21, 33]) {
        p.agent = id;
    }
    let renamed = panel(&renamed);
    assert_eq!(
        exact_best_response(&original).unwrap(),
        exact_best_response(&renamed).unwrap()
    );
}
// Additive holdout independent oracle SHA-256 c6c4b48bfe15770b1564a68e6062ee985391e1173ad264ab82d26bde97935a63.
#[test]
fn independent_holdout_optima_controls_and_actions() {
    {
        // evolved_q3_5
        let mut c = config();
        c.accuracy = Probability {
            numerator: 3,
            denominator: 5,
        };
        let algorithms = vec![Listener::Evolved(Genome {
            b: -3,
            u: 0,
            d: 2,
            k: 0,
        })];
        let listeners: Vec<_> = algorithms
            .into_iter()
            .map(|algorithm| FrozenListener {
                algorithm,
                assumed_copy_prior: c.fixed_copy_prior.clone(),
            })
            .collect();
        let panel = TrainingPanel::new(&c, listeners.clone()).unwrap();
        let best = exact_best_response(&panel).unwrap();
        assert_eq!(
            (best.policy.bits, best.fitness_numerator, best.denominator),
            (98342, 310, 10000)
        );
        assert_eq!(panel.fitness(&Policy::new(0).unwrap()).unwrap(), -100);
        assert_eq!(panel.fitness(&Policy::new(262143).unwrap()).unwrap(), -350);
        assert_eq!(panel.fitness(&Policy::new(98342).unwrap()).unwrap(), 310);
        assert_eq!(panel.fitness(&Policy::new(174762).unwrap()).unwrap(), -560);
        assert_eq!(panel.fitness(&Policy::new(87382).unwrap()).unwrap(), 130);
        assert_eq!(panel.fitness(&Policy::new(87381).unwrap()).unwrap(), -80);
        let expected = [
            false, false, false, true, false, false, true, true, false, true, false, true, true,
            false, false, false, true, false, false, false, false, true, false, true, false, false,
            true, true, false, false, false, true,
        ];
        for (index, intervene) in expected.into_iter().enumerate() {
            let view = DecisionObservation {
                rules: c.clone(),
                calibration_truth: index & 16 != 0,
                calibration_reports: [index & 8 != 0, index & 4 != 0],
                live_reports: [index & 2 != 0, index & 1 != 0],
            };
            assert_eq!(
                listeners[0].decide(&view).unwrap() == DecisionAction::Intervene,
                intervene
            );
        }
    }
    {
        // evolved_q4_5
        let mut c = config();
        c.accuracy = Probability {
            numerator: 4,
            denominator: 5,
        };
        let algorithms = vec![Listener::Evolved(Genome {
            b: -3,
            u: 0,
            d: 2,
            k: 0,
        })];
        let listeners: Vec<_> = algorithms
            .into_iter()
            .map(|algorithm| FrozenListener {
                algorithm,
                assumed_copy_prior: c.fixed_copy_prior.clone(),
            })
            .collect();
        let panel = TrainingPanel::new(&c, listeners.clone()).unwrap();
        let best = exact_best_response(&panel).unwrap();
        assert_eq!(
            (best.policy.bits, best.fitness_numerator, best.denominator),
            (98342, 720, 10000)
        );
        assert_eq!(panel.fitness(&Policy::new(0).unwrap()).unwrap(), -900);
        assert_eq!(panel.fitness(&Policy::new(262143).unwrap()).unwrap(), -1650);
        assert_eq!(panel.fitness(&Policy::new(98342).unwrap()).unwrap(), 720);
        assert_eq!(panel.fitness(&Policy::new(174762).unwrap()).unwrap(), -2520);
        assert_eq!(panel.fitness(&Policy::new(87382).unwrap()).unwrap(), 510);
        assert_eq!(panel.fitness(&Policy::new(87381).unwrap()).unwrap(), -1560);
        let expected = [
            false, false, false, true, false, false, true, true, false, true, false, true, true,
            false, false, false, true, false, false, false, false, true, false, true, false, false,
            true, true, false, false, false, true,
        ];
        for (index, intervene) in expected.into_iter().enumerate() {
            let view = DecisionObservation {
                rules: c.clone(),
                calibration_truth: index & 16 != 0,
                calibration_reports: [index & 8 != 0, index & 4 != 0],
                live_reports: [index & 2 != 0, index & 1 != 0],
            };
            assert_eq!(
                listeners[0].decide(&view).unwrap() == DecisionAction::Intervene,
                intervene
            );
        }
    }
    {
        // skeptical_q3_5
        let mut c = config();
        c.accuracy = Probability {
            numerator: 3,
            denominator: 5,
        };
        let algorithms = vec![Listener::Skeptical];
        let listeners: Vec<_> = algorithms
            .into_iter()
            .map(|algorithm| FrozenListener {
                algorithm,
                assumed_copy_prior: c.fixed_copy_prior.clone(),
            })
            .collect();
        let panel = TrainingPanel::new(&c, listeners.clone()).unwrap();
        let best = exact_best_response(&panel).unwrap();
        assert_eq!(
            (best.policy.bits, best.fitness_numerator, best.denominator),
            (5140, 250, 10000)
        );
        assert_eq!(panel.fitness(&Policy::new(0).unwrap()).unwrap(), 0);
        assert_eq!(panel.fitness(&Policy::new(262143).unwrap()).unwrap(), -500);
        assert_eq!(panel.fitness(&Policy::new(5140).unwrap()).unwrap(), 250);
        assert_eq!(panel.fitness(&Policy::new(174762).unwrap()).unwrap(), -750);
        assert_eq!(panel.fitness(&Policy::new(87382).unwrap()).unwrap(), 250);
        assert_eq!(panel.fitness(&Policy::new(87381).unwrap()).unwrap(), 250);
        let expected = [
            false, false, false, true, false, false, false, true, false, false, false, true, false,
            false, false, true, false, false, false, true, false, false, false, true, false, false,
            false, true, false, false, false, true,
        ];
        for (index, intervene) in expected.into_iter().enumerate() {
            let view = DecisionObservation {
                rules: c.clone(),
                calibration_truth: index & 16 != 0,
                calibration_reports: [index & 8 != 0, index & 4 != 0],
                live_reports: [index & 2 != 0, index & 1 != 0],
            };
            assert_eq!(
                listeners[0].decide(&view).unwrap() == DecisionAction::Intervene,
                intervene
            );
        }
    }
    {
        // skeptical_q4_5
        let mut c = config();
        c.accuracy = Probability {
            numerator: 4,
            denominator: 5,
        };
        let algorithms = vec![Listener::Skeptical];
        let listeners: Vec<_> = algorithms
            .into_iter()
            .map(|algorithm| FrozenListener {
                algorithm,
                assumed_copy_prior: c.fixed_copy_prior.clone(),
            })
            .collect();
        let panel = TrainingPanel::new(&c, listeners.clone()).unwrap();
        let best = exact_best_response(&panel).unwrap();
        assert_eq!(
            (best.policy.bits, best.fitness_numerator, best.denominator),
            (5140, 750, 10000)
        );
        assert_eq!(panel.fitness(&Policy::new(0).unwrap()).unwrap(), 0);
        assert_eq!(panel.fitness(&Policy::new(262143).unwrap()).unwrap(), -1500);
        assert_eq!(panel.fitness(&Policy::new(5140).unwrap()).unwrap(), 750);
        assert_eq!(panel.fitness(&Policy::new(174762).unwrap()).unwrap(), -2250);
        assert_eq!(panel.fitness(&Policy::new(87382).unwrap()).unwrap(), 750);
        assert_eq!(panel.fitness(&Policy::new(87381).unwrap()).unwrap(), 750);
        let expected = [
            false, false, false, true, false, false, false, true, false, false, false, true, false,
            false, false, true, false, false, false, true, false, false, false, true, false, false,
            false, true, false, false, false, true,
        ];
        for (index, intervene) in expected.into_iter().enumerate() {
            let view = DecisionObservation {
                rules: c.clone(),
                calibration_truth: index & 16 != 0,
                calibration_reports: [index & 8 != 0, index & 4 != 0],
                live_reports: [index & 2 != 0, index & 1 != 0],
            };
            assert_eq!(
                listeners[0].decide(&view).unwrap() == DecisionAction::Intervene,
                intervene
            );
        }
    }
    {
        // training_pair_q3_5
        let mut c = config();
        c.accuracy = Probability {
            numerator: 3,
            denominator: 5,
        };
        let algorithms = vec![Listener::Bayesian, Listener::Credulous];
        let listeners: Vec<_> = algorithms
            .into_iter()
            .map(|algorithm| FrozenListener {
                algorithm,
                assumed_copy_prior: c.fixed_copy_prior.clone(),
            })
            .collect();
        let panel = TrainingPanel::new(&c, listeners.clone()).unwrap();
        let best = exact_best_response(&panel).unwrap();
        assert_eq!(
            (best.policy.bits, best.fitness_numerator, best.denominator),
            (81942, 500, 20000)
        );
        assert_eq!(panel.fitness(&Policy::new(0).unwrap()).unwrap(), -175);
        assert_eq!(panel.fitness(&Policy::new(262143).unwrap()).unwrap(), -925);
        assert_eq!(panel.fitness(&Policy::new(81942).unwrap()).unwrap(), 500);
        assert_eq!(panel.fitness(&Policy::new(174762).unwrap()).unwrap(), -1550);
        assert_eq!(panel.fitness(&Policy::new(87382).unwrap()).unwrap(), 500);
        assert_eq!(panel.fitness(&Policy::new(87381).unwrap()).unwrap(), 350);
        let expected = [
            false, false, false, true, false, false, true, true, false, true, false, true, false,
            false, false, true, false, false, false, true, false, true, false, true, false, false,
            true, true, false, false, false, true,
        ];
        for (index, intervene) in expected.into_iter().enumerate() {
            let view = DecisionObservation {
                rules: c.clone(),
                calibration_truth: index & 16 != 0,
                calibration_reports: [index & 8 != 0, index & 4 != 0],
                live_reports: [index & 2 != 0, index & 1 != 0],
            };
            assert_eq!(
                listeners[0].decide(&view).unwrap() == DecisionAction::Intervene,
                intervene
            );
        }
        let expected = [
            false, false, false, true, false, false, false, true, false, false, false, true, false,
            false, false, true, false, false, false, true, false, false, false, true, false, false,
            false, true, false, false, false, true,
        ];
        for (index, intervene) in expected.into_iter().enumerate() {
            let view = DecisionObservation {
                rules: c.clone(),
                calibration_truth: index & 16 != 0,
                calibration_reports: [index & 8 != 0, index & 4 != 0],
                live_reports: [index & 2 != 0, index & 1 != 0],
            };
            assert_eq!(
                listeners[1].decide(&view).unwrap() == DecisionAction::Intervene,
                intervene
            );
        }
    }
}
#[test]
fn arithmetic_bounds_and_zero_world_retention() {
    let mut c = config();
    c.accuracy = Probability {
        numerator: 16,
        denominator: 16,
    };
    c.fixed_copy_prior = Probability {
        numerator: 16,
        denominator: 16,
    };
    let d = enumerate(&c).unwrap();
    assert_eq!(d.denominator(), 4194304);
    assert_eq!(d.worlds.len(), 128);
    assert!(d.worlds.iter().any(|w| w.mass == 0));
    let panel = TrainingPanel::new(
        &c,
        vec![
            FrozenListener {
                algorithm: Listener::Passive,
                assumed_copy_prior: c.fixed_copy_prior.clone()
            };
            16
        ],
    )
    .unwrap();
    assert_eq!(panel.denominator(), 67108864);
    assert_eq!(panel.fitness(&Policy::positive()).unwrap(), 0);
}
