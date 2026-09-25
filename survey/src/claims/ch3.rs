//! Chapter III: iii-2-sex, iii-4-inheritance, iii-6-culture,
//! iii-6-three-tribes, iii-9-combat, iii-11-combat-fixed, iii-12-collision,
//! iii-14-combat-culture.

use std::collections::{BTreeMap, BTreeSet};

use sugarscape_core::agent::Tribe;
use sugarscape_core::config::Config;
use sugarscape_core::world::{DeathCause, World};

use crate::claim::{greater, range, untestable, Claim, Source};
use crate::runner::{after, each_seed, preset, series, window_mean};
use crate::stats;

/// Largest single-tick sugar gain possible without loot or inheritance: the
/// default map's highest site capacity.
const MAX_SITE: f64 = 4.0;

fn flag(b: bool) -> f64 {
    f64::from(u8::from(b))
}

/// Fraction of von Neumann-adjacent agent pairs (each pair counted once) that
/// share a tribe (as `local_homogeneity` in tests/book.rs). NaN with no pairs.
fn local_homogeneity(w: &World) -> f64 {
    let (mut same, mut pairs) = (0u32, 0u32);
    for a in w.agents() {
        for (dx, dy) in [(1, 0), (0, 1)] {
            if let Some(b) = w.agent_at(w.torus.offset(a.pos, dx, dy)) {
                pairs += 1;
                same += u32::from(a.tribe() == b.tribe());
            }
        }
    }
    if pairs == 0 {
        return f64::NAN;
    }
    f64::from(same) / f64::from(pairs)
}

/// Share of the population in the larger tribe.
fn majority_share(w: &World) -> f64 {
    let blue = w.agents().filter(|a| a.tribe() == Tribe::Blue).count() as f64;
    let n = w.population() as f64;
    if n == 0.0 {
        return f64::NAN;
    }
    (blue / n).max(1.0 - blue / n)
}

/// Mean torus distance of agents from the grid's center.
fn mean_distance_to_center(w: &World) -> f64 {
    let (cw, ch) = (w.config.width, w.config.height);
    let d = |v: u32, c: u32, size: u32| {
        let a = v.abs_diff(c);
        f64::from(a.min(size - a))
    };
    let v: Vec<f64> = w
        .agents()
        .map(|a| d(a.pos.x, cw / 2, cw).hypot(d(a.pos.y, ch / 2, ch)))
        .collect();
    if v.is_empty() { f64::NAN } else { stats::mean(&v) }
}

fn combat_deaths(w: &World) -> usize {
    w.events()
        .deaths
        .iter()
        .filter(|d| d.cause == DeathCause::Combat)
        .count()
}

/// Per agent before a step: (sugar, metabolism, tribe, children).
type Snapshot = BTreeMap<u64, (f64, f64, Tribe, Vec<u64>)>;

fn snapshot(w: &World) -> Snapshot {
    w.agents()
        .map(|a| {
            (
                a.id,
                (a.holdings[0], f64::from(a.metabolism[0]), a.tribe(), a.children.clone()),
            )
        })
        .collect()
}

/// Sugar gained over the last step by each agent alive before and after it.
fn gains(w: &World, pre: &Snapshot) -> BTreeMap<u64, f64> {
    w.agents()
        .filter_map(|a| pre.get(&a.id).map(|p| (a.id, a.holdings[0] - p.0)))
        .collect()
}

