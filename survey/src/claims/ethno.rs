//! The ethnocentrism model (milestone 14): Hammond & Axelrod 2006 (its
//! text, its appendix and its archived code), Hartshorn, Kaznatcheev &
//! Shultz 2013 and Jansson 2013, each claim in its source's words. A run is
//! summarized by the mean of each series over its last 100 periods (HA06's
//! summary); HKS13's Studies 1 and 3 run their own 50 worlds (seeds 1–50,
//! 1,000 periods). Runs that several claims share are memoized per process.

use std::sync::{Arc, Mutex, OnceLock};

use sugarscape_core::ethno::{
    Discrimination, EthnoConfig, EthnoWorld, KinBasis, Offspring, PairPlay, Start, Strategy, SERIES,
};
use sugarscape_core::model::{ModelConfig, ModelWorld};

use crate::claim::{all_of, equivalent, greater, range, Claim, Outcome, Source, Verdict};
use crate::runner::model_after;

const HA06: &str = "Hammond & Axelrod 2006, J. Conflict Resolution 50";
const HA06_APPENDIX: &str = "Hammond & Axelrod 2006, appendix";
const HA_JAVA: &str = "Hammond & Axelrod's Java code (2003)";
const HKS13: &str = "Hartshorn, Kaznatcheev & Shultz 2013, JASSS 16(3)";
const J13: &str = "Jansson 2013, JASSS 16(3)";
const OURS: &str = "spec 2026-09-25-ethnocentrism-design.md; plan Decision 12";

/// A world's tolerance around a source's mean (Decision 12): two per-world
/// standard deviations of the standard case (its s.e. over ten seeds × √10)
/// — 7 points of ethnocentric share, 3 of cooperation.
const E_TOL: f64 = 0.07;
const C_TOL: f64 = 0.03;

fn ethno(w: &ModelWorld) -> &EthnoWorld {
    match w {
        ModelWorld::Ethno(w) => w,
        _ => unreachable!("an ethnocentrism world"),
    }
}

/// The mean of `name` over the run's last 100 periods, skipping undefined ones.
fn last_100(w: &EthnoWorld, name: &str) -> f64 {
    let s = w.stats.series(name).expect("an ethnocentrism series");
    let v: Vec<f64> = s[s.len() - 100..]
        .iter()
        .copied()
        .filter(|x| x.is_finite())
        .collect();
    v.iter().sum::<f64>() / v.len() as f64
}

/// Each series' last-100 mean, in `SERIES` order.
type Summary = [f64; SERIES.len()];

/// HA06's defaults with `edit` applied, run `ticks` periods from every seed (memoized).
fn summaries(seeds: &[u64], edit: impl FnOnce(&mut EthnoConfig), ticks: u32) -> Arc<Vec<Summary>> {
    type Cache = Mutex<Vec<(String, u32, Vec<u64>, Arc<Vec<Summary>>)>>;
    static CACHE: Cache = Mutex::new(Vec::new());
    let mut c = EthnoConfig::default();
    edit(&mut c);
    c.end = ticks;
    let key = serde_json::to_string(&c).expect("configs serialize");
    if let Some((.., v)) = CACHE
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .iter()
        .find(|(k, t, s, _)| *k == key && *t == ticks && s == seeds)
    {
        return v.clone();
    }
    let v = Arc::new(model_after(&ModelConfig::Ethno(c), seeds, ticks, |w| {
        let w = ethno(w);
        std::array::from_fn(|k| last_100(w, SERIES[k]))
    }));
    CACHE
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .push((key, ticks, seeds.to_vec(), v.clone()));
    v
}

/// Each world's last-100 mean of `name`.
fn last(seeds: &[u64], edit: impl FnOnce(&mut EthnoConfig), ticks: u32, name: &str) -> Vec<f64> {
    let k = SERIES
        .iter()
        .position(|n| *n == name)
        .expect("an ethnocentrism series");
    summaries(seeds, edit, ticks).iter().map(|s| s[k]).collect()
}

