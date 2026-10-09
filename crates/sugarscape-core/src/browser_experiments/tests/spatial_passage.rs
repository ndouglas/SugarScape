use super::super::{self as browser, spatial::scenes, wire, Input};
use crate::foraging::passage as native;
use serde_json::json;

fn input(scene: &str, ticks: u32, sample: u32, value: u64) -> Input {
    let mut input = scenes::input_for(scene).unwrap();
    if let Input::ForagingPassage {
        seed,
        ticks: t,
        sample_every,
        ..
    } = &mut input
    {
        *seed = serde_json::from_value(json!(value.to_string())).unwrap();
        *t = ticks;
        *sample_every = sample;
    } else {
        panic!()
    }
    input
}
fn original(input: &Input) -> native::Episode {
    let Input::ForagingPassage {
        setup,
        seed,
        ticks,
        sample_every,
    } = input
    else {
        panic!()
    };
    native::run(
        setup.to_core().unwrap(),
        seed.value(),
        native::RunOptions {
            ticks: *ticks,
            sample_every: *sample_every,
            snapshots: true,
        },
    )
    .unwrap()
}
#[test]
fn complete_native_episode_normalization_and_sampling_match_original_runner() {
    for seed in [7, u64::MAX] {
        for (ticks, sample) in [(40, 7), (8, 2), (8, 20), (1, 1)] {
            let input = input("foraging-passage-default", ticks, sample, seed);
            let native = original(&input);
            let record = browser::run(&input).unwrap();
            assert_eq!(
                record.payload["native"],
                wire::lossless_value(&native).unwrap()
            );
            assert_eq!(record.checkpoints.len(), native.snapshots.len());
            assert_eq!(
                native.snapshot_bytes,
                native
                    .snapshots
                    .iter()
                    .map(|s| serde_json::to_vec(s).unwrap().len() as u64)
                    .sum::<u64>()
            );
            assert_eq!(
                record.payload["capture"]["sampling"],
                json!({"ticks":ticks,"sample_every":sample})
            );
            assert_eq!(
                record.payload["capture"]["clock_semantics"],
                "completed_tick_boundary"
            );
            for (index, (checkpoint, snapshot)) in
                record.checkpoints.iter().zip(&native.snapshots).enumerate()
            {
                assert_eq!(
                    checkpoint.clock,
                    json!({"completed_ticks":snapshot.summary.completed_ticks.to_string()})
                );
                assert_eq!(checkpoint.index, index as u32);
                assert_eq!(
                    checkpoint.kind,
                    if index == 0 {
                        "spatial_initial"
                    } else if index + 1 == native.snapshots.len() {
                        "spatial_terminal"
                    } else {
                        "spatial_sample"
                    }
                );
                assert_eq!(
                    checkpoint.researcher.as_ref().unwrap()["snapshot"],
                    wire::lossless_value(snapshot).unwrap()
                );
            }
        }
    }
}
#[test]
fn every_agent_private_map_matches_original_world_at_each_boundary() {
    let input = input("foraging-passage-default", 40, 7, 7);
    let record = browser::run(&input).unwrap();
    let Input::ForagingPassage { setup, seed, .. } = &input else {
        panic!()
    };
    let mut world = native::World::new(setup.to_core().unwrap(), seed.value()).unwrap();
    let mut completed = 0;
    for checkpoint in &record.checkpoints {
        let at = checkpoint.clock["completed_ticks"]
            .as_str()
            .unwrap()
            .parse::<u32>()
            .unwrap();
        while completed < at {
            world.step().unwrap();
            completed += 1;
        }
        let snapshot = world.snapshot().unwrap();
        assert_eq!(checkpoint.local.len(), snapshot.agents.len());
        for agent in &snapshot.agents {
            let local = &checkpoint.local[&agent.id.to_string()];
            assert_eq!(local["agent"], wire::lossless_value(agent).unwrap());
            assert_eq!(
                local["knowledge"],
                wire::lossless_value(&world.knowledge(agent.id).unwrap()).unwrap()
            );
            assert_eq!(local["capture_at"], checkpoint.clock);
            assert_eq!(local["cell_observed_at"], serde_json::Value::Null);
            assert_eq!(local.as_object().unwrap().len(), 5);
        }
        assert!(checkpoint.public.get("open").is_none());
        assert!(checkpoint.public.get("food").is_none());
        assert!(checkpoint.public.get("resources").is_none());
        assert!(checkpoint.public.get("diggable").is_none());
    }
}
#[test]
fn original_setup_normalizes_geometry_resources_but_preserves_worker_identity() {
    let mut input = input("foraging-passage-information", 8, 2, 7);
    if let Input::ForagingPassage { setup, .. } = &mut input {
        setup.open.reverse();
        setup.nest.reverse();
        setup.resources.reverse();
        setup.workers = vec![setup.nest[0], setup.nest[0]];
    }
    let native = original(&input);
    let record = browser::run(&input).unwrap();
    assert_eq!(
        record.payload["native"],
        wire::lossless_value(&native).unwrap()
    );
    assert_eq!(
        record.checkpoints[0].public["nest"],
        wire::lossless_value(&native.setup.nest).unwrap()
    );
    assert_eq!(native.setup.workers[0], native.setup.workers[1]);
}
#[test]
fn empty_food_keeps_missing_milestones_null_and_runs_full_horizon() {
    let mut input = input("foraging-passage-default", 8, 2, 7);
    if let Input::ForagingPassage { setup, .. } = &mut input {
        setup.resources.clear();
    }
    let native = original(&input);
    let record = browser::run(&input).unwrap();
    assert_eq!(
        record.payload["native"],
        wire::lossless_value(&native).unwrap()
    );
    assert_eq!(record.payload["native"]["summary"]["completed_ticks"], "8");
    for key in [
        "first_pickup_tick",
        "first_delivery_tick",
        "all_delivered_tick",
    ] {
        assert!(record.payload["native"]["summary"][key].is_null());
    }
}
#[test]
fn same_target_reconstruction_is_exact_and_rejects_private_map_and_target_edits() {
    let record = browser::run(&input("foraging-passage-information", 8, 2, 7)).unwrap();
    assert_eq!(
        browser::validate_episode(&browser::episode_json(&record).unwrap()).unwrap(),
        record
    );
    let target = format!("{}-{}", std::env::consts::ARCH, std::env::consts::OS);
    assert!(record
        .rules_identity
        .ends_with(&format!(":target:{target}")));
    assert_eq!(record.payload["capture"]["target"], target);
    for edit in 0..3 {
        let mut changed = record.clone();
        match edit {
            0 => {
                changed.checkpoints[0].local.get_mut("0").unwrap()["knowledge"]["cells"] = json!([])
            }
            1 => {
                let x = changed.payload["native"]["setup"]["parameters"]["lambda_publish"]
                    .as_f64()
                    .unwrap();
                changed.payload["native"]["setup"]["parameters"]["lambda_publish"] =
                    json!(f64::from_bits(x.to_bits() + 1));
            }
            _ => changed.rules_identity = changed.rules_identity.replace(&target, "foreign-target"),
        }
        let errors =
            browser::validate_episode(&browser::episode_json(&changed).unwrap()).unwrap_err();
        if edit == 2 {
            assert!(errors
                .iter()
                .any(|e| e.field == "rules_identity" && e.message.contains("target")));
        }
    }
}

