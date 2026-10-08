use super::super::{
    archive::Index,
    manifest::{candidate, condition, expected_keys},
    report::{analyze, build_contrasts, markdown, ContrastRole},
    report_rows::{project_row, EpisodeRow},
    sha256, CollectionMode, Panel, Regime,
};
use super::support::{complete_construction_archive, decode_fixture, TestArchiveFixture};
use std::fs;

#[test]
fn construction_analysis_keeps_every_zero_and_emits_no_scientific_contrasts() {
    let archive = complete_construction_archive();
    let a = analyze(&archive.index, &archive.temp.path().join("analysis")).unwrap();
    assert_eq!(a.rows.len(), 30);
    assert!(a.contrasts.is_empty());
    assert_eq!(
        a.rows
            .iter()
            .filter(|r| r.key.condition == "access.straight.protected")
            .map(|r| r.summary.food.delivered)
            .collect::<Vec<_>>(),
        vec![0, 0]
    );
    assert_eq!(
        a.rows.iter().map(|r| r.key.clone()).collect::<Vec<_>>(),
        expected_keys(&candidate().unwrap(), CollectionMode::Construction)
    );
    let text = markdown(&a).unwrap();
    assert!(text.contains("Construction engineering checks"));
    assert!(!text.contains("Condition means"));
    assert!(text.contains("sampled physical projection"));
    for r in a.rows.iter().filter(|r| r.panel == Panel::Access) {
        assert_eq!((r.initial_distance, r.gain), (None, None));
        assert!(r.first_shortening.is_none());
    }
}
#[test]
fn saved_reanalysis_is_byte_identical_and_operational_metadata_is_excluded() {
    let archive = complete_construction_archive();
    let first = archive.temp.path().join("first");
    let second = archive.temp.path().join("second");
    analyze(&archive.index, &first).unwrap();
    let mut index: Index = serde_json::from_slice(&fs::read(&archive.index).unwrap()).unwrap();
    index.approval_context = "different operational approval text".into();
    fs::write(&archive.index, serde_json::to_vec(&index).unwrap()).unwrap();
    fs::write(
        archive.index.parent().unwrap().join("operational.json"),
        b"{\"elapsed_seconds\":123}",
    )
    .unwrap();
    analyze(&archive.index, &second).unwrap();
    for name in ["analysis.json", "results.md"] {
        assert_eq!(
            fs::read(first.join(name)).unwrap(),
            fs::read(second.join(name)).unwrap()
        );
    }
    assert!(analyze(&archive.index, &first).is_err());
}
#[test]
fn projection_preserves_delivered_resource_routes_and_full_summary() {
    let m = candidate().unwrap();
    let envelope = decode_fixture("route.straight.protected", 7).unwrap();
    let r = project_row(
        condition(&m, &envelope.key.condition).unwrap(),
        &envelope.key,
        &envelope.episode,
    )
    .unwrap();
    assert_eq!(r.summary, envelope.episode.summary);
    assert_eq!(
        (r.initial_distance, r.final_distance, r.gain),
        (Some(15), Some(15), Some(0))
    );
    assert!(r
        .route
        .iter()
        .all(|p| p.per_food.len() == 16 && p.per_food.iter().all(|(_, d)| d.is_some())));
    assert!(r.summary.food.delivered > 0);
    assert_eq!(r.first_shortening, None);
}
fn rewrite_index(a: &TestArchiveFixture, f: impl FnOnce(&mut Index)) {
    let mut index: Index = serde_json::from_slice(&fs::read(&a.index).unwrap()).unwrap();
    f(&mut index);
    fs::write(&a.index, serde_json::to_vec(&index).unwrap()).unwrap();
}
fn mutate_record(
    a: &TestArchiveFixture,
    ordinal: usize,
    resign: bool,
    f: impl FnOnce(&mut super::super::wire::WireEnvelope),
) {
    rewrite_index(a, |index| {
        let reference = &mut index.runs[ordinal];
        let path = a.index.parent().unwrap().join(&reference.path);
        let mut value: super::super::wire::WireEnvelope =
            serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        f(&mut value);
        let bytes = serde_json::to_vec(&value).unwrap();
        fs::write(path, &bytes).unwrap();
        if resign {
            index.raw_bytes = index.raw_bytes - reference.bytes + bytes.len() as u64;
            reference.bytes = bytes.len() as u64;
            reference.sha256 = sha256(&bytes);
        }
    });
}
fn mutate_last(
    a: &TestArchiveFixture,
    resign: bool,
    f: impl FnOnce(&mut super::super::wire::WireEnvelope),
) {
    mutate_record(a, 29, resign, f);
}
#[test]
fn analysis_rejects_bad_last_saved_state_before_creating_output() {
    let archive = complete_construction_archive();
    mutate_last(&archive, true, |v| v.episode.summary.food.delivered = 1);
    let out = archive.temp.path().join("analysis");
    assert!(analyze(&archive.index, &out).is_err());
    assert!(!out.exists());
}
#[test]
fn analysis_rejects_hash_mismatch_before_creating_output() {
    let archive = complete_construction_archive();
    mutate_last(&archive, false, |v| v.episode.summary.food.delivered = 1);
    let out = archive.temp.path().join("analysis");
    assert!(analyze(&archive.index, &out).is_err());
    assert!(!out.exists());
}
#[test]
fn analysis_rejects_missing_duplicate_extra_keys_and_terminal_byte_total() {
    for mutation in 0..4 {
        let archive = complete_construction_archive();
        rewrite_index(&archive, |i| match mutation {
            0 => {
                i.runs.pop();
            }
            1 => {
                i.runs[29] = i.runs[28].clone();
            }
            2 => {
                i.runs.push(i.runs[0].clone());
            }
            _ => i.raw_bytes += 1,
        });
        let out = archive.temp.path().join("analysis");
        assert!(analyze(&archive.index, &out).is_err());
        assert!(!out.exists());
    }
}
// Forty seed labels are synthetic statistics only; never run a scientific episode.
fn synthetic_rows() -> Vec<EpisodeRow> {
    let m = candidate().unwrap();
    let e = decode_fixture("route.straight.protected", 7).unwrap();
    let template =
        project_row(condition(&m, &e.key.condition).unwrap(), &e.key, &e.episode).unwrap();
    expected_keys(&m, CollectionMode::Scientific)
        .iter()
        .map(|key| {
            let c = condition(&m, &key.condition).unwrap();
            let mut row = template.clone();
            row.key = key.clone();
            row.panel = c.panel;
            row.geometry = c.geometry;
            row.regime = c.regime;
            row.summary.food.delivered = match c.regime {
                Regime::Paid => (key.seed % 3) as u32,
                Regime::Protected => 1,
                Regime::AlreadyOpen => 2,
            };
            row.summary.access.accessible = if c.regime == Regime::Paid { 16 } else { 0 };
            row
        })
        .collect()
}
#[test]
fn synthetic_contrasts_keep_roles_signed_pairs_and_stable_order() {
    let mut rows = synthetic_rows();
    let contrasts = build_contrasts(&rows).unwrap();
    assert_eq!(contrasts.len(), 15);
    assert_eq!(
        contrasts
            .iter()
            .filter(|c| c.role == ContrastRole::Primary)
            .count(),
        3
    );
    assert_eq!(
        contrasts.iter().map(|c| c.stats.n).collect::<Vec<_>>(),
        vec![40; 15]
    );
    assert!(contrasts
        .iter()
        .any(|c| c.stats.positive > 0 && c.stats.negative > 0 && c.stats.zero > 0));
    assert!(contrasts.iter().any(|c| c.stats.mean < 0.0));
    rows.reverse();
    assert_eq!(build_contrasts(&rows).unwrap(), contrasts);
}
#[test]
fn synthetic_contrasts_reject_missing_duplicate_and_ambiguous_roles() {
    let rows = synthetic_rows();
    let mut bad = rows.clone();
    bad.pop();
    assert!(build_contrasts(&bad).is_err());
    let mut bad = rows.clone();
    bad.push(rows[0].clone());
    assert!(build_contrasts(&bad).is_err());
    let mut bad = rows;
    bad[0].panel = Panel::Access;
    assert!(build_contrasts(&bad).is_err());
}
#[test]
fn synthetic_tied_contrasts_preserve_null_route_outcomes() {
    let mut rows = synthetic_rows();
    for row in &mut rows {
        row.summary.food.delivered = 0;
        row.summary.access.accessible = 0;
        row.initial_distance = None;
        row.final_distance = None;
        row.gain = None;
        row.first_shortening = None;
    }
    for c in build_contrasts(&rows).unwrap() {
        assert_eq!(
            (
                c.stats.mean,
                c.stats.ci95,
                c.stats.positive,
                c.stats.zero,
                c.stats.negative
            ),
            (0.0, Some((0.0, 0.0)), 0, 40, 0)
        );
    }
}

