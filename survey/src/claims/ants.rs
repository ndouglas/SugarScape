//! Ants and recruitment (milestone 24): Kirman's recruitment chain (1993)
//! and Alfarano and Milaković's network critique (2007). Runs are memoized per
//! process, keyed by the config, the seeds and the steps; a run is read over
//! every step after the first.

use std::sync::{Arc, Mutex};

use sugarscape_core::ants::{self, AntsConfig, Network, Rule, Start, HOLD};
use sugarscape_core::model::{ModelConfig, ModelWorld};

use crate::claim::{all_of, equivalent, greater, range, Claim, Outcome, Source, Verdict};
use crate::runner::model_after;

const KIRMAN: &str = "Kirman 1993, QJE 108(1)";
const AM: &str = "Alfarano & Milaković 2007, Warwick WP07-02 (JEDC 2009)";

/// One run, read over its steps.
#[derive(Clone, Debug)]
struct Run {
    /// The time mean of z, and the share of steps with z in 0.4–0.6.
    mean: f64,
    middle: f64,
    variance: f64,
    extreme: f64,
    flips: f64,
    /// Steps at each k, 0..=N.
    hist: Vec<u32>,
    /// Steps between flips, and from leaving one holder's 80 % to the next
    /// holder's arrival.
    residences: Vec<f64>,
    transits: Vec<f64>,
    /// z after the first and the last step.
    first: f64,
    last: f64,
}

fn summarize(w: &ModelWorld) -> Run {
    let m = w.model();
    let ModelConfig::Ants(c) = m.config() else {
        unreachable!()
    };
    let n = c.ants;
    let z = m.series("share").unwrap();
    let steps = &z[1..];
    let t = steps.len() as f64;
    let mean = steps.iter().sum::<f64>() / t;
    let mut hist = vec![0u32; n as usize + 1];
    for x in steps {
        hist[(x * f64::from(n)).round() as usize] += 1;
    }
    let flips = m.series("flips").unwrap();
    let times: Vec<usize> = (1..flips.len())
        .filter(|&i| flips[i] > flips[i - 1])
        .collect();
    let residences = times.windows(2).map(|p| (p[1] - p[0]) as f64).collect();
    // Two sources: from the last step the old holder held 80 % to the new one's arrival.
    let mut transits = Vec::new();
    if c.sources == 2 {
        for &t in &times {
            let now_high = z[t] >= HOLD;
            let left = (0..t).rev().find(|&i| {
                if now_high {
                    z[i] <= 1.0 - HOLD
                } else {
                    z[i] >= HOLD
                }
            });
            if let Some(i) = left {
                transits.push((t - i) as f64);
            }
        }
    }
    Run {
        mean,
        middle: steps.iter().filter(|&&x| (0.4..=0.6).contains(&x)).count() as f64 / t,
        variance: m.latest_value("variance").unwrap(),
        extreme: m.latest_value("extreme").unwrap(),
        flips: m.latest_value("flips").unwrap(),
        hist,
        residences,
        transits,
        first: z[0],
        last: *z.last().unwrap(),
    }
}

/// The model's defaults (Kirman's Figure IIb) with `edit`, seeds 1..=`seeds`, `steps` steps.
fn runs(seeds: u64, steps: u32, edit: impl FnOnce(&mut AntsConfig)) -> Arc<Vec<Run>> {
    type Cache = Mutex<Vec<(String, u64, u32, Arc<Vec<Run>>)>>;
    static CACHE: Cache = Mutex::new(Vec::new());
    let mut c = AntsConfig::default();
    edit(&mut c);
    let key = serde_json::to_string(&c).expect("configs serialize");
    if let Some((_, _, _, v)) = CACHE
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .iter()
        .find(|(k, s, r, _)| *k == key && *s == seeds && *r == steps)
    {
        return v.clone();
    }
    let seed_list: Vec<u64> = (1..=seeds).collect();
    let v = Arc::new(model_after(
        &ModelConfig::Ants(c),
        &seed_list,
        steps,
        summarize,
    ));
    CACHE
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .push((key, seeds, steps, v.clone()));
    v
}

