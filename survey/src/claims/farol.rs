//! El Farol and the minority game (milestone 23): Arthur's bar (1994),
//! Challet and Zhang's minority game (1997), Savit, Manuca and Riolo's
//! memory transition (1999) and Challet, Marsili and Ottino's critique
//! (2004). Runs are memoized per process, keyed by the config, the seeds and
//! the rounds; a run is read over its last four fifths.

use std::sync::{Arc, Mutex};

use sugarscape_core::farol::{
    Evolution, FarolConfig, Game, MixedMemory, Payoff, Rounding, Scoring,
};
use sugarscape_core::model::{ModelConfig, ModelWorld};

use crate::claim::{all_of, equivalent, greater, Claim, Outcome, Source, Verdict};
use crate::runner::model_after;

const ARTHUR: &str = "Arthur 1994, AER 84(2)";
const CZ: &str = "Challet & Zhang 1997, Physica A 246";
const SMR: &str = "Savit, Manuca & Riolo 1999, PRL 82";
const CMO: &str = "Challet, Marsili & Ottino 2004, Physica A 332";

/// One run, read over its last four fifths.
#[derive(Clone, Debug)]
struct Run {
    mean: f64,
    /// ⟨(A − c)²⟩ / N, and the coin-flippers' value.
    fluct: f64,
    random: f64,
    lag1: f64,
    success: f64,
    above: f64,
    /// σ²/N over the first and the last tenth.
    early: f64,
    late: f64,
    memory: f64,
    /// Share of rounds with attendance within 5 % of N of the center.
    central: f64,
    /// Per agent: memory, wins per round, switches.
    agents: Vec<(u32, f64, f64)>,
}

fn summarize(w: &ModelWorld, rounds: u32) -> Run {
    let m = w.model();
    let ModelConfig::Farol(c) = m.config() else {
        unreachable!()
    };
    let n = f64::from(c.agents);
    let center = if c.game == Game::Minority && 2 * c.capacity + 1 == c.agents {
        n / 2.0
    } else {
        f64::from(c.capacity)
    };
    let a = m.series("attendance").unwrap();
    let played = &a[1..];
    let tail = &played[played.len() / 5..];
    let t = tail.len() as f64;
    let mean = tail.iter().sum::<f64>() / t;
    let sq = |s: &[f64]| s.iter().map(|x| (x - center).powi(2)).sum::<f64>() / s.len() as f64 / n;
    let var = tail.iter().map(|x| (x - mean).powi(2)).sum::<f64>();
    let lag1 = if var > 0.0 {
        tail.windows(2)
            .map(|w| (w[0] - mean) * (w[1] - mean))
            .sum::<f64>()
            / var
    } else {
        0.0
    };
    let avg = |name: &str| {
        let s = m.series(name).unwrap();
        s[s.len() - tail.len()..].iter().sum::<f64>() / t
    };
    let tenth = (played.len() / 10).max(1);
    let agents = m
        .agents_csv()
        .lines()
        .skip(1)
        .map(|l| {
            let f: Vec<&str> = l.split(',').collect();
            (
                f[1].parse().unwrap(),
                f[3].parse::<f64>().unwrap() / f64::from(rounds),
                f[4].parse().unwrap(),
            )
        })
        .collect();
    Run {
        mean,
        fluct: sq(tail),
        random: m.latest_value("random_fluctuation").unwrap(),
        lag1,
        success: avg("success"),
        above: avg("forecast_above"),
        early: sq(&played[..tenth]),
        late: sq(&played[played.len() - tenth..]),
        memory: m.latest_value("mean_memory").unwrap(),
        central: tail
            .iter()
            .filter(|&&x| (x - center).abs() <= 0.05 * n)
            .count() as f64
            / t,
        agents,
    }
}

