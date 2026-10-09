//! Shared integration-test policy, compiled independently for native and WASM.
//! This module defines inputs/oracles; each bridge retains its own execution and assertions.
use serde_json::{json, Value};
use sugarscape_core::browser_experiments as core;

pub(super) fn spatial_defaults() -> Vec<Value> {
    core::catalog()
        .into_iter()
        .filter(|descriptor| descriptor.family == core::StudyFamily::Spatial)
        .map(|descriptor| descriptor.default_input)
        .collect()
}

// Recompute original numeric Snapshot JSON size on this actual compilation target.
// Wire integer quoting is deliberately not substituted for native snapshot_bytes.
// Returns (reported, independently counted); each target wrapper asserts equality.
pub(super) fn checked_snapshot_bytes(text: &str) -> Option<(u64, u64)> {
    use core::Input;
    use sugarscape_core::foraging::{construction, fixed, passage};
    match core::normalize_input(text).unwrap() {
        Input::ForagingFixed {
            setup,
            seed,
            ticks,
            sample_every,
        } => {
            let episode = fixed::run(
                setup.to_core().unwrap(),
                seed.value(),
                fixed::RunOptions {
                    ticks,
                    sample_every,
                    snapshots: true,
                },
            )
            .unwrap();
            let bytes = episode
                .snapshots
                .iter()
                .map(|snapshot| serde_json::to_vec(snapshot).unwrap().len() as u64)
                .sum();
            Some((episode.snapshot_bytes, bytes))
        }
        Input::ForagingPassage {
            setup,
            seed,
            ticks,
            sample_every,
        } => {
            let episode = passage::run(
                setup.to_core().unwrap(),
                seed.value(),
                passage::RunOptions {
                    ticks,
                    sample_every,
                    snapshots: true,
                },
            )
            .unwrap();
            let bytes = episode
                .snapshots
                .iter()
                .map(|snapshot| serde_json::to_vec(snapshot).unwrap().len() as u64)
                .sum();
            Some((episode.snapshot_bytes, bytes))
        }
        Input::ForagingConstruction {
            setup,
            seed,
            ticks,
            sample_every,
        } => {
            let episode = construction::run(
                setup.to_core().unwrap(),
                seed.value(),
                construction::RunOptions {
                    ticks,
                    sample_every,
                    snapshots: true,
                },
            )
            .unwrap();
            let bytes = episode
                .snapshots
                .iter()
                .map(|snapshot| serde_json::to_vec(snapshot).unwrap().len() as u64)
                .sum();
            Some((episode.snapshot_bytes, bytes))
        }
        _ => None,
    }
}

pub(super) fn default_seed_cases() -> Vec<Value> {
    let mut inputs = Vec::new();
    for input in spatial_defaults() {
        inputs.push(input.clone());
        let mut max = input;
        max["seed"] = json!(u64::MAX.to_string());
        inputs.push(max);
    }
    inputs
}

pub(super) fn invalid_spatial_inputs(input: &Value) -> Vec<(Value, &'static str)> {
    let mut cases = Vec::new();
    let mut bad = input.clone();
    bad.as_object_mut().unwrap().remove("sample_every");
    cases.push((bad, "input"));
    let mut bad = input.clone();
    bad["unexpected"] = json!(true);
    cases.push((bad, "input"));
    let mut bad = input.clone();
    bad["seed"] = json!("18446744073709551616");
    cases.push((bad, "input"));
    let mut bad = input.clone();
    bad["sample_every"] = json!(0);
    cases.push((bad, "sample_every"));
    let mut bad = input.clone();
    bad["ticks"] = json!(u32::MAX);
    cases.push((
        bad,
        if input["setup"].is_object() {
            "ticks"
        } else {
            "opportunities"
        },
    ));
    if input["setup"].is_object() {
        let mut bad = input.clone();
        bad["setup"]["width"] = json!(65);
        bad["setup"]["height"] = json!(65);
        cases.push((bad, "cells"));
        let mut bad = input.clone();
        let resources = if input["study"] == "foraging_construction" {
            "food"
        } else {
            "resources"
        };
        bad["setup"][resources][0]["id"] = json!(18446744073709551615_u64);
        cases.push((bad, "input"));
        let mut bad = input.clone();
        bad["setup"][resources][0]["id"] = json!("18446744073709551616");
        cases.push((bad, "input"));
        let mut bad = input.clone();
        bad["setup"]["parameters"]["p_search"] = json!(2.0);
        cases.push((bad, "parameters.p_search"));
    }
    cases
}

