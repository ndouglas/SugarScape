//! Generic host dispatch must preserve the original model's period clock.
use sugarscape_core::model::{ModelConfig, ModelWorld};

#[test]
fn polarity_config_routes_to_a_complete_partial_final_tick() {
    let config = ModelConfig::from_json(
        r#"{"model":"polarity","predator_share":0,"horizon":3,"periods_per_tick":2}"#,
    )
    .unwrap();
    let mut world = ModelWorld::new(config, 7).unwrap();
    world.model_mut().run(100);
    assert_eq!(world.model().latest_value("period"), Some(3.));
    assert_eq!(world.model().latest_value("sovereign_count"), Some(100.));
    assert!(world.model().finished());
    let endpoint = world.model().fingerprint();
    world.model_mut().run(100);
    assert_eq!(world.model().fingerprint(), endpoint);
}
