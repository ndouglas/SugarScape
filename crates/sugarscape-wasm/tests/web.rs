//! Run with `wasm-pack test --node crates/sugarscape-wasm`.

use sugarscape_wasm::{presets_json, Sim};
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
