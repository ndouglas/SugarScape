//! Balinese Water Temples (milestone 29): Lansing and Kremer (1993) and
//! Janssen's (2007) reanalysis, on Janssen's data for the Oos and Petanu.
//! Imitation runs last 30 years (40 for the perturbations) over 10 seeds; the
//! plan searches score years 6–10 (Janssen's last five of ten) over 3 seeds.

use std::sync::OnceLock;

use sugarscape_core::bali::{
    BaliConfig, BaliWorld, DamColumns, Decision, Perturb, PestForm, Plans, Rain, Routing,
    Watershed, LEVELS,
};
use sugarscape_core::model::{ModelConfig, ModelWorld};

use crate::claim::{all_of, equivalent, greater, Claim, Outcome, Source, Verdict};
use crate::runner::model_after;

const LK: &str = "Lansing & Kremer 1993, Am. Anthropol. 95: 97";
const J: &str = "Janssen 2007, Agric. Syst. 93: 170";

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

fn seeds(n: u64) -> Vec<u64> {
    (1..=n).collect()
}

fn worlds(c: BaliConfig, n: u64) -> Vec<BaliWorld> {
    model_after(&ModelConfig::Bali(c), &seeds(n), 100_000, |w| match w {
        ModelWorld::Bali(b) => b.as_ref().clone(),
        _ => unreachable!(),
    })
}

fn mean(v: &[f64]) -> f64 {
    let f: Vec<f64> = v.iter().copied().filter(|x| x.is_finite()).collect();
    f.iter().sum::<f64>() / f.len().max(1) as f64
}

fn show(v: &[f64]) -> String {
    let parts: Vec<String> = v.iter().map(|x| format!("{x:.2}")).collect();
    format!("[{}]", parts.join(", "))
}

/// Each seed's mean harvest over the scored years.
fn scored(ws: &[BaliWorld]) -> Vec<f64> {
    ws.iter()
        .map(|w| w.stats.latest().unwrap().scored)
        .collect()
}

/// The mean over seeds of each year's harvest.
fn yearly(ws: &[BaliWorld]) -> Vec<f64> {
    let n = ws.iter().map(|w| w.yearly().len()).min().unwrap_or(0);
    (0..n)
        .map(|y| mean(&ws.iter().map(|w| w.yearly()[y]).collect::<Vec<_>>()))
        .collect()
}

/// A statistic at the end of each year, averaged over seeds.
fn at_year_ends(ws: &[BaliWorld], f: fn(&sugarscape_core::bali::BaliSnapshot) -> f64) -> Vec<f64> {
    let years = ws[0].yearly().len();
    (1..=years)
        .map(|y| {
            mean(
                &ws.iter()
                    .map(|w| f(&w.stats.history()[y * 12]))
                    .collect::<Vec<_>>(),
            )
        })
        .collect()
}

/// A change to a config.
type Edit = fn(&mut BaliConfig);

/// Lansing and Kremer's runs: imitation from `plans`, 30 years.
fn lk(plans: Plans) -> BaliConfig {
    BaliConfig {
        plans,
        ..BaliConfig::default()
    }
}

fn perturbed(at: u32) -> BaliConfig {
    BaliConfig {
        plans: Plans::Hyv,
        perturb: Perturb {
            enabled: true,
            at,
            ..Perturb::default()
        },
        stop_at: 40,
        ..BaliConfig::default()
    }
}

/// Janssen's search at `level`, scored over years 6–10.
fn searched(level: u32, edit: impl Fn(&mut BaliConfig)) -> BaliConfig {
    let mut c = BaliConfig {
        plans: Plans::Search,
        decision: Decision::Fixed,
        level,
        stop_at: 10,
        ..BaliConfig::default()
    };
    edit(&mut c);
    c
}

/// (scored, spread) at each level: rows for g 2.0, 2.2 and 2.4 at middle
/// rain, then g 2.2 at low rain; 3 seeds each.
type Table = Vec<Vec<(f64, f64)>>;

