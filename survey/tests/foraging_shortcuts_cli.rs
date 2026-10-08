use std::{
    fs,
    process::{Command, Output},
    sync::atomic::{AtomicU64, Ordering},
};
struct OwnedDir(std::path::PathBuf);
impl Drop for OwnedDir {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}
fn owned_dir() -> OwnedDir {
    static NEXT: AtomicU64 = AtomicU64::new(0);
    loop {
        let path = std::env::temp_dir().join(format!(
            "f5-cli-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        match fs::create_dir(&path) {
            Ok(()) => return OwnedDir(path),
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(e) => panic!("{e}"),
        }
    }
}
fn binary(args: &[&str], directory: &OwnedDir) -> Output {
    Command::new(env!("CARGO_BIN_EXE_survey"))
        .current_dir(&directory.0)
        .args(args)
        .output()
        .unwrap()
}
// Guard RED runs against the legacy generic survey fallback, which executes worlds.
fn assert_selector_available(directory: &OwnedDir) {
    let output = binary(&["--foraging-shortcuts", "--help"], directory);
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("survey --foraging-shortcuts"),
        "new pure selector is not available"
    );
}
#[test]
fn default_shortcut_command_prints_draft_manifest_without_files() {
    let dir = owned_dir();
    assert_selector_available(&dir);
    let output = binary(&["--foraging-shortcuts"], &dir);
    assert!(output.status.success());
    let m: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(m["schema"], "foraging-shortcut-manifest-v1");
    assert_eq!(m["status"], "draft");
    assert_eq!(m["execution_authorized"], false);
    assert_eq!(fs::read_dir(&dir.0).unwrap().count(), 0);
}
#[test]
fn help_is_pure_and_draft_scientific_run_rejects_before_output() {
    let dir = owned_dir();
    assert_selector_available(&dir);
    let output = binary(
        &[
            "--foraging-shortcuts",
            "--run",
            "--protocol-revision",
            &"a".repeat(40),
            "--approval-context",
            "test",
            "--out",
            "new",
        ],
        &dir,
    );
    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("draft"));
    assert_eq!(fs::read_dir(&dir.0).unwrap().count(), 0);
}
#[test]
fn selector_collision_is_rejected_before_any_dispatch() {
    let dir = owned_dir();
    for selector in [
        "--burrow",
        "--protection",
        "--minds9",
        "--calibration",
        "--presets",
        "--baseline",
        "--usage",
        "--only",
    ] {
        let output = binary(&[selector, "--foraging-shortcuts", "--help"], &dir);
        assert_eq!(
            output.status.code(),
            Some(2),
            "accepted collision {selector}"
        );
        assert!(String::from_utf8_lossy(&output.stderr).contains("foraging-shortcuts"));
    }
    assert_eq!(fs::read_dir(&dir.0).unwrap().count(), 0);
}
#[test]
fn analysis_rejects_override_and_missing_saved_index_before_output() {
    let dir = owned_dir();
    assert_selector_available(&dir);
    let output = binary(
        &[
            "--foraging-shortcuts",
            "--analyze",
            "index.json",
            "--out",
            "new",
        ],
        &dir,
    );
    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("index.json"));
    let output = binary(
        &[
            "--foraging-shortcuts",
            "--analyze",
            "index.json",
            "--out",
            "new",
            "--seeds",
            "1",
        ],
        &dir,
    );
    assert_eq!(output.status.code(), Some(2));
    assert_eq!(fs::read_dir(&dir.0).unwrap().count(), 0);
}

#[test]
fn existing_selector_help_remains_available() {
    let dir = owned_dir();
    for selector in ["--burrow", "--protection", "--minds9"] {
        let output = binary(&[selector, "--help"], &dir);
        assert!(
            output.status.success(),
            "existing help {selector}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(String::from_utf8_lossy(&output.stdout).contains(selector));
    }
    let output = binary(&["--help"], &dir);
    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("--foraging-shortcuts"));
    assert_eq!(fs::read_dir(&dir.0).unwrap().count(), 0);
}

