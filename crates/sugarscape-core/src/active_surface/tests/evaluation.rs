use super::super::*;

// Pins costs and target decoding across both chronological orientations.
#[test]
fn inspection_only_has_zero_probes_and_perfect_experimenter_accuracy() {
    for role in [Role::A, Role::B] {
        let p = Protocol::new(role, PolicyKind::InspectOnly);
        let c = CompiledPolicy::build(&p, OwnPrior::Uniform).unwrap();
        let replay = Replay::build(&p, &c).unwrap();
        for model in Mechanism::ALL {
            let panel = evaluate_panel(&p, Environment::InFamily(model), &c, &replay).unwrap();
            assert_eq!(panel.episodes.len(), 256);
            for episode in panel.episodes {
                assert_eq!(episode.metrics[role.index()].correct, Some(4));
                assert_eq!(episode.metrics[role.index()].spent, 36);
                assert!(!episode
                    .decisions
                    .iter()
                    .any(|d| d.decision.choice == Choice::ContinueProbe));
            }
        }
    }
}

// A paid unsupported echo must halt before B's later wait and next task bit.
#[test]
fn inverted_own_echo_stops_at_charged_read_without_terminal_means() {
    let p = Protocol::new(Role::B, PolicyKind::InspectOnly);
    let c = CompiledPolicy::build(&p, OwnPrior::Uniform).unwrap();
    let replay = Replay::build(&p, &c).unwrap();
    let panel = evaluate_panel(&p, Environment::DataFlip, &c, &replay).unwrap();
    assert_eq!(panel.aggregate.failed_mass, Probability::new(1, 1).unwrap());
    assert_eq!(panel.aggregate.net, [None, None]);
    assert_eq!(panel.aggregate.group_net, None);
    for e in panel.episodes {
        let failure = e.failure.unwrap();
        assert_eq!(failure.role, Role::A);
        assert_eq!(failure.spent, [5, 7]);
        assert_eq!(
            failure.prefix.checkpoint,
            Checkpoint::AfterSlot(Position {
                phase: Phase::Live { trial: 0 },
                round: 3,
                slot: 1
            })
        );
        assert!(failure.last_supported.is_some());
        assert_eq!(
            e.metrics.iter().map(|m| m.spent).collect::<Vec<_>>(),
            vec![5, 7]
        );
        assert!(e
            .metrics
            .iter()
            .all(|m| m.reward.is_none() && m.net.is_none() && m.correct.is_none()));
        assert!(e.predictions.iter().all(Vec::is_empty));
        assert!(e
            .histories
            .iter()
            .all(|h| !h.entries.iter().any(|x| matches!(
                x,
                Entry::Physical(LocalEntry::PrivateBit { trial: 1.., .. })
            ))));
    }
}

use serde_json::{json, Value};
use std::collections::BTreeMap;

