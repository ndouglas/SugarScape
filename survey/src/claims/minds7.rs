//! Minds 7: the evolution of larder hoarding
//! (docs/superpowers/specs/2026-09-30-minds-7-hoarding-evolution-design.md).
//!
//! Vander Wall and Jenkins's (2003) genetic algorithm, `HoardWorld`, run in
//! full: 60 generations of a 100-day season of 20 bouts (120 000 ticks), or
//! fewer when the population dies out. The judges and thresholds below were
//! fixed and committed before any survey run.
//!
//! **Runs.** Every condition runs 50 seeds (1–50; `--seeds` above 50 uses the
//! given seeds instead), more than V&J's 35. Every configuration compared
//! runs the same seeds.
//!
//! **The threshold grid** (claims 1–6): app_lard 1, 2 and 3, by the ratio
//! app_scat ÷ app_lard 0.05, 0.10, 0.15, 0.20, 0.22, 0.25, 0.30, 0.35,
//! 0.40 and 0.45, kept to V&J's range of app_scat, 0.05–0.90 (p.665): all
//! ten ratios at app_lard 1 and 2, and 0.05–0.30 at app_lard 3. That is 27
//! cells × 50 runs = 1 350 runs per larder weighting, and 2 700 for both.
//! Per ratio: 150 runs at 0.05–0.30 (three app_lard values), 100 at
//! 0.35–0.45. The logistic is fitted to all 1 350.
//!
//! **Both larder weightings** (contradiction 2). `per_burrow` is the default.
//! Claims 1, 2 and 4 are judged under both readings as full rows; the
//! `per_item` rows are reported as the reading that fails to reproduce V&J's
//! threshold (the default changed after per item gave no takeover; the
//! disclosure is in the spec).
//!
//! **Measures, per run** (amendments items 10 and 11):
//! - **Fate** is `HoardWorld::outcome`: takeover when the hoarders' mean L,
//!   averaged over the last 10 generations (51–60), is above 0.95; stayed low
//!   when it is below 0.2 in every one of them; otherwise intermediate;
//!   extinct when every agent died (never a takeover). **The rise** is the
//!   first generation whose hoarders' mean L is above 0.95, and **the lift**
//!   the first above 0.2.
//! - **Loss rates** (item 10): each agent's per-item daily hazard, larder
//!   items lost ÷ (Σ bout-start larder stock ÷ 20), for agents with a stock
//!   above 0, likewise for scatter. A run's **mean larder (scatter) loss** is
//!   the mean of these per-agent rates over every agent of generations 1–10.
//!   Its **CV** is the population SD ÷ the mean of the same rates.
//! - **The minimum larder loss** (claim 4) is the least per-agent larder rate
//!   in generations 1–10 over agents at the pre-registered exposure floor (a
//!   mean larder stock of at least 1 item at bout starts while alive,
//!   `EXPOSURE_FLOOR`), and without the floor over every agent with a rate.
//!
//! **Judges** (claims 1, 2 and 4 under each weighting):
//! 1. **All-or-nothing** (`hoard-threshold.all-or-nothing`): over the grid's
//!    runs, (a) the share whose fate is takeover or stayed low: Holds at
//!    100 % ("always", p.663), Weak at 95 % or more, Fails below (an
//!    intermediate or extinct run counts against it); (b) among the
//!    takeovers, the share whose rise is at generation 10 or earlier: Holds
//!    above 50 % ("usually within 10 generations"), Fails otherwise,
//!    Untestable with fewer than 5 takeovers. Both parts (`all_of`).
//!    (Note, fix round 1: the 100 % bar of (a) is stricter than V&J's own
//!    Fig. 2B, which shows an intermediate point near 0.22 at ratio 0.2.)
//! 2. **Threshold** (`hoard-threshold.threshold`): a logistic regression of
//!    takeover (1) against not (0: stayed low, intermediate or extinct) on
//!    the ratio, by maximum likelihood over the grid's runs. (a) Its 50 %
//!    point, −b0 ÷ b1, against V&J's 8.00 ÷ 36.59 = 0.219: Holds within
//!    ±0.03, Weak within ±0.06, Fails beyond or when the slope is not
//!    above 0. Reason for ±0.03: V&J's fit rests on 35 runs, and with their
//!    slope the delta-method standard error of the 50 % point is about
//!    0.015–0.02 for 35 runs spread over ratios 0.05–0.45 (our grid's
//!    span), so ±0.03 is about two of
//!    their standard errors; it is also under a third of the 0.2–0.3 band
//!    their text names. (b) No takeover at ratios below 0.2 (0.05, 0.10,
//!    0.15): Holds at 0 takeovers ("never evolved"), Weak at 5 % or fewer,
//!    Fails above. (c) Takeover at ratios above 0.3 (0.35, 0.40, 0.45): Holds
//!    at 90 % or more ("almost invariably"), Weak at 75 % or more, Fails
//!    below. All three (`all_of`). V&J's fit (logit = −8.00 + 36.59 × ratio,
//!    McFadden's ρ² 0.61) is reported beside ours.
//! 3. **Larder loss** (`hoard-threshold.larder-loss`, per burrow): per run,
//!    the mean larder loss above the mean scatter loss. Holds in every run
//!    ("in all 35 simulations"), Weak in 90 % or more, Fails below. A run
//!    without both rates has no value and is counted. The CV ratio (the
//!    mean over runs of the larder CV ÷ the mean of the scatter CV) is
//!    reported against 57 ÷ 33 = 1.73, not judged; 186 % and 24 % a day are
//!    context only (the paper's definition is unknown, item 10).
//!    **Disclosed omission (fix round 1):** amendments item 10 called for a
//!    CV-ratio judge; the judge commit left it reported, an omission made
//!    before any run. No judge is added after the runs.
//! 4. **Predictor** (`hoard-threshold.predictor`): per run, "min" predicts
//!    takeover when the minimum larder loss (at the floor) in generations
//!    1–10 is below the mean scatter loss there; "means" predicts it when the
//!    mean larder loss is below the mean scatter loss. Accuracy is the share
//!    of runs where the prediction equals the fate (takeover or not). Holds
//!    when min's accuracy exceeds means' and is at least 80 %, and when min
//!    predicts takeover, takeover follows in more than half those runs
//!    ("usually became established", p.663); Weak when min's accuracy only
//!    exceeds means'; Fails otherwise. Untestable with no run where min
//!    predicts takeover. Reported, not judged: the share of runs whose
//!    minimum is 0, with and without the floor; the agents at the floor;
//!    the same predictor on generation 1 alone and on generations 1–3.
//!
//! **Reported, not judged** (rows with an Untestable verdict and the
//! measurements):
//! 5. **Scatter withstands loss** (`hoard-scatter.withstands`): the mean
//!    daily scatter loss in the grid's runs without takeover, against 18 %
//!    (p.663), over generations 1–10 and 1–60.
//! 6. **Visibility** (`hoard-scatter.visibility`, V&J's p.663 prediction):
//!    takeover by app_scat at each app_lard, from the grid.
//! 7. **Owner recovery** (`hoard-no-free-recovery.recovery`): owner_recovery
//!    0.25, 0.5, 0.75 and 1 at ratios 0.1 and 0.22 (app_lard 2): fates and
//!    survival.
//! 8. **The non-hoarding cheater** (`hoard-cheaters.cheater`):
//!    hoard-cheaters under `cheater_fitness` stores and survival: the cheater
//!    share by generation, and survival against hoarders.
//! 9. **Sensitivity** (`hoard-threshold.sensitivity`, the spec's concerns 3
//!    and 5): `defense_slope` 4 and 10 by `v_seg` 0.25, 0.5 and 1, at ratios
//!    0.1, 0.2, 0.3 and 0.4 (app_lard 2).
//! 10. **The presets** (`hoard-threshold.presets`): each of the five presets'
//!    measurements, for their descriptions and titles.
//!
//! Causes in the details are labeled "likely" unless isolated.

use std::sync::Mutex;

use sugarscape_core::hoard::{
    CheaterFitness, Fate, HoardConfig, HoardWorld, LarderWeight, Season, EXPOSURE_FLOOR,
};
use sugarscape_core::model::ModelConfig;

use crate::claim::{all_of, Claim, Outcome, Source, Verdict};
use crate::claims::minds3::{med_or_nan, q_or_nan};
use crate::claims::minds5::med;
use crate::runner::{model_preset, on_threads};
use crate::stats;

const SPEC: &str = "docs/superpowers/specs/2026-09-30-minds-7-hoarding-evolution-design.md";

/// Runs per condition (seeds 1–50).
const RUNS: u64 = 50;
/// 60 generations of 100 days of 20 bouts.
const TICKS: u32 = 120_000;
/// The grid: app_lard, and the ratio app_scat ÷ app_lard.
const LARDS: [f64; 3] = [1.0, 2.0, 3.0];
const RATIOS: [f64; 10] = [0.05, 0.10, 0.15, 0.20, 0.22, 0.25, 0.30, 0.35, 0.40, 0.45];
/// V&J's largest app_scat (p.665).
const MAX_SCAT: f64 = 0.9;
/// Generations 1–10 (p.663: "within this time").
const EARLY: u32 = 10;
/// V&J's Fig. 2B fit and its 50 % point.
const VJ_B0: f64 = -8.00;
const VJ_B1: f64 = 36.59;
const VJ_RHO2: f64 = 0.61;
/// Claim 2's tolerances on the 50 % point.
const X50_HOLDS: f64 = 0.03;
const X50_WEAK: f64 = 0.06;
/// Claim 2's bands: "less than 0.2" and "greater than 0.3".
const LOW_RATIO: f64 = 0.2;
const HIGH_RATIO: f64 = 0.3;
/// Claim 2 (b): the most takeovers below 0.2 for Weak; (c): above 0.3.
const LOW_WEAK: f64 = 0.05;
const HIGH_HOLDS: f64 = 0.9;
const HIGH_WEAK: f64 = 0.75;
/// Claim 1 (a): the least share of clean runs for Weak.
const CLEAN_WEAK: f64 = 0.95;
/// Claim 3: the least share of runs for Weak.
const LOSS_WEAK: f64 = 0.9;
/// Claim 4: the least accuracy for Holds.
const PREDICT_HOLDS: f64 = 0.8;
/// Reported against: V&J's CVs, daily losses and the loss scatter withstands.
const VJ_CV_LARDER: f64 = 0.57;
const VJ_CV_SCATTER: f64 = 0.33;
const VJ_LARDER_LOSS: f64 = 1.86;
const VJ_SCATTER_LOSS: f64 = 0.24;
const VJ_WITHSTAND: f64 = 0.18;
/// New ground.
const RECOVERIES: [f64; 4] = [0.25, 0.5, 0.75, 1.0];
const RECOVERY_RATIOS: [f64; 2] = [0.1, 0.22];
const SLOPES: [f64; 2] = [4.0, 10.0];
const VSEGS: [f64; 3] = [0.25, 0.5, 1.0];
const SENS_RATIOS: [f64; 4] = [0.1, 0.2, 0.3, 0.4];
/// The five presets.
const PRESETS: [&str; 5] = [
    "hoard-threshold",
    "hoard-scatter",
    "hoard-larder",
    "hoard-no-free-recovery",
    "hoard-cheaters",
];

