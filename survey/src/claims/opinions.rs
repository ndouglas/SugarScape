//! Hegselmann & Krause's bounded confidence (milestone 17): the paper's
//! Section 4 figures, its two unfigured claims (serial updating, lattice
//! neighborhoods) and Lorenz's dependence on the number of agents. Runs go
//! to stability; runs that several claims share are memoized per process,
//! keyed by the config and the seeds.

use std::sync::{Arc, Mutex};

use sugarscape_core::model::{ModelConfig, ModelWorld};
use sugarscape_core::opinions::{Confidence, Interaction, Neighborhood, OpinionsConfig, Updating};

use crate::claim::{greater, range, Claim, Outcome, Source, Verdict};
use crate::runner::{model_after, model_preset};

const HK: &str = "Hegselmann & Krause 2002, JASSS 5(3)";
const LORENZ: &str = "Lorenz 2006, JASSS 9(1)";
/// Far past any all-to-all stability time (hundreds of periods at most).
const CAP: u32 = 20_000;

/// One run at its end.
#[derive(Clone, Copy, Debug)]
struct Run {
    clusters: f64,
    largest: f64,
    second: f64,
    mean: f64,
    range: f64,
    stable_at: f64,
    /// The first period with a two-sided split (NaN if none).
    split_at: f64,
    /// Whether a one-sided split ever closed (their count ever fell).
    one_sided_closed: bool,
}

impl Run {
    fn consensus(&self) -> bool {
        self.largest >= 0.99
    }

    /// Two or more major camps: the second holds at least a fifth.
    fn polarized(&self) -> bool {
        self.second >= 0.2
    }
}

fn summarize(w: &ModelWorld) -> Run {
    let s = |n: &str| w.model().series(n).expect("an opinions series");
    let last = |n: &str| *s(n).last().unwrap();
    let splits = s("splits");
    let one = s("one_sided_splits");
    Run {
        clusters: last("clusters"),
        largest: last("largest"),
        second: last("second"),
        mean: last("mean_opinion"),
        range: last("range"),
        stable_at: last("stable_at"),
        split_at: splits
            .iter()
            .position(|&k| k > 0.0)
            .map_or(f64::NAN, |t| t as f64),
        one_sided_closed: one.windows(2).any(|w| w[1] < w[0]),
    }
}

/// HK's defaults (625 random opinions, ε 0.15) with `edit`, run to stability.
fn runs(seeds: &[u64], edit: impl FnOnce(&mut OpinionsConfig)) -> Arc<Vec<Run>> {
    type Cache = Mutex<Vec<(String, Vec<u64>, Arc<Vec<Run>>)>>;
    static CACHE: Cache = Mutex::new(Vec::new());
    let mut c = OpinionsConfig::default();
    edit(&mut c);
    let key = serde_json::to_string(&c).expect("configs serialize");
    if let Some((_, _, v)) = CACHE
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .iter()
        .find(|(k, s, _)| *k == key && s == seeds)
    {
        return v.clone();
    }
    let v = Arc::new(model_after(
        &ModelConfig::Opinions(c),
        seeds,
        CAP,
        summarize,
    ));
    CACHE
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .push((key, seeds.to_vec(), v.clone()));
    v
}

/// A preset's single run (the evenly spaced ones have no randomness).
fn preset_run(id: &str) -> Run {
    model_after(&model_preset(id), &[1], CAP, summarize)[0]
}

/// HK's Fig. 3 and 11 used 50 runs per point.
fn fifty() -> Vec<u64> {
    (1..=50).collect()
}

