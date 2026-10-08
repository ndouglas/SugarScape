use super::super::*;
use crate::shared_surface::Outcome;
use serde_json::Value;
use std::sync::OnceLock;

fn policy(role: Role, kind: PolicyKind, prior: OwnPrior) -> &'static CompiledPolicy {
    static POLICIES: OnceLock<Vec<CompiledPolicy>> = OnceLock::new();
    let policies = POLICIES.get_or_init(|| {
        [Role::A, Role::B]
            .into_iter()
            .flat_map(|r| {
                [
                    PolicyKind::Adaptive,
                    PolicyKind::FixedThree,
                    PolicyKind::NoProbe,
                    PolicyKind::InspectOnly,
                    PolicyKind::Known,
                ]
                .into_iter()
                .flat_map(move |k| {
                    let priors = if k == PolicyKind::Known {
                        Mechanism::ALL.map(OwnPrior::PointMass).to_vec()
                    } else {
                        vec![OwnPrior::Uniform]
                    };
                    priors
                        .into_iter()
                        .map(move |p| CompiledPolicy::build(&Protocol::new(r, k), p).unwrap())
                })
            })
            .collect()
    });
    let offset = if role == Role::A { 0 } else { 8 };
    let index = match kind {
        PolicyKind::Adaptive => 0,
        PolicyKind::FixedThree => 1,
        PolicyKind::NoProbe => 2,
        PolicyKind::InspectOnly => 3,
        PolicyKind::Known => {
            4 + match prior {
                OwnPrior::PointMass(m) => m.index(),
                _ => unreachable!(),
            }
        }
    };
    &policies[offset + index]
}
fn fixture() -> Value {
    serde_json::from_str(include_str!("fixtures/planning-reference.json")).unwrap()
}
fn kind(v: &Value) -> PolicyKind {
    serde_json::from_value(v.clone()).unwrap()
}
fn role(v: &Value) -> Role {
    serde_json::from_value(v.clone()).unwrap()
}
fn choice(v: &Value) -> Choice {
    match v.as_str().unwrap() {
        "Stop" => Choice::StopProbing,
        "Continue" => Choice::ContinueProbe,
        "Inspect" => Choice::Inspect,
        "Attempt" => Choice::AttemptCommunication,
        _ => panic!(),
    }
}
fn model(v: &Value) -> Mechanism {
    match v.as_str().unwrap() {
        "SP" => Mechanism::SharedPersistent,
        "SR" => Mechanism::SharedResetting,
        "PP" => Mechanism::PrivatePersistent,
        "I" => Mechanism::Inert,
        _ => panic!(),
    }
}
fn num(v: &Value) -> u8 {
    u8::try_from(v.as_u64().unwrap()).unwrap()
}
fn phase(v: &Value) -> Phase {
    if v.as_i64() == Some(-1) {
        Phase::Calibration
    } else {
        Phase::Live { trial: num(v) }
    }
}
fn boundary(v: &Value, n: &Value) -> Boundary {
    if v == "probe" {
        Boundary::ProbeChoice { completed: num(n) }
    } else {
        Boundary::TrialChoice { trial: num(n) }
    }
}
fn prefix(row: &Value, p: &Protocol, prior: OwnPrior) -> Prefix {
    Prefix {
        role: p.experimenter,
        ids: p.ids.clone(),
        own_prior: prior,
        checkpoint: Checkpoint::Boundary(boundary(&row["stage"][0], &row["stage"][1])),
        entries: row["prefix"]
            .as_array()
            .unwrap()
            .iter()
            .map(|e| match e[0].as_str().unwrap() {
                "Choice" => Entry::OwnChoice {
                    boundary: boundary(&e[1], &e[2]),
                    choice: choice(&e[3]),
                },
                "Stop" => Entry::PublicProbeStop {
                    completed: num(&e[1]),
                },
                "Reset" => Entry::Physical(LocalEntry::Reset {
                    phase: phase(&e[1]),
                }),
                "Bit" => Entry::Physical(LocalEntry::PrivateBit {
                    trial: num(&e[1]),
                    bit: e[2] == 1,
                }),
                "Act" => Entry::Physical(LocalEntry::Action(Event {
                    position: Position {
                        phase: phase(&e[1]),
                        round: num(&e[2]),
                        slot: num(&e[3]),
                    },
                    action: match e[4][0].as_str().unwrap() {
                        "Write" => Action::Write(serde_json::from_value(e[4][1].clone()).unwrap()),
                        "Read" => Action::Read,
                        "Wait" => Action::Wait,
                        "Inspect" => Action::InspectOwnTarget,
                        _ => panic!(),
                    },
                    outcome: match e[5][0].as_str().unwrap() {
                        "Accepted" => Outcome::Accepted,
                        "Read" => Outcome::Read(serde_json::from_value(e[5][1].clone()).unwrap()),
                        "Waited" => Outcome::Waited,
                        "Inspected" => Outcome::Inspected(e[5][1] == 1),
                        _ => panic!(),
                    },
                    credits_after: num(&e[6]),
                })),
                _ => panic!(),
            })
            .collect(),
    }
}
fn check_row(row: &Value, kind: PolicyKind, prior: OwnPrior) {
    let p = Protocol::new(role(&row["experimenter"]), kind);
    let c = policy(p.experimenter, kind, prior);
    let prefix = prefix(row, &p, prior);
    let belief = c.infer(&prefix).unwrap();
    assert_eq!(
        belief.models,
        serde_json::from_value::<[Probability; 4]>(row["belief"]["models"].clone()).unwrap()
    );
    assert_eq!(
        belief.target,
        serde_json::from_value::<Option<Probability>>(row["belief"]["target"].clone()).unwrap()
    );
    let d = c
        .decide(&View::from_prefix(prefix, belief).unwrap())
        .unwrap();
    assert_eq!(d.choice, choice(&row["chosen"]));
    assert_eq!(d.continuation_policy, kind);
    for (name, value) in row["alternatives"].as_object().unwrap() {
        assert_eq!(
            d.alternatives
                .iter()
                .find(|a| a.choice == choice(&Value::String(name.clone())))
                .unwrap()
                .value,
            serde_json::from_value::<ScoreFraction>(value.clone()).unwrap(),
            "row {} alternative {name}",
            row["id"]
        );
    }
}
// Catches wrong continuation policies, peer costs in objective, future-bit leakage,
// and normalization at every full-prefix reference decision (including real ties).
#[test]
fn independent_exact_initial_and_full_prefix_decisions() {
    let f = fixture();
    for row in f["initial"].as_array().unwrap() {
        let k = kind(&row["policy"]);
        let prior = if row["known"].is_null() {
            OwnPrior::Uniform
        } else {
            OwnPrior::PointMass(model(&row["known"]))
        };
        let mut row = row.clone();
        row["prefix"] = serde_json::json!([]);
        row["stage"] = serde_json::json!(["probe", 0]);
        check_row(&row, k, prior);
    }
    for row in f["selected"].as_array().unwrap() {
        check_row(row, kind(&row["policy"]), OwnPrior::Uniform);
    }
    for row in f["live_ties"].as_array().unwrap() {
        check_row(row, PolicyKind::Adaptive, OwnPrior::Uniform);
    }
}
// Catches pruning point-prior zero-mass metadata or executing excluded worlds.
#[test]
fn point_control_retains_original_domain() {
    let c = policy(
        Role::B,
        PolicyKind::Known,
        OwnPrior::PointMass(Mechanism::SharedPersistent),
    );
    assert_eq!(
        (c.stats().candidate_worlds, c.stats().positive_worlds),
        (1024, 256)
    );
}
// Synthetic normative tie input, not a claimed observed calibration tie.
#[test]
fn synthetic_probe_and_live_ties_use_actual_numeric_selector() {
    for (preferred, other) in [
        (Choice::StopProbing, Choice::ContinueProbe),
        (Choice::Inspect, Choice::AttemptCommunication),
    ] {
        let values = [
            ChoiceValue {
                choice: other,
                value: ScoreFraction::new(-1, 2).unwrap(),
            },
            ChoiceValue {
                choice: preferred,
                value: ScoreFraction::new(-1, 2).unwrap(),
            },
        ];
        assert_eq!(
            super::super::planner::maximizing_choice(&values).unwrap(),
            preferred
        );
    }
    let values = [
        ChoiceValue {
            choice: Choice::Inspect,
            value: ScoreFraction::new(-1, 2).unwrap(),
        },
        ChoiceValue {
            choice: Choice::AttemptCommunication,
            value: ScoreFraction::new(-1, 3).unwrap(),
        },
    ];
    assert_eq!(
        super::super::planner::maximizing_choice(&values).unwrap(),
        Choice::AttemptCommunication
    );
}

