//! Chapter VI (vi-1-everything, vi-2-no-trade, vi-3-trade), the N-goods
//! presets (n-3-trade, n-4-peaks, n-2-pollutants) and the built-in sweeps
//! (fig-ii-5, fig-iv-6, fig-iv-10-11, n-goods-carrying-capacity,
//! bargaining-rules).
//!
//! Runs that several claims share are memoized per process (`Memo` for
//! preset runs keyed by the seeds, `OnceLock` for sweeps, which use their
//! own files' seeds and ticks).

use std::collections::BTreeSet;
use std::sync::{Arc, Mutex, OnceLock};

use sugarscape_core::config::{Config, Map, Peak};
use sugarscape_core::econ;
use sugarscape_core::network;
use sugarscape_core::sweep::{self, Metric, Outcome as RunOutcome, Sweep, SweepResult};
use sugarscape_core::world::World;

use crate::claim::{all_of, equivalent, greater, range, untestable, Claim, Outcome, Source};
use crate::runner::{each_seed, preset, series, window_mean};
use crate::stats;

// ---------------------------------------------------------------- helpers

/// The Sugarscape config of a sweep point (every built-in sweep surveyed here
/// is a Sugarscape sweep; `config_for` returns a `ModelConfig` since the
/// Schelling and Ring World models were added).
fn sugarscape_config(s: &Sweep, p: &sweep::Point) -> Config {
    s.config_for(p)
        .expect("a valid config")
        .sugarscape()
        .expect("a Sugarscape sweep")
        .clone()
}

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

fn col<const N: usize>(rows: &[[f64; N]], i: usize) -> Vec<f64> {
    rows.iter().map(|r| r[i]).collect()
}

fn min_of(v: &[f64]) -> f64 {
    v.iter().copied().fold(f64::INFINITY, f64::min)
}

fn max_of(v: &[f64]) -> f64 {
    v.iter().copied().fold(f64::NEG_INFINITY, f64::max)
}

/// Tick of the (first) minimum of `pop[from..=to]`.
fn argmin(pop: &[f64], from: usize, to: usize) -> f64 {
    let mut best = from;
    for t in from..=to {
        if pop[t] < pop[best] {
            best = t;
        }
    }
    best as f64
}

/// The dominant oscillation period of `x`: past the first lag in `lo..=hi`
/// where the autocorrelation (mean removed) turns negative, the lag in
/// `lo..=hi` with the highest autocorrelation. NaN when it never turns
/// negative (no oscillation).
//
// Changed after first run: the first version took the highest
// autocorrelation over all of 20..=300, which for any persistent series is
// the smallest lag, so it returned 20 on every seed (0/20 in range) and
// measured short-lag persistence rather than a period. Skipping to the
// first negative autocorrelation is the standard way to find the first
// periodic peak; the range 20..=300 and the threshold (about 115) are
// unchanged.
fn dominant_period(x: &[f64], lo: usize, hi: usize) -> f64 {
    let m = stats::mean(x);
    let d: Vec<f64> = x.iter().map(|v| v - m).collect();
    let var: f64 = d.iter().map(|v| v * v).sum();
    if var == 0.0 {
        return f64::NAN;
    }
    let r = |lag: usize| (0..d.len() - lag).map(|t| d[t] * d[t + lag]).sum::<f64>() / var;
    let Some(start) = (lo..=hi).find(|&lag| r(lag) < 0.0) else {
        return f64::NAN;
    };
    let mut best = start;
    for lag in start..=hi {
        if r(lag) > r(best) {
            best = lag;
        }
    }
    best as f64
}

// ------------------------------------------------------------- vi-1 runs

/// One `vi-1-everything` run to t = 1000.
struct Vi1 {
    pop: Vec<f64>,
    /// Infected agents per tick (infected_fraction × population).
    infected: Vec<f64>,
    /// How many of births, trades, loans and disease transmissions (an
    /// infection with an infector) occurred at least once over t = 1..=1000.
    rules_active: f64,
    /// How many of the six network overlays (neighbors, friends, family,
    /// trade, credit, disease) were non-empty at least once over t = 151..=200.
    networks_shown: f64,
}

static VI1: Memo<Vec<Vi1>> = Memo::new();

fn vi1(seeds: &[u64]) -> Arc<Vec<Vi1>> {
    VI1.get(seeds, || {
        each_seed(&preset("vi-1-everything"), seeds, |mut w| {
            let (mut births, mut trades, mut loans, mut transmissions) = (false, false, false, false);
            let mut shown = [false; 6];
            for _ in 0..1000 {
                w.step();
                let e = w.events();
                births |= e.births > 0;
                trades |= !e.trades.is_empty();
                loans |= e.loans_made > 0;
                transmissions |= e.infections.iter().any(|i| i.infector.is_some());
                if (151..=200).contains(&w.tick) {
                    let now = [
                        !w.neighbor_edges().is_empty(),
                        !w.friend_edges().is_empty(),
                        !w.family_edges().is_empty(),
                        !network::trade_edges(&w).is_empty(),
                        !network::credit_edges(&w).is_empty(),
                        !network::disease_edges(&w).is_empty(),
                    ];
                    for (s, n) in shown.iter_mut().zip(now) {
                        *s |= n;
                    }
                }
            }
            let pop = series(&w, "population");
            let frac = series(&w, "infected_fraction");
            Vi1 {
                infected: frac.iter().zip(&pop).map(|(f, p)| (f * p).round()).collect(),
                pop,
                rules_active: [births, trades, loans, transmissions].iter().filter(|b| **b).count() as f64,
                networks_shown: shown.iter().filter(|b| **b).count() as f64,
            }
        })
    })
}

// --------------------------------------------------------- vi-2/3 runs

/// (population series t = 0..=1000, total exchanges over t = 1..=1000).
static VI2: Memo<Vec<(Vec<f64>, f64)>> = Memo::new();
static VI3: Memo<Vec<(Vec<f64>, f64)>> = Memo::new();

fn indecomposability(id: &str, seeds: &[u64]) -> Arc<Vec<(Vec<f64>, f64)>> {
    let memo = if id == "vi-2-no-trade" { &VI2 } else { &VI3 };
    memo.get(seeds, || {
        each_seed(&preset(id), seeds, |mut w| {
            w.run(1000);
            let trades: f64 = series(&w, "trade_volume").iter().sum();
            (series(&w, "population"), trades)
        })
    })
}

fn pops(id: &str, s: &[u64]) -> Vec<Vec<f64>> {
    indecomposability(id, s).iter().map(|(p, _)| p.clone()).collect()
}

/// Minimum population over t = 0..=150 (`measure_indecomposability`'s trough).
fn trough(id: &str, s: &[u64]) -> Vec<f64> {
    pops(id, s).iter().map(|p| min_of(&p[..=150])).collect()
}

/// Tick of the minimum population over t = 0..=300.
fn trough_tick(id: &str, s: &[u64]) -> Vec<f64> {
    pops(id, s).iter().map(|p| argmin(p, 0, 300)).collect()
}

/// Peak population over t = 0..=1000 divided by the initial 500.
fn peak_factor(id: &str, s: &[u64]) -> Vec<f64> {
    pops(id, s).iter().map(|p| max_of(p) / 500.0).collect()
}

/// Mean population over t = 300..=1000 (after the recovery).
fn plateau(id: &str, s: &[u64]) -> Vec<f64> {
    pops(id, s).iter().map(|p| window_mean(p, 300, 1000)).collect()
}

// ------------------------------------------------------------- n-3 runs

