//! Minds 6: theft
//! (docs/superpowers/specs/2026-09-30-minds-6-theft-design.md).
//!
//! Pairing as in Minds 3–5: every configuration compared runs the same seeds
//! and is judged on per-seed values or per-seed differences. Every run lasts
//! 200 ticks: a summer (ticks 0–99) and the first winter (100–199). The
//! judges and thresholds below were fixed before any run.
//!
//! **`find` is a free parameter.** The presets' 0.25 is a labeled anchor
//! (the smallest of 0.25, 0.3, 0.4 and 0.5 whose winter pilferage reached
//! 2 %, Task 5). Every claim is judged at the anchor and reported across the
//! sweep 0.02, 0.05, 0.1, 0.25, 0.5 and 1.
//!
//! **Measures** (per run):
//! - **Survival** of a group (hoarders, cheaters) is alive at 200 ÷ alive at
//!   100, both counted in that group: the population at tick 100, not the
//!   founders times the share. Each is reported per founding agent too
//!   (alive at 200 ÷ the group's founders, the dead as 0).
//! - **Wealth** of a group is Σ holdings + caches + stomach of its living
//!   members at tick 200 ÷ its founders (the dead as 0).
//! - **The pilferage rate** is Σ caches pilfered ÷ Σ caches at the ticks'
//!   starts, over ticks 1–200 (`caches_pilfered` and `pilfer_candidates`:
//!   a tick stands for a day). **v**, non-owner visits per cache per tick,
//!   is Σ find draws on other agents' caches (`pilfer_draws`: an arrival
//!   that didn't dig its own cache there, once per foreign cache on the
//!   site) ÷ the same Σ caches.
//! - **p_s and p_o** are amount-weighted, from the tick events (exact; the
//!   stats series' fate shares are the same sums): p_s = sugar dug by its
//!   owner ÷ (dug + pilfered), and p_o = pilfered ÷ (dug + pilfered + lost
//!   with a dead owner). Sugar still buried at tick 200 is **excluded**: it
//!   has met no fate yet, and counting it as neither recovered nor found
//!   would shrink both by the same unknown share. C/G is `caching.bury_cost`
//!   (sugar lost per unit buried, G the unit).
//! - **The fate log** gives cache ages at digging and pilfering (amount
//!   weighted) and the cohort fit. A run whose log reached its cap
//!   (`cache_log_full`) is skipped for those and counted.
//!
//! **Judges** (all over seeds 1–20 unless said):
//! 1. **Pilferage as a decomposition** (`theft-winter.pilferage`): at the
//!    anchor in theft-winter, per seed, rate ÷ (v × find) within
//!    [1 − 0.2, 1 + 0.2] in at least 80 % of seeds (`range`). The spec's
//!    cohort fit to (1 − r)^t and the literature's 2–30 % a day (median
//!    9 %) are reported as context, not judged.
//! 2. **Andersson and Krebs's threshold** (`theft-arena-4.threshold`): the
//!    arenas at the anchor, n = 2, 4 and 8 by bury cost 0, 0.1, 0.25, 0.5
//!    and 1 (15 cells × 20 seeds). Per run the condition is p_s ÷ p_o >
//!    (C/G)(n − 1) + 1 (p_o = 0 with p_s > 0 counts as holding; a run with
//!    no ended sugar has no value). **Fitness is wealth per founder at the
//!    winter's end** (tick 200), since in the arenas nearly every agent
//!    survives and survival can't discriminate. A run agrees when "hoarders
//!    are richer" (hoarder − cheater wealth > 0; a tie is not richer) equals
//!    the condition. Holds when at least 80 % of runs agree (`range` on
//!    flags).
//! 3. **Frequency independence** (`theft-cheaters.frequency`): theft-winter's
//!    world at the anchor with cheater shares 0.1, 0.2, …, 0.9. Per seed,
//!    the OLS slope of the hoarder advantage (hoarder − cheater survival)
//!    on the share. Holds when the 95 % t confidence interval of the mean
//!    per-seed slope includes 0; fails otherwise.
//! 4. **Equal recovery** (`theft-cheaters.equal-recovery`): the same worlds
//!    with `owner_memory: off`. At every share, cheater survival above
//!    hoarder survival in at least 80 % of seeds (`paired_greater`), all
//!    shares (`all_of`).
//! 5. **The mixed equilibrium** (`theft-cheaters.mixed`): claim 3's runs.
//!    Per seed, the advantage crosses zero in Andersson and Krebs's
//!    orientation (hoarders fitter when few: the advantage below 0 at share
//!    0.1 and above 0 at 0.9). Holds when that is so in at least 80 % of
//!    seeds; the crossing (the first sign change from below 0 to at least
//!    0, interpolated) is located per seed and reported.
//! 6. **Reciprocity** (`theft-winter.reciprocity`): theft-winter at the
//!    anchor, `keep` against `eat`, paired on seeds: the hoarders' caches at
//!    tick 100 per founding hoarder, and hoarder survival, each greater
//!    under `keep` in at least 80 % of seeds; both (`all_of`).
//! 7. **Loss that hoarding withstands** (`theft-winter-half.loss`): the
//!    configurations at the anchor (the winter field at each cheater share,
//!    and the arenas at each n and bury cost). Hoarders "win" a
//!    configuration when their fitness (survival in the field, wealth per
//!    founder in the arenas) exceeds the cheaters' in at least 80 % of
//!    seeds (`paired_greater` holds). Holds when some configuration whose
//!    median pilferage rate is at least 18 % a day has hoarders winning;
//!    fails when such configurations exist and in none do hoarders win;
//!    untestable when no configuration reaches 18 %.
//! 8. **The mild-winter cheater** (`theft-winter.mild`): theft-winter-half's
//!    world at the anchor with β = 2, 4, 8, 16, 32. Per seed, the OLS slope
//!    of the cheater advantage (cheater − hoarder wealth per founder at tick
//!    200) on log₂ β. Wealth, not survival, because in mild winters nearly
//!    everyone survives. The prediction (cheaters gain as winter softens) is
//!    a slope below 0; holds when it is in at least 80 % of seeds (`range`).
//! 9. **Usage** (`theft-winter-half.usage`): in each of the six Minds 6
//!    presets, some sugar is pilfered in at least 80 % of seeds (`range` on
//!    flags, `all_of` over presets). Everything else is reported: fate
//!    shares, p_s and p_o, when pilfering happens, cache ages, the log flag.
//!
//! Reported, not judged: the carried findings (each "likely" until
//! isolated), the dig-at-R probe (`World::probe_dig_at_reserve`: owners dig
//! below their whole reserve R instead of R / 2), and every claim's measure
//! across the find sweep.

use std::collections::{HashMap, HashSet};
use std::sync::Mutex;

use sugarscape_core::agent::{Agent, AgentId};
use sugarscape_core::config::{Config, Loot};
use sugarscape_core::minds::caching::fates::Fate;
use sugarscape_core::world::World;

use crate::claim::{all_of, range, untestable, Claim, Outcome, Source, Verdict};
use crate::claims::minds1::slope;
use crate::claims::minds2::paired_greater;
use crate::claims::minds3::{med_or_nan, q_or_nan};
use crate::claims::minds4::list;
use crate::claims::minds5::med;
use crate::runner::{each_seed, preset};
use crate::stats::{self, mean};

const SPEC: &str = "docs/superpowers/specs/2026-09-30-minds-6-theft-design.md";

/// The find sweep, and the presets' anchor.
const FINDS: [f64; 6] = [0.02, 0.05, 0.1, 0.25, 0.5, 1.0];
const ANCHOR: f64 = 0.25;
/// Cheater shares for claims 3–5.
const SHARES: [f64; 9] = [0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9];
/// Winter severities for claim 8.
const BETAS: [u32; 5] = [2, 4, 8, 16, 32];
/// Bury costs (C/G) and group sizes for claim 2.
const COSTS: [f64; 5] = [0.0, 0.1, 0.25, 0.5, 1.0];
const NS: [u32; 3] = [2, 4, 8];
/// Every run: a summer and the first winter.
const TICKS: u64 = 200;
/// Claim 1's tolerance on rate ÷ (v × find).
const DECOMP_TOL: f64 = 0.2;
/// Vander Wall and Jenkins's simulated daily loss (p.663).
const VJ_LOSS: f64 = 0.18;

/// Group indices: hoarders and cheaters.
const H: usize = 0;
const C: usize = 1;

// ------------------------------------------------------------------- the run

