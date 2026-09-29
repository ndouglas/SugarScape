//! Minds 5: caching for the future
//! (docs/superpowers/specs/2026-09-29-minds-5-caching-design.md).
//!
//! Pairing as in Minds 3 and 4: within-world comparisons are paired by
//! construction, and every configuration compared runs the same seeds, judged
//! on the per-seed (or, in the labs, per-agent) differences. The judges and
//! thresholds are Minds 3's and 4's, fixed here before any run:
//!
//! - **Raby** (`cache-raby.*`): per rule, a population of 8 agents per seed
//!   in each counterbalancing (breakfast first and breakfast second, pooled:
//!   8 × 2 × the seeds), `lab::run_population` (each agent's λ drawn; the
//!   share is drawn but unused in the lab). Holds when the no-breakfast
//!   compartment's caches exceed the breakfast compartment's for at least
//!   80 % of agents (`paired_greater` over agents).
//! - **Amodio Experiment 2** (`cache-amodio.*`): per rule, 6 agents per seed
//!   in each group (Food-First, Empty-First). Three parts, the worst verdict
//!   wins (`all_of`):
//!   - each group's modal pattern (below) is the rule's oracle, in at least
//!     80 % of seeds;
//!   - the Bayesian model comparison of the paper's Experiment 2 (below), on
//!     each seed's 12 agents, gives the rule's own hypothesis the highest
//!     posterior of the five (compartment-independent, compartment-dependent
//!     unconstrained, FPH 1, FPH 2, CCH; each hypothesis summing its
//!     bird-dependent and bird-independent variants), in at least 80 % of
//!     seeds. `even` is the compartment-independent model's, `compensate`
//!     CCH's, `plan` with lookahead 1 FPH 1's and with lookahead 3 FPH 2's.
//! - **Winter** (`cache-winter-*`): 20 seeds, 1 000 ticks (five winters:
//!   ticks 100–199, 300–399, …, 900–999). Survival through the first winter
//!   is alive at 200 ÷ alive at 100; after five winters, alive at 1000 ÷
//!   alive at 100. Each rule against `none`, and `plan` against `even` and
//!   `compensate`: `paired_greater` over seeds (the difference above 0 in at
//!   least 80 % of seeds), first winter and five winters, both parts
//!   (`all_of`).
//! - **Central place, distance** (`central-distance.slope`): central-near's
//!   world with the homes `d` columns from the patches (placement.x = 45 −
//!   d, d = 4, 8, …, 24, the sweep's world). Per seed, the slope of
//!   `mean_load` (the cumulative mean per trip, mean over ticks 900–1000) on
//!   d. Holds when the slope is above 0 in at least 80 % of seeds (`range`,
//!   as Minds 4's travel claim).
//! - **Central place, Lima** (`central-near.lima`): one habitat with two
//!   identical patches, one 8 and one 20 columns from the same homes (the
//!   near and far presets' distances; [`lima_world`]). Per seed, the mean
//!   delivered load of trips from the near patch against trips from the far
//!   one (a trip belongs to the patch that gave over half its gross load;
//!   a seed needs at least 3 trips of each, else it has no value). **The
//!   tolerance:** |near − far| ≤ 10 % of the seed's mean delivered load over
//!   its near and far trips. Holds when that is true in at least 80 % of
//!   seeds (`range` on (near − far) ÷ mean, in [−0.10, 0.10]).
//!
//! Everything else is reported, not judged: every survival and wealth figure
//! per founding agent (the dead as 0), burying and digging per world, the
//! share of buried sugar never dug, `cache_lost`, the mechanisms named in
//! the claims, linear loading, trip endings and GOAP's loads.
//!
//! **Patterns** (Amodio). An agent's pattern is the set of compartments
//! whose caches are within 1 unit of its largest (whole units put
//! remainders in K order, so 10/10/10 and 11/10/9 are both "all three"),
//! or "nothing" when it cached nothing. The oracles are the hand-derived
//! predictions the lab's tests check: `even` all three in both groups;
//! `compensate` K2 (Food-First) and K1 with K3 (Empty-First); `plan`
//! lookahead 1: K1 and nothing (tomorrow has food); lookahead 3: K1 with K3
//! and K2.
//!
//! **The model comparison** is the paper's (Amodio et al. 2021, Methods,
//! Experiment 2), written from its text: each agent's counts are multinomial
//! over K1–K3; nine models, a uniform prior over them; each model's rates
//! integrated out under a flat Dirichlet prior (α = 1), truncated to the
//! hypothesis's region for the constrained ones:
//!
//! - compartment-independent: rates 1/3 (one model);
//! - compartment-dependent, bird-dependent (each agent its own rates) and
//!   bird-independent (rates shared);
//! - FPH 1: Food-First r_K1 ≥ r_K2, r_K3; Empty-First r_K2 ≥ r_K1, r_K3;
//! - FPH 2: Food-First r_K2 ≤ r_K1, r_K3; Empty-First r_K2 ≥ r_K1, r_K3;
//! - CCH: Food-First r_K2 ≥ r_K1, r_K3; Empty-First r_K2 ≤ r_K1, r_K3;
//!
//! each constrained one bird-dependent and bird-independent. The paper
//! quotes the constraints: "rbK1 ≥ rbK2 and rbK1 ≥ rbK3 for Prediction 1 of
//! the Future Planning Hypothesis for birds in the Food-First group, rbK2 ≤
//! rbK1 and rbK2 ≤ rbK3 for Prediction 2 of the Future Planning Hypothesis
//! for birds in the Food-First group and for the Compensatory Caching
//! Hypothesis for birds in the Empty-First group, rbK2 ≥ rbK1 and rbK2 ≥
//! rbK3 for the Compensatory Caching Hypothesis for birds in the Food-First
//! group and for the Future Planning Hypotheses (both variants) for birds in
//! the Empty-First group." One reading the text leaves open: a
//! bird-independent model's rates are shared **within each group** (the
//! constraints differ by group, so one vector can't serve both). With that
//! reading the comparison on the paper's own Table 2 gives 0.72 for the
//! compartment-independent model, 0.16 for CCH and 0.002 for each FPH, the
//! published figures (a test below); the published CCH and FPH figures are
//! single models' posteriors, the bird-independent variants. The judge sums
//! each hypothesis's two variants; the detail reports all nine models.

use std::collections::{BTreeMap, HashMap};
use std::sync::Mutex;

use sugarscape_core::agent::AgentId;
use sugarscape_core::config::{
    CachingRule, Config, DecisionRule, Lab, LabProtocol, Map, Peak, Placement,
};
use sugarscape_core::geometry::{Pos, Torus};
use sugarscape_core::landscape::patch_of;
use sugarscape_core::minds::caching::lab::{run_population, LabParams, LabResult};
use sugarscape_core::world::World;

use crate::claim::{all_of, equivalent, range, Claim, Outcome, Source};
use crate::claims::minds1::slope;
use crate::claims::minds2::paired_greater;
use crate::claims::minds3::{med_or_nan, q_or_nan};
use crate::claims::minds4::list;
use crate::runner::{each_seed, preset, series, window_mean};
use crate::stats::{self, ln_gamma, mean, median};

const SPEC: &str = "docs/superpowers/specs/2026-09-29-minds-5-caching-design.md";

/// Mean ± s.e.m. of `v`.
fn mean_sem(v: &[f64]) -> String {
    if v.is_empty() {
        return "none".into();
    }
    let m = mean(v);
    let sem = if v.len() < 2 {
        f64::NAN
    } else {
        let var = v.iter().map(|x| (x - m).powi(2)).sum::<f64>() / (v.len() as f64 - 1.0);
        (var / v.len() as f64).sqrt()
    };
    format!("{m:.2} ± {sem:.2}")
}

fn pct(n: usize, d: usize) -> f64 {
    if d == 0 {
        f64::NAN
    } else {
        100.0 * n as f64 / d as f64
    }
}

/// Torus Manhattan distance.
fn dist(t: Torus, a: Pos, b: Pos) -> u32 {
    let d = |p: u32, q: u32, n: u32| {
        let d = p.abs_diff(q);
        d.min(n - d)
    };
    d(a.x, b.x, t.width) + d(a.y, b.y, t.height)
}

// ---------------------------------------------------------------------- Raby

/// Agents per Raby population (Raby et al.'s eight birds).
const RABY_N: u32 = 8;

/// A seed's Amodio populations: (Food-First, Empty-First).
type Groups = (Vec<LabResult>, Vec<LabResult>);
/// Cached winter runs: (preset, seeds, runs).
type WinterCache = Vec<(String, Vec<u64>, Vec<WinterRun>)>;

/// One agent's test evening in Raby's protocol.
struct RabyAgent {
    no_breakfast: f64,
    breakfast: f64,
    breakfast_first: bool,
}

