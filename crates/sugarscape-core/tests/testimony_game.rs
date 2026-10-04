use sugarscape_core::deduction::testimony_game::*;

fn probability(numerator: u16, denominator: u16) -> Probability {
    Probability {
        numerator,
        denominator,
    }
}
fn config() -> Config {
    Config::standard(probability(4, 5), probability(3, 4))
}

#[test]
fn rational_bounds_and_endpoints_are_validated() {
    for bad in [probability(0, 0), probability(1, 17), probability(3, 2)] {
        let mut c = config();
        c.accuracy = bad.clone();
        assert!(matches!(c.validate(), Err(Error::InvalidConfig(_))));
        let mut c = config();
        c.copy_prior = bad;
        assert!(c.validate().is_err());
    }
    for p in [probability(0, 1), probability(1, 1), probability(16, 16)] {
        assert!(Config::standard(p.clone(), p).validate().is_ok());
    }
}
#[test]
fn permissions_require_exact_three_distinct_participants() {
    let mut c = config();
    c.reporters = [42, 700];
    c.decider = 65535;
    for (p, id) in c.permissions.iter_mut().zip([42, 700, 65535]) {
        p.agent = id;
    }
    assert!(c.validate().is_ok());
    let valid = c.clone();
    c.reporters[1] = 42;
    assert!(c.validate().is_err());
    c = valid.clone();
    c.permissions.pop();
    assert!(c.validate().is_err());
    c = valid.clone();
    c.permissions[1].agent = 42;
    assert!(c.validate().is_err());
    c = valid.clone();
    c.permissions[1].agent = 900;
    assert!(c.validate().is_err());
    for index in 0..3 {
        for field in 0..4 {
            c = valid.clone();
            let p = &mut c.permissions[index];
            match field {
                0 => p.receive_signal = !p.receive_signal,
                1 => p.report = !p.report,
                2 => p.observe_verification = !p.observe_verification,
                _ => p.decide = !p.decide,
            };
            assert!(c.validate().is_err());
        }
    }
    c = valid;
    c.version = 2;
    assert!(c.validate().is_err());
}
#[test]
fn reporting_atoms_copy_or_invert_both_signal_values() {
    assert!(report(Profile::Copy, true));
    assert!(!report(Profile::Copy, false));
    assert!(!report(Profile::Invert, true));
    assert!(report(Profile::Invert, false));
}
#[test]
fn config_and_actions_reject_unknown_fields() {
    for input in [
        r#"{"kind":"abstain","truth":true}"#,
        r#"{"kind":"intervene","truth":true}"#,
        r#"{"kind":"report","positive":true,"truth":true}"#,
    ] {
        assert!(serde_json::from_str::<Action>(input).is_err());
    }
    let mut value = serde_json::to_value(config()).unwrap();
    for path in ["", "/accuracy", "/copy_prior", "/permissions/0"] {
        let mut bad = value.clone();
        bad.pointer_mut(path)
            .unwrap()
            .as_object_mut()
            .unwrap()
            .insert("secret".into(), true.into());
        assert!(serde_json::from_value::<Config>(bad).is_err());
    }
    value["seed"] = 7.into();
    assert!(serde_json::from_value::<Config>(value).is_err());
}