/// One run's measures (see the module's list).
#[derive(Clone, Default)]
struct Run {
    founders: [f64; 2],
    alive100: [f64; 2],
    alive200: [f64; 2],
    /// Σ holdings + caches + stomach of the living at 200, by group.
    wealth200: [f64; 2],
    /// Σ caches of the living at 100, by group.
    cached100: [f64; 2],
    /// Sugar pilfered over the run, by the thief's group.
    stolen: [f64; 2],
    buried: f64,
    dug: f64,
    pilfered: f64,
    lost: f64,
    bury_cost: f64,
    loot_eaten: f64,
    /// Pilfered and dug in summer (steps from ticks 0–99) and winter
    /// (100–199).
    pilfered_season: [f64; 2],
    dug_season: [f64; 2],
    caches_pilfered: f64,
    candidates: f64,
    draws: f64,
    /// Non-owner agents ending a tick on a cache's site, per cache there at
    /// the tick's start (whether they drew or dug).
    visits: f64,
    pilfers: f64,
    owner_finds: f64,
    /// Owner finds (under `owner_memory` off) in summer and winter.
    owner_finds_season: [f64; 2],
    /// Σ over ticks of the sugar cached at the tick's start.
    cached_at_start: f64,
    /// Deaths in steps from ticks 100–199.
    deaths_winter: f64,
    /// Living agent-ticks and open (non-wall) sites, for the density.
    agent_ticks: f64,
    open_sites: f64,
    /// The stats series' fate shares at 200 (dug, pilfered, lost, buried);
    /// NaN when theft is off.
    fates: [f64; 4],
    log_full: bool,
    /// From the log (NaN when full or empty): amount-weighted mean age at
    /// digging and at pilfering.
    age_dug: f64,
    age_pilfered: f64,
    /// The cohort fit: r and the RMS residual of the survival curve.
    km_r: f64,
    km_rms: f64,
    /// Reported (fix round 1): holdings + stomach, and caches, of the
    /// living at 200, by group (wealth with caches valued at 0, and the
    /// caches themselves).
    held200: [f64; 2],
    cached200: [f64; 2],
    /// Σ over arrivals of 1 − (1 − find)^k, k the foreign caches on the
    /// arrival's site at the tick's start: the takes predicted when an
    /// arrival takes at most one of the k caches on a site.
    pred_takes: f64,
    /// Foreign caches on arrivals' sites (Σ k), those on sites with k ≥ 2,
    /// and arrivals with k > 0 at the carrying limit at the tick's start.
    k_sum: f64,
    k_stacked: f64,
    full_arrivals: f64,
}

fn nan_div(a: f64, b: f64) -> f64 {
    if b == 0.0 {
        f64::NAN
    } else {
        a / b
    }
}

impl Run {
    fn surv(&self, g: usize) -> f64 {
        nan_div(self.alive200[g], self.alive100[g])
    }
    fn surv_f(&self, g: usize) -> f64 {
        nan_div(self.alive200[g], self.founders[g])
    }
    fn wealth_f(&self, g: usize) -> f64 {
        nan_div(self.wealth200[g], self.founders[g])
    }
    fn surv_all(&self) -> f64 {
        nan_div(
            self.alive200[H] + self.alive200[C],
            self.alive100[H] + self.alive100[C],
        )
    }
    fn surv_all_f(&self) -> f64 {
        nan_div(
            self.alive200[H] + self.alive200[C],
            self.founders[H] + self.founders[C],
        )
    }
    /// Hoarder − cheater survival.
    fn adv(&self) -> f64 {
        self.surv(H) - self.surv(C)
    }
    fn rate(&self) -> f64 {
        nan_div(self.caches_pilfered, self.candidates)
    }
    fn v(&self) -> f64 {
        nan_div(self.draws, self.candidates)
    }
    fn raw_v(&self) -> f64 {
        nan_div(self.visits, self.candidates)
    }
    fn p_s(&self) -> f64 {
        nan_div(self.dug, self.dug + self.pilfered)
    }
    fn p_o(&self) -> f64 {
        nan_div(self.pilfered, self.dug + self.pilfered + self.lost)
    }
    /// Sugar pilfered per sugar cached, per tick.
    fn loss(&self) -> f64 {
        nan_div(self.pilfered, self.cached_at_start)
    }
    /// Reported: holdings + stomach per founder at 200 (caches valued at 0).
    fn held_f(&self, g: usize) -> f64 {
        nan_div(self.held200[g], self.founders[g])
    }
    fn density(&self) -> f64 {
        nan_div(self.agent_ticks / TICKS as f64, self.open_sites)
    }
}

fn wealth(a: &Agent) -> f64 {
    a.holdings[0] + a.caches.values().sum::<f64>() + a.fed
}

/// Amount-weighted Kaplan–Meier survival of cached sugar against pilfering
/// (digging, loss with the owner and the run's end censor), by age, while
/// at least 5 % of the sugar is at risk; then the fit S(a) = (1 − r)^a by
/// least squares on ln S through the origin. Returns (r, RMS of S − fit).
fn cohort_fit(records: &[(u64, f64, bool)]) -> (f64, f64) {
    let mut v = records.to_vec();
    v.sort_by_key(|r| r.0);
    let total: f64 = v.iter().map(|r| r.1).sum();
    if total <= 0.0 {
        return (f64::NAN, f64::NAN);
    }
    let mut at_risk = total;
    let mut s = 1.0;
    let mut pts: Vec<(f64, f64)> = Vec::new();
    let mut i = 0;
    while i < v.len() {
        let age = v[i].0;
        let (mut d, mut all) = (0.0, 0.0);
        while i < v.len() && v[i].0 == age {
            all += v[i].1;
            if v[i].2 {
                d += v[i].1;
            }
            i += 1;
        }
        if at_risk < 0.05 * total {
            break;
        }
        s *= 1.0 - d / at_risk;
        at_risk -= all;
        if s > 0.0 && age > 0 {
            pts.push((age as f64, s));
        }
    }
    if pts.is_empty() {
        return (f64::NAN, f64::NAN);
    }
    let b = pts.iter().map(|(a, s)| a * s.ln()).sum::<f64>()
        / pts.iter().map(|(a, _)| a * a).sum::<f64>();
    let rms = (pts
        .iter()
        .map(|(a, s)| (s - (b * a).exp()).powi(2))
        .sum::<f64>()
        / pts.len() as f64)
        .sqrt();
    (1.0 - b.exp(), rms)
}

fn run(mut w: World, probe: bool) -> Run {
    w.probe_dig_at_reserve = probe;
    // The fate log is off unless asked for; the cohort fits read it.
    w.record_fates = true;
    let torus = w.torus;
    let group = |a: &Agent| usize::from(a.cheater);
    let find = w.config.theft.find;
    let cap = f64::from(w.config.caching.capacity);
    let mut r = Run {
        open_sites: w.sites.iter().filter(|s| s.capacity[0] > 0.0).count() as f64,
        ..Run::default()
    };
    for a in w.agents() {
        r.founders[group(a)] += 1.0;
    }
    while w.tick < TICKS {
        let t0 = w.tick;
        let season = usize::from(t0 >= 100);
        let mut per_site: HashMap<u32, u32> = HashMap::new();
        let mut own: HashSet<(AgentId, u32)> = HashSet::new();
        let mut stolen: HashMap<AgentId, f64> = HashMap::new();
        let mut held: HashMap<AgentId, f64> = HashMap::new();
        for a in w.agents() {
            for (&site, &q) in &a.caches {
                *per_site.entry(site).or_default() += 1;
                own.insert((a.id, site));
                r.cached_at_start += q;
            }
            stolen.insert(a.id, a.stolen_by_me);
            held.insert(a.id, a.holdings[0]);
        }
        w.step();
        let e = w.events();
        r.buried += e.buried;
        r.dug += e.dug;
        r.pilfered += e.pilfered;
        r.lost += e.cache_lost;
        r.bury_cost += e.bury_cost;
        r.loot_eaten += e.loot_eaten;
        r.pilfered_season[season] += e.pilfered;
        r.dug_season[season] += e.dug;
        r.caches_pilfered += f64::from(e.caches_pilfered);
        r.candidates += f64::from(e.pilfer_candidates);
        r.draws += f64::from(e.pilfer_draws);
        r.pilfers += f64::from(e.pilfers);
        r.owner_finds += f64::from(e.owner_finds);
        r.owner_finds_season[season] += f64::from(e.owner_finds);
        if season == 1 {
            r.deaths_winter += e.deaths.len() as f64;
        }
        for a in w.agents() {
            let Some(before) = stolen.get(&a.id) else {
                continue;
            };
            r.stolen[group(a)] += a.stolen_by_me - before;
            let site = torus.index(a.pos) as u32;
            let n = per_site.get(&site).copied().unwrap_or(0);
            let k = n - u32::from(own.contains(&(a.id, site)));
            r.visits += f64::from(k);
            if k > 0 {
                r.pred_takes += 1.0 - (1.0 - find).powi(k as i32);
                r.k_sum += f64::from(k);
                if k >= 2 {
                    r.k_stacked += f64::from(k);
                }
                if cap > 0.0 && held[&a.id] >= cap {
                    r.full_arrivals += 1.0;
                }
            }
        }
        r.agent_ticks += w.population() as f64;
        match w.tick {
            100 => {
                for a in w.agents() {
                    r.alive100[group(a)] += 1.0;
                    r.cached100[group(a)] += a.caches.values().sum::<f64>();
                }
            }
            200 => {
                for a in w.agents() {
                    r.alive200[group(a)] += 1.0;
                    r.wealth200[group(a)] += wealth(a);
                    r.held200[group(a)] += a.holdings[0] + a.fed;
                    r.cached200[group(a)] += a.caches.values().sum::<f64>();
                }
            }
            _ => {}
        }
    }
    let last = |name: &str| {
        w.stats
            .series(name)
            .and_then(|s| s.last().copied())
            .unwrap_or(f64::NAN)
    };
    r.fates = [
        last("fate_dug"),
        last("fate_pilfered"),
        last("fate_lost"),
        last("fate_buried"),
    ];
    r.log_full = w.cache_log_full;
    let (mut d, mut da, mut p, mut pa) = (0.0, 0.0, 0.0, 0.0);
    let mut cohort = Vec::new();
    if !r.log_full {
        for rec in &w.cache_log {
            match rec.fate {
                Some((t, f)) => {
                    let age = t - rec.buried;
                    match f {
                        Fate::Dug => {
                            d += rec.amount;
                            da += rec.amount * age as f64;
                        }
                        Fate::Pilfered { .. } => {
                            p += rec.amount;
                            pa += rec.amount * age as f64;
                        }
                        Fate::Lost => {}
                    }
                    cohort.push((age, rec.amount, matches!(f, Fate::Pilfered { .. })));
                }
                None => cohort.push((TICKS - rec.buried, rec.amount, false)),
            }
        }
    }
    r.age_dug = nan_div(da, d);
    r.age_pilfered = nan_div(pa, p);
    (r.km_r, r.km_rms) = if r.log_full {
        (f64::NAN, f64::NAN)
    } else {
        cohort_fit(&cohort)
    };
    r
}

