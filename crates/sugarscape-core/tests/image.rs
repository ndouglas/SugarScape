//! Image scoring against its sources: NS98's Figs. 1–4 and Methods, and
//! LH01's Figs. 1–4 and standing condition. Ignored by default (release:
//! `cargo test -p sugarscape-core --release --test image -- --ignored
//! --nocapture`). NS98 averaged over 10⁷ generations and LH01 over 10⁵–10⁶;
//! these runs are shorter, averaged over seeds, as each test says. Every
//! world is a function of (config, seed), so the numbers pinned here are the
//! measured ones (recorded 2026-09-26), each with the source's beside it;
//! claims that fail are pinned as measured.

use std::sync::Mutex;

use sugarscape_core::image::analytic::{self, Binary, Start};
use sugarscape_core::image::{
    Class, ImageConfig, ImageWorld, Initial, Offset, Records, Seeded, Strategy,
};
use sugarscape_core::model::{Model, ModelConfig};
use sugarscape_core::presets;

fn config(id: &str, edit: impl FnOnce(&mut ImageConfig)) -> ImageConfig {
    let ModelConfig::Image(mut c) = presets::find(id).unwrap().config else {
        panic!("{id} is not an image preset")
    };
    edit(&mut c);
    c
}

/// `f(seed)` for each seed, on up to ten threads, in seed order.
fn par<T: Send>(seeds: impl IntoIterator<Item = u64>, f: impl Fn(u64) -> T + Sync) -> Vec<T> {
    let next = Mutex::new(seeds.into_iter().collect::<Vec<_>>().into_iter());
    let out = Mutex::new(Vec::new());
    std::thread::scope(|s| {
        for _ in 0..10 {
            s.spawn(|| loop {
                let Some(seed) = next.lock().unwrap().next() else {
                    break;
                };
                let v = f(seed);
                out.lock().unwrap().push((seed, v));
            });
        }
    });
    let mut v = out.into_inner().unwrap();
    v.sort_by_key(|p| p.0);
    v.into_iter().map(|p| p.1).collect()
}

fn mean(v: &[f64]) -> f64 {
    v.iter().sum::<f64>() / v.len() as f64
}

/// The mean of `series` over generations `from..=ticks`, one per seed.
fn window(c: &ImageConfig, series: &str, seeds: u64, ticks: u32, from: usize) -> Vec<f64> {
    windows(c, &[series], seeds, ticks, from)
        .into_iter()
        .map(|v| v[0])
        .collect()
}

/// The means of several series over generations `from..=ticks`, per seed.
fn windows(c: &ImageConfig, series: &[&str], seeds: u64, ticks: u32, from: usize) -> Vec<Vec<f64>> {
    par(1..=seeds, |seed| {
        let mut w = ImageWorld::new(c.clone(), seed).unwrap();
        w.run(ticks);
        series
            .iter()
            .map(|s| mean(&w.series(s).unwrap()[from..]))
            .collect()
    })
}

/// Asserts a measured value (printing it beside the source's).
fn pin(what: &str, got: f64, want: f64, tol: f64, source: &str) {
    println!("{what}: {got} (pinned {want}; source: {source})");
    assert!((got - want).abs() <= tol, "{what}: {got} vs pinned {want}");
}

fn share(w: &ImageWorld, f: impl Fn(Strategy) -> bool) -> f64 {
    let a = w.agents();
    a.iter().filter(|x| f(x.strategy)).count() as f64 / a.len() as f64
}

/// The strategy every agent plays, if they all play one.
fn fixed(w: &ImageWorld) -> Option<Strategy> {
    let a = w.agents();
    a.iter()
        .all(|x| x.strategy == a[0].strategy)
        .then(|| a[0].strategy)
}

/// Seeds 1–100 of `c` to fixation (at most 5,000 generations): how many fix
/// k = 0, how many any k ≤ 0, and the median generation k = 0 fixed at.
fn fixation(c: &ImageConfig) -> (usize, usize, u64) {
    let r = par(1..=100, |seed| {
        let mut w = ImageWorld::new(c.clone(), seed).unwrap();
        for _ in 0..5000 {
            w.step();
            if let Some(s) = fixed(&w) {
                return Some((s, w.tick));
            }
        }
        None
    });
    let mut t0: Vec<u64> = r
        .iter()
        .flatten()
        .filter(|(s, _)| *s == Strategy::K(0))
        .map(|p| p.1)
        .collect();
    t0.sort();
    let coop = r.iter().flatten().filter(|(s, _)| s.cooperative()).count();
    (t0.len(), coop, t0.get(t0.len() / 2).copied().unwrap_or(0))
}