fn respond(game: &mut Session, action: Action) {
    let r = game.request().unwrap();
    game.submit(Response {
        request_id: r.request_id,
        actor: r.actor,
        action,
    })
    .unwrap();
}
fn drive_report(game: &mut Session) {
    let r = game.request().unwrap();
    let Observation::Reporter { view } = r.observation else {
        panic!("expected reporter")
    };
    respond(
        game,
        Action::Report {
            positive: report(view.profile, view.signal),
        },
    );
}
fn finish(game: &mut Session) -> Vec<Request> {
    let mut requests = vec![];
    while let Some(r) = game.request() {
        requests.push(r.clone());
        if r.step == Step::Decision {
            respond(game, Action::Intervene);
        } else {
            drive_report(game);
        }
    }
    requests
}
#[test]
fn buffered_reports_and_verified_truth_arrive_only_after_calibration() {
    let mut game = Session::new(config(), 7).unwrap();
    for index in 0..4 {
        let r = game.request().unwrap();
        assert_eq!(r.actor, [0, 1, 0, 1][index]);
        let Observation::Reporter { view } = r.observation else {
            panic!()
        };
        if index < 2 {
            assert_eq!(
                (view.calibration_reports, view.calibration_truth),
                (None, None)
            );
        } else {
            assert_eq!(view.calibration_reports, Some([false, false]));
            assert!(view.calibration_truth.is_some());
        }
        let serialized = serde_json::to_value(&view).unwrap();
        assert!(serialized.get("live_reports").is_none());
        respond(&mut game, Action::Report { positive: false });
    }
    let r = game.request().unwrap();
    assert_eq!(r.actor, 2);
    assert_eq!(r.step, Step::Decision);
    assert_eq!(r.legal, vec![LegalAction::Intervene, LegalAction::Abstain]);
    let Observation::Decider { view } = &r.observation else {
        panic!()
    };
    assert_eq!(view.calibration_reports, [false, false]);
    assert_eq!(view.live_reports, [false, false]);
    let json = serde_json::to_value(r).unwrap();
    let obj = json["observation"]["view"].as_object().unwrap();
    for hidden in ["signal", "profile", "seed", "live_truth"] {
        assert!(!obj.contains_key(hidden));
    }
    assert!(game.outcome().is_none());
}
#[test]
fn rejected_responses_preserve_archive_and_entire_future() {
    for accepted in 0..=5 {
        let mut game = Session::new(config(), 77).unwrap();
        for _ in 0..accepted {
            if game.request().unwrap().step == Step::Decision {
                respond(&mut game, Action::Abstain);
            } else {
                drive_report(&mut game);
            }
        }
        let before = game.archive();
        let mut baseline_game = replay(&before).unwrap();
        let baseline = finish(&mut baseline_game);
        let baseline_outcome = baseline_game.outcome().cloned();
        let (id, actor) = game
            .request()
            .map(|r| (r.request_id, r.actor))
            .unwrap_or((6, 2));
        let legal = if accepted < 4 {
            Action::Report { positive: true }
        } else {
            Action::Abstain
        };
        let illegal = if accepted < 4 {
            Action::Intervene
        } else {
            Action::Report { positive: true }
        };
        for response in [
            Response {
                request_id: id + 1,
                actor,
                action: legal.clone(),
            },
            Response {
                request_id: id,
                actor: actor + 10,
                action: legal.clone(),
            },
            Response {
                request_id: id,
                actor,
                action: illegal,
            },
        ] {
            assert_eq!(game.submit(response), Err(Error::InvalidResponse));
            assert_eq!(game.archive(), before);
        }
        assert_eq!(finish(&mut game), baseline);
        assert_eq!(game.outcome(), baseline_outcome.as_ref());
    }
}
#[test]
fn partial_and_completed_archives_replay_and_reject_tampering() {
    let mut game = Session::new(config(), 91).unwrap();
    for count in 0..=5 {
        let archive = game.archive();
        assert_eq!(replay(&archive).unwrap().archive(), archive);
        let mut bad = archive.clone();
        bad.protocol_version += 1;
        assert!(matches!(replay(&bad), Err(Error::VersionMismatch)));
        let mut bad = archive.clone();
        bad.config.version += 1;
        assert!(matches!(replay(&bad), Err(Error::VersionMismatch)));
        if count > 0 {
            let mut bad = archive.clone();
            bad.responses[count - 1].actor = 99;
            assert!(matches!(replay(&bad),Err(Error::InvalidArchive{index:Some(i)}) if i==count-1));
        }
        let mut bad = archive.clone();
        if count < 5 {
            bad.checkpoint.request.as_mut().unwrap().request_id += 1;
        } else {
            bad.checkpoint.outcome.as_mut().unwrap().payoff = 99;
        }
        assert!(matches!(
            replay(&bad),
            Err(Error::InvalidArchive { index: None })
        ));
        if count < 4 {
            drive_report(&mut game);
        } else if count == 4 {
            respond(&mut game, Action::Abstain);
        }
    }
}
#[test]
fn completion_reveals_truth_and_fixed_payoffs() {
    let mut seen = [false, false];
    for seed in 0..32 {
        for action in [Action::Intervene, Action::Abstain] {
            let mut game = Session::new(config(), seed).unwrap();
            for _ in 0..4 {
                drive_report(&mut game);
            }
            assert!(game.outcome().is_none());
            respond(&mut game, action.clone());
            let o = game.outcome().unwrap();
            seen[usize::from(o.live_truth)] = true;
            assert_eq!(
                o.payoff,
                match action {
                    Action::Abstain => 0,
                    _ =>
                        if o.live_truth {
                            1
                        } else {
                            -1
                        },
                }
            );
            assert!(game.request().is_none());
        }
    }
    assert_eq!(seen, [true, true]);
}
#[test]
fn renaming_ids_preserves_reporter_slots_and_outcomes() {
    let mut renamed = config();
    renamed.reporters = [500, 12];
    renamed.decider = 900;
    for (p, id) in renamed.permissions.iter_mut().zip([500, 12, 900]) {
        p.agent = id;
    }
    let mut original = Session::new(config(), 42).unwrap();
    let mut other = Session::new(renamed, 42).unwrap();
    while let Some(r) = original.request() {
        let s = other.request().unwrap();
        assert_eq!(
            s.actor,
            match r.actor {
                0 => 500,
                1 => 12,
                _ => 900,
            }
        );
        match (&r.observation, &s.observation) {
            (Observation::Reporter { view: a }, Observation::Reporter { view: b }) => assert_eq!(
                (
                    a.signal,
                    a.profile,
                    a.calibration_reports,
                    a.calibration_truth
                ),
                (
                    b.signal,
                    b.profile,
                    b.calibration_reports,
                    b.calibration_truth
                )
            ),
            (Observation::Decider { view: a }, Observation::Decider { view: b }) => assert_eq!(
                (a.calibration_reports, a.calibration_truth, a.live_reports),
                (b.calibration_reports, b.calibration_truth, b.live_reports)
            ),
            _ => panic!(),
        }
        if r.step == Step::Decision {
            respond(&mut original, Action::Intervene);
            respond(&mut other, Action::Intervene);
        } else {
            drive_report(&mut original);
            drive_report(&mut other);
        }
    }
    assert_eq!(original.outcome(), other.outcome());
}
#[test]
fn endpoints_produce_exact_private_signals_and_profiles() {
    for accuracy in [0, 1] {
        for copy in [0, 1] {
            for seed in 0..16 {
                let mut game = Session::new(
                    Config::standard(probability(accuracy, 1), probability(copy, 1)),
                    seed,
                )
                .unwrap();
                let mut signals = [false; 4];
                for signal in &mut signals {
                    let Observation::Reporter { view } = game.request().unwrap().observation else {
                        panic!()
                    };
                    assert_eq!(
                        view.profile,
                        if copy == 1 {
                            Profile::Copy
                        } else {
                            Profile::Invert
                        }
                    );
                    *signal = view.signal;
                    drive_report(&mut game);
                }
                let Observation::Decider { view } = game.request().unwrap().observation else {
                    panic!()
                };
                assert_eq!(
                    [signals[0], signals[1]],
                    [view.calibration_truth == (accuracy == 1); 2]
                );
                respond(&mut game, Action::Intervene);
                let truth = game.outcome().unwrap().live_truth;
                assert_eq!([signals[2], signals[3]], [truth == (accuracy == 1); 2]);
            }
        }
    }
}
#[test]
fn every_nested_session_wire_object_rejects_unknown_fields() {
    let mut game = Session::new(config(), 7).unwrap();
    for _ in 0..5 {
        let archive = serde_json::to_value(game.archive()).unwrap();
        for path in [
            "",
            "/config",
            "/config/accuracy",
            "/config/copy_prior",
            "/config/permissions/0",
            "/checkpoint",
            "/checkpoint/request",
            "/checkpoint/request/observation",
            "/checkpoint/request/observation/view",
            "/checkpoint/request/observation/view/rules",
        ] {
            let mut bad = archive.clone();
            bad.pointer_mut(path)
                .unwrap()
                .as_object_mut()
                .unwrap()
                .insert("secret".into(), true.into());
            assert!(serde_json::from_value::<Archive>(bad).is_err(), "{path}");
        }
        if game.request().unwrap().step == Step::Decision {
            respond(&mut game, Action::Abstain);
        } else {
            drive_report(&mut game);
        }
    }
    let value = serde_json::to_value(game.archive()).unwrap();
    for path in [
        "/responses/0",
        "/responses/0/action",
        "/responses/4/action",
        "/checkpoint/outcome",
    ] {
        let mut bad = value.clone();
        bad.pointer_mut(path)
            .unwrap()
            .as_object_mut()
            .unwrap()
            .insert("secret".into(), true.into());
        assert!(serde_json::from_value::<Archive>(bad).is_err(), "{path}");
    }
}

