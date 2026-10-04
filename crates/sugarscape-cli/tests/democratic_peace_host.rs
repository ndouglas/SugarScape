//! Actual CLI discovery, finite exports, and grouped source clocks.
use std::process::Command;
#[test]
fn democratic_peace_cli_discovers_presets_and_exports_grouped_outcome() {
    let binary = env!("CARGO_BIN_EXE_sugarscape");
    let catalog = Command::new(binary).arg("presets").output().unwrap();
    assert!(catalog.status.success());
    assert!(String::from_utf8(catalog.stdout)
        .unwrap()
        .contains("democratic-peace-printed-2001"));
    let dir = std::env::temp_dir().join(format!("democratic-peace-cli-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let config = dir.join("config.json");
    std::fs::write(&config, r#"{"model":"democratic_peace","width":4,"height":4,"horizon_periods":14,"periods_per_tick":3,"probability_direction":"prose_increasing","zero_ratio":"equal_zero_neutral"}"#).unwrap();
    let output = Command::new(binary)
        .args(["run", "--config"])
        .arg(&config)
        .args(["--seed", "312312", "--ticks", "100", "--outcome-out"])
        .arg(dir.join("outcome.json"))
        .arg("--state-out")
        .arg(dir.join("state.json"))
        .arg("--config-out")
        .arg(dir.join("resolved.json"))
        .arg("--series-csv")
        .arg(dir.join("series.csv"))
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stderr).contains("14 completed periods"));
    let state: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(dir.join("state.json")).unwrap()).unwrap();
    assert!(state.is_object());
    let outcome: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(dir.join("outcome.json")).unwrap()).unwrap();
    assert_eq!(outcome["valid"], true);
    assert_eq!(outcome["completed_periods"], 14);
    let resolved: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(dir.join("resolved.json")).unwrap()).unwrap();
    assert_eq!(resolved["probability_direction"], "prose_increasing");
    assert_eq!(resolved["zero_ratio"], "equal_zero_neutral");
    let csv = std::fs::read_to_string(dir.join("series.csv")).unwrap();
    assert!(csv.contains("periods"));
    std::fs::remove_dir_all(dir).unwrap();
}
