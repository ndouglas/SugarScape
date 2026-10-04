use super::super::burrow::{manifest, Condition};
use super::*;
use sugarscape_core::burrow::{run_episode, LabConfig, RunOptions, Transport};

fn growing_controls() -> (Vec<Condition>, Vec<String>, BTreeMap<RunKey, Episode>) {
    let m = manifest().unwrap();
    let conditions: Vec<_> = m
        .conditions
        .iter()
        .filter(|c| matches!(c.config.fixture, Fixture::Growing { .. }))
        .cloned()
        .collect();
    let seeds = vec!["7".into(), "8".into()];
    let records = conditions
        .iter()
        .flat_map(|c| {
            seeds.iter().map(move |s: &String| {
                let key = RunKey {
                    condition: c.id.clone(),
                    seed: s.clone(),
                };
                let e = run_episode(
                    c.config.clone(),
                    s.parse().unwrap(),
                    RunOptions {
                        ticks: 16,
                        sample_every: 4,
                    },
                )
                .unwrap();
                (key, e)
            })
        })
        .collect();
    (conditions, seeds, records)
}
#[test]
fn burrow_report_projection_ignores_configuration_fingerprints() {
    let options = RunOptions {
        ticks: 16,
        sample_every: 4,
    };
    let blind = LabConfig::default();
    let responsive = LabConfig {
        cue: Cue::Responsive,
        ..blind.clone()
    };
    let a = run_episode(blind, 7, options.clone()).unwrap();
    let b = run_episode(responsive, 7, options).unwrap();
    assert_eq!(physical_projection(&a), physical_projection(&b));
    assert_ne!(a.frames, b.frames);
}
#[test]
fn burrow_report_all_direct_controls_and_weight_one_transports() {
    let (conditions, seeds, records) = growing_controls();
    let controls = check_controls(&conditions, &seeds, &records).unwrap();
    assert_eq!(controls.len(), 7); // five direct strata/weights, two weight-one transports
    assert!(controls.iter().all(|c| c.identical && c.seeds_checked == 2));
}
#[test]
fn burrow_report_tampered_direct_trajectory_stops_analysis() {
    let (conditions, seeds, mut records) = growing_controls();
    records
        .get_mut(&RunKey {
            condition: "primary.direct.responsive".into(),
            seed: "7".into(),
        })
        .unwrap()
        .choices
        .clear();
    assert!(check_controls(&conditions, &seeds, &records)
        .unwrap_err()
        .contains("seed 7"));
}
#[test]
fn burrow_report_six_signed_estimates_per_separate_workforce_and_exact_seeds() {
    let m = manifest().unwrap();
    let maps: BTreeMap<_, _> = m
        .conditions
        .iter()
        .filter(|c| matches!(c.config.fixture, Fixture::Growing { .. }))
        .map(|c| {
            let base = match (c.config.transport, c.config.cue) {
                (Transport::Direct, _) => 1.0,
                (Transport::Relay, Cue::Blind) => 3.0,
                (Transport::Relay, Cue::Responsive) => 2.0,
            };
            (
                c.id.clone(),
                (10001..=10040)
                    .map(|s| (s, base))
                    .collect::<BTreeMap<_, _>>(),
            )
        })
        .collect();
    let estimates = estimate_contrasts(&m, &maps, "excavation").unwrap();
    let disposal = estimate_contrasts(&m, &maps, "disposal").unwrap();
    assert_eq!(
        estimates
            .iter()
            .chain(&disposal)
            .filter(|r| r.primary)
            .count(),
        6
    );
    assert_eq!(estimates.len(), 16); // three signed plus redundant interaction, four strata
    for stratum in ["primary", "workforce.2", "workforce.4", "workforce.16"] {
        let rows: Vec<_> = estimates.iter().filter(|r| r.stratum == stratum).collect();
        assert_eq!(rows.len(), 4);
        assert_eq!(
            rows.iter().map(|r| r.mean).collect::<Vec<_>>(),
            vec![-1.0, 2.0, 1.0, -1.0]
        );
        assert!(rows.iter().all(|r| r.n == 40
            && r.differences.first().unwrap().0 == "10001"
            && r.differences.last().unwrap().0 == "10040"));
    }
    let mut missing = maps.clone();
    missing
        .get_mut("primary.relay.responsive")
        .unwrap()
        .remove(&10040);
    assert!(estimate_contrasts(&m, &missing, "excavation").is_err());
    let mut shifted = maps;
    let shifted_map = shifted.get_mut("primary.relay.responsive").unwrap();
    shifted_map.remove(&10040);
    shifted_map.insert(999, 2.0);
    assert!(estimate_contrasts(&m, &shifted, "excavation").is_err());
}
#[test]
fn burrow_report_materials_retain_terminal_carried_and_loose_units() {
    let e = run_episode(
        LabConfig {
            transport: Transport::Relay,
            ..Default::default()
        },
        7,
        RunOptions {
            ticks: 64,
            sample_every: 16,
        },
    )
    .unwrap();
    let row = seed_row(
        RunKey {
            condition: "test".into(),
            seed: "7".into(),
        },
        &e,
    )
    .unwrap();
    assert_eq!(
        row.materials.len() as u64,
        row.delivered + row.censored_carried + row.censored_loose
    );
    assert_eq!(row.censored_carried, e.final_summary.inventory.carried);
    assert_eq!(row.censored_loose, e.final_summary.inventory.loose);
    assert!(row.censored_carried > 0 && row.censored_loose > 0);
    for r in row
        .materials
        .iter()
        .filter(|r| r.fate != MaterialFate::Delivered)
    {
        assert_eq!(r.tick_age, e.final_summary.tick - r.born);
        assert!(r.opportunity_age.is_some());
    }
}
#[test]
fn burrow_report_within_round_tick_latency_zero_preserves_opportunities() {
    let mut e = run_episode(
        LabConfig::default(),
        7,
        RunOptions {
            ticks: 0,
            sample_every: 1,
        },
    )
    .unwrap();
    // A saved-history reduction fixture, not a simulated scientific replicate.
    e.events = vec![
        sugarscape_core::burrow::ActionEvent {
            tick: 0,
            worker: 0,
            action: Action::Dig(sugarscape_core::burrow::Pos { x: 3, y: 10 }),
            outcome: Outcome::Success,
            material: Some(0),
            from: sugarscape_core::burrow::Pos { x: 0, y: 12 },
            to: sugarscape_core::burrow::Pos { x: 0, y: 12 },
        },
        sugarscape_core::burrow::ActionEvent {
            tick: 0,
            worker: 1,
            action: Action::Dispose,
            outcome: Outcome::Success,
            material: Some(0),
            from: sugarscape_core::burrow::Pos { x: 0, y: 12 },
            to: sugarscape_core::burrow::Pos { x: 0, y: 12 },
        },
    ];
    e.deliveries = vec![sugarscape_core::burrow::Delivery {
        material: 0,
        born: 0,
        disposed_at: Some(0),
        carriers: vec![0, 1],
        carried_moves: 0,
        waiting_ticks: 0,
    }];
    let rows = material_rows(&e).unwrap();
    assert_eq!((rows[0].tick_age, rows[0].opportunity_age), (0, Some(1)));
}
#[test]
fn burrow_report_choice_old_age_is_not_observed_waiting() {
    let e = run_episode(
        LabConfig {
            fixture: Fixture::Choice {
                side: Side::Left,
                pile: Pile::OldAccumulation,
            },
            cue: Cue::Responsive,
            ..Default::default()
        },
        7,
        RunOptions {
            ticks: 1,
            sample_every: 1,
        },
    )
    .unwrap();
    let row = seed_row(
        RunKey {
            condition: "choice".into(),
            seed: "7".into(),
        },
        &e,
    )
    .unwrap();
    assert_eq!(row.excavation_rate, None);
    assert!(row.materials.iter().all(|r| r.tick_age > 0
        && r.waiting_ticks == 0
        && r.fate == MaterialFate::CensoredLoose
        && r.opportunity_age.is_none()));
}
#[test]
fn burrow_report_saved_construction_is_complete_deterministic_and_exclusive() {
    // Generate only the canonical corridor panel, seeds 7 and 8.
    let m = manifest().unwrap();
    let records = m
        .construction_conditions
        .iter()
        .flat_map(|c| {
            m.construction_seeds.iter().map(move |s| {
                (
                    RunKey {
                        condition: c.id.clone(),
                        seed: s.clone(),
                    },
                    run_episode(c.config.clone(), s.parse().unwrap(), c.options.clone()).unwrap(),
                )
            })
        })
        .collect();
    let index = super::super::burrow_archive::Index {
        schema: "burrow-archive-v1".into(),
        code_revision: "a".repeat(40),
        protocol_revision: "b".repeat(40),
        manifest_sha256: super::super::burrow::manifest_sha256(),
        panel: Panel::Construction,
        approval_context: "test".into(),
        expected_keys: super::super::burrow::expected_keys(&m, Panel::Construction),
        completed: true,
        runs: vec![],
        manifest: m,
    };
    let mut archive = Archive { index, records };
    let root =
        std::env::temp_dir().join(format!("burrow-report-canonical36-{}", std::process::id()));
    fs::create_dir(&root).unwrap();
    fs::create_dir(root.join("raw")).unwrap();
    for (ordinal, key) in archive.index.expected_keys.iter().enumerate() {
        let c = archive
            .index
            .manifest
            .construction_conditions
            .iter()
            .find(|c| c.id == key.condition)
            .unwrap();
        let envelope = super::super::burrow_archive::Envelope {
            schema: archive.index.schema.clone(),
            key: key.clone(),
            code_revision: archive.index.code_revision.clone(),
            protocol_revision: archive.index.protocol_revision.clone(),
            manifest_sha256: archive.index.manifest_sha256.clone(),
            options: c.options.clone(),
            episode: archive.records[key].clone(),
        };
        let bytes = serde_json::to_vec(&envelope).unwrap();
        let path = format!("raw/{ordinal:03}.json");
        fs::write(root.join(&path), &bytes).unwrap();
        archive
            .index
            .runs
            .push(super::super::burrow_archive::RawRef {
                key: key.clone(),
                path,
                sha256: hash(&bytes),
            });
    }
    fs::write(
        root.join("index.json"),
        serde_json::to_vec(&archive.index).unwrap(),
    )
    .unwrap();
    let a = analyze(&archive).unwrap();
    let b = analyze(&archive).unwrap();
    assert_eq!(a.rows.len(), 36);
    assert!(a.means.is_empty() && a.contrasts.is_empty());
    assert_eq!(
        serde_json::to_vec_pretty(&a).unwrap(),
        serde_json::to_vec_pretty(&b).unwrap()
    );
    assert_eq!(render_results(&a), render_results(&b));
    assert!(!render_results(&a).contains("Holds"));
    let first = root.join("analysis-a");
    let second = root.join("analysis-b");
    analyze_saved(&root.join("index.json"), &first).unwrap();
    analyze_saved(&root.join("index.json"), &second).unwrap();
    for file in ["analysis.json", "results.md"] {
        assert_eq!(
            fs::read(first.join(file)).unwrap(),
            fs::read(second.join(file)).unwrap()
        );
    }
    assert!(analyze_saved(&root.join("index.json"), &first).is_err());
    let raw = root.join(&archive.index.runs[0].path);
    fs::write(raw, b"tampered").unwrap();
    let bad = root.join("bad-analysis");
    assert!(analyze_saved(&root.join("index.json"), &bad)
        .unwrap_err()
        .contains("SHA-256"));
    assert!(!bad.exists());
    fs::remove_dir_all(root).unwrap();
}
#[test]
fn burrow_report_bad_saved_input_creates_no_output() {
    let out = std::env::temp_dir().join(format!("burrow-report-bad-{}", std::process::id()));
    assert!(analyze_saved(Path::new("/nonexistent/burrow-index.json"), &out).is_err());
    assert!(!out.exists());
}

#[test]
fn burrow_report_choice_counts_keep_mirrored_sides_and_supplied_probabilities() {
    let m = manifest().unwrap();
    let seeds = vec!["7".into(), "8".into()];
    for c in m
        .conditions
        .iter()
        .filter(|c| matches!(c.config.fixture, Fixture::Choice { .. }))
    {
        let records = seeds
            .iter()
            .map(|s: &String| {
                (
                    RunKey {
                        condition: c.id.clone(),
                        seed: s.clone(),
                    },
                    run_episode(c.config.clone(), s.parse().unwrap(), c.options.clone()).unwrap(),
                )
            })
            .collect();
        let row = choice_row(c, &seeds, &records).unwrap();
        let Fixture::Choice { side, pile } = c.config.fixture else {
            unreachable!()
        };
        assert_eq!(row.left + row.right, 2);
        assert_eq!(
            row.pile_side,
            if side == Side::Left { "left" } else { "right" }
        );
        assert_eq!(
            row.expected_pile_probability,
            if c.config.cue == Cue::Responsive && pile == Pile::FreshAccumulation {
                0.75
            } else {
                0.5
            }
        );
    }
}