// Frozen independent Fraction enumeration; no Rust implementation generated these values.
#[test]
fn exact_histories_and_baselines_match_independent_fraction_references() {
    let c = Config::standard(probability(4, 5), probability(3, 4));
    let dist = enumerate(&c).unwrap();
    assert_eq!(dist.denominator, 40000);
    let expected = [
        ([false, false, false, false, false], 2657u64, 256u64),
        ([false, false, false, false, true], 1568u64, 784u64),
        ([false, false, false, true, false], 1568u64, 784u64),
        ([false, false, false, true, true], 2657u64, 2401u64),
        ([false, false, true, false, false], 1088u64, 304u64),
        ([false, false, true, false, true], 1187u64, 256u64),
        ([false, false, true, true, false], 1187u64, 931u64),
        ([false, false, true, true, true], 1088u64, 784u64),
        ([false, true, false, false, false], 1088u64, 304u64),
        ([false, true, false, false, true], 1187u64, 931u64),
        ([false, true, false, true, false], 1187u64, 256u64),
        ([false, true, false, true, true], 1088u64, 784u64),
        ([false, true, true, false, false], 617u64, 361u64),
        ([false, true, true, false, true], 608u64, 304u64),
        ([false, true, true, true, false], 608u64, 304u64),
        ([false, true, true, true, true], 617u64, 256u64),
        ([true, false, false, false, false], 617u64, 361u64),
        ([true, false, false, false, true], 608u64, 304u64),
        ([true, false, false, true, false], 608u64, 304u64),
        ([true, false, false, true, true], 617u64, 256u64),
        ([true, false, true, false, false], 1088u64, 304u64),
        ([true, false, true, false, true], 1187u64, 931u64),
        ([true, false, true, true, false], 1187u64, 256u64),
        ([true, false, true, true, true], 1088u64, 784u64),
        ([true, true, false, false, false], 1088u64, 304u64),
        ([true, true, false, false, true], 1187u64, 256u64),
        ([true, true, false, true, false], 1187u64, 931u64),
        ([true, true, false, true, true], 1088u64, 784u64),
        ([true, true, true, false, false], 2657u64, 256u64),
        ([true, true, true, false, true], 1568u64, 784u64),
        ([true, true, true, true, false], 1568u64, 784u64),
        ([true, true, true, true, true], 2657u64, 2401u64),
    ];
    assert_eq!(dist.histories.len(), expected.len());
    for (h, (key, total, truth)) in dist.histories.iter().zip(expected) {
        assert_eq!(
            [
                h.observation.calibration_truth,
                h.observation.calibration_reports[0],
                h.observation.calibration_reports[1],
                h.observation.live_reports[0],
                h.observation.live_reports[1]
            ],
            key
        );
        assert_eq!((h.total_mass, h.true_mass), (total, truth));
    }
    for (policy, payoff) in [
        (Listener::Bayesian, 9120),
        (Listener::Credulous, 6000),
        (Listener::Skeptical, 6000),
        (Listener::Passive, 0),
        (Listener::DirectEvidence, 0),
    ] {
        let e = evaluate(&dist, &policy).unwrap();
        assert_eq!(e.payoff_numerator, payoff);
        assert_eq!(e.optimal_numerator, 9120);
        assert_eq!(e.regret_numerator, 9120 - payoff);
        if policy == Listener::Bayesian {
            assert!(e.posterior_max_error.unwrap() < 1e-12);
            assert_eq!(e.regret_numerator, 0);
        }
    }
    let c = Config::standard(probability(4, 5), probability(1, 4));
    let dist = enumerate(&c).unwrap();
    assert_eq!(dist.denominator, 40000);
    let expected = [
        ([false, false, false, false, false], 617u64, 256u64),
        ([false, false, false, false, true], 608u64, 304u64),
        ([false, false, false, true, false], 608u64, 304u64),
        ([false, false, false, true, true], 617u64, 361u64),
        ([false, false, true, false, false], 1088u64, 784u64),
        ([false, false, true, false, true], 1187u64, 256u64),
        ([false, false, true, true, false], 1187u64, 931u64),
        ([false, false, true, true, true], 1088u64, 304u64),
        ([false, true, false, false, false], 1088u64, 784u64),
        ([false, true, false, false, true], 1187u64, 931u64),
        ([false, true, false, true, false], 1187u64, 256u64),
        ([false, true, false, true, true], 1088u64, 304u64),
        ([false, true, true, false, false], 2657u64, 2401u64),
        ([false, true, true, false, true], 1568u64, 784u64),
        ([false, true, true, true, false], 1568u64, 784u64),
        ([false, true, true, true, true], 2657u64, 256u64),
        ([true, false, false, false, false], 2657u64, 2401u64),
        ([true, false, false, false, true], 1568u64, 784u64),
        ([true, false, false, true, false], 1568u64, 784u64),
        ([true, false, false, true, true], 2657u64, 256u64),
        ([true, false, true, false, false], 1088u64, 784u64),
        ([true, false, true, false, true], 1187u64, 931u64),
        ([true, false, true, true, false], 1187u64, 256u64),
        ([true, false, true, true, true], 1088u64, 304u64),
        ([true, true, false, false, false], 1088u64, 784u64),
        ([true, true, false, false, true], 1187u64, 256u64),
        ([true, true, false, true, false], 1187u64, 931u64),
        ([true, true, false, true, true], 1088u64, 304u64),
        ([true, true, true, false, false], 617u64, 256u64),
        ([true, true, true, false, true], 608u64, 304u64),
        ([true, true, true, true, false], 608u64, 304u64),
        ([true, true, true, true, true], 617u64, 361u64),
    ];
    assert_eq!(dist.histories.len(), expected.len());
    for (h, (key, total, truth)) in dist.histories.iter().zip(expected) {
        assert_eq!(
            [
                h.observation.calibration_truth,
                h.observation.calibration_reports[0],
                h.observation.calibration_reports[1],
                h.observation.live_reports[0],
                h.observation.live_reports[1]
            ],
            key
        );
        assert_eq!((h.total_mass, h.true_mass), (total, truth));
    }
    for (policy, payoff) in [
        (Listener::Bayesian, 9120),
        (Listener::Credulous, -6000),
        (Listener::Skeptical, 6000),
        (Listener::Passive, 0),
        (Listener::DirectEvidence, 0),
    ] {
        let e = evaluate(&dist, &policy).unwrap();
        assert_eq!(e.payoff_numerator, payoff);
        assert_eq!(e.optimal_numerator, 9120);
        assert_eq!(e.regret_numerator, 9120 - payoff);
        if policy == Listener::Bayesian {
            assert!(e.posterior_max_error.unwrap() < 1e-12);
            assert_eq!(e.regret_numerator, 0);
        }
    }
    let c = Config::standard(probability(3, 5), probability(3, 4));
    let dist = enumerate(&c).unwrap();
    assert_eq!(dist.denominator, 40000);
    let expected = [
        ([false, false, false, false, false], 1537u64, 576u64),
        ([false, false, false, false, true], 1488u64, 744u64),
        ([false, false, false, true, false], 1488u64, 744u64),
        ([false, false, false, true, true], 1537u64, 961u64),
        ([false, false, true, false, false], 1248u64, 504u64),
        ([false, false, true, false, true], 1227u64, 576u64),
        ([false, false, true, true, false], 1227u64, 651u64),
        ([false, false, true, true, true], 1248u64, 744u64),
        ([false, true, false, false, false], 1248u64, 504u64),
        ([false, true, false, false, true], 1227u64, 651u64),
        ([false, true, false, true, false], 1227u64, 576u64),
        ([false, true, false, true, true], 1248u64, 744u64),
        ([false, true, true, false, false], 1017u64, 441u64),
        ([false, true, true, false, true], 1008u64, 504u64),
        ([false, true, true, true, false], 1008u64, 504u64),
        ([false, true, true, true, true], 1017u64, 576u64),
        ([true, false, false, false, false], 1017u64, 441u64),
        ([true, false, false, false, true], 1008u64, 504u64),
        ([true, false, false, true, false], 1008u64, 504u64),
        ([true, false, false, true, true], 1017u64, 576u64),
        ([true, false, true, false, false], 1248u64, 504u64),
        ([true, false, true, false, true], 1227u64, 651u64),
        ([true, false, true, true, false], 1227u64, 576u64),
        ([true, false, true, true, true], 1248u64, 744u64),
        ([true, true, false, false, false], 1248u64, 504u64),
        ([true, true, false, false, true], 1227u64, 576u64),
        ([true, true, false, true, false], 1227u64, 651u64),
        ([true, true, false, true, true], 1248u64, 744u64),
        ([true, true, true, false, false], 1537u64, 576u64),
        ([true, true, true, false, true], 1488u64, 744u64),
        ([true, true, true, true, false], 1488u64, 744u64),
        ([true, true, true, true, true], 1537u64, 961u64),
    ];
    assert_eq!(dist.histories.len(), expected.len());
    for (h, (key, total, truth)) in dist.histories.iter().zip(expected) {
        assert_eq!(
            [
                h.observation.calibration_truth,
                h.observation.calibration_reports[0],
                h.observation.calibration_reports[1],
                h.observation.live_reports[0],
                h.observation.live_reports[1]
            ],
            key
        );
        assert_eq!((h.total_mass, h.true_mass), (total, truth));
    }
    for (policy, payoff) in [
        (Listener::Bayesian, 2300),
        (Listener::Credulous, 2000),
        (Listener::Skeptical, 2000),
        (Listener::Passive, 0),
        (Listener::DirectEvidence, 0),
    ] {
        let e = evaluate(&dist, &policy).unwrap();
        assert_eq!(e.payoff_numerator, payoff);
        assert_eq!(e.optimal_numerator, 2300);
        assert_eq!(e.regret_numerator, 2300 - payoff);
        if policy == Listener::Bayesian {
            assert!(e.posterior_max_error.unwrap() < 1e-12);
            assert_eq!(e.regret_numerator, 0);
        }
    }
    let c = Config::standard(probability(1, 2), probability(3, 4));
    let dist = enumerate(&c).unwrap();
    assert_eq!(dist.denominator, 1024);
    let expected = [
        ([false, false, false, false, false], 32u64, 16u64),
        ([false, false, false, false, true], 32u64, 16u64),
        ([false, false, false, true, false], 32u64, 16u64),
        ([false, false, false, true, true], 32u64, 16u64),
        ([false, false, true, false, false], 32u64, 16u64),
        ([false, false, true, false, true], 32u64, 16u64),
        ([false, false, true, true, false], 32u64, 16u64),
        ([false, false, true, true, true], 32u64, 16u64),
        ([false, true, false, false, false], 32u64, 16u64),
        ([false, true, false, false, true], 32u64, 16u64),
        ([false, true, false, true, false], 32u64, 16u64),
        ([false, true, false, true, true], 32u64, 16u64),
        ([false, true, true, false, false], 32u64, 16u64),
        ([false, true, true, false, true], 32u64, 16u64),
        ([false, true, true, true, false], 32u64, 16u64),
        ([false, true, true, true, true], 32u64, 16u64),
        ([true, false, false, false, false], 32u64, 16u64),
        ([true, false, false, false, true], 32u64, 16u64),
        ([true, false, false, true, false], 32u64, 16u64),
        ([true, false, false, true, true], 32u64, 16u64),
        ([true, false, true, false, false], 32u64, 16u64),
        ([true, false, true, false, true], 32u64, 16u64),
        ([true, false, true, true, false], 32u64, 16u64),
        ([true, false, true, true, true], 32u64, 16u64),
        ([true, true, false, false, false], 32u64, 16u64),
        ([true, true, false, false, true], 32u64, 16u64),
        ([true, true, false, true, false], 32u64, 16u64),
        ([true, true, false, true, true], 32u64, 16u64),
        ([true, true, true, false, false], 32u64, 16u64),
        ([true, true, true, false, true], 32u64, 16u64),
        ([true, true, true, true, false], 32u64, 16u64),
        ([true, true, true, true, true], 32u64, 16u64),
    ];
    assert_eq!(dist.histories.len(), expected.len());
    for (h, (key, total, truth)) in dist.histories.iter().zip(expected) {
        assert_eq!(
            [
                h.observation.calibration_truth,
                h.observation.calibration_reports[0],
                h.observation.calibration_reports[1],
                h.observation.live_reports[0],
                h.observation.live_reports[1]
            ],
            key
        );
        assert_eq!((h.total_mass, h.true_mass), (total, truth));
    }
    for (policy, payoff) in [
        (Listener::Bayesian, 0),
        (Listener::Credulous, 0),
        (Listener::Skeptical, 0),
        (Listener::Passive, 0),
        (Listener::DirectEvidence, 0),
    ] {
        let e = evaluate(&dist, &policy).unwrap();
        assert_eq!(e.payoff_numerator, payoff);
        assert_eq!(e.optimal_numerator, 0);
        assert_eq!(e.regret_numerator, 0 - payoff);
        if policy == Listener::Bayesian {
            assert!(e.posterior_max_error.unwrap() < 1e-12);
            assert_eq!(e.regret_numerator, 0);
        }
    }
}