#[test]
#[ignore]
fn ns98_fig_1_k_0_fixes_but_only_in_a_fifth_of_runs() {
    // NS98 Fig. 1: one run in which k = 0 is fixed after t = 166.
    let (k0, coop, median) = fixation(&config("ns-fig-1", |_| {}));
    println!("k = 0 fixed in {k0}/100 (median generation {median}); some k ≤ 0 in {coop}/100");
    assert_eq!((k0, coop, median), (20, 40, 56));
    // More rounds, more cooperation ("more likely to win the greater the number m").
    let (k0, coop, _) = fixation(&config("ns-fig-1", |c| c.rounds = 300));
    println!("m = 300: k = 0 in {k0}/100, some k ≤ 0 in {coop}/100");
    assert_eq!((k0, coop), (14, 91));
}

#[test]
#[ignore]
fn ns98_fig_1_without_the_offset() {
    // Ours: the offset weakens selection; without it cooperation wins more often.
    let (k0, coop, median) = fixation(&config("ns-no-offset", |_| {}));
    println!("no offset: k = 0 in {k0}/100 (median {median}), some k ≤ 0 in {coop}/100");
    assert_eq!((k0, coop, median), (10, 63, 93));
}

/// Collapses of cooperation in `c` over `ticks` generations, seeds 1–10: a
/// collapse is the share of k ≤ 0 falling from at least 0.9 to at most 0.1,
/// a recovery the reverse. Also the mean share of k ≤ −4 in cooperative
/// generations (≥ 0.9) and over the 51 generations up to each collapse's
/// last cooperative one.
fn cycles(c: &ImageConfig, ticks: u64) -> (usize, usize, f64, f64) {
    let r = par(1..=10, |seed| {
        let mut w = ImageWorld::new(c.clone(), seed).unwrap();
        let (mut coop, mut unc) = (Vec::new(), Vec::new());
        for _ in 0..ticks {
            w.step();
            coop.push(share(&w, |s| s.k().is_some_and(|k| k <= 0)));
            unc.push(share(&w, |s| s.k().is_some_and(|k| k <= -4)));
        }
        let (mut state, mut last_hi, mut collapses, mut recoveries) = (0, 0, Vec::new(), 0);
        for (t, &x) in coop.iter().enumerate() {
            if x >= 0.9 {
                recoveries += usize::from(state == -1);
                (state, last_hi) = (1, t);
            } else if x <= 0.1 {
                if state == 1 {
                    collapses.push(last_hi);
                }
                state = -1;
            }
        }
        let in_coop: Vec<f64> = (0..coop.len())
            .filter(|&t| coop[t] >= 0.9)
            .map(|t| unc[t])
            .collect();
        let before: Vec<f64> = collapses
            .iter()
            .map(|&t| mean(&unc[t.saturating_sub(50)..=t]))
            .collect();
        (collapses.len(), recoveries, mean(&in_coop), before)
    });
    let collapses = r.iter().map(|p| p.0).sum();
    let recoveries = r.iter().map(|p| p.1).sum();
    let base = mean(&r.iter().map(|p| p.2).collect::<Vec<_>>());
    let before: Vec<f64> = r.iter().flat_map(|p| p.3.clone()).collect();
    (collapses, recoveries, base, mean(&before))
}

