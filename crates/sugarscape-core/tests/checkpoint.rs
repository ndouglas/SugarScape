//! Keyframes: a world restored from a checkpoint and run on is the world
//! that ran straight there, for every model.

use sugarscape_core::model::ModelWorld;
use sugarscape_core::presets;

/// One preset per model kind, and a second for the spatial games' asynchronous updating.
/// Image scoring's is AND strategies with observers (private records, mutation).
/// Minds 3's `mem-truffles` checks that memory and truffle spots come back too.
/// Minds 5's `cache-raby` has its test evening at tick 24, inside the 20 → 60
/// window (walls opened by `open_wall`, frozen allocations), and
/// `central-near` checks homes and loads.
const IDS: &[&str] = &[
    "vi-1-everything",
    "vi-4-schelling-25",
    "vi-8-ring-world",
    "lhv-published",
    "cv-run-8-nasty-regime",
    "nbm-probabilistic",
    "hg-async-kaleidoscope",
    "jansson-kin",
    "dpd-rr-best",
    "ns-fig-4b",
    "mem-truffles",
    "cache-raby",
    "central-near",
    "theft-winter-half",
    "spatial-larder-guard",
];

fn world(id: &str) -> ModelWorld {
    let preset = presets::find(id).unwrap_or_else(|| panic!("unknown preset {id}"));
    ModelWorld::new(preset.config, 1).unwrap()
}

/// Every series as bits, so an undefined statistic (NaN) compares equal to itself.
fn all_series(w: &ModelWorld) -> Vec<Option<Vec<u64>>> {
    let m = w.model();
    let mut names = m.series_names();
    names.push("tick".into());
    names
        .iter()
        .map(|n| {
            m.series(n)
                .map(|v| v.into_iter().map(f64::to_bits).collect())
        })
        .collect()
}

#[test]
fn restore_then_run_equals_a_straight_run() {
    for &id in IDS {
        let mut straight = world(id);
        straight.model_mut().run(60);

        let mut w = world(id);
        w.model_mut().run(20);
        let cp = w.checkpoint().expect("every current model has keyframes");
        assert_eq!(cp.tick(), 20);
        w.model_mut().run(25);
        w.restore(&cp).unwrap();
        assert_eq!(w.model().tick(), 20, "{id}");
        assert_eq!(
            w.model().series("tick").unwrap().len(),
            21,
            "{id}: history cut to the keyframe"
        );
        w.model_mut().run(40);

        assert_eq!(
            w.model().fingerprint(),
            straight.model().fingerprint(),
            "{id}"
        );
        assert_eq!(all_series(&w), all_series(&straight), "{id}");
    }
}

/// Inspect at every cell of an image world's frame.
fn inspect_all(w: &ModelWorld) -> Vec<String> {
    let m = w.model();
    let (width, height) = m.size();
    let mut out = Vec::new();
    for y in 0..height {
        for x in 0..width {
            out.push(m.inspect_json(x, y).unwrap());
        }
    }
    out
}

/// Image keyframes leave out the private records (the next generation
/// rebuilds them) but keep what Inspect shows of them, so a restored world
/// inspects, and runs on, as the straight run does.
#[test]
fn an_image_world_restored_without_its_records_inspects_and_runs_on_unchanged() {
    for id in ["ns-fig-4b", "ns-fig-3-n20"] {
        let mut straight = world(id);
        straight.model_mut().run(20);
        let at_keyframe = inspect_all(&straight);
        assert!(
            at_keyframe
                .iter()
                .any(|v| v.contains("\"mean_view\":") && !v.contains("\"mean_view\":null")),
            "{id}: some agent has been seen"
        );

        let mut w = world(id);
        w.model_mut().run(20);
        let cp = w.checkpoint().unwrap();
        w.model_mut().run(7);
        w.restore(&cp).unwrap();
        assert_eq!(inspect_all(&w), at_keyframe, "{id}: at the keyframe");
        assert_eq!(
            w.model().fingerprint(),
            straight.model().fingerprint(),
            "{id}"
        );

        let cp = w.checkpoint().unwrap();
        w.restore(&cp).unwrap();
        assert_eq!(
            inspect_all(&w),
            at_keyframe,
            "{id}: a restored world's keyframe"
        );

        straight.model_mut().run(30);
        w.model_mut().run(30);
        assert_eq!(
            w.model().fingerprint(),
            straight.model().fingerprint(),
            "{id}"
        );
        assert_eq!(all_series(&w), all_series(&straight), "{id}");
        assert_eq!(
            inspect_all(&w),
            inspect_all(&straight),
            "{id}: after running on"
        );
    }
}

