use super::super::diagnostics::{self, HistoryLibrary};
use super::super::*;
use std::sync::OnceLock;

fn fixture(
    role: Role,
    kind: PolicyKind,
    env: Environment,
) -> (Protocol, CompiledPolicy, Replay, Episode) {
    let protocol = Protocol::new(role, kind);
    let prior = if kind == PolicyKind::Known {
        let Environment::InFamily(model) = env else {
            panic!("known catalog control")
        };
        OwnPrior::PointMass(model)
    } else {
        OwnPrior::Uniform
    };
    let policy = CompiledPolicy::build(&protocol, prior).unwrap();
    let replay = Replay::build(&protocol, &policy).unwrap();
    let episode = run_episode(&protocol, env, 0, &policy, &replay).unwrap();
    (protocol, policy, replay, episode)
}
fn supported() -> &'static (Protocol, CompiledPolicy, Replay, Episode) {
    static FIXTURE: OnceLock<(Protocol, CompiledPolicy, Replay, Episode)> = OnceLock::new();
    FIXTURE.get_or_init(|| {
        fixture(
            Role::A,
            PolicyKind::Known,
            Environment::InFamily(Mechanism::SharedPersistent),
        )
    })
}
fn compact(episode: &Episode, replay: &Replay) -> (Vec<Prefix>, EpisodeRef) {
    let mut library = HistoryLibrary::default();
    let row = library.compact(episode, replay).unwrap();
    (library.histories, row)
}
// Removing a setting, changing census order, or adding a secondary policy must fail.
#[test]
fn frozen_census_has_exact_declared_scope() {
    let settings = frozen_settings();
    assert_eq!(settings.len(), 46);
    let kinds = [
        PolicyKind::Adaptive,
        PolicyKind::FixedThree,
        PolicyKind::NoProbe,
        PolicyKind::InspectOnly,
        PolicyKind::Known,
    ];
    let mut index = 0;
    for model in Mechanism::ALL {
        for role in [Role::A, Role::B] {
            for kind in kinds {
                assert_eq!(
                    settings[index],
                    (Protocol::new(role, kind), Environment::InFamily(model))
                );
                index += 1;
            }
        }
    }
    for role in [Role::A, Role::B] {
        for kind in [
            PolicyKind::Adaptive,
            PolicyKind::NoProbe,
            PolicyKind::InspectOnly,
        ] {
            assert_eq!(
                settings[index],
                (Protocol::new(role, kind), Environment::DataFlip)
            );
            index += 1;
        }
    }
}
// Dropping an observation or confusing equal-entry checkpoints changes the reconstructed episode.
#[test]
fn compact_histories_reconstruct_complete_episode_and_exact_checkpoints() {
    let (protocol, policy, replay, episode) = supported();
    let (histories, row) = compact(episode, replay);
    assert_eq!(histories.len(), 2);
    assert_eq!(reconstruct_episode(&histories, &row).unwrap(), *episode);
    assert!(diagnostics::episode_integrity(
        &histories,
        &row,
        protocol,
        Environment::InFamily(Mechanism::SharedPersistent),
        policy,
        replay
    )
    .unwrap());
    let boundary = row.decisions[0].prefix.clone();
    let mut queued = boundary.clone();
    queued.entry_count += 2; // private Stop plus public Stop, at the same free boundary.
    let before = boundary.reconstruct(&histories).unwrap();
    let after = queued.reconstruct(&histories).unwrap();
    assert!(before.entries.is_empty());
    assert_eq!(after.entries.len(), 2);
    assert_eq!(before.checkpoint, after.checkpoint);
    assert!(policy
        .decide(&View::from_prefix(after.clone(), replay.infer(&after).unwrap()).unwrap())
        .is_err());
    let mut renamed = episode.clone();
    renamed.histories[0].ids.surface = "different".into();
    let mut library = HistoryLibrary::default();
    let first = library.intern(episode.histories[0].clone()).unwrap();
    assert_eq!(first, library.intern(episode.histories[0].clone()).unwrap());
    assert_ne!(first, library.intern(renamed.histories[0].clone()).unwrap());
}
// A forged current value, selection, continuation tag or belief must fail even with cached success.
#[test]
fn current_payload_rejects_forged_decisions_and_beliefs() {
    let (protocol, policy, replay, episode) = supported();
    let (histories, row) = compact(episode, replay);
    let mutations: Vec<fn(&mut EpisodeRef)> = vec![
        |r| r.decisions[0].decision.alternatives[0].value = ScoreFraction::new(999, 1).unwrap(),
        |r| r.decisions[1].decision.choice = Choice::Inspect,
        |r| r.decisions[0].decision.continuation_policy = PolicyKind::Adaptive,
        |r| r.beliefs[0].belief.models = [Probability::new(1, 4).unwrap(); 4],
        |r| r.prediction_beliefs[0][0].target = Some(Probability::new(1, 2).unwrap()),
        |r| r.metrics[0].spent += 1,
        |r| r.predictions[0][0] = !r.predictions[0][0],
        |r| r.privileged[0].event.credits_after -= 1,
        |r| r.model_projections[0].as_mut().unwrap().true_model_mass = None,
        |r| r.catalog_discoveries[1].as_mut().unwrap().source = CatalogCertaintySource::Observed,
    ];
    for mutation in mutations {
        let mut forged = row.clone();
        mutation(&mut forged);
        assert!(!diagnostics::episode_integrity(
            &histories,
            &forged,
            protocol,
            Environment::InFamily(Mechanism::SharedPersistent),
            policy,
            replay
        )
        .unwrap());
    }
}
// Changing opaque reference targets, own priors, Stop evidence or peer-private entries must fail.
#[test]
fn current_payload_rejects_forged_own_histories_and_references() {
    let (protocol, policy, replay, episode) = supported();
    let (histories, row) = compact(episode, replay);
    for corrupt in 0..8 {
        let mut forged = row.clone();
        let mut local = histories.clone();
        match corrupt {
            0 => forged.decisions[0].prefix.history = u64::MAX,
            1 => forged.decisions[0].prefix.entry_count = u64::MAX,
            2 => forged.decisions[0].prefix.entry_count = 1,
            3 => local[0].own_prior = OwnPrior::Uniform,
            4 => local[1]
                .entries
                .retain(|e| !matches!(e, Entry::PublicProbeStop { .. })),
            5 => local[1].entries.push(Entry::OwnChoice {
                boundary: Boundary::TrialChoice { trial: 3 },
                choice: Choice::Inspect,
            }),
            6 => forged.histories.swap(0, 1),
            7 => forged.beliefs[0].prefix.checkpoint = Checkpoint::Prediction { trial: 3 },
            _ => unreachable!(),
        }
        assert!(!diagnostics::episode_integrity(
            &local,
            &forged,
            protocol,
            Environment::InFamily(Mechanism::SharedPersistent),
            policy,
            replay
        )
        .unwrap());
    }
}
// A subset or duplicate row cannot become a complete diagnostic by setting passed/check flags.
#[test]
fn report_census_rejects_missing_duplicate_and_forged_cached_success() {
    let mut report = diagnostics::empty_report();
    report.passed = true;
    assert!(!report_integrity(&report).unwrap());
    let (_, policy, replay, episode) = supported();
    let (histories, row) = compact(episode, replay);
    report.histories = histories;
    report.panels = frozen_settings()
        .into_iter()
        .map(|(protocol, environment)| PanelReport {
            protocol,
            environment,
            episodes: vec![row.clone(); 256],
            aggregate: Aggregate {
                valid_mass: Probability::new(1, 1).unwrap(),
                failed_mass: Probability::new(0, 1).unwrap(),
                spent: [ScoreFraction::new(24, 1).unwrap(); 2],
                reward: [Some(ScoreFraction::new(48, 1).unwrap()); 2],
                net: [Some(ScoreFraction::new(24, 1).unwrap()); 2],
                group_net: Some(ScoreFraction::new(48, 1).unwrap()),
            },
            search_stats: policy.stats().clone(),
        })
        .collect();
    assert!(!report_integrity(&report).unwrap());
    report.panels[0].episodes.pop();
    assert!(!report_integrity(&report).unwrap());
}
// Unsupported partial history retains exact charged cost and supplied nominal certainty.
#[test]
fn unsupported_and_supported_wrong_dataflip_records_validate_without_false_engineering_failure() {
    for kind in [PolicyKind::InspectOnly, PolicyKind::NoProbe] {
        let (protocol, policy, replay, episode) = fixture(Role::B, kind, Environment::DataFlip);
        let (histories, mut row) = compact(&episode, &replay);
        assert!(diagnostics::episode_integrity(
            &histories,
            &row,
            &protocol,
            Environment::DataFlip,
            &policy,
            &replay
        )
        .unwrap());
        if kind == PolicyKind::InspectOnly {
            assert_eq!(row.metrics.each_ref().map(|m| m.spent), [5, 7]);
            assert_eq!(row.model_projections, [None, None]);
            assert_eq!(
                row.catalog_discoveries[0].as_ref().unwrap().source,
                CatalogCertaintySource::Supplied
            );
            row.failure = None;
        } else {
            assert_eq!(
                row.metrics.each_ref().map(|m| m.correct),
                [Some(0), Some(0)]
            );
            assert!(row
                .model_projections
                .iter()
                .flatten()
                .all(|m| m.true_model_mass.is_none() && m.uniquely_identified.is_none()));
            row.metrics[0].correct = Some(4);
        }
        assert!(!diagnostics::episode_integrity(
            &histories,
            &row,
            &protocol,
            Environment::DataFlip,
            &policy,
            &replay
        )
        .unwrap());
    }
}

