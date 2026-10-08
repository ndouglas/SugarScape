use super::super::{
    archive::ArchiveReader,
    cli::{parse, Command},
    manifest::{candidate, condition, expected_keys, manifest_bytes},
    run::{collect, preflight, ExecutionContext, RunRequest, MANIFEST_PATH},
    sha256, CollectionMode, Panel, Regime,
};
use super::support::{fixture_commit, fixture_git, owned_git_context, owned_tempdir};
use std::fs;

fn args(values: &[&str]) -> Vec<String> {
    values.iter().map(|s| (*s).into()).collect()
}
#[test]
fn parser_accepts_only_explicit_declared_forms() {
    assert!(matches!(parse(&[]).unwrap(), Command::Manifest));
    assert!(matches!(parse(&args(&["--help"])).unwrap(), Command::Help));
    let command = parse(&args(&[
        "--out",
        "new",
        "--approval-context",
        "approval",
        "--run",
        "--construction",
        "--protocol-revision",
        &"A".repeat(40),
    ]))
    .unwrap();
    assert!(matches!(command, Command::Run(r) if r.mode == CollectionMode::Construction));
    assert!(matches!(
        parse(&args(&["--analyze", "index.json", "--out", "new"])).unwrap(),
        Command::Analyze { .. }
    ));
}
#[test]
fn parser_rejects_overrides_repeats_missing_empty_and_incompatible_flags() {
    for values in [
        vec!["--analyze", "index.json", "--out", "new", "--seeds", "1"],
        vec!["--analyze", "index.json", "--out", "new", "--run"],
        vec!["--run", "--construction"],
        vec!["--help", "--run"],
        vec!["--help", "--help"],
        vec!["--out", ""],
        vec!["--analyze", "--out"],
        vec!["--construction"],
        vec!["--manifest"],
        vec!["--repo", "other"],
        vec![
            "--run",
            "--protocol-revision",
            "abc",
            "--approval-context",
            "approval",
            "--out",
            "new",
        ],
        vec!["--analyze", "index", "--out", "new", "--out", "other"],
    ] {
        assert!(parse(&args(&values)).is_err(), "accepted {values:?}");
    }
}
#[test]
fn collector_rejects_draft_scientific_request_before_output() {
    let tmp = owned_tempdir();
    let r = RunRequest {
        mode: CollectionMode::Scientific,
        protocol_revision: "a".repeat(40),
        approval_context: "test".into(),
        out: tmp.path().join("new"),
    };
    let ctx = ExecutionContext {
        repo: tmp.path().to_owned(),
        executable: tmp.path().join("collector"),
    };
    assert!(collect(&ctx, &r).unwrap_err().contains("draft"));
    assert!(!r.out.exists());
}
#[test]
fn preflight_binds_canonical_commit_manifest_and_executable_bytes() {
    let (_tmp, ctx, mut r) = owned_git_context();
    r.protocol_revision = r.protocol_revision.to_uppercase();
    let p = preflight(&ctx, &candidate().unwrap(), &r).unwrap();
    assert_eq!(
        p.code_revision,
        fixture_git(&ctx.repo, &["rev-parse", "HEAD"])
    );
    assert_eq!(p.protocol_revision, r.protocol_revision.to_lowercase());
    assert_eq!(p.manifest_sha256, sha256(&manifest_bytes().unwrap()));
    assert_eq!(
        p.collector_sha256,
        sha256(&fs::read(&ctx.executable).unwrap())
    );
    assert!(!r.out.exists());
}
#[test]
fn preflight_rejects_malformed_nonexistent_and_noncommit_revisions() {
    let (_tmp, ctx, mut r) = owned_git_context();
    let m = candidate().unwrap();
    for value in [
        "abc".into(),
        "g".repeat(40),
        "0".repeat(40),
        fixture_git(
            &ctx.repo,
            &[
                "rev-parse",
                "HEAD:docs/superpowers/specs/2026-10-07-foraging-5-shortcut-comparison-design.md",
            ],
        ),
    ] {
        r.protocol_revision = value;
        assert!(preflight(&ctx, &m, &r).is_err());
    }
    assert!(!r.out.exists());
}
#[test]
fn preflight_rejects_dirty_protocol_and_wrong_protocol_commit() {
    let (_tmp, ctx, r) = owned_git_context();
    let m = candidate().unwrap();
    fs::write(ctx.repo.join(&m.protocol), "wrong protocol").unwrap();
    assert!(preflight(&ctx, &m, &r)
        .unwrap_err()
        .contains("clean tracked"));
    fixture_commit(&ctx.repo);
    assert!(preflight(&ctx, &m, &r).unwrap_err().contains("protocol"));
    assert!(!r.out.exists());
}
#[test]
fn preflight_rejects_checkout_committed_and_factory_candidate_mismatches() {
    let (_tmp, ctx, mut r) = owned_git_context();
    let m = candidate().unwrap();
    fs::write(ctx.repo.join(MANIFEST_PATH), b"{}\n").unwrap();
    assert!(preflight(&ctx, &m, &r).is_err());
    r.protocol_revision = fixture_commit(&ctx.repo);
    assert!(preflight(&ctx, &m, &r).unwrap_err().contains("manifest"));
    fs::write(ctx.repo.join(MANIFEST_PATH), manifest_bytes().unwrap()).unwrap();
    fixture_commit(&ctx.repo);
    let mut changed = m;
    changed.conditions[0].options.ticks = 1;
    assert!(preflight(&ctx, &changed, &r)
        .unwrap_err()
        .contains("factory"));
    assert!(!r.out.exists());
}
#[test]
fn preflight_rejects_empty_approval_existing_output_and_executable_errors() {
    let (_tmp, mut ctx, mut r) = owned_git_context();
    let m = candidate().unwrap();
    r.approval_context = " ".into();
    assert!(preflight(&ctx, &m, &r).is_err());
    r.approval_context = "approval".into();
    fs::create_dir(&r.out).unwrap();
    assert!(preflight(&ctx, &m, &r).unwrap_err().contains("output"));
    fs::remove_dir(&r.out).unwrap();
    ctx.executable = ctx.repo.join("missing");
    assert!(preflight(&ctx, &m, &r).is_err());
    ctx.executable = ctx.repo.clone();
    assert!(preflight(&ctx, &m, &r).is_err());
    ctx.executable = ctx.repo.join("empty-executable");
    fs::write(&ctx.executable, []).unwrap();
    assert!(preflight(&ctx, &m, &r).unwrap_err().contains("empty"));
    assert!(!r.out.exists());
}
#[cfg(unix)]
#[test]
fn preflight_refuses_dangling_output_and_executable_symlinks() {
    let (_tmp, ctx, r) = owned_git_context();
    std::os::unix::fs::symlink(ctx.repo.join("missing"), &r.out).unwrap();
    assert!(preflight(&ctx, &candidate().unwrap(), &r).is_err());
    fs::remove_file(&r.out).unwrap();
    let linked = ctx.repo.join("linked-executable");
    std::os::unix::fs::symlink(&ctx.executable, &linked).unwrap();
    let linked_context = ExecutionContext {
        repo: ctx.repo.clone(),
        executable: linked,
    };
    assert!(preflight(&linked_context, &candidate().unwrap(), &r).is_err());
}
#[test]
fn committed_draft_artifact_matches_pure_factory_bytes() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join(MANIFEST_PATH);
    assert_eq!(fs::read(path).unwrap(), manifest_bytes().unwrap());
}
#[test]
fn collection_saves_all_thirty_full_canonical_episodes_and_controls() {
    let (_tmp, ctx, r) = owned_git_context();
    let index = collect(&ctx, &r).unwrap();
    let m = candidate().unwrap();
    assert_eq!(
        index.expected_keys,
        expected_keys(&m, CollectionMode::Construction)
    );
    assert_eq!(index.runs.len(), 30);
    assert!(index.completed);
    assert_eq!(index.mode, CollectionMode::Construction);
    let mut reader = ArchiveReader::open(&r.out.join("index.json")).unwrap();
    let mut protected_straight = std::collections::BTreeMap::new();
    let mut seen = Vec::new();
    while let Some((key, e)) = reader.next().unwrap() {
        let c = condition(&m, &key.condition).unwrap();
        assert_eq!(
            serde_json::to_vec(&e.setup).unwrap(),
            serde_json::to_vec(&c.setup).unwrap()
        );
        assert_eq!(
            serde_json::to_vec(&e.options).unwrap(),
            serde_json::to_vec(&c.options).unwrap()
        );
        assert_eq!(e.seed, key.seed);
        assert_eq!(e.summary.completed_ticks, 512);
        assert_eq!(e.summary.work.opportunities, 4096);
        assert_eq!(
            e.snapshots
                .iter()
                .map(|f| f.summary.completed_ticks)
                .collect::<Vec<_>>(),
            vec![0, 128, 256, 384, 512]
        );
        if c.regime != Regime::Paid {
            assert_eq!(e.summary.work.digs, 0);
            assert_eq!(e.summary.spoil.excavated, 0);
        }
        assert!(e
            .snapshots
            .iter()
            .flat_map(|f| &f.summary.access.records)
            .all(|record| record.distance.is_none_or(|d| d >= 15)));
        assert_eq!(
            e.summary.food.hidden
                + e.summary.food.available
                + e.summary.food.carried
                + e.summary.food.delivered,
            16
        );
        if c.panel == Panel::Access && c.regime == Regime::Protected {
            assert_eq!(e.summary.work.pickups, 0);
            assert_eq!(e.summary.food.delivered, 0);
            assert_eq!(e.summary.access.accessible, 0);
        }
        if key.condition == "route.straight.protected" {
            protected_straight.insert(key.seed, e.clone());
        }
        if key.condition == "route.straight.already_open" {
            assert_eq!(protected_straight[&key.seed], e);
        }
        seen.push(key);
    }
    assert_eq!(seen, index.expected_keys);
}