/// Cross-agent standard deviation of ln MRSᵢⱼ (as in tests/book.rs).
fn ln_mrs_spread(w: &World, i: usize, j: usize) -> f64 {
    let n = w.config.goods.len();
    let logs: Vec<f64> = w
        .agents()
        .filter_map(|a| {
            let m: Vec<f64> = a.metabolism[..n].iter().map(|&x| f64::from(x)).collect();
            let v = econ::mrs_n(&a.holdings[..n], &m, i, j);
            (v.is_finite() && v > 0.0).then(|| v.ln())
        })
        .collect();
    if logs.is_empty() {
        return f64::NAN;
    }
    let mean = stats::mean(&logs);
    (logs.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / logs.len() as f64).sqrt()
}

const PAIRS: [(usize, usize); 3] = [(0, 1), (0, 2), (1, 2)];

/// One n-3-trade run: distinct pairs traded over t = 1..=1000, ln-MRS
/// spread per pair at t = 0 and t = 500, population at t = 500 and 1000.
struct N3 {
    pairs_traded: f64,
    spread0: [f64; 3],
    spread500: [f64; 3],
    pop500: f64,
    pop1000: f64,
}

static N3_RUNS: Memo<Vec<N3>> = Memo::new();

fn n3(seeds: &[u64]) -> Arc<Vec<N3>> {
    N3_RUNS.get(seeds, || {
        each_seed(&preset("n-3-trade"), seeds, |mut w| {
            let spread0 = PAIRS.map(|(i, j)| ln_mrs_spread(&w, i, j));
            let mut pairs = BTreeSet::new();
            let mut spread500 = [f64::NAN; 3];
            let mut pop500 = f64::NAN;
            for _ in 0..1000 {
                w.step();
                pairs.extend(w.events().trades.iter().map(|t| t.goods));
                if w.tick == 500 {
                    spread500 = PAIRS.map(|(i, j)| ln_mrs_spread(&w, i, j));
                    pop500 = w.population() as f64;
                }
            }
            N3 {
                pairs_traded: pairs.len() as f64,
                spread0,
                spread500,
                pop500,
                pop1000: w.population() as f64,
            }
        })
    })
}

// ------------------------------------------------------------- n-2 runs

/// Coefficient of variation of pollutant `k` over sites.
fn pollution_cv(w: &World, k: usize) -> f64 {
    let p: Vec<f64> = w.sites.iter().map(|s| s.pollution[k]).collect();
    let m = stats::mean(&p);
    if m == 0.0 {
        return f64::NAN;
    }
    let sd = (p.iter().map(|x| (x - m).powi(2)).sum::<f64>() / p.len() as f64).sqrt();
    sd / m
}

/// Mean pollutant `k` on occupied sites over mean pollutant `k` on all sites.
fn occupied_pollution_ratio(w: &World, k: usize) -> f64 {
    let all = stats::mean(&w.sites.iter().map(|s| s.pollution[k]).collect::<Vec<_>>());
    let occ: Vec<f64> = w.agents().map(|a| w.site(a.pos).pollution[k]).collect();
    if all == 0.0 || occ.is_empty() {
        return f64::NAN;
    }
    stats::mean(&occ) / all
}

/// Per seed at t = 500: [CV smoke, CV runoff, ratio smoke, ratio runoff].
fn n2_at_500(config: &Config, s: &[u64]) -> Vec<[f64; 4]> {
    each_seed(config, s, |mut w| {
        w.run(500);
        [
            pollution_cv(&w, 0),
            pollution_cv(&w, 1),
            occupied_pollution_ratio(&w, 0),
            occupied_pollution_ratio(&w, 1),
        ]
    })
}

static N2: Memo<Vec<[f64; 4]>> = Memo::new();
static N2_NO_DIFFUSION: Memo<Vec<[f64; 4]>> = Memo::new();
static N2_NO_DEVALUE: Memo<Vec<[f64; 4]>> = Memo::new();

fn n2(s: &[u64]) -> Arc<Vec<[f64; 4]>> {
    N2.get(s, || n2_at_500(&preset("n-2-pollutants"), s))
}

fn n2_no_diffusion(s: &[u64]) -> Arc<Vec<[f64; 4]>> {
    N2_NO_DIFFUSION.get(s, || {
        let mut c = preset("n-2-pollutants");
        c.diffusion.enabled = false;
        n2_at_500(&c, s)
    })
}

fn n2_no_devalue(s: &[u64]) -> Arc<Vec<[f64; 4]>> {
    N2_NO_DEVALUE.get(s, || {
        let mut c = preset("n-2-pollutants");
        for p in &mut c.pollution.pollutants {
            p.devalues.iter_mut().for_each(|d| *d = false);
        }
        n2_at_500(&c, s)
    })
}

// ---------------------------------------------------------------- sweeps

fn jobs() -> usize {
    std::thread::available_parallelism().map_or(4, |n| n.get())
}

fn run_sweep(s: &Sweep) -> SweepResult {
    sweep::run_all(s, jobs(), |_, _| {}).expect("a valid built-in sweep")
}