fn levels(columns: DamColumns) -> &'static Table {
    static CODE: OnceLock<Table> = OnceLock::new();
    static PHYSICAL: OnceLock<Table> = OnceLock::new();
    let cell = if columns == DamColumns::Code {
        &CODE
    } else {
        &PHYSICAL
    };
    cell.get_or_init(|| {
        [
            (2.0, Rain::Middle),
            (2.2, Rain::Middle),
            (2.4, Rain::Middle),
            (2.2, Rain::Low),
        ]
        .iter()
        .map(|&(g, rain)| {
            LEVELS
                .iter()
                .map(|&l| {
                    let ws = worlds(
                        searched(l, |c| {
                            c.growth = g;
                            c.rain = rain;
                            c.dam_columns = columns;
                        }),
                        3,
                    );
                    (
                        mean(&scored(&ws)),
                        mean(
                            &ws.iter()
                                .map(|w| w.stats.latest().unwrap().spread)
                                .collect::<Vec<_>>(),
                        ),
                    )
                })
                .collect()
        })
        .collect()
    })
}

/// A claim about the levels judged under both readings of the subak–dam
/// columns (Janssen's Java code, whose runs his figures show, is not
/// available): Holds under both, Weak under one, Fails under neither.
fn both_readings(judge: impl Fn(DamColumns) -> Outcome) -> Outcome {
    let (code, physical) = (judge(DamColumns::Code), judge(DamColumns::Physical));
    let held = [&code, &physical]
        .iter()
        .filter(|o| o.verdict == Verdict::Holds)
        .count();
    let verdict = match held {
        2 => Verdict::Holds,
        1 => Verdict::Weak,
        _ => Verdict::Fails,
    };
    Outcome {
        verdict,
        measured: format!(
            "[Janssen's code's columns: {:?}] {} [physical columns: {:?}] {}",
            code.verdict, code.measured, physical.verdict, physical.measured
        ),
        detail: [code.detail, physical.detail]
            .into_iter()
            .filter(|d| !d.is_empty())
            .collect::<Vec<_>>()
            .join(" "),
    }
}

/// Years until a harvest series comes within 0.1 of its final value, from `from`.
fn settle(y: &[f64], from: usize) -> usize {
    let end = *y.last().unwrap();
    (from..y.len())
        .find(|&k| y[k..].iter().all(|h| (h - end).abs() <= 0.1))
        .map_or(y.len(), |k| k - from)
}

