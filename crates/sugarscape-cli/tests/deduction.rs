use serde_json::Value;
use std::process::{Command, Output, Stdio};
fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_sugarscape"))
        .args(args)
        .stdin(Stdio::null())
        .output()
        .unwrap()
}
#[test]
fn run_is_deterministic() {
    let args = [
        "deduction",
        "run",
        "--scenario",
        "wink",
        "--seed",
        "7",
        "--policy",
        "evidence",
    ];
    let a = run(&args);
    let b = run(&args);
    assert!(a.status.success());
    assert_eq!(a.stdout, b.stdout);
    let _: Value = serde_json::from_slice(&a.stdout).unwrap();
}
#[test]
fn play_exposes_only_selected_request_and_pauses() {
    let out = run(&["deduction", "play", "--seed", "7", "--agent", "0"]);
    assert!(out.status.success());
    let v: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(v["actor"], 0);
    assert!(v.get("seed").is_none());
    assert!(v.get("fingerprint").is_none());
    assert!(v.get("archive").is_none());
}
#[test]
fn diagnose_has_frozen_metadata() {
    let out = run(&["deduction", "diagnose"]);
    assert!(out.status.success());
    let v: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert!(v.get("frozen_config").is_some());
}
fn scratch(name: &str) -> std::path::PathBuf {
    let p = std::env::temp_dir().join(format!("deduction-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&p);
    std::fs::create_dir_all(&p).unwrap();
    p
}
fn interact(args: &[&str], replies: usize, invalid: bool) -> (Vec<Value>, Output) {
    use std::io::{BufRead, BufReader, Write};
    let mut child = Command::new(env!("CARGO_BIN_EXE_sugarscape"))
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let mut input = child.stdin.take().unwrap();
    let mut output = BufReader::new(child.stdout.take().unwrap());
    let mut requests = Vec::new();
    let mut stale = None;
    for index in 0..=replies {
        let mut line = String::new();
        if output.read_line(&mut line).unwrap() == 0 {
            break;
        }
        let request: Value = serde_json::from_str(&line).unwrap();
        requests.push(request.clone());
        if request.get("status").is_some() {
            break;
        }
        if index == replies {
            break;
        }
        if invalid && index == 1 {
            let extras = serde_json::json!({
                "request_id": request["request_id"], "actor": 0,
                "action": {"kind": "pass", "secret": 1}
            });
            let wrong_actor = serde_json::json!({
                "request_id": request["request_id"], "actor": 1,
                "action": {"kind": "pass"}
            });
            for bad in [
                "malformed".to_string(),
                stale.clone().unwrap(),
                extras.to_string(),
                wrong_actor.to_string(),
            ] {
                writeln!(input, "{bad}").unwrap();
                let mut retry = String::new();
                output.read_line(&mut retry).unwrap();
                assert_eq!(serde_json::from_str::<Value>(&retry).unwrap(), request);
            }
        }
        let response = serde_json::json!({
            "request_id":request["request_id"], "actor":request["actor"],
            "action":{"kind":"pass"}
        })
        .to_string();
        writeln!(input, "{response}").unwrap();
        stale = Some(response);
    }
    drop(input);
    let mut remaining = String::new();
    std::io::Read::read_to_string(&mut output, &mut remaining).unwrap();
    requests.extend(remaining.lines().map(|s| serde_json::from_str(s).unwrap()));
    (requests, child.wait_with_output().unwrap())
}
#[test]
fn split_resume_matches_continuous_future_policy_rng() {
    let directory = scratch("split");
    let p = directory.join("session.json");
    let final_path = directory.join("final.json");
    let continuous_path = directory.join("continuous.json");
    let path = p.to_str().unwrap();
    let base = [
        "deduction",
        "play",
        "--seed",
        "17",
        "--agent",
        "0",
        "--policy",
        "random",
        "--archive",
        continuous_path.to_str().unwrap(),
    ];
    let (continuous, out) = interact(&base, 20, false);
    assert!(out.status.success());
    assert!(
        continuous.len() > 4,
        "must compare multiple future requests"
    );
    let (_, out) = interact(
        &[
            "deduction",
            "play",
            "--seed",
            "17",
            "--agent",
            "0",
            "--policy",
            "random",
            "--archive",
            path,
        ],
        2,
        false,
    );
    assert!(out.status.success());
    let (resumed, out) = interact(
        &[
            "deduction",
            "play",
            "--resume",
            path,
            "--agent",
            "0",
            "--archive",
            final_path.to_str().unwrap(),
        ],
        18,
        false,
    );
    assert!(out.status.success());
    assert_eq!(resumed, continuous[2..]);
    assert_eq!(
        std::fs::read(final_path).unwrap(),
        std::fs::read(continuous_path).unwrap()
    );
}
#[test]
fn invalid_replies_retry_and_archive_keeps_pending_request() {
    let p = scratch("retry").join("session.json");
    let path = p.to_str().unwrap();
    let (requests, out) = interact(
        &[
            "deduction",
            "play",
            "--seed",
            "7",
            "--agent",
            "0",
            "--archive",
            path,
        ],
        2,
        true,
    );
    assert!(out.status.success());
    assert!(String::from_utf8(out.stderr)
        .unwrap()
        .contains("invalid response"));
    let resumed = run(&["deduction", "play", "--resume", path, "--agent", "0"]);
    assert!(resumed.status.success());
    assert_eq!(
        serde_json::from_slice::<Value>(&resumed.stdout).unwrap(),
        *requests.last().unwrap()
    );
    for flags in [
        ["--seed", "8"],
        ["--agent", "1"],
        ["--policy", "passive"],
        ["--scenario", "other"],
    ] {
        let out = run(&["deduction", "play", "--resume", path, flags[0], flags[1]]);
        assert_eq!(out.status.code(), Some(2));
        assert!(out.stdout.is_empty());
    }
    let mut v: Value = serde_json::from_str(&std::fs::read_to_string(&p).unwrap()).unwrap();
    v["unexpected"] = Value::Bool(true);
    std::fs::write(&p, v.to_string()).unwrap();
    assert_eq!(
        run(&["deduction", "play", "--resume", path]).status.code(),
        Some(2)
    );
}
#[test]
fn unavailable_seat_and_io_failures_are_explicit() {
    let out = run(&["deduction", "play", "--agent", "99"]);
    assert_eq!(out.status.code(), Some(2));
    assert!(out.stdout.is_empty());
    let p = scratch("io");
    let out = run(&[
        "deduction",
        "play",
        "--agent",
        "0",
        "--archive",
        p.to_str().unwrap(),
    ]);
    assert_eq!(out.status.code(), Some(1));
}

#[test]
fn replay_and_corrupted_archives_never_emit_private_state() {
    let file = scratch("tamper").join("session.json");
    let path = file.to_str().unwrap();
    assert!(
        run(&["deduction", "play", "--agent", "0", "--archive", path])
            .status
            .success()
    );
    let original: Value = serde_json::from_str(&std::fs::read_to_string(&file).unwrap()).unwrap();
    let output = run(&["deduction", "replay", path]);
    assert!(output.status.success());
    let result: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(result["fingerprint"], original["archive"]["fingerprint"]);
    for (field, bad) in [
        ("host_version", serde_json::json!(2)),
        ("selected_agent", serde_json::json!(65536)),
        ("policy_seed_derivation", serde_json::json!("unknown")),
    ] {
        let mut changed = original.clone();
        changed[field] = bad;
        std::fs::write(&file, changed.to_string()).unwrap();
        let output = run(&["deduction", "play", "--resume", path]);
        assert_eq!(output.status.code(), Some(2));
        assert!(output.stdout.is_empty());
        assert!(!String::from_utf8(output.stderr)
            .unwrap()
            .contains("fingerprint"));
    }
    let mut changed = original;
    changed["archive"]["fingerprint"] = serde_json::json!(0);
    std::fs::write(&file, changed.to_string()).unwrap();
    assert_eq!(run(&["deduction", "replay", path]).status.code(), Some(2));
}

#[test]
fn inactive_selected_agent_gets_a_public_pause() {
    use std::io::{BufRead, BufReader, Write};
    use sugarscape_core::deduction::{wink_config, Action, Engine, TurnResponse};
    // Choose a seed where Agents 0 and 1 are civilians using host-only fixture queries.
    let seed = (0..100)
        .find(|seed| {
            let mut engine = Engine::new(wink_config(6), *seed).unwrap();
            for _ in 0..6 {
                let request = engine.request().unwrap();
                if request.actor <= 1 && !request.observation.grants.is_empty() {
                    return false;
                }
                engine
                    .submit(TurnResponse {
                        request_id: request.request_id,
                        actor: request.actor,
                        action: Action::Pass,
                    })
                    .unwrap();
            }
            true
        })
        .unwrap()
        .to_string();
    let mut child = Command::new(env!("CARGO_BIN_EXE_sugarscape"))
        .args([
            "deduction",
            "play",
            "--seed",
            &seed,
            "--agent",
            "0",
            "--policy",
            "passive",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let mut input = child.stdin.take().unwrap();
    let mut output = BufReader::new(child.stdout.take().unwrap());
    for index in 0..3 {
        let mut line = String::new();
        output.read_line(&mut line).unwrap();
        let request: Value = serde_json::from_str(&line).unwrap();
        assert_eq!(request["actor"], 0);
        let action = if index == 2 {
            serde_json::json!({"kind":"accuse","target":1})
        } else {
            serde_json::json!({"kind":"pass"})
        };
        writeln!(
            input,
            "{}",
            serde_json::json!({"actor":0,"request_id":request["request_id"],"action":action})
        )
        .unwrap();
    }
    let mut line = String::new();
    output.read_line(&mut line).unwrap();
    let pause: Value = serde_json::from_str(&line).unwrap();
    assert_eq!(
        pause,
        serde_json::json!({"status":"paused","reason":"selected_agent_inactive","actor":0})
    );
    drop(input);
    assert!(child.wait().unwrap().success());
}