// ------------------------------------------------------------------- the run

/// One agent of generations 1–10, for the loss measures.
#[derive(Clone, Debug)]
struct Loss {
    generation: u32,
    larder: Option<f64>,
    scatter: Option<f64>,
    /// At the exposure floor (mean larder stock ≥ 1 at bout starts).
    exposed: bool,
    larder_lost: u64,
    larder_stock: u64,
    scatter_lost: u64,
    scatter_stock: u64,
}

/// One run's measures (see the module's list).
#[derive(Clone, Debug)]
struct Run {
    fate: Fate,
    /// The hoarders' mean L averaged over the window.
    window_l: f64,
    rise: Option<u32>,
    lift: Option<u32>,
    generations: u32,
    /// By generation: the hoarders' mean L, mean D, survivors ÷ born,
    /// cheaters born ÷ born, cheater and hoarder survival (NaN when none),
    /// and the survivors' larder share (NaN when they hold nothing).
    l: Vec<f64>,
    d: Vec<f64>,
    survival: Vec<f64>,
    cheaters: Vec<f64>,
    cheater_survival: Vec<f64>,
    hoarder_survival: Vec<f64>,
    larder_share: Vec<f64>,
    starved: f64,
    preyed: f64,
    /// Generations 1–10, per agent.
    early: Vec<Loss>,
    /// Means of the per-agent rates over every generation.
    larder_all: f64,
    scatter_all: f64,
    /// Owner recovery: bout-1 tries on scattered caches and misses.
    tries: f64,
    misses: f64,
}

fn mean_or_nan(v: &[f64]) -> f64 {
    if v.is_empty() {
        f64::NAN
    } else {
        stats::mean(v)
    }
}

/// Population SD ÷ mean (NaN with no values or a zero mean).
fn cv(v: &[f64]) -> f64 {
    let m = mean_or_nan(v);
    if !m.is_finite() || m == 0.0 {
        return f64::NAN;
    }
    let var = v.iter().map(|x| (x - m).powi(2)).sum::<f64>() / v.len() as f64;
    var.sqrt() / m
}

impl Run {
    fn takeover(&self) -> bool {
        self.fate == Fate::Takeover
    }
    fn larder_rates(&self, k: u32) -> Vec<f64> {
        self.early
            .iter()
            .filter(|a| a.generation <= k)
            .filter_map(|a| a.larder)
            .collect()
    }
    fn scatter_rates(&self, k: u32) -> Vec<f64> {
        self.early
            .iter()
            .filter(|a| a.generation <= k)
            .filter_map(|a| a.scatter)
            .collect()
    }
    /// The mean per-agent larder and scatter loss, generations 1–k.
    fn larder(&self, k: u32) -> f64 {
        mean_or_nan(&self.larder_rates(k))
    }
    fn scatter(&self, k: u32) -> f64 {
        mean_or_nan(&self.scatter_rates(k))
    }
    /// Claim 3, per run: the mean larder loss above the mean scatter loss
    /// (false when either is undefined).
    fn larder_above(&self) -> bool {
        self.larder(EARLY) > self.scatter(EARLY)
    }
    fn cv_larder(&self) -> f64 {
        cv(&self.larder_rates(EARLY))
    }
    fn cv_scatter(&self) -> f64 {
        cv(&self.scatter_rates(EARLY))
    }
    /// The pooled rates, generations 1–10: Σ lost ÷ (Σ stock ÷ 20).
    fn pooled(&self, larder: bool) -> f64 {
        let (lost, stock) = self.early.iter().fold((0u64, 0u64), |(l, s), a| {
            if larder {
                (l + a.larder_lost, s + a.larder_stock)
            } else {
                (l + a.scatter_lost, s + a.scatter_stock)
            }
        });
        if stock == 0 {
            f64::NAN
        } else {
            lost as f64 / (stock as f64 / 20.0)
        }
    }
    /// The least larder rate in generations 1–k, at the floor or over every
    /// agent with a rate; NaN when there is none.
    fn min_larder(&self, k: u32, floor: bool) -> f64 {
        self.early
            .iter()
            .filter(|a| a.generation <= k && (!floor || a.exposed))
            .filter_map(|a| a.larder)
            .reduce(f64::min)
            .unwrap_or(f64::NAN)
    }
    fn exposed(&self, k: u32) -> usize {
        self.early
            .iter()
            .filter(|a| a.generation <= k && a.exposed && a.larder.is_some())
            .count()
    }
    /// Claim 4's predictors (a NaN comparison predicts no takeover).
    fn predicts_min(&self, k: u32) -> bool {
        self.min_larder(k, true) < self.scatter(k)
    }
    fn predicts_means(&self, k: u32) -> bool {
        self.larder(k) < self.scatter(k)
    }
    fn survival_mean(&self) -> f64 {
        mean_or_nan(&self.survival)
    }
    fn window<'a>(&self, v: &'a [f64]) -> &'a [f64] {
        &v[v.len().saturating_sub(10)..]
    }
    fn d_window(&self) -> f64 {
        mean_or_nan(self.window(&self.d))
    }
    fn share_window(&self) -> f64 {
        mean_or_nan(&stats::finite(self.window(&self.larder_share)))
    }
    /// The first generation with no cheater born (None if never).
    fn cheaters_gone(&self) -> Option<u32> {
        self.cheaters
            .iter()
            .position(|&c| c == 0.0)
            .map(|i| i as u32 + 1)
    }
}

fn opt(x: Option<f64>) -> f64 {
    x.unwrap_or(f64::NAN)
}

fn measure(w: &HoardWorld) -> Run {
    let o = w.outcome().expect("a finished run");
    let bouts = w.config.bouts;
    let s: &[Season] = w.seasons();
    let expressed = |x: &Season| x.summary.hoarder_mean_l.unwrap_or(0.0);
    let born = |x: &Season| x.agents.len() as f64;
    let per = |f: &dyn Fn(&Season) -> f64| s.iter().map(f).collect::<Vec<f64>>();
    let mut early = Vec::new();
    let (mut larder_all, mut scatter_all) = (Vec::new(), Vec::new());
    let (mut tries, mut misses) = (0.0, 0.0);
    for season in s {
        for a in &season.agents {
            let r = &a.record;
            let (larder, scatter) = (r.larder_rate(bouts), r.scatter_rate(bouts));
            larder_all.extend(larder);
            scatter_all.extend(scatter);
            tries += r.recovery_tries as f64;
            misses += r.recovery_misses as f64;
            if season.generation <= EARLY {
                early.push(Loss {
                    generation: season.generation,
                    larder,
                    scatter,
                    exposed: r.mean_larder_stock() >= EXPOSURE_FLOOR,
                    larder_lost: r.larder_lost,
                    larder_stock: r.larder_stock,
                    scatter_lost: r.scatter_lost,
                    scatter_stock: r.scatter_stock,
                });
            }
        }
    }
    Run {
        fate: o.fate,
        window_l: o.window_hoarder_mean_l,
        rise: o.rise,
        lift: s.iter().find(|x| expressed(x) > 0.2).map(|x| x.generation),
        generations: s.len() as u32,
        l: per(&expressed),
        d: per(&|x| x.summary.mean_d),
        survival: per(&|x| f64::from(x.summary.survivors) / born(x)),
        cheaters: per(&|x| f64::from(x.summary.cheaters) / born(x)),
        cheater_survival: per(&|x| opt(x.summary.cheater_survival)),
        hoarder_survival: per(&|x| opt(x.summary.hoarder_survival)),
        larder_share: per(&|x| opt(x.summary.larder_share)),
        starved: s.iter().map(|x| f64::from(x.summary.starved)).sum(),
        preyed: s.iter().map(|x| f64::from(x.summary.preyed)).sum(),
        early,
        larder_all: mean_or_nan(&larder_all),
        scatter_all: mean_or_nan(&scatter_all),
        tries,
        misses,
    }
}

fn run_one(c: &HoardConfig, seed: u64) -> Run {
    let mut w = HoardWorld::new(c.clone(), seed).expect("a valid hoard config");
    w.run(TICKS);
    assert!(w.is_finished(), "a run of {TICKS} ticks finishes");
    measure(&w)
}

/// Cached runs: (key: seeds and config, runs).
type RunCache = Vec<(String, Vec<Run>)>;
static CACHE: Mutex<RunCache> = Mutex::new(Vec::new());

fn key(c: &HoardConfig, seeds: &[u64]) -> String {
    format!(
        "{seeds:?}{}",
        serde_json::to_string(c).expect("a config serializes")
    )
}

