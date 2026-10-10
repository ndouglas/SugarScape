use super::config::{EngagementConfig, Geometry, Side};

fn valid_config() -> EngagementConfig {
    EngagementConfig {
        blue: 16,
        red: 16,
        blue_rate: 0.1,
        red_rate: 0.05,
        dt: 1.0,
        max_steps: 10,
        geometry: Geometry::AimedFire,
    }
}

fn rejects_field(config: &EngagementConfig, field: &str) {
    assert!(
        config
            .validate()
            .unwrap_err()
            .iter()
            .any(|e| e.field == field),
        "missing error for {field}: {config:?}"
    );
}

#[test]
fn counts_respect_each_side_boundary() {
    for (count, accepted) in [(0, true), (4096, true), (4097, false)] {
        for side in [Side::Blue, Side::Red] {
            let mut config = valid_config();
            let field = match side {
                Side::Blue => {
                    config.blue = count;
                    "blue"
                }
                Side::Red => {
                    config.red = count;
                    "red"
                }
            };
            assert_eq!(config.validate().is_ok(), accepted, "{field}={count}");
            if !accepted {
                rejects_field(&config, field);
            }
        }
    }
}

#[test]
fn rates_reject_nonfinite_and_negative_values() {
    for rate in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY, -0.1] {
        for field in ["blue_rate", "red_rate"] {
            let mut config = valid_config();
            if field == "blue_rate" {
                config.blue_rate = rate;
            } else {
                config.red_rate = rate;
            }
            rejects_field(&config, field);
        }
    }
}

#[test]
fn negative_zero_rates_are_valid_zero() {
    let mut config = valid_config();
    config.blue_rate = -0.0;
    config.red_rate = -0.0;
    assert!(config.validate().is_ok());
}

#[test]
fn timestep_requires_finite_positive_value() {
    for dt in [0.0, -0.0, -1.0, f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        let mut config = valid_config();
        config.dt = dt;
        rejects_field(&config, "dt");
    }
}

#[test]
fn horizon_steps_respect_inclusive_boundary() {
    for (steps, accepted) in [(0, false), (1, true), (1_000_000, true), (1_000_001, false)] {
        let mut config = valid_config();
        config.max_steps = steps;
        assert_eq!(config.validate().is_ok(), accepted, "steps={steps}");
    }
}

#[test]
fn work_limit_accepts_exact_boundary() {
    let mut config = valid_config();
    config.blue = 32;
    config.red = 32;
    config.max_steps = 1_000_000;
    assert!(config.validate().is_ok());
}

#[test]
fn work_limit_rejects_one_actor_above_boundary() {
    let mut config = valid_config();
    config.blue = 33;
    config.red = 32;
    config.max_steps = 1_000_000;
    rejects_field(&config, "max_steps");
}

#[test]
fn oversized_count_products_fail_without_integer_overflow() {
    let mut config = valid_config();
    config.blue = u32::MAX;
    config.red = u32::MAX;
    config.max_steps = u64::MAX;
    let errors = config.validate().unwrap_err();
    for field in ["blue", "red", "max_steps"] {
        assert!(errors.iter().any(|error| error.field == field));
    }
}

#[test]
fn horizon_time_must_remain_finite() {
    let mut config = valid_config();
    config.blue_rate = 0.0;
    config.red_rate = 0.0;
    config.dt = f64::MAX;
    config.max_steps = 1;
    assert!(config.validate().is_ok());
    config.max_steps = 2;
    rejects_field(&config, "dt");
}

#[test]
fn per_source_exposure_respects_boundary_for_both_sides() {
    for field in ["blue_rate", "red_rate"] {
        let mut config = valid_config();
        config.blue_rate = 0.0;
        config.red_rate = 0.0;
        for (rate, accepted) in [(0.1, true), (0.100_000_001, false), (f64::MAX, false)] {
            if field == "blue_rate" {
                config.blue_rate = rate;
            } else {
                config.red_rate = rate;
            }
            assert_eq!(config.validate().is_ok(), accepted, "{field}={rate}");
        }
    }
}

#[test]
fn config_wire_format_roundtrips_and_rejects_unknown_fields() {
    let config = valid_config();
    let mut value = serde_json::to_value(&config).unwrap();
    assert_eq!(value["geometry"], "aimed_fire");
    assert_eq!(
        serde_json::from_value::<EngagementConfig>(value.clone()).unwrap(),
        config
    );
    value["typo"] = serde_json::json!(1);
    assert!(serde_json::from_value::<EngagementConfig>(value).is_err());
    assert_eq!(serde_json::to_string(&Side::Red).unwrap(), "\"red\"");
    assert_eq!(
        serde_json::from_str::<Geometry>("\"duel_contact\"").unwrap(),
        Geometry::DuelContact
    );
}