fn environment(v: &Value) -> Environment {
    match v.as_str().unwrap() {
        "SP" => Environment::InFamily(Mechanism::SharedPersistent),
        "SR" => Environment::InFamily(Mechanism::SharedResetting),
        "PP" => Environment::InFamily(Mechanism::PrivatePersistent),
        "I" => Environment::InFamily(Mechanism::Inert),
        "DF" => Environment::DataFlip,
        _ => panic!("unknown reference environment"),
    }
}
fn phase_value(phase: Phase) -> Value {
    match phase {
        Phase::Calibration => json!(-1),
        Phase::Live { trial } => json!(trial),
    }
}
fn choice_value(choice: Choice) -> &'static str {
    match choice {
        Choice::ContinueProbe => "Continue",
        Choice::StopProbing => "Stop",
        Choice::Inspect => "Inspect",
        Choice::AttemptCommunication => "Attempt",
    }
}
fn boundary_value(boundary: Boundary) -> (&'static str, u8) {
    match boundary {
        Boundary::ProbeChoice { completed } => ("probe", completed),
        Boundary::TrialChoice { trial } => ("trial", trial),
    }
}
fn entries_value(entries: &[Entry]) -> Value {
    use crate::shared_surface::Outcome;
    json!(entries
        .iter()
        .map(|entry| match entry {
            Entry::OwnChoice { boundary, choice } => {
                let (kind, n) = boundary_value(*boundary);
                json!(["Choice", kind, n, choice_value(*choice)])
            }
            Entry::PublicProbeStop { completed } => json!(["Stop", completed]),
            Entry::Physical(LocalEntry::Reset { phase }) => json!(["Reset", phase_value(*phase)]),
            Entry::Physical(LocalEntry::PrivateBit { trial, bit }) =>
                json!(["Bit", trial, u8::from(*bit)]),
            Entry::Physical(LocalEntry::Action(event)) => {
                let action = match event.action {
                    Action::Write(symbol) => json!(["Write", symbol]),
                    Action::Read => json!(["Read"]),
                    Action::Wait => json!(["Wait"]),
                    Action::InspectOwnTarget => json!(["Inspect"]),
                };
                let outcome = match event.outcome {
                    Outcome::Read(symbol) => json!(["Read", symbol]),
                    Outcome::Accepted => json!(["Accepted"]),
                    Outcome::Waited => json!(["Waited"]),
                    Outcome::Inspected(bit) => json!(["Inspected", u8::from(bit)]),
                };
                json!([
                    "Act",
                    phase_value(event.position.phase),
                    event.position.round,
                    event.position.slot,
                    action,
                    outcome,
                    event.credits_after
                ])
            }
        })
        .collect::<Vec<_>>())
}
fn checkpoint_value(checkpoint: Checkpoint) -> Value {
    match checkpoint {
        Checkpoint::Boundary(b) => {
            let (kind, n) = boundary_value(b);
            json!(["boundary", kind, n])
        }
        Checkpoint::AfterSlot(p) => json!(["slot", phase_value(p.phase), p.round, p.slot]),
        Checkpoint::BeforeSlot(p) => json!(["before", phase_value(p.phase), p.round, p.slot]),
        Checkpoint::Prediction { trial } => json!(["prediction", trial]),
        Checkpoint::Finished => json!(["finished"]),
    }
}
fn assert_belief(actual: &Belief, expected: &Value) {
    assert_eq!(json!(actual.models), expected["models"]);
    assert_eq!(json!(actual.target), expected["target"]);
}
// The independent aggregate uses one null for an unavailable pair; the public
// Rust contract uses two explicit Option values. No observation is normalized.
fn unavailable_pair(value: &Value) -> Value {
    if value.is_null() {
        json!([null, null])
    } else {
        value.clone()
    }
}
fn assert_aggregate(panel: &Panel, row: &Value) {
    let expected = &row["aggregate"];
    assert_eq!(json!(panel.aggregate.spent), expected["spent"]);
    assert_eq!(
        json!(panel.aggregate.reward),
        unavailable_pair(&expected["reward"])
    );
    assert_eq!(
        json!(panel.aggregate.net),
        unavailable_pair(&expected["net"])
    );
    assert_eq!(json!(panel.aggregate.group_net), expected["group_net"]);
    assert_eq!(
        panel.aggregate.failed_mass,
        Probability::new(expected["failure"].as_u64().unwrap(), 256).unwrap()
    );
    let mut stops = BTreeMap::<String, u64>::new();
    for e in &panel.episodes {
        let stop = e.histories[0]
            .entries
            .iter()
            .find_map(|e| match e {
                Entry::PublicProbeStop { completed } => Some(*completed),
                _ => None,
            })
            .unwrap();
        *stops.entry(stop.to_string()).or_default() += 1;
    }
    assert_eq!(json!(stops), expected["stops"]);
    for i in 0..2 {
        let identified = panel
            .episodes
            .iter()
            .filter(|e| e.catalog_discoveries[i].is_some())
            .count();
        assert_eq!(json!(identified), expected["identified"][i]);
        assert_eq!(
            json!(256 - identified),
            expected["identification_censored"][i]
        );
    }
    for (field, metric) in [
        (
            "attempted_messages",
            (|m: &AgentMetrics| m.attempted_sends) as fn(&AgentMetrics) -> u8,
        ),
        ("received_peer_task_writes", |m: &AgentMetrics| {
            m.lineage_reads
        }),
        ("inspections", |m: &AgentMetrics| m.inspections),
    ] {
        for i in 0..2 {
            let total: i64 = panel
                .episodes
                .iter()
                .map(|e| i64::from(metric(&e.metrics[i])))
                .sum();
            assert_eq!(
                json!(ScoreFraction::new(total, 256).unwrap()),
                expected[field][i],
                "{field} role {i}"
            );
        }
    }
}