#[test]
fn a_checkpoint_leaves_the_live_world_untouched() {
    for &id in IDS {
        let mut a = world(id);
        let mut b = world(id);
        a.model_mut().run(15);
        b.model_mut().run(15);
        let _ = a.checkpoint();
        a.model_mut().run(15);
        b.model_mut().run(15);
        assert_eq!(a.model().fingerprint(), b.model().fingerprint(), "{id}");
        assert_eq!(all_series(&a), all_series(&b), "{id}");
    }
}

#[test]
fn restore_refuses_another_models_checkpoint() {
    let mut sugar = world("vi-1-everything");
    let mut ring = world("vi-8-ring-world");
    let cp = ring.checkpoint().unwrap();
    assert!(sugar.restore(&cp).is_err());
}

#[test]
fn latest_value_is_the_last_element_of_every_series() {
    for &id in IDS {
        let mut w = world(id);
        w.model_mut().run(12);
        let m = w.model();
        let mut names = m.series_names();
        names.push("tick".into());
        for name in names {
            let last = m.series(&name).and_then(|v| v.last().copied());
            let got = m.latest_value(&name);
            // NaN (an undefined statistic) compares by bits.
            assert_eq!(
                got.map(f64::to_bits),
                last.map(f64::to_bits),
                "{id}: {name}"
            );
        }
        assert_eq!(m.latest_value("no-such-series"), None, "{id}");
    }
}

use sugarscape_core::config::{Map, Placement, URange};
use sugarscape_core::geometry::Pos;
use sugarscape_core::minds::spatial_hoarding::state::{
    Delivery, EpisodeProbe, FounderTraits, SeenLarder,
};
use sugarscape_core::stats::Snapshot;
use sugarscape_core::world::World;

fn spatial_fixture() -> World {
    let sugarscape_core::model::ModelConfig::Sugarscape(mut c) =
        presets::find("spatial-larder-guard").unwrap().config
    else {
        panic!("sugarscape preset")
    };
    c.width = 9;
    c.height = 9;
    c.population = 2;
    c.placement = Placement::Block {
        x: 4,
        y: 4,
        width: 2,
        height: 1,
    };
    c.vision = URange::new(1, 1);
    c.goods[0].metabolism = URange::new(1, 1);
    c.goods[0].map = Map::Flat { capacity: 0.0 };
    c.watching.on = true;
    c.theft.find = 0.0;
    c.spatial_hoarding.find_larder = 0.0;
    World::new_with_spatial_probe(
        c,
        7,
        &[
            FounderTraits {
                larder: 1.0,
                defense: 0.5,
                cheater: false,
                watches: true,
            },
            FounderTraits {
                larder: 0.5,
                defense: 0.25,
                cheater: true,
                watches: true,
            },
        ],
        EpisodeProbe {
            guard_harvest: true,
            scatter_first: true,
        },
    )
    .unwrap()
}

fn sugar(w: &ModelWorld) -> &World {
    let ModelWorld::Sugarscape(w) = w else {
        panic!("sugarscape fixture")
    };
    w
}

fn compare_spatial(a: &ModelWorld, b: &ModelWorld) {
    assert_eq!(a.model().fingerprint(), b.model().fingerprint());
    assert_eq!(a.model().latest_json(), b.model().latest_json());
    assert_eq!(
        serde_json::to_string(&Snapshot::of(sugar(a))).unwrap(),
        serde_json::to_string(&Snapshot::of(sugar(b))).unwrap()
    );
    assert_eq!(all_series(a), all_series(b));
    assert_eq!(inspect_all(a), inspect_all(b));
    assert_eq!(a.model().agents_csv(), b.model().agents_csv());
    assert_eq!(sugar(a).spatial_probe, sugar(b).spatial_probe);
    for id in sugar(a).agents().map(|a| a.id) {
        let aa = sugar(a).agent(id).unwrap();
        let bb = sugar(b).agent(id).unwrap();
        assert_eq!(aa.spatial, bb.spatial);
        assert_eq!(aa.caches, bb.caches);
        assert_eq!(aa.cache_since, bb.cache_since);
        assert_eq!(aa.seen, bb.seen);
        assert_eq!(aa.holdings, bb.holdings);
    }
}

