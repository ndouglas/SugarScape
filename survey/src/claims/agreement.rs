//! Relative agreement (milestone 22): Deffuant et al.'s pairwise bounded
//! confidence (2000), relative agreement with extremists (2002) and its §6
//! variants, Meadows and Cliff's replication (2012) and the authors' reply
//! (2013), Amblard and Deffuant's networks (2004) and Weisbuch's (2004).
//! Runs go to stability unless a claim says otherwise; runs several claims
//! share are memoized per process, keyed by the config, the seeds and the cap.

use std::sync::{Arc, Mutex};

use sugarscape_core::agreement::{
    AgreementConfig, Network, PairUpdate, Pairing, Placement, Rule, ScaleFreeConfig,
    SmallWorldConfig, Substrate, Window, GAP,
};
use sugarscape_core::model::{ModelConfig, ModelWorld};
use sugarscape_core::opinions::groups;

use crate::claim::{all_of, equivalent, greater, Claim, Outcome, Source, Verdict};
use crate::runner::model_after;

const DNAW: &str = "Deffuant, Neau, Amblard & Weisbuch 2000, ACS 3";
const DAWF: &str = "Deffuant, Amblard, Weisbuch & Faure 2002, JASSS 5(4) 1";
const MC: &str = "Meadows & Cliff 2012, JASSS 15(4) 4";
const DAW: &str = "Deffuant, Amblard & Weisbuch 2013, JASSS 16(1) 11";
const AD: &str = "Amblard & Deffuant 2004, Physica A 343";
const W: &str = "Weisbuch 2004, EPJ B 38";
/// Far past any fully connected stability time (hundreds of periods; a
/// lattice's can take thousands).
const CAP: u32 = 20_000;

/// One run at its end.
#[derive(Clone, Copy, Debug)]
struct Run {
    y: f64,
    p_plus: f64,
    p_minus: f64,
    outcome: u32,
    major: f64,
    isolated: f64,
    largest: f64,
    dispersion: f64,
    unmoved: f64,
    /// |mean opinion|: how far the population has drifted.
    drift: f64,
    tick: f64,
    stable: bool,
    /// The share of the ten best-connected agents in the largest cluster
    /// (NaN when anyone meets anyone).
    hubs_in_largest: f64,
}

impl Run {
    fn central(&self) -> bool {
        self.outcome == 0
    }
    fn both(&self) -> bool {
        self.outcome == 1
    }
    fn single(&self) -> bool {
        self.outcome == 2
    }
}

fn summarize(w: &ModelWorld) -> Run {
    let m = w.model();
    let last = |n: &str| m.latest_value(n).expect("an agreement series");
    let hubs_in_largest = match &w {
        ModelWorld::Agreement(a) if a.config.network != Network::All => {
            // agents.csv: id,role,start,opinion,uncertainty,degree,meetings,moves
            let rows: Vec<(f64, u32)> = m
                .agents_csv()
                .lines()
                .skip(1)
                .map(|l| {
                    let f: Vec<&str> = l.split(',').collect();
                    (f[3].parse().unwrap(), f[5].parse().unwrap())
                })
                .collect();
            let mut sorted: Vec<f64> = rows.iter().map(|r| r.0).collect();
            sorted.sort_by(f64::total_cmp);
            // The largest group's opinion range.
            let sizes = groups(&sorted, GAP);
            let (mut at, mut best) = (0, (0, 0));
            for &s in &sizes {
                if s > best.1 - best.0 {
                    best = (at, at + s);
                }
                at += s;
            }
            let (lo, hi) = (sorted[best.0], sorted[best.1 - 1]);
            let mut by_degree = rows.clone();
            by_degree.sort_by(|a, b| b.1.cmp(&a.1));
            let hubs = &by_degree[..10];
            hubs.iter().filter(|r| r.0 >= lo && r.0 <= hi).count() as f64 / 10.0
        }
        _ => f64::NAN,
    };
    let c = match m.config() {
        ModelConfig::Agreement(c) => c,
        _ => unreachable!(),
    };
    let tick = m.tick() as f64;
    Run {
        y: last("y"),
        p_plus: last("p_plus"),
        p_minus: last("p_minus"),
        outcome: last("outcome") as u32,
        major: last("major"),
        isolated: last("isolated"),
        largest: last("largest"),
        dispersion: last("dispersion"),
        unmoved: last("unmoved"),
        drift: last("mean_opinion").abs(),
        tick,
        stable: c.stop_when_stable
            && m.finished()
            && (c.stop_at == 0 || tick < f64::from(c.stop_at)),
        hubs_in_largest,
    }
}

/// The model's defaults with `edit`, seeds 1..=`seeds`, at most `cap` periods.
fn runs(seeds: u64, cap: u32, edit: impl FnOnce(&mut AgreementConfig)) -> Arc<Vec<Run>> {
    type Cache = Mutex<Vec<(String, u64, u32, Arc<Vec<Run>>)>>;
    static CACHE: Cache = Mutex::new(Vec::new());
    let mut c = AgreementConfig::default();
    edit(&mut c);
    let key = serde_json::to_string(&c).expect("configs serialize");
    if let Some((_, _, _, v)) = CACHE
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .iter()
        .find(|(k, s, t, _)| *k == key && *s == seeds && *t == cap)
    {
        return v.clone();
    }
    let seed_list: Vec<u64> = (1..=seeds).collect();
    let v = Arc::new(model_after(
        &ModelConfig::Agreement(c),
        &seed_list,
        cap,
        summarize,
    ));
    CACHE
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .push((key, seeds, cap, v.clone()));
    v
}

