use super::super::diagnostics::protocol;
use super::super::*;
fn incomplete_report() -> DiagnosticReport {
    DiagnosticReport {
        version: "shared-surface-diagnostic-v1".into(),
        protocol_version: 1,
        supplied_structure: vec![],
        traces: vec![],
        primary: vec![],
        secondary: vec![],
        checks: vec![],
        passed: true,
    }
}
#[test]
fn incomplete_report_never_passes_production_integrity() {
    assert!(!report_integrity(&incomplete_report()).unwrap());
}
#[test]
fn frozen_grid_covers_all_220_settings_and_56320_episodes() {
    let grid = protocol::settings();
    let counts = [
        PanelKind::Primary,
        PanelKind::Asymmetric,
        PanelKind::Restart,
        PanelKind::Stale,
        PanelKind::DataFlip,
    ]
    .map(|kind| grid.iter().filter(|row| row.0 == kind).count());
    assert_eq!(counts, [48, 32, 64, 64, 12]);
    assert_eq!(grid.len() * 256, 56320);
}

fn bounded_fixture() -> &'static (PanelReport, Vec<AgentTrace>, Panel) {
    static FIXTURE: std::sync::OnceLock<(PanelReport, Vec<AgentTrace>, Panel)> =
        std::sync::OnceLock::new();
    FIXTURE.get_or_init(|| {
        let (kind, protocol, environment, _) = protocol::settings().remove(0);
        let panel = evaluate_panel(&protocol, environment).unwrap();
        let traces: Vec<_> = panel
            .episodes
            .iter()
            .flat_map(|e| e.local.clone())
            .collect();
        let episodes = panel
            .episodes
            .iter()
            .map(|e| EpisodeReference {
                sequence: e.sequence,
                agent_traces: [u64::from(e.sequence) * 2, u64::from(e.sequence) * 2 + 1],
                privileged: e.privileged.clone(),
                model_projections: super::super::diagnostics::projections(&e.metrics).unwrap(),
                metrics: e.metrics.clone(),
                failure: e.failure.clone(),
                group_net_utility: e.group_net_utility,
            })
            .collect();
        (
            PanelReport {
                kind,
                protocol,
                environment,
                origin: None,
                scope: super::super::diagnostics::scope(&panel).unwrap(),
                episodes,
                aggregate: panel.aggregate.clone(),
                sensitivity: panel.sensitivity.clone(),
            },
            traces,
            panel,
        )
    })
}
#[test]
fn bounded_payload_accepts_full_histories_and_rejects_edited_current_values() {
    let (panel, traces, expected) = bounded_fixture();
    assert!(super::super::diagnostics::validate_panel_payload(panel, traces, expected).unwrap());
    for edit in 0..14 {
        let mut changed = panel.clone();
        let mut changed_traces = traces.clone();
        match edit {
            0 => changed.episodes[0].agent_traces[0] = u64::MAX,
            1 => {
                changed.episodes.pop();
            }
            2 => changed.episodes.swap(0, 1),
            3 => changed.episodes[0].sequence = 256,
            4 => changed.episodes[0].metrics[0].spent += 1,
            5 => changed_traces[0].steps[0].credits -= 1,
            6 => changed.sensitivity[0].prediction_changed = Some(false),
            7 => changed.aggregate.agents[0].expected_credits = ScoreFraction::new(0, 1).unwrap(),
            8 => changed.scope.valid_mass = Probability::new(0, 1).unwrap(),
            9 => {
                changed.episodes[0].model_projections[0]
                    .as_mut()
                    .unwrap()
                    .useful_channel = Probability::new(1, 1).unwrap()
            }
            10 => changed.episodes[0].privileged[0].event.credits_after -= 1,
            11 => {
                changed.episodes[0].privileged[0].read_lineage = Some(Lineage {
                    writer: Role::B,
                    write_position: Position {
                        phase: Phase::Live { trial: 0 },
                        round: 1,
                        slot: 1,
                    },
                    symbol: Symbol::Data0,
                    task_trial: Some(0),
                })
            }
            12 => changed_traces[0].steps[0].prefix.ids.surface = "foreign".into(),
            13 => {
                changed_traces[0].steps[0].belief.as_mut().unwrap().models[0] =
                    Probability::new(1, 1).unwrap()
            }
            _ => unreachable!(),
        }
        assert!(
            !super::super::diagnostics::validate_panel_payload(&changed, &changed_traces, expected)
                .unwrap(),
            "edit {edit}"
        );
    }
}
#[test]
fn report_wire_rejects_unknown_fields_and_invalid_exact_fractions() {
    let mut value = serde_json::to_value(incomplete_report()).unwrap();
    value["fixture_scope"] = true.into();
    assert!(serde_json::from_value::<DiagnosticReport>(value).is_err());
    assert!(serde_json::from_str::<Probability>(r#"{"numerator":2,"denominator":1}"#).is_err());
    assert!(serde_json::from_str::<ScoreFraction>(r#"{"numerator":48,"denominator":1}"#).is_ok());
    assert!(serde_json::from_str::<ScoreFraction>(r#"{"numerator":1,"denominator":0}"#).is_err());
}
#[test]
fn independent_reference_rejects_wrong_credits_and_uniform_catalog_beliefs() {
    let (_, _, panel) = bounded_fixture();
    assert!(
        super::super::diagnostics::references::matches_panel(panel, PanelKind::Primary, None)
            .unwrap()
    );
    let mut changed = panel.clone();
    changed.aggregate.agents[0].expected_credits = ScoreFraction::new(35, 1).unwrap();
    assert!(!super::super::diagnostics::references::matches_panel(
        &changed,
        PanelKind::Primary,
        None
    )
    .unwrap());
    let mut changed = panel.clone();
    changed.episodes[0].metrics[0]
        .final_belief
        .as_mut()
        .unwrap()
        .models = [Probability::new(0, 1).unwrap(); 4];
    assert!(!super::super::diagnostics::references::matches_panel(
        &changed,
        PanelKind::Primary,
        None
    )
    .unwrap());
}

#[test]
fn lossless_interning_uses_complete_trace_and_stable_first_occurrence() {
    let (_, traces, _) = bounded_fixture();
    let mut library = super::super::diagnostics::TraceLibrary::default();
    assert_eq!(library.intern(traces[0].clone()).unwrap(), 0);
    assert_eq!(library.intern(traces[0].clone()).unwrap(), 0);
    let mut changed = traces[0].clone();
    changed.steps[0].belief.as_mut().unwrap().models = [Probability::new(0, 1).unwrap(); 4];
    assert_eq!(library.intern(changed.clone()).unwrap(), 1);
    assert_eq!(library.traces[1], changed);
    assert_eq!(library.traces[0], traces[0]);
}
#[test]
fn canonical_library_rejects_unused_duplicates_foreign_and_noncanonical_references() {
    let (_, _, panel) = bounded_fixture();
    let mut library = super::super::diagnostics::TraceLibrary::default();
    let row = library
        .panel(panel.clone(), PanelKind::Primary, None)
        .unwrap();
    let mut report = incomplete_report();
    report.primary = vec![row];
    report.traces = library.traces;
    assert!(super::super::diagnostics::canonical_library(&report).unwrap());
    for mutation in 0..5 {
        let mut changed = report.clone();
        match mutation {
            0 => changed.traces.push(changed.traces[0].clone()),
            1 => changed.primary[0].episodes[0].agent_traces[0] = u64::MAX,
            2 => changed.primary[0].episodes[0].agent_traces.swap(0, 1),
            3 => changed.primary[0].episodes[1].sequence = 0,
            4 => {
                changed.traces[1] = changed.traces[0].clone();
            }
            _ => unreachable!(),
        }
        assert!(
            !super::super::diagnostics::canonical_library(&changed).unwrap(),
            "mutation {mutation}"
        );
    }
}
#[test]
fn episode_reference_wire_rejects_out_of_range_sequence() {
    let (panel, _, _) = bounded_fixture();
    let mut wire = serde_json::to_value(&panel.episodes[0]).unwrap();
    wire["sequence"] = 256.into();
    assert!(serde_json::from_value::<EpisodeReference>(wire).is_err());
}
#[test]
fn cached_panel_evaluation_retains_all_costs_and_sensitivity() {
    let (_, _, panel) = bounded_fixture();
    let ensemble = Ensemble::build(&panel.protocol).unwrap();
    assert_eq!(
        evaluate_panel_with_ensemble(&panel.protocol, panel.environment, &ensemble).unwrap(),
        *panel
    );
}

// Complete panel verification is deliberately separate from diagnose and CLI
// collection: no DiagnosticReport is constructed or serialized here.
#[test]
fn precollection_grid_verification() {
    use super::super::diagnostics::{normalized_renamed_panel, references, ReplayCache};
    let evidence =
        std::env::var_os("SHARED_SURFACE_VERIFICATION_DIR").map(std::path::PathBuf::from);
    let source = evidence.as_ref().map(|_| {
        std::env::var("SHARED_SURFACE_SOURCE_SHA256")
            .expect("source manifest hash required when retaining verification")
    });
    let mut cache = ReplayCache::new();
    let mut rows = 0usize;
    let mut renamed_rows = 0usize;
    for (index, (kind, protocol, environment, old)) in protocol::settings().into_iter().enumerate()
    {
        let panel = cache.panel(&protocol, environment).unwrap();
        assert!(
            references::matches_panel(&panel, kind.clone(), old).unwrap(),
            "setting {index}: {kind:?} {protocol:?} {environment:?}"
        );
        let origin = cache.origin(old).unwrap();
        if let Some(origin) = &origin {
            assert_eq!(origin.spent, [6, 6]);
            for trace in &origin.acquisition {
                assert_eq!(trace.own_prior, OwnPrior::Uniform);
                assert!(trace.steps.iter().all(|s| s
                    .prefix
                    .entries
                    .iter()
                    .all(|e| !matches!(e, LocalEntry::PrivateBit { .. }))));
            }
        }
        rows += panel.episodes.len();
        let mut verification = serde_json::json!({"setting":index,"kind":kind,"protocol":protocol,"environment":environment,"old_mechanism":old,"episode_rows":panel.episodes.len(),"independent_reference_matches":true,"aggregate":panel.aggregate,"origin":origin,"source_manifest_sha256":source});
        if kind == PanelKind::Primary {
            let renamed = normalized_renamed_panel(&panel, &mut cache).unwrap();
            assert_eq!(
                renamed, panel,
                "normalized complete typed Panel, setting {index}"
            );
            renamed_rows += renamed.episodes.len();
            if evidence.is_some() {
                verification["renaming"] = serde_json::json!({"actual_rows":renamed.episodes.len(),"complete_typed_panel_equality":true,"original_ids":protocol.ids,"renamed_ids":Ids{surface:"renamed-object-Q".into(),agents:["renamed-role-Z".into(),"renamed-role-X".into()]},"original_normalized_panel_sha256":panel_sha256(&panel),"renamed_normalized_panel_sha256":panel_sha256(&renamed)});
            }
        }
        if let Some(directory) = &evidence {
            let file = std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(directory.join(format!("setting-{index:03}.json")))
                .unwrap();
            serde_json::to_writer_pretty(file, &verification).unwrap();
        }
        eprintln!(
            "verified setting {index:03}; total actual rows {rows}; renamed rows {renamed_rows}"
        );
    }
    assert_eq!((rows, renamed_rows), (56320, 12288));
}

fn panel_sha256(panel: &Panel) -> serde_json::Value {
    use std::process::{Command, Stdio};
    let mut child = Command::new("shasum")
        .args(["-a", "256"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    struct Counted<W> {
        writer: W,
        bytes: u64,
    }
    impl<W: std::io::Write> std::io::Write for Counted<W> {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            let n = self.writer.write(bytes)?;
            self.bytes += n as u64;
            Ok(n)
        }
        fn flush(&mut self) -> std::io::Result<()> {
            self.writer.flush()
        }
    }
    let mut counted = Counted {
        writer: std::io::BufWriter::new(child.stdin.take().unwrap()),
        bytes: 0,
    };
    serde_json::to_writer(&mut counted, panel).unwrap();
    std::io::Write::flush(&mut counted).unwrap();
    let bytes = counted.bytes;
    drop(counted);
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    serde_json::json!({"sha256":stdout.split_whitespace().next().unwrap(),"serialized_bytes":bytes})
}

#[test]
fn bounded_adversarial_and_ungraded_controls_match_independent_references() {
    use super::super::diagnostics::{references, ReplayCache};
    let mut cache = ReplayCache::new();
    let grid = protocol::settings();
    // Known SP, NoCommunication, asymmetric self-retained data, stale charged
    // contradiction, and the knowingly false SP control in actual DataFlip.
    for index in [10usize, 11, 48, 148, 218] {
        let (kind, p, e, old) = &grid[index];
        let panel = cache.panel(p, *e).unwrap();
        assert!(
            references::matches_panel(&panel, kind.clone(), *old).unwrap(),
            "setting {index}"
        );
        if *e == Environment::DataFlip && p.pair.roles == [Knowledge::Known; 2] {
            assert_eq!(panel.aggregate.failed, 0);
            assert_eq!(
                panel.aggregate.agents[0].expected_accuracy,
                Some(Probability::new(0, 1).unwrap())
            );
            assert!(panel.episodes.iter().all(|episode| episode
                .metrics
                .iter()
                .all(|m| m.true_model_probability.is_none())));
        }
        if p.pair.roles == [Knowledge::NoCommunication; 2] {
            assert!(panel
                .episodes
                .iter()
                .all(|episode| episode
                    .metrics
                    .iter()
                    .all(|m| m.true_model_probability.is_none()
                        && m.final_belief.as_ref().unwrap().models
                            == [Probability::new(1, 4).unwrap(); 4])));
        }
        if panel.aggregate.failed > 0 {
            assert!(panel
                .aggregate
                .agents
                .iter()
                .all(|m| m.expected_accuracy.is_none()
                    && m.expected_correct_count.is_none()
                    && m.expected_net.is_none()
                    && m.expected_gross.is_none()
                    && m.benefit_gross.is_none()
                    && m.benefit_net.is_none()));
            assert!(panel.aggregate.expected_group_net.is_none());
        }
    }
}

#[test]
fn designated_independent_histories_reject_edited_early_belief_and_observation() {
    use super::super::diagnostics::{references, ReplayCache};
    let (_, protocol, environment, _) = protocol::settings().remove(9);
    let panel = ReplayCache::new().panel(&protocol, environment).unwrap();
    assert!(references::matches_designated_histories(&panel).unwrap());
    let mut changed = panel.clone();
    changed.episodes[42].local[0].steps[0]
        .belief
        .as_mut()
        .unwrap()
        .models[0] = Probability::new(1, 1).unwrap();
    assert!(!references::matches_designated_histories(&changed).unwrap());
    let mut changed = panel.clone();
    let step = changed.episodes[42].local[0]
        .steps
        .iter_mut()
        .find(|s| {
            matches!(
                s.prefix.checkpoint,
                Checkpoint::AfterSlot(Position {
                    phase: Phase::Calibration,
                    round: 1,
                    slot: 1
                })
            )
        })
        .unwrap();
    if let Some(LocalEntry::Action(event)) = step.prefix.entries.last_mut() {
        event.outcome = Outcome::Waited;
    } else {
        panic!("own event missing")
    }
    assert!(!references::matches_designated_histories(&changed).unwrap());
}
