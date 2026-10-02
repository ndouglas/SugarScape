//! Fixed-count, preregistered Banchio–Skrzypacz auction checks.
//! Every learner workload uses seeds 1..N from the approved design, rather
//! than the survey CLI's generic seed count. Sessions are cached by full
//! configuration and seed and appended to a raw JSONL export before judging.

use std::collections::{BTreeMap, HashMap};
use std::io::Write;
use std::sync::{Arc, Mutex, OnceLock};

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sugarscape_core::auctions::{AuctionsConfig, AuctionsWorld};

use crate::claim::{Claim, Outcome, Source, Verdict};
use crate::runner::on_threads;

const PAPER: &str = "Banchio & Skrzypacz 2022, arXiv 2202.05947v1; preregistered rules: docs/superpowers/specs/2026-10-02-q-learning-auctions-design.md";
const READING: &str = "Underdetermined-source reading: this scores the named reconstruction, not a literal disproof of an unpublished threshold, Q vector or time origin.";
const DATA: &str = include_str!("auctions-source-figures.json");

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Run {
    #[serde(default)]
    seed: u64,
    converged: bool,
    #[serde(rename = "terminal_revenue")]
    revenue: f64,
    greedy: Vec<f64>,
    top_profile: bool,
    below_top: bool,
    low_non_nash: bool,
    whole_revenue: f64,
    late_revenue: f64,
    grid: Vec<f64>,
    occupancy: Vec<u64>,
    late_occupancy: Vec<u64>,
    whole_count: u64,
    late_count: u64,
}

type Cache = HashMap<String, Arc<Run>>;
static MEMO: OnceLock<Mutex<Cache>> = OnceLock::new();
static EXPORT: OnceLock<Mutex<std::fs::File>> = OnceLock::new();

fn config(edits: Value) -> AuctionsConfig {
    let mut value = serde_json::to_value(AuctionsConfig::default()).unwrap();
    for (key, val) in edits.as_object().expect("config edits object") {
        value[key] = val.clone();
    }
    serde_json::from_value(value).expect("declared auction configuration")
}

fn runs(edits: Value, n: u64) -> Vec<Run> {
    let c = config(edits);
    let config_json = serde_json::to_string(&c).unwrap();
    let seeds: Vec<_> = (1..=n).collect();
    on_threads(&seeds, |seed| {
        let key = format!("{config_json}|{seed}");
        let cache = MEMO.get_or_init(Default::default);
        if let Some(r) = cache.lock().unwrap().get(&key) {
            return (**r).clone();
        }
        let mut world = AuctionsWorld::new(c.clone(), seed).expect("valid registered config");
        while !world.is_finished() {
            world.run(100_000);
        }
        let raw = serde_json::to_value(world.outcome().expect("finished outcome")).unwrap();
        let mut run: Run = serde_json::from_value(raw.clone()).expect("auction outcome contract");
        run.seed = seed;
        export_session(&c, seed, raw);
        cache.lock().unwrap().insert(key, Arc::new(run.clone()));
        run
    })
}

fn export_session(config: &AuctionsConfig, seed: u64, raw: Value) {
    let export = EXPORT.get_or_init(|| {
        std::fs::create_dir_all("out").expect("survey output directory");
        Mutex::new(
            std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open("out/auctions-sessions.jsonl")
                .expect("raw auction sessions export"),
        )
    });
    let record = json!({"config": config, "seed": seed, "outcome": raw});
    let mut file = export.lock().unwrap();
    serde_json::to_writer(&mut *file, &record).expect("write raw auction session");
    writeln!(file).expect("finish raw auction session");
}

fn same_economic_state(a: &AuctionsWorld, b: &AuctionsWorld) -> bool {
    use sugarscape_core::model::Model;
    let mut oa = serde_json::to_value(a.outcome()).unwrap();
    let mut ob = serde_json::to_value(b.outcome()).unwrap();
    oa.as_object_mut().unwrap().remove("config");
    ob.as_object_mut().unwrap().remove("config");
    oa == ob && a.series_csv() == b.series_csv() && a.agents_csv() == b.agents_csv()
}

fn selected(r: &[Run]) -> Vec<&Run> {
    r.iter().filter(|r| r.converged).collect()
}

fn mean(values: impl Iterator<Item = f64>) -> f64 {
    let values: Vec<_> = values.collect();
    if values.is_empty() {
        return f64::NAN;
    }
    values.iter().sum::<f64>() / values.len() as f64
}

fn estimate(values: impl Iterator<Item = f64>) -> String {
    let values: Vec<_> = values.collect();
    let m = mean(values.iter().copied());
    let se = if values.len() > 1 {
        (values.iter().map(|v| (v - m) * (v - m)).sum::<f64>()
            / ((values.len() - 1) * values.len()) as f64)
            .sqrt()
    } else {
        f64::NAN
    };
    format!(
        "{m:.6} (session SE {se:.6}, normal 95% interval [{:.6},{:.6}])",
        m - 1.96 * se,
        m + 1.96 * se
    )
}

fn revenue(r: &[Run]) -> f64 {
    mean(selected(r).into_iter().map(|r| r.revenue))
}
fn bid(r: &Run) -> f64 {
    mean(r.greedy.iter().copied())
}
fn mean_bid(r: &[Run]) -> f64 {
    mean(selected(r).into_iter().map(bid))
}
fn share(r: &[Run], f: impl Fn(&Run) -> bool) -> f64 {
    let selected = selected(r);
    selected.iter().filter(|r| f(r)).count() as f64 / selected.len() as f64
}

fn wilson_interval(successes: usize, n: usize) -> Option<(f64, f64)> {
    if n == 0 {
        return None;
    }
    let n = n as f64;
    let p = successes as f64 / n;
    let z = 1.96;
    let d = 1.0 + z * z / n;
    let center = (p + z * z / (2.0 * n)) / d;
    let half = z * (p * (1.0 - p) / n + z * z / (4.0 * n * n)).sqrt() / d;
    Some(((center - half).max(0.0), (center + half).min(1.0)))
}