/// The model's defaults (Arthur's bar) with `edit`, seeds 1..=`seeds`, `rounds` rounds.
fn runs(seeds: u64, rounds: u32, edit: impl FnOnce(&mut FarolConfig)) -> Arc<Vec<Run>> {
    type Cache = Mutex<Vec<(String, u64, u32, Arc<Vec<Run>>)>>;
    static CACHE: Cache = Mutex::new(Vec::new());
    let mut c = FarolConfig::default();
    edit(&mut c);
    let key = serde_json::to_string(&c).expect("configs serialize");
    if let Some((_, _, _, v)) = CACHE
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .iter()
        .find(|(k, s, r, _)| *k == key && *s == seeds && *r == rounds)
    {
        return v.clone();
    }
    let seed_list: Vec<u64> = (1..=seeds).collect();
    let v = Arc::new(model_after(
        &ModelConfig::Farol(c),
        &seed_list,
        rounds,
        |w| summarize(w, rounds),
    ));
    CACHE
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .push((key, seeds, rounds, v.clone()));
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

fn arthur(k: u32) -> impl FnOnce(&mut FarolConfig) {
    move |c| c.strategies = k
}

/// The plain minority game.
fn mg(n: u32, s: u32, m: u32) -> impl FnOnce(&mut FarolConfig) {
    move |c| {
        c.game = Game::Minority;
        c.agents = n;
        c.strategies = s;
        c.memory = m;
        c.capacity = (n - 1) / 2;
    }
}

/// CMO's binary El Farol: 60 seats, bias 0.5, 2 strategies.
fn binary(n: u32, m: u32) -> impl FnOnce(&mut FarolConfig) {
    move |c| {
        mg(n, 2, m)(c);
        c.capacity = 60;
    }
}

fn evolving(
    n: u32,
    m: u32,
    every: u32,
    strategy: f64,
    memory: f64,
) -> impl FnOnce(&mut FarolConfig) {
    move |c| {
        mg(n, 5, m)(c);
        c.evolution = Evolution {
            enabled: true,
            every,
            strategy_mutation: strategy,
            memory_mutation: memory,
        };
    }
}

/// σ²/N for the plain minority game at N, S 2, memories 1..=14 (10 seeds, 5 000 rounds).
fn curve(n: u32) -> Vec<f64> {
    (1..=14)
        .map(|m| mean(&runs(10, 5_000, mg(n, 2, m)), |r| r.fluct))
        .collect()
}

fn argmin(v: &[f64]) -> usize {
    (0..v.len()).fold(0, |k, i| if v[i] < v[k] { i } else { k })
}

pub fn claims() -> Vec<Claim> {
    vec![
        Claim {
            id: "farol.arthur.mean-60",
            item: "ef-arthur",
            source: Source::Book,
            citation: ARTHUR,
            text: "'mean attendance converges always to 60' (k 6, 12, 23; within 2 of 60; 20 runs of 2 000 rounds)",
            check: |_| {
                let parts = [6, 12, 23]
                    .into_iter()
                    .map(|k| {
                        let m = mean(&runs(20, 2_000, arthur(k)), |r| r.mean);
                        (format!("k {k}"), outcome((m - 60.0).abs() <= 2.0, format!("mean {m:.1}")))
                    })
                    .collect();
                all_of(parts)
            },
        },
        Claim {
            id: "farol.arthur.no-cycles",
            item: "ef-arthur",
            source: Source::Book,
            citation: ARTHUR,
            text: "'cycles are quickly arbitraged away so there are no persistent cycles' (lag-1 autocorrelation of attendance within ±0.2; k 6, 12, 23)",
            check: |_| {
                let parts = [6, 12, 23]
                    .into_iter()
                    .map(|k| {
                        let a = mean(&runs(20, 2_000, arthur(k)), |r| r.lag1);
                        (format!("k {k}"), outcome(a.abs() <= 0.2, format!("lag-1 autocorrelation {a:.2}")))
                    })
                    .collect();
                let payoff = mean(&runs(20, 2_000, |c| c.scoring = Scoring::Payoff), |r| r.lag1);
                all_of(parts).with(&format!("Rated by payoff instead (CMO), k 12: {payoff:.2}."))
            },
        },
        Claim {
            id: "farol.arthur.forty-percent",
            item: "ef-arthur",
            source: Source::Book,
            citation: ARTHUR,
            text: "'of the active predictors … on average 40 percent are forecasting above 60' (35–45 %; k 6, 12, 23)",
            check: |_| {
                let parts = [6, 12, 23]
                    .into_iter()
                    .map(|k| {
                        let a = mean(&runs(20, 2_000, arthur(k)), |r| r.above);
                        (format!("k {k}"), outcome((0.35..=0.45).contains(&a), format!("{:.0} % above 60", 100.0 * a)))
                    })
                    .collect();
                all_of(parts)
            },
        },
        Claim {
            id: "farol.arthur.robust",
            item: "ef-predictors",
            source: Source::Book,
            citation: ARTHUR,
            text: "The results 'are robust to changes in types of predictors created and in numbers assigned' (mean within 2 of 60 for k 2 to 32, and with either rule at exactly 60)",
            check: |_| {
                let mut parts: Vec<(String, Outcome)> = [2, 4, 16, 32]
                    .into_iter()
                    .map(|k| {
                        let m = mean(&runs(20, 2_000, arthur(k)), |r| r.mean);
                        (format!("k {k}"), outcome((m - 60.0).abs() <= 2.0, format!("mean {m:.1}")))
                    })
                    .collect();
                let go = mean(&runs(20, 2_000, |c| c.at_capacity = sugarscape_core::farol::AtCapacity::Go), |r| r.mean);
                parts.push(("going at exactly 60".into(), outcome((go - 60.0).abs() <= 2.0, format!("mean {go:.1}"))));
                all_of(parts)
            },
        },
        Claim {
            id: "farol.cmo.trivial-mean",
            item: "ef-random",
            source: Source::Book,
            citation: CMO,
            text: "'even zero-intelligence agents … are able to self-organize to the comfort level': agents going at random with probability 0.6 average 60 (within 0.5)",
            check: |_| {
                let m = mean(&runs(20, 2_000, |c| c.behavior = sugarscape_core::farol::Behavior::Random), |r| r.mean);
                outcome((m - 60.0).abs() <= 0.5, format!("mean {m:.2}"))
            },
        },
        Claim {
            id: "farol.cmo.worse-than-random",
            item: "ef-predictors",
            source: Source::Book,
            citation: CMO,
            text: "The real question is the fluctuations around the comfort level; Arthur's inductive agents fluctuate more than agents going at random (both scorings, k 12)",
            check: |_| {
                let acc = runs(20, 2_000, arthur(12));
                let pay = runs(20, 2_000, |c| c.scoring = Scoring::Payoff);
                let random = runs(20, 2_000, |c| c.behavior = sugarscape_core::farol::Behavior::Random);
                all_of(vec![
                    ("accuracy".into(), greater(&col(&acc, |r| r.fluct), &col(&random, |r| r.fluct), "rated by accuracy", "random")),
                    ("payoff".into(), greater(&col(&pay, |r| r.fluct), &col(&random, |r| r.fluct), "rated by payoff", "random")),
                ])
            },
        },
        Claim {
            id: "farol.cmo.binary-worse",
            item: "cmo-bias",
            source: Source::Book,
            citation: CMO,
            text: "The binary El Farol: 'in the region āN ≈ L adaptive agents behave less efficiently than random agents … stronger for small values of m' (N 120, L 60, m 2; 10 runs of 5 000 rounds)",
            check: |_| {
                let r = runs(10, 5_000, binary(120, 2));
                let (f, base) = (mean(&r, |r| r.fluct), r[0].random);
                outcome(f > base, format!("σ²/N {f:.2} against random agents' {base:.2}"))
            },
        },
        Claim {
            id: "farol.cmo.bias-helps",
            item: "cmo-bias",
            source: Source::Book,
            citation: CMO,
            text: "'a small bias in the strategies, of either sign, is beneficial as it decreases the fluctuations' (m 2: N 90 and 150 against N 120)",
            check: |_| {
                let at = |n| runs(10, 5_000, binary(n, 2));
                all_of(vec![
                    ("N 90".into(), greater(&col(&at(120), |r| r.fluct), &col(&at(90), |r| r.fluct), "N 120", "N 90")),
                    ("N 150".into(), greater(&col(&at(120), |r| r.fluct), &col(&at(150), |r| r.fluct), "N 120", "N 150")),
                ])
            },
        },
        Claim {
            id: "farol.cmo.intermediate-memory",
            item: "cmo-bias",
            source: Source::Book,
            citation: CMO,
            text: "'there is an intermediate memory length which is optimal for the collective behavior' (N 120, L 60: m 6 against m 2 and m 12)",
            check: |_| {
                let at = |m| runs(10, 5_000, binary(120, m));
                all_of(vec![
                    ("m 2".into(), greater(&col(&at(2), |r| r.fluct), &col(&at(6), |r| r.fluct), "m 2", "m 6")),
                    ("m 12".into(), greater(&col(&at(12), |r| r.fluct), &col(&at(6), |r| r.fluct), "m 12", "m 6")),
                ])
            },
        },
        Claim {
            id: "farol.cmo.interval-shrinks",
            item: "cmo-bias",
            source: Source::Book,
            citation: CMO,
            text: "⟨A⟩ ≈ L 'in a whole interval around Nā = L' for small m, and 'the region where ⟨A⟩ ≈ L shrinks' as m grows (|⟨A⟩ − 60| at N 90 and 150: m 2 against m 6)",
            check: |_| {
                let off = |n, m| mean(&runs(10, 5_000, binary(n, m)), |r| (r.mean - 60.0).abs());
                let (a, b) = (off(90, 2) + off(150, 2), off(90, 6) + off(150, 6));
                outcome(a < b, format!("summed |⟨A⟩ − 60|: m 2 {a:.2}, m 6 {b:.2}"))
            },
        },
        Claim {
            id: "farol.cz.fig1",
            item: "mg-fig-1",
            source: Source::Book,
            citation: CZ,
            text: "Fig. 1 (N 1001, M 6, 8, 10): 'the fluctuation are indeed in decreasing order for ever increasingly intelligent players' (S 5; 5 runs of 3 000 rounds)",
            check: |_| {
                let f: Vec<f64> = [6, 8, 10].into_iter().map(|m| mean(&runs(5, 3_000, mg(1001, 5, m)), |r| r.fluct)).collect();
                let s2: Vec<f64> = [6, 8, 10].into_iter().map(|m| mean(&runs(5, 3_000, mg(1001, 2, m)), |r| r.fluct)).collect();
                outcome(f[0] > f[1] && f[1] > f[2], format!("σ²/N {:.2}, {:.2}, {:.2}", f[0], f[1], f[2]))
                    .with(&format!("With S 2: {:.2}, {:.2}, {:.2}.", s2[0], s2[1], s2[2]))
            },
        },
        Claim {
            id: "farol.cz.fig2",
            item: "mg-mixed",
            source: Source::Book,
            citation: CZ,
            text: "Fig. 2 (memories 1–10 mixed, N 1001, S 5): longer memories win more, and 'above a certain size (M ≈ 6) the average performance … appears to saturate'",
            check: |_| {
                let r = runs(5, 5_000, |c| {
                    mg(1001, 5, 10)(c);
                    c.mixed_memory = MixedMemory { enabled: true, min: 1, max: 10 };
                });
                let by = |m: u32| {
                    let v: Vec<f64> = r.iter().flat_map(|x| x.agents.iter().filter(|a| a.0 == m).map(|a| a.1)).collect();
                    v.iter().sum::<f64>() / v.len() as f64
                };
                let s: Vec<f64> = (1..=10).map(by).collect();
                let rising = s[..6].windows(2).all(|w| w[1] > w[0] - 0.002);
                let flat = s[5..].iter().all(|&x| (x - s[5]).abs() < 0.01);
                outcome(rising && flat && s[5] > s[0] + 0.1, format!("wins per round {}", s.iter().map(|x| format!("{x:.3}")).collect::<Vec<_>>().join(", ")))
            },
        },
        Claim {
            id: "farol.cz.fig4",
            item: "mg-inverse",
            source: Source::Book,
            citation: CZ,
            text: "Fig. 4 (payoff N/x − 2, 'nearest integer values', N 1001, M 4, S 5): attendance has 'two peaks' (most rounds more than 5 % of N from the center)",
            check: |_| {
                let at = |rounding| {
                    runs(5, 5_000, move |c| {
                        mg(1001, 5, 4)(c);
                        c.payoff = Payoff::Inverse;
                        c.rounding = rounding;
                    })
                };
                let (near, exact) = (at(Rounding::Nearest), at(Rounding::Exact));
                let (a, b) = (mean(&near, |r| r.central), mean(&exact, |r| r.central));
                outcome(a < 0.5, format!("{:.0} % of rounds within 5 % of the center (rounded)", 100.0 * a))
                    .with(&format!("Unrounded: {:.0} %, σ²/N {:.2}.", 100.0 * b, mean(&exact, |r| r.fluct)))
            },
        },
        Claim {
            id: "farol.cz.fig5",
            item: "mg-strategies",
            source: Source::Book,
            citation: CZ,
            text: "Fig. 5 (N 1001, M 5): 'with increasing number of alternatives the players tend to perform worse' (S 2 against S 9)",
            check: |_| {
                let at = |s| runs(5, 3_000, mg(1001, s, 5));
                greater(&col(&at(2), |r| r.success), &col(&at(9), |r| r.success), "S 2", "S 9")
            },
        },
        Claim {
            id: "farol.cz.fig6",
            item: "mg-m8",
            source: Source::Book,
            citation: CZ,
            text: "Fig. 6: 'the oftener one switches, less successful one would end up' (switches against wins across agents, N 1001, M 5, S 5: a negative correlation in every run)",
            check: |_| {
                let r = runs(5, 3_000, mg(1001, 5, 5));
                let corr = |x: &Run| {
                    let n = x.agents.len() as f64;
                    let (ms, mw) = (x.agents.iter().map(|a| a.2).sum::<f64>() / n, x.agents.iter().map(|a| a.1).sum::<f64>() / n);
                    let cov: f64 = x.agents.iter().map(|a| (a.2 - ms) * (a.1 - mw)).sum();
                    let vs: f64 = x.agents.iter().map(|a| (a.2 - ms).powi(2)).sum();
                    let vw: f64 = x.agents.iter().map(|a| (a.1 - mw).powi(2)).sum();
                    cov / (vs * vw).sqrt()
                };
                let c = col(&r, corr);
                outcome(c.iter().all(|&x| x < 0.0), format!("correlations {}", c.iter().map(|x| format!("{x:.2}")).collect::<Vec<_>>().join(", ")))
            },
        },
        Claim {
            id: "farol.cz.fig9",
            item: "mg-evolution",
            source: Source::Book,
            citation: CZ,
            text: "Fig. 9: with the worst player replaced by a mutated copy of the best, 'Fluctuations are reduced' (N 1001, M 6, S 5, every 10 rounds, 5 runs of 40 000 rounds: the first tenth against the last)",
            check: |_| {
                let r = runs(5, 40_000, evolving(1001, 6, 10, 0.1, 0.0));
                greater(&col(&r, |r| r.early), &col(&r, |r| r.late), "first tenth", "last tenth")
            },
        },
        Claim {
            id: "farol.cz.fig10",
            item: "mg-inbred",
            source: Source::Book,
            citation: CZ,
            text: "Fig. 10: with perfect cloning and no mutation, 'there appears tremendous waste' (more fluctuation in the last tenth than with mutation; N 1001 and 101)",
            check: |_| {
                let parts = [1001, 101]
                    .into_iter()
                    .map(|n| {
                        let pure = runs(5, 40_000, evolving(n, 6, 10, 0.0, 0.0));
                        let mutated = runs(5, 40_000, evolving(n, 6, 10, 0.1, 0.0));
                        (format!("N {n}"), greater(&col(&pure, |r| r.late), &col(&mutated, |r| r.late), "no mutation", "mutation"))
                    })
                    .collect();
                all_of(parts)
            },
        },
        Claim {
            id: "farol.cz.fig11",
            item: "mg-arms-race",
            source: Source::Book,
            citation: CZ,
            text: "Fig. 11: from M 2, with memory mutation, an 'arm race' raises the mean memory and settles; 'Larger population (N = 1001) needs more powerful brains' (every 50 rounds, 5 runs of 100 000 rounds)",
            check: |_| {
                let small = runs(5, 100_000, evolving(101, 2, 50, 0.1, 0.1));
                let big = runs(5, 100_000, evolving(1001, 2, 50, 0.1, 0.1));
                let (a, b) = (mean(&small, |r| r.memory), mean(&big, |r| r.memory));
                outcome(a > 2.5 && b > a, format!("final mean memory {a:.2} (N 101), {b:.2} (N 1001)"))
            },
        },
        Claim {
            id: "farol.smr.collapse",
            item: "mg-memory",
            source: Source::Book,
            citation: SMR,
            text: "'σ²/N is a function only of 2^m/N': the minimum moves one memory step per doubling of N (N 51, 101, 201; S 2)",
            check: |_| {
                let m: Vec<usize> = [51, 101, 201].into_iter().map(|n| argmin(&curve(n)) + 1).collect();
                outcome(m[1] == m[0] + 1 && m[2] == m[1] + 1, format!("minimum at M {}, {}, {}", m[0], m[1], m[2]))
            },
        },
        Claim {
            id: "farol.smr.minimum",
            item: "mg-memory",
            source: Source::Book,
            citation: SMR,
            text: "The minimum lies 'near 2^m/N = zc ≈ 0.5' (between 0.3 and 0.7, for N 51, 101, 201)",
            check: |_| {
                let z: Vec<f64> = [51u32, 101, 201]
                    .into_iter()
                    .map(|n| f64::powi(2.0, argmin(&curve(n)) as i32 + 1) / f64::from(n))
                    .collect();
                outcome(z.iter().all(|x| (0.3..=0.7).contains(x)), format!("2^m/N at the minimum {:.2}, {:.2}, {:.2}", z[0], z[1], z[2]))
            },
        },
        Claim {
            id: "farol.smr.scaling",
            item: "mg-memory",
            source: Source::Book,
            citation: SMR,
            text: "'for fixed m, and m < mc, σ is proportional to N, while for fixed m and m > mc σ is proportional to N^½' (σ²/N at N 201 over N 51: about 4 at m 2, about 1 at m 14)",
            check: |_| {
                let (lo, hi) = (curve(51), curve(201));
                let (r2, r14) = (hi[1] / lo[1], hi[13] / lo[13]);
                outcome((2.8..=5.5).contains(&r2) && (0.7..=1.4).contains(&r14), format!("ratio {r2:.2} at m 2, {r14:.2} at m 14"))
            },
        },
        Claim {
            id: "farol.smr.beat-random",
            item: "mg-memory",
            source: Source::Book,
            citation: SMR,
            text: "For m < 6 (N 101, s 2) 'no agent ever achieves results statistically greater than 50 %', while 'for m ≥ 6 some agents do win more than 50 % of the time' (the best agent's wins per round at m 3, 6, 8, 10; 10 runs of 10 000 rounds)",
            check: |_| {
                let best = |m| {
                    mean(&runs(10, 10_000, mg(101, 2, m)), |r| {
                        r.agents.iter().map(|a| a.1).fold(0.0, f64::max)
                    })
                };
                let mut parts = vec![(
                    "m 3".to_string(),
                    {
                        let b = best(3);
                        outcome(b < 0.5, format!("best agent {b:.3}"))
                    },
                )];
                for m in [6, 8, 10] {
                    let b = best(m);
                    parts.push((format!("m {m}"), outcome(b > 0.5, format!("best agent {b:.3}"))));
                }
                all_of(parts)
            },
        },
        Claim {
            id: "farol.cmo.random-history",
            item: "mg-information",
            source: Source::Book,
            citation: CMO,
            text: "'The behavior of the Minority Game is largely unaffected if this dynamics is replaced by a random draw of μ(t)' (N 101, S 2, m 3 and 6: σ²/N with the true and a random history)",
            check: |_| {
                let parts = [3, 6]
                    .into_iter()
                    .map(|m| {
                        let t = runs(10, 5_000, mg(101, 2, m));
                        let r = runs(10, 5_000, move |c| {
                            mg(101, 2, m)(c);
                            c.information = sugarscape_core::farol::Information::Random;
                        });
                        let margin = 0.2 * mean(&t, |x| x.fluct);
                        (format!("m {m}"), equivalent(&col(&t, |x| x.fluct), &col(&r, |x| x.fluct), Some(margin), "true", "random"))
                    })
                    .collect();
                all_of(parts)
            },
        },
    ]
}
