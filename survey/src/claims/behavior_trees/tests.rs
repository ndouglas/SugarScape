use super::*;
use report_types::*;
#[test]
fn behavior_tree_manifest_has_exact_registered_budget() {
    let m = manifest::manifest();
    assert_eq!(
        (m.conditions.len(), m.seeds),
        (96, (30001..=30040).collect::<Vec<_>>())
    );
}
#[test]
fn behavior_tree_all_four_families_keep_all_pairs() {
    let m = manifest::manifest();
    let rows:Vec<_>=m.conditions.iter().flat_map(|c| m.seeds.iter().map(move |seed| Endpoint{condition:c.id.clone(),seed:*seed,quota_attained:false,first_completion:None,restricted_completion_ticks:64,right_censored:true,unattained_reason:Some(sugarscape_core::minds::behavior_tree::records::UnattainedReason::DiedBeforeQuota),gross_gathered:0.0,living_ticks:16,alive_at_horizon:false})).collect();
    let estimates = report::summarize(&m, &rows).unwrap();
    assert_eq!(estimates.len(), 256);
    assert!(estimates.iter().all(|e| e.denominator == 40
        && e.summary.n == 40
        && e.summary.ci95 == Some((0.0, 0.0))
        && e.summary.zero == 40));
    let mut duplicated = rows.clone();
    duplicated.push(rows[0].clone());
    assert!(report::summarize(&m, &duplicated).is_err());
    assert!(report::summarize(&m, &rows[1..]).is_err());
}
#[test]
fn behavior_tree_actual_durable_create_never_overwrites() {
    let dir = std::env::temp_dir()
        .canonicalize()
        .unwrap()
        .join(format!("bt-io-{}", std::process::id()));
    std::fs::create_dir(&dir).unwrap();
    let file = dir.join("start.json");
    io::create(&file, b"first").unwrap();
    assert!(io::create(&file, b"second").is_err());
    assert_eq!(std::fs::read(&file).unwrap(), b"first");
    std::fs::remove_dir_all(dir).unwrap();
}
fn construction() -> sugarscape_core::minds::behavior_tree::EpisodeRecord {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src/claims/behavior_trees/fixtures/guarded_tree-stable-q20-original-seed7.json");
    wire::decode(&std::fs::read(path).unwrap()).unwrap()
}
#[test]
fn behavior_tree_saved_mutated_initial_memory_rejected() {
    let mut r = construction();
    r.frames[0]
        .actor
        .as_mut()
        .unwrap()
        .memory
        .iter_mut()
        .find(|m| m.site == 103)
        .unwrap()
        .most[0] = 36.0;
    assert!(validate::validate_episode(&r, &r.lab, 7).is_err())
}
#[test]
fn behavior_tree_saved_impossible_action_rejected() {
    let mut r = construction();
    r.frames[1].receipt.as_mut().unwrap().destination.x = 9;
    assert!(validate::validate_episode(&r, &r.lab, 7).is_err())
}
#[test]
fn behavior_tree_strict_coordinates_reject_unknown_fields() {
    let r = construction();
    let raw = serde_json::to_string(&r).unwrap();
    let changed = raw.replacen("\"x\":2,\"y\":5", "\"x\":2,\"y\":5,\"hidden\":1", 1);
    assert_ne!(raw, changed);
    assert!(
        wire::decode::<sugarscape_core::minds::behavior_tree::EpisodeRecord>(changed.as_bytes())
            .is_err()
    )
}
#[test]
fn behavior_tree_valid_saved_record_and_all_prefixes() {
    let r = construction();
    validate::validate_episode(&r, &r.lab, 7).unwrap();
    for n in [1, 2, 4, 8, 33, 65] {
        validate::validate_frames(&r.frames[..n], &r.lab, 7).unwrap();
    }
}
#[test]
fn behavior_tree_saved_semantic_mutations_rejected() {
    let r = construction();
    let mut cases = vec![];
    let mut b = r.clone();
    b.frames[1].observation.as_mut().unwrap().candidates[0].value = 999.0;
    cases.push(b);
    let mut b = r.clone();
    b.frames[1].task.fsm.phase = sugarscape_core::minds::behavior_tree::state::FsmPhase::Moving;
    cases.push(b);
    let mut b = r.clone();
    b.frames[6].receipt.as_mut().unwrap().gathered = 1.0;
    cases.push(b);
    let mut b = r.clone();
    b.frames[1].task.failed_until.insert(62, 99);
    cases.push(b);
    let mut b = r.clone();
    b.frames[1].actor.as_mut().unwrap().memory[0].tick = 1;
    cases.push(b);
    let mut b = r.clone();
    b.frames[1].rng_state_json = b.frames[0].rng_state_json.clone();
    cases.push(b);
    for (i, b) in cases.into_iter().enumerate() {
        assert!(
            validate::validate_episode(&b, &b.lab, 7).is_err(),
            "accepted mutation {i}"
        )
    }
}
#[test]
fn behavior_tree_verification_domain_rejects_unsafe_bounds() {
    use sugarscape_core::{geometry::Pos, minds::behavior_tree::verification, rng};
    assert!(verification::plan(&[], 1.0).is_err());
    assert!(verification::plan(&[(Pos::new(1, 1), f64::NAN)], 1.0).is_err());
    assert!(verification::choose(&[], &mut rng::seeded(7)).is_err());
}
fn directory() -> std::path::PathBuf {
    static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let p = std::env::temp_dir().canonicalize().unwrap().join(format!(
        "bt-tests-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    ));
    std::fs::create_dir(&p).unwrap();
    p
}
fn provenance(m: &manifest::Manifest) -> archive::Provenance {
    let file = archive::FileIdentity {
        path: "engineering-only".into(),
        sha256: "a".repeat(64),
        bytes: 1,
        mode: "100644".into(),
    };
    archive::Provenance {
        source_root: "engineering-only".into(),
        source_revision: "a".repeat(40),
        protocol: file.clone(),
        binary: file,
        source_inventory: vec![],
        source_sha256: "b".repeat(64),
        compiled_inputs_sha256: "c".repeat(64),
        manifest_sha256: archive::hash(&serde_json::to_vec(m).unwrap()),
        toolchain: "failure-only engineering injection; no measurement".into(),
        build_flags: vec![],
        compiled_input_paths: vec![],
        selector_scope: "failure-only test".into(),
    }
}
#[test]
fn behavior_tree_failure_collector_has_durable_start_before_callback() {
    let root = directory();
    let out = root.join("archive");
    let mut m = manifest::manifest();
    m.conditions.truncate(1);
    m.seeds.truncate(1);
    let condition = m.conditions[0].id.clone();
    let seed = m.seeds[0];
    let p = provenance(&m);
    let mut calls = 0;
    archive::collect(&out, m.clone(), p, |_, _, _| {
        calls += 1;
        let start = out.join(format!("{condition}-{seed}.start.json"));
        let frames = out.join(format!("{condition}-{seed}.frames.jsonl"));
        assert!(start.exists());
        assert_eq!(std::fs::metadata(frames).unwrap().len(), 0);
        Err(sugarscape_core::minds::behavior_tree::EpisodeFailure {
            message: "injected failure only; no World constructed".into(),
            partial: None,
        })
    })
    .unwrap();
    assert_eq!(calls, 1);
    let a = archive::load_declared(&out.join("final-index.json"), &m).unwrap();
    assert_eq!(a.index.census.unwrap().failed, 1);
    assert!(archive::load(&out.join("final-index.json")).is_err());
    std::fs::remove_dir_all(root).unwrap();
}
#[test]
fn behavior_tree_outcome_io_failure_retains_start_and_stops() {
    let root = directory();
    let out = root.join("archive");
    let mut m = manifest::manifest();
    m.conditions.truncate(1);
    m.seeds.truncate(2);
    let p = provenance(&m);
    let c = m.conditions[0].id.clone();
    let mut calls = 0;
    let result = archive::collect(&out, m.clone(), p, |_, seed, _| {
        calls += 1;
        std::fs::create_dir(out.join(format!("{c}-{seed}.outcome.json"))).unwrap();
        Err(sugarscape_core::minds::behavior_tree::EpisodeFailure {
            message: "test only".into(),
            partial: None,
        })
    });
    assert!(result.unwrap_err().contains("prefix census"));
    assert_eq!(calls, 1);
    assert!(out.join("index.json").exists());
    assert!(!out.join("final-index.json").exists());
    std::fs::remove_dir_all(root).unwrap();
}
#[test]
fn behavior_tree_real_frame_and_parent_sync_errors() {
    let root = directory();
    let path = root.join("read-only");
    io::create(&path, b"").unwrap();
    let mut file = std::fs::File::open(&path).unwrap();
    assert!(io::append(&mut file, &construction().frames[0]).is_err());
    assert!(io::sync_parent(&path.join("child")).is_err());
    assert!(io::relative(&root, "../escape").is_err());
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(&path, root.join("link")).unwrap();
        assert!(io::read(&root.join("link")).is_err());
    }
    std::fs::remove_dir_all(root).unwrap();
}
#[test]
fn behavior_tree_build_stamp_order_duplicate_path_and_presence() {
    use archive::inputs::*;
    let a = ("survey/src/a.rs".into(), b"one".to_vec());
    let b = ("survey/src/b.rs".into(), b"two".to_vec());
    let hash = |v: Vec<(String, Vec<u8>)>| fingerprint(v.into_iter().map(Ok));
    assert_eq!(hash(vec![a.clone(), b.clone()]), hash(vec![b, a.clone()]));
    assert!(hash(vec![a.clone(), a.clone()]).is_err());
    assert!(hash(vec![("survey/src/../bad".into(), vec![])]).is_err());
    assert_ne!(
        hash(vec![a.clone()]),
        hash(vec![(a.0.clone(), b"changed".to_vec())])
    );
    assert_ne!(
        hash(vec![a.clone()]),
        hash(vec![a, (PROTOCOL.into(), vec![])])
    );
    assert!(!selected("survey/src/target/file"));
}
#[test]
fn behavior_tree_completion_never_drops_unattained_pairs() {
    let m = manifest::manifest();
    let rows:Vec<_>=m.conditions.iter().flat_map(|c|m.seeds.iter().map(move|s|Endpoint{condition:c.id.clone(),seed:*s,quota_attained:false,first_completion:None,restricted_completion_ticks:64,right_censored:true,unattained_reason:Some(sugarscape_core::minds::behavior_tree::records::UnattainedReason::DiedBeforeQuota),gross_gathered:0.0,living_ticks:16,alive_at_horizon:false})).collect();
    let c = report::completion(&m, &rows).unwrap();
    assert_eq!(c.len(), 64);
    assert!(c
        .iter()
        .all(|c| c.denominator == 40 && c.summary.is_none() && c.unavailable.len() == 40));
}
#[test]
fn behavior_tree_six_saved_controller_fixtures_validate() {
    let root =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/claims/behavior_trees/fixtures");
    let mut count = 0;
    for path in std::fs::read_dir(root).unwrap() {
        let path = path.unwrap().path();
        if path.extension().is_some_and(|s| s == "json") {
            let r: sugarscape_core::minds::behavior_tree::EpisodeRecord =
                wire::decode(&std::fs::read(path).unwrap()).unwrap();
            validate::validate_episode(&r, &r.lab, r.seed).unwrap();
            count += 1;
        }
    }
    assert_eq!(count, 6);
}
#[test]
fn behavior_tree_actual_frame_write_failure_stops_constructor_prefix() {
    let root = directory();
    let out = root.join("archive");
    let mut m = manifest::manifest();
    m.conditions.truncate(1);
    m.seeds.truncate(2);
    let p = provenance(&m);
    let f = construction().frames[0].clone();
    let mut calls = 0;
    let e = archive::collect_with_frames(
        &out,
        m.clone(),
        p,
        |_, _, sink| {
            calls += 1;
            let error = sink(&f).unwrap_err();
            Err(sugarscape_core::minds::behavior_tree::EpisodeFailure {
                message: error,
                partial: None,
            })
        },
        |path| std::fs::File::open(path).map_err(|e| e.to_string()),
    )
    .unwrap_err();
    assert!(e.contains("frame write/sync"));
    assert!(e.contains("pending: 1"));
    assert!(e.contains("unstarted: 1"));
    assert_eq!(calls, 1);
    let a = archive::load_declared(&out.join("index.json"), &m).unwrap();
    assert_eq!(a.index.census.unwrap().pending, 1);
    std::fs::remove_dir_all(root).unwrap();
}
#[test]
fn behavior_tree_start_only_and_torn_stream_are_pending() {
    let root = directory();
    let out = root.join("archive");
    let mut m = manifest::manifest();
    m.conditions.truncate(1);
    m.seeds.truncate(2);
    let p = provenance(&m);
    let _ = archive::collect_with_frames(
        &out,
        m.clone(),
        p,
        |_, _, _| panic!("must not construct after open error"),
        |_| Err("injected open error".into()),
    );
    let a = archive::load_declared(&out.join("index.json"), &m).unwrap();
    assert_eq!(a.index.census.as_ref().unwrap().pending, 1);
    let frames = out.join(&a.index.attempts[0].frames);
    use std::io::Write;
    std::fs::OpenOptions::new()
        .append(true)
        .open(&frames)
        .unwrap()
        .write_all(b"{\"tick\":")
        .unwrap();
    let a = archive::load_declared(&out.join("index.json"), &m).unwrap();
    assert!(a.attempts[0].torn_tail.is_some());
    assert_eq!(a.index.census.unwrap().pending, 1);
    std::fs::remove_dir_all(root).unwrap();
}
#[test]
fn behavior_tree_all_coordinate_owners_deny_unknown_keys() {
    fn rejects<T: serde::Serialize + serde::de::DeserializeOwned>(value: &T) {
        let raw = serde_json::to_string(value).unwrap();
        let at = raw.find("\"x\":").unwrap();
        let end = at + raw[at..].find('}').unwrap();
        let bad = format!("{},\"unexpected\":1{}", &raw[..end], &raw[end..]);
        assert!(wire::decode::<T>(bad.as_bytes()).is_err());
    }
    let r = construction();
    let f = &r.frames[1];
    rejects(f.actor.as_ref().unwrap());
    rejects(&f.actor.as_ref().unwrap().motion_plan);
    rejects(f.receipt.as_ref().unwrap());
    let o = f.observation.as_ref().unwrap();
    rejects(o);
    rejects(&o.candidates[0]);
    let p = sugarscape_core::geometry::Pos::new(2, 5);
    rejects(&sugarscape_core::minds::behavior_tree::state::TaskPlan {
        goal: 20.0,
        steps: vec![(p, 4.0)],
    });
    rejects(
        &sugarscape_core::minds::behavior_tree::records::LegacyPlan {
            goal: 20.0,
            gathers: 4.0,
            steps: vec![(p, 4.0)],
        },
    );
    for raw in [
        r#"{"x":1.5,"y":5}"#,
        r#"{"x":11,"y":5}"#,
        r#"{"x":-1,"y":5}"#,
    ] {
        assert!(wire::decode::<sugarscape_core::geometry::Pos>(raw.as_bytes()).is_err())
    }
}
#[test]
fn behavior_tree_stale_actual_source_rejected_before_output() {
    let root = directory();
    let repo = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap();
    let entries = archive::inputs::read(repo).unwrap();
    for (name, bytes) in &entries {
        let p = root.join(name);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, bytes).unwrap();
    }
    let path = root.join("survey/src/claims/behavior_trees/manifest.rs");
    use std::io::Write;
    std::fs::OpenOptions::new()
        .append(true)
        .open(path)
        .unwrap()
        .write_all(b"\n// stale input probe\n")
        .unwrap();
    let result = archive::preflight(
        &root,
        &std::env::current_exe().unwrap(),
        &"a".repeat(40),
        &manifest::manifest(),
    );
    assert!(result.unwrap_err().contains("stale binary"));
    std::fs::remove_dir_all(root).unwrap();
}
#[test]
fn behavior_tree_source_archive_without_git_and_missing_protocol() {
    let root = directory();
    for name in archive::inputs::DIRECTORIES {
        std::fs::create_dir_all(root.join(name)).unwrap();
    }
    for name in archive::inputs::FILES {
        if *name != archive::inputs::PROTOCOL {
            let p = root.join(name);
            std::fs::create_dir_all(p.parent().unwrap()).unwrap();
            std::fs::write(p, b"fixture").unwrap();
        }
    }
    let a = archive::inputs::read(&root).unwrap();
    assert!(!a.iter().any(|e| e.0 == archive::inputs::PROTOCOL));
    let p = root.join(archive::inputs::PROTOCOL);
    std::fs::write(&p, b"").unwrap();
    let b = archive::inputs::read(&root).unwrap();
    assert_ne!(
        archive::inputs::fingerprint(a.into_iter().map(Ok)).unwrap(),
        archive::inputs::fingerprint(b.into_iter().map(Ok)).unwrap()
    );
    std::fs::remove_file(root.join("Cargo.lock")).unwrap();
    assert!(archive::inputs::read(&root).is_err());
    std::fs::remove_dir_all(root).unwrap();
}
#[test]
fn behavior_tree_all_family_metric_ids_and_signed_pairs() {
    let m = manifest::manifest();
    let rows: Vec<_> = m
        .conditions
        .iter()
        .flat_map(|c| {
            m.seeds.iter().map(move |s| {
                let guard = c.lab.controller
                    == sugarscape_core::minds::behavior_tree::state::Controller::GuardedTree;
                Endpoint {
                    condition: c.id.clone(),
                    seed: *s,
                    quota_attained: true,
                    first_completion: Some(if guard { 10 } else { 20 }),
                    restricted_completion_ticks: if guard { 10 } else { 20 },
                    right_censored: false,
                    unattained_reason: None,
                    gross_gathered: if guard { 40.0 } else { 30.0 },
                    living_ticks: if guard { 60 } else { 50 },
                    alive_at_horizon: false,
                }
            })
        })
        .collect();
    let estimates = report::summarize(&m, &rows).unwrap();
    let mut ids = std::collections::BTreeSet::new();
    for e in &estimates {
        assert!(ids.insert(&e.id));
        assert_eq!(e.denominator, 40);
        match e.metric {
            Metric::QuotaAttained => {
                assert_eq!((e.summary.zero, e.summary.ci95), (40, Some((0.0, 0.0))))
            }
            Metric::RestrictedCompletionTicks => {
                assert_eq!((e.summary.negative, e.summary.mean), (40, -10.0))
            }
            _ => assert_eq!((e.summary.positive, e.summary.mean), (40, 10.0)),
        }
    }
    for family in [
        Family::Primary,
        Family::Secondary,
        Family::Ablation,
        Family::LegacyReference,
    ] {
        assert_eq!(estimates.iter().filter(|e| e.family == family).count(), 64);
    }
    assert!(report::completion(&m, &rows)
        .unwrap()
        .iter()
        .all(|c| c.summary.as_ref().unwrap().n == 40));
}
#[test]
fn behavior_tree_well_formed_conflict_is_invalid_and_raw_preserved() {
    let root = directory();
    let out = root.join("archive");
    let mut m = manifest::manifest();
    m.conditions.truncate(1);
    m.seeds.truncate(1);
    archive::collect(&out, m.clone(), provenance(&m), |_, _, _| {
        Err(sugarscape_core::minds::behavior_tree::EpisodeFailure {
            message: "failure-only fixture".into(),
            partial: None,
        })
    })
    .unwrap();
    let a = archive::load_declared(&out.join("final-index.json"), &m).unwrap();
    let path = out.join(&a.index.attempts[0].outcome);
    let bytes = std::fs::read(&path).unwrap();
    let mut receipt: archive::OutcomeReceipt = wire::decode(&bytes).unwrap();
    receipt.seed += 1;
    let changed = serde_json::to_vec(&receipt).unwrap();
    std::fs::write(&path, &changed).unwrap();
    assert!(archive::load_declared(&out.join("final-index.json"), &m)
        .unwrap_err()
        .contains("conflicting outcome"));
    assert_eq!(std::fs::read(path).unwrap(), changed);
    std::fs::remove_dir_all(root).unwrap();
}
#[test]
fn behavior_tree_final_index_must_match_immutable_start_index() {
    let root = directory();
    let out = root.join("archive");
    let mut m = manifest::manifest();
    m.conditions.truncate(1);
    m.seeds.truncate(1);
    archive::collect(&out, m.clone(), provenance(&m), |_, _, _| {
        Err(sugarscape_core::minds::behavior_tree::EpisodeFailure {
            message: "failure-only fixture".into(),
            partial: None,
        })
    })
    .unwrap();
    let path = out.join("index.json");
    let mut initial: archive::Index = wire::decode(&std::fs::read(&path).unwrap()).unwrap();
    initial.provenance.toolchain = "conflicting identity".into();
    std::fs::write(path, serde_json::to_vec(&initial).unwrap()).unwrap();
    assert!(archive::load_declared(&out.join("final-index.json"), &m).is_err());
    std::fs::remove_dir_all(root).unwrap();
}
#[test]
fn behavior_tree_bound_binary_protocol_and_source_mutation_precede_constructor() {
    for changed in ["binary", "protocol", "source"] {
        let root = directory();
        let mut m = manifest::manifest();
        m.conditions.truncate(1);
        m.seeds.truncate(1);
        let mut p = provenance(&m);
        p.source_root = root.to_string_lossy().into();
        for name in ["binary", "protocol", "source"] {
            std::fs::write(root.join(name), b"original").unwrap();
        }
        p.binary = archive::identity(
            &root.join("binary"),
            root.join("binary").to_string_lossy().into(),
        )
        .unwrap();
        p.protocol = archive::identity(&root.join("protocol"), "protocol".into()).unwrap();
        p.source_inventory =
            vec![archive::identity(&root.join("source"), "source".into()).unwrap()];
        std::fs::write(root.join(changed), b"mutated").unwrap();
        let bound = p.clone();
        let out = root.join("archive");
        let mut calls = 0;
        let result = archive::collect_with_frames(
            &out,
            m,
            p,
            |_, _, _| {
                calls += 1;
                panic!("identity failure must precede constructor")
            },
            |path| {
                archive::verify_bound_files(&bound)?;
                std::fs::OpenOptions::new()
                    .append(true)
                    .open(path)
                    .map_err(|e| e.to_string())
            },
        );
        assert!(result.unwrap_err().contains("before constructor"));
        assert_eq!(calls, 0);
        assert!(out.join("index.json").exists());
        std::fs::remove_dir_all(root).unwrap();
    }
}
#[test]
fn behavior_tree_untrusted_maximum_work_counters_are_errors_not_panics() {
    let original = construction();
    for field in ["candidate", "fallback", "expansions"] {
        let mut r = original.clone();
        let work = r.frames[1].work.as_mut().unwrap();
        match field {
            "candidate" => work.candidate_evaluations = u64::MAX,
            "fallback" => {
                work.fallback_short = u64::MAX;
                work.fallback_limit = u64::MAX;
            }
            _ => work.search_expansions = Some(u64::MAX),
        }
        let result = std::panic::catch_unwind(|| validate::validate_episode(&r, &r.lab, r.seed));
        assert!(
            matches!(result, Ok(Err(_))),
            "invalid {field} work must return an error without panic"
        );
    }
}