pub(super) fn spatial_saved_mutations(input: &Value, actual: &Value) -> Vec<(Value, &'static str)> {
    let mut cases = Vec::new();
    let mut bad = actual.clone();
    bad["payload"]["native"]["seed"] = json!("999");
    cases.push((bad, "episode"));
    let mut bad = actual.clone();
    bad["checkpoints"][0]["public"]["invented"] = json!(true);
    cases.push((bad, "episode"));
    let mut bad = actual.clone();
    bad["checkpoints"][0]["extra"] = json!(true);
    cases.push((bad, "episode"));
    let mut bad = actual.clone();
    bad["rules_identity"] = json!("wrong-source:target:foreign");
    cases.push((bad, "rules_identity"));
    let mut bad = actual.clone();
    bad["checkpoints"] = json!(vec![
        json!({"index":0,"clock":{},"kind":"spatial_initial","public":null,"local":{},"researcher":null});
        4097
    ]);
    cases.push((bad, "checkpoints"));
    if input["setup"].is_object() {
        let mut bad = actual.clone();
        let prefix = actual["rules_identity"]
            .as_str()
            .unwrap()
            .rsplit_once(":target:")
            .unwrap()
            .0;
        bad["rules_identity"] = json!(format!("{prefix}:target:foreign-os"));
        cases.push((bad, "rules_identity"));
        let mut bad = actual.clone();
        bad["payload"]["capture"]["target"] = json!("foreign-os");
        cases.push((bad, "episode"));
        let mut bad = actual.clone();
        bad["payload"]["native"]["snapshot_bytes"] = json!("0");
        cases.push((bad, "episode"));
        if input["study"] == "foraging_fixed" {
            let mut bad = actual.clone();
            let heading = bad["payload"]["native"]["snapshots"][0]["agents"][0]["heading"]
                .as_f64()
                .unwrap();
            bad["payload"]["native"]["snapshots"][0]["agents"][0]["heading"] =
                json!(heading + 1e-13);
            cases.push((bad, "episode"));
        }
    }
    cases
}

pub(super) fn spatial_snapshot_inputs() -> Vec<Value> {
    let scenes: Value = serde_json::from_str(include_str!(
        "../sugarscape-core/src/browser_experiments/fixtures/spatial-scenes.json"
    ))
    .unwrap();
    let mut inputs: Vec<Value> = scenes["scenes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|scene| scene["input"].clone())
        .collect();
    for input in spatial_defaults()
        .into_iter()
        .filter(|input| input["setup"].is_object())
    {
        let mut off = input.clone();
        off["ticks"] = json!(8);
        off["sample_every"] = json!(20);
        inputs.push(off);
        let mut expensive = input.clone();
        let fixed = input["study"] == "foraging_fixed";
        expensive["setup"]["width"] = json!(if fixed { 32 } else { 64 });
        expensive["setup"]["height"] = expensive["setup"]["width"].clone();
        if fixed {
            expensive["setup"]["agents"] = json!(16);
        }
        expensive["ticks"] = json!(if fixed { 512 } else { 7200 });
        expensive["sample_every"] = expensive["ticks"].clone();
        inputs.push(expensive);
    }
    inputs
        .into_iter()
        .filter(|input| input["setup"].is_object())
        .collect()
}
