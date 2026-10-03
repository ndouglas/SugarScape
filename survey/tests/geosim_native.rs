use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
    sync::atomic::{AtomicU64, Ordering},
};
static NEXT: AtomicU64 = AtomicU64::new(0);
struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        let p = std::env::temp_dir().join(format!(
            "geosim-native-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&p).unwrap();
        Self(p)
    }
    fn path(&self, name: &str) -> PathBuf {
        self.0.join(name)
    }
    fn manifest(&self, synthetic: bool) -> PathBuf {
        let mut v: Value =
            serde_json::from_str(include_str!("fixtures/geosim-registered-manifest.json")).unwrap();
        if synthetic {
            v["execution_mode"] = json!("synthetic_unregistered");
            v["arms"] = json!([
             {"id":"fixture.invalid","index":0,"family":"fixture","preset":"paper","sessions":1,"first_seed":1,"config_overrides":{"width":2,"height":2,"initial_states":4,"initialization_periods":0,"observation_periods":4,"periods_per_tick":4,"resource_adjustment":1,"damage_fraction":1,"attack_probability":1,"superiority_threshold":0.1}},
             {"id":"fixture.valid","index":1,"family":"fixture","preset":"artifact_2017","sessions":1,"first_seed":2,"config_overrides":{"width":2,"height":2,"initial_states":1,"initialization_periods":0,"observation_periods":2,"periods_per_tick":2,"shock_probability":0,"attack_probability":0}}
            ]);
        }
        let p = self.path("manifest.json");
        fs::write(&p, serde_json::to_vec(&v).unwrap()).unwrap();
        p
    }
    fn command(&self, manifest: &Path, out: &Path, resolved: &Path) -> Command {
        let mut command = Command::new(env!("CARGO_BIN_EXE_geosim"));
        command
            .args(["--manifest"])
            .arg(manifest)
            .args([
                "--manifest-sha256",
                &hash(&fs::read(manifest).unwrap()),
                "--source-root",
            ])
            .arg(Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap())
            .arg("--resolved-out")
            .arg(resolved)
            .arg("--out")
            .arg(out);
        command
    }
    fn run(&self, manifest: &Path, flags: &[&str]) -> Output {
        self.command(
            manifest,
            &self.path("sessions.jsonl"),
            &self.path("resolved.json"),
        )
        .args(flags)
        .output()
        .unwrap()
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn hash(b: &[u8]) -> String {
    format!("{:x}", Sha256::digest(b))
}
fn success(out: Output) {
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
}
#[test]
fn output_aliases_reject_before_writes_in_validation_and_execution() {
    let mut failures = Vec::new();
    for validate in [true, false] {
        let f = Fixture::new();
        let m = f.manifest(true);
        success(f.run(&m, &["--allow-unfrozen-fixture"]));
        let out = f.path("sessions.jsonl");
        let before = fs::read(&out).unwrap();
        fs::create_dir(f.path("sub")).unwrap();
        let mut aliases = vec![
            f.path("sub/../sessions.jsonl"),
            PathBuf::from("sessions.jsonl"),
            f.path("missing/../sessions.jsonl"),
        ];
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(&out, f.path("history-link")).unwrap();
            std::os::unix::fs::symlink(&f.0, f.path("directory-link")).unwrap();
            aliases.push(f.path("history-link"));
            aliases.push(f.path("directory-link/sessions.jsonl"));
        }
        for alias in aliases {
            // Restore only the temporary fixture after an unfixed recorder destroys it.
            fs::write(&out, &before).unwrap();
            let mut command = f.command(&m, &out, &alias);
            command.current_dir(&f.0).arg("--allow-unfrozen-fixture");
            if validate {
                command.arg("--validate");
            }
            let result = command.output().unwrap();
            if result.status.success() || fs::read(&out).unwrap() != before {
                failures.push(format!(
                    "validate={validate}, alias={alias:?}: exit={:?}, preserved={}",
                    result.status.code(),
                    fs::read(&out).unwrap() == before
                ));
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn missing_output_aliases_reject_without_creating_ancestors() {
    for validate in [true, false] {
        let f = Fixture::new();
        let m = f.manifest(true);
        let out = f.path("missing/nested/sessions.jsonl");
        let resolved = f.path("missing/nested/../nested/sessions.jsonl");
        let mut command = f.command(&m, &out, &resolved);
        command.arg("--allow-unfrozen-fixture");
        if validate {
            command.arg("--validate");
        }
        let result = command.output().unwrap();
        assert!(
            !result.status.success(),
            "aliased missing destinations accepted"
        );
        assert!(
            !f.path("missing").exists(),
            "ancestor created before rejection"
        );
    }
}

#[cfg(any(target_os = "macos", target_os = "windows"))]
#[test]
fn missing_case_aliases_reject_before_creating_or_mixing_outputs() {
    for validate in [true, false] {
        for nested in [true, false] {
            let f = Fixture::new();
            let m = f.manifest(true);
            let out = f.path(if nested {
                "Histories/History.jsonl"
            } else {
                "History.jsonl"
            });
            let resolved = f.path(if nested {
                "histories/history.jsonl"
            } else {
                "history.jsonl"
            });
            let mut command = f.command(&m, &out, &resolved);
            command.arg("--allow-unfrozen-fixture");
            if validate {
                command.arg("--validate");
            }
            let result = command.output().unwrap();
            assert!(
                !result.status.success(),
                "case aliases accepted: validate={validate}, nested={nested}"
            );
            assert!(!out.exists());
            assert!(!resolved.exists());
            assert!(!f.path("Histories").exists());
            assert!(!f.path("histories").exists());
        }
    }
}

#[test]
fn missing_distinct_destinations_work_and_unresolvable_ancestors_fail_before_writes() {
    let f = Fixture::new();
    let m = f.manifest(true);
    success(
        f.command(
            &m,
            &f.path("histories/nested/sessions.jsonl"),
            &f.path("receipts/nested/resolved.json"),
        )
        .arg("--allow-unfrozen-fixture")
        .output()
        .unwrap(),
    );
    assert!(f.path("histories/nested/sessions.jsonl").is_file());
    assert!(f.path("receipts/nested/resolved.json").is_file());
    fs::write(f.path("not-directory"), b"preserved ancestor file").unwrap();
    let mut invalid_outs = vec![f.path("not-directory/sessions.jsonl")];
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(f.path("absent-target"), f.path("dangling-link")).unwrap();
        invalid_outs.push(f.path("dangling-link/sessions.jsonl"));
    }
    for out in invalid_outs {
        for validate in [true, false] {
            let mut command = f.command(&m, &out, &f.path("must-not-be-written.json"));
            command.arg("--allow-unfrozen-fixture");
            if validate {
                command.arg("--validate");
            }
            assert!(!command.output().unwrap().status.success());
            assert!(!f.path("must-not-be-written.json").exists());
            assert!(!f.path("absent-target").exists());
            assert_eq!(
                fs::read(f.path("not-directory")).unwrap(),
                b"preserved ancestor file"
            );
        }
    }
}

#[test]
fn atomic_temporary_history_collision_rejects_and_preserves_actual_process_bytes() {
    for validate in [true, false] {
        let f = Fixture::new();
        let m = f.manifest(true);
        success(f.run(&m, &["--allow-unfrozen-fixture"]));
        let before = fs::read(f.path("sessions.jsonl")).unwrap();
        // exec retains Bash's PID, so this is the recorder's actual temporary pathname.
        let mut command = Command::new("bash");
        command.arg("-c").arg("history=\"$1/export.tmp-$$\"; cp \"$1/sessions.jsonl\" \"$history\"; shift; exec \"$@\" --out \"$history\"")
            .arg("geosim-temp-collision").arg(&f.0)
            .arg(env!("CARGO_BIN_EXE_geosim"))
            .arg("--manifest").arg(&m)
            .args(["--manifest-sha256", &hash(&fs::read(&m).unwrap())])
            .arg("--source-root").arg(Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap())
            .arg("--resolved-out").arg(f.path("export.json"))
            .arg("--allow-unfrozen-fixture");
        if validate {
            command.arg("--validate");
        }
        let result = command.output().unwrap();
        assert!(
            !result.status.success(),
            "history/atomic temporary collision accepted"
        );
        let history = fs::read_dir(&f.0)
            .unwrap()
            .map(|e| e.unwrap().path())
            .find(|p| {
                p.file_name()
                    .unwrap()
                    .to_string_lossy()
                    .starts_with("export.tmp-")
            })
            .unwrap();
        assert_eq!(fs::read(history).unwrap(), before);
        assert!(!f.path("export.json").exists());
    }
}

#[test]
fn malformed_attempt_diagnostics_reject_before_any_write() {
    let f = Fixture::new();
    let m = f.manifest(true);
    success(f.run(&m, &["--allow-unfrozen-fixture"]));
    let bytes = fs::read_to_string(f.path("sessions.jsonl")).unwrap();
    let baseline: Value = serde_json::from_str(bytes.lines().next().unwrap()).unwrap();
    let resolved_before = fs::read(f.path("resolved.json")).unwrap();
    let mut attempts = Vec::new();
    for errors in [
        json!([17]),
        json!([null]),
        json!([[]]),
        json!([{}]),
        json!([{"field":"config"}]),
        json!([{"message":"failure"}]),
        json!([{"field":17,"message":"failure"}]),
        json!([{"field":"config","message":false}]),
        json!([{"field":"config","message":"failure","unexpected":true}]),
    ] {
        attempts.push(
            json!({"status":"construction_error","construction_errors":errors,
            "panic_context":null,"recorder_error":null}),
        );
    }
    for context in [
        json!(null),
        json!(17),
        json!(""),
        json!(" \t\n"),
        json!("\u{0085}\u{001c}"),
    ] {
        for status in ["construction_panic", "implementation_panic", "incomplete"] {
            attempts.push(json!({"status":status,"construction_errors":[],
                "panic_context":if status=="incomplete" { Value::Null } else { context.clone() },
                "recorder_error":if status=="incomplete" { context.clone() } else { Value::Null }}));
        }
    }
    let mut failures = Vec::new();
    for attempt in attempts {
        let mut row = baseline.clone();
        row["outcome"] = Value::Null;
        row["attempt"] = attempt.clone();
        if attempt["status"] == "implementation_panic" {
            row["outcome"] = baseline["outcome"].clone();
        }
        let corrupt = format!("{row}\n");
        for validate in [true, false] {
            fs::write(f.path("sessions.jsonl"), &corrupt).unwrap();
            let flags = if validate {
                vec!["--allow-unfrozen-fixture", "--validate"]
            } else {
                vec!["--allow-unfrozen-fixture"]
            };
            let result = f.run(&m, &flags);
            if result.status.success()
                || fs::read_to_string(f.path("sessions.jsonl")).unwrap() != corrupt
                || fs::read(f.path("resolved.json")).unwrap() != resolved_before
            {
                failures.push(format!(
                    "validate={validate}, attempt={attempt}: exit={:?}",
                    result.status.code()
                ));
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn valid_unavailable_attempts_remain_resumable_without_replacing_keys() {
    let f = Fixture::new();
    let m = f.manifest(true);
    success(f.run(&m, &["--allow-unfrozen-fixture"]));
    let bytes = fs::read_to_string(f.path("sessions.jsonl")).unwrap();
    let baseline: Value = serde_json::from_str(bytes.lines().next().unwrap()).unwrap();
    for attempt in [
        json!({"status":"construction_error","construction_errors":[{"field":"config","message":"failure"}],"panic_context":null,"recorder_error":null}),
        json!({"status":"construction_panic","construction_errors":[],"panic_context":"constructor failure","recorder_error":null}),
        json!({"status":"incomplete","construction_errors":[],"panic_context":null,"recorder_error":"missing Outcome"}),
    ] {
        let mut row = baseline.clone();
        row["outcome"] = Value::Null;
        row["attempt"] = attempt;
        let retained = format!("{row}\n");
        fs::write(f.path("sessions.jsonl"), &retained).unwrap();
        success(f.run(&m, &["--allow-unfrozen-fixture", "--validate"]));
        success(f.run(
            &m,
            &["--allow-unfrozen-fixture", "--arm", "fixture.invalid"],
        ));
        assert_eq!(
            fs::read_to_string(f.path("sessions.jsonl")).unwrap(),
            retained
        );
    }
}
#[test]
fn actual_native_registered_validation_resolves_all_configs_without_histories() {
    let f = Fixture::new();
    let m = f.manifest(false);
    success(f.run(&m, &["--validate"]));
    let v: Value = serde_json::from_slice(&fs::read(f.path("resolved.json")).unwrap()).unwrap();
    assert_eq!(v["arms"].as_array().unwrap().len(), 37);
    assert!(!f.path("sessions.jsonl").exists());
    assert_eq!(
        v["binary_sha256"],
        hash(&fs::read(env!("CARGO_BIN_EXE_geosim")).unwrap())
    );
    let payload = v["resolved_configs_json"].as_str().unwrap();
    assert_eq!(v["resolved_configs_sha256"], hash(payload.as_bytes()));
    assert!(!f.run(&m, &[]).status.success());
    assert!(!f.run(&m, &["--allow-unfrozen-fixture"]).status.success());
    assert!(!f
        .run(&m, &["--validate", "--arm", "original.b"])
        .status
        .success());
}
#[test]
fn actual_native_fixture_preserves_invalid_and_resumes_exact_keys_with_all_rows_checked() {
    let f = Fixture::new();
    let m = f.manifest(true);
    assert!(!f.run(&m, &[]).status.success());
    success(f.run(&m, &["--allow-unfrozen-fixture"]));
    let bytes = fs::read(f.path("sessions.jsonl")).unwrap();
    let rows = String::from_utf8(bytes.clone())
        .unwrap()
        .lines()
        .map(|l| serde_json::from_str::<Value>(l).unwrap())
        .collect::<Vec<_>>();
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0]["attempt"]["status"], "invalid");
    assert_eq!(rows[0]["outcome"]["periods"], 3);
    assert_eq!(rows[0]["outcome"]["attempted_period"], 4);
    assert_eq!(rows[1]["attempt"]["status"], "completed");
    success(f.run(
        &m,
        &["--allow-unfrozen-fixture", "--arm", "fixture.invalid"],
    ));
    assert_eq!(fs::read(f.path("sessions.jsonl")).unwrap(), bytes);
    for field in ["binary_sha256", "resolved_configs_sha256"] {
        let mut changed = rows.clone();
        changed[1][field] = json!("a".repeat(64));
        let corrupt = changed.iter().map(|v| format!("{v}\n")).collect::<String>();
        fs::write(f.path("sessions.jsonl"), &corrupt).unwrap();
        assert!(!f
            .run(
                &m,
                &["--allow-unfrozen-fixture", "--arm", "fixture.invalid"]
            )
            .status
            .success());
        assert_eq!(
            fs::read_to_string(f.path("sessions.jsonl")).unwrap(),
            corrupt
        );
    }
    fs::write(
        f.path("sessions.jsonl"),
        format!(
            "{}{}",
            String::from_utf8(bytes.clone()).unwrap(),
            String::from_utf8(bytes).unwrap()
        ),
    )
    .unwrap();
    assert!(!f.run(&m, &["--allow-unfrozen-fixture"]).status.success());
}
fn copy_sources(root: &Path, relative: &str, destination: &Path, entries: &mut Vec<Value>) {
    let source = root.join(relative);
    if source.is_dir() {
        for e in fs::read_dir(&source).unwrap() {
            let e = e.unwrap();
            if e.file_name() == "__pycache__"
                || e.path().strip_prefix(root).unwrap() == Path::new("survey/geosim/reference/raw")
            {
                continue;
            }

            let rel = e
                .path()
                .strip_prefix(root)
                .unwrap()
                .to_str()
                .unwrap()
                .to_owned();
            if e.file_type().unwrap().is_dir()
                || rel.ends_with(".rs")
                || rel.ends_with(".py")
                || rel.ends_with(".java")
            {
                copy_sources(root, &rel, destination, entries);
            }
        }
    } else {
        let b = fs::read(&source).unwrap();
        let path = destination.join(relative);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, &b).unwrap();
        entries.push(json!({"path":relative,"bytes":b.len(),"sha256":hash(&b)}));
    }
}
#[test]
fn actual_native_frozen_fixture_recomputes_inventory_and_pre_post_receipt_bindings() {
    let f = Fixture::new();
    let original = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    let frozen_root = f.path("immutable-source-fixture");
    let mut inventory = Vec::new();
    for relative in [
        "Cargo.toml",
        "Cargo.lock",
        "crates/sugarscape-core/Cargo.toml",
        "crates/sugarscape-core/assets/sugar-map.txt",
        "crates/sugarscape-core/tests/geosim_discovery.rs",
        "crates/sugarscape-core/tests/geosim_host_sweep.rs",
        "crates/sugarscape-core/tests/geosim_invalid_sweep.rs",
        "survey/Cargo.toml",
        "survey/Cargo.lock",
        "survey/src/bin/geosim.rs",
        "survey/tests/geosim_native.rs",
        "survey/tests/fixtures/geosim-registered-manifest.json",
        "survey/geosim/.python-version",
        "survey/geosim/requirements.txt",
        "survey/geosim/README.md",
        "survey/geosim/NUMERICAL_METHODS.md",
        "survey/geosim/reference/extract.py",
        "survey/geosim/reference/test_extract.py",
        "survey/geosim/reference/test_launch.py",
        "survey/geosim/reference/GeoSimReferenceProbe.java",
        "docs/superpowers/specs/2026-10-03-geosim-source-table.json",
        "docs/superpowers/specs/2026-10-03-geosim-design.md",
        "docs/superpowers/specs/2026-10-03-geosim-source-extraction.md",
        "docs/superpowers/specs/2026-10-03-geosim-reading-notes.md",
        "docs/superpowers/specs/2026-10-03-geosim-author-code-reading-notes.md",
        "docs/superpowers/specs/2026-10-03-geosim-mechanics-reading-notes.md",
        "docs/superpowers/specs/2026-10-03-geosim-defender-threshold-amendment.md",
        "docs/superpowers/specs/2026-10-03-geosim-damage-incidence-audit.md",
        "docs/superpowers/specs/2026-10-03-geosim-artifact-audit.md",
        "docs/superpowers/specs/2026-10-03-geosim-finite-technology-amendment.md",
        "docs/superpowers/specs/2026-10-03-geosim-portability-amendment.md",
        "crates/sugarscape-core/src",
        "crates/sugarscape-core/tests",
        "survey/tests",
        "survey/geosim",
    ] {
        copy_sources(original, relative, &frozen_root, &mut inventory);
    }
    inventory.sort_by(|a, b| a["path"].as_str().cmp(&b["path"].as_str()));
    inventory.dedup_by(|a, b| a["path"] == b["path"]);
    inventory.sort_by(|a, b| a["path"].as_str().cmp(&b["path"].as_str()));
    let inventory_sha = hash(&serde_json::to_vec(&inventory).unwrap());
    let m = f.manifest(true);
    let mut manifest: Value = serde_json::from_slice(&fs::read(&m).unwrap()).unwrap();
    manifest["provenance_status"] = json!("frozen");
    manifest["source_inventory"] = json!(inventory);
    manifest["source_inventory_sha256"] = json!(inventory_sha);
    fs::write(&m, serde_json::to_vec(&manifest).unwrap()).unwrap();
    let manifest_sha = hash(&fs::read(&m).unwrap());
    let version = |tool: &str| {
        String::from_utf8(Command::new(tool).arg("--version").output().unwrap().stdout)
            .unwrap()
            .trim()
            .to_owned()
    };
    let receipt = json!({"schema_version":1,"model":"geosim","manifest_sha256":manifest_sha,"binary_sha256":hash(&fs::read(env!("CARGO_BIN_EXE_geosim")).unwrap()),"source_inventory_sha256":inventory_sha,"prebuild_inventory_sha256":inventory_sha,"postbuild_inventory_sha256":inventory_sha,"build_command":["cargo","build","--manifest-path","survey/Cargo.toml","--bin","geosim"],"cwd":frozen_root,"target":"synthetic-validator-fixture","rustc_version":version("rustc"),"cargo_version":version("cargo"),"lockfile_hashes":{"Cargo.lock":hash(&fs::read(frozen_root.join("Cargo.lock")).unwrap()),"survey/Cargo.lock":hash(&fs::read(frozen_root.join("survey/Cargo.lock")).unwrap())},"features":[],"build_flags":[]});
    let receipt_path = f.path("receipt.json");
    fs::write(&receipt_path, serde_json::to_vec(&receipt).unwrap()).unwrap();
    let run = || {
        Command::new(env!("CARGO_BIN_EXE_geosim"))
            .arg("--manifest")
            .arg(&m)
            .args(["--manifest-sha256", &manifest_sha])
            .arg("--source-root")
            .arg(&frozen_root)
            .arg("--build-receipt")
            .arg(&receipt_path)
            .arg("--resolved-out")
            .arg(f.path("resolved.json"))
            .arg("--out")
            .arg(f.path("sessions.jsonl"))
            .arg("--validate")
            .output()
            .unwrap()
    };
    success(run());
    fs::write(
        frozen_root.join("docs/superpowers/specs/2026-10-03-geosim-findings.md"),
        b"publication fixture outside normative inputs",
    )
    .unwrap();
    success(run());

    assert!(!f.path("sessions.jsonl").exists());
    let export: Value =
        serde_json::from_slice(&fs::read(f.path("resolved.json")).unwrap()).unwrap();
    assert_eq!(
        export["build_receipt_sha256"],
        hash(&fs::read(&receipt_path).unwrap())
    );
    assert_eq!(export["source_inventory_sha256"], inventory_sha);
    let mut changed = receipt.clone();
    changed["prebuild_inventory_sha256"] = json!("a".repeat(64));
    fs::write(&receipt_path, serde_json::to_vec(&changed).unwrap()).unwrap();
    assert!(!run().status.success());
    fs::write(&receipt_path, serde_json::to_vec(&receipt).unwrap()).unwrap();
    fs::write(
        frozen_root.join("crates/sugarscape-core/src/geosim/world.rs"),
        b"changed engine bytes",
    )
    .unwrap();
    assert!(!run().status.success());
    fs::write(
        frozen_root.join("crates/sugarscape-core/src/geosim/world.rs"),
        fs::read(original.join("crates/sugarscape-core/src/geosim/world.rs")).unwrap(),
    )
    .unwrap();
    let mut changed = manifest;
    changed["source_inventory"]
        .as_array_mut()
        .unwrap()
        .retain(|e| e["path"] != "crates/sugarscape-core/src/geosim/world.rs");
    changed["source_inventory_sha256"] = json!(hash(
        &serde_json::to_vec(&changed["source_inventory"]).unwrap()
    ));
    fs::write(&m, serde_json::to_vec(&changed).unwrap()).unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_geosim"))
        .arg("--manifest")
        .arg(&m)
        .args(["--manifest-sha256", &hash(&fs::read(&m).unwrap())])
        .arg("--source-root")
        .arg(&frozen_root)
        .arg("--build-receipt")
        .arg(&receipt_path)
        .arg("--resolved-out")
        .arg(f.path("resolved.json"))
        .arg("--out")
        .arg(f.path("sessions.jsonl"))
        .arg("--validate")
        .output()
        .unwrap();
    assert!(!out.status.success());
    assert!(String::from_utf8_lossy(&out.stderr).contains("omits required"));
}

#[test]
fn omitted_resolved_output_defaults_adjacent_to_sessions_without_worlds() {
    let f = Fixture::new();
    let manifest = f.manifest(false);
    let out = Command::new(env!("CARGO_BIN_EXE_geosim"))
        .arg("--manifest")
        .arg(&manifest)
        .args(["--manifest-sha256", &hash(&fs::read(&manifest).unwrap())])
        .arg("--source-root")
        .arg(Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap())
        .arg("--out")
        .arg(f.path("sessions.jsonl"))
        .arg("--validate")
        .output()
        .unwrap();
    success(out);
    let export: Value =
        serde_json::from_slice(&fs::read(f.path("sessions.jsonl.resolved.json")).unwrap()).unwrap();
    assert_eq!(export["arms"].as_array().unwrap().len(), 37);
    assert!(!f.path("sessions.jsonl").exists());
}