/// Runs of every config over `seeds`, shared by the claims: the configs not
/// yet run go to the thread pool together, one job per (config, seed).
fn batch(configs: &[HoardConfig], seeds: &[u64]) -> Vec<Vec<Run>> {
    let mut todo: Vec<HoardConfig> = Vec::new();
    {
        let cache = CACHE.lock().unwrap();
        for c in configs {
            let k = key(c, seeds);
            if !cache.iter().any(|(x, _)| *x == k) && !todo.contains(c) {
                todo.push(c.clone());
            }
        }
    }
    if !todo.is_empty() {
        let jobs: Vec<u64> = (0..(todo.len() * seeds.len()) as u64).collect();
        let out = on_threads(&jobs, |j| {
            let j = j as usize;
            run_one(&todo[j / seeds.len()], seeds[j % seeds.len()])
        });
        let mut cache = CACHE.lock().unwrap();
        for (i, c) in todo.iter().enumerate() {
            let runs = out[i * seeds.len()..(i + 1) * seeds.len()].to_vec();
            cache.push((key(c, seeds), runs));
        }
    }
    let cache = CACHE.lock().unwrap();
    configs
        .iter()
        .map(|c| {
            let k = key(c, seeds);
            cache.iter().find(|(x, _)| *x == k).unwrap().1.clone()
        })
        .collect()
}

/// Seeds 1–50, or the given seeds when there are at least 50.
fn seeds_for(seeds: &[u64]) -> Vec<u64> {
    if seeds.len() >= RUNS as usize {
        seeds.to_vec()
    } else {
        (1..=RUNS).collect()
    }
}

// ------------------------------------------------------------------ worlds

fn hoard_preset(id: &str) -> HoardConfig {
    match model_preset(id) {
        ModelConfig::Hoard(c) => c,
        _ => panic!("{id} is not a hoard preset"),
    }
}

/// V&J's defaults at a ratio and app_lard, under a larder weighting.
fn world(ratio: f64, lard: f64, weight: LarderWeight) -> HoardConfig {
    HoardConfig {
        app_lard: lard,
        app_scat: ratio * lard,
        larder_weight: weight,
        ..HoardConfig::default()
    }
}

/// The grid's cells, (app_lard, ratio), app_scat within V&J's range.
fn cells() -> Vec<(f64, f64)> {
    LARDS
        .iter()
        .flat_map(|&l| RATIOS.iter().map(move |&r| (l, r)))
        .filter(|&(l, r)| r * l <= MAX_SCAT + 1e-9)
        .collect()
}

/// One cell of a grid and its runs.
struct Cell {
    lard: f64,
    ratio: f64,
    runs: Vec<Run>,
}

fn grid(weight: LarderWeight, seeds: &[u64]) -> Vec<Cell> {
    let seeds = seeds_for(seeds);
    let cs = cells();
    let configs: Vec<HoardConfig> = cs.iter().map(|&(l, r)| world(r, l, weight)).collect();
    batch(&configs, &seeds)
        .into_iter()
        .zip(cs)
        .map(|(runs, (lard, ratio))| Cell { lard, ratio, runs })
        .collect()
}

fn weight_name(w: LarderWeight) -> &'static str {
    match w {
        LarderWeight::PerBurrow => "per burrow",
        LarderWeight::PerItem => "per item",
    }
}

// ----------------------------------------------------------------- judges

/// Holds at a share of at least `holds`, Weak at least `weak`, else Fails.
fn at_least(share: f64, holds: f64, weak: f64) -> Verdict {
    if share >= holds {
        Verdict::Holds
    } else if share >= weak {
        Verdict::Weak
    } else {
        Verdict::Fails
    }
}

/// Holds at a share of at most `holds`, Weak at most `weak`, else Fails.
fn at_most(share: f64, holds: f64, weak: f64) -> Verdict {
    if share <= holds {
        Verdict::Holds
    } else if share <= weak {
        Verdict::Weak
    } else {
        Verdict::Fails
    }
}

fn outcome(verdict: Verdict, measured: String) -> Outcome {
    Outcome {
        verdict,
        measured,
        detail: String::new(),
    }
}

/// A row that is reported, not judged.
fn reported(measured: String) -> Outcome {
    Outcome {
        verdict: Verdict::Untestable,
        measured,
        detail: "Reported, not judged.".into(),
    }
}

fn frac(k: usize, n: usize) -> f64 {
    if n == 0 {
        f64::NAN
    } else {
        k as f64 / n as f64
    }
}

fn pct(x: f64) -> String {
    format!("{:.1} %", 100.0 * x)
}

/// A logistic regression of 0/1 outcomes on one predictor, by Newton's
/// method on the log-likelihood (step halving; at most 200 steps).
#[derive(Clone, Copy, Debug)]
struct Fit {
    b0: f64,
    b1: f64,
    /// The 50 % point −b0 ÷ b1 and its delta-method standard error.
    x50: f64,
    se_x50: f64,
    /// McFadden's ρ²: 1 − LL ÷ LL(intercept only).
    rho2: f64,
    n: usize,
    /// Runs that took over (outcome 1).
    ones: usize,
    /// Whether the outcomes are completely separated by the predictor (the
    /// slope then grows without bound; the 50 % point is still located).
    separated: bool,
}

fn log_lik(xs: &[f64], ys: &[f64], b0: f64, b1: f64) -> f64 {
    xs.iter()
        .zip(ys)
        .map(|(&x, &y)| {
            let z = b0 + b1 * x;
            // log σ(z) = −softplus(−z) and log(1 − σ(z)) = −softplus(z).
            y * -softplus(-z) + (1.0 - y) * -softplus(z)
        })
        .sum()
}

/// ln(1 + e^t), without overflow.
fn softplus(t: f64) -> f64 {
    if t > 0.0 {
        t + (-t).exp().ln_1p()
    } else {
        t.exp().ln_1p()
    }
}

fn sigmoid(z: f64) -> f64 {
    1.0 / (1.0 + (-z).exp())
}

fn logistic(xs: &[f64], ys: &[f64]) -> Fit {
    let n = xs.len();
    let ybar = ys.iter().sum::<f64>() / n as f64;
    let ll0 = if ybar <= 0.0 || ybar >= 1.0 {
        0.0
    } else {
        n as f64 * (ybar * ybar.ln() + (1.0 - ybar) * (1.0 - ybar).ln())
    };
    let (mut b0, mut b1) = (0.0, 0.0);
    let mut ll = log_lik(xs, ys, b0, b1);
    let mut info = [[0.0; 2]; 2];
    for _ in 0..200 {
        let (mut g0, mut g1) = (0.0, 0.0);
        info = [[0.0; 2]; 2];
        for (&x, &y) in xs.iter().zip(ys) {
            let p = sigmoid(b0 + b1 * x);
            g0 += y - p;
            g1 += (y - p) * x;
            let w = p * (1.0 - p);
            info[0][0] += w;
            info[0][1] += w * x;
            info[1][1] += w * x * x;
        }
        info[1][0] = info[0][1];
        let det = info[0][0] * info[1][1] - info[0][1] * info[1][0];
        if det.abs() < 1e-300 {
            break;
        }
        let d0 = (info[1][1] * g0 - info[0][1] * g1) / det;
        let d1 = (info[0][0] * g1 - info[1][0] * g0) / det;
        let mut step = 1.0;
        let mut moved = false;
        while step > 1e-8 {
            let (n0, n1) = (b0 + step * d0, b1 + step * d1);
            let nl = log_lik(xs, ys, n0, n1);
            if nl >= ll {
                moved = (n0 - b0).abs() + (n1 - b1).abs() > 1e-12;
                b0 = n0;
                b1 = n1;
                ll = nl;
                break;
            }
            step *= 0.5;
        }
        if !moved {
            break;
        }
    }
    let x50 = -b0 / b1;
    let det = info[0][0] * info[1][1] - info[0][1] * info[1][0];
    let se_x50 = if det.abs() > 1e-300 && b1 != 0.0 {
        let (v00, v11, v01) = (info[1][1] / det, info[0][0] / det, -info[0][1] / det);
        ((v00 + x50 * x50 * v11 + 2.0 * x50 * v01) / (b1 * b1)).sqrt()
    } else {
        f64::NAN
    };
    let max0 = xs
        .iter()
        .zip(ys)
        .filter(|(_, &y)| y < 0.5)
        .map(|(&x, _)| x)
        .fold(f64::NEG_INFINITY, f64::max);
    let min1 = xs
        .iter()
        .zip(ys)
        .filter(|(_, &y)| y > 0.5)
        .map(|(&x, _)| x)
        .fold(f64::INFINITY, f64::min);
    Fit {
        b0,
        b1,
        x50,
        se_x50,
        rho2: if ll0 == 0.0 { f64::NAN } else { 1.0 - ll / ll0 },
        n,
        ones: ys.iter().filter(|&&y| y > 0.5).count(),
        separated: max0 < min1,
    }
}

impl Fit {
    fn text(&self) -> String {
        // A display fix after the runs (fix round 1), not a judge: with no
        // takeover the maximum-likelihood fit is degenerate.
        if self.ones == 0 {
            return format!(
                "no fit: no run took over (n = {}; display fix after the runs, the verdict is the judge's)",
                self.n
            );
        }
        format!(
            "logit = {:.2} + {:.2} × ratio, McFadden's ρ² {:.2}, 50 % point {:.3} (SE {:.3}), n = {}{}",
            self.b0,
            self.b1,
            self.rho2,
            self.x50,
            self.se_x50,
            self.n,
            if self.separated {
                " (completely separated)"
            } else {
                ""
            }
        )
    }
}

/// Claim 2 (a): the 50 % point against V&J's.
fn x50_verdict(fit: &Fit) -> Verdict {
    let vj = -VJ_B0 / VJ_B1;
    if fit.b1.is_nan() || fit.b1 <= 0.0 || !fit.x50.is_finite() {
        return Verdict::Fails;
    }
    let off = (fit.x50 - vj).abs();
    if off <= X50_HOLDS {
        Verdict::Holds
    } else if off <= X50_WEAK {
        Verdict::Weak
    } else {
        Verdict::Fails
    }
}

