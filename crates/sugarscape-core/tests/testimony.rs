use sugarscape_core::deduction::*;

fn binary_model(channel: [f64; 2], accuracy: f64) -> TestimonyModel {
    TestimonyModel {
        propositions: vec![Proposition { id: 7, label: None }],
        speakers: vec![19],
        profiles: vec![ReportingProfile {
            id: 3,
            positive_given_signal: channel,
        }],
        hypotheses: [false, true]
            .into_iter()
            .enumerate()
            .map(|(i, value)| TestimonyHypothesis {
                id: [4, 9][i],
                prior: 0.5,
                propositions: vec![PropositionValue {
                    proposition: 7,
                    value,
                }],
                profiles: vec![SpeakerProfile {
                    speaker: 19,
                    profile: 3,
                }],
            })
            .collect(),
        groups: [11, 12]
            .into_iter()
            .map(|id| SignalGroup {
                id,
                proposition: 7,
                accuracy,
            })
            .collect(),
    }
}

#[test]
fn testimony_empty_history_returns_normalized_prior() {
    let s = Belief::new(binary_model([0.0, 1.0], 0.8))
        .unwrap()
        .snapshot();
    assert_eq!(s.evidence_count, 0);
    assert_eq!(
        s.hypotheses.iter().map(|h| h.id).collect::<Vec<_>>(),
        vec![4, 9]
    );
    assert!((s.propositions[0].probability_true - 0.5).abs() < 1e-12);
    assert_eq!(s.speakers[0].profiles[0].probability, 1.0);
}

#[test]
fn testimony_rejects_invalid_models() {
    let base = binary_model([0.0, 1.0], 0.8);
    let mut cases = vec![];
    macro_rules! bad {
        ($edit:expr) => {{
            let mut m = base.clone();
            $edit(&mut m);
            cases.push(m);
        }};
    }
    for p in [f64::NAN, f64::INFINITY, -0.1, 1.1] {
        bad!(|m: &mut TestimonyModel| m.hypotheses[0].prior = p);
        bad!(|m: &mut TestimonyModel| m.profiles[0].positive_given_signal[0] = p);
        bad!(|m: &mut TestimonyModel| m.groups[0].accuracy = p);
    }
    bad!(|m: &mut TestimonyModel| m.hypotheses.iter_mut().for_each(|h| h.prior = 0.0));
    bad!(|m: &mut TestimonyModel| m.hypotheses[0].prior = 0.4);
    bad!(|m: &mut TestimonyModel| m.propositions.push(m.propositions[0].clone()));
    bad!(|m: &mut TestimonyModel| m.speakers.push(19));
    bad!(|m: &mut TestimonyModel| m.profiles.push(m.profiles[0].clone()));
    bad!(|m: &mut TestimonyModel| m.hypotheses.push(m.hypotheses[0].clone()));
    bad!(|m: &mut TestimonyModel| m.groups.push(m.groups[0].clone()));
    bad!(|m: &mut TestimonyModel| m.hypotheses[0].propositions.clear());
    bad!(|m: &mut TestimonyModel| m.hypotheses[0].profiles.clear());
    bad!(|m: &mut TestimonyModel| {
        let value = m.hypotheses[0].propositions[0].clone();
        m.hypotheses[0].propositions.push(value);
    });
    bad!(|m: &mut TestimonyModel| {
        let value = m.hypotheses[0].profiles[0].clone();
        m.hypotheses[0].profiles.push(value);
    });
    bad!(|m: &mut TestimonyModel| m.hypotheses[0].propositions[0].proposition = 99);
    bad!(|m: &mut TestimonyModel| m.hypotheses[0].profiles[0].speaker = 99);
    bad!(|m: &mut TestimonyModel| m.hypotheses[0].profiles[0].profile = 99);
    bad!(|m: &mut TestimonyModel| m.groups[0].proposition = 99);
    bad!(|m: &mut TestimonyModel| {
        m.hypotheses[0].prior = 0.0;
        m.hypotheses[1].prior = 1.0;
        m.hypotheses[0].profiles.clear();
    });
    bad!(|m: &mut TestimonyModel| {
        m.hypotheses[0].prior = 0.0;
        m.hypotheses[1].prior = 1.0;
        m.hypotheses[0].profiles[0].profile = 99;
    });
    for m in cases {
        assert!(matches!(
            Belief::new(m),
            Err(TestimonyError::InvalidModel(_))
        ));
    }
}