/// A Table 1 row: each world's ethnocentric share and cooperation within
/// `E_TOL` and `C_TOL` of HA06's (`e`, `c`); the worse verdict wins.
fn table_row(s: &[u64], edit: impl Fn(&mut EthnoConfig), ticks: u32, e: f64, c: f64) -> Outcome {
    let es = last(s, &edit, ticks, "ethnocentric");
    let cs = last(s, &edit, ticks, "cooperation");
    all_of(vec![
        (
            "ethnocentric".into(),
            range(&es, e - E_TOL, e + E_TOL, false),
        ),
        (
            "cooperation".into(),
            range(&cs, c - C_TOL, c + C_TOL, false),
        ),
    ])
}

/// Each-colour strategies over periods 1,901–2,000: (helps its own colour
/// only, helps its own colour and refuses at least one other).
fn each_color(seeds: &[u64]) -> Vec<(f64, f64)> {
    let c = EthnoConfig {
        discrimination: Discrimination::EachColor,
        ..EthnoConfig::default()
    };
    model_after(&ModelConfig::Ethno(c), seeds, 1900, |w| {
        let mut w = ethno(w).clone();
        let (mut strict, mut loose) = (0.0, 0.0);
        for _ in 0..100 {
            w.step();
            let n = w.population() as f64;
            let all = (1u64 << w.config.colors) - 1;
            strict += w.agents().filter(|a| a.help == 1 << a.tag).count() as f64 / n;
            loose += w
                .agents()
                .filter(|a| a.help & (1 << a.tag) != 0 && a.help != all)
                .count() as f64
                / n;
        }
        (strict / 100.0, loose / 100.0)
    })
}

/// HKS13's chi-square dominance at one cycle (p < .01, Decision 21): the
/// index (E, H, S, T) of a strategy that beats a uniform split (3 df) and
/// the runner-up (1 df).
fn dominant(counts: [f64; 4]) -> Option<usize> {
    let n: f64 = counts.iter().sum();
    if n < 1.0 {
        return None;
    }
    let e = n / 4.0;
    let chi3: f64 = counts.iter().map(|o| (o - e).powi(2) / e).sum();
    let mut order = [0, 1, 2, 3];
    order.sort_by(|&a, &b| counts[b].total_cmp(&counts[a]));
    let (a, b) = (counts[order[0]], counts[order[1]]);
    let chi1 = if a + b > 0.0 {
        (a - b).powi(2) / (a + b)
    } else {
        0.0
    };
    (chi3 > 11.345 && chi1 > 6.635).then_some(order[0])
}

/// One of HKS13's 50 worlds: when ethnocentrics came to dominate, its early
/// pattern, and its final (last-100) S, T, E, H shares and population.
struct Hks {
    onset: Option<usize>,
    /// 0 humanitarian, 1 ethnocentric, 2 strong competition (Decision 21).
    pattern: usize,
    last: [f64; 5],
}

/// HKS13 Studies 1 and 3: seeds 1–50, 1,000 periods, the defaults (memoized).
fn hks() -> &'static [Hks] {
    static RUNS: OnceLock<Vec<Hks>> = OnceLock::new();
    RUNS.get_or_init(|| {
        let seeds: Vec<u64> = (1..=50).collect();
        let c = EthnoConfig {
            end: 1000,
            ..EthnoConfig::default()
        };
        model_after(&ModelConfig::Ethno(c), &seeds, 1000, |w| {
            let w = ethno(w);
            let names = ["ethnocentric", "humanitarian", "selfish", "traitorous"];
            let pop = w.stats.series("population").unwrap();
            let shares: Vec<Vec<f64>> = names.iter().map(|n| w.stats.series(n).unwrap()).collect();
            let dom: Vec<Option<usize>> = (0..pop.len())
                .map(|t| {
                    if pop[t] == 0.0 {
                        return None;
                    }
                    dominant(std::array::from_fn(|k| (shares[k][t] * pop[t]).round()))
                })
                .collect();
            let onset = (1..=901).find(|&t| (t..t + 100).all(|u| dom[u] == Some(0)));
            let (mut run, mut best) = (0, 0);
            for d in &dom[1..=300] {
                run = if *d == Some(1) { run + 1 } else { 0 };
                best = best.max(run);
            }
            let h = dom[1..=300].iter().filter(|d| **d == Some(1)).count();
            let e = dom[1..=300].iter().filter(|d| **d == Some(0)).count();
            let pattern = if best >= 50 && h > e {
                0
            } else if e >= 150 && e > h {
                1
            } else {
                2
            };
            let last = [
                "selfish",
                "traitorous",
                "ethnocentric",
                "humanitarian",
                "population",
            ]
            .map(|n| last_100(w, n));
            Hks {
                onset,
                pattern,
                last,
            }
        })
    })
}