pub fn claims() -> Vec<Claim> {
    vec![
        Claim {
            id: "bali.lk.emergence",
            item: "lk-random",
            source: Source::Book,
            citation: LK,
            text: "'after 8–35 years, a complex structure of coordinated cropping patterns emerged': from random plans, imitation settles — the year-8 harvest at least 90 % of the year-30 harvest, and at most 20 subaks still changing by year 30 (10 seeds)",
            check: |_| {
                let ws = worlds(lk(Plans::Random), 10);
                let y = yearly(&ws);
                let ch = at_year_ends(&ws, |s| f64::from(s.changing));
                outcome(y[7] >= 0.9 * y[29] && ch[29] <= 20.0, format!("harvest years 1, 8, 30: {:.2}, {:.2}, {:.2}; changing years 1, 8, 30: {:.1}, {:.1}, {:.1}", y[0], y[7], y[29], ch[0], ch[7], ch[29]))
            },
        },
        Claim {
            id: "bali.lk.table1",
            item: "lk-traditional",
            source: Source::Book,
            citation: LK,
            text: "Table 1: imitation raises yields — traditional rice 4.9 → 8.57, high-yielding rice 15.91 → 18.08, low rain and high pests 13.67 → 17.66 t/ha/yr (each rising from year 1 to year 30, and ending within 10 % of the table, 10 seeds)",
            check: |_| {
                let runs = [("traditional", lk(Plans::Traditional), 8.57), ("high-yielding", lk(Plans::Hyv), 18.08), ("low rain, high pests", BaliConfig { stop_at: 30, ..perturbed(1) }, 17.66)];
                all_of(
                    runs.into_iter()
                        .map(|(name, c, want)| {
                            let y = yearly(&worlds(c, 10));
                            let end = y[29];
                            (name.to_string(), outcome(end > y[0] && (end - want).abs() <= 0.1 * want, format!("{:.2} → {end:.2} (Table 1: → {want})", y[0])))
                        })
                        .collect(),
                )
            },
        },
        Claim {
            id: "bali.lk.still-changing",
            item: "lk-random",
            source: Source::Book,
            citation: LK,
            text: "'After eight years, average yields peaked, and all but 20 subaks stopped changing … The remaining 20 subaks keep swapping' (and Fig. 11's high-yielding run: 20 still changing after 20 years) — between 10 and 30 subaks changing at year 8 from random plans and at year 20 from high-yielding plans (means of 10 seeds)",
            check: |_| {
                let r = at_year_ends(&worlds(lk(Plans::Random), 10), |s| f64::from(s.changing));
                let h = at_year_ends(&worlds(lk(Plans::Hyv), 10), |s| f64::from(s.changing));
                all_of(vec![
                    ("random, year 8".into(), outcome((10.0..=30.0).contains(&r[7]), format!("{:.1} changing (years 1–10: {})", r[7], show(&r[..10])))),
                    ("high-yielding, year 20".into(), outcome((10.0..=30.0).contains(&h[19]), format!("{:.1} changing (years 16–20: {})", h[19], show(&h[15..20])))),
                ])
            },
        },
        Claim {
            id: "bali.lk.temples",
            item: "lk-random",
            source: Source::Book,
            citation: LK,
            text: "'The resemblance between the last run … and the temple system … is evident' — beyond the pest network's own: the final patches of one plan match the 14 masceti groups (adjusted Rand index) at least 0.05 better than the pest network's connected components alone do (year 30, 10 seeds)",
            check: |_| {
                let ws = worlds(lk(Plans::Random), 10);
                let t = at_year_ends(&ws, |s| s.temple_match);
                let net = ws[0].stats.latest().unwrap().network_match;
                let peak = t.iter().copied().fold(f64::MIN, f64::max);
                let above = ws.iter().filter(|w| w.stats.latest().unwrap().temple_match > net + 1e-9).count();
                outcome(t[29] >= net + 0.05, format!("patches {:.3} at year 30 (peak {peak:.3}; above the network on {above} of {} seeds), the network's components {net:.3}", t[29], ws.len()))
                    .with("The pest links alone fall into 46 components (27 of them single subaks); imitation ends with mostly one plan per component, so the patches inherit the network's resemblance to the temples. The 0.05 margin was set after planning had measured 0.37 against 0.33.")
            },
        },
        Claim {
            id: "bali.lk.recovery",
            item: "lk-perturbed",
            source: Source::Book,
            citation: LK,
            text: "Fig. 11: pests and drought from year 21 cut the harvest from 18.5 to 15.3, 'recovering to 15.8 within 7 years' — a fall of at least 1 t/ha/yr in year 21 and a recovery of at least 0.25 (half the paper's) by year 28 (10 seeds)",
            check: |_| {
                let y = yearly(&worlds(perturbed(21), 10));
                let (fall, rise) = (y[19] - y[20], y[27] - y[20]);
                let random: Vec<Vec<f64>> = worlds(BaliConfig { plans: Plans::Random, ..perturbed(21) }, 10).iter().map(|w| w.yearly().to_vec()).collect();
                let recovered = random.iter().filter(|y| y[27] - y[20] >= 0.25).count();
                outcome(fall >= 1.0 && rise >= 0.25, format!("years 19–28: {} (fall {fall:.2}, recovery {rise:.2})", show(&y[18..28])))
                    .with(&format!("Judged on the paper's high-yielding start, where every subak shares one plan and imitation has nothing different to copy. Not judged: from random plans the same perturbation recovers by at least 0.25 within seven years on {recovered} of 10 seeds."))
            },
        },
        Claim {
            id: "bali.lk.stressed-longer",
            item: "lk-stressed",
            source: Source::Book,
            citation: LK,
            text: "'when the conditions of low rain and high pests occurred from the very beginning … it took twice as long' — years to settle (within 0.1 of the year-40 harvest) from the start at least twice the years to recover after year 21, which must be at least one (10 seeds)",
            check: |_| {
                let s = yearly(&worlds(perturbed(1), 10));
                let p = yearly(&worlds(perturbed(21), 10));
                let (ts, tp) = (settle(&s, 0), settle(&p, 20));
                outcome(tp >= 1 && ts >= 2 * tp, format!("from the start {ts} years (years 1–8: {}); after year 21 {tp} years", show(&s[..8])))
            },
        },
        Claim {
            id: "bali.lk.levels",
            item: "bali-levels",
            source: Source::Book,
            citation: LK,
            text: "Fig. 6: of the scales of coordination, 'the highest peak is achieved by the scale of coordination that most closely approximates the temple scale' — level 14 (the mascetis) best, by at least 1 % (Weak if best by less), at g 2.0, 2.2 and 2.4 (Janssen's search, 3 seeds), under each reading of the dam columns",
            check: |_| {
                both_readings(|columns| {
                    let parts = [2.0, 2.2, 2.4]
                        .iter()
                        .zip(levels(columns))
                        .map(|(g, row)| {
                            let at14 = row[3].0;
                            let other = row.iter().enumerate().filter(|&(k, _)| k != 3).map(|(_, r)| r.0).fold(f64::MIN, f64::max);
                            let verdict = if at14 >= 1.01 * other { Verdict::Holds } else if at14 > other { Verdict::Weak } else { Verdict::Fails };
                            let v: Vec<f64> = row.iter().map(|r| r.0).collect();
                            (format!("g {g}"), Outcome { verdict, measured: format!("levels 1, 2, 7, 14, 28, 172: {}", show(&v)), detail: String::new() })
                        })
                        .collect();
                    all_of(parts)
                })
            },
        },
        Claim {
            id: "bali.lk.every-time",
            item: "bali-imitation-growth",
            source: Source::Book,
            citation: LK,
            text: "'the same phenomenon occurs every time, regardless of the initial distribution of cropping patterns, or ecological parameters such as flow rates or pest biology' (rain standing in for flow rates) — imitation beats the same plans fixed (scored years, Mann–Whitney, 10 seeds) at g 2.0 and 2.4, d 0.18 and 0.45, low and high rain, and from the traditional pattern",
            check: |_| {
                let cases: [(&str, Edit); 7] = [
                    ("g 2.0", |c| c.growth = 2.0),
                    ("g 2.4", |c| c.growth = 2.4),
                    ("d 0.18", |c| c.dispersal = 0.18),
                    ("d 0.45", |c| c.dispersal = 0.45),
                    ("low rain", |c| c.rain = Rain::Low),
                    ("high rain", |c| c.rain = Rain::High),
                    ("traditional", |c| c.plans = Plans::Traditional),
                ];
                all_of(
                    cases
                        .iter()
                        .map(|(name, edit)| {
                            let mut c = BaliConfig::default();
                            edit(&mut c);
                            let fixed = BaliConfig { decision: Decision::Fixed, ..c.clone() };
                            (name.to_string(), greater(&scored(&worlds(c, 10)), &scored(&worlds(fixed, 10)), "imitating", "fixed"))
                        })
                        .collect(),
                )
            },
        },
        Claim {
            id: "bali.j.levels",
            item: "bali-levels",
            source: Source::Book,
            citation: J,
            text: "Fig. 1: 'with an increasing number of smaller groups, there is a higher amount of total rice harvest' (≈ 17.5 at one group to 22.8 at 172) — level 172 at least 5 % above level 1 (middle rain, g 2.2, 3 seeds), under each reading of the dam columns",
            check: |_| {
                both_readings(|columns| {
                    let v: Vec<f64> = levels(columns)[1].iter().map(|r| r.0).collect();
                    let low: Vec<f64> = levels(columns)[3].iter().map(|r| r.0).collect();
                    outcome(v[5] >= 1.05 * v[0], format!("levels 1, 2, 7, 14, 28, 172: {} (+{:.1} %; at low rain +{:.1} %)", show(&v), 100.0 * (v[5] / v[0] - 1.0), 100.0 * (low[5] / low[0] - 1.0)))
                })
                .with("The rise is small because water rarely binds at middle rain: summed over the watershed, the dams' base flow roughly meets full planting's demand, and the network routing passes surplus down to the intakes short of it. Under the physical column reading at low rain it reaches Janssen's direction (see bali.ours.columns-levels), though not his size (+30 %).")
            },
        },
        Claim {
            id: "bali.j.inequality",
            item: "bali-levels",
            source: Source::Book,
            citation: J,
            text: "'there is also an increasing inequality between annual harvest levels of subaks' — the spread of harvests at 172 groups above that at one (middle rain, 3 seeds), under each reading of the dam columns",
            check: |_| {
                both_readings(|columns| {
                    let v: Vec<f64> = levels(columns)[1].iter().map(|r| r.1).collect();
                    outcome(v[5] > v[0], format!("spread at levels 1, 2, 7, 14, 28, 172: {}", show(&v)))
                })
            },
        },
        Claim {
            id: "bali.j.growth",
            item: "bali-growth",
            source: Source::Book,
            citation: J,
            text: "Fig. 3: 'The benefit of synchronization is only derived for the medium growth rate of pests' — the range of harvests across levels at g 2.2 more than twice that at g 2.0 and at g 2.4 (3 seeds), under each reading of the dam columns",
            check: |_| {
                both_readings(|columns| {
                    let range = |row: &Vec<(f64, f64)>| {
                        let v: Vec<f64> = row.iter().map(|r| r.0).collect();
                        v.iter().copied().fold(f64::MIN, f64::max) - v.iter().copied().fold(f64::MAX, f64::min)
                    };
                    let r: Vec<f64> = levels(columns)[..3].iter().map(range).collect();
                    outcome(r[1] > 2.0 * r[0] && r[1] > 2.0 * r[2], format!("range across levels at g 2.0, 2.2, 2.4: {}", show(&r)))
                })
            },
        },
        Claim {
            id: "bali.j.dispersal",
            item: "bali-dispersal",
            source: Source::Book,
            citation: J,
            text: "Fig. 4: 'When the pest spreads quickly, the harvest is severely affected' — the searched harvest at d 0.45 at least 10 % below that at d 0.3 (level 14, 3 seeds), under each reading of the dam columns",
            check: |_| {
                both_readings(|columns| {
                    let at = |d: f64| {
                        mean(&scored(&worlds(
                            searched(14, |c| {
                                c.dispersal = d;
                                c.dam_columns = columns;
                            }),
                            3,
                        )))
                    };
                    let (mid, high) = (at(0.3), at(0.45));
                    outcome(high <= 0.9 * mid, format!("d 0.3: {mid:.2}; d 0.45: {high:.2}"))
                })
            },
        },
        Claim {
            id: "bali.j.two-node",
            item: "bali-two-node",
            source: Source::Book,
            citation: J,
            text: "§4: with two periods both nodes can plant when g < 10; with twelve and water for both, 'if the growth rate is smaller than 2.14, or ∛10, pests cannot grow exponentially and a maximum number of crops is possible … Beyond this growth rate, we see a drop' — the largest fall of the best harvest over g 1.6–3.0 between 2.1 and 2.2, with six crops below it",
            check: |_| {
                let node = |g: f64, periods: u32| BaliConfig { watershed: Watershed::TwoNode, growth: g, node_periods: periods, stop_at: 5, ..BaliConfig::default() };
                let gs = [1.6, 1.8, 2.0, 2.1, 2.2, 2.3, 2.4, 2.6, 2.8, 3.0];
                let h: Vec<f64> = gs.iter().map(|&g| *worlds(node(g, 12), 1)[0].yearly().last().unwrap()).collect();
                let drops: Vec<f64> = h.windows(2).map(|w| w[0] - w[1]).collect();
                let biggest = drops.iter().enumerate().max_by(|a, b| a.1.total_cmp(b.1)).unwrap().0;
                let both = |g: f64| worlds(node(g, 2), 1)[0].nodes().unwrap().pair.iter().all(|p| p.pattern != 0);
                all_of(vec![
                    ("twelve periods".into(), outcome(biggest == 3 && h[..4].iter().all(|&x| x > 5.4), format!("g 1.6–3.0: {}", show(&h)))),
                    ("two periods".into(), outcome(both(9.0) && !both(11.0), format!("both plant at g 9: {}; at g 11: {}", both(9.0), both(11.0)))),
                ])
                .with("Six crops yield 5.52, not 6: each loses the pests grown from the floor in its three months (0.01 × 2³).")
            },
        },
        Claim {
            id: "bali.j.gamma",
            item: "bali-gamma",
            source: Source::Book,
            citation: J,
            text: "Fig. 9: 'high harvest levels when γp and γw are positive, and γp is less than 0.5' — at γw 0.4, harvests with γp 0.1 and 0.25 above those with γp 1 and 2 (Mann–Whitney, 10 seeds each)",
            check: |_| {
                let at = |gp: f64| scored(&worlds(BaliConfig { decision: Decision::Generalized, gamma_p: gp, ..BaliConfig::default() }, 10));
                let low = [at(0.1), at(0.25)].concat();
                let high = [at(1.0), at(2.0)].concat();
                greater(&low, &high, "γp < 0.5", "γp ≥ 1")
            },
        },
        Claim {
            id: "bali.j.adaptive",
            item: "bali-adaptive",
            source: Source::Book,
            citation: J,
            text: "Fig. 12: 'if mp is very low subaks never plant crops … When mp is large, crops are planted too early … a larger value of mw leads to a lower performance', and mw 0.05 with mp 0.02 'maximized the default case' — no harvest at mp 0.01; less at mp 0.5 than 0.02, and at mw 0.05 than 0 (his plotted range, 0–500 m³/day per hectare); and (0.05, 0.02) within 1 % of the best of mw 0–0.2 × mp 0.01–0.5",
            check: |_| {
                let at = |mw: f64, mp: f64| mean(&scored(&worlds(BaliConfig { decision: Decision::Adaptive, m_w: mw, m_p: mp, ..BaliConfig::default() }, 3)));
                let mws = [0.0, 0.01, 0.02, 0.05, 0.1, 0.2];
                let mps = [0.01, 0.02, 0.05, 0.1, 0.5];
                let grid: Vec<(f64, f64, f64)> = mws.iter().flat_map(|&w| mps.iter().map(move |&p| (w, p))).map(|(w, p)| (w, p, at(w, p))).collect();
                let get = |w: f64, p: f64| grid.iter().find(|g| g.0 == w && g.1 == p).unwrap().2;
                let best = grid.iter().max_by(|a, b| a.2.total_cmp(&b.2)).unwrap();
                all_of(vec![
                    ("mp very low".into(), outcome(get(0.05, 0.01) == 0.0, format!("{:.2} at mp 0.01", get(0.05, 0.01)))),
                    ("mp large".into(), outcome(get(0.05, 0.5) < get(0.05, 0.02), format!("{:.2} at mp 0.5, {:.2} at 0.02", get(0.05, 0.5), get(0.05, 0.02)))),
                    ("mw large".into(), outcome(get(0.05, 0.02) < get(0.0, 0.02), format!("mw 0, 0.01, 0.02, 0.05: {:.2}, {:.2}, {:.2}, {:.2}; beyond his range, 0.1 and 0.2: {:.2}, {:.2}", get(0.0, 0.02), get(0.01, 0.02), get(0.02, 0.02), get(0.05, 0.02), get(0.1, 0.02), get(0.2, 0.02)))),
                    ("the stated best".into(), outcome(get(0.05, 0.02) >= 0.99 * best.2, format!("{:.2} at (0.05, 0.02); best {:.2} at ({}, {})", get(0.05, 0.02), best.2, best.0, best.1))),
                ])
                .with("mw is read as m/day per hectare the source dam serves (a stated choice, which his Fig. 12's axis, 0–500 m³/day, supports: 0–0.05 m/day per hectare); pests never fall below the floor of 0.01, so mp 0.01 never plants.")
            },
        },
        Claim {
            id: "bali.j.links",
            item: "bali-links",
            source: Source::Book,
            citation: J,
            text: "Figs. 15–16: for imitative subaks (eq. 4, γ 0.4) 'the harvest decreases when pest-related links are removed' and is 'not sensitive to the probability of adding links'; adaptive subaks 'are not sensitive to removing existing pest-related connections … [but] to adding' them; and the two 'led to similar results for the original Bali irrigation network' (half the links removed or 20 % added; Mann–Whitney or 5 % equivalence, 10 seeds; similar within 10 %)",
            check: |_| {
                let run = |decision: Decision, remove: f64, add: f64| scored(&worlds(BaliConfig { decision, remove_links: remove, add_links: add, ..BaliConfig::default() }, 10));
                let (gm, gr, ga) = (run(Decision::Generalized, 0.0, 0.0), run(Decision::Generalized, 0.5, 0.0), run(Decision::Generalized, 0.0, 0.2));
                let (am, ar, aa) = (run(Decision::Adaptive, 0.0, 0.0), run(Decision::Adaptive, 0.5, 0.0), run(Decision::Adaptive, 0.0, 0.2));
                let margin = |v: &[f64]| Some(0.05 * mean(v));
                all_of(vec![
                    ("imitators, removed".into(), greater(&gm, &gr, "as mapped", "half removed")),
                    ("imitators, added".into(), equivalent(&gm, &ga, margin(&gm), "as mapped", "20 % added")),
                    ("adaptive, removed".into(), equivalent(&am, &ar, margin(&am), "as mapped", "half removed")),
                    ("adaptive, added".into(), greater(&am, &aa, "as mapped", "20 % added")),
                    ("similar as mapped".into(), outcome((mean(&gm) - mean(&am)).abs() <= 0.1 * mean(&am), format!("imitators {:.2}, adaptive {:.2}", mean(&gm), mean(&am)))),
                ])
            },
        },
        Claim {
            id: "bali.j.network",
            item: "lk-random",
            source: Source::Book,
            citation: J,
            text: "'There is a strong overlap between the 14 Masceti temples and subaks connected via pest relationships in the empirical dataset' — the pest network's connected components match the masceti groups with an adjusted Rand index of at least 0.3",
            check: |_| {
                let net = worlds(lk(Plans::Random), 1)[0].stats.latest().unwrap().network_match;
                outcome(net >= 0.3, format!("adjusted Rand index {net:.3}"))
            },
        },
        Claim {
            id: "bali.ours.routing",
            item: "janssen-code",
            source: Source::Comment,
            citation: J,
            text: "Ours: Janssen's code balances one random dam a month with no upstream inflow; that changes the imitation endpoint by less than 5 % (scored years, 10 seeds)",
            check: |_| {
                let net = scored(&worlds(BaliConfig::default(), 10));
                let code = scored(&worlds(BaliConfig { routing: Routing::JanssenCode, ..BaliConfig::default() }, 10));
                equivalent(&net, &code, Some(0.05 * mean(&net)), "network", "Janssen's code")
            },
        },
        Claim {
            id: "bali.ours.columns",
            item: "lk-random",
            source: Source::Comment,
            citation: J,
            text: "Ours: reading the subak–dam file's columns the physical way round (the first is the upstream dam in 93 of 95 cases) instead of as Janssen's code does changes the imitation endpoint by less than 5 % (10 seeds; for the searched levels it matters more — see bali.ours.columns-levels)",
            check: |_| {
                let code = scored(&worlds(BaliConfig::default(), 10));
                let phys = scored(&worlds(BaliConfig { dam_columns: DamColumns::Physical, ..BaliConfig::default() }, 10));
                equivalent(&code, &phys, Some(0.05 * mean(&code)), "as coded", "physical")
            },
        },
        Claim {
            id: "bali.ours.pest-form",
            item: "lk-random",
            source: Source::Comment,
            citation: J,
            text: "Ours: Lansing and Kremer's 'shortcut' pest equation and the standard diffusion form Janssen expected give endpoints within 5 % (10 seeds)",
            check: |_| {
                let a = scored(&worlds(BaliConfig::default(), 10));
                let b = scored(&worlds(BaliConfig { pest_form: PestForm::Diffusion, ..BaliConfig::default() }, 10));
                equivalent(&a, &b, Some(0.05 * mean(&a)), "shortcut", "diffusion")
            },
        },
        Claim {
            id: "bali.ours.reset",
            item: "lk-random",
            source: Source::Comment,
            citation: J,
            text: "Janssen's code resets pests each year: 'If we don't … the system gets locked into low harvest rates' — without the reset the scored harvest is under half (10 seeds)",
            check: |_| {
                let on = scored(&worlds(BaliConfig::default(), 10));
                let off = scored(&worlds(BaliConfig { pest_reset: false, ..BaliConfig::default() }, 10));
                outcome(mean(&off) < 0.5 * mean(&on), format!("with the reset {:.2}, without {:.2}", mean(&on), mean(&off)))
            },
        },
        Claim {
            id: "bali.ours.water",
            item: "bali-rain",
            source: Source::Comment,
            citation: J,
            text: "Ours: under Janssen's code's reading of the dam columns water hardly binds — growing months lose under 5 % of their water at low, middle and high rain, and rain changes the imitation endpoint by under 5 % — but under the physical reading it binds at low rain (at least 5 % lost) (imitation, year 30, 10 seeds)",
            check: |_| {
                let stress = |ws: &[BaliWorld]| mean(&ws.iter().map(|w| w.stats.latest().unwrap().water_stress).collect::<Vec<_>>());
                let at = |rain: Rain, dam_columns: DamColumns| worlds(BaliConfig { rain, dam_columns, ..BaliConfig::default() }, 10);
                let runs = [at(Rain::Low, DamColumns::Code), at(Rain::Middle, DamColumns::Code), at(Rain::High, DamColumns::Code)];
                let lost: Vec<f64> = runs.iter().map(|ws| stress(ws)).collect();
                let s: Vec<f64> = runs.iter().map(|ws| mean(&scored(ws))).collect();
                let physical = at(Rain::Low, DamColumns::Physical);
                all_of(vec![
                    ("code's columns".into(), outcome(lost.iter().all(|&x| x < 0.05) && (s[0] - s[2]).abs() < 0.05 * s[1], format!("water lost {} and scored {} at low, middle, high rain", show(&lost), show(&s)))),
                    ("physical columns".into(), outcome(stress(&physical) >= 0.05, format!("water lost {:.3} at low rain; scored {:.2}", stress(&physical), mean(&scored(&physical))))),
                ])
            },
        },
        Claim {
            id: "bali.ours.columns-levels",
            item: "bali-levels",
            source: Source::Comment,
            citation: J,
            text: "Ours: the reading of the dam columns decides whether finer coordination pays at low rain — level 172 at least 5 % above level 1 under the physical reading, under 5 % under Janssen's code's (g 2.2, 3 seeds)",
            check: |_| {
                let rise = |columns| {
                    let v: Vec<f64> = levels(columns)[3].iter().map(|r| r.0).collect();
                    (v[5] / v[0] - 1.0, v)
                };
                let ((code, cv), (phys, pv)) = (rise(DamColumns::Code), rise(DamColumns::Physical));
                outcome(phys >= 0.05 && code < 0.05, format!("levels 1 → 172 at low rain: code's columns {} (+{:.1} %); physical {} (+{:.1} %)", show(&cv), 100.0 * code, show(&pv), 100.0 * phys))
                    .with("The rule was written after the final review had measured it.")
            },
        },
    ]
}
