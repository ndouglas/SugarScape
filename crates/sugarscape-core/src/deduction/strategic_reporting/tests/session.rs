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
fn response(request: &Request) -> Response {
    let action = match &request.observation {
        Observation::Strategic { view } => Action::Report {
            positive: Policy::copy().report(view).unwrap(),
        },
        Observation::Fixed { view } => Action::Report {
            positive: crate::deduction::testimony_game::report(view.profile, view.signal),
        },
        Observation::Decider { .. } => Action::Intervene,
    };
    Response {
        request_id: request.request_id,
        actor: request.actor,
        action,
    }
}
fn advance(s: &mut Session) {
    let r = s.request().unwrap();
    s.submit(response(&r)).unwrap();
}
#[test]
fn phase_redaction_buffering_and_five_response_schedule() {
    let mut s = Session::new(config(), 0).unwrap();
    let mut calibration_signal = None;
    for index in 0..5 {
        assert!(s.outcome().is_none());
        let r = s.request().unwrap();
        assert_eq!(r.request_id, index + 1);
        assert_eq!(r.protocol_version, 1);
        assert_eq!(r.actor, [0, 1, 0, 1, 2][index as usize]);
        match &r.observation {
            Observation::Strategic { view } => {
                assert_eq!(
                    view.calibration_signal,
                    if index == 0 { None } else { calibration_signal }
                );
                if index == 0 {
                    calibration_signal = Some(view.signal);
                }
                assert_eq!(view.calibration_truth.is_some(), index >= 2);
                assert_eq!(view.calibration_reports.is_some(), index >= 2);
                assert!(!serde_json::to_value(view)
                    .unwrap()
                    .as_object()
                    .unwrap()
                    .contains_key("profile"));
            }
            Observation::Fixed { view } => {
                assert_eq!(view.calibration_truth.is_some(), index >= 2);
                assert_eq!(view.calibration_reports.is_some(), index >= 2);
            }
            Observation::Decider { view } => {
                assert!(view.calibration_truth);
                assert_eq!(view.calibration_reports, [true, false]);
                assert_eq!(view.live_reports, [false, false]);
            }
        }
        let json = serde_json::to_value(&r).unwrap();
        let fields = json["observation"]["view"].as_object().unwrap();
        for key in ["seed", "live_truth", "truths", "signals"] {
            assert!(!fields.contains_key(key));
        }
        if index < 4 {
            assert!(!fields.contains_key("live_reports"));
            assert_eq!(r.legal, vec![LegalAction::Report]);
        } else {
            assert_eq!(r.legal, vec![LegalAction::Intervene, LegalAction::Abstain]);
        }
        advance(&mut s);
    }
    assert!(s.request().is_none());
    let o = s.outcome().unwrap();
    assert!(!o.live_truth);
    assert_eq!(o.payoff, -1);
    assert_eq!(o.reporter_utility, 1);
    assert!(s
        .submit(Response {
            request_id: 6,
            actor: 2,
            action: Action::Abstain
        })
        .is_err());
}
#[test]
fn literal_rng_fixtures_preserve_endpoint_draws() {
    // Standalone rng probe, without session/report code, retained task-4-rng-probe.rs/.txt.
    for (seed, qn, qd, rn, rd, c, t, profile, signals) in [
        (
            0,
            4,
            5,
            3,
            4,
            true,
            false,
            Profile::Copy,
            [true, false, false, false],
        ),
        (
            2,
            4,
            5,
            3,
            4,
            false,
            true,
            Profile::Copy,
            [true, false, true, true],
        ),
        (
            0,
            4,
            5,
            1,
            1,
            true,
            false,
            Profile::Copy,
            [true, false, false, false],
        ),
        (
            0,
            4,
            5,
            0,
            1,
            true,
            false,
            Profile::Invert,
            [true, false, false, false],
        ),
        (
            7,
            1,
            1,
            1,
            1,
            false,
            true,
            Profile::Copy,
            [false, false, true, true],
        ),
        (
            7,
            0,
            1,
            0,
            1,
            false,
            true,
            Profile::Invert,
            [true, true, false, false],
        ),
    ] {
        let mut rules = config();
        rules.accuracy = Probability {
            numerator: qn,
            denominator: qd,
        };
        rules.fixed_copy_prior = Probability {
            numerator: rn,
            denominator: rd,
        };
        let mut s = Session::new(rules, seed).unwrap();
        for (index, signal) in signals.into_iter().enumerate() {
            match s.request().unwrap().observation {
                Observation::Strategic { view } => assert_eq!(view.signal, signal),
                Observation::Fixed { view } => {
                    assert_eq!(view.signal, signal);
                    assert_eq!(view.profile, profile);
                }
                _ => panic!("report request expected"),
            };
            if index >= 2 {
                match s.request().unwrap().observation {
                    Observation::Strategic { view } => assert_eq!(view.calibration_truth, Some(c)),
                    Observation::Fixed { view } => assert_eq!(view.calibration_truth, Some(c)),
                    _ => panic!(),
                }
            }
            advance(&mut s);
        }
        advance(&mut s);
        assert_eq!(s.outcome().unwrap().live_truth, t);
    }
}
#[test]
fn rejected_responses_preserve_entire_future() {
    for count in 0..5 {
        for corruption in 0..3 {
            let mut baseline = Session::new(config(), 7).unwrap();
            for _ in 0..count {
                advance(&mut baseline);
            }
            let mut rejected = Session::replay(&baseline.archive()).unwrap();
            let r = rejected.request().unwrap();
            let mut bad = response(&r);
            match corruption {
                0 => bad.actor = 99,
                1 => bad.request_id = 0,
                _ => {
                    bad.action = if count < 4 {
                        Action::Abstain
                    } else {
                        Action::Report { positive: true }
                    }
                }
            };
            let checkpoint = rejected.archive();
            assert!(rejected.submit(bad).is_err());
            assert_eq!(rejected.archive(), checkpoint);
            loop {
                assert_eq!(rejected.request(), baseline.request());
                if baseline.request().is_none() {
                    break;
                }
                advance(&mut baseline);
                advance(&mut rejected);
            }
            assert_eq!(rejected.outcome(), baseline.outcome());
            assert_eq!(rejected.archive(), baseline.archive());
        }
    }
}
#[test]
fn replay_accepts_every_partial_prefix_and_rejects_tampering() {
    let mut s = Session::new(config(), 7).unwrap();
    for count in 0..=5 {
        let a = s.archive();
        let decoded: Archive = serde_json::from_slice(&serde_json::to_vec(&a).unwrap()).unwrap();
        assert_eq!(Session::replay(&decoded).unwrap().archive(), a);
        let mut bad = a.clone();
        bad.protocol_version = 2;
        assert!(matches!(Session::replay(&bad), Err(Error::VersionMismatch)));
        bad = a.clone();
        bad.config.version = 2;
        assert!(matches!(Session::replay(&bad), Err(Error::VersionMismatch)));
        bad = a.clone();
        if let Some(request) = bad.checkpoint.request.as_mut() {
            request.request_id += 1;
        } else {
            bad.checkpoint.outcome.as_mut().unwrap().reporter_utility = 42;
        }
        assert!(Session::replay(&bad).is_err());
        if count > 0 {
            bad = a;
            bad.responses[0].actor = 99;
            assert!(matches!(
                Session::replay(&bad),
                Err(Error::InvalidArchive { index: Some(0) })
            ));
        }
        if count < 5 {
            advance(&mut s);
        }
    }
}
#[test]
fn strict_protocol_decoding_including_fieldless_actions() {
    fn unknown<T: serde::Serialize + serde::de::DeserializeOwned>(v: T) {
        let mut j = serde_json::to_value(v).unwrap();
        j.as_object_mut()
            .unwrap()
            .insert("unknown".into(), true.into());
        assert!(serde_json::from_value::<T>(j).is_err());
    }
    let mut s = Session::new(config(), 0).unwrap();
    unknown(s.request().unwrap());
    unknown(response(&s.request().unwrap()));
    unknown(s.archive());
    unknown(s.archive().checkpoint);
    for _ in 0..5 {
        advance(&mut s);
    }
    unknown(s.outcome().unwrap());
    for wire in [
        r#"{"kind":"intervene","unknown":true}"#,
        r#"{"kind":"abstain","unknown":true}"#,
        r#"{"kind":"report","positive":true,"unknown":true}"#,
    ] {
        assert!(serde_json::from_str::<Action>(wire).is_err());
    }
    let mut r = Session::new(config(), 0).unwrap().request().unwrap();
    for kind in ["strategic", "fixed", "decider"] {
        let mut j = serde_json::to_value(&r.observation).unwrap();
        j["kind"] = kind.into();
        j["unknown"] = true.into();
        assert!(serde_json::from_value::<Observation>(j).is_err());
    }
    r.protocol_version = 2;
    unknown(r);
}
#[test]
fn renamed_ids_and_invalid_config() {
    let mut c = config();
    c.strategic = 8;
    c.fixed = 13;
    c.decider = 21;
    for (p, id) in c.permissions.iter_mut().zip([8, 13, 21]) {
        p.agent = id;
    }
    let mut s = Session::new(c.clone(), 7).unwrap();
    for id in [8, 13, 8, 13, 21] {
        assert_eq!(s.request().unwrap().actor, id);
        advance(&mut s);
    }
    c.permissions[0].receive_signal = false;
    assert!(Session::new(c, 7).is_err());
}
#[test]
fn numerical_integrity_is_search_free_and_reproducible() {
    let a = numerical_integrity().unwrap();
    let b = numerical_integrity().unwrap();
    assert_eq!(a, b);
    assert!(a.iter().all(|c| c.passed));
    assert!(a.len() > 1000);
}
