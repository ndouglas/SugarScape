//! Chapter IV: iv-1-spice, iv-3-trade, iv-15-trade-sex, iv-3-pollution,
//! iv-18-foresight, iv-5-credit. (The fig-iv-6 and fig-iv-10-11 sweeps are
//! in ch6.)

use std::collections::{BTreeMap, BTreeSet};

use sugarscape_core::world::World;

use crate::claim::{equivalent, greater, range, untestable, Claim, Source};
use crate::runner::{after, each_seed, preset, series, window_mean};
use crate::stats;

/// Mean of series `name` over the ticks in `from..=to` that had at least one
/// trade (price series are 0 on ticks without trades); NaN if none did.
fn trading_mean(w: &World, name: &str, from: usize, to: usize) -> f64 {
    let s = series(w, name);
    let v = series(w, "trade_volume");
    let x: Vec<f64> = (from..=to).filter(|&t| v[t] > 0.0).map(|t| s[t]).collect();
    if x.is_empty() {
        f64::NAN
    } else {
        stats::mean(&x)
    }
}

/// Population standard deviation of `v`.
fn sd(v: &[f64]) -> f64 {
    let m = stats::mean(v);
    (v.iter().map(|x| (x - m).powi(2)).sum::<f64>() / v.len() as f64).sqrt()
}

/// Per seed, `(a, b)` from `f` split into two vectors.
fn pairs(r: Vec<(f64, f64)>) -> (Vec<f64>, Vec<f64>) {
    r.into_iter().unzip()
}

/// 1 when `ok`, else 0 (for all-or-nothing per-seed checks judged by range).
fn flag(ok: bool) -> f64 {
    f64::from(u8::from(ok))
}

/// Which good dominates an agent's site: Some(false) sugar, Some(true) spice,
/// None a tie.
fn side(w: &World, a: &sugarscape_core::agent::Agent) -> Option<bool> {
    let c = w.site(a.pos).capacity;
    if c[0] > c[1] {
        Some(false)
    } else if c[1] > c[0] {
        Some(true)
    } else {
        None
    }
}

/// Share of agents alive over all of t = 100..=200 that switch between a
/// sugar-dominated and a spice-dominated site at least twice (sites where the
/// capacities tie leave the agent's side unchanged).
fn shuttle_share(seeds: &[u64]) -> Vec<f64> {
    each_seed(&preset("iv-1-spice"), seeds, |mut w| {
        w.run(100);
        let mut last: BTreeMap<u64, Option<bool>> = w.agents().map(|a| (a.id, side(&w, a))).collect();
        let mut switches: BTreeMap<u64, u32> = last.keys().map(|id| (*id, 0)).collect();
        for _ in 0..100 {
            w.step();
            let now: BTreeMap<u64, Option<bool>> = w.agents().map(|a| (a.id, side(&w, a))).collect();
            switches.retain(|id, _| now.contains_key(id));
            for (id, n) in switches.iter_mut() {
                if let Some(s) = now[id] {
                    if last[id].is_some_and(|l| l != s) {
                        *n += 1;
                    }
                    last.insert(*id, Some(s));
                }
            }
        }
        if switches.is_empty() {
            return f64::NAN;
        }
        switches.values().filter(|n| **n >= 2).count() as f64 / switches.len() as f64
    })
}

/// Coefficient of variation of pollutant 0 over sites.
fn pollution_cv(w: &World) -> f64 {
    let p: Vec<f64> = w.sites.iter().map(|s| s.pollution[0]).collect();
    let m = stats::mean(&p);
    if m == 0.0 {
        return f64::NAN;
    }
    sd(&p) / m
}

/// iv-5-credit at t = 500, over outstanding loans between living agents:
/// [mean lender age, mean borrower age, share of loans whose lender is past
/// fertility, 1 if some agent both lends and borrows else 0].
fn credit_snapshot(seeds: &[u64]) -> Vec<[f64; 4]> {
    after(&preset("iv-5-credit"), seeds, 500, |w| {
        let live: Vec<_> = w
            .loans()
            .filter_map(|l| Some((w.agent(l.lender)?, w.agent(l.borrower)?)))
            .collect();
        if live.is_empty() {
            return [f64::NAN, f64::NAN, f64::NAN, 0.0];
        }
        let n = live.len() as f64;
        let lender_age = live.iter().map(|(l, _)| f64::from(l.age)).sum::<f64>() / n;
        let borrower_age = live.iter().map(|(_, b)| f64::from(b.age)).sum::<f64>() / n;
        let old = live.iter().filter(|(l, _)| l.age > l.fertility_end).count() as f64 / n;
        let lenders: BTreeSet<u64> = live.iter().map(|(l, _)| l.id).collect();
        let both = live.iter().any(|(_, b)| lenders.contains(&b.id));
        [lender_age, borrower_age, old, flag(both)]
    })
}

