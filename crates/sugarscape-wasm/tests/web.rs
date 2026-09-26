//! Run with `wasm-pack test --node crates/sugarscape-wasm`.

use sugarscape_core::sweep::{self as core_sweep, Sweep};
use sugarscape_wasm::{
    aggregate, builtin_sweeps, config_series_names, fingerprint_hex, model_schemas_json,
    parse_sweep, presets_json, run_point, sweep_csv, sweep_points, sweep_result, Sim,
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
            "n-goods-carrying-capacity",
            "bargaining-rules",
            "schelling-tipping",
            "lhv-calibration",
            "lhv-quirks",
            "cv-ratio-rules",
            "cv-peacekeeping",
            "cv-jail-waits",
            "nm-universal",
            "hg-async",
            "nbm-grid-discrete",
            "nbm-grid-continuous",
            "nbm-radius",
            "rca-pairings",
            "rca-cost",
            "rca-clones",
            "rca-population",
            "ac-table-2",
            "ac-neighborhoods",
            "ac-territory",
            "ac-activation",
            "ac-traits-transition",
            "ac-drift",
            "dock-mobility",
            "aey-memory",
            "aey-population",
            "aey-first-attractor",
            "aey-tag-regimes",
            "pvplh-payoffs",
            "ha-cost",
            "ha-colors",
            "ha-mutation",
            "ha-immigration",
            "ha-lattice",
            "jansson-tag-mutation",
            "jansson-markers",
            "hk-diagonal",
            "hk-asymmetry",
            "hk-bias",
            "hk-updating",
            "hk-lattice",
            "hk-population",
            "cra-table-2",
            "cra-dial",
            "cra-threshold",
            "cra-noise",
            "cra-population"
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

#[wasm_bindgen_test]
fn set_landscape_replaces_a_goods_capacities() {
    let mut sim = Sim::new("{}", 1, JsValue::NULL).unwrap();
    let caps: Vec<u8> = (0..2500).map(|i| (i % 11) as u8).collect();
    sim.set_landscape(0, &caps).unwrap();
    assert!(sim.landscape_edited(0));
    assert_eq!(sim.export_landscape(0).unwrap(), caps);
    assert!(sim.set_landscape(0, &caps[..10]).is_err());
    assert!(sim.set_landscape(0, &[11; 2500]).is_err());
    assert!(sim.set_landscape(1, &caps).is_err());
}

#[wasm_bindgen_test]
fn following_an_agent_records_its_trail() {
    let mut sim = Sim::new("{}", 1, JsValue::NULL).unwrap();
    assert_eq!(sim.followed(), -1.0);
    assert!(sim.trail().is_empty());
    let config = sim.export_config();
    sim.follow(1.0);
    assert_eq!(sim.followed(), 1.0);
    assert_eq!(sim.trail(), sim.locate(1.0).unwrap());
    for _ in 0..3 {
        sim.step(1);
        if let Some(at) = sim.locate(1.0) {
            let trail = sim.trail();
            assert_eq!(&trail[trail.len() - 2..], &at[..], "newest last");
        }
    }
    assert!(sim.trail().len() >= 2 && sim.trail().len().is_multiple_of(2));
    assert_eq!(sim.export_config(), config, "trails are not config");
    sim.unfollow();
    assert_eq!(sim.followed(), -1.0);
    assert!(sim.trail().is_empty());
}

#[wasm_bindgen_test]
fn credit_graph_lists_the_outstanding_loans() {
    let config = serde_json::to_string(
        &sugarscape_core::presets::by_id("iv-5-credit")
            .unwrap()
            .config,
    )
    .unwrap();
    let mut sim = Sim::new(&config, 1, JsValue::NULL).unwrap();
    assert_eq!(sim.credit_graph(), r#"{"agents":[],"loans":[]}"#);
    // Measured natively: seed 1 has 9 loans outstanding at t = 20 and 15 at t = 50.
    sim.step(50);
    let graph: serde_json::Value = serde_json::from_str(&sim.credit_graph()).unwrap();
    let loans = graph["loans"].as_array().unwrap();
    assert!(!loans.is_empty());
    let agents = graph["agents"].as_array().unwrap();
    let ids: Vec<u64> = agents.iter().map(|a| a["id"].as_u64().unwrap()).collect();
    for l in loans {
        assert!(ids.contains(&l["lender"].as_u64().unwrap()));
        assert!(ids.contains(&l["borrower"].as_u64().unwrap()));
    }
    assert!(agents
        .iter()
        .all(|a| ["lender", "borrower", "both"].contains(&a["role"].as_str().unwrap())));
}

#[wasm_bindgen_test]
fn series_downsampled_pairs_ticks_and_values() {
    let mut sim = Sim::new("{}", 1, JsValue::NULL).unwrap();
    sim.step(50);
    let pop = sim.series("population").unwrap();
    let all = sim.series_downsampled("population", 100).unwrap();
    assert_eq!(all.len(), 2 * 51);
    for (i, (pair, p)) in all.chunks(2).zip(&pop).enumerate() {
        assert_eq!(pair, [i as f64, *p]);
    }
    let few = sim.series_downsampled("population", 10).unwrap();
    assert_eq!(few.len(), 2 * 10);
    assert_eq!((few[0], few[18]), (0.0, 50.0));
    assert!(few.chunks(2).all(|p| pop[p[0] as usize] == p[1]));
    assert!(sim.series_downsampled("nope", 10).is_err());
}

#[wasm_bindgen_test]
fn series_group_shares_one_tick_axis() {
    let mut sim = Sim::new("{}", 1, JsValue::NULL).unwrap();
    sim.step(300);
    let g = sim.series_group(r#"["population","gini"]"#, 20).unwrap();
    let n = g[0] as usize;
    assert!((20..=40).contains(&n), "{n}");
    assert_eq!(g.len(), 1 + 3 * n);
    let ticks = &g[1..1 + n];
    assert_eq!((ticks[0], ticks[n - 1]), (0.0, 300.0));
    assert!(ticks.windows(2).all(|w| w[0] < w[1]));
    let pop = sim.series("population").unwrap();
    let gini = sim.series("gini").unwrap();
    for (k, &t) in ticks.iter().enumerate() {
        assert_eq!(g[1 + n + k], pop[t as usize]);
        assert_eq!(g[1 + 2 * n + k], gini[t as usize]);
    }
    assert!(sim.series_group(r#"["population","nope"]"#, 20).is_err());
    assert!(sim.series_group("not json", 20).is_err());
}

#[wasm_bindgen_test]
fn fingerprint_matches_the_golden_entry() {
    let preset = sugarscape_core::presets::by_id("ii-2-unit").unwrap();
    let json = serde_json::to_string(&preset.config).unwrap();
    let mut sim = Sim::new(&json, 1, JsValue::NULL).unwrap();
    sim.step(200);
    // crates/sugarscape-core/tests/golden.rs
    let fp = sim.fingerprint();
    assert_eq!(fp, "0x75b93943813545e4");
    assert_eq!(
        fp.len(),
        18,
        "fingerprint should be 18 chars (0x + 16 hex digits)"
    );
}

#[wasm_bindgen_test]
fn fingerprint_hex_keeps_leading_zeros() {
    assert_eq!(fingerprint_hex(0xab), "0x00000000000000ab");
    assert_eq!(fingerprint_hex(0), "0x0000000000000000");
    assert_eq!(fingerprint_hex(0x0f14_b0be_4cd3_b21e), "0x0f14b0be4cd3b21e");
    assert_eq!(fingerprint_hex(u64::MAX), "0xffffffffffffffff");
}

#[wasm_bindgen_test]
fn chapter_vi_networks_histograms_and_lineage_colors() {
    let preset = sugarscape_core::presets::by_id("vi-1-everything").unwrap();
    let json = serde_json::to_string(&preset.config).unwrap();
    let mut sim = Sim::new(&json, 1, JsValue::NULL).unwrap();
    assert!(
        sim.networks("neighbors").unwrap().is_empty(),
        "nobody has moved yet"
    );
    sim.step(30);
    for kind in ["neighbors", "friends", "family"] {
        let edges = sim.networks(kind).unwrap();
        assert!(!edges.is_empty(), "{kind}");
        assert_eq!(edges.len() % 4, 0, "{kind}");
    }
    let ages = sim.age_hist(5);
    assert_eq!(ages[0], 5.0);
    assert_eq!(ages.len(), 1 + 21);
    assert_eq!(ages[1..].iter().sum::<f64>(), f64::from(sim.population()));
    let tags = sim.tag_hist();
    assert_eq!(tags.len(), 11);
    assert!(tags.iter().all(|p| (0.0..=100.0).contains(p)));
    sim.render("lineage", "sugar").unwrap();
}

#[wasm_bindgen_test]
fn per_good_wealth_histograms_and_the_total_wealth_lorenz_curve() {
    let preset = sugarscape_core::presets::by_id("iv-1-spice").unwrap();
    let json = serde_json::to_string(&preset.config).unwrap();
    let mut sim = Sim::new(&json, 1, JsValue::NULL).unwrap();
    sim.step(5);
    assert_eq!(sim.good_wealth_hist(0, 20).unwrap(), sim.wealth_hist(20));
    let spice = sim.good_wealth_hist(1, 20).unwrap();
    assert_eq!(spice.len(), 21);
    assert_eq!(spice[1..].iter().sum::<f64>(), f64::from(sim.population()));
    assert!(sim.good_wealth_hist(2, 20).is_err(), "no good 2");
    let total = sim.lorenz_total(101);
    assert_eq!(total.len(), 101);
    assert_eq!((total[0], total[100]), (0.0, 1.0));
    assert_ne!(total, sim.lorenz(101), "spice counts too");
    let one = Sim::new("{}", 1, JsValue::NULL).unwrap();
    assert_eq!(
        one.lorenz_total(11),
        one.lorenz(11),
        "one good: the sugar curve"
    );
}

/// Preset `id`'s config JSON (any model).
fn preset_json(id: &str) -> String {
    serde_json::to_string(&sugarscape_core::presets::find(id).unwrap().config).unwrap()
}

#[wasm_bindgen_test]
fn presets_list_every_model_with_sugarscape_configs_untagged() {
    let list: Vec<serde_json::Value> = serde_json::from_str(&presets_json()).unwrap();
    let model = |id: &str| {
        let p = list.iter().find(|p| p["id"] == id).unwrap();
        p["config"]
            .get("model")
            .and_then(|m| m.as_str())
            .map(String::from)
    };
    assert_eq!(model("ii-2-unit"), None);
    assert_eq!(model("vi-4-schelling-25").as_deref(), Some("schelling"));
    assert_eq!(model("vi-9-ring-megagroup").as_deref(), Some("ring"));
    let first = &list[0];
    let direct = serde_json::to_value(&sugarscape_core::presets::all()[0]).unwrap();
    assert_eq!(first, &direct, "sugarscape presets serialize as before");
}

#[wasm_bindgen_test]
fn schemas_are_listed_for_the_other_models() {
    let schemas: serde_json::Value = serde_json::from_str(&model_schemas_json()).unwrap();
    assert!(schemas.get("sugarscape").is_none());
    let schelling = schemas["schelling"].as_array().unwrap();
    assert!(schelling
        .iter()
        .any(|p| p["path"] == "preference" && p["kind"] == "range" && p["apply"] == "reset"));
    let ring = schemas["ring"].as_array().unwrap();
    let growback = ring.iter().find(|p| p["path"] == "growback").unwrap();
    assert_eq!(
        (growback["kind"].as_str(), growback["apply"].as_str()),
        (Some("number"), Some("live"))
    );
    let start = ring.iter().find(|p| p["path"] == "start").unwrap();
    assert_eq!(start["choices"][1]["value"], "megagroup");
    assert!(start.get("min").is_none());
}

#[wasm_bindgen_test]
fn a_schelling_sim_runs_renders_and_inspects() {
    let mut sim = Sim::new(&preset_json("vi-4-schelling-25"), 1, JsValue::NULL).unwrap();
    assert_eq!(sim.model_kind(), "schelling");
    assert_eq!(
        (sim.width(), sim.height(), sim.population()),
        (50, 50, 2000)
    );
    sim.render("satisfaction", "resource:0").unwrap();
    assert_eq!(sim.frame_len(), 50 * 50 * 4);
    assert!(sim.render("tribe", "resource:0").is_err());
    sim.step(200);
    // crates/sugarscape-core/tests/golden.rs, MODEL_GOLDEN.
    assert_eq!(sim.fingerprint(), "0x7a7072c3433f5f6f");
    let latest: serde_json::Value = serde_json::from_str(&sim.stats_latest()).unwrap();
    assert_eq!(
        (latest["tick"].as_u64(), latest["quiet"].as_u64()),
        (Some(200), Some(1))
    );
    let names: Vec<String> = serde_json::from_str(&sim.series_names()).unwrap();
    assert_eq!(names[..2], ["unsatisfied", "segregation"]);
    assert_eq!(sim.series("segregation").unwrap().len(), 201);
    let view: serde_json::Value = serde_json::from_str(&sim.inspect(0, 0).unwrap()).unwrap();
    assert_eq!(view["site"]["x"], 0);
    let config: serde_json::Value = serde_json::from_str(&sim.export_config()).unwrap();
    assert_eq!(config["model"], "schelling");
    assert!(sim
        .export_series_csv()
        .starts_with("tick,unsatisfied,segregation,"));
    assert!(sim.export_agents_csv().starts_with("id,x,y,color,"));
}

#[wasm_bindgen_test]
fn sugarscape_only_calls_are_empty_or_refused_in_other_models() {
    let mut sim = Sim::new(&preset_json("vi-8-ring-world"), 1, JsValue::NULL).unwrap();
    assert!(sim.lorenz(101).is_empty() && sim.wealth_hist(20).is_empty());
    assert!(sim.age_hist(5).is_empty() && sim.tag_hist().is_empty());
    assert!(sim.lorenz_total(101).is_empty() && sim.supply_demand().is_empty());
    assert!(sim.good_wealth_hist(0, 20).is_err());
    assert!(sim.networks("trade").unwrap().is_empty());
    assert_eq!(sim.credit_graph(), r#"{"agents":[],"loans":[]}"#);
    assert_eq!(sim.disease_list(), "[]");
    assert!(!sim.landscape_edited(0) && sim.export_landscape(0).is_err());
    assert!(sim.paint_capacity(0, 0, 1, 1.0, 0).is_err());
    assert!(sim.place_agent(0, 0, "{}").is_err());
    assert!(sim.remove_agent(0, 0).is_err());
    sim.follow(1.0);
    assert_eq!((sim.followed(), sim.trail().len()), (-1.0, 0));
}

#[wasm_bindgen_test]
fn a_ring_sim_draws_its_space_time_diagram_and_reports_its_ring() {
    let mut sim = Sim::new(&preset_json("vi-8-ring-world"), 1, JsValue::NULL).unwrap();
    assert_eq!(sim.model_kind(), "ring");
    assert_eq!((sim.width(), sim.height()), (150, 150));
    assert_eq!((sim.ring_sugar().len(), sim.ring_agents().len()), (150, 40));
    sim.render("", "").unwrap();
    assert_eq!(sim.frame_len(), 150 * 150 * 4);
    sim.step(200);
    assert_eq!(sim.fingerprint(), "0x1c341361c466db90");
    let site = sim.ring_agents()[0];
    let view: serde_json::Value = serde_json::from_str(&sim.inspect(site, 7).unwrap()).unwrap();
    assert_eq!(view["agent"]["id"], 1);
    assert_eq!(sim.locate(1.0), Some(vec![site, 149]));
    assert!(sim.inspect(150, 0).is_err());
    // Capacity and growback apply live; the ring's size does not.
    let mut config: serde_json::Value = serde_json::from_str(&sim.export_config()).unwrap();
    config["growback"] = serde_json::json!(2.0);
    sim.set_config(&config.to_string()).unwrap();
    config["sites"] = serde_json::json!(200);
    let err = sim
        .set_config(&config.to_string())
        .unwrap_err()
        .as_string()
        .unwrap();
    assert!(err.contains(r#""field":"sites""#), "{err}");
    assert!(Sim::new(&preset_json("ii-2-unit"), 1, JsValue::NULL)
        .unwrap()
        .ring_sugar()
        .is_empty());
    let names: Vec<String> =
        serde_json::from_str(&config_series_names(r#"{"model":"ring"}"#).unwrap()).unwrap();
    assert_eq!(names[0], "flocks");
}

#[wasm_bindgen_test]
fn an_anasazi_sim_matches_the_native_golden_entry_and_finishes() {
    let mut sim = Sim::new(&preset_json("lhv-published"), 1, JsValue::NULL).unwrap();
    assert_eq!(sim.model_kind(), "anasazi");
    assert_eq!((sim.width(), sim.height(), sim.population()), (80, 120, 14));
    for mode in ["occupation", "zones", "yield"] {
        sim.render(mode, "resource:0").unwrap();
    }
    assert_eq!(sim.frame_len(), 80 * 120 * 4);
    sim.step(200);
    // crates/sugarscape-core/tests/golden.rs, MODEL_GOLDEN: the harvest
    // noise's logarithm is the same bits here as natively.
    assert_eq!(sim.fingerprint(), "0x3b357e6f0cc5f74a");
    let latest: serde_json::Value = serde_json::from_str(&sim.stats_latest()).unwrap();
    assert_eq!(
        (latest["tick"].as_u64(), latest["year"].as_u64()),
        (Some(200), Some(1000))
    );
    assert!(!sim.finished());
    sim.step(1000);
    assert!(sim.finished());
    assert_eq!(sim.tick(), 550.0);
    sim.step(1);
    assert_eq!(sim.tick(), 550.0, "a finished world does not step");
    assert!(sim
        .export_series_csv()
        .starts_with("tick,households,historical,fit,"));
    assert!(sim
        .export_agents_csv()
        .starts_with("id,farm_x,farm_y,home_x,home_y,"));
}

#[wasm_bindgen_test]
fn civil_sims_match_the_native_golden_entries() {
    // crates/sugarscape-core/tests/golden.rs, MODEL_GOLDEN: the arrest
    // probability's exponential is the same bits here as natively.
    for (id, fp) in [
        ("cv-run-2-punctuated", "0x7888f03e0d511f6d"),
        ("cv-run-8-nasty-regime", "0x5ce734d905ba0c5d"),
        ("cv-netlogo", "0x87a92345c017b0ae"),
    ] {
        let mut sim = Sim::new(&preset_json(id), 1, JsValue::NULL).unwrap();
        assert_eq!(sim.model_kind(), "civil");
        sim.step(200);
        assert_eq!(sim.fingerprint(), fp, "{id}");
    }
}

#[wasm_bindgen_test]
fn spatial_sims_match_the_native_golden_entries() {
    // crates/sugarscape-core/tests/golden.rs, MODEL_GOLDEN: Eq. 1's powers
    // are the same bits here as natively.
    for (id, fp) in [
        ("nbm-probabilistic", "0x79a048f606d28d74"),
        ("hg-async-kaleidoscope", "0xef13c172a34a980d"),
        ("nbm-random-array", "0xcf2c74041806d530"),
    ] {
        let mut sim = Sim::new(&preset_json(id), 1, JsValue::NULL).unwrap();
        assert_eq!(sim.model_kind(), "spatial");
        sim.step(200);
        assert_eq!(sim.fingerprint(), fp, "{id}");
    }
}

#[wasm_bindgen_test]
fn tags_sims_match_the_native_golden_entries() {
    // crates/sugarscape-core/tests/golden.rs, MODEL_GOLDEN: the Gaussian
    // mutations use the anasazi's portable normal draw.
    for (id, fp) in [
        ("rca-published", "0x1c83900b9b9f0b94"),
        ("eh-no-exact-clones", "0xafe94d6c0bb3b8ab"),
        ("rca-adopt-p1", "0x0c3f75d53d237a89"),
    ] {
        let mut sim = Sim::new(&preset_json(id), 1, JsValue::NULL).unwrap();
        assert_eq!(sim.model_kind(), "tags");
        sim.step(200);
        assert_eq!(sim.fingerprint(), fp, "{id}");
    }
}

#[wasm_bindgen_test]
fn classes_sims_match_the_native_golden_entries() {
    // crates/sugarscape-core/tests/golden.rs, MODEL_GOLDEN.
    for (id, fp) in [
        ("aey-equity", "0x6aa634df2b9c3944"),
        ("aey-tags", "0x1455ea172db78c68"),
        ("pvplh-lattice", "0x7975f5afc2d06c10"),
    ] {
        let mut sim = Sim::new(&preset_json(id), 1, JsValue::NULL).unwrap();
        assert_eq!(sim.model_kind(), "classes");
        sim.step(200);
        assert_eq!(sim.fingerprint(), fp, "{id}");
    }
}

#[wasm_bindgen_test]
fn structure_sims_match_the_native_golden_entries() {
    // crates/sugarscape-core/tests/golden.rs, MODEL_GOLDEN.
    for (id, fp) in [
        ("cra-rwr", "0xc7f45f59d9b25490"),
        ("cra-2dk", "0x3b8c19aab4aae805"),
        ("cra-frne", "0xdf2fc96965742a81"),
    ] {
        let mut sim = Sim::new(&preset_json(id), 1, JsValue::NULL).unwrap();
        assert_eq!(sim.model_kind(), "structure");
        sim.step(200);
        assert_eq!(sim.fingerprint(), fp, "{id}");
    }
}

#[wasm_bindgen_test]
fn opinions_sims_match_the_native_golden_entries() {
    // crates/sugarscape-core/tests/golden.rs, MODEL_GOLDEN.
    for (id, fp) in [
        ("hk-polarisation", "0x204ac33894adc63e"),
        ("hk-serial", "0xb06db73333504889"),
        ("hk-lattice", "0xe33359f120b204d9"),
    ] {
        let mut sim = Sim::new(&preset_json(id), 1, JsValue::NULL).unwrap();
        assert_eq!(sim.model_kind(), "opinions");
        sim.step(200);
        assert_eq!(sim.fingerprint(), fp, "{id}");
    }
}

#[wasm_bindgen_test]
fn culture_sims_match_the_native_golden_entries() {
    // crates/sugarscape-core/tests/golden.rs, MODEL_GOLDEN.
    for (id, fp) in [
        ("ac-sample-run", "0xeb302b62eb20f85d"),
        ("ac-soup", "0xe15b8cab349e25fa"),
        ("ac-drift", "0xf254ab408f46810f"),
    ] {
        let mut sim = Sim::new(&preset_json(id), 1, JsValue::NULL).unwrap();
        assert_eq!(sim.model_kind(), "culture");
        sim.step(200);
        assert_eq!(sim.fingerprint(), fp, "{id}");
    }
}

#[wasm_bindgen_test]
fn ethno_sims_match_the_native_golden_entries() {
    // crates/sugarscape-core/tests/golden.rs, MODEL_GOLDEN: PTR sums and
    // uniform draws are the same bits here as natively.
    for (id, fp) in [
        ("ha-standard", "0xf07433e56417f07c"),
        ("ha-misperception", "0x5567187174fd1c15"),
        ("jansson-kin", "0x265998639eacfbd0"),
    ] {
        let mut sim = Sim::new(&preset_json(id), 1, JsValue::NULL).unwrap();
        assert_eq!(sim.model_kind(), "ethno");
        sim.step(200);
        assert_eq!(sim.fingerprint(), fp, "{id}");
    }
}

#[wasm_bindgen_test]
fn anasazi_overlays_and_inspection() {
    let sim = Sim::new(&preset_json("lhv-published-defaults"), 2, JsValue::NULL).unwrap();
    let n = sim.population() as usize;
    let settlements = sim.anasazi_settlements();
    assert_eq!(
        settlements.chunks(3).map(|s| s[2] as usize).sum::<usize>(),
        n
    );
    let links = sim.anasazi_links();
    assert_eq!(links.len(), 4 * n);
    assert!(!sim.anasazi_water().is_empty());
    let (fx, fy) = (links[0], links[1]);
    let view: serde_json::Value = serde_json::from_str(&sim.inspect(fx, fy).unwrap()).unwrap();
    assert_eq!(view["site"]["farmed_by"], view["agent"]["id"]);
    assert_eq!(view["agent"]["farm"], serde_json::json!([fx, fy]));
    let schemas: serde_json::Value = serde_json::from_str(&model_schemas_json()).unwrap();
    let quirk = schemas["anasazi"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["path"] == "quirks.occupancy_leak")
        .unwrap();
    assert_eq!(
        (quirk["group"].as_str(), quirk["apply"].as_str()),
        (Some("Replication quirks"), Some("reset"))
    );
    assert!(quirk["help"].as_str().unwrap().contains("(A-19)"));
    for other in ["vi-8-ring-world", "ii-2-unit", "vi-4-schelling-25"] {
        let sim = Sim::new(&preset_json(other), 1, JsValue::NULL).unwrap();
        assert!(sim.anasazi_water().is_empty(), "{other}");
        assert!(sim.anasazi_settlements().is_empty(), "{other}");
        assert!(sim.anasazi_links().is_empty(), "{other}");
        assert!(!sim.finished(), "{other}");
    }
}

#[wasm_bindgen_test]
fn the_other_anasazi_presets_match_their_native_golden_entries() {
    // crates/sugarscape-core/tests/golden.rs, MODEL_GOLDEN (200 ticks from seed 1).
    for (id, golden) in [
        ("lhv-published-defaults", "0x7cdec8b85b1f909a"),
        ("lhv-documented", "0x033ffb6476e824b0"),
    ] {
        let mut sim = Sim::new(&preset_json(id), 1, JsValue::NULL).unwrap();
        sim.step(200);
        assert_eq!(sim.fingerprint(), golden, "{id}");
    }
}

#[wasm_bindgen_test]
fn checkpoint_restores_the_same_world() {
    let config = sugarscape_core::presets::find("ii-2-unit").unwrap().config;
    let json = serde_json::to_string(&config).unwrap();
    let mut straight = Sim::new(&json, 1, JsValue::UNDEFINED).unwrap();
    straight.step(30);
    let mut sim = Sim::new(&json, 1, JsValue::UNDEFINED).unwrap();
    sim.step(10);
    let cp = sim.checkpoint().unwrap();
    sim.step(7);
    sim.restore(&cp).unwrap();
    assert_eq!(sim.tick(), 10.0);
    sim.step(20);
    assert_eq!(sim.fingerprint(), straight.fingerprint());
    assert_eq!(
        sim.latest_value("population"),
        Some(f64::from(sim.population()))
    );
}