// Forty catalog and six misspecification settings use independent exact means.
#[test]
fn all_46_aggregates_match_independent_reference() {
    let fixture: Value =
        serde_json::from_str(include_str!("fixtures/evaluation-reference.json")).unwrap();
    verify_reference(&fixture, false);
}
fn verify_reference(reference: &Value, complete: bool) {
    let rows = reference["panels"].as_array().unwrap();
    let mut count = 0;
    let mut observed_panels = Vec::new();
    for role in [Role::A, Role::B] {
        for kind in [
            PolicyKind::Adaptive,
            PolicyKind::FixedThree,
            PolicyKind::NoProbe,
            PolicyKind::InspectOnly,
            PolicyKind::Known,
        ] {
            let priors = if kind == PolicyKind::Known {
                Mechanism::ALL.map(OwnPrior::PointMass).to_vec()
            } else {
                vec![OwnPrior::Uniform]
            };
            for prior in priors {
                let p = Protocol::new(role, kind);
                let c = CompiledPolicy::build(&p, prior).unwrap();
                let replay = Replay::build(&p, &c).unwrap();
                for row in rows
                    .iter()
                    .filter(|r| r["experimenter"] == json!(role) && r["policy"] == json!(kind))
                {
                    let env = environment(&row["environment"]);
                    if let OwnPrior::PointMass(m) = prior {
                        if env != Environment::InFamily(m) {
                            continue;
                        }
                    }
                    let panel = evaluate_panel(&p, env, &c, &replay).unwrap();
                    assert_aggregate(&panel, row);
                    if let Some(traces) = row["traces"].as_array() {
                        for trace in traces {
                            let episode =
                                &panel.episodes[trace["sequence"].as_u64().unwrap() as usize];
                            assert_eq!(json!(episode.predictions), trace["predictions"]);
                            for i in 0..2 {
                                assert_eq!(
                                    entries_value(&episode.histories[i].entries),
                                    trace["histories"][i]
                                );
                                for (b, want) in episode.prediction_beliefs[i]
                                    .iter()
                                    .zip(trace["prediction_beliefs"][i].as_array().unwrap())
                                {
                                    assert_belief(b, want);
                                }
                            }
                        }
                    }
                    if complete {
                        compare_episodes(&panel, row, reference, &c, &replay);
                    }
                    observed_panels.push(panel);
                    count += 1;
                }
            }
        }
    }
    assert_eq!(count, 46);
    let projections: Value =
        serde_json::from_str(include_str!("fixtures/evaluation-reference.json")).unwrap();
    let pairs = projections["paired_comparisons"].as_array().unwrap();
    for pair in pairs {
        let find = |field: &str| {
            observed_panels
                .iter()
                .find(|p| {
                    json!(p.protocol.experimenter) == pair["experimenter"]
                        && p.environment == environment(&pair["environment"])
                        && json!(p.protocol.policy) == pair[field]
                })
                .unwrap()
        };
        let result = compare_panels(find("left_policy"), find("right_policy")).unwrap();
        assert_eq!(json!(result.difference), pair["difference"]);
    }
    assert_eq!(pairs.len(), 86);
    if complete {
        let episodes: usize = rows
            .iter()
            .map(|r| r["episodes"].as_array().unwrap().len())
            .sum();
        let traces: usize = rows
            .iter()
            .flat_map(|r| r["episodes"].as_array().unwrap())
            .filter(|e| !e["histories"].is_null())
            .count();
        assert_eq!(episodes, 11776);
        assert_eq!(traces, 322);
        println!("Verified {count} aggregates, {episodes} episodes, {traces} designated full traces, and {} exact paired differences",pairs.len());
    }
}

