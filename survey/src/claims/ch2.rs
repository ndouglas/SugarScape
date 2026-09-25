//! Chapter II: ii-1-instant, ii-2-unit, ii-5-wealth, ii-6-waves,
//! ii-7-seasons, ii-8-pollution. (Figure II-5's sweep is in ch6.)

use std::collections::{BTreeMap, BTreeSet};

use sugarscape_core::world::World;

use crate::claim::{greater, range, untestable, Claim, Source};
use crate::runner::{after, each_seed, preset, series, window_mean};
use crate::stats;

fn positions(w: &World) -> BTreeMap<u64, (u32, u32)> {
    w.agents().map(|a| (a.id, (a.pos.x, a.pos.y))).collect()
}

/// Share of agents alive at both times that did not move between them.
fn stationary_share(before: &BTreeMap<u64, (u32, u32)>, after: &BTreeMap<u64, (u32, u32)>) -> f64 {
    let both: Vec<_> = after.iter().filter(|(id, _)| before.contains_key(id)).collect();
    if both.is_empty() {
        return f64::NAN;
    }
    both.iter().filter(|(id, p)| before[id] == **p).count() as f64 / both.len() as f64
}

fn stationary_between(id: &str, from: u32, to: u32, seeds: &[u64]) -> Vec<f64> {
    each_seed(&preset(id), seeds, |mut w| {
        w.run(from);
        let before = positions(&w);
        w.run(to - from);
        stationary_share(&before, &positions(&w))
    })
}

/// (mean metabolism, mean vision) of agents dead by `ticks`, and of survivors.
fn dead_vs_alive(seeds: &[u64], ticks: u32) -> Vec<[f64; 4]> {
    each_seed(&preset("ii-1-instant"), seeds, |mut w| {
        let start: Vec<(u64, f64, f64)> = w
            .agents()
            .map(|a| (a.id, f64::from(a.metabolism[0]), f64::from(a.vision)))
            .collect();
        w.run(ticks);
        let alive: BTreeSet<u64> = w.agents().map(|a| a.id).collect();
        let pick = |dead: bool, f: fn(&(u64, f64, f64)) -> f64| {
            let v: Vec<f64> = start.iter().filter(|a| alive.contains(&a.0) != dead).map(f).collect();
            if v.is_empty() { f64::NAN } else { stats::mean(&v) }
        };
        [pick(true, |a| a.1), pick(false, |a| a.1), pick(true, |a| a.2), pick(false, |a| a.2)]
    })
}

fn col(rows: &[[f64; 4]], i: usize) -> Vec<f64> {
    rows.iter().map(|r| r[i]).collect()
}

/// Per seed: (migrator mean vision, hibernator mean vision, migrator mean
/// metabolism, hibernator mean metabolism) over agents alive for all of
/// t = 100..=300. Migrators change hemisphere (north is y < 25) at least
/// twice; hibernators never do.
fn seasonal_groups(seeds: &[u64]) -> Vec<[f64; 4]> {
    each_seed(&preset("ii-7-seasons"), seeds, |mut w| {
        w.run(100);
        let mut last: BTreeMap<u64, bool> = w.agents().map(|a| (a.id, a.pos.y < 25)).collect();
        let mut switches: BTreeMap<u64, u32> = last.keys().map(|id| (*id, 0)).collect();
        for _ in 0..200 {
            w.step();
            let now: BTreeMap<u64, bool> = w.agents().map(|a| (a.id, a.pos.y < 25)).collect();
            switches.retain(|id, _| now.contains_key(id));
            for (id, n) in switches.iter_mut() {
                if now[id] != last[id] {
                    *n += 1;
                }
            }
            last = now;
        }
        let traits: BTreeMap<u64, (f64, f64)> = w
            .agents()
            .map(|a| (a.id, (f64::from(a.vision), f64::from(a.metabolism[0]))))
            .collect();
        let group = |migrant: bool, f: fn(&(f64, f64)) -> f64| {
            let v: Vec<f64> = switches
                .iter()
                .filter(|(_, n)| if migrant { **n >= 2 } else { **n == 0 })
                .map(|(id, _)| f(&traits[id]))
                .collect();
            if v.is_empty() { f64::NAN } else { stats::mean(&v) }
        };
        [group(true, |t| t.0), group(false, |t| t.0), group(true, |t| t.1), group(false, |t| t.1)]
    })
}

fn wealths(w: &World) -> Vec<f64> {
    w.agents().map(|a| a.holdings[0]).collect()
}