/// Every agent of a population of [`RABY_N`] per seed, in both
/// counterbalancings (`food_first`: breakfast in K1 and on the first
/// morning; otherwise in K3, second).
fn raby(rule: CachingRule, seeds: &[u64]) -> Vec<RabyAgent> {
    let mut out = Vec::new();
    for &seed in seeds {
        for first in [true, false] {
            let lab = Lab {
                protocol: LabProtocol::Raby,
                food_first: first,
            };
            let (bf, no) = if first { (0, 2) } else { (2, 0) };
            for r in run_population(lab, rule, LabParams::default(), RABY_N, seed) {
                out.push(RabyAgent {
                    no_breakfast: f64::from(r.caches[no]),
                    breakfast: f64::from(r.caches[bf]),
                    breakfast_first: first,
                });
            }
        }
    }
    out
}

fn raby_order(v: &[&RabyAgent], label: &str) -> String {
    let no: Vec<f64> = v.iter().map(|a| a.no_breakfast).collect();
    let bf: Vec<f64> = v.iter().map(|a| a.breakfast).collect();
    let d: Vec<f64> = no.iter().zip(&bf).map(|(a, b)| a - b).collect();
    let nothing = v
        .iter()
        .filter(|a| a.no_breakfast == 0.0 && a.breakfast == 0.0)
        .count();
    format!(
        "{label} ({} agents): no-breakfast {}, breakfast {} (mean ± s.e.m.); difference above 0 for {}, 0 for {}, below 0 for {}; caching nothing at all: {} ({:.1} %)",
        v.len(),
        mean_sem(&no),
        mean_sem(&bf),
        d.iter().filter(|&&x| x > 0.0).count(),
        d.iter().filter(|&&x| x == 0.0).count(),
        d.iter().filter(|&&x| x < 0.0).count(),
        nothing,
        pct(nothing, v.len()),
    )
}

fn raby_claim(rule: CachingRule, seeds: &[u64]) -> Outcome {
    let v = raby(rule, seeds);
    let no: Vec<f64> = v.iter().map(|a| a.no_breakfast).collect();
    let bf: Vec<f64> = v.iter().map(|a| a.breakfast).collect();
    let d: Vec<f64> = no.iter().zip(&bf).map(|(a, b)| a - b).collect();
    let sd = {
        let m = mean(&d);
        (d.iter().map(|x| (x - m).powi(2)).sum::<f64>() / (d.len() as f64 - 1.0)).sqrt()
    };
    let t = mean(&d) / (sd / (d.len() as f64).sqrt());
    let first: Vec<&RabyAgent> = v.iter().filter(|a| a.breakfast_first).collect();
    let second: Vec<&RabyAgent> = v.iter().filter(|a| !a.breakfast_first).collect();
    paired_greater(&no, &bf, "no-breakfast caches", "breakfast caches")
        .with(&format!(
            "Per agent: {RABY_N} agents per seed × {} seeds × 2 orders = {} agents, each with its own λ drawn in [0.3, 0.7) (share drawn, unused in the lab), given 30 sugar on the test evening.",
            seeds.len(),
            v.len()
        ))
        .with(&format!(
            "Pooled: no-breakfast {}, breakfast {} (mean ± s.e.m.), paired t = {t:.2} (df {}); Raby et al.: 16.3 ± 1.8 against 5.4 ± 1.8, paired t₇ = 3.01. {}. {}.",
            mean_sem(&no),
            mean_sem(&bf),
            d.len() - 1,
            raby_order(&first, "Breakfast first (K1, first morning)"),
            raby_order(&second, "Breakfast second (K3)"),
        ))
}

// -------------------------------------------------------------------- Amodio

/// Agents per Amodio population and group (the paper tested six birds).
const AMODIO_N: u32 = 6;

/// A rule arm of the Amodio claims.
#[derive(Clone, Copy)]
struct Arm {
    rule: CachingRule,
    lookahead: u32,
    /// Oracle patterns (Food-First, Empty-First), as [`pattern`] masks.
    oracle: (u8, u8),
    /// The hypothesis whose signature the rule is built to leave.
    own: Hyp,
}

const EVEN: Arm = Arm {
    rule: CachingRule::Even,
    lookahead: 1,
    oracle: (0b111, 0b111),
    own: Hyp::Independent,
};
const COMPENSATE: Arm = Arm {
    rule: CachingRule::Compensate,
    lookahead: 1,
    oracle: (0b010, 0b101),
    own: Hyp::Cch,
};
const PLAN_1: Arm = Arm {
    rule: CachingRule::Plan,
    lookahead: 1,
    oracle: (0b001, 0b000),
    own: Hyp::Fph1,
};
const PLAN_3: Arm = Arm {
    rule: CachingRule::Plan,
    lookahead: 3,
    oracle: (0b101, 0b010),
    own: Hyp::Fph2,
};

/// An agent's pattern: bit k set when compartment K(k+1)'s caches are
/// within 1 unit of the largest; 0 when it cached nothing.
fn pattern(c: [u32; 3]) -> u8 {
    let max = *c.iter().max().expect("three compartments");
    if max == 0 {
        return 0;
    }
    (0..3)
        .filter(|&k| c[k] + 1 >= max)
        .fold(0, |m, k| m | 1 << k)
}

fn pattern_name(p: u8) -> String {
    if p == 0 {
        return "nothing".into();
    }
    (0..3)
        .filter(|k| p & (1 << k) != 0)
        .map(|k| format!("K{}", k + 1))
        .collect::<Vec<_>>()
        .join("+")
}

/// The most common pattern among `agents` (ties to the lower mask), and
/// whether it was tied.
fn modal(agents: &[LabResult]) -> (u8, bool) {
    let mut counts: BTreeMap<u8, usize> = BTreeMap::new();
    for a in agents {
        *counts.entry(pattern(a.caches)).or_default() += 1;
    }
    let best = counts.values().copied().max().unwrap_or(0);
    let tops: Vec<u8> = counts
        .iter()
        .filter(|(_, &n)| n == best)
        .map(|(&p, _)| p)
        .collect();
    (tops[0], tops.len() > 1)
}

/// A seed's populations: (Food-First, Empty-First).
fn amodio(arm: Arm, seed: u64) -> Groups {
    let params = LabParams {
        lookahead: arm.lookahead,
        ..LabParams::default()
    };
    let run = |first| {
        let lab = Lab {
            protocol: LabProtocol::Amodio,
            food_first: first,
        };
        run_population(lab, arm.rule, params, AMODIO_N, seed)
    };
    (run(true), run(false))
}

/// The hypotheses (and the unconstrained compartment-dependent model).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Hyp {
    Independent,
    Dependent,
    Fph1,
    Fph2,
    Cch,
}

const HYPS: [Hyp; 5] = [
    Hyp::Independent,
    Hyp::Dependent,
    Hyp::Fph1,
    Hyp::Fph2,
    Hyp::Cch,
];

fn hyp_name(h: Hyp) -> &'static str {
    match h {
        Hyp::Independent => "compartment-independent",
        Hyp::Dependent => "compartment-dependent (unconstrained)",
        Hyp::Fph1 => "FPH 1",
        Hyp::Fph2 => "FPH 2",
        Hyp::Cch => "CCH",
    }
}

/// A constrained region of the rates: compartment k largest, or smallest.
#[derive(Clone, Copy, Debug)]
enum Region {
    Max(usize),
    Min(usize),
}

/// The paper's constraint for `h` in a group (`food_first`).
fn region(h: Hyp, food_first: bool) -> Region {
    match (h, food_first) {
        (Hyp::Fph1, true) => Region::Max(0),
        (Hyp::Fph2, true) => Region::Min(1),
        (Hyp::Cch, true) => Region::Max(1),
        (Hyp::Fph1 | Hyp::Fph2, false) => Region::Max(1),
        (Hyp::Cch, false) => Region::Min(1),
        _ => unreachable!("only the hypotheses are constrained"),
    }
}

/// Whether counts `c` satisfy `r` (ties satisfy it, as ≥ and ≤ do).
fn satisfies(c: [u32; 3], r: Region) -> bool {
    match r {
        Region::Max(k) => (0..3).all(|j| c[k] >= c[j]),
        Region::Min(k) => (0..3).all(|j| c[k] <= c[j]),
    }
}

