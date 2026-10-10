//! Node tests of actual checked WASM exports, including complete reproduction.
use serde_json::{json, Value};
use sugarscape_core::browser_experiments as core;
use sugarscape_wasm::{
    experiment_catalog_json, experiment_recorded_json, experiment_run_json,
    experiment_validate_json,
};
use wasm_bindgen::JsValue;
use wasm_bindgen_test::*;
fn errors(result: Result<String, JsValue>, field: &str) {
    let errors: Vec<Value> =
        serde_json::from_str(&result.unwrap_err().as_string().unwrap()).unwrap();
    assert!(
        errors.iter().any(|error| error["field"] == field),
        "expected {field}, actual {errors:?}"
    );
}
#[wasm_bindgen_test]
fn catalog_and_retained_measurements_are_complete_core_records() {
    assert_eq!(
        serde_json::from_str::<Value>(&experiment_catalog_json().unwrap()).unwrap(),
        serde_json::to_value(core::catalog()).unwrap()
    );
    assert_eq!(
        experiment_recorded_json().unwrap(),
        core::recorded_results_json().unwrap()
    );
}
#[wasm_bindgen_test]
fn max_seed_and_complete_reconstruction_are_lossless() {
    let input =
        r#"{"study":"wink","seed":"18446744073709551615","policy":"passive","mode":"ordinary"}"#;
    let record = experiment_run_json(input).unwrap();
    assert_eq!(
        serde_json::from_str::<Value>(&record).unwrap()["input"]["seed"],
        "18446744073709551615"
    );
    assert_eq!(experiment_validate_json(&record).unwrap(), record);
}
#[wasm_bindgen_test]
fn malformed_overflow_unknown_fields_and_raw_size_limits_are_checked() {
    for text in [
        "{".into(),
        r#"{"study":"wink","seed":"18446744073709551616","policy":"passive","mode":"ordinary"}"#
            .into(),
        r#"{"study":"testimony","fixture":"transfer","extra":true}"#.into(),
        " ".repeat(core::MAX_INPUT_BYTES + 1),
    ] {
        errors(experiment_run_json(&text), "input");
    }
    errors(
        experiment_validate_json(&" ".repeat(core::MAX_EPISODE_BYTES + 1)),
        "episode",
    );
}
#[wasm_bindgen_test]
fn forged_flags_zero_denominators_versions_and_checkpoint_limits_are_checked() {
    let base: Value = serde_json::from_str(&experiment_run_json(r#"{"study":"testimony_game","environment":"training","history":31,"listener":"bayesian"}"#).unwrap()).unwrap();
    let mut forged = base.clone();
    forged["passed"] = json!(true);
    errors(experiment_validate_json(&forged.to_string()), "episode");
    let mut forged = base.clone();
    forged["version"] = json!(2);
    errors(experiment_validate_json(&forged.to_string()), "version");
    let mut forged = base.clone();
    forged["payload"]["reference_posterior"]["denominator"] = json!("0");
    errors(experiment_validate_json(&forged.to_string()), "episode");
    let mut forged = base.clone();
    forged["checkpoints"] = json!(vec![base["checkpoints"][0].clone(); 4097]);
    errors(experiment_validate_json(&forged.to_string()), "checkpoints");
}

#[path = "../../test-support/spatial_experiments.rs"]
mod spatial_test_support;
use spatial_test_support::{
    checked_snapshot_bytes, default_seed_cases, invalid_spatial_inputs, spatial_defaults,
    spatial_saved_mutations, spatial_snapshot_inputs,
};

#[wasm_bindgen_test]
fn runnable_catalog_and_five_legacy_spatial_defaults() {
    assert_eq!(core::catalog().len(), 14);
    assert_eq!(spatial_defaults().len(), 5);
    for input in default_seed_cases() {
        let text = input.to_string();
        let expected =
            serde_json::to_value(core::run(&core::normalize_input(&text).unwrap()).unwrap())
                .unwrap();
        let actual = serde_json::from_str::<Value>(&experiment_run_json(&text).unwrap()).unwrap();
        assert_eq!(actual, expected);
        assert_eq!(
            serde_json::from_str::<Value>(&experiment_validate_json(&actual.to_string()).unwrap())
                .unwrap(),
            actual
        );
        assert!(actual["checkpoints"].as_array().unwrap().len() <= core::MAX_CHECKPOINTS);
        assert!(actual.to_string().len() <= core::MAX_EPISODE_BYTES);
        if let Some((reported, bytes)) = checked_snapshot_bytes(&text) {
            assert_eq!(reported, bytes);
            assert_eq!(
                actual["payload"]["native"]["snapshot_bytes"],
                json!(bytes.to_string())
            );
        }
    }
}

#[wasm_bindgen_test]
fn spatial_boundary_rejects_missing_fields_large_ids_and_resource_products() {
    for input in spatial_defaults() {
        for (bad, field) in invalid_spatial_inputs(&input) {
            errors(experiment_run_json(&bad.to_string()), field);
        }
    }
}

#[wasm_bindgen_test]
fn spatial_saved_records_require_complete_same_target_reconstruction() {
    for input in spatial_defaults() {
        let text = input.to_string();
        let actual = serde_json::from_str::<Value>(&experiment_run_json(&text).unwrap()).unwrap();
        for (bad, field) in spatial_saved_mutations(&input, &actual) {
            errors(experiment_validate_json(&bad.to_string()), field);
        }
    }
}

#[wasm_bindgen_test]
fn spatial_scene_and_bounded_extreme_snapshot_bytes_are_recomputed_on_this_target() {
    for input in spatial_snapshot_inputs() {
        let text = input.to_string();
        let (reported, bytes) = checked_snapshot_bytes(&text).unwrap();
        assert_eq!(reported, bytes);
        let actual = serde_json::from_str::<Value>(&experiment_run_json(&text).unwrap()).unwrap();
        assert_eq!(
            actual["payload"]["native"]["snapshot_bytes"],
            json!(bytes.to_string())
        );
    }
}
