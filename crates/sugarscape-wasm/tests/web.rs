//! Run with `wasm-pack test --node crates/sugarscape-wasm`.

use sugarscape_wasm::{presets_json, Sim};
use wasm_bindgen_test::*;

#[wasm_bindgen_test]
fn default_sim_steps_and_renders() {
    let mut sim = Sim::new("{}", 1, None).unwrap();
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
    let err = Sim::new(r#"{"population": 99999}"#, 1, None).err().unwrap();
    let text = err.as_string().unwrap();
    assert!(text.contains(r#""field":"population""#), "{text}");
}

#[wasm_bindgen_test]
fn painted_landscape_round_trips() {
    let mut sim = Sim::new("{}", 1, None).unwrap();
    sim.paint_capacity(10, 10, 2, 4.0).unwrap();
    assert!(sim.landscape_edited());
    let caps = sim.export_landscape();
    let again = Sim::new("{}", 1, Some(caps.clone())).unwrap();
    assert_eq!(again.export_landscape(), caps);
}

#[wasm_bindgen_test]
fn presets_are_listed() {
    assert!(presets_json().contains("ii-2-unit"));
}

#[wasm_bindgen_test]
fn partial_config_exports_in_full() {
    // The front end relies on this to normalize share-link configs.
    let sim = Sim::new(r#"{"population": 100}"#, 1, None).unwrap();
    let full = sim.export_config();
    assert!(full.contains(r#""population":100"#), "{full}");
    assert!(full.contains(r#""landscape":{"kind":"#), "{full}");
    assert!(full.contains(r#""combat":"#), "{full}");
}

#[wasm_bindgen_test]
fn trade_preset_exposes_networks_and_supply_demand() {
    let presets = presets_json();
    assert!(presets.contains("\"iv-3-trade\""));
    let config = r#"{"population":200,"vision":{"min":1,"max":5},"metabolism":{"min":1,"max":5},
        "endowment":{"min":25,"max":50},"spice":{"enabled":true,"metabolism":{"min":1,"max":5},
        "endowment":{"min":25,"max":50}},"trade":{"enabled":true}}"#;
    let mut sim = Sim::new(config, 1, None).unwrap();
    sim.step(5);
    let edges = sim.networks("trade").unwrap();
    assert_eq!(edges.len() % 4, 0);
    let sd = sim.supply_demand();
    assert_eq!(sd[0] as usize, 41);
    assert_eq!(sd.len(), 1 + 3 * 41 + 4);
    assert!(sim.networks("gossip").is_err());
    sim.render("credit", "spice").unwrap();
}