#[test]
fn behavior_tree_impossible_native_duration_is_rejected() {
    let mut r = construction();
    r.frames[1].controller_seconds = Some(f64::MAX);
    assert!(validate::validate_episode(&r, &r.lab, r.seed).is_err());
}

#[test]
fn behavior_tree_complete_invalid_tail_without_newline_is_rejected() {
    let record = construction();
    let mut order = record.frames[0].clone();
    order.tick = 99;
    let mut semantic = record.frames[0].clone();
    semantic.actor.as_mut().unwrap().holdings += 1.0;
    for bytes in [
        serde_json::to_vec(&order).unwrap(),
        b"{}".to_vec(),
        serde_json::to_vec(&semantic).unwrap(),
    ] {
        let root = directory();
        let out = root.join("archive");
        let m = manifest::Manifest {
            schema: manifest::manifest().schema,
            conditions: vec![manifest::Condition {
                id: manifest::id(&record.lab),
                lab: record.lab.clone(),
            }],
            seeds: vec![record.seed],
        };
        let error = archive::collect_with_frames(
            &out,
            m.clone(),
            provenance(&m),
            |_, _, _| panic!("failure-only fixture must not construct"),
            |_| Err("injected open failure".into()),
        )
        .unwrap_err();
        assert!(error.contains("pending: 1"));
        let index: archive::Index =
            wire::decode(&std::fs::read(out.join("index.json")).unwrap()).unwrap();
        let frames = out.join(&index.attempts[0].frames);
        std::fs::write(&frames, &bytes).unwrap();
        let result = archive::load_declared(&out.join("index.json"), &m);
        assert_eq!(std::fs::read(&frames).unwrap(), bytes);
        assert!(
            result.is_err(),
            "complete invalid JSON cannot be a torn tail"
        );
        std::fs::remove_dir_all(root).unwrap();
    }
}

