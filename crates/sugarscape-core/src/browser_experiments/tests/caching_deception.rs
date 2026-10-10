use super::caching_input::P4_JSON;
use crate::browser_experiments as browser;
use serde_json::{json, Value};

fn input(changes: Value) -> browser::Input {
    let mut raw: Value = serde_json::from_str(P4_JSON).unwrap();
    for (key, value) in changes.as_object().unwrap() {
        raw["lab"][key] = value.clone();
    }
    browser::normalize_input(&raw.to_string()).unwrap()
}
#[test]
fn caching_deception_receiver_separation_and_native_payload() {
    for view in ["ambiguous", "clear"] {
        let input = input(json!({"view":view}));
        let browser::Input::DeceptionGestures { lab, seed } = &input else {
            unreachable!()
        };
        let native =
            crate::minds::deception::run_episode(lab.to_core().unwrap(), seed.value(), true)
                .unwrap();
        let record = browser::run(&input).unwrap();
        assert_eq!(
            record.payload["native"],
            browser::wire::lossless_value(&native).unwrap()
        );
        for (at, frame) in record.checkpoints.iter().zip(&native.frames) {
            assert_eq!(
                at.researcher.as_ref().unwrap()["frame"],
                browser::wire::lossless_value(frame).unwrap()
            );
            for role in &frame.roles {
                let local = &at.local[&role.id.to_string()];
                let received: Vec<_> = frame
                    .observations
                    .iter()
                    .filter(|o| o.receiver == role.id)
                    .map(|o| browser::wire::lossless_value(&o.public).unwrap())
                    .collect();
                assert_eq!(local["received_observations"], json!(received));
                let serialized = local.to_string();
                for forbidden in [
                    "actual_transfer",
                    "actual_stock",
                    "actual_value",
                    "target_occupant",
                    "source_recovered",
                    "choices",
                    "cancellation",
                    "restrictions",
                    "stocks",
                ] {
                    assert!(!serialized.contains(forbidden), "leaked {forbidden}");
                }
            }
        }
    }
}
#[test]
fn caching_deception_all_native_treatment_axes_and_repeat() {
    // Full 96-cell factorial spans crucial signal/visibility/layout/cost interactions.
    for sender in ["ordinary", "matched_neutral", "sham"] {
        for view in ["ambiguous", "clear"] {
            for seen in [false, true] {
                for layout in ["on_route", "off_route"] {
                    for cost in [0.0, 3.0] {
                        for mirrored in [false, true] {
                            let mut input = input(
                                json!({"sender":sender,"view":view,"display_seen":seen,"layout":layout,"effort_cost":cost,"mirrored":mirrored}),
                            );
                            let browser::Input::DeceptionGestures { lab, seed } = &mut input else {
                                unreachable!()
                            };
                            *seed = serde_json::from_value(json!(u64::MAX.to_string())).unwrap();
                            let native = crate::minds::deception::run_episode(
                                lab.to_core().unwrap(),
                                seed.value(),
                                true,
                            )
                            .unwrap();
                            let record = browser::run(&input).unwrap();
                            assert_eq!(
                                record.payload["native"],
                                browser::wire::lossless_value(&native).unwrap()
                            );
                            assert_eq!(record.checkpoints.len(), native.frames.len());
                            assert_eq!(
                                record.checkpoints.len(),
                                65,
                                "fixed observer survives horizon"
                            );
                            for (at, frame) in record.checkpoints.iter().zip(&native.frames) {
                                assert_eq!(
                                    at.researcher.as_ref().unwrap()["frame"],
                                    browser::wire::lossless_value(frame).unwrap()
                                );
                                assert_eq!(at.local.len(), frame.roles.len());
                            }
                            assert_eq!(record, browser::run(&input).unwrap());
                        }
                    }
                }
            }
        }
    }
}
#[test]
fn caching_deception_clear_zero_and_stale_memory_have_original_clocks() {
    let record = browser::run(&input(json!({"view":"clear","layout":"on_route"}))).unwrap();
    assert_eq!(record.checkpoints[0].clock["action_interval"], Value::Null);
    assert!(record.checkpoints[0]
        .local
        .values()
        .all(|l| l["own_actions"].as_array().unwrap().is_empty()));
    let before = &record.checkpoints[13].local["2"]["seen"];
    let at = &record.checkpoints[14];
    assert_eq!(
        at.clock["action_interval"],
        json!({"start_tick":"13","end_tick":"14"})
    );
    assert_eq!(
        at.local["2"]["received_observations"][0]["signal"],
        json!({"visible_transfer":{"amount":0.0}})
    );
    let after = &at.local["2"]["seen"];
    assert_eq!(before[0]["amount"], after[0]["amount"]);
    assert_eq!(before[0]["tick"], after[0]["tick"]);
    assert_eq!(after[0]["tick"], "0");
    assert_eq!(after[0]["age"], "14");
    assert_eq!(at.local["2"]["received_observations"][0]["tick"], "13");
    assert!(
        record.checkpoints[15].local["2"]["received_observations"]
            .as_array()
            .unwrap()
            .is_empty(),
        "native begin_tick resets observations"
    );
    assert!(record.checkpoints[1].local["2"]["received_observations"]
        .as_array()
        .unwrap()
        .iter()
        .any(|o| o["signal"]["visible_transfer"]["amount"] == 12.0));
}
#[test]
fn caching_deception_frozen_presets_and_exact_import() {
    for scene in [
        "deception-sham-ambiguous",
        "deception-neutral-ambiguous",
        "deception-sham-clear",
        "deception-sham-unseen",
        "deception-sham-free",
    ] {
        let record = browser::run(&browser::caching::scenes::input_for(scene).unwrap()).unwrap();
        assert_eq!(
            record,
            browser::validate_episode(&browser::episode_json(&record).unwrap()).unwrap()
        );
    }
}
#[test]
fn caching_deception_frozen_original_fingerprints() {
    for (seed, sender, first, last) in [
        (7u64, "ordinary", "08b997d646bc343d", "4274a73ad81da34a"),
        (
            7u64,
            "matched_neutral",
            "1108520b271a2e82",
            "1de2e222304b87d1",
        ),
        (7u64, "sham", "c2cee7b3d5cdb9ba", "6793d3456b0ec7a3"),
        (
            18446744073709551615u64,
            "ordinary",
            "4bf08a90dd896fec",
            "bbe22c0838190322",
        ),
        (
            18446744073709551615u64,
            "matched_neutral",
            "ad47557b8ac93b6b",
            "df54219956585d49",
        ),
        (
            18446744073709551615u64,
            "sham",
            "e43753c7e576c603",
            "fa6ceedaf4ef4077",
        ),
    ] {
        let mut raw: Value = serde_json::from_str(P4_JSON).unwrap();
        raw["seed"] = json!(seed.to_string());
        raw["lab"]["sender"] = json!(sender);
        let record = browser::run(&browser::normalize_input(&raw.to_string()).unwrap()).unwrap();
        assert_eq!(
            record.checkpoints[0].researcher.as_ref().unwrap()["frame"]["fingerprint"],
            first
        );
        assert_eq!(
            record.checkpoints[64].researcher.as_ref().unwrap()["frame"]["fingerprint"],
            last
        );
    }
}
#[test]
fn caching_deception_privileged_sentinels_do_not_change_local_projection() {
    let input = input(json!({}));
    let browser::Input::DeceptionGestures { lab, seed } = &input else {
        unreachable!()
    };
    let native =
        crate::minds::deception::run_episode(lab.to_core().unwrap(), seed.value(), true).unwrap();
    for original in native.frames {
        let before = browser::caching::deception::checkpoint(
            &original,
            original.tick == 0,
            original.tick == 64,
        )
        .unwrap();
        let mut contrast = original.clone();
        for o in &mut contrast.observations {
            o.actual_stock = 876543.25;
            o.actual_transfer = 876544.25;
        }
        for a in &mut contrast.actions {
            a.target_occupant = Some(u64::MAX);
            a.source_recovered = !a.source_recovered;
            a.cancellation = Some(crate::minds::deception::controller::BoutResult::Occupied);
        }
        for c in &mut contrast.choices {
            c.actual_value = 876545.25;
            c.remembered_value = 876546.25;
            c.inspected_stock = Some(876547.25);
        }
        for s in &mut contrast.stocks {
            s.amount = 876548.25;
        }
        let after = browser::caching::deception::checkpoint(
            &contrast,
            contrast.tick == 0,
            contrast.tick == 64,
        )
        .unwrap();
        assert_eq!(
            before.local, after.local,
            "physical researcher diagnostics must not decorate receiver state"
        );
        assert_eq!(before.public, after.public);
        if !original.roles.is_empty() {
            contrast.roles.remove(0);
            let absent = browser::caching::deception::checkpoint(
                &contrast,
                contrast.tick == 0,
                contrast.tick == 64,
            )
            .unwrap();
            assert!(!absent.local.contains_key(&original.roles[0].id.to_string()));
        }
    }
}
#[test]
fn caching_deception_native_owner_death_never_creates_local_state() {
    let record = browser::run(&input(json!({}))).unwrap();
    let absent = record
        .checkpoints
        .iter()
        .position(|at| !at.local.contains_key("1"))
        .expect("native owner dies in default rig");
    assert!(record.checkpoints[absent..]
        .iter()
        .all(|at| !at.local.contains_key("1")));
    assert!(
        record
            .checkpoints
            .iter()
            .all(|at| at.local.contains_key("2")),
        "fixed observer survives"
    );
    assert_eq!(
        record.checkpoints.last().unwrap().clock["native_tick"],
        "64"
    );
}
#[test]
fn caching_deception_import_rejects_changed_signal_and_frame() {
    let record = browser::run(&input(json!({}))).unwrap();
    for selector in 0..3 {
        let mut edited = record.clone();
        match selector {
            0 => {
                edited.checkpoints[14].local.get_mut("2").unwrap()["received_observations"][0]
                    ["signal"] = json!({"visible_transfer":{"amount":0.0}})
            }
            1 => edited.payload["native"]["seed"] = json!("8"),
            _ => {
                edited.checkpoints[14].researcher.as_mut().unwrap()["frame"]["fingerprint"] =
                    json!("0")
            }
        }
        assert!(browser::validate_episode(&browser::episode_json(&edited).unwrap()).is_err());
    }
}
