use sugarscape_core::model::{ModelConfig, ModelKind, ModelWorld};
#[test]
fn democratic_peace_discovery_resolves_source_horizon_and_preset() {
    let c = ModelConfig::from_json(r#"{"model":"democratic_peace","width":2,"height":2,"horizon_periods":13,"periods_per_tick":5,"initial_democratic_share":1}"#).expect("democratic_peace must be recognized");
    assert_eq!(c.max_ticks(), Some(3));
    let mut w = ModelWorld::new(c, 17).unwrap();
    w.model_mut().run(100);
    assert_eq!(w.model().tick(), 3);
    assert!(w.model().finished());
    assert_eq!(w.model().latest_value("democratic_share"), Some(1.0));
    assert!(ModelKind::ALL
        .iter()
        .any(|k| k.as_str() == "democratic_peace"));
    assert!(sugarscape_core::presets::find("democratic-peace-printed-2001").is_some());
}
#[test]
fn democratic_peace_t0_census_and_inspection_are_read_only() {
    let c = ModelConfig::from_json(
        r#"{"model":"democratic_peace","width":2,"height":2,"initial_democratic_share":0}"#,
    )
    .expect("democratic_peace must parse");
    let w = ModelWorld::new(c, 4).unwrap();
    let before = w.model().fingerprint();
    let j: serde_json::Value =
        serde_json::from_str(&w.model().inspect_json(3, 3).unwrap()).unwrap();
    assert_eq!(j["metrics"]["first_extinction_period"], 0);
    assert_eq!(
        j["metrics"]["clustering_reason"],
        "undefined_initial_density"
    );
    let mut buf = Vec::new();
    w.model().render("governing_regime", "", &mut buf).unwrap();
    assert_eq!(buf.len(), 12 * 12 * 4);
    assert_eq!(before, w.model().fingerprint());
}
#[test]
fn democratic_peace_invalid_zero_policy_retains_committed_clock() {
    let c=ModelConfig::from_json(r#"{"model":"democratic_peace","width":2,"height":2,"initial_resourced_share":0,"initial_democratic_share":0,"zero_ratio":"reject_zero_denominator"}"#).unwrap();
    let mut w = ModelWorld::new(c, 4).unwrap();
    w.model_mut().run(1);
    let j: serde_json::Value =
        serde_json::from_str(&w.model().inspect_json(3, 3).unwrap()).unwrap();
    assert_eq!(j["outcome"]["valid"], false);
    assert_eq!(j["completed_periods"], 0);
    assert_eq!(j["attempted_period"], 1);
    assert!(j["outcome"]["final_metrics"].is_null());
}