/// Built-in sweep `id` as its file defines it, run once per process.
fn builtin(id: &'static str) -> &'static SweepResult {
    static CELLS: [(&str, OnceLock<SweepResult>); 5] = [
        ("fig-ii-5", OnceLock::new()),
        ("fig-iv-6", OnceLock::new()),
        ("fig-iv-10-11", OnceLock::new()),
        ("n-goods-carrying-capacity", OnceLock::new()),
        ("bargaining-rules", OnceLock::new()),
    ];
    let cell = &CELLS.iter().find(|(k, _)| *k == id).expect("a known sweep").1;
    cell.get_or_init(|| run_sweep(&sweep::builtin(id).expect("a built-in sweep")))
}

/// Per-seed 50-tick population block means of every cell of built-in sweep
/// `id` (its own configs and seeds) run to `ticks`, as `[series][x][seed]`
/// (for the experiments plan's settlement rule), run once per process.
//
// Changed after first run: this first set the sweep's metric to a timeseries
// and called `run_all`, which panicked ("a timeseries needs a single x
// value"), so every settlement claim was an Error. It now runs each cell's
// `config_for` config itself and applies `sweep::measure` with the same
// timeseries metric.
fn builtin_blocks(id: &'static str, ticks: u32) -> &'static Vec<Vec<Vec<Vec<f64>>>> {
    static CELLS: [(&str, OnceLock<Vec<Vec<Vec<Vec<f64>>>>>); 3] = [
        ("fig-ii-5", OnceLock::new()),
        ("fig-iv-6", OnceLock::new()),
        ("n-goods-carrying-capacity", OnceLock::new()),
    ];
    let cell = &CELLS.iter().find(|(k, _)| *k == id).expect("a known sweep").1;
    cell.get_or_init(|| {
        let s = sweep::builtin(id).expect("a built-in sweep");
        let metric = Metric::Timeseries { series: "population".into(), every: 50 };
        let seeds: Vec<u64> = (s.seeds.from..s.seeds.from + u64::from(s.seeds.count)).collect();
        let points = s.points().expect("valid points");
        (0..s.series_count())
            .map(|series| {
                (0..s.x.values.len())
                    .map(|x| {
                        let p = points.iter().find(|p| p.series == series && p.x == x).unwrap();
                        let c = sugarscape_config(&s, p);
                        each_seed(&c, &seeds, |mut w| {
                            w.run(ticks);
                            match sweep::measure(&metric, ticks, &series_of(&w)) {
                                RunOutcome::Series { values } => values,
                                RunOutcome::Scalar { .. } => unreachable!(),
                            }
                        })
                    })
                    .collect()
            })
            .collect()
    })
}

fn series_of(w: &World) -> Vec<f64> {
    series(w, "population")
}

/// A scalar cell's per-seed values, in seed order.
fn cell(r: &SweepResult, series: usize, x: usize) -> Vec<f64> {
    let mut runs: Vec<_> = r.runs.iter().filter(|p| p.series == series && p.x == x).collect();
    runs.sort_by_key(|p| p.point);
    runs.iter()
        .map(|p| match &p.outcome {
            RunOutcome::Scalar { value } => *value,
            RunOutcome::Series { .. } => panic!("a scalar metric was expected"),
        })
        .collect()
}

/// A timeseries cell's per-seed block values, in seed order.
fn cell_blocks(r: &SweepResult, series: usize, x: usize) -> Vec<Vec<f64>> {
    let mut runs: Vec<_> = r.runs.iter().filter(|p| p.series == series && p.x == x).collect();
    runs.sort_by_key(|p| p.point);
    runs.iter()
        .map(|p| match &p.outcome {
            RunOutcome::Series { values } => values.clone(),
            RunOutcome::Scalar { .. } => panic!("a timeseries metric was expected"),
        })
        .collect()
}

/// Per seed, the 50-tick block mean ending at tick `t`.
fn block_at(blocks: &[Vec<f64>], t: u32) -> Vec<f64> {
    blocks.iter().map(|b| b[(t / 50 - 1) as usize]).collect()
}

/// The experiments plan's settlement rule for one cell at `t`: with B the
/// cell's mean 50-tick block population, max(|B(t−100) − B(t)|,
/// |B(t−50) − B(t)|) / max(3, 0.05·B(t)); the cell is settled when ≤ 1.
fn settle_ratio(blocks: &[Vec<f64>], t: u32) -> f64 {
    let b = |t| stats::mean(&stats::finite(&block_at(blocks, t)));
    let bt = b(t);
    (b(t - 100) - bt).abs().max((b(t - 50) - bt).abs()) / 3f64.max(0.05 * bt)
}

/// Per cell of a block sweep, the worst settlement ratio over `ts`.
fn settle_ratios(cells: &[Vec<Vec<Vec<f64>>>], ts: &[u32]) -> Vec<f64> {
    cells
        .iter()
        .flatten()
        .map(|b| ts.iter().map(|&t| settle_ratio(b, t)).fold(0.0, f64::max))
        .collect()
}

/// "a exceeds b" at every x, a and b being series indices of a scalar sweep.
fn line_above(r: &SweepResult, a: usize, b: usize, a_name: &str, b_name: &str) -> Outcome {
    let xs = r.sweep.x.values.len();
    all_of(
        (0..xs)
            .map(|x| {
                let at = r.sweep.x.values[x].at;
                (format!("x = {at}"), greater(&cell(r, a, x), &cell(r, b, x), a_name, b_name))
            })
            .collect(),
    )
}

/// "each line rises from its first x to its last".
fn lines_rise(r: &SweepResult) -> Outcome {
    let last = r.sweep.x.values.len() - 1;
    all_of(
        (0..r.sweep.series_count())
            .map(|s| {
                (
                    r.sweep.series_name(s),
                    greater(&cell(r, s, last), &cell(r, s, 0), "last x", "first x"),
                )
            })
            .collect(),
    )
}

/// The n-goods sweep's x = `n` config with every good's peak moved onto
/// sugar's (10, 10), and trade set by `trade`.
fn shared_peak(n_index: usize, trade: usize) -> Config {
    let s = sweep::builtin("n-goods-carrying-capacity").unwrap();
    let point = s
        .points()
        .unwrap()
        .into_iter()
        .find(|p| p.x == n_index && p.series == trade)
        .unwrap();
    let mut c = sugarscape_config(&s, &point);
    for g in &mut c.goods {
        g.map = Map::Peaks { peaks: vec![Peak { x: 10, y: 10, radius: 20.0, height: 4.0 }] };
    }
    c
}

/// Mean population over t = 900..=1000 with every good on one shared peak,
/// both trade settings pooled (2 × seeds values), at x index `n_index`.
static SHARED: Memo<Vec<Vec<f64>>> = Memo::new();

fn shared_peak_capacity(n_index: usize, s: &[u64]) -> Vec<f64> {
    SHARED
        .get(s, || {
            (0..5)
                .map(|x| {
                    (0..2)
                        .flat_map(|t| {
                            each_seed(&shared_peak(x, t), s, |mut w| {
                                w.run(1000);
                                window_mean(&series(&w, "population"), 900, 1000)
                            })
                        })
                        .collect()
                })
                .collect()
        })[n_index]
        .clone()
}

const SWEEP_SEEDS: &str = "the sweep file's own seeds 1–10 and ticks, not the survey's seeds";

// ----------------------------------------------------------------- claims

pub fn claims() -> Vec<Claim> {
    vec![
        // ------------------------------------------------ vi-1-everything
        Claim {
            id: "vi-1.every-rule-acts",
            item: "vi-1-everything",
            source: Source::App,
            citation: "presets.rs vi-1-everything description",
            text: "every rule at once: births, trades, loans and disease transmissions (infections with an infector) each occur at least once over t = 1..=1000 (4 of 4 per seed)",
            check: |s| range(&vi1(s).iter().map(|r| r.rules_active).collect::<Vec<_>>(), 4.0, 4.0, false),
        },
        Claim {
            id: "vi-1.flares",
            item: "vi-1-everything",
            source: Source::App,
            citation: "presets.rs vi-1-everything description",
            text: "disease flares after each outbreak (t = 150, 400, 650): within 50 ticks the infected count exceeds the 20 agents seeded, after all 3 outbreaks (share of outbreaks = 1 per seed)",
            check: |s| {
                let v: Vec<f64> = vi1(s)
                    .iter()
                    .map(|r| {
                        let flared = [150usize, 400, 650]
                            .iter()
                            .filter(|&&t| max_of(&r.infected[t + 1..=t + 50]) > 20.0)
                            .count();
                        flared as f64 / 3.0
                    })
                    .collect();
                range(&v, 1.0, 1.0, false)
            },
        },
        Claim {
            id: "vi-1.dies-out",
            item: "vi-1-everything",
            source: Source::App,
            citation: "presets.rs vi-1-everything description",
            text: "disease tends to die out again before the next outbreak: no agent infected at t = 399 and t = 649 for at least half of those two outbreaks (share ≥ 0.5 per seed)",
            check: |s| {
                let v: Vec<f64> = vi1(s)
                    .iter()
                    .map(|r| [399usize, 649].iter().filter(|&&t| r.infected[t] == 0.0).count() as f64 / 2.0)
                    .collect();
                range(&v, 0.5, 1.0, false)
            },
        },
        Claim {
            id: "vi-1.network-views",
            item: "vi-1-everything",
            source: Source::App,
            citation: "presets.rs vi-1-everything description (views 2, 8, 10, 14, 15, 18)",
            text: "the six network overlays have something to show: neighbor, friends, family, trade, credit and disease edges each non-empty at some tick in t = 151..=200 (6 of 6 per seed)",
            check: |s| range(&vi1(s).iter().map(|r| r.networks_shown).collect::<Vec<_>>(), 6.0, 6.0, false),
        },
        Claim {
            id: "vi-1.chart-views",
            item: "vi-1-everything",
            source: Source::App,
            citation: "presets.rs vi-1-everything description",
            text: "the eighteen views live under the named Charts, Agents and Credit tab menus",
            check: |_| untestable("where each view lives in the web page's menus is UI, not observable through sugarscape_core; only the network data behind the overlays is checked (vi-1.network-views)"),
        },
        Claim {
            id: "vi-1.survives",
            item: "vi-1-everything",
            source: Source::Book,
            citation: "tests/book.rs everything_on_society_survives (Chapter VI's everything-on run)",
            text: "the everything-on society survives: population ≥ 50 at t = 1000",
            check: |s| range(&vi1(s).iter().map(|r| r.pop[1000]).collect::<Vec<_>>(), 50.0, f64::INFINITY, false),
        },
        Claim {
            id: "vi-1.measured-t1000",
            item: "vi-1-everything",
            source: Source::Comment,
            citation: "presets.rs vi-1-everything comment: t=1000 population 1832, 1767, 1800, 1790, 1752 (seeds 1-5)",
            text: "population at t = 1000 about 1752–1832",
            check: |s| range(&vi1(s).iter().map(|r| r.pop[1000]).collect::<Vec<_>>(), 1752.0, 1832.0, true),
        },
        Claim {
            id: "vi-1.measured-trough",
            item: "vi-1-everything",
            source: Source::Comment,
            citation: "presets.rs vi-1-everything comment: \"t=0 400 -> a trough within t<=200 of 78-189\"",
            text: "early crash: minimum population over t = 0..=200 about 78–189",
            check: |s| range(&vi1(s).iter().map(|r| min_of(&r.pop[..=200])).collect::<Vec<_>>(), 78.0, 189.0, true),
        },
        // -------------------------------------------------- vi-2-no-trade
        Claim {
            id: "vi-2.never-trades",
            item: "vi-2-no-trade",
            source: Source::App,
            citation: "presets.rs vi-2-no-trade description",
            text: "the agents never trade (total exchanges over t = 1..=1000 = 0)",
            check: |s| range(&indecomposability("vi-2-no-trade", s).iter().map(|r| r.1).collect::<Vec<_>>(), 0.0, 0.0, false),
        },
        Claim {
            id: "vi-2.no-crash",
            item: "vi-2-no-trade",
            source: Source::App,
            citation: "presets.rs vi-2-no-trade description",
            text: "here the population does not crash: population at t = 1000 is at least half its initial 500",
            check: |s| range(&pops("vi-2-no-trade", s).iter().map(|p| p[1000]).collect::<Vec<_>>(), 250.0, f64::INFINITY, false),
        },
        Claim {
            id: "vi-2.dip",
            item: "vi-2-no-trade",
            source: Source::App,
            citation: "presets.rs vi-2-no-trade description",
            text: "it dips to about 150–235 (minimum population over t = 0..=150)",
            check: |s| range(&trough("vi-2-no-trade", s), 150.0, 235.0, true),
        },
        Claim {
            id: "vi-2.dip-timing",
            item: "vi-2-no-trade",
            source: Source::App,
            citation: "presets.rs vi-2-no-trade description",
            text: "the dip comes by t = 100–150 (tick of the minimum population over t = 0..=300, about 100–150)",
            check: |s| range(&trough_tick("vi-2-no-trade", s), 100.0, 150.0, true),
        },
        Claim {
            id: "vi-2.recovers",
            item: "vi-2-no-trade",
            source: Source::App,
            citation: "presets.rs vi-2-no-trade description",
            text: "recovers to about 1.8 times its start (peak population over t = 0..=1000 / 500)",
            check: |s| range(&peak_factor("vi-2-no-trade", s), 1.8, 1.8, true),
        },
        Claim {
            id: "vi-2.around-800",
            item: "vi-2-no-trade",
            source: Source::App,
            citation: "presets.rs vi-2-no-trade description",
            text: "fluctuates around 800 (mean population over t = 300..=1000, about 800)",
            check: |s| range(&plateau("vi-2-no-trade", s), 800.0, 800.0, true),
        },
        Claim {
            id: "vi-2.like-vi-3",
            item: "vi-2-no-trade",
            source: Source::App,
            citation: "presets.rs vi-2-no-trade description",
            text: "like VI-3: mean population over t = 300..=1000 equivalent to vi-3-trade's (margin 10% of the pooled mean)",
            check: |s| equivalent(&plateau("vi-2-no-trade", s), &plateau("vi-3-trade", s), None, "vi-2", "vi-3"),
        },
        Claim {
            id: "vi-2.book-crash",
            item: "vi-2-no-trade",
            source: Source::Book,
            citation: "docs/superpowers/specs/2026-09-24-chapter-vi-design.md quoting Animation VI-2: \"This population crashes\"",
            text: "without trade the population crashes: population at t = 1000 below half its initial 500",
            check: |s| range(&pops("vi-2-no-trade", s).iter().map(|p| p[1000]).collect::<Vec<_>>(), 0.0, 250.0, false),
        },
        Claim {
            id: "vi-2.measured-t1000",
            item: "vi-2-no-trade",
            source: Source::Comment,
            citation: "presets.rs indecomposability comment: vi-2-no-trade t=1000 populations 851, 815, 856, 781, 880 (seeds 1-5)",
            text: "population at t = 1000 about 781–880",
            check: |s| range(&pops("vi-2-no-trade", s).iter().map(|p| p[1000]).collect::<Vec<_>>(), 781.0, 880.0, true),
        },
        // ----------------------------------------------------- vi-3-trade
        Claim {
            id: "vi-3.trades",
            item: "vi-3-trade",
            source: Source::App,
            citation: "presets.rs vi-3-trade description",
            text: "trade is on: at least one exchange over t = 1..=1000",
            check: |s| range(&indecomposability("vi-3-trade", s).iter().map(|r| r.1).collect::<Vec<_>>(), 1.0, f64::INFINITY, false),
        },
        Claim {
            id: "vi-3.dip",
            item: "vi-3-trade",
            source: Source::App,
            citation: "presets.rs vi-3-trade description",
            text: "the population dips to about 100–175 (minimum population over t = 0..=150)",
            check: |s| range(&trough("vi-3-trade", s), 100.0, 175.0, true),
        },
        Claim {
            id: "vi-3.dip-timing",
            item: "vi-3-trade",
            source: Source::App,
            citation: "presets.rs vi-3-trade description",
            text: "the dip comes by t = 100–150 (tick of the minimum population over t = 0..=300, about 100–150)",
            check: |s| range(&trough_tick("vi-3-trade", s), 100.0, 150.0, true),
        },
        Claim {
            id: "vi-3.recovers",
            item: "vi-3-trade",
            source: Source::App,
            citation: "presets.rs vi-3-trade description",
            text: "recovers to 1.7–2.0 times its initial 500 (peak population over t = 0..=1000 / 500)",
            check: |s| range(&peak_factor("vi-3-trade", s), 1.7, 2.0, false),
        },
        Claim {
            id: "vi-3.minima-near-700",
            item: "vi-3-trade",
            source: Source::App,
            citation: "presets.rs vi-3-trade description",
            text: "then fluctuates with minima near 700 (minimum population over t = 300..=1000, about 700)",
            check: |s| range(&pops("vi-3-trade", s).iter().map(|p| min_of(&p[300..=1000])).collect::<Vec<_>>(), 700.0, 700.0, true),
        },
        Claim {
            id: "vi-3.same-as-vi-2",
            item: "vi-3-trade",
            source: Source::App,
            citation: "presets.rs vi-3-trade description: \"VI-2 without trade does the same here\"",
            text: "VI-2 does the same: peak population factor equivalent between vi-3-trade and vi-2-no-trade (margin 10% of the pooled mean)",
            check: |s| equivalent(&peak_factor("vi-3-trade", s), &peak_factor("vi-2-no-trade", s), None, "vi-3", "vi-2"),
        },
        Claim {
            id: "vi-3.book-twice",
            item: "vi-3-trade",
            source: Source::Book,
            citation: "docs/superpowers/specs/2026-09-24-chapter-vi-design.md quoting Animation VI-3: \"to a level more than twice that of the initial population\"",
            text: "the population rises to more than twice its initial 500 (peak over t = 0..=1000 > 1000)",
            check: |s| range(&pops("vi-3-trade", s).iter().map(|p| max_of(p)).collect::<Vec<_>>(), 1000.0 + 1e-9, f64::INFINITY, false),
        },
        Claim {
            id: "vi-3.book-period",
            item: "vi-3-trade",
            source: Source::Book,
            citation: "docs/superpowers/specs/2026-09-24-chapter-vi-design.md quoting Animation VI-3: peaks \"roughly 115 years\" apart",
            text: "sustained oscillations with peaks roughly 115 ticks apart (population over t = 300..=1000: past the first lag in 20..=300 where its autocorrelation turns negative, the lag with the highest autocorrelation, about 115)",
            check: |s| range(&pops("vi-3-trade", s).iter().map(|p| dominant_period(&p[300..=1000], 20, 300)).collect::<Vec<_>>(), 115.0, 115.0, true),
        },
        Claim {
            id: "vi-3.book-trade-raises",
            item: "vi-3-trade",
            source: Source::Book,
            citation: "docs/superpowers/specs/2026-09-24-chapter-vi-design.md: VI-2 \"crashes\"; VI-3 with trade recovers past twice its start",
            text: "trade raises the population: mean over t = 300..=1000 higher in vi-3-trade than vi-2-no-trade",
            check: |s| greater(&plateau("vi-3-trade", s), &plateau("vi-2-no-trade", s), "vi-3", "vi-2"),
        },
        Claim {
            id: "vi-3.measured-t1000",
            item: "vi-3-trade",
            source: Source::Comment,
            citation: "presets.rs indecomposability comment: vi-3-trade t=1000 populations 857, 793, 747, 860, 898 (seeds 1-5)",
            text: "population at t = 1000 about 747–898",
            check: |s| range(&pops("vi-3-trade", s).iter().map(|p| p[1000]).collect::<Vec<_>>(), 747.0, 898.0, true),
        },
        // ------------------------------------------------------ n-3-trade
        Claim {
            id: "n-3.turned-copies",
            item: "n-3-trade",
            source: Source::App,
            citation: "presets.rs n-3-trade description",
            text: "three turned copies of the two-peak map: every good's capacities are a rearrangement of sugar's and no two goods share a map (1 per seed)",
            check: |s| {
                let v = each_seed(&preset("n-3-trade"), s, |w| {
                    let caps: Vec<Vec<f64>> = (0..3).map(|g| w.capacities(g)).collect();
                    let sorted = |c: &Vec<f64>| {
                        let mut c = c.clone();
                        c.sort_by(f64::total_cmp);
                        c
                    };
                    let same = caps.iter().all(|c| sorted(c) == sorted(&caps[0]));
                    let distinct = caps[0] != caps[1] && caps[0] != caps[2] && caps[1] != caps[2];
                    f64::from(u8::from(same && distinct))
                });
                range(&v, 1.0, 1.0, false)
            },
        },
        Claim {
            id: "n-3.all-pairs-trade",
            item: "n-3-trade",
            source: Source::App,
            citation: "presets.rs n-3-trade description",
            text: "prices form for all three pairs: each of (sugar, spice), (sugar, salt), (spice, salt) trades at least once over t = 1..=1000 (3 pairs per seed)",
            check: |s| range(&n3(s).iter().map(|r| r.pairs_traded).collect::<Vec<_>>(), 3.0, 3.0, false),
        },
        Claim {
            id: "n-3.widest-gap",
            item: "n-3-trade",
            source: Source::App,
            citation: "presets.rs n-3-trade description",
            text: "neighbors barter over whichever pair they value most differently",
            check: |_| untestable("the pair choice depends on each encounter's pre-trade MRSs, which the public API does not expose (trade events carry only post-choice exchanges); the rule is covered by core unit tests"),
        },
        Claim {
            id: "n-3.prices-converge",
            item: "n-3-trade",
            source: Source::Book,
            citation: "tests/book.rs three_good_prices_converge_for_every_pair (Figure IV-3's convergence for all three pairs)",
            text: "prices converge for every pair: the worst pair's ratio of cross-agent sd(ln MRS) at t = 500 to t = 0 is below 1",
            check: |s| {
                let v: Vec<f64> = n3(s)
                    .iter()
                    .map(|r| (0..3).map(|k| r.spread500[k] / r.spread0[k]).fold(f64::NEG_INFINITY, f64::max))
                    .collect();
                range(&v, 0.0, 1.0 - 1e-9, false)
            },
        },
        Claim {
            id: "n-3.trade-raises",
            item: "n-3-trade",
            source: Source::Book,
            citation: "tests/book.rs three_good_trade_raises_carrying_capacity (Figure IV-6 with three goods); docs/superpowers/specs/2026-09-23-n-goods-design.md",
            text: "with three goods, more agents survive with trade: population at t = 500 higher than the same preset with trade off",
            check: |s| {
                let with: Vec<f64> = n3(s).iter().map(|r| r.pop500).collect();
                let mut c = preset("n-3-trade");
                c.trade.enabled = false;
                let without = each_seed(&c, s, |mut w| {
                    w.run(500);
                    w.population() as f64
                });
                greater(&with, &without, "trade", "no trade")
            },
        },
        Claim {
            id: "n-3.measured-t1000",
            item: "n-3-trade",
            source: Source::Comment,
            citation: "presets.rs n-3-trade comment: (1,3)/(15,40) t=1000 populations [858, 820, 838, 791, 760]",
            text: "population at t = 1000 about 760–858",
            check: |s| range(&n3(s).iter().map(|r| r.pop1000).collect::<Vec<_>>(), 760.0, 858.0, true),
        },
        // ------------------------------------------------------ n-4-peaks
        Claim {
            id: "n-4.corners",
            item: "n-4-peaks",
            source: Source::App,
            citation: "presets.rs n-4-peaks description",
            text: "four goods, each on one peak near a different corner: the four goods' highest-capacity sites lie in four different quadrants (1 per seed)",
            check: |s| {
                let v = each_seed(&preset("n-4-peaks"), s, |w| {
                    let (half_w, half_h) = (w.config.width / 2, w.config.height / 2);
                    let quadrants: BTreeSet<(bool, bool)> = (0..4)
                        .map(|g| {
                            let c = w.capacities(g);
                            let i = (0..c.len()).fold(0, |b, i| if c[i] > c[b] { i } else { b });
                            let p = w.torus.pos(i);
                            (p.x < half_w, p.y < half_h)
                        })
                        .collect();
                    f64::from(u8::from(quadrants.len() == 4))
                });
                range(&v, 1.0, 1.0, false)
            },
        },
        Claim {
            id: "n-4.no-site-has-all",
            item: "n-4-peaks",
            source: Source::App,
            citation: "presets.rs n-4-peaks description",
            text: "agents must travel or trade to hold all four: no site has positive capacity for all four goods (count of such sites = 0)",
            check: |s| {
                let v = each_seed(&preset("n-4-peaks"), s, |w| {
                    w.sites.iter().filter(|site| (0..4).all(|g| site.capacity[g] > 0.0)).count() as f64
                });
                range(&v, 0.0, 0.0, false)
            },
        },
        Claim {
            id: "n-4.trades",
            item: "n-4-peaks",
            source: Source::App,
            citation: "presets.rs n-4-peaks description",
            text: "agents trade: at least one exchange over t = 1..=1000",
            check: |s| {
                let v = each_seed(&preset("n-4-peaks"), s, |mut w| {
                    w.run(1000);
                    series(&w, "trade_volume").iter().sum::<f64>()
                });
                range(&v, 1.0, f64::INFINITY, false)
            },
        },
        Claim {
            id: "n-4.measured-t1000",
            item: "n-4-peaks",
            source: Source::Comment,
            citation: "presets.rs n-4-peaks comment: (1,2)/(25,50) t=1000 populations [83, 95, 92, 94, 88]",
            text: "population at t = 1000 about 83–95",
            check: |s| {
                let v = each_seed(&preset("n-4-peaks"), s, |mut w| {
                    w.run(1000);
                    w.population() as f64
                });
                range(&v, 83.0, 95.0, true)
            },
        },
        // -------------------------------------------------- n-2-pollutants
        Claim {
            id: "n-2.both-pollute",
            item: "n-2-pollutants",
            source: Source::App,
            citation: "presets.rs n-2-pollutants description",
            text: "sugar gives off smoke and spice gives off runoff: both mean pollution levels positive at t = 1000 (smaller of the two > 0)",
            check: |s| {
                let v = each_seed(&preset("n-2-pollutants"), s, |mut w| {
                    w.run(1000);
                    series(&w, "mean_pollution_0")[1000].min(series(&w, "mean_pollution_1")[1000])
                });
                range(&v, 1e-9, f64::INFINITY, false)
            },
        },
        Claim {
            id: "n-2.smoke-repels",
            item: "n-2-pollutants",
            source: Source::App,
            citation: "presets.rs n-2-pollutants description",
            text: "smoke makes sugar sites less attractive: at t = 500 the ratio (mean smoke on occupied sites / mean smoke on all sites) is lower than with no pollutant devaluing any good",
            check: |s| greater(&col(&n2_no_devalue(s), 2), &col(&n2(s), 2), "no devaluing", "preset"),
        },
        Claim {
            id: "n-2.runoff-repels",
            item: "n-2-pollutants",
            source: Source::App,
            citation: "presets.rs n-2-pollutants description",
            text: "runoff spoils spice sites: at t = 500 the ratio (mean runoff on occupied sites / mean runoff on all sites) is lower than with no pollutant devaluing any good",
            check: |s| greater(&col(&n2_no_devalue(s), 3), &col(&n2(s), 3), "no devaluing", "preset"),
        },
        Claim {
            id: "n-2.smoke-diffuses",
            item: "n-2-pollutants",
            source: Source::App,
            citation: "presets.rs n-2-pollutants description",
            text: "both diffuse (smoke): coefficient of variation of site smoke at t = 500 lower than with diffusion off",
            check: |s| greater(&col(&n2_no_diffusion(s), 0), &col(&n2(s), 0), "no diffusion", "preset"),
        },
        Claim {
            id: "n-2.runoff-diffuses",
            item: "n-2-pollutants",
            source: Source::App,
            citation: "presets.rs n-2-pollutants description",
            text: "both diffuse (runoff): coefficient of variation of site runoff at t = 500 lower than with diffusion off",
            check: |s| greater(&col(&n2_no_diffusion(s), 1), &col(&n2(s), 1), "no diffusion", "preset"),
        },
        Claim {
            id: "n-2.measured-t1000",
            item: "n-2-pollutants",
            source: Source::Comment,
            citation: "presets.rs n-2-pollutants comment: k=1.0 t=1000 populations [82, 86, 85, 86, 85]",
            text: "population at t = 1000 about 82–86",
            check: |s| {
                let v = each_seed(&preset("n-2-pollutants"), s, |mut w| {
                    w.run(1000);
                    w.population() as f64
                });
                range(&v, 82.0, 86.0, true)
            },
        },
        Claim {
            id: "n-2.measured-pollution",
            item: "n-2-pollutants",
            source: Source::Comment,
            citation: "presets.rs n-2-pollutants comment: seed-1 mean pollution [smoke, runoff] at t=1000 [154.96, 161.48]",
            text: "mean of the two mean pollution levels at t = 1000 about 154.96–161.48",
            check: |s| {
                let v = each_seed(&preset("n-2-pollutants"), s, |mut w| {
                    w.run(1000);
                    (series(&w, "mean_pollution_0")[1000] + series(&w, "mean_pollution_1")[1000]) / 2.0
                });
                range(&v, 154.96, 161.48, true)
            },
        },
        // ------------------------------------------------------- fig-ii-5
        // Series 0, 1, 2 = mean metabolism 1, 2, 3; x 0..=5 = mean vision 1..=6.
        Claim {
            id: "fig-ii-5.vision-raises",
            item: "fig-ii-5",
            source: Source::Book,
            citation: "docs/superpowers/specs/2026-09-23-experiments-design.md and tests/book.rs fig_ii_5_carrying_capacity_rises_with_vision_and_falls_with_metabolism",
            text: "carrying capacity rises with vision: on every metabolism line, mean population over t = 200..=300 at mean vision 6 exceeds vision 1 (sweep's own seeds 1–10)",
            check: |_| lines_rise(builtin("fig-ii-5")).with(SWEEP_SEEDS),
        },
        Claim {
            id: "fig-ii-5.metabolism-lowers",
            item: "fig-ii-5",
            source: Source::Book,
            citation: "docs/superpowers/specs/2026-09-23-experiments-design.md and tests/book.rs fig_ii_5_carrying_capacity_rises_with_vision_and_falls_with_metabolism",
            text: "carrying capacity falls with metabolism: at every mean vision, metabolism 1 > 2 and 2 > 3 (sweep's own seeds 1–10)",
            check: |_| {
                let r = builtin("fig-ii-5");
                let mut parts = Vec::new();
                for x in 0..6 {
                    for m in 0..2 {
                        parts.push((
                            format!("vision {} m{} > m{}", x + 1, m + 1, m + 2),
                            greater(&cell(r, m, x), &cell(r, m + 1, x), "lower metabolism", "higher metabolism"),
                        ));
                    }
                }
                all_of(parts).with(SWEEP_SEEDS)
            },
        },
        Claim {
            id: "fig-ii-5.settled",
            item: "fig-ii-5",
            source: Source::App,
            citation: "sweeps/fig-ii-5.json description",
            text: "ticks 300, 500 and 1000 all satisfy the settlement rule for every cell (per cell, worst over T of max(|B(T−100)−B(T)|, |B(T−50)−B(T)|) / max(3, 0.05·B(T)) ≤ 1, B = mean 50-tick block population; the judge counts cells, 18)",
            check: |_| range(&settle_ratios(builtin_blocks("fig-ii-5", 1000), &[300, 500, 1000]), 0.0, 1.0, false).with(SWEEP_SEEDS),
        },
        Claim {
            id: "fig-ii-5.measured-m1-v1",
            item: "fig-ii-5",
            source: Source::Comment,
            citation: "sweeps/fig-ii-5.json description, Measured: metabolism 1 mean population 438.7 at vision 1",
            text: "metabolism 1, vision 1 cell about 438.7",
            check: |_| range(&cell(builtin("fig-ii-5"), 0, 0), 438.7, 438.7, true).with(SWEEP_SEEDS),
        },
        Claim {
            id: "fig-ii-5.measured-m3-v6",
            item: "fig-ii-5",
            source: Source::Comment,
            citation: "sweeps/fig-ii-5.json description, Measured: metabolism 3 mean population 250.1 at vision 6",
            text: "metabolism 3, vision 6 cell about 250.1",
            check: |_| range(&cell(builtin("fig-ii-5"), 2, 5), 250.1, 250.1, true).with(SWEEP_SEEDS),
        },
        // ------------------------------------------------------- fig-iv-6
        // Series 0 = no trade, 1 = trade.
        Claim {
            id: "fig-iv-6.trade-raises",
            item: "fig-iv-6",
            source: Source::Book,
            citation: "docs/superpowers/specs/2026-09-23-experiments-design.md (\"the trade line lies above the no-trade line at every x\") and tests/book.rs fig_iv_6_trade_raises_carrying_capacity_at_every_vision",
            text: "trade raises carrying capacity at every mean vision (sweep's own seeds 1–10)",
            check: |_| line_above(builtin("fig-iv-6"), 1, 0, "trade", "no trade").with(SWEEP_SEEDS),
        },
        Claim {
            id: "fig-iv-6.vision-raises",
            item: "fig-iv-6",
            source: Source::Book,
            citation: "book, from memory: Figure IV-6, carrying capacity rises with mean vision with and without trade",
            text: "on both lines, carrying capacity at mean vision 6 exceeds vision 1 (sweep's own seeds 1–10)",
            check: |_| lines_rise(builtin("fig-iv-6")).with(SWEEP_SEEDS),
        },
        Claim {
            id: "fig-iv-6.settled",
            item: "fig-iv-6",
            source: Source::App,
            citation: "sweeps/fig-iv-6.json description",
            text: "ticks 300, 500 and 1000 all satisfy the settlement rule for every cell (settlement ratio ≤ 1 as in fig-ii-5.settled; the judge counts cells, 12)",
            check: |_| range(&settle_ratios(builtin_blocks("fig-iv-6", 1000), &[300, 500, 1000]), 0.0, 1.0, false).with(SWEEP_SEEDS),
        },
        Claim {
            id: "fig-iv-6.measured-no-trade-v1",
            item: "fig-iv-6",
            source: Source::Comment,
            citation: "sweeps/fig-iv-6.json description, Measured: no trade mean population 33.8 at vision 1",
            text: "no-trade, vision 1 cell about 33.8",
            check: |_| range(&cell(builtin("fig-iv-6"), 0, 0), 33.8, 33.8, true).with(SWEEP_SEEDS),
        },
        Claim {
            id: "fig-iv-6.measured-trade-v6",
            item: "fig-iv-6",
            source: Source::Comment,
            citation: "sweeps/fig-iv-6.json description, Measured: trade mean population 76.6 at vision 6",
            text: "trade, vision 6 cell about 76.6",
            check: |_| range(&cell(builtin("fig-iv-6"), 1, 5), 76.6, 76.6, true).with(SWEEP_SEEDS),
        },
        // --------------------------------------------------- fig-iv-10-11
        // Series 0 = lifetimes 60–100, 1 = 960–1000; 20 blocks of 50 ticks.
        Claim {
            id: "fig-iv-10-11.long-lives-less-dispersion",
            item: "fig-iv-10-11",
            source: Source::Book,
            citation: "docs/superpowers/specs/2026-09-23-experiments-design.md and tests/book.rs fig_iv_10_11_long_lives_end_with_less_price_dispersion",
            text: "the long-lifetime series ends with lower sd(ln price) than the short one (last block, t = 951..=1000; sweep's own seeds 1–10)",
            check: |_| {
                let r = builtin("fig-iv-10-11");
                let last = |s| block_at(&cell_blocks(r, s, 0), 1000);
                greater(&last(0), &last(1), "lifetimes 60–100", "lifetimes 960–1000").with(SWEEP_SEEDS)
            },
        },
        Claim {
            id: "fig-iv-10-11.long-lives-converge",
            item: "fig-iv-10-11",
            source: Source::Book,
            citation: "book, from memory: Figure IV-11, with lifetimes 960–1000 the price dispersion falls over time",
            text: "with lifetimes 960–1000, sd(ln price) in the last block is lower than in the first (t = 1..=50; sweep's own seeds 1–10)",
            check: |_| {
                let b = cell_blocks(builtin("fig-iv-10-11"), 1, 0);
                greater(&block_at(&b, 50), &block_at(&b, 1000), "first block", "last block").with(SWEEP_SEEDS)
            },
        },
        Claim {
            id: "fig-iv-10-11.short-lives-persist",
            item: "fig-iv-10-11",
            source: Source::Book,
            citation: "book, from memory: Figure IV-10, with lifetimes 60–100 price dispersion persists instead of converging",
            text: "with lifetimes 60–100, the last block's sd(ln price) is at least half the first block's (ratio ≥ 0.5; sweep's own seeds 1–10)",
            check: |_| {
                let b = cell_blocks(builtin("fig-iv-10-11"), 0, 0);
                let v: Vec<f64> = block_at(&b, 1000).iter().zip(block_at(&b, 50)).map(|(l, f)| l / f).collect();
                range(&v, 0.5, f64::INFINITY, false).with(SWEEP_SEEDS)
            },
        },
        Claim {
            id: "fig-iv-10-11.measured-short-last",
            item: "fig-iv-10-11",
            source: Source::Comment,
            citation: "sweeps/fig-iv-10-11.json description, Measured: lifetimes 60–100 mean sd(ln price) 0.444 in the last block",
            text: "lifetimes 60–100, last block sd(ln price) about 0.444",
            check: |_| range(&block_at(&cell_blocks(builtin("fig-iv-10-11"), 0, 0), 1000), 0.444, 0.444, true).with(SWEEP_SEEDS),
        },
        Claim {
            id: "fig-iv-10-11.measured-long-last",
            item: "fig-iv-10-11",
            source: Source::Comment,
            citation: "sweeps/fig-iv-10-11.json description, Measured: lifetimes 960–1000 mean sd(ln price) 0.138 in the last block",
            text: "lifetimes 960–1000, last block sd(ln price) about 0.138",
            check: |_| range(&block_at(&cell_blocks(builtin("fig-iv-10-11"), 1, 0), 1000), 0.138, 0.138, true).with(SWEEP_SEEDS),
        },
        // ---------------------------------------- n-goods-carrying-capacity
        // x 0..=4 = 2..=6 goods; series 0 = no trade, 1 = trade.
        Claim {
            id: "n-goods-carrying-capacity.shared-sites",
            item: "n-goods-carrying-capacity",
            source: Source::App,
            citation: "sweeps/n-goods-carrying-capacity.json description",
            text: "some sites grow every good at every n: per number of goods (2–6), the count of sites with positive capacity for every good is ≥ 1 (the judge counts the 5 values of n)",
            check: |_| {
                let s = sweep::builtin("n-goods-carrying-capacity").unwrap();
                let v: Vec<f64> = s
                    .points()
                    .unwrap()
                    .into_iter()
                    .filter(|p| p.series == 0 && p.seed == s.seeds.from)
                    .map(|p| {
                        let c = sugarscape_config(&s, &p);
                        let n = c.goods.len();
                        let w = World::new(c, p.seed).unwrap();
                        w.sites.iter().filter(|site| (0..n).all(|g| site.capacity[g] > 0.0)).count() as f64
                    })
                    .collect();
                range(&v, 1.0, f64::INFINITY, false)
            },
        },
        Claim {
            id: "n-goods-carrying-capacity.four-is-n-4-peaks",
            item: "n-goods-carrying-capacity",
            source: Source::App,
            citation: "sweeps/n-goods-carrying-capacity.json description",
            text: "4 goods is exactly n-4-peaks: the trade line's 4-good config and n-4-peaks give identical fingerprints after 100 ticks, per seed (sweep's seeds 1–10)",
            check: |_| {
                let s = sweep::builtin("n-goods-carrying-capacity").unwrap();
                let point = s.points().unwrap().into_iter().find(|p| p.series == 1 && p.x == 2).unwrap();
                let c = sugarscape_config(&s, &point);
                let seeds: Vec<u64> = (s.seeds.from..s.seeds.from + u64::from(s.seeds.count)).collect();
                let fp = |c: &Config| {
                    each_seed(c, &seeds, |mut w| {
                        w.run(100);
                        w.fingerprint()
                    })
                };
                let (a, b) = (fp(&c), fp(&preset("n-4-peaks")));
                let v: Vec<f64> = a.iter().zip(&b).map(|(x, y)| f64::from(u8::from(x == y))).collect();
                range(&v, 1.0, 1.0, false).with(&format!("configs equal: {}", c == preset("n-4-peaks")))
            },
        },
        Claim {
            id: "n-goods-carrying-capacity.two-goods-low",
            item: "n-goods-carrying-capacity",
            source: Source::App,
            citation: "sweeps/n-goods-carrying-capacity.json description",
            text: "at 2 goods the peaks share only weak sites, so that point is low: on both lines the 3-good cell exceeds the 2-good cell (sweep's own seeds 1–10)",
            check: |_| {
                let r = builtin("n-goods-carrying-capacity");
                all_of(
                    (0..2)
                        .map(|s| (r.sweep.series_name(s), greater(&cell(r, s, 1), &cell(r, s, 0), "3 goods", "2 goods")))
                        .collect(),
                )
                .with(SWEEP_SEEDS)
            },
        },
        Claim {
            id: "n-goods-carrying-capacity.shared-peak-2",
            item: "n-goods-carrying-capacity",
            source: Source::App,
            citation: "sweeps/n-goods-carrying-capacity.json description",
            text: "with every good on one shared peak (all at (10, 10)), mean population over t = 900..=1000 at 2 goods is about 257, trade or not (both lines pooled, 2 × survey seeds)",
            check: |s| range(&shared_peak_capacity(0, s), 257.0, 257.0, true),
        },
        Claim {
            id: "n-goods-carrying-capacity.shared-peak-6",
            item: "n-goods-carrying-capacity",
            source: Source::App,
            citation: "sweeps/n-goods-carrying-capacity.json description",
            text: "with every good on one shared peak (all at (10, 10)), mean population over t = 900..=1000 at 6 goods is about 246, trade or not (both lines pooled, 2 × survey seeds)",
            check: |s| range(&shared_peak_capacity(4, s), 246.0, 246.0, true),
        },
        Claim {
            id: "n-goods-carrying-capacity.settled-1000",
            item: "n-goods-carrying-capacity",
            source: Source::App,
            citation: "sweeps/n-goods-carrying-capacity.json description",
            text: "1000 ticks satisfies the settlement rule for every cell (settlement ratio at T = 1000 ≤ 1 as in fig-ii-5.settled; the judge counts cells, 10)",
            check: |_| range(&settle_ratios(builtin_blocks("n-goods-carrying-capacity", 1000), &[1000]), 0.0, 1.0, false).with(SWEEP_SEEDS),
        },
        Claim {
            id: "n-goods-carrying-capacity.still-falling-500",
            item: "n-goods-carrying-capacity",
            source: Source::App,
            citation: "sweeps/n-goods-carrying-capacity.json description",
            text: "the 4-good, no-trade cell is still falling at 500: its 50-tick block mean population ending at t = 400 exceeds the one ending at t = 500 (sweep's own seeds 1–10)",
            check: |_| {
                let b = &builtin_blocks("n-goods-carrying-capacity", 1000)[0][2];
                greater(&block_at(&b, 400), &block_at(&b, 500), "B(400)", "B(500)").with(SWEEP_SEEDS)
            },
        },
        Claim {
            id: "n-goods-carrying-capacity.trade-raises",
            item: "n-goods-carrying-capacity",
            source: Source::Book,
            citation: "book, from memory: Figure IV-6, trade raises carrying capacity (here for 2–6 goods)",
            text: "trade raises carrying capacity at every number of goods (sweep's own seeds 1–10)",
            check: |_| line_above(builtin("n-goods-carrying-capacity"), 1, 0, "trade", "no trade").with(SWEEP_SEEDS),
        },
        Claim {
            id: "n-goods-carrying-capacity.measured-no-trade-2",
            item: "n-goods-carrying-capacity",
            source: Source::Comment,
            citation: "sweeps/n-goods-carrying-capacity.json description, Measured: no trade mean population 25.8 at 2 goods",
            text: "no-trade, 2-good cell about 25.8",
            check: |_| range(&cell(builtin("n-goods-carrying-capacity"), 0, 0), 25.8, 25.8, true).with(SWEEP_SEEDS),
        },
        Claim {
            id: "n-goods-carrying-capacity.measured-trade-3",
            item: "n-goods-carrying-capacity",
            source: Source::Comment,
            citation: "sweeps/n-goods-carrying-capacity.json description, Measured: trade mean population 99.6 at 3 goods",
            text: "trade, 3-good cell about 99.6",
            check: |_| range(&cell(builtin("n-goods-carrying-capacity"), 1, 1), 99.6, 99.6, true).with(SWEEP_SEEDS),
        },
        // ------------------------------------------------ bargaining-rules
        // Series 0 = geometric mean, 1 = random in [MRS_A, MRS_B].
        Claim {
            id: "bargaining-rules.similar",
            item: "bargaining-rules",
            source: Source::Book,
            citation: "tests/book.rs bargaining_rules_give_similar_carrying_capacities quoting Chapter IV note 15 (\"insensitive to this change\"); sweeps/bargaining-rules.json (τ = 0.20)",
            text: "the two price rules give equivalent carrying capacities at every mean vision (TOST, margin 0.20 × the geometric-mean cell's mean; sweep's own seeds 1–10)",
            check: |_| {
                let r = builtin("bargaining-rules");
                all_of(
                    (0..6)
                        .map(|x| {
                            let (g, rnd) = (cell(r, 0, x), cell(r, 1, x));
                            let margin = 0.2 * stats::mean(&g);
                            (format!("vision {}", x + 1), equivalent(&g, &rnd, Some(margin), "geometric", "random"))
                        })
                        .collect(),
                )
                .with(SWEEP_SEEDS)
            },
        },
        Claim {
            id: "bargaining-rules.vision-raises",
            item: "bargaining-rules",
            source: Source::Book,
            citation: "book, from memory: Chapter IV note 15, the qualitative character (carrying capacity rising with vision, Figure IV-6) survives the random price rule",
            text: "under both price rules, carrying capacity at mean vision 6 exceeds vision 1 (sweep's own seeds 1–10)",
            check: |_| lines_rise(builtin("bargaining-rules")).with(SWEEP_SEEDS),
        },
        Claim {
            id: "bargaining-rules.measured-geometric-v1",
            item: "bargaining-rules",
            source: Source::Comment,
            citation: "sweeps/bargaining-rules.json description, Measured: geometric mean 41.8 at vision 1",
            text: "geometric-mean, vision 1 cell about 41.8",
            check: |_| range(&cell(builtin("bargaining-rules"), 0, 0), 41.8, 41.8, true).with(SWEEP_SEEDS),
        },
        Claim {
            id: "bargaining-rules.measured-random-v6",
            item: "bargaining-rules",
            source: Source::Comment,
            citation: "sweeps/bargaining-rules.json description, Measured: random 75.5 at vision 6",
            text: "random-price, vision 6 cell about 75.5",
            check: |_| range(&cell(builtin("bargaining-rules"), 1, 5), 75.5, 75.5, true).with(SWEEP_SEEDS),
        },
    ]
}
