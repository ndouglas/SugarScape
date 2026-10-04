use serde_json::json;
use sugarscape_core::sweep::{aggregate, run_point, Outcome, Summary, Sweep};

fn fixture(metric: serde_json::Value, damage: f64) -> Sweep {
    Sweep::from_json(
        &json!({
            "name":"invalid terminal GeoSim", "base":{"config":{
                "model":"geosim", "width":2,"height":2,"initial_states":4,
                "initialization_periods":0,"observation_periods":4,"periods_per_tick":4,
                "resource_adjustment":1.0,"damage_fraction":damage,
                "attack_probability":1.0,"superiority_threshold":0.1
            }}, "x":{"path":"shock_shift","values":[20]},
            "seeds":{"from":1,"count":1},"ticks":1,"metric":metric
        })
        .to_string(),
    )
    .unwrap()
}

#[test]
fn invalid_geosim_final_and_window_are_excluded_from_scalar_aggregates() {
    for metric in [
        json!({"kind":"final","series":"completed_wars"}),
        json!({"kind":"window_mean","series":"completed_wars","from":0}),
    ] {
        let sweep = fixture(metric, 1.0);
        let point = sweep.points().unwrap().remove(0);
        let run = run_point(&sweep, &point);
        assert!(matches!(run.outcome, Outcome::Scalar{value} if value.is_nan()));
        let Summary::Scalar(rows) = aggregate(&sweep, &[run]) else {
            panic!()
        };
        assert_eq!((rows[0].n, rows[0].nan), (0, 1));
    }
}
#[test]
fn invalid_geosim_timeseries_is_missing_but_valid_sibling_is_counted() {
    let metric = json!({"kind":"timeseries","series":"completed_wars","every":1});
    let sweep = fixture(metric, 1.0);
    let run = run_point(&sweep, &sweep.points().unwrap()[0]);
    assert!(matches!(run.outcome, Outcome::Series{values} if values.iter().all(|x|x.is_nan())));
    let sweep = fixture(json!({"kind":"final","series":"completed_wars"}), 0.0);
    let run = run_point(&sweep, &sweep.points().unwrap()[0]);
    assert!(matches!(run.outcome, Outcome::Scalar{value} if value.is_finite()));
    let Summary::Scalar(rows) = aggregate(&sweep, &[run]) else {
        panic!()
    };
    assert_eq!((rows[0].n, rows[0].nan), (1, 0));
}

#[test]
fn invalid_partial_outcome_and_raw_statistics_remain_inspectable() {
    let sweep = fixture(json!({"kind":"final","series":"completed_wars"}), 1.0);
    let point = &sweep.points().unwrap()[0];
    let mut world =
        sugarscape_core::model::ModelWorld::new(sweep.config_for(point).unwrap(), 1).unwrap();
    world.model_mut().run(1);
    assert!(world
        .model()
        .series("completed_wars")
        .unwrap()
        .iter()
        .all(|v| v.is_finite()));
    let sugarscape_core::model::ModelWorld::Geosim(world) = world else {
        panic!()
    };
    let outcome = world.outcome().unwrap();
    assert_eq!(
        (outcome.valid, outcome.periods, outcome.attempted_period),
        (false, 3, 4)
    );
}
