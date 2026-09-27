//! Axelrod's norms and metanorms (milestone 20): his Figs. 2 and 4 and his
//! dominance variant, and Galán & Izquierdo's long runs, departures and
//! readings. Runs are memoized per process, keyed by the config, the seeds
//! and the generations at which they are read.

use std::sync::{Arc, Mutex};

use sugarscape_core::norms::{AllEqual, GroupsConfig, NormsConfig, NormsWorld, Refill, Selection};

use crate::claim::{greater, Claim, Outcome, Source, Verdict};

const AXELROD: &str = "Axelrod 1986, APSR 80(4)";
const GI: &str = "Galán & Izquierdo 2005, JASSS 8(3) 2";
/// Seeds for Axelrod's 100-generation claims, and for the long runs.
const SHORT: u64 = 100;
const LONG: u64 = 50;

/// A generation's reading.
#[derive(Clone, Copy, Debug)]
struct Reading {
    boldness: f64,
    vengefulness: f64,
    established: bool,
    collapsed: bool,
    strong_boldness: f64,
    weak_boldness: f64,
    strong_vengefulness: f64,
    weak_vengefulness: f64,
}

/// Axelrod's defaults with `edit`, each seed read at every generation in `at`.
fn runs(seeds: u64, at: &[u64], edit: impl FnOnce(&mut NormsConfig)) -> Arc<Vec<Vec<Reading>>> {
    type Cache = Mutex<Vec<(String, u64, Vec<u64>, Arc<Vec<Vec<Reading>>>)>>;
    static CACHE: Cache = Mutex::new(Vec::new());
    let mut c = NormsConfig::default();
    edit(&mut c);
    let key = serde_json::to_string(&c).expect("configs serialize");
    if let Some((_, _, _, v)) = CACHE
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .iter()
        .find(|(k, s, a, _)| *k == key && *s == seeds && a == at)
    {
        return v.clone();
    }
    let v: Vec<Vec<Reading>> = std::thread::scope(|scope| {
        let handles: Vec<_> = (1..=seeds)
            .map(|seed| {
                let c = &c;
                scope.spawn(move || {
                    let mut w = NormsWorld::new(c.clone(), seed).expect("a valid config");
                    at.iter()
                        .map(|&t| {
                            w.run((t - w.tick) as u32);
                            let s = w.stats.latest().expect("a generation");
                            Reading {
                                boldness: s.mean_boldness,
                                vengefulness: s.mean_vengefulness,
                                established: s.established == 1,
                                collapsed: s.collapsed == 1,
                                strong_boldness: s.strong_boldness,
                                weak_boldness: s.weak_boldness,
                                strong_vengefulness: s.strong_vengefulness,
                                weak_vengefulness: s.weak_vengefulness,
                            }
                        })
                        .collect()
                })
            })
            .collect();
        handles
            .into_iter()
            .map(|h| h.join().expect("a run"))
            .collect()
    });
    let v = Arc::new(v);
    CACHE
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .push((key, seeds, at.to_vec(), v.clone()));
    v
}

/// How many runs satisfy `pred` at reading `k`.
fn count(runs: &[Vec<Reading>], k: usize, pred: impl Fn(&Reading) -> bool) -> usize {
    runs.iter().filter(|r| pred(&r[k])).count()
}

fn outcome(holds: bool, measured: String) -> Outcome {
    Outcome {
        verdict: if holds {
            Verdict::Holds
        } else {
            Verdict::Fails
        },
        measured,
        detail: String::new(),
    }
}

fn meta(c: &mut NormsConfig) {
    c.metanorms = true;
}

fn groups(metanorms: bool) -> impl FnOnce(&mut NormsConfig) {
    move |c| {
        c.metanorms = metanorms;
        c.groups = GroupsConfig {
            enabled: true,
            ..GroupsConfig::default()
        };
    }
}

