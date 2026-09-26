//! Cohen, Riolo & Axelrod's social structure (milestone 18): Table 2, Fig. 1,
//! the crucial p–q region and Figs. 5–6, notes 1 and 5, Table A1, and the
//! paper's two readings of its own method. Runs use the paper's 30 seeds and
//! 2500 periods and are memoized per process, keyed by the config.

use std::sync::{Arc, Mutex};

use sugarscape_core::structure::{
    fanout, regression, NoiseOn, Start, Structure, StructureConfig, StructureWorld,
};

use crate::claim::{greater, Claim, Outcome, Source, Verdict};
use crate::stats::mean;

const CRA: &str = "Cohen, Riolo & Axelrod 2001, Rationality and Society 13(1)";
const PERIODS: u32 = 2500;
/// The paper's replications per case.
const SEEDS: u64 = 30;
/// The crucial region of population means (§3.3).
const P_RANGE: (f64, f64) = (0.30, 0.35);
const Q_RANGE: (f64, f64) = (0.05, 0.10);

/// One run, summarized.
#[derive(Clone, Debug, Default)]
struct Run {
    /// Mean payoff per move in every period (index 0: the start, no games).
    payoff: Vec<f64>,
    /// In the crucial region: the change of mean p in each visit, and the
    /// (own p, partners' mean p) pairs of those periods.
    deltas: Vec<f64>,
    pairs: Vec<(f64, f64)>,
}

impl Run {
    /// Table 2's mean payoff: periods 1501–2500.
    fn late_mean(&self) -> f64 {
        mean(&self.payoff[1501..=2500])
    }

    /// The first period at or above `high`, if any.
    fn attained(&self, high: f64) -> Option<usize> {
        (1..self.payoff.len()).find(|&t| self.payoff[t] >= high)
    }

    /// Table 2's "Remain High": the share of periods at or above `high` from
    /// the first one on.
    fn remain(&self, high: f64) -> Option<f64> {
        let t = self.attained(high)?;
        let since = &self.payoff[t..];
        Some(since.iter().filter(|&&v| v >= high).count() as f64 / since.len() as f64)
    }

    /// Stretches of at least `len` periods both above and below `high`.
    fn bistable(&self, high: f64, len: usize) -> bool {
        let (mut up, mut down, mut run, mut cur) = (false, false, 0, self.payoff[1] >= high);
        for &v in &self.payoff[1..] {
            let h = v >= high;
            if h == cur {
                run += 1;
            } else {
                run = 1;
                cur = h;
            }
            if run >= len {
                if cur {
                    up = true;
                } else {
                    down = true;
                }
            }
        }
        up && down
    }
}

fn simulate(c: &StructureConfig, seed: u64) -> Run {
    let mut w = StructureWorld::new(c.clone(), seed).expect("a valid config");
    let mut run = Run {
        payoff: vec![0.0],
        ..Run::default()
    };
    for _ in 0..PERIODS {
        let before = w.stats.latest().expect("a period").clone();
        w.step();
        let after = w.stats.latest().expect("a period");
        run.payoff.push(after.mean_payoff);
        if (P_RANGE.0..=P_RANGE.1).contains(&before.mean_p)
            && (Q_RANGE.0..=Q_RANGE.1).contains(&before.mean_q)
        {
            run.deltas.push(after.mean_p - before.mean_p);
            run.pairs.extend(w.partner_p_pairs());
        }
    }
    run
}