#[test]
fn enumeration_endpoints_normalize_and_invalid_distributions_reject() {
    for q in [probability(0, 1), probability(1, 1), probability(7, 16)] {
        for rho in [probability(0, 1), probability(1, 1), probability(9, 16)] {
            let d = enumerate(&Config::standard(q.clone(), rho)).unwrap();
            assert_eq!(
                d.histories.iter().map(|h| h.total_mass).sum::<u64>(),
                d.denominator
            );
            assert_eq!(
                d.histories.iter().map(|h| h.true_mass).sum::<u64>(),
                d.denominator / 2
            );
            let e = evaluate(&d, &Listener::Bayesian).unwrap();
            assert_eq!(e.regret_numerator, 0);
        }
    }
    let good = enumerate(&config()).unwrap();
    let mut bads = Vec::new();
    let mut d = good.clone();
    d.denominator = 0;
    bads.push(d);
    let mut d = good.clone();
    d.histories[0].total_mass = 0;
    bads.push(d);
    let mut d = good.clone();
    d.histories[0].true_mass = d.histories[0].total_mass + 1;
    bads.push(d);
    let mut d = good.clone();
    d.histories.push(d.histories[0].clone());
    bads.push(d);
    let mut d = good.clone();
    d.histories[0].observation.rules.accuracy = probability(1, 0);
    bads.push(d);
    let mut d = good.clone();
    d.histories[0].total_mass = u64::MAX;
    bads.push(d);
    let mut d = good.clone();
    d.denominator += 1;
    bads.push(d);
    for d in bads {
        assert!(matches!(
            evaluate(&d, &Listener::Passive),
            Err(Error::InvalidConfig(_))
        ));
    }
}

