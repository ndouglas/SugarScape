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