#[test]
fn testimony_checks_all_count_bounds() {
    for (field, max) in [
        ("propositions", 16),
        ("speakers", 32),
        ("profiles", 16),
        ("hypotheses", 256),
        ("groups", 256),
    ] {
        for count in [0, max + 1] {
            if field == "groups" && count == 0 {
                continue;
            }
            let mut value = serde_json::to_value(binary_model([0.0, 1.0], 0.8)).unwrap();
            let original = value[field][0].clone();
            value[field] = serde_json::Value::Array(vec![original; count]);
            let m = serde_json::from_value(value).unwrap();
            let Err(TestimonyError::InvalidModel(errors)) = Belief::new(m) else {
                panic!("expected invalid {field} count {count}");
            };
            assert!(errors.iter().any(|error| error.field == field && error.message.starts_with("count must be")), "{field} count {count}");
        }
    }
}

#[test]
fn testimony_accepts_zero_prior_zero_groups_and_sparse_ids() {
    let mut m = binary_model([0.0, 1.0], 0.8);
    m.groups.clear();
    m.hypotheses[0].prior = 0.0;
    m.hypotheses[1].prior = 1.0 + 5e-13;
    assert!(Belief::new(m.clone()).is_err());
    m.hypotheses[1].prior = 1.0;
    m.hypotheses[1].id = u16::MAX;
    let s = Belief::new(m).unwrap().snapshot();
    assert_eq!(s.hypotheses[0].probability, 0.0);
    assert_eq!(s.hypotheses[1].id, u16::MAX);
}

#[test]
fn testimony_json_rejects_unknown_fields_and_wrong_channels() {
    let base = serde_json::to_value(binary_model([0.0, 1.0], 0.8)).unwrap();
    for path in [
        vec![],
        vec!["propositions", "0"],
        vec!["profiles", "0"],
        vec!["hypotheses", "0"],
        vec!["hypotheses", "0", "propositions", "0"],
        vec!["hypotheses", "0", "profiles", "0"],
        vec!["groups", "0"],
    ] {
        let mut value = base.clone();
        let mut target = &mut value;
        for key in path {
            target = if key == "0" {
                &mut target[0]
            } else {
                &mut target[key]
            };
        }
        target
            .as_object_mut()
            .unwrap()
            .insert("unknown".into(), true.into());
        assert!(serde_json::from_value::<TestimonyModel>(value).is_err());
    }
    let mut value = base;
    value["profiles"][0]["positive_given_signal"] = serde_json::json!([0.0]);
    assert!(serde_json::from_value::<TestimonyModel>(value).is_err());
    for json in [
        r#"{"id":1,"content":{"kind":"report","group":11,"speaker":19,"positive":true,"unknown":1}}"#,
        r#"{"id":1,"content":{"kind":"verified","proposition":7,"value":true,"unknown":1}}"#,
        r#"{"id":1,"content":{"kind":"verified","proposition":7,"value":true},"unknown":1}"#,
    ] {
        assert!(serde_json::from_str::<EvidenceRecord>(json).is_err());
    }
}