fn decision_view(
    q: Probability,
    rho: Probability,
    cal: [bool; 2],
    live: [bool; 2],
) -> DecisionObservation {
    DecisionObservation {
        rules: Config::standard(q, rho),
        calibration_truth: true,
        calibration_reports: cal,
        live_reports: live,
    }
}
#[test]
fn calibration_updates_bayesian_but_not_fixed_channel_policies() {
    let a = decision_view(
        probability(4, 5),
        probability(3, 4),
        [true, true],
        [true, false],
    );
    let b = DecisionObservation {
        calibration_reports: [false, true],
        ..a.clone()
    };
    assert_eq!(
        Listener::Bayesian.decide(&a).unwrap().action,
        DecisionAction::Abstain
    );
    assert_eq!(
        Listener::Bayesian.decide(&b).unwrap().action,
        DecisionAction::Abstain
    );
    let c = DecisionObservation {
        calibration_reports: [true, false],
        ..a.clone()
    };
    assert_eq!(
        Listener::Bayesian.decide(&c).unwrap().action,
        DecisionAction::Intervene
    );
    for p in [Listener::Credulous, Listener::Skeptical] {
        assert_eq!(p.decide(&a).unwrap(), p.decide(&c).unwrap());
    }
}
#[test]
fn deterministic_assumed_channels_reject_impossible_histories() {
    let view = decision_view(
        probability(1, 1),
        probability(1, 1),
        [true, true],
        [true, false],
    );
    for p in [Listener::Bayesian, Listener::Credulous, Listener::Skeptical] {
        assert!(matches!(p.decide(&view), Err(Error::Inference(_))));
    }
    let known_invert = decision_view(
        probability(4, 5),
        probability(0, 1),
        [false, false],
        [false, false],
    );
    assert_eq!(
        Listener::Bayesian.decide(&known_invert).unwrap().action,
        DecisionAction::Intervene
    );
    assert_eq!(
        Listener::Skeptical.decide(&known_invert).unwrap().action,
        DecisionAction::Intervene
    );
}
#[test]
fn evolved_genome_bounds_endpoints_ties_and_score_privacy() {
    let g = Genome {
        b: 0,
        u: 6,
        d: 6,
        k: 0,
    };
    for bad in [
        Genome { b: 17, ..g.clone() },
        Genome { u: -1, ..g.clone() },
        Genome { d: 17, ..g.clone() },
        Genome {
            k: -17,
            ..g.clone()
        },
    ] {
        assert_eq!(bad.validate(), Err(Error::InvalidGenome));
    }
    for (rho, cal, live, want) in [
        (
            probability(1, 1),
            [false, false],
            [true, true],
            DecisionAction::Intervene,
        ),
        (
            probability(0, 1),
            [true, true],
            [true, true],
            DecisionAction::Abstain,
        ),
        (
            probability(1, 2),
            [true, false],
            [true, false],
            DecisionAction::Intervene,
        ),
        (
            probability(1, 2),
            [false, true],
            [true, false],
            DecisionAction::Abstain,
        ),
    ] {
        let v = decision_view(probability(4, 5), rho, cal, live);
        let result = Listener::Evolved(g.clone()).decide(&v).unwrap();
        assert_eq!(result.action, want);
        assert_eq!(result.posterior_true, None);
    }
    let v = decision_view(
        probability(1, 2),
        probability(3, 4),
        [true, true],
        [true, true],
    );
    assert_eq!(
        Listener::Evolved(g.clone()).decide(&v).unwrap().action,
        DecisionAction::Abstain
    );
    let dist = enumerate(&config()).unwrap();
    let e = evaluate(&dist, &Listener::Evolved(g.clone())).unwrap();
    assert_eq!(e.posterior_max_error, None);
    assert_eq!(e.brier, None);
    assert_eq!(fitness(&dist, &g).unwrap(), e.payoff_numerator);
    let mut renamed = v.clone();
    renamed.rules.reporters = [20, 99];
    renamed.rules.decider = 77;
    for (p, id) in renamed.rules.permissions.iter_mut().zip([20, 99, 77]) {
        p.agent = id;
    }
    for p in [Listener::Bayesian, Listener::Evolved(g)] {
        assert_eq!(p.decide(&v).unwrap(), p.decide(&renamed).unwrap());
    }
}

