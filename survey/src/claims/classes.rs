//! Axtell, Epstein & Young's Emergence of Classes (milestone 15): the
//! paper's figures and Poza et al.'s replication (2011). Runs that several
//! claims share are memoized per process, keyed by the config and seeds.

use std::sync::{Arc, Mutex};

use sugarscape_core::classes::{
    expected_best, ClassesConfig, Decision, Interaction, Start, CLASSES, DIVIDED_BELOW, EQUITY,
    EQUITY_BETWEEN, FRACTIOUS, M,
};
use sugarscape_core::model::{ModelConfig, ModelWorld};

use crate::claim::{greater, range, Claim, Outcome, Source, Verdict};
use crate::runner::model_after;

const AEY: &str = "Axtell, Epstein & Young 2000";
const PVPLH: &str = "Poza, Villafáñez, Pajares, López-Paredes & Hernández 2011";

/// One run, summarized.
#[derive(Clone, Copy, Debug)]
struct Run {
    /// The first period in the equity regime (the run's length if never).
    first_equity: f64,
    /// `equity_at` at the end (AEY's transition target, or the run's length).
    equity_at: f64,
    first_attractor: f64,
    /// Whether the run was ever in each regime, and its last regime.
    ever: [bool; 6],
    last: f64,
    /// The mean realized error rate over periods 100 on (0 for shorter runs).
    realized_noise: f64,
    /// Mean payoff over the periods in the fractious regime (NaN if none),
    /// and how many there were.
    fractious_payoff: f64,
    fractious_periods: f64,
}

fn summarize(w: &ModelWorld) -> Run {
    let s = |n: &str| w.model().series(n).expect("a classes series");
    let regime = s("regime");
    let payoff = s("mean_payoff");
    let noise = s("realized_noise");
    let fractious: Vec<f64> = regime
        .iter()
        .zip(&payoff)
        .filter(|(&r, _)| r == f64::from(FRACTIOUS))
        .map(|(_, &p)| p)
        .collect();
    let mut ever = [false; 6];
    for &r in &regime {
        ever[r as usize] = true;
    }
    let tail = if noise.len() > 100 {
        &noise[100..]
    } else {
        &[][..]
    };
    Run {
        first_equity: regime
            .iter()
            .position(|&r| r == f64::from(EQUITY))
            .unwrap_or(regime.len()) as f64,
        equity_at: *s("equity_at").last().unwrap(),
        first_attractor: *s("first_attractor").last().unwrap(),
        ever,
        last: *regime.last().unwrap(),
        realized_noise: if tail.is_empty() {
            0.0
        } else {
            tail.iter().sum::<f64>() / tail.len() as f64
        },
        fractious_payoff: fractious.iter().sum::<f64>() / fractious.len() as f64,
        fractious_periods: fractious.len() as f64,
    }
}

/// The defaults with `edit` applied, run `ticks` periods for every seed.
fn runs(seeds: &[u64], ticks: u32, edit: impl FnOnce(&mut ClassesConfig)) -> Arc<Vec<Run>> {
    type Cache = Mutex<Vec<(String, u32, Vec<u64>, Arc<Vec<Run>>)>>;
    static CACHE: Cache = Mutex::new(Vec::new());
    let mut c = ClassesConfig::default();
    edit(&mut c);
    let key = serde_json::to_string(&c).expect("configs serialize");
    if let Some((_, _, _, v)) = CACHE
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .iter()
        .find(|(k, t, s, _)| *k == key && *t == ticks && s == seeds)
    {
        return v.clone();
    }
    let v = Arc::new(model_after(
        &ModelConfig::Classes(c),
        seeds,
        ticks,
        summarize,
    ));
    CACHE
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .push((key, ticks, seeds.to_vec(), v.clone()));
    v
}

/// A share of seeds with `pred`: holds when it lies in `lo..=hi`.
fn share(runs: &[Run], pred: impl Fn(&Run) -> bool, lo: f64, hi: f64, what: &str) -> Outcome {
    let k = runs.iter().filter(|r| pred(r)).count();
    let s = k as f64 / runs.len() as f64;
    Outcome {
        verdict: if (lo..=hi).contains(&s) {
            Verdict::Holds
        } else {
            Verdict::Fails
        },
        measured: format!("{k}/{} seeds {what} ({:.0} %)", runs.len(), 100.0 * s),
        detail: String::new(),
    }
}

