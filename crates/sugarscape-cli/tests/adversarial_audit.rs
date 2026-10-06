use std::collections::BTreeSet;
use std::process::Command;
use sugarscape_core::deduction::adversarial_audit::{self, DiagnosticReport};

#[test]
fn adversarial_audit_rejects_all_tuning_arguments() {
    for extra in [
        vec!["--seed", "7"],
        vec!["--generation", "1"],
        vec!["--generations", "1"],
        vec!["--prior", "uniform"],
        vec!["--q", "0.8"],
        vec!["--out", "ignored.json"],
        vec!["--output-path", "ignored.json"],
        vec!["unexpected"],
    ] {
        let out = Command::new(env!("CARGO_BIN_EXE_sugarscape"))
            .args(["deduction", "adversarial-audit"])
            .args(extra)
            .output()
            .unwrap();
        assert_eq!(out.status.code(), Some(2));
        assert!(out.stdout.is_empty());
        assert!(
            !String::from_utf8_lossy(&out.stderr).contains("unrecognized subcommand"),
            "mode must exist before tuning rejection is meaningful"
        );
    }
}

// Collection gate: run this test only after the root retains the first external report.
#[test]
fn adversarial_audit_report_is_complete_and_repeatable() {
    let run = || {
        Command::new(env!("CARGO_BIN_EXE_sugarscape"))
            .args(["deduction", "adversarial-audit"])
            .output()
            .unwrap()
    };
    let first = run();
    let repeat = run();
    assert!(
        first.status.success(),
        "{}",
        String::from_utf8_lossy(&first.stderr)
    );
    assert!(repeat.status.success());
    assert_eq!(first.stdout, repeat.stdout);
    let r: DiagnosticReport = serde_json::from_slice(&first.stdout).unwrap();
    assert!(r.passed && adversarial_audit::report_integrity(&r).unwrap());
    assert_eq!(r.version, adversarial_audit::REPORT_VERSION);
    let kinds = [
        "strategy_uniform",
        "strategy_optimization_informed",
        "fixed_only",
        "passive",
    ];
    let kind = |k| {
        serde_json::to_value(k)
            .unwrap()
            .as_str()
            .unwrap()
            .to_owned()
    };
    let expected: BTreeSet<_> = ["q4_5", "q3_5"]
        .into_iter()
        .flat_map(|env| kinds.into_iter().map(move |k| format!("{env}/{k}")))
        .collect();
    let snapshots: BTreeSet<_> = r
        .controller_snapshots
        .iter()
        .map(|row| format!("{}/{}", row.environment_id, kind(row.snapshot.controller)))
        .collect();
    let fitness: BTreeSet<_> = r
        .fitness_tables
        .iter()
        .map(|row| format!("{}/{}", row.environment_id, kind(row.controller)))
        .collect();
    let targeted: BTreeSet<_> = r
        .targeted_audits
        .iter()
        .map(|row| format!("{}/{}", row.environment_id, kind(row.controller)))
        .collect();
    assert_eq!(snapshots, expected);
    assert_eq!(fitness, expected);
    assert_eq!(targeted, expected);
    assert!(r
        .controller_snapshots
        .iter()
        .all(|row| row.snapshot.rows.len() == 32));
    assert!(r.fitness_tables.iter().all(|row| row.rows.len() == 1024));
    assert_eq!(r.fixed_only_models.len(), 2);
    assert!(r.fixed_only_models.iter().all(|row| row.rows.len() == 8));
    let expected_controls: BTreeSet<_> = expected
        .iter()
        .flat_map(|id| {
            [163882, 5441, 246723, 0, 81942, 98342]
                .into_iter()
                .map(move |bits| format!("{id}/{bits}"))
        })
        .collect();
    let controls: BTreeSet<_> = r
        .control_evaluations
        .iter()
        .map(|row| {
            format!(
                "{}/{}/{}",
                row.environment_id,
                kind(row.controller),
                row.canonical_bits
            )
        })
        .collect();
    assert_eq!(controls, expected_controls);
    assert_eq!(r.control_evaluations.len(), 48);
    let expected_nominal: BTreeSet<_> = expected
        .iter()
        .flat_map(|id| {
            ["strategy_uniform", "strategy_optimization_informed"]
                .into_iter()
                .map(move |population| format!("{id}/{population}"))
        })
        .collect();
    let nominal: BTreeSet<_> = r
        .nominal_evaluations
        .iter()
        .map(|row| {
            format!(
                "{}/{}/{}",
                row.environment_id,
                kind(row.controller),
                row.population
            )
        })
        .collect();
    assert_eq!(nominal, expected_nominal);
    assert_eq!(r.nominal_evaluations.len(), 16);
    let expected_cross: BTreeSet<_> = expected
        .iter()
        .flat_map(|id| {
            kinds
                .into_iter()
                .map(move |target| format!("{id}/{target}"))
        })
        .collect();
    let cross: BTreeSet<_> = r
        .cross_target_evaluations
        .iter()
        .map(|row| {
            format!(
                "{}/{}/{}",
                row.environment_id,
                kind(row.controller),
                kind(row.target_controller)
            )
        })
        .collect();
    assert_eq!(cross, expected_cross);
    assert_eq!(r.cross_target_evaluations.len(), 32);
    assert_eq!(r.policy_provenance.len(), 46);
    let champions: BTreeSet<_> = r
        .policy_provenance
        .iter()
        .filter(|p| p.method == "genetic" || p.method == "random")
        .map(|p| (p.method.clone(), p.seed.unwrap()))
        .collect();
    let expected_champions: BTreeSet<_> = ["genetic", "random"]
        .into_iter()
        .flat_map(|method| (0..20).map(move |seed| (method.to_owned(), seed)))
        .collect();
    assert_eq!(champions, expected_champions);
}