#[test]
#[ignore]
fn ns98_fig_2_cycles_and_unconditional_cooperators_come_first() {
    // NS98 Fig. 2: "endless cycles of cooperation and defection"; k = −4 or
    // −5 undermine cooperative populations, then defectors invade. 10⁶
    // generations in all (seeds 1–10 × 10⁵).
    let (collapses, recoveries, base, before) = cycles(&config("ns-fig-2", |_| {}), 100_000);
    println!("collapses {collapses}, recoveries {recoveries}; k ≤ −4: {base:.3} in cooperative generations, {before:.3} before collapses");
    assert_eq!((collapses, recoveries), (172, 167));
    pin("k ≤ −4 in cooperative generations", base, 0.081, 0.001, "—");
    pin(
        "k ≤ −4 before collapses",
        before,
        0.682,
        0.001,
        "rises first",
    );
    let c = config("ns-fig-2", |_| {});
    pin(
        "Fig. 2 cooperative (k ≤ 0), gens 1,001–100,000",
        mean(&window(&c, "cooperative", 10, 100_000, 1001)),
        0.672,
        0.001,
        "not given",
    );
    // Ours: without the offset.
    let c = config("ns-fig-2", |c| c.offset = Offset::None);
    pin(
        "Fig. 2 without the offset",
        mean(&window(&c, "cooperative", 10, 100_000, 1001)),
        0.777,
        0.001,
        "—",
    );
}

fn fig_3(n: u32, edit: impl FnOnce(&mut ImageConfig)) -> f64 {
    let id = match n {
        20 => "ns-fig-3-n20",
        50 => "ns-fig-3-n50",
        _ => "ns-fig-3-n100",
    };
    mean(&window(&config(id, edit), "cooperative", 10, 20_000, 1001))
}

#[test]
#[ignore]
fn ns98_fig_3_group_size_with_ten_observers() {
    // NS98 Fig. 3: cooperative strategies (k ≤ 0) 90%, 47%, 18% at n = 20,
    // 50, 100 (10⁷ generations). Here seeds 1–10, generations 1,001–20,000,
    // each observer keeping its own tally (the default).
    pin("n = 20", fig_3(20, |_| {}), 0.8629, 0.0005, "0.90");
    pin("n = 50", fig_3(50, |_| {}), 0.4397, 0.0005, "0.47");
    pin("n = 100", fig_3(100, |_| {}), 0.2008, 0.0005, "0.18");
}

#[test]
#[ignore]
fn ns98_fig_3_when_an_observer_learns_the_whole_score() {
    // `records: score`: one sighting reveals the donor's score. The group-size
    // effect all but vanishes.
    let score = |c: &mut ImageConfig| c.records = Records::Score;
    pin("n = 20, score", fig_3(20, score), 0.9676, 0.0005, "0.90");
    pin("n = 50, score", fig_3(50, score), 0.9279, 0.0005, "0.47");
    pin("n = 100, score", fig_3(100, score), 0.9234, 0.0005, "0.18");
}

#[test]
#[ignore]
fn ns98_fig_3_with_fair23s_fixed_visibility() {
    // FAIR23 sees each interaction with a fixed probability (0.1 by
    // default), not ten observers: 1.8, 4.8 and 9.8 observers at n = 20, 50,
    // 100. The group-size effect goes with it.
    pin(
        "n = 20, visibility 0.1",
        fig_3(20, |c| c.observers = 1.8),
        0.2824,
        0.0005,
        "0.90",
    );
    pin(
        "n = 50, visibility 0.1",
        fig_3(50, |c| c.observers = 4.8),
        0.1551,
        0.0005,
        "0.47",
    );
}

/// The most frequent strategies of `c` over generations 1,001–`ticks`,
/// seeds 1–10, with their shares.
fn most_frequent(c: &ImageConfig, ticks: u64) -> Vec<(Strategy, f64)> {
    let r = par(1..=10, |seed| {
        let mut w = ImageWorld::new(c.clone(), seed).unwrap();
        let mut counts = std::collections::HashMap::new();
        for t in 1..=ticks {
            w.step();
            if t > 1000 {
                for a in w.agents() {
                    *counts.entry(a.strategy).or_insert(0u64) += 1;
                }
            }
        }
        counts
    });
    let mut all = std::collections::HashMap::new();
    for c in r {
        for (s, n) in c {
            *all.entry(s).or_insert(0u64) += n;
        }
    }
    let total: u64 = all.values().sum();
    let mut v: Vec<(Strategy, f64)> = all
        .into_iter()
        .map(|(s, n)| (s, n as f64 / total as f64))
        .collect();
    v.sort_by(|a, b| b.1.total_cmp(&a.1).then(a.0.code().cmp(&b.0.code())));
    v.truncate(3);
    v
}

