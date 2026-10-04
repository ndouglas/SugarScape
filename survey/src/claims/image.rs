//! Image scoring (milestone 21): Nowak & Sigmund 1998 (NS98, its figures
//! and Methods) and Leimar & Hammerstein 2001 (LH01), each claim in its
//! source's words, and our own switches (the offset, FAIR23's visibility,
//! how an observer records). NS98 averaged over 10⁷ generations and LH01
//! over 10⁵–10⁶; these runs are shorter, each claim says how long, and use
//! their own seeds (seeds 1–10 unless they say otherwise, whatever `--seeds`
//! says), so their numbers are the plan's measurements. A source's
//! percentage is judged by a one-sample equivalence test of our seeds'
//! window means against it (`equivalent` against the constant), with a
//! margin of 5 points: the most our own estimates move between these
//! windows and the measurements' longer ones (NS98 Fig. 3 at n = 20: 0.863
//! to 0.912; LH01 Fig. 3a: 0.522 to 0.468). Runs that several claims share
//! are memoized per process.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use sugarscape_core::image::analytic::{self, Binary, Start};
use sugarscape_core::image::{
    Class, ImageConfig, ImageWorld, Initial, Offset, Records, Seeded, Strategy,
};
use sugarscape_core::model::{ModelConfig, ModelWorld};

use crate::claim::{all_of, equivalent, greater, range, untestable, Claim, Outcome, Source};
use crate::runner::{model_after, model_preset};

const NS98: &str = "Nowak & Sigmund, Nature 393 (1998) 573–577";
const NS98_METHODS: &str = "Nowak & Sigmund, Nature 393 (1998), Methods";
const LH01: &str = "Leimar & Hammerstein, Proc. R. Soc. B 268 (2001) 745–753";
const OURS: &str = "spec 2026-09-26-image-scoring-design.md; plan Decisions 4, 16 and 22";

/// The equivalence margin for a source's share (see the module comment).
const MARGIN: f64 = 0.05;

fn image(w: &ModelWorld) -> &ImageWorld {
    match w {
        ModelWorld::Image(w) => w,
        _ => unreachable!("an image-scoring world"),
    }
}

fn preset(id: &str, edit: impl FnOnce(&mut ImageConfig)) -> ImageConfig {
    let ModelConfig::Image(mut c) = model_preset(id) else {
        panic!("{id} is not an image-scoring preset")
    };
    edit(&mut c);
    c
}

fn seeds(n: u64) -> Vec<u64> {
    (1..=n).collect()
}

fn mean(v: &[f64]) -> f64 {
    v.iter().sum::<f64>() / v.len() as f64
}

/// Each seed's `f` of a fresh world of `c`, stepped by `f` itself.
fn each<T: Send>(c: &ImageConfig, seeds: &[u64], f: impl Fn(ImageWorld) -> T + Sync) -> Vec<T> {
    model_after(&ModelConfig::Image(c.clone()), seeds, 0, |w| {
        f(image(w).clone())
    })
}

/// Each seed's mean of each of `names` over generations `from..=ticks` (memoized).
fn windows(c: &ImageConfig, names: &[&str], n: u64, ticks: u32, from: usize) -> Arc<Vec<Vec<f64>>> {
    type Cache = Mutex<Vec<(String, u32, usize, Arc<Vec<Vec<f64>>>)>>;
    static CACHE: Cache = Mutex::new(Vec::new());
    let key = format!(
        "{}|{names:?}|{n}",
        serde_json::to_string(c).expect("configs serialize")
    );
    if let Some((.., v)) = CACHE
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .iter()
        .find(|(k, t, f, _)| *k == key && *t == ticks && *f == from)
    {
        return v.clone();
    }
    let v = Arc::new(model_after(
        &ModelConfig::Image(c.clone()),
        &seeds(n),
        ticks,
        |w| {
            let w = image(w);
            names
                .iter()
                .map(|s| mean(&w.stats.series(s).expect("an image series")[from..]))
                .collect()
        },
    ));
    CACHE
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .push((key, ticks, from, v.clone()));
    v
}

/// Each seed's mean of `name` over generations `from..=ticks`.
fn window(c: &ImageConfig, name: &str, n: u64, ticks: u32, from: usize) -> Vec<f64> {
    windows(c, &[name], n, ticks, from)
        .iter()
        .map(|v| v[0])
        .collect()
}

/// Our seeds against a source's share: equivalent within `MARGIN`.
fn about(ours: &[f64], source: f64, who: &str) -> Outcome {
    equivalent(ours, &vec![source; ours.len()], Some(MARGIN), "ours", who)
}

fn share(w: &ImageWorld, f: impl Fn(Strategy) -> bool) -> f64 {
    let a = w.agents();
    a.iter().filter(|x| f(x.strategy)).count() as f64 / a.len() as f64
}

/// 1 where `ok`, else 0.
fn ind(ok: bool) -> f64 {
    f64::from(u8::from(ok))
}