// ----------------------------------------------------------------- helpers

fn all_runs(g: &[Cell]) -> Vec<&Run> {
    g.iter().flat_map(|c| c.runs.iter()).collect()
}

fn col<'a>(r: impl IntoIterator<Item = &'a Run>, f: impl Fn(&Run) -> f64) -> Vec<f64> {
    r.into_iter().map(f).collect()
}

/// Median and IQR as percentages.
fn medp(v: &[f64]) -> String {
    format!(
        "{:.1} % (IQR {:.1}–{:.1} %)",
        100.0 * med_or_nan(v),
        100.0 * q_or_nan(v, 0.25),
        100.0 * q_or_nan(v, 0.75)
    )
}

fn fates(r: &[&Run]) -> String {
    let n = |f: Fate| r.iter().filter(|x| x.fate == f).count();
    format!(
        "takeover {}, stayed low {}, intermediate {}, extinct {} of {}",
        n(Fate::Takeover),
        n(Fate::StayedLow),
        n(Fate::Intermediate),
        n(Fate::Extinct),
        r.len()
    )
}

/// Takeovers per cell, by app_lard.
fn cell_table(g: &[Cell]) -> String {
    LARDS
        .iter()
        .map(|&l| {
            let row: Vec<String> = g
                .iter()
                .filter(|c| c.lard == l)
                .map(|c| {
                    format!(
                        "{:.2} {}/{}",
                        c.ratio,
                        c.runs.iter().filter(|x| x.takeover()).count(),
                        c.runs.len()
                    )
                })
                .collect();
            format!("app_lard {l}: {}", row.join(", "))
        })
        .collect::<Vec<_>>()
        .join("; ")
}

fn fit_cells<'a>(cells: impl IntoIterator<Item = &'a Cell>) -> Fit {
    let (mut xs, mut ys) = (Vec::new(), Vec::new());
    for c in cells {
        for r in &c.runs {
            xs.push(c.ratio);
            ys.push(if r.takeover() { 1.0 } else { 0.0 });
        }
    }
    logistic(&xs, &ys)
}

fn opt_u(x: Option<u32>) -> f64 {
    x.map_or(f64::NAN, f64::from)
}

// -------------------------------------------------------- 1. all-or-nothing

fn all_or_nothing(weight: LarderWeight, seeds: &[u64]) -> Outcome {
    let g = grid(weight, seeds);
    let r = all_runs(&g);
    let clean = r
        .iter()
        .filter(|x| matches!(x.fate, Fate::Takeover | Fate::StayedLow))
        .count();
    let a = outcome(
        at_least(frac(clean, r.len()), 1.0, CLEAN_WEAK),
        format!(
            "{clean} of {} runs end below 0.2 or above 0.95 ({})",
            r.len(),
            fates(&r)
        ),
    );
    let ups: Vec<&Run> = r.iter().copied().filter(|x| x.takeover()).collect();
    let quick = ups
        .iter()
        .filter(|x| x.rise.is_some_and(|g| g <= EARLY))
        .count();
    let b = if ups.len() < 5 {
        Outcome {
            verdict: Verdict::Untestable,
            measured: format!("{} takeovers (need 5)", ups.len()),
            detail: String::new(),
        }
    } else {
        let share = frac(quick, ups.len());
        outcome(
            if share > 0.5 {
                Verdict::Holds
            } else {
                Verdict::Fails
            },
            format!(
                "{quick} of {} takeovers rise above 0.95 by generation 10 ({})",
                ups.len(),
                pct(share)
            ),
        )
    };
    let odd: Vec<String> = g
        .iter()
        .flat_map(|c| {
            c.runs
                .iter()
                .enumerate()
                .filter(|(_, x)| matches!(x.fate, Fate::Intermediate | Fate::Extinct))
                .map(move |(i, x)| {
                    format!(
                        "app_lard {} ratio {:.2} run {}: {:?}, window L {:.3}, generations {}",
                        c.lard,
                        c.ratio,
                        i + 1,
                        x.fate,
                        x.window_l,
                        x.generations
                    )
                })
        })
        .collect();
    let rise = col(ups.iter().copied(), |x| opt_u(x.rise));
    let span = col(ups.iter().copied(), |x| opt_u(x.rise) - opt_u(x.lift));
    let late_up = ups
        .iter()
        .filter(|x| x.rise.is_some_and(|g| g > EARLY))
        .count();
    let rose_fell = r
        .iter()
        .filter(|x| !x.takeover() && x.rise.is_some())
        .count();
    all_of(vec![
        ("ends low or high".into(), a),
        ("rise within 10 generations".into(), b),
    ])
    .with(&format!(
        "Larder weighting {}; {} cells × {} runs, 60 generations each. The rise generation over takeovers: {}; generations from the lift above 0.2 to the rise above 0.95: {}; takeovers rising after generation 10: {late_up}. Runs that rose above 0.95 once and did not take over: {rose_fell}. Intermediate or extinct runs: {}.",
        weight_name(weight),
        g.len(),
        g[0].runs.len(),
        med(&rise),
        med(&span),
        if odd.is_empty() { "none".into() } else { odd.join("; ") },
    ))
    .with(if weight == LarderWeight::PerItem {
        "Note (fix round 1): under per item nothing rises, so part (a) only says every run stayed low or wavered below 0.95; its Weak is vacuous as a test of all-or-nothing, and part (b) is untestable."
    } else {
        ""
    })
    .with("Vander Wall and Jenkins: \"In 35 runs of the model, average probability of larder hoarding always remained less than 0.2 or increased rapidly to more than 0.95, usually within 10 generations (Figure 2A).\" (p.663)")
}

// ------------------------------------------------------------- 2. threshold

fn threshold(weight: LarderWeight, seeds: &[u64]) -> Outcome {
    let g = grid(weight, seeds);
    let fit = fit_cells(&g);
    let a = outcome(
        x50_verdict(&fit),
        format!(
            "ours: {}; V&J: logit = {VJ_B0:.2} + {VJ_B1:.2} × ratio, ρ² {VJ_RHO2:.2}, 50 % point {:.3} (tolerance ±{X50_HOLDS} Holds, ±{X50_WEAK} Weak)",
            fit.text(),
            -VJ_B0 / VJ_B1
        ),
    );
    let band = |keep: &dyn Fn(f64) -> bool| {
        let r: Vec<&Run> = g
            .iter()
            .filter(|c| keep(c.ratio))
            .flat_map(|c| c.runs.iter())
            .collect();
        let k = r.iter().filter(|x| x.takeover()).count();
        (k, r.len())
    };
    let (kl, nl) = band(&|x| x < LOW_RATIO - 1e-9);
    let (kh, nh) = band(&|x| x > HIGH_RATIO + 1e-9);
    let b = outcome(
        at_most(frac(kl, nl), 0.0, LOW_WEAK),
        format!("{kl} of {nl} runs at ratios below 0.2 take over"),
    );
    let c = outcome(
        at_least(frac(kh, nh), HIGH_HOLDS, HIGH_WEAK),
        format!(
            "{kh} of {nh} runs at ratios above 0.3 take over ({})",
            pct(frac(kh, nh))
        ),
    );
    let per_lard: Vec<String> = LARDS
        .iter()
        .map(|&l| {
            format!(
                "app_lard {l}: {}",
                fit_cells(g.iter().filter(|c| c.lard == l)).text()
            )
        })
        .collect();
    all_of(vec![
        ("50 % point".into(), a),
        ("none below 0.2".into(), b),
        ("almost all above 0.3".into(), c),
    ])
    .with(&format!(
        "Larder weighting {}. Takeovers per cell (ratio k/n): {}. Fits per app_lard (reported, not judged): {}.",
        weight_name(weight),
        cell_table(&g),
        per_lard.join("; ")
    ))
    .with("Vander Wall and Jenkins: \"For ratios less than 0.2, high probabilities of larder hoarding never evolved; for ratios greater than 0.3, high probabilities of larder hoarding almost invariably evolved.\" (p.663); Fig. 2B: \"logit = −8.00 + 36.59 × ratio; McFadden's ρ2 = 0.61\" (p.662)")
}

// ----------------------------------------------------------- 3. larder loss

/// Claim 3's numbers for a grid.
fn loss_numbers(g: &[Cell]) -> String {
    let r = all_runs(g);
    let larder = col(r.iter().copied(), |x| x.larder(EARLY));
    let scatter = col(r.iter().copied(), |x| x.scatter(EARLY));
    let cvl = mean_or_nan(&stats::finite(&col(r.iter().copied(), Run::cv_larder)));
    let cvs = mean_or_nan(&stats::finite(&col(r.iter().copied(), Run::cv_scatter)));
    let across = |v: &[f64]| cv(&stats::finite(v));
    let unrated_l: usize = r
        .iter()
        .map(|x| {
            x.early
                .iter()
                .filter(|a| a.larder.is_none() && a.larder_lost > 0)
                .count()
        })
        .sum();
    let unrated_s: usize = r
        .iter()
        .map(|x| {
            x.early
                .iter()
                .filter(|a| a.scatter.is_none() && a.scatter_lost > 0)
                .count()
        })
        .sum();
    let by_ratio: Vec<String> = RATIOS
        .iter()
        .map(|&q| {
            let rr: Vec<&Run> = g
                .iter()
                .filter(|c| c.ratio == q)
                .flat_map(|c| c.runs.iter())
                .collect();
            let above = rr.iter().filter(|x| x.larder_above()).count();
            format!(
                "{q:.2}: {above}/{} (larder {}, scatter {})",
                rr.len(),
                medp(&col(rr.iter().copied(), |x| x.larder(EARLY))),
                medp(&col(rr.iter().copied(), |x| x.scatter(EARLY)))
            )
        })
        .collect();
    format!(
        "Mean per-agent daily loss, generations 1–10, median over runs: larder {}, scatter {} (V&J's {:.0} % and {:.0} %, context only: their definition is unknown). Pooled (Σ lost ÷ item-days): larder {}, scatter {}. CV within runs, mean over runs: larder {:.2}, scatter {:.2}, ratio {:.2} (V&J {:.0} % and {:.0} %, ratio {:.2}; reported, not judged); the alternative, CV across runs of the run means: larder {:.2}, scatter {:.2}, ratio {:.2}. Agents with losses but no rate (held none at a bout start), generations 1–10: larder {unrated_l}, scatter {unrated_s}. Larder above scatter, by ratio (runs; median losses): {}.",
        medp(&larder),
        medp(&scatter),
        100.0 * VJ_LARDER_LOSS,
        100.0 * VJ_SCATTER_LOSS,
        medp(&col(r.iter().copied(), |x| x.pooled(true))),
        medp(&col(r.iter().copied(), |x| x.pooled(false))),
        cvl,
        cvs,
        cvl / cvs,
        100.0 * VJ_CV_LARDER,
        100.0 * VJ_CV_SCATTER,
        VJ_CV_LARDER / VJ_CV_SCATTER,
        across(&larder),
        across(&scatter),
        across(&larder) / across(&scatter),
        by_ratio.join("; ")
    )
}

