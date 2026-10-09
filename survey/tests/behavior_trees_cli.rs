use std::process::Command;
#[test]
fn behavior_tree_missing_route_red() {
    let out = Command::new(env!("CARGO_BIN_EXE_survey"))
        .args(["--behavior-trees", "--help"])
        .output()
        .unwrap();
    assert!(out.status.success());
    assert!(
        String::from_utf8_lossy(&out.stdout).contains("survey --behavior-trees"),
        "missing pure route"
    );
}
#[test]
fn behavior_tree_fixed_manifest_and_rejected_flags_are_pure() {
    let dir = std::env::temp_dir()
        .canonicalize()
        .unwrap()
        .join(format!("bt-cli-{}", std::process::id()));
    std::fs::create_dir(&dir).unwrap();
    let run = |args: &[&str]| {
        Command::new(env!("CARGO_BIN_EXE_survey"))
            .current_dir(&dir)
            .args(args)
            .output()
            .unwrap()
    };
    let out = run(&["--behavior-trees", "--manifest"]);
    assert!(out.status.success());
    let m: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(m["conditions"].as_array().unwrap().len(), 96);
    assert_eq!(m["seeds"].as_array().unwrap().len(), 40);
    assert_eq!(m["schema"], "minds-behavior-tree-manifest-v1");
    assert_eq!(out.stdout, run(&["--behavior-trees"]).stdout);
    for flag in [
        "--behavior-trees",
        "--foraging-shortcuts",
        "--burrow",
        "--protection",
        "--minds9",
        "--only",
        "--seeds",
        "--threads",
        "--fuel",
        "--retry",
        "--resume",
        "--claim",
    ] {
        let out = run(&[flag, "--behavior-trees", "--help"]);
        assert_eq!(out.status.code(), Some(2), "accepted {flag}");
    }
    for args in [
        vec![
            "--behavior-trees",
            "--run",
            "--protocol-revision",
            "short",
            "--out",
            "new",
        ],
        vec![
            "--behavior-trees",
            "--analyze",
            "missing-index.json",
            "--out",
            "new",
        ],
        vec!["--behavior-trees", "--manifest", "--manifest"],
    ] {
        assert_eq!(run(&args).status.code(), Some(2));
    }
    assert_eq!(std::fs::read_dir(&dir).unwrap().count(), 0);
    std::fs::remove_dir(dir).unwrap();
}