// External full reference is ignored evidence, not a raw code/trace fixture.
#[test]
#[ignore = "requires independently reviewed 202 MB reference-v2.json"]
fn full_independent_reference_all_11776_episodes_and_322_traces() {
    let path = std::env::var("ACTIVE_SURFACE_REFERENCE").expect("reference-v2.json path");
    let reference: Value = serde_json::from_reader(std::fs::File::open(path).unwrap()).unwrap();
    assert_eq!(
        reference["source_sha256"],
        "6eafbad345a3e23e312929e54ff5c9aac1538496bc1c1a73a26cd769f3300675"
    );
    verify_reference(&reference, true);
}
fn compare_episodes(
    panel: &Panel,
    row: &Value,
    reference: &Value,
    policy: &CompiledPolicy,
    replay: &Replay,
) {
    let planning = reference["planning"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| {
            r["experimenter"] == json!(panel.protocol.experimenter)
                && r["policy"] == json!(panel.protocol.policy)
                && match policy.own_prior() {
                    OwnPrior::Uniform => r["known"].is_null(),
                    OwnPrior::PointMass(m) => environment(&r["known"]) == Environment::InFamily(m),
                }
        })
        .unwrap();
    let decisions: BTreeMap<&str, &Value> = planning["records"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| (r["id"].as_str().unwrap(), r))
        .collect();
    for (e, expected) in panel
        .episodes
        .iter()
        .zip(row["episodes"].as_array().unwrap())
    {
        assert_eq!(json!(e.sequence), expected["sequence"]);
        assert_eq!(json!(e.predictions), expected["predictions"]);
        assert_eq!(
            json!(e.metrics.each_ref().map(|m| m.spent)),
            expected["spent"]
        );
        assert_eq!(
            json!(e.metrics.each_ref().map(|m| m.correct)),
            expected["correct"]
        );
        assert_eq!(
            json!(e.metrics.each_ref().map(|m| m.reward)),
            expected["reward"]
        );
        assert_eq!(json!(e.metrics.each_ref().map(|m| m.net)), expected["net"]);
        assert_eq!(
            json!(e.metrics.each_ref().map(|m| m.attempted_sends)),
            expected["attempted_messages"]
        );
        assert_eq!(
            json!(e.metrics.each_ref().map(|m| m.lineage_reads)),
            expected["received_peer_task_writes"]
        );
        for i in 0..2 {
            let identification = &expected["first_identification"][i];
            if let Some(discovery) = &e.catalog_discoveries[i] {
                assert_eq!(
                    checkpoint_value(discovery.checkpoint),
                    identification["checkpoint"]
                );
                assert_eq!(
                    match discovery.source {
                        CatalogCertaintySource::Supplied => "supplied",
                        CatalogCertaintySource::Observed => "observed",
                    },
                    identification["kind"]
                );
                let checkpoint_entries =
                    identification["own_entry_count"].as_u64().unwrap() as usize;
                let spent = e.histories[i].entries[..checkpoint_entries]
                    .iter()
                    .rev()
                    .find_map(|x| {
                        if let Entry::Physical(LocalEntry::Action(event)) = x {
                            Some(48 - event.credits_after)
                        } else {
                            None
                        }
                    })
                    .unwrap_or(0);
                assert_eq!(discovery.spent, spent);
            } else {
                assert!(identification.is_null());
            }
            for (belief, want) in e.prediction_beliefs[i]
                .iter()
                .zip(expected["prediction_beliefs"][i].as_array().unwrap())
            {
                assert_belief(belief, want);
            }
            if let Some(projection) = &e.model_projections[i] {
                assert_eq!(
                    json!(projection.final_belief.models),
                    expected["final_model_beliefs"][i]
                );
                let id = &expected["first_identification"][i];
                assert_eq!(
                    projection.first_identification.map(checkpoint_value),
                    if id.is_null() {
                        None
                    } else {
                        Some(id["checkpoint"].clone())
                    }
                );
                if matches!(panel.environment, Environment::DataFlip) {
                    assert_eq!(projection.true_model_mass, None);
                    assert_eq!(projection.uniquely_identified, None);
                }
            } else {
                assert!(expected["final_model_beliefs"][i].is_null());
            }
        }
        assert_eq!(
            e.decisions.len(),
            expected["decisions"].as_array().unwrap().len()
        );
        for (actual, id) in e
            .decisions
            .iter()
            .zip(expected["decisions"].as_array().unwrap())
        {
            let want = decisions[id.as_str().unwrap()];
            assert_eq!(entries_value(&actual.prefix.entries), want["prefix"]);
            assert_eq!(choice_value(actual.decision.choice), want["chosen"]);
            assert_belief(&replay.infer(&actual.prefix).unwrap(), &want["belief"]);
            assert_eq!(
                replay.infer(&actual.prefix).unwrap(),
                policy.infer(&actual.prefix).unwrap()
            );
            for alt in &actual.decision.alternatives {
                assert_eq!(
                    json!(alt.value),
                    want["alternatives"][choice_value(alt.choice)]
                );
            }
        }
        if let Some(f) = &e.failure {
            assert_eq!(json!(f.role.index()), expected["failure"]["role"]);
            assert_eq!(
                checkpoint_value(f.prefix.checkpoint),
                expected["failure"]["checkpoint"]
            );
            assert_eq!(json!(f.spent), expected["failure"]["spent"]);
            assert_belief(
                f.last_supported.as_ref().unwrap(),
                &expected["failure"]["last_supported"],
            );
        } else {
            assert!(expected["failure"].is_null());
        }
        if !expected["histories"].is_null() {
            for i in 0..2 {
                assert_eq!(
                    entries_value(&e.histories[i].entries),
                    expected["histories"][i]
                );
            }
            compare_checkpoints(
                e,
                &panel.protocol,
                panel.environment,
                policy,
                replay,
                expected,
            );
        }
    }
}
fn compare_checkpoints(
    e: &Episode,
    p: &Protocol,
    env: Environment,
    policy: &CompiledPolicy,
    replay: &Replay,
    expected: &Value,
) {
    let model = match env {
        Environment::InFamily(m) => m,
        Environment::DataFlip => Mechanism::SharedPersistent,
    };
    let mut priors = [OwnPrior::PointMass(model); 2];
    priors[p.experimenter.index()] = policy.own_prior();
    let mut state =
        EpisodeState::new(p, env, EpisodeBits::from_index(e.sequence).unwrap(), priors).unwrap();
    let mut snapshots = Vec::<(Value, [Prefix; 2])>::new();
    let mut next = true;
    loop {
        let prefix = state.prefix(p.experimenter);
        if matches!(
            prefix.checkpoint,
            Checkpoint::Boundary(_) | Checkpoint::AfterSlot(_) | Checkpoint::Prediction { .. }
        ) {
            snapshots.push((
                checkpoint_value(prefix.checkpoint),
                [Role::A, Role::B].map(|r| state.prefix(r).clone()),
            ));
        }
        if e.failure
            .as_ref()
            .is_some_and(|f| state.prefix(f.role) == &f.prefix)
            || prefix.checkpoint == Checkpoint::Finished
        {
            break;
        }
        if next {
            let decision = policy
                .decide(&View::from_prefix(prefix.clone(), replay.infer(prefix).unwrap()).unwrap())
                .unwrap();
            select_choice(&mut state, decision.choice).unwrap();
            if decision.choice == Choice::StopProbing {
                let Checkpoint::Boundary(Boundary::ProbeChoice { completed }) = e
                    .decisions
                    .iter()
                    .find(|d| d.decision.choice == Choice::StopProbing)
                    .unwrap()
                    .prefix
                    .checkpoint
                else {
                    unreachable!()
                };
                snapshots.push((
                    json!(["stop", completed]),
                    [Role::A, Role::B].map(|r| state.prefix(r).clone()),
                ));
            }
        }
        let step = advance_one(&mut state).unwrap();
        next = step.next.is_some();
    }
    for want in expected["checkpoints"].as_array().unwrap() {
        let (_, prefixes) = snapshots
            .iter()
            .find(|(key, _)| key == &want["checkpoint"])
            .unwrap_or_else(|| panic!("missing {:?}", want["checkpoint"]));
        for i in 0..2 {
            let belief = replay.infer(&prefixes[i]);
            if want["beliefs"][i].is_null() {
                assert!(matches!(belief, Err(Error::UnsupportedHistory { .. })));
            } else {
                assert_belief(&belief.unwrap(), &want["beliefs"][i]);
            }
            let spent = prefixes[i]
                .entries
                .iter()
                .rev()
                .find_map(|entry| {
                    if let Entry::Physical(LocalEntry::Action(event)) = entry {
                        Some(48 - event.credits_after)
                    } else {
                        None
                    }
                })
                .unwrap_or(0);
            assert_eq!(json!(spent), want["spent"][i]);
        }
    }
}