fn share_estimate(r: &[Run], f: impl Fn(&Run) -> bool, conditional: bool) -> String {
    let selected: Vec<_> = r.iter().filter(|r| !conditional || r.converged).collect();
    let n = selected.len();
    let successes = selected.iter().filter(|r| f(r)).count();
    let Some((lo, hi)) = wilson_interval(successes, n) else {
        return "unavailable (no selected sessions)".into();
    };
    let p = successes as f64 / n as f64;
    // Unbiased empirical session variance of Bernoulli observations,
    // divided by the session count, agrees with the existing mean SE.
    let se = if n > 1 {
        format!("{:.6}", (p * (1.0 - p) / (n - 1) as f64).sqrt())
    } else {
        "unavailable".into()
    };
    format!("{p:.6} (session SE {se}, Wilson 95% interval [{lo:.6},{hi:.6}]; sessions {n})")
}

fn share_summary(r: &[Run], f: impl Fn(&Run) -> bool) -> String {
    format!(
        "conditional {} / unconditional {}",
        share_estimate(r, &f, true),
        share_estimate(r, &f, false)
    )
}

/// Conditional arm means can select different seed subsets. The paired
/// influence value I*(value-conditional_mean)/acceptance_rate keeps each
/// arm's registered population while accounting for covariance by seed.
/// With complete convergence this reduces to the SE of session differences.
fn paired_difference(
    a: &[Run],
    b: &[Run],
    metric: impl Fn(&Run) -> f64,
    conditional: bool,
) -> String {
    let by_seed: BTreeMap<_, _> = b.iter().map(|r| (r.seed, r)).collect();
    assert_eq!(by_seed.len(), b.len(), "unique seeds in effect comparison");
    assert_eq!(
        a.len(),
        b.len(),
        "matching requested populations in effect comparison"
    );
    let accepted = |r: &&Run| !conditional || r.converged;
    let left_count = a.iter().filter(accepted).count();
    let right_count = b.iter().filter(accepted).count();
    let left_mean = mean(a.iter().filter(accepted).map(&metric));
    let right_mean = mean(b.iter().filter(accepted).map(&metric));
    let difference = left_mean - right_mean;
    let n = a.len();
    let influence: Vec<_> = a
        .iter()
        .map(|left| {
            let right = by_seed
                .get(&left.seed)
                .expect("matching seeds in effect comparison");
            let left_value = if !conditional || left.converged {
                (metric(left) - left_mean) * n as f64 / left_count as f64
            } else {
                0.0
            };
            let right_value = if !conditional || right.converged {
                (metric(right) - right_mean) * n as f64 / right_count as f64
            } else {
                0.0
            };
            left_value - right_value
        })
        .collect();
    let center = mean(influence.iter().copied());
    let se = if n > 1 && left_count > 0 && right_count > 0 {
        (influence
            .iter()
            .map(|v| (v - center) * (v - center))
            .sum::<f64>()
            / ((n - 1) * n) as f64)
            .sqrt()
    } else {
        f64::NAN
    };
    format!("{difference:.6} (paired-by-seed delta-method SE {se:.6}, normal 95% interval [{:.6},{:.6}]; requested pairs {n}, selected {left_count} vs {right_count})",difference-1.96*se,difference+1.96*se)
}

fn difference_summary(a: &[Run], b: &[Run], metric: impl Fn(&Run) -> f64) -> String {
    format!(
        "conditional {}; unconditional {}",
        paired_difference(a, b, &metric, true),
        paired_difference(a, b, &metric, false)
    )
}

fn population(name: &str, r: &[Run]) -> String {
    let accepted = selected(r);
    let discarded: Vec<_> = r.iter().filter(|r| !r.converged).map(|r| r.seed).collect();
    format!("{name}: requested {}; converged {}; discarded {discarded:?}; conditional {}; unconditional {}; conditional mean greedy bid {}; unconditional mean greedy bid {}; below_top {}; top_profile {}; low_non_nash {}; whole realized {}, late realized {}",
        r.len(), accepted.len(), estimate(accepted.iter().map(|r| r.revenue)),
        estimate(r.iter().map(|r| r.revenue)),estimate(accepted.iter().map(|r|bid(r))),estimate(r.iter().map(bid)),
        share_summary(r,|r|r.below_top),share_summary(r,|r|r.top_profile),share_summary(r,|r|r.low_non_nash),
        estimate(r.iter().map(|r| r.whole_revenue)), estimate(r.iter().map(|r| r.late_revenue)))
}

fn judge(arms: &[&[Run]], holds: bool, measured: String) -> Outcome {
    let sufficient = arms
        .iter()
        .all(|r| !r.is_empty() && selected(r).len() * 100 >= r.len() * 95);
    Outcome {
        verdict: if !sufficient { Verdict::Inconclusive }
            else if holds { Verdict::Holds } else { Verdict::Fails },
        measured,
        detail: "Terminal populations select the final stable window; 95% coverage gate, session-level uncertainty, and fixed thresholds are project rules. Raw sessions: out/auctions-sessions.jsonl. Generic --seeds does not reduce registered workloads.".into(),
    }
}

fn key(bids: &[f64]) -> Vec<i64> {
    let mut key: Vec<_> = bids.iter().map(|v| (v * 1e12).round() as i64).collect();
    key.sort_unstable();
    key
}

fn source_cells(format: &str) -> Vec<(Vec<f64>, u64)> {
    let source: Value = serde_json::from_str(DATA).unwrap();
    source["figure_1"][format!("{format}_nonzero_cells")]
        .as_array()
        .unwrap()
        .iter()
        .map(|cell| {
            (
                serde_json::from_value(cell["bids"].clone()).unwrap(),
                cell["count"].as_u64().unwrap(),
            )
        })
        .collect()
}