/// ln of the regularized lower and upper incomplete gamma, (ln P(a, x),
/// ln Q(a, x)), by the series for x < a + 1 and the continued fraction
/// otherwise (Numerical Recipes' gser and gcf), each in log space.
fn ln_gamma_pq(a: f64, x: f64) -> (f64, f64) {
    if x <= 0.0 {
        return (f64::NEG_INFINITY, 0.0);
    }
    let front = -x + a * x.ln() - ln_gamma(a);
    if x < a + 1.0 {
        let (mut ap, mut del) = (a, 1.0 / a);
        let mut sum = del;
        for _ in 0..100_000 {
            ap += 1.0;
            del *= x / ap;
            sum += del;
            if del.abs() < sum.abs() * 1e-16 {
                break;
            }
        }
        let lp = front + sum.ln();
        (lp, (-lp.exp()).ln_1p())
    } else {
        let tiny = 1e-300;
        let mut b = x + 1.0 - a;
        let mut c = 1.0 / tiny;
        let mut d = 1.0 / b;
        let mut h = d;
        for i in 1..100_000 {
            let an = -f64::from(i) * (f64::from(i) - a);
            b += 2.0;
            d = an * d + b;
            if d.abs() < tiny {
                d = tiny;
            }
            c = b + an / c;
            if c.abs() < tiny {
                c = tiny;
            }
            d = 1.0 / d;
            let del = d * c;
            h *= del;
            if (del - 1.0).abs() < 1e-16 {
                break;
            }
        }
        let lq = front + h.ln();
        ((-lq.exp()).ln_1p(), lq)
    }
}

/// ln P(rates in `r`) under Dirichlet(`alpha`), by the gamma representation
/// (rates ∝ independent Gamma(α_k, 1) draws): ∫ f_{α_k}(x) Π_{j≠k} F_j(x) dx,
/// F_j the CDF for the largest and the survival function for the smallest,
/// by Simpson's rule in log space over [0, max_k(α_k + 20√α_k + 30)] with a
/// step of a fortieth of the smallest √α (at most 400 000 intervals).
fn ln_region_prob(alpha: [f64; 3], r: Region) -> f64 {
    let (k, largest) = match r {
        Region::Max(k) => (k, true),
        Region::Min(k) => (k, false),
    };
    let top = alpha
        .iter()
        .map(|a| a + 20.0 * a.sqrt() + 30.0)
        .fold(0.0, f64::max);
    let h0 = alpha.iter().map(|a| a.sqrt()).fold(f64::INFINITY, f64::min) / 40.0;
    let mut n = ((top / h0).ceil() as usize).clamp(2_000, 400_000);
    n += n % 2;
    let h = top / n as f64;
    let ak = alpha[k];
    let log_f = |x: f64| {
        if x <= 0.0 {
            return if ak == 1.0 { 0.0 } else { f64::NEG_INFINITY };
        }
        (ak - 1.0) * x.ln() - x - ln_gamma(ak)
    };
    let terms: Vec<f64> = (0..=n)
        .map(|i| {
            let x = i as f64 * h;
            let w = if i == 0 || i == n {
                1.0
            } else if i % 2 == 1 {
                4.0
            } else {
                2.0
            };
            let mut g = log_f(x) + (w * h / 3.0).ln();
            for (j, &aj) in alpha.iter().enumerate() {
                if j != k && g.is_finite() {
                    let (lp, lq) = ln_gamma_pq(aj, x);
                    g += if largest { lp } else { lq };
                }
            }
            g
        })
        .collect();
    let m = terms.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    if !m.is_finite() {
        return f64::NEG_INFINITY;
    }
    m + terms.iter().map(|g| (g - m).exp()).sum::<f64>().ln()
}

fn ln_coef(n: [u32; 3]) -> f64 {
    let total: u32 = n.iter().sum();
    ln_gamma(f64::from(total) + 1.0) - n.iter().map(|&c| ln_gamma(f64::from(c) + 1.0)).sum::<f64>()
}

/// ln ∫ Π r_c^{n_c} Dir(r; 1, 1, 1) dr = ln 2 + Σ ln n_c! − ln (N + 2)!.
fn ln_dir_int(n: [u32; 3]) -> f64 {
    let total: u32 = n.iter().sum();
    std::f64::consts::LN_2 + n.iter().map(|&c| ln_gamma(f64::from(c) + 1.0)).sum::<f64>()
        - ln_gamma(f64::from(total) + 3.0)
}

/// ln of the posterior-to-prior ratio of region `r` given counts `n`: the
/// flat prior puts 1/3 on each region used here (one compartment largest,
/// or smallest).
fn ln_region_ratio(n: [u32; 3], r: Region) -> f64 {
    let alpha = n.map(|c| f64::from(c) + 1.0);
    ln_region_prob(alpha, r) + 3f64.ln()
}

/// The paper's nine-model comparison on `data` (group Food-First?, counts
/// K1–K3 per agent): each model's hypothesis, whether its rates are
/// bird-dependent, and its posterior (a uniform prior over the nine).
fn models(data: &[(bool, [u32; 3])]) -> Vec<(Hyp, bool, f64)> {
    let coef: f64 = data.iter().map(|(_, n)| ln_coef(*n)).sum();
    let group = |ff: bool| {
        data.iter()
            .filter(|(g, _)| *g == ff)
            .fold([0u32; 3], |t, (_, n)| {
                [t[0] + n[0], t[1] + n[1], t[2] + n[2]]
            })
    };
    let (tf, te) = (group(true), group(false));
    let total_n: u32 = data.iter().map(|(_, n)| n.iter().sum::<u32>()).sum();
    // (hypothesis, bird-dependent, ln marginal) for each of the nine.
    let mut models: Vec<(Hyp, bool, f64)> = vec![
        (
            Hyp::Independent,
            false,
            coef + f64::from(total_n) * (1.0f64 / 3.0).ln(),
        ),
        (
            Hyp::Dependent,
            true,
            coef + data.iter().map(|(_, n)| ln_dir_int(*n)).sum::<f64>(),
        ),
        (
            Hyp::Dependent,
            false,
            coef + ln_dir_int(tf) + ln_dir_int(te),
        ),
    ];
    for h in [Hyp::Fph1, Hyp::Fph2, Hyp::Cch] {
        let bird: f64 = data
            .iter()
            .map(|(g, n)| ln_dir_int(*n) + ln_region_ratio(*n, region(h, *g)))
            .sum();
        models.push((h, true, coef + bird));
        let shared = [(true, tf), (false, te)]
            .iter()
            .map(|&(g, t)| ln_dir_int(t) + ln_region_ratio(t, region(h, g)))
            .sum::<f64>();
        models.push((h, false, coef + shared));
    }
    let m = models.iter().map(|x| x.2).fold(f64::NEG_INFINITY, f64::max);
    let z: f64 = models.iter().map(|x| (x.2 - m).exp()).sum();
    models
        .into_iter()
        .map(|(h, b, l)| (h, b, (l - m).exp() / z))
        .collect()
}

/// The posterior of each of [`HYPS`]: the constrained hypotheses and the
/// unconstrained dependent model each sum their bird-dependent and
/// bird-independent variants.
fn compare(data: &[(bool, [u32; 3])]) -> [f64; 5] {
    let mut out = [0.0; 5];
    for (h, _, p) in models(data) {
        out[HYPS.iter().position(|&x| x == h).expect("a hypothesis")] += p;
    }
    out
}

/// The nine models' posteriors, for the detail.
fn model_posteriors(data: &[(bool, [u32; 3])]) -> String {
    models(data)
        .iter()
        .map(|(h, bird, p)| {
            let v = match (h, bird) {
                (Hyp::Independent, _) => "",
                (_, true) => ", bird-dependent",
                (_, false) => ", bird-independent",
            };
            format!("{}{v} {p:.3e}", hyp_name(*h))
        })
        .collect::<Vec<_>>()
        .join("; ")
}

fn winner(post: &[f64; 5]) -> Hyp {
    let i = (0..5)
        .max_by(|&a, &b| post[a].total_cmp(&post[b]).then(b.cmp(&a)))
        .expect("five");
    HYPS[i]
}

fn posteriors(post: &[f64; 5]) -> String {
    HYPS.iter()
        .zip(post)
        .map(|(h, p)| format!("{} {p:.3e}", hyp_name(*h)))
        .collect::<Vec<_>>()
        .join(", ")
}

/// Per-group counts of agents satisfying each hypothesis's constraint.
fn constraint_shares(ff: &[LabResult], ef: &[LabResult]) -> String {
    [Hyp::Fph1, Hyp::Fph2, Hyp::Cch]
        .iter()
        .map(|&h| {
            let n = |v: &[LabResult], g| {
                v.iter()
                    .filter(|a| satisfies(a.caches, region(h, g)))
                    .count()
            };
            format!(
                "{}: Food-First {} of {}, Empty-First {} of {}",
                hyp_name(h),
                n(ff, true),
                ff.len(),
                n(ef, false),
                ef.len()
            )
        })
        .collect::<Vec<_>>()
        .join("; ")
}

