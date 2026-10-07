use std::process::{Command, Stdio};
use sugarscape_core::shared_surface::{report_integrity, DiagnosticReport};

#[test]
fn shared_surface_usage_requires_diagnose_and_rejects_tuning() {
    let binary = env!("CARGO_BIN_EXE_sugarscape");
    let help = Command::new(binary)
        .args(["shared-surface", "--help"])
        .output()
        .unwrap();
    assert!(help.status.success());
    for args in [
        vec!["shared-surface"],
        vec!["shared-surface", "diagnose", "--seed", "7"],
        vec!["shared-surface", "diagnose", "--fixture"],
    ] {
        let output = Command::new(binary).args(args).output().unwrap();
        assert_eq!(output.status.code(), Some(2));
    }
}

// Authored before source freeze. First execution is gated on retaining first collection.
#[test]
fn shared_surface_diagnose_success() {
    let unique = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = std::env::temp_dir().join(format!(
        "sugarscape-shared-surface-{}-{unique}.json",
        std::process::id()
    ));
    let file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
        .unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_sugarscape"))
        .args(["shared-surface", "diagnose"])
        .stdin(Stdio::null())
        .stdout(Stdio::from(file))
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let reader = std::io::BufReader::new(std::fs::File::open(&path).unwrap());
    let mut report: DiagnosticReport = serde_json::from_reader(reader).unwrap();
    assert_eq!(report.version, "shared-surface-diagnostic-v1");
    assert!(report.passed);
    assert!(report_integrity(&report).unwrap());
    assert_eq!(report.primary.len(), 48);
    assert_eq!(report.secondary.len(), 172);
    report.primary[0].episodes[0].agent_traces[0] = u64::MAX;
    assert!(!report_integrity(&report).unwrap());
    report.primary[0].episodes[0].agent_traces[0] = 0;
    report.checks[0].passed = false;
    assert!(!report_integrity(&report).unwrap());
    std::fs::remove_file(path).unwrap();
}