#[test]
fn posterior_metrics_and_intervention_accounting_have_independent_references() {
    let d = enumerate(&config()).unwrap();
    let e = evaluate(&d, &Listener::Credulous).unwrap();
    assert_eq!(e.intervention_mass, 10900);
    assert_eq!(e.correct_intervention_mass, 8450);
    assert_eq!(e.incorrect_intervention_mass, 2450);
    assert!((e.posterior_squared_error.unwrap() - 850180627143.0 / 17995969915744.0).abs() < 1e-12);
    assert!((e.brier.unwrap() - 2069.0 / 9248.0).abs() < 1e-12);
    let direct = evaluate(&d, &Listener::DirectEvidence).unwrap();
    assert!(
        (direct.posterior_squared_error.unwrap() - 9728248977.0 / 132323308204.0).abs() < 1e-12
    );
    assert_eq!(direct.brier, Some(0.25));
    let bayesian = evaluate(&d, &Listener::Bayesian).unwrap();
    assert!((bayesian.brier.unwrap() - 11676289037.0 / 66161654102.0).abs() < 1e-12);
    assert!(bayesian
        .histories
        .iter()
        .all(|h| h.conditional_regret == 0.0));
    let bad = evaluate(
        &enumerate(&Config::standard(probability(4, 5), probability(1, 4))).unwrap(),
        &Listener::Credulous,
    )
    .unwrap();
    assert!(bad.histories.iter().any(|h| h.conditional_regret > 0.0));
    assert_eq!(bad.regret_numerator, 15120);
    let none = evaluate(&d, &Listener::Passive).unwrap();
    assert_eq!(none.posterior_squared_error, None);
    assert_eq!(none.intervention_mass, 0);
}
#[test]
fn genome_decoding_uses_quarter_trust_and_eighth_threshold_with_strict_ties() {
    let v = decision_view(
        probability(3, 4),
        probability(1, 1),
        [false, false],
        [true, true],
    );
    for (k, want) in [
        (7, DecisionAction::Intervene),
        (8, DecisionAction::Abstain),
        (9, DecisionAction::Abstain),
    ] {
        assert_eq!(
            Listener::Evolved(Genome {
                b: 16,
                u: 16,
                d: 16,
                k
            })
            .decide(&v)
            .unwrap()
            .action,
            want
        );
    }
    let v = decision_view(
        probability(1, 1),
        probability(1, 2),
        [true, false],
        [true, false],
    );
    // sigmoid(1)-sigmoid(-1) gives score 0.924234..., between 7/8 and 8/8.
    assert_eq!(
        Listener::Evolved(Genome {
            b: 0,
            u: 4,
            d: 4,
            k: 7
        })
        .decide(&v)
        .unwrap()
        .action,
        DecisionAction::Intervene
    );
    assert_eq!(
        Listener::Evolved(Genome {
            b: 0,
            u: 4,
            d: 4,
            k: 8
        })
        .decide(&v)
        .unwrap()
        .action,
        DecisionAction::Abstain
    );
    let v = decision_view(
        probability(1, 1),
        probability(1, 2),
        [true, true],
        [true, true],
    );
    assert_eq!(
        Listener::Evolved(Genome {
            b: 4,
            u: 0,
            d: 0,
            k: 7
        })
        .decide(&v)
        .unwrap()
        .action,
        DecisionAction::Intervene
    );
    assert_eq!(
        Listener::Evolved(Genome {
            b: 4,
            u: 0,
            d: 0,
            k: 8
        })
        .decide(&v)
        .unwrap()
        .action,
        DecisionAction::Abstain
    );
}
#[test]
fn uninformative_channel_abstains_and_has_no_attainable_gain() {
    let d = enumerate(&Config::standard(probability(1, 2), probability(3, 4))).unwrap();
    for p in [
        Listener::Bayesian,
        Listener::Credulous,
        Listener::Skeptical,
        Listener::DirectEvidence,
        Listener::Passive,
    ] {
        let e = evaluate(&d, &p).unwrap();
        assert_eq!(e.optimal_numerator, 0);
        assert_eq!(e.payoff_numerator, 0);
        assert_eq!(e.intervention_mass, 0);
    }
}
#[test]
fn listener_wire_unit_variants_reject_extra_fields() {
    for kind in [
        "bayesian",
        "credulous",
        "skeptical",
        "direct_evidence",
        "passive",
    ] {
        assert!(
            serde_json::from_value::<Listener>(serde_json::json!({"kind":kind,"hidden":true}))
                .is_err()
        );
    }
}

