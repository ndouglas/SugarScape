//! Epstein 2002's civil violence (milestone 11): the paper's figures, the
//! presets' descriptions and the Finding that its stated arrest rule gives
//! Model I no rebellion. Runs that several claims share are memoized per
//! process, keyed by the seeds.

use std::sync::{Arc, Mutex};

use sugarscape_core::model::{ModelConfig, ModelWorld};

use crate::claim::{greater, range, Claim, Source};
use crate::runner::{model_after, model_preset};

const PAPER: &str = "Epstein 2002, PNAS 99 suppl. 3";

/// A per-process cache of one computation per seed list.
struct Memo<T>(Mutex<Vec<(Vec<u64>, Arc<T>)>>);

impl<T> Memo<T> {
    const fn new() -> Self {
        Memo(Mutex::new(Vec::new()))
    }

    fn get(&self, seeds: &[u64], f: impl FnOnce() -> T) -> Arc<T> {
        let mut m = self.0.lock().unwrap_or_else(|e| e.into_inner());
        if let Some((_, v)) = m.iter().find(|(s, _)| s == seeds) {
            return v.clone();
        }
        let v = Arc::new(f());
        m.push((seeds.to_vec(), v.clone()));
        v
    }
}

fn series(w: &ModelWorld, name: &str) -> Vec<f64> {
    w.model()
        .series(name)
        .unwrap_or_else(|| panic!("no series {name}"))
}

fn last(w: &ModelWorld, name: &str) -> f64 {
    *series(w, name).last().expect("a recorded tick")
}

fn max(v: &[f64]) -> f64 {
    v.iter().copied().fold(f64::NEG_INFINITY, f64::max)
}

/// A preset with its arrest ratio rounded down or not.
fn with_floor(id: &str, floor: bool) -> ModelConfig {
    let mut c = model_preset(id);
    if let ModelConfig::Civil(c) = &mut c {
        c.quirks.floor_ratio = floor;
    }
    c
}

/// A Model II preset that runs on past extinction (so a seed is measured
/// for the whole run).
fn running_on(id: &str) -> ModelConfig {
    let mut c = model_preset(id);
    if let ModelConfig::Civil(c) = &mut c {
        c.stop_at_extinction = false;
    }
    c
}

/// Run 2 over 3000 ticks: outbursts, mean activation, mean wait, peak.
struct Run2 {
    outbursts: f64,
    activation: f64,
    wait: f64,
    peak: f64,
}

fn run_2(seeds: &[u64], floor: bool) -> Arc<Vec<Run2>> {
    static FLOOR: Memo<Vec<Run2>> = Memo::new();
    static LITERAL: Memo<Vec<Run2>> = Memo::new();
    let memo = if floor { &FLOOR } else { &LITERAL };
    memo.get(seeds, || {
        model_after(
            &with_floor("cv-run-2-punctuated", floor),
            seeds,
            3000,
            |w| Run2 {
                outbursts: last(w, "outbursts"),
                activation: last(w, "mean_activation"),
                wait: last(w, "mean_wait"),
                peak: max(&series(w, "active")),
            },
        )
    })
}

/// Runs 3 and 4 (salami tactics, one jump) over 300 ticks: the peak of
/// actives after t = 77 and the jailed at t = 300.
fn runs_3_4(seeds: &[u64]) -> Arc<[Vec<(f64, f64)>; 2]> {
    static M: Memo<[Vec<(f64, f64)>; 2]> = Memo::new();
    M.get(seeds, || {
        let one = |id| {
            model_after(&model_preset(id), seeds, 300, |w| {
                (max(&series(w, "active")[77..]), last(w, "jailed"))
            })
        };
        [one("cv-run-3-salami"), one("cv-run-4-one-jump")]
    })
}

/// Whether both groups are alive after `ticks`, per seed.
fn both_alive(id: &str, seeds: &[u64], ticks: u32) -> Vec<f64> {
    model_after(&running_on(id), seeds, ticks, |w| {
        f64::from(u8::from(last(w, "blue") > 0.0 && last(w, "green") > 0.0))
    })
}

/// The tick at which a group was gone (3000 if neither was), run 7 at a
/// cop density.
fn extinction(seeds: &[u64], cops: f64) -> Vec<f64> {
    let mut c = running_on("cv-run-7-cleansing");
    if let ModelConfig::Civil(c) = &mut c {
        c.cop_density = cops;
    }
    model_after(&c, seeds, 3000, |w| last(w, "extinction"))
}