fn pattern_counts(v: &[LabResult]) -> String {
    let mut counts: BTreeMap<u8, usize> = BTreeMap::new();
    for a in v {
        *counts.entry(pattern(a.caches)).or_default() += 1;
    }
    counts
        .iter()
        .map(|(p, n)| format!("{} {n}", pattern_name(*p)))
        .collect::<Vec<_>>()
        .join(", ")
}

fn mean_counts(v: &[LabResult]) -> String {
    let m = |k: usize| v.iter().map(|a| f64::from(a.caches[k])).sum::<f64>() / v.len() as f64;
    format!("{:.1}/{:.1}/{:.1}", m(0), m(1), m(2))
}

fn amodio_claim(arm: Arm, seeds: &[u64]) -> Outcome {
    let runs: Vec<Groups> = seeds.iter().map(|&s| amodio(arm, s)).collect();
    let flag = |b: bool| if b { 1.0 } else { 0.0 };
    let mut ties = 0;
    let mut modal_flags = |pick: fn(&Groups) -> &Vec<LabResult>, want: u8| {
        runs.iter()
            .map(|r| {
                let (m, tied) = modal(pick(r));
                ties += usize::from(tied);
                flag(m == want)
            })
            .collect::<Vec<f64>>()
    };
    let ff_flags = modal_flags(|r| &r.0, arm.oracle.0);
    let ef_flags = modal_flags(|r| &r.1, arm.oracle.1);
    let data = |r: &Groups| {
        r.0.iter()
            .map(|a| (true, a.caches))
            .chain(r.1.iter().map(|a| (false, a.caches)))
            .collect::<Vec<_>>()
    };
    let per_seed: Vec<[f64; 5]> = runs.iter().map(|r| compare(&data(r))).collect();
    let win_flags: Vec<f64> = per_seed
        .iter()
        .map(|p| flag(winner(p) == arm.own))
        .collect();
    let own = HYPS.iter().position(|&h| h == arm.own).expect("own");
    let own_post: Vec<f64> = per_seed.iter().map(|p| p[own]).collect();
    let winners: BTreeMap<&str, usize> = per_seed.iter().fold(BTreeMap::new(), |mut m, p| {
        *m.entry(hyp_name(winner(p))).or_default() += 1;
        m
    });
    let all_ff: Vec<LabResult> = runs.iter().flat_map(|r| r.0.clone()).collect();
    let all_ef: Vec<LabResult> = runs.iter().flat_map(|r| r.1.clone()).collect();
    let pooled: Vec<(bool, [u32; 3])> = runs.iter().flat_map(data).collect();
    let pooled_post = compare(&pooled);
    // The paper's six birds are three per group: the first three agents of
    // each group, per seed, for the comparison at the paper's size.
    let six: Vec<[f64; 5]> = runs
        .iter()
        .map(|r| {
            let d: Vec<(bool, [u32; 3])> = r.0[..3]
                .iter()
                .map(|a| (true, a.caches))
                .chain(r.1[..3].iter().map(|a| (false, a.caches)))
                .collect();
            compare(&d)
        })
        .collect();
    let six_own: Vec<f64> = six.iter().map(|p| p[own]).collect();
    all_of(vec![
        (
            format!("Food-First modal pattern {}", pattern_name(arm.oracle.0)),
            range(&ff_flags, 1.0, 1.0, false),
        ),
        (
            format!("Empty-First modal pattern {}", pattern_name(arm.oracle.1)),
            range(&ef_flags, 1.0, 1.0, false),
        ),
        (
            format!("{} wins the comparison", hyp_name(arm.own)),
            range(&win_flags, 1.0, 1.0, false),
        ),
    ])
    .with(&format!(
        "{AMODIO_N} agents per group per seed, {} seeds, each agent's λ drawn in [0.3, 0.7), lookahead {}; flags per seed are 1 when the part holds. Modal ties (to the lower mask): {ties}. Patterns over all agents: Food-First {} (mean caches K1/K2/K3 {}); Empty-First {} (mean {}).",
        seeds.len(),
        arm.lookahead,
        pattern_counts(&all_ff),
        mean_counts(&all_ff),
        pattern_counts(&all_ef),
        mean_counts(&all_ef),
    ))
    .with(&format!(
        "Agents satisfying the paper's constraints: {}.",
        constraint_shares(&all_ff, &all_ef)
    ))
    .with(&format!(
        "The paper's Bayesian comparison (nine models, flat Dirichlet priors, uniform over models; bird-independent rates shared within each group), per seed on its 12 agents: winners {winners:?}; the rule's own hypothesis ({}) median posterior {:.4} (IQR {:.4}–{:.4}). At the paper's size (the first 3 agents of each group per seed): median {:.4}, winner the own hypothesis in {} of {} seeds. Pooled over all seeds ({} agents), by hypothesis: {}; by model: {}. Amodio et al.'s birds (by model): compartment-independent 0.72 (0.997 in Experiment 1), CCH 0.16, FPH 0.002.",
        hyp_name(arm.own),
        median(&own_post),
        stats::quantile(&own_post, 0.25),
        stats::quantile(&own_post, 0.75),
        median(&six_own),
        six.iter().filter(|p| winner(p) == arm.own).count(),
        six.len(),
        pooled.len(),
        posteriors(&pooled_post),
        model_posteriors(&pooled),
    ))
}

// -------------------------------------------------------------------- Winter

/// The winter worlds' run length: five summers and five winters.
const WINTER_TICKS: u64 = 1000;
const RULES: [CachingRule; 4] = [
    CachingRule::None,
    CachingRule::Even,
    CachingRule::Compensate,
    CachingRule::Plan,
];

fn rule_label(r: CachingRule) -> &'static str {
    match r {
        CachingRule::None => "none",
        CachingRule::Even => "even",
        CachingRule::Compensate => "compensate",
        CachingRule::Plan => "plan",
    }
}

/// What the survey keeps of an agent before a step.
struct Pre {
    pos: Pos,
    target: Option<Pos>,
    caches: BTreeMap<u32, f64>,
    hungry: bool,
}

/// One winter-world run.
#[derive(Clone, Default)]
struct WinterRun {
    /// Population at ticks 0, 100, …, 1000.
    pop: Vec<f64>,
    founders: f64,
    /// Σ holdings + caches of the living per founding agent (the dead as 0)
    /// at ticks 200 and 1000.
    wealth: [f64; 2],
    buried: f64,
    dug: f64,
    lost: f64,
    cached_end: f64,
    /// Agent-ticks on which an agent's cache at its site grew.
    burials: u64,
    digs: u64,
    dig_age_sum: u64,
    /// Sugar buried in summer 1 (steps from ticks 0–99) and summer 2
    /// (200–299); the population at each summer's start.
    summer_buried: [f64; 2],
    summer_pop: [f64; 2],
    /// Plan agents at tick 201: count with a winter record, their
    /// forecasts' sum and how many forecast under 10 (a tenth of the need).
    forecasts: (usize, f64, usize),
    /// Hungry agents' new choices of a cache: count, Σ distance to it, Σ
    /// distance to their nearest cache, how many chose their biggest.
    cache_choices: (u64, u64, u64, u64),
    /// Deaths of agents holding caches, and of those, heading for one.
    deaths_with_caches: (u64, u64),
    /// Per rule (none, even, compensate, plan): founders and alive at 200
    /// and 1000 (the mixed world; one rule elsewhere).
    by_rule: [[f64; 3]; 4],
}

impl WinterRun {
    fn first(&self) -> f64 {
        self.pop[2] / self.pop[1]
    }
    fn five(&self) -> f64 {
        self.pop[10] / self.pop[1]
    }
    fn founder(&self, i: usize) -> f64 {
        self.pop[i] / self.founders
    }
}

fn rule_index(r: CachingRule) -> usize {
    RULES.iter().position(|&x| x == r).expect("a rule")
}