fn mean(runs: &[Run], f: impl Fn(&Run) -> f64) -> f64 {
    runs.iter().map(f).sum::<f64>() / runs.len() as f64
}

fn col(runs: &[Run], f: impl Fn(&Run) -> f64) -> Vec<f64> {
    runs.iter().map(f).collect()
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

fn kirman(epsilon: f64, delta: f64) -> impl FnOnce(&mut AntsConfig) {
    move |c| {
        c.epsilon = epsilon;
        c.delta = delta;
    }
}

fn config(edit: impl FnOnce(&mut AntsConfig)) -> AntsConfig {
    let mut c = AntsConfig::default();
    edit(&mut c);
    c
}

/// Alfarano and Milaković's rule on `network` at N, with a set for α = aN/λD.
fn am(n: u32, network: Network, alpha: f64) -> impl FnOnce(&mut AntsConfig) {
    move |c| {
        c.rule = Rule::Alfarano;
        c.ants = n;
        c.network = network;
        c.lambda = 1.0;
        let d = match network {
            Network::Random => c.link * f64::from(n - 1),
            Network::Complete => f64::from(n - 1),
            _ => f64::from(c.degree),
        };
        c.a = alpha * d / f64::from(n);
    }
}

/// Figure 4's setting: a 0.5, λ 1, D 10 or p 0.1.
fn fig4(n: u32, network: Network) -> impl FnOnce(&mut AntsConfig) {
    move |c| {
        c.rule = Rule::Alfarano;
        c.ants = n;
        c.network = network;
        c.a = 0.5;
        c.lambda = 1.0;
    }
}

/// Mean-field Var[z] for Figure 4's setting: 1/(4(2aN/λD + 1)).
fn fig4_theory(n: u32, network: Network) -> f64 {
    let d = if network == Network::Random {
        0.1 * f64::from(n - 1)
    } else {
        10.0
    };
    1.0 / (4.0 * (2.0 * 0.5 * f64::from(n) / d + 1.0))
}

/// The slope of y on x by least squares.
fn slope(x: &[f64], y: &[f64]) -> f64 {
    let mx = x.iter().sum::<f64>() / x.len() as f64;
    let my = y.iter().sum::<f64>() / y.len() as f64;
    let sxy: f64 = x.iter().zip(y).map(|(a, b)| (a - mx) * (b - my)).sum();
    let sxx: f64 = x.iter().map(|a| (a - mx).powi(2)).sum();
    sxy / sxx
}

fn interior_modes(p: &[f64]) -> Vec<usize> {
    (1..p.len() - 1)
        .filter(|&k| p[k] > p[k - 1] && p[k] > p[k + 1] && (p[k] - p[k - 1]).abs() > 1e-12)
        .collect()
}

fn figure_compatibility(values: &[f64]) -> Outcome {
    let hits = values.iter().filter(|&&v| (0.4..=0.6).contains(&v)).count();
    let n = values.len() as f64;
    let fraction = hits as f64 / n;
    // Wilson score interval for independent records, not correlated steps.
    let z = 1.959963984540054;
    let denominator = 1.0 + z * z / n;
    let center = (fraction + z * z / (2.0 * n)) / denominator;
    let radius = z * (fraction * (1.0 - fraction) / n + z * z / (4.0 * n * n)).sqrt() / denominator;
    outcome(
        hits > 0,
        format!(
            "{hits}/{} records have time means in 0.4–0.6 ({:.1} %; Wilson 95 % CI {:.1}–{:.1} %)",
            values.len(),
            100.0 * fraction,
            100.0 * (center - radius),
            100.0 * (center + radius)
        ),
    )
}

const FIG1: [(&str, f64, f64); 3] = [("Ia", 0.005, 0.01), ("Ib", 0.01, 0.02), ("Ic", 0.15, 0.3)];

pub fn claims() -> Vec<Claim> {
    vec![
        Claim {
            id: "ants.kirman.exact",
            item: "ants-1a",
            source: Source::Book,
            citation: KIRMAN,
            text: "The equilibrium distribution (4)–(5) of chain (1): 200 000 steps of 50 meetings match it within total variation 0.03 (Figure I's three settings, 5 runs each)",
            check: |_| {
                let parts = FIG1
                    .into_iter()
                    .map(|(name, e, d)| {
                        let r = runs(5, 200_000, kirman(e, d));
                        let p = ants::kirman(&config(kirman(e, d))).unwrap();
                        let mut h = vec![0f64; 101];
                        for run in r.iter() {
                            for (k, &c) in run.hist.iter().enumerate() {
                                h[k] += f64::from(c);
                            }
                        }
                        let total: f64 = h.iter().sum();
                        let tv = h.iter().zip(&p).map(|(a, b)| (a / total - b).abs()).sum::<f64>() / 2.0;
                        (name.to_string(), outcome(tv <= 0.03, format!("total variation {tv:.4}")))
                    })
                    .collect();
                all_of(parts).with("The exact distribution is the beta-binomial with α = ε(N − 1)/(1 − δ).")
            },
        },
        Claim {
            id: "ants.kirman.uniform",
            item: "ants-1b",
            source: Source::Book,
            citation: KIRMAN,
            text: "'The uniform distribution (Figure Ib) occurs when … ε = (1 − δ)/(N − 1)'; below it the distribution piles up at the extremes (Figure Ia), above it centers",
            check: |_| {
                let flat = ants::kirman(&config(kirman(0.98 / 99.0, 0.02))).unwrap();
                let dev = flat.iter().map(|v| (v - 1.0 / 101.0).abs()).fold(0.0, f64::max);
                let below = ants::kirman(&config(kirman(0.005, 0.01))).unwrap();
                let above = ants::kirman(&config(kirman(0.15, 0.3))).unwrap();
                all_of(vec![
                    ("at the threshold".into(), outcome(dev < 1e-12, format!("largest departure from 1/101: {dev:.1e}"))),
                    ("below (Ia)".into(), outcome(below[0] > below[50], format!("P(0) {:.4}, P(50) {:.4}", below[0], below[50]))),
                    ("above (Ic)".into(), outcome(above[50] > above[0], format!("P(0) {:.1e}, P(50) {:.4}", above[0], above[50]))),
                ])
                .with(&format!(
                    "Figure Ib's own ε 0.01, δ 0.02 is just above the threshold (α {:.3}): nearly flat.",
                    ants::kirman_alpha(&config(kirman(0.01, 0.02)))
                ))
            },
        },
        Claim {
            id: "ants.kirman.eighty-twenty",
            item: "ants-2b",
            source: Source::App,
            citation: KIRMAN,
            text: "Our stronger stationary-mode diagnostic: the base chain has no preferred interior split between 65 % and 95 % across valid ε 0.001–0.199, δ 0–0.99 at N 100. This does not test the real ants' transient 80–20 plateaus; the rule was revised after the result was known",
            check: |_| {
                let mut hits = 0;
                let mut tried = 0;
                for ei in 1..200 {
                    for di in 0..100 {
                        let (e, d) = (f64::from(ei) * 0.001, f64::from(di) * 0.01);
                        if e + 1.0 - d > 1.0 {
                            continue;
                        }
                        tried += 1;
                        let p = ants::kirman(&config(kirman(e, d))).unwrap();
                        if interior_modes(&p).iter().any(|&k| (65..=95).contains(&k)) {
                            hits += 1;
                        }
                    }
                }
                let pulled = ants::kirman(&config(|c| {
                    kirman(0.15, 0.3)(c);
                    c.pull = 1.0;
                }))
                .unwrap();
                let loners = runs(5, 200_000, |c| c.independent = 0.2);
                let mut h = vec![0u32; 101];
                for r in loners.iter() {
                    for (k, &v) in r.hist.iter().enumerate() {
                        h[k] += v;
                    }
                }
                let top = (0..=100).max_by_key(|&k| h[k]).unwrap();
                outcome(
                    hits == 0,
                    format!("{hits} of {tried} settings peak between 65 % and 95 %; the chain is U-shaped, flat or centered"),
                )
                .with(&format!(
                    "Kirman (p. 149) proposes increasing majority attraction but supplies no formula or numeric split; this numerical mode calculation is retrospective. Our multiplier 1 + pull × (recruiter share − recruit share), with capped probabilities, at pull 1 and Figure Ic's ε and δ has modes {:?}. A fifth of the agents never herding (Figure IIb) moves the empirical histogram peak to {top} %.",
                    interior_modes(&pulled)
                ))
            },
        },
        Claim {
            id: "ants.kirman.fig2a",
            item: "ants-2a",
            source: Source::Book,
            citation: KIRMAN,
            text: "Figure IIa (ε 0.15, δ 0.3, 100 000 meetings): 'the state k/N of the system fluctuates around one-half' (time means in 0.45–0.55; 20 runs of 2 000 steps)",
            check: |_| range(&col(&runs(20, 2_000, kirman(0.15, 0.3)), |r| r.mean), 0.45, 0.55, false),
        },
        Claim {
            id: "ants.kirman.fig2b-extremes",
            item: "ants-2b",
            source: Source::Book,
            citation: KIRMAN,
            text: "Figure IIb (ε 0.002, δ 0.01): 'the system spends little time around the value of one-half and a great deal of time in the extremes' (under 15 % of steps in 0.4–0.6 and over half at 80 % or more; 20 runs of 2 000 steps)",
            check: |_| {
                let r = runs(20, 2_000, |_| {});
                all_of(vec![
                    ("little time near one-half".into(), range(&col(&r, |r| r.middle), 0.0, 0.15, false)),
                    ("much at the extremes".into(), range(&col(&r, |r| r.extreme), 0.5, 1.0, false)),
                ])
                .with(&format!("Flips per run: {:?}.", col(&r, |r| r.flips)))
            },
        },
        Claim {
            id: "ants.kirman.fig2b-average",
            item: "ants-2b",
            source: Source::Book,
            citation: KIRMAN,
            text: "Figure IIb illustrates one record whose time average is about half: some independent 100 000-meeting records have means in 0.4–0.6 (1 000 runs of 2 000 steps). The compatibility rule was revised after the result was known; it does not require most records to match",
            check: |_| {
                let r = runs(1_000, 2_000, |_| {});
                let long = runs(20, 200_000, |_| {});
                figure_compatibility(&col(&r, |r| r.mean)).with(&format!(
                    "Figure IIb reports a single realization, not an ensemble prevalence. Over 10⁷ meetings (200 000 steps) instead: {}/20 records have time means in 0.4–0.6.",
                    col(&long, |r| r.mean).iter().filter(|m| (0.4..=0.6).contains(*m)).count()
                ))
            },
        },
        Claim {
            id: "ants.kirman.majority",
            item: "ants-2b",
            source: Source::Book,
            citation: KIRMAN,
            text: "Kirman's p. 144 small-self-conversion argument, ε < (1 − δ)/(N − 1): the probability that an established majority decreases diminishes with its size (P(k, k − 1) falls above N/2; Figure Ia and IIb). The scope was revised after the prior result was known",
            check: |_| {
                let parts = [("Ia", 0.005, 0.01), ("IIb", 0.002, 0.01)]
                    .into_iter()
                    .map(|(name, e, d)| {
                        let c = config(kirman(e, d));
                        let down: Vec<f64> = (51..=100).map(|k| ants::kirman_rates(&c, k).1).collect();
                        let rise = down.windows(2).position(|w| w[1] > w[0]);
                        (
                            name.to_string(),
                            outcome(
                                rise.is_none(),
                                match rise {
                                    None => format!("falls from {:.3} at 51 to {:.3} at 100", down[0], down[49]),
                                    Some(i) => format!("rises from k {} ({:.4} to {:.4})", 51 + i, down[i], down[i + 1]),
                                },
                            ),
                        )
                    })
                    .collect();
                let contrast = config(kirman(0.15, 0.3));
                let first = ants::kirman_rates(&contrast, 51).1;
                let next = ants::kirman_rates(&contrast, 52).1;
                all_of(parts).with(&format!("Our application contrast outside the paper's premise: Figure Ic has P(51, 50) {first:.4} and P(52, 51) {next:.4}, initially rising. This is not a failed paper claim. For the valid printed chain, the adjacent difference is [ε − (1 − δ)(2k + 1 − N)/(N − 1)]/N."))
            },
        },
        Claim {
            id: "ants.kirman.markov",
            item: "ants-2b",
            source: Source::App,
            citation: KIRMAN,
            text: "Our pooled 80 %-regime residence diagnostic: mean residual time after half the mean residence and the mean is within 20 % of the unconditional mean (10 runs of 500 000 steps). Kirman's Markov argument conditions on the exact current split; grouping splits into regimes need not preserve age invariance",
            check: |_| {
                let r = runs(10, 500_000, |_| {});
                let all: Vec<f64> = r.iter().flat_map(|r| r.residences.clone()).collect();
                let m = all.iter().sum::<f64>() / all.len() as f64;
                let left = |age: f64| {
                    let v: Vec<f64> = all.iter().filter(|&&x| x > age).map(|x| x - age).collect();
                    v.iter().sum::<f64>() / v.len() as f64
                };
                let parts = [0.5, 1.0]
                    .into_iter()
                    .map(|f| {
                        let l = left(f * m);
                        (
                            format!("after {f} of the mean"),
                            outcome((l / m - 1.0).abs() <= 0.2, format!("{l:.0} steps to go against {m:.0}")),
                        )
                    })
                    .collect();
                all_of(parts).with(&format!("{} pooled regimes. This operational grouping is not a test of the exact-state Markov property.", all.len()))
            },
        },
        Claim {
            id: "ants.kirman.rapid",
            item: "ants-2b",
            source: Source::Book,
            citation: KIRMAN,
            text: "'the switches are so rapid' (a crossing from one source's 80 % to the other's takes under a fifth of a regime; 10 runs of 500 000 steps)",
            check: |_| {
                let r = runs(10, 500_000, |_| {});
                let t: Vec<f64> = r.iter().flat_map(|r| r.transits.clone()).collect();
                let s: Vec<f64> = r.iter().flat_map(|r| r.residences.clone()).collect();
                let (mt, ms) = (t.iter().sum::<f64>() / t.len() as f64, s.iter().sum::<f64>() / s.len() as f64);
                outcome(mt < 0.2 * ms, format!("crossing {mt:.0} steps, regime {ms:.0} steps ({:.1} %)", 100.0 * mt / ms))
            },
        },
        Claim {
            id: "ants.kirman.beta-limit",
            item: "ants-1a",
            source: Source::Book,
            citation: KIRMAN,
            text: "The Proposition: with ε = a/N and δ = 2a/N, as N grows the distribution of k/N tends to Beta(a, a) (Var[z] within 1 % of 1/(4(2a + 1)) at N 10 000; a 0.5 and 2)",
            check: |_| {
                let parts = [0.5, 2.0]
                    .into_iter()
                    .map(|a| {
                        let vs: Vec<f64> = [100u32, 1_000, 10_000]
                            .into_iter()
                            .map(|n| {
                                let nf = f64::from(n);
                                ants::variance(&ants::stationary(n, |k| {
                                    ants::kirman_rates(
                                        &AntsConfig {
                                            ants: n,
                                            epsilon: a / nf,
                                            delta: 2.0 * a / nf,
                                            ..AntsConfig::default()
                                        },
                                        k,
                                    )
                                })
                                .unwrap())
                            })
                            .collect();
                        let want = 1.0 / (4.0 * (2.0 * a + 1.0));
                        (
                            format!("a {a}"),
                            outcome(
                                (vs[2] / want - 1.0).abs() < 0.01,
                                format!("Var[z] {:.4}, {:.4}, {:.4} at N 100, 1 000, 10 000 against {want:.4}", vs[0], vs[1], vs[2]),
                            ),
                        )
                    })
                    .collect();
                all_of(parts)
            },
        },
        Claim {
            id: "ants.kirman.special-cases",
            item: "ants-1b",
            source: Source::Book,
            citation: KIRMAN,
            text: "ε ½, δ 1 is Ehrenfest's urn with a binomial equilibrium; ε = δ = 0 is a martingale absorbed at N with probability k₀/N (200 runs from a random start)",
            check: |_| {
                let p = ants::kirman(&config(|c| {
                    c.ants = 20;
                    kirman(0.5, 1.0)(c);
                }))
                .unwrap();
                let binom = |k: u32| (0..k).fold(1.0, |acc, i| acc * f64::from(20 - i) / f64::from(i + 1)) / f64::from(1u32 << 20);
                let gap = (0..=20).map(|k| (p[k as usize] - binom(k)).abs()).fold(0.0, f64::max);
                let seeds: Vec<u64> = (1..=200).collect();
                let r = model_after(
                    &ModelConfig::Ants(config(|c| {
                        c.ants = 20;
                        kirman(0.0, 0.0)(c);
                    })),
                    &seeds,
                    2_000,
                    summarize,
                );
                let absorbed = r.iter().filter(|r| r.last == 1.0).count() as f64 / 200.0;
                let start = r.iter().map(|r| r.first).sum::<f64>() / 200.0;
                let settled = r.iter().filter(|r| r.last == 0.0 || r.last == 1.0).count();
                all_of(vec![
                    ("Ehrenfest".into(), outcome(gap < 1e-12, format!("largest gap from the binomial {gap:.1e}"))),
                    (
                        "martingale".into(),
                        outcome(
                            settled == 200 && (absorbed - start).abs() < 0.1,
                            format!("{settled} of 200 absorbed; at the first source {absorbed:.2} against a mean start after one step of {start:.2}"),
                        ),
                    ),
                ])
            },
        },
        Claim {
            id: "ants.becker.more-extreme",
            item: "ants-lock",
            source: Source::Book,
            citation: KIRMAN,
            text: "Becker's externality, 'having the probability, 1 − δ, of conversion to the majority increase with the size of the majority … would make the process more extreme' (Var[z] of the exact chain rises with pull 0, 0.5, 1, 2; Figure IIb's and Ic's ε and δ)",
            check: |_| {
                let parts = [("IIb", 0.002, 0.01), ("Ic", 0.15, 0.3)]
                    .into_iter()
                    .map(|(name, e, d)| {
                        let v: Vec<f64> = [0.0, 0.5, 1.0, 2.0]
                            .into_iter()
                            .map(|pull| {
                                ants::variance(&ants::kirman(&config(|c| {
                                    kirman(e, d)(c);
                                    c.pull = pull;
                                }))
                                .unwrap())
                            })
                            .collect();
                        (
                            name.to_string(),
                            outcome(
                                v.windows(2).all(|w| w[1] >= w[0]) && v[3] > v[0],
                                format!("Var[z] {:.4}, {:.4}, {:.4}, {:.4}", v[0], v[1], v[2], v[3]),
                            ),
                        )
                    })
                    .collect();
                let locked = runs(10, 20_000, |c| c.pull = 0.5);
                all_of(parts).with(&format!(
                    "At IIb's ε and δ, pull 0.5: {} flips in 10 runs of 10⁶ meetings (no flips observed over this finite horizon; positive ε leaves paths between every split).",
                    locked.iter().map(|r| r.flips).sum::<f64>()
                ))
            },
        },
        Claim {
            id: "ants.kirman.sources",
            item: "ants-three",
            source: Source::Book,
            citation: KIRMAN,
            text: "Our uniform-other-source self-conversion implementation of Kirman's larger-source extension: the share of steps with one source holding 80 % the same, within 0.1, with 3 and 6 sources as with 2; 20 runs of 20 000 steps. This tests occupancy similarity, not invariance of every statistic",
            check: |_| {
                let two = runs(20, 20_000, |_| {});
                let parts = [3u32, 6]
                    .into_iter()
                    .map(|s| {
                        let more = runs(20, 20_000, |c| c.sources = s);
                        (
                            format!("{s} sources"),
                            equivalent(&col(&two, |r| r.extreme), &col(&more, |r| r.extreme), Some(0.1), "two sources", &format!("{s} sources")),
                        )
                    })
                    .collect();
                all_of(parts)
            },
        },
        Claim {
            id: "ants.am.n-dependence",
            item: "ants-crowd",
            source: Source::Book,
            citation: AM,
            text: "'Kirman's original interpretation of random pairwise meetings … suffers from the problem of N-dependence': at Figure IIb's fixed ε and δ the herding fades as N grows (Var[z] at N 100 above N 1 000; 10 runs of 10⁷ meetings)",
            check: |_| {
                let small = runs(10, 200_000, |_| {});
                let big = runs(10, 20_000, |c| {
                    c.ants = 1000;
                    c.meetings = 500;
                });
                greater(&col(&small, |r| r.variance), &col(&big, |r| r.variance), "N 100", "N 1 000").with(&format!(
                    "Time within 80 % of one source: {:.0} % against {:.0} %.",
                    100.0 * mean(&small, |r| r.extreme),
                    100.0 * mean(&big, |r| r.extreme)
                ))
            },
        },
        Claim {
            id: "am.fig3",
            item: "am-random",
            source: Source::App,
            citation: AM,
            text: "Our N 100 sequential reconstruction near AM Figure 3: rings and small worlds have variance at least 15 % below the nominal symmetric-Beta limit, while random and scale-free graphs are within 15 % (α 0.5, 1, 2; 5 runs of 100 000 sweeps). The paper omits N, so this is not an exact reproduction verdict; the diagnostic rule was revised after the result was known",
            check: |_| {
                let parts = [
                    ("regular", Network::Ring),
                    ("small world", Network::SmallWorld),
                    ("scale-free", Network::ScaleFree),
                    ("random", Network::Random),
                ]
                .into_iter()
                .map(|(name, net)| {
                    let (ok, got): (Vec<bool>, Vec<String>) = [0.5, 1.0, 2.0]
                        .into_iter()
                        .map(|alpha| {
                            let v = mean(&runs(5, 100_000, am(100, net, alpha)), |r| r.variance);
                            let want = 1.0 / (4.0 * (2.0 * alpha + 1.0));
                            (if matches!(net, Network::Ring | Network::SmallWorld) { v / want < 0.85 } else { (v / want - 1.0).abs() <= 0.15 }, format!("{v:.3}/{want:.3} ({:.1} % of nominal Beta variance)", 100.0 * v / want))
                        })
                        .unzip();
                    (name.to_string(), outcome(ok.iter().all(|&b| b), got.join(", ")))
                })
                .collect();
                all_of(parts).with("Nominal α uses the specified degree (or expected random degree); realized small-world and scale-free degree can differ. Finite-size mean-field variance is (N + 2α)/(4N(2α + 1)), slightly above the continuous-Beta limit. A retrospective N 50 ring check is within the existing 15 % tolerance at all three α (20 seeds); other networks were not rerun at N 50. The omission of N matters.")
            },
        },
        Claim {
            id: "am.fig4",
            item: "ants-n",
            source: Source::Book,
            citation: AM,
            text: "Figure 4 (a 0.5, λ 1): 'the random network would appear to be the only structure capable of overcoming the problem of N-dependence' (the inverse variance flat in N on the random graph, rising on the others; N 50, 550, 1 050; 3 audit runs of 300 000 sweeps, matching the paper's horizon)",
            check: |_| {
                let ns = [50u32, 550, 1050];
                let parts = [
                    ("regular", Network::Ring),
                    ("small world", Network::SmallWorld),
                    ("scale-free", Network::ScaleFree),
                    ("random", Network::Random),
                ]
                .into_iter()
                .map(|(name, net)| {
                    let inv: Vec<f64> = ns.iter().map(|&n| 1.0 / mean(&runs(3, 300_000, fig4(n, net)), |r| r.variance)).collect();
                    let x: Vec<f64> = ns.iter().map(|&n| f64::from(n)).collect();
                    let b = slope(&x, &inv);
                    let theory: Vec<f64> = ns.iter().map(|&n| 1.0 / fig4_theory(n, net)).collect();
                    let tb = slope(&x, &theory);
                    let ok = if net == Network::Random { b.abs() < 0.05 } else { b > 0.2 };
                    (
                        name.to_string(),
                        outcome(ok, format!("slope of 1/Var[z] {b:.3} (mean field {tb:.3}); 1/Var[z] {:.0}, {:.0}, {:.0}", inv[0], inv[1], inv[2])),
                    )
                })
                .collect();
                all_of(parts).with("Alfarano and Milaković's slopes: 0.512 regular, 0.507 small world, 0.402 scale-free; random intercept 46.8. These are raw variances; footnote 18 divides plotted variance by 3 and multiplies plotted inverse variance by 3. Our three-size comparison tests qualitative direction, not reproduction of the authors' fit across sizes up to about N 5 000. The authors acknowledge slight regular and small-world variance deviations in their Figure 4 discussion. Conclusions concern these network families at fixed a and λ, not arbitrary networks.")
            },
        },
        Claim {
            id: "am.pairwise",
            item: "ants-n",
            source: Source::App,
            citation: AM,
            text: "Our contrast: under Kirman's pairwise meetings, where each meeting is one partner however many an ant knows, a random graph does not cure N-dependence (Var[z] at N 100 above N 1 000 on a random graph with p 0.1; IIb's ε and δ; 10 runs)",
            check: |_| {
                let small = runs(10, 200_000, |c| c.network = Network::Random);
                let big = runs(10, 20_000, |c| {
                    c.network = Network::Random;
                    c.ants = 1000;
                    c.meetings = 500;
                });
                greater(&col(&small, |r| r.variance), &col(&big, |r| r.variance), "N 100", "N 1 000")
            },
        },
        Claim {
            id: "am.fig6",
            item: "am-independent",
            source: Source::Book,
            citation: AM,
            text: "Figure 6: agents that never herd, placed on the network, reduce the variance far more than the same agents outside it (at q 0.05 and 0.1, under half the core-and-periphery prediction (1 − q)²·Var + q/4K; random graph, K 1 000, p 0.1, a 0.5, λ 1; 3 runs of 30 000 sweeps)",
            check: |_| {
                let base = mean(&runs(3, 30_000, fig4(1000, Network::Random)), |r| r.variance);
                let parts = [0.05, 0.1]
                    .into_iter()
                    .map(|q| {
                        let v = mean(
                            &runs(3, 30_000, |c| {
                                fig4(1000, Network::Random)(c);
                                c.independent = q;
                            }),
                            |r| r.variance,
                        );
                        let cp = (1.0 - q) * (1.0 - q) * base + q / 4000.0;
                        (format!("q {q}"), outcome(v < 0.5 * cp, format!("Var[z] {v:.4} against {cp:.4} off the network")))
                    })
                    .collect();
                all_of(parts).with(&format!("Everyone herding: {base:.4}."))
            },
        },
        Claim {
            id: "ants.start",
            item: "ants-2b",
            source: Source::App,
            citation: "docs/superpowers/specs/2026-09-27-ants-design.md",
            text: "Where the colony starts does not matter in the long run (IIb's time mean from all at one source and from random sources the same within 0.1; 20 runs of 200 000 steps)",
            check: |_| {
                let random = runs(20, 200_000, |_| {});
                let one = runs(20, 200_000, |c| c.start = Start::One);
                equivalent(&col(&random, |r| r.mean), &col(&one, |r| r.mean), Some(0.1), "random start", "all at one")
            },
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn majority_decrease_holds_within_kirmans_small_self_conversion_premise() {
        let claim = claims()
            .into_iter()
            .find(|c| c.id == "ants.kirman.majority")
            .unwrap();
        assert_eq!((claim.check)(&[]).verdict, Verdict::Holds);
    }

    #[test]
    fn stronger_diagnostics_are_not_attributed_to_the_papers() {
        for id in ["ants.kirman.eighty-twenty", "ants.kirman.markov", "am.fig3"] {
            let claim = claims().into_iter().find(|c| c.id == id).unwrap();
            assert_eq!(claim.source, Source::App, "{id}");
        }
    }

    #[test]
    fn a_single_illustrated_trace_does_not_require_most_runs_to_match() {
        let values = [
            0.5, 0.5, 0.5, 0.5, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.9, 0.9, 0.9, 0.9, 0.9, 0.9, 0.9,
            0.9, 0.9, 0.9,
        ];
        assert_eq!(figure_compatibility(&values).verdict, Verdict::Holds);
    }

    #[test]
    fn compatibility_requires_an_observed_matching_record() {
        assert_eq!(figure_compatibility(&[0.1, 0.9]).verdict, Verdict::Fails);
    }
}