/// The paper's defaults (256 agents, grid start, noise on everyone, no stop)
/// with `edit`, over `seeds` seeds.
fn runs(seeds: u64, edit: impl FnOnce(&mut StructureConfig)) -> Arc<Vec<Run>> {
    type Cache = Mutex<Vec<(String, u64, Arc<Vec<Run>>)>>;
    static CACHE: Cache = Mutex::new(Vec::new());
    let mut c = StructureConfig::default();
    edit(&mut c);
    let key = serde_json::to_string(&c).expect("configs serialize");
    if let Some((_, _, v)) = CACHE
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .iter()
        .find(|(k, s, _)| *k == key && *s == seeds)
    {
        return v.clone();
    }
    let v: Vec<Run> = std::thread::scope(|scope| {
        let handles: Vec<_> = (1..=seeds)
            .map(|seed| {
                let c = &c;
                scope.spawn(move || simulate(c, seed))
            })
            .collect();
        handles
            .into_iter()
            .map(|h| h.join().expect("a run"))
            .collect()
    });
    let v = Arc::new(v);
    CACHE
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .push((key, seeds, v.clone()));
    v
}

/// Table 2's rows: (name, structure, substitution, attain, mean, remain).
const TABLE_2: [(&str, Structure, f64, f64, f64, f64); 7] = [
    ("RWR", Structure::Rwr, 0.0, 0.30, 1.091, 0.015),
    ("2DK", Structure::Torus, 0.0, 1.00, 2.557, 0.997),
    ("FRNE", Structure::Frne, 0.0, 1.00, 2.575, 0.995),
    ("FRN", Structure::Frn, 0.0, 1.00, 2.480, 0.942),
    ("FFR 0.1", Structure::Frn, 0.1, 1.00, 2.385, 0.844),
    ("FFR 0.3", Structure::Frn, 0.3, 1.00, 2.100, 0.402),
    ("FFR 0.5", Structure::Frn, 0.5, 0.93, 1.257, 0.061),
];