#[test]
fn core_straight_identity_and_observational_repeated_steps_match_full_episodes() {
    use super::support::cached_candidate_episode;
    use sugarscape_core::foraging::construction as core;
    let m = candidate().unwrap();
    for seed in [7, 8] {
        let protected = cached_candidate_episode("route.straight.protected", seed).unwrap();
        let already_open = cached_candidate_episode("route.straight.already_open", seed).unwrap();
        assert_eq!(
            serde_json::to_vec(&protected).unwrap(),
            serde_json::to_vec(&already_open).unwrap()
        );
        let c = condition(&m, "route.detour.paid").unwrap();
        let expected = cached_candidate_episode(&c.id, seed).unwrap();
        let mut world = core::World::new(c.setup.clone(), seed).unwrap();
        let mut snapshots = vec![world.snapshot().unwrap()];
        for completed in 1..=512 {
            // Read-only diagnostics must not consume RNG or alter controller state.
            let _ = world.summary().unwrap();
            let _ = world.snapshot().unwrap();
            world.step().unwrap();
            if completed % 128 == 0 {
                snapshots.push(world.snapshot().unwrap());
            }
        }
        let observed = core::Episode {
            setup: c.setup.clone(),
            seed,
            options: c.options.clone(),
            summary: world.summary().unwrap(),
            snapshot_bytes: snapshots
                .iter()
                .map(|f| serde_json::to_vec(f).unwrap().len() as u64)
                .sum(),
            snapshots,
        };
        assert_eq!(observed, expected);
    }
}
#[test]
fn synthetic_reports_are_descriptive_ordered_and_match_existing_student_t_stats() {
    use super::super::report::Analysis;
    let rows = synthetic_rows();
    let contrasts = build_contrasts(&rows).unwrap();
    let values = |id: &str| {
        rows.iter()
            .filter(|r| r.key.condition == id)
            .map(|r| (r.key.seed, f64::from(r.summary.food.delivered)))
            .collect()
    };
    let expected = crate::stats::paired_summary(
        &values("route.straight.paid"),
        &values("route.straight.protected"),
    )
    .unwrap();
    assert_eq!(
        (
            contrasts[0].stats.mean,
            contrasts[0].stats.ci95,
            contrasts[0].stats.positive,
            contrasts[0].stats.zero,
            contrasts[0].stats.negative
        ),
        (
            expected.mean,
            expected.ci95,
            expected.positive,
            expected.zero,
            expected.negative
        )
    );
    let mut a = Analysis {
        schema: "foraging-shortcut-analysis-v1".into(),
        mode: CollectionMode::Scientific,
        provenance: super::support::test_provenance(),
        rows,
        contrasts,
    };
    let text = markdown(&a).unwrap();
    assert!(text.contains("Condition means (descriptive)"));
    assert!(text.contains("Primary route delivery differences"));
    assert!(text.contains("Secondary route references and sealed access differences"));
    a.rows.reverse();
    assert_eq!(markdown(&a).unwrap(), text);
}
#[test]
fn saved_analysis_rejects_resigned_cache_clocks_tags_ownership_and_metadata_paths() {
    // Each entire complete archive is cloned; corrupt the final record to require
    // full streaming validation, not merely validation of an early prefix.
    for mutation in 0..8 {
        let archive = complete_construction_archive();
        if mutation < 6 {
            mutate_last(&archive, true, |v| {
                let e = &mut v.episode;
                match mutation {
                    0 => e.snapshots[4].summary.access.records[0].distance = Some(15),
                    1 => e.snapshots[4].summary.completed_ticks = 511,
                    2 => {
                        e.snapshots[4].agents[0].cargo =
                            Some(super::super::wire_state::WireCargo::Food(999))
                    }
                    3 => {
                        e.snapshots[4].food[0].state =
                            super::super::wire_state::WireFoodState::Carried { agent: 99 }
                    }
                    4 => e.setup.parameters.p_search = super::super::wire::WireFloat("0.5".into()),
                    _ => e.snapshot_bytes = 0,
                }
                if mutation < 5 {
                    e.summary = e.snapshots.last().unwrap().summary.clone();
                    e.snapshot_bytes = e
                        .snapshots
                        .iter()
                        .map(|f| serde_json::to_vec(f).unwrap().len() as u64)
                        .sum();
                }
            });
        } else {
            rewrite_index(&archive, |i| {
                if mutation == 6 {
                    i.runs[29].path = "../escape.json".into();
                } else {
                    i.runs[29].bytes = 4 * 1024 * 1024 + 1;
                }
            });
        }
        let out = archive.temp.path().join("analysis");
        assert!(
            analyze(&archive.index, &out).is_err(),
            "mutation {mutation}"
        );
        assert!(!out.exists());
    }
}
#[test]
fn saved_analysis_rejects_draft_scientific_archives_and_metadata_byte_overflow() {
    let archive = complete_construction_archive();
    rewrite_index(&archive, |i| i.mode = CollectionMode::Scientific);
    let out = archive.temp.path().join("analysis");
    assert!(analyze(&archive.index, &out).unwrap_err().contains("draft"));
    assert!(!out.exists());
    fs::write(&archive.index, vec![b' '; 4 * 1024 * 1024 + 1]).unwrap();
    assert!(analyze(&archive.index, &out)
        .unwrap_err()
        .contains("byte limit"));
    assert!(!out.exists());
}