/// Counts of categories among `n` worlds against a source's: each within
/// two binomial standard deviations of the source's count holds, within
/// three is weak.
fn counts_near(names: [&str; 3], ours: [usize; 3], theirs: [usize; 3], n: usize) -> Outcome {
    let z = (0..3)
        .map(|k| {
            let p = theirs[k] as f64 / n as f64;
            let sd = (n as f64 * p * (1.0 - p)).sqrt();
            (ours[k] as f64 - theirs[k] as f64).abs() / sd
        })
        .fold(0.0, f64::max);
    let verdict = if z <= 2.0 {
        Verdict::Holds
    } else if z <= 3.0 {
        Verdict::Weak
    } else {
        Verdict::Fails
    };
    let list = |c: [usize; 3]| {
        (0..3)
            .map(|k| format!("{} {}", names[k], c[k]))
            .collect::<Vec<_>>()
            .join(" / ")
    };
    Outcome {
        verdict,
        measured: format!(
            "{} of {n} (source {}); largest gap {z:.2} binomial s.d.",
            list(ours),
            list(theirs)
        ),
        detail: String::new(),
    }
}

/// HKS13 Study 2: the mean number of agents of each strategy (E, H, S, T)
/// over the last 100 of 2,000 periods, per world, with only `allowed`.
fn study_2(s: &[u64], allowed: &[Strategy]) -> [Vec<f64>; 4] {
    let edit = |c: &mut EthnoConfig| c.allowed = allowed.to_vec();
    let pop = last(s, edit, 2000, "population");
    ["ethnocentric", "humanitarian", "selfish", "traitorous"].map(|n| {
        last(s, edit, 2000, n)
            .iter()
            .zip(&pop)
            .map(|(x, p)| x * p)
            .collect()
    })
}

/// Study 2's order in the subset `letters` (e.g. "EHS"): each strategy
/// outnumbers the next, one judged part per adjacent pair.
fn order(s: &[u64], letters: &str) -> Vec<(String, Outcome)> {
    let index = |l: char| "EHST".find(l).expect("E, H, S or T");
    let allowed: Vec<Strategy> = letters.chars().map(|l| Strategy::FOUR[index(l)]).collect();
    let counts = study_2(s, &allowed);
    let l: Vec<char> = letters.chars().collect();
    l.windows(2)
        .map(|p| {
            let (a, b) = (index(p[0]), index(p[1]));
            let (x, y) = (p[0].to_string(), p[1].to_string());
            (
                format!("{letters} {x} > {y}"),
                greater(&counts[a], &counts[b], &x, &y),
            )
        })
        .collect()
}

/// J13's kin minus ingroup (tag-ethnocentric) share per world, with kin
/// strategies and `colors` markers.
fn kin_gap(s: &[u64], colors: u32, kin_basis: KinBasis) -> Vec<f64> {
    let edit = |c: &mut EthnoConfig| {
        c.kin_strategies = true;
        c.kin_basis = kin_basis;
        c.colors = colors;
    };
    let kin = last(s, edit, 2000, "kin");
    let e = last(s, edit, 2000, "ethnocentric");
    kin.iter().zip(&e).map(|(k, e)| k - e).collect()
}

fn median(v: &[f64]) -> f64 {
    let mut v = v.to_vec();
    v.sort_by(f64::total_cmp);
    v[v.len() / 2]
}