fn row(i: usize) -> Arc<Vec<Run>> {
    let (_, structure, x, ..) = TABLE_2[i];
    runs(SEEDS, move |c| {
        c.structure = structure;
        c.substitution = x;
    })
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

/// The mean over runs of each run's late mean payoff.
fn late(runs: &[Run]) -> f64 {
    mean(&runs.iter().map(Run::late_mean).collect::<Vec<_>>())
}

/// Remain High over the runs that attained high (NaN if none did).
fn remain(runs: &[Run], high: f64) -> f64 {
    mean(
        &runs
            .iter()
            .filter_map(|r| r.remain(high))
            .collect::<Vec<_>>(),
    )
}

/// The mean payoff of row `i` within 0.1 of Table 2.
fn mean_claim(i: usize) -> Outcome {
    let (name, _, _, _, paper, _) = TABLE_2[i];
    let m = late(&row(i));
    outcome(
        (m - paper).abs() <= 0.1,
        format!("{name}: {m:.3} over periods 1501–2500 (the paper {paper:.3}; within 0.1)"),
    )
}

/// Remain High of row `i` at the 2.3 threshold, within 0.05 of Table 2.
fn remain_claim(i: usize) -> Outcome {
    let (name, _, _, _, _, paper) = TABLE_2[i];
    let r = remain(&row(i), 2.3);
    outcome(
        (r - paper).abs() <= 0.05,
        format!("{name}: {r:.3} at the 2.3 threshold (the paper {paper:.3}; within 0.05)"),
    )
}

pub fn claims() -> Vec<Claim> {
    macro_rules! table_rows {
        ($($i:literal => $mean_id:literal, $remain_id:literal, $item:literal;)*) => {
            vec![$(
                Claim {
                    id: $mean_id,
                    item: $item,
                    source: Source::Book,
                    citation: CRA,
                    text: "Table 2: the mean payoff per move over the last 1000 of 2500 periods (30 runs; within 0.1)",
                    check: |_| mean_claim($i),
                },
                Claim {
                    id: $remain_id,
                    item: $item,
                    source: Source::Book,
                    citation: CRA,
                    text: "Table 2: 'Remain High', the share of time at high cooperation once reached (the unstated threshold read as 2.3; within 0.05)",
                    check: |_| remain_claim($i),
                },
            )*]
        };
    }
    let mut out = table_rows! {
        0 => "structure.table-2.rwr.mean", "structure.table-2.rwr.remain", "cra-rwr";
        1 => "structure.table-2.2dk.mean", "structure.table-2.2dk.remain", "cra-2dk";
        2 => "structure.table-2.frne.mean", "structure.table-2.frne.remain", "cra-frne";
        3 => "structure.table-2.frn.mean", "structure.table-2.frn.remain", "cra-frn";
        4 => "structure.table-2.ffr-01.mean", "structure.table-2.ffr-01.remain", "cra-ffr-01";
        5 => "structure.table-2.ffr-03.mean", "structure.table-2.ffr-03.remain", "cra-ffr-03";
        6 => "structure.table-2.ffr-05.mean", "structure.table-2.ffr-05.remain", "cra-ffr-05";
    };
    out.extend(vec![
        Claim {
            id: "structure.table-2.attain",
            item: "cra-table-2",
            source: Source::Book,
            citation: CRA,
            text: "Table 2: 'Attain High C', the share of runs ever reaching high cooperation (each row within two binomial standard deviations of the paper's, at least 0.07)",
            check: |_| {
                let mut ok = true;
                let mut parts = Vec::new();
                for (i, &(name, _, _, paper, _, _)) in TABLE_2.iter().enumerate() {
                    let rs = row(i);
                    let share = rs.iter().filter(|r| r.attained(2.3).is_some()).count() as f64 / rs.len() as f64;
                    let tol = (2.0 * (paper * (1.0 - paper) / rs.len() as f64).sqrt()).max(0.07);
                    ok &= (share - paper).abs() <= tol;
                    parts.push(format!("{name} {share:.2} ({paper:.2})"));
                }
                outcome(ok, parts.join(", "))
            },
        },
        Claim {
            id: "structure.table-2.threshold",
            item: "cra-threshold",
            source: Source::Book,
            citation: CRA,
            text: "the paper never states its 'high cooperation' threshold; 2.3 fits every row's Remain High better than 2.2 or 2.4 (largest miss over the seven rows)",
            check: |_| {
                let miss = |high: f64| {
                    (0..7)
                        .map(|i| (remain(&row(i), high) - TABLE_2[i].5).abs())
                        .fold(0.0, f64::max)
                };
                let (a, b, c) = (miss(2.2), miss(2.3), miss(2.4));
                outcome(
                    b < a && b < c,
                    format!("largest miss {a:.3} at 2.2, {b:.3} at 2.3, {c:.3} at 2.4"),
                )
            },
        },
        Claim {
            id: "structure.table-2.dial",
            item: "cra-dial",
            source: Source::Book,
            citation: CRA,
            text: "'Around a parameter value of 0.3 the dynamics shift … at levels of 0.5 and above, it collapses' (mean payoffs fall RWR < FFR-0.5 < FFR-0.3 < FFR-0.1 < FRN, and FFR-0.5 within 0.3 of RWR)",
            check: |_| {
                let m: Vec<f64> = [0, 6, 5, 4, 3].iter().map(|&i| late(&row(i))).collect();
                outcome(
                    m.windows(2).all(|w| w[0] < w[1]) && m[1] - m[0] <= 0.3,
                    format!("RWR {:.3}, FFR-0.5 {:.3}, FFR-0.3 {:.3}, FFR-0.1 {:.3}, FRN {:.3}", m[0], m[1], m[2], m[3], m[4]),
                )
            },
        },
        Claim {
            id: "structure.ffr-03.bistable",
            item: "cra-ffr-03",
            source: Source::Book,
            citation: CRA,
            text: "FFR-0.3 shows 'a bi-stable condition in which long stretches at high-p alternate with long stretches at low-p' (most runs spend 50 periods or more both high and low)",
            check: |_| {
                let rs = row(5);
                let k = rs.iter().filter(|r| r.bistable(2.3, 50)).count();
                outcome(2 * k > rs.len(), format!("{k}/{} runs", rs.len()))
            },
        },
        Claim {
            id: "structure.fig-1.first-period",
            item: "cra-table-2",
            source: Source::Book,
            citation: CRA,
            text: "Fig. 1: 'each of the four cells … is realized one quarter of the time, making average payoff 2.25' in the first period (RWR, 2DK, FRNE, FRN; 2.2–2.3)",
            check: |_| {
                let firsts: Vec<f64> = (0..4).map(|i| mean(&row(i).iter().map(|r| r.payoff[1]).collect::<Vec<_>>())).collect();
                outcome(
                    firsts.iter().all(|&f| (2.2..=2.3).contains(&f)),
                    format!("{:.3}, {:.3}, {:.3}, {:.3}", firsts[0], firsts[1], firsts[2], firsts[3]),
                )
            },
        },
        Claim {
            id: "structure.fig-1.recovery",
            item: "cra-table-2",
            source: Source::Book,
            citation: CRA,
            text: "Fig. 1: every structure collapses from the start; the context-preserving ones recover within 50 periods 'while the random pairing system never does' (period 50 above the dip by 0.4 for 2DK, FRNE, FRN; RWR within 0.1 of its dip)",
            check: |_| {
                let curve = |i: usize| -> Vec<f64> {
                    let rs = row(i);
                    (1..=50).map(|t| mean(&rs.iter().map(|r| r.payoff[t]).collect::<Vec<_>>())).collect()
                };
                let mut parts = Vec::new();
                let mut ok = true;
                for (i, name) in [(0, "RWR"), (1, "2DK"), (2, "FRNE"), (3, "FRN")] {
                    let c = curve(i);
                    let dip = c.iter().cloned().fold(f64::INFINITY, f64::min);
                    let rise = c[49] - dip;
                    ok &= if i == 0 { rise <= 0.1 } else { rise >= 0.4 };
                    parts.push(format!("{name}: dip {dip:.2}, period 50 {:.2}", c[49]));
                }
                outcome(ok, parts.join("; "))
            },
        },
        Claim {
            id: "structure.region.direction",
            item: "cra-frn",
            source: Source::Book,
            citation: CRA,
            text: "§3.3: with population p in [0.30, 0.35] and q in [0.05, 0.10], p moves by −0.016 under RWR and +0.052 under FRN (signs as stated; each within 0.02)",
            check: |_| {
                let d = |i: usize| {
                    let all: Vec<f64> = row(i).iter().flat_map(|r| r.deltas.clone()).collect();
                    (mean(&all), all.len())
                };
                let ((r, rn), (f, fn_)) = (d(0), d(3));
                outcome(
                    r < 0.0 && f > 0.0 && (r + 0.016).abs() <= 0.02 && (f - 0.052).abs() <= 0.02,
                    format!("RWR {r:+.4} over {rn} visits, FRN {f:+.4} over {fn_} visits"),
                )
            },
        },
        Claim {
            id: "structure.fig-5-6.slope",
            item: "cra-frn",
            source: Source::Book,
            citation: CRA,
            text: "Figs. 5–6: in that region an agent's partners' mean p regressed on its own p is 'not significant for RWR' and 'highly significant for FRN (slope = 0.1580; F = 717.20; N = 5376)' (FRN slope within 0.05, F > 100; RWR F below 3.84)",
            check: |_| {
                let pooled = |i: usize| {
                    let pairs: Vec<(f64, f64)> = row(i).iter().flat_map(|r| r.pairs.clone()).collect();
                    let (s, f) = regression(&pairs);
                    (s, f, pairs.len())
                };
                let ((rs, rf, rn), (fs, ff, fn_)) = (pooled(0), pooled(3));
                outcome(
                    (fs - 0.158).abs() <= 0.05 && ff > 100.0 && rf < 3.84,
                    format!("FRN slope {fs:.3}, F {ff:.0}, N {fn_}; RWR slope {rs:.3}, F {rf:.1}, N {rn}"),
                )
            },
        },
        Claim {
            id: "structure.note-5",
            item: "cra-frne",
            source: Source::Book,
            citation: CRA,
            text: "note 5: 'the FRNE populations actually have a better average score than the 2DK populations'",
            check: |_| {
                let per = |i: usize| row(i).iter().map(Run::late_mean).collect::<Vec<_>>();
                greater(&per(2), &per(1), "FRNE", "2DK")
            },
        },
        Claim {
            id: "structure.note-1",
            item: "cra-population",
            source: Source::Book,
            citation: CRA,
            text: "note 1: 'populations up to 4096 agents display very similar aggregate statistics' (RWR and FRN mean payoffs within 0.05 of 256 agents'; 8 runs at 4096)",
            check: |_| {
                let big = |s: Structure| {
                    late(&runs(8, move |c| {
                        c.structure = s;
                        c.agents = 4096;
                    }))
                };
                let (r, f) = (big(Structure::Rwr), big(Structure::Frn));
                let (r0, f0) = (late(&row(0)), late(&row(3)));
                outcome(
                    (r - r0).abs() <= 0.05 && (f - f0).abs() <= 0.05,
                    format!("RWR {r:.3} (256: {r0:.3}), FRN {f:.3} (256: {f0:.3})"),
                )
            },
        },
        Claim {
            id: "structure.table-a1",
            item: "cra-frne",
            source: Source::Book,
            citation: CRA,
            text: "Table A1: agents d links away in FRNE — 4.00, 11.74, 32.38, 74.59, 98.42, 33.23 for d = 1…6 (averaged over 10 graphs; d ≤ 5 within 5 %)",
            check: |_| {
                let paper = [4.00, 11.74, 32.38, 74.59, 98.42, 33.23];
                let mut f = [0.0; 6];
                for seed in 1..=10 {
                    let c = StructureConfig {
                        structure: Structure::Frne,
                        ..StructureConfig::default()
                    };
                    let w = StructureWorld::new(c, seed).expect("a valid config");
                    for (d, v) in fanout(&w.graph().chosen, 6).iter().enumerate() {
                        f[d] += v / 10.0;
                    }
                }
                outcome(
                    (0..5).all(|d| (f[d] - paper[d]).abs() <= 0.05 * paper[d]),
                    format!("{:.2}, {:.2}, {:.2}, {:.2}, {:.2}, {:.2}", f[0], f[1], f[2], f[3], f[4], f[5]),
                )
            },
        },
        Claim {
            id: "structure.readings.start",
            item: "cra-random-start",
            source: Source::Book,
            citation: CRA,
            text: "the paper's two starts ('evenly distributing … throughout the strategy space' and 'initialized randomly') give the same FRN result (mean payoffs within 0.02)",
            check: |_| {
                let r = late(&runs(SEEDS, |c| {
                    c.structure = Structure::Frn;
                    c.start = Start::Random;
                }));
                let g = late(&row(3));
                outcome((r - g).abs() <= 0.02, format!("random {r:.3}, even {g:.3}"))
            },
        },
        Claim {
            id: "structure.readings.noise",
            item: "cra-copy-noise",
            source: Source::Book,
            citation: CRA,
            text: "the paper's two noise rules ('regardless of which … is adopted' and 'errors in the actual copying process') give the same FRN result (mean payoffs within 0.02)",
            check: |_| {
                let copy = late(&runs(SEEDS, |c| {
                    c.structure = Structure::Frn;
                    c.noise_on = NoiseOn::Copy;
                }));
                let always = late(&row(3));
                outcome(
                    (copy - always).abs() <= 0.02,
                    format!("noise only on copying {copy:.3}, on everyone {always:.3} (Table 2: 2.480)"),
                )
            },
        },
    ]);
    out
}