#[test]
fn testimony_accepts_all_maximum_counts_and_preserves_declared_order() {
    let m = TestimonyModel {
        propositions: (0..16)
            .rev()
            .map(|id| Proposition { id, label: None })
            .collect(),
        speakers: (1000..1032).rev().collect(),
        profiles: (0..16)
            .rev()
            .map(|id| ReportingProfile {
                id,
                positive_given_signal: [0.0, 1.0],
            })
            .collect(),
        hypotheses: (0..256)
            .rev()
            .map(|id| TestimonyHypothesis {
                id,
                prior: 1.0 / 256.0,
                propositions: (0..16)
                    .rev()
                    .map(|proposition| PropositionValue {
                        proposition,
                        value: proposition % 2 == 0,
                    })
                    .collect(),
                profiles: (1000..1032)
                    .rev()
                    .map(|speaker| SpeakerProfile {
                        speaker,
                        profile: 15,
                    })
                    .collect(),
            })
            .collect(),
        groups: (0..256)
            .rev()
            .map(|id| SignalGroup {
                id,
                proposition: 0,
                accuracy: 0.8,
            })
            .collect(),
    };
    let s = Belief::new(m.clone()).unwrap().snapshot();
    assert_eq!(
        s.hypotheses.iter().map(|p| p.id).collect::<Vec<_>>(),
        m.hypotheses.iter().map(|h| h.id).collect::<Vec<_>>()
    );
    assert_eq!(
        s.propositions.iter().map(|p| p.id).collect::<Vec<_>>(),
        m.propositions.iter().map(|p| p.id).collect::<Vec<_>>()
    );
    assert_eq!(
        s.speakers.iter().map(|s| s.speaker).collect::<Vec<_>>(),
        m.speakers
    );
    for speaker in s.speakers {
        assert_eq!(
            speaker.profiles.iter().map(|p| p.id).collect::<Vec<_>>(),
            m.profiles.iter().map(|p| p.id).collect::<Vec<_>>()
        );
        assert_eq!(speaker.profiles[0].probability, 1.0);
        assert!(speaker.profiles[1..].iter().all(|p| p.probability == 0.0));
    }
}

#[test]
fn testimony_normalizes_accepted_near_unit_priors() {
    let mut m = binary_model([0.0, 1.0], 0.8);
    m.hypotheses[0].prior += 5e-13;
    let total = m.hypotheses.iter().map(|h| h.prior).sum::<f64>();
    let expected = m.hypotheses[0].prior / total;
    let s = Belief::new(m).unwrap().snapshot();
    assert_eq!(s.hypotheses[0].probability, expected);
    assert!((s.hypotheses.iter().map(|p| p.probability).sum::<f64>() - 1.0).abs() < 1e-12);
}

#[test]
fn testimony_errors_include_field_context() {
    let mut m = binary_model([0.0, 1.0], 0.8);
    m.groups[0].accuracy = f64::NAN;
    let error = Belief::new(m).unwrap_err();
    assert!(error.to_string().contains("groups[0].accuracy"));
}

#[test]
fn testimony_snapshot_json_rejects_unknown_fields() {
    let base = serde_json::to_value(
        Belief::new(binary_model([0.0, 1.0], 0.8))
            .unwrap()
            .snapshot(),
    )
    .unwrap();
    for path in [
        vec![],
        vec!["hypotheses", "0"],
        vec!["propositions", "0"],
        vec!["speakers", "0"],
        vec!["speakers", "0", "profiles", "0"],
    ] {
        let mut value = base.clone();
        let mut target = &mut value;
        for key in path {
            target = if key == "0" {
                &mut target[0]
            } else {
                &mut target[key]
            };
        }
        target
            .as_object_mut()
            .unwrap()
            .insert("unknown".into(), true.into());
        assert!(serde_json::from_value::<BeliefSnapshot>(value).is_err());
    }
}

