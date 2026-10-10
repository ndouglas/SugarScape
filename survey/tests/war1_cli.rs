#![cfg(feature = "war-benchmarks")]

use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{
    fs,
    path::PathBuf,
    process::Command,
    sync::atomic::{AtomicU64, Ordering},
};

const GRAPH: &str = r#"{"mode":"reciprocal_graph","config":{"blue":2,"red":2,"blue_rate":1.0,"red_rate":1.0,"dt":0.05,"max_steps":8,"geometry":"aimed_fire"},"seed":7,"capture_steps":[0,1,8]}"#;
struct Case(PathBuf);
impl Case {
    fn new() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let root = std::env::temp_dir().canonicalize().unwrap().join(format!(
            "war1-cli-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&root).unwrap();
        Self(root)
    }
    fn run(&self, input: &str) -> std::process::Output {
        fs::write(self.0.join("input.json"), input).unwrap();
        Command::new(env!("CARGO_BIN_EXE_war1"))
            .args(["run", "--input"])
            .arg(self.0.join("input.json"))
            .arg("--out")
            .arg(self.0.join("out"))
            .output()
            .unwrap()
    }
}
impl Drop for Case {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}

#[test]
fn help_explains_native_engineering_commands() {
    let out = Command::new(env!("CARGO_BIN_EXE_war1"))
        .arg("--help")
        .output()
        .unwrap();
    assert!(out.status.success());
    let text = String::from_utf8(out.stdout).unwrap();
    assert!(text.contains("validate --input FILE"));
    assert!(text.contains("run --input FILE --out DIRECTORY"));
    assert!(text.contains("engineering"));
}

#[test]
fn invalid_inputs_fail_before_output_creation() {
    // Removing typed validation, recursive duplicate detection, literal checks or bounds must fail this table.
    let bad = [
        GRAPH.replace("\"seed\":7", "\"seed\":7,\"seed\":8"),
        GRAPH.replace("\"blue\":2", "\"blue\":2,\"blue\":3"),
        GRAPH.replace("\"blue\":2", "\"unknown\":2"),
        GRAPH.replace("\"seed\":7", "\"seed\":7.0"),
        GRAPH.replace("\"seed\":7", "\"seed\":-1"),
        GRAPH.replace("\"seed\":7", "\"seed\":18446744073709551616"),
        format!("{GRAPH} {{}}"),
        GRAPH.replace("[0,1,8]", "[9]"),
        GRAPH.replace("[0,1,8]", "[1,1]"),
        GRAPH.replace("\"blue_rate\":1.0", "\"blue_rate\":1e-400"),
        GRAPH.replace("\"blue_rate\":1.0", "\"blue_rate\":1e400"),
        GRAPH.replace("\"blue\":2", "\"blue\":4097"),
        GRAPH.replace("\"max_steps\":8", "\"max_steps\":0"),
        r#"{"mode":"book_c","preset":"missing","seed":8,"max_steps":4,"capture_steps":[]}"#.into(),
        " ".repeat(1024 * 1024 + 1),
    ];
    for input in bad {
        let case = Case::new();
        let out = case.run(&input);
        assert_eq!(
            (out.status.code(), case.0.join("out").exists()),
            (Some(2), false),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
    }
}

#[test]
fn validate_preserves_exact_u64_seed_without_output() {
    let case = Case::new();
    fs::write(
        case.0.join("input.json"),
        GRAPH.replace("\"seed\":7", "\"seed\":18446744073709551615"),
    )
    .unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_war1"))
        .args(["validate", "--input"])
        .arg(case.0.join("input.json"))
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let resolved: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(resolved["input"]["data"]["seed"].as_u64(), Some(u64::MAX));
    assert!(!case.0.join("out").exists());
}

#[test]
fn successful_graph_binds_exact_saved_bytes_and_clocks() {
    let case = Case::new();
    let out = case.run(GRAPH);
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let dir = case.0.join("out");
    assert_eq!(fs::read(dir.join("input.json")).unwrap(), GRAPH.as_bytes());
    let metadata: Value =
        serde_json::from_slice(&fs::read(dir.join("metadata.json")).unwrap()).unwrap();
    assert_eq!(metadata["purpose"], "engineering_case");
    assert_eq!(metadata["registered"], false);
    assert_eq!(
        metadata["input_sha256"],
        format!("{:x}", Sha256::digest(GRAPH.as_bytes()))
    );
    assert_eq!(
        metadata["binary_sha256"],
        format!(
            "{:x}",
            Sha256::digest(fs::read(env!("CARGO_BIN_EXE_war1")).unwrap())
        )
    );
    let frames = fs::read(dir.join("frames.jsonl")).unwrap();
    let rows: Vec<Value> = frames
        .split(|b| *b == b'\n')
        .filter(|l| !l.is_empty())
        .map(|l| serde_json::from_slice(l).unwrap())
        .collect();
    assert_eq!(rows[0]["payload"]["kind"], "header");
    assert_eq!(metadata["input_identity"], rows[0]["input_identity"]);
    assert_eq!(rows.last().unwrap()["payload"]["kind"], "terminal");
    let receipt: Value =
        serde_json::from_slice(&fs::read(dir.join("success.json")).unwrap()).unwrap();
    assert_eq!(
        receipt["journal"]["sha256"],
        format!("{:x}", Sha256::digest(&frames))
    );
    assert_eq!(receipt["journal"]["bytes"], frames.len() as u64);
    assert_eq!(
        receipt["journal"]["acknowledged_steps"],
        receipt["summary"]["completed_steps"]
    );
}

#[test]
fn reused_directory_is_refused_without_changing_bytes() {
    let case = Case::new();
    fs::create_dir(case.0.join("out")).unwrap();
    fs::write(case.0.join("out/keep"), b"keep").unwrap();
    assert_eq!(case.run(GRAPH).status.code(), Some(1));
    assert_eq!(fs::read(case.0.join("out/keep")).unwrap(), b"keep");
    assert_eq!(fs::read_dir(case.0.join("out")).unwrap().count(), 1);
}

#[test]
fn book_uses_resolved_preset_and_literal_ticks() {
    let case = Case::new();
    let out = case.run(
        r#"{"mode":"book_c","preset":"iii-9-combat","seed":8,"max_steps":4,"capture_steps":[4]}"#,
    );
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let frames = fs::read_to_string(case.0.join("out/frames.jsonl")).unwrap();
    let rows: Vec<Value> = frames
        .lines()
        .map(|l| serde_json::from_str(l).unwrap())
        .collect();
    assert_eq!(
        rows[0]["payload"]["data"]["header"]["input"]["data"]["config"]["combat"]["enabled"],
        true
    );
    assert_eq!(
        rows.last().unwrap()["payload"]["data"]["summary"]["completed_steps"],
        4
    );
}

#[test]
fn numeric_failure_preserves_durable_prefix_without_success() {
    let case = Case::new();
    let out = case.run(&GRAPH.replace("\"blue_rate\":1.0", "\"blue_rate\":1e-300"));
    assert_eq!(out.status.code(), Some(1));
    let dir = case.0.join("out");
    assert!(!dir.join("success.json").exists());
    let failure: Value =
        serde_json::from_slice(&fs::read(dir.join("failure.json")).unwrap()).unwrap();
    assert_eq!(failure["completed_steps"], 0);
    assert_eq!(failure["emitted_steps"], 0);
    assert_eq!(failure["journal"]["acknowledged_steps"], 0);
    assert_eq!(
        failure["journal"]["sha256"],
        format!(
            "{:x}",
            Sha256::digest(fs::read(dir.join("frames.jsonl")).unwrap())
        )
    );
}

#[cfg(unix)]
#[test]
fn symlink_input_and_output_parent_are_rejected() {
    use std::os::unix::fs::symlink;
    let case = Case::new();
    fs::write(case.0.join("real.json"), GRAPH).unwrap();
    symlink(case.0.join("real.json"), case.0.join("input.json")).unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_war1"))
        .args(["run", "--input"])
        .arg(case.0.join("input.json"))
        .arg("--out")
        .arg(case.0.join("out"))
        .output()
        .unwrap();
    assert_eq!(
        (out.status.code(), case.0.join("out").exists()),
        (Some(2), false)
    );
    fs::remove_file(case.0.join("input.json")).unwrap();
    fs::write(case.0.join("input.json"), GRAPH).unwrap();
    fs::create_dir(case.0.join("real")).unwrap();
    symlink(case.0.join("real"), case.0.join("alias")).unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_war1"))
        .args(["run", "--input"])
        .arg(case.0.join("input.json"))
        .arg("--out")
        .arg(case.0.join("alias/out"))
        .output()
        .unwrap();
    assert_eq!(
        (out.status.code(), case.0.join("real/out").exists()),
        (Some(1), false)
    );
}

#[test]
fn extra_or_reordered_arguments_are_usage_errors() {
    for args in [
        vec!["run"],
        vec!["help", "extra"],
        vec!["validate", "--input", "x", "extra"],
        vec!["run", "--out", "x", "--input", "y"],
    ] {
        assert_eq!(
            Command::new(env!("CARGO_BIN_EXE_war1"))
                .args(args)
                .output()
                .unwrap()
                .status
                .code(),
            Some(2)
        );
    }
}

#[cfg(unix)]
#[test]
fn fifo_input_is_rejected_without_waiting_for_a_writer() {
    let case = Case::new();
    let input = case.0.join("fifo");
    assert!(Command::new("mkfifo")
        .arg(&input)
        .status()
        .unwrap()
        .success());
    let mut child = Command::new(env!("CARGO_BIN_EXE_war1"))
        .args(["validate", "--input"])
        .arg(&input)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .unwrap();
    let mut status = None;
    for _ in 0..50 {
        status = child.try_wait().unwrap();
        if status.is_some() {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    if status.is_none() {
        child.kill().unwrap();
        child.wait().unwrap();
    }
    assert_eq!(
        status.and_then(|s| s.code()),
        Some(2),
        "FIFO must be rejected before blocking open"
    );
}

// Catches treating literal zero as an underflowed nonzero JSON token.
#[test]
fn literal_zero_rates_are_accepted_and_end_without_a_settlement() {
    let case = Case::new();
    let input = GRAPH
        .replace("\"blue_rate\":1.0", "\"blue_rate\":0")
        .replace("\"red_rate\":1.0", "\"red_rate\":0.0");
    let out = case.run(&input);
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let receipt: Value =
        serde_json::from_slice(&fs::read(case.0.join("out/success.json")).unwrap()).unwrap();
    assert_eq!(receipt["summary"]["completed_steps"], 0);
    assert_eq!(receipt["summary"]["ending"]["reason"], "rate_zero");
}