fn histogram_gap(r: &[Run], source: &[(Vec<f64>, u64)]) -> (f64, f64) {
    let selected = selected(r);
    let mut observed = BTreeMap::<Vec<i64>, f64>::new();
    for run in &selected {
        *observed.entry(key(&run.greedy)).or_default() += 1.0 / selected.len() as f64;
    }
    let source_count = source.iter().map(|(_, n)| n).sum::<u64>() as f64;
    for (bids, n) in source {
        *observed.entry(key(bids)).or_default() -= *n as f64 / source_count;
    }
    let tv = observed.values().map(|v| v.abs()).sum::<f64>() * 0.5;
    let max = observed.values().map(|v| v.abs()).fold(0.0, f64::max);
    (tv, max)
}

fn played_top(r: &Run, late: bool) -> f64 {
    let occupancy = if late {
        &r.late_occupancy
    } else {
        &r.occupancy
    };
    let total = if late { r.late_count } else { r.whole_count };
    occupancy.last().copied().unwrap_or(0) as f64 / total as f64
}

/// Reserve experiment has two strategic bidders and no fringe, so the
/// played-pair histogram counts every unsold auction exactly.
fn unsold_played(r: &Run, reserve: f64) -> f64 {
    let m = r.grid.len();
    let count = r
        .occupancy
        .iter()
        .enumerate()
        .filter(|(cell, _)| {
            let a = r.grid[*cell / m];
            let b = r.grid[*cell % m];
            (a <= 0.0 || a < reserve) && (b <= 0.0 || b < reserve)
        })
        .map(|(_, n)| n)
        .sum::<u64>();
    count as f64 / r.whole_count as f64
}

fn direction(fpa: &[Run], spa: &[Run]) -> bool {
    revenue(fpa) < 0.5 && revenue(spa) >= 0.9 && revenue(spa) - revenue(fpa) >= 0.4
}

fn scored_pair(edits: Value, n: u64, rule: impl Fn(&[Run], &[Run]) -> bool) -> Outcome {
    let mut edits = edits;
    edits["auction"] = json!("first_price");
    let fpa = runs(edits.clone(), n);
    edits["auction"] = json!("second_price");
    let spa = runs(edits, n);
    judge(
        &[&fpa, &spa],
        rule(&fpa, &spa),
        format!(
            "{}; {}; SPA-minus-FPA terminal revenue {}",
            population("FPA", &fpa),
            population("SPA", &spa),
            difference_summary(&spa, &fpa, |r| r.revenue)
        ),
    )
}

