//! Checked, complete display records. Called only inside the browser episode worker.
use sugarscape_core::browser_experiments as core;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn experiment_catalog_json() -> Result<String, JsValue> {
    serde_json::to_string(&core::catalog()).map_err(|error| {
        crate::field_errors(vec![core::FieldError::new("catalog", error.to_string())])
    })
}
#[wasm_bindgen]
pub fn experiment_run_json(input_json: &str) -> Result<String, JsValue> {
    let input = core::normalize_input(input_json).map_err(crate::field_errors)?;
    let record = core::run(&input).map_err(crate::field_errors)?;
    core::episode_json(&record).map_err(crate::field_errors)
}
#[wasm_bindgen]
pub fn experiment_validate_json(record_json: &str) -> Result<String, JsValue> {
    let record = core::validate_episode(record_json).map_err(crate::field_errors)?;
    core::episode_json(&record).map_err(crate::field_errors)
}
#[wasm_bindgen]
pub fn experiment_recorded_json() -> Result<String, JsValue> {
    core::recorded_results_json().map_err(crate::field_errors)
}