fn report(id: u64, group: u16, speaker: AgentId, positive: bool) -> EvidenceRecord {
    EvidenceRecord {
        id,
        content: TestimonyEvidence::Report {
            group,
            speaker,
            positive,
        },
    }
}
fn truth(b: &Belief, id: u16) -> f64 {
    b.snapshot()
        .propositions
        .iter()
        .find(|p| p.id == id)
        .unwrap()
        .probability_true
}
fn profile(b: &Belief, speaker: AgentId, id: u16) -> f64 {
    b.snapshot()
        .speakers
        .iter()
        .find(|s| s.speaker == speaker)
        .unwrap()
        .profiles
        .iter()
        .find(|p| p.id == id)
        .unwrap()
        .probability
}
fn close(actual: f64, expected: f64) {
    assert!((actual - expected).abs() < 1e-12, "{actual} != {expected}");
}
fn uncertain_speaker_model() -> TestimonyModel {
    let mut m = binary_model([0.0, 1.0], 0.8);
    m.profiles.push(ReportingProfile {
        id: 8,
        positive_given_signal: [1.0, 0.0],
    });
    m.hypotheses = [false, true]
        .into_iter()
        .flat_map(|value| {
            [3, 8].into_iter().map(move |profile| TestimonyHypothesis {
                id: (if value { 20 } else { 10 }) + profile,
                prior: if profile == 3 { 3.0 / 8.0 } else { 1.0 / 8.0 },
                propositions: vec![PropositionValue {
                    proposition: 7,
                    value,
                }],
                profiles: vec![SpeakerProfile {
                    speaker: 19,
                    profile,
                }],
            })
        })
        .collect();
    m
}
#[test]
fn testimony_persistent_profile_and_shared_signals() {
    let mut b = Belief::new(uncertain_speaker_model()).unwrap();
    b.observe(report(1, 11, 19, true)).unwrap();
    close(truth(&b, 7), 13.0 / 20.0);
    close(profile(&b, 19, 3), 3.0 / 4.0);
    let before = b.clone();
    b.observe(report(1, 11, 19, true)).unwrap();
    assert_eq!(b, before);
    let mut shared = b.clone();
    shared.observe(report(2, 11, 19, true)).unwrap();
    close(truth(&shared, 7), 13.0 / 20.0);
    b.observe(report(2, 12, 19, true)).unwrap();
    close(truth(&b, 7), 49.0 / 68.0);
}
#[test]
fn testimony_known_profiles_and_witness_dependence() {
    for (channel, q, expected) in [
        ([1.0, 0.0], 0.8, 1.0 / 5.0),
        ([1.0, 1.0], 0.8, 0.5),
        ([0.0, 1.0], 0.5, 0.5),
    ] {
        let mut b = Belief::new(binary_model(channel, q)).unwrap();
        b.observe(report(1, 11, 19, true)).unwrap();
        close(truth(&b, 7), expected);
    }
    let mut m = binary_model([0.0, 1.0], 0.8);
    m.speakers.push(23);
    for h in &mut m.hypotheses {
        h.profiles.push(SpeakerProfile {
            speaker: 23,
            profile: 3,
        });
    }
    for (group, expected) in [(11, 4.0 / 5.0), (12, 16.0 / 17.0)] {
        let mut b = Belief::new(m.clone()).unwrap();
        b.observe(report(1, 11, 19, true)).unwrap();
        b.observe(report(2, group, 23, true)).unwrap();
        close(truth(&b, 7), expected);
    }
}
#[test]
fn testimony_verified_truth_transfers_persistent_profile() {
    let mut m = uncertain_speaker_model();
    m.propositions.push(Proposition {
        id: 31,
        label: None,
    });
    m.groups.push(SignalGroup {
        id: 45,
        proposition: 31,
        accuracy: 0.8,
    });
    m.hypotheses = m
        .hypotheses
        .into_iter()
        .flat_map(|h| {
            [false, true].into_iter().map(move |value| {
                let mut h = h.clone();
                h.id = h.id * 2 + u16::from(value);
                h.prior /= 2.0;
                h.propositions.push(PropositionValue {
                    proposition: 31,
                    value,
                });
                h
            })
        })
        .collect();
    let mut b = Belief::new(m).unwrap();
    b.observe(report(1, 11, 19, true)).unwrap();
    b.observe(EvidenceRecord {
        id: 2,
        content: TestimonyEvidence::Verified {
            proposition: 7,
            value: true,
        },
    })
    .unwrap();
    close(profile(&b, 19, 3), 12.0 / 13.0);
    close(truth(&b, 31), 0.5);
    b.observe(report(3, 45, 19, true)).unwrap();
    close(truth(&b, 31), 49.0 / 65.0);
}
#[test]
fn testimony_observation_errors_are_atomic() {
    let mut b = Belief::new(binary_model([0.0, 1.0], 0.8)).unwrap();
    b.observe(report(1, 11, 19, true)).unwrap();
    for (r, error) in [
        (report(2, 999, 19, true), TestimonyError::UnknownGroup(999)),
        (
            report(2, 11, 999, true),
            TestimonyError::UnknownSpeaker(999),
        ),
        (
            EvidenceRecord {
                id: 2,
                content: TestimonyEvidence::Verified {
                    proposition: 999,
                    value: true,
                },
            },
            TestimonyError::UnknownProposition(999),
        ),
        (
            report(1, 12, 19, true),
            TestimonyError::ConflictingEvidence(1),
        ),
        (report(2, 11, 19, false), TestimonyError::ZeroEvidence),
    ] {
        let before = b.clone();
        assert_eq!(b.observe(r), Err(error));
        assert_eq!(b, before);
    }
}
#[test]
fn testimony_capacity_duplicate_and_underflow() {
    for channel in [[1.0, 1.0], [0.001, 0.001]] {
        let mut m = binary_model(channel, 0.8);
        m.hypotheses[0].prior = 0.3;
        m.hypotheses[1].prior = 0.7;
        let mut b = Belief::new(m).unwrap();
        for id in 1..=512 {
            b.observe(report(id, 11, 19, true)).unwrap();
        }
        close(truth(&b, 7), 0.7);
        let before = b.clone();
        b.observe(report(1, 11, 19, true)).unwrap();
        assert_eq!(b, before);
        assert_eq!(
            b.observe(report(513, 11, 19, true)),
            Err(TestimonyError::CapacityExceeded)
        );
        assert_eq!(b, before);
        assert_eq!(
            b.observe(report(1, 11, 19, false)),
            Err(TestimonyError::ConflictingEvidence(1))
        );
        assert_eq!(b, before);
    }
}
#[test]
fn testimony_endpoints_preserve_zero_support() {
    for (q, expected) in [(0.0, 0.0), (1.0, 1.0)] {
        let mut b = Belief::new(binary_model([0.0, 1.0], q)).unwrap();
        b.observe(report(1, 11, 19, true)).unwrap();
        close(truth(&b, 7), expected);
        let before = b.clone();
        assert_eq!(
            b.observe(EvidenceRecord {
                id: 2,
                content: TestimonyEvidence::Verified {
                    proposition: 7,
                    value: expected == 0.0
                }
            }),
            Err(TestimonyError::ZeroEvidence)
        );
        assert_eq!(b, before);
    }
    let mut m = binary_model([0.0, 1.0], 1.0);
    m.hypotheses[0].prior = 1.0;
    m.hypotheses[1].prior = 0.0;
    let mut b = Belief::new(m).unwrap();
    let before = b.clone();
    assert_eq!(
        b.observe(report(1, 11, 19, true)),
        Err(TestimonyError::ZeroEvidence)
    );
    assert_eq!(b, before);
}
#[test]
fn testimony_ids_order_and_projections_are_invariant() {
    let mut m = uncertain_speaker_model();
    m.propositions.push(Proposition {
        id: 32,
        label: None,
    });
    m.speakers.push(29);
    for h in &mut m.hypotheses {
        h.propositions.push(PropositionValue {
            proposition: 32,
            value: false,
        });
        h.profiles.push(SpeakerProfile {
            speaker: 29,
            profile: 3,
        });
    }
    let mut original = Belief::new(m.clone()).unwrap();
    original.observe(report(1, 11, 19, true)).unwrap();
    original.observe(report(2, 12, 19, true)).unwrap();
    m.propositions[0].id = 501;
    m.speakers[0] = 611;
    m.propositions.reverse();
    m.speakers.reverse();
    m.profiles.reverse();
    m.hypotheses.reverse();
    m.groups.reverse();
    for p in &mut m.profiles {
        p.id += 700;
    }
    for h in &mut m.hypotheses {
        h.id += 900;
        h.propositions[0].proposition = 501;
        h.profiles[0].speaker = 611;
        for p in &mut h.profiles {
            p.profile += 700;
        }
        h.propositions.reverse();
        h.profiles.reverse();
    }
    for g in &mut m.groups {
        g.id += 800;
        g.proposition = 501;
    }
    let mut b = Belief::new(m).unwrap();
    b.observe(report(2, 812, 611, true)).unwrap();
    b.observe(report(1, 811, 611, true)).unwrap();
    close(truth(&b, 501), truth(&original, 7));
    close(profile(&b, 611, 703), profile(&original, 19, 3));
    let before = b.clone();
    let s = b.snapshot();
    assert_eq!(s, b.snapshot());
    assert_eq!(b, before);
    close(s.hypotheses.iter().map(|h| h.probability).sum(), 1.0);
    for speaker in &s.speakers {
        close(speaker.profiles.iter().map(|p| p.probability).sum(), 1.0);
    }
    assert!(s.hypotheses.iter().all(|h| h.probability.is_finite()));
}