pub fn claims() -> Vec<Claim> {
    vec![
        Claim {
            id: "norms.table-1",
            item: "ax-norms",
            source: Source::Book,
            citation: AXELROD,
            text: "Table 1: Lee (boldness 2/7, vengefulness 4/7) — one defection (3), punished once (−9), hurt by 36 defections (−36), punishing 9 (−18) — scores −60 (a check that the default payoff constants T, P, H and E combine to −60 under Lee's tallies, not a played generation: the core's `table_1s_arithmetic` test plays a hand-built world and checks the same arithmetic against the engine's own counts)",
            check: |_| {
                let c = NormsConfig::default();
                let lee = c.temptation + c.punishment + 36.0 * c.hurt + 9.0 * c.enforcement;
                outcome(
                    lee == -60.0,
                    format!("{lee} from the default payoff constants (not a played generation)"),
                )
            },
        },
        Claim {
            id: "norms.fig-2.spread",
            item: "ax-norms",
            source: Source::Book,
            citation: AXELROD,
            text: "Fig. 2 (5 runs × 100 generations): three outcomes — low boldness with moderate vengefulness, both low, and high boldness with almost no vengefulness; the norm is not established (each outcome in ≥ 10 % of runs at generation 100, established in ≤ 10 %)",
            check: |_| {
                let r = runs(SHORT, &[100], |_| {});
                let a = count(&r, 0, |x| x.boldness < 2.0 / 7.0 && x.vengefulness >= 3.0 / 7.0);
                let b = count(&r, 0, |x| x.boldness < 2.0 / 7.0 && x.vengefulness < 3.0 / 7.0);
                let h = count(&r, 0, |x| x.boldness >= 4.0 / 7.0 && x.vengefulness < 2.0 / 7.0);
                let e = count(&r, 0, |x| x.established);
                let n = r.len();
                outcome(
                    [a, b, h].iter().all(|&k| 10 * k >= n) && 10 * e <= n,
                    format!("of {n}: low B, moderate V {a}; both low {b}; high B, low V {h}; established {e}"),
                )
            },
        },
        Claim {
            id: "norms.fig-4.established",
            item: "ax-metanorms",
            source: Source::Book,
            citation: AXELROD,
            text: "Fig. 4: 'In all five runs a norm against defection was established' (established in ≥ 80 % of runs at generation 100)",
            check: |_| {
                let r = runs(SHORT, &[100], meta);
                let e = count(&r, 0, |x| x.established);
                outcome(10 * e >= 8 * r.len(), format!("established in {e} of {}", r.len()))
            },
        },
        Claim {
            id: "norms.dominance.free-riders",
            item: "ax-dominance",
            source: Source::Book,
            citation: AXELROD,
            text: "Dominance: 'Without metanorms, even members of the stronger group tend to be free riders … low vengefulness and high boldness in both groups' (both groups' mean boldness ≥ 0.6 and vengefulness ≤ 0.2 at generation 100)",
            check: |_| {
                let r = runs(SHORT, &[100], groups(false));
                let m = |f: fn(&Reading) -> f64| r.iter().map(|x| f(&x[0])).sum::<f64>() / r.len() as f64;
                let (sb, wb, sv, wv) = (
                    m(|x| x.strong_boldness),
                    m(|x| x.weak_boldness),
                    m(|x| x.strong_vengefulness),
                    m(|x| x.weak_vengefulness),
                );
                outcome(
                    sb >= 0.6 && wb >= 0.6 && sv <= 0.2 && wv <= 0.2,
                    format!("boldness strong {sb:.2}, weak {wb:.2}; vengefulness strong {sv:.2}, weak {wv:.2}"),
                )
            },
        },
        Claim {
            id: "norms.dominance.metanorms",
            item: "ax-dominance-metanorms",
            source: Source::Book,
            citation: AXELROD,
            text: "Dominance: 'When metanorms are added, it becomes relatively easier for the strong group to keep the weak group from being bold' (the weak group's boldness at generation 100 lower with metanorms)",
            check: |_| {
                let weak = |m: bool| {
                    runs(SHORT, &[100], groups(m))
                        .iter()
                        .map(|x| x[0].weak_boldness)
                        .collect::<Vec<_>>()
                };
                greater(&weak(false), &weak(true), "without metanorms", "with")
            },
        },
        Claim {
            id: "norms.gi.norms-collapse",
            item: "norms-horizon",
            source: Source::Comment,
            citation: GI,
            text: "the norms game: 'the norm collapses almost always' in the long run (collapsed in ≥ 90 % of runs by 10⁵ generations)",
            check: |_| {
                let r = runs(LONG, &[1_000, 100_000], |_| {});
                let (a, b) = (count(&r, 0, |x| x.collapsed), count(&r, 1, |x| x.collapsed));
                outcome(10 * b >= 9 * r.len(), format!("collapsed in {a} of {} by 1 000, {b} by 10⁵", r.len()))
            },
        },
        Claim {
            id: "norms.gi.metanorms-decay",
            item: "gi-metanorms-long",
            source: Source::Comment,
            citation: GI,
            text: "the metanorms game: 'Even though after 100 generations the norm is almost always established, as time goes by … the norm usually collapses' (established at 100 in ≥ 80 %; collapsed by 10⁵ in more runs than at 1 000)",
            check: |_| {
                let r = runs(LONG, &[100, 1_000, 100_000], meta);
                let e = count(&r, 0, |x| x.established);
                let (c1, c2) = (count(&r, 1, |x| x.collapsed), count(&r, 2, |x| x.collapsed));
                outcome(
                    10 * e >= 8 * r.len() && c2 > c1,
                    format!("of {}: established at 100 in {e}; collapsed by 1 000 in {c1}, by 10⁵ in {c2}", r.len()),
                )
            },
        },
        Claim {
            id: "norms.gi.metanorms-million",
            item: "gi-metanorms-long",
            source: Source::Comment,
            citation: GI,
            text: "at their own horizon, 10⁶ generations, the metanorms game has usually collapsed (collapsed in more than half of 10 runs; each run takes minutes)",
            check: |_| {
                let r = runs(10, &[1_000_000], meta);
                let c = count(&r, 0, |x| x.collapsed);
                outcome(2 * c > r.len(), format!("collapsed in {c} of {} at 10⁶", r.len()))
            },
        },
        Claim {
            id: "norms.gi.low-mutation",
            item: "gi-low-mutation",
            source: Source::Comment,
            citation: GI,
            text: "Fig. 7: with mutation 0.001 'the system reaches the states of norm collapse much more quickly' (collapsed by 2·10⁴ generations in more runs than at mutation 0.01)",
            check: |_| {
                let low = runs(LONG, &[20_000], |c| {
                    meta(c);
                    c.mutation = 0.001;
                });
                let base = runs(LONG, &[20_000], meta);
                let (a, b) = (count(&low, 0, |x| x.collapsed), count(&base, 0, |x| x.collapsed));
                outcome(a > b, format!("collapsed by 2·10⁴: {a} of {} at 0.001, {b} at 0.01", low.len()))
            },
        },
        Claim {
            id: "norms.gi.mild-metanorms",
            item: "gi-mild-metanorms",
            source: Source::Comment,
            citation: GI,
            text: "Fig. 9: with ME −0.2 and MP −0.9 'the norm quickly collapses and such state is sustained … Axelrod's conclusions are reversed' (collapsed in ≥ 90 % by 2·10⁴ generations)",
            check: |_| {
                let r = runs(LONG, &[1_000, 20_000], |c| {
                    meta(c);
                    c.meta_enforcement = -0.2;
                    c.meta_punishment = -0.9;
                });
                let (a, b) = (count(&r, 0, |x| x.collapsed), count(&r, 1, |x| x.collapsed));
                outcome(10 * b >= 9 * r.len(), format!("collapsed in {a} of {} by 1 000, {b} by 2·10⁴", r.len()))
            },
        },
        Claim {
            id: "norms.gi.temptation-10",
            item: "gi-temptation-10",
            source: Source::Comment,
            citation: GI,
            text: "Fig. 11: with T = 10 'the norm is clearly established in almost all runs' (established in ≥ 90 % at 10⁵ generations)",
            check: |_| {
                let r = runs(LONG, &[100_000], |c| {
                    meta(c);
                    c.temptation = 10.0;
                });
                let e = count(&r, 0, |x| x.established);
                outcome(10 * e >= 9 * r.len(), format!("established in {e} of {} at 10⁵", r.len()))
            },
        },
        Claim {
            id: "norms.gi.selection",
            item: "norms-selection",
            source: Source::Comment,
            citation: GI,
            text: "Fig. 12: under a random tournament, a roulette wheel or above-the-mean selection the norm collapses sooner than under Axelrod's rule (each collapsed by 2·10⁴ in more runs)",
            check: |_| {
                let at = |s: Selection| {
                    count(
                        &runs(LONG, &[20_000], move |c| {
                            meta(c);
                            c.selection = s;
                        }),
                        0,
                        |x| x.collapsed,
                    )
                };
                let (ax, t, r, a) = (
                    at(Selection::Axelrod),
                    at(Selection::Tournament),
                    at(Selection::Roulette),
                    at(Selection::Average),
                );
                outcome(
                    t > ax && r > ax && a > ax,
                    format!("collapsed by 2·10⁴ of {LONG}: Axelrod {ax}, tournament {t}, roulette {r}, average {a}"),
                )
            },
        },
        Claim {
            id: "norms.gi.ties",
            item: "norms-readings",
            source: Source::Comment,
            citation: GI,
            text: "note 4: how Axelrod's rule treats a generation in which every payoff ties 'can alter the long-term results significantly' (at 10⁵ generations, ties kept (one offspring each) vs ties drifting (two each, half removed): collapsed in at least 20 percentage points fewer runs)",
            check: |_| {
                let at = |a: AllEqual| {
                    count(
                        &runs(LONG, &[100_000], move |c| {
                            meta(c);
                            c.all_equal = a;
                        }),
                        0,
                        |x| x.collapsed,
                    )
                };
                let (drift, keep) = (at(AllEqual::Drift), at(AllEqual::Keep));
                outcome(
                    (drift as f64 - keep as f64) >= 0.2 * LONG as f64,
                    format!("collapsed at 10⁵ of {LONG}: ties drift {drift}, ties kept {keep}"),
                )
            },
        },
        Claim {
            id: "norms.readings.refill",
            item: "norms-readings",
            source: Source::Book,
            citation: AXELROD,
            text: "'For convenience, the number of offspring is adjusted to maintain a constant population of 20' — how does not matter (random and ranked refill collapse in about as many runs by 10⁵ generations: within 20 percentage points)",
            check: |_| {
                let at = |r: Refill| {
                    count(
                        &runs(LONG, &[100_000], move |c| {
                            meta(c);
                            c.refill = r;
                        }),
                        0,
                        |x| x.collapsed,
                    )
                };
                let (random, ranked) = (at(Refill::Random), at(Refill::Ranked));
                outcome(
                    (random as f64 - ranked as f64).abs() <= 0.2 * LONG as f64,
                    format!("collapsed at 10⁵ of {LONG}: random refill {random}, ranked {ranked}"),
                )
            },
        },
    ]
}
