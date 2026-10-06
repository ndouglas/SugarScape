use super::super::*;
use serde_json::{json, Value};
use std::sync::OnceLock;

fn report() -> DiagnosticReport {
    static REPORT: OnceLock<DiagnosticReport> = OnceLock::new();
    REPORT.get_or_init(|| diagnose().unwrap()).clone()
}
fn wire() -> Value {
    serde_json::to_value(report()).unwrap()
}
fn rejected(value: Value) {
    let bad: DiagnosticReport = serde_json::from_value(value).unwrap();
    assert!(!report_integrity(&bad).unwrap());
}

#[test]
fn diagnostic_has_all_complete_frozen_rows() {
    let r = report();
    assert!(r.passed);
    let independent_fitness = r
        .checks
        .iter()
        .find(|c| c.quantity == "independent_canonical_fitness_rows")
        .unwrap();
    assert_eq!(
        (
            independent_fitness.expected_numerator,
            independent_fitness.actual_numerator
        ),
        (8192, 8192)
    );
    let independent_evaluations = r
        .checks
        .iter()
        .find(|c| c.quantity == "independent_complete_evaluations")
        .unwrap();
    assert_eq!(
        (
            independent_evaluations.expected_numerator,
            independent_evaluations.actual_numerator
        ),
        (104, 104)
    );
    assert!(report_integrity(&r).unwrap());
    assert_eq!(
        (
            r.controller_snapshots.len(),
            r.fixed_only_models.len(),
            r.policy_provenance.len()
        ),
        (8, 2, 46)
    );
    assert_eq!(
        r.fitness_tables.iter().map(|t| t.rows.len()).sum::<usize>(),
        8192
    );
    assert_eq!(
        (
            r.targeted_audits.len(),
            r.control_evaluations.len(),
            r.nominal_evaluations.len(),
            r.cross_target_evaluations.len()
        ),
        (8, 48, 16, 32)
    );
    assert_eq!(
        serde_json::to_vec(&r).unwrap(),
        serde_json::to_vec(&diagnose().unwrap()).unwrap()
    );
}

#[test]
fn current_payload_corruptions_reject_stale_success_flags() {
    let changes = [
        ("/version", json!("wrong")),
        (
            "/controller_snapshots/0/snapshot/rows/0/decision/action",
            json!("intervene"),
        ),
        (
            "/controller_snapshots/0/snapshot/rows/0/decision/posterior_true/numerator",
            json!(1),
        ),
        ("/environments/0/rules/accuracy/numerator", json!(3)),
        ("/metadata/uniform_prior/0", json!(2)),
        ("/metadata/optimization_informed_prior/4", json!(15)),
        ("/metadata/learning", json!(true)),
        ("/controller_snapshots/0/snapshot/version", json!(2)),
        ("/controller_snapshots/0/snapshot/rows/0/index", json!(1)),
        (
            "/controller_snapshots/0/snapshot/rules/accuracy/numerator",
            json!(3),
        ),
        (
            "/controller_snapshots/0/snapshot/rows/0/observation/live_reports/0",
            json!(true),
        ),
        ("/fitness_tables/0/basis/0/deltas/0", json!(1)),
        ("/fitness_tables/0/rows/0/utility_numerator", json!(1)),
        ("/targeted_audits/0/witness/policy/bits", json!(0)),
        ("/targeted_audits/0/canonical_tie_count", json!(1)),
        ("/targeted_audits/0/guarantee_shortfall/numerator", json!(0)),
        (
            "/nominal_evaluations/0/evaluation/optimal_numerator",
            json!(0),
        ),
        (
            "/cross_target_evaluations/0/target_controller",
            json!("passive"),
        ),
        ("/policy_provenance/6/raw_bits", json!(81943)),
        ("/bound_checks/0/fixed_guarantee/numerator", json!(0)),
        ("/checks/0/actual_numerator", json!(0)),
        ("/checks/0/passed", json!(false)),
        ("/checks/0/expected_numerator", json!(0)),
        ("/checks/0/denominator", json!(2)),
        (
            "/control_evaluations/0/evaluation/histories/0/true_mass",
            json!(1),
        ),
        ("/fitness_tables/0/denominator", json!(9999)),
        ("/passed", json!(false)),
        (
            "/fixed_only_models/0/rows/0/decision/posterior_true/numerator",
            json!(1),
        ),
    ];
    let mut accepted = Vec::new();
    for (path, replacement) in changes {
        let mut value = wire();
        let slot = value
            .pointer_mut(path)
            .unwrap_or_else(|| panic!("missing {path}"));
        assert_ne!(*slot, replacement, "ineffective corruption {path}");
        *slot = replacement;
        let bad: DiagnosticReport = serde_json::from_value(value).unwrap();
        if report_integrity(&bad).unwrap() {
            accepted.push(path);
        }
    }
    assert!(
        accepted.is_empty(),
        "flag-trusting gate accepted corrupt payloads: {accepted:?}"
    );
}
#[test]
fn deleting_rows_or_checks_invalidates_complete_payload() {
    for path in [
        "/controller_snapshots",
        "/fixed_only_models/0/rows",
        "/fitness_tables/0/rows",
        "/targeted_audits",
        "/control_evaluations",
        "/nominal_evaluations",
        "/cross_target_evaluations",
        "/policy_provenance",
        "/bound_checks",
        "/checks",
    ] {
        let mut value = wire();
        value
            .pointer_mut(path)
            .unwrap()
            .as_array_mut()
            .unwrap()
            .pop();
        // Fixed model rows are a Vec so an incomplete wire payload remains typed.
        rejected(value);
    }
}
#[test]
fn strict_decoding_rejects_unknown_fields_at_every_object_level() {
    fn corrupt_objects(value: &Value, path: &mut Vec<String>, out: &mut Vec<Vec<String>>) {
        match value {
            Value::Object(map) => {
                out.push(path.clone());
                for (key, value) in map {
                    path.push(key.clone());
                    corrupt_objects(value, path, out);
                    path.pop();
                }
            }
            Value::Array(values) => {
                for (i, value) in values.iter().enumerate() {
                    path.push(i.to_string());
                    corrupt_objects(value, path, out);
                    path.pop();
                }
            }
            _ => (),
        }
    }
    // One representative row per repeated table avoids thousands of equivalent decodes.
    let mut value = wire();
    fn truncate(value: &mut Value) {
        match value {
            Value::Array(a) => {
                a.truncate(1);
                for v in a {
                    truncate(v);
                }
            }
            Value::Object(m) => {
                for v in m.values_mut() {
                    truncate(v);
                }
            }
            _ => (),
        }
    }
    // Discover object paths on truncated data, then inject into the valid full report.
    truncate(&mut value);
    let mut paths = Vec::new();
    corrupt_objects(&value, &mut Vec::new(), &mut paths);
    for path in paths {
        let mut value = wire();
        let pointer = path
            .iter()
            .map(|s| format!("/{}", s.replace('~', "~0").replace('/', "~1")))
            .collect::<String>();
        value
            .pointer_mut(&pointer)
            .unwrap()
            .as_object_mut()
            .unwrap()
            .insert("unknown".into(), json!(0));
        assert!(
            serde_json::from_value::<DiagnosticReport>(value).is_err(),
            "accepted {pointer}"
        );
    }
}