#[test]
fn saved_analysis_rejects_resigned_event_distances_and_raw_parser_overflow() {
    let archive = complete_construction_archive();
    mutate_record(&archive, 0, true, |v| {
        for frame in &mut v.episode.snapshots {
            if let Some(event) = &mut frame.summary.milestones.first_excavation {
                event.nest_distance += 1;
            }
        }
        assert!(v
            .episode
            .snapshots
            .last()
            .unwrap()
            .summary
            .milestones
            .first_excavation
            .is_some());
        v.episode.summary = v.episode.snapshots.last().unwrap().summary.clone();
        v.episode.snapshot_bytes = v
            .episode
            .snapshots
            .iter()
            .map(|f| serde_json::to_vec(f).unwrap().len() as u64)
            .sum();
    });
    let out = archive.temp.path().join("bad-event");
    assert!(analyze(&archive.index, &out)
        .unwrap_err()
        .contains("distance"));
    assert!(!out.exists());
    let archive = complete_construction_archive();
    let index: Index = serde_json::from_slice(&fs::read(&archive.index).unwrap()).unwrap();
    let raw = archive.index.parent().unwrap().join(&index.runs[29].path);
    fs::write(raw, vec![b' '; 4 * 1024 * 1024 + 1]).unwrap();
    let out = archive.temp.path().join("bad-size");
    assert!(analyze(&archive.index, &out)
        .unwrap_err()
        .contains("byte limit"));
    assert!(!out.exists());
}