fn check(id: &str) -> Outcome {
    match id {
        "baseline-direction" => scored_pair(json!({}), 1000, direction),
        "local" => {
            let mut result = scored_pair(json!({"exploration_set":"neighbors"}), 100, direction);
            let fpa = runs(json!({}), 100);
            let spa = runs(json!({"auction":"second_price"}), 100);
            let local_fpa = runs(json!({"exploration_set":"neighbors"}), 100);
            let local_spa = runs(
                json!({"exploration_set":"neighbors","auction":"second_price"}),
                100,
            );
            result.measured.push_str(&format!(
                "; global reference {}; {}; local-minus-global terminal revenue FPA {}, SPA {}",
                population("FPA", &fpa),
                population("SPA", &spa),
                difference_summary(&local_fpa, &fpa, |r| r.revenue),
                difference_summary(&local_spa, &spa, |r| r.revenue)
            ));
            result
        }
        "biased-reading" => scored_pair(json!({"q_init":"biased","epsilon":0.25}), 1000, |a, b| {
            revenue(a) < 0.5 && revenue(b) >= 0.9
        })
        .with(READING),
        "downward-reading" => {
            let mut result = scored_pair(json!({"downward_trigger":"stable"}), 1000, |a, b| {
                share(a, |r| r.below_top) >= 0.5 && share(b, |r| r.below_top) >= 0.5
            })
            .with(READING);
            for trigger in ["stable", "period"] {
                for clock in ["activation", "global"] {
                    if trigger == "stable" && clock == "activation" {
                        continue;
                    }
                    for format in ["first_price", "second_price"] {
                        let r = runs(
                            json!({"downward_trigger":trigger,"downward_clock":clock,"auction":format}),
                            1000,
                        );
                        result.measured.push_str(&format!(
                            "; alternative {trigger}/{clock}/{}",
                            population(format, &r)
                        ));
                    }
                }
            }
            result
        }
        "figure-1-fpa" | "figure-1-spa" | "text-fpa-revenue" | "text-spa-revenue" => {
            let spa = id.contains("spa");
            let r = runs(
                json!({"auction":if spa {"second_price"} else {"first_price"}}),
                1000,
            );
            let (target, tolerance) = match id {
                "figure-1-fpa" => (0.2265, 0.02),
                "figure-1-spa" => (0.9471, 0.005),
                "text-fpa-revenue" => (0.24, 0.02),
                _ => (0.95, 0.005),
            };
            let (tv, gap) = histogram_gap(&r, &source_cells(if spa { "spa" } else { "fpa" }));
            let holds = (revenue(&r) - target).abs() <= tolerance
                && (id.starts_with("text") || (tv <= 0.10 && gap <= 0.05));
            judge(&[&r], holds, format!("{}; target {target}, mean gap {:.6}, unordered full-profile TV {tv:.6}, largest bin gap {gap:.6}", population(id, &r), (revenue(&r)-target).abs()))
        }
        "feedback-direction" | "figure-5" => {
            let all = runs(json!({"feedback":"rival_bids","update":"all"}), 500);
            let chosen = runs(json!({}), 500);
            let at_ninety = share(&all, |r| r.greedy.iter().all(|b| (*b - 0.9).abs() < 1e-12));
            let holds = if id == "figure-5" {
                at_ninety >= 0.9 && (revenue(&all) - 0.9).abs() <= 0.02
            } else {
                revenue(&all) >= 0.85 && revenue(&all) - revenue(&chosen) >= 0.3
            };
            let arms: Vec<&[Run]> = if id == "figure-5" {
                vec![&all]
            } else {
                vec![&all, &chosen]
            };
            judge(
                &arms,
                holds,
                format!(
                    "{}; {}; all-at-.90 share {}; all-minus-chosen terminal revenue {}",
                    population("all", &all),
                    population("chosen", &chosen),
                    share_summary(&all, |r| r.greedy.iter().all(|b| (*b - 0.9).abs() < 1e-12)),
                    difference_summary(&all, &chosen, |r| r.revenue)
                ),
            )
        }
        "figure-2-below-top" => {
            let source: Value = serde_json::from_str(DATA).unwrap();
            let mut gaps = Vec::new();
            let mut populations = Vec::new();
            let mut detail = Vec::new();
            for (i, point) in source["figure_2"]["coordinates"]
                .as_array()
                .unwrap()
                .iter()
                .enumerate()
            {
                let alpha = point[0].as_f64().unwrap();
                let target = point[1].as_f64().unwrap() / 100.0;
                let r = runs(json!({"auction":"mixture","auction_alpha":alpha}), 1000);
                let observed = share(&r, |r| r.below_top);
                if i < 10 {
                    gaps.push((observed - target).abs());
                }
                detail.push(format!(
                    "a={alpha}: proxy {}, source {target:.6}; {}",
                    share_summary(&r, |r| r.below_top),
                    population("mixture", &r)
                ));
                populations.push(r);
            }
            let average = mean(gaps.iter().copied());
            let worst = gaps.iter().copied().fold(0.0, f64::max);
            let arms: Vec<_> = populations[..10].iter().map(Vec::as_slice).collect();
            judge(
                &arms,
                average <= 0.05 && worst <= 0.10,
                format!(
                    "mean gap {average:.6}, worst gap {worst:.6}; {}",
                    detail.join("; ")
                ),
            )
            .with(READING)
        }
        "nonparticipation" => {
            let base = runs(json!({}), 1000);
            let fpa = runs(json!({"out_bids":7}), 1000);
            let spa = runs(json!({"out_bids":7,"auction":"second_price"}), 1000);
            judge(&[&base,&fpa,&spa], revenue(&base)-revenue(&fpa)>=0.025,
                format!("{}; {}; {}; baseline-minus-out FPA terminal revenue {}; FPA all-out {}, lowest-positive {}; SPA all-out {}, lowest-positive {}",
                    population("baseline",&base),population("out FPA",&fpa),population("out SPA",&spa),
                    difference_summary(&base,&fpa,|r|r.revenue),
                    share_summary(&fpa, |r| r.greedy.iter().all(|b| *b<=0.0)),share_summary(&fpa, |r| r.greedy.iter().all(|b| (*b-0.05).abs()<1e-12)),
                    share_summary(&spa, |r| r.greedy.iter().all(|b| *b<=0.0)),share_summary(&spa, |r| r.greedy.iter().all(|b| (*b-0.05).abs()<1e-12))))
        }
        "reserve" => {
            let base = runs(json!({}), 1000);
            let reserve = runs(json!({"reserve":0.2}), 1000);
            judge(
                &[&base, &reserve],
                mean_bid(&reserve) - mean_bid(&base) >= 0.025,
                format!(
                    "{}; {}; reserve-minus-baseline mean greedy bid {}; per-bidder mass at reserve {}; all-at-reserve {}; terminal unsold {}; whole played unsold {}",
                    population("baseline", &base),population("reserve", &reserve),
                    difference_summary(&reserve,&base,bid),
                    estimate(selected(&reserve).into_iter().map(|r| r.greedy.iter().filter(|b|(**b-0.2).abs()<1e-12).count() as f64/r.greedy.len() as f64)),
                    share_summary(&reserve, |r| r.greedy.iter().all(|b| (*b-0.2).abs()<1e-12)),
                    share_summary(&reserve, |r| r.greedy.iter().all(|b| *b<0.2)),
                    estimate(reserve.iter().map(|r|unsold_played(r,0.2)))
                ),
            )
        }
        "three-bidders" => {
            let mut arms = Vec::new();
            for format in ["first_price", "second_price"] {
                for discount in [0.99, 0.999] {
                    arms.push(runs(
                        json!({"bidders":3,"auction":format,"discount":discount}),
                        500,
                    ));
                }
            }
            let holds = share(&arms[1], |r| r.below_top) - share(&arms[0], |r| r.below_top) >= 0.10
                && share(&arms[2], |r| r.top_profile) >= 0.95
                && share(&arms[3], |r| r.top_profile) >= 0.95;
            judge(
                &arms.iter().map(Vec::as_slice).collect::<Vec<_>>(),
                holds,
                arms.iter()
                    .zip(["FPA .99", "FPA .999", "SPA .99", "SPA .999"])
                    .map(|(r, n)| population(n, r))
                    .collect::<Vec<_>>()
                    .join("; ")
                    + &format!(
                        "; FPA .999-minus-.99 below_top share {}",
                        difference_summary(&arms[1], &arms[0], |r| f64::from(r.below_top))
                    ),
            )
        }
        "fringe" => {
            let fpa = runs(json!({"fringe":"uniform"}), 1000);
            let spa = runs(json!({"fringe":"uniform","auction":"second_price"}), 1000);
            let mid = share(&fpa, |r| {
                r.greedy
                    .iter()
                    .all(|b| (*b - 0.6).abs() < 1e-12 || (*b - 0.65).abs() < 1e-12)
            });
            judge(
                &[&fpa, &spa],
                mean_bid(&fpa) > 0.50 && mid >= 0.5 && share(&spa, |r| r.top_profile) >= 0.95,
                format!(
                    "{}; {}; FPA all strategic bids in {{.60,.65}} share {}",
                    population("FPA", &fpa),
                    population("SPA", &spa),
                    share_summary(&fpa, |r| r
                        .greedy
                        .iter()
                        .all(|b| (*b - 0.6).abs() < 1e-12 || (*b - 0.65).abs() < 1e-12))
                ),
            )
        }
        "persistent" => {
            let edits = json!({"exploration":"constant","epsilon":0.001,"horizon":100_000_000});
            let mut spa_edits = edits.clone();
            spa_edits["auction"] = json!("second_price");
            let fpa = runs(edits, 1);
            let spa = runs(spa_edits, 1);
            Outcome {verdict:if played_top(&spa[0],false)>=0.8 && fpa[0].whole_revenue<0.5 {Verdict::Holds}else{Verdict::Fails},
                measured: format!("whole played-pair top FPA {:.6}, SPA {:.6}; late FPA {:.6}, SPA {:.6}; whole realized revenue FPA {:.6}, SPA {:.6}; late revenue FPA {:.6}, SPA {:.6}",
                    played_top(&fpa[0],false),played_top(&spa[0],false),played_top(&fpa[0],true),played_top(&spa[0],true),fpa[0].whole_revenue,spa[0].whole_revenue,fpa[0].late_revenue,spa[0].late_revenue),
                detail:"Seed 1, 100 million played auctions each; no terminal stability selection or early stopping. Raw sessions: out/auctions-sessions.jsonl.".into()}
        }
        _ => panic!("unknown registered auction check {id}"),
    }
}