#[test]
fn seeded_draw_schedule_matches_independent_literal_rng_fixture() {
    // Standalone RNG probe, without Session/report code, pinned these version-1 draws.
    for (seed, rho, profiles, signals, c, t) in [
        (
            0,
            probability(3, 4),
            [Profile::Copy, Profile::Copy],
            [false, true, false, false],
            true,
            false,
        ),
        (
            2,
            probability(3, 4),
            [Profile::Copy, Profile::Invert],
            [false, false, true, true],
            false,
            true,
        ),
        (
            0,
            probability(1, 1),
            [Profile::Copy, Profile::Copy],
            [false, true, false, false],
            true,
            false,
        ),
    ] {
        let mut game = Session::new(Config::standard(probability(4, 5), rho), seed).unwrap();
        for (i, signal) in signals.into_iter().enumerate() {
            let Observation::Reporter { view } = game.request().unwrap().observation else {
                panic!()
            };
            assert_eq!(view.signal, signal);
            assert_eq!(view.profile, profiles[i % 2]);
            drive_report(&mut game);
        }
        let Observation::Decider { view } = game.request().unwrap().observation else {
            panic!()
        };
        assert_eq!(view.calibration_truth, c);
        respond(&mut game, Action::Intervene);
        assert_eq!(game.outcome().unwrap().live_truth, t);
    }
}

