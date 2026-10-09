use serde_json::{json, Value};
use std::{
    process::{Command, Output},
    sync::atomic::{AtomicUsize, Ordering},
};
use sugarscape_core::browser_experiments as core;

fn invoke(op: &str, text: Option<&str>) -> Output {
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    let path = std::env::temp_dir().join(format!(
        "experiment-view-{}-{}.json",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    let mut command = Command::new(env!("CARGO_BIN_EXE_sugarscape"));
    command.args(["experiment-view", op]);
    if let Some(text) = text {
        std::fs::write(&path, text).unwrap();
        command.arg("--input").arg(&path);
    }
    let output = command.output().unwrap();
    if text.is_some() {
        std::fs::remove_file(path).unwrap();
    }
    output
}
fn successful(output: Output) -> Value {
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stdout.ends_with(b"\n"));
    serde_json::from_slice(&output.stdout).unwrap()
}
fn rejected(op: &str, text: &str, context: &str) {
    let output = invoke(op, Some(text));
    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains(context),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}
#[test]
fn catalog_and_recorded_are_complete_core_exports() {
    assert_eq!(
        successful(invoke("catalog", None)),
        serde_json::to_value(core::catalog()).unwrap()
    );
    assert_eq!(
        successful(invoke("recorded", None)),
        serde_json::from_str::<Value>(&core::recorded_results_json().unwrap()).unwrap()
    );
}
#[test]
fn run_and_import_reconstruct_complete_record_with_max_seed() {
    let input =
        r#"{"study":"wink","seed":"18446744073709551615","policy":"passive","mode":"ordinary"}"#;
    let expected = core::run(&core::normalize_input(input).unwrap()).unwrap();
    let actual = successful(invoke("run", Some(input)));
    assert_eq!(actual, serde_json::to_value(expected).unwrap());
    assert_eq!(
        successful(invoke("validate", Some(&actual.to_string()))),
        actual
    );
}
#[test]
fn malformed_overflow_unknown_fields_and_raw_whitespace_are_rejected() {
    rejected("run", "{", "input");
    rejected(
        "run",
        r#"{"study":"wink","seed":"18446744073709551616","policy":"passive","mode":"ordinary"}"#,
        "input",
    );
    rejected(
        "run",
        r#"{"study":"testimony","fixture":"transfer","extra":true}"#,
        "input",
    );
    rejected("run", &" ".repeat(core::MAX_INPUT_BYTES + 1), "input");
    rejected(
        "validate",
        &" ".repeat(core::MAX_EPISODE_BYTES + 1),
        "episode",
    );
}
#[test]
fn imported_flags_versions_zero_denominators_and_checkpoint_overflow_are_rejected() {
    let input =
        r#"{"study":"testimony_game","environment":"training","history":31,"listener":"bayesian"}"#;
    let base =
        serde_json::to_value(core::run(&core::normalize_input(input).unwrap()).unwrap()).unwrap();
    let mut forged = base.clone();
    forged["passed"] = json!(true);
    rejected("validate", &forged.to_string(), "episode");
    let mut forged = base.clone();
    forged["version"] = json!(2);
    rejected("validate", &forged.to_string(), "version");
    let mut forged = base.clone();
    forged["payload"]["reference_posterior"]["denominator"] = json!("0");
    rejected("validate", &forged.to_string(), "episode");
    let mut forged = base.clone();
    forged["checkpoints"] = json!(vec![base["checkpoints"][0].clone(); 4097]);
    rejected("validate", &forged.to_string(), "checkpoints");
}
#[test]
fn missing_file_remains_operational_exit_one() {
    let output = Command::new(env!("CARGO_BIN_EXE_sugarscape"))
        .args([
            "experiment-view",
            "run",
            "--input",
            "/no/such/experiment-view-input.json",
        ])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
}
#[cfg(unix)]
#[test]
fn closed_output_pipe_reports_operational_failure() {
    use std::process::Stdio;
    let mut child = Command::new(env!("CARGO_BIN_EXE_sugarscape"))
        .args(["experiment-view", "recorded"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    drop(child.stdout.take());
    assert_eq!(child.wait_with_output().unwrap().status.code(), Some(1));
}

#[path = "../../test-support/spatial_experiments.rs"]
mod spatial_test_support;
use spatial_test_support::{
    checked_snapshot_bytes, default_seed_cases, invalid_spatial_inputs, spatial_defaults,
    spatial_saved_mutations, spatial_snapshot_inputs,
};

#[test]
fn thirteen_catalog_entries_and_five_complete_spatial_defaults() {
    assert_eq!(core::catalog().len(), 13);
    assert_eq!(spatial_defaults().len(), 5);
    for input in default_seed_cases() {
        let text = input.to_string();
        let expected =
            serde_json::to_value(core::run(&core::normalize_input(&text).unwrap()).unwrap())
                .unwrap();
        let actual = successful(invoke("run", Some(&text)));
        assert_eq!(actual, expected);
        assert_eq!(
            successful(invoke("validate", Some(&actual.to_string()))),
            actual
        );
        assert!(actual["checkpoints"].as_array().unwrap().len() <= core::MAX_CHECKPOINTS);
        assert!(actual.to_string().len() <= core::MAX_EPISODE_BYTES);
        if let Some((reported, bytes)) = checked_snapshot_bytes(&text) {
            assert_eq!(reported, bytes);
            assert_eq!(
                actual["payload"]["native"]["snapshot_bytes"],
                json!(bytes.to_string())
            );
        }
    }
}

#[test]
fn spatial_boundary_rejects_missing_fields_large_ids_and_resource_products() {
    for input in spatial_defaults() {
        for (bad, field) in invalid_spatial_inputs(&input) {
            rejected("run", &bad.to_string(), field);
        }
    }
}

#[test]
fn spatial_saved_records_require_complete_same_target_reconstruction() {
    for input in spatial_defaults() {
        let text = input.to_string();
        let actual = successful(invoke("run", Some(&text)));
        for (bad, field) in spatial_saved_mutations(&input, &actual) {
            rejected("validate", &bad.to_string(), field);
        }
    }
}

#[test]
fn spatial_scene_and_bounded_extreme_snapshot_bytes_are_recomputed_on_this_target() {
    for input in spatial_snapshot_inputs() {
        let text = input.to_string();
        let (reported, bytes) = checked_snapshot_bytes(&text).unwrap();
        assert_eq!(reported, bytes);
        let actual = successful(invoke("run", Some(&text)));
        assert_eq!(
            actual["payload"]["native"]["snapshot_bytes"],
            json!(bytes.to_string())
        );
    }
}