fn record(holds: bool, measured: String, detail: &str) -> Outcome {
    Outcome {
        verdict: if holds {
            Verdict::Holds
        } else {
            Verdict::Fails
        },
        measured,
        detail: detail.into(),
    }
}

fn control(id: &str) -> Outcome {
    match id {
        "source-consistency" => {
            let cells_fpa = source_cells("fpa");
            let cells_spa = source_cells("spa");
            let fpa = cells_fpa
                .iter()
                .map(|(b, n)| b[0].max(b[1]) * *n as f64)
                .sum::<f64>()
                / 1000.0;
            let spa = cells_spa
                .iter()
                .map(|(b, n)| b[0].min(b[1]) * *n as f64)
                .sum::<f64>()
                / 1000.0;
            let source: Value = serde_json::from_str(DATA).unwrap();
            let fpa_count: u64 = cells_fpa.iter().map(|(_, n)| n).sum();
            let spa_count: u64 = cells_spa.iter().map(|(_, n)| n).sum();
            let feedback_count = source["figure_5"]["nonzero_cells"]
                .as_array()
                .unwrap()
                .iter()
                .map(|c| c["count"].as_u64().unwrap())
                .sum::<u64>();
            record(fpa_count == 1000 && spa_count == 1000 && feedback_count == 500
                && (fpa-0.2265).abs()<1e-12 && (spa-0.9471).abs()<1e-12,
                format!("FPA figure {fpa:.6} vs prose .24; SPA figure {spa:.6} vs rounded prose .95; SPA 994 top pairs and six exceptions, including (.65,.95)."),
                "Figure 2 interval bars score a=1..1.9; a=2 closes the final bar. Nearby low/high sentence is reversed. Figure 5 annotates 500 at (.90,.90), footnote 16 agrees; colorbar normalization unresolved. No cause assigned.")
        }
        "stage-game" => {
            use sugarscape_core::auctions::analysis::{equilibria, evaluate};
            let fpa = config(json!({}));
            let spa = config(json!({"auction":"second_price"}));
            let fpa_eq = equilibria(&fpa);
            let spa_eq = equilibria(&spa);
            let has = |eq: &[Vec<f64>], bid: f64| {
                eq.iter()
                    .any(|b| b.iter().all(|v| (*v - bid).abs() < 1e-12))
            };
            let deviation = evaluate(&spa, &[0.9, 0.9]).deviation_gain;
            use sugarscape_core::auctions::mechanism::{clear, grid};
            let weak_payoff = clear(&fpa, &[0.9, 0.9], &[0.0, 0.0], true).rewards[0];
            let weak_deviation = clear(&fpa, &[0.95, 0.9], &[0.0, 0.0], true).rewards[0];
            let top_payoff = clear(&fpa, &[0.95, 0.95], &[0.0, 0.0], true).rewards[0];
            let strict = grid(&fpa).into_iter().filter(|b| *b < 0.95).all(|b| {
                clear(&fpa, &[b, 0.95], &[0.0, 0.0], true).rewards[0] < top_payoff - 1e-12
            });
            record(has(&fpa_eq,0.9) && has(&fpa_eq,0.95) && has(&spa_eq,0.95) && !has(&spa_eq,0.9)
                    && (weak_payoff-0.05).abs()<1e-12 && (weak_deviation-weak_payoff).abs()<1e-12 && strict
                    && deviation.iter().all(|g|(*g-0.05).abs()<1e-12),
                format!("FPA pure equilibria {fpa_eq:?}; SPA {spa_eq:?}; SPA .90-to-.95 gain {deviation:?}"),
                "At .90 FPA, tie payoff .05 equals a .95 deviation payoff .05 (weak equilibrium). At .95 the tie payoff .025 strictly exceeds every alternative's zero. Documentary stage-game check; no learning selection.")
        }
        "unused-feedback" => {
            let seeds: Vec<_> = (1..=100).collect();
            let matches = on_threads(&seeds, |seed| {
                let mut a = AuctionsWorld::new(config(json!({})), seed).unwrap();
                let mut b =
                    AuctionsWorld::new(config(json!({"feedback":"rival_bids"})), seed).unwrap();
                while !a.is_finished() {
                    a.run(100_000);
                    b.run(100_000);
                }
                export_session(&a.config, seed, serde_json::to_value(a.outcome()).unwrap());
                export_session(&b.config, seed, serde_json::to_value(b.outcome()).unwrap());
                same_economic_state(&a, &b)
            });
            let mismatches: Vec<_> = seeds
                .iter()
                .zip(matches)
                .filter_map(|(s, ok)| (!ok).then_some(*s))
                .collect();
            record(mismatches.is_empty(),format!("100 full-horizon paired seeds; mismatches {mismatches:?}"),
                "Compared serialized Q, choices, update counts, outcomes and full period/tick statistics. Config-inclusive fingerprints are intentionally excluded.")
        }
        "initialization" | "ties" | "hindsight" => {
            let mut arms: Vec<(String, Value)> = Vec::new();
            match id {
                "initialization" => {
                    for optimism in ["discounted", "stage"] {
                        arms.push((optimism.into(), json!({"optimism":optimism})));
                    }
                    for level in [2, 10, 100, 1000] {
                        arms.push((
                            format!("constant {level}"),
                            json!({"q_init":"constant","q_level":level}),
                        ));
                    }
                }
                "ties" => {
                    for actual in ["sampled", "expected"] {
                        for greedy in ["lowest", "highest", "random", "incumbent"] {
                            arms.push((
                                format!("{actual}/{greedy}"),
                                json!({"auction_ties":actual,"greedy_ties":greedy}),
                            ));
                        }
                    }
                }
                _ => {
                    for actual in ["sampled", "expected"] {
                        for hindsight in ["expected", "realized"] {
                            arms.push((format!("{actual}/{hindsight}"),json!({"auction_ties":actual,"hindsight_ties":hindsight,"feedback":"rival_bids","update":"all"})));
                        }
                    }
                }
            }
            let primary_edits = if id == "hindsight" {
                json!({"feedback":"rival_bids","update":"all"})
            } else {
                json!({})
            };
            let primary_fpa = runs(primary_edits, 100);
            let primary_spa = if id == "hindsight" {
                vec![]
            } else {
                runs(json!({"auction":"second_price"}), 100)
            };
            let mut measured = Vec::new();
            let mut revenues = Vec::new();
            let mut survives = true;
            for (name, edits) in arms {
                let mut fpa_edits = edits.clone();
                fpa_edits["auction"] = json!("first_price");
                let fpa = runs(fpa_edits, 100);
                let (tv, gap) = histogram_gap(&fpa, &source_cells("fpa"));
                measured.push(format!(
                    "{name}: {}; figure TV {tv:.6}, largest gap {gap:.6}; arm-minus-primary terminal revenue {}",
                    population("FPA", &fpa),difference_summary(&fpa,&primary_fpa,|r|r.revenue)
                ));
                revenues.push(revenue(&fpa));
                if id != "hindsight" {
                    let mut spa_edits = edits;
                    spa_edits["auction"] = json!("second_price");
                    let spa = runs(spa_edits, 100);
                    let (tv, gap) = histogram_gap(&spa, &source_cells("spa"));
                    measured.push(format!(
                        "{name}: {}; figure TV {tv:.6}, largest gap {gap:.6}; arm-minus-primary terminal revenue {}",
                        population("SPA", &spa),difference_summary(&spa,&primary_spa,|r|r.revenue)
                    ));
                    survives &= direction(&fpa, &spa)
                        && selected(&fpa).len() >= 95
                        && selected(&spa).len() >= 95;
                }
            }
            let spread = revenues.iter().copied().fold(f64::NEG_INFINITY, f64::max)
                - revenues.iter().copied().fold(f64::INFINITY, f64::min);
            Outcome {verdict:Verdict::Untestable, measured:format!("{}; FPA largest arm effect (range) {spread:.6}; baseline directional distinction survives every covered format pair: {}",measured.join("; "),if id=="hindsight" {"not applicable (feedback FPA only)".into()} else {survives.to_string()}),
                detail:"Descriptive sensitivity table: no binary acceptance rule preregistered; all declared arms reported, primary reconstruction retained, no arm selected by fit.".into()}
        }
        "duration" | "persistent-seeds" => {
            let horizons: &[u64] = if id == "duration" {
                &[1_000_000, 10_000_000, 100_000_000]
            } else {
                &[10_000_000]
            };
            let mut detail = Vec::new();
            for horizon in horizons {
                for format in ["first_price", "second_price"] {
                    let mut edits = json!({"horizon":horizon,"auction":format});
                    if id == "persistent-seeds" {
                        edits["exploration"] = json!("constant");
                        edits["epsilon"] = json!(0.001);
                    }
                    let r = runs(edits, 20);
                    detail.push(format!("horizon {horizon}: {}; whole top occupancy {}; late top occupancy {}; terminal sorted revenues {:?}",
                        population(format,&r),estimate(r.iter().map(|r|played_top(r,false))),estimate(r.iter().map(|r|played_top(r,true))),{
                            let mut v:Vec<_>=r.iter().map(|r|r.revenue).collect();v.sort_by(f64::total_cmp);v
                        }));
                    if id == "duration" && *horizon > 1_000_000 {
                        let baseline = runs(json!({"auction":format}), 20);
                        detail.push(format!(
                            "{format}, horizon {horizon} minus 1m terminal revenue {}",
                            difference_summary(&r, &baseline, |r| r.revenue)
                        ));
                    }
                }
            }
            Outcome {verdict:Verdict::Untestable,measured:detail.join("; "),
                detail:"Descriptive duration/seed sensitivity study with 20 independent sessions per arm, selected by us; no author ensemble count or preregistered pass threshold. No early stopping.".into()}
        }
        _ => panic!("unknown auction control {id}"),
    }
}

