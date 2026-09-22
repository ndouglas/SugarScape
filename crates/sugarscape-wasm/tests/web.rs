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