fn col(rows: &[[f64; 4]], i: usize) -> Vec<f64> {
    rows.iter().map(|r| r[i]).collect()
}

pub fn claims() -> Vec<Claim> {
    vec![
        // ---- iv-1-spice ----
        Claim {
            id: "iv-1.opposite-mountains",
            item: "iv-1-spice",
            source: Source::App,
            citation: "presets.rs iv-1-spice description",
            text: "two goods on opposite mountains (Pearson correlation of sugar and spice capacity over sites < 0)",
            check: |s| {
                let r = after(&preset("iv-1-spice"), s, 0, |w| {
                    let a: Vec<f64> = w.sites.iter().map(|x| x.capacity[0]).collect();
                    let b: Vec<f64> = w.sites.iter().map(|x| x.capacity[1]).collect();
                    let (ma, mb) = (stats::mean(&a), stats::mean(&b));
                    let cov = a.iter().zip(&b).map(|(x, y)| (x - ma) * (y - mb)).sum::<f64>() / a.len() as f64;
                    cov / (sd(&a) * sd(&b))
                });
                range(&r, -1.0, -f64::MIN_POSITIVE, false)
            },
        },
        Claim {
            id: "iv-1.spice-nw-se",
            item: "iv-1-spice",
            source: Source::Book,
            citation: "docs/superpowers/specs/2026-09-22-chapter-iv-sugar-and-spice-design.md: \"spice mountains in the northwest and southeast, as in the book's Figure IV-1\"",
            text: "spice mountains lie in the northwest and southeast (more than half of total spice capacity in the NW and SE quadrants, row 0 = north)",
            check: |s| {
                let r = after(&preset("iv-1-spice"), s, 0, |w| {
                    let width = w.config.width as usize;
                    let half = (width / 2, w.config.height as usize / 2);
                    let (mut diag, mut total) = (0.0, 0.0);
                    for (i, site) in w.sites.iter().enumerate() {
                        let (x, y) = (i % width, i / width);
                        let west = x < half.0;
                        let north = y < half.1;
                        total += site.capacity[1];
                        if west == north {
                            diag += site.capacity[1];
                        }
                    }
                    diag / total
                });
                range(&r, 0.5 + f64::EPSILON, 1.0, false)
            },
        },
        Claim {
            id: "iv-1.shuttle",
            item: "iv-1-spice",
            source: Source::App,
            citation: "presets.rs iv-1-spice description",
            text: "agents shuttle between sugar and spice (a majority, ≥ 50%, of agents alive over t = 100..=200 switch between sugar-dominated and spice-dominated sites at least twice)",
            check: |s| range(&shuttle_share(s), 0.5, 1.0, false),
        },
        Claim {
            id: "iv-1.stay-alive",
            item: "iv-1-spice",
            source: Source::App,
            citation: "presets.rs iv-1-spice description",
            text: "agents stay alive (population at t = 500 is positive)",
            check: |s| {
                let p = after(&preset("iv-1-spice"), s, 500, |w| w.population() as f64);
                range(&p, 1.0, f64::INFINITY, false)
            },
        },
        // ---- iv-3-trade ----
        Claim {
            id: "iv-3.prices-near-one",
            item: "iv-3-trade",
            source: Source::Book,
            citation: "docs/superpowers/specs/2026-09-22-chapter-iv-sugar-and-spice-design.md: \"trade price clusters near 1 (mean mean_log_price over ticks 500–1000 of iv-3-trade … within ±0.25)\"; tests/book.rs trade_prices_cluster_near_one (Figure IV-3)",
            text: "prices cluster near 1: mean ln price over trading ticks t = 500..=1000 within ±0.25",
            check: |s| {
                let m = after(&preset("iv-3-trade"), s, 1000, |w| trading_mean(w, "mean_log_price", 500, 1000));
                range(&m, -0.25, 0.25, false)
            },
        },
        Claim {
            id: "iv-3.converge",
            item: "iv-3-trade",
            source: Source::App,
            citation: "presets.rs iv-3-trade description",
            text: "prices converge (mean sd of ln price over trading ticks is lower at t = 950..=1000 than at t = 1..=50)",
            check: |s| {
                let (early, late) = pairs(after(&preset("iv-3-trade"), s, 1000, |w| {
                    (trading_mean(w, "sd_log_price", 1, 50), trading_mean(w, "sd_log_price", 950, 1000))
                }));
                greater(&early, &late, "sd t=1..50", "sd t=950..1000")
            },
        },
        Claim {
            id: "iv-3.market-clearing",
            item: "iv-3-trade",
            source: Source::App,
            citation: "presets.rs iv-3-trade description",
            text: "the market-clearing level is 1 (supply-and-demand equilibrium price at t = 1000 within e^±0.25, the spec's ±0.25 band on ln price)",
            check: |s| {
                let p = after(&preset("iv-3-trade"), s, 1000, |w| sugarscape_core::stats::supply_demand(w).equilibrium_price);
                range(&p, (-0.25f64).exp(), 0.25f64.exp(), false)
            },
        },
        Claim {
            id: "iv-3.measured-ln-price",
            item: "iv-3-trade",
            source: Source::Comment,
            citation: "tests/book.rs trade_prices_cluster_near_one: \"Observed (mean ln price over t=500..1000): seed 1 = 0.008676, seed 2 = 0.007983, seed 3 = 0.009746\"",
            text: "mean of mean_log_price over t = 500..=1000 (all ticks, as the test measures it) is about 0.0080–0.0097",
            check: |s| {
                let m = after(&preset("iv-3-trade"), s, 1000, |w| window_mean(&series(w, "mean_log_price"), 500, 1000));
                range(&m, 0.007983, 0.009746, true)
            },
        },
        // ---- iv-15-trade-sex ----
        Claim {
            id: "iv-15.prices-unsettled",
            item: "iv-15-trade-sex",
            source: Source::App,
            citation: "presets.rs iv-15-trade-sex description",
            text: "finite lives and evolving preferences keep prices from settling (mean sd of ln price over trading ticks t = 950..=1000 is higher than in iv-3-trade)",
            check: |s| {
                let m = |id| after(&preset(id), s, 1000, |w| trading_mean(w, "sd_log_price", 950, 1000));
                greater(&m("iv-15-trade-sex"), &m("iv-3-trade"), "iv-15-trade-sex", "iv-3-trade")
            },
        },
        Claim {
            id: "iv-15.mean-price-wanders",
            item: "iv-15-trade-sex",
            source: Source::App,
            citation: "presets.rs iv-15-trade-sex description",
            text: "prices do not settle (the tick-to-tick sd of mean ln price over trading ticks t = 500..=1000 is higher than in iv-3-trade)",
            check: |s| {
                let m = |id| {
                    after(&preset(id), s, 1000, |w| {
                        let p = series(w, "mean_log_price");
                        let v = series(w, "trade_volume");
                        let x: Vec<f64> = (500..=1000).filter(|&t| v[t] > 0.0).map(|t| p[t]).collect();
                        if x.len() < 2 { f64::NAN } else { sd(&x) }
                    })
                };
                greater(&m("iv-15-trade-sex"), &m("iv-3-trade"), "iv-15-trade-sex", "iv-3-trade")
            },
        },
        Claim {
            id: "iv-15.preferences-evolve",
            item: "iv-15-trade-sex",
            source: Source::App,
            citation: "presets.rs iv-15-trade-sex description",
            text: "preferences evolve (metabolisms, which set the welfare weights, are selected: mean sugar + spice metabolism is lower at t = 1000 than at t = 0)",
            check: |s| {
                let (start, end) = pairs(after(&preset("iv-15-trade-sex"), s, 1000, |w| {
                    let m = |t: usize| series(w, "mean_metabolism")[t] + series(w, "mean_spice_metabolism")[t];
                    (m(0), if w.population() == 0 { f64::NAN } else { m(1000) })
                }));
                greater(&start, &end, "t=0", "t=1000")
            },
        },
        Claim {
            id: "iv-15.dispersion-level",
            item: "iv-15-trade-sex",
            source: Source::Book,
            citation: "book, from memory: Figure IV-15, the sd of ln price under ({G₁}, {M, S, T}) fluctuates without declining",
            text: "price dispersion does not decline (mean sd of ln price over trading ticks is equivalent at t = 100..=200 and t = 900..=1000, margin 10% of the pooled mean)",
            check: |s| {
                let (mid, late) = pairs(after(&preset("iv-15-trade-sex"), s, 1000, |w| {
                    (trading_mean(w, "sd_log_price", 100, 200), trading_mean(w, "sd_log_price", 900, 1000))
                }));
                equivalent(&mid, &late, None, "sd t=100..200", "sd t=900..1000")
            },
        },
        // ---- iv-3-pollution ----
        Claim {
            id: "iv-3-pollution.onset",
            item: "iv-3-pollution",
            source: Source::App,
            citation: "presets.rs iv-3-pollution description",
            text: "sugar becomes a dirty good at t = 100 (mean pollution zero through t = 99, positive at t = 110)",
            check: |s| {
                let ok = after(&preset("iv-3-pollution"), s, 110, |w| {
                    let p = series(w, "mean_pollution_0");
                    flag(p[..=99].iter().all(|x| *x == 0.0) && p[110] > 0.0)
                });
                range(&ok, 1.0, 1.0, false)
            },
        },
        Claim {
            id: "iv-3-pollution.price-up",
            item: "iv-3-pollution",
            source: Source::App,
            citation: "presets.rs iv-3-pollution description",
            text: "pollution drives sugar's price up (mean ln price over trading ticks t = 101..=150 is higher than in iv-3-trade, which is identical until t = 100)",
            check: |s| {
                let m = |id| after(&preset(id), s, 150, |w| trading_mean(w, "mean_log_price", 101, 150));
                greater(&m("iv-3-pollution"), &m("iv-3-trade"), "iv-3-pollution", "iv-3-trade")
            },
        },
        Claim {
            id: "iv-3-pollution.stops",
            item: "iv-3-pollution",
            source: Source::App,
            citation: "presets.rs iv-3-pollution description",
            text: "at t = 150 agents stop polluting (mean pollution never rises between consecutive ticks over series index 151..=300, the first tick run under the t = 150 change onward; tolerance 1e-9 relative)",
            // Changed after first run: the check compared indices 150..=300,
            // giving Fails, 0/20. A change scheduled for tick 150 fires at
            // the start of the step from completed tick 150 to 151
            // (`World::apply_schedule` matches `self.tick`), so index 150 is
            // still a tick with production on; the window now starts at 151.
            check: |s| {
                let ok = after(&preset("iv-3-pollution"), s, 300, |w| {
                    let p = series(w, "mean_pollution_0");
                    flag((151..=300).all(|t| p[t] <= p[t - 1] * (1.0 + 1e-9) + 1e-12))
                });
                range(&ok, 1.0, 1.0, false)
            },
        },
        Claim {
            id: "iv-3-pollution.diffuses",
            item: "iv-3-pollution",
            source: Source::App,
            citation: "presets.rs iv-3-pollution description",
            text: "the pollution already made diffuses away (site pollution is less uneven, by coefficient of variation, at t = 200 than at t = 149)",
            check: |s| {
                let (before, after_) = pairs(each_seed(&preset("iv-3-pollution"), s, |mut w| {
                    w.run(149);
                    let before = pollution_cv(&w);
                    w.run(51);
                    (before, pollution_cv(&w))
                }));
                greater(&before, &after_, "CV t=149", "CV t=200")
            },
        },
        // ---- iv-18-foresight ----
        Claim {
            id: "iv-18.nonzero",
            item: "iv-18-foresight",
            source: Source::Book,
            citation: "docs/superpowers/specs/2026-09-22-chapter-iv-sugar-and-spice-design.md: \"mean foresight in iv-18-foresight stays above 0\"; tests/book.rs foresight_evolves_to_a_modest_nonzero_level (end > 0.1)",
            text: "mean foresight stays above 0 (> 0.1, the book test's bound, at t = 1000)",
            check: |s| {
                let f = after(&preset("iv-18-foresight"), s, 1000, |w| {
                    if w.population() == 0 { f64::NAN } else { series(w, "mean_foresight")[1000] }
                });
                range(&f, 0.1, f64::INFINITY, false)
            },
        },
        Claim {
            id: "iv-18.falls",
            item: "iv-18-foresight",
            source: Source::Book,
            citation: "docs/superpowers/specs/2026-09-22-chapter-iv-sugar-and-spice-design.md: \"falls below its initial mean\"; tests/book.rs (Figure IV-18: \"large foresight is not [fit]\")",
            text: "mean foresight falls below its initial mean (t = 1000 lower than t = 0)",
            check: |s| {
                let (start, end) = pairs(after(&preset("iv-18-foresight"), s, 1000, |w| {
                    let f = series(w, "mean_foresight");
                    (f[0], if w.population() == 0 { f64::NAN } else { f[1000] })
                }));
                greater(&start, &end, "t=0", "t=1000")
            },
        },
        Claim {
            id: "iv-18.modest-kept",
            item: "iv-18-foresight",
            source: Source::App,
            citation: "presets.rs iv-18-foresight description",
            text: "evolution keeps a modest, non-zero foresight (mean foresight at t = 2000 in [0.1, 5]: above the book test's 0.1, at most the initial 0–10 draw's mean of 5)",
            check: |s| {
                let f = after(&preset("iv-18-foresight"), s, 2000, |w| {
                    if w.population() == 0 { f64::NAN } else { series(w, "mean_foresight")[2000] }
                });
                range(&f, 0.1, 5.0, false)
            },
        },
        Claim {
            id: "iv-18.plan-ahead",
            item: "iv-18-foresight",
            source: Source::App,
            citation: "presets.rs iv-18-foresight description",
            text: "agents plan φ periods ahead",
            check: |_| untestable("a statement of the movement rule itself; the survey does not audit rule implementations line by line (spec, Out of scope)"),
        },
        Claim {
            id: "iv-18.measured-population",
            item: "iv-18-foresight",
            source: Source::Comment,
            citation: "presets.rs iv-18-foresight comment: \"population grows from 400 to ~600-700 by t=1000\"",
            text: "population at t = 1000 is about 600–700",
            check: |s| {
                let p = after(&preset("iv-18-foresight"), s, 1000, |w| w.population() as f64);
                range(&p, 600.0, 700.0, true)
            },
        },
        // ---- iv-5-credit ----
        Claim {
            id: "iv-5.lenders-older",
            item: "iv-5-credit",
            source: Source::App,
            citation: "presets.rs iv-5-credit description",
            text: "old agents lend to young ones (over outstanding loans at t = 500, mean lender age exceeds mean borrower age)",
            check: |s| {
                let r = credit_snapshot(s);
                greater(&col(&r, 0), &col(&r, 1), "lender age", "borrower age")
            },
        },
        Claim {
            id: "iv-5.lenders-old",
            item: "iv-5-credit",
            source: Source::App,
            citation: "presets.rs iv-5-credit description",
            text: "old agents lend (a majority, ≥ 50%, of outstanding loans at t = 500 have a lender past its fertility end)",
            check: |s| range(&col(&credit_snapshot(s), 2), 0.5, 1.0, false),
        },
        Claim {
            id: "iv-5.for-childbearing",
            item: "iv-5-credit",
            source: Source::App,
            citation: "presets.rs iv-5-credit description",
            text: "loans are for childbearing (≥ 80% of loans originated over t = 401..=500, rollovers included, go to borrowers of childbearing age: onset ≤ age ≤ end + 1, the +1 for ageing after credit within a tick)",
            check: |s| {
                let r = each_seed(&preset("iv-5-credit"), s, |mut w| {
                    w.run(400);
                    let mut seen: BTreeSet<u64> = w.loans().map(|l| l.id).collect();
                    let (mut fertile, mut total) = (0usize, 0usize);
                    for _ in 0..100 {
                        w.step();
                        for l in w.loans() {
                            if seen.insert(l.id) {
                                if let Some(b) = w.agent(l.borrower) {
                                    total += 1;
                                    if (b.fertility_onset..=b.fertility_end + 1).contains(&b.age) {
                                        fertile += 1;
                                    }
                                }
                            }
                        }
                    }
                    if total == 0 { f64::NAN } else { fertile as f64 / total as f64 }
                });
                range(&r, 0.8, 1.0, false)
            },
        },
        Claim {
            id: "iv-5.hierarchy",
            item: "iv-5-credit",
            source: Source::App,
            citation: "presets.rs iv-5-credit description; book, from memory: Animation IV-5 shows agents who are both lenders and borrowers",
            text: "lender–borrower hierarchies emerge (at t = 500 some agent is both a lender and a borrower, i.e. a chain of at least two loans)",
            check: |s| range(&col(&credit_snapshot(s), 3), 1.0, 1.0, false),
        },
    ]
}
