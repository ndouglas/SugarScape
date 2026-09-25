//! The spatial games (milestone 12) against Nowak & May 1992, Huberman &
//! Glance 1993 and Nowak, Bonhoeffer & May 1994. The kaleidoscope's exact
//! claims run with the other tests; the statistical claims run over seeds
//! 1–20 in release: `cargo test -p sugarscape-core --release --test spatial
//! -- --ignored`. Thresholds come from the measurements recorded 2026-09-25
//! (docs/superpowers/plans/2026-09-25-spatial-games.md, Decision 12); claims
//! that do not hold are pinned as measured.

use std::thread;

use sugarscape_core::model::ModelConfig;
use sugarscape_core::presets;
use sugarscape_core::spatial::{SpatialConfig, SpatialWorld, Update, Winning};

fn config(id: &str) -> SpatialConfig {
    match presets::find(id)
        .unwrap_or_else(|| panic!("no preset {id}"))
        .config
    {
        ModelConfig::Spatial(c) => c,
        _ => panic!("{id} is not a spatial preset"),
    }
}

/// Runs `c` for `ticks` from seeds 1–20 in parallel and measures each run.
fn each_seed<T: Send>(
    c: &SpatialConfig,
    ticks: u32,
    f: impl Fn(&SpatialWorld) -> T + Sync,
) -> Vec<T> {
    thread::scope(|s| {
        let handles: Vec<_> = (1..=20u64)
            .map(|seed| {
                let f = &f;
                s.spawn(move || {
                    let mut w = SpatialWorld::new(c.clone(), seed).unwrap();
                    w.run(ticks);
                    f(&w)
                })
            })
            .collect();
        handles.into_iter().map(|h| h.join().unwrap()).collect()
    })
}

fn series(w: &SpatialWorld, name: &str) -> Vec<f64> {
    w.stats.series(name).unwrap()
}

fn last(w: &SpatialWorld, name: &str) -> f64 {
    *series(w, name).last().unwrap()
}

/// The mean of `fraction_c` from tick `from` on.
fn tail(w: &SpatialWorld, from: usize) -> f64 {
    let s = series(w, "fraction_c");
    s[from..].iter().sum::<f64>() / (s.len() - from) as f64
}

/// Whether the square lattice's strategies are symmetric under the
/// square's reflections (and so its rotations).
fn four_fold(w: &SpatialWorld) -> bool {
    let (n, _, _) = w.geometry.dims;
    let at = |x: u32, y: u32| w.is_cooperator(w.geometry.at(x, y, 0).unwrap());
    (0..n).all(|y| {
        (0..n).all(|x| {
            let s = at(x, y);
            s == at(y, x) && s == at(n - 1 - x, y) && s == at(x, n - 1 - y)
        })
    })
}

#[test]
fn the_kaleidoscope_keeps_its_symmetry_and_reaches_the_boundary_at_49() {
    // NM92 Fig. 3: "the pattern has reached the boundary (which happens at
    // t = 49)"; "the initial symmetry is always maintained".
    let mut w = SpatialWorld::new(config("nm-3-kaleidoscope"), 1).unwrap();
    let edge = |w: &SpatialWorld| {
        (0..99).any(|k| {
            [(k, 0), (k, 98), (0, k), (98, k)]
                .iter()
                .any(|&(x, y)| !w.is_cooperator(w.geometry.at(x, y, 0).unwrap()))
        })
    };
    let mut reached = None;
    for t in 1..=221u32 {
        w.step();
        if reached.is_none() && edge(&w) {
            reached = Some(t);
        }
        if [30, 217, 219, 221].contains(&t) {
            assert!(four_fold(&w), "t = {t}");
        }
    }
    assert_eq!(reached, Some(49));
}

#[test]
#[ignore]
fn spatial_chaos_settles_at_12_ln_2_minus_8() {
    // Measured: 400², 40% D: 0.3179 ± 0.0005 over t = 201–300.
    let target = 12.0 * std::f64::consts::LN_2 - 8.0;
    let v = each_seed(&config("nm-2a-universal"), 300, |w| tail(w, 201));
    assert!(v.iter().all(|f| (f - target).abs() < 0.003), "{v:?}");
    // From 5% to 60% defectors (200²) every seed settles at 0.318–0.323;
    // from 80%, 19 of 20 do and one (seed 12) ends all C.
    for (d, exceptions) in [(0.05, 0), (0.3, 0), (0.6, 0), (0.8, 1)] {
        let mut c = config("nm-1b-chaos");
        c.defectors = d;
        let v = each_seed(&c, 400, |w| tail(w, 301));
        let off = v.iter().filter(|f| !(0.312..0.33).contains(*f)).count();
        assert_eq!(off, exceptions, "{d}: {v:?}");
    }
}