/// The CV ratio (larder ÷ scatter) within runs (the mean over runs of
/// each CV) and across runs (the CV of the run means).
fn cv_ratios(g: &[Cell]) -> (f64, f64) {
    let r = all_runs(g);
    let within = mean_or_nan(&stats::finite(&col(r.iter().copied(), Run::cv_larder)))
        / mean_or_nan(&stats::finite(&col(r.iter().copied(), Run::cv_scatter)));
    let across = cv(&stats::finite(&col(r.iter().copied(), |x| x.larder(EARLY))))
        / cv(&stats::finite(&col(r.iter().copied(), |x| {
            x.scatter(EARLY)
        })));
    (within, across)
}

fn larder_loss_claim(seeds: &[u64]) -> Outcome {
    let g = grid(LarderWeight::PerBurrow, seeds);
    let r = all_runs(&g);
    let valued: Vec<&Run> = r
        .iter()
        .copied()
        .filter(|x| x.larder(EARLY).is_finite() && x.scatter(EARLY).is_finite())
        .collect();
    let above = valued.iter().filter(|x| x.larder_above()).count();
    let o = if valued.len() < 5 {
        Outcome {
            verdict: Verdict::Untestable,
            measured: format!("only {} runs have both rates", valued.len()),
            detail: String::new(),
        }
    } else {
        outcome(
            at_least(frac(above, valued.len()), 1.0, LOSS_WEAK),
            format!(
                "larder loss above scatter loss in {above} of {} runs ({}); {} runs without both rates",
                valued.len(),
                pct(frac(above, valued.len())),
                r.len() - valued.len()
            ),
        )
    };
    let not: Vec<String> = g
        .iter()
        .flat_map(|c| {
            c.runs
                .iter()
                .enumerate()
                .filter(|(_, x)| !x.larder_above())
                .map(move |(i, x)| {
                    format!(
                        "app_lard {} ratio {:.2} run {} (larder {}, scatter {}, {:?})",
                        c.lard,
                        c.ratio,
                        i + 1,
                        pct(x.larder(EARLY)),
                        pct(x.scatter(EARLY)),
                        x.fate
                    )
                })
        })
        .collect();
    let (r_above, r_n) = (r.iter().filter(|x| x.larder_above()).count(), r.len());
    let item = grid(LarderWeight::PerItem, seeds);
    let ri = all_runs(&item);
    let above_i = ri.iter().filter(|x| x.larder_above()).count();
    o.with(&format!(
        "Per burrow (the judged rows): {} Runs with larder at or below scatter: {}.",
        loss_numbers(&g),
        if not.is_empty() { "none".into() } else { not.join("; ") }
    ))
    .with(&format!(
        "Per item (reported, not judged): larder above scatter in {above_i} of {} runs. {}",
        ri.len(),
        loss_numbers(&item)
    ))
    .with(&{
        let (w, a) = cv_ratios(&g);
        let (wi, ai) = cv_ratios(&item);
        let med_l = |gg: &[Cell]| med_or_nan(&col(all_runs(gg), |x| x.larder(EARLY)));
        format!(
            "Disclosed (fix round 1): item 10 called for a CV-ratio judge; the judge commit left it reported, an omission made before any run. The ratio is {w:.2} within runs ({a:.2} across runs) against V&J's {:.2}: \"almost twice\" is not matched under our primary definition. Neither reading reproduces both the outcome and the loss statistics: per item has larder above scatter in {above_i} of {} runs (V&J: \"all 35\"), a CV ratio of {wi:.2} within runs ({ai:.2} across) and a median larder loss of {} a day (V&J 186 %), but larders never take over; per burrow reproduces the outcome but has larder above scatter in {} and a CV ratio of {w:.2} (median larder loss {}). Likely V&J's detection or defense differs from both readings in some unstated way; per burrow may stand in for that difference rather than being their rule.",
            VJ_CV_LARDER / VJ_CV_SCATTER,
            ri.len(),
            pct(med_l(&item)),
            pct(frac(r_above, r_n)),
            pct(med_l(&g)),
        )
    })
    .with("Vander Wall and Jenkins: items in larders \"were lost at a much greater rate (mean = 186% per day) than were scattered caches (24%/day). In addition, the coefficient of variation in rate of loss of larder-hoarded items (57%) was almost twice that of scattered caches (33%).\" \"the average daily rate of loss of larder-hoarded items exceeded that of scatter-hoarded items in all 35 simulations\" (p.663)")
}

// -------------------------------------------------------------- 4. predictor

/// (accuracy of min, of means, takeovers among min's positives, positives).
fn predictor_numbers(r: &[&Run], k: u32) -> (f64, f64, f64, usize) {
    let n = r.len();
    let acc = |p: &dyn Fn(&Run) -> bool| frac(r.iter().filter(|x| p(x) == x.takeover()).count(), n);
    let pos: Vec<&&Run> = r.iter().filter(|x| x.predicts_min(k)).collect();
    let ppv = frac(pos.iter().filter(|x| x.takeover()).count(), pos.len());
    (
        acc(&|x| x.predicts_min(k)),
        acc(&|x| x.predicts_means(k)),
        ppv,
        pos.len(),
    )
}

fn confusion(r: &[&Run], p: impl Fn(&Run) -> bool) -> String {
    let c = |pred: bool, t: bool| {
        r.iter()
            .filter(|x| p(x) == pred && x.takeover() == t)
            .count()
    };
    format!(
        "predicted and took over {}, predicted and didn't {}, not predicted and took over {}, neither {}",
        c(true, true),
        c(true, false),
        c(false, true),
        c(false, false)
    )
}

fn predictor_claim(weight: LarderWeight, seeds: &[u64]) -> Outcome {
    let g = grid(weight, seeds);
    let r = all_runs(&g);
    let (acc_min, acc_means, ppv, pos) = predictor_numbers(&r, EARLY);
    let verdict = if pos == 0 {
        Verdict::Untestable
    } else if acc_min > acc_means && acc_min >= PREDICT_HOLDS && ppv > 0.5 {
        Verdict::Holds
    } else if acc_min > acc_means {
        Verdict::Weak
    } else {
        Verdict::Fails
    };
    let zero = |floor: bool| {
        r.iter()
            .filter(|x| x.min_larder(EARLY, floor) == 0.0)
            .count()
    };
    let none = r
        .iter()
        .filter(|x| x.min_larder(EARLY, true).is_nan())
        .count();
    let exposed = col(r.iter().copied(), |x| x.exposed(EARLY) as f64);
    let context: Vec<String> = [1, 3]
        .iter()
        .map(|&k| {
            let (a, m, p, n) = predictor_numbers(&r, k);
            format!(
                "generations 1–{k}: accuracy {} against means' {}; takeover in {} of the {n} runs it predicts",
                pct(a),
                pct(m),
                pct(p)
            )
        })
        .collect();
    let by_fate = |t: bool| {
        let rr: Vec<&Run> = r.iter().copied().filter(|x| x.takeover() == t).collect();
        format!(
            "min larder loss at the floor {}, mean scatter loss {}",
            medp(&col(rr.iter().copied(), |x| x.min_larder(EARLY, true))),
            medp(&col(rr.iter().copied(), |x| x.scatter(EARLY)))
        )
    };
    Outcome {
        verdict,
        measured: format!(
            "accuracy of min (at the floor) < mean scatter: {}; of mean larder < mean scatter: {}; takeover in {} of the {pos} runs min predicts ({})",
            pct(acc_min),
            pct(acc_means),
            pct(ppv),
            fates(&r)
        ),
        detail: String::new(),
    }
    .with(&format!(
        "Larder weighting {}; generations 1–10. Min: {}. Means: {}. Runs whose minimum is 0: {} with the floor, {} without; runs with no agent at the floor: {none}. Agents at the floor per run (generations 1–10): {}. Takeover runs: {}; the rest: {}. Reported, not judged: {}.",
        weight_name(weight),
        confusion(&r, |x| x.predicts_min(EARLY)),
        confusion(&r, |x| x.predicts_means(EARLY)),
        zero(true),
        zero(false),
        med(&exposed),
        by_fate(true),
        by_fate(false),
        context.join("; ")
    ))
    .with("Vander Wall and Jenkins: \"If one or more individuals in the first 10 generations lost items from larders at a lower rate than the average rate of loss of scattered caches in the population, then larder hoarding usually became established at high levels in these simulations (Figure 3).\" (p.663)")
}

// ------------------------------------------- reported rows (fix round 1)

/// The sensitivity sweep's config.
fn sens(slope: f64, v_seg: f64, ratio: f64) -> HoardConfig {
    HoardConfig {
        defense_slope: slope,
        v_seg,
        ..world(ratio, 2.0, LarderWeight::PerBurrow)
    }
}

