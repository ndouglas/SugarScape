use super::super::{self as browser, spatial::scenes, wire, Input, StudyFamily, StudyId};
use crate::burrow;
use serde_json::{json, Value};

fn input(scene: &str, ticks: u32, sample: u32, seed_value: u64) -> Input {
    let mut input = scenes::input_for(scene).unwrap();
    match &mut input {
        Input::BurrowExcavation {
            seed,
            ticks: t,
            sample_every,
            ..
        }
        | Input::BurrowAccess {
            seed,
            ticks: t,
            sample_every,
            ..
        } => {
            *seed = serde_json::from_value(json!(seed_value.to_string())).unwrap();
            *t = ticks;
            *sample_every = sample;
        }
        _ => panic!(),
    }
    input
}
fn original(input: &Input) -> (burrow::Episode, Option<burrow::AccessEpisode>, Value) {
    match input {
        Input::BurrowExcavation {
            config,
            seed,
            ticks,
            sample_every,
        } => {
            let native = burrow::run_episode(
                config.to_core().unwrap(),
                seed.value(),
                burrow::RunOptions {
                    ticks: *ticks,
                    sample_every: *sample_every,
                },
            )
            .unwrap();
            let raw = serde_json::to_value(&native).unwrap();
            (native, None, raw)
        }
        Input::BurrowAccess {
            config,
            seed,
            ticks,
            sample_every,
        } => {
            let native = burrow::run_access_episode(
                config.to_core().unwrap(),
                seed.value(),
                burrow::RunOptions {
                    ticks: *ticks,
                    sample_every: *sample_every,
                },
            )
            .unwrap();
            let raw = serde_json::to_value(&native).unwrap();
            (native.episode.clone(), Some(native), raw)
        }
        _ => panic!(),
    }
}
#[test]
fn full_native_payload_and_boundaries_match_original_runs() {
    for scene in [
        "burrow-excavation-default",
        "burrow-corridor",
        "burrow-access-default",
        "burrow-access-known-goal",
    ] {
        for seed in [7, u64::MAX] {
            for (ticks, sample) in [(0, 8), (128, 16), (17, 8)] {
                let input = input(scene, ticks, sample, seed);
                let (native, access, raw) = original(&input);
                let record = browser::run(&input).unwrap();
                assert_eq!(
                    record.payload["native"],
                    wire::lossless_value(&raw).unwrap()
                );
                assert_eq!(record.checkpoints.len(), native.frames.len());
                for (index, checkpoint) in record.checkpoints.iter().enumerate() {
                    let frame = &native.frames[index];
                    let researcher = checkpoint.researcher.as_ref().unwrap();
                    assert_eq!(checkpoint.local.len(), native.worker_work.len());
                    assert_eq!(researcher["frame"], wire::lossless_value(frame).unwrap());
                    assert_eq!(checkpoint.clock["native_tick"], frame.tick.to_string());
                    assert_eq!(
                        checkpoint.clock["completed_ticks"],
                        (frame.tick - native.setup.start_tick).to_string()
                    );
                    let events: Vec<_> = native
                        .events
                        .iter()
                        .filter(|e| e.tick < frame.tick)
                        .collect();
                    let snapshot = native
                        .series
                        .iter()
                        .find(|s| s.tick == frame.tick && s.opportunities == events.len() as u64)
                        .unwrap();
                    assert_eq!(
                        researcher["snapshot"],
                        wire::lossless_value(snapshot).unwrap()
                    );
                    assert_eq!(
                        researcher["grid"]["cells"].as_array().unwrap().len(),
                        (native.setup.width * native.setup.height) as usize
                    );
                    assert!(researcher["grid"]["overlay_caveat"]
                        .as_str()
                        .unwrap()
                        .contains("overlays hide loose material"));
                    for (cell_index, glyph) in frame
                        .ascii
                        .lines()
                        .take(native.setup.height as usize)
                        .flat_map(str::bytes)
                        .enumerate()
                    {
                        let cell = &researcher["grid"]["cells"][cell_index];
                        assert_eq!(cell["glyph"], char::from(glyph).to_string());
                        assert_eq!(
                            cell["x"],
                            (cell_index % native.setup.width as usize).to_string()
                        );
                        assert_eq!(
                            cell["y"],
                            (cell_index / native.setup.width as usize).to_string()
                        );
                    }
                    for (id, local) in &checkpoint.local {
                        let worker: u32 = id.parse().unwrap();
                        let own: Vec<_> = events.iter().filter(|e| e.worker == worker).collect();
                        assert_eq!(local["events"], wire::lossless_value(&own).unwrap());
                        let choices: Vec<_> = native
                            .choices
                            .iter()
                            .filter(|c| c.worker == worker && c.tick < frame.tick)
                            .collect();
                        assert_eq!(local["choices"], wire::lossless_value(&choices).unwrap());
                        assert!(local["holdings"].is_null());
                        assert!(local["observations"].is_null());
                        assert!(local["map"].is_null());
                        assert!(local.get("structurally_accessible").is_none());
                        if let Some(access) = &access {
                            if access.config.task.objective == burrow::AccessObjective::Explore {
                                assert!(local.get("goal").is_none());
                                assert!(local.get("completion_observations").is_none());
                            } else {
                                assert_eq!(
                                    local["goal"],
                                    wire::lossless_value(&access.config.task.goal).unwrap()
                                );
                                let markers: Vec<_> = access
                                    .task_diagnostics
                                    .completion_observations
                                    .iter()
                                    .filter(|m| {
                                        m.worker == worker
                                            && m.opportunities_before < events.len() as u64
                                    })
                                    .collect();
                                assert_eq!(
                                    local["completion_observations"],
                                    wire::lossless_value(&markers).unwrap()
                                );
                            }
                        }
                    }
                }
            }
        }
    }
}
#[test]
fn choice_keeps_distinct_same_tick_stages_and_unpaid_selection() {
    let record = browser::run(&input("burrow-choice", 1, 1, 7)).unwrap();
    let (native, _, raw) = original(&input("burrow-choice", 1, 1, 7));
    assert_eq!(
        record.payload["native"],
        wire::lossless_value(&raw).unwrap()
    );
    assert_eq!(
        record.checkpoints[0].researcher.as_ref().unwrap()["snapshot"],
        wire::lossless_value(&native.series[0]).unwrap()
    );
    assert_eq!(
        record.checkpoints[1].researcher.as_ref().unwrap()["snapshot"],
        wire::lossless_value(&native.final_summary).unwrap()
    );
    assert_eq!(record.checkpoints.len(), 2);
    assert_eq!(record.checkpoints[0].clock, record.checkpoints[1].clock);
    assert_eq!(record.checkpoints[0].kind, "spatial_initial");
    assert_eq!(record.checkpoints[1].kind, "spatial_terminal");
    assert_eq!(record.checkpoints[0].local["0"]["choices"], json!([]));
    assert_eq!(
        record.checkpoints[1].local["0"]["choices"],
        record.payload["native"]["choices"]
    );
    assert_eq!(record.payload["native"]["events"], json!([]));
    assert_eq!(
        record.payload["native"]["rates"],
        json!({"excavation":null,"disposal":null})
    );
}
#[test]
fn zero_tick_choice_uses_original_runner_and_fresh_reconstruction_authority() {
    let record = browser::run(&input("burrow-choice", 0, 1, 7)).unwrap();
    assert_eq!(record.checkpoints.len(), 1);
    assert_eq!(record.checkpoints[0].local["0"]["choices"], json!([]));
    assert_eq!(
        browser::validate_episode(&browser::episode_json(&record).unwrap()).unwrap(),
        record
    );
}
#[test]
fn exact_reconstruction_rejects_edited_native_and_local_claims() {
    let record = browser::run(&input("burrow-access-known-goal", 17, 8, 7)).unwrap();
    for selector in 0..3 {
        let mut altered = record.clone();
        match selector {
            0 => altered.payload["native"]["episode"]["seed"] = json!("8"),
            1 => altered.checkpoints[0].local.get_mut("0").unwrap()["holdings"] = json!("1"),
            _ => {
                altered.checkpoints[0].researcher.as_mut().unwrap()["frame"]["fingerprint"] =
                    json!("0")
            }
        }
        assert!(browser::validate_episode(&browser::episode_json(&altered).unwrap()).is_err());
    }
}
#[test]
fn catalog_registers_only_implemented_spatial_adapters() {
    assert_eq!(browser::catalog().len(), 13);
    let spatial: Vec<_> = browser::catalog()
        .into_iter()
        .filter(|d| d.family == StudyFamily::Spatial)
        .collect();
    assert_eq!(spatial.len(), 5);
    assert_eq!(
        spatial.iter().map(|d| d.id).collect::<Vec<_>>(),
        [
            StudyId::BurrowExcavation,
            StudyId::BurrowAccess,
            StudyId::ForagingFixed,
            StudyId::ForagingPassage,
            StudyId::ForagingConstruction
        ]
    );
    for descriptor in spatial {
        assert!(descriptor.title.contains("engineering"));
        let input = browser::normalize_input(&descriptor.default_input.to_string()).unwrap();
        assert!(browser::run(&input).is_ok());
        assert!(descriptor.controls["sample_every"].is_object());
    }
}
#[test]
fn native_event_outcomes_material_ids_and_null_access_distance_are_preserved() {
    let mut waited = false;
    let mut material = false;
    for scene in ["burrow-excavation-default", "burrow-corridor"] {
        let input = input(scene, 128, 16, 7);
        let (native, _, _) = original(&input);
        let record = browser::run(&input).unwrap();
        for (i, event) in native.events.iter().enumerate() {
            let projected = &record.payload["native"]["events"][i];
            assert_eq!(*projected, wire::lossless_value(event).unwrap());
            waited |= event.action == burrow::Action::Wait;
            if let Some(id) = event.material {
                material = true;
                assert_eq!(projected["material"], id.to_string());
            }
        }
    }
    assert!(waited && material);
    let record = browser::run(&input("burrow-access-default", 0, 8, 7)).unwrap();
    assert!(record.payload["native"]["access"]["final_exit_distance"].is_null());
}
#[test]
fn completion_at_equal_opportunity_clock_is_private_to_next_sample() {
    let mut input = input("burrow-access-known-goal", 64, 1, 7);
    if let Input::BurrowAccess { config, .. } = &mut input {
        config.lab.fixture = burrow::Fixture::Growing {
            width: 41,
            height: 25,
            workers: 1,
        };
        config.task.goal = burrow::Pos { x: 0, y: 9 };
    }
    let (native, access, _) = original(&input);
    let access = access.unwrap();
    let marker = access
        .task_diagnostics
        .completion_observations
        .first()
        .expect("fixture must reach and locally observe the adjacent goal");
    assert_eq!(marker.tick, marker.opportunities_before);
    let record = browser::run(&input).unwrap();
    let marker_tick = marker.tick.to_string();
    let next_tick = (marker.tick + 1).to_string();
    let before = record
        .checkpoints
        .iter()
        .find(|c| c.clock["native_tick"].as_str() == Some(marker_tick.as_str()))
        .unwrap();
    let after = record
        .checkpoints
        .iter()
        .find(|c| c.clock["native_tick"].as_str() == Some(next_tick.as_str()))
        .unwrap();
    assert_eq!(before.local["0"]["completion_observations"], json!([]));
    assert_eq!(
        after.local["0"]["completion_observations"],
        wire::lossless_value(&vec![marker]).unwrap()
    );
    assert_eq!(
        native.series[marker.tick as usize].opportunities,
        marker.opportunities_before
    );
}
#[test]
#[ignore = "requires retained Task 1 original CLI export directory via BURROW_BASELINE_DIR"]
fn retained_native_baselines_match_raw_before_wire_projection() {
    let root = std::path::PathBuf::from(std::env::var("BURROW_BASELINE_DIR").unwrap());
    for name in [
        "direct-blind",
        "direct-responsive",
        "relay-blind",
        "relay-responsive",
        "access-explore",
        "access-known-goal",
    ] {
        for seed in [7, u64::MAX] {
            let dir = root.join(format!("{name}-{seed}"));
            let config: Value =
                serde_json::from_slice(&std::fs::read(dir.join("config.json")).unwrap()).unwrap();
            let access = name.starts_with("access-");
            let mut browser_config = config.clone();
            let lab = if access {
                &mut browser_config["lab"]
            } else {
                &mut browser_config
            };
            lab["freshness_window"] = json!(lab["freshness_window"].as_u64().unwrap().to_string());
            let input = browser::normalize_input(&json!({"study":if access {"burrow_access"} else {"burrow_excavation"},"config":browser_config,"seed":seed.to_string(),"ticks":128,"sample_every":16}).to_string()).unwrap();
            let (_, _, raw) = original(&input);
            let baseline_bytes = std::fs::read(dir.join("episode.json")).unwrap();
            let baseline: Value = serde_json::from_slice(&baseline_bytes).unwrap();
            assert_eq!(raw, baseline, "raw native output {name}/{seed}");
            let record = browser::run(&input).unwrap();
            assert_eq!(
                record.payload["native"],
                wire::lossless_value(&raw).unwrap(),
                "wire output {name}/{seed}"
            );
        }
    }
}
