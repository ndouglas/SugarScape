use super::super::{self as browser, spatial::scenes, wire, Input, StudyId};
use crate::foraging::fixed;
use serde_json::json;

fn input(ticks: u32, sample: u32, seed_value: u64) -> Input {
    let mut input = scenes::input_for("foraging-fixed-default").unwrap();
    if let Input::ForagingFixed {
        seed,
        ticks: t,
        sample_every,
        ..
    } = &mut input
    {
        *seed = serde_json::from_value(json!(seed_value.to_string())).unwrap();
        *t = ticks;
        *sample_every = sample;
    }
    input
}

fn original(input: &Input) -> fixed::Episode {
    let Input::ForagingFixed {
        setup,
        seed,
        ticks,
        sample_every,
    } = input
    else {
        panic!()
    };
    fixed::run(
        setup.to_core().unwrap(),
        seed.value(),
        fixed::RunOptions {
            ticks: *ticks,
            sample_every: *sample_every,
            snapshots: true,
        },
    )
    .unwrap()
}

#[test]
fn complete_native_payload_and_completed_tick_boundaries_match_original_runner() {
    for seed in [12, u64::MAX] {
        for (ticks, sample) in [(20, 7), (1, 1), (8, 2), (8, 20)] {
            let input = input(ticks, sample, seed);
            let native = original(&input);
            let record = browser::run(&input).unwrap();
            assert_eq!(
                record.payload["native"],
                wire::lossless_value(&native).unwrap()
            );
            assert_eq!(record.checkpoints.len(), native.snapshots.len());
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
                    json!({"completed_ticks":snapshot.completed_ticks.to_string()})
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
            assert_eq!(
                record
                    .checkpoints
                    .iter()
                    .filter(|c| c.kind == "spatial_initial")
                    .count(),
                1
            );
            assert_eq!(
                record
                    .checkpoints
                    .iter()
                    .filter(|c| c.kind == "spatial_terminal")
                    .count(),
                1
            );
            assert_eq!(native.summary.completed_ticks, ticks);
        }
    }
}

#[test]
fn own_state_contains_only_actual_agent_view_and_supplied_geometry() {
    let mut input = input(20, 1, 12);
    if let Input::ForagingFixed { setup, .. } = &mut input {
        setup.agents = 2;
    }
    let native = original(&input);
    let record = browser::run(&input).unwrap();
    for (checkpoint, snapshot) in record.checkpoints.iter().zip(&native.snapshots) {
        assert_eq!(checkpoint.local.len(), snapshot.agents.len());
        for agent in &snapshot.agents {
            assert_eq!(
                checkpoint.local[&agent.id.to_string()],
                wire::lossless_value(&json!({"agent":agent})).unwrap()
            );
        }
        let Input::ForagingFixed { setup, .. } = &input else {
            panic!()
        };
        assert_eq!(
            checkpoint.public["arena"],
            wire::lossless_value(&json!({"width":setup.width,"height":setup.height})).unwrap()
        );
        assert_eq!(
            checkpoint.public["nest"],
            wire::lossless_value(&setup.to_core().unwrap().nest).unwrap()
        );
        assert!(checkpoint.public.get("resources").is_none());
        assert!(checkpoint.public.get("waypoints").is_none());
    }
}

#[test]
fn large_resource_and_cargo_ids_are_exact_and_missing_event_times_stay_null() {
    // Original native probe: seed 60 picks up at processing tick 8 and delivers
    // at tick 18 on this target; retain the remaining empty-world horizon too.
    let input = input(40, 1, 60);
    let native = original(&input);
    assert!(native
        .snapshots
        .iter()
        .any(|s| s.agents.iter().any(|a| a.cargo == Some(u64::MAX))));
    assert!(native.summary.all_delivered_tick.is_some());
    let record = browser::run(&input).unwrap();
    assert_eq!(
        record.payload["native"],
        wire::lossless_value(&native).unwrap()
    );
    assert_eq!(record.payload["native"]["summary"]["completed_ticks"], "40");
    assert_eq!(
        record.payload["native"]["summary"]["work"]["opportunities"],
        "40"
    );
    assert_eq!(
        record.payload["native"]["snapshots"][0]["resources"][0]["resource"]["id"],
        u64::MAX.to_string()
    );
    for (checkpoint, snapshot) in record.checkpoints.iter().zip(&native.snapshots) {
        for agent in snapshot.agents.iter().filter(|a| a.cargo.is_some()) {
            assert_eq!(
                checkpoint.local[&agent.id.to_string()]["agent"]["cargo"],
                u64::MAX.to_string()
            );
        }
    }
    let early = browser::run(&self::input(1, 1, 12)).unwrap();
    assert!(early.payload["native"]["summary"]["first_pickup_tick"].is_null());
    assert!(early.payload["native"]["summary"]["first_delivery_tick"].is_null());
    assert!(early.payload["native"]["summary"]["all_delivered_tick"].is_null());
}

