use super::super::{diagnose, report_integrity, DiagnosticReport};
use serde_json::{json, Value};

#[test]
fn fixed_protocol_is_complete_and_search_free() {
    let report = diagnose().unwrap();
    assert!(
        report.passed,
        "{:?}",
        report
            .checks
            .iter()
            .filter(|check| !check.passed)
            .take(12)
            .collect::<Vec<_>>()
    );
    assert!(report_integrity(&report).unwrap());
    let wire = serde_json::to_value(&report).unwrap();
    for (key, count) in [
        ("inference_models", 4),
        ("mixture_evaluations", 24),
        ("fixed_evaluations", 84),
    ] {
        assert_eq!(wire[key].as_array().unwrap().len(), count, "{key}");
    }
    assert_eq!(
        wire["policy_provenance"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|p| p["method"] == "genetic" || p["method"] == "random")
            .count(),
        40
    );
    for row in wire["inference_models"].as_array().unwrap() {
        assert_eq!(row["calibrations"].as_array().unwrap().len(), 8);
        assert_eq!(row["histories"].as_array().unwrap().len(), 32);
    }
    assert_eq!(report.checks.len(), 33548);
    assert!(report
        .policy_provenance
        .iter()
        .filter(|row| row.seed.is_some())
        .all(|row| row.canonical_bits == 81942));
    assert_eq!(report, diagnose().unwrap());
}

#[test]
fn integrity_rejects_current_payload_corruption_with_stale_success_flags() {
    let baseline = diagnose().unwrap();
    assert!(baseline.passed);
    let valid = serde_json::to_value(baseline).unwrap();
    for mutation in 0..17 {
        let mut wire = valid.clone();
        match mutation {
            0 => wire["catalogs"][0]["catalog"]["entries"][0]["weight"] = json!(2),
            1 => wire["environments"][0]["rules"]["accuracy"]["numerator"] = json!(3),
            2 => wire["environments"][0]["rules"]["fixed_copy_prior"]["numerator"] = json!(2),
            3 => {
                wire["inference_models"][0]["calibrations"][0]["belief"]["policies"][0]
                    ["probability"]["numerator"] = json!(1)
            }
            4 => {
                wire["inference_models"][0]["calibrations"][0]["belief"]["live"][0]["probability"]
                    ["numerator"] = json!(1)
            }
            5 => {
                wire["inference_models"][0]["histories"][0]["decision"]["policies"][0]
                    ["probability"]["numerator"] = json!(1)
            }
            6 => {
                wire["inference_models"][0]["histories"][0]["decision"]["action"] =
                    json!("intervene")
            }
            7 => {
                wire["fixed_evaluations"][0]["evaluation"]["supported_payoff_numerator"] = json!(1)
            }
            8 => {
                wire["fixed_evaluations"][0]["evaluation"]["maximum_belief_error"]["numerator"] =
                    json!(1)
            }
            9 => wire["fixed_evaluations"][0]["evaluation"]["unsupported_mass"] = json!(1),
            10 => wire["policy_provenance"][6]["raw_bits"] = json!(0),
            11 => wire["metadata"]["reference_sha256"] = json!("corrupt"),
            12 => {
                wire["fixed_evaluations"].as_array_mut().unwrap().pop();
            }
            13 => wire["checks"][0]["passed"] = json!(false),
            14 => wire["passed"] = json!(false),
            15 => wire["metadata"]["prior_motivation"] = json!("learned objective rationality"),
            _ => wire["mixture_evaluations"][0]["catalog_id"] = json!("optimization_informed"),
        }
        let report: DiagnosticReport = serde_json::from_value(wire).unwrap();
        assert!(!report_integrity(&report).unwrap(), "mutation {mutation}");
    }
}

fn object_paths(value: &Value, path: Vec<String>, output: &mut Vec<Vec<String>>) {
    match value {
        Value::Object(map) => {
            output.push(path.clone());
            for (key, value) in map {
                let mut next = path.clone();
                next.push(key.clone());
                object_paths(value, next, output);
            }
        }
        Value::Array(rows) => {
            // Shapes repeat; traverse one representative of each homogeneous array.
            if let Some(value) = rows.first() {
                let mut next = path;
                next.push("0".into());
                object_paths(value, next, output);
            }
        }
        _ => {}
    }
}
#[test]
fn report_wire_rejects_unknown_fields_at_every_nested_shape() {
    let base = serde_json::to_value(diagnose().unwrap()).unwrap();
    let decoded: DiagnosticReport = serde_json::from_value(base.clone()).unwrap();
    assert!(report_integrity(&decoded).unwrap());
    let mut paths = Vec::new();
    object_paths(&base, vec![], &mut paths);
    for path in paths {
        let mut wire = base.clone();
        let mut node = &mut wire;
        for part in &path {
            node = if let Ok(index) = part.parse::<usize>() {
                &mut node[index]
            } else {
                &mut node[part]
            };
        }
        node["unrecognized"] = json!(true);
        assert!(
            serde_json::from_value::<DiagnosticReport>(wire).is_err(),
            "{path:?}"
        );
    }
}

#[test]
fn legacy_nested_listener_wire_is_strict_and_has_no_fabricated_beliefs() {
    let report = diagnose().unwrap();
    for row in &report.fixed_evaluations {
        if row.listener.legacy.is_some() {
            assert_eq!(row.evaluation.maximum_belief_error, None);
            for history in &row.evaluation.histories {
                assert_eq!(history.listener_posterior, None);
                assert_eq!(history.belief_error, None);
            }
        }
    }
    let base = serde_json::to_value(&report).unwrap();
    for id in ["bayesian", "evolved"] {
        let mut wire = base.clone();
        let row = wire["fixed_evaluations"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|row| row["listener"]["id"] == id)
            .unwrap();
        row["listener"]["legacy"]["algorithm"]["unexpected"] = json!(true);
        assert!(serde_json::from_value::<DiagnosticReport>(wire).is_err());
    }
}

#[test]
fn exact_reference_checks_recompute_stale_scalar_values() {
    let baseline = diagnose().unwrap();
    assert!(baseline.checks.iter().all(|check| check.passed));
    for field in ["expected", "actual"] {
        let mut wire = serde_json::to_value(&baseline).unwrap();
        wire["checks"][0][field]["numerator"] = json!(1);
        let report: DiagnosticReport = serde_json::from_value(wire).unwrap();
        assert!(!report_integrity(&report).unwrap());
    }
    let mut missing = baseline;
    missing.checks.pop();
    assert!(!report_integrity(&missing).unwrap());
}
