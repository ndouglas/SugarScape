//! Epstein's civil violence (milestone 11) against the paper, over seeds
//! 1–20 in release: `cargo test -p sugarscape-core --release --test civil
//! -- --ignored`. Thresholds come from the measurements recorded
//! 2026-09-25 (docs/superpowers/plans/2026-09-25-civil-violence.md,
//! Decision 14), each with room around the measured range; the claims the
//! paper makes that do not reproduce are pinned as measured, so a change
//! that would reproduce them shows up here.

use std::thread;

use sugarscape_core::civil::CivilConfig;
use sugarscape_core::model::{ModelConfig, ModelWorld};
use sugarscape_core::presets;

const SEEDS: u64 = 20;

fn config(id: &str) -> CivilConfig {
    match presets::find(id)
        .unwrap_or_else(|| panic!("no preset {id}"))
        .config
    {
        ModelConfig::Civil(c) => c,
        _ => panic!("{id} is not a civil preset"),
    }
}

/// Runs `c` for `ticks` from seeds 1–20 in parallel and measures each run.
fn each_seed<T: Send>(c: &CivilConfig, ticks: u32, f: impl Fn(&ModelWorld) -> T + Sync) -> Vec<T> {
    thread::scope(|s| {
        let handles: Vec<_> = (1..=SEEDS)
            .map(|seed| {
                let f = &f;
                s.spawn(move || {
                    let mut w = ModelWorld::new(ModelConfig::Civil(c.clone()), seed).unwrap();
                    w.model_mut().run(ticks);
                    f(&w)
                })
            })
            .collect();
        handles.into_iter().map(|h| h.join().unwrap()).collect()
    })
}

fn series(w: &ModelWorld, name: &str) -> Vec<f64> {
    w.model().series(name).unwrap()
}

fn last(w: &ModelWorld, name: &str) -> f64 {
    *series(w, name).last().unwrap()
}

fn max(v: &[f64]) -> f64 {
    v.iter().copied().fold(f64::NEG_INFINITY, f64::max)
}

fn mean(v: &[f64]) -> f64 {
    v.iter().sum::<f64>() / v.len() as f64
}

fn count(v: &[bool]) -> usize {
    v.iter().filter(|&&b| b).count()
}

#[test]
#[ignore]
fn run_2_is_punctuated_only_with_the_ratio_rounded_down() {
    // Measured: 101–122 outbursts in 3000 ticks, mean activation 779 (paper
    // 708 ± 230), mean wait 21.8 (paper 60 ± 55), 61–68% of ticks calm.
    let c = config("cv-run-2-punctuated");
    let runs = each_seed(&c, 3000, |w| {
        let a = series(w, "active");
        (
            last(w, "outbursts"),
            last(w, "mean_activation"),
            last(w, "mean_wait"),
            a.iter().filter(|&&x| x < 10.0).count() as f64 / a.len() as f64,
        )
    });
    for (seed, &(outbursts, activation, wait, calm)) in runs.iter().enumerate() {
        assert!(
            outbursts >= 80.0,
            "seed {}: {outbursts} outbursts",
            seed + 1
        );
        assert!(calm >= 0.5, "seed {}: calm {calm}", seed + 1);
        assert!(
            (500.0..=1100.0).contains(&activation),
            "seed {}: {activation}",
            seed + 1
        );
        assert!(
            (12.0..=35.0).contains(&wait),
            "seed {}: wait {wait}",
            seed + 1
        );
    }
    // The paper's rule: no outburst at all (at most 13–34 actives).
    let mut literal = c;
    literal.quirks.floor_ratio = false;
    let peaks = each_seed(&literal, 3000, |w| max(&series(w, "active")));
    assert!(peaks.iter().all(|&p| p < 50.0), "{peaks:?}");
}