#[test]
fn extra_snapshot_and_summary_calls_leave_next_original_world_boundary_unchanged() {
    let Input::ForagingFixed { setup, .. } = input(20, 7, 12) else {
        panic!()
    };
    let mut plain = fixed::World::new(setup.to_core().unwrap(), 12).unwrap();
    let mut observed = fixed::World::new(setup.to_core().unwrap(), 12).unwrap();
    for _ in 0..20 {
        let _ = observed.snapshot().unwrap();
        let _ = observed.summary().unwrap();
        plain.step().unwrap();
        observed.step().unwrap();
        assert_eq!(plain.snapshot().unwrap(), observed.snapshot().unwrap());
    }
    assert_eq!(plain.summary().unwrap(), observed.summary().unwrap());
}

#[test]
fn exact_same_target_import_rejects_float_native_local_and_foreign_target_edits() {
    let record = browser::run(&input(20, 7, 12)).unwrap();
    let target = format!("{}-{}", std::env::consts::ARCH, std::env::consts::OS);
    assert!(record
        .rules_identity
        .ends_with(&format!(":target:{target}")));
    let identities: serde_json::Value =
        serde_json::from_str(include_str!("../fixtures/spatial-identities.json")).unwrap();
    assert!(record.rules_identity.contains(
        identities["foraging_fixed"]["source_sha256"]
            .as_str()
            .unwrap()
    ));
    assert_eq!(record.payload["capture"]["target"], target);
    assert_eq!(
        browser::validate_episode(&browser::episode_json(&record).unwrap()).unwrap(),
        record
    );
    for edit in 0..5 {
        let mut changed = record.clone();
        match edit {
            0 => changed.payload["native"]["seed"] = json!("13"),
            1 => changed.checkpoints[0].local.get_mut("0").unwrap()["resources"] = json!([]),
            2 => {
                let heading = changed.checkpoints[0].local["0"]["agent"]["heading"]
                    .as_f64()
                    .unwrap();
                changed.checkpoints[0].local.get_mut("0").unwrap()["agent"]["heading"] =
                    json!(f64::from_bits(heading.to_bits() + 1));
            }
            3 => {
                changed.rules_identity = changed.rules_identity.replace(&target, "foreign-unknown")
            }
            _ => {
                changed.rules_identity = format!(
                    "changed-source:{}",
                    changed.rules_identity.replace(&target, "foreign-unknown")
                )
            }
        }
        let errors =
            browser::validate_episode(&browser::episode_json(&changed).unwrap()).unwrap_err();
        if edit == 3 {
            assert!(
                errors
                    .iter()
                    .any(|e| e.field == "rules_identity" && e.message.contains("target")),
                "{errors:?}"
            );
        }
        if edit == 4 {
            assert!(
                errors.iter().any(|e| e.field == "rules_identity"
                    && e.message == "engine source or rules version mismatch"),
                "{errors:?}"
            );
        }
    }
}

#[test]
fn fixed_descriptor_is_runnable_and_preserves_browser_profile_rejection() {
    let descriptors = browser::catalog();
    assert_eq!(descriptors.len(), 11);
    let fixed = descriptors
        .iter()
        .find(|d| d.id == StudyId::ForagingFixed)
        .unwrap();
    assert!(fixed.title.contains("engineering"));
    assert!(
        browser::run(&browser::normalize_input(&fixed.default_input.to_string()).unwrap()).is_ok()
    );
    let mut oversized = input(20, 7, 12);
    if let Input::ForagingFixed { setup, .. } = &mut oversized {
        setup.agents = 17;
    }
    assert!(browser::run(&oversized)
        .unwrap_err()
        .iter()
        .any(|e| e.field == "agents"));
}