#[test]
fn spatial_delivery_guard_and_fresh_dead_owner_sighting_restore_exactly() {
    for fixture in ["delivery", "guard", "sighting"] {
        let mut w = spatial_fixture();
        let ids: Vec<_> = w.agents().map(|a| a.id).collect();
        w.agent_mut(ids[0]).unwrap().caches.insert(0, 3.0);
        w.agent_mut(ids[0]).unwrap().cache_since.insert(0, 0);
        let home = w.agent(ids[1]).unwrap().spatial.as_ref().unwrap().home;
        match fixture {
            "delivery" => {
                let a = w.agent_mut(ids[0]).unwrap();
                a.holdings[0] = 20.0;
                a.spatial.as_mut().unwrap().home = Pos::new(1, 1);
                a.spatial.as_mut().unwrap().delivery = Some(Delivery { amount: 8.0 });
            }
            "guard" => {
                let a = w.agent_mut(ids[0]).unwrap();
                a.holdings[0] = 30.0;
                let s = a.spatial.as_mut().unwrap();
                s.larder = 10.0;
                s.larder_since = Some(0);
                s.guarding = true;
            }
            "sighting" => {
                // Recorded coordinates must survive the owner's removal.
                w.agent_mut(ids[0])
                    .unwrap()
                    .spatial
                    .as_mut()
                    .unwrap()
                    .seen_larders
                    .insert(
                        ids[1],
                        SeenLarder {
                            home,
                            amount: 6.0,
                            tick: 0,
                        },
                    );
                let pos = w.agent(ids[1]).unwrap().pos;
                w.remove_agent(pos.x, pos.y).unwrap();
            }
            _ => unreachable!(),
        }
        let mut restored = ModelWorld::Sugarscape(Box::new(w.clone()));
        let mut original = ModelWorld::Sugarscape(Box::new(w));
        let cp = restored.checkpoint().unwrap();
        restored.model_mut().run(3);
        restored.restore(&cp).unwrap();
        compare_spatial(&original, &restored);
        for _ in 0..6 {
            original.model_mut().run(1);
            restored.model_mut().run(1);
            compare_spatial(&original, &restored);
        }
    }
}