fn mean(runs: &[Run], f: impl Fn(&Run) -> f64) -> f64 {
    runs.iter().map(f).sum::<f64>() / runs.len() as f64
}

fn col(runs: &[Run], f: impl Fn(&Run) -> f64) -> Vec<f64> {
    runs.iter().map(f).collect()
}

fn count(runs: &[Run], pred: impl Fn(&Run) -> bool) -> usize {
    runs.iter().filter(|r| pred(r)).count()
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

/// A share of runs with `pred`: holds when it lies in `lo..=hi`.
fn share(runs: &[Run], pred: impl Fn(&Run) -> bool, lo: f64, hi: f64, what: &str) -> Outcome {
    let k = count(runs, pred);
    let s = k as f64 / runs.len() as f64;
    outcome(
        (lo..=hi).contains(&s),
        format!("{k}/{} runs {what} ({:.0} %)", runs.len(), 100.0 * s),
    )
}

/// How many runs end central, in both extremes and in one.
fn outcomes(runs: &[Run]) -> String {
    format!(
        "central {}, both extremes {}, single extreme {}, intermediate {} of {}; mean y {:.2}",
        count(runs, Run::central),
        count(runs, Run::both),
        count(runs, Run::single),
        runs.len() - count(runs, Run::central) - count(runs, Run::both) - count(runs, Run::single),
        runs.len(),
        mean(runs, |r| r.y)
    )
}

/// DNAW's pairwise model: bounded confidence without extremists, d on
/// [0, 1] as U = 2d.
fn dnaw(d: f64, agents: u32) -> impl FnOnce(&mut AgreementConfig) {
    move |c| {
        c.rule = Rule::Bc;
        c.extremists = 0.0;
        c.uncertainty = 2.0 * d;
        c.mu = 0.5;
        c.agents = agents;
    }
}

/// Relative agreement with extremists at Fig. 9's μ and ue.
fn ra(agents: u32, pe: f64, u: f64) -> impl FnOnce(&mut AgreementConfig) {
    move |c| {
        c.agents = agents;
        c.extremists = pe;
        c.uncertainty = u;
    }
}

/// Meadows and Cliff's placement at pe 0.05, U 1.4, N 200, read at `stop`
/// (0: to stability) with a margin.
fn reading(margin: f64, stop: u32) -> impl FnOnce(&mut AgreementConfig) {
    move |c| {
        c.extremists = 0.05;
        c.uncertainty = 1.4;
        c.placement = Placement::Band;
        c.extreme_margin = margin;
        if stop > 0 {
            c.stop_when_stable = false;
            c.stop_at = stop;
        }
    }
}

/// §6's rules at N 1000, pe 0.05.
fn bc(rule: Rule, window: Window, u: f64, delta: f64) -> impl FnOnce(&mut AgreementConfig) {
    move |c| {
        c.agents = 1000;
        c.rule = rule;
        c.window = window;
        c.extremists = 0.05;
        c.uncertainty = u;
        c.delta = delta;
    }
}

/// Amblard and Deffuant's small world: N 1000, U 1.8, pe 0.05 at ±1, μ 0.1.
fn small_world(k: u32, p: f64, margin: f64) -> impl FnOnce(&mut AgreementConfig) {
    move |c| {
        c.agents = 1000;
        c.mu = 0.1;
        c.uncertainty = 1.8;
        c.extremists = 0.05;
        c.placement = Placement::Bounds;
        c.extreme_margin = margin;
        c.network = Network::SmallWorld;
        c.small_world = SmallWorldConfig {
            substrate: Substrate::Ring,
            degree: k,
            rewire: p,
        };
    }
}

/// The smallest k (powers of 2) at which most runs end in a single extreme.
fn critical_k(p: f64, margin: f64) -> Option<u32> {
    [2, 4, 8, 16, 32, 64, 128, 256]
        .into_iter()
        .find(|&k| 2 * count(&runs(20, 5_000, small_world(k, p, margin)), Run::single) > 20)
}

/// Weisbuch's networks: pairwise bounded confidence at d, N 900, μ 0.5; on
/// networks a random agent picks a random neighbor and only it moves.
fn weisbuch(net: &'static str, d: f64) -> impl FnOnce(&mut AgreementConfig) {
    move |c| {
        dnaw(d, 900)(c);
        match net {
            "all" => {}
            "lattice" => {
                c.network = Network::Lattice;
                c.lattice.width = 30;
                c.lattice.height = 30;
                c.pairing = Pairing::Node;
                c.pair_update = PairUpdate::OneWay;
            }
            _ => {
                c.network = Network::ScaleFree;
                c.scale_free = ScaleFreeConfig {
                    links: if net == "sf8" { 4 } else { 2 },
                };
                c.pairing = Pairing::Node;
                c.pair_update = PairUpdate::OneWay;
            }
        }
    }
}

const FIG9_U: [f64; 19] = [
    0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0, 1.1, 1.2, 1.3, 1.4, 1.5, 1.6, 1.7, 1.8, 1.9, 2.0,
];

/// Fig. 9's pe axis: 0.025 to 0.3 by 0.0125.
fn fig9_pe() -> Vec<f64> {
    (0..23).map(|i| 0.025 + 0.0125 * f64::from(i)).collect()
}

/// Mean y over 50 runs at each grid cell with `keep(U, pe)`, N agents, δ.
fn fig9_cells(agents: u32, delta: f64, keep: impl Fn(f64, f64) -> bool) -> Vec<(f64, f64, f64)> {
    let mut out = Vec::new();
    for pe in fig9_pe() {
        for u in FIG9_U {
            if keep(u, pe) {
                let r = runs(50, CAP, move |c| {
                    ra(agents, pe, u)(c);
                    c.delta = delta;
                });
                out.push((u, pe, mean(&r, |r| r.y)));
            }
        }
    }
    out
}

pub fn claims() -> Vec<Claim> {
    vec![
        Claim {
            id: "agreement.dnaw.consensus",
            item: "dnaw-consensus",
            source: Source::Book,
            citation: DNAW,
            text: "Figs. 1–2: 'uniformity is only achieved for the larger value of d' — consensus at d 0.5, two clusters at d 0.2 (N 1000, μ 0.5)",
            check: |_| {
                let one = runs(50, CAP, dnaw(0.5, 1000));
                let two = runs(50, CAP, dnaw(0.2, 1000));
                let (a, b) = (count(&one, |r| r.largest >= 0.99), count(&two, |r| r.major == 2.0));
                outcome(a >= 45 && b >= 45, format!("consensus in {a}/50 at d 0.5; two major clusters in {b}/50 at d 0.2"))
            },
        },
        Claim {
            id: "agreement.dnaw.peaks",
            item: "dnaw-clusters",
            source: Source::Book,
            citation: DNAW,
            text: "Fig. 4: the number of peaks falls as d grows, at most about 1/(2d) (N 1000, μ 0.5, wings excluded; 50 of the paper's 250 samples)",
            check: |_| {
                let ds = [0.1, 0.15, 0.2, 0.25, 0.3, 0.35, 0.4, 0.45];
                let means: Vec<f64> = ds.iter().map(|&d| mean(&runs(50, CAP, dnaw(d, 1000)), |r| r.major)).collect();
                let falling = means.windows(2).all(|w| w[1] <= w[0] + 0.05);
                let bounded = ds.iter().zip(&means).all(|(&d, &m)| m <= 1.0 / (2.0 * d) + 0.5);
                let shown: Vec<String> = ds.iter().zip(&means).map(|(d, m)| format!("{m:.2} at d {d}")).collect();
                outcome(falling && bounded, format!("major clusters {}", shown.join(", ")))
            },
        },
        Claim {
            id: "agreement.dnaw.lattice",
            item: "dnaw-lattice",
            source: Source::Book,
            citation: DNAW,
            text: "Fig. 5: on a 29 × 29 lattice at d 0.3 'a large majority of agents … reached consensus … apart from isolated agents' (run to stability)",
            check: |_| {
                let r = runs(20, CAP, |c| {
                    dnaw(0.3, 841)(c);
                    c.mu = 0.3;
                    c.network = Network::Lattice;
                });
                let k = count(&r, |r| r.largest >= 0.8 && r.isolated >= 5.0);
                let mut o = outcome(k >= 16, format!("{k}/20 runs with a cluster of 80 % or more and 5 or more isolated agents"));
                o.detail = format!("median isolated {:.0}; mean largest {:.2}", {
                    let mut v = col(&r, |r| r.isolated);
                    v.sort_by(f64::total_cmp);
                    v[10]
                }, mean(&r, |r| r.largest));
                o
            },
        },
        Claim {
            id: "agreement.dnaw.lattice-119",
            item: "dnaw-lattice",
            source: Source::Book,
            citation: DNAW,
            text: "Fig. 5's caption: that picture 'after 100 000 iterations' (119 periods of 841 meetings)",
            check: |_| {
                let r = runs(20, 119, |c| {
                    dnaw(0.3, 841)(c);
                    c.mu = 0.3;
                    c.network = Network::Lattice;
                    c.stop_when_stable = false;
                    c.stop_at = 119;
                });
                let k = count(&r, |r| r.largest >= 0.8);
                outcome(k >= 16, format!("{k}/20 runs with a cluster of 80 % or more at period 119; mean largest {:.2}", mean(&r, |r| r.largest)))
                    .with("Run to stability the picture appears (agreement.dnaw.lattice), at a median period of about 1 800.")
            },
        },
        Claim {
            id: "agreement.dawf.fig4-ra",
            item: "ra-clusters",
            source: Source::Book,
            citation: DAWF,
            text: "Fig. 4: with identical uncertainties, relative agreement's cluster number 'is close to w/2u' (within a fifth; 50 runs, μ 0.5, N 1000 — the figure does not say; N 200 shown too)",
            check: |_| {
                let clusters = |agents: u32, w2u: f64| mean(&runs(50, CAP, move |c| {
                    c.agents = agents;
                    c.extremists = 0.0;
                    c.mu = 0.5;
                    c.uncertainty = 1.0 / w2u;
                }), |r| r.major);
                let points = [2.0, 2.5, 10.0 / 3.0, 5.0, 10.0];
                let parts = points
                    .into_iter()
                    .map(|w2u: f64| {
                        let m = clusters(1000, w2u);
                        (format!("w/2u {w2u:.2}"), outcome((m - w2u).abs() <= 0.2 * w2u, format!("{m:.2} clusters")))
                    })
                    .collect();
                let small: Vec<String> = points.iter().map(|&w| format!("{:.2}", clusters(200, w))).collect();
                all_of(parts).with(&format!("At N 200: {} (w/2u 2, 2.5, 3.33, 5, 10).", small.join(", ")))
            },
        },
        Claim {
            id: "agreement.dawf.fig4-bc",
            item: "ra-clusters",
            source: Source::Book,
            citation: DAWF,
            text: "§2.7: in the bounded-confidence model the cluster number 'is roughly the integer part of w/2u' (50 runs, N 1000, μ 0.5, clusters of 1 % or more)",
            check: |_| {
                let parts = [2.5, 10.0 / 3.0, 4.0, 5.0, 10.0]
                    .into_iter()
                    .map(|w2u: f64| {
                        let m = mean(&runs(50, CAP, dnaw(1.0 / (2.0 * w2u), 1000)), |r| r.major);
                        let want = w2u.floor();
                        (format!("w/2u {w2u:.2}"), outcome((m - want).abs() <= 0.5, format!("{m:.2} clusters (integer part {want})")))
                    })
                    .collect();
                all_of(parts)
            },
        },
        Claim {
            id: "agreement.dawf.fig5-central",
            item: "ra-central",
            source: Source::Book,
            citation: DAWF,
            text: "Fig. 5 (pe 0.2, U 0.4, μ 0.5, N 200): central convergence, 'only a marginal part of the initially non-extremists became extremist (4%)' (at most 10 % of moderates, over 40 runs)",
            check: |_| {
                let r = runs(40, CAP, |c| {
                    c.mu = 0.5;
                    c.extremists = 0.2;
                    c.uncertainty = 0.4;
                });
                let joined = mean(&r, |r| r.p_plus + r.p_minus);
                let bounds = mean(
                    &runs(40, CAP, |c| {
                        c.mu = 0.5;
                        c.extremists = 0.2;
                        c.uncertainty = 0.4;
                        c.placement = Placement::Bounds;
                    }),
                    |r| r.p_plus + r.p_minus,
                );
                outcome(joined <= 0.1, format!("{:.0} % of moderates became extremists ({:.0} % with extremists at ±1)", 100.0 * joined, 100.0 * bounds))
            },
        },
        Claim {
            id: "agreement.dawf.fig6-both",
            item: "ra-both",
            source: Source::Book,
            citation: DAWF,
            text: "Fig. 6 (pe 0.25, U 1.2, μ 0.5, N 200): the moderates 'split and become extremists' (both extremes in most runs)",
            check: |_| {
                let r = runs(40, CAP, |c| {
                    c.mu = 0.5;
                    c.extremists = 0.25;
                    c.uncertainty = 1.2;
                });
                share(&r, Run::both, 0.8, 1.0, "in both extremes")
            },
        },
        Claim {
            id: "agreement.dawf.fig7-single",
            item: "ra-single",
            source: Source::Book,
            citation: DAWF,
            text: "Figs. 7–8 (pe 0.1, U 1.4, μ 0.5, N 200): a single extreme (98 % of moderates) or, 'for another sample … all other parameters being equals', central convergence",
            check: |_| {
                let r = runs(40, CAP, |c| {
                    c.mu = 0.5;
                    c.extremists = 0.1;
                    c.uncertainty = 1.4;
                });
                let at = runs(40, CAP, ra(200, 0.1, 1.4));
                let k = count(&r, |r| r.single() || r.central());
                outcome(2 * k >= r.len(), format!("single or central in {k}/40 runs: {}", outcomes(&r)))
                    .with(&format!("At Fig. 9's μ 0.2: {}.", outcomes(&at)))
            },
        },
        Claim {
            id: "agreement.dawf.fig9-layout",
            item: "ra-map",
            source: Source::Book,
            citation: DAWF,
            text: "Fig. 9 (δ 0, μ 0.2, N 1000, 50 runs a point): central on the left, both extremes in the middle above, a central diagonal, a single extreme at the bottom right",
            check: |_| {
                let y = |pe: f64, u: f64| mean(&runs(50, CAP, ra(1000, pe, u)), |r| r.y);
                let parts = vec![
                    ("left, U 0.2, pe 0.2".to_string(), { let v = y(0.2, 0.2); outcome(v < 0.15, format!("y {v:.2}")) }),
                    ("middle, U 1.0, pe 0.2".to_string(), { let v = y(0.2, 1.0); outcome((0.4..=0.6).contains(&v), format!("y {v:.2}")) }),
                    ("diagonal, U 1.0, pe 0.05".to_string(), { let v = y(0.05, 1.0); outcome(v < 0.3, format!("y {v:.2}")) }),
                    ("bottom right, U 1.6, pe 0.025".to_string(), { let v = y(0.025, 1.6); outcome(v > 0.9, format!("y {v:.2}")) }),
                ];
                all_of(parts)
            },
        },
        Claim {
            id: "agreement.dawf.fig9-single-zone",
            item: "ra-map",
            source: Source::Book,
            citation: DAWF,
            text: "Fig. 9 (δ 0): the single-extreme zone (mean y ≥ 0.75, brown and red) covers pe up to 0.075 at U ≥ 1.4 (N 1000, 50 runs a cell; at least 80 % of those cells)",
            check: |_| {
                let keep = |u: f64, pe: f64| u >= 1.35 && pe <= 0.076;
                let cells = fig9_cells(1000, 0.0, keep);
                let k = cells.iter().filter(|c| c.2 >= 0.75).count();
                let small = fig9_cells(200, 0.0, keep);
                let k200 = small.iter().filter(|c| c.2 >= 0.75).count();
                let rows: Vec<String> = fig9_pe()
                    .into_iter()
                    .filter(|&pe| pe <= 0.076)
                    .map(|pe| {
                        let row: Vec<&(f64, f64, f64)> = cells.iter().filter(|c| c.1 == pe).collect();
                        format!("pe {pe:.4}: {:.2}", row.iter().map(|c| c.2).sum::<f64>() / row.len() as f64)
                    })
                    .collect();
                outcome(k * 5 >= cells.len() * 4, format!("{k}/{} cells at N 1000 ({k200}/{} at N 200); mean y by pe: {}", cells.len(), small.len(), rows.join(", ")))
            },
        },
        Claim {
            id: "agreement.dawf.fig9-delta",
            item: "ra-map",
            source: Source::Book,
            citation: DAWF,
            text: "Fig. 9 (δ 0.1): the single-extreme zone grows — mean y ≥ 0.75 at U ≥ 1.6 for pe up to 0.15 (N 1000, 50 runs a cell; at least 80 % of those cells)",
            check: |_| {
                let cells = fig9_cells(1000, 0.1, |u, pe| u >= 1.55 && pe <= 0.151);
                let k = cells.iter().filter(|c| c.2 >= 0.75).count();
                outcome(k * 5 >= cells.len() * 4, format!("{k}/{} cells", cells.len()))
            },
        },
        Claim {
            id: "agreement.dawf.fig10",
            item: "ra-map",
            source: Source::Book,
            citation: DAWF,
            text: "Fig. 10 (pe 0.125, δ 0): 'In the medium u (0.5 < U < 1) we only get both extremes'; above U 1 'either central or single extreme clustering (y close to 0 or close to 1)' (N 1000, 50 runs)",
            check: |_| {
                let mid = runs(50, CAP, ra(1000, 0.125, 0.8));
                let hi: Vec<Run> = [1.4, 1.6, 1.8].into_iter().flat_map(|u| runs(50, CAP, ra(1000, 0.125, u)).to_vec()).collect();
                let bimodal = count(&hi, |r| r.y < 0.1 || r.y > 0.9);
                let (c, s) = (count(&hi, |r| r.y < 0.1), count(&hi, |r| r.y > 0.9));
                all_of(vec![
                    ("U 0.8".into(), share(&mid, Run::both, 0.9, 1.0, "in both extremes")),
                    ("U 1.4–1.8".into(), outcome(bimodal * 10 >= hi.len() * 9 && c > 0 && s > 0, format!("{bimodal}/{} runs near 0 or 1 ({c} central, {s} single)", hi.len()))),
                ])
            },
        },
        Claim {
            id: "agreement.dawf.mu",
            item: "ra-map",
            source: Source::Book,
            citation: DAWF,
            text: "§4.8: 'When the intensity of interactions (μ) increases, the both extremes convergence zone increases and the single extreme convergence zone decreases' (pe 0.1, U 1.4, N 200, 50 runs)",
            check: |_| {
                let at = |mu: f64| runs(50, CAP, move |c| {
                    ra(200, 0.1, 1.4)(c);
                    c.mu = mu;
                });
                let (lo, hi) = (at(0.1), at(0.5));
                all_of(vec![
                    ("both".into(), greater(&col(&hi, |r| f64::from(u8::from(r.both()))), &col(&lo, |r| f64::from(u8::from(r.both()))), "μ 0.5", "μ 0.1")),
                    ("single".into(), greater(&col(&lo, |r| f64::from(u8::from(r.single()))), &col(&hi, |r| f64::from(u8::from(r.single()))), "μ 0.1", "μ 0.5")),
                ])
            },
        },
        Claim {
            id: "agreement.dawf.delta",
            item: "ra-delta",
            source: Source::Book,
            citation: DAWF,
            text: "§4.8: 'When the initial bias between the extremists (δ) increases the single extreme convergence zone increases' (pe 0.1, U 1.4, N 1000, 50 runs)",
            check: |_| {
                let at = |d: f64| runs(50, CAP, move |c| {
                    ra(1000, 0.1, 1.4)(c);
                    c.delta = d;
                });
                greater(&col(&at(0.2), |r| r.y), &col(&at(0.0), |r| r.y), "δ 0.2", "δ 0")
            },
        },
        Claim {
            id: "agreement.dawf.ue",
            item: "ra-map",
            source: Source::Book,
            citation: DAWF,
            text: "§4.8: 'We did not find any significant influence of the uncertainty of the extremists (ue)' (how far the population drifts, |mean opinion|, at pe 0.05, U 1.4, N 1000: ue 0.05 against 0.2, 50 runs)",
            check: |_| {
                let at = |ue: f64, margin: f64| runs(50, CAP, move |c| {
                    ra(1000, 0.05, 1.4)(c);
                    c.extremist_uncertainty = ue;
                    c.extreme_margin = margin;
                });
                let (lo, hi) = (at(0.05, 0.1), at(0.2, 0.1));
                equivalent(&col(&lo, |r| r.drift), &col(&hi, |r| r.drift), Some(0.15), "ue 0.05", "ue 0.2").with(&format!(
                    "y (new extremists past the innermost extremist less 0.1): {:.2} at ue 0.05, {:.2} at ue 0.2; counted past 0.3 inside it, {:.2} and {:.2} — with ue 0.2 the extreme cluster settles inside the reply's cutoff.",
                    mean(&lo, |r| r.y),
                    mean(&hi, |r| r.y),
                    mean(&at(0.05, 0.3), |r| r.y),
                    mean(&at(0.2, 0.3), |r| r.y)
                ))
            },
        },
        Claim {
            id: "agreement.dawf.fig20-printed",
            item: "ra-rules",
            source: Source::Book,
            citation: DAWF,
            text: "§6, Fig. 20, with eq. 11 as printed (|x − x′| < u′, the influencer's uncertainty): single extreme 'around … U = 1' (in at least half of 50 runs at U 1.0; U 0.5 reported; δ 0.1, N 1000)",
            check: |_| {
                let one = runs(50, CAP, bc(Rule::Bc, Window::Influencer, 1.0, 0.1));
                let half = runs(50, CAP, bc(Rule::Bc, Window::Influencer, 0.5, 0.1));
                outcome(count(&one, Run::single) >= 25, format!("U 1.0: {}; U 0.5: {}", outcomes(&one), outcomes(&half)))
                    .with("The confident extremists move toward every uncertain moderate who meets them.")
            },
        },
        Claim {
            id: "agreement.dawf.fig20-listener",
            item: "ra-rules",
            source: Source::Book,
            citation: DAWF,
            text: "§6, Fig. 20, with the listener's own uncertainty as the window: single extreme around U = 1 and none above U 1.2 (at least 40 of 50 runs single at U 1.0, none at U 1.6; δ 0.1, N 1000; at most 2 000 periods)",
            check: |_| {
                let one = runs(50, 2_000, bc(Rule::Bc, Window::Listener, 1.0, 0.1));
                let high = runs(50, 2_000, bc(Rule::Bc, Window::Listener, 1.6, 0.1));
                outcome(
                    count(&one, Run::single) >= 40 && count(&high, Run::single) == 0,
                    format!("U 1.0: {}; U 1.6: {}", outcomes(&one), outcomes(&high)),
                )
            },
        },
        Claim {
            id: "agreement.dawf.fig21",
            item: "ra-rules",
            source: Source::Book,
            citation: DAWF,
            text: "§6, Fig. 21 (averaging uncertainties, the listener's window): 'a band of central convergence for U close to 0.8'; above U 1.1 single or central, single when δ > 0 (N 1000, 50 runs)",
            check: |_| {
                let band = runs(50, CAP, bc(Rule::BcAveraging, Window::Listener, 0.8, 0.1));
                let high = runs(50, CAP, bc(Rule::BcAveraging, Window::Listener, 1.6, 0.1));
                all_of(vec![
                    ("U 0.8".into(), share(&band, Run::central, 0.8, 1.0, "central")),
                    ("U 1.6".into(), share(&high, Run::single, 0.5, 1.0, "in a single extreme")),
                ])
            },
        },
        Claim {
            id: "agreement.dawf.fig22",
            item: "ra-rules",
            source: Source::Book,
            citation: DAWF,
            text: "§6, Fig. 22 (uncertainty from variance): 'the absence of single extreme convergence, and the very rare presence of double extreme convergence' — mean y below 0.15 (the figure's white) at every point (U 0.4–2, pe 0.05 and 0.2, δ 0.1, N 1000, 20 runs a point, α 0.8, either window)",
            check: |_| {
                let mut single = 0;
                let mut both = 0;
                let mut n = 0;
                let mut top: f64 = 0.0;
                for window in [Window::Listener, Window::Influencer] {
                    for pe in [0.05, 0.2] {
                        for u in [0.4, 0.8, 1.2, 1.6, 2.0] {
                            let r = runs(20, CAP, move |c| {
                                bc(Rule::BcVariance, window, u, 0.1)(c);
                                c.extremists = pe;
                            });
                            single += count(&r, Run::single);
                            both += count(&r, Run::both);
                            n += r.len();
                            top = top.max(mean(&r, |r| r.y));
                        }
                    }
                }
                outcome(top < 0.15, format!("largest mean y {top:.2}; single extreme in {single}, both extremes in {both} of {n} runs"))
            },
        },
        Claim {
            id: "agreement.mc.no-single",
            item: "ra-meadows-cliff",
            source: Source::Book,
            citation: MC,
            text: "Meadows & Cliff: 'no conditions under which single extreme convergence will occur in the majority of the simulations' — under their reading (band, 200 periods, cutoff 0.8) at Fig. 9's corner (pe 0.05, U 1.4, N 200, 50 runs)",
            check: |_| {
                let r = runs(50, CAP, reading(0.0, 200));
                share(&r, Run::single, 0.0, 0.49, "in a single extreme")
                    .with(&format!("Mean |opinion| {:.2}: the majority has drifted but not past 0.8.", mean(&r, |r| r.drift)))
            },
        },
        Claim {
            id: "agreement.daw.single",
            item: "ra-deffuant-2013",
            source: Source::Book,
            citation: DAW,
            text: "The reply: with 1 200 periods (240 000 meetings) and new extremists past ±0.7, 'the single extreme convergence is very frequent (often more than 80% of the simulations) for low values of pe and large values of U' (pe 0.05, U 1.4, N 200, 50 runs)",
            check: |_| share(&runs(50, CAP, reading(0.1, 1200)), Run::single, 0.8, 1.0, "in a single extreme"),
        },
        Claim {
            id: "agreement.daw.both-fixes",
            item: "ra-readings",
            source: Source::Book,
            citation: DAW,
            text: "The reply names two fixes (the horizon and the cutoff); neither alone reproduces the single extreme (each gives mean y below 0.6 where both give above 0.9; pe 0.05, U 1.4, N 200, 50 runs)",
            check: |_| {
                let y = |m: f64, stop: u32| mean(&runs(50, CAP, reading(m, stop)), |r| r.y);
                let (mc, horizon, cutoff, daw) = (y(0.0, 200), y(0.0, 1200), y(0.1, 200), y(0.1, 1200));
                outcome(
                    horizon < 0.6 && cutoff < 0.6 && daw > 0.9,
                    format!("mean y: Meadows & Cliff {mc:.2}, horizon fixed {horizon:.2}, cutoff fixed {cutoff:.2}, both {daw:.2}"),
                )
            },
        },
        Claim {
            id: "agreement.daw.stability",
            item: "ra-literal",
            source: Source::Book,
            citation: DAW,
            text: "The reply measures y 'when all the agent opinions are completely stabilized' by a fixed 1 200 periods; the literal rule (run until nothing moves by 10⁻⁶ in a period) reads the same y (band, cutoff 0.7, pe 0.05, U 1.4, N 200, 50 runs)",
            check: |_| {
                let fixed = runs(50, CAP, reading(0.1, 1200));
                let stable = runs(50, CAP, reading(0.1, 0));
                let unstable = count(&stable, |r| !r.stable);
                equivalent(&col(&stable, |r| r.y), &col(&fixed, |r| r.y), Some(0.1), "to stability", "1 200 periods")
                    .with(&format!("{unstable} runs reached the cap; median stable period {:.0}.", {
                        let mut v = col(&stable, |r| r.tick);
                        v.sort_by(f64::total_cmp);
                        v[25]
                    }))
            },
        },
        Claim {
            id: "agreement.mc.population",
            item: "ra-population",
            source: Source::Book,
            citation: MC,
            text: "Meadows & Cliff §5.3: as N grows the single-extreme zone shrinks (y at pe 0.1, U 1.6, δ 0: N 200 against N 2000, 50 runs)",
            check: |_| {
                let at = |n: u32| runs(50, CAP, ra(n, 0.1, 1.6));
                greater(&col(&at(200), |r| r.y), &col(&at(2000), |r| r.y), "N 200", "N 2000")
            },
        },
        Claim {
            id: "agreement.daw.large-n",
            item: "ra-population",
            source: Source::Book,
            citation: DAW,
            text: "The reply: single extreme convergence 'takes place with any large number of agents' (single extreme in at least a third of runs at N 2000; pe 0.05 and 0.1, U 1.6, δ 0, 50 runs)",
            check: |_| {
                let parts = [0.05, 0.1]
                    .into_iter()
                    .map(|pe| (format!("pe {pe}"), share(&runs(50, CAP, ra(2000, pe, 1.6)), Run::single, 0.33, 1.0, "in a single extreme")))
                    .collect();
                all_of(parts).with("With δ 0.1 the single extreme grows with N (the ra-population sweep): a lean decides it, balanced extremists only chance.")
            },
        },
        Claim {
            id: "agreement.ad.moore",
            item: "ad-moore",
            source: Source::Book,
            citation: AD,
            text: "Fig. 3: on a Moore torus 'y is always below 0.6 which shows that the single extreme convergence never occurs' (30 × 30, μ 0.2, extremists at ±1, U 0.4–1.8, pe 0.05–0.3, 10 runs a point, at most 20 000 periods)",
            check: |_| {
                let mut all = Vec::new();
                for pe in [0.05, 0.1, 0.2, 0.3] {
                    for u in [0.4, 0.8, 1.2, 1.8] {
                        all.extend(runs(10, CAP, move |c| {
                            c.network = Network::Lattice;
                            c.lattice.width = 30;
                            c.lattice.height = 30;
                            c.lattice.neighborhood = sugarscape_core::opinions::Neighborhood::Moore;
                            c.extremists = pe;
                            c.uncertainty = u;
                            c.placement = Placement::Bounds;
                        }).iter().copied());
                    }
                }
                let top = all.iter().map(|r| r.y).fold(0.0, f64::max);
                outcome(top < 0.6 && count(&all, Run::single) == 0, format!("largest y {top:.2}; {}", outcomes(&all)))
            },
        },
        Claim {
            id: "agreement.ad.critical-k",
            item: "ad-connectivity",
            source: Source::Book,
            citation: AD,
            text: "Figs. 4–5: single extreme needs a critical connectivity, which 'takes place for higher connectivity when p decreases' (ring, N 1000, 20 runs a point, at most 5 000 periods)",
            check: |_| {
                let (low, high) = (critical_k(0.2, 0.1), critical_k(1.0, 0.1));
                let shown = |k: Option<u32>| k.map_or("none up to 256".to_string(), |k| k.to_string());
                outcome(
                    matches!((low, high), (Some(a), Some(b)) if a > b),
                    format!("most runs single from k {} at p 0.2, from k {} at p 1", shown(low), shown(high)),
                )
            },
        },
        Claim {
            id: "agreement.ad.around-8",
            item: "ad-connectivity",
            source: Source::Book,
            citation: AD,
            text: "Fig. 5 (β 0.8): 'the phase transition … occurs for values of connectivity around 8' (most runs single from k 4, 8 or 16)",
            check: |_| {
                let k = critical_k(0.8, 0.1);
                outcome(matches!(k, Some(4..=16)), format!("most runs single from k {}", k.map_or("none".into(), |k| k.to_string())))
            },
        },
        Claim {
            id: "agreement.ad.low-k-both",
            item: "ad-connectivity",
            source: Source::Book,
            citation: AD,
            text: "Fig. 4: at low connectivity 'double extreme convergence' (k 2 and 4, p 0.8; new extremists counted past 0.9, the reply's rule for extremists at ±1)",
            check: |_| {
                let low: Vec<Run> = [2, 4].into_iter().flat_map(|k| runs(20, 5_000, small_world(k, 0.8, 0.1)).to_vec()).collect();
                let wide: Vec<Run> = [2, 4].into_iter().flat_map(|k| runs(20, 5_000, small_world(k, 0.8, 0.3)).to_vec()).collect();
                share(&low, Run::both, 0.5, 1.0, "in both extremes")
                    .with(&format!("Counted past 0.7 instead: {}.", outcomes(&wide)))
            },
        },
        Claim {
            id: "agreement.ad.grid",
            item: "ad-connectivity",
            source: Source::Book,
            citation: AD,
            text: "Fig. 6: on a grid substrate 'the same phenomenon' — more single extremes as connectivity rises (32 × 32, p 0.8, k 8 against 120, 20 runs)",
            check: |_| {
                let at = |k: u32| runs(20, 5_000, move |c| {
                    small_world(k, 0.8, 0.1)(c);
                    c.lattice.width = 32;
                    c.lattice.height = 32;
                    c.small_world.substrate = Substrate::Grid;
                });
                greater(&col(&at(120), |r| r.y), &col(&at(8), |r| r.y), "k 120", "k 8")
            },
        },
        Claim {
            id: "agreement.w.steps",
            item: "w-dispersion",
            source: Source::Book,
            citation: W,
            text: "Fig. 2: well mixed, 'two distinct steps at y = 0.5 and y = 0.33' (dispersion 0.45–0.55 at d 0.2 and 0.25, 0.28–0.4 at d 0.15; N 900, 50 runs)",
            check: |_| {
                let d = |x: f64| mean(&runs(50, CAP, weisbuch("all", x)), |r| r.dispersion);
                let (a, b, c) = (d(0.15), d(0.2), d(0.25));
                outcome(
                    (0.28..=0.4).contains(&a) && (0.45..=0.55).contains(&b) && (0.45..=0.55).contains(&c),
                    format!("dispersion {a:.2}, {b:.2}, {c:.2} at d 0.15, 0.2, 0.25"),
                )
            },
        },
        Claim {
            id: "agreement.w.scale-free",
            item: "w-dispersion",
            source: Source::Book,
            citation: W,
            text: "Figs. 2–3: on scale-free networks 'a continuous increase … with only a kink in the d = 0.25, y = 0.7 region', similar to the square lattice (a rise through 0.5–0.8 at d 0.25, and a mean difference from the lattice of at most 0.1; N 900, 50 runs)",
            check: |_| {
                let ds = [0.15, 0.2, 0.25, 0.3];
                let curve = |net: &'static str| -> Vec<f64> { ds.iter().map(|&d| mean(&runs(50, CAP, weisbuch(net, d)), |r| r.dispersion)).collect() };
                let (sf, lat) = (curve("sf4"), curve("lattice"));
                let rising = sf.windows(2).all(|w| w[1] > w[0]);
                let near = sf.iter().zip(&lat).map(|(a, b)| (a - b).abs()).sum::<f64>() / ds.len() as f64 <= 0.1;
                let show = |v: &[f64]| v.iter().map(|x| format!("{x:.2}")).collect::<Vec<_>>().join(", ");
                outcome(rising && near && (0.5..=0.8).contains(&sf[2]), format!("scale-free {}; lattice {} (d 0.15–0.3)", show(&sf), show(&lat)))
            },
        },
        Claim {
            id: "agreement.w.connectivity",
            item: "w-dispersion",
            source: Source::Book,
            citation: W,
            text: "Fig. 3: 'Increasing the average connectivity by a factor 2 brings the scale free network results closer to those of the well-mixed case' (d 0.15–0.3, N 900, 50 runs)",
            check: |_| {
                let gap = |net: &'static str| -> f64 {
                    [0.15, 0.2, 0.25, 0.3]
                        .iter()
                        .map(|&d| (mean(&runs(50, CAP, weisbuch(net, d)), |r| r.dispersion) - mean(&runs(50, CAP, weisbuch("all", d)), |r| r.dispersion)).abs())
                        .sum()
                };
                let (four, eight) = (gap("sf4"), gap("sf8"));
                outcome(eight < four, format!("summed distance from well mixed: 4 links {four:.2}, 8 links {eight:.2}"))
            },
        },
        Claim {
            id: "agreement.w.hubs",
            item: "w-scale-free",
            source: Source::Book,
            citation: W,
            text: "Fig. 4: 'Most of the well connected nodes belong to horizontal cluster[s]' — the ten best-connected agents mostly end in the largest cluster (d 0.2, N 900, 50 runs)",
            check: |_| {
                let r = runs(50, CAP, weisbuch("sf4", 0.2));
                let hubs = mean(&r, |r| r.hubs_in_largest);
                let everyone = mean(&r, |r| r.largest);
                outcome(hubs > 0.5 && hubs > everyone, format!("{:.0} % of hubs in the largest cluster, which holds {:.0} % of all agents", 100.0 * hubs, 100.0 * everyone))
            },
        },
        Claim {
            id: "agreement.w.outlying",
            item: "w-scale-free",
            source: Source::Book,
            citation: W,
            text: "Weisbuch: on scale-free networks 'Many of them are not affected by the convergence process' (outlying nodes that never move), unlike the well-mixed case (d 0.2, N 900, 50 runs)",
            check: |_| {
                let sf = mean(&runs(50, CAP, weisbuch("sf4", 0.2)), |r| r.unmoved);
                let mixed = mean(&runs(50, CAP, weisbuch("all", 0.2)), |r| r.unmoved);
                outcome(sf >= 0.05 && mixed < 0.01, format!("never moved: {:.1} % on the network, {:.1} % well mixed", 100.0 * sf, 100.0 * mixed))
            },
        },
    ]
}