#[test]
fn extra_adapter_captures_preserve_private_maps_counters_and_future_continuation() {
    let Input::ForagingPassage { setup, .. } = input("foraging-passage-information", 40, 1, 12)
    else {
        panic!()
    };
    let mut plain = native::World::new(setup.to_core().unwrap(), 12).unwrap();
    let mut observed = plain.clone();
    for _ in 0..40 {
        let snapshot = observed.snapshot().unwrap();
        let checkpoint = browser::spatial::passage::adapter_checkpoint(&observed).unwrap();
        assert_eq!(observed.snapshot().unwrap(), snapshot);
        for agent in &snapshot.agents {
            assert_eq!(
                checkpoint.local[&agent.id.to_string()]["knowledge"],
                wire::lossless_value(&observed.knowledge(agent.id).unwrap()).unwrap()
            );
        }
        plain.step().unwrap();
        observed.step().unwrap();
        assert_eq!(plain.snapshot().unwrap(), observed.snapshot().unwrap());
        for agent in &snapshot.agents {
            assert_eq!(
                plain.knowledge(agent.id).unwrap(),
                observed.knowledge(agent.id).unwrap()
            );
        }
    }
    assert_eq!(plain.summary().unwrap(), observed.summary().unwrap());
}

#[test]
fn sixteen_agents_each_retain_their_own_original_map() {
    let mut input = input("foraging-passage-default", 8, 2, 12);
    if let Input::ForagingPassage { setup, .. } = &mut input {
        setup.width = 9;
        setup.height = 3;
        setup.nest = (0..8)
            .map(|x| browser::spatial::input::PositionInput { x, y: 0 })
            .collect();
        setup.open = setup.nest.clone();

        setup.workers = setup.nest.iter().flat_map(|p| [*p, *p]).collect();
        setup.resources.clear();
    }
    let record = browser::run(&input).unwrap();
    let Input::ForagingPassage { setup, .. } = &input else {
        panic!()
    };
    let mut world = native::World::new(setup.to_core().unwrap(), 12).unwrap();
    for (index, checkpoint) in record.checkpoints.iter().enumerate() {
        if index > 0 {
            world.step().unwrap();
            world.step().unwrap();
        }
        assert_eq!(checkpoint.local.len(), 16);
        for id in 0..16 {
            assert_eq!(
                checkpoint.local[&id.to_string()]["knowledge"],
                wire::lossless_value(&world.knowledge(id).unwrap()).unwrap()
            );
        }
    }
}
#[test]
fn checkpoint_budget_includes_actual_keyed_local_wire_entries() {
    let Input::ForagingPassage { setup, .. } = input("foraging-passage-default", 8, 2, 12) else {
        panic!()
    };
    let world = native::World::new(setup.to_core().unwrap(), 12).unwrap();
    let snapshot = world.snapshot().unwrap();
    let mut budget = browser::spatial::budget::CaptureBudget::new();
    let before = budget.used_bytes();
    let checkpoint = browser::spatial::passage::checkpoint_from_snapshot(
        &world,
        &snapshot,
        "spatial_sample",
        0,
        &serde_json::Value::Null,
        &mut budget,
    )
    .unwrap();
    let mut skeleton = checkpoint.clone();
    skeleton.local.clear();
    let expected = serde_json::to_vec(&skeleton).unwrap().len()
        + 2
        + checkpoint
            .local
            .iter()
            .map(|(id, local)| {
                serde_json::to_vec(&std::collections::BTreeMap::from([(id, local)]))
                    .unwrap()
                    .len()
                    + 2
            })
            .sum::<usize>();
    assert_eq!(budget.used_bytes() - before, expected);
    assert!(expected >= serde_json::to_vec(&checkpoint).unwrap().len() + 2);
}

#[test]
fn disconnected_exposed_food_remains_uncollected_and_unknown_locally() {
    let input = input("foraging-passage-disconnected", 40, 7, 12);
    let native = original(&input);
    assert_eq!(native.summary.inventory.available, 1);
    assert!(native.summary.first_pickup_tick.is_none());
    let record = browser::run(&input).unwrap();
    assert_eq!(
        record.payload["native"],
        wire::lossless_value(&native).unwrap()
    );
    for checkpoint in &record.checkpoints {
        assert!(!checkpoint.local["0"]["knowledge"]["cells"]
            .as_array()
            .unwrap()
            .iter()
            .any(|c| c["pos"] == json!({"x":"3","y":"3"})));
    }
}