// New apparatus entries are explicitly checked, then only physical entries are
// projected for comparison to the frozen controller, which has no choice/stop API.
#[test]
fn fixed_three_matches_all_frozen_asymmetric_physical_episodes() {
    use crate::shared_surface as old;
    for role in [Role::A, Role::B] {
        let p = Protocol::new(role, PolicyKind::FixedThree);
        let c = CompiledPolicy::build(&p, OwnPrior::Uniform).unwrap();
        let replay = Replay::build(&p, &c).unwrap();
        let mut roles = [old::Knowledge::Known; 2];
        roles[role.index()] = old::Knowledge::Unknown;
        let old_p = old::Protocol {
            calibration_rounds: 3,
            pair: old::Pair { roles },
            prior_mode: old::PriorMode::Treatment,
            ids: p.ids.clone(),
        };
        for model in Mechanism::ALL {
            let panel = evaluate_panel(&p, Environment::InFamily(model), &c, &replay).unwrap();
            let frozen = old::evaluate_panel(&old_p, Environment::InFamily(model)).unwrap();
            for (new, old) in panel.episodes.iter().zip(frozen.episodes) {
                assert_eq!(
                    new.privileged
                        .iter()
                        .map(|s| (&s.role, &s.event, &s.read_lineage))
                        .collect::<Vec<_>>(),
                    old.privileged
                        .iter()
                        .map(|s| (&s.role, &s.event, &s.read_lineage))
                        .collect::<Vec<_>>()
                );
                assert_eq!(
                    new.decisions
                        .iter()
                        .filter(|d| d.decision.choice == Choice::ContinueProbe)
                        .count(),
                    3
                );
                assert!(new
                    .histories
                    .iter()
                    .all(|h| h.entries.contains(&Entry::PublicProbeStop { completed: 3 })));
                for i in 0..2 {
                    let physical: Vec<_> = new.histories[i]
                        .entries
                        .iter()
                        .filter_map(|e| {
                            if let Entry::Physical(e) = e {
                                Some(e.clone())
                            } else {
                                None
                            }
                        })
                        .collect();
                    assert_eq!(physical, old.local[i].steps.last().unwrap().prefix.entries);
                    assert_eq!(new.metrics[i].spent, old.metrics[i].spent);
                    assert_eq!(
                        new.predictions[i],
                        old.metrics[i].terminal.as_ref().unwrap().predictions
                    );
                }
            }
        }
    }
}