#[test]
#[ignore]
fn ns98_fig_4_and_and_or_strategies() {
    // NS98 Fig. 4: cooperative interactions 55%, 57%, 70%, 80% in (a)–(d);
    // most frequent (k 0, h 1) in (a), (k 0, h 4) in (b), and the defectors
    // (k 6, h −5) in (c) and (d). Seeds 1–10, generations 1,001–50,000.
    let rows = [
        ("ns-fig-4a", 0.5316, "0.55", Strategy::And { k: 0, h: 1 }),
        ("ns-fig-4b", 0.5344, "0.57", Strategy::And { k: 0, h: 5 }),
        ("ns-fig-4c", 0.7843, "0.70", Strategy::Or { k: 3, h: 4 }),
        ("ns-fig-4d", 0.8545, "0.80", Strategy::Or { k: 2, h: 5 }),
    ];
    for (id, want, source, first) in rows {
        let c = config(id, |_| {});
        let help = mean(&window(&c, "help_rate", 10, 50_000, 1001));
        pin(&format!("{id} help rate"), help, want, 0.0005, source);
        let top = most_frequent(&c, 50_000);
        println!("{id} most frequent: {top:?}");
        assert_eq!(top[0].0, first, "{id}");
    }
}

#[test]
#[ignore]
fn ns98_own_score_strategies_barely_cooperate() {
    // NS98: "less than 0.1% cooperation" with strategies that only consider
    // their own score. Mutation (0.001, uniform) keeps a floor above that.
    let c = config("ns-own-only", |_| {});
    pin(
        "own-only help rate",
        mean(&window(&c, "help_rate", 10, 20_000, 1001)),
        0.0019,
        0.0001,
        "< 0.001",
    );
}

#[test]
#[ignore]
fn ns98_two_interactions_per_lifetime() {
    // NS98: "it suffices that each player is chosen only for about 2
    // interactions per life-time" — m ≈ n (each player takes part in 2m/n).
    // Fig. 2's settings, seeds 1–10, generations 1,001–20,000.
    for (m, want) in [(25, 0.0134), (50, 0.1004), (100, 0.1815), (200, 0.5003)] {
        let c = config("ns-fig-2", |c| c.rounds = m);
        pin(
            &format!("m = {m} cooperative"),
            mean(&window(&c, "cooperative", 10, 20_000, 1001)),
            want,
            0.0005,
            "prevails from m ≈ n",
        );
    }
}

#[test]
#[ignore]
fn ns98_methods_thresholds() {
    let m = Binary::ns98();
    // "about 1.2 rounds per generation" at b = 1, c = 0.1, q = 1.
    pin(
        "minimum rounds (bq + c)/(bq − c)",
        m.min_rounds(),
        1.2222,
        0.0001,
        "about 1.2",
    );
    // q > c/b: at q = 0.1 no number of rounds suffices.
    assert_eq!(Binary { q: 0.1, ..m }.min_rounds(), f64::INFINITY);
    // The equilibrium with cooperators, x = c(2 − w)/(bwq), at w = 0.9.
    pin(
        "x with cooperators (w 0.9)",
        m.cooperator_equilibrium(0.9),
        0.1222,
        0.0001,
        "c(2 − w)/(bwq)",
    );
}

#[test]
#[ignore]
fn ns98_methods_x_min_against_a_simulation() {
    // Binary discriminators (k 0) against defectors (k 1), no offset,
    // perfect information, n = 100, m = 250 (5 rounds of the Methods'
    // everyone-plays-once rounds): one generation's payoff gap, seeds
    // 1–2,000 at each starting share.
    let a = Binary::ns98();
    let x_min = a.x_min_fixed(5).unwrap();
    let gap = |x: f64| {
        let c = ImageConfig {
            rounds: 250,
            offset: Offset::None,
            strategies: vec![Class::Binary],
            initial: Initial::Seeded(Seeded {
                only: Strategy::Binary(1),
                invader: Some(Strategy::Binary(0)),
                share: x,
            }),
            ..Default::default()
        };
        mean(&par(1..=2000, |seed| {
            let mut w = ImageWorld::new(c.clone(), seed).unwrap();
            w.step();
            let (mut d, mut e) = (Vec::new(), Vec::new());
            for a in w.agents() {
                if a.strategy == Strategy::Binary(0) {
                    d.push(a.payoff)
                } else {
                    e.push(a.payoff)
                }
            }
            mean(&d) - mean(&e)
        }))
    };
    // The share where the gap changes sign, to 0.01.
    let mut crossing = 0.0;
    for i in 10..=20 {
        let x = f64::from(i) / 100.0;
        if gap(x) > 0.0 {
            crossing = x;
            break;
        }
    }
    pin(
        "analytic x_min (5 rounds)",
        x_min,
        0.1228,
        0.0001,
        "Methods",
    );
    pin("simulated crossing", crossing, 0.16, 0.0005, "x_min");
}

