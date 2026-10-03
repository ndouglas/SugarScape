use sugarscape_core::model::{ModelConfig, ModelWorld};
#[test]
fn geosim_is_a_validated_generic_model_and_horizon_is_source_periods() {
    let c=ModelConfig::from_json(r#"{"model":"geosim","width":3,"height":3,"initial_states":1,"initialization_periods":0,"observation_periods":13,"periods_per_tick":5}"#).expect("GeoSim kind must parse");
    assert_eq!(c.kind().as_str(), "geosim");
    assert_eq!(c.max_ticks(), Some(3));
    let mut w = ModelWorld::new(c, 17).unwrap();
    w.model_mut().run(100);
    assert_eq!(w.model().tick(), 3);
    assert!(w.model().finished());
    assert_eq!(w.model().population(), 1);
    let before = w.model().fingerprint();
    w.model_mut().run(1);
    assert_eq!(w.model().fingerprint(), before);
}
#[test]
fn inspect_exposes_recurrence_and_render_is_read_only() {
    let c=ModelConfig::from_json(r#"{"model":"geosim","width":2,"height":2,"initial_states":1,"initialization_periods":0,"observation_periods":1,"shock_probability":0}"#).unwrap();
    let mut w = ModelWorld::new(c, 17).unwrap();
    w.model_mut().run(1);
    let before = w.model().fingerprint();
    let j: serde_json::Value =
        serde_json::from_str(&w.model().inspect_json(3, 3).unwrap()).unwrap();
    assert!(j["resource_recurrence"]["old_capacity"].is_number());
    assert_eq!(j["resource_recurrence"]["applied_damage"], 0.0);
    let mut buf = Vec::new();
    w.model().render("technology", "", &mut buf).unwrap();
    assert_eq!(before, w.model().fingerprint());
}
#[test]
fn terminal_inspect_keeps_complete_census_when_ui_trace_is_disabled() {
    let c=ModelConfig::from_json(r#"{"model":"geosim","width":2,"height":2,"initial_states":1,"initialization_periods":0,"observation_periods":1,"event_log":false}"#).unwrap();
    let mut w = ModelWorld::new(c, 5).unwrap();
    w.model_mut().run(1);
    let j: serde_json::Value =
        serde_json::from_str(&w.model().inspect_json(3, 3).unwrap()).unwrap();
    assert!(j["outcome"]["completed_wars"].is_array());
    assert!(j["outcome"]["censored_wars"].is_array());
    assert!(j["outcome"]["exporter_backlog"].is_array());
}
#[test]
fn source_and_artifact_presets_are_discoverable_validated_and_resolved() {
    for p in sugarscape_core::geosim::presets() {
        p.config.validate().unwrap();
        let json = serde_json::to_string(&p.config).unwrap();
        let parsed = ModelConfig::from_json(&json).unwrap();
        assert_eq!(parsed.kind().as_str(), "geosim");
        assert!(sugarscape_core::presets::find(p.id).is_some());
    }
}
