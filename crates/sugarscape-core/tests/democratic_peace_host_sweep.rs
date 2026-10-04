use sugarscape_core::sweep::{aggregate, run_point, runs_csv, Sweep, SweepResult};
fn sweep(extra: &str, metric: &str) -> Sweep {
    Sweep::from_json(&format!(r#"{{"name":"tiny exploratory host probe","base":{{"config":{{"model":"democratic_peace","width":4,"height":4,"horizon_periods":14,"periods_per_tick":3,{extra}}}}},"x":{{"path":"initial_democratic_share","values":[0]}},"seeds":{{"from":1,"count":1}},"ticks":5,"metric":{{"kind":"final","series":"{metric}"}}}}"#)).unwrap()
}
#[test]
fn undefined_clustering_is_complete_and_exported_with_its_reason() {
    let sweep = sweep(r#""zero_ratio":"equal_zero_neutral""#, "clustering_ratio");
    let point = sweep.points().unwrap().remove(0);
    let run = run_point(&sweep, &point);
    let json = serde_json::to_value(&run).unwrap();
    assert!(json["value"].is_null());
    assert_eq!(json["democratic_peace"]["status"], "complete");
    assert_eq!(
        json["democratic_peace"]["clustering_reason"],
        "undefined_initial_density"
    );
    assert_eq!(json["democratic_peace"]["completed_periods"], 14);
    assert_eq!(json["democratic_peace"]["last_tick_periods"], 2);
    let result = SweepResult::new(sweep, vec![run]);
    assert!(runs_csv(&result).contains("complete,14,14,2,,,undefined_initial_density"));
}
#[test]
fn complete_zero_territory_share_remains_in_exploratory_aggregate() {
    let sweep = sweep(r#""zero_ratio":"equal_zero_neutral""#, "democratic_share");
    let point = sweep.points().unwrap().remove(0);
    let run = run_point(&sweep, &point);
    let summary = serde_json::to_value(aggregate(&sweep, &[run])).unwrap();
    assert_eq!(summary["rows"][0]["n"], 1);
    assert_eq!(summary["rows"][0]["mean"], 0.0);
}
#[test]
fn invalid_attempt_preserves_clock_and_excludes_entire_metric_history() {
    let sweep = sweep(
        r#""initial_resourced_share":0,"zero_ratio":"reject_zero_denominator""#,
        "democratic_share",
    );
    let point = sweep.points().unwrap().remove(0);
    let run = run_point(&sweep, &point);
    let json = serde_json::to_value(&run).unwrap();
    assert!(json["value"].is_null());
    assert_eq!(json["democratic_peace"]["status"], "invalid");
    assert_eq!(json["democratic_peace"]["completed_periods"], 0);
    assert_eq!(json["democratic_peace"]["attempted_period"], 1);
    assert!(json["democratic_peace"]["invalid_reason"].is_string());
    let summary = serde_json::to_value(aggregate(&sweep, &[run])).unwrap();
    assert_eq!(summary["rows"][0]["n"], 0);
    assert_eq!(summary["rows"][0]["nan"], 1);
}