/// Takeovers rising by generation 10, and within 10 generations of the
/// lift above 0.2: (by 10, within 10 of the lift, takeovers).
fn rise_counts<'a>(r: impl IntoIterator<Item = &'a Run>) -> (usize, usize, usize) {
    let ups: Vec<&Run> = r.into_iter().filter(|x| x.takeover()).collect();
    let by = ups
        .iter()
        .filter(|x| x.rise.is_some_and(|g| g <= EARLY))
        .count();
    let lifted = ups
        .iter()
        .filter(|x| opt_u(x.rise) - opt_u(x.lift) <= f64::from(EARLY))
        .count();
    (by, lifted, ups.len())
}

/// Reported, not judged (fix round 1): the two readings of "within 10
/// generations", per cell, and by V_seg.
fn rise_claim(seeds: &[u64]) -> Outcome {
    let g = grid(LarderWeight::PerBurrow, seeds);
    let (by, lifted, n) = rise_counts(all_runs(&g));
    let cells: Vec<String> = g
        .iter()
        .filter(|c| c.runs.iter().any(Run::takeover))
        .map(|c| {
            let ups: Vec<&Run> = c.runs.iter().filter(|x| x.takeover()).collect();
            let (b, l, k) = rise_counts(c.runs.iter());
            format!(
                "app_lard {} ratio {:.2}: rise median {:.1}, lift median {:.1}, by 10 {b}/{k}, within 10 of the lift {l}/{k}",
                c.lard,
                c.ratio,
                med_or_nan(&col(ups.iter().copied(), |x| opt_u(x.rise))),
                med_or_nan(&col(ups.iter().copied(), |x| opt_u(x.lift))),
            )
        })
        .collect();
    let seeds = seeds_for(seeds);
    let vsegs: Vec<String> = VSEGS
        .iter()
        .map(|&v| {
            let configs: Vec<HoardConfig> = SENS_RATIOS.iter().map(|&q| sens(10.0, v, q)).collect();
            let out = batch(&configs, &seeds);
            let (b4, l4, k4) = rise_counts(out[3].iter());
            let (b, l, k) = rise_counts(out.iter().flatten());
            format!(
                "V_seg {v}: at ratio 0.4 by 10 {b4}/{k4}, within 10 of the lift {l4}/{k4}; at ratios 0.1–0.4 by 10 {b}/{k}, within 10 of the lift {l}/{k}"
            )
        })
        .collect();
    reported(format!(
        "Takeovers (per burrow grid) rising above 0.95 by generation 10: {by} of {n} ({}); within 10 generations of lifting above 0.2: {lifted} of {n} ({})",
        pct(frac(by, n)),
        pct(frac(lifted, n))
    ))
    .with(&format!(
        "Per cell with a takeover: {}. By V_seg (defense slope 10, app_lard 2, the sensitivity runs): {}. V&J's own Fig. 2A example (app_lard 1.0) passes 0.95 near generation 16. The shape: L lifts early and saturates slowly; likely the segregation variance, which the paper doesn't print, sets the speed.",
        cells.join("; "),
        vsegs.join("; ")
    ))
}

/// Reported, not judged (fix round 1): the predictor beside baselines.
fn baselines_claim(seeds: &[u64]) -> Outcome {
    let rows: Vec<String> = [LarderWeight::PerBurrow, LarderWeight::PerItem]
        .iter()
        .map(|&w| {
            let g = grid(w, seeds);
            let n: usize = g.iter().map(|c| c.runs.len()).sum();
            let acc = |p: &dyn Fn(f64, &Run) -> bool| {
                let k: usize = g
                    .iter()
                    .map(|c| c.runs.iter().filter(|x| p(c.ratio, x) == x.takeover()).count())
                    .sum();
                pct(frac(k, n))
            };
            format!(
                "{}: min at the floor below mean scatter {}; mean larder below mean scatter {}; ratio ≥ 0.25 {}; always no takeover {}",
                weight_name(w),
                acc(&|_, x| x.predicts_min(EARLY)),
                acc(&|_, x| x.predicts_means(EARLY)),
                acc(&|q, _| q >= 0.25 - 1e-9),
                acc(&|_, _| false),
            )
        })
        .collect();
    reported(format!(
        "Accuracy against takeover over the grid's runs: {}",
        rows.join(". ")
    ))
    .with("The minimum-larder predictor is barely more accurate than the ratio alone, so it is not claimed to explain the outcome.")
}

// ------------------------------------------------- 5. scatter withstands loss

fn withstands_claim(seeds: &[u64]) -> Outcome {
    let g = grid(LarderWeight::PerBurrow, seeds);
    let r = all_runs(&g);
    let rest: Vec<&Run> = r.iter().copied().filter(|x| !x.takeover()).collect();
    let ups: Vec<&Run> = r.iter().copied().filter(|x| x.takeover()).collect();
    let line = |rr: &[&Run]| {
        let early = col(rr.iter().copied(), |x| x.scatter(EARLY));
        let all = col(rr.iter().copied(), |x| x.scatter_all);
        format!(
            "generations 1–10 median {} (mean {}), generations 1–60 median {} (mean {}); pooled 1–10 {}; larder loss over generations 1–60 {}",
            medp(&early),
            pct(mean_or_nan(&stats::finite(&early))),
            medp(&all),
            pct(mean_or_nan(&stats::finite(&all))),
            medp(&col(rr.iter().copied(), |x| x.pooled(false))),
            medp(&col(rr.iter().copied(), |x| x.larder_all))
        )
    };
    let by_ratio: Vec<String> = RATIOS
        .iter()
        .map(|&q| {
            let rr: Vec<&Run> = g
                .iter()
                .filter(|c| c.ratio == q)
                .flat_map(|c| c.runs.iter())
                .filter(|x| !x.takeover())
                .collect();
            format!(
                "{q:.2}: {} ({} runs)",
                medp(&col(rr.iter().copied(), |x| x.scatter_all)),
                rr.len()
            )
        })
        .collect();
    reported(format!(
        "Mean per-agent daily scatter loss in the {} grid runs without takeover (per burrow): {}; against V&J's {:.0} %",
        rest.len(),
        line(&rest),
        100.0 * VJ_WITHSTAND
    ))
    .with(&format!(
        "In the {} takeover runs: {}. Without takeover, by ratio (generations 1–60): {}.",
        ups.len(),
        line(&ups),
        by_ratio.join("; ")
    ))
    .with("Vander Wall and Jenkins: \"the average daily rate of loss of scatter hoards was 18% in cases in which larder hoarding did not become established\" (p.663). Which generations they averaged is not stated, so both are given.")
}

// ------------------------------------------------------------- 6. visibility

fn visibility_claim(seeds: &[u64]) -> Outcome {
    let g = grid(LarderWeight::PerBurrow, seeds);
    let rows: Vec<String> = LARDS
        .iter()
        .map(|&l| {
            let row: Vec<String> = g
                .iter()
                .filter(|c| c.lard == l)
                .map(|c| {
                    format!(
                        "app_scat {:.2}: {}",
                        c.ratio * l,
                        pct(frac(
                            c.runs.iter().filter(|x| x.takeover()).count(),
                            c.runs.len()
                        ))
                    )
                })
                .collect();
            format!("app_lard {l}: {}", row.join(", "))
        })
        .collect();
    let same_ratio: Vec<String> = [0.15, 0.2, 0.22, 0.25, 0.3]
        .iter()
        .map(|&q| {
            let row: Vec<String> = g
                .iter()
                .filter(|c| c.ratio == q)
                .map(|c| {
                    format!(
                        "app_lard {} {}",
                        c.lard,
                        pct(frac(
                            c.runs.iter().filter(|x| x.takeover()).count(),
                            c.runs.len()
                        ))
                    )
                })
                .collect();
            format!("{q:.2}: {}", row.join(", "))
        })
        .collect();
    reported(format!(
        "Takeover share by app_scat (per burrow, 50 runs a cell): {}",
        rows.join("; ")
    ))
    .with(&format!(
        "At the same ratio, by app_lard (does the ratio alone set the outcome?): {}.",
        same_ratio.join("; ")
    ))
    .with("Vander Wall and Jenkins, untested there: \"the prevalence of scatter hoarding in communities should depend on environmental conditions that affect the apparency of scattered caches and larders\" (p.663).")
}

// --------------------------------------------------------- 7. owner recovery

fn summary_line(label: &str, r: &[Run]) -> String {
    let rr: Vec<&Run> = r.iter().collect();
    format!(
        "{label}: {}; window L {}; rise {}; survival per generation {}; starved per run {}; preyed per run {}; mean D (window) {}; larder share (window) {}",
        fates(&rr),
        med(&col(r, |x| x.window_l)),
        med(&col(r, |x| opt_u(x.rise))),
        med(&col(r, Run::survival_mean)),
        med(&col(r, |x| x.starved)),
        med(&col(r, |x| x.preyed)),
        med(&col(r, Run::d_window)),
        med(&col(r, Run::share_window)),
    )
}

fn recovery_claim(seeds: &[u64]) -> Outcome {
    let seeds = seeds_for(seeds);
    let mut configs = Vec::new();
    for &q in &RECOVERY_RATIOS {
        for &o in &RECOVERIES {
            configs.push(HoardConfig {
                owner_recovery: o,
                ..world(q, 2.0, LarderWeight::PerBurrow)
            });
        }
    }
    let out = batch(&configs, &seeds);
    let rows: Vec<String> = configs
        .iter()
        .zip(&out)
        .map(|(c, r)| {
            let tries: f64 = r.iter().map(|x| x.tries).sum();
            let misses: f64 = r.iter().map(|x| x.misses).sum();
            format!(
                "{}; tries missed {}",
                summary_line(
                    &format!(
                        "ratio {:.2}, owner_recovery {}",
                        c.app_scat / c.app_lard,
                        c.owner_recovery
                    ),
                    r
                ),
                if tries > 0.0 {
                    pct(misses / tries)
                } else {
                    "none tried".into()
                }
            )
        })
        .collect();
    reported(format!(
        "Owner recovery at ratios 0.1 and 0.22 (app_lard 2, per burrow, 50 runs each): {}",
        rows.join(". ")
    ))
    .with("New ground: Vander Wall and Jenkins give owners free recovery of their scattered caches (p.665: they \"were assumed to use one of these items\").")
}

