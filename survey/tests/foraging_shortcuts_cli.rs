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
fn analysis_rejects_override_and_has_explicit_temporary_unsupported_error() {
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
    assert!(String::from_utf8_lossy(&output.stderr).contains("not yet supported"));
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