/// Seeds 1–100 of `c`, each until one strategy is fixed (at most 5,000
/// generations): the fixed strategy and its generation (memoized).
fn fixation(c: &ImageConfig) -> Arc<Vec<Option<(Strategy, u64)>>> {
    type Cache = Mutex<Vec<(String, Arc<Vec<Option<(Strategy, u64)>>>)>>;
    static CACHE: Cache = Mutex::new(Vec::new());
    let key = serde_json::to_string(c).expect("configs serialize");
    if let Some((_, v)) = CACHE
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .iter()
        .find(|(k, _)| *k == key)
    {
        return v.clone();
    }
    let v = Arc::new(each(c, &seeds(100), |mut w| {
        for _ in 0..5000 {
            w.step();
            let a = w.agents();
            if a.iter().all(|x| x.strategy == a[0].strategy) {
                return Some((a[0].strategy, w.tick));
            }
        }
        None
    }));
    CACHE
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .push((key, v.clone()));
    v
}

/// Per seed, 1 where some k ≤ 0 was fixed (cooperation won), else 0.
fn cooperation_won(c: &ImageConfig) -> Vec<f64> {
    fixation(c)
        .iter()
        .map(|r| ind(r.is_some_and(|(s, _)| s.cooperative())))
        .collect()
}

/// "k = 0 fixed in 20/100 (median generation 56); some k ≤ 0 in 40/100".
fn fixation_text(c: &ImageConfig) -> String {
    let r = fixation(c);
    let mut t0: Vec<u64> = r
        .iter()
        .flatten()
        .filter(|(s, _)| *s == Strategy::K(0))
        .map(|p| p.1)
        .collect();
    t0.sort_unstable();
    let coop = r.iter().flatten().filter(|(s, _)| s.cooperative()).count();
    format!(
        "k = 0 fixed in {}/100 (median generation {}); some k ≤ 0 in {coop}/100",
        t0.len(),
        t0.get(t0.len() / 2).copied().unwrap_or(0)
    )
}

