use std::process::Command;
#[test]
fn testimony_command_is_exact_and_deterministic() {
    let run = || {
        Command::new(env!("CARGO_BIN_EXE_sugarscape"))
            .args(["deduction", "testimony"])
            .output()
            .unwrap()
    };
    let a = run();
    let b = run();
    assert!(a.status.success());
    assert_eq!(a.stdout, b.stdout);
    let report: serde_json::Value = serde_json::from_slice(&a.stdout).unwrap();
    assert_eq!(report["version"], "testimony-v1");
    assert_eq!(report["passed"], true);
}
#[test]
fn testimony_rejects_unknown_flags() {
    let out = Command::new(env!("CARGO_BIN_EXE_sugarscape"))
        .args(["deduction", "testimony", "--seed", "7"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2));
}
