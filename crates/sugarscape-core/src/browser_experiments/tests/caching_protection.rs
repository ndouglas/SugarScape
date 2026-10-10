use super::caching_input::P3_JSON;
use crate::browser_experiments as browser;
#[test]
fn caching_p3_original_payload_and_every_fingerprint() {
    let input = browser::normalize_input(P3_JSON).unwrap();
    let browser::Input::ProtectionRecaching { lab, seed } = &input else {
        panic!("wrong study")
    };
    let native =
        crate::minds::protection::runner::run_episode(lab.to_core().unwrap(), seed.value(), true)
            .unwrap();
    let record = browser::run(&input).unwrap();
    assert_eq!(
        record.payload["native"],
        browser::wire::lossless_value(&native).unwrap()
    );
    assert_eq!(record.checkpoints.len(), native.frames.len());
    for (checkpoint, frame) in record.checkpoints.iter().zip(&native.frames) {
        assert_eq!(checkpoint.clock["native_tick"], frame.tick.to_string());
        assert_eq!(
            checkpoint.researcher.as_ref().unwrap()["frame"]["fingerprint"],
            frame.fingerprint
        );
        for local in checkpoint.local.values() {
            assert!(!serde_json::to_string(local)
                .unwrap()
                .contains("actual_watchers"));
        }
    }
}
#[test]
fn caching_p3_local_fields_are_allowlisted_at_native_boundaries() {
    let input = browser::normalize_input(P3_JSON).unwrap();
    let record = browser::run(&input).unwrap();
    for at in &record.checkpoints {
        let tick = at.clock["native_tick"]
            .as_str()
            .unwrap()
            .parse::<u64>()
            .unwrap();
        let frame = &at.researcher.as_ref().unwrap()["frame"];
        assert_eq!(at.local.len(), frame["roles"].as_array().unwrap().len());
        for (id, local) in &at.local {
            assert_eq!(
                local
                    .as_object()
                    .unwrap()
                    .keys()
                    .map(String::as_str)
                    .collect::<Vec<_>>(),
                vec!["agent", "availability", "own_actions", "protection", "seen"]
            );
            let role = frame["roles"]
                .as_array()
                .unwrap()
                .iter()
                .find(|r| r["id"] == *id)
                .unwrap();
            assert_eq!(local["agent"]["id"], role["id"]);
            assert_eq!(local["agent"]["holdings"], role["holdings"]);
            assert_eq!(
                local["seen"].as_array().unwrap().len(),
                role["seen"].as_array().unwrap().len()
            );
            for seen in local["seen"].as_array().unwrap() {
                let observed = seen["tick"].as_str().unwrap().parse::<u64>().unwrap();
                assert!(
                    observed < tick,
                    "memory must not be moved ahead of its native action interval"
                );
                assert_eq!(seen["age"], (tick - observed).to_string());
                let site = seen["site"].as_u64().unwrap();
                assert_eq!(seen["pos"], serde_json::json!({"x":site%9,"y":site/9}));
            }
            for a in local["own_actions"].as_array().unwrap() {
                assert_eq!(
                    a.as_object()
                        .unwrap()
                        .keys()
                        .map(String::as_str)
                        .collect::<Vec<_>>(),
                    vec![
                        "action",
                        "burial_cost",
                        "gross_buried",
                        "gross_dug",
                        "harvest",
                        "metabolic_consumed",
                        "metabolic_demand",
                        "perceived_exposure",
                        "phase",
                        "source",
                        "target"
                    ]
                );
            }
        }
        if tick == 0 {
            assert!(at
                .local
                .values()
                .all(|local| local["own_actions"].as_array().unwrap().is_empty()));
        }
    }
}
#[test]
fn caching_p3_native_treatments_preserve_frames_and_mirrors() {
    for policy in ["off", "selective", "indiscriminate", "erased"] {
        for fixture in [
            serde_json::json!({"kind":"single","initial_observed":false,"redeposit_observed":true}),
            serde_json::json!({"kind":"mixed","observed_first":true}),
            serde_json::json!({"kind":"mixed","observed_first":false}),
            serde_json::json!({"kind":"stumble","initial_observed":true}),
            serde_json::json!({"kind":"cue_visible_nonwatcher"}),
            serde_json::json!({"kind":"cue_unseen_watcher"}),
        ] {
            for mirrored in [false, true] {
                let mut raw: serde_json::Value = serde_json::from_str(P3_JSON).unwrap();
                raw["lab"]["policy"] = policy.into();
                raw["lab"]["fixture"] = fixture.clone();
                raw["lab"]["mirrored"] = mirrored.into();
                raw["seed"] = u64::MAX.to_string().into();
                raw["lab"]["exposure_span"] = u64::MAX.to_string().into();
                let input = browser::normalize_input(&raw.to_string()).unwrap();
                let mut native_lab = raw["lab"].clone();
                native_lab["exposure_span"] = serde_json::json!(u64::MAX);
                let native = crate::minds::protection::runner::run_episode(
                    serde_json::from_value(native_lab).unwrap(),
                    u64::MAX,
                    true,
                )
                .unwrap();
                let record = browser::run(&input).unwrap();
                assert_eq!(
                    record.payload["native"],
                    browser::wire::lossless_value(&native).unwrap()
                );
                assert_eq!(record.checkpoints.len(), 65);
                assert_eq!(record.checkpoints.first().unwrap().kind, "caching_initial");
                assert_eq!(record.checkpoints.last().unwrap().kind, "caching_terminal");
                for (at, frame) in record.checkpoints.iter().zip(&native.frames) {
                    assert_eq!(at.clock["native_tick"], frame.tick.to_string());
                    assert_eq!(at.local.len(), frame.roles.len());
                    for role in &frame.roles {
                        assert_eq!(
                            at.local[&role.id.to_string()]["agent"]["pos"],
                            serde_json::to_value(role.pos).unwrap()
                        );
                    }
                }
            }
        }
    }
}
#[test]
fn caching_p3_death_removes_local_state_and_keeps_terminal_boundary() {
    let raw = P3_JSON.replace("0.25", "100.0");
    let input = browser::normalize_input(&raw).unwrap();
    let record = browser::run(&input).unwrap();
    let absent = record
        .checkpoints
        .iter()
        .position(|at| !at.local.contains_key("1"))
        .expect("costly relocation kills native owner");
    assert!(record.checkpoints[absent..]
        .iter()
        .all(|at| !at.local.contains_key("1")));
    assert_eq!(
        record.checkpoints.last().unwrap().clock["native_tick"],
        "64"
    );
}
#[test]
fn caching_p3_import_requires_exact_reconstruction() {
    let input = browser::normalize_input(P3_JSON).unwrap();
    let record = browser::run(&input).unwrap();
    let fresh = browser::validate_episode(&browser::episode_json(&record).unwrap()).unwrap();
    assert_eq!(fresh, record);
    for selector in 0..4 {
        let mut edited = record.clone();
        match selector {
            0 => {
                edited.checkpoints[1].local.get_mut("1").unwrap()["agent"]["holdings"] =
                    serde_json::json!(31.0000000000001)
            }
            1 => edited.payload["native"]["seed"] = serde_json::json!("8"),
            2 => {
                edited.checkpoints[1].researcher.as_mut().unwrap()["frame"]["fingerprint"] =
                    serde_json::json!("0")
            }
            _ => edited.checkpoints[1].clock["native_tick"] = serde_json::json!("0"),
        }
        assert!(browser::validate_episode(&browser::episode_json(&edited).unwrap()).is_err());
    }
}
#[test]
fn caching_p3_short_span_keeps_original_aged_memory_and_native_payload() {
    // Native expiry is checked at selection; retained exposure entries are not
    // swept out of this capture. Preserve that evidence, including its old tick.
    let raw = P3_JSON.replace("\"exposure_span\":\"64\"", "\"exposure_span\":\"1\"");
    let input = browser::normalize_input(&raw).unwrap();
    let browser::Input::ProtectionRecaching { lab, seed } = &input else {
        panic!("wrong study")
    };
    let native =
        crate::minds::protection::runner::run_episode(lab.to_core().unwrap(), seed.value(), true)
            .unwrap();
    assert!(native.fixture_errors.is_empty());
    let record = browser::run(&input).unwrap();
    assert_eq!(
        record.payload["native"],
        browser::wire::lossless_value(&native).unwrap()
    );
    let retained = &record.checkpoints[8].local["1"]["protection"]["exposure"]["entries"][0];
    assert_eq!(retained["tick"], "0");
    assert_eq!(retained["age"], "8");
}
#[test]
fn caching_p3_initial_deposit_and_memory_keep_distinct_action_clocks() {
    let record = browser::run(&browser::normalize_input(P3_JSON).unwrap()).unwrap();
    assert_eq!(
        record.checkpoints[0].clock["action_interval"],
        serde_json::Value::Null
    );
    assert_eq!(
        record.checkpoints[1].clock["action_interval"],
        serde_json::json!({"start_tick":"0","end_tick":"1"})
    );
    assert_eq!(
        record.checkpoints[1].local["1"]["own_actions"][0]["action"],
        "prepare_deposit"
    );
    let memory = &record.checkpoints[1].local["2"]["seen"][0];
    assert_eq!(memory["tick"], "0");
    assert_eq!(memory["age"], "1");
    assert_eq!(
        record.checkpoints[8].local["1"]["protection"]["sources"][0]["age"],
        "8"
    );
}
#[test]
fn caching_p3_frozen_examples_and_original_seed7_fingerprints() {
    let expected = [
        (
            "protection-observed-off",
            "ddac81d55b27021f",
            "1942912d774c25ee",
        ),
        (
            "protection-observed-selective",
            "eecb6fc4990b80e4",
            "7c1aa41a639a0753",
        ),
        (
            "protection-observed-indiscriminate",
            "7cbcf98d1da206d2",
            "0edb8bcbe58193d3",
        ),
        (
            "protection-observed-erased",
            "56f013897704a74d",
            "8facc7d034c84dfe",
        ),
    ];
    for (scene, first, last) in expected {
        let input = browser::caching::scenes::input_for(scene).unwrap();
        let record = browser::run(&input).unwrap();
        assert_eq!(
            record
                .checkpoints
                .first()
                .unwrap()
                .researcher
                .as_ref()
                .unwrap()["frame"]["fingerprint"],
            first
        );
        assert_eq!(
            record
                .checkpoints
                .last()
                .unwrap()
                .researcher
                .as_ref()
                .unwrap()["frame"]["fingerprint"],
            last
        );
    }
    for scene in [
        "protection-unobserved-selective",
        "protection-mixed-selective",
        "protection-visible-nonwatcher",
        "protection-unseen-watcher",
        "protection-stumble",
    ] {
        assert!(browser::run(&browser::caching::scenes::input_for(scene).unwrap()).is_ok());
    }
}
#[test]
fn caching_p3_memory_expiry_matches_native_cutoff_at_current_boundary() {
    let input = browser::normalize_input(
        &P3_JSON.replace("\"exposure_span\":\"64\"", "\"exposure_span\":\"8\""),
    )
    .unwrap();
    let record = browser::run(&input).unwrap();
    for (boundary, expired) in [(8, false), (9, true)] {
        let state = &record.checkpoints[boundary].local["1"]["protection"];
        assert_eq!(state["exposure_span"], "8");
        for entry in [&state["sources"][0], &state["exposure"]["entries"][0]] {
            assert_eq!(entry["age"], boundary.to_string());
            assert_eq!(entry["expired"], expired);
            assert_eq!(
                entry["expiry_semantics"],
                "retained_memory_eligibility_at_current_boundary"
            );
        }
    }
}
