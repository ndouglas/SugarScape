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