pub fn claims() -> Vec<Claim> {
    vec![
        Claim {
            id: "ha-table-1.a",
            item: "ha-standard",
            source: Source::Book,
            citation: HA06,
            text: "Table 1 a (the standard case, last 100 of 2,000 periods): 76.3% ethnocentric, 74.2% of interactions cooperative",
            check: |s| {
                table_row(s, |_| {}, 2000, 0.763, 0.742).with(
                    "Cooperation runs about 2 points above HA06 in most rows (mean +2.4, seeds 1–10); HA-Java's pooled ratio differs from our window mean by far less.",
                )
            },
        },
        Claim {
            id: "ha-table-1.b",
            item: "ha-cost",
            source: Source::Book,
            citation: HA06,
            text: "Table 1 b (cost of giving help 0.5%): 76.0% ethnocentric, 77.8% cooperative",
            check: |s| table_row(s, |c| c.cost = 0.005, 2000, 0.76, 0.778),
        },
        Claim {
            id: "ha-table-1.c",
            item: "ha-cost-2",
            source: Source::Book,
            citation: HA06,
            text: "Table 1 c (cost of giving help 2%): 61.8% ethnocentric, 56.1% cooperative",
            check: |s| {
                table_row(s, |c| c.cost = 0.02, 2000, 0.618, 0.561)
                    .with("Measured (seeds 1–10): 63.5% ethnocentric but 64.7% cooperative, 8.6 points above HA06.")
            },
        },
        Claim {
            id: "ha-table-1.d",
            item: "ha-colors",
            source: Source::Book,
            citation: HA06,
            text: "Table 1 d (2 colors): 69.4% ethnocentric, 78.1% cooperative",
            check: |s| table_row(s, |c| c.colors = 2, 2000, 0.694, 0.781),
        },
        Claim {
            id: "ha-table-1.e",
            item: "ha-colors",
            source: Source::Book,
            citation: HA06,
            text: "Table 1 e (8 colors): 79.1% ethnocentric, 71.7% cooperative",
            check: |s| table_row(s, |c| c.colors = 8, 2000, 0.791, 0.717),
        },
        Claim {
            id: "ha-table-1.f",
            item: "ha-figure-1",
            source: Source::Book,
            citation: HA06,
            text: "Table 1 f (mutation rate 0.25%): 82.8% ethnocentric, 79.8% cooperative",
            check: |s| table_row(s, |c| c.mutation = 0.0025, 2000, 0.828, 0.798),
        },
        Claim {
            id: "ha-table-1.g",
            item: "ha-mutation",
            source: Source::Book,
            citation: HA06,
            text: "Table 1 g (mutation rate 1%): 67.1% ethnocentric, 69.0% cooperative",
            check: |s| {
                table_row(s, |c| c.mutation = 0.01, 2000, 0.671, 0.69)
                    .with("Measured (seeds 1–10): 63.0% ethnocentric, 4.1 points short.")
            },
        },
        Claim {
            id: "ha-table-1.h",
            item: "ha-immigration",
            source: Source::Book,
            citation: HA06,
            text: "Table 1 h (immigration rate 0.5): 77.5% ethnocentric, 75.5% cooperative",
            check: |s| table_row(s, |c| c.immigration = 0.5, 2000, 0.775, 0.755),
        },
        Claim {
            id: "ha-table-1.i",
            item: "ha-immigration",
            source: Source::Book,
            citation: HA06,
            text: "Table 1 i (immigration rate 2): 74.4% ethnocentric, 71.4% cooperative",
            check: |s| {
                table_row(s, |c| c.immigration = 2.0, 2000, 0.744, 0.714)
                    .with("Measured (seeds 1–10): 70.5% ethnocentric, 3.9 points short; 73.9% cooperative.")
            },
        },
        Claim {
            id: "ha-table-1.j",
            item: "ha-lattice",
            source: Source::Book,
            citation: HA06,
            text: "Table 1 j (lattice 25 × 25): 70.5% ethnocentric, 69.9% cooperative",
            check: |s| {
                table_row(s, |c| c.width = 25, 2000, 0.705, 0.699)
                    .with("Measured (seeds 1–10): 64.4% ethnocentric, 6.1 points short, with a wide spread between worlds.")
            },
        },
        Claim {
            id: "ha-table-1.k",
            item: "ha-lattice",
            source: Source::Book,
            citation: HA06,
            text: "Table 1 k (lattice 100 × 100): 78.2% ethnocentric, 76.0% cooperative",
            check: |s| {
                table_row(s, |c| c.width = 100, 2000, 0.782, 0.76)
                    .with("Measured (seeds 1–10): 76.5% ethnocentric; 79.1% cooperative, 3.1 points high, and a large lattice's worlds barely differ.")
            },
        },
        Claim {
            id: "ha-table-1.l",
            item: "ha-standard",
            source: Source::Book,
            citation: HA06,
            text: "Table 1 l (run length 500, the last 100 periods): 73.9% ethnocentric, 73.4% cooperative",
            check: |s| {
                table_row(s, |_| {}, 500, 0.739, 0.734).with(
                    "Measured (seeds 1–10): 57.3% ethnocentric, 16.6 points short: ethnocentrics take over later than HA06's row says (72.4% by period 1,500).",
                )
            },
        },
        Claim {
            id: "ha-table-1.m",
            item: "ha-standard",
            source: Source::Book,
            citation: HA06,
            text: "Table 1 m (\"run length 2,000\", read as 4,000 since the standard is 2,000; the last 100 periods): 77.3% ethnocentric, 74.4% cooperative",
            check: |s| table_row(s, |_| {}, 4000, 0.773, 0.744),
        },
        Claim {
            id: "ha-egoist-start.just-as-dominant",
            item: "ha-egoist-start",
            source: Source::Book,
            citation: HA06,
            text: "starting with a full lattice of egoists and no immigration, ethnocentrism becomes just as dominant: its share equals the standard case's within 5 points",
            check: |s| {
                let egoists = last(s, |c| {
                    c.start = Start::Selfish;
                    c.immigration = 0.0;
                }, 2000, "ethnocentric");
                let standard = last(s, |_| {}, 2000, "ethnocentric");
                equivalent(&egoists, &standard, Some(0.05), "egoist start", "standard").with(
                    "Measured (seeds 1–10): 78.6% against 75.9% — if anything more dominant, reached later (7% at period 100, 70% at 500).",
                )
            },
        },
        Claim {
            id: "ha-each-color.eighty",
            item: "ha-each-color",
            source: Source::Book,
            citation: HA06,
            text: "when agents can distinguish all four colors, the result is 80 percent ethnocentric strategies: 75–85% help their own color only",
            check: |s| {
                let v = each_color(s);
                let strict: Vec<f64> = v.iter().map(|x| x.0).collect();
                let loose: Vec<f64> = v.iter().map(|x| x.1).collect();
                range(&strict, 0.75, 0.85, false).with(&format!(
                    "Helping one's own color and refusing at least one other (the loose reading): median {:.3}.",
                    median(&loose)
                ))
            },
        },
        Claim {
            id: "ha-misperception.two-thirds",
            item: "ha-misperception",
            source: Source::Book,
            citation: HA06,
            text: "with a 10 percent chance of misperceiving whether the other agent has the same color, more than two-thirds of agents are ethnocentric",
            check: |s| {
                let v = last(s, |c| c.misperception = 0.1, 2000, "ethnocentric");
                range(&v, 2.0 / 3.0, 1.0, false)
            },
        },
        Claim {
            id: "ha-cost-2.fifty-six",
            item: "ha-cost-2",
            source: Source::Book,
            citation: HA06,
            text: "when the cost of giving help is doubled, cooperation is 56 percent (within 3 points)",
            check: |s| {
                let v = last(s, |c| c.cost = 0.02, 2000, "cooperation");
                range(&v, 0.56 - C_TOL, 0.56 + C_TOL, false)
            },
        },
        Claim {
            id: "ha-cost-2-blind.fourteen",
            item: "ha-cost-2-blind",
            source: Source::Book,
            citation: HA06,
            text: "when agents are unable to distinguish their own color from others, cooperation in the doubled-cost case falls to 14 percent (within 3 points)",
            check: |s| {
                let v = last(s, |c| {
                    c.cost = 0.02;
                    c.discrimination = Discrimination::None;
                }, 2000, "cooperation");
                range(&v, 0.14 - C_TOL, 0.14 + C_TOL, false).with(
                    "No reading of \"unable to distinguish\" reaches 14% (one colour 40.2%, a coin per decision 44.9%, deciding twice 24.5%); only a harsher game does (cost 3%: 12.7%), and then seeing agents cooperate 29.8%, not 56%.",
                )
            },
        },
        Claim {
            id: "ha-cost-2-blind.falls",
            item: "ha-cost-2-blind",
            source: Source::Book,
            citation: HA06,
            text: "at doubled cost, cooperation falls when agents cannot distinguish colors: seeing agents cooperate more than blind ones",
            check: |s| {
                let seeing = last(s, |c| c.cost = 0.02, 2000, "cooperation");
                let blind = last(s, |c| {
                    c.cost = 0.02;
                    c.discrimination = Discrimination::None;
                }, 2000, "cooperation");
                greater(&seeing, &blind, "seeing", "blind")
            },
        },
        Claim {
            id: "ha-appendix-mutation.table-1",
            item: "ha-appendix-mutation",
            source: Source::Book,
            citation: HA06_APPENDIX,
            text: "the appendix's MutationRate = 0.05 is the standard case of Table 1 a (76.3% ethnocentric, 74.2% cooperative)",
            check: |s| {
                table_row(s, |c| c.mutation = 0.05, 2000, 0.763, 0.742).with(
                    "Measured (seeds 1–10): 36.0% ethnocentric, 56.4% cooperative; the text's 0.5% fits, so the appendix's 5% is a slip.",
                )
            },
        },
        Claim {
            id: "ha-appendix-double-play.table-1",
            item: "ha-appendix-double-play",
            source: Source::Book,
            citation: HA06_APPENDIX,
            text: "the appendix's interaction loop (A decides whether to donate to N, then N to A, for each neighbor N of each agent A — every direction decided twice) is the standard case of Table 1 a (76.3% ethnocentric, 74.2% cooperative)",
            check: |s| {
                table_row(s, |c| c.pair_play = PairPlay::Twice, 2000, 0.763, 0.742).with(
                    "Measured (seeds 1–10): 80.9% ethnocentric, 77.3% cooperative — five points above Table 1 a; the code decides once.",
                )
            },
        },
        Claim {
            id: "ha-java-archive.same-outcome",
            item: "ha-java-archive",
            source: Source::App,
            citation: HA_JAVA,
            text: "the archived Java, run as is (five colors, a full random start, no immigration), lands where the paper does: its ethnocentric share equals the standard case's within 5 points",
            check: |s| {
                let archive = last(s, |c| {
                    c.colors = 5;
                    c.start = Start::Random;
                    c.immigration = 0.0;
                }, 2000, "ethnocentric");
                let standard = last(s, |_| {}, 2000, "ethnocentric");
                equivalent(&archive, &standard, Some(0.05), "archive", "standard")
            },
        },
        Claim {
            id: "ha-colors.four-vs-five",
            item: "ha-colors",
            source: Source::App,
            citation: OURS,
            text: "Table 1 cannot tell four colors from the Java's five: at 4 and 5 colors the ethnocentric share is the same within 3 points",
            check: |s| {
                let four = last(s, |_| {}, 2000, "ethnocentric");
                let five = last(s, |c| c.colors = 5, 2000, "ethnocentric");
                equivalent(&four, &five, Some(0.03), "4 colors", "5 colors")
            },
        },
        Claim {
            id: "hks-study-1.shares",
            item: "ha-standard",
            source: Source::Book,
            citation: HKS13,
            text: "final (last 100 of 1,000 cycles) shares .08 selfish, .02 traitorous, .73 ethnocentric, .17 humanitarian, over 50 worlds (each world within two of its standard deviations: 4, 1.4, 10 and 10 points)",
            check: |_| {
                let runs = hks();
                let col = |k: usize| runs.iter().map(|r| r.last[k]).collect::<Vec<f64>>();
                all_of(vec![
                    ("selfish".into(), range(&col(0), 0.04, 0.12, false)),
                    ("traitorous".into(), range(&col(1), 0.006, 0.034, false)),
                    ("ethnocentric".into(), range(&col(2), 0.63, 0.83, false)),
                    ("humanitarian".into(), range(&col(3), 0.07, 0.27, false)),
                ])
            },
        },
        Claim {
            id: "hks-study-1.saturates",
            item: "ha-standard",
            source: Source::Book,
            citation: HKS13,
            text: "the population saturates just under 1,600 (1,500–1,600 over the last 100 of 1,000 cycles)",
            check: |_| {
                let v: Vec<f64> = hks().iter().map(|r| r.last[4]).collect();
                range(&v, 1500.0, 1600.0, false)
            },
        },
        Claim {
            id: "hks-study-3.around-300",
            item: "ha-standard",
            source: Source::Book,
            citation: HKS13,
            text: "ethnocentric dominance is established at around 300 evolutionary cycles: in each world within 200–400 (the first cycle from which ethnocentrics dominate, by HKS13's chi-square tests, for 100 cycles running)",
            check: |_| {
                let v: Vec<f64> = hks().iter().map(|r| r.onset.map_or(f64::NAN, |t| t as f64)).collect();
                range(&v, 200.0, 400.0, false).with(&format!(
                    "The median world settles at cycle {:.0}; worlds range from {:.0} to {:.0}.",
                    median(&v),
                    v.iter().copied().fold(f64::INFINITY, f64::min),
                    v.iter().copied().fold(f64::NEG_INFINITY, f64::max)
                ))
            },
        },
        Claim {
            id: "hks-study-3.early-patterns",
            item: "ha-standard",
            source: Source::Book,
            citation: HKS13,
            text: "of 50 worlds, 16 showed early humanitarian dominance, 16 early ethnocentric dominance and 18 strong early competition (our rule, Decision 21)",
            check: |_| {
                let runs = hks();
                let count = |k| runs.iter().filter(|r| r.pattern == k).count();
                counts_near(
                    ["humanitarian", "ethnocentric", "competition"],
                    [count(0), count(1), count(2)],
                    [16, 16, 18],
                    runs.len(),
                )
            },
        },
        Claim {
            id: "hks-study-2.order",
            item: "hks-no-ethnocentrics",
            source: Source::Book,
            citation: HKS13,
            text: "Table 3: in every subset of strategies but HST, ethnocentric > humanitarian > selfish > traitorous (mean agents over the last 100 of 2,000 cycles)",
            check: |s| {
                let subsets = ["EHST", "EHS", "EHT", "EST", "EH", "ES", "ET", "HS", "HT", "ST"];
                all_of(subsets.iter().flat_map(|l| order(s, l)).collect())
            },
        },
        Claim {
            id: "hks-study-2.hst-reversal",
            item: "hks-no-ethnocentrics",
            source: Source::Book,
            citation: HKS13,
            text: "Table 3: without ethnocentrics (HST), traitorous beats selfish (1,368 humanitarian, 150 traitorous, 115 selfish)",
            check: |s| {
                all_of(order(s, "HTS")).with(
                    "Measured (seeds 1–10): 136 traitorous against 114 selfish (Table 3: 150, 115) — the order holds on average, but worlds overlap.",
                )
            },
        },
        Claim {
            id: "jansson-offspring-anywhere.null-model",
            item: "jansson-offspring-anywhere",
            source: Source::Book,
            citation: J13,
            text: "with offspring placed on a random site the results are similar to the null model: at most 12% of interactions cooperative",
            check: |s| {
                let v = last(s, |c| c.offspring = Offspring::Anywhere, 2000, "cooperation");
                range(&v, 0.0, 0.12, false)
            },
        },
        Claim {
            id: "jansson-tag-mutation.thirty",
            item: "jansson-tag-mutation",
            source: Source::Book,
            citation: J13,
            text: "at a marker mutation rate of 30%, altruists (humanitarians) surpass ethnocentrics",
            check: |s| {
                let h = last(s, |c| c.tag_mutation = Some(0.3), 2000, "humanitarian");
                let e = last(s, |c| c.tag_mutation = Some(0.3), 2000, "ethnocentric");
                greater(&h, &e, "humanitarian", "ethnocentric").with(
                    "Measured (seeds 1–10): 44.9% humanitarian, 39.9% ethnocentric; they cross between 25% and 30% (at 25%: 48.8% ethnocentric, 37.0% humanitarian).",
                )
            },
        },
        Claim {
            id: "jansson-tag-mutation.sixty",
            item: "jansson-tag-mutation",
            source: Source::Book,
            citation: J13,
            text: "at a marker mutation rate of 60%, traitors surpass ethnocentrics",
            check: |s| {
                let t = last(s, |c| c.tag_mutation = Some(0.6), 2000, "traitorous");
                let e = last(s, |c| c.tag_mutation = Some(0.6), 2000, "ethnocentric");
                greater(&t, &e, "traitorous", "ethnocentric").with(
                    "Measured (seeds 1–10): 20.8% traitorous, 21.8% ethnocentric — level at 60%; traitors pass by 75% (29.8 vs 14.7).",
                )
            },
        },
        Claim {
            id: "jansson-tag-mutation.ninety",
            item: "jansson-tag-mutation",
            source: Source::Book,
            citation: J13,
            text: "at a marker mutation rate of 90%, traitors outnumber altruists (humanitarians)",
            check: |s| {
                let t = last(s, |c| c.tag_mutation = Some(0.9), 2000, "traitorous");
                let h = last(s, |c| c.tag_mutation = Some(0.9), 2000, "humanitarian");
                greater(&t, &h, "traitorous", "humanitarian").with(
                    "Measured (seeds 1–10): 39.8% traitorous, 43.6% humanitarian.",
                )
            },
        },
        Claim {
            id: "jansson-standard.table-4",
            item: "ha-standard",
            source: Source::Book,
            citation: J13,
            text: "Table 4: relatives are 74.7% of neighboring pairs, P(same marker | relatives) 95.3%, P(relatives | same marker) 89.2% (each world within two of its standard deviations: 3.8, 2 and 4.4 points)",
            check: |s| {
                let m = |n| last(s, |_| {}, 2000, n);
                all_of(vec![
                    ("relatives".into(), range(&m("relatives"), 0.709, 0.785, false)),
                    ("p(i|r)".into(), range(&m("tag_given_relative"), 0.933, 0.973, false)),
                    ("p(r|i)".into(), range(&m("relative_given_tag"), 0.848, 0.936, false)),
                ])
            },
        },
        Claim {
            id: "jansson-standard.kin-help",
            item: "ha-standard",
            source: Source::Book,
            citation: J13,
            text: "89% of an ethnocentric's donations go to relatives (ours: of all helps; within 4.4 points)",
            check: |s| {
                let v = last(s, |_| {}, 2000, "kin_help");
                range(&v, 0.846, 0.934, false)
            },
        },
        Claim {
            id: "jansson-kin.table-5",
            item: "jansson-kin",
            source: Source::Book,
            citation: J13,
            text: "Table 5: with kin strategies, kin discriminators take 76.2% and ingroup (tag) ethnocentrics 16.4% (each world within two of its standard deviations: 14 points)",
            check: |s| {
                let m = |n| last(s, |c| c.kin_strategies = true, 2000, n);
                all_of(vec![
                    ("kin".into(), range(&m("kin"), 0.622, 0.902, false)),
                    ("ingroup".into(), range(&m("ethnocentric"), 0.024, 0.304, false)),
                ])
                .with("With the basis fixed at immigration (jansson-kin-fixed) kin take 65.5%, ingroup 12.6% (seeds 1–10) — nearer Table 5; J13 does not say how the basis is inherited.")
            },
        },
        Claim {
            id: "jansson-kin-fixed.table-5",
            item: "jansson-kin-fixed",
            source: Source::App,
            citation: OURS,
            text: "Table 5 with the kin basis fixed at immigration (Decision 22): kin 76.2%, ingroup 16.4%, each world within 14 points",
            check: |s| {
                let m = |n| {
                    last(s, |c| {
                        c.kin_strategies = true;
                        c.kin_basis = KinBasis::Fixed;
                    }, 2000, n)
                };
                all_of(vec![
                    ("kin".into(), range(&m("kin"), 0.622, 0.902, false)),
                    ("ingroup".into(), range(&m("ethnocentric"), 0.024, 0.304, false)),
                ])
            },
        },
        Claim {
            id: "jansson-markers.thirty-six",
            item: "jansson-markers",
            source: Source::Book,
            citation: J13,
            text: "the gap between kin and ingroup strategies falls below ten points only at 36 markers: at 20 markers it is still ten points or more",
            check: |s| {
                range(&kin_gap(s, 20, KinBasis::Mutates), 0.1, 1.0, false).with(&format!(
                    "Median gap at 36 markers: {:.3}; with the basis fixed, at 20: {:.3}.",
                    median(&kin_gap(s, 36, KinBasis::Mutates)),
                    median(&kin_gap(s, 20, KinBasis::Fixed))
                ))
            },
        },
    ]
}