fn eps(e: f64) -> impl FnOnce(&mut OpinionsConfig) {
    move |c| c.epsilon = e
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

pub fn claims() -> Vec<Claim> {
    vec![
        Claim {
            id: "opinions.fig-2a.survivors",
            item: "hk-plurality",
            source: Source::Book,
            citation: HK,
            text: "Fig. 2a: with ε = 0.01 'exactly 38 different opinions survive' (one run; 35–41 in 80 % of runs)",
            check: |s| range(&col(&runs(s, eps(0.01)), |r| r.clusters), 35.0, 41.0, false),
        },
        Claim {
            id: "opinions.fig-2b.two-camps",
            item: "hk-polarisation",
            source: Source::Book,
            citation: HK,
            text: "Fig. 2b: with ε = 0.15 'the agents end up in two camps' (exactly two surviving opinions in most runs)",
            check: |s| share(&runs(s, eps(0.15)), |r| r.clusters == 2.0, 0.5, 1.0, "in exactly two camps"),
        },
        Claim {
            id: "opinions.fig-2c.consensus",
            item: "hk-consensus",
            source: Source::Book,
            citation: HK,
            text: "Fig. 2c: with ε = 0.25 'the result of the dynamics is consensus'",
            check: |s| share(&runs(s, eps(0.25)), Run::consensus, 0.9, 1.0, "in consensus"),
        },
        Claim {
            id: "opinions.fig-2.fast",
            item: "hk-consensus",
            source: Source::Book,
            citation: HK,
            text: "Fig. 2: 'it takes less than 15 periods to get a stable pattern' (at ε = 0.01, 0.15 and 0.25)",
            check: |s| {
                let mut all = Vec::new();
                for e in [0.01, 0.15, 0.25] {
                    all.extend(runs(s, eps(e)).iter().copied());
                }
                let slowest = all.iter().map(|r| r.stable_at).fold(0.0, f64::max);
                let mut o = share(&all, |r| r.stable_at < 15.0, 0.9, 1.0, "stable before period 15");
                o.detail = format!("slowest: stable at period {slowest}");
                o
            },
        },
        Claim {
            id: "opinions.fig-3.phases",
            item: "hk-diagonal",
            source: Source::Book,
            citation: HK,
            text: "Fig. 3: 'we step from fragmentation (plurality) over polarisation (polarity) to consensus' (50 runs: ≥ 5 opinions at ε 0.05, two or more major camps at 0.18, consensus at 0.3)",
            check: |_| {
                let s = fifty();
                let plural = runs(&s, eps(0.05));
                let polar = runs(&s, eps(0.18));
                let one = runs(&s, eps(0.3));
                let (a, b, c) = (
                    count(&plural, |r| r.clusters >= 5.0),
                    count(&polar, Run::polarized),
                    count(&one, Run::consensus),
                );
                outcome(
                    a >= 45 && b >= 40 && c >= 45,
                    format!("plurality {a}/50 at 0.05, polarization {b}/50 at 0.18, consensus {c}/50 at 0.30"),
                )
            },
        },
        Claim {
            id: "opinions.fig-3.step-25",
            item: "hk-diagonal",
            source: Source::Book,
            citation: HK,
            text: "Fig. 3: 'at about step 25' (ε = 0.25) the camps end and consensus takes over (50 runs: consensus in at most half at 0.21, in 90 % from 0.25)",
            check: |_| {
                let s = fifty();
                let before = count(&runs(&s, eps(0.21)), Run::consensus);
                let at = count(&runs(&s, eps(0.25)), Run::consensus);
                let mid = count(&runs(&s, eps(0.22)), Run::consensus);
                outcome(
                    before <= 25 && at >= 45,
                    format!("consensus in {before}/50 at 0.21, {mid}/50 at 0.22, {at}/50 at 0.25"),
                )
            },
        },
        Claim {
            id: "opinions.fig-3.above-04",
            item: "hk-diagonal",
            source: Source::Book,
            citation: HK,
            text: "'For all values εl, εr > 0.4 the result is always conformity' (ε = 0.45, 0.6, 0.8)",
            check: |s| {
                let mut all = Vec::new();
                for e in [0.45, 0.6, 0.8] {
                    all.extend(runs(s, eps(e)).iter().copied());
                }
                share(&all, Run::consensus, 1.0, 1.0, "in consensus")
            },
        },
        Claim {
            id: "opinions.fig-4.still",
            item: "hk-regular-50",
            source: Source::Book,
            citation: HK,
            text: "Figs. 4–5 (50 evenly spaced, ε 0.2): 'the ε-profile splits in t6'; 'from period 7 to 8 onwards nothing changes anymore'",
            check: |_| {
                let r = preset_run("hk-regular-50");
                outcome(
                    r.split_at == 6.0 && r.stable_at == 8.0,
                    format!("split at period {}, stable at period {}, {} camps", r.split_at, r.stable_at, r.clusters),
                )
            },
        },
        Claim {
            id: "opinions.fig-7.splits",
            item: "hk-regular-plurality",
            source: Source::Book,
            citation: HK,
            text: "Fig. 7 (100 evenly spaced, ε 0.05): 'the profile splits 8 times'",
            check: |_| {
                let r = preset_run("hk-regular-plurality");
                outcome(
                    r.clusters == 9.0,
                    format!("{} surviving opinions ({} splits)", r.clusters, r.clusters - 1.0),
                )
            },
        },
        Claim {
            id: "opinions.fig-8.consensus",
            item: "hk-regular-consensus",
            source: Source::Book,
            citation: HK,
            text: "Fig. 8 (100 evenly spaced, ε 0.25): 'no split, total consensus'",
            check: |_| {
                let r = preset_run("hk-regular-consensus");
                outcome(
                    r.clusters == 1.0 && r.split_at.is_nan(),
                    format!("{} surviving opinion(s), stable at period {}", r.clusters, r.stable_at),
                )
            },
        },
        Claim {
            id: "opinions.fig-12c.drift",
            item: "hk-asymmetry",
            source: Source::Book,
            citation: HK,
            text: "Fig. 12c: 'with asymmetric confidence the mean opinion moves into the direction favoured by the asymmetry. This effect is extreme if there is only little confidence in the non favoured direction' (εr 0.2: εl 0.02 against 0.18)",
            check: |s| {
                let asym = |left: f64| {
                    move |c: &mut OpinionsConfig| {
                        c.confidence = Confidence::Asymmetric;
                        c.epsilon_left = left;
                        c.epsilon_right = 0.2;
                    }
                };
                greater(
                    &col(&runs(s, asym(0.02)), |r| r.mean),
                    &col(&runs(s, asym(0.18)), |r| r.mean),
                    "εl 0.02",
                    "εl 0.18",
                )
            },
        },
        Claim {
            id: "opinions.fig-13.closing",
            item: "hk-one-sided",
            source: Source::Book,
            citation: HK,
            text: "Fig. 13: 'contrary to two-sided splits one-sided splits can close again' (100 evenly spaced, εl 0.08, εr 0.24)",
            check: |_| {
                let r = preset_run("hk-one-sided");
                outcome(
                    r.one_sided_closed,
                    format!("a one-sided split closed: {}; {} camp(s) at the end", r.one_sided_closed, r.clusters),
                )
            },
        },
        Claim {
            id: "opinions.fig-17.break",
            item: "hk-bias",
            source: Source::Book,
            citation: HK,
            text: "Fig. 17c (ε 0.6): consensus survives a mild bias, and at 'about step 11, i.e. m ≈ 0.4' it breaks down (consensus in every run at m 0.36, in at most half by m 0.52)",
            check: |s| {
                let at = |m: f64| {
                    count(
                        &runs(s, move |c| {
                            c.confidence = Confidence::OpinionDependent;
                            c.epsilon = 0.6;
                            c.bias = m;
                        }),
                        Run::consensus,
                    )
                };
                let (a, b, c) = (at(0.36), at(0.44), at(0.52));
                let n = s.len();
                outcome(
                    a == n && 2 * c <= n,
                    format!("consensus in {a}/{n} at m 0.36, {b}/{n} at 0.44, {c}/{n} at 0.52"),
                )
            },
        },
        Claim {
            id: "opinions.fig-17.extremes",
            item: "hk-bias",
            source: Source::Book,
            citation: HK,
            text: "Fig. 17: 'for an m = 1 the two camps occupy the most extreme positions 0 and 1' (ε 0.2, 0.4, 0.6: final range at least 0.95)",
            check: |s| {
                let mut ranges = Vec::new();
                for e in [0.2, 0.4, 0.6] {
                    ranges.extend(col(
                        &runs(s, move |c| {
                            c.confidence = Confidence::OpinionDependent;
                            c.epsilon = e;
                            c.bias = 1.0;
                        }),
                        |r| r.range,
                    ));
                }
                range(&ranges, 0.95, 1.0, false)
            },
        },
        Claim {
            id: "opinions.fig-17.slower",
            item: "hk-bias",
            source: Source::Book,
            citation: HK,
            text: "§4.3: 'If consensus is still feasible, it takes more time to get there' as the bias grows (ε 0.6: m 0.4 against 0)",
            check: |s| {
                let at = |m: f64| {
                    col(
                        &runs(s, move |c| {
                            c.confidence = Confidence::OpinionDependent;
                            c.epsilon = 0.6;
                            c.bias = m;
                        }),
                        |r| r.stable_at,
                    )
                };
                greater(&at(0.4), &at(0.0), "m 0.4", "m 0")
            },
        },
        Claim {
            id: "opinions.serial.phases",
            item: "hk-updating",
            source: Source::Book,
            citation: HK,
            text: "§4.3: 'none of the results stated above depends crucially on simultaneous updating' (both serial readings: plurality at 0.05, polarization at 0.18, consensus at 0.3, as simultaneous)",
            check: |s| {
                let mut out = Vec::new();
                let mut ok = true;
                for (name, u) in [
                    ("shuffled", Updating::SerialShuffled),
                    ("random draws", Updating::SerialRandom),
                ] {
                    let at = |e: f64| {
                        runs(s, move |c| {
                            c.epsilon = e;
                            c.updating = u;
                        })
                    };
                    let n = s.len();
                    let (a, b, c) = (
                        count(&at(0.05), |r| r.clusters >= 5.0),
                        count(&at(0.18), Run::polarized),
                        count(&at(0.3), Run::consensus),
                    );
                    ok &= 10 * a >= 9 * n && 10 * b >= 8 * n && 10 * c >= 9 * n;
                    out.push(format!("{name}: {a}, {b}, {c} of {n}"));
                }
                outcome(ok, out.join("; "))
            },
        },
        Claim {
            id: "opinions.serial.extremes",
            item: "hk-updating",
            source: Source::Book,
            citation: HK,
            text: "§4.3: 'Random serial updating gives extreme opinions a slightly better chance to survive' (more surviving opinions at ε 0.05, shuffled order)",
            check: |s| {
                let at = |u: Updating| {
                    col(
                        &runs(s, move |c| {
                            c.epsilon = 0.05;
                            c.updating = u;
                        }),
                        |r| r.clusters,
                    )
                };
                greater(
                    &at(Updating::SerialShuffled),
                    &at(Updating::Simultaneous),
                    "serial",
                    "simultaneous",
                )
            },
        },
        Claim {
            id: "opinions.lattice.no-polarization",
            item: "hk-lattice",
            source: Source::Book,
            citation: HK,
            text: "§4.3: 'If the neighbourhoods in which the agents interact are fairly small (though overlapping!), then … polarization, disappears' (25 × 25 Moore and von Neumann tori, ε 0.1–0.3: runs with a second camp of a fifth, pooled, at most 10 %)",
            check: |s| {
                let es = [0.1, 0.15, 0.2, 0.25, 0.3];
                let (mut lattice, mut all, mut worst) = (0, 0, String::new());
                let mut worst_k = 0;
                for e in es {
                    all += count(&runs(s, eps(e)), Run::polarized);
                    for (name, nb) in [
                        ("Moore", Neighborhood::Moore),
                        ("von Neumann", Neighborhood::VonNeumann),
                    ] {
                        let k = count(
                            &runs(s, move |c| {
                                c.epsilon = e;
                                c.interaction = Interaction::Lattice;
                                c.lattice.neighborhood = nb;
                            }),
                            Run::polarized,
                        );
                        lattice += k;
                        if k > worst_k {
                            worst_k = k;
                            worst = format!("; most: {name} at ε {e}, {k}/{}", s.len());
                        }
                    }
                }
                let n = s.len();
                outcome(
                    lattice * 10 <= 2 * es.len() * n,
                    format!(
                        "polarized: lattices {lattice}/{}, everyone {all}/{}{worst}",
                        2 * es.len() * n,
                        es.len() * n
                    ),
                )
            },
        },
        Claim {
            id: "opinions.lorenz.agents",
            item: "hk-population",
            source: Source::Comment,
            citation: LORENZ,
            text: "the confidence at which consensus becomes typical depends on the number of agents (ε 0.22: consensus more often with 1000 agents than with 50)",
            check: |s| {
                let at = |n: u32| {
                    col(
                        &runs(s, move |c| {
                            c.epsilon = 0.22;
                            c.agents = n;
                        }),
                        |r| f64::from(u8::from(r.consensus())),
                    )
                };
                greater(&at(1000), &at(50), "1000 agents", "50 agents")
            },
        },
    ]
}