#[test]
fn behavior_tree_per_attempt_git_inventory_drift_precedes_constructor() {
    git_drift_precedes_constructor("tracked_addition");
}
#[test]
fn behavior_tree_per_attempt_git_index_only_drift_precedes_constructor() {
    git_drift_precedes_constructor("index_only");
}
fn git_drift_precedes_constructor(drift: &str) {
    let temp = directory();
    let root = temp.join("repo");
    std::fs::create_dir(&root).unwrap();
    let git = |args: &[&str]| {
        let output = std::process::Command::new("git")
            .arg("-C")
            .arg(&root)
            .args(args)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "git {args:?}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        output.stdout
    };
    for name in archive::inputs::DIRECTORIES {
        std::fs::create_dir_all(root.join(name)).unwrap();
    }
    for name in archive::inputs::FILES
        .iter()
        .copied()
        .chain(["binary", "outside.txt"])
    {
        let path = root.join(name);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, b"original").unwrap();
    }
    git(&["init", "-q"]);
    git(&["config", "user.name", "BT engineering test"]);
    git(&["config", "user.email", "bt-test@example.invalid"]);
    git(&["add", "."]);
    git(&["commit", "-qm", "Bind engineering fixture"]);
    let mut m = manifest::manifest();
    m.conditions.truncate(1);
    m.seeds.truncate(2);
    let mut p = provenance(&m);
    p.source_root = root.to_string_lossy().into();
    p.source_revision = String::from_utf8(git(&["rev-parse", "HEAD"]))
        .unwrap()
        .trim()
        .into();
    p.binary = archive::identity(
        &root.join("binary"),
        root.join("binary").to_string_lossy().into(),
    )
    .unwrap();
    p.protocol = archive::identity(
        &root.join(archive::inputs::PROTOCOL),
        archive::inputs::PROTOCOL.into(),
    )
    .unwrap();
    p.source_inventory = git(&["ls-files", "-z"])
        .split(|b| *b == 0)
        .filter(|b| !b.is_empty())
        .map(|b| {
            let name = std::str::from_utf8(b).unwrap();
            archive::identity(&root.join(name), name.into()).unwrap()
        })
        .collect();
    p.source_sha256 = archive::hash(&serde_json::to_vec(&p.source_inventory).unwrap());
    let entries = archive::inputs::read(&root).unwrap();
    p.compiled_inputs_sha256 =
        archive::inputs::fingerprint(entries.clone().into_iter().map(Ok)).unwrap();
    p.compiled_input_paths = entries.into_iter().map(|e| e.0).collect();
    archive::verify_bound_files(&p).unwrap();
    let bound = p.clone();
    let out = temp.join("archive");
    let mut calls = 0;
    let result = archive::collect_with_frames(
        &out,
        m.clone(),
        p,
        |_, _, _| {
            calls += 1;
            Err(sugarscape_core::minds::behavior_tree::EpisodeFailure {
                message: "failure-only callback, no World".into(),
                partial: None,
            })
        },
        |path| {
            assert!(path.exists());
            match drift {
                "tracked_addition" => {
                    std::fs::write(root.join("new-outside.txt"), b"new tracked bytes").unwrap();
                    git(&["add", "new-outside.txt"]);
                }
                _ => {
                    std::fs::write(root.join("outside.txt"), b"index-only bytes").unwrap();
                    git(&["add", "outside.txt"]);
                    std::fs::write(root.join("outside.txt"), b"original").unwrap();
                }
            }
            archive::verify_bound_files(&bound)?;
            std::fs::OpenOptions::new()
                .append(true)
                .open(path)
                .map_err(|e| e.to_string())
        },
    );
    assert_eq!(calls, 0, "{drift} must stop before constructor");
    assert!(result.unwrap_err().contains("pending: 1"));
    let a = archive::load_declared(&out.join("index.json"), &m).unwrap();
    assert_eq!(a.index.census.as_ref().unwrap().pending, 1);
    assert_eq!(a.index.census.as_ref().unwrap().unstarted, 1);
    let raw = std::fs::read(out.join(&a.index.attempts[0].start)).unwrap();
    assert_eq!(
        wire::decode::<archive::AttemptStart>(&raw).unwrap(),
        a.attempts[0].start
    );
    assert!(std::fs::read(out.join(&a.index.attempts[0].frames))
        .unwrap()
        .is_empty());
    assert!(!out.join(&a.index.attempts[0].outcome).exists());
    std::fs::remove_dir_all(temp).unwrap();
}