// The full proof is deliberately separate from report collection. It streams
// one reference planning group/panel at a time, retains no full oracle DOM,
// and never constructs or serializes DiagnosticReport.
#[test]
#[ignore = "requires root-reviewed source manifest and explicit panel-only precollection authorization"]
fn precollection_grid_verification() {
    grid_proof::run();
}

mod grid_proof {
    use super::diagnostics::{protocol, references};
    use super::*;
    use references::{
        boundary_value, checkpoint_value, choice_name, entries_value, environment_name,
    };
    use serde::de::{DeserializeSeed, IgnoredAny, MapAccess, SeqAccess, Visitor};
    use serde::Deserializer;
    use serde_json::{json, Value};
    use std::collections::{BTreeMap, BTreeSet};
    use std::io::{BufReader, BufWriter, Write};
    use std::path::{Path, PathBuf};

    // projection grammar is independently authored in the retained reference;
    // decoding it is separate from running the production kernel/planner.
    fn number(v: &Value) -> u8 {
        u8::try_from(v.as_u64().unwrap()).unwrap()
    }
    fn phase(v: &Value) -> Phase {
        if v == -1 {
            Phase::Calibration
        } else {
            Phase::Live { trial: number(v) }
        }
    }
    fn boundary(kind: &Value, value: &Value) -> Boundary {
        match kind.as_str().unwrap() {
            "probe" => Boundary::ProbeChoice {
                completed: number(value),
            },
            "trial" => Boundary::TrialChoice {
                trial: number(value),
            },
            _ => panic!("reference boundary"),
        }
    }
    fn choice(v: &Value) -> Choice {
        match v.as_str().unwrap() {
            "Continue" => Choice::ContinueProbe,
            "Stop" => Choice::StopProbing,
            "Inspect" => Choice::Inspect,
            "Attempt" => Choice::AttemptCommunication,
            _ => panic!("reference choice"),
        }
    }
    fn entry(v: &Value) -> Entry {
        use crate::shared_surface::Outcome;
        match v[0].as_str().unwrap() {
            "Choice" => Entry::OwnChoice {
                boundary: boundary(&v[1], &v[2]),
                choice: choice(&v[3]),
            },
            "Stop" => Entry::PublicProbeStop {
                completed: number(&v[1]),
            },
            "Reset" => Entry::Physical(LocalEntry::Reset {
                phase: phase(&v[1]),
            }),
            "Bit" => Entry::Physical(LocalEntry::PrivateBit {
                trial: number(&v[1]),
                bit: v[2] == 1,
            }),
            "Act" => Entry::Physical(LocalEntry::Action(Event {
                position: Position {
                    phase: phase(&v[1]),
                    round: number(&v[2]),
                    slot: number(&v[3]),
                },
                action: match v[4][0].as_str().unwrap() {
                    "Write" => Action::Write(serde_json::from_value(v[4][1].clone()).unwrap()),
                    "Read" => Action::Read,
                    "Wait" => Action::Wait,
                    "Inspect" => Action::InspectOwnTarget,
                    _ => panic!("reference action"),
                },
                outcome: match v[5][0].as_str().unwrap() {
                    "Read" => Outcome::Read(serde_json::from_value(v[5][1].clone()).unwrap()),
                    "Accepted" => Outcome::Accepted,
                    "Waited" => Outcome::Waited,
                    "Inspected" => Outcome::Inspected(v[5][1] == 1),
                    _ => panic!("reference outcome"),
                },
                credits_after: number(&v[6]),
            })),
            _ => panic!("reference entry"),
        }
    }
    fn prefix(row: &Value, protocol: &Protocol, prior: OwnPrior) -> Prefix {
        Prefix {
            role: protocol.experimenter,
            ids: protocol.ids.clone(),
            own_prior: prior,
            checkpoint: Checkpoint::Boundary(boundary(&row["stage"][0], &row["stage"][1])),
            entries: row["prefix"]
                .as_array()
                .unwrap()
                .iter()
                .map(entry)
                .collect(),
        }
    }
    fn assert_belief(actual: &Belief, expected: &Value) {
        assert_eq!(json!(actual.models), expected["models"]);
        assert_eq!(json!(actual.target), expected["target"]);
    }
    fn matches_group(row: &Value, policy: &CompiledPolicy) -> bool {
        json!(policy.protocol().experimenter) == row["experimenter"]
            && json!(policy.protocol().policy) == row["policy"]
            && match policy.own_prior() {
                OwnPrior::Uniform => row["known"].is_null(),
                OwnPrior::PointMass(model) => {
                    row["known"] == environment_name(Environment::InFamily(model))
                }
            }
    }
    fn matches_panel(row: &Value, policy: &CompiledPolicy) -> bool {
        json!(policy.protocol().experimenter) == row["experimenter"]
            && json!(policy.protocol().policy) == row["policy"]
            && match policy.own_prior() {
                OwnPrior::Uniform => true,
                OwnPrior::PointMass(model) => {
                    row["environment"] == environment_name(Environment::InFamily(model))
                }
            }
    }