// Renaming the full typed reconstructed episode must change no policy/physics.
#[test]
fn opaque_ids_preserve_full_typed_episodes() {
    for role in [Role::A, Role::B] {
        let p = Protocol::new(role, PolicyKind::Adaptive);
        let mut renamed = p.clone();
        renamed.ids = Ids {
            surface: "x7".into(),
            agents: ["z9".into(), "q4".into()],
        };
        let c = CompiledPolicy::build(&p, OwnPrior::Uniform).unwrap();
        let replay = Replay::build(&p, &c).unwrap();
        let d = CompiledPolicy::build(&renamed, OwnPrior::Uniform).unwrap();
        let other = Replay::build(&renamed, &d).unwrap();
        for env in [
            Environment::InFamily(Mechanism::SharedPersistent),
            Environment::DataFlip,
        ] {
            for sequence in 0..256 {
                let a = run_episode(&p, env, sequence, &c, &replay).unwrap();
                let mut b = run_episode(&renamed, env, sequence, &d, &other).unwrap();
                for prefix in &mut b.histories {
                    prefix.ids = p.ids.clone();
                }
                for decision in &mut b.decisions {
                    decision.prefix.ids = p.ids.clone();
                }
                if let Some(f) = &mut b.failure {
                    f.prefix.ids = p.ids.clone();
                }
                assert_eq!(a, b);
            }
        }
    }
}