#[test]
fn every_authoritative_spatial_field_changes_only_enabled_fingerprints() {
    type Mutation = (&'static str, fn(&mut World));
    let changes: &[Mutation] = &[
        ("config larder", |w| w.config.spatial_hoarding.larder = 0.3),
        ("config defense", |w| {
            w.config.spatial_hoarding.defense = 0.3
        }),
        ("config guard", |w| w.config.spatial_hoarding.guard = false),
        ("config slope", |w| {
            w.config.spatial_hoarding.defense_slope = 4.0
        }),
        ("config find", |w| {
            w.config.spatial_hoarding.find_larder = 0.2
        }),
        ("guard probe", |w| w.spatial_probe.guard_harvest = false),
        ("scatter probe", |w| w.spatial_probe.scatter_first = false),
        ("state presence", |w| w.agent_mut(1).unwrap().spatial = None),
        ("home x", |w| {
            w.agent_mut(1).unwrap().spatial.as_mut().unwrap().home.x += 1
        }),
        ("home y", |w| {
            w.agent_mut(1).unwrap().spatial.as_mut().unwrap().home.y += 1
        }),
        ("trait larder", |w| {
            w.agent_mut(1)
                .unwrap()
                .spatial
                .as_mut()
                .unwrap()
                .traits
                .larder = 0.2
        }),
        ("trait defense", |w| {
            w.agent_mut(1)
                .unwrap()
                .spatial
                .as_mut()
                .unwrap()
                .traits
                .defense = 0.2
        }),
        ("trait cheater", |w| {
            w.agent_mut(1)
                .unwrap()
                .spatial
                .as_mut()
                .unwrap()
                .traits
                .cheater = true
        }),
        ("trait watches", |w| {
            w.agent_mut(1)
                .unwrap()
                .spatial
                .as_mut()
                .unwrap()
                .traits
                .watches = false
        }),
        ("cheater", |w| w.agent_mut(1).unwrap().cheater = true),
        ("watches", |w| w.agent_mut(1).unwrap().watches = false),
        ("larder", |w| {
            w.agent_mut(1).unwrap().spatial.as_mut().unwrap().larder += 1.0
        }),
        ("age presence", |w| {
            w.agent_mut(1)
                .unwrap()
                .spatial
                .as_mut()
                .unwrap()
                .larder_since = None
        }),
        ("age tick", |w| {
            w.agent_mut(1)
                .unwrap()
                .spatial
                .as_mut()
                .unwrap()
                .larder_since = Some(1)
        }),
        ("delivery presence", |w| {
            w.agent_mut(1).unwrap().spatial.as_mut().unwrap().delivery = None
        }),
        ("delivery amount", |w| {
            w.agent_mut(1)
                .unwrap()
                .spatial
                .as_mut()
                .unwrap()
                .delivery
                .as_mut()
                .unwrap()
                .amount += 1.0
        }),
        ("guarding", |w| {
            w.agent_mut(1).unwrap().spatial.as_mut().unwrap().guarding = true
        }),
        ("sighting presence", |w| {
            w.agent_mut(1)
                .unwrap()
                .spatial
                .as_mut()
                .unwrap()
                .seen_larders
                .clear()
        }),
        ("sighting owner", |w| {
            let s = w.agent_mut(1).unwrap().spatial.as_mut().unwrap();
            let seen = s.seen_larders.remove(&99).unwrap();
            s.seen_larders.insert(98, seen);
        }),
        ("sighting home x", |w| {
            w.agent_mut(1)
                .unwrap()
                .spatial
                .as_mut()
                .unwrap()
                .seen_larders
                .get_mut(&99)
                .unwrap()
                .home
                .x += 1
        }),
        ("sighting home y", |w| {
            w.agent_mut(1)
                .unwrap()
                .spatial
                .as_mut()
                .unwrap()
                .seen_larders
                .get_mut(&99)
                .unwrap()
                .home
                .y += 1
        }),
        ("sighting amount", |w| {
            w.agent_mut(1)
                .unwrap()
                .spatial
                .as_mut()
                .unwrap()
                .seen_larders
                .get_mut(&99)
                .unwrap()
                .amount += 1.0
        }),
        ("sighting tick", |w| {
            w.agent_mut(1)
                .unwrap()
                .spatial
                .as_mut()
                .unwrap()
                .seen_larders
                .get_mut(&99)
                .unwrap()
                .tick += 1
        }),
        ("scatter age presence", |w| {
            w.agent_mut(1).unwrap().cache_since.clear()
        }),
        ("scatter age site", |w| {
            let a = w.agent_mut(1).unwrap();
            a.cache_since.remove(&0);
            a.cache_since.insert(1, 0);
        }),
        ("scatter age tick", |w| {
            w.agent_mut(1).unwrap().cache_since.insert(0, 1);
        }),
        ("scatter seen presence", |w| {
            w.agent_mut(1).unwrap().seen.clear()
        }),
        ("scatter seen site", |w| {
            let a = w.agent_mut(1).unwrap();
            let v = a.seen.remove(&(0, 99)).unwrap();
            a.seen.insert((1, 99), v);
        }),
        ("scatter seen owner", |w| {
            let a = w.agent_mut(1).unwrap();
            let v = a.seen.remove(&(0, 99)).unwrap();
            a.seen.insert((0, 98), v);
        }),
        ("scatter seen amount", |w| {
            w.agent_mut(1)
                .unwrap()
                .seen
                .get_mut(&(0, 99))
                .unwrap()
                .amount += 1.0
        }),
        ("scatter seen tick", |w| {
            w.agent_mut(1).unwrap().seen.get_mut(&(0, 99)).unwrap().tick += 1
        }),
    ];
    let mut base = spatial_fixture();
    let a = base.agent_mut(1).unwrap();
    a.cache_since.insert(0, 0);
    a.seen.insert(
        (0, 99),
        sugarscape_core::minds::caching::watching::SeenCache {
            amount: 3.0,
            tick: 0,
        },
    );
    let s = a.spatial.as_mut().unwrap();
    s.larder = 10.0;
    s.larder_since = Some(0);
    s.delivery = Some(Delivery { amount: 5.0 });
    s.seen_larders.insert(
        99,
        SeenLarder {
            home: Pos::new(2, 3),
            amount: 4.0,
            tick: 0,
        },
    );
    for &(name, change) in changes {
        let mut altered = base.clone();
        change(&mut altered);
        assert_ne!(base.fingerprint(), altered.fingerprint(), "enabled {name}");
        altered.config.spatial_hoarding.enabled = false;
        let mut off = base.clone();
        off.config.spatial_hoarding.enabled = false;
        assert_eq!(off.fingerprint(), altered.fingerprint(), "disabled {name}");
    }
    let mut off = base.clone();
    off.config.spatial_hoarding.enabled = false;
    assert_ne!(base.fingerprint(), off.fingerprint(), "enabled gate");
}

#[test]
fn spatial_memory_insertion_order_does_not_change_the_hash() {
    let mut a = spatial_fixture();
    let mut b = a.clone();
    for (w, owners) in [(&mut a, [88, 99]), (&mut b, [99, 88])] {
        for owner in owners {
            w.agent_mut(1)
                .unwrap()
                .spatial
                .as_mut()
                .unwrap()
                .seen_larders
                .insert(
                    owner,
                    SeenLarder {
                        home: Pos::new(2, 3),
                        amount: owner as f64,
                        tick: 0,
                    },
                );
        }
    }
    assert_eq!(a.fingerprint(), b.fingerprint());
}

#[test]
fn protection_checkpoints_cover_prepared_memory_and_every_stage() {
    use sugarscape_core::minds::protection::{lab::rig_config, state::*};
    let mut stages = Vec::new();
    for fixture in [
        Fixture::Single {
            initial_observed: true,
            redeposit_observed: false,
        },
        Fixture::Mixed {
            observed_first: true,
        },
    ] {
        let config = rig_config(LabConfig {
            policy: Policy::Selective,
            fixture,
            ..Default::default()
        });
        let mut straight = ModelWorld::Sugarscape(Box::new(World::new(config, 7).unwrap()));
        for tick in 0..64 {
            if (8..=11).contains(&tick) {
                let state = sugar(&straight).agent(1).unwrap().protection.clone();
                if let Some(stage) = state
                    .as_ref()
                    .and_then(|s| s.intent.as_ref())
                    .map(|i| i.stage.clone())
                {
                    stages.push(stage);
                }
                let cp = straight.checkpoint().unwrap();
                let mut restored = ModelWorld::Sugarscape(Box::new(sugar(&straight).clone()));
                restored.model_mut().run(2);
                restored.restore(&cp).unwrap();
                assert_eq!(sugar(&restored).agent(1).unwrap().protection, state);
                let mut expected = ModelWorld::Sugarscape(Box::new(sugar(&straight).clone()));
                for _ in tick..64 {
                    expected.model_mut().run(1);
                    restored.model_mut().run(1);
                    assert_eq!(
                        expected.model().fingerprint(),
                        restored.model().fingerprint()
                    );
                    assert_eq!(inspect_all(&expected), inspect_all(&restored));
                }
                assert_eq!(all_series(&expected), all_series(&restored));
            }
            straight.model_mut().run(1);
        }
    }
    for stage in [Stage::Retrieve, Stage::ToDestination, Stage::Deposit] {
        assert!(stages.contains(&stage), "missing {stage:?}");
    }
}

#[test]
fn deception_checkpoints_restore_phases_memory_diagnostics_and_rng_continuation() {
    use sugarscape_core::minds::deception::{lab::rig_config, state::*};
    let mut covered = Vec::new();
    let mut paid = false;
    let mut memory = false;
    for sender in [SenderPolicy::Sham, SenderPolicy::MatchedNeutral] {
        let mut straight = ModelWorld::Sugarscape(Box::new(
            World::new(
                rig_config(LabConfig {
                    sender,
                    effort_cost: 3.0,
                    ..Default::default()
                }),
                7,
            )
            .unwrap(),
        ));
        for tick in 0..=32 {
            let owner = sugar(&straight).agent(1).unwrap();
            let state = owner.deception.clone();
            covered.push(state.as_ref().unwrap().stage);
            let seen = sugar(&straight).agent(2).unwrap().seen.clone();
            memory |= !seen.is_empty();
            let runtime = sugar(&straight).deception.clone();
            paid |= runtime
                .as_ref()
                .unwrap()
                .actions
                .iter()
                .any(|a| a.effort == 3.0);
            let mut expected = ModelWorld::Sugarscape(Box::new(sugar(&straight).clone()));
            let cp = straight.checkpoint().unwrap();
            let mut restored = ModelWorld::Sugarscape(Box::new(sugar(&straight).clone()));
            restored.model_mut().run(3);
            restored.restore(&cp).unwrap();
            assert_eq!(
                sugar(&restored).agent(1).unwrap().deception,
                state,
                "state tick {tick}"
            );
            assert_eq!(
                sugar(&restored).agent(2).unwrap().seen,
                seen,
                "memory tick {tick}"
            );
            assert_eq!(sugar(&restored).deception, runtime, "runtime tick {tick}");
            for next in tick..64 {
                expected.model_mut().run(1);
                restored.model_mut().run(1);
                assert_eq!(
                    expected.model().fingerprint(),
                    restored.model().fingerprint(),
                    "continuation {tick} -> {next}"
                );
                assert_eq!(
                    sugar(&expected).deception,
                    sugar(&restored).deception,
                    "shuffled action stream {tick} -> {next}"
                );
                assert_eq!(inspect_all(&expected), inspect_all(&restored));
            }
            assert_eq!(all_series(&expected), all_series(&restored));
            straight.model_mut().run(1);
        }
    }
    for stage in [
        Stage::Preparation,
        Stage::ToDisplay,
        Stage::Display,
        Stage::Return,
    ] {
        assert!(covered.contains(&stage), "missing {stage:?}");
    }
    assert!(paid, "paid bout exercised");
    assert!(memory, "observer memory exercised");
}

#[test]
fn deception_checkpoint_preserves_pending_departure_and_its_next_action() {
    use sugarscape_core::minds::deception::{lab::rig_config, state::*};
    let mut w = World::new(
        rig_config(LabConfig {
            sender: SenderPolicy::Sham,
            effort_cost: 3.0,
            ..Default::default()
        }),
        7,
    )
    .unwrap();
    for _ in 0..20 {
        w.step();
    }
    // Counterfactual source-state control: this queue is authoritative regardless
    // of whether this finite construction episode happens to recover its source.
    w.agent_mut(1)
        .unwrap()
        .deception
        .as_mut()
        .unwrap()
        .pending_departure = true;
    let mut expected = ModelWorld::Sugarscape(Box::new(w.clone()));
    let mut restored = ModelWorld::Sugarscape(Box::new(w));
    let cp = restored.checkpoint().unwrap();
    restored.model_mut().run(3);
    restored.restore(&cp).unwrap();
    assert!(
        sugar(&restored)
            .agent(1)
            .unwrap()
            .deception
            .as_ref()
            .unwrap()
            .pending_departure
    );
    for tick in 20..64 {
        expected.model_mut().run(1);
        restored.model_mut().run(1);
        if tick == 20 {
            let action = sugar(&restored)
                .deception
                .as_ref()
                .unwrap()
                .actions
                .iter()
                .find(|a| a.actor == 1)
                .unwrap();
            assert_eq!(action.action, "departure");
            assert_eq!(action.target, Some(Pos::new(3, 2)));
            assert!(
                !sugar(&restored)
                    .agent(1)
                    .unwrap()
                    .deception
                    .as_ref()
                    .unwrap()
                    .pending_departure
            );
        }
        assert_eq!(
            sugar(&expected).deception,
            sugar(&restored).deception,
            "actions tick {tick}"
        );
        assert_eq!(
            expected.model().fingerprint(),
            restored.model().fingerprint()
        );
        assert_eq!(inspect_all(&expected), inspect_all(&restored));
    }
}