// ---------------------------------------------------------------- 8. cheater

fn cheater_line(label: &str, r: &[Run]) -> String {
    let at = |g: usize| {
        let v: Vec<f64> = r
            .iter()
            .filter_map(|x| x.cheaters.get(g - 1).copied())
            .collect();
        format!("{g}: {}", pct(mean_or_nan(&v)))
    };
    let gens = [1, 2, 3, 5, 10, 20, 30, 40, 50, 60];
    let gone: Vec<f64> = col(r, |x| opt_u(x.cheaters_gone()));
    let never = r.iter().filter(|x| x.cheaters_gone().is_none()).count();
    // Survival while both types are present, per run.
    let both = |x: &Run, f: &dyn Fn(&Run, usize) -> f64| {
        let v: Vec<f64> = (0..x.cheaters.len())
            .filter(|&i| x.cheater_survival[i].is_finite() && x.hoarder_survival[i].is_finite())
            .map(|i| f(x, i))
            .collect();
        mean_or_nan(&v)
    };
    let cs = col(r, |x| both(x, &|x, i| x.cheater_survival[i]));
    let hs = col(r, |x| both(x, &|x, i| x.hoarder_survival[i]));
    let ahead = r
        .iter()
        .filter(|x| x.cheater_survival[0] > x.hoarder_survival[0])
        .count();
    format!(
        "{label}: cheater share born by generation (mean over runs) {}; the first generation with no cheater born {} (never in {never} of {}); survival in generation 1, cheaters {} against hoarders {} (cheaters ahead in {ahead} runs); survival over generations with both present, cheaters {} against hoarders {}; {}",
        gens.iter().map(|&g| at(g)).collect::<Vec<_>>().join(", "),
        med(&gone),
        r.len(),
        med(&col(r, |x| x.cheater_survival[0])),
        med(&col(r, |x| x.hoarder_survival[0])),
        med(&cs),
        med(&hs),
        summary_line("fates on the hoarders' L", r),
    )
}

fn cheater_claim(seeds: &[u64]) -> Outcome {
    let seeds = seeds_for(seeds);
    let base = hoard_preset("hoard-cheaters");
    let configs = vec![
        base.clone(),
        HoardConfig {
            cheater_fitness: CheaterFitness::Survival,
            ..base
        },
        hoard_preset("hoard-threshold"),
    ];
    let out = batch(&configs, &seeds);
    reported(format!(
        "hoard-cheaters (a quarter of the founders, per burrow, 50 runs each). {}. {}",
        cheater_line("cheater_fitness stores", &out[0]),
        cheater_line("cheater_fitness survival", &out[1]),
    ))
    .with(&format!(
        "Without cheaters, hoard-threshold: {}.",
        summary_line("no cheaters", &out[2])
    ))
    .with("Vander Wall and Jenkins, untested there: \"It is conceivable that under ideal conditions (e.g., mild winters), a nonhoarding cheater could survive and even flourish at the expense of conspecific hoarders.\" (p.661)")
}

// ------------------------------------------------------------ 9. sensitivity

fn sensitivity_claim(seeds: &[u64]) -> Outcome {
    let seeds = seeds_for(seeds);
    let mut combos = Vec::new();
    for &s in &SLOPES {
        for &v in &VSEGS {
            combos.push((s, v));
        }
    }
    let configs: Vec<HoardConfig> = combos
        .iter()
        .flat_map(|&(s, v)| SENS_RATIOS.iter().map(move |&q| sens(s, v, q)))
        .collect();
    let out = batch(&configs, &seeds);
    let rows: Vec<String> = combos
        .iter()
        .enumerate()
        .map(|(i, &(s, v))| {
            let runs = &out[i * SENS_RATIOS.len()..(i + 1) * SENS_RATIOS.len()];
            let cells: Vec<Cell> = SENS_RATIOS
                .iter()
                .zip(runs)
                .map(|(&ratio, r)| Cell {
                    lard: 2.0,
                    ratio,
                    runs: r.clone(),
                })
                .collect();
            let all: Vec<&Run> = all_runs(&cells);
            let takes: Vec<String> = cells
                .iter()
                .map(|c| {
                    format!(
                        "{:.1} {}/{}",
                        c.ratio,
                        c.runs.iter().filter(|x| x.takeover()).count(),
                        c.runs.len()
                    )
                })
                .collect();
            let above = all
                .iter()
                .filter(|x| x.larder_above())
                .count();
            format!(
                "defense_slope {s}, v_seg {v}{}: takeovers {}; fit {}; {}; mean D (window) {}; larder loss above scatter in {above} of {} runs; mean L in generation 1 {}",
                if s == 10.0 && v == 0.5 { " (the defaults)" } else { "" },
                takes.join(", "),
                fit_cells(&cells).text(),
                fates(&all),
                med(&col(all.iter().copied(), Run::d_window)),
                all.len(),
                med(&col(all.iter().copied(), |x| x.l[0])),
            )
        })
        .collect();
    reported(format!(
        "Sensitivity (app_lard 2, per burrow, 50 runs a cell, ratios 0.1, 0.2, 0.3 and 0.4): {}",
        rows.join(". ")
    ))
    .with("Both are gaps the paper leaves (amendments item 3 for the slope, item 8 for V_seg: \"a moderate amount of genetic variation\", p.662); reported as a sensitivity, never tuned.")
}

// ---------------------------------------------------------------- 10. presets

fn presets_claim(seeds: &[u64]) -> Outcome {
    let seeds = seeds_for(seeds);
    let configs: Vec<HoardConfig> = PRESETS.iter().map(|id| hoard_preset(id)).collect();
    let out = batch(&configs, &seeds);
    let rows: Vec<String> = PRESETS
        .iter()
        .zip(&out)
        .map(|(id, r)| {
            format!(
                "{}; generations 1–10 larder loss {}, scatter loss {}; mean L in generation 10 {}; larder loss above scatter in {} of {} runs",
                summary_line(id, r),
                medp(&col(r, |x| x.larder(EARLY))),
                medp(&col(r, |x| x.scatter(EARLY))),
                med(&col(r, |x| x.l.get(9).copied().unwrap_or(f64::NAN))),
                r.iter().filter(|x| x.larder_above()).count(),
                r.len()
            )
        })
        .collect();
    reported(format!(
        "The five presets, 50 runs each, 60 generations: {}",
        rows.join(". ")
    ))
}