// Only a real other-Agent current-trial write is a task transmission; correlation
// or reading one's own retained write does not increment the lineage metric.
#[test]
fn sender_bit_changes_recipient_posterior_with_same_recipient_bits() {
    for role in [Role::A, Role::B] {
        let p = Protocol::new(role, PolicyKind::Known);
        let c =
            CompiledPolicy::build(&p, OwnPrior::PointMass(Mechanism::SharedPersistent)).unwrap();
        let replay = Replay::build(&p, &c).unwrap();
        let original = run_episode(
            &p,
            Environment::InFamily(Mechanism::SharedPersistent),
            0,
            &c,
            &replay,
        )
        .unwrap();
        for sender in [Role::A, Role::B] {
            let recipient = 1 - sender.index();
            let flipped = if sender == Role::A { 1 } else { 2 };
            let paired = run_episode(
                &p,
                Environment::InFamily(Mechanism::SharedPersistent),
                flipped,
                &c,
                &replay,
            )
            .unwrap();
            assert_ne!(
                original.prediction_beliefs[recipient][0].target,
                paired.prediction_beliefs[recipient][0].target
            );
            assert_ne!(
                original.predictions[recipient][0],
                paired.predictions[recipient][0]
            );
            assert_eq!(
                original.predictions[recipient][1..],
                paired.predictions[recipient][1..]
            );
            assert_eq!(original.metrics[recipient].lineage_reads, 4);
            assert_eq!(paired.metrics[recipient].lineage_reads, 4);
            assert_eq!(
                original.metrics[recipient].correct,
                paired.metrics[recipient].correct
            );
        }
    }
}

#[test]
fn lineage_snapshot_is_readonly_and_before_round_reset() {
    let p = Protocol::new(Role::A, PolicyKind::Known);
    let mut state = EpisodeState::new(
        &p,
        Environment::InFamily(Mechanism::SharedPersistent),
        EpisodeBits::from_index(0).unwrap(),
        [OwnPrior::PointMass(Mechanism::SharedPersistent); 2],
    )
    .unwrap();
    select_choice(&mut state, Choice::StopProbing).unwrap();
    advance_one(&mut state).unwrap();
    select_choice(&mut state, Choice::AttemptCommunication).unwrap();
    loop {
        let step = advance_one(&mut state).unwrap();
        if step
            .events
            .iter()
            .any(|(role, e)| *role == Role::B && e.action == Action::Read)
        {
            let before = [state.prefix(Role::A).clone(), state.prefix(Role::B).clone()];
            let credits = [state.credits(Role::A), state.credits(Role::B)];
            let lineage = state.read_lineage(Role::B).unwrap();
            assert_eq!(lineage.writer, Role::A);
            assert_eq!(lineage.task_trial, Some(0));
            assert_eq!(state.read_lineage(Role::B), Some(lineage));
            assert_eq!(
                before,
                [state.prefix(Role::A).clone(), state.prefix(Role::B).clone()]
            );
            assert_eq!(credits, [state.credits(Role::A), state.credits(Role::B)]);
            break;
        }
    }
}

// Catalog certainty on DataFlip is nominal evidence, never true-model grading.
#[test]
fn dataflip_retains_nominal_catalog_identification_without_true_model_grade() {
    let p = Protocol::new(Role::A, PolicyKind::Adaptive);
    let c = CompiledPolicy::build(&p, OwnPrior::Uniform).unwrap();
    let replay = Replay::build(&p, &c).unwrap();
    let e = run_episode(&p, Environment::DataFlip, 0, &c, &replay).unwrap();
    let actor = e.model_projections[0].as_ref().unwrap();
    assert_eq!(
        actor.first_identification,
        Some(Checkpoint::AfterSlot(Position {
            phase: Phase::Live { trial: 0 },
            round: 3,
            slot: 1
        }))
    );
    assert_eq!(actor.spent_to_identify, Some(7));
    assert_eq!(actor.true_model_mass, None);
    assert_eq!(actor.uniquely_identified, None);
    let responder = e.model_projections[1].as_ref().unwrap();
    assert_eq!(
        responder.first_identification,
        Some(Checkpoint::Boundary(Boundary::ProbeChoice { completed: 0 }))
    );
    assert_eq!(responder.spent_to_identify, Some(0));
}

