use super::super::{normalize_input, Input};
use serde_json::{json, Value};

fn raw_fixed_with_id(id: &str) -> String {
    json!({"study":"foraging_fixed", "seed":"12", "ticks":20, "sample_every":7,
        "setup":{"width":5,"height":5,"nest":{"x":2,"y":2},"agents":1,
        "resources":[{"id":id,"pos":{"x":3,"y":2}}],
        "parameters":{"p_search":1.0,"p_return":0.0,"omega":0.0,
        "lambda_informed":0.0,"lambda_fidelity":0.0,"lambda_publish":0.0,"lambda_waypoint":0.0}}})
    .to_string()
}
fn fixed() -> Value {
    serde_json::from_str(&raw_fixed_with_id("18446744073709551615")).unwrap()
}
fn check(value: &Value) -> Result<Input, Vec<crate::config::FieldError>> {
    normalize_input(&value.to_string())
}

#[test]
fn fixed_accepts_explicit_parameters_and_lossless_id_text() {
    let input = normalize_input(&raw_fixed_with_id("18446744073709551615"));
    assert!(input.is_ok(), "valid explicit F2 input rejected: {input:?}");
    assert_eq!(
        serde_json::to_value(input.unwrap()).unwrap()["setup"]["resources"][0]["id"],
        "18446744073709551615"
    );
}
#[test]
fn required_fields_are_never_filled_from_native_defaults() {
    for field in ["seed", "ticks", "sample_every", "setup"] {
        let mut raw = fixed();
        raw.as_object_mut().unwrap().remove(field);
        assert!(check(&raw).is_err(), "missing {field}");
    }
    let raw = fixed();
    for field in raw["setup"].as_object().unwrap().keys() {
        let mut raw = fixed();
        raw["setup"].as_object_mut().unwrap().remove(field);
        assert!(check(&raw).is_err(), "missing setup.{field}");
    }
    for field in raw["setup"]["parameters"].as_object().unwrap().keys() {
        let mut raw = fixed();
        raw["setup"]["parameters"]
            .as_object_mut()
            .unwrap()
            .remove(field);
        assert!(check(&raw).is_err(), "missing parameter {field}");
    }
}
#[test]
fn ids_and_seeds_reject_malformed_or_noncanonical_text() {
    for bad in [
        json!(""),
        json!("-1"),
        json!("+1"),
        json!("1.0"),
        json!("1e3"),
        json!(" 1"),
        json!("01"),
        json!("18446744073709551616"),
        json!(7),
    ] {
        let mut raw = fixed();
        raw["setup"]["resources"][0]["id"] = bad.clone();
        assert!(check(&raw).is_err(), "invalid id {bad}");
        raw = fixed();
        raw["seed"] = bad.clone();
        assert!(check(&raw).is_err(), "invalid seed {bad}");
    }
}
#[test]
fn coordinate_and_parameter_objects_deny_extra_fields() {
    for pointer in [
        "",
        "/setup",
        "/setup/nest",
        "/setup/resources/0",
        "/setup/resources/0/pos",
        "/setup/parameters",
    ] {
        let mut raw = fixed();
        raw.pointer_mut(pointer)
            .unwrap()
            .as_object_mut()
            .unwrap()
            .insert("unexpected".into(), json!(1));
        assert!(check(&raw).is_err(), "extra field at {pointer}");
    }
}
#[test]
fn native_setup_errors_are_not_normalized_away() {
    let mut raw = fixed();
    raw["setup"]["resources"]
        .as_array_mut()
        .unwrap()
        .push(json!({"id":"18446744073709551615","pos":{"x":4,"y":2}}));
    let errors = check(&raw).unwrap_err();
    assert!(
        errors.iter().any(|e| e.field.contains("resources")),
        "{errors:?}"
    );
    for (pointer, value, expected) in [
        ("/setup/nest/x", json!(5), "nest"),
        ("/setup/resources/0/pos/y", json!(5), "resources"),
        ("/setup/parameters/p_search", json!(1.1), "parameters"),
        ("/setup/parameters/omega", json!(-1), "parameters"),
        (
            "/setup/parameters/lambda_fidelity",
            json!(257),
            "parameters",
        ),
    ] {
        let mut raw = fixed();
        *raw.pointer_mut(pointer).unwrap() = value;
        let errors = check(&raw).unwrap_err();
        assert!(
            errors.iter().any(|e| e.field.contains(expected)),
            "{errors:?}"
        );
    }
}
#[test]
fn browser_profile_rejects_native_valid_large_grid_population_and_opportunities() {
    for (width, height, agents, ticks, expected) in [
        (65, 65, 1, 20, "cells"),
        (5, 5, 17, 20, "agents"),
        (5, 5, 3, 2731, "opportunities"),
    ] {
        let mut raw = fixed();
        raw["setup"]["width"] = json!(width);
        raw["setup"]["height"] = json!(height);
        raw["setup"]["agents"] = json!(agents);
        raw["ticks"] = json!(ticks);
        let errors = check(&raw).unwrap_err();
        assert!(errors.iter().any(|e| e.field == expected), "{errors:?}");
    }
}
#[test]
fn sampling_and_native_tick_bounds_reject_before_capture() {
    for (ticks, sample, expected) in [
        (20, 0, "sample_every"),
        (7201, 7, "ticks"),
        (0, 7, "ticks"),
        (5000, 1, "checkpoints"),
    ] {
        let mut raw = fixed();
        raw["ticks"] = json!(ticks);
        raw["sample_every"] = json!(sample);
        let errors = check(&raw).unwrap_err();
        assert!(errors.iter().any(|e| e.field == expected), "{errors:?}");
    }
}
#[test]
fn storage_estimate_includes_all_agents_maps_and_repeated_native_snapshots() {
    let mut raw = fixed();
    raw["setup"]["width"] = json!(64);
    raw["setup"]["height"] = json!(64);
    raw["setup"]["agents"] = json!(16);
    raw["ticks"] = json!(512);
    raw["sample_every"] = json!(1);
    let errors = check(&raw).unwrap_err();
    assert!(errors.iter().any(|e| e.field == "episode"), "{errors:?}");
}