    // Bounded top-level streaming. A decoded planning group is released before
    // the next, as is each panel; the entire 202 MB reference is never retained.
    struct Values<'a, F>(&'a mut F, &'static str);
    impl<'de, F: FnMut(&str, Value)> DeserializeSeed<'de> for Values<'_, F> {
        type Value = ();
        fn deserialize<D: Deserializer<'de>>(self, deserializer: D) -> Result<(), D::Error> {
            struct List<'a, F>(&'a mut F, &'static str);
            impl<'de, F: FnMut(&str, Value)> Visitor<'de> for List<'_, F> {
                type Value = ();
                fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    f.write_str("reference array")
                }
                fn visit_seq<A: SeqAccess<'de>>(self, mut sequence: A) -> Result<(), A::Error> {
                    while let Some(value) = sequence.next_element::<Value>()? {
                        self.0(self.1, value);
                    }
                    Ok(())
                }
            }
            deserializer.deserialize_seq(List(self.0, self.1))
        }
    }
    fn stream(path: &Path, mut callback: impl FnMut(&str, Value)) {
        struct Oracle<'a, F>(&'a mut F);
        impl<'de, F: FnMut(&str, Value)> Visitor<'de> for Oracle<'_, F> {
            type Value = ();
            fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str("reference object")
            }
            fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<(), A::Error> {
                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "planning" => map.next_value_seed(Values(self.0, "planning"))?,
                        "panels" => map.next_value_seed(Values(self.0, "panels"))?,
                        "source_sha256" => {
                            assert_eq!(
                                map.next_value::<String>()?,
                                "6eafbad345a3e23e312929e54ff5c9aac1538496bc1c1a73a26cd769f3300675"
                            );
                        }
                        _ => {
                            map.next_value::<IgnoredAny>()?;
                        }
                    }
                }
                Ok(())
            }
        }
        let mut deserializer = serde_json::Deserializer::from_reader(BufReader::new(
            std::fs::File::open(path).unwrap(),
        ));
        deserializer.deserialize_map(Oracle(&mut callback)).unwrap();
        deserializer.end().unwrap();
    }
    fn sha256(path: &Path) -> String {
        let output = std::process::Command::new("shasum")
            .args(["-a", "256"])
            .arg(path)
            .output()
            .unwrap();
        assert!(output.status.success());
        String::from_utf8(output.stdout)
            .unwrap()
            .split_whitespace()
            .next()
            .unwrap()
            .to_owned()
    }
    fn source_root() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .to_owned()
    }
    fn verify_manifest(path: &Path) -> Value {
        let manifest: Value =
            serde_json::from_reader(BufReader::new(std::fs::File::open(path).unwrap())).unwrap();
        let files = manifest["files"].as_object().unwrap();
        assert!(files.contains_key("Cargo.lock"));
        assert!(files.contains_key("crates/sugarscape-cli/tests/active_surface.rs"));
        assert!(
            files.contains_key("crates/sugarscape-core/src/active_surface/tests/diagnostics.rs")
        );
        for (name, hash) in files {
            assert_eq!(
                sha256(&source_root().join(name)),
                hash.as_str().unwrap(),
                "source changed: {name}"
            );
        }
        manifest
    }
    fn normalize_ids(episode: &mut Episode, ids: &Ids) {
        for prefix in &mut episode.histories {
            prefix.ids = ids.clone();
        }
        for decision in &mut episode.decisions {
            decision.prefix.ids = ids.clone();
        }
        if let Some(failure) = &mut episode.failure {
            failure.prefix.ids = ids.clone();
        }
    }
    fn compare_checkpoints(
        episode: &Episode,
        protocol: &Protocol,
        environment: Environment,
        policy: &CompiledPolicy,
        replay: &Replay,
        expected: &Value,
    ) {
        let nominal = match environment {
            Environment::InFamily(model) => model,
            Environment::DataFlip => Mechanism::SharedPersistent,
        };
        let mut priors = [OwnPrior::PointMass(nominal); 2];
        priors[protocol.experimenter.index()] = policy.own_prior();
        let mut state = EpisodeState::new(
            protocol,
            environment,
            EpisodeBits::from_index(episode.sequence).unwrap(),
            priors,
        )
        .unwrap();
        let mut snapshots = Vec::<(Value, [Prefix; 2])>::new();
        let mut next = true;
        loop {
            let own = state.prefix(protocol.experimenter);
            if matches!(
                own.checkpoint,
                Checkpoint::Boundary(_) | Checkpoint::AfterSlot(_) | Checkpoint::Prediction { .. }
            ) {
                snapshots.push((
                    checkpoint_value(own.checkpoint),
                    [Role::A, Role::B].map(|r| state.prefix(r).clone()),
                ));
            }
            if episode
                .failure
                .as_ref()
                .is_some_and(|f| state.prefix(f.role) == &f.prefix)
                || own.checkpoint == Checkpoint::Finished
            {
                break;
            }
            if next {
                let decision = policy
                    .decide(&View::from_prefix(own.clone(), replay.infer(own).unwrap()).unwrap())
                    .unwrap();
                let checkpoint = own.checkpoint;
                select_choice(&mut state, decision.choice).unwrap();
                if decision.choice == Choice::StopProbing {
                    let Checkpoint::Boundary(Boundary::ProbeChoice { completed }) = checkpoint
                    else {
                        unreachable!()
                    };
                    snapshots.push((
                        json!(["stop", completed]),
                        [Role::A, Role::B].map(|r| state.prefix(r).clone()),
                    ));
                }
            }
            next = advance_one(&mut state).unwrap().next.is_some();
        }
        for want in expected["checkpoints"].as_array().unwrap() {
            let (_, prefixes) = snapshots
                .iter()
                .find(|(key, _)| key == &want["checkpoint"])
                .unwrap();
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
                    .find_map(|e| {
                        if let Entry::Physical(LocalEntry::Action(event)) = e {
                            Some(48 - event.credits_after)
                        } else {
                            None
                        }
                    })
                    .unwrap_or(0);
                assert_eq!(json!(spent), want["spent"][i]);
                // Exact prefix reconstruction also covers Before/After and
                // free queued clocks that share entries at the same public time.
                let history = &episode.histories[i];
                let reference = PrefixRef {
                    history: 0,
                    entry_count: u64::try_from(prefixes[i].entries.len()).unwrap(),
                    checkpoint: prefixes[i].checkpoint,
                };
                assert_eq!(
                    reference
                        .reconstruct(std::slice::from_ref(history))
                        .unwrap(),
                    prefixes[i]
                );
            }
        }
    }
    fn compare_episode(
        episode: &Episode,
        expected: &Value,
        decisions: &BTreeMap<String, Value>,
        policy: &CompiledPolicy,
        replay: &Replay,
        environment: Environment,
    ) -> bool {
        assert_eq!(json!(episode.sequence), expected["sequence"]);
        assert_eq!(json!(episode.predictions), expected["predictions"]);
        for (field, metric) in [
            (
                "spent",
                (|m: &AgentMetrics| json!(m.spent)) as fn(&AgentMetrics) -> Value,
            ),
            ("correct", |m| json!(m.correct)),
            ("reward", |m| json!(m.reward)),
            ("net", |m| json!(m.net)),
            ("attempted_messages", |m| json!(m.attempted_sends)),
            ("received_peer_task_writes", |m| json!(m.lineage_reads)),
        ] {
            assert_eq!(
                json!(episode.metrics.each_ref().map(metric)),
                expected[field],
                "{field}"
            );
        }
        for i in 0..2 {
            let identification = &expected["first_identification"][i];
            if let Some(discovery) = &episode.catalog_discoveries[i] {
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
                let count =
                    usize::try_from(identification["own_entry_count"].as_u64().unwrap()).unwrap();
                let spent = episode.histories[i].entries[..count]
                    .iter()
                    .rev()
                    .find_map(|e| {
                        if let Entry::Physical(LocalEntry::Action(event)) = e {
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
            let beliefs = expected["prediction_beliefs"][i].as_array().unwrap();
            assert_eq!(episode.prediction_beliefs[i].len(), beliefs.len());
            for (belief, want) in episode.prediction_beliefs[i].iter().zip(beliefs) {
                assert_belief(belief, want);
            }
            if let Some(projection) = &episode.model_projections[i] {
                assert_eq!(
                    json!(projection.final_belief.models),
                    expected["final_model_beliefs"][i]
                );
                assert_eq!(
                    projection.first_identification.map(checkpoint_value),
                    if identification.is_null() {
                        None
                    } else {
                        Some(identification["checkpoint"].clone())
                    }
                );
                if matches!(environment, Environment::DataFlip) {
                    assert_eq!(projection.true_model_mass, None);
                    assert_eq!(projection.uniquely_identified, None);
                }
            } else {
                assert!(expected["final_model_beliefs"][i].is_null());
            }
        }
        let decision_ids = expected["decisions"].as_array().unwrap();
        assert_eq!(episode.decisions.len(), decision_ids.len());
        for (actual, id) in episode.decisions.iter().zip(decision_ids) {
            let want = &decisions[id.as_str().unwrap()];
            assert_eq!(entries_value(&actual.prefix.entries), want["prefix"]);
            let Checkpoint::Boundary(boundary) = actual.prefix.checkpoint else {
                panic!("decision clock")
            };
            let (kind, n) = boundary_value(boundary);
            assert_eq!(json!([kind, n]), want["stage"]);
            assert_eq!(choice_name(actual.decision.choice), want["chosen"]);
            assert_belief(&replay.infer(&actual.prefix).unwrap(), &want["belief"]);
            assert_eq!(
                replay.infer(&actual.prefix).unwrap(),
                policy.infer(&actual.prefix).unwrap()
            );
            assert_eq!(
                actual.decision.alternatives.len(),
                want["alternatives"].as_object().unwrap().len()
            );
            for alternative in &actual.decision.alternatives {
                assert_eq!(
                    json!(alternative.value),
                    want["alternatives"][choice_name(alternative.choice)]
                );
            }
        }
        if let Some(failure) = &episode.failure {
            assert_eq!(json!(failure.role.index()), expected["failure"]["role"]);
            assert_eq!(
                checkpoint_value(failure.prefix.checkpoint),
                expected["failure"]["checkpoint"]
            );
            assert_eq!(json!(failure.spent), expected["failure"]["spent"]);
            assert_belief(
                failure.last_supported.as_ref().unwrap(),
                &expected["failure"]["last_supported"],
            );
        } else {
            assert!(expected["failure"].is_null());
        }
        if !expected["histories"].is_null() {
            for i in 0..2 {
                assert_eq!(
                    entries_value(&episode.histories[i].entries),
                    expected["histories"][i]
                );
            }
            compare_checkpoints(
                episode,
                policy.protocol(),
                environment,
                policy,
                replay,
                expected,
            );
            true
        } else {
            false
        }
    }

    pub(super) fn run() {
        let reference = PathBuf::from(
            std::env::var("ACTIVE_SURFACE_REFERENCE")
                .expect("absolute reviewed reference-v2.json path"),
        );
        let manifest_path = PathBuf::from(
            std::env::var("ACTIVE_SURFACE_PROOF_MANIFEST")
                .expect("frozen full-source manifest path"),
        );
        let receipt_path = PathBuf::from(
            std::env::var("ACTIVE_SURFACE_PROOF_RECEIPT").expect("exclusive proof receipt path"),
        );
        assert!(
            reference.is_absolute() && manifest_path.is_absolute() && receipt_path.is_absolute()
        );
        let manifest = verify_manifest(&manifest_path);
        assert_eq!(
            sha256(&reference),
            "e39b6bb0f76a626892223343f7c41a9843a8622363bb421fa54d0aedfe2e9f83"
        );
        assert_eq!(
            sha256(&reference.with_file_name("reference-v2.py")),
            "6eafbad345a3e23e312929e54ff5c9aac1538496bc1c1a73a26cd769f3300675"
        );
        // Reserve before doing work: a prior receipt can never be overwritten.
        let receipt_file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&receipt_path)
            .unwrap();
        let started = std::time::Instant::now();
        let settings = frozen_settings();
        let portable = references::Reference::read().unwrap();
        let mut verified = BTreeSet::<usize>::new();
        let mut groups = Vec::<(Protocol, OwnPrior)>::new();
        let mut receipts = vec![Value::Null; settings.len()];
        let mut decision_count = 0;
        let mut alternative_count = 0;
        let mut trace_count = 0;
        let mut renamed_count = 0;
        let mut summaries = Vec::<Panel>::new();
        protocol::for_each_engine(&settings, |_, policy, replay| {
            let key = (policy.protocol().clone(), policy.own_prior());
            if groups.contains(&key) { return Ok(()); }
            groups.push(key);
            let mut renamed_protocol = policy.protocol().clone();
            renamed_protocol.ids = Ids { surface: "x7".into(), agents: ["z9".into(), "q4".into()] };
            let renamed_policy = CompiledPolicy::build(&renamed_protocol, policy.own_prior()).unwrap();
            let renamed_replay = Replay::build(&renamed_protocol, &renamed_policy).unwrap();
            let mut decisions = BTreeMap::<String, Value>::new();
            let mut group_seen = false;
            stream(&reference, |section, row| {
                if section == "planning" && matches_group(&row, policy) {
                    assert!(!group_seen); group_seen = true;
                    for record in row["records"].as_array().unwrap() {
                        let own = prefix(record, policy.protocol(), policy.own_prior());
                        let belief = policy.infer(&own).unwrap();
                        assert_belief(&belief, &record["belief"]);
                        let actual = policy.decide(&View::from_prefix(own, belief).unwrap()).unwrap();
                        assert_eq!(choice_name(actual.choice), record["chosen"]);
                        assert_eq!(json!(actual.continuation_policy), record["continuation_policy"]);
                        assert_eq!(actual.alternatives.len(), record["alternatives"].as_object().unwrap().len());
                        for alternative in &actual.alternatives {
                            assert_eq!(json!(alternative.value), record["alternatives"][choice_name(alternative.choice)]);
                            alternative_count += 1;
                        }
                        assert!(decisions.insert(record["id"].as_str().unwrap().to_owned(), record.clone()).is_none());
                        decision_count += 1;
                    }
                } else if section == "panels" && matches_panel(&row, policy) {
                    assert!(group_seen);
                    let (index, (protocol, environment)) = settings.iter().enumerate().find(|(_, (p, env))| *p == *policy.protocol() && environment_name(*env) == row["environment"]).unwrap();
                    assert!(verified.insert(index));
                    let mut panel = evaluate_panel(protocol, *environment, policy, replay).unwrap();
                    assert!(portable.validate_panel(&panel).unwrap());
                    assert!(references::Reference::validate_row(&panel, &row).unwrap());
                    let expected = row["episodes"].as_array().unwrap();
                    assert_eq!(expected.len(), 256);
                    assert_eq!(panel.episodes.iter().map(|e| e.sequence).collect::<BTreeSet<_>>(), (0..256).collect());
                    let mut renamed = evaluate_panel(&renamed_protocol, *environment, &renamed_policy, &renamed_replay).unwrap();
                    assert_eq!(panel.aggregate, renamed.aggregate);
                    assert_eq!(policy.stats(), renamed_policy.stats());
                    let mut library = HistoryLibrary::default();
                    let mut normalized_file = std::fs::OpenOptions::new().write(true).create_new(true).open(receipt_path.with_extension(format!("setting-{index}-typed.ndjson"))).unwrap();
                    let raw_path = receipt_path.with_extension(format!("setting-{index}-renamed.ndjson"));
                    let mut raw_file = BufWriter::new(std::fs::OpenOptions::new().write(true).create_new(true).open(&raw_path).unwrap());
                    for ((episode, want), other) in panel.episodes.iter().zip(expected).zip(&mut renamed.episodes) {
                        trace_count += usize::from(compare_episode(episode, want, &decisions, policy, replay, *environment));
                        serde_json::to_writer(&mut raw_file, &other).unwrap(); raw_file.write_all(b"\n").unwrap();
                        normalize_ids(other, &protocol.ids);
                        assert_eq!(episode, other);
                        let compact = library.compact(episode, replay).unwrap();
                        assert_eq!(reconstruct_episode(&library.histories, &compact).unwrap(), *episode);
                        assert!(diagnostics::episode_matches(&library.histories, &compact, episode, replay).unwrap());
                        serde_json::to_writer(&mut normalized_file, episode).unwrap(); normalized_file.write_all(b"\n").unwrap();
                        renamed_count += 1;
                    }
                    raw_file.flush().unwrap(); normalized_file.flush().unwrap();
                    let normalized_path = receipt_path.with_extension(format!("setting-{index}-typed.ndjson"));
                    receipts[index] = json!({"index": index, "protocol": protocol, "environment": environment, "episodes": 256,
                        "sequence_first": 0, "sequence_last": 255, "aggregate": panel.aggregate, "search_stats": policy.stats(),
                        "compact_histories": library.histories.len(), "typed_original_normalized_sha256": sha256(&normalized_path),
                        "raw_renamed_sha256": sha256(&raw_path), "renamed_ids": renamed_protocol.ids,
                        "ids_normalized_at": ["histories[*].ids", "decisions[*].prefix.ids", "failure.prefix.ids"]});
                    // Pair comparisons retain only validated per-sequence outcome
                    // rows; no full report or duplicate global raw history library.
                    for episode in &mut panel.episodes {
                        episode.histories.each_mut().iter_mut().for_each(|p| p.entries.clear());
                        episode.decisions.clear(); episode.privileged.clear();
                        episode.predictions.each_mut().iter_mut().for_each(|p| p.clear());
                        episode.prediction_beliefs.each_mut().iter_mut().for_each(|p| p.clear());
                        if let Some(failure) = &mut episode.failure { failure.prefix.entries.clear(); }
                    }
                    summaries.push(panel);
                }
            });
            assert!(group_seen);
            Ok(())
        }).unwrap();
        assert_eq!(groups.len(), 16);
        assert_eq!(verified.len(), 46);
        assert_eq!(decision_count, 93928);
        assert_eq!(alternative_count, 187816);
        assert_eq!(trace_count, 322);
        assert_eq!(renamed_count, 11776);
        let fixture: Value =
            serde_json::from_str(include_str!("fixtures/evaluation-reference.json")).unwrap();
        let pairs = fixture["paired_comparisons"].as_array().unwrap();
        assert_eq!(pairs.len(), 86);
        for pair in pairs {
            let find = |field: &str| {
                summaries
                    .iter()
                    .find(|p| {
                        json!(p.protocol.experimenter) == pair["experimenter"]
                            && environment_name(p.environment) == pair["environment"]
                            && json!(p.protocol.policy) == pair[field]
                    })
                    .unwrap()
            };
            assert_eq!(
                json!(
                    compare_panels(find("left_policy"), find("right_policy"))
                        .unwrap()
                        .difference
                ),
                pair["difference"]
            );
        }
        assert_eq!(verify_manifest(&manifest_path), manifest);
        let receipt = json!({"version": "active-surface-precollection-grid-v1", "report_constructed": false,
            "source_manifest_sha256": sha256(&manifest_path), "source_manifest": manifest,
            "reference_source_sha256": "6eafbad345a3e23e312929e54ff5c9aac1538496bc1c1a73a26cd769f3300675",
            "reference_output_sha256": "e39b6bb0f76a626892223343f7c41a9843a8622363bb421fa54d0aedfe2e9f83",
            "settings": receipts, "decision_records": decision_count, "alternative_values": alternative_count,
            "episodes": renamed_count, "designated_traces": trace_count, "paired_comparisons": pairs.len(),
            "elapsed_seconds": started.elapsed().as_secs_f64(), "passed": true});
        let mut writer = BufWriter::new(receipt_file);
        serde_json::to_writer_pretty(&mut writer, &receipt).unwrap();
        writer.write_all(b"\n").unwrap();
        writer.flush().unwrap();
        println!("PASS 46 settings / 11776 episodes / 93928 decision records / 187816 alternatives / 322 full traces / 86 pairs / 11776 typed ID renames; exclusive receipt {}", receipt_path.display());
    }
}

// Invalid bit-sequence rows must be rejected at decode, before any fresh host run.
#[test]
fn compact_row_decode_rejects_sequence_outside_the_frozen_bit_grid() {
    let (_, _, replay, episode) = supported();
    let (_, row) = compact(episode, replay);
    let mut value = serde_json::to_value(row).unwrap();
    value["sequence"] = serde_json::json!(256);
    assert!(serde_json::from_value::<EpisodeRef>(value).is_err());
}
