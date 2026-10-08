use std::process::{Command, Stdio};
use sugarscape_core::active_surface::{report_integrity, DiagnosticReport};

#[test]
fn active_surface_usage_requires_diagnose_and_rejects_tuning() {
    let binary = env!("CARGO_BIN_EXE_sugarscape");
    let help = Command::new(binary)
        .args(["active-surface", "--help"])
        .output()
        .unwrap();
    assert!(help.status.success());
    for args in [
        vec!["active-surface"],
        vec!["active-surface", "other"],
        vec!["active-surface", "diagnose", "extra"],
        vec!["active-surface", "diagnose", "--seed", "7"],
        vec!["active-surface", "diagnose", "--fixture"],
        vec!["active-surface", "diagnose", "--prior", "known"],
        vec!["active-surface", "diagnose", "--rounds", "0"],
    ] {
        let output = Command::new(binary).args(args).output().unwrap();
        assert_eq!(output.status.code(), Some(2));
    }
}

// Authored before source freeze; first execution follows retained first/repeat collection.
#[test]
fn active_surface_diagnose_success() {
    let unique = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = std::env::temp_dir().join(format!(
        "sugarscape-active-surface-{}-{unique}.json",
        std::process::id()
    ));
    let file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
        .unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_sugarscape"))
        .args(["active-surface", "diagnose"])
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
    assert_eq!(report.version, "active-surface-diagnostic-v1");
    assert!(report.passed);
    assert_eq!(report.panels.len(), 46);
    assert_eq!(
        report
            .panels
            .iter()
            .map(|p| p.episodes.len())
            .sum::<usize>(),
        11776
    );
    assert!(report_integrity(&report).unwrap());
    report.panels[0].episodes[0].decisions[0].decision.choice =
        sugarscape_core::active_surface::Choice::StopProbing;
    assert!(!report_integrity(&report).unwrap());
    std::fs::remove_file(path).unwrap();
}