pub fn claims() -> Vec<Claim> {
    vec![
        Claim {
            id: "hoard-threshold.all-or-nothing",
            item: "hoard-threshold",
            source: Source::Book,
            citation: "Vander Wall & Jenkins 2003, p.663",
            text: "Larder hoarding is all-or-nothing: across the ratio grid every run ends with mean L below 0.2 or above 0.95, and most takeovers rise within 10 generations (larders weighted per burrow, the default)",
            check: |s| all_or_nothing(LarderWeight::PerBurrow, s),
        },
        Claim {
            id: "hoard-threshold.all-or-nothing-per-item",
            item: "hoard-threshold",
            source: Source::Book,
            citation: "Vander Wall & Jenkins 2003, p.663; the spec's contradiction 2",
            text: "Larder hoarding is all-or-nothing, with larders weighted per item (the Appendix's other reading, reported as a failure to reproduce)",
            check: |s| all_or_nothing(LarderWeight::PerItem, s),
        },
        Claim {
            id: "hoard-threshold.threshold",
            item: "hoard-threshold",
            source: Source::Book,
            citation: "Vander Wall & Jenkins 2003, pp.662–663, Fig. 2B",
            text: "The outcome turns on app_scat ÷ app_lard: a logistic fit's 50 % point near 0.219, no takeover below 0.2 and takeover almost always above 0.3 (per burrow)",
            check: |s| threshold(LarderWeight::PerBurrow, s),
        },
        Claim {
            id: "hoard-threshold.threshold-per-item",
            item: "hoard-threshold",
            source: Source::Book,
            citation: "Vander Wall & Jenkins 2003, pp.662–663, Fig. 2B; the spec's contradiction 2",
            text: "The threshold, with larders weighted per item (reported as a failure to reproduce)",
            check: |s| threshold(LarderWeight::PerItem, s),
        },
        Claim {
            id: "hoard-threshold.larder-loss",
            item: "hoard-threshold",
            source: Source::Book,
            citation: "Vander Wall & Jenkins 2003, p.663",
            text: "Larders lose food faster than scattered caches: the mean larder loss exceeds the mean scatter loss in generations 1–10 in every run (per burrow; the CV ratio reported against 57/33)",
            check: larder_loss_claim,
        },
        Claim {
            id: "hoard-threshold.predictor",
            item: "hoard-threshold",
            source: Source::Book,
            citation: "Vander Wall & Jenkins 2003, p.663, Fig. 3",
            text: "The best early larder predicts takeover: \"min larder loss in generations 1–10 below the mean scatter loss\" (exposure floor 1 item) predicts takeover better than comparing means (per burrow)",
            check: |s| predictor_claim(LarderWeight::PerBurrow, s),
        },
        Claim {
            id: "hoard-threshold.predictor-per-item",
            item: "hoard-threshold",
            source: Source::Book,
            citation: "Vander Wall & Jenkins 2003, p.663, Fig. 3; the spec's contradiction 2",
            text: "The best early larder predicts takeover, with larders weighted per item (reported as a failure to reproduce)",
            check: |s| predictor_claim(LarderWeight::PerItem, s),
        },
        Claim {
            id: "hoard-threshold.predictor-baselines",
            item: "hoard-threshold",
            source: Source::Comment,
            citation: SPEC,
            text: "The predictor beside baselines: a ratio-only rule (takeover at ratio ≥ 0.25) and always-no (reported, fix round 1)",
            check: baselines_claim,
        },
        Claim {
            id: "hoard-threshold.rise",
            item: "hoard-threshold",
            source: Source::Book,
            citation: "Vander Wall & Jenkins 2003, p.663, Fig. 2A",
            text: "\"Usually within 10 generations\", two readings: by generation 10, and within 10 generations of lifting above 0.2, per cell and by V_seg (reported, fix round 1)",
            check: rise_claim,
        },
        Claim {
            id: "hoard-scatter.withstands",
            item: "hoard-scatter",
            source: Source::Book,
            citation: "Vander Wall & Jenkins 2003, p.663",
            text: "Scatter hoarding withstands about 18 % loss a day: the mean daily scatter loss in runs without takeover (reported, not judged)",
            check: withstands_claim,
        },
        Claim {
            id: "hoard-scatter.visibility",
            item: "hoard-scatter",
            source: Source::Book,
            citation: "Vander Wall & Jenkins 2003, p.663 (untested there)",
            text: "Lower visibility of scattered caches favors scatter hoarding: takeover by app_scat at each app_lard (new ground, reported)",
            check: visibility_claim,
        },
        Claim {
            id: "hoard-no-free-recovery.recovery",
            item: "hoard-no-free-recovery",
            source: Source::Comment,
            citation: SPEC,
            text: "Owners who must search for their own scattered caches: takeover and survival at owner_recovery 0.25, 0.5, 0.75 and 1 (new ground, reported)",
            check: recovery_claim,
        },
        Claim {
            id: "hoard-cheaters.cheater",
            item: "hoard-cheaters",
            source: Source::Book,
            citation: "Vander Wall & Jenkins 2003, p.661 (untested there)",
            text: "A non-hoarding cheater: its share over generations and its survival against hoarders, with fitness as stores and as survival (new ground, reported)",
            check: cheater_claim,
        },
        Claim {
            id: "hoard-threshold.sensitivity",
            item: "hoard-threshold",
            source: Source::Comment,
            citation: SPEC,
            text: "The threshold's sensitivity to the gaps the paper leaves: defense_slope 4 and 10 by V_seg 0.25, 0.5 and 1 (reported)",
            check: sensitivity_claim,
        },
        Claim {
            id: "hoard-threshold.presets",
            item: "hoard-threshold",
            source: Source::Comment,
            citation: SPEC,
            text: "What each of the five hoard presets does, measured for its description and title (reported)",
            check: presets_claim,
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_worlds_are_the_presets() {
        let d = LarderWeight::PerBurrow;
        assert_eq!(world(0.22, 2.0, d), hoard_preset("hoard-threshold"));
        assert_eq!(world(0.1, 2.0, d), hoard_preset("hoard-scatter"));
        assert_eq!(world(0.4, 2.0, d), hoard_preset("hoard-larder"));
        assert_eq!(
            HoardConfig {
                owner_recovery: 0.5,
                ..world(0.22, 2.0, d)
            },
            hoard_preset("hoard-no-free-recovery")
        );
        assert_eq!(
            HoardConfig {
                cheaters: 0.25,
                ..HoardConfig::default()
            },
            hoard_preset("hoard-cheaters")
        );
        let cs = cells();
        assert_eq!(cs.len(), 27, "10 + 10 + 7 cells");
        for (l, r) in cs {
            let c = world(r, l, LarderWeight::PerItem);
            c.validate().expect("valid");
            assert!(c.app_scat >= 0.05 - 1e-12 && c.app_scat <= 0.9 + 1e-9);
        }
        assert_eq!(seeds_for(&[1, 2, 3]).len(), 50);
        assert_eq!(TICKS, 60 * 100 * 20);
    }

    #[test]
    fn the_logistic_recovers_a_known_fit() {
        // Proportions exactly on σ(−8 + 36.59 x), as weighted 0/1 rows.
        let mut xs = Vec::new();
        let mut ys = Vec::new();
        for i in 0..19 {
            let x = 0.05 * f64::from(i);
            let p = sigmoid(VJ_B0 + VJ_B1 * x);
            let ones = (p * 1000.0).round() as usize;
            for j in 0..1000 {
                xs.push(x);
                ys.push(if j < ones { 1.0 } else { 0.0 });
            }
        }
        let f = logistic(&xs, &ys);
        assert!((f.b0 - VJ_B0).abs() < 0.1, "{f:?}");
        assert!((f.b1 - VJ_B1).abs() < 0.5, "{f:?}");
        assert!((f.x50 - 0.2186).abs() < 0.002, "{f:?}");
        assert!(f.se_x50 > 0.0 && f.se_x50 < 0.01, "{f:?}");
        assert!(f.rho2 > 0.0 && f.rho2 < 1.0);
        assert!(!f.separated);
        assert_eq!(x50_verdict(&f), Verdict::Holds);
        // Complete separation between 0.2 and 0.25: the 50 % point lands in
        // the gap.
        let xs = [0.1, 0.15, 0.2, 0.25, 0.3, 0.35];
        let ys = [0.0, 0.0, 0.0, 1.0, 1.0, 1.0];
        let f = logistic(&xs, &ys);
        assert!(f.separated);
        assert!(f.x50 > 0.2 && f.x50 < 0.25, "{f:?}");
        assert_eq!(x50_verdict(&f), Verdict::Holds);
        let off = Fit {
            x50: 0.27,
            b1: 10.0,
            ..f
        };
        assert_eq!(x50_verdict(&off), Verdict::Weak);
        let far = Fit {
            x50: 0.4,
            b1: 10.0,
            ..f
        };
        assert_eq!(x50_verdict(&far), Verdict::Fails);
        let down = Fit {
            x50: 0.22,
            b1: -1.0,
            ..f
        };
        assert_eq!(x50_verdict(&down), Verdict::Fails);
    }

    #[test]
    fn the_log_likelihood_is_stable() {
        let ll = log_lik(&[0.0, 1.0], &[1.0, 0.0], 0.0, 0.0);
        assert!((ll - 2.0 * 0.5f64.ln()).abs() < 1e-12);
        let ll = log_lik(&[1.0], &[1.0], 0.0, 1000.0);
        assert!(ll.is_finite() && ll.abs() < 1e-9, "{ll}");
        let ll = log_lik(&[1.0], &[0.0], 0.0, 1000.0);
        assert!(ll.is_finite() && ll < -900.0, "{ll}");
    }

    #[test]
    fn the_judges_bands() {
        assert_eq!(at_least(1.0, 1.0, CLEAN_WEAK), Verdict::Holds);
        assert_eq!(at_least(0.96, 1.0, CLEAN_WEAK), Verdict::Weak);
        assert_eq!(at_least(0.9, 1.0, CLEAN_WEAK), Verdict::Fails);
        assert_eq!(at_most(0.0, 0.0, LOW_WEAK), Verdict::Holds);
        assert_eq!(at_most(0.04, 0.0, LOW_WEAK), Verdict::Weak);
        assert_eq!(at_most(0.06, 0.0, LOW_WEAK), Verdict::Fails);
        assert_eq!(at_least(0.9, HIGH_HOLDS, HIGH_WEAK), Verdict::Holds);
        assert_eq!(at_least(0.8, HIGH_HOLDS, HIGH_WEAK), Verdict::Weak);
        assert_eq!(cv(&[1.0, 3.0]), 0.5);
        assert!(cv(&[]).is_nan());
    }

    fn fake(fate: Fate, losses: &[(u32, Option<f64>, Option<f64>, bool)]) -> Run {
        Run {
            fate,
            window_l: 0.0,
            rise: None,
            lift: None,
            generations: 60,
            l: vec![0.1],
            d: vec![0.5],
            survival: vec![1.0],
            cheaters: vec![0.0],
            cheater_survival: vec![f64::NAN],
            hoarder_survival: vec![1.0],
            larder_share: vec![0.1],
            starved: 0.0,
            preyed: 0.0,
            early: losses
                .iter()
                .map(|&(generation, larder, scatter, exposed)| Loss {
                    generation,
                    larder,
                    scatter,
                    exposed,
                    larder_lost: 0,
                    larder_stock: 0,
                    scatter_lost: 0,
                    scatter_stock: 0,
                })
                .collect(),
            larder_all: 0.0,
            scatter_all: 0.0,
            tries: 0.0,
            misses: 0.0,
        }
    }

    #[test]
    fn the_predictors_follow_their_definitions() {
        // Mean larder (1.0 + 3.0 + 0.1) / 3 above mean scatter 0.3, but one
        // agent at the floor loses 0.1 < 0.3; an unexposed agent at 0 is
        // ignored at the floor.
        let r = fake(
            Fate::Takeover,
            &[
                (1, Some(1.0), Some(0.2), false),
                (2, Some(3.0), Some(0.4), true),
                (3, Some(0.1), None, true),
                (4, Some(0.0), None, false),
                (11, Some(0.0), Some(9.0), true),
            ],
        );
        assert_eq!(r.min_larder(EARLY, true), 0.1);
        assert_eq!(r.min_larder(EARLY, false), 0.0);
        assert!((r.scatter(EARLY) - 0.3).abs() < 1e-12);
        assert!(r.predicts_min(EARLY));
        assert!(!r.predicts_means(EARLY));
        assert!(
            !r.predicts_min(1),
            "generation 1 alone: no agent at the floor"
        );
        assert_eq!(r.exposed(EARLY), 2);
        let low = fake(Fate::StayedLow, &[(1, Some(2.0), Some(0.3), true)]);
        let rs = [&r, &low];
        let (acc_min, acc_means, ppv, pos) = predictor_numbers(&rs, EARLY);
        assert_eq!((acc_min, acc_means, ppv, pos), (1.0, 0.5, 1.0, 1));
    }
}
