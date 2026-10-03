use sugarscape_core::sweep::Sweep;
#[test]
fn excessive_grouped_geosim_sweep_names_source_horizon() {
    let spec = r#"{"name":"bounded host fixture","base":{"config":{"model":"geosim","width":2,"height":2,"initial_states":1,"initialization_periods":2,"observation_periods":13,"periods_per_tick":7}},"x":{"path":"shock_shift","values":[0]},"seeds":{"from":1,"count":1},"ticks":4,"metric":{"kind":"final","series":"completed_wars"}}"#;
    let errors = Sweep::from_json(spec).unwrap().points().unwrap_err();
    assert_eq!(errors[0].field, "ticks");
    assert!(
        errors[0]
            .message
            .contains("GeoSim source horizon of 15 periods"),
        "{errors:?}"
    );
    assert!(errors[0].message.contains("3 display ticks"), "{errors:?}");
}