#[test]
fn max_id_is_converted_exactly_to_the_original_core_setup() {
    let input = normalize_input(&raw_fixed_with_id("18446744073709551615")).unwrap();
    let Input::ForagingFixed { setup, .. } = input else {
        panic!("wrong study")
    };
    assert_eq!(setup.to_core().unwrap().resources[0].id, u64::MAX);
    assert!(normalize_input(&raw_fixed_with_id("18446744073709551616")).is_err());
}
#[test]
fn incremental_budget_rejects_whole_payload_and_preserves_charge_on_error() {
    use super::super::{spatial::budget::CaptureBudget, MAX_EPISODE_BYTES};
    let mut budget = CaptureBudget::new();
    let before = budget.used_bytes();
    assert!(budget.charge(&"x".repeat(MAX_EPISODE_BYTES)).is_err());
    assert_eq!(budget.used_bytes(), before);
    budget
        .charge(&json!({"id":"18446744073709551615"}))
        .unwrap();
    assert!(budget.used_bytes() > before);
}
#[test]
fn incremental_budget_counts_repeated_occurrences_and_punctuation() {
    use super::super::spatial::budget::CaptureBudget;
    let mut budget = CaptureBudget::new();
    let before = budget.used_bytes();
    budget.charge(&json!({"x":"1"})).unwrap();
    let once = budget.used_bytes() - before;
    assert!(once > 9);
    budget.charge(&json!({"x":"1"})).unwrap();
    assert_eq!(budget.used_bytes() - before, once * 2);
}