fn winter_run(mut w: World) -> WinterRun {
    let torus = w.torus;
    let horizon = f64::from(w.config.goap.horizon);
    let gamma = w.config.seasons.period;
    let mut run = WinterRun {
        founders: w.population() as f64,
        pop: vec![w.population() as f64],
        ..WinterRun::default()
    };
    for a in w.agents() {
        run.by_rule[rule_index(a.caching_rule_or(&w.config))][0] += 1.0;
    }
    let wealth = |w: &World| -> f64 {
        w.agents()
            .map(|a| a.holdings[0] + a.caches.values().sum::<f64>())
            .sum()
    };
    while w.tick < WINTER_TICKS {
        let pre: HashMap<AgentId, Pre> = w
            .agents()
            .map(|a| {
                let r = f64::from(a.metabolism[0]) * horizon;
                (
                    a.id,
                    Pre {
                        pos: a.pos,
                        target: a.plan.target,
                        caches: a.caches.clone(),
                        hungry: !a.caches.is_empty() && a.holdings[0] < r / 2.0,
                    },
                )
            })
            .collect();
        let t0 = w.tick;
        w.step();
        let e = w.events();
        run.buried += e.buried;
        run.dug += e.dug;
        run.lost += e.cache_lost;
        run.digs += u64::from(e.digs);
        run.dig_age_sum += e.dig_ages_sum;
        for (s, from) in [(0, 0), (1, 200)] {
            if (from..from + 100).contains(&t0) {
                run.summer_buried[s] += e.buried;
            }
        }
        for a in w.agents() {
            let Some(p) = pre.get(&a.id) else { continue };
            let here = torus.index(a.pos) as u32;
            let was = p.caches.get(&here).copied().unwrap_or(0.0);
            if a.caches.get(&here).copied().unwrap_or(0.0) > was {
                run.burials += 1;
            }
            if let Some(t) = a.plan.target {
                let ti = torus.index(t) as u32;
                if p.hungry && a.plan.target != p.target && p.caches.contains_key(&ti) {
                    let nearest = p
                        .caches
                        .keys()
                        .map(|&i| dist(torus, p.pos, torus.pos(i as usize)))
                        .min()
                        .expect("has caches");
                    let biggest = p.caches.values().copied().fold(0.0, f64::max);
                    let c = &mut run.cache_choices;
                    c.0 += 1;
                    c.1 += u64::from(dist(torus, p.pos, t));
                    c.2 += u64::from(nearest);
                    c.3 += u64::from(p.caches[&ti] >= biggest);
                }
            }
        }
        for (id, p) in &pre {
            if w.agent(*id).is_none() && !p.caches.is_empty() {
                run.deaths_with_caches.0 += 1;
                let heading = p
                    .target
                    .is_some_and(|t| p.caches.contains_key(&(torus.index(t) as u32)));
                run.deaths_with_caches.1 += u64::from(heading);
            }
        }
        match w.tick {
            200 | 1000 => {
                let i = usize::from(w.tick == 1000);
                run.wealth[i] = wealth(&w) / run.founders;
                for a in w.agents() {
                    run.by_rule[rule_index(a.caching_rule_or(&w.config))][1 + i] += 1.0;
                }
            }
            201 => {
                for a in w.agents() {
                    if a.caching_rule_or(&w.config) != CachingRule::Plan {
                        continue;
                    }
                    if let Some(r) = &a.last_winter {
                        let f = r.forecast(gamma);
                        run.forecasts.0 += 1;
                        run.forecasts.1 += f;
                        run.forecasts.2 += usize::from(f < 10.0);
                    }
                }
            }
            _ => {}
        }
        if w.tick == 200 {
            run.summer_pop[1] = w.population() as f64;
        }
        if w.tick.is_multiple_of(100) {
            run.pop.push(w.population() as f64);
        }
    }
    run.summer_pop[0] = run.founders;
    run.cached_end = w.agents().map(|a| a.caches.values().sum::<f64>()).sum();
    run
}

/// The rule an agent follows (its own under `caching.mixed`).
trait RuleOf {
    fn caching_rule_or(&self, c: &Config) -> CachingRule;
}

impl RuleOf for sugarscape_core::agent::Agent {
    fn caching_rule_or(&self, c: &Config) -> CachingRule {
        if c.caching.mixed {
            self.caching_rule
        } else {
            c.caching.rule
        }
    }
}

/// Runs are shared by the claims (each preset once per seed list).
fn winter(id: &str, seeds: &[u64]) -> Vec<WinterRun> {
    static CACHE: Mutex<WinterCache> = Mutex::new(Vec::new());
    if let Some(hit) = CACHE
        .lock()
        .unwrap()
        .iter()
        .find(|(i, s, _)| i == id && s == seeds)
    {
        return hit.2.clone();
    }
    let runs = each_seed(&preset(id), seeds, winter_run);
    CACHE
        .lock()
        .unwrap()
        .push((id.to_string(), seeds.to_vec(), runs.clone()));
    runs
}

fn col(r: &[WinterRun], f: impl Fn(&WinterRun) -> f64) -> Vec<f64> {
    r.iter().map(f).collect()
}

fn med(v: &[f64]) -> String {
    format!(
        "{:.3} (IQR {:.3}–{:.3})",
        med_or_nan(v),
        q_or_nan(v, 0.25),
        q_or_nan(v, 0.75)
    )
}

/// A winter world's survival, wealth and usage, for the detail.
fn winter_report(id: &str, r: &[WinterRun]) -> String {
    let sum = |f: fn(&WinterRun) -> f64| r.iter().map(f).sum::<f64>();
    let (buried, dug, lost, left) = (
        sum(|x| x.buried),
        sum(|x| x.dug),
        sum(|x| x.lost),
        sum(|x| x.cached_end),
    );
    let digs: u64 = r.iter().map(|x| x.digs).sum();
    let ages: u64 = r.iter().map(|x| x.dig_age_sum).sum();
    let burials: u64 = r.iter().map(|x| x.burials).sum();
    let usage = if buried == 0.0 {
        "nothing buried".to_string()
    } else {
        format!(
            "burials {burials} agent-ticks and digs {digs} over {} seeds (per seed median {:.0} and {:.0}); buried {buried:.0}, dug {dug:.0} (recovery {:.3}), lost with the dead {lost:.0} ({:.1} %), still cached at tick 1000 {left:.0} ({:.1} %): never dug {:.1} %; mean cache age at digging {:.1} ticks",
            r.len(),
            median(&col(r, |x| x.burials as f64)),
            median(&col(r, |x| x.digs as f64)),
            dug / buried,
            100.0 * lost / buried,
            100.0 * left / buried,
            100.0 * (1.0 - dug / buried),
            ages as f64 / digs.max(1) as f64,
        )
    };
    let (n, dsum, nsum, big) = r.iter().fold((0, 0, 0, 0), |a, x| {
        let c = x.cache_choices;
        (a.0 + c.0, a.1 + c.1, a.2 + c.2, a.3 + c.3)
    });
    let choices = if n == 0 {
        "no hungry agent chose a cache".to_string()
    } else {
        format!(
            "hungry agents chose a cache {n} times: mean distance to it {:.1}, to their nearest cache {:.1}; their biggest cache {:.1} % of the time",
            dsum as f64 / n as f64,
            nsum as f64 / n as f64,
            pct(big as usize, n as usize)
        )
    };
    let (dw, dh) = r.iter().fold((0, 0), |a, x| {
        (a.0 + x.deaths_with_caches.0, a.1 + x.deaths_with_caches.1)
    });
    format!(
        "{id} (median over seeds): survival through the first winter {}, after five {}; per founding agent (the dead as 0) alive at 200 {}, at 1000 {}; sugar held and cached per founding agent at 200 {}, at 1000 {}. Usage: {usage}. Mechanism: {choices}; {dw} agents died holding caches, {dh} of them ({:.1} %) heading for one.",
        med(&col(r, WinterRun::first)),
        med(&col(r, WinterRun::five)),
        med(&col(r, |x| x.founder(2))),
        med(&col(r, |x| x.founder(10))),
        med(&col(r, |x| x.wealth[0])),
        med(&col(r, |x| x.wealth[1])),
        pct(dh as usize, dw as usize),
    )
}

/// `plan`'s burial in its second summer against its first, and its
/// forecasts after the first winter.
fn plan_report(r: &[WinterRun]) -> String {
    let per = |s: usize| col(r, |x| x.summer_buried[s] / x.summer_pop[s]);
    let (n, f, low) = r.iter().fold((0, 0.0, 0), |a, x| {
        (
            a.0 + x.forecasts.0,
            a.1 + x.forecasts.1,
            a.2 + x.forecasts.2,
        )
    });
    let ratio: Vec<f64> = r
        .iter()
        .map(|x| (x.summer_buried[1] / x.summer_pop[1]) / (x.summer_buried[0] / x.summer_pop[0]))
        .collect();
    format!(
        "Plan's forecast: buried per agent alive at the summer's start, summer 1 {}, summer 2 {} (ratio {}); at tick 201, {n} planners with a winter record forecast a mean {:.1} of winter intake from sites (need 100), {low} ({:.1} %) under 10.",
        med(&per(0)),
        med(&per(1)),
        med(&ratio),
        f / n.max(1) as f64,
        pct(low, n),
    )
}

