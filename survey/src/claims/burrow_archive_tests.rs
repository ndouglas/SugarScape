//! Tiny internal construction_fixture archives cannot pass the public canonical loader.
use super::*;
use std::sync::atomic::{AtomicU64, Ordering};
static NEXT: AtomicU64 = AtomicU64::new(0);
struct Temp(std::path::PathBuf);
impl Temp {
    fn new() -> Self {
        Self(std::env::temp_dir().join(format!(
            "burrow-archive-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        )))
    }
}
impl Drop for Temp {
    fn drop(&mut self) {
        if self.0.exists() {
            fs::remove_dir_all(&self.0).unwrap();
        }
    }
}
fn fixture() -> Index {
    let mut m = burrow::manifest().unwrap();
    m.status = "construction_fixture".into();
    m.conditions.clear();
    m.scientific_seeds.clear();
    m.construction_conditions.truncate(1);
    m.expected_construction_conditions = 1;
    m.expected_construction_episodes = 2;
    initial_index(
        m,
        Panel::Construction,
        "1".repeat(40),
        "2".repeat(40),
        "internal construction fixture".into(),
        "fixture-sha256".into(),
    )
}
fn saved() -> (Temp, Index) {
    let out = Temp::new();
    write_archive(fixture(), &out.0, |c, seed| {
        run_episode(c.config.clone(), seed, c.options.clone()).map_err(|e| format!("{e:?}"))
    })
    .unwrap();
    let index = serde_json::from_slice(&fs::read(out.0.join("index.json")).unwrap()).unwrap();
    (out, index)
}
fn checked(out: &Temp, index: Index) -> Result<Archive, String> {
    load_against(&out.0, index, &fixture().manifest, "fixture-sha256")
}
fn mutate_envelope(out: &Temp, index: &mut Index, change: impl FnOnce(&mut Envelope)) {
    let path = out.0.join(&index.runs[0].path);
    let mut e = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    change(&mut e);
    let bytes = serde_json::to_vec(&e).unwrap();
    fs::write(path, &bytes).unwrap();
    index.runs[0].sha256 = digest(&bytes);
}
#[test]
fn burrow_archive_scientific_gate_creates_nothing() {
    let out = Temp::new();
    let e = run(
        Panel::Scientific,
        &"0".repeat(40),
        "approval reference",
        &out.0,
    )
    .unwrap_err();
    assert!(e.contains("unregistered"));
    assert!(!out.0.exists());
}
#[test]
fn burrow_archive_real_fixture_is_valid_but_unavailable_to_public_loader() {
    let (out, index) = saved();
    let archive = checked(&out, index).unwrap();
    assert_eq!(archive.records.len(), 2);
    assert_eq!(archive.index.panel, Panel::Construction);
    assert!(load(&out.0.join("index.json"))
        .unwrap_err()
        .contains("candidate"));
    assert!(!out.0.join("analysis.json").exists());
}
#[test]
fn burrow_archive_does_not_overwrite_existing_directory_or_file() {
    let out = Temp::new();
    fs::create_dir(&out.0).unwrap();
    assert!(run(
        Panel::Construction,
        "bad-revision",
        "construction test",
        &out.0
    )
    .is_err());
    assert!(write_archive(fixture(), &out.0, |_, _| panic!("must not simulate")).is_err());
    let path = out.0.join("exclusive");
    write_new(&path, b"first").unwrap();
    assert!(write_new(&path, b"second").is_err());
    assert_eq!(fs::read(path).unwrap(), b"first");
}
#[test]
fn burrow_archive_rejects_missing_duplicate_extra_and_reordered_keys() {
    let (out, original) = saved();
    for case in 0..4 {
        let mut index = original.clone();
        match case {
            0 => {
                index.runs.remove(0);
            }
            1 => index.runs.push(index.runs[0].clone()),
            2 => index.runs[0].key.seed = "9".into(),
            _ => index.runs.reverse(),
        }
        assert!(checked(&out, index).is_err(), "case {case}");
    }
    let mut index = original;
    index.expected_keys.pop();
    assert!(checked(&out, index).is_err());
}
#[test]
fn burrow_archive_rejects_incomplete_and_index_identity_changes() {
    let (out, original) = saved();
    for case in 0..7 {
        let mut i = original.clone();
        match case {
            0 => i.completed = false,
            1 => i.code_revision = "short".into(),
            2 => i.protocol_revision = "short".into(),
            3 => i.manifest_sha256 = "wrong".into(),
            4 => i.approval_context.clear(),
            5 => i.schema = "wrong".into(),
            _ => i.manifest.construction_seeds.reverse(),
        }
        assert!(checked(&out, i).is_err());
    }
}
#[test]
fn burrow_archive_rejects_changed_bytes_with_condition_seed_context() {
    let (out, index) = saved();
    fs::write(out.0.join(&index.runs[0].path), b"{}").unwrap();
    let e = checked(&out, index).unwrap_err();
    assert!(e.contains("SHA-256") && e.contains("seed 7") && e.contains("condition"));
}
#[test]
fn burrow_archive_rejects_envelope_and_episode_mismatches_even_rehashed() {
    for case in 0..10 {
        let (out, mut index) = saved();
        mutate_envelope(&out, &mut index, |e| match case {
            0 => e.options.ticks -= 1,
            1 => e.episode.seed = "8".into(),
            2 => e.episode.config.response_weight += 1,
            3 => e.episode.setup.width += 1,
            4 => e.episode.requested_ticks -= 1,
            5 => e.episode.completed_ticks -= 1,
            6 => e.episode.stop_reason = "fabricated".into(),
            7 => e.code_revision = "3".repeat(40),
            8 => e.key.seed = "8".into(),
            _ => e.manifest_sha256 = "wrong".into(),
        });
        let error = checked(&out, index).unwrap_err();
        assert!(error.contains("seed 7"), "case {case}: {error}");
    }
}
#[test]
fn burrow_archive_rejects_parent_absolute_reused_missing_and_symlink_paths() {
    let (out, original) = saved();
    for path in ["../escape.json", "/tmp/escape.json", "missing.json", ""] {
        let mut index = original.clone();
        index.runs[0].path = path.into();
        assert!(checked(&out, index).is_err());
    }
    let mut i = original.clone();
    i.runs[1].path = i.runs[0].path.clone();
    assert!(checked(&out, i).unwrap_err().contains("reused"));
    #[cfg(unix)]
    {
        let outside = Temp::new();
        fs::create_dir(&outside.0).unwrap();
        fs::copy(
            out.0.join(&original.runs[0].path),
            outside.0.join("escape.json"),
        )
        .unwrap();
        std::os::unix::fs::symlink(outside.0.join("escape.json"), out.0.join("escape.json"))
            .unwrap();
        let mut i = original;
        i.runs[0].path = "escape.json".into();
        assert!(checked(&out, i).unwrap_err().contains("escapes"));
    }
}
#[test]
fn burrow_archive_interruption_preserves_full_identity_raw_and_receipt() {
    let out = Temp::new();
    let mut count = 0;
    let error = write_archive(fixture(), &out.0, |c, seed| {
        count += 1;
        if count == 2 {
            return Err("injected interruption".into());
        }
        run_episode(c.config.clone(), seed, c.options.clone()).map_err(|e| format!("{e:?}"))
    })
    .unwrap_err();
    assert!(error.contains("seed 8") && error.contains(&out.0.display().to_string()));
    let index: Index =
        serde_json::from_slice(&fs::read(out.0.join("index.incomplete.json")).unwrap()).unwrap();
    assert_eq!(index.expected_keys.len(), 2);
    assert!(!index.completed);
    assert!(out.0.join("raw/000-7.json").exists());
    assert!(out.0.join("progress/0000.json").exists());
    assert!(out.0.join("failure.json").exists());
    assert!(!out.0.join("index.json").exists());
    assert!(checked(&out, index).is_err());
}
#[test]
fn burrow_archive_raw_survives_progress_receipt_failure() {
    let out = Temp::new();
    let error = write_archive(fixture(), &out.0, |c, seed| {
        fs::create_dir(out.0.join("progress/0000.json")).unwrap();
        run_episode(c.config.clone(), seed, c.options.clone()).map_err(|e| format!("{e:?}"))
    })
    .unwrap_err();
    assert!(error.contains("seed 7"));
    assert!(out.0.join("raw/000-7.json").exists());
    assert!(out.0.join("failure.json").exists());
    assert!(!out.0.join("index.json").exists());
}
#[test]
fn burrow_archive_cli_flags_are_strict_and_scientific_gate_has_no_output() {
    let out = Temp::new();
    let base = vec![
        "--run".into(),
        "--protocol-revision".into(),
        "1".repeat(40),
        "--approval-context".into(),
        "test reference".into(),
        "--out".into(),
        out.0.display().to_string(),
    ];
    assert!(burrow::cli(&base).unwrap_err().contains("unregistered"));
    assert!(!out.0.exists());
    for extra in [
        vec!["--construction", "--construction"],
        vec!["--run"],
        vec!["--out", "other"],
        vec!["--approval-context"],
        vec!["--manifest"],
    ] {
        let mut args = base.clone();
        args.extend(extra.into_iter().map(String::from));
        assert!(burrow::cli(&args).is_err());
    }
}
#[test]
fn burrow_archive_public_loader_accepts_exact_canonical_36_construction_records() {
    let out = Temp::new();
    let index = initial_index(
        burrow::manifest().unwrap(),
        Panel::Construction,
        "1".repeat(40),
        "2".repeat(40),
        "construction protocol design approval 2026-10-04".into(),
        burrow::manifest_sha256(),
    );
    write_archive(index, &out.0, |c, seed| {
        run_episode(c.config.clone(), seed, c.options.clone()).map_err(|e| format!("{e:?}"))
    })
    .unwrap();
    let saved = load(&out.0.join("index.json")).unwrap();
    assert_eq!(saved.records.len(), 36);
    assert!(saved.records.keys().all(|k| k.seed == "7" || k.seed == "8"));
}
