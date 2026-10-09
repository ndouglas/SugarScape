//! Approved computed-float paths are narrow; fresh reconstruction remains authority.
use super::super::*;
use serde_json::{json, Value};
fn episode(input: Value) -> EpisodeRecord {
    run(&normalize_input(&input.to_string()).unwrap()).unwrap()
}
fn testimony() -> EpisodeRecord {
    episode(json!({"study":"testimony","fixture":"verified_truth"}))
}
fn game() -> EpisodeRecord {
    episode(
        json!({"study":"testimony_game","environment":"training","history":31,"listener":"bayesian"}),
    )
}
fn changed(record: &EpisodeRecord, pointer: &str, replacement: Value) -> String {
    let mut value = serde_json::to_value(record).unwrap();
    *value.pointer_mut(pointer).unwrap() = replacement;
    value.to_string()
}
#[test]
fn imported_computed_snapshot_probabilities_use_original_tolerance() {
    let record = testimony();
    for base in [
        "/payload/snapshot",
        "/checkpoints/0/local/listener/snapshot",
        "/checkpoints/0/researcher/snapshot",
    ] {
        for suffix in [
            "/hypotheses/0/probability",
            "/propositions/0/probability_true",
            "/speakers/0/profiles/0/probability",
        ] {
            let pointer = format!("{base}{suffix}");
            let original = serde_json::to_value(&record)
                .unwrap()
                .pointer(&pointer)
                .unwrap()
                .as_f64()
                .unwrap();
            assert_eq!(
                validate_episode(&changed(&record, &pointer, json!(original + 1e-13))).unwrap(),
                record,
                "{pointer}"
            );
        }
    }
}
#[test]
fn tolerance_boundary_accepts_one_e_minus_twelve_and_rejects_next_float() {
    let record = testimony();
    let value = serde_json::to_value(&record).unwrap();
    let index = value["payload"]["snapshot"]["hypotheses"]
        .as_array()
        .unwrap()
        .iter()
        .position(|h| h["probability"].as_f64() == Some(0.0))
        .unwrap();
    let pointer = format!("/payload/snapshot/hypotheses/{index}/probability");
    assert_eq!(
        validate_episode(&changed(&record, &pointer, json!(1e-12))).unwrap(),
        record
    );
    let outside = f64::from_bits(1e-12f64.to_bits() + 1);
    assert!(validate_episode(&changed(&record, &pointer, json!(outside))).is_err());
}
#[test]
fn game_posterior_and_float_regret_are_reconstructed_with_narrow_tolerance() {
    let record = game();
    let value = serde_json::to_value(&record).unwrap();
    for pointer in [
        "/payload/decision/posterior_true",
        "/checkpoints/1/local/2/decision/posterior_true",
        "/payload/conditional_regret",
    ] {
        let original = value.pointer(pointer).unwrap().as_f64().unwrap();
        assert_eq!(
            validate_episode(&changed(&record, pointer, json!(original + 1e-13))).unwrap(),
            record
        );
        assert!(validate_episode(&changed(&record, pointer, json!(original + 2e-12))).is_err());
        assert!(validate_episode(&changed(&record, pointer, Value::Null)).is_err());
    }
}
#[test]
fn browser_integer_numeric_spellings_preserve_computed_and_declared_values() {
    let record = testimony();
    let mut value = serde_json::to_value(&record).unwrap();
    fn integral_floats(value: &mut Value) {
        match value {
            Value::Number(n) if n.is_f64() && n.as_f64().unwrap().fract() == 0.0 => {
                *value = json!(n.as_f64().unwrap() as i64)
            }
            Value::Array(a) => a.iter_mut().for_each(integral_floats),
            Value::Object(o) => o.values_mut().for_each(integral_floats),
            _ => {}
        }
    }
    integral_floats(&mut value);
    assert_eq!(validate_episode(&value.to_string()).unwrap(), record);
}
#[test]
fn model_parameters_observations_and_assumptions_remain_exact() {
    let record = testimony();
    let value = serde_json::to_value(&record).unwrap();
    for pointer in [
        "/payload/model/groups/0/accuracy",
        "/payload/model/hypotheses/0/prior",
        "/checkpoints/0/public/inference_assumptions/model/groups/0/accuracy",
    ] {
        let original = value.pointer(pointer).unwrap().as_f64().unwrap();
        assert!(
            validate_episode(&changed(&record, pointer, json!(original + 1e-13))).is_err(),
            "{pointer}"
        );
    }
    let record = game();
    for (pointer, replacement) in [
        ("/payload/decision/action", json!("abstain")),
        ("/checkpoints/1/public/action", json!("abstain")),
        ("/payload/reference_posterior/numerator", json!("2402")),
        ("/payload/reference_posterior/denominator", json!("0")),
        ("/payload/availability", json!("zero_mass")),
        ("/checkpoints/0/clock/history", json!(31)),
        ("/checkpoints/1/index", json!(0)),
    ] {
        assert!(
            validate_episode(&changed(&record, pointer, replacement)).is_err(),
            "{pointer}"
        );
    }
}
#[test]
fn snapshot_shapes_ids_counts_and_null_availability_cannot_be_forged() {
    let record = testimony();
    let value = serde_json::to_value(&record).unwrap();
    for (pointer, replacement) in [
        ("/payload/snapshot/hypotheses/0/id", json!("999")),
        ("/payload/snapshot/evidence_count", json!("999")),
        ("/payload/snapshot/hypotheses/0/probability", Value::Null),
        ("/checkpoints/0/researcher/label", json!("Agent")),
        ("/payload/snapshot/hypotheses", json!([])),
    ] {
        assert!(
            validate_episode(&changed(&record, pointer, replacement)).is_err(),
            "{pointer}"
        );
    }
    let mut forged = value;
    forged["payload"]["snapshot"]["invented"] = json!(0.5);
    assert!(validate_episode(&forged.to_string()).is_err());
    let passive = episode(
        json!({"study":"testimony_game","environment":"training","history":31,"listener":"passive"}),
    );
    assert!(validate_episode(&changed(
        &passive,
        "/payload/decision/posterior_true",
        json!(0.0)
    ))
    .is_err());
}
#[test]
fn viewer_wink_identity_declares_portability_separately_from_original_measurement() {
    let record = episode(json!({"study":"wink","seed":"7","policy":"evidence","mode":"ordinary"}));
    assert!(record.rules_identity.contains("measurement-source-sha256:c5af2cf7baea872bcef4628a37efdf5a3b7c47b05405752232c3096c8e812c09"));
    assert!(record
        .rules_identity
        .contains("portability-runtime-u64-index-v1:viewer-source-sha256:"));
}