#[test]
fn collection_stops_at_first_failure_and_preserves_prior_receipts() {
    let (_tmp, ctx, r) = owned_git_context();
    let mut seen = Vec::new();
    let error = super::super::run::collect_with(&ctx, &r, |c, seed| {
        seen.push((c.id.clone(), seed));
        if seed == 8 {
            return Err("injected core failure".into());
        }
        super::support::cached_candidate_episode(&c.id, seed)
    })
    .unwrap_err();
    assert!(error.contains("route.straight.paid seed 8: injected core failure"));
    assert_eq!(
        seen,
        vec![
            ("route.straight.paid".into(), 7),
            ("route.straight.paid".into(), 8)
        ]
    );
    assert!(r.out.join("index.incomplete.json").is_file());
    assert!(r.out.join("progress/000.json").is_file());
    assert!(!r.out.join("progress/001.json").exists());
    assert!(!r.out.join("index.json").exists());
    let failure: serde_json::Value =
        serde_json::from_slice(&fs::read(r.out.join("failure.json")).unwrap()).unwrap();
    assert_eq!(failure["next_key"]["seed"], 8);
    assert_eq!(failure["message"], error);
}
#[test]
fn collector_preserves_primary_error_when_failure_evidence_cannot_be_written() {
    let (_tmp, ctx, r) = owned_git_context();
    let error = super::super::run::collect_with(&ctx, &r, |_, _| {
        fs::write(r.out.join("failure.json"), "existing evidence").unwrap();
        Err("primary episode failure".into())
    })
    .unwrap_err();
    assert!(error.contains("route.straight.paid seed 7: primary episode failure"));
    assert!(error.contains("also failed to preserve failure evidence"));
    assert_eq!(
        fs::read_to_string(r.out.join("failure.json")).unwrap(),
        "existing evidence"
    );
    assert!(!r.out.join("index.json").exists());
}
#[test]
fn collector_reports_raw_write_failure_without_overwriting_or_advancing() {
    let (_tmp, ctx, r) = owned_git_context();
    let mut calls = 0;
    let error = super::super::run::collect_with(&ctx, &r, |c, seed| {
        calls += 1;
        fs::write(r.out.join("raw/route.straight.paid/7.json"), "existing raw").unwrap();
        super::support::cached_candidate_episode(&c.id, seed)
    })
    .unwrap_err();
    assert!(error.contains("route.straight.paid seed 7"));
    assert_eq!(calls, 1);
    assert_eq!(
        fs::read_to_string(r.out.join("raw/route.straight.paid/7.json")).unwrap(),
        "existing raw"
    );
    assert!(r.out.join("failure.json").is_file());
    assert!(!r.out.join("progress/000.json").exists());
    assert!(!r.out.join("index.json").exists());
}
#[test]
fn pure_output_propagates_write_and_flush_errors() {
    struct FailingOutput {
        flush: bool,
    }
    impl std::io::Write for FailingOutput {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            if self.flush {
                Ok(bytes.len())
            } else {
                Err(std::io::Error::new(
                    std::io::ErrorKind::BrokenPipe,
                    "injected write failure",
                ))
            }
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Err(std::io::Error::new(
                std::io::ErrorKind::BrokenPipe,
                "injected flush failure",
            ))
        }
    }
    for flush in [false, true] {
        assert!(
            super::super::cli::write_output(&mut FailingOutput { flush }, b"manifest")
                .unwrap_err()
                .contains(if flush { "flush" } else { "write" })
        );
    }
}