fn digest(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    format!("{:x}", Sha256::digest(bytes))
}
/// Construction-only external fixture; serialize original core Episodes directly,
/// never reconstruct their numeric tokens through serde_json::Value.
fn saved_archive(dir: &OwnedDir) -> std::path::PathBuf {
    use sugarscape_core::foraging::construction as core;
    let manifest_bytes = binary(&["--foraging-shortcuts"], dir).stdout;
    let manifest: serde_json::Value = serde_json::from_slice(&manifest_bytes).unwrap();
    let provenance = serde_json::json!({"code_revision":"a".repeat(40),
        "protocol_revision":"b".repeat(40), "manifest_sha256":digest(&manifest_bytes),
        "collector_sha256":"c".repeat(64)});
    let root = dir.0.join("saved");
    fs::create_dir(&root).unwrap();
    let mut runs = Vec::new();
    let mut keys = Vec::new();
    let mut total = 0u64;
    let pos = |p: &serde_json::Value| core::Pos {
        x: p["x"].as_u64().unwrap() as u32,
        y: p["y"].as_u64().unwrap() as u32,
    };
    let positions =
        |p: &serde_json::Value| p.as_array().unwrap().iter().map(pos).collect::<Vec<_>>();
    for condition in manifest["conditions"].as_array().unwrap() {
        let s = &condition["setup"];
        let p = &s["parameters"];
        let setup = core::Setup {
            width: s["width"].as_u64().unwrap() as u32,
            height: s["height"].as_u64().unwrap() as u32,
            open: positions(&s["open"]),
            diggable: positions(&s["diggable"]),
            waste: pos(&s["waste"]),
            nest: positions(&s["nest"]),
            workers: positions(&s["workers"]),
            food: s["food"]
                .as_array()
                .unwrap()
                .iter()
                .map(|r| core::Resource {
                    id: r["id"].as_u64().unwrap(),
                    pos: pos(&r["pos"]),
                })
                .collect(),
            parameters: core::Parameters {
                p_search: p["p_search"].as_f64().unwrap(),
                p_return: p["p_return"].as_f64().unwrap(),
                lambda_fidelity: p["lambda_fidelity"].as_f64().unwrap(),
                lambda_publish: p["lambda_publish"].as_f64().unwrap(),
                lambda_waypoint: p["lambda_waypoint"].as_f64().unwrap(),
            },
        };
        for seed in [7, 8] {
            let episode = core::run(
                setup.clone(),
                seed,
                core::RunOptions {
                    ticks: 512,
                    sample_every: 128,
                    snapshots: true,
                },
            )
            .unwrap();
            let key = serde_json::json!({"condition": condition["id"], "seed":seed});
            #[derive(serde::Serialize)]
            struct Envelope<'a> {
                schema: &'a str,
                key: &'a serde_json::Value,
                mode: &'a str,
                provenance: &'a serde_json::Value,
                episode: &'a core::Episode,
            }
            let bytes = serde_json::to_vec(&Envelope {
                schema: "foraging-shortcut-episode-v1",
                key: &key,
                mode: "construction",
                provenance: &provenance,
                episode: &episode,
            })
            .unwrap();
            let relative = format!("raw/{}/{seed}.json", condition["id"].as_str().unwrap());
            let path = root.join(&relative);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, &bytes).unwrap();
            total += bytes.len() as u64;
            runs.push(serde_json::json!({"key":key,"path":relative,"sha256":digest(&bytes),"bytes":bytes.len()}));
            keys.push(key);
        }
    }
    let index = serde_json::json!({"schema":"foraging-shortcut-archive-v1", "manifest":manifest,
        "mode":"construction", "provenance":provenance, "approval_context":"external construction engineering fixture",
        "expected_keys":keys,"completed":true,"runs":runs,"raw_bytes":total});
    let path = root.join("index.json");
    fs::write(&path, serde_json::to_vec(&index).unwrap()).unwrap();
    path
}
#[test]
fn saved_analysis_cli_reproduces_complete_construction_artifacts_exclusively() {
    let dir = owned_dir();
    assert_selector_available(&dir);
    let index = saved_archive(&dir);
    for out in ["first", "second"] {
        let output = binary(
            &[
                "--foraging-shortcuts",
                "--analyze",
                index.to_str().unwrap(),
                "--out",
                out,
            ],
            &dir,
        );
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    let a: serde_json::Value =
        serde_json::from_slice(&fs::read(dir.0.join("first/analysis.json")).unwrap()).unwrap();
    assert_eq!(a["rows"].as_array().unwrap().len(), 30);
    assert_eq!(a["contrasts"], serde_json::json!([]));
    for file in ["analysis.json", "results.md"] {
        assert_eq!(
            fs::read(dir.0.join("first").join(file)).unwrap(),
            fs::read(dir.0.join("second").join(file)).unwrap()
        );
    }
    let output = binary(
        &[
            "--foraging-shortcuts",
            "--analyze",
            index.to_str().unwrap(),
            "--out",
            "first",
        ],
        &dir,
    );
    assert_eq!(output.status.code(), Some(2));
    let mut raw = fs::read(dir.0.join("saved/raw/access.twisting.protected/8.json")).unwrap();
    raw.push(b' ');
    fs::write(
        dir.0.join("saved/raw/access.twisting.protected/8.json"),
        raw,
    )
    .unwrap();
    let output = binary(
        &[
            "--foraging-shortcuts",
            "--analyze",
            index.to_str().unwrap(),
            "--out",
            "bad",
        ],
        &dir,
    );
    assert_eq!(output.status.code(), Some(2));
    assert!(!dir.0.join("bad").exists());
}