// Catches conditioning on an own bit twice, counting replicated paths instead
// of original worlds, and a marginal-only cache that erases earlier inspection.
#[test]
fn original_mass_and_same_marginals_preserve_distinct_full_histories() {
    let f = fixture();
    for row in f["selected"]
        .as_array()
        .unwrap()
        .iter()
        .chain(f["live_ties"].as_array().unwrap())
    {
        let k = if row["policy"].is_null() {
            PolicyKind::Adaptive
        } else {
            kind(&row["policy"])
        };
        let p = Protocol::new(role(&row["experimenter"]), k);
        let own = prefix(row, &p, OwnPrior::Uniform);
        let domain = super::super::belief::Domain::new(&p, OwnPrior::Uniform).unwrap();
        let ids = domain.support(&own).unwrap();
        assert_eq!(
            ids.len(),
            usize::try_from(row["belief"]["mass"].as_u64().unwrap()).unwrap()
        );
        let distinct: std::collections::BTreeSet<_> = ids.iter().copied().collect();
        assert_eq!(distinct.len(), ids.len());
        assert_eq!(
            domain.belief(&own, ids).unwrap(),
            policy(p.experimenter, k, OwnPrior::Uniform)
                .infer(&own)
                .unwrap()
        );
    }
    let rows = f["selected"].as_array().unwrap();
    let first = rows
        .iter()
        .find(|r| {
            r["experimenter"] == "B"
                && r["policy"] == "Adaptive"
                && r["stage"] == serde_json::json!(["trial", 0])
        })
        .unwrap();
    let later = &f["live_ties"][1];
    assert_eq!(first["belief"]["models"], later["belief"]["models"]);
    assert_ne!(first["alternatives"], later["alternatives"]);
}
fn actual_state(p: &Protocol, model: Mechanism, bits: u16) -> EpisodeState {
    let mut priors = [OwnPrior::PointMass(model); 2];
    priors[p.experimenter.index()] = OwnPrior::Uniform;
    EpisodeState::new(
        p,
        Environment::InFamily(model),
        EpisodeBits::from_index(bits).unwrap(),
        priors,
    )
    .unwrap()
}
// Catches consulting A's private bit or first action while B sees neither.
#[test]
fn b_choice_is_invariant_to_unobserved_peer_bit_and_history() {
    let p = Protocol::new(Role::B, PolicyKind::Adaptive);
    let c = policy(Role::B, PolicyKind::Adaptive, OwnPrior::Uniform);
    let mut a = actual_state(&p, Mechanism::SharedPersistent, 0);
    let mut b = actual_state(&p, Mechanism::SharedPersistent, 1);
    apply_choice(&mut a, Choice::StopProbing).unwrap();
    apply_choice(&mut b, Choice::StopProbing).unwrap();
    assert_ne!(a.prefix(Role::A), b.prefix(Role::A));
    assert_eq!(a.prefix(Role::B), b.prefix(Role::B));
    let va = View::from_prefix(
        a.prefix(Role::B).clone(),
        c.infer(a.prefix(Role::B)).unwrap(),
    )
    .unwrap();
    let vb = View::from_prefix(
        b.prefix(Role::B).clone(),
        c.infer(b.prefix(Role::B)).unwrap(),
    )
    .unwrap();
    assert_eq!(c.decide(&va).unwrap(), c.decide(&vb).unwrap());
}
// Catches accepting host-supplied credits, current bits, or a forged posterior.
#[test]
fn decide_reconstructs_current_view_from_own_entries() {
    let p = Protocol::new(Role::A, PolicyKind::Adaptive);
    let c = policy(Role::A, PolicyKind::Adaptive, OwnPrior::Uniform);
    let mut s = actual_state(&p, Mechanism::Inert, 0);
    apply_choice(&mut s, Choice::StopProbing).unwrap();
    let v = View::from_prefix(
        s.prefix(Role::A).clone(),
        c.infer(s.prefix(Role::A)).unwrap(),
    )
    .unwrap();
    let mut bad = v.clone();
    bad.credits -= 1;
    assert!(matches!(c.decide(&bad), Err(Error::InvalidHistory(_))));
    let mut bad = v.clone();
    bad.private_bit = Some(true);
    assert!(matches!(c.decide(&bad), Err(Error::InvalidHistory(_))));
    let mut bad = v;
    bad.belief.target = Some(Probability::new(1, 1).unwrap());
    assert!(matches!(c.decide(&bad), Err(Error::InvalidHistory(_))));
}
// Catches conflating malformed chronology with well-shaped zero support.
#[test]
fn malformed_histories_and_unsupported_observations_are_distinct() {
    let p = Protocol::new(Role::A, PolicyKind::Adaptive);
    let c = policy(Role::A, PolicyKind::Adaptive, OwnPrior::Uniform);
    let mut s = actual_state(&p, Mechanism::SharedPersistent, 0);
    apply_choice(&mut s, Choice::ContinueProbe).unwrap();
    let mut malformed = s.prefix(Role::A).clone();
    malformed.entries.remove(0);
    assert!(matches!(c.infer(&malformed), Err(Error::InvalidHistory(_))));
    let mut unsupported = s.prefix(Role::A).clone();
    if let Some(Entry::Physical(LocalEntry::Action(event))) = unsupported.entries.last_mut() {
        event.outcome = Outcome::Read(Symbol::Data1);
    }
    p.validate_prefix(&unsupported).unwrap();
    assert!(matches!(
        c.infer(&unsupported),
        Err(Error::UnsupportedHistory { .. })
    ));
    let mut future = s.prefix(Role::A).clone();
    future.entries.push(Entry::Physical(LocalEntry::PrivateBit {
        trial: 3,
        bit: true,
    }));
    assert!(matches!(c.infer(&future), Err(Error::InvalidHistory(_))));
}

// Catches reversing target majority, derived fraction ordering, or predicting
// one on a fair tie; the helper is also the planner's reward decoder.
#[test]
fn target_prediction_uses_numeric_majority_and_zero_on_ties() {
    for (numerator, denominator, want) in [(1, 2, false), (49, 100, false), (51, 100, true)] {
        let belief = Belief {
            models: [Probability::new(1, 4).unwrap(); 4],
            target: Some(Probability::new(numerator, denominator).unwrap()),
        };
        assert_eq!(predict_target(&belief).unwrap(), want);
    }
}
// Catches turning a missing target into a guess or accepting invalid rationals.
#[test]
fn target_prediction_rejects_missing_or_invalid_target_probability() {
    for target in [
        None,
        Some(Probability {
            numerator: 1,
            denominator: 0,
        }),
        Some(Probability {
            numerator: 2,
            denominator: 1,
        }),
    ] {
        let belief = Belief {
            models: [Probability::new(1, 4).unwrap(); 4],
            target,
        };
        assert!(predict_target(&belief).is_err());
    }
}