/// Coefficient of variation of site pollution.
fn pollution_cv(w: &World) -> f64 {
    let p: Vec<f64> = w.sites.iter().map(|s| s.pollution[0]).collect();
    let m = stats::mean(&p);
    if m == 0.0 {
        return f64::NAN;
    }
    let sd = (p.iter().map(|x| (x - m).powi(2)).sum::<f64>() / p.len() as f64).sqrt();
    sd / m
}

pub fn claims() -> Vec<Claim> {
    vec![
        Claim {
            id: "ii-1.settle",
            item: "ii-1-instant",
            source: Source::App,
            citation: "presets.rs ii-1-instant description",
            text: "agents climb to the best ridge they can see and settle",
            // Metric: share of agents that do not move between t = 90 and t = 100.
            check: |s| range(&stationary_between("ii-1-instant", 90, 100, s), 0.9, 1.0, false),
        },
        Claim {
            id: "ii-1.starve-metabolism",
            item: "ii-1-instant",
            source: Source::App,
            citation: "presets.rs ii-1-instant description",
            text: "the poorly endowed starve (higher metabolism among the dead)",
            check: |s| {
                let r = dead_vs_alive(s, 100);
                greater(&col(&r, 0), &col(&r, 1), "dead metabolism", "survivor metabolism")
            },
        },
        Claim {
            id: "ii-1.starve-vision",
            item: "ii-1-instant",
            source: Source::App,
            citation: "presets.rs ii-1-instant description",
            text: "the poorly endowed starve (lower vision among the dead)",
            check: |s| {
                let r = dead_vs_alive(s, 100);
                greater(&col(&r, 3), &col(&r, 2), "survivor vision", "dead vision")
            },
        },
        Claim {
            id: "ii-1.static",
            item: "ii-1-instant",
            source: Source::Book,
            citation: "book, from memory: Animation II-1 reaches a static configuration",
            text: "once settled, nobody else dies (deaths over t = 101..=200 are 0)",
            check: |s| {
                let d = after(&preset("ii-1-instant"), s, 200, |w| series(w, "deaths")[101..=200].iter().sum::<f64>());
                range(&d, 0.0, 0.0, false)
            },
        },
        Claim {
            id: "ii-2.capacity",
            item: "ii-2-unit",
            source: Source::Book,
            citation: "tests/book.rs quoting Chapter II: \"a carrying capacity of approximately 224 is eventually reached\"; presets.rs: \"near 224\"",
            text: "population falls to a carrying capacity near 224 (mean over t = 400..=500)",
            check: |s| {
                let p = after(&preset("ii-2-unit"), s, 500, |w| window_mean(&series(w, "population"), 400, 500));
                range(&p, 224.0, 224.0, true)
            },
        },
        Claim {
            id: "ii-2.on-mountains",
            item: "ii-2-unit",
            source: Source::App,
            citation: "presets.rs ii-2-unit description",
            text: "hiving on the two sugar mountains (≥ 80% of agents on sites of capacity ≥ 2 at t = 500)",
            check: |s| {
                let f = after(&preset("ii-2-unit"), s, 500, |w| {
                    let on = w.agents().filter(|a| w.site(a.pos).capacity[0] >= 2.0).count();
                    on as f64 / w.population() as f64
                });
                range(&f, 0.8, 1.0, false)
            },
        },
        Claim {
            id: "ii-2.continuous",
            item: "ii-2-unit",
            source: Source::App,
            citation: "presets.rs ii-2-unit description",
            text: "continuous hiving (at most half the agents stay put over t = 490..500)",
            check: |s| range(&stationary_between("ii-2-unit", 490, 500, s), 0.0, 0.5, false),
        },
        Claim {
            id: "ii-5.gini",
            item: "ii-5-wealth",
            source: Source::Book,
            citation: "docs/superpowers/specs/2026-09-22-sugarscape-wasm-playground-design.md: \"R[60,100] Gini exceeds ~0.5\"",
            text: "the Gini coefficient exceeds about 0.5 (t = 500)",
            check: |s| {
                let g = after(&preset("ii-5-wealth"), s, 500, |w| w.stats.latest().unwrap().gini);
                range(&g, 0.5, 1.0, true)
            },
        },
        Claim {
            id: "ii-5.skewed",
            item: "ii-5-wealth",
            source: Source::App,
            citation: "presets.rs ii-5-wealth description",
            text: "a skewed wealth distribution emerges (sample skewness ≥ 1 at t = 500)",
            check: |s| {
                let k = after(&preset("ii-5-wealth"), s, 500, |w| stats::skewness(&wealths(w)));
                range(&k, 1.0, f64::INFINITY, false)
            },
        },
        Claim {
            id: "ii-5.emerges",
            item: "ii-5-wealth",
            source: Source::App,
            citation: "presets.rs ii-5-wealth description",
            text: "the skew emerges (Gini at t = 500 exceeds Gini at t = 0)",
            check: |s| {
                let g = after(&preset("ii-5-wealth"), s, 500, |w| {
                    let g = series(w, "gini");
                    (g[0], g[500])
                });
                let (start, end): (Vec<f64>, Vec<f64>) = g.into_iter().unzip();
                greater(&end, &start, "Gini t=500", "Gini t=0")
            },
        },
        Claim {
            id: "ii-6.propagates",
            item: "ii-6-waves",
            source: Source::App,
            citation: "presets.rs ii-6-waves description",
            text: "a block in the southwest propagates northeast (≥ 20% of agents in the NE quadrant x ≥ 25, y < 25 at t = 200)",
            check: |s| {
                let f = after(&preset("ii-6-waves"), s, 200, |w| {
                    let ne = w.agents().filter(|a| a.pos.x >= 25 && a.pos.y < 25).count();
                    ne as f64 / w.population() as f64
                });
                range(&f, 0.2, 1.0, false)
            },
        },
        Claim {
            id: "ii-6.waves",
            item: "ii-6-waves",
            source: Source::App,
            citation: "presets.rs ii-6-waves description",
            text: "collective waves no individual can move in",
            check: |_| untestable("a visual claim about wave fronts; no agreed metric distinguishes a wave from a drift, so only propagation (ii-6.propagates) is checked"),
        },
        Claim {
            id: "ii-7.migrants-see-further",
            item: "ii-7-seasons",
            source: Source::App,
            citation: "presets.rs ii-7-seasons description",
            text: "high-vision agents migrate (migrators have higher mean vision than hibernators)",
            check: |s| {
                let r = seasonal_groups(s);
                greater(&col(&r, 0), &col(&r, 1), "migrator vision", "hibernator vision")
            },
        },
        Claim {
            id: "ii-7.hibernators-burn-less",
            item: "ii-7-seasons",
            source: Source::App,
            citation: "presets.rs ii-7-seasons description",
            text: "low-metabolism agents hibernate (hibernators have lower mean metabolism than migrators)",
            check: |s| {
                let r = seasonal_groups(s);
                greater(&col(&r, 2), &col(&r, 3), "migrator metabolism", "hibernator metabolism")
            },
        },
        Claim {
            id: "ii-8.onset",
            item: "ii-8-pollution",
            source: Source::App,
            citation: "presets.rs ii-8-pollution description",
            text: "pollution starts at t = 50 (zero through t = 49, positive at t = 60)",
            check: |s| {
                let ok = after(&preset("ii-8-pollution"), s, 60, |w| {
                    let p = series(w, "mean_pollution_0");
                    f64::from(u8::from(p[..=49].iter().all(|x| *x == 0.0) && p[60] > 0.0))
                });
                range(&ok, 1.0, 1.0, false)
            },
        },
        Claim {
            id: "ii-8.diffusion",
            item: "ii-8-pollution",
            source: Source::App,
            citation: "presets.rs ii-8-pollution description",
            text: "diffusion spreads pollution from t = 100 (site pollution less uneven at t = 110 than t = 99)",
            check: |s| {
                let r = each_seed(&preset("ii-8-pollution"), s, |mut w| {
                    w.run(99);
                    let before = pollution_cv(&w);
                    w.run(11);
                    (before, pollution_cv(&w))
                });
                let (before, after): (Vec<f64>, Vec<f64>) = r.into_iter().unzip();
                greater(&before, &after, "CV t=99", "CV t=110")
            },
        },
        Claim {
            id: "ii-8.lowers-capacity",
            item: "ii-8-pollution",
            source: Source::Book,
            citation: "book, from memory: Animation II-8, pollution makes the landscape less habitable",
            text: "carrying capacity is lower with pollution than ii-2-unit (mean population t = 400..=500)",
            check: |s| {
                let pop = |id| after(&preset(id), s, 500, |w| window_mean(&series(w, "population"), 400, 500));
                greater(&pop("ii-2-unit"), &pop("ii-8-pollution"), "ii-2-unit", "ii-8-pollution")
            },
        },
    ]
}