#[test]
#[ignore]
fn ns98_universal_constant_under_each_start() {
    // "0.7380294688360…": the largest fraction below 0 from which everyone
    // at k = 0 reaches all-out cooperation; the start is not stated.
    let rows = [
        (Start::AtMinusOne { rest: Some(0) }, 0.5),
        (Start::AtMinusOne { rest: Some(1) }, 0.6420045049661),
        (Start::AtMinusOne { rest: Some(2) }, 0.6878695524337),
        (Start::AtMinusOne { rest: Some(5) }, 0.7263483514787),
        (Start::AtMinusOne { rest: Some(20) }, 0.7379660293712),
        (Start::AtMinusOne { rest: Some(80) }, 0.7380294688227),
        (Start::AtMinusOne { rest: None }, 0.7380294688360),
        (Start::Spread { below: 2 }, 0.4209145147265),
        (Start::Spread { below: 5 }, 0.3384593809259),
        (Start::SpreadBoth { below: 5, above: 5 }, 0.5157902242303),
    ];
    let got = par(0..rows.len() as u64, |i| {
        analytic::universal_threshold(rows[i as usize].0)
    });
    for ((start, want), got) in rows.iter().zip(got) {
        pin(&format!("{start:?}"), got, *want, 1e-12, "0.7380294688360");
    }
}

/// The share of strategies matching `is` at each generation in `at`, per seed.
fn invasion(
    c: &ImageConfig,
    seeds: u64,
    at: &[u64],
    is: impl Fn(Strategy) -> bool + Sync,
) -> Vec<Vec<f64>> {
    par(1..=seeds, |seed| {
        let mut w = ImageWorld::new(c.clone(), seed).unwrap();
        at.iter()
            .map(|&t| {
                while w.tick < t {
                    w.step();
                }
                share(&w, &is)
            })
            .collect()
    })
}

fn column(v: &[Vec<f64>], i: usize) -> Vec<f64> {
    v.iter().map(|r| r[i]).collect()
}

#[test]
#[ignore]
fn lh01_fig_1_h_1_invades() {
    // LH01 Fig. 1a: h = 1 invades k = 0 (e = 0); 1b: it invades (k 0, h 1)
    // with e = 0.05, "but does not wipe out". Invaders start at 1% of each
    // group; seeds 1–10.
    let v = invasion(&config("lh-fig-1a", |_| {}), 10, &[50, 100, 150], |s| {
        s == Strategy::H(1)
    });
    pin(
        "1a h = 1 at 50",
        mean(&column(&v, 0)),
        0.3704,
        0.0005,
        "rising",
    );
    pin(
        "1a h = 1 at 150",
        mean(&column(&v, 2)),
        1.0,
        0.0005,
        "about 0.8",
    );
    let v = invasion(&config("lh-fig-1b", |_| {}), 10, &[150, 500, 1000], |s| {
        s == Strategy::H(1)
    });
    pin(
        "1b h = 1 at 150",
        mean(&column(&v, 0)),
        0.0183,
        0.0005,
        "invading",
    );
    pin("1b h = 1 at 500", mean(&column(&v, 1)), 0.1317, 0.0005, "—");
    pin(
        "1b h = 1 at 1,000",
        mean(&column(&v, 2)),
        0.4232,
        0.0005,
        "—",
    );
}

#[test]
#[ignore]
fn lh01_fig_2a_one_group() {
    // LH01 Fig. 2a: help in 39% of rounds (10⁶ generations), (k 0, h 1)
    // dominant. Seeds 1–10, generations 1,001–100,000.
    let c = config("lh-fig-2a", |_| {});
    pin(
        "2a help",
        mean(&window(&c, "help_rate", 10, 100_000, 1001)),
        0.3742,
        0.0005,
        "0.39",
    );
    let top = most_frequent(&c, 100_000);
    println!("2a most frequent: {top:?}");
    assert_eq!(top[0].0, Strategy::And { k: 0, h: 1 });
}