// Paired differences are computed on matching actual sequences, with failure
// costs retained and terminal means unavailable when either policy fails.
#[test]
fn paired_policy_differences_validate_matching_panels_and_partial_failure_costs() {
    let p = Protocol::new(Role::B, PolicyKind::Adaptive);
    let q = Protocol::new(Role::B, PolicyKind::InspectOnly);
    let c = CompiledPolicy::build(&p, OwnPrior::Uniform).unwrap();
    let r = Replay::build(&p, &c).unwrap();
    let d = CompiledPolicy::build(&q, OwnPrior::Uniform).unwrap();
    let s = Replay::build(&q, &d).unwrap();
    let a = evaluate_panel(
        &p,
        Environment::InFamily(Mechanism::SharedPersistent),
        &c,
        &r,
    )
    .unwrap();
    let b = evaluate_panel(&q, a.environment, &d, &s).unwrap();
    let delta = compare_panels(&a, &b).unwrap();
    assert_eq!(
        delta.difference.spent,
        [
            ScoreFraction::new(0, 1).unwrap(),
            ScoreFraction::new(-12, 1).unwrap()
        ]
    );
    assert_eq!(
        delta.difference.net,
        [
            Some(ScoreFraction::new(24, 1).unwrap()),
            Some(ScoreFraction::new(12, 1).unwrap())
        ]
    );
    let mut reordered = b.clone();
    reordered.episodes.reverse();
    assert_eq!(delta, compare_panels(&a, &reordered).unwrap());
    let mut duplicate = b.clone();
    duplicate.episodes[1] = duplicate.episodes[0].clone();
    assert!(compare_panels(&a, &duplicate).is_err());
    let mut wrong = b.clone();
    wrong.protocol.experimenter = Role::A;
    assert!(compare_panels(&a, &wrong).is_err());
    let mut wrong = b.clone();
    wrong.protocol.ids.surface = "other".into();
    assert!(compare_panels(&a, &wrong).is_err());
    let mut missing = b.clone();
    missing.episodes.pop();
    assert!(compare_panels(&a, &missing).is_err());
    let a = evaluate_panel(&p, Environment::DataFlip, &c, &r).unwrap();
    let b = evaluate_panel(&q, Environment::DataFlip, &d, &s).unwrap();
    let delta = compare_panels(&a, &b).unwrap();
    assert_eq!(
        delta.difference.spent,
        [
            ScoreFraction::new(19, 1).unwrap(),
            ScoreFraction::new(17, 1).unwrap()
        ]
    );
    assert_eq!(delta.difference.net, [None, None]);
    assert_eq!(delta.difference.group_net, None);
    assert_eq!(
        b.episodes[0].catalog_discoveries[0]
            .as_ref()
            .unwrap()
            .source,
        CatalogCertaintySource::Supplied
    );
}

// The outer actual-mechanism distribution is not the informed Known Agent prior.
#[test]
fn uniform_actual_mechanism_summary_rejects_pooling_and_missing_models() {
    for kind in [PolicyKind::Adaptive, PolicyKind::Known] {
        let p = Protocol::new(Role::B, kind);
        let mut panels = Vec::new();
        if kind == PolicyKind::Known {
            for model in Mechanism::ALL {
                let c = CompiledPolicy::build(&p, OwnPrior::PointMass(model)).unwrap();
                let replay = Replay::build(&p, &c).unwrap();
                panels.push(evaluate_panel(&p, Environment::InFamily(model), &c, &replay).unwrap());
            }
        } else {
            let c = CompiledPolicy::build(&p, OwnPrior::Uniform).unwrap();
            let replay = Replay::build(&p, &c).unwrap();
            for model in Mechanism::ALL {
                panels.push(evaluate_panel(&p, Environment::InFamily(model), &c, &replay).unwrap());
            }
        }
        let summary = uniform_actual_mechanism_summary(&panels).unwrap();
        assert_eq!(
            summary.actual_mechanism_distribution,
            Mechanism::ALL.map(|model| ActualMechanismWeight {
                model,
                weight: Probability::new(1, 4).unwrap()
            })
        );
        assert_eq!(
            summary.aggregate.net[1],
            Some(if kind == PolicyKind::Known {
                ScoreFraction::new(15, 1).unwrap()
            } else {
                ScoreFraction::new(51, 4).unwrap()
            })
        );
        assert!(uniform_actual_mechanism_summary(&panels[..3]).is_err());
        let mut invalid = panels.clone();
        invalid[1] = invalid[0].clone();
        assert!(uniform_actual_mechanism_summary(&invalid).is_err());
        let mut invalid = panels.clone();
        invalid[0].protocol.experimenter = Role::A;
        assert!(uniform_actual_mechanism_summary(&invalid).is_err());
        let mut invalid = panels.clone();
        invalid[0].environment = Environment::DataFlip;
        assert!(uniform_actual_mechanism_summary(&invalid).is_err());
    }
}