#[test]
fn testimony_diagnostic_matrix_is_frozen_and_deterministic() {
    let a = diagnose_testimony().unwrap();
    assert_eq!(a.version, "testimony-v1");
    assert_eq!(a.tolerance, 1e-12);
    assert_eq!(a.fixtures.len(), 13);
    assert_eq!(a.decisions.len(), 3);
    assert!(a.passed);
    assert_eq!(
        serde_json::to_string(&a).unwrap(),
        serde_json::to_string(&diagnose_testimony().unwrap()).unwrap()
    );
    assert_eq!(
        a.passed,
        a.fixtures.iter().all(|f| f.passed
            && f.references
                .iter()
                .all(|r| r.passed && r.absolute_error <= a.tolerance)
            && f.checks.iter().all(|c| c.passed && c.actual == c.expected))
            && a.decisions.iter().all(|d| d.passed)
    );
    for exact in ["13/20", "49/68", "16/17", "4/5", "1/5", "12/13", "49/65"] {
        assert!(a
            .fixtures
            .iter()
            .flat_map(|f| &f.references)
            .any(|r| r.expected_exact == exact));
    }
    assert!(a.fixtures.iter().any(|f| f.name == "model_contradiction"
        && f.checks
            .iter()
            .any(|c| c.quantity == "zero_evidence" && c.actual)));
}