#[test]
fn bayesian_exact_ties_abstain_and_preserve_independent_intervention_masses() {
    // Independent Fraction references, including ties whose floating inference was above 1/2.
    for (q, rho, intervention, correct, incorrect) in [
        (probability(4, 5), probability(3, 4), 15648, 12384, 3264),
        (probability(4, 5), probability(1, 4), 15648, 12384, 3264),
        (probability(3, 5), probability(3, 4), 15008, 8654, 6354),
        (probability(1, 2), probability(3, 4), 0, 0, 0),
    ] {
        let d = enumerate(&Config::standard(q, rho)).unwrap();
        let e = evaluate(&d, &Listener::Bayesian).unwrap();
        for h in &e.histories {
            if 2 * h.true_mass == h.total_mass {
                assert_eq!(
                    h.decision.action,
                    DecisionAction::Abstain,
                    "exact tie: {:?}",
                    h.observation
                );
                assert_eq!(h.decision.posterior_true, Some(0.5));
            } else {
                let p = h.decision.posterior_true.unwrap();
                assert!((p - 0.5).abs() > 1e-12);
                assert_eq!(
                    h.decision.action,
                    if 2 * h.true_mass > h.total_mass {
                        DecisionAction::Intervene
                    } else {
                        DecisionAction::Abstain
                    }
                );
            }
        }
        assert_eq!(e.intervention_mass, intervention);
        assert_eq!(e.correct_intervention_mass, correct);
        assert_eq!(e.incorrect_intervention_mass, incorrect);
    }
}
#[test]
fn bayesian_normalization_preserves_genuine_non_ties_at_maximum_denominators() {
    let d = enumerate(&Config::standard(probability(9, 16), probability(9, 16))).unwrap();
    let e = evaluate(&d, &Listener::Bayesian).unwrap();
    assert!(e.histories.iter().any(|h| 2 * h.true_mass != h.total_mass));
    for h in e.histories {
        if 2 * h.true_mass != h.total_mass {
            assert!((h.decision.posterior_true.unwrap() - 0.5).abs() > 1e-12);
            assert_eq!(
                h.decision.action,
                if 2 * h.true_mass > h.total_mass {
                    DecisionAction::Intervene
                } else {
                    DecisionAction::Abstain
                }
            );
        }
    }
}

#[test]
fn frozen_diagnostic_has_complete_collection() {
    let r = diagnose_testimony_game().unwrap();
    assert_eq!(r.version, "testimony-game-v1");
    assert!(r.passed);
    assert_eq!(r.environments.len(), 4);
    assert_eq!(r.fixed_evaluations.len(), 20);
    assert_eq!(r.runs.len(), 40);
    assert_eq!(r.frozen_evaluations.len(), 160);
    assert_eq!(r.paired_differences.len(), 80);
    assert_eq!(r.summaries.len(), 8);
    assert!(r.runs.iter().all(|run| run.evaluations == 3164));
    assert!(r.checks.iter().all(|c| c.passed));
    assert!(r.integrity_checks.iter().all(|c| c.passed));
    assert!(r.summaries.iter().all(|s| s.count == 20));
    for env in &r.environments {
        assert_eq!(env.distribution.histories.len(), 32);
        assert_eq!(
            r.fixed_evaluations
                .iter()
                .filter(|e| e.environment == env.name)
                .count(),
            5
        );
        assert_eq!(
            r.frozen_evaluations
                .iter()
                .filter(|e| e.environment == env.name)
                .count(),
            40
        );
        assert_eq!(
            r.paired_differences
                .iter()
                .filter(|e| e.environment == env.name)
                .count(),
            20
        );
    }
    for e in &r.frozen_evaluations {
        assert_eq!(e.evaluation.histories.len(), 32);
        assert!(e.evaluation.brier.is_none());
        let method = if e.policy == "genetic" {
            SearchMethod::Genetic
        } else {
            SearchMethod::Random
        };
        let run = r
            .runs
            .iter()
            .find(|run| run.seed == e.seed.unwrap() && run.method == method)
            .unwrap();
        assert_eq!(e.genome.as_ref(), Some(&run.champion));
    }
}