#[test]
#[ignore]
fn nm92s_other_regimes_and_neighborhoods() {
    // Fig. 1a: f_C "usually between 0.7 and 0.95" (measured 0.737–0.748).
    let v = each_seed(&config("nm-1a-static"), 200, |w| last(w, "fraction_c"));
    assert!(v.iter().all(|f| (0.7..0.95).contains(f)), "{v:?}");
    // Without self-interaction ≈ 0.299 (measured 0.301–0.305); four
    // neighbors ≈ 0.374 (measured 0.379–0.382).
    let v = each_seed(&config("nm-no-self"), 1000, |w| tail(w, 501));
    assert!(v.iter().all(|f| (0.295..0.31).contains(f)), "{v:?}");
    let v = each_seed(&config("nm-four-neighbors"), 1000, |w| tail(w, 501));
    assert!(v.iter().all(|f| (0.37..0.39).contains(f)), "{v:?}");
}

#[test]
#[ignore]
fn asynchronous_updating_ends_in_defection_only_above_1_8() {
    // HG93: all D "within a hundred generations or so" (measured t = 56–149).
    let v = each_seed(&config("hg-async-kaleidoscope"), 300, |w| {
        series(w, "fraction_c").iter().position(|&f| f == 0.0)
    });
    assert!(
        v.iter().all(|t| t.is_some_and(|t| (30..=250).contains(&t))),
        "{v:?}"
    );
    // But not "as long as there is at least one defector": at b = 1.7 the
    // defector dies out (measured f_C 0.974–0.999).
    let mut c = config("hg-async-kaleidoscope");
    c.b = 1.7;
    let v = each_seed(&c, 300, |w| last(w, "fraction_c"));
    assert!(v.iter().all(|&f| f > 0.95), "{v:?}");
}

#[test]
#[ignore]
fn nbm94s_continuous_time_and_self_interaction() {
    // b = 1.9, deterministic, 80² periodic, 50% D: all D in continuous time
    // (20 of 20); spatial chaos in discrete time in 16 of 20 (two seeds end
    // all C, two all but all D).
    let mut c = config("nbm-continuous");
    c.b = 1.9;
    let v = each_seed(&c, 200, |w| last(w, "fraction_c"));
    assert!(v.iter().all(|&f| f == 0.0), "{v:?}");
    c.update = Update::Synchronous;
    let v = each_seed(&c, 200, |w| last(w, "fraction_c"));
    let chaotic = v.iter().filter(|&&f| f > 0.05 && f < 0.95).count();
    assert!(chaotic >= 12, "{chaotic}: {v:?}");
    // "for m = 1 … C cannot persist in the absence of self-interaction"
    // (measured: f_C ≤ 0.007 at b = 1.13 and 1.35; but 0.16–0.33 at 1.05),
    // while deterministic winning keeps C (0.85–0.95).
    let mut c = config("nbm-probabilistic");
    c.self_weight = 0.0;
    for b in [1.13, 1.35] {
        c.b = b;
        c.winning = Winning::Probabilistic;
        let v = each_seed(&c, 200, |w| last(w, "fraction_c"));
        assert!(v.iter().all(|&f| f < 0.02), "{b}: {v:?}");
        c.winning = Winning::Deterministic;
        let v = each_seed(&c, 200, |w| last(w, "fraction_c"));
        assert!(v.iter().all(|&f| f > 0.8), "{b}: {v:?}");
    }
    c.b = 1.05;
    c.winning = Winning::Probabilistic;
    let v = each_seed(&c, 200, |w| last(w, "fraction_c"));
    assert!(v.iter().all(|&f| f > 0.1), "C persists at b = 1.05: {v:?}");
}

#[test]
#[ignore]
fn random_arrays_lose_cooperation_near_radius_9() {
    // NBM94: "for b = 1.6, r_c ~ 9" — from a 50% start (measured all D in
    // 0, 10 and 20 of 20 seeds at r = 5, 9 and 11).
    let all_d = |r: f64| {
        let mut c = config("nbm-random-array");
        c.radius = r;
        each_seed(&c, 300, |w| last(w, "fraction_c") == 0.0)
            .iter()
            .filter(|&&d| d)
            .count()
    };
    assert_eq!(all_d(5.0), 0);
    assert!((4..=16).contains(&all_d(9.0)));
    assert_eq!(all_d(11.0), 20);
    // From NM92's 10% start no radius up to 11 ends all D.
    let mut c = config("nbm-random-array");
    c.defectors = 0.1;
    c.radius = 11.0;
    let v = each_seed(&c, 300, |w| last(w, "fraction_c"));
    assert!(v.iter().all(|&f| f > 0.3), "{v:?}");
}

#[test]
#[ignore]
fn the_cube_coexists_like_the_square() {
    // b = 1.6, 30³ periodic: f_C 0.331–0.342, a quarter of the cube changing
    // each generation.
    let v = each_seed(&config("nbm-cube"), 200, |w| {
        (tail(w, 101), last(w, "changed"))
    });
    assert!(
        v.iter()
            .all(|&(f, ch)| (0.3..0.37).contains(&f) && ch > 0.2),
        "{v:?}"
    );
}