#[test]
#[ignore]
fn lh01_fig_2bc_the_island_model() {
    // LH01 Fig. 2b: 9% help (p 0.9, 10⁵ generations); 2c: 2% (p 0.5).
    // Seeds 1–10, generations 1,001–5,000.
    let c = config("lh-fig-2b", |_| {});
    pin(
        "2b help",
        mean(&window(&c, "help_rate", 10, 5000, 1001)),
        0.4413,
        0.0005,
        "0.09",
    );
    let c = config("lh-fig-2c", |_| {});
    pin(
        "2c help",
        mean(&window(&c, "help_rate", 10, 5000, 1001)),
        0.1544,
        0.0005,
        "0.02",
    );
}

#[test]
#[ignore]
fn lh01_fig_3_a_small_cost_and_q_strategies() {
    // LH01 Fig. 3a: 45% help (c 0.1, u₀ 5, p 0.5, e 0.02; 2 × 10⁵
    // generations); 3b–c with q strategies: 15% help, 12% q strategies.
    // Seeds 1–10, generations 1,001–3,000.
    let c = config("lh-fig-3a", |_| {});
    pin(
        "3a help",
        mean(&window(&c, "help_rate", 10, 3000, 1001)),
        0.522,
        0.0005,
        "0.45",
    );
    let c = config("lh-fig-3b", |_| {});
    let v = windows(&c, &["help_rate", "q"], 10, 3000, 1001);
    pin("3b help", mean(&column(&v, 0)), 0.1653, 0.0005, "0.15");
    pin("3b q share", mean(&column(&v, 1)), 0.2579, 0.0005, "0.12");
}

#[test]
#[ignore]
fn lh01_fig_4ab_standing_invades_discriminators() {
    // LH01 Fig. 4a (e 0.05) and 4b (e = ε = 0.025): standing takes over a
    // population of binary discriminators within 1,000 generations. Standing
    // starts at 1% of each group; seeds 1–10 (4a), 1–5 (4b).
    let standing = |s: Strategy| s == Strategy::Standing;
    let v = invasion(
        &config("lh-fig-4a", |_| {}),
        10,
        &[250, 500, 1000],
        standing,
    );
    pin(
        "4a standing at 500",
        mean(&column(&v, 1)),
        0.6939,
        0.0005,
        "rising",
    );
    pin(
        "4a standing at 1,000",
        mean(&column(&v, 2)),
        0.9841,
        0.0005,
        "near 1",
    );
    let v = invasion(&config("lh-fig-4b", |_| {}), 5, &[500, 1000], standing);
    pin(
        "4b standing at 500",
        mean(&column(&v, 0)),
        0.3523,
        0.0005,
        "rising",
    );
    pin(
        "4b standing at 1,000",
        mean(&column(&v, 1)),
        0.7459,
        0.0005,
        "near 1",
    );
}

#[test]
#[ignore]
fn lh01_fig_4c_standing_persists_with_cooperators() {
    // LH01 Fig. 4c: over 10⁵ generations standing dominates, cooperators
    // stay at appreciable frequencies, discriminators occasionally reach 5%,
    // defectors stay below 1%. Here seeds 1–3, generations 1,001–1,500 (a
    // uniform start).
    let c = config("lh-fig-4c", |_| {});
    let v = windows(
        &c,
        &["standing", "binary_c", "binary_x", "binary_d"],
        3,
        1500,
        1001,
    );
    pin(
        "4c standing",
        mean(&column(&v, 0)),
        0.5309,
        0.0005,
        "dominant",
    );
    pin(
        "4c cooperators",
        mean(&column(&v, 1)),
        0.3886,
        0.0005,
        "appreciable",
    );
    pin(
        "4c discriminators",
        mean(&column(&v, 2)),
        0.0803,
        0.0005,
        "occasionally 0.05",
    );
    pin(
        "4c defectors",
        mean(&column(&v, 3)),
        0.0002,
        0.0001,
        "< 0.01",
    );
    // The condition vrb < c < rb: v = 0.5, r = 0.833 — not met at c = 0.25.
    assert!(!analytic::standing_stable(
        1.0, 0.25, 100, 500, 0.025, 0.025
    ));
    pin("r", analytic::standing_r(100, 500), 0.833, 0.001, "0.833");
}