/// The mixed world: each rule's survival and share of the survivors.
fn mixed_report(seeds: &[u64]) -> String {
    let r = winter("cache-winter-mixed", seeds);
    let rows: Vec<String> = RULES
        .iter()
        .enumerate()
        .map(|(k, &rule)| {
            let alive = |i: usize| col(&r, |x| x.by_rule[k][i] / x.by_rule[k][0]);
            let share = |i: usize| {
                col(&r, |x| {
                    let all: f64 = x.by_rule.iter().map(|b| b[i]).sum();
                    x.by_rule[k][i] / all
                })
            };
            format!(
                "{}: alive per founder at 200 {}, at 1000 {}; share of survivors at 200 {}, at 1000 {}",
                rule_label(rule),
                med(&alive(1)),
                med(&alive(2)),
                med(&share(1)),
                med(&share(2)),
            )
        })
        .collect();
    format!(
        "The mixed world (a quarter on each rule; medians over seeds): {}. {}",
        rows.join("; "),
        winter_report("cache-winter-mixed", &r)
    )
}

fn winter_vs(a: &str, b: &str, seeds: &[u64]) -> Vec<(String, Outcome)> {
    let (x, y) = (winter(a, seeds), winter(b, seeds));
    vec![
        (
            format!("first winter, {a} against {b}"),
            paired_greater(&col(&x, WinterRun::first), &col(&y, WinterRun::first), a, b),
        ),
        (
            format!("five winters, {a} against {b}"),
            paired_greater(&col(&x, WinterRun::five), &col(&y, WinterRun::five), a, b),
        ),
    ]
}

fn winter_claim(id: &str, seeds: &[u64]) -> Outcome {
    let (x, none) = (winter(id, seeds), winter("cache-winter-none", seeds));
    let mut out = all_of(winter_vs(id, "cache-winter-none", seeds))
        .with(&format!(
            "20 seeds as given, ticks 0–1000; survival through the first winter = alive at 200 ÷ alive at 100; after five = alive at 1000 ÷ alive at 100. Per seed, first winter: {} {}; none {}. Five winters: {} {}; none {}.",
            id,
            list(&col(&x, WinterRun::first), 3),
            list(&col(&none, WinterRun::first), 3),
            id,
            list(&col(&x, WinterRun::five), 3),
            list(&col(&none, WinterRun::five), 3),
        ))
        .with(&winter_report(id, &x))
        .with(&winter_report("cache-winter-none", &none));
    if id == "cache-winter-plan" {
        out = out.with(&plan_report(&x));
    }
    out
}

fn plan_best(seeds: &[u64]) -> Outcome {
    let mut parts = winter_vs("cache-winter-plan", "cache-winter-even", seeds);
    parts.extend(winter_vs(
        "cache-winter-plan",
        "cache-winter-compensate",
        seeds,
    ));
    all_of(parts).with(&mixed_report(seeds))
}

// ------------------------------------------------------------- Central place

/// The distances of the central-distance sweep (columns from home to the
/// patches).
const DISTANCES: [u32; 6] = [4, 8, 12, 16, 20, 24];
const CENTRAL_TICKS: u32 = 1000;

/// central-near's world with the homes `d` columns west of the patches.
fn at_distance(d: u32) -> Config {
    let mut c = preset("central-near");
    let Placement::Block { x, .. } = &mut c.placement else {
        unreachable!("central-near places a block")
    };
    *x = 45 - d;
    c
}