pub fn claims() -> Vec<Claim> {
    macro_rules! source {
        ($id:literal,$text:literal) => {
            Claim {
                id: concat!("auctions.bs.", $id),
                item: "auctions-first-price",
                source: Source::App,
                citation: PAPER,
                text: $text,
                check: |_| check($id),
            }
        };
    }
    macro_rules! control {
        ($id:literal,$text:literal) => {
            Claim {
                id: concat!("auctions.controls.", $id),
                item: "auctions-first-price",
                source: Source::App,
                citation: PAPER,
                text: $text,
                check: |_| control($id),
            }
        };
    }
    vec![
        source!("baseline-direction","FPA terminal revenue <.50, SPA >=.90, difference >=.40 (1000 sessions each)."),
        source!("figure-1-fpa","Recovered FPA histogram TV<=.10, max bin gap<=.05, mean within .02 of .2265."),
        source!("text-fpa-revenue","FPA mean within .02 of separately stated textual revenue .24."),
        source!("figure-1-spa","Recovered SPA histogram TV<=.10, max bin gap<=.05, mean within .005 of .9471."),
        source!("text-spa-revenue","SPA mean within .005 of textual revenue .95."),
        source!("feedback-direction","500 all-action FPA sessions have revenue >=.85 and >=.30 above 500 matched chosen-action sessions."),
        source!("figure-5","At least .90 of converged feedback profiles (.90,.90), revenue within .02 of .90."),
        source!("figure-2-below-top","Below-top proxy over a=1..1.9: mean gap <=.05, worst <=.10 vs recovered percentages; a=2 separately."),
        source!("local","100 sessions each with available-neighbor exploration satisfy baseline direction; global shifts reported."),
        source!("biased-reading","1000 sessions each under named .40/30/0 biased-Q reconstruction: FPA <.50 and SPA >=.90."),
        source!("downward-reading","1000 sessions each, stable trigger/activation clock: >=.50 below-top profiles; alternate readings reported."),
        source!("nonparticipation","1000 sessions each, seven out actions: FPA revenue >=.025 below baseline."),
        source!("reserve","1000 FPA sessions, reserve .20: mean greedy bid >=.025 above baseline."),
        source!("three-bidders","500 sessions per format/discount: FPA below-top share rises >=.10 at .999, SPA >=.95 top at both discounts."),
        source!("fringe","1000 sessions each: FPA mean strategic bid >.50 and >=.50 profiles confined to {.60,.65}, SPA >=.95 top."),
        source!("persistent","Seed 1, 100m periods each: SPA whole played-pair top occupancy >=.80, FPA realized revenue <.50."),
        control!("unused-feedback","100 full-horizon paired seeds produce identical economic state with unused feedback."),
        control!("stage-game","Finite-grid equilibria include weak .90 FPA and strict .95; SPA .90 has profitable .95 deviation."),
        control!("source-consistency","Recovered counts preserve separate figure/text targets, SPA exceptions and documentary discrepancies."),
        control!("initialization","All predeclared initialization readings, 100 sessions per format: report every arm and directional survival."),
        control!("ties","All actual/greedy tie readings, 100 sessions per format: report every arm and directional survival."),
        control!("hindsight","All actual/hindsight tie readings, 100 feedback FPA sessions per arm: report every arm."),
        control!("duration","20 decaying-exploration seeds each at 1m/10m/100m per format; descriptive distributions and coverage."),
        control!("persistent-seeds","20 constant-epsilon .001 seeds per format at 10m; descriptive occupancy/revenue distributions."),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::claim::Verdict;

    fn fixture(converged: usize, n: usize, revenue: f64, bids: &[f64]) -> Vec<Run> {
        (0..n)
            .map(|i| Run {
                seed: (i + 1) as u64,
                converged: i < converged,
                revenue,
                greedy: bids.to_vec(),
                top_profile: false,
                below_top: true,
                low_non_nash: true,
                whole_revenue: revenue,
                late_revenue: revenue,
                grid: vec![0.05, 0.95],
                occupancy: vec![0; 4],
                late_occupancy: vec![0; 4],
                whole_count: 100,
                late_count: 20,
            })
            .collect()
    }

    #[test]
    fn unused_feedback_has_no_learning_effect_for_ten_short_seeds() {
        for seed in 1..=10 {
            let mut a = AuctionsWorld::new(
                config(json!({"horizon":37,"window":5,"periods_per_tick":7})),
                seed,
            )
            .unwrap();
            let mut b = AuctionsWorld::new(
                config(
                    json!({"horizon":37,"window":5,"periods_per_tick":7,"feedback":"rival_bids"}),
                ),
                seed,
            )
            .unwrap();
            a.run(100);
            b.run(100);
            assert!(same_economic_state(&a, &b), "seed {seed}");
        }
    }

    #[test]
    fn documentary_recovered_counts_preserve_the_two_distinct_targets() {
        let result = control("source-consistency");
        assert_eq!(result.verdict, Verdict::Holds);
    }

    #[test]
    fn one_undercovered_arm_makes_a_paired_claim_inconclusive() {
        let a = fixture(100, 100, 0.2, &[0.2, 0.2]);
        let b = fixture(94, 100, 0.95, &[0.95, 0.95]);
        assert_eq!(
            judge(&[&a, &b], true, String::new()).verdict,
            Verdict::Inconclusive
        );
    }

    #[test]
    fn uncertainty_is_over_sessions_instead_of_auction_periods() {
        assert_eq!(
            estimate([0.2, 0.8].into_iter()),
            "0.500000 (session SE 0.300000, normal 95% interval [-0.088000,1.088000])"
        );
    }

    #[test]
    fn sampled_stage_game_control_preserves_weak_equilibrium() {
        assert_eq!(control("stage-game").verdict, Verdict::Holds);
    }

    #[test]
    fn baseline_direction_preserves_strict_low_revenue_and_inclusive_spa_cutoffs() {
        let low = fixture(100, 100, 0.49, &[0.49, 0.49]);
        let edge = fixture(100, 100, 0.50, &[0.50, 0.50]);
        let spa = fixture(100, 100, 0.90, &[0.90, 0.90]);
        let below = fixture(100, 100, 0.899, &[0.899, 0.899]);
        assert!(direction(&low, &spa));
        assert!(!direction(&edge, &spa));
        assert!(!direction(&low, &below));
    }

    #[test]
    fn reserve_unsold_fraction_counts_every_ineligible_played_pair() {
        let mut r = fixture(1, 1, 0.95, &[0.95, 0.95]).remove(0);
        r.occupancy = vec![10, 10, 0, 80];
        assert_eq!(unsold_played(&r, 0.10), 0.10);
    }

    #[test]
    fn bernoulli_share_uncertainty_counts_sessions_and_filters_convergence() {
        let mut r = fixture(4, 4, 0.2, &[0.2, 0.2]);
        r[0].top_profile = true;
        r[1].top_profile = true;
        r[3].converged = false;
        assert_eq!(
            share_estimate(&r, |r| r.top_profile, true),
            "0.666667 (session SE 0.333333, Wilson 95% interval [0.207655,0.938510]; sessions 3)"
        );
        assert_eq!(
            share_estimate(&r, |r| r.top_profile, false),
            "0.500000 (session SE 0.288675, Wilson 95% interval [0.150036,0.849964]; sessions 4)"
        );
    }

    #[test]
    fn paired_difference_uses_seed_matching_and_preserves_covariance() {
        let mut left = fixture(4, 4, 0.2, &[0.2, 0.2]);
        let mut right = fixture(4, 4, 0.2, &[0.2, 0.2]);
        for (r, value) in left.iter_mut().zip([0.4, 0.2, 0.5, 0.6]) {
            r.revenue = value;
        }
        for (r, value) in right.iter_mut().zip([0.2, 0.1, 0.3, 0.4]) {
            r.revenue = value;
        }
        right.reverse();
        assert_eq!(paired_difference(&left,&right,|r|r.revenue,true),
            "0.175000 (paired-by-seed delta-method SE 0.025000, normal 95% interval [0.126000,0.224000]; requested pairs 4, selected 4 vs 4)");
    }

    #[test]
    fn paired_difference_keeps_each_arms_conditional_population() {
        let mut left = fixture(2, 3, 0.2, &[0.2, 0.2]);
        let mut right = fixture(3, 3, 0.2, &[0.2, 0.2]);
        for (r, value) in left.iter_mut().zip([0.2, 0.4, 0.8]) {
            r.revenue = value;
        }
        for (r, value) in right.iter_mut().zip([0.1, 0.2, 0.6]) {
            r.revenue = value;
        }
        assert_eq!(paired_difference(&left,&right,|r|r.revenue,true),
            "0.000000 (paired-by-seed delta-method SE 0.160728, normal 95% interval [-0.315026,0.315026]; requested pairs 3, selected 2 vs 3)");
    }

    #[test]
    fn all_success_proportions_have_a_non_degenerate_bounded_interval() {
        let (lo, hi) = wilson_interval(500, 500).unwrap();
        assert!((lo - 0.992375381469096).abs() < 1e-12);
        assert!((hi - 1.0).abs() < 1e-12);
    }

    #[test]
    fn zero_success_interval_has_a_positive_upper_bound() {
        let (lo, hi) = wilson_interval(0, 500).unwrap();
        assert!(lo >= 0.0);
        assert!((hi - 0.007624618530903).abs() < 1e-12);
    }

    #[test]
    fn empty_and_single_session_proportions_distinguish_missing_standard_error() {
        assert_eq!(
            share_estimate(&[], |r| r.top_profile, true),
            "unavailable (no selected sessions)"
        );
        let mut r = fixture(1, 1, 0.95, &[0.95, 0.95]);
        r[0].top_profile = true;
        assert_eq!(share_estimate(&r,|r|r.top_profile,true),
            "1.000000 (session SE unavailable, Wilson 95% interval [0.206543,1.000000]; sessions 1)");
    }

    #[test]
    fn inadequate_coverage_overrides_an_apparently_perfect_fit() {
        let r = fixture(94, 100, 0.24, &[0.24, 0.24]);
        assert_eq!(
            judge(&[&r], true, "fits".into()).verdict,
            Verdict::Inconclusive
        );
    }

    #[test]
    fn coverage_includes_exactly_ninety_five_percent() {
        let r = fixture(95, 100, 0.24, &[0.24, 0.24]);
        assert_eq!(judge(&[&r], true, "fits".into()).verdict, Verdict::Holds);
    }

    #[test]
    fn summaries_keep_discarded_seeds_and_unconditional_outcomes() {
        let mut r = fixture(1, 2, 0.20, &[0.20, 0.20]);
        r[1].revenue = 0.80;
        let text = population("FPA", &r);
        assert!(text.contains("unconditional 0.500000"), "{text}");
        assert!(text.contains("conditional 0.200000"), "{text}");
        assert!(text.contains("discarded [2]"), "{text}");
    }

    #[test]
    fn histogram_keeps_off_diagonal_mass_and_ignores_bidder_labels() {
        let source = vec![(vec![0.65, 0.95], 1), (vec![0.95, 0.95], 1)];
        let mut r = fixture(2, 2, 0.95, &[0.95, 0.65]);
        r[1].greedy = vec![0.95, 0.95];
        let (tv, gap) = histogram_gap(&r, &source);
        assert_eq!((tv, gap), (0.0, 0.0));
        r[0].greedy = vec![0.65, 0.65];
        assert_eq!(histogram_gap(&r, &source), (0.5, 0.5));
    }

    #[test]
    fn persistent_occupancy_uses_the_whole_run_and_all_played_pairs() {
        let mut r = fixture(1, 1, 0.95, &[0.95, 0.95]).remove(0);
        r.grid = vec![0.05, 0.95];
        r.occupancy = vec![10, 10, 0, 80];
        r.whole_count = 100;
        assert_eq!(played_top(&r, false), 0.8);
    }
}
