//! Tiny deterministic tests of the production WASM boundary; no registered study.
use sugarscape_wasm::{model_schemas_json, presets_json, Sim};
use wasm_bindgen::JsValue;
use wasm_bindgen_test::*;
wasm_bindgen_test_configure!(run_in_browser);
#[wasm_bindgen_test]
fn democratic_peace_discovers_presets_schema_and_finite_partial_tick_exports() {
    let presets: serde_json::Value = serde_json::from_str(&presets_json()).unwrap();
    let entries = presets
        .as_array()
        .unwrap()
        .iter()
        .filter(|entry| entry["config"]["model"] == "democratic_peace")
        .collect::<Vec<_>>();
    assert_eq!(entries.len(), 6);
    let schemas: serde_json::Value = serde_json::from_str(&model_schemas_json()).unwrap();
    assert!(schemas["democratic_peace"]
        .as_array()
        .unwrap()
        .iter()
        .any(|param| param["path"] == "probability_direction"));
    let mut sim = Sim::new(r#"{"model":"democratic_peace","width":4,"height":4,"horizon_periods":14,"periods_per_tick":3}"#, 393939, JsValue::NULL).unwrap();
    sim.step(5);
    let latest: serde_json::Value = serde_json::from_str(&sim.stats_latest()).unwrap();
    assert_eq!(latest["periods"], 14);
    assert_eq!(latest["last_tick_periods"], 2);
    assert!(sim.finished());
    let fingerprint = sim.fingerprint();
    for mode in [
        "territory",
        "governing_regime",
        "latent_regime",
        "resources",
        "alliances",
        "pariahs",
    ] {
        assert!(sim.render(mode, "sugar").is_ok());
        assert!(sim.frame_len() > 0);
    }
    let inspect: serde_json::Value = serde_json::from_str(&sim.inspect(0, 0).unwrap()).unwrap();
    assert_eq!(inspect["model"], "democratic_peace");
    let state: serde_json::Value = serde_json::from_str(&sim.export_model_json().unwrap()).unwrap();
    assert!(state.is_object());
    assert!(sim.export_series_csv().contains("periods"));
    sim.step(1);
    assert_eq!(sim.fingerprint(), fingerprint);
}