#[test]
fn testimony_diagnostic_reports_every_declared_reference_and_decision() {
    let report = diagnose_testimony().unwrap();
    let expected = [
        ("empty_evidence", vec![("1/2", 0.5), ("3/4", 0.75)]),
        (
            "uncertain_speaker",
            vec![("13/20", 13.0 / 20.0), ("3/4", 0.75)],
        ),
        ("two_signals", vec![("49/68", 49.0 / 68.0)]),
        ("shared_signal", vec![("13/20", 13.0 / 20.0)]),
        ("duplicate_record", vec![]),
        ("independent_witnesses", vec![("16/17", 16.0 / 17.0)]),
        ("common_signal_witnesses", vec![("4/5", 4.0 / 5.0)]),
        ("inversion", vec![("1/5", 1.0 / 5.0)]),
        ("always_positive", vec![("1/2", 0.5)]),
        ("half_accuracy", vec![("1/2", 0.5)]),
        (
            "verified_truth",
            vec![("1", 1.0), ("1/2", 0.5), ("12/13", 12.0 / 13.0)],
        ),
        ("transfer", vec![("49/65", 49.0 / 65.0)]),
        ("model_contradiction", vec![]),
    ];
    for (fixture, (name, references)) in report.fixtures.iter().zip(expected) {
        assert_eq!(fixture.name, name);
        assert_eq!(fixture.references.len(), references.len());
        Belief::new(fixture.model.clone()).unwrap();
        for (actual, (exact, expected)) in fixture.references.iter().zip(references) {
            assert_eq!(actual.expected_exact, exact);
            assert_eq!(actual.expected, expected);
            assert!((actual.actual - expected).abs() <= 1e-12);
            assert_eq!(actual.absolute_error, (actual.actual - expected).abs());
        }
    }
    for name in ["duplicate_record", "model_contradiction"] {
        let fixture = report.fixtures.iter().find(|f| f.name == name).unwrap();
        assert_eq!(fixture.checks.len(), 2);
        assert!(fixture
            .checks
            .iter()
            .all(|c| c.expected && c.actual && c.passed));
    }
    for (decision, (action, utility, credulous, regret)) in report.decisions.iter().zip([
        (TestimonyDecision::Accuse, 0.3, 0.3, 0.0),
        (TestimonyDecision::Abstain, 0.0, -0.6, 0.6),
        (TestimonyDecision::Abstain, 0.0, 0.0, 0.0),
    ]) {
        assert_eq!(decision.chosen_action, action);
        assert!((decision.chosen_utility - utility).abs() <= 1e-12);
        assert!((decision.credulous_utility - credulous).abs() <= 1e-12);
        assert!((decision.credulous_regret - regret).abs() <= 1e-12);
        assert!(decision.regret.abs() <= 1e-12);
    }
}
