use std::collections::BTreeSet;
use std::process::Command;

#[test]
fn strategy_inference_rejects_tuning_arguments() {
    for extra in [
        vec!["--seed", "7"],
        vec!["--generations", "1"],
        vec!["--out", "ignored.json"],
        vec!["--q", "0.8"],
        vec!["unexpected"],
    ] {
        let out = Command::new(env!("CARGO_BIN_EXE_sugarscape"))
            .args(["deduction", "strategy-inference"])
            .args(extra)
            .output()
            .unwrap();
        assert_eq!(out.status.code(), Some(2));
        assert!(out.stdout.is_empty());
    }
}

// Run only after independent precollection review and exclusive retention of the first external report.
#[test]
fn strategy_inference_report_is_complete_and_repeatable() {
    let run = || {
        Command::new(env!("CARGO_BIN_EXE_sugarscape"))
            .args(["deduction", "strategy-inference"])
            .output()
            .unwrap()
    };
    let a = run();
    let b = run();
    assert!(a.status.success(), "{}", String::from_utf8_lossy(&a.stderr));
    assert!(b.status.success(), "{}", String::from_utf8_lossy(&b.stderr));
    assert_eq!(a.stdout, b.stdout);
    let report: sugarscape_core::deduction::strategy_inference::DiagnosticReport =
        serde_json::from_slice(&a.stdout).unwrap();
    assert!(sugarscape_core::deduction::strategy_inference::report_integrity(&report).unwrap());
    let wire: serde_json::Value = serde_json::from_slice(&a.stdout).unwrap();
    assert_eq!(wire["version"], "strategy-inference-diagnostic-v1");
    assert_eq!(wire["passed"], true);
    assert_eq!(wire["metadata"]["optimization"], false);
    let mut expected_fixed = BTreeSet::new();
    let mut expected_mixture = BTreeSet::new();
    let mut expected_models = BTreeSet::new();
    for env in ["q4_5", "q3_5"] {
        for catalog in ["uniform", "optimization_informed"] {
            expected_models.insert(format!("{env}/{catalog}/strategy_{catalog}"));
            for listener in [
                format!("strategy_{catalog}"),
                "bayesian".into(),
                "credulous".into(),
                "skeptical".into(),
                "evolved".into(),
                "passive".into(),
            ] {
                expected_mixture.insert(format!("{env}/{catalog}/{listener}"));
            }
        }
        for bits in [163882, 5441, 246723, 0, 81942, 98342] {
            for listener in [
                "strategy_uniform",
                "strategy_optimization_informed",
                "bayesian",
                "credulous",
                "skeptical",
                "evolved",
                "passive",
            ] {
                expected_fixed.insert(format!("{env}/{bits}/{listener}"));
            }
        }
    }
    let text = |row: &serde_json::Value, key: &str| row[key].as_str().unwrap().to_owned();
    let fixed: BTreeSet<_> = wire["fixed_evaluations"]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| {
            format!(
                "{}/{}/{}",
                text(row, "environment_id"),
                row["canonical_bits"],
                text(&row["listener"], "id")
            )
        })
        .collect();
    let mixture: BTreeSet<_> = wire["mixture_evaluations"]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| {
            format!(
                "{}/{}/{}",
                text(row, "environment_id"),
                text(row, "catalog_id"),
                text(&row["listener"], "id")
            )
        })
        .collect();
    let models: BTreeSet<_> = wire["inference_models"]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| {
            format!(
                "{}/{}/{}",
                text(row, "environment_id"),
                text(row, "catalog_id"),
                text(&row["listener"], "id")
            )
        })
        .collect();
    assert_eq!(fixed, expected_fixed);
    assert_eq!(mixture, expected_mixture);
    assert_eq!(models, expected_models);
    assert_eq!(wire["fixed_evaluations"].as_array().unwrap().len(), 84);
    assert_eq!(wire["mixture_evaluations"].as_array().unwrap().len(), 24);
    for row in wire["inference_models"].as_array().unwrap() {
        assert_eq!(row["calibrations"].as_array().unwrap().len(), 8);
        assert_eq!(row["histories"].as_array().unwrap().len(), 32);
    }
    assert_eq!(
        wire["policy_provenance"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|row| row["method"] == "genetic" || row["method"] == "random")
            .count(),
        40
    );
}
