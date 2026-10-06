use std::process::Command;
#[test]
fn strategic_reporting_rejects_tuning_arguments() {
    for extra in [
        vec!["--seed", "7"],
        vec!["--generations", "1"],
        vec!["--out", "ignored.json"],
        vec!["unexpected"],
    ] {
        let out = Command::new(env!("CARGO_BIN_EXE_sugarscape"))
            .args(["deduction", "strategic-reporting"])
            .args(extra)
            .output()
            .unwrap();
        assert_eq!(out.status.code(), Some(2));
        assert!(out.stdout.is_empty());
    }
}
#[test]
fn strategic_reporting_report_is_complete_and_repeatable() {
    let run = || {
        Command::new(env!("CARGO_BIN_EXE_sugarscape"))
            .args(["deduction", "strategic-reporting"])
            .output()
            .unwrap()
    };
    let a = run();
    let b = run();
    assert!(a.status.success(), "{}", String::from_utf8_lossy(&a.stderr));
    assert_eq!(a.stdout, b.stdout);
    let r: serde_json::Value = serde_json::from_slice(&a.stdout).unwrap();
    assert_eq!(r["passed"], true);
    assert_eq!(r["version"], "strategic-reporting-diagnostic-v1");
    assert_eq!(r["runs"].as_array().unwrap().len(), 40);
    assert_eq!(r["frozen_evaluations"].as_array().unwrap().len(), 240);
    assert_eq!(r["metadata"]["search_seeds"]["end_exclusive"], 20);
    assert_eq!(r["environments"][0]["optimum"]["policy"]["bits"], 81942);
    for method in ["genetic", "random"] {
        let runs: Vec<_> = r["runs"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|x| x["method"] == method)
            .collect();
        assert_eq!(
            runs.iter()
                .map(|x| x["seed"].as_u64().unwrap())
                .collect::<Vec<_>>(),
            (0..20).collect::<Vec<_>>()
        );
        for run in runs {
            assert_eq!(run["evaluations"], 3164);
            assert_eq!(run["curve"].as_array().unwrap().len(), 3164);
            assert_eq!(run["seed_derivation"], "strategic-reporting-search-seed-v1");
        }
    }
}