/// iii-9's "whole wealth" check. Over t = 1..=500, for each combat death of a
/// victim holding ≥ 10 sugar before the tick, look for a distinct survivor of
/// the other tribe whose one-tick gain is at least the victim's pre-tick sugar
/// minus both agents' metabolisms (the victim may have eaten and burned once
/// before it was killed; the winner burns once after). Victims are matched
/// richest first, each to the smallest sufficient gain. Returns the share of
/// such victims matched; NaN when there are none. A capped reward could not be
/// matched: nobody else gains more than a site's 4 sugar in a tick.
fn whole_wealth_matched(seeds: &[u64]) -> Vec<f64> {
    each_seed(&preset("iii-9-combat"), seeds, |mut w| {
        let (mut eligible, mut matched) = (0u32, 0u32);
        for _ in 0..500 {
            let pre = snapshot(&w);
            w.step();
            let gain = gains(&w, &pre);
            let mut victims: Vec<(f64, f64, Tribe)> = w
                .events()
                .deaths
                .iter()
                .filter(|d| d.cause == DeathCause::Combat)
                .filter_map(|d| pre.get(&d.id).map(|p| (p.0, p.1, d.tribe)))
                .filter(|v| v.0 >= 10.0)
                .collect();
            victims.sort_by(|a, b| b.0.total_cmp(&a.0));
            let mut used = BTreeSet::new();
            for (sugar, metab, tribe) in victims {
                eligible += 1;
                let best = gain
                    .iter()
                    .filter(|(id, _)| !used.contains(*id) && pre[*id].2 != tribe)
                    .filter(|(id, g)| **g >= sugar - metab - pre[*id].1 - 1e-9)
                    .min_by(|a, b| a.1.total_cmp(b.1));
                if let Some((id, _)) = best {
                    used.insert(*id);
                    matched += 1;
                }
            }
        }
        if eligible == 0 {
            f64::NAN
        } else {
            f64::from(matched) / f64::from(eligible)
        }
    })
}

/// iii-4: per seed, the mean one-tick sugar gain of a living child in the tick
/// one of its parents dies, over t = 1..=300. NaN when no parent died.
fn child_gain_at_parent_death(config: &Config, seeds: &[u64]) -> Vec<f64> {
    each_seed(config, seeds, |mut w| {
        let mut v = Vec::new();
        for _ in 0..300 {
            let pre = snapshot(&w);
            w.step();
            let gain = gains(&w, &pre);
            for d in &w.events().deaths {
                for child in pre.get(&d.id).map(|p| p.3.clone()).unwrap_or_default() {
                    if let Some(g) = gain.get(&child) {
                        v.push(*g);
                    }
                }
            }
        }
        if v.is_empty() { f64::NAN } else { stats::mean(&v) }
    })
}

/// iii-2: distinct generations among agents alive at t = 600 (founders are
/// generation 0; a child is one more than its older-generation parent).
fn generations_alive(seeds: &[u64]) -> Vec<f64> {
    each_seed(&preset("iii-2-sex"), seeds, |mut w| {
        let mut gen: BTreeMap<u64, u32> = BTreeMap::new();
        let mut record = |w: &World| {
            for a in w.agents() {
                if gen.contains_key(&a.id) {
                    continue;
                }
                let g = match a.parents {
                    None => 0,
                    Some([p, q]) => 1 + gen.get(&p).copied().unwrap_or(0).max(gen.get(&q).copied().unwrap_or(0)),
                };
                gen.insert(a.id, g);
            }
        };
        record(&w);
        for _ in 0..600 {
            w.step();
            record(&w);
        }
        let alive: BTreeSet<u32> = w.agents().map(|a| gen[&a.id]).collect();
        alive.len() as f64
    })
}

/// iii-14: per seed, (combat deaths, agents whose tribe ever changed) over
/// t = 1..=500.
fn conquest_and_conversion(seeds: &[u64]) -> Vec<[f64; 2]> {
    each_seed(&preset("iii-14-combat-culture"), seeds, |mut w| {
        let mut first: BTreeMap<u64, Tribe> = w.agents().map(|a| (a.id, a.tribe())).collect();
        let mut converted = BTreeSet::new();
        let mut kills = 0usize;
        for _ in 0..500 {
            w.step();
            kills += combat_deaths(&w);
            for a in w.agents() {
                let t = *first.entry(a.id).or_insert(a.tribe());
                if t != a.tribe() {
                    converted.insert(a.id);
                }
            }
        }
        [kills as f64, converted.len() as f64]
    })
}