/// Lima's habitat: central-near's agents and rules, with homes in rows
/// 12–18 of column 25 and two identical patches (radius 3, height 4) on row
/// 15, one at column 33 (8 columns from home, the near preset's distance)
/// and one at column 45 (20, the far preset's).
fn lima_world() -> Config {
    let mut c = preset("central-near");
    c.placement = Placement::Block {
        x: 25,
        y: 12,
        width: 1,
        height: 7,
    };
    let peak = |x| Peak {
        x,
        y: 15,
        radius: 3.0,
        height: 4.0,
    };
    c.goods[0].map = Map::Peaks {
        peaks: vec![peak(33), peak(45)],
    };
    c
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum End {
    Full,
    Low,
    Rho,
    /// Not seen deciding to go home (GOAP; or the trip began before tick 0).
    Other,
}

#[derive(Clone, Copy)]
struct Trip {
    gross: f64,
    delivered: f64,
    /// Gross gathered in patch 0 (near in Lima's habitat) and patch 1.
    by_patch: [f64; 2],
    harvest_ticks: u32,
    end: End,
}

#[derive(Default)]
struct Current {
    by_patch: [f64; 2],
    harvest_ticks: u32,
    end: Option<End>,
}

/// One central-place run, traced.
#[derive(Default)]
struct CentralRun {
    trips: Vec<Trip>,
    alive: usize,
    /// `mean_load` over ticks 900–1000.
    mean_load: f64,
    /// ρ's change on delivery ticks, and ρ when a trip ended by ρ.
    rho_jumps: Vec<f64>,
    rho_at_leave: Vec<f64>,
}

struct CentralPre {
    pos: Pos,
    load: f64,
    held: f64,
    leaving: Option<Pos>,
    rho: f64,
}

fn central_run(mut w: World) -> CentralRun {
    let torus = w.torus;
    let cap = f64::from(w.config.caching.capacity);
    let peaks = match &w.config.goods[0].map {
        Map::Peaks { peaks } => peaks.clone(),
        _ => Vec::new(),
    };
    let (gw, gh) = (w.config.width, w.config.height);
    let mut out = CentralRun::default();
    let mut cur: HashMap<AgentId, Current> = HashMap::new();
    for _ in 0..CENTRAL_TICKS {
        let pre: HashMap<AgentId, CentralPre> = w
            .agents()
            .map(|a| {
                (
                    a.id,
                    CentralPre {
                        pos: a.pos,
                        load: a.load_trip,
                        held: a.holdings[0],
                        leaving: a.leaving,
                        rho: a.delivery_rate,
                    },
                )
            })
            .collect();
        w.step();
        for a in w.agents() {
            let Some(p) = pre.get(&a.id) else { continue };
            let home = a.home.expect("a central-place agent has a home");
            let r = f64::from(a.metabolism[0]);
            let c = cur.entry(a.id).or_default();
            let mut gained = a.load_trip - p.load;
            // At home a trip ends (the load is reset); one that brought
            // nothing home isn't a delivery.
            if p.pos == home {
                if p.load > 0.0 {
                    out.trips.push(Trip {
                        gross: p.load,
                        delivered: p.load.min(p.held - r).max(0.0),
                        by_patch: c.by_patch,
                        harvest_ticks: c.harvest_ticks,
                        end: c.end.unwrap_or(End::Other),
                    });
                    out.rho_jumps.push(a.delivery_rate - p.rho);
                }
                *c = Current::default();
                gained = a.load_trip;
            }
            if gained > 0.0 {
                c.harvest_ticks += 1;
                if let Some(k) = patch_of(&peaks, a.pos.x, a.pos.y, gw, gh) {
                    if k < 2 {
                        c.by_patch[k] += gained;
                    }
                }
            }
            if p.pos != home && p.leaving != Some(home) && a.leaving == Some(home) {
                let end = if cap > 0.0 && p.load + r >= cap {
                    End::Full
                } else if r > 0.0 && p.held <= r * f64::from(dist(torus, p.pos, home) + 1) {
                    End::Low
                } else {
                    out.rho_at_leave.push(p.rho);
                    End::Rho
                };
                c.end = Some(end);
            }
        }
    }
    out.alive = w.population();
    out.mean_load = window_mean(&series(&w, "mean_load"), 900, 1000);
    out
}

fn central(c: &Config, seeds: &[u64]) -> Vec<CentralRun> {
    each_seed(c, seeds, central_run)
}

/// Trip endings and loads over all seeds, for the detail.
fn trips_report(name: &str, runs: &[CentralRun]) -> String {
    let trips: Vec<&Trip> = runs.iter().flat_map(|r| &r.trips).collect();
    let n = trips.len();
    let count = |e: End| trips.iter().filter(|t| t.end == e).count();
    let gross: Vec<f64> = trips.iter().map(|t| t.gross).collect();
    let delivered: Vec<f64> = trips.iter().map(|t| t.delivered).collect();
    let short = trips.iter().filter(|t| t.harvest_ticks <= 2).count();
    format!(
        "{name}: {n} trips over {} seeds; ended by full {:.1} %, low (food for the walk home) {:.1} %, ρ or an empty site {:.1} %, not seen deciding {:.1} %; mean gross load {:.1}, delivered {:.1}; trips with at most 2 harvest ticks {:.1} %; mean_load (ticks 900–1000) median {:.1}; alive at tick {CENTRAL_TICKS}: {} of {}",
        runs.len(),
        pct(count(End::Full), n),
        pct(count(End::Low), n),
        pct(count(End::Rho), n),
        pct(count(End::Other), n),
        if n == 0 { f64::NAN } else { mean(&gross) },
        if n == 0 { f64::NAN } else { mean(&delivered) },
        pct(short, n),
        median(&runs.iter().map(|r| r.mean_load).collect::<Vec<_>>()),
        runs.iter().map(|r| r.alive).sum::<usize>(),
        5 * runs.len(),
    )
}

/// The pulse: ρ's jump at a delivery, against the best site's yield.
fn pulse_report(name: &str, runs: &[CentralRun]) -> String {
    let jumps: Vec<f64> = runs.iter().flat_map(|r| r.rho_jumps.clone()).collect();
    let at_leave: Vec<f64> = runs.iter().flat_map(|r| r.rho_at_leave.clone()).collect();
    let delivered: Vec<f64> = runs
        .iter()
        .flat_map(|r| r.trips.iter().map(|t| t.delivered))
        .collect();
    format!(
        "{name}'s ρ: jumps a mean {:.2} on a delivery tick (0.05 × the mean delivered load {:.1} = {:.2}; the best site yields at most 4 a tick); at the decision to leave on ρ or an empty site, ρ is a median {:.2} (IQR {:.2}–{:.2})",
        mean(&jumps),
        mean(&delivered),
        0.05 * mean(&delivered),
        med_or_nan(&at_leave),
        q_or_nan(&at_leave, 0.25),
        q_or_nan(&at_leave, 0.75),
    )
}

fn with_rule(mut c: Config, rule: DecisionRule) -> Config {
    c.decision.rule = rule;
    c
}

fn distance_claim(seeds: &[u64]) -> Outcome {
    let loads: Vec<Vec<f64>> = DISTANCES
        .iter()
        .map(|&d| {
            crate::runner::after(&at_distance(d), seeds, CENTRAL_TICKS, |w| {
                window_mean(&series(w, "mean_load"), 900, 1000)
            })
        })
        .collect();
    let x: Vec<f64> = DISTANCES.iter().map(|&d| f64::from(d)).collect();
    let slopes: Vec<f64> = (0..seeds.len())
        .map(|i| slope(&x, &loads.iter().map(|v| v[i]).collect::<Vec<_>>()))
        .collect();
    let by_d: Vec<String> = DISTANCES
        .iter()
        .zip(&loads)
        .map(|(d, v)| format!("{d}: {:.1}", med_or_nan(v)))
        .collect();
    let near = central(&preset("central-near"), seeds);
    let far = central(&preset("central-far"), seeds);
    let linear = central(&preset("central-linear"), seeds);
    let mut linear_far = preset("central-linear");
    let Placement::Block { x: hx, .. } = &mut linear_far.placement else {
        unreachable!("a block")
    };
    *hx = 25;
    let linear_far = central(&linear_far, seeds);
    let goap: Vec<String> = ["central-near", "central-far", "central-linear"]
        .iter()
        .map(|id| {
            trips_report(
                &format!("GOAP on {id}"),
                &central(&with_rule(preset(id), DecisionRule::Goap), seeds),
            )
        })
        .collect();
    range(&slopes, f64::MIN_POSITIVE, f64::INFINITY, false)
        .with(&format!(
            "central-near's world with homes d columns from the patches (placement.x = 45 − d), ticks 1–{CENTRAL_TICKS}; mean_load is Σ delivered ÷ Σ deliveries since tick 0, averaged over ticks 900–1000. Per-seed slope of load on d: finite in {} of {}, positive in {}, median {:.3} (IQR {:.3}–{:.3}); per seed {}. Median load by d: {}.",
            stats::finite(&slopes).len(),
            slopes.len(),
            slopes.iter().filter(|&&s| s > 0.0).count(),
            med_or_nan(&slopes),
            q_or_nan(&slopes, 0.25),
            q_or_nan(&slopes, 0.75),
            list(&slopes, 2),
            by_d.join(", "),
        ))
        .with(&format!(
            "Trip endings (the rule's own tests, read from state: full = load + R ≥ C; low = holdings ≤ R × (distance home + 1); otherwise ρ or an empty site). {}. {}. {}. {}.",
            trips_report("central-near (d 8)", &near),
            trips_report("central-far (d 20)", &far),
            trips_report("central-linear (d 8)", &linear),
            trips_report("linear loading at d 20", &linear_far),
        ))
        .with(&format!(
            "Linear loading, the pulse: {}. {}. {}.",
            pulse_report("central-linear", &linear),
            pulse_report("central-near", &near),
            pulse_report("central-far", &far),
        ))
        .with(&format!("Reported, GOAP's \"deliver G\" (G = 10): {}.", goap.join(". ")))
}

/// Per seed: (near − far) ÷ mean of the delivered loads of trips from each
/// patch (NaN with fewer than 3 of either), and the gross version.
fn lima_diffs(r: &CentralRun) -> (f64, f64, f64, f64, usize, usize) {
    let of = |k: usize| -> Vec<&Trip> {
        r.trips
            .iter()
            .filter(|t| t.gross > 0.0 && t.by_patch[k] > 0.5 * t.gross)
            .collect()
    };
    let (near, far) = (of(0), of(1));
    if near.len() < 3 || far.len() < 3 {
        return (
            f64::NAN,
            f64::NAN,
            f64::NAN,
            f64::NAN,
            near.len(),
            far.len(),
        );
    }
    let m = |v: &[&Trip], f: fn(&Trip) -> f64| v.iter().map(|t| f(t)).sum::<f64>() / v.len() as f64;
    let both: Vec<&Trip> = near.iter().chain(&far).copied().collect();
    let rel = |f: fn(&Trip) -> f64| (m(&near, f) - m(&far, f)) / m(&both, f);
    (
        rel(|t| t.delivered),
        rel(|t| t.gross),
        m(&near, |t| t.delivered),
        m(&far, |t| t.delivered),
        near.len(),
        far.len(),
    )
}

fn lima_claim(seeds: &[u64]) -> Outcome {
    let runs = central(&lima_world(), seeds);
    let d: Vec<(f64, f64, f64, f64, usize, usize)> = runs.iter().map(lima_diffs).collect();
    let rel: Vec<f64> = d.iter().map(|x| x.0).collect();
    let gross: Vec<f64> = d.iter().map(|x| x.1).collect();
    let near: Vec<f64> = d.iter().map(|x| x.2).collect();
    let far: Vec<f64> = d.iter().map(|x| x.3).collect();
    let trips: Vec<Trip> = runs.iter().flat_map(|r| r.trips.clone()).collect();
    let mixed = trips
        .iter()
        .filter(|t| t.by_patch[0] <= 0.5 * t.gross && t.by_patch[1] <= 0.5 * t.gross)
        .count();
    range(&rel, -0.10, 0.10, false)
        .with(&format!(
            "One habitat: homes in rows 12–18 of column 25, identical patches (radius 3, height 4) at column 33 (8 columns) and 45 (20), central-near's agents and rules otherwise; ticks 1–{CENTRAL_TICKS}. Per seed, (near − far) ÷ mean of the delivered loads: {}; near trips per seed {}; far trips per seed {}. Mean delivered load per seed, near {} (median {:.1}), far {} (median {:.1}). Trips from neither patch by more than half: {mixed} of {}.",
            list(&rel, 3),
            d.iter().map(|x| x.4.to_string()).collect::<Vec<_>>().join(" "),
            d.iter().map(|x| x.5.to_string()).collect::<Vec<_>>().join(" "),
            list(&near, 1),
            med_or_nan(&near),
            list(&far, 1),
            med_or_nan(&far),
            trips.len(),
        ))
        .with(&format!(
            "Reported: the same on gross loads, median {:.3} (IQR {:.3}–{:.3}); across seeds, {}.",
            med_or_nan(&gross),
            q_or_nan(&gross, 0.25),
            q_or_nan(&gross, 0.75),
            equivalent(&near, &far, None, "near", "far").measured,
        ))
        .with(&trips_report("Lima's habitat", &runs))
}

pub fn claims() -> Vec<Claim> {
    vec![
        Claim {
            id: "cache-raby.even",
            item: "cache-raby",
            source: Source::Book,
            citation: "Raby et al. 2007",
            text: "Planning for breakfast, under the even rule: agents cache more in the compartment that had no breakfast than in the one that had, agent by agent (the rule predicts a tie)",
            check: |seeds| raby_claim(CachingRule::Even, seeds),
        },
        Claim {
            id: "cache-raby.compensate",
            item: "cache-raby",
            source: Source::Book,
            citation: "Raby et al. 2007; Amodio et al. 2021",
            text: "Planning for breakfast, under the compensating rule: agents cache more in the compartment that had no breakfast than in the one that had, agent by agent",
            check: |seeds| raby_claim(CachingRule::Compensate, seeds),
        },
        Claim {
            id: "cache-raby.plan",
            item: "cache-raby",
            source: Source::Book,
            citation: "Raby et al. 2007",
            text: "Planning for breakfast, under the planning rule (lookahead 1): agents cache more in the compartment that had no breakfast than in the one that had, agent by agent",
            check: |seeds| raby_claim(CachingRule::Plan, seeds),
        },
        Claim {
            id: "cache-amodio.even",
            item: "cache-amodio",
            source: Source::Comment,
            citation: SPEC,
            text: "Amodio's Experiment 2 under the even rule: both groups cache evenly, and the paper's model comparison picks the compartment-independent model",
            check: |seeds| amodio_claim(EVEN, seeds),
        },
        Claim {
            id: "cache-amodio.compensate",
            item: "cache-amodio",
            source: Source::Comment,
            citation: SPEC,
            text: "Amodio's Experiment 2 under the compensating rule: Food-First caches most in K2 and Empty-First least in K2, and the paper's model comparison picks CCH",
            check: |seeds| amodio_claim(COMPENSATE, seeds),
        },
        Claim {
            id: "cache-amodio.plan-1",
            item: "cache-amodio",
            source: Source::Comment,
            citation: SPEC,
            text: "Amodio's Experiment 2 under the planning rule, lookahead 1: Food-First caches in K1 and Empty-First caches nothing, and the paper's model comparison picks FPH 1",
            check: |seeds| amodio_claim(PLAN_1, seeds),
        },
        Claim {
            id: "cache-amodio.plan-3",
            item: "cache-amodio",
            source: Source::Comment,
            citation: SPEC,
            text: "Amodio's Experiment 2 under the planning rule, lookahead 3: Food-First caches in K1 and K3 and Empty-First in K2, and the paper's model comparison picks FPH 2",
            check: |seeds| amodio_claim(PLAN_3, seeds),
        },
        Claim {
            id: "cache-winter-even.survival",
            item: "cache-winter-even",
            source: Source::Comment,
            citation: SPEC,
            text: "Burying an even share gets more agents through winter than burying nothing: through the first winter and after five, seed by seed",
            check: |seeds| winter_claim("cache-winter-even", seeds),
        },
        Claim {
            id: "cache-winter-compensate.survival",
            item: "cache-winter-compensate",
            source: Source::Comment,
            citation: SPEC,
            text: "Compensating gets more agents through winter than burying nothing: through the first winter and after five, seed by seed",
            check: |seeds| winter_claim("cache-winter-compensate", seeds),
        },
        Claim {
            id: "cache-winter-plan.survival",
            item: "cache-winter-plan",
            source: Source::Comment,
            citation: SPEC,
            text: "Planning gets more agents through winter than burying nothing: through the first winter and after five, seed by seed",
            check: |seeds| winter_claim("cache-winter-plan", seeds),
        },
        Claim {
            id: "cache-winter-plan.best",
            item: "cache-winter-plan",
            source: Source::Comment,
            citation: SPEC,
            text: "Planning gets more agents through winter than an even share and than compensating: through the first winter and after five, seed by seed (the mixed world reported)",
            check: plan_best,
        },
        Claim {
            id: "central-distance.slope",
            item: "central-distance",
            source: Source::Book,
            citation: "Orians & Pearson 1979; Stephens & Krebs 1986 §3.5",
            text: "Increasing distance from the central place is matched by increasing load size: the per-seed slope of mean load on distance is above 0 in at least 80 % of seeds (linear loading, trip endings and GOAP reported)",
            check: distance_claim,
        },
        Claim {
            id: "central-near.lima",
            item: "central-near",
            source: Source::Book,
            citation: "Lima 1983, in Stephens & Krebs 1986 §3.5",
            text: "In one habitat, a near and a far patch with the same loading curve give the same load: the near and far mean delivered loads within 10 % of the seed's mean, in at least 80 % of seeds",
            check: lima_claim,
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64, tol: f64) -> bool {
        (a - b).abs() <= tol
    }

    #[test]
    fn region_probabilities_match_symmetry_and_monte_carlo() {
        let p = |a: [f64; 3], r| ln_region_prob(a, r).exp();
        for k in 0..3 {
            assert!(close(p([1.0; 3], Region::Max(k)), 1.0 / 3.0, 1e-6));
            assert!(close(p([1.0; 3], Region::Min(k)), 1.0 / 3.0, 1e-6));
            assert!(close(p([11.0; 3], Region::Max(k)), 1.0 / 3.0, 1e-6));
        }
        // Monte Carlo, 10⁷ draws (numpy): Dir(3, 8, 2) and Dir(31, 1, 1).
        assert!(close(p([3.0, 8.0, 2.0], Region::Max(1)), 0.93065, 1e-3));
        assert!(close(p([3.0, 8.0, 2.0], Region::Max(0)), 0.05258, 1e-3));
        assert!(close(p([3.0, 8.0, 2.0], Region::Min(0)), 0.31029, 1e-3));
        assert!(close(p([3.0, 8.0, 2.0], Region::Min(1)), 0.00482, 5e-4));
        assert!(close(p([31.0, 1.0, 1.0], Region::Min(1)), 0.5, 1e-6));
        assert!(close(p([16.0, 1.0, 16.0], Region::Max(0)), 0.5, 1e-6));
        // A large pooled count stays finite and in [0, 1].
        let big = p([3601.0, 1.0, 1801.0], Region::Max(0));
        assert!(big > 0.999 && big <= 1.0 + 1e-9, "{big}");
        assert!(ln_region_prob([1.0, 3601.0, 1.0], Region::Max(0)).is_finite());
    }

    #[test]
    fn the_comparison_reproduces_the_papers_experiment_2() {
        // Table 2, sorted K1/K2/K3: Washington FF, Wellington EF, Caracas
        // FF, Rome EF, Lisbon EF, Quito FF. Published: 0.72, CCH 0.16,
        // FPH 0.002.
        let data = [
            (true, [3, 2, 2]),
            (false, [1, 0, 1]),
            (true, [8, 7, 7]),
            (false, [6, 3, 7]),
            (false, [0, 1, 7]),
            (true, [9, 8, 8]),
        ];
        // The published values are single models' posteriors (the
        // bird-independent variants of the hypotheses).
        let m = models(&data);
        let get = |h: Hyp, bird: bool| {
            m.iter()
                .find(|x| x.0 == h && (x.1 == bird || h == Hyp::Independent))
                .expect("a model")
                .2
        };
        assert_eq!(m.len(), 9);
        assert!(close(get(Hyp::Independent, false), 0.72, 0.005), "{m:?}");
        assert!(close(get(Hyp::Cch, false), 0.16, 0.005), "{m:?}");
        assert!((0.0015..0.0025).contains(&get(Hyp::Fph1, false)), "{m:?}");
        assert!((0.0015..0.0025).contains(&get(Hyp::Fph2, false)), "{m:?}");
        assert!(close(compare(&data).iter().sum::<f64>(), 1.0, 1e-9));
    }

    #[test]
    fn patterns_allow_a_unit_of_remainder() {
        assert_eq!(pattern([10, 10, 10]), 0b111);
        assert_eq!(pattern([11, 10, 9]), 0b011);
        assert_eq!(pattern([15, 0, 15]), 0b101);
        assert_eq!(pattern([8, 15, 7]), 0b010);
        assert_eq!(pattern([0, 0, 0]), 0);
        assert_eq!(pattern_name(0b101), "K1+K3");
        let r = |c| LabResult { caches: c };
        assert_eq!(
            modal(&[r([30, 0, 0]), r([30, 0, 0]), r([0, 0, 0])]),
            (0b001, false)
        );
        assert_eq!(modal(&[r([30, 0, 0]), r([0, 0, 0])]), (0, true));
    }

    #[test]
    fn the_oracles_are_the_labs_default_allocations() {
        for arm in [EVEN, COMPENSATE, PLAN_1, PLAN_3] {
            let (ff, ef) = amodio(arm, 1);
            assert_eq!(pattern(ff[0].caches), arm.oracle.0, "{:?}", arm.rule);
            assert_eq!(pattern(ef[0].caches), arm.oracle.1, "{:?}", arm.rule);
        }
    }

    #[test]
    fn the_constructed_worlds_validate() {
        assert_eq!(at_distance(8), preset("central-near"));
        assert_eq!(at_distance(20), preset("central-far"));
        let c = lima_world();
        c.validate().expect("Lima's habitat is valid");
        let Map::Peaks { peaks } = &c.goods[0].map else {
            panic!()
        };
        assert_eq!(
            peaks.iter().map(|p| p.x - 25).collect::<Vec<_>>(),
            [8, 20],
            "near and far from the homes' column"
        );
        with_rule(preset("central-near"), DecisionRule::Goap)
            .validate()
            .expect("GOAP central is valid");
    }
}