pub fn claims() -> Vec<Claim> {
    vec![
        Claim {
            id: "cv-run-2.no-outbursts-as-stated",
            item: "cv-run-2-punctuated",
            source: Source::App,
            citation: "spec 2026-09-25-civil-violence-design.md, Finding",
            text: "with the paper's P = 1 − exp(−k·C/A), Run 2 never has an outburst: at most 49 actives in 3000 ticks",
            check: |s| {
                let v: Vec<f64> = run_2(s, false).iter().map(|r| r.peak).collect();
                range(&v, 0.0, 49.0, false)
            },
        },
        Claim {
            id: "cv-run-2.punctuated",
            item: "cv-run-2-punctuated",
            source: Source::Book,
            citation: PAPER,
            text: "punctuated equilibrium (Figs. 3–4): with C/A rounded down, at least 80 outbursts above 50 actives in 3000 ticks",
            check: |s| {
                let v: Vec<f64> = run_2(s, true).iter().map(|r| r.outbursts).collect();
                range(&v, 80.0, f64::INFINITY, false)
            },
        },
        Claim {
            id: "cv-run-2.activation",
            item: "cv-run-2-punctuated",
            source: Source::Book,
            citation: PAPER,
            text: "total activation per outburst has mean 708 and s.d. 230 (Fig. 7): the mean within 708 ± 230",
            check: |s| {
                let v: Vec<f64> = run_2(s, true).iter().map(|r| r.activation).collect();
                range(&v, 478.0, 938.0, false)
            },
        },
        Claim {
            id: "cv-run-2.wait",
            item: "cv-run-2-punctuated",
            source: Source::Book,
            citation: PAPER,
            text: "the average duration between outbursts is 60 (Fig. 5): the mean wait about 60 (50–70, widened by 10%)",
            check: |s| {
                let v: Vec<f64> = run_2(s, true).iter().map(|r| r.wait).collect();
                range(&v, 50.0, 70.0, true)
            },
        },
        Claim {
            id: "cv-run-3.no-spike",
            item: "cv-run-3-salami",
            source: Source::Book,
            citation: PAPER,
            text: "a large legitimacy reduction in small increments gives no red spike (Fig. 9): at most 50 actives after t = 77",
            check: |s| {
                let v: Vec<f64> = runs_3_4(s)[0].iter().map(|r| r.0).collect();
                range(&v, 0.0, 50.0, false)
            },
        },
        Claim {
            id: "cv-run-4.explosion",
            item: "cv-run-4-one-jump",
            source: Source::Book,
            citation: PAPER,
            text: "a smaller reduction in one jump gives an explosion of actives (Fig. 10): its peak after t = 77 exceeds salami tactics'",
            check: |s| {
                let r = runs_3_4(s);
                let peak = |i: usize| r[i].iter().map(|x| x.0).collect::<Vec<_>>();
                greater(&peak(1), &peak(0), "one jump", "salami")
            },
        },
        Claim {
            id: "cv-run-4.more-jailed",
            item: "cv-run-4-one-jump",
            source: Source::Book,
            citation: PAPER,
            text: "the jailed population's absolute size exceeds that of the previous run (Fig. 10): more jailed at t = 300 than salami tactics",
            check: |s| {
                let r = runs_3_4(s);
                let jailed = |i: usize| r[i].iter().map(|x| x.1).collect::<Vec<_>>();
                greater(&jailed(1), &jailed(0), "one jump", "salami")
            },
        },
        Claim {
            id: "cv-run-5.tips",
            item: "cv-run-5-cop-reductions",
            source: Source::Book,
            citation: PAPER,
            text: "a marginal reduction in cops tips society into rebellion (Fig. 11): more than 50 actives at some tick of 700",
            check: |s| {
                let v = model_after(&model_preset("cv-run-5-cop-reductions"), s, 700, |w| {
                    max(&series(w, "active"))
                });
                range(&v, 50.0, f64::INFINITY, false)
            },
        },
        Claim {
            id: "cv-run-6.coexistence",
            item: "cv-run-6-coexistence",
            source: Source::Book,
            citation: PAPER,
            text: "with high legitimacy and no cops, peaceful coexistence prevails (Fig. 12): both groups alive after 1000 ticks",
            check: |s| range(&both_alive("cv-run-6-coexistence", s, 1000), 1.0, 1.0, false),
        },
        Claim {
            id: "cv-run-7.genocide",
            item: "cv-run-7-cleansing",
            source: Source::Book,
            citation: PAPER,
            text: "at L = 0.8 with no cops genocide is always observed (Fig. 13): one group gone within 3000 ticks",
            check: |s| range(&both_alive("cv-run-7-cleansing", s, 3000), 0.0, 0.0, false),
        },
        Claim {
            id: "cv-run-8.stable",
            item: "cv-run-8-nasty-regime",
            source: Source::Book,
            citation: PAPER,
            text: "with cop density 0.04 from the outset a stable, but nasty, regime emerges; the cops prevent either side wiping the other out: both groups alive after 3000 ticks",
            check: |s| range(&both_alive("cv-run-8-nasty-regime", s, 3000), 1.0, 1.0, false),
        },
        Claim {
            id: "cv-safe-havens.havens",
            item: "cv-safe-havens",
            source: Source::Book,
            citation: PAPER,
            text: "peacekeepers deployed at t = 50 typically produce safe havens (Fig. 14): both groups alive after 3000 ticks",
            check: |s| range(&both_alive("cv-safe-havens", s, 3000), 1.0, 1.0, false),
        },
        Claim {
            id: "cv-peacekeeping.delay",
            item: "cv-peacekeeping",
            source: Source::Book,
            citation: PAPER,
            text: "the larger the initial force of peacekeepers, the more time one buys (Fig. 16): extinction later at density 0.1 than at 0",
            check: |s| greater(&extinction(s, 0.1), &extinction(s, 0.0), "0.1", "none"),
        },
    ]
}
