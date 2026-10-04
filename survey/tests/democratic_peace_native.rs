//! Boundary tests execute the actual native recorder, never a mocked world.
use std::{
    fs,
    process::Command,
    sync::atomic::{AtomicU64, Ordering},
};
static SERIAL: AtomicU64 = AtomicU64::new(0);
fn directory() -> std::path::PathBuf {
    let p = std::env::temp_dir().join(format!(
        "democratic-peace-native-{}-{}",
        std::process::id(),
        SERIAL.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir_all(&p).unwrap();
    p
}
#[test]
fn duplicate_manifest_fields_are_rejected_before_resolved_export() {
    let d = directory();
    let manifest = d.join("manifest.json");
    let resolved = d.join("resolved.json");
    fs::write(
        &manifest,
        r#"{"model":"democratic_peace","model":"democratic_peace"}"#,
    )
    .unwrap();
    let result = Command::new(env!("CARGO_BIN_EXE_democratic_peace"))
        .args([
            "--manifest",
            manifest.to_str().unwrap(),
            "--resolved",
            resolved.to_str().unwrap(),
            "--validate-only",
        ])
        .output()
        .unwrap();
    assert!(!result.status.success());
    assert!(String::from_utf8_lossy(&result.stderr).contains("duplicate JSON field"));
    assert!(!resolved.exists());
    fs::remove_dir_all(d).unwrap();
}
#[test]
fn malformed_manifest_is_rejected_before_output_creation() {
    let d = directory();
    let manifest = d.join("manifest.json");
    let out = d.join("sessions.jsonl");
    fs::write(&manifest, b"{not JSON}").unwrap();
    let result = Command::new(env!("CARGO_BIN_EXE_democratic_peace"))
        .args([
            "--manifest",
            manifest.to_str().unwrap(),
            "--out",
            out.to_str().unwrap(),
            "--resolved",
            d.join("resolved.json").to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(!result.status.success());
    assert!(!out.exists());
    fs::remove_dir_all(d).unwrap();
}

// Each process fixture copies only the complete scientific inventory into an
// owned temporary repo, so even a regressed output guard cannot touch originals.
struct Fixture {
    dir: std::path::PathBuf,
    repo: std::path::PathBuf,
}
impl Fixture {
    fn new() -> Self {
        let dir = directory();
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap();
        let script = r#"
from pathlib import Path
import json, shutil, subprocess, sys
from survey.democratic_peace.provenance import required_inventory_paths, bind_unregistered_manifest
from survey.democratic_peace.manifest import build_unregistered_manifest, build_manifest
from survey.democratic_peace.records import strict_json
from survey.democratic_peace.run import make_build_receipt
root, out, binary = map(Path, sys.argv[1:])
repo = out/'repo'; repo.mkdir()
for name in required_inventory_paths(root):
    dest = repo/name; dest.parent.mkdir(parents=True, exist_ok=True)
    shutil.copyfile(root/name, dest)
source = (repo/'docs/superpowers/specs/2026-10-03-democratic-peace-source-table.json').read_bytes()
table = strict_json(source)
configs = [dict(width=3, height=2, horizon_periods=7, periods_per_tick=3,
    initial_democratic_share=d) for d in (0., 1., 0.)]
configs[-1].update(initial_resourced_share=0., zero_ratio='reject_zero_denominator')
m = bind_unregistered_manifest(build_unregistered_manifest(table, source,
    configurations=configs, sessions=2), repo, source)
mp = out/'manifest.json'; mp.write_text(json.dumps(m)+'\n')
bp = out/'recorder'; shutil.copyfile(binary, bp)
rustc = subprocess.check_output(['rustc', '-Vv'], text=True)
cargo = subprocess.check_output(['cargo', '--version'], text=True)
target = next(s.split(': ',1)[1] for s in rustc.splitlines() if s.startswith('host: '))
r = make_build_receipt(mp, bp, repo, prebuild_inventory_sha256=m['source_inventory_sha256'],
    build_command=['cargo','test','--locked','--manifest-path','survey/Cargo.toml',
        '--bin','democratic_peace','--test','democratic_peace_native'], build_exit_code=0,
    toolchain=dict(target=target, rustc_version=rustc, cargo_version=cargo),
    lockfile_paths=['Cargo.lock','survey/Cargo.lock'])
(out/'receipt.json').write_text(json.dumps(r)+'\n')
(out/'provisional.json').write_text(json.dumps(build_manifest(table, source, precision_registered=True))+'\n')
"#;
        let result = Command::new("python3")
            .args(["-c", script])
            .arg(root)
            .arg(&dir)
            .arg(env!("CARGO_BIN_EXE_democratic_peace"))
            .current_dir(root)
            .output()
            .unwrap();
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
        let fixture = Self {
            repo: dir.join("repo"),
            dir,
        };
        let result = fixture.run("manifest.json", "resolved.json", Some("sessions.jsonl"));
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
        fixture
    }
    fn run(&self, manifest: &str, resolved: &str, raw: Option<&str>) -> std::process::Output {
        let mut command = Command::new(env!("CARGO_BIN_EXE_democratic_peace"));
        command
            .arg("--manifest")
            .arg(self.dir.join(manifest))
            .arg("--resolved")
            .arg(self.dir.join(resolved))
            .arg("--repo")
            .arg(&self.repo);
        if let Some(raw) = raw {
            command
                .arg("--out")
                .arg(self.dir.join(raw))
                .arg("--receipt")
                .arg(self.dir.join("receipt.json"));
        } else {
            command.arg("--validate-only");
        }
        command.output().unwrap()
    }
    fn rows(&self) -> Vec<serde_json::Value> {
        fs::read_to_string(self.dir.join("sessions.jsonl"))
            .unwrap()
            .lines()
            .map(|line| serde_json::from_str(line).unwrap())
            .collect()
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.dir).unwrap();
    }
}

#[test]
fn corrupt_scientific_rows_reject_without_touching_raw_or_resolved_outputs() {
    use serde_json::json;
    let f = Fixture::new();
    let baseline = f.rows();
    let mut corruptions = Vec::new();
    let mut add = |name: &str, index: usize, mutate: fn(&mut serde_json::Value)| {
        let mut rows = baseline.clone();
        mutate(&mut rows[index]);
        corruptions.push((name.to_string(), rows));
    };
    add("empty-final", 0, |r| {
        r["outcome"]["outcome"]["final_metrics"] = json!({})
    });
    add("unknown-science-field", 0, |r| {
        r["outcome"]["extra"] = json!(0)
    });
    add("unknown-metric-field", 0, |r| {
        r["outcome"]["current_metrics"]["extra"] = json!(0)
    });
    add("missing-metric-field", 0, |r| {
        r["outcome"]["current_metrics"]
            .as_object_mut()
            .unwrap()
            .remove("conflict_fronts");
    });
    add("unknown-counter", 0, |r| {
        r["outcome"]["census"]["extra"] = json!(0)
    });
    add("fractional-counter", 0, |r| {
        r["outcome"]["census"]["initiated_fronts"] = json!(0.0)
    });
    add("unknown-setup", 0, |r| {
        r["outcome"]["setup"]["extra"] = json!(0)
    });
    add("setup-overflow", 0, |r| {
        r["outcome"]["setup"]["initial_resourced_cells"] = json!(7)
    });
    add("unknown-terminal", 0, |r| {
        r["outcome"]["outcome"]["extra"] = json!(0)
    });
    add("terminal-census", 0, |r| {
        r["outcome"]["outcome"]["census"]["initiated_fronts"] = json!(1)
    });
    add("terminal-clock", 0, |r| {
        r["outcome"]["outcome"]["attempted_period"] = json!(6)
    });
    add("period-clock", 0, |r| r["outcome"]["period"] = json!(6));
    add("display-clock", 0, |r| {
        r["outcome"]["last_tick_periods"] = json!(4)
    });
    add("state-hash", 0, |r| {
        r["outcome"]["final_state_hash"] = json!("corrupt")
    });
    add("atomic-gap", 4, |r| {
        r["outcome"]["attempted_period"] = json!(3);
        r["outcome"]["outcome"]["attempted_period"] = json!(3);
    });
    add("invalid-context", 4, |r| {
        r["outcome"]["outcome"]["invalid_phase"] = json!("")
    });
    add("terminal-bool-type", 0, |r| {
        r["outcome"]["outcome"]["valid"] = json!(1)
    });
    add("share-census", 0, |r| {
        r["outcome"]["current_metrics"]["democratic_share"] = json!(0.5);
        r["outcome"]["outcome"]["final_metrics"] = r["outcome"]["current_metrics"].clone();
    });
    add("regime-census", 0, |r| {
        r["outcome"]["current_metrics"]["predatory_states"] = json!(0);
        r["outcome"]["outcome"]["final_metrics"] = r["outcome"]["current_metrics"].clone();
    });
    add("imputed-clustering", 0, |r| {
        r["outcome"]["current_metrics"]["clustering_ratio"] = json!(0.0);
        r["outcome"]["outcome"]["final_metrics"] = r["outcome"]["current_metrics"].clone();
    });
    add("regime-reason", 0, |r| {
        r["outcome"]["current_metrics"]["democratic_size_reason"] = json!(null);
        r["outcome"]["outcome"]["final_metrics"] = r["outcome"]["current_metrics"].clone();
    });
    add("first-passage", 0, |r| {
        r["outcome"]["current_metrics"]["first_extinction_period"] = json!(8);
        r["outcome"]["outcome"]["final_metrics"] = r["outcome"]["current_metrics"].clone();
    });
    add("missing-endpoint", 0, |r| {
        r["outcome"]["current_metrics"]["first_extinction_period"] = json!(null);
        r["outcome"]["outcome"]["final_metrics"] = r["outcome"]["current_metrics"].clone();
    });
    add("panic-science", 0, |r| {
        r["attempt"]["status"] = json!("implementation_panic");
        r["attempt"]["panic_context"] = json!("fixture");
        r["outcome"]["outcome"] = json!(null);
        r["outcome"]["current_metrics"] = json!({});
    });
    add("incomplete-science", 0, |r| {
        r["attempt"]["status"] = json!("incomplete");
        r["attempt"]["recorder_error"] = json!("fixture");
        r["outcome"]["outcome"] = json!(null);
        r["outcome"]["setup"] = json!({});
    });
    add("panic-envelope", 0, |r| {
        r["attempt"]["status"] = json!("implementation_panic");
        r["attempt"]["panic_context"] = json!("fixture");
        r["outcome"]["seed"] = json!(0);
    });
    let mut failures = Vec::new();
    for name in ["duplicate", "truncated"] {
        let original = fs::read(f.dir.join("sessions.jsonl")).unwrap();
        let mut bytes = original.clone();
        if name == "duplicate" {
            bytes.extend(format!("{}\n", baseline[0]).as_bytes());
        } else {
            bytes.pop();
        }
        for existed in [false, true] {
            fs::write(f.dir.join("corrupt.jsonl"), &bytes).unwrap();
            let resolved = f.dir.join("reject-resolved.json");
            if existed {
                fs::write(&resolved, b"existing output sentinel").unwrap();
            }
            let before = fs::read(&resolved).ok();
            let result = f.run(
                "manifest.json",
                "reject-resolved.json",
                Some("corrupt.jsonl"),
            );
            if result.status.success()
                || fs::read(f.dir.join("corrupt.jsonl")).unwrap() != bytes
                || fs::read(&resolved).ok() != before
            {
                failures.push(format!(
                    "{name}/existing={existed}: failed write-free rejection"
                ));
            }
            if resolved.exists() {
                fs::remove_file(resolved).unwrap();
            }
        }
    }
    for (name, rows) in corruptions {
        // Include a missing last key: acceptance would also append a history.
        let bytes = rows[..5]
            .iter()
            .map(|r| format!("{r}\n"))
            .collect::<String>();
        for existed in [false, true] {
            fs::write(f.dir.join("corrupt.jsonl"), &bytes).unwrap();
            let resolved = f.dir.join("reject-resolved.json");
            if existed {
                fs::write(&resolved, b"existing output sentinel").unwrap();
            }
            let before = fs::read(&resolved).ok();
            let result = f.run(
                "manifest.json",
                "reject-resolved.json",
                Some("corrupt.jsonl"),
            );
            if result.status.success()
                || fs::read(f.dir.join("corrupt.jsonl")).unwrap() != bytes.as_bytes()
                || fs::read(&resolved).ok() != before
            {
                failures.push(format!(
                    "{name}/existing={existed}: exit={:?}, stderr={}",
                    result.status.code(),
                    String::from_utf8_lossy(&result.stderr)
                ));
            }
            if resolved.exists() {
                fs::remove_file(resolved).unwrap();
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn canonical_registered_arm_and_analysis_job_drift_reject_before_outputs() {
    use serde_json::json;
    let f = Fixture::new();
    let baseline: serde_json::Value =
        serde_json::from_slice(&fs::read(f.dir.join("provisional.json")).unwrap()).unwrap();
    let mut changes = Vec::new();
    for name in [
        "arm-id",
        "job-id",
        "job-state",
        "job-root",
        "job-spawn",
        "job-index",
        "job-order",
    ] {
        let mut value = baseline.clone();
        match name {
            "arm-id" => value["arms"][0]["id"] = json!("arbitrary-noncanonical-name"),
            "job-id" => value["analysis_jobs"][0]["id"] = json!("arbitrary-job"),
            "job-state" => value["analysis_jobs"][0]["state_u32"][0] = json!(0),
            "job-root" => value["analysis_jobs"][0]["root_entropy"] = json!(0),
            "job-spawn" => value["analysis_jobs"][0]["spawn_key"] = json!([1]),
            "job-index" => value["analysis_jobs"][0]["index"] = json!(1),
            "job-order" => value["analysis_jobs"].as_array_mut().unwrap().swap(0, 1),
            _ => unreachable!(),
        }
        changes.push((name, value));
    }
    let mut failures = Vec::new();
    for (name, value) in changes {
        fs::write(f.dir.join("changed.json"), format!("{value}\n")).unwrap();
        let result = f.run("changed.json", "changed-resolved.json", None);
        if result.status.success() || f.dir.join("changed-resolved.json").exists() {
            failures.push(name);
        }
        if f.dir.join("changed-resolved.json").exists() {
            fs::remove_file(f.dir.join("changed-resolved.json")).unwrap();
        }
    }
    assert!(
        failures.is_empty(),
        "accepted or wrote outputs: {failures:?}"
    );
    let valid = f.run("provisional.json", "valid-resolved.json", None);
    assert!(
        valid.status.success(),
        "{}",
        String::from_utf8_lossy(&valid.stderr)
    );
}

#[test]
fn analysis_cli_rejects_input_and_source_aliases_before_either_write() {
    let f = Fixture::new();
    let script = r#"
from pathlib import Path
import json, os, subprocess, sys
out, repo = map(Path, sys.argv[1:])
m = json.loads((out/'manifest.json').read_bytes())
inputs = [out/'manifest.json', repo/'docs/superpowers/specs/2026-10-03-democratic-peace-source-table.json',
    out/'sessions.jsonl', out/'resolved.json', out/'receipt.json', out/'recorder']
protected = inputs + [repo/e['path'] for e in m['source_inventory']]
snapshots = {p:p.read_bytes() for p in protected}
failures = []
cases = [(name, path) for name, path in zip(('manifest','source','resolved','receipt'), (inputs[0],inputs[1],inputs[3],inputs[4]))]
session_alias=out/'sessions-alias.json'; os.link(inputs[2],session_alias)
cases.append(('sessions',session_alias))
cases += [('inventory', repo/'docs/superpowers/specs/2026-10-03-democratic-peace-reading-notes.md')]
for kind, target in [('symlink', inputs[0]), ('hardlink', inputs[0]), ('binary', inputs[-1])]:
    link=out/(kind+'.json')
    if kind == 'symlink': link.symlink_to(target)
    else: os.link(target,link)
    cases.append((kind,link))
parent=out/'parent-link';parent.symlink_to(out, target_is_directory=True)
cases.append(('parent-symlink',parent/'manifest.json'))
cases.append(('absent-parent',out/'uncreated'/'..'/'manifest.json'))
paired=out/'paired.json';paired.write_bytes(b'paired output sentinel');os.link(paired,out/'paired.md')
cases.append(('output-hardlinks',paired))
for name, output in cases:
    destinations = [output.with_suffix('.json'), output.with_suffix('.md')]
    # Preserve a sibling sentinel wherever the parent already exists.
    sibling = destinations[1]
    if sibling.parent.exists() and sibling not in snapshots and not sibling.exists(): sibling.write_bytes(b'sibling output sentinel')
    before = {p:p.read_bytes() if p.exists() else None for p in destinations}
    command=[sys.executable,'-m','survey.democratic_peace.analysis','--manifest',str(inputs[0]),
        '--source',str(inputs[1]),'--sessions',str(inputs[2]),'--resolved',str(inputs[3]),
        '--build-receipt',str(inputs[4]),'--binary',str(inputs[5]),'--source-root',str(repo),
        '--output',str(output)]
    result=subprocess.run(command,capture_output=True,text=True)
    unchanged=all(p.read_bytes()==data for p,data in snapshots.items()) and all(
        (p.read_bytes() if p.exists() else None)==data for p,data in before.items())
    if result.returncode == 0 or 'output aliases input or another output' not in result.stderr or not unchanged or (out/'uncreated').exists():
        failures.append(dict(case=name,exit=result.returncode,unchanged=unchanged,stderr=result.stderr))
    for p,data in snapshots.items(): p.write_bytes(data)
    for p,data in before.items():
        if data is not None: p.write_bytes(data)
        elif p.exists(): p.unlink()
    if (out/'uncreated').exists(): (out/'uncreated').rmdir()
print(json.dumps(failures, indent=2))
assert not failures
"#;
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap();
    let result = Command::new("python3")
        .args(["-c", script])
        .arg(&f.dir)
        .arg(&f.repo)
        .current_dir(root)
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&result.stdout),
        String::from_utf8_lossy(&result.stderr)
    );
}

#[test]
fn valid_complete_invalid_construction_and_partial_outcomes_remain_resumable() {
    use serde_json::json;
    let f = Fixture::new();
    let baseline = f.rows();
    for status in [
        "completed",
        "invalid",
        "construction_error",
        "construction_panic",
        "implementation_panic",
        "incomplete",
    ] {
        let mut rows = baseline.clone();
        if status != "completed" && status != "invalid" {
            rows[0]["attempt"]["status"] = json!(status);
            match status {
                "construction_error" => {
                    rows[0]["attempt"]["construction_errors"] =
                        json!([{"field":"width","message":"fixture"}]);
                    rows[0]["outcome"] = json!(null);
                }
                "construction_panic" => {
                    rows[0]["attempt"]["panic_context"] = json!("fixture");
                    rows[0]["outcome"] = json!(null);
                }
                "implementation_panic" | "incomplete" => {
                    let context = if status == "incomplete" {
                        "recorder_error"
                    } else {
                        "panic_context"
                    };
                    rows[0]["attempt"][context] = json!("fixture");
                    for key in [
                        "period",
                        "periods",
                        "completed_periods",
                        "attempted_period",
                        "tick",
                        "last_tick_periods",
                    ] {
                        rows[0]["outcome"][key] = json!(1);
                    }
                    rows[0]["outcome"]["outcome"] = json!(null);
                }
                _ => unreachable!(),
            }
        }
        let bytes = rows.iter().map(|r| format!("{r}\n")).collect::<String>();
        fs::write(f.dir.join("valid.jsonl"), &bytes).unwrap();
        let result = f.run("manifest.json", "valid-resolved.json", Some("valid.jsonl"));
        assert!(
            result.status.success(),
            "{status}: {}",
            String::from_utf8_lossy(&result.stderr)
        );
        {
            assert_eq!(
                fs::read(f.dir.join("valid.jsonl")).unwrap(),
                bytes.as_bytes()
            );
        }
    }
}