/// Cached runs: (key, seeds, runs).
type RunCache = Vec<(String, Vec<u64>, Vec<Run>)>;

/// Runs of `c` (with the dig-at-R probe when `probe`), shared by the claims.
fn runs_of(c: &Config, probe: bool, seeds: &[u64]) -> Vec<Run> {
    static CACHE: Mutex<RunCache> = Mutex::new(Vec::new());
    let key = format!(
        "{probe}{}",
        serde_json::to_string(c).expect("a config serializes")
    );
    if let Some(hit) = CACHE
        .lock()
        .unwrap()
        .iter()
        .find(|(k, s, _)| *k == key && s == seeds)
    {
        return hit.2.clone();
    }
    let out = each_seed(c, seeds, |w| run(w, probe));
    CACHE
        .lock()
        .unwrap()
        .push((key, seeds.to_vec(), out.clone()));
    out
}

fn runs(c: &Config, seeds: &[u64]) -> Vec<Run> {
    runs_of(c, false, seeds)
}

// ------------------------------------------------------------------ worlds

/// theft-winter's world (`even` hoarders, keep, owner memory on, no bury
/// cost) at `find` with a `cheaters` share. find 0 with no cheaters is
/// cache-winter-even.
fn field(find: f64, cheaters: f64) -> Config {
    let mut c = preset("theft-winter");
    c.theft.find = find;
    c.theft.cheaters = cheaters;
    c
}

fn without_memory(mut c: Config) -> Config {
    c.theft.owner_memory = false;
    c
}

fn eating(mut c: Config) -> Config {
    c.theft.loot = Loot::Eat;
    c
}

fn with_beta(mut c: Config, beta: u32) -> Config {
    c.seasons.winter_divisor = beta;
    c
}

/// The arena of `n` agents at `find` and bury cost `cost`.
fn arena(n: u32, find: f64, cost: f64) -> Config {
    let mut c = preset(&format!("theft-arena-{n}"));
    c.theft.find = find;
    c.caching.bury_cost = cost;
    c
}

// ----------------------------------------------------------------- helpers

fn col(r: &[Run], f: impl Fn(&Run) -> f64) -> Vec<f64> {
    r.iter().map(f).collect()
}

/// Median and IQR as percentages.
fn medp(v: &[f64]) -> String {
    format!(
        "{:.2} % (IQR {:.2}–{:.2} %)",
        100.0 * med_or_nan(v),
        100.0 * q_or_nan(v, 0.25),
        100.0 * q_or_nan(v, 0.75)
    )
}

fn m(v: &[f64]) -> f64 {
    med_or_nan(v)
}