fn col(runs: &[Run], f: impl Fn(&Run) -> f64) -> Vec<f64> {
    runs.iter().map(f).collect()
}

fn tags(c: &mut ClassesConfig) {
    c.tags = true;
    c.memory = 20;
}

fn small_tags(c: &mut ClassesConfig) {
    c.tags = true;
    c.agents = 20;
    c.memory = 5;
    c.noise = 0.05;
}

/// A fractious start leaving for equity (AEY Figs. 4–5): N, m, ε.
fn transition(seeds: &[u64], n: u32, m: u32, e: f64) -> Vec<f64> {
    col(
        &runs(seeds, 1_000_000, |c| {
            c.agents = n;
            c.memory = m;
            c.noise = e;
            c.start = Start::Fractious;
            c.stop_at_equity = true;
        }),
        |r| r.equity_at,
    )
}

fn segregated(r: &Run) -> bool {
    r.ever[CLASSES as usize] || r.ever[DIVIDED_BELOW as usize]
}

pub fn claims() -> Vec<Claim> {
    vec![
        Claim {
            id: "classes.noise",
            item: "aey-equity",
            source: Source::Book,
            citation: AEY,
            text: "note 9: at ε = 0.2 'the probability that an \"error\" is realized is 0.1333' (0.12–0.15, periods 100–2000)",
            check: |s| range(&col(&runs(s, 2000, |_| {}), |r| r.realized_noise), 0.12, 0.15, false),
        },
        Claim {
            id: "classes.m-never-best-without-m",
            item: "aey-equity",
            source: Source::Book,
            citation: AEY,
            text: "'M is never a best response for someone who has never experienced an opponent who played M' (every memory of up to 20 L's and H's)",
            check: |_| {
                let ok = (1..=20u32)
                    .all(|m| (0..=m).all(|l| expected_best([l, 0, m - l], 30) & 1 << M == 0));
                Outcome {
                    verdict: if ok { Verdict::Holds } else { Verdict::Fails },
                    measured: format!("checked every memory of length 1–20: {ok}"),
                    detail: String::new(),
                }
            },
        },
        Claim {
            id: "classes.fig-2",
            item: "aey-equity",
            source: Source::Book,
            citation: AEY,
            text: "Fig. 2 (N 100, m 10, ε 0.2, random start): the equity norm after about 80 periods (every agent's best reply M within 150)",
            check: |s| range(&col(&runs(s, 2000, |_| {}), |r| r.first_equity), 0.0, 150.0, false),
        },
        Claim {
            id: "classes.fig-3.reached",
            item: "aey-fractious",
            source: Source::Book,
            citation: AEY,
            text: "Fig. 3: from another random start, a fractious state (at least one seed in 20 reaches it first)",
            check: |s| share(&runs(s, 2000, |_| {}), |r| r.first_attractor == f64::from(FRACTIOUS), 0.05, 1.0, "fractious first"),
        },
        Claim {
            id: "classes.fig-3.persists",
            item: "aey-fractious",
            source: Source::Book,
            citation: AEY,
            text: "Fig. 3: the fractious state 'persists in excess of 10⁹ time periods' — from a fractious start, no equity within 2000 periods",
            check: |s| share(&runs(s, 2000, |c| c.start = Start::Fractious), |r| r.first_equity >= 2000.0, 0.8, 1.0, "still out of equity at 2000"),
        },
        Claim {
            id: "classes.fig-3.payoff",
            item: "aey-fractious",
            source: Source::Book,
            citation: AEY,
            text: "Fig. 3: in the fractious state the average share per person is about one-quarter (mean payoff over the periods a fractious start spends in the fractious regime: 15–35)",
            check: |s| {
                let r = runs(s, 2000, |c| c.start = Start::Fractious);
                let mut o = range(&col(&r, |r| r.fractious_payoff).into_iter().filter(|p| !p.is_nan()).collect::<Vec<_>>(), 15.0, 35.0, false);
                let periods = col(&r, |r| r.fractious_periods);
                let first = col(&r, |r| r.first_equity);
                o.detail = format!(
                    "fractious for {}–{} periods; first equity at periods {}–{}",
                    periods.iter().cloned().fold(f64::INFINITY, f64::min),
                    periods.iter().cloned().fold(0.0, f64::max),
                    first.iter().cloned().fold(f64::INFINITY, f64::min),
                    first.iter().cloned().fold(0.0, f64::max),
                );
                o
            },
        },
        Claim {
            id: "classes.fig-4.magnitude",
            item: "aey-transition",
            source: Source::Book,
            citation: AEY,
            text: "Fig. 4 (N 10, ε 0.1): 'when m = 13 it takes in excess of 10⁵ periods' to leave the fractious regime",
            check: |s| range(&transition(s, 10, 13, 0.1), 100_000.0, f64::INFINITY, false),
        },
        Claim {
            id: "classes.fig-4.growth",
            item: "aey-transition",
            source: Source::Book,
            citation: AEY,
            text: "Fig. 4: the waiting time grows with memory (ε 0.05: m 14 longer than m 8)",
            check: |s| greater(&transition(s, 10, 14, 0.05), &transition(s, 10, 8, 0.05), "m 14", "m 8"),
        },
        Claim {
            id: "classes.fig-5.growth",
            item: "aey-transition",
            source: Source::Book,
            citation: AEY,
            text: "Fig. 5 (m 10): the waiting time grows with the population (ε 0.1: N 60 longer than N 20)",
            check: |s| greater(&transition(s, 60, 10, 0.1), &transition(s, 20, 10, 0.1), "N 60", "N 20"),
        },
        Claim {
            id: "classes.tags.classes",
            item: "aey-tags",
            source: Source::Book,
            citation: AEY,
            text: "Fig. 8 (N 100, m 20, ε 0.2, random start): classes emerge — in at least one seed of 20 within 5000 periods",
            check: |s| share(&runs(s, 5000, tags), |r| r.ever[CLASSES as usize], 0.05, 1.0, "ever in classes"),
        },
        Claim {
            id: "classes.tags.divided-below",
            item: "aey-tags",
            source: Source::Book,
            citation: AEY,
            text: "Fig. 9: 'equity above, division below' — in at least one seed of 20",
            check: |s| share(&runs(s, 5000, tags), |r| r.ever[DIVIDED_BELOW as usize], 0.05, 1.0, "ever divided below"),
        },
        Claim {
            id: "classes.tags.equity-between",
            item: "aey-tags",
            source: Source::Book,
            citation: AEY,
            text: "Fig. 7: equity between types but not within — in at least one seed of 20",
            check: |s| share(&runs(s, 5000, tags), |r| r.ever[EQUITY_BETWEEN as usize], 0.05, 1.0, "ever equity between"),
        },
        Claim {
            id: "classes.tags.persist",
            item: "aey-classes",
            source: Source::Book,
            citation: AEY,
            text: "transitions out of classes are 'very rare': a class system is still one after 20 000 periods (at least 80 % of seeds)",
            check: |s| {
                share(
                    &runs(s, 20_000, |c| {
                        tags(c);
                        c.start = Start::Classes;
                    }),
                    |r| r.last == f64::from(CLASSES),
                    0.8,
                    1.0,
                    "in classes at 20 000",
                )
            },
        },
        Claim {
            id: "classes.pvplh.never",
            item: "aey-tags",
            source: Source::Comment,
            citation: PVPLH,
            text: "'when we tried the same parameters that AEY used … segregation never emerged' (no classes and no division below in any seed)",
            check: |s| share(&runs(s, 5000, tags), segregated, 0.0, 0.0, "segregated"),
        },
        Claim {
            id: "classes.pvplh.small",
            item: "pvplh-small-tags",
            source: Source::Comment,
            citation: PVPLH,
            text: "with 20 agents, m 5, ε 0.05 segregation appears (Figs. 9–10; AEY's rule: classes or division below in at least one seed)",
            check: |s| share(&runs(s, 5000, small_tags), segregated, 0.05, 1.0, "segregated"),
        },
        Claim {
            id: "classes.pvplh.mode-segregates",
            item: "pvplh-mode",
            source: Source::Comment,
            citation: PVPLH,
            text: "with the mode rule 'segregation emerged spontaneously much more often' (AEY's parameters: more seeds ever in classes or division below)",
            check: |s| {
                let aey = runs(s, 5000, tags).iter().filter(|r| segregated(r)).count();
                let mode = runs(s, 5000, |c| {
                    tags(c);
                    c.decision = Decision::Mode;
                })
                .iter()
                .filter(|r| segregated(r))
                .count();
                Outcome {
                    verdict: if mode > aey { Verdict::Holds } else { Verdict::Fails },
                    measured: format!("segregated seeds: AEY's rule {aey}, mode rule {mode} of {}", s.len()),
                    detail: String::new(),
                }
            },
        },
        Claim {
            id: "classes.pvplh.mode-fractious-first",
            item: "pvplh-mode",
            source: Source::Comment,
            citation: PVPLH,
            text: "with the mode rule fewer runs reach equity first (N 20, m 10, ε 0.2: more fractious-first seeds than AEY's rule)",
            check: |s| {
                let first = |d| {
                    runs(s, 5000, |c| {
                        c.agents = 20;
                        c.decision = d;
                    })
                    .iter()
                    .filter(|r| r.first_attractor == f64::from(FRACTIOUS))
                    .count()
                };
                let (aey, mode) = (first(Decision::Expected), first(Decision::Mode));
                Outcome {
                    verdict: if mode > aey { Verdict::Holds } else { Verdict::Fails },
                    measured: format!("fractious first: AEY's rule {aey}, mode rule {mode} of {}", s.len()),
                    detail: String::new(),
                }
            },
        },
        Claim {
            id: "classes.pvplh.low",
            item: "aey-transition",
            source: Source::Comment,
            citation: PVPLH,
            text: "'the higher the reward assigned to low, the longer it took … to reach the equitable equilibrium' (N 20, m 10, ε 0.1: L 40 longer than L 15)",
            check: |s| {
                let t = |low| {
                    col(
                        &runs(s, 1_000_000, |c| {
                            c.agents = 20;
                            c.noise = 0.1;
                            c.low = low;
                            c.start = Start::Fractious;
                            c.stop_at_equity = true;
                        }),
                        |r| r.equity_at,
                    )
                };
                greater(&t(40), &t(15), "L 40", "L 15")
            },
        },
        Claim {
            id: "classes.pvplh.progressive",
            item: "pvplh-progressive",
            source: Source::Comment,
            citation: PVPLH,
            text: "progressive memory lengthens the time to equity (N 20, m 12, ε 0.1)",
            check: |s| {
                let t = |start| {
                    col(
                        &runs(s, 1_000_000, |c| {
                            c.agents = 20;
                            c.memory = 12;
                            c.noise = 0.1;
                            c.start = start;
                            c.stop_at_equity = true;
                        }),
                        |r| r.equity_at,
                    )
                };
                greater(&t(Start::Progressive), &t(Start::Random), "progressive", "random")
            },
        },
        Claim {
            id: "classes.pvplh.lattice",
            item: "pvplh-lattice",
            source: Source::Comment,
            citation: PVPLH,
            text: "on a 10 × 10 Moore torus (tags at random, mode rule, m 5, ε 0.05) the well-mixed attractors recur: classes, or equity between types without it within, in at least one seed",
            check: |s| {
                share(
                    &runs(s, 5000, |c| {
                        small_tags(c);
                        c.agents = 100;
                        c.decision = Decision::Mode;
                        c.interaction = Interaction::Lattice;
                    }),
                    |r| r.ever[CLASSES as usize] || r.ever[EQUITY_BETWEEN as usize],
                    0.05,
                    1.0,
                    "in classes or equity between only",
                )
            },
        },
    ]
}