pub fn claims() -> Vec<Claim> {
    vec![
        // ---- iii-2-sex ----
        Claim {
            id: "iii-2.stable",
            item: "iii-2-sex",
            source: Source::App,
            citation: "presets.rs iii-2-sex description; bound from tests/book.rs sexual_reproduction_sustains_a_population",
            text: "a roughly stable population (max/min population over t = 300..=600 at most 1.6)",
            // 1.6 is the bound the existing book test uses for the same words.
            check: |s| {
                let r = after(&preset("iii-2-sex"), s, 600, |w| {
                    let p = &series(w, "population")[300..=600];
                    let (lo, hi) = p.iter().fold((f64::MAX, 0.0f64), |(l, h), &x| (l.min(x), h.max(x)));
                    if lo <= 0.0 { f64::INFINITY } else { hi / lo }
                });
                range(&r, 1.0, 1.6, false)
            },
        },
        Claim {
            id: "iii-2.persists",
            item: "iii-2-sex",
            source: Source::App,
            citation: "presets.rs iii-2-sex description (\"a roughly stable population\"); bound from tests/book.rs sexual_reproduction_sustains_a_population",
            text: "the population persists (minimum population over t = 300..=600 above 100)",
            check: |s| {
                let m = after(&preset("iii-2-sex"), s, 600, |w| {
                    series(w, "population")[300..=600].iter().copied().fold(f64::MAX, f64::min)
                });
                range(&m, 100.0 + 1e-9, f64::INFINITY, false)
            },
        },
        Claim {
            id: "iii-2.generations",
            item: "iii-2-sex",
            source: Source::App,
            citation: "presets.rs iii-2-sex description",
            text: "made of many generations (at least 3 distinct generations alive at t = 600; founders are generation 0)",
            check: |s| range(&generations_alive(s), 3.0, f64::INFINITY, false),
        },
        Claim {
            id: "iii-2.vision-rises",
            item: "iii-2-sex",
            source: Source::Book,
            citation: "book, from memory: Chapter III, sexual reproduction; selection raises mean vision",
            text: "mean vision at t = 500 exceeds mean vision at t = 0",
            check: |s| {
                let v = after(&preset("iii-2-sex"), s, 500, |w| {
                    let m = series(w, "mean_vision");
                    (m[0], m[500])
                });
                let (start, end): (Vec<f64>, Vec<f64>) = v.into_iter().unzip();
                greater(&end, &start, "vision t=500", "vision t=0")
            },
        },
        Claim {
            id: "iii-2.metabolism-falls",
            item: "iii-2-sex",
            source: Source::Book,
            citation: "book, from memory: Chapter III, sexual reproduction; selection lowers mean metabolism",
            text: "mean metabolism at t = 500 is below mean metabolism at t = 0",
            check: |s| {
                let v = after(&preset("iii-2-sex"), s, 500, |w| {
                    let m = series(w, "mean_metabolism");
                    (m[0], m[500])
                });
                let (start, end): (Vec<f64>, Vec<f64>) = v.into_iter().unzip();
                greater(&start, &end, "metabolism t=0", "metabolism t=500")
            },
        },
        // ---- iii-4-inheritance ----
        Claim {
            id: "iii-4.passes-wealth",
            item: "iii-4-inheritance",
            source: Source::App,
            citation: "presets.rs iii-4-inheritance description",
            text: "inheritance passes wealth to children (a child's one-tick sugar gain in the tick a parent dies, mean over t = 1..=300, is higher in iii-4-inheritance than in iii-2-sex)",
            check: |s| {
                let with = child_gain_at_parent_death(&preset("iii-4-inheritance"), s);
                let without = child_gain_at_parent_death(&preset("iii-2-sex"), s);
                greater(&with, &without, "iii-4-inheritance", "iii-2-sex")
            },
        },
        Claim {
            id: "iii-4.raises-gini",
            item: "iii-4-inheritance",
            source: Source::Book,
            citation: "book, from memory: Animation III-4, inheritance increases inequality; presets.rs: \"compare the Gini coefficient with and without it\"",
            text: "the Gini coefficient is higher with inheritance than without (mean Gini over t = 400..=500, iii-4-inheritance vs iii-2-sex)",
            check: |s| {
                let g = |id| after(&preset(id), s, 500, |w| window_mean(&series(w, "gini"), 400, 500));
                greater(&g("iii-4-inheritance"), &g("iii-2-sex"), "iii-4-inheritance", "iii-2-sex")
            },
        },
        // ---- iii-6-culture ----
        Claim {
            id: "iii-6.toward-uniform",
            item: "iii-6-culture",
            source: Source::App,
            citation: "presets.rs iii-6-culture description",
            text: "tag-flipping converts groups toward uniform tribes (share of adjacent agent pairs in the same tribe is higher at t = 3000 than at t = 0)",
            check: |s| {
                let r = each_seed(&preset("iii-6-culture"), s, |mut w| {
                    let start = local_homogeneity(&w);
                    w.run(3000);
                    (start, local_homogeneity(&w))
                });
                let (start, end): (Vec<f64>, Vec<f64>) = r.into_iter().unzip();
                greater(&end, &start, "homogeneity t=3000", "homogeneity t=0")
            },
        },
        Claim {
            id: "iii-6.one-tribe",
            item: "iii-6-culture",
            source: Source::Book,
            citation: "docs/superpowers/specs/2026-09-22-sugarscape-wasm-playground-design.md: \"K converges toward single-tribe dominance\"; tests/book.rs: \"the book: after ~2700 ticks\"",
            text: "single-tribe dominance (the larger tribe holds ≥ 90% of agents at t = 3000)",
            check: |s| {
                let m = after(&preset("iii-6-culture"), s, 3000, majority_share);
                range(&m, 0.9, 1.0, false)
            },
        },
        Claim {
            id: "iii-6.homogeneity-measured",
            item: "iii-6-culture",
            source: Source::Comment,
            citation: "tests/book.rs culture_drives_neighbors_toward_one_tribe: observed means 0.465 -> 0.973 (seeds 1-5), asserts end > 0.9",
            text: "local homogeneity at t = 3000 exceeds 0.9",
            check: |s| {
                let h = after(&preset("iii-6-culture"), s, 3000, local_homogeneity);
                range(&h, 0.9, 1.0, false)
            },
        },
        // ---- iii-6-three-tribes ----
        Claim {
            id: "iii-6-three.scheme",
            item: "iii-6-three-tribes",
            source: Source::App,
            citation: "presets.rs iii-6-three-tribes description",
            text: "groups are Blue 0–3 zeros, Green 4–7, Red 8–11",
            check: |s| {
                let ok = after(&preset("iii-6-three-tribes"), s, 0, |w| {
                    let g: Vec<(&str, u32, u32)> = w
                        .config
                        .culture
                        .groups
                        .iter()
                        .map(|g| (g.name.as_str(), g.zeros.min, g.zeros.max))
                        .collect();
                    flag(g == [("Blue", 0, 3), ("Green", 4, 7), ("Red", 8, 11)] && w.config.culture.enabled)
                });
                range(&ok, 1.0, 1.0, false)
            },
        },
        Claim {
            id: "iii-6-three.shares-move",
            item: "iii-6-three-tribes",
            source: Source::App,
            citation: "presets.rs iii-6-three-tribes description (\"Watch the Group shares chart\")",
            text: "the group shares change visibly (some group's share moves by ≥ 0.1 between t = 0 and t = 3000)",
            check: |s| {
                let d = after(&preset("iii-6-three-tribes"), s, 3000, |w| {
                    (0..3)
                        .map(|k| {
                            let g = series(w, &format!("group_share_{k}"));
                            (g[3000] - g[0]).abs()
                        })
                        .fold(0.0, f64::max)
                });
                range(&d, 0.1, 1.0, false)
            },
        },
        Claim {
            id: "iii-6-three.green-start",
            item: "iii-6-three-tribes",
            source: Source::Comment,
            citation: "tests/book.rs three_tribes_start_with_every_group_present: \"about 11% ... Blue, 77% Green and 11% Red\"",
            text: "about 77% of agents are Green at t = 0",
            check: |s| {
                let g = after(&preset("iii-6-three-tribes"), s, 0, |w| series(w, "group_share_1")[0]);
                range(&g, 0.77, 0.77, true)
            },
        },
        // ---- iii-9-combat ----
        Claim {
            id: "iii-9.fighting",
            item: "iii-9-combat",
            source: Source::App,
            citation: "presets.rs iii-9-combat description",
            text: "combat between two tribes happens (at least one combat death over t = 1..=500)",
            check: |s| {
                let k = each_seed(&preset("iii-9-combat"), s, |mut w| {
                    let mut k = 0;
                    for _ in 0..500 {
                        w.step();
                        k += combat_deaths(&w);
                    }
                    k as f64
                });
                range(&k, 1.0, f64::INFINITY, false)
            },
        },
        Claim {
            id: "iii-9.whole-wealth",
            item: "iii-9-combat",
            source: Source::App,
            citation: "presets.rs iii-9-combat description",
            text: "the winner accumulates its victims' whole wealth (≥ 95% of combat victims holding ≥ 10 sugar are matched to an other-tribe survivor whose one-tick gain covers the victim's sugar less both metabolisms; t = 1..=500)",
            // 95%, not 100%: a winner killed later in the same tick cannot be matched.
            check: |s| range(&whole_wealth_matched(s), 0.95, 1.0, false),
        },
        // ---- iii-11-combat-fixed ----
        Claim {
            id: "iii-11.held-at-400",
            item: "iii-11-combat-fixed",
            source: Source::App,
            citation: "presets.rs iii-11-combat-fixed description",
            text: "population held at 400 by replacement (population is 400 at every tick t = 0..=500)",
            check: |s| {
                let ok = after(&preset("iii-11-combat-fixed"), s, 500, |w| {
                    flag(series(w, "population").iter().all(|p| *p == 400.0))
                });
                range(&ok, 1.0, 1.0, false)
            },
        },
        Claim {
            id: "iii-11.reward-2",
            item: "iii-11-combat-fixed",
            source: Source::App,
            citation: "presets.rs iii-11-combat-fixed description",
            text: "fixed reward of 2 per kill (over t = 1..=300 combat deaths occur, and no surviving agent gains more than 2 + the 4-sugar maximum site in one tick)",
            check: |s| {
                let ok = each_seed(&preset("iii-11-combat-fixed"), s, |mut w| {
                    let (mut kills, mut max_gain) = (0usize, f64::MIN);
                    for _ in 0..300 {
                        let pre = snapshot(&w);
                        w.step();
                        kills += combat_deaths(&w);
                        max_gain = gains(&w, &pre).values().copied().fold(max_gain, f64::max);
                    }
                    flag(kills > 0 && max_gain <= 2.0 + MAX_SITE + 1e-9)
                });
                range(&ok, 1.0, 1.0, false)
            },
        },
        Claim {
            id: "iii-11.prolonged-fronts",
            item: "iii-11-combat-fixed",
            source: Source::App,
            citation: "presets.rs iii-11-combat-fixed description",
            text: "prolonged battle fronts (at t = 500 both tribes hold ≥ 10% of agents and there were ≥ 10 combat deaths over t = 401..=500)",
            // ≥ 10 kills per 100 ticks: fighting still going on, not a stray kill.
            check: |s| {
                let ok = each_seed(&preset("iii-11-combat-fixed"), s, |mut w| {
                    w.run(400);
                    let mut k = 0;
                    for _ in 0..100 {
                        w.step();
                        k += combat_deaths(&w);
                    }
                    flag(k >= 10 && majority_share(&w) <= 0.9)
                });
                range(&ok, 1.0, 1.0, false)
            },
        },
        // ---- iii-12-collision ----
        Claim {
            id: "iii-12.combat-off",
            item: "iii-12-collision",
            source: Source::App,
            citation: "presets.rs iii-12-collision description",
            text: "combat off, and no combat deaths over t = 1..=100",
            check: |s| {
                let ok = each_seed(&preset("iii-12-collision"), s, |mut w| {
                    let mut k = 0;
                    for _ in 0..100 {
                        w.step();
                        k += combat_deaths(&w);
                    }
                    flag(!w.config.combat.enabled && k == 0)
                });
                range(&ok, 1.0, 1.0, false)
            },
        },
        Claim {
            id: "iii-12.high-vision",
            item: "iii-12-collision",
            source: Source::App,
            citation: "presets.rs iii-12-collision description",
            text: "high-vision agents (mean vision at t = 0 higher than iii-9-combat's)",
            check: |s| {
                let v = |id| after(&preset(id), s, 0, |w| series(w, "mean_vision")[0]);
                greater(&v("iii-12-collision"), &v("iii-9-combat"), "iii-12-collision", "iii-9-combat")
            },
        },
        Claim {
            id: "iii-12.toward-center",
            item: "iii-12-collision",
            source: Source::App,
            citation: "presets.rs iii-12-collision description",
            text: "the blocks propagate toward the center (minimum over t = 1..=100 of agents' mean torus distance to the grid center is ≤ 75% of its t = 0 value)",
            // 25% closer: about 5 cells from the blocks' starting ~21, a real move.
            check: |s| {
                let r = each_seed(&preset("iii-12-collision"), s, |mut w| {
                    let d0 = mean_distance_to_center(&w);
                    let mut best = f64::INFINITY;
                    for _ in 0..100 {
                        w.step();
                        best = best.min(mean_distance_to_center(&w));
                    }
                    best / d0
                });
                range(&r, 0.0, 0.75, false)
            },
        },
        Claim {
            id: "iii-12.interpenetrate",
            item: "iii-12-collision",
            source: Source::App,
            citation: "presets.rs iii-12-collision description",
            text: "the tribes interpenetrate (maximum over t = 1..=100 of the share of adjacent agent pairs that are Blue–Red is ≥ 0.2)",
            // 0.2 is 40% of the 0.5 that fully mixed tribes would give; t = 0 gives 0.
            check: |s| {
                let r = each_seed(&preset("iii-12-collision"), s, |mut w| {
                    let mut best = 0.0f64;
                    for _ in 0..100 {
                        w.step();
                        let h = local_homogeneity(&w);
                        if h.is_finite() {
                            best = best.max(1.0 - h);
                        }
                    }
                    best
                });
                range(&r, 0.2, 1.0, false)
            },
        },
        // ---- iii-14-combat-culture ----
        Claim {
            id: "iii-14.conquest",
            item: "iii-14-combat-culture",
            source: Source::App,
            citation: "presets.rs iii-14-combat-culture description",
            text: "conquest (at least one combat death over t = 1..=500)",
            check: |s| {
                let r = conquest_and_conversion(s);
                range(&r.iter().map(|x| x[0]).collect::<Vec<_>>(), 1.0, f64::INFINITY, false)
            },
        },
        Claim {
            id: "iii-14.conversion",
            item: "iii-14-combat-culture",
            source: Source::App,
            citation: "presets.rs iii-14-combat-culture description",
            text: "conversion (at least one agent changes tribe over t = 1..=500)",
            check: |s| {
                let r = conquest_and_conversion(s);
                range(&r.iter().map(|x| x[1]).collect::<Vec<_>>(), 1.0, f64::INFINITY, false)
            },
        },
        Claim {
            id: "iii-14.together",
            item: "iii-14-combat-culture",
            source: Source::App,
            citation: "presets.rs iii-14-combat-culture description",
            text: "conquest and conversion together, as distinct from iii-9-combat",
            check: |_| untestable("\"together\" names no outcome beyond both processes occurring (iii-14.conquest, iii-14.conversion); the description states no result of their interaction to compare with iii-9-combat"),
        },
    ]
}
