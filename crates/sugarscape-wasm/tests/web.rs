//! Run with `wasm-pack test --node crates/sugarscape-wasm`.

use sugarscape_core::sweep::{self as core_sweep, Sweep};
use sugarscape_wasm::{
    aggregate, builtin_sweeps, config_series_names, parse_sweep, presets_json, run_point,
    sweep_csv, sweep_points, sweep_result, Sim,
};
use wasm_bindgen::JsValue;
use wasm_bindgen_test::*;

#[wasm_bindgen_test]
fn default_sim_steps_and_renders() {
    let mut sim = Sim::new("{}", 1, JsValue::NULL).unwrap();
    assert_eq!((sim.width(), sim.height()), (50, 50));
    sim.step(3);
    assert_eq!(sim.tick(), 3.0);
    let ptr = sim.render("tribe", "sugar").unwrap();
    assert_ne!(ptr, 0);
    assert_eq!(sim.frame_len(), 50 * 50 * 4);
    assert_eq!(sim.series("population").unwrap().len(), 4);
    assert_eq!(sim.wealth_hist(10).len(), 11);
}

#[wasm_bindgen_test]
fn invalid_config_is_a_json_field_error() {
    let err = Sim::new(r#"{"population": 99999}"#, 1, JsValue::NULL)
        .err()
        .unwrap();
    let text = err.as_string().unwrap();
    assert!(text.contains(r#""field":"population""#), "{text}");
}

const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";

fn base64url_decode(text: &str) -> Vec<u8> {
    let (mut out, mut acc, mut bits) = (Vec::new(), 0u32, 0u32);
    for c in text.bytes() {
        let v = ALPHABET
            .iter()
            .position(|&a| a == c)
            .expect("base64url character") as u32;
        acc = (acc << 6) | v;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push((acc >> bits) as u8);
            acc &= (1 << bits) - 1;
        }
    }
    out
}

const SPICE: &str =
    r#"{"spice":{"enabled":true,"metabolism":{"min":1,"max":4},"endowment":{"min":5,"max":25}}}"#;

#[wasm_bindgen_test]
fn painted_landscapes_round_trip_per_good() {
    let mut sim = Sim::new(SPICE, 1, JsValue::NULL).unwrap();
    assert!(!sim.landscape_edited(0) && !sim.landscape_edited(1));
    sim.paint_capacity(10, 10, 2, 1.0, 1).unwrap();
    assert!(!sim.landscape_edited(0) && sim.landscape_edited(1));
    let caps = sim.export_landscape(1).unwrap();
    let list = js_sys::Array::new();
    list.push(&JsValue::NULL);
    list.push(&js_sys::Uint8Array::from(&caps[..]));
    let again = Sim::new(SPICE, 1, list.into()).unwrap();
    assert_eq!(again.export_landscape(1).unwrap(), caps);
    assert!(!again.landscape_edited(0));
    assert!(sim.export_landscape(2).is_err());
    assert!(sim.paint_capacity(0, 0, 0, 1.0, 2).is_err());
    let sugar = sim.export_landscape(0).unwrap();
    let legacy = Sim::new(SPICE, 1, js_sys::Uint8Array::from(&sugar[..]).into()).unwrap();
    assert_eq!(
        legacy.export_landscape(0).unwrap(),
        sugar,
        "a single array is good 0's"
    );
}

#[wasm_bindgen_test]
fn a_pre_n_goods_share_link_loads_and_runs() {
    let wire: serde_json::Value = serde_json::from_str(include_str!(
        "../../sugarscape-core/tests/fixtures/legacy-share.json"
    ))
    .unwrap();
    let caps = base64url_decode(wire["l"].as_str().unwrap());
    let seed = wire["s"].as_u64().unwrap() as u32;
    let mut sim = Sim::new(
        &wire["c"].to_string(),
        seed,
        js_sys::Uint8Array::from(&caps[..]).into(),
    )
    .unwrap();
    assert_eq!(sim.export_landscape(0).unwrap(), caps);
    sim.step(5);
    assert!(sim.population() > 0);
    let config: serde_json::Value = serde_json::from_str(&sim.export_config()).unwrap();
    assert_eq!(config["goods"].as_array().unwrap().len(), 2);
    assert!(config.get("spice").is_none());
    let names: Vec<String> = serde_json::from_str(&sim.series_names()).unwrap();
    assert!(names.contains(&"mean_holding_1".to_string()));
    sim.render("tribe", "resource:1").unwrap();
    sim.render("tribe", "spice").unwrap();
    assert!(sim.render("tribe", "resource:2").is_err());
}

#[wasm_bindgen_test]
fn presets_are_listed() {
    assert!(presets_json().contains("ii-2-unit"));
}

#[wasm_bindgen_test]
fn partial_config_exports_in_full() {
    // The front end relies on this to normalize share-link configs.
    let sim = Sim::new(r#"{"population": 100}"#, 1, JsValue::NULL).unwrap();
    let full = sim.export_config();
    assert!(full.contains(r#""population":100"#), "{full}");
    assert!(full.contains(r#""goods":[{"name":"sugar""#), "{full}");
    assert!(full.contains(r#""combat":"#), "{full}");
}

#[wasm_bindgen_test]
fn trade_preset_exposes_networks_and_supply_demand() {
    let presets = presets_json();
    assert!(presets.contains("\"iv-3-trade\""));
    let config = r#"{"population":200,"vision":{"min":1,"max":5},"metabolism":{"min":1,"max":5},
        "endowment":{"min":25,"max":50},"spice":{"enabled":true,"metabolism":{"min":1,"max":5},
        "endowment":{"min":25,"max":50}},"trade":{"enabled":true}}"#;
    let mut sim = Sim::new(config, 1, JsValue::NULL).unwrap();
    sim.step(5);
    let edges = sim.networks("trade").unwrap();
    assert_eq!(edges.len() % 4, 0);
    let sd = sim.supply_demand();
    assert_eq!(sd[0] as usize, 41);
    assert_eq!(sd.len(), 1 + 3 * 41 + 4);
    assert!(sim.networks("gossip").is_err());
    sim.render("credit", "spice").unwrap();
}

#[wasm_bindgen_test]
fn disease_api_lists_infects_and_vaccinates() {
    let mut off = Sim::new("{}", 1, JsValue::NULL).unwrap();
    assert!(off.infect(0, 0, -1).is_err(), "disease is off");

    let mut sim = Sim::new(r#"{"disease":{"enabled":true}}"#, 1, JsValue::NULL).unwrap();
    let list: serde_json::Value = serde_json::from_str(&sim.disease_list()).unwrap();
    assert_eq!(list.as_array().unwrap().len(), 10);
    assert!(list[0]["bits"].is_string() && list[0]["carriers"].is_number());
    let p = sim.locate(1.0).unwrap();
    sim.infect(p[0], p[1], -1).unwrap();
    let list: serde_json::Value = serde_json::from_str(&sim.disease_list()).unwrap();
    assert_eq!(
        list.as_array().unwrap().len(),
        11,
        "a new disease was appended"
    );
    assert_eq!(sim.vaccinate(p[0], p[1], 0, 10).unwrap(), 1);
    let view: serde_json::Value = serde_json::from_str(&sim.inspect(p[0], p[1]).unwrap()).unwrap();
    let carried = view["agent"]["diseases"].as_array().unwrap();
    assert!(
        carried.iter().all(|d| d["id"] != 10),
        "vaccinated against #10"
    );
    assert!(sim.vaccinate(p[0], p[1], 0, 99).is_err());
    sim.step(3);
    assert_eq!(sim.networks("disease").unwrap().len() % 4, 0);
    sim.render("disease", "sugar").unwrap();
    assert_eq!(sim.series("new_infections").unwrap().len(), 4);
}

/// The core's `tiny()` sweep: 2 series × 3 x values × 2 seeds, 20 ticks.
const TINY: &str = r#"{
  "name": "tiny",
  "base": { "preset": "ii-2-unit" },
  "set": { "population": 50 },
  "x": { "path": "vision.max", "values": [2, 4, 6] },
  "series": { "label": "Metabolism", "values": [
    { "at": 1, "set": { "goods.0.metabolism": { "min": 1, "max": 1 } } },
    { "at": 3, "name": "wide", "set": { "goods.0.metabolism": { "min": 1, "max": 5 } } }
  ] },
  "seeds": { "from": 5, "count": 2 },
  "ticks": 20,
  "metric": { "kind": "window_mean", "series": "population", "from": 10 }
}"#;

#[wasm_bindgen_test]
fn sweep_points_lists_points_or_errors() {
    let points: serde_json::Value = serde_json::from_str(&sweep_points(TINY).unwrap()).unwrap();
    assert_eq!(points.as_array().unwrap().len(), 12);
    assert_eq!(
        points[7],
        serde_json::json!({ "index": 7, "series": 1, "x": 0, "seed": 6 })
    );
    let err = sweep_points(&TINY.replace("\"ticks\": 20", "\"ticks\": 0")).unwrap_err();
    assert!(err.as_string().unwrap().contains(r#""field":"ticks""#));
    let err = sweep_points("{").unwrap_err();
    assert!(err.as_string().unwrap().contains(r#""field":"sweep""#));
}

#[wasm_bindgen_test]
fn run_point_and_aggregate_match_the_core_run_all() {
    // Completion order does not matter: run the points backwards.
    let runs: Vec<String> = (0..12u32)
        .rev()
        .map(|i| run_point(TINY, i).unwrap())
        .collect();
    let runs = format!("[{}]", runs.join(","));
    let expected = core_sweep::run_all(&Sweep::from_json(TINY).unwrap(), 1, |_, _| {}).unwrap();
    assert_eq!(sweep_result(TINY, &runs).unwrap(), expected.to_json());
    assert_eq!(
        aggregate(TINY, &runs).unwrap(),
        serde_json::to_string(&expected.summary).unwrap()
    );
    assert_eq!(
        sweep_csv(TINY, &runs, "runs").unwrap(),
        core_sweep::runs_csv(&expected)
    );
    assert_eq!(
        sweep_csv(TINY, &runs, "summary").unwrap(),
        core_sweep::summary_csv(&expected)
    );
    assert!(sweep_csv(TINY, &runs, "other").is_err());
    assert!(run_point(TINY, 12).is_err());
}

#[wasm_bindgen_test]
fn partial_runs_aggregate_and_foreign_runs_are_rejected() {
    let one = run_point(TINY, 3).unwrap();
    let summary: serde_json::Value =
        serde_json::from_str(&aggregate(TINY, &format!("[{one}]")).unwrap()).unwrap();
    assert_eq!(summary["rows"].as_array().unwrap().len(), 6);
    assert_eq!(summary["rows"][1]["n"], 1);
    assert!(sweep_result(TINY, &format!("[{one}]"))
        .unwrap()
        .contains("\"incomplete\": true"));
    let foreign = one.replace("\"seed\":6", "\"seed\":99");
    let err = aggregate(TINY, &format!("[{foreign}]")).unwrap_err();
    assert!(err.as_string().unwrap().contains("runs[0]"));
    assert!(aggregate(TINY, "not json").is_err());
}

#[wasm_bindgen_test]
fn builtins_and_series_names_are_listed() {
    let list: serde_json::Value = serde_json::from_str(&builtin_sweeps()).unwrap();
    let ids: Vec<&str> = list
        .as_array()
        .unwrap()
        .iter()
        .map(|b| b["id"].as_str().unwrap())
        .collect();
    assert_eq!(
        ids,
        [
            "fig-ii-5",
            "fig-iv-6",
            "fig-iv-10-11",
            "n-goods-carrying-capacity"
        ]
    );
    assert!(list[0]["sweep"]["name"]
        .as_str()
        .unwrap()
        .starts_with("Figure II-5"));
    let names: Vec<String> = serde_json::from_str(&config_series_names("{}").unwrap()).unwrap();
    assert!(names.iter().any(|n| n == "population"));
    assert!(names.iter().any(|n| n == "mean_holding_0"));
    assert!(config_series_names(r#"{"population": -1}"#).is_err());
}

#[wasm_bindgen_test]
fn parse_sweep_writes_axes_in_full_or_returns_field_errors() {
    let full: serde_json::Value = serde_json::from_str(&parse_sweep(TINY).unwrap()).unwrap();
    assert_eq!(full["x"]["label"], "vision.max");
    assert_eq!(
        full["x"]["values"][1],
        serde_json::json!({ "at": 4.0, "set": { "vision.max": 4 } })
    );
    assert!(full["x"].get("path").is_none());
    // Full form parses back to the same sweep.
    assert_eq!(
        Sweep::from_json(&full.to_string()).unwrap(),
        Sweep::from_json(TINY).unwrap()
    );
    let err = parse_sweep("{").unwrap_err().as_string().unwrap();
    assert!(err.contains(r#""field":"sweep""#), "{err}");
    let err = parse_sweep(&TINY.replace("\"ticks\": 20", "\"ticks\": 0"))
        .unwrap_err()
        .as_string()
        .unwrap();
    assert!(err.contains(r#""field":"ticks""#), "{err}");
}