/// The two-sided t quantile, by bisection on the CDF.
fn t_crit(p: f64, df: f64) -> f64 {
    let (mut lo, mut hi) = (0.0, 1000.0);
    for _ in 0..200 {
        let mid = 0.5 * (lo + hi);
        if stats::student_t_cdf(mid, df) < p {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    0.5 * (lo + hi)
}

/// (mean, lower, upper) of the 95 % t interval of the finite `v`.
fn ci95(v: &[f64]) -> (f64, f64, f64) {
    let v = stats::finite(v);
    let n = v.len() as f64;
    let mu = mean(&v);
    let sd = (v.iter().map(|x| (x - mu).powi(2)).sum::<f64>() / (n - 1.0)).sqrt();
    let h = t_crit(0.975, n - 1.0) * sd / n.sqrt();
    (mu, mu - h, mu + h)
}

/// Holds when the 95 % t interval of the mean of `slopes` includes 0.
fn ci_includes_zero(slopes: &[f64]) -> Outcome {
    let v = stats::finite(slopes);
    if v.len() < 5 {
        return untestable(&format!(
            "only {} seeds gave a finite slope (need 5)",
            v.len()
        ));
    }
    let (mu, lo, hi) = ci95(&v);
    let verdict = if lo <= 0.0 && 0.0 <= hi {
        Verdict::Holds
    } else {
        Verdict::Fails
    };
    Outcome {
        verdict,
        measured: format!(
            "mean per-seed slope {mu:.4}, 95 % CI {lo:.4} to {hi:.4} (t, df {}); {} of {} slopes above 0",
            v.len() - 1,
            v.iter().filter(|&&x| x > 0.0).count(),
            v.len()
        ),
        detail: String::new(),
    }
}

/// Per seed, the slope of `y(run)` on `xs` across `worlds` (one run list per
/// x, seeds aligned).
fn seed_slopes(xs: &[f64], worlds: &[Vec<Run>], y: impl Fn(&Run) -> f64) -> Vec<f64> {
    (0..worlds[0].len())
        .map(|i| {
            let ys: Vec<f64> = worlds.iter().map(|w| y(&w[i])).collect();
            slope(xs, &ys)
        })
        .collect()
}

/// The first sign change of `ys` from below 0 to at least 0, interpolated.
fn crossing(xs: &[f64], ys: &[f64]) -> Option<f64> {
    (0..xs.len().saturating_sub(1)).find_map(|i| {
        let (y0, y1) = (ys[i], ys[i + 1]);
        (y0 < 0.0 && y1 >= 0.0).then(|| xs[i] + (0.0 - y0) * (xs[i + 1] - xs[i]) / (y1 - y0))
    })
}

fn flag(b: bool) -> f64 {
    if b {
        1.0
    } else {
        0.0
    }
}

/// Survival, per founder and wealth, both groups, for the detail.
/// Each group's survival and wealth; a group with no founders in any run
/// (cheaters in a hoarders-only world) is left out, not printed as NaN.
fn groups(r: &[Run]) -> String {
    [(H, "hoarders"), (C, "cheaters")]
        .into_iter()
        .filter(|&(g, _)| r.iter().any(|x| x.founders[g] > 0.0))
        .map(|(g, name)| {
            format!(
                "{name}: survival {}, per founder {}, wealth per founder {}",
                med(&col(r, move |x| x.surv(g))),
                med(&col(r, move |x| x.surv_f(g))),
                med(&col(r, move |x| x.wealth_f(g))),
            )
        })
        .collect::<Vec<_>>()
        .join("; ")
}

/// A world's theft usage, for the detail.
fn usage(label: &str, r: &[Run]) -> String {
    let sum = |f: fn(&Run) -> f64| r.iter().map(f).sum::<f64>();
    let pil = sum(|x| x.pilfered);
    let dug = sum(|x| x.dug);
    let full = r.iter().filter(|x| x.log_full).count();
    format!(
        "{label}: pilfers per seed {}, owner finds per seed {}; pilferage rate {}, v {}; sugar buried per seed {}, dug {}, pilfered {}, lost {}; fate shares at 200 (stats series) dug {}, pilfered {}, lost {}, still buried {}; p_s {}, p_o {}; pilfered sugar taken in summer {:.1} %, in winter {:.1} % (dug: {:.1} % and {:.1} %); mean cache age (amount-weighted, from the log) when dug {} and when pilfered {} ticks (loot buried again opens fresh records, which lowers the pilfered age); logs skipped as full: {full} of {}",
        med(&col(r, |x| x.pilfers)),
        med(&col(r, |x| x.owner_finds)),
        medp(&col(r, Run::rate)),
        med(&col(r, Run::v)),
        med(&col(r, |x| x.buried)),
        med(&col(r, |x| x.dug)),
        med(&col(r, |x| x.pilfered)),
        med(&col(r, |x| x.lost)),
        med(&col(r, |x| x.fates[0])),
        med(&col(r, |x| x.fates[1])),
        med(&col(r, |x| x.fates[2])),
        med(&col(r, |x| x.fates[3])),
        med(&col(r, Run::p_s)),
        med(&col(r, Run::p_o)),
        100.0 * nan_div(sum(|x| x.pilfered_season[0]), pil),
        100.0 * nan_div(sum(|x| x.pilfered_season[1]), pil),
        100.0 * nan_div(sum(|x| x.dug_season[0]), dug),
        100.0 * nan_div(sum(|x| x.dug_season[1]), dug),
        med(&col(r, |x| x.age_dug)),
        med(&col(r, |x| x.age_pilfered)),
        r.len(),
    )
}

// --------------------------------------------------------- 1. pilferage

fn pilferage_row(label: &str, find: f64, r: &[Run]) -> String {
    let ratio = col(r, |x| x.rate() / (x.v() * find));
    format!(
        "{label} at find {find}: rate {}; v {} (non-owner agents on a cache's site per cache per tick, drawing or not: {}); v × find {}; rate ÷ (v × find) {}; sugar pilfered per sugar cached per tick {}; agents per open site {}; the cohort fit r {} (RMS residual {}); reported (fix round 1): takes ÷ Σ over arrivals of 1 − (1 − find)^k (k the foreign caches on the site at the tick's start) {}, draws on sites with two or more foreign caches {}, arrivals on a cache at the carrying limit (holdings at the tick's start) per seed {}",
        medp(&col(r, Run::rate)),
        med(&col(r, Run::v)),
        med(&col(r, Run::raw_v)),
        medp(&col(r, |x| x.v() * find)),
        med(&ratio),
        medp(&col(r, Run::loss)),
        med(&col(r, Run::density)),
        medp(&col(r, |x| x.km_r)),
        med(&col(r, |x| x.km_rms)),
        med(&col(r, |x| x.pilfers / x.pred_takes)),
        medp(&col(r, |x| x.k_stacked / x.k_sum)),
        med(&col(r, |x| x.full_arrivals)),
    )
}

fn pilferage_claim(seeds: &[u64]) -> Outcome {
    let anchor = runs(&field(ANCHOR, 0.0), seeds);
    let ratio = col(&anchor, |x| x.rate() / (x.v() * ANCHOR));
    let mut sweep = Vec::new();
    for (label, cheaters) in [
        ("theft-winter", 0.0),
        ("a quarter cheaters", 0.25),
        ("half cheaters", 0.5),
    ] {
        for f in FINDS {
            sweep.push(pilferage_row(label, f, &runs(&field(f, cheaters), seeds)));
        }
    }
    let mut arenas = Vec::new();
    for n in NS {
        for f in FINDS {
            arenas.push(pilferage_row(
                &format!("theft-arena-{n}"),
                f,
                &runs(&arena(n, f, 0.0), seeds),
            ));
        }
    }
    let at_one = runs(&field(1.0, 0.0), seeds);
    range(&ratio, 1.0 - DECOMP_TOL, 1.0 + DECOMP_TOL, false)
        .with(&format!(
            "theft-winter at the anchor (find {ANCHOR}), ticks 1–200, per seed: rate {}; v {}; rate ÷ (v × find) {}.",
            list(&col(&anchor, |x| 100.0 * x.rate()), 2),
            list(&col(&anchor, Run::v), 4),
            list(&ratio, 3),
        ))
        .with(&format!("Across the find sweep (medians over seeds): {}.", sweep.join("; ")))
        .with(&format!("The arenas (half cheaters, no bury cost): {}.", arenas.join("; ")))
        .with(&format!(
            "Context, not judged. Vander Wall and Jenkins: \"Most pilferage rates for long-term hoarders fall between 2–30% per day\" (p.656), with \"the median empirical rate of loss for long-term scatter hoarders of 9%\" (p.663); their rates assume \"the rate of removal over the duration of a study was constant\" (p.658). The cohort fit above is that assumption tested on the log: an amount-weighted Kaplan–Meier curve of cached sugar against pilfering (digging, a dead owner and tick 200 censor), fitted to (1 − r)^age. Why stumbling can't reach the field's rates: a cache is found only when a non-owner ends a tick on its site and draws, and an arrival takes at most one of the k caches on a site, so it takes one with chance 1 − (1 − find)^k, not k × find; the rate is at most v × find, and at most v at find 1. That one-take rule on stacked sites is the gap between the rate and v × find: in the winter field the takes match Σ 1 − (1 − find)^k over arrivals within 3 % at every find (the reported ratio above; the arenas, with few caches, are noisier at low find), while draws on sites with two or more foreign caches fall as find rises, and no thief arrived on a cache at the carrying limit (measured at the tick's start). v is set by how crowded the sites are, not by find: at find 1 in theft-winter v is {} with {} agents per open site. The field's 2–30 % a day are reached by animals that search for caches (by smell, or by watching others cache, which P2 adds).",
            med(&col(&at_one, Run::v)),
            med(&col(&at_one, Run::density)),
        ))
}

// ------------------------------------------------------- 2. the threshold

/// One arena run's reading of condition (3): (condition, hoarders richer),
/// or `None` with no ended sugar.
fn threshold(r: &Run, n: u32, cost: f64) -> Option<(bool, bool)> {
    let (ps, po) = (r.p_s(), r.p_o());
    if !ps.is_finite() || !po.is_finite() {
        return None;
    }
    let need = cost * f64::from(n - 1) + 1.0;
    let cond = if po == 0.0 { ps > 0.0 } else { ps / po > need };
    Some((cond, r.wealth_f(H) - r.wealth_f(C) > 0.0))
}

/// Agreement under three readings, over every run of `cells` ((n, cost,
/// runs)): the ruled one (fate shares), the Appendix's P_H − P_N > C/G with
/// P_H = dug ÷ ended and P_N = p_o ÷ (n − 1), and per visit (p_s = 1 for an
/// owner who remembers, p_o = find).
fn agreement(cells: &[(u32, f64, Vec<Run>)], find: f64) -> (Vec<f64>, String) {
    let mut flags = Vec::new();
    let (mut appendix, mut per_visit, mut n_all) = (0, 0, 0);
    let (mut rich_not_cond, mut rich) = (0, 0);
    for (n, cost, rs) in cells {
        for r in rs {
            let Some((cond, richer)) = threshold(r, *n, *cost) else {
                flags.push(f64::NAN);
                continue;
            };
            flags.push(flag(cond == richer));
            n_all += 1;
            rich += usize::from(richer);
            rich_not_cond += usize::from(richer && !cond);
            let ended = r.dug + r.pilfered + r.lost;
            let ph = r.dug / ended;
            let pn = r.p_o() / f64::from(n - 1);
            appendix += usize::from((ph - pn > *cost) == richer);
            let visit = 1.0 / find > cost * f64::from(n - 1) + 1.0;
            per_visit += usize::from(visit == richer);
        }
    }
    let agree = flags.iter().filter(|&&f| f == 1.0).count();
    let text = format!(
        "at find {find}: {agree} of {n_all} runs with a value agree ({:.1} %); hoarders richer in {rich}, of which {rich_not_cond} where the condition fails (against its necessity); the Appendix's form P_H − P_N > C/G agrees in {appendix} ({:.1} %); per visit (p_s = 1 for an owner who remembers, p_o = find) in {per_visit} ({:.1} %)",
        pct(agree, n_all),
        pct(appendix, n_all),
        pct(per_visit, n_all),
    );
    (flags, text)
}

/// Reported (fix round 1): the ruled reading of condition (3) against
/// fitness with still-buried caches valued at 0 (holdings + stomach per
/// founder at 200).
fn held_agreement(cells: &[(u32, f64, Vec<Run>)], find: f64) -> String {
    let (mut agree, mut rich, mut rich_not_cond, mut n_all) = (0, 0, 0, 0);
    for (n, cost, rs) in cells {
        for r in rs {
            let Some((cond, _)) = threshold(r, *n, *cost) else {
                continue;
            };
            let richer = r.held_f(H) - r.held_f(C) > 0.0;
            n_all += 1;
            agree += usize::from(cond == richer);
            rich += usize::from(richer);
            rich_not_cond += usize::from(richer && !cond);
        }
    }
    format!(
        "at find {find}, caches valued at 0: {agree} of {n_all} agree ({:.1} %); hoarders richer in {rich}, of which {rich_not_cond} where the condition fails",
        pct(agree, n_all)
    )
}

fn pct(n: usize, d: usize) -> f64 {
    crate::claims::minds5::pct(n, d)
}

fn arena_cells(find: f64, seeds: &[u64]) -> Vec<(u32, f64, Vec<Run>)> {
    let mut out = Vec::new();
    for n in NS {
        for cost in COSTS {
            out.push((n, cost, runs(&arena(n, find, cost), seeds)));
        }
    }
    out
}

fn threshold_claim(seeds: &[u64]) -> Outcome {
    let cells = arena_cells(ANCHOR, seeds);
    let (flags, summary) = agreement(&cells, ANCHOR);
    let rows: Vec<String> = cells
        .iter()
        .map(|(n, cost, r)| {
            let t: Vec<(bool, bool)> = r.iter().filter_map(|x| threshold(x, *n, *cost)).collect();
            format!(
                "n {n}, C/G {cost}: p_s {}, p_o {}, p_s ÷ p_o {} against {:.2}; condition holds in {} of {}, hoarders richer in {}, agree in {}; {}; everyone alive at 200 of those at 100 {}; reported: holdings + stomach per founder (caches at 0) hoarders {}, cheaters {}, hoarders richer so in {}; caches per founding hoarder at 200 {}; sugar pilfered per founder, hoarders {}, cheaters {}",
                med(&col(r, Run::p_s)),
                med(&col(r, Run::p_o)),
                med(&col(r, |x| x.p_s() / x.p_o())),
                cost * f64::from(n - 1) + 1.0,
                t.iter().filter(|x| x.0).count(),
                t.len(),
                t.iter().filter(|x| x.1).count(),
                t.iter().filter(|x| x.0 == x.1).count(),
                groups(r),
                med(&col(r, Run::surv_all)),
                med(&col(r, |x| x.held_f(H))),
                med(&col(r, |x| x.held_f(C))),
                r.iter().filter(|x| x.held_f(H) > x.held_f(C)).count(),
                med(&col(r, |x| nan_div(x.cached200[H], x.founders[H]))),
                med(&col(r, |x| nan_div(x.stolen[H], x.founders[H]))),
                med(&col(r, |x| nan_div(x.stolen[C], x.founders[C]))),
            )
        })
        .collect();
    let held: Vec<String> = FINDS
        .iter()
        .map(|&f| {
            if f == ANCHOR {
                held_agreement(&cells, f)
            } else {
                held_agreement(&arena_cells(f, seeds), f)
            }
        })
        .collect();
    let others: Vec<String> = FINDS
        .iter()
        .filter(|&&f| f != ANCHOR)
        .map(|&f| agreement(&arena_cells(f, seeds), f).1)
        .collect();
    let rates: Vec<String> = NS
        .iter()
        .map(|&n| {
            let r = runs(&arena(n, ANCHOR, 0.0), seeds);
            format!(
                "n {n}: rate {}, v {}, p_o {}",
                medp(&col(&r, Run::rate)),
                med(&col(&r, Run::v)),
                med(&col(&r, Run::p_o))
            )
        })
        .collect();
    range(&flags, 1.0, 1.0, false)
        .with(&format!(
            "Arenas at the anchor, n 2, 4, 8 × C/G (bury cost) 0, 0.1, 0.25, 0.5, 1, ticks 1–200; fitness is wealth per founder at 200 (holdings + caches + stomach, the dead as 0), since nearly every agent survives. Overall {summary}. Per cell (medians over seeds): {}.",
            rows.join("; ")
        ))
        .with(&format!("Across the find sweep: {}.", others.join("; ")))
        .with(&format!(
            "Reported, not judged (fix round 1): p_s and p_o leave out still-buried sugar, while the judged fitness counts it at full value. With caches valued at 0 instead: {}. Whether condition (3) is necessary here depends on how still-buried sugar is valued.",
            held.join("; ")
        ))
        .with(&format!(
            "Andersson and Krebs's reasoning for the (n − 1): \"the probability that another individual will find the food before the one that hoarded it increases with group size\" (p.708). The arenas' pilferage at the anchor, no bury cost: {}. The paper's p_s and p_o are per visit (\"let ps = the visiting hoarder's probability of finding it, and let po = any other visiting individual's probability of finding it\", p.708); the ruled measure is each cache's fate, which the per-visit reading above sets beside it. Condition (3) is \"a necessary condition for hoarders to be more fit than non-hoarding group-members\" (p.711), so a run where hoarders are richer and it fails counts against it; one where it holds and cheaters are richer does not.",
            rates.join("; ")
        ))
}

// ------------------------------------------- 3–5. shares of cheaters

fn share_worlds(find: f64, memory: bool, seeds: &[u64]) -> Vec<Vec<Run>> {
    SHARES
        .iter()
        .map(|&s| {
            let c = field(find, s);
            runs(&if memory { c } else { without_memory(c) }, seeds)
        })
        .collect()
}

fn share_rows(worlds: &[Vec<Run>]) -> String {
    SHARES
        .iter()
        .zip(worlds)
        .map(|(s, r)| {
            let pf = |g: usize| col(r, move |x| nan_div(x.stolen[g], x.founders[g]));
            format!(
                "share {s}: {}; advantage (hoarder − cheater survival) {}; sugar pilfered per founder, hoarders {}, cheaters {}; rate {}",
                groups(r),
                med(&col(r, Run::adv)),
                med(&pf(H)),
                med(&pf(C)),
                medp(&col(r, Run::rate)),
            )
        })
        .collect::<Vec<_>>()
        .join("; ")
}

fn frequency_claim(seeds: &[u64]) -> Outcome {
    let worlds = share_worlds(ANCHOR, true, seeds);
    let slopes = seed_slopes(&SHARES, &worlds, Run::adv);
    let wealth_slopes = seed_slopes(&SHARES, &worlds, |x| x.wealth_f(H) - x.wealth_f(C));
    let mut sweep = Vec::new();
    for f in [0.0].iter().chain(&FINDS) {
        let w = share_worlds(*f, true, seeds);
        let s = seed_slopes(&SHARES, &w, Run::adv);
        let (mu, lo, hi) = ci95(&s);
        let beat = (0..SHARES.len())
            .filter(|&k| m(&col(&w[k], Run::adv)) < 0.0)
            .count();
        sweep.push(format!(
            "find {f}: mean slope {mu:.4} (95 % CI {lo:.4} to {hi:.4}); cheaters' median survival above hoarders' at {beat} of {} shares; medians by share of hoarder, cheater survival: {}",
            SHARES.len(),
            w.iter()
                .map(|r| format!("{:.3}/{:.3}", m(&col(r, |x| x.surv(H))), m(&col(r, |x| x.surv(C)))))
                .collect::<Vec<_>>()
                .join(" "),
        ));
    }
    let none = runs(&preset("cache-winter-none"), seeds);
    let even = runs(&field(0.0, 0.0), seeds);
    ci_includes_zero(&slopes)
        .with(&format!(
            "theft-winter's world at the anchor, cheater shares 0.1–0.9, ticks 1–200; survival = alive at 200 ÷ alive at 100 within each group. Per-seed slopes {}. Reported: the same on wealth per founder, mean slope {:.3} (95 % CI {:.3} to {:.3}). By share (medians): {}.",
            list(&slopes, 3),
            ci95(&wealth_slopes).0,
            ci95(&wealth_slopes).1,
            ci95(&wealth_slopes).2,
            share_rows(&worlds),
        ))
        .with(&format!("Across the find sweep (find 0: cheaters never cache, and nobody finds anything): {}.", sweep.join("; ")))
        .with(&format!(
            "Baselines without theft: cache-winter-none (nobody caches) first-winter survival {} (per founder {}); cache-winter-even (everyone caches) {} (per founder {}). Andersson and Krebs: \"If a hoarder is fitter than a non-hoarder, this applies irrespective of the proportion of hoarders in the group. This is because hoarders and non-hoarders benefit equally from the presence of additional hoarders.\" (p.708)",
            med(&col(&none, |x| x.surv(H))),
            med(&col(&none, |x| x.surv_f(H))),
            med(&col(&even, |x| x.surv(H))),
            med(&col(&even, |x| x.surv_f(H))),
        ))
}

fn equal_recovery_claim(seeds: &[u64]) -> Outcome {
    let worlds = share_worlds(ANCHOR, false, seeds);
    let parts: Vec<(String, Outcome)> = SHARES
        .iter()
        .zip(&worlds)
        .map(|(s, r)| {
            (
                format!("share {s}"),
                paired_greater(
                    &col(r, |x| x.surv(C)),
                    &col(r, |x| x.surv(H)),
                    "cheater survival",
                    "hoarder survival",
                ),
            )
        })
        .collect();
    let sweep: Vec<String> = FINDS
        .iter()
        .map(|&f| {
            let w = share_worlds(f, false, seeds);
            let wins = w
                .iter()
                .filter(|r| {
                    paired_greater(&col(r, |x| x.surv(C)), &col(r, |x| x.surv(H)), "c", "h").verdict
                        == Verdict::Holds
                })
                .count();
            format!(
                "find {f}: cheaters ahead in at least 80 % of seeds at {wins} of {} shares; p_s {}, p_o {} (share 0.5)",
                SHARES.len(),
                med(&col(&w[4], Run::p_s)),
                med(&col(&w[4], Run::p_o)),
            )
        })
        .collect();
    // Equal chance per draw is not equal recovery: owners bury where they
    // stand, so they cross their own caches far more often than others do.
    let half = |r: &[Run]| {
        let total = |f: fn(&Run) -> f64| col(r, f).iter().sum::<f64>();
        format!(
            "p_s {}, p_o {}; over the {} seeds, owner finds {} (summer {}, winter {}), pilfers {}",
            med(&col(r, Run::p_s)),
            med(&col(r, Run::p_o)),
            r.len(),
            total(|x| x.owner_finds),
            total(|x| x.owner_finds_season[0]),
            total(|x| x.owner_finds_season[1]),
            total(|x| x.pilfers),
        )
    };
    let on = runs(&field(ANCHOR, 0.5), seeds);
    all_of(parts)
        .with(&format!(
            "owner_memory off (an owner finds its own cache only by stumbling on it, at rate find), theft-winter's world at the anchor. By share (medians): {}.",
            share_rows(&worlds)
        ))
        .with(&format!("Across the find sweep: {}.", sweep.join("; ")))
        .with(&format!(
            "Reported, not judged (final review): owner_memory off makes the chance per draw equal, not recovery. Owners bury where they stand, so they cross their own caches far more often than others do. theft-winter-half's world at the anchor, ticks 1–200: memory on, {}; memory off, {}.",
            half(&on),
            half(&worlds[4]),
        ))
        .with("Andersson and Krebs: \"if the probability that a stored item is utilized by a certain group member is the same for all items and members (ps = po) then hoarders will have a lower fitness than `cheaters'.\" (p.710)")
}

fn mixed_claim(seeds: &[u64]) -> Outcome {
    let worlds = share_worlds(ANCHOR, true, seeds);
    let per_seed: Vec<Vec<f64>> = (0..seeds.len())
        .map(|i| worlds.iter().map(|w| w[i].adv()).collect())
        .collect();
    let flags: Vec<f64> = per_seed
        .iter()
        .map(|a| {
            if a[0].is_finite() && a[8].is_finite() {
                flag(a[0] < 0.0 && a[8] > 0.0)
            } else {
                f64::NAN
            }
        })
        .collect();
    let crossings: Vec<String> = per_seed
        .iter()
        .map(|a| crossing(&SHARES, a).map_or("none".into(), |x| format!("{x:.2}")))
        .collect();
    let curve = |w: &[Vec<Run>]| -> Vec<f64> { w.iter().map(|r| m(&col(r, Run::adv))).collect() };
    let median_curve = curve(&worlds);
    let off = curve(&share_worlds(ANCHOR, false, seeds));
    let sweep: Vec<String> = FINDS
        .iter()
        .map(|&f| {
            let c = curve(&share_worlds(f, true, seeds));
            format!(
                "find {f}: median advantage by share {} (crossing {})",
                list(&c, 3),
                crossing(&SHARES, &c).map_or("none".into(), |x| format!("{x:.2}"))
            )
        })
        .collect();
    range(&flags, 1.0, 1.0, false)
        .with(&format!(
            "theft-winter's world at the anchor; advantage = hoarder − cheater survival. Per seed, the first crossing from below 0 to at least 0 (share of cheaters): {}. The median advantage by share 0.1–0.9: {} (crossing {}); with owner_memory off: {}.",
            crossings.join(" "),
            list(&median_curve, 3),
            crossing(&SHARES, &median_curve).map_or("none".into(), |x| format!("{x:.2}")),
            list(&off, 3),
        ))
        .with(&format!("Across the find sweep: {}.", sweep.join("; ")))
        .with("Andersson and Krebs (p.708), assuming hoarders are poorer thieves than non-hoarders (ps > po > pt): \"a stable mixture of hoarders and non-hoarders may result. This is because hoarders are fitter than non-hoarders when at a low proportion in the group, whereas a reversal occurs at some point as hoarders increase in proportion.\" The model has no such difference per draw: hoarders and cheaters find each cache they stand on at the same rate. By amount they differ: in the winter field hoarders pilfer 5.5–6.9 times as much sugar per founder as cheaters (claim 3's rows; the arenas show 1.1–1.7 times at n = 4 and 8 with no bury cost), consistent with Vander Wall and Jenkins (\"food hoarders are expected to pilfer far more than they can consume, recaching the excess\", p.661), so any crossing here has another cause. Its stability is P1b's.")
}

// ---------------------------------------------------------- 6. reciprocity

fn reciprocity_claim(seeds: &[u64]) -> Outcome {
    let keep = runs(&field(ANCHOR, 0.0), seeds);
    let eat = runs(&eating(field(ANCHOR, 0.0)), seeds);
    let stored = |r: &[Run]| col(r, |x| nan_div(x.cached100[H], x.founders[H]));
    let rows = |label: &str, k: &[Run], e: &[Run]| {
        format!(
            "{label}: caches per founding hoarder at 100, keep {} against eat {}; hoarder survival keep {} against eat {} (per founder {} and {}); {}",
            med(&stored(k)),
            med(&stored(e)),
            med(&col(k, |x| x.surv(H))),
            med(&col(e, |x| x.surv(H))),
            med(&col(k, |x| x.surv_f(H))),
            med(&col(e, |x| x.surv_f(H))),
            if k[0].founders[C] > 0.0 {
                format!(
                    "cheater survival keep {} against eat {}",
                    med(&col(k, |x| x.surv(C))),
                    med(&col(e, |x| x.surv(C)))
                )
            } else {
                "no cheaters".into()
            }
        )
    };
    let mut sweep = Vec::new();
    for (label, share) in [
        ("theft-winter", 0.0),
        ("a quarter cheaters", 0.25),
        ("half cheaters", 0.5),
    ] {
        for f in FINDS {
            let c = field(f, share);
            sweep.push(rows(
                &format!("{label}, find {f}"),
                &runs(&c, seeds),
                &runs(&eating(c.clone()), seeds),
            ));
        }
    }
    all_of(vec![
        (
            "stored food".into(),
            paired_greater(&stored(&keep), &stored(&eat), "keep", "eat"),
        ),
        (
            "hoarder survival".into(),
            paired_greater(
                &col(&keep, |x| x.surv(H)),
                &col(&eat, |x| x.surv(H)),
                "keep",
                "eat",
            ),
        ),
    ])
    .with(&format!(
        "theft-winter at the anchor, keep (loot into holdings, where a hoarding thief may bury it again) against eat (loot into the thief's stomach, which feeds it but can't be buried), same seeds. {}; loot eaten per seed under eat {}.",
        rows("At the anchor", &keep, &eat),
        med(&col(&eat, |x| x.loot_eaten)),
    ))
    .with(&format!("Across the find sweep and cheater shares: {}.", sweep.join("; ")))
    .with("Vander Wall and Jenkins: \"If the pilferer recaches the food, and if pilfering is reciprocal, then long-term scatter hoarding can persist unless some alternative form of food storage (larder hoarding or internal energy storage) provides more benefits.\" (p.661). Under the ruled eat the loot still feeds the thief, which is internal storage in their terms.")
}

// ------------------------------------------------------------- 7. loss

/// (label, median rate, median sugar loss per tick, hoarders win) for each
/// configuration at `find`.
fn loss_configs(find: f64, seeds: &[u64]) -> Vec<(String, f64, f64, bool)> {
    let mut out = Vec::new();
    for (s, r) in SHARES.iter().zip(share_worlds(find, true, seeds)) {
        let win = paired_greater(&col(&r, |x| x.surv(H)), &col(&r, |x| x.surv(C)), "h", "c")
            .verdict
            == Verdict::Holds;
        out.push((
            format!("field, share {s}"),
            m(&col(&r, Run::rate)),
            m(&col(&r, Run::loss)),
            win,
        ));
    }
    for (n, cost, r) in arena_cells(find, seeds) {
        let win = paired_greater(
            &col(&r, |x| x.wealth_f(H)),
            &col(&r, |x| x.wealth_f(C)),
            "h",
            "c",
        )
        .verdict
            == Verdict::Holds;
        out.push((
            format!("arena n {n}, C/G {cost}"),
            m(&col(&r, Run::rate)),
            m(&col(&r, Run::loss)),
            win,
        ));
    }
    out
}

/// Reported (fix round 1): the arena configurations where hoarders win
/// on holdings + stomach per founder (caches valued at 0).
fn held_wins(find: f64, seeds: &[u64]) -> String {
    let wins: Vec<String> = arena_cells(find, seeds)
        .into_iter()
        .filter(|(_, _, r)| {
            paired_greater(&col(r, |x| x.held_f(H)), &col(r, |x| x.held_f(C)), "h", "c").verdict
                == Verdict::Holds
        })
        .map(|(n, cost, r)| {
            format!(
                "arena n {n}, C/G {cost} ({:.2} %)",
                100.0 * m(&col(&r, Run::rate))
            )
        })
        .collect();
    format!(
        "find {find}: hoarders win on holdings in {} of {} arena configurations{}",
        wins.len(),
        NS.len() * COSTS.len(),
        if wins.is_empty() {
            String::new()
        } else {
            format!(": {}", wins.join(", "))
        }
    )
}

fn loss_summary(find: f64, v: &[(String, f64, f64, bool)]) -> String {
    let max = v.iter().map(|x| x.1).fold(f64::NAN, f64::max);
    let wins: Vec<String> = v
        .iter()
        .filter(|x| x.3)
        .map(|x| format!("{} ({:.2} %)", x.0, 100.0 * x.1))
        .collect();
    format!(
        "find {find}: highest median rate {:.2} %; hoarders win in {} of {} configurations{}",
        100.0 * max,
        wins.len(),
        v.len(),
        if wins.is_empty() {
            String::new()
        } else {
            format!(": {}", wins.join(", "))
        }
    )
}

fn loss_claim(seeds: &[u64]) -> Outcome {
    let v = loss_configs(ANCHOR, seeds);
    let high: Vec<&(String, f64, f64, bool)> = v.iter().filter(|x| x.1 >= VJ_LOSS).collect();
    let table: Vec<String> = v
        .iter()
        .map(|x| {
            format!(
                "{}: rate {:.2} %, sugar {:.2} % a tick, hoarders win {}",
                x.0,
                100.0 * x.1,
                100.0 * x.2,
                x.3
            )
        })
        .collect();
    let mut out = if high.is_empty() {
        untestable(&format!(
            "no configuration at the anchor reaches a median daily pilferage rate of {:.0} %",
            100.0 * VJ_LOSS
        ))
    } else if high.iter().any(|x| x.3) {
        Outcome {
            verdict: Verdict::Holds,
            measured: format!(
                "{} configurations at ≥ 18 %, hoarders win in some",
                high.len()
            ),
            detail: String::new(),
        }
    } else {
        Outcome {
            verdict: Verdict::Fails,
            measured: format!(
                "{} configurations at ≥ 18 %, hoarders win in none",
                high.len()
            ),
            detail: String::new(),
        }
    };
    out.measured = format!("{} [{}]", out.measured, loss_summary(ANCHOR, &v));
    let sweep: Vec<String> = FINDS
        .iter()
        .filter(|&&f| f != ANCHOR)
        .map(|&f| loss_summary(f, &loss_configs(f, seeds)))
        .collect();
    out.with(&format!(
        "Configurations at the anchor (medians over seeds; the field judged on survival, the arenas on wealth per founder; \"win\" is paired_greater holding): {}.",
        table.join("; ")
    ))
    .with(&format!("Across the find sweep: {}.", sweep.join("; ")))
    .with(&format!(
        "Reported, not judged (fix round 1): the arenas' wins with still-buried caches valued at 0 (holdings + stomach per founder at 200): {}.",
        FINDS
            .iter()
            .map(|&f| held_wins(f, seeds))
            .collect::<Vec<_>>()
            .join("; ")
    ))
    .with("Vander Wall and Jenkins: \"In our simulations, the average daily rate of loss of scatter hoards was 18% in cases in which larder hoarding did not become established.\" (p.663). Their hoarders compete with larder-hoarding cheaters; ours with agents that never store.")
}

// ------------------------------------------------------ 8. mild winters

fn mild_claim(seeds: &[u64]) -> Outcome {
    let xs: Vec<f64> = BETAS.iter().map(|&b| f64::from(b).log2()).collect();
    let worlds = |f: f64| -> Vec<Vec<Run>> {
        BETAS
            .iter()
            .map(|&b| runs(&with_beta(field(f, 0.5), b), seeds))
            .collect()
    };
    let cheat = |x: &Run| x.wealth_f(C) - x.wealth_f(H);
    let w = worlds(ANCHOR);
    let slopes = seed_slopes(&xs, &w, cheat);
    let surv_slopes = seed_slopes(&xs, &w, |x| x.surv(C) - x.surv(H));
    let held_slopes = seed_slopes(&xs, &w, |x| x.held_f(C) - x.held_f(H));
    let rows: Vec<String> = BETAS
        .iter()
        .zip(&w)
        .map(|(b, r)| {
            format!(
                "β {b}: {}; cheater advantage in wealth {}; reported: caches per founding hoarder at 200 {}, holdings + stomach per founder hoarders {}, cheaters {}, cheater advantage so {}",
                groups(r),
                med(&col(r, cheat)),
                med(&col(r, |x| nan_div(x.cached200[H], x.founders[H]))),
                med(&col(r, |x| x.held_f(H))),
                med(&col(r, |x| x.held_f(C))),
                med(&col(r, |x| x.held_f(C) - x.held_f(H))),
            )
        })
        .collect();
    let sweep: Vec<String> = FINDS
        .iter()
        .map(|&f| {
            let s = seed_slopes(&xs, &worlds(f), cheat);
            format!(
                "find {f}: slope median {:.3}, below 0 in {} of {}",
                med_or_nan(&s),
                s.iter().filter(|&&x| x < 0.0).count(),
                stats::finite(&s).len()
            )
        })
        .collect();
    range(&slopes, f64::NEG_INFINITY, -f64::MIN_POSITIVE, false)
        .with(&format!(
            "theft-winter-half's world at the anchor, β = 2, 4, 8, 16, 32 (the winter's growback is the summer's ÷ β); per seed the slope of the cheater advantage (cheater − hoarder wealth per founder at 200) on log₂ β, where below 0 means cheaters gain as winter softens: {}. Reported: the same on survival (alive at 200 ÷ alive at 100 in each group), median slope {:.4}, below 0 in {} of {}. By β (medians): {}.",
            list(&slopes, 2),
            med_or_nan(&surv_slopes),
            surv_slopes.iter().filter(|&&x| x < 0.0).count(),
            stats::finite(&surv_slopes).len(),
            rows.join("; "),
        ))
        .with(&format!("Across the find sweep: {}.", sweep.join("; ")))
        .with(&format!(
            "Reported, not judged (fix round 1): the pre-registered wealth counts the hoarders' still-buried caches at full value. With caches valued at 0 (holdings + stomach per founder at 200), the per-seed slope of the cheater advantage on log₂ β has median {:.3}, below 0 in {} of {} seeds.",
            med_or_nan(&held_slopes),
            held_slopes.iter().filter(|&&x| x < 0.0).count(),
            stats::finite(&held_slopes).len(),
        ))
        .with("Vander Wall and Jenkins, untested in their paper: \"It is conceivable that under ideal conditions (e.g., mild winters), a nonhoarding cheater could survive and even flourish at the expense of conspecific hoarders.\" (p.661)")
}

// ------------------------------------------------------------ 9. usage

const PRESETS: [&str; 6] = [
    "theft-winter",
    "theft-winter-quarter",
    "theft-winter-half",
    "theft-arena-2",
    "theft-arena-4",
    "theft-arena-8",
];

fn usage_claim(seeds: &[u64]) -> Outcome {
    let mut parts = Vec::new();
    let mut rows = Vec::new();
    for id in PRESETS {
        let r = runs(&preset(id), seeds);
        parts.push((
            id.to_string(),
            range(&col(&r, |x| flag(x.pilfered > 0.0)), 1.0, 1.0, false),
        ));
        rows.push(format!("{}; {}", usage(id, &r), groups(&r)));
    }
    // Theft pools stores.
    let pools: Vec<String> = [0.0]
        .iter()
        .chain(&FINDS)
        .map(|&f| {
            let r = runs(&field(f, 0.0), seeds);
            let mut mixed = preset("cache-winter-mixed");
            mixed.theft.find = f;
            let x = runs(&mixed, seeds);
            format!(
                "find {f}: theft-winter survival {} (per founder {}), deaths in the winter {}; cache-winter-mixed deaths in the winter {}, survival {}",
                med(&col(&r, Run::surv_all)),
                med(&col(&r, Run::surv_all_f)),
                med(&col(&r, |x| x.deaths_winter)),
                med(&col(&x, |x| x.deaths_winter)),
                med(&col(&x, Run::surv_all)),
            )
        })
        .collect();
    let half = runs(&preset("theft-winter-half"), seeds);
    let probe = runs_of(&preset("theft-winter-half"), true, seeds);
    let skipped: usize = PRESETS
        .iter()
        .map(|id| {
            runs(&preset(id), seeds)
                .iter()
                .filter(|x| x.log_full)
                .count()
        })
        .sum();
    all_of(parts)
        .with(&format!(
            "Per preset (medians over seeds, ticks 1–200): {}. Fate logs skipped as full over the six presets: {skipped}.",
            rows.join(". ")
        ))
        .with(&format!(
            "Theft pools stores (likely: a thief is likely a hungry agent near someone else's surplus): {}.",
            pools.join("; ")
        ))
        .with(&format!(
            "The owner's advantage, theft-winter-half: {}. Reported, not judged: the same world with owners digging below their whole reserve R instead of R / 2 (the probe; no hysteresis band): {}; {}.",
            usage("as recorded", &half),
            usage("digging below R", &probe),
            groups(&probe),
        ))
        .with(&format!(
            "Reported (fix round 1), cheaters with nobody finding caches (find 0): a quarter cheaters, {}; half cheaters, {}.",
            groups(&runs(&field(0.0, 0.25), seeds)),
            groups(&runs(&field(0.0, 0.5), seeds)),
        ))
}

pub fn claims() -> Vec<Claim> {
    vec![
        Claim {
            id: "theft-winter.pilferage",
            item: "theft-winter",
            source: Source::Comment,
            citation: "Vander Wall & Jenkins 2003, pp.656, 658, 663; the survey's reframe",
            text: "Pilferage decomposes into visits and finding: the daily rate is non-owner visits per cache per tick × find, within 20 %, per seed at find 0.25 (the literature's 2–30 % a day, median 9 %, as context)",
            check: pilferage_claim,
        },
        Claim {
            id: "theft-arena-4.threshold",
            item: "theft-arena-4",
            source: Source::Book,
            citation: "Andersson & Krebs 1978, condition (3), p.708",
            text: "Hoarders beat cheaters where p_s/p_o > (C/G)(n − 1) + 1: across arenas of 2, 4 and 8 agents and bury costs, the sign of the advantage agrees with the condition in at least 80 % of runs",
            check: threshold_claim,
        },
        Claim {
            id: "theft-cheaters.frequency",
            item: "theft-cheaters",
            source: Source::Book,
            citation: "Andersson & Krebs 1978, p.708",
            text: "If a hoarder is fitter than a non-hoarder, this applies irrespective of the proportion of hoarders: the hoarder advantage's slope on the cheater share (0.1–0.9) has a confidence interval including 0",
            check: frequency_claim,
        },
        Claim {
            id: "theft-cheaters.equal-recovery",
            item: "theft-cheaters",
            source: Source::Book,
            citation: "Andersson & Krebs 1978, p.710",
            text: "With no owner's advantage (p_s = p_o), hoarders have a lower fitness than cheaters: with owner memory off, cheaters survive better at every share, seed by seed",
            check: equal_recovery_claim,
        },
        Claim {
            id: "theft-cheaters.mixed",
            item: "theft-cheaters",
            source: Source::Book,
            citation: "Andersson & Krebs 1978, p.708",
            text: "A stable mixture: hoarders are fitter when few, and the advantage crosses zero at an interior cheater share (located per seed)",
            check: mixed_claim,
        },
        Claim {
            id: "theft-winter.reciprocity",
            item: "theft-winter",
            source: Source::Book,
            citation: "Vander Wall & Jenkins 2003, p.661",
            text: "Reciprocal pilferage keeps hoarding worthwhile: with loot kept (and recached) rather than eaten, hoarders store more and survive better, seed by seed",
            check: reciprocity_claim,
        },
        Claim {
            id: "theft-winter-half.loss",
            item: "theft-winter-half",
            source: Source::Book,
            citation: "Vander Wall & Jenkins 2003, p.663",
            text: "Hoarding withstands a daily loss of 18 %: hoarders still beat cheaters in a world losing at least 18 % of its caches a day",
            check: loss_claim,
        },
        Claim {
            id: "theft-winter.mild",
            item: "theft-winter",
            source: Source::Book,
            citation: "Vander Wall & Jenkins 2003, p.661 (untested there)",
            text: "In mild winters a non-hoarding cheater could flourish: the cheater advantage grows as β falls, per seed",
            check: mild_claim,
        },
        Claim {
            id: "theft-winter-half.usage",
            item: "theft-winter-half",
            source: Source::Comment,
            citation: SPEC,
            text: "Theft acts in every Minds 6 preset (fate shares, p_s and p_o, the season and age of pilfering, and the fate log's flag reported)",
            check: usage_claim,
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_worlds_are_the_presets() {
        assert_eq!(field(ANCHOR, 0.0), preset("theft-winter"));
        assert_eq!(field(ANCHOR, 0.25), preset("theft-winter-quarter"));
        assert_eq!(field(ANCHOR, 0.5), preset("theft-winter-half"));
        let mut even = field(0.0, 0.0);
        assert!(!even.theft.is_on());
        even.theft = Default::default();
        assert_eq!(even, preset("cache-winter-even"));
        for n in NS {
            assert_eq!(arena(n, ANCHOR, 0.0), preset(&format!("theft-arena-{n}")));
            for cost in COSTS {
                arena(n, 1.0, cost).validate().expect("valid arena");
            }
        }
        for s in SHARES {
            without_memory(field(ANCHOR, s)).validate().expect("valid");
            eating(field(ANCHOR, s)).validate().expect("valid");
        }
        for b in BETAS {
            with_beta(field(ANCHOR, 0.5), b).validate().expect("valid");
        }
        assert_eq!(
            with_beta(field(ANCHOR, 0.5), 32),
            preset("theft-winter-half")
        );
    }

    #[test]
    fn the_measures_follow_their_definitions() {
        let r = Run {
            founders: [10.0, 10.0],
            alive100: [8.0, 5.0],
            alive200: [6.0, 5.0],
            wealth200: [30.0, 20.0],
            dug: 10.0,
            pilfered: 30.0,
            lost: 10.0,
            caches_pilfered: 3.0,
            candidates: 100.0,
            draws: 20.0,
            ..Run::default()
        };
        assert_eq!(r.surv(H), 0.75, "alive at 200 ÷ alive at 100");
        assert_eq!(r.surv_f(H), 0.6, "per founder");
        assert_eq!(r.adv(), -0.25);
        assert_eq!(r.p_s(), 0.25, "dug ÷ (dug + pilfered)");
        assert_eq!(r.p_o(), 0.6, "pilfered ÷ everything ended");
        assert_eq!((r.rate(), r.v()), (0.03, 0.2));
        assert!(Run::default().surv(C).is_nan());
        // Condition (3): 0.25 ÷ 0.6 < 1, so it fails; hoarders are richer.
        assert_eq!(threshold(&r, 4, 0.0), Some((false, true)));
    }

    #[test]
    fn the_cohort_fit_recovers_a_constant_rate() {
        // 1000 units each of age 1..=40 with a constant 5 % pilfered a tick:
        // a geometric cohort, with the survivors censored at 40.
        let r = 0.05;
        let mut recs = Vec::new();
        let mut left = 1000.0;
        for a in 1..=40u64 {
            recs.push((a, left * r, true));
            left *= 1.0 - r;
        }
        recs.push((40, left, false));
        let (fit, rms) = cohort_fit(&recs);
        assert!((fit - r).abs() < 1e-9, "{fit}");
        assert!(rms < 1e-9, "{rms}");
    }

    #[test]
    fn the_judges_helpers() {
        assert!((t_crit(0.975, 19.0) - 2.093).abs() < 1e-3);
        let flat = [0.1, -0.1, 0.05, -0.05, 0.0, 0.02];
        assert_eq!(ci_includes_zero(&flat).verdict, Verdict::Holds);
        let up = [1.0, 1.1, 0.9, 1.2, 1.0];
        assert_eq!(ci_includes_zero(&up).verdict, Verdict::Fails);
        assert_eq!(ci_includes_zero(&up[..3]).verdict, Verdict::Untestable);
        assert_eq!(crossing(&[0.1, 0.2, 0.3], &[-1.0, -0.5, 0.5]), Some(0.25));
        assert_eq!(crossing(&[0.1, 0.2], &[1.0, -1.0]), None);
    }
}