#[test]
#[ignore]
fn one_jump_outpeaks_salami_tactics_but_does_not_outjail_them() {
    // Measured (paired by seed): the jump's peak after t = 77 is higher in
    // 17 of 20, passes 50 in 9 (salami 3); its jail ends larger in 7.
    let peak_and_jail = |w: &ModelWorld| (max(&series(w, "active")[77..]), last(w, "jailed"));
    let salami = each_seed(&config("cv-run-3-salami"), 300, peak_and_jail);
    let jump = each_seed(&config("cv-run-4-one-jump"), 300, peak_and_jail);
    let higher: Vec<bool> = jump.iter().zip(&salami).map(|(j, s)| j.0 > s.0).collect();
    let jailed: Vec<bool> = jump.iter().zip(&salami).map(|(j, s)| j.1 > s.1).collect();
    let spikes = |r: &[(f64, f64)]| r.iter().filter(|(p, _)| *p > 50.0).count();
    assert!(count(&higher) >= 14, "higher in {}", count(&higher));
    assert!(
        spikes(&jump) > spikes(&salami),
        "{} vs {}",
        spikes(&jump),
        spikes(&salami)
    );
    assert!(count(&jailed) <= 12, "jailed more in {}", count(&jailed));
}

#[test]
#[ignore]
fn walking_the_cops_down_tips_the_society() {
    // Measured: peaks of 64–345 actives (mean 194) with about 88 of 118
    // cops left; the paper's rule peaks at 14–33 and never tips.
    let c = config("cv-run-5-cop-reductions");
    let peaks = each_seed(&c, 700, |w| max(&series(w, "active")));
    assert!(peaks.iter().all(|&p| p >= 50.0), "{peaks:?}");
    assert!(mean(&peaks) >= 120.0, "{peaks:?}");
    let mut literal = c;
    literal.quirks.floor_ratio = false;
    let peaks = each_seed(&literal, 700, |w| max(&series(w, "active")));
    assert!(peaks.iter().all(|&p| p < 50.0), "{peaks:?}");
}

#[test]
#[ignore]
fn model_two_coexists_at_high_legitimacy_and_cleanses_at_low() {
    // Measured: run 6 no kills; run 7 genocide in every seed at t = 30–190,
    // Blue surviving 12 times and Green 8.
    let kills = each_seed(&config("cv-run-6-coexistence"), 1000, |w| {
        (
            series(w, "killed").iter().sum::<f64>(),
            last(w, "blue"),
            last(w, "green"),
        )
    });
    assert!(
        kills
            .iter()
            .all(|&(k, b, g)| k == 0.0 && b > 0.0 && g > 0.0),
        "{kills:?}"
    );
    let ends = each_seed(&config("cv-run-7-cleansing"), 3000, |w| {
        (
            w.model().finished(),
            w.model().tick(),
            last(w, "blue") > 0.0,
        )
    });
    assert!(
        ends.iter().all(|&(done, tick, _)| done && tick <= 400),
        "{ends:?}"
    );
    let blue = ends.iter().filter(|e| e.2).count();
    assert!(
        (4..=16).contains(&blue),
        "the victor is random: Blue {blue} of 20"
    );
}

#[test]
#[ignore]
fn cops_do_not_keep_both_groups_alive() {
    // The paper's "stable, but nasty, regime" (Run 8) and safe havens do
    // not reproduce: measured, one group gone in every seed by t = 882 and
    // t = 377. Its Fig. 15's rapid genocide at every density does.
    for id in ["cv-run-8-nasty-regime", "cv-safe-havens"] {
        let mut c = config(id);
        c.stop_at_extinction = false;
        let alive = each_seed(&c, 3000, |w| {
            last(w, "blue") > 0.0 && last(w, "green") > 0.0
        });
        assert_eq!(
            count(&alive),
            0,
            "{id}: both groups alive in {}",
            count(&alive)
        );
    }
}

#[test]
#[ignore]
fn netlogos_rebellion_is_punctuated() {
    // Measured: 93–109 outbursts in 3000 ticks, mean activation 1107.
    let runs = each_seed(&config("cv-netlogo"), 3000, |w| last(w, "outbursts"));
    assert!(runs.iter().all(|&o| o >= 70.0), "{runs:?}");
}