/// Per seed of Fig. 2 over 10⁵ generations: collapses (the share of k ≤ 0
/// falling from ≥ 0.9 to ≤ 0.1), recoveries, the mean share of k ≤ −4 in
/// cooperative generations (≥ 0.9) and over the 51 generations up to each
/// collapse's last cooperative one (memoized).
fn cycles() -> Arc<Vec<(f64, f64, f64, f64)>> {
    type CycleMeasurements = Arc<Vec<(f64, f64, f64, f64)>>;
    static CACHE: Mutex<Option<CycleMeasurements>> = Mutex::new(None);
    let mut cache = CACHE.lock().unwrap_or_else(|e| e.into_inner());
    if let Some(v) = cache.as_ref() {
        return v.clone();
    }
    let v = Arc::new(each(&preset("ns-fig-2", |_| {}), &seeds(10), |mut w| {
        let (mut coop, mut unc) = (Vec::new(), Vec::new());
        for _ in 0..100_000 {
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
        let base: Vec<f64> = (0..coop.len())
            .filter(|&t| coop[t] >= 0.9)
            .map(|t| unc[t])
            .collect();
        let before: Vec<f64> = collapses
            .iter()
            .map(|&t| mean(&unc[t.saturating_sub(50)..=t]))
            .collect();
        (
            collapses.len() as f64,
            recoveries as f64,
            mean(&base),
            mean(&before),
        )
    }));
    *cache = Some(v.clone());
    v
}

/// Fig. 4: each seed's help rate over generations 1,001–50,000, its most
/// frequent strategy over them, and the three most frequent pooled over
/// the seeds with their shares.
struct Fig4 {
    help: Vec<f64>,
    top: Vec<Strategy>,
    pooled: Vec<(Strategy, f64)>,
}

fn fig_4(id: &str) -> Fig4 {
    let r = each(&preset(id, |_| {}), &seeds(10), |mut w| {
        let mut counts = HashMap::new();
        for t in 1..=50_000u32 {
            w.step();
            if t > 1000 {
                for a in w.agents() {
                    *counts.entry(a.strategy).or_insert(0u64) += 1;
                }
            }
        }
        (
            counts,
            mean(&w.stats.series("help_rate").expect("an image series")[1001..]),
        )
    });
    let top = |counts: &HashMap<Strategy, u64>| -> Vec<(Strategy, f64)> {
        let total: u64 = counts.values().sum();
        let mut v: Vec<(Strategy, f64)> = counts
            .iter()
            .map(|(s, n)| (*s, *n as f64 / total as f64))
            .collect();
        v.sort_by(|a, b| b.1.total_cmp(&a.1).then(a.0.code().cmp(&b.0.code())));
        v.truncate(3);
        v
    };
    let mut all = HashMap::new();
    for (c, _) in &r {
        for (s, n) in c {
            *all.entry(*s).or_insert(0u64) += n;
        }
    }
    Fig4 {
        help: r.iter().map(|p| p.1).collect(),
        top: r.iter().map(|p| top(&p.0)[0].0).collect(),
        pooled: top(&all),
    }
}

fn pooled_text(v: &[(Strategy, f64)]) -> String {
    v.iter()
        .map(|(s, x)| format!("{} {:.1}%", s.label(), 100.0 * x))
        .collect::<Vec<_>>()
        .join("; ")
}

/// A Fig. 4 claim: the help rate against NS98's, and each seed's most
/// frequent strategy against NS98's.
fn fig_4_claim(id: &str, help: f64, first: Strategy) -> Outcome {
    let f = fig_4(id);
    let firsts: Vec<f64> = f.top.iter().map(|s| ind(*s == first)).collect();
    all_of(vec![
        ("help rate".into(), about(&f.help, help, "NS98")),
        (
            format!("most frequent {}", first.label()),
            range(&firsts, 1.0, 1.0, false),
        ),
    ])
    .with(&format!(
        "Help rate {:.4} (seeds 1–10, generations 1,001–50,000); most frequent over them: {}.",
        mean(&f.help),
        pooled_text(&f.pooled)
    ))
}

/// Each seed's share of strategies matching `is` at each generation of `at`.
fn invasion(
    c: &ImageConfig,
    n: u64,
    at: &[u64],
    is: impl Fn(Strategy) -> bool + Sync,
) -> Vec<Vec<f64>> {
    each(c, &seeds(n), |mut w| {
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

/// NS98's Methods game in the simulation: binary discriminators (k 0)
/// against defectors (k 1) at share `x`, no offset, perfect information,
/// n = 100, m = 250 (five of the Methods' rounds, in which everyone plays
/// once, half as donor): each seed's payoff gap after one generation.
fn methods_gap(x: f64, n: u64) -> Vec<f64> {
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
    each(&c, &seeds(n), |mut w| {
        w.step();
        let (mut d, mut e) = (Vec::new(), Vec::new());
        for a in w.agents() {
            if a.strategy == Strategy::Binary(0) {
                d.push(a.payoff);
            } else {
                e.push(a.payoff);
            }
        }
        mean(&d) - mean(&e)
    })
}

/// Fig. 3's cooperative share at group size `n` (seeds 1–10, generations
/// 1,001–20,000).
fn fig_3(n: u32, edit: impl FnOnce(&mut ImageConfig)) -> Vec<f64> {
    let id = match n {
        20 => "ns-fig-3-n20",
        50 => "ns-fig-3-n50",
        _ => "ns-fig-3-n100",
    };
    window(&preset(id, edit), "cooperative", 10, 20_000, 1001)
}

pub fn claims() -> Vec<Claim> {
    vec![
        Claim {
            id: "ns-fig-1.cooperation-wins",
            item: "ns-fig-1",
            source: Source::Book,
            citation: NS98,
            text: "Fig. 1 (n 100, m 125, k −5 … +6 at random): the discriminating strategy k = 0 is fixed (after 166 generations in the run shown) — cooperation wins (some k ≤ 0 fixed in each run)",
            check: |_| {
                let c = preset("ns-fig-1", |_| {});
                range(&cooperation_won(&c), 1.0, 1.0, false).with(&format!(
                    "Seeds 1–100, each to fixation (at most 5,000 generations; all fixed): {}. k = 0 is the most common single winner, but defection (k ≥ 1) wins the rest.",
                    fixation_text(&c)
                ))
            },
        },
        Claim {
            id: "ns-fig-1.more-rounds",
            item: "ns-fig-1",
            source: Source::Book,
            citation: NS98,
            text: "Cooperation is more likely to win the greater the number m of interactions per generation (runs won by some k ≤ 0 at m = 300 against m = 125)",
            check: |_| {
                let more = preset("ns-fig-1", |c| c.rounds = 300);
                let base = preset("ns-fig-1", |_| {});
                greater(&cooperation_won(&more), &cooperation_won(&base), "m 300", "m 125")
                    .with(&format!("m = 300: {}.", fixation_text(&more)))
            },
        },
        Claim {
            id: "ns-fig-2.cycles",
            item: "ns-fig-2",
            source: Source::Book,
            citation: NS98,
            text: "Fig. 2 (m 300, mutation 0.001): endless cycles of cooperation and defection (each run's collapses — the share of k ≤ 0 falling from at least 90% to at most 10% — over 10⁵ generations, at least one)",
            check: |_| {
                let v = cycles();
                let collapses: Vec<f64> = v.iter().map(|r| r.0).collect();
                let recoveries: f64 = v.iter().map(|r| r.1).sum();
                range(&collapses, 1.0, f64::INFINITY, false).with(&format!(
                    "Seeds 1–10 × 10⁵ generations: {} collapses and {recoveries} recoveries (per seed {}–{}).",
                    collapses.iter().sum::<f64>(),
                    collapses.iter().copied().fold(f64::INFINITY, f64::min),
                    collapses.iter().copied().fold(0.0, f64::max)
                ))
            },
        },
        Claim {
            id: "ns-fig-2.cooperators-first",
            item: "ns-fig-2",
            source: Source::Book,
            citation: NS98,
            text: "Fig. 2: unconditional cooperators (k −4, −5) spread in a cooperative population before defectors invade (each seed's share of k ≤ −4 over the 51 generations up to a collapse, against its share in cooperative generations)",
            check: |_| {
                let v = cycles();
                let before: Vec<f64> = v.iter().map(|r| r.3).collect();
                let base: Vec<f64> = v.iter().map(|r| r.2).collect();
                greater(&before, &base, "before collapses", "cooperative generations")
            },
        },
        Claim {
            id: "ns-fig-3-n20.cooperative",
            item: "ns-fig-3-n20",
            source: Source::Book,
            citation: NS98,
            text: "Fig. 3 (ten observers per interaction, m = 10n, mutation 0.001): cooperative strategies (k ≤ 0) 90% of the time at n = 20",
            check: |_| {
                about(&fig_3(20, |_| {}), 0.90, "NS98")
                    .with("Seeds 1–10, generations 1,001–20,000 (NS98: 10⁷); to 100,000 our mean is 0.912 ± 0.056.")
            },
        },
        Claim {
            id: "ns-fig-3-n50.cooperative",
            item: "ns-fig-3-n50",
            source: Source::Book,
            citation: NS98,
            text: "Fig. 3: cooperative strategies 47% of the time at n = 50",
            check: |_| {
                about(&fig_3(50, |_| {}), 0.47, "NS98")
                    .with("Seeds 1–10, generations 1,001–20,000; the seeds spread widely (to 50,000: 0.459 ± 0.131).")
            },
        },
        Claim {
            id: "ns-fig-3-n100.cooperative",
            item: "ns-fig-3-n100",
            source: Source::Book,
            citation: NS98,
            text: "Fig. 3: cooperative strategies 18% of the time at n = 100",
            check: |_| {
                about(&fig_3(100, |_| {}), 0.18, "NS98")
                    .with("Seeds 1–10, generations 1,001–20,000 (to 30,000: 0.215 ± 0.131).")
            },
        },
        Claim {
            id: "ns-fig-3.falls-with-n",
            item: "ns-fig-3",
            source: Source::Book,
            citation: NS98,
            text: "Fig. 3: with a fixed number of observers, cooperation falls as the group grows (n = 20 above n = 50 above n = 100)",
            check: |_| {
                let (a, b, c) = (fig_3(20, |_| {}), fig_3(50, |_| {}), fig_3(100, |_| {}));
                all_of(vec![
                    ("20 > 50".into(), greater(&a, &b, "n 20", "n 50")),
                    ("50 > 100".into(), greater(&b, &c, "n 50", "n 100")),
                ])
            },
        },
        Claim {
            id: "ns-fig-4a.help-and-mode",
            item: "ns-fig-4a",
            source: Source::Book,
            citation: NS98,
            text: "Fig. 4a (AND strategies, perfect information, m 500): 55% of interactions cooperative; (k 0, h 1) the most frequent strategy",
            check: |_| fig_4_claim("ns-fig-4a", 0.55, Strategy::And { k: 0, h: 1 }),
        },
        Claim {
            id: "ns-fig-4b.help-and-mode",
            item: "ns-fig-4b",
            source: Source::Book,
            citation: NS98,
            text: "Fig. 4b (AND, observers, n 20, m 200): 57% cooperative; (k 0, h 4) the most frequent",
            check: |_| fig_4_claim("ns-fig-4b", 0.57, Strategy::And { k: 0, h: 4 }),
        },
        Claim {
            id: "ns-fig-4c.help-and-mode",
            item: "ns-fig-4c",
            source: Source::Book,
            citation: NS98,
            text: "Fig. 4c (OR strategies, perfect information, m 500): 70% cooperative; the defectors (k 6, h −5) the most frequent single strategy",
            check: |_| fig_4_claim("ns-fig-4c", 0.70, Strategy::Or { k: 6, h: -5 }),
        },
        Claim {
            id: "ns-fig-4d.help-and-mode",
            item: "ns-fig-4d",
            source: Source::Book,
            citation: NS98,
            text: "Fig. 4d (OR, observers, n 20, m 200): 80% cooperative; the defectors (k 6, h −5) the most frequent",
            check: |_| fig_4_claim("ns-fig-4d", 0.80, Strategy::Or { k: 6, h: -5 }),
        },
        Claim {
            id: "ns-own-only.help",
            item: "ns-own-only",
            source: Source::Book,
            citation: NS98,
            text: "Strategies that only consider their own image give less than 0.1% cooperation (each seed's help rate below 0.001)",
            check: |_| {
                let c = preset("ns-own-only", |_| {});
                range(&window(&c, "help_rate", 10, 20_000, 1001), 0.0, 0.001, false).with(
                    "Seeds 1–10, generations 1,001–20,000. The floor is mutation: mutants are uniform over h −5 … +6, and the 5 of 12 with h ≥ 1 help at a generation's start.",
                )
            },
        },
        Claim {
            id: "ns-rounds.two-interactions",
            item: "ns-rounds",
            source: Source::Book,
            citation: NS98,
            text: "It suffices that each player is chosen only for about 2 interactions per life-time (Fig. 2's settings at m = n = 100: cooperative strategies prevail, above half the time in each seed)",
            check: |_| {
                let at = |m: u32| window(&preset("ns-fig-2", |c| c.rounds = m), "cooperative", 10, 20_000, 1001);
                range(&at(100), 0.5, 1.0, false).with(&format!(
                    "Seeds 1–10, generations 1,001–20,000: cooperative strategies {:.3} at m = 100 and {:.3} at m = 200 (four interactions per lifetime); the sweep ns-rounds has the whole curve.",
                    mean(&at(100)),
                    mean(&at(200))
                ))
            },
        },
        Claim {
            id: "ns-methods.rounds",
            item: "ns-methods",
            source: Source::Book,
            citation: NS98_METHODS,
            text: "Discriminators are stable against defectors if the mean number of rounds 1/(1 − w) exceeds (bq + c)/(bq − c): about 1.2 rounds at b = 1, c = 0.1, q = 1 (stability, from the w-weighted payoff sums, at mean rounds 1.00 … 2.00 in steps of 0.01, agreeing with 1.2 away from 1.20–1.25)",
            check: |_| {
                let m = Binary::ns98();
                let agree: Vec<f64> = (100..=200)
                    .map(|i| f64::from(i) / 100.0)
                    .filter(|r| !(1.2..=1.25).contains(r))
                    .map(|r| ind(m.stable(1.0 - 1.0 / r) == (r > 1.2)))
                    .collect();
                range(&agree, 1.0, 1.0, false).with(&format!(
                    "The exact threshold (bq + c)/(bq − c) = {:.4}.",
                    m.min_rounds()
                ))
            },
        },
        Claim {
            id: "ns-methods.q",
            item: "ns-methods",
            source: Source::Book,
            citation: NS98_METHODS,
            text: "Discriminators can be stable only if q > c/b (at b = 1, c = 0.1, q = 0.01 … 1: some continuation w < 1 stabilizes them exactly when q > 0.1)",
            check: |_| {
                let agree: Vec<f64> = (1..=100)
                    .map(|i| {
                        let m = Binary {
                            q: f64::from(i) / 100.0,
                            ..Binary::ns98()
                        };
                        let some_w = (1..1000).any(|j| m.stable(f64::from(j) / 1000.0));
                        ind(some_w == (m.q > 0.1 + 1e-12))
                    })
                    .collect();
                range(&agree, 1.0, 1.0, false)
            },
        },
        Claim {
            id: "ns-methods.cooperators",
            item: "ns-methods",
            source: Source::Book,
            citation: NS98_METHODS,
            text: "With unconditional cooperators, defectors win below the discriminator frequency x = c(2 − w)/(bwq) and cooperators do better above it (Dc − De changes sign there, at w = 0.5 … 0.95)",
            check: |_| {
                let m = Binary::ns98();
                let agree: Vec<f64> = (10..=19)
                    .map(|i| {
                        let w = f64::from(i) / 20.0;
                        let x = m.cooperator_equilibrium(w);
                        let d = |x: f64| m.cooperators_over_defectors(w, x);
                        ind(d(x).abs() < 1e-12 && d(x - 0.01) < 0.0 && d(x + 0.01) > 0.0)
                    })
                    .collect();
                range(&agree, 1.0, 1.0, false).with(&format!(
                    "At w = 0.9: x = {:.4}.",
                    m.cooperator_equilibrium(0.9)
                ))
            },
        },
        Claim {
            id: "ns-methods.threshold-in-simulation",
            item: "ns-methods",
            source: Source::Book,
            citation: NS98_METHODS,
            text: "Discriminators do better than defectors once their frequency exceeds x_min (0.123 over five rounds): in the simulation at x = 0.14, discriminators' mean payoff above defectors' (one generation, n 100, m 250, no offset, seeds 1–2,000)",
            check: |_| {
                let gap = methods_gap(0.14, 2000);
                let zero = vec![0.0; gap.len()];
                greater(&gap, &zero, "discriminators − defectors", "0").with(&format!(
                    "Analytic x_min over five rounds {:.4}; the simulated mean gap {:+.4} at 0.14, {:+.4} at 0.20. The Methods' rounds (everyone plays once, half as donor) are not NS98's random pairs, and the simulated threshold is higher (the first positive gap at 0.16 in 0.01 steps).",
                    Binary::ns98().x_min_fixed(5).unwrap_or(f64::NAN),
                    mean(&gap),
                    mean(&methods_gap(0.20, 2000))
                ))
            },
        },
        Claim {
            id: "ns-universal.constant",
            item: "ns-universal",
            source: Source::Book,
            citation: NS98_METHODS,
            text: "Everyone k = 0 with unbounded scores: the maximum initial fraction below 0 that still converges to all-out cooperation is 0.7380294688360… (a universal constant; the start is not stated) — to every printed digit with the negatives at −1 and the rest never falling below 0, and within 10⁻¹⁰ with the rest at +100 … +300",
            check: |_| {
                let rests = [None, Some(100), Some(150), Some(200), Some(300)];
                let t: Vec<f64> = rests
                    .iter()
                    .map(|&rest| analytic::universal_threshold(Start::AtMinusOne { rest }))
                    .collect();
                let others = [
                    ("rest at 0", Start::AtMinusOne { rest: Some(0) }),
                    ("rest at +1", Start::AtMinusOne { rest: Some(1) }),
                    ("rest at +2", Start::AtMinusOne { rest: Some(2) }),
                    ("negatives over −1 … −5", Start::Spread { below: 5 }),
                ];
                let others = others
                    .iter()
                    .map(|(name, s)| format!("{name} {:.4}", analytic::universal_threshold(*s)))
                    .collect::<Vec<_>>()
                    .join(", ");
                all_of(vec![
                    ("out of reach".into(), range(&[t[0]; 5], 0.738_029_468_836_0, 0.738_029_468_836_1, false)),
                    ("+100 … +300".into(), range(&t[1..].to_vec().repeat(2), 0.738_029_468_7, 0.738_029_468_9, false)),
                ])
                .with(&format!(
                    "Out of reach: {:.16}. Other starts give other constants: {others}.",
                    t[0]
                ))
            },
        },
        Claim {
            id: "lh-fig-1a.h-invades",
            item: "lh-fig-1a",
            source: Source::Book,
            citation: LH01,
            text: "Fig. 1a (100 groups of 100, m 500, c 0.25, p 0.9): h = 1 invades a population of k = 0 (from 1% of each group; above half by generation 150)",
            check: |_| {
                let v = invasion(&preset("lh-fig-1a", |_| {}), 10, &[50, 150], |s| s == Strategy::H(1));
                range(&column(&v, 1), 0.5, 1.0, false).with(&format!(
                    "Seeds 1–10: h = 1 at {:.3} by generation 50 and {:.3} by 150 — faster than LH01's figure (about 0.8 at 150).",
                    mean(&column(&v, 0)),
                    mean(&column(&v, 1))
                ))
            },
        },
        Claim {
            id: "lh-fig-1b.h-invades",
            item: "lh-fig-1b",
            source: Source::Book,
            citation: LH01,
            text: "Fig. 1b (execution errors 0.05): h = 1 invades (k 0, h 1) within the figure's 150 generations but does not wipe it out (h = 1 between 2% and 99% at generation 150)",
            check: |_| {
                let v = invasion(&preset("lh-fig-1b", |_| {}), 10, &[150, 500, 1000], |s| s == Strategy::H(1));
                range(&column(&v, 0), 0.02, 0.99, false).with(&format!(
                    "Seeds 1–10: h = 1 at {:.3} by generation 150, {:.3} by 500, {:.3} by 1,000 — it invades, an order of magnitude more slowly than LH01 show.",
                    mean(&column(&v, 0)),
                    mean(&column(&v, 1)),
                    mean(&column(&v, 2))
                ))
            },
        },
        Claim {
            id: "lh-fig-2a.help",
            item: "lh-fig-2a",
            source: Source::Book,
            citation: LH01,
            text: "Fig. 2a (AND strategies, one group of 100, m 500, c 0.25, mutation 0.001, no errors): help in 39% of rounds",
            check: |_| {
                let c = preset("lh-fig-2a", |_| {});
                about(&window(&c, "help_rate", 10, 100_000, 1001), 0.39, "LH01")
                    .with("Seeds 1–10, generations 1,001–100,000 (LH01: 10⁶).")
            },
        },
        Claim {
            id: "lh-fig-2b.help",
            item: "lh-fig-2b",
            source: Source::Book,
            citation: LH01,
            text: "Fig. 2b (the island model: 100 groups, p 0.9, execution errors 0.02): with drift limited, image scoring fades — help in 9% of rounds",
            check: |_| {
                let c = preset("lh-fig-2b", |_| {});
                about(&window(&c, "help_rate", 10, 3000, 1001), 0.09, "LH01").with(
                    "Seeds 1–10, generations 1,001–3,000 (LH01: 10⁵); to 5,000: 0.441; to 20,000: 0.345 ± 0.197, runs from 0.02 to 0.54. Cooperative AND strategies persist in most runs.",
                )
            },
        },
        Claim {
            id: "lh-fig-2c.help",
            item: "lh-fig-2c",
            source: Source::Book,
            citation: LH01,
            text: "Fig. 2c (p 0.5): help in 2% of rounds, mainly a consequence of execution errors",
            check: |_| {
                let c = preset("lh-fig-2c", |_| {});
                about(&window(&c, "help_rate", 10, 3000, 1001), 0.02, "LH01")
                    .with("Seeds 1–10, generations 1,001–3,000; to 5,000: 0.154; to 20,000: 0.094 ± 0.125.")
            },
        },
        Claim {
            id: "lh-fig-2b.below-one-group",
            item: "lh-fig-2b",
            source: Source::Book,
            citation: LH01,
            text: "Limited dispersal undoes image scoring: with the same errors (0.02), one group of 100 helps more than the island model with p = 0.9",
            check: |_| {
                let one = preset("lh-fig-2a", |c| c.execution_error = 0.02);
                let islands = preset("lh-fig-2b", |_| {});
                greater(
                    &window(&one, "help_rate", 10, 3000, 1001),
                    &window(&islands, "help_rate", 10, 3000, 1001),
                    "one group",
                    "islands",
                )
                .with("Seeds 1–10, generations 1,001–3,000 (to 5,000: 0.351 against 0.441). Help is not monotone in gene flow either (the sweep lh-gene-flow: p = 1, isolated groups, 0.27; p = 0.8, 0.50).")
            },
        },
        Claim {
            id: "lh-fig-3a.help",
            item: "lh-fig-3a",
            source: Source::Book,
            citation: LH01,
            text: "Fig. 3a (c 0.1, u₀ 5, p 0.5, execution errors 0.02): help in 45% of rounds",
            check: |_| {
                let c = preset("lh-fig-3a", |_| {});
                about(&window(&c, "help_rate", 10, 3000, 1001), 0.45, "LH01")
                    .with("Seeds 1–10, generations 1,001–3,000 (LH01: 2 × 10⁵); to 10,000: 0.468 ± 0.060.")
            },
        },
        Claim {
            id: "lh-fig-3b.help-and-q",
            item: "lh-fig-3b",
            source: Source::Book,
            citation: LH01,
            text: "Fig. 3b–c (with q strategies): help in 15% of rounds, q strategies 12% of the population",
            check: |_| {
                let c = preset("lh-fig-3b", |_| {});
                let v = windows(&c, &["help_rate", "q"], 10, 3000, 1001);
                all_of(vec![
                    ("help".into(), about(&column(&v, 0), 0.15, "LH01")),
                    ("q share".into(), about(&column(&v, 1), 0.12, "LH01")),
                ])
                .with("Seeds 1–10, generations 1,001–3,000; to 10,000: help 0.148 ± 0.076, q 0.177 ± 0.111.")
            },
        },
        Claim {
            id: "lh-fig-4a.standing-invades",
            item: "lh-fig-4a",
            source: Source::Book,
            citation: LH01,
            text: "Fig. 4a (binary discriminators, 1% standing, execution errors 0.05): standing invades and takes over (above half by generation 1,000)",
            check: |_| {
                let v = invasion(&preset("lh-fig-4a", |_| {}), 10, &[500, 1000], |s| s == Strategy::Standing);
                range(&column(&v, 1), 0.5, 1.0, false).with(&format!(
                    "Seeds 1–10: standing {:.3} at generation 500, {:.3} at 1,000.",
                    mean(&column(&v, 0)),
                    mean(&column(&v, 1))
                ))
            },
        },
        Claim {
            id: "lh-fig-4b.standing-invades",
            item: "lh-fig-4b",
            source: Source::Book,
            citation: LH01,
            text: "Fig. 4b (execution and perception errors 0.025): standing still invades (from 1% of each group to above 10% by generation 500)",
            check: |_| {
                let v = invasion(&preset("lh-fig-4b", |_| {}), 5, &[250, 500], |s| s == Strategy::Standing);
                range(&column(&v, 1), 0.1, 1.0, false).with(&format!(
                    "Seeds 1–5: standing {:.3} at generation 250, {:.3} at 500 (the book test: 0.746 at 1,000; seeds 1–10: 0.767, 8 of 10 above half). A generation with perception errors updates five million records, so the survey stops at 500.",
                    mean(&column(&v, 0)),
                    mean(&column(&v, 1))
                ))
            },
        },
        Claim {
            id: "lh-fig-4b.condition",
            item: "lh-fig-4b",
            source: Source::Book,
            citation: LH01,
            text: "With perception errors standing is a strict best reply to itself only if vrb < c < rb (v = ε/(e + ε), r = (m − 1)/(n + m − 1)); at Fig. 4's parameters (n 100, m 500, b 1, e = ε = 0.025) the condition is not fulfilled for costs up to c = 0.25",
            check: |_| {
                let unmet: Vec<f64> = [0.05, 0.1, 0.15, 0.2, 0.25]
                    .iter()
                    .map(|&c| ind(!analytic::standing_stable(1.0, c, 100, 500, 0.025, 0.025)))
                    .collect();
                range(&unmet, 1.0, 1.0, false).with(&format!(
                    "r = {:.4}, v = {:.2}, vrb = {:.3}: the condition needs c above vrb. With e = 0.04, ε = 0.01 it holds at c = 0.25 ({}). Standing invades anyway (lh-fig-4b.standing-invades).",
                    analytic::standing_r(100, 500),
                    analytic::standing_v(0.025, 0.025),
                    analytic::standing_v(0.025, 0.025) * analytic::standing_r(100, 500),
                    analytic::standing_stable(1.0, 0.25, 100, 500, 0.04, 0.01)
                ))
            },
        },
        Claim {
            id: "lh-fig-4c.long-run",
            item: "lh-fig-4c",
            source: Source::Book,
            citation: LH01,
            text: "Fig. 4c (all four binary strategies, errors 0.025, mutation 0.0001): standing dominates, cooperators stay at appreciable frequencies, discriminators reach 5% only occasionally, defectors stay below 1%",
            check: |_| {
                untestable("Pinned by the book test lh01_fig_4c_standing_persists_with_cooperators instead (55 s): from a uniform start standing needs about 1,000 generations to dominate, and a generation with perception errors updates five million records, so no subset stays near 30 s (seeds 1–5, generations 401–600 took 33 s and standing was still at 0.31). The book test (seeds 1–3, generations 1,001–1,500): standing 0.531, cooperators 0.389, discriminators 0.080, defectors 0.0002 — each as LH01 describe.")
            },
        },
        Claim {
            id: "ns-no-offset.cooperation-wins-more",
            item: "ns-no-offset",
            source: Source::App,
            citation: OURS,
            text: "Ours: NS98's \"we add 0.1 in each interaction\" (to both players, LH01 and FAIR23) weakens selection, and cooperation wins Fig. 1 more often without it (runs won by some k ≤ 0)",
            check: |_| {
                let none = preset("ns-no-offset", |_| {});
                let both = preset("ns-fig-1", |_| {});
                greater(&cooperation_won(&none), &cooperation_won(&both), "no offset", "offset")
                    .with(&format!("Seeds 1–100 to fixation, without the offset: {}. With Fig. 2's settings: 0.777 cooperative against 0.672.", fixation_text(&none)))
            },
        },
        Claim {
            id: "ns-fig-3.fair23-visibility",
            item: "ns-fig-3",
            source: Source::App,
            citation: OURS,
            text: "Ours: FAIR23's fixed visibility (each member sees an interaction with probability 0.1) is not NS98's ten observers: at n = 20 and 50 (1.8 and 4.8 observers) it gives less cooperation than ten",
            check: |_| {
                let (a20, f20) = (fig_3(20, |_| {}), fig_3(20, |c| c.observers = 1.8));
                let (a50, f50) = (fig_3(50, |_| {}), fig_3(50, |c| c.observers = 4.8));
                all_of(vec![
                    ("n 20".into(), greater(&a20, &f20, "ten observers", "visibility 0.1")),
                    ("n 50".into(), greater(&a50, &f50, "ten observers", "visibility 0.1")),
                ])
                .with(&format!(
                    "Seeds 1–10, generations 1,001–20,000: visibility 0.1 gives {:.3} at n = 20 and {:.3} at n = 50 (ten observers: {:.3}, {:.3}); at n = 100 it is 9.8 observers, NS98's ten.",
                    mean(&f20),
                    mean(&f50),
                    mean(&a20),
                    mean(&a50)
                ))
            },
        },
        Claim {
            id: "ns-fig-3.records",
            item: "ns-fig-3",
            source: Source::App,
            citation: OURS,
            text: "Ours: Fig. 3's group-size effect needs each observer's own tally (records: tally, FAIR23's); if one sighting revealed the donor's whole score (records: score, the spec's first reading) n = 20 and n = 50 would cooperate alike",
            check: |_| {
                let score = |c: &mut ImageConfig| c.records = Records::Score;
                let (s20, s50) = (fig_3(20, score), fig_3(50, score));
                all_of(vec![
                    ("tally: 20 > 50".into(), greater(&fig_3(20, |_| {}), &fig_3(50, |_| {}), "n 20", "n 50")),
                    ("score: 20 ≈ 50".into(), equivalent(&s20, &s50, Some(MARGIN), "n 20", "n 50")),
                ])
                .with(&format!(
                    "Seeds 1–10, generations 1,001–20,000, records: score: {:.3} at n = 20, {:.3} at n = 50 (0.923 at n = 100).",
                    mean(&s20),
                    mean(&s50)
                ))
            },
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::claim::Verdict;

    #[test]
    fn about_is_an_equivalence_test_against_the_source() {
        let near: Vec<f64> = (0..10).map(|i| 0.88 + 0.004 * f64::from(i)).collect();
        assert_eq!(about(&near, 0.90, "NS98").verdict, Verdict::Holds);
        let far: Vec<f64> = near.iter().map(|x| x - 0.3).collect();
        assert_eq!(about(&far, 0.90, "NS98").verdict, Verdict::Fails);
        // Noisy seeds around the source: neither shown equal nor different.
        let noisy: Vec<f64> = (0..10)
            .map(|i| 0.47 + if i % 2 == 0 { 0.2 } else { -0.2 })
            .collect();
        assert_eq!(about(&noisy, 0.47, "NS98").verdict, Verdict::Weak);
    }

    #[test]
    fn presets_and_seeded_starts_are_image_configs() {
        let c = preset("lh-fig-4a", |_| {});
        assert_eq!(c.groups, 100);
        assert!(
            matches!(c.initial, Initial::Seeded(ref s) if s.invader == Some(Strategy::Standing))
        );
        assert_eq!(preset("ns-fig-2", |c| c.rounds = 100).rounds, 100);
        assert_eq!(seeds(3), vec![1, 2, 3]);
    }

    #[test]
    fn a_short_run_of_every_window_is_finite() {
        let c = preset("ns-fig-1", |c| c.mutation = 0.001);
        let v = windows(&c, &["help_rate", "cooperative"], 5, 20, 1);
        assert_eq!(v.len(), 5);
        assert!(v
            .iter()
            .flatten()
            .all(|x| x.is_finite() && (0.0..=1.0).contains(x)));
    }
}
