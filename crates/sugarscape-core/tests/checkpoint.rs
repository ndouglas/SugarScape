//! Keyframes: a world restored from a checkpoint and run on is the world
//! that ran straight there, for every model.

use sugarscape_core::model::ModelWorld;
use sugarscape_core::presets;

/// One preset per model kind.
const IDS: &[&str] = &[
    "vi-1-everything",
    "vi-4-schelling-25",
    "vi-8-ring-world",
    "lhv-published",
];

fn world(id: &str) -> ModelWorld {
    let preset = presets::find(id).unwrap_or_else(|| panic!("unknown preset {id}"));
    ModelWorld::new(preset.config, 1).unwrap()
}

fn all_series(w: &ModelWorld) -> Vec<Option<Vec<f64>>> {
    let m = w.model();
    let mut names = m.series_names();
    names.push("tick".into());
    names.iter().map(|n| m.series(n)).collect()
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