fn scene(id: &str) -> Value {
    serde_json::to_value(super::super::spatial::scenes::input_for(id).unwrap()).unwrap()
}
#[test]
fn every_declared_engineering_scene_passes_preflight_and_only_implemented_studies_run() {
    use super::super::{catalog, run, spatial};
    let definitions: Value =
        serde_json::from_str(include_str!("../fixtures/spatial-scenes.json")).unwrap();
    for row in definitions["scenes"].as_array().unwrap() {
        let id = row["id"].as_str().unwrap();
        let input = spatial::scenes::input_for(id).unwrap_or_else(|e| panic!("{id}: {e:?}"));
        assert_eq!(serde_json::to_value(&input).unwrap(), row["input"]);
        let implemented = matches!(
            input,
            Input::BurrowExcavation { .. } | Input::BurrowAccess { .. }
        );
        assert_eq!(
            catalog()
                .iter()
                .any(|descriptor| descriptor.id == input.study()),
            implemented,
            "catalog: {id}"
        );
        if implemented {
            assert!(run(&input).is_ok(), "implemented scene: {id}");
        } else {
            assert!(run(&input)
                .unwrap_err()
                .iter()
                .any(|e| e.field == "study" && e.message.contains("not implemented")));
        }
    }
    assert!(spatial::scenes::input_for("unknown").is_err());
}
#[test]
fn native_example_defaults_keep_original_config_setup_seed_and_options() {
    use super::super::spatial::scenes::input_for;
    let Input::BurrowExcavation {
        config,
        seed,
        ticks,
        sample_every,
    } = input_for("burrow-excavation-default").unwrap()
    else {
        panic!()
    };
    let original: crate::burrow::LabConfig = serde_json::from_str(include_str!(
        "../../../../../docs/examples/burrow/direct-blind.json"
    ))
    .unwrap();
    assert_eq!(config.to_core().unwrap(), original);
    assert_eq!((seed.value(), ticks, sample_every), (7, 128, 16));
    let Input::BurrowAccess {
        config,
        seed,
        ticks,
        sample_every,
    } = input_for("burrow-access-default").unwrap()
    else {
        panic!()
    };
    let original: crate::burrow::AccessConfig = serde_json::from_str(include_str!(
        "../../../../../docs/examples/burrow/access-explore.json"
    ))
    .unwrap();
    assert_eq!(config.to_core().unwrap(), original);
    assert_eq!((seed.value(), ticks, sample_every), (7, 128, 16));
    assert_eq!(scene("foraging-fixed-default"), fixed());
    let passage = scene("foraging-passage-default");
    assert_eq!(
        passage["setup"]["open"],
        json!([{"x":0,"y":0},{"x":1,"y":0},{"x":2,"y":0},{"x":3,"y":0},{"x":3,"y":1}])
    );
    let construction = scene("foraging-construction-default");
    assert_eq!(construction["setup"]["diggable"], json!([{"x":3,"y":0}]));
    assert_eq!(construction["setup"]["waste"], json!({"x":0,"y":1}));
}
#[test]
fn all_spatial_dto_objects_require_every_field_and_deny_extra_keys() {
    fn objects(value: &Value, path: String, out: &mut Vec<(String, Vec<String>)>) {
        match value {
            Value::Object(fields) => {
                out.push((path.clone(), fields.keys().cloned().collect()));
                for (key, value) in fields {
                    objects(value, format!("{path}/{key}"), out);
                }
            }
            Value::Array(values) => {
                for (index, value) in values.iter().enumerate() {
                    objects(value, format!("{path}/{index}"), out);
                }
            }
            _ => {}
        }
    }
    for id in [
        "burrow-excavation-default",
        "burrow-access-default",
        "foraging-fixed-default",
        "foraging-passage-default",
        "foraging-construction-default",
    ] {
        let raw = scene(id);
        let mut paths = vec![];
        objects(&raw, String::new(), &mut paths);
        for (path, fields) in paths {
            for field in fields {
                let mut changed = raw.clone();
                changed
                    .pointer_mut(&path)
                    .unwrap()
                    .as_object_mut()
                    .unwrap()
                    .remove(&field);
                assert!(
                    check(&changed).is_err(),
                    "{id} allowed absent {path}/{field}"
                );
            }
            let mut changed = raw.clone();
            changed
                .pointer_mut(&path)
                .unwrap()
                .as_object_mut()
                .unwrap()
                .insert("extra".into(), json!(0));
            assert!(check(&changed).is_err(), "{id} allowed extra {path}");
        }
    }
}
#[test]
fn window_codec_is_lossless_and_choice_clock_overflow_rejects() {
    let mut raw = scene("burrow-excavation-default");
    raw["config"]["freshness_window"] = json!("18446744073709551615");
    let Input::BurrowExcavation { config, .. } = check(&raw).unwrap() else {
        panic!()
    };
    assert_eq!(config.to_core().unwrap().freshness_window, u64::MAX);
    for bad in [
        json!(""),
        json!("0"),
        json!("01"),
        json!("-1"),
        json!("1.1"),
        json!(32),
        json!("18446744073709551616"),
    ] {
        raw["config"]["freshness_window"] = bad.clone();
        assert!(check(&raw).is_err(), "invalid window {bad}");
    }
    let mut raw = scene("burrow-choice");
    raw["config"]["freshness_window"] = json!("9223372036854775807");
    raw["ticks"] = json!(2);
    assert!(check(&raw).unwrap_err().iter().any(|e| e.field == "ticks"));
}
#[test]
fn pure_access_preflight_agrees_with_original_growing_runner_boundaries() {
    use crate::burrow::{self, AccessConfig, RunOptions};
    for (x, y, weight) in [
        (0, 12, 3),
        (2, 10, 3),
        (3, 12, 3),
        (40, 24, 3),
        (41, 12, 3),
        (3, 25, 3),
        (7, 12, 0),
        (7, 12, u32::MAX),
    ] {
        let mut raw = scene("burrow-access-default");
        raw["ticks"] = json!(0);
        raw["config"]["task"]["goal"] = json!({"x":x,"y":y});
        raw["config"]["task"]["goal_weight"] = json!(weight);
        let mut original = raw["config"].clone();
        original["lab"]["freshness_window"] = json!(32);
        let config: AccessConfig = serde_json::from_value(original).unwrap();
        let native = burrow::run_access_episode(
            config,
            7,
            RunOptions {
                ticks: 0,
                sample_every: 16,
            },
        );
        assert_eq!(
            check(&raw).is_ok(),
            native.is_ok(),
            "goal({x},{y}), weight {weight}"
        );
    }
    let mut raw = scene("burrow-access-default");
    raw["config"]["lab"]["fixture"] = json!({"corridor":{"length":9,"workers":2}});
    assert!(check(&raw)
        .unwrap_err()
        .iter()
        .any(|e| e.field == "lab.fixture"));
}
#[test]
fn passage_and_construction_delegate_native_geometry_and_parameter_validation() {
    for (id, resource_field) in [
        ("foraging-passage-default", "resources"),
        ("foraging-construction-default", "food"),
    ] {
        let raw = scene(id);
        for pointer in ["/setup/workers/0/x", "/setup/nest/0/x"] {
            let mut changed = raw.clone();
            *changed.pointer_mut(pointer).unwrap() = json!(99);
            assert!(check(&changed).is_err());
        }
        for parameter in [
            "p_search",
            "p_return",
            "lambda_fidelity",
            "lambda_publish",
            "lambda_waypoint",
        ] {
            let mut changed = raw.clone();
            changed["setup"]["parameters"][parameter] = json!(-1);
            assert!(check(&changed).is_err());
        }
        let mut changed = raw.clone();
        let duplicate = changed["setup"][resource_field][0].clone();
        changed["setup"][resource_field]
            .as_array_mut()
            .unwrap()
            .push(duplicate);
        assert!(check(&changed)
            .unwrap_err()
            .iter()
            .any(|e| e.field.ends_with(".id")));
    }
}
#[test]
fn burrow_budget_counts_repeated_event_prefixes_before_running() {
    let mut raw = scene("burrow-excavation-default");
    raw["sample_every"] = json!(1);
    assert!(check(&raw)
        .unwrap_err()
        .iter()
        .any(|e| e.field == "episode"));
}
#[test]
fn spatial_seed_strictness_preserves_existing_batch_a_normalization() {
    let input =
        normalize_input(r#"{"study":"wink","seed":"0007","policy":"evidence","mode":"ordinary"}"#)
            .unwrap();
    assert_eq!(serde_json::to_value(input).unwrap()["seed"], "7");
}

#[test]
fn zero_tick_records_keep_runner_behavior_despite_choice_validator_gap() {
    use crate::burrow::{self, RunOptions};
    for id in [
        "burrow-choice",
        "burrow-excavation-default",
        "burrow-corridor",
    ] {
        let mut raw = scene(id);
        raw["ticks"] = json!(0);
        let decoded: Input = serde_json::from_value(raw.clone()).unwrap();
        let Input::BurrowExcavation { config, .. } = decoded else {
            panic!()
        };
        let options = RunOptions {
            ticks: 0,
            sample_every: 1,
        };
        // The runner returns an empty-horizon record even for Choice, but its
        // original physical validator requires an actual Choice selection.
        let native = burrow::run_episode(config.to_core().unwrap(), 7, options.clone()).unwrap();
        assert_eq!(
            burrow::validate_episode(&native, &options).is_ok(),
            id != "burrow-choice"
        );
        assert!(
            check(&raw).is_ok(),
            "the original runner accepts {id} at zero ticks"
        );
        assert_eq!(native.completed_ticks, 0);
        assert!(native.events.is_empty());
        assert!(native.choices.is_empty());
        assert_eq!(native.frames.len(), 1);
    }
}
