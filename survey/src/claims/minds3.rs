//! Minds 3: memory, belief and truffles
//! (docs/superpowers/specs/2026-09-28-minds-3-memory-design.md). Memory's
//! value is measured as an information asymmetry: rememberers against
//! non-rememberers in the same world, on the same seed.
//!
//! Pairing: every within-world comparison (rememberers against others) is
//! paired per seed by construction; every comparison of two configurations
//! (`project` against `recall`, 5 Flumps against 20, memory on against off)
//! runs the same seeds in both arms and is judged on the per-seed
//! differences (`paired_greater`). The advantage is the per-seed mean of
//! `wealth_advantage` over ticks 200–500.

use std::collections::{BTreeMap, HashMap};
use std::sync::Mutex;

use rand::seq::SliceRandom;
use rand::SeedableRng;
use rand_pcg::Pcg64;
use sugarscape_core::agent::AgentId;
use sugarscape_core::config::{Belief, Config, DecisionRule, Map, MoveMode, Peak, URange};
use sugarscape_core::landscape::patch_of;
use sugarscape_core::world::World;

use crate::claim::{all_of, range, Claim, Outcome, Source};
use crate::claims::minds1::slope;
use crate::claims::minds2::paired_greater;
use crate::runner::{after, each_seed, each_seed_with, preset, series, window_mean};
use crate::stats::{self, median};

const SPEC: &str = "docs/superpowers/specs/2026-09-28-minds-3-memory-design.md";

/// What one 500-tick run says about rememberers and others.
pub(crate) struct Groups {
    /// Mean `wealth_advantage` over ticks 200–500.
    pub(crate) adv: f64,
    /// Mean `wealth_rememberers` and `wealth_others` over ticks 200–500.
    pub(crate) rem: f64,
    pub(crate) oth: f64,
    /// Mean `belief_error`, `stale_choices` and `remembered_moves` over the
    /// ticks 200–500 with at least one remembered choice (NaN with none).
    pub(crate) err: f64,
    pub(crate) stale: f64,
    pub(crate) moves: f64,
    /// Share of each group alive at tick 500 (of those alive at tick 0).
    pub(crate) rem_alive: f64,
    pub(crate) oth_alive: f64,
}

/// Rememberers and others alive at tick `t`.
fn group_sizes(w: &World, t: usize) -> (f64, f64) {
    let (share, pop) = (series(w, "remembering")[t], series(w, "population")[t]);
    let rem = (share * pop).round();
    (rem, pop - rem)
}

pub(crate) fn groups(c: &Config, seeds: &[u64]) -> Vec<Groups> {
    after(c, seeds, 500, measure)
}

/// `Groups` of a world run to tick 500.
pub(crate) fn measure(w: &World) -> Groups {
    let moves = series(w, "remembered_moves");
    let active: Vec<usize> = (200..=500).filter(|&t| moves[t] > 0.0).collect();
    let over_active = |name: &str| {
        let s = series(w, name);
        if active.is_empty() {
            f64::NAN
        } else {
            active.iter().map(|&t| s[t]).sum::<f64>() / active.len() as f64
        }
    };
    let (r0, o0) = group_sizes(w, 0);
    let (r1, o1) = group_sizes(w, 500);
    Groups {
        adv: window_mean(&series(w, "wealth_advantage"), 200, 500),
        rem: window_mean(&series(w, "wealth_rememberers"), 200, 500),
        oth: window_mean(&series(w, "wealth_others"), 200, 500),
        err: over_active("belief_error"),
        stale: over_active("stale_choices"),
        moves: over_active("remembered_moves"),
        rem_alive: r1 / r0,
        oth_alive: o1 / o0,
    }
}

pub(crate) fn col(g: &[Groups], f: impl Fn(&Groups) -> f64) -> Vec<f64> {
    g.iter().map(f).collect()
}

/// The medians a memory claim reports alongside its verdict.
pub(crate) fn describe(g: &[Groups]) -> String {
    format!(
        "Medians over seeds (ticks 200–500): advantage {:.2}; wealth rememberers {:.2}, others {:.2}; remembered_moves {:.3}; belief_error {:.3}; stale_choices {:.3}; alive at tick 500: rememberers {:.3}, others {:.3}.",
        median(&col(g, |x| x.adv)),
        median(&col(g, |x| x.rem)),
        median(&col(g, |x| x.oth)),
        median(&stats::finite(&col(g, |x| x.moves))),
        median(&stats::finite(&col(g, |x| x.err))),
        median(&stats::finite(&col(g, |x| x.stale))),
        median(&col(g, |x| x.rem_alive)),
        median(&col(g, |x| x.oth_alive)),
    )
}

/// Rememberers wealthier than others, paired within each seed.
pub(crate) fn richer(c: &Config, seeds: &[u64]) -> Outcome {
    let g = groups(c, seeds);
    paired_greater(
        &col(&g, |x| x.rem),
        &col(&g, |x| x.oth),
        "rememberers",
        "others",
    )
    .with(&describe(&g))
}

fn with_belief(mut c: Config, b: Belief) -> Config {
    c.memory.belief = b;
    c
}

/// The utility mind with travel cost `k` (crowding 0, idle as configured):
/// a site's value falls with its distance, unlike rule M's.
fn priced_travel(mut c: Config, k: f64) -> Config {
    c.decision.rule = DecisionRule::Utility;
    c.decision.travel = k;
    c
}

fn with_share(mut c: Config, share: f64) -> Config {
    c.memory.share = share;
    c
}

/// Truffles per Flump-tick over ticks 1..=`ticks` for (rememberers, others):
/// each group's truffles over the sum of its living counts at the start of
/// each tick.
fn truffle_rates(c: &Config, seeds: &[u64], ticks: u32) -> Vec<(f64, f64)> {
    after(c, seeds, ticks, |w| {
        let (found, by_rem) = (
            series(w, "truffles_found"),
            series(w, "truffles_by_rememberers"),
        );
        let (mut tr, mut to, mut nr, mut no) = (0.0, 0.0, 0.0, 0.0);
        for t in 1..found.len() {
            let (r, o) = group_sizes(w, t - 1);
            tr += by_rem[t];
            to += found[t] - by_rem[t];
            nr += r;
            no += o;
        }
        (tr / nr, to / no)
    })
}

// ---------------------------------------------------------------- traplining

/// Return lengths: for each visit, the number of spot visits since the last
/// visit to the same spot (first visits have none).
fn return_lengths(seq: &[u32]) -> Vec<f64> {
    let mut last = HashMap::new();
    let mut out = Vec::new();
    for (i, s) in seq.iter().enumerate() {
        if let Some(j) = last.insert(*s, i) {
            out.push((i - j) as f64);
        }
    }
    out
}

fn variance(v: &[f64]) -> f64 {
    if v.len() < 2 {
        return f64::NAN;
    }
    let m = stats::mean(v);
    v.iter().map(|x| (x - m).powi(2)).sum::<f64>() / (v.len() as f64 - 1.0)
}

/// Thomson, Slatkin & Thomson (1997)'s index of return variability: the
/// variance of the sequence's return lengths over its mean over 999
/// shuffles of the same sequence. 0 for a perfect trapliner, about 1 for
/// the null model. The shuffles draw from `rng`, never `World.rng`.
fn trapline_index(seq: &[u32], rng: &mut Pcg64) -> f64 {
    let observed = variance(&return_lengths(seq));
    let mut s = seq.to_vec();
    let mut sum = 0.0;
    for _ in 0..999 {
        s.shuffle(rng);
        sum += variance(&return_lengths(&s));
    }
    let null = sum / 999.0;
    if null > 0.0 {
        observed / null
    } else {
        f64::NAN
    }
}

/// A survey-side generator per (world seed, Flump).
fn shuffle_rng(seed: u64, id: AgentId) -> Pcg64 {
    Pcg64::seed_from_u64(seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ id.rotate_left(32))
}

/// One trapline run's visit logs, reduced.
struct Trap {
    /// Indices of each qualifying sequence (≥ 10 visits), by group.
    idx_rem: Vec<f64>,
    idx_oth: Vec<f64>,
    /// Ticks between successive arrivals at the same spot, rememberers only.
    intervals: Vec<f64>,
}

/// Steps `ticks` ticks, logging each Flump's arrivals at truffle spots (a
/// visit is ending a tick on a spot site it wasn't on the tick before).
fn trapline(c: &Config, seeds: &[u64], ticks: u32) -> Vec<Trap> {
    each_seed_with(c, seeds, move |seed, mut w| {
        let width = w.config.width;
        let mut last: HashMap<AgentId, (u32, u32)> =
            w.agents().map(|a| (a.id, (a.pos.x, a.pos.y))).collect();
        let mut logs: BTreeMap<AgentId, (bool, Vec<(u32, u64)>)> = BTreeMap::new();
        for _ in 0..ticks {
            w.step();
            let now = w.tick;
            for a in w.agents() {
                let here = (a.pos.x, a.pos.y);
                let moved = last.insert(a.id, here) != Some(here);
                if moved && w.truffle(a.pos).is_some() {
                    logs.entry(a.id)
                        .or_insert_with(|| (a.remembers, Vec::new()))
                        .1
                        .push((a.pos.y * width + a.pos.x, now));
                }
            }
        }
        let mut trap = Trap {
            idx_rem: Vec::new(),
            idx_oth: Vec::new(),
            intervals: Vec::new(),
        };
        for (id, (remembers, visits)) in &logs {
            if *remembers {
                let mut at: HashMap<u32, u64> = HashMap::new();
                for &(s, t) in visits {
                    if let Some(prev) = at.insert(s, t) {
                        trap.intervals.push((t - prev) as f64);
                    }
                }
            }
            if visits.len() < 10 {
                continue;
            }
            let seq: Vec<u32> = visits.iter().map(|v| v.0).collect();
            let i = trapline_index(&seq, &mut shuffle_rng(seed, *id));
            if i.is_finite() {
                if *remembers {
                    trap.idx_rem.push(i);
                } else {
                    trap.idx_oth.push(i);
                }
            }
        }
        trap
    })
}

/// The `q` quantile of the finite values, NaN with none.
pub(crate) fn q_or_nan(v: &[f64], q: f64) -> f64 {
    let v = stats::finite(v);
    if v.is_empty() {
        f64::NAN
    } else {
        stats::quantile(&v, q)
    }
}

pub(crate) fn med_or_nan(v: &[f64]) -> f64 {
    q_or_nan(v, 0.5)
}

// ----------------------------------------------------------------------- MVT

pub(crate) const SPACINGS: [u32; 4] = [12, 16, 20, 24];
pub(crate) const MVT_TICKS: u32 = 1000;

/// `mem-mvt` at lattice spacing `s`: nine peaks at (s/2 + s·i, s/2 + s·j)
/// on a 3s × 3s torus, so each patch keeps its share of the torus and the
/// 10 Flumps their 10/9 per patch; only the walk between patches changes.
/// Vision is 1–18 at every spacing (the 36 × 36 torus of spacing 12 allows
/// at most 18, half the grid), so s = 20 is the preset but for vision.
/// `utility` is the preset's mind (travel 0.5); otherwise rule M.
fn mvt(s: u32, utility: bool) -> Config {
    let mut c = preset("mem-mvt");
    let Map::Peaks { peaks } = &c.goods[0].map else {
        unreachable!("mem-mvt is a peaks map")
    };
    let (radius, height) = (peaks[0].radius, peaks[0].height);
    c.width = 3 * s;
    c.height = 3 * s;
    c.vision = URange::new(1, 18);
    c.goods[0].map = Map::Peaks {
        peaks: (0..3u32)
            .flat_map(|i| {
                (0..3u32).map(move |j| Peak {
                    x: s / 2 + s * i,
                    y: s / 2 + s * j,
                    radius,
                    height,
                })
            })
            .collect(),
    };
    if !utility {
        c.decision.rule = DecisionRule::Book;
        c.decision.travel = 0.0;
    }
    c
}

/// The marginal value theorem in both regimes these engines were tried in
/// (Minds 3, Task 10 fix rounds 1–3); the claims' details carry it.
const MVT_REGIMES: &str = "Rich patches (the preset): under the utility mind with travel, foragers who find a patch never leave it (rule M's foragers do move between patches, but its residence slope is unreliable; rule M's overstay figures are reported in `mem-mvt.overstay`). Depleting patches (tried in fix rounds 1 and 2, radius 2 and growback 0.05, 0.45 sugar a tick per patch): foragers starve before any switch; with 10 Flumps no one is alive after tick 200; with 3, no one is alive at tick 1000 and only 3 departures happen across 20 seeds. Likely reason: rule M and the utility mind compare the values of sites, not rates of intake, and hold no estimate of the habitat's average rate; and with sight only along rows and columns, a forager that has emptied a patch often has no other patch in sight. So the theorem's leave-when-your-rate-falls-to-the-average decision can't be expressed here; it is left to Minds 4 (planning).";

pub(crate) struct Visits {
    /// Mean length in ticks of completed patch visits (NaN with none).
    pub(crate) residence: f64,
    pub(crate) visits: usize,
    pub(crate) departures: usize,
    /// Share of the living Flumps on a patch at the last tick.
    pub(crate) on_patch: f64,
    /// Flumps alive at the last tick.
    pub(crate) alive: f64,
    /// Departures whose last tick in the patch gathered less than the
    /// Flump's mean gathered per tick so far.
    pub(crate) overstays: usize,
}

impl Visits {
    /// The share of departures that overstay (NaN with none).
    pub(crate) fn overstay_share(&self) -> f64 {
        if self.departures == 0 {
            f64::NAN
        } else {
            self.overstays as f64 / self.departures as f64
        }
    }
}

#[derive(Clone, Copy)]
struct Track {
    patch: Option<usize>,
    start: u64,
    /// The visit was already under way at tick 0 (not a completed visit).
    censored: bool,
    held: f64,
    gathered: f64,
    last_gain: f64,
}

/// Steps `MVT_TICKS` ticks, following each Flump's patch (`patch_of`) and
/// its sugar gathered each tick (the change in holdings plus metabolism;
/// nothing else moves sugar in these worlds: no births, no trade).
pub(crate) fn visits(c: &Config, seeds: &[u64]) -> Vec<Visits> {
    let Map::Peaks { peaks } = &c.goods[0].map else {
        unreachable!("a patch world is a peaks map")
    };
    let peaks = peaks.clone();
    let (gw, gh) = (c.width, c.height);
    each_seed(c, seeds, move |mut w| {
        let on = |x: u32, y: u32| patch_of(&peaks, x, y, gw, gh);
        let mut tracks: HashMap<AgentId, Track> = w
            .agents()
            .map(|a| {
                let patch = on(a.pos.x, a.pos.y);
                let t = Track {
                    patch,
                    start: 0,
                    censored: patch.is_some(),
                    held: a.holdings[0],
                    gathered: 0.0,
                    last_gain: 0.0,
                };
                (a.id, t)
            })
            .collect();
        let mut lengths = Vec::new();
        let (mut departures, mut overstays) = (0, 0);
        for _ in 0..MVT_TICKS {
            w.step();
            let t = w.tick;
            for a in w.agents() {
                let tr = tracks.get_mut(&a.id).expect("no births in a patch world");
                let gain = a.holdings[0] - tr.held + f64::from(a.metabolism[0]);
                let patch = on(a.pos.x, a.pos.y);
                if patch != tr.patch {
                    if tr.patch.is_some() {
                        if !tr.censored {
                            lengths.push((t - tr.start) as f64);
                        }
                        // The last tick in the patch was t − 1.
                        if t >= 2 {
                            departures += 1;
                            if tr.last_gain < tr.gathered / (t - 1) as f64 {
                                overstays += 1;
                            }
                        }
                    }
                    tr.patch = patch;
                    tr.start = t;
                    tr.censored = false;
                }
                tr.gathered += gain;
                tr.last_gain = gain;
                tr.held = a.holdings[0];
            }
        }
        Visits {
            residence: if lengths.is_empty() {
                f64::NAN
            } else {
                stats::mean(&lengths)
            },
            visits: lengths.len(),
            departures,
            alive: w.population() as f64,
            on_patch: {
                let pop = w.population() as f64;
                w.agents()
                    .filter(|a| on(a.pos.x, a.pos.y).is_some())
                    .count() as f64
                    / pop
            },
            overstays,
        }
    })
}

/// Per seed: the slope of mean residence on spacing, and each spacing's
/// residences; `world(s)` is the world at spacing `s`.
pub(crate) fn residence_slopes(
    seeds: &[u64],
    world: impl Fn(u32) -> Config,
) -> (Vec<f64>, Vec<Vec<Visits>>) {
    let runs: Vec<Vec<Visits>> = SPACINGS.iter().map(|&s| visits(&world(s), seeds)).collect();
    let x: Vec<f64> = SPACINGS.iter().map(|&s| f64::from(s)).collect();
    let slopes = (0..seeds.len())
        .map(|i| slope(&x, &runs.iter().map(|r| r[i].residence).collect::<Vec<_>>()))
        .collect();
    (slopes, runs)
}

/// Each spacing's median residence, visit count, alive and on-patch
/// figures, for a world of `pop` Flumps.
pub(crate) fn residence_medians(runs: &[Vec<Visits>], pop: u32) -> String {
    SPACINGS
        .iter()
        .zip(runs)
        .map(|(s, r)| {
            let res: Vec<f64> = r.iter().map(|v| v.residence).collect();
            let n: usize = r.iter().map(|v| v.visits).sum();
            let on: Vec<f64> = r.iter().map(|v| v.on_patch).collect();
            let alive: Vec<f64> = r.iter().map(|v| v.alive).collect();
            format!(
                "{s}: {:.2} ({n} completed visits; at tick {MVT_TICKS}, a median {:.1} of the {pop} Flumps alive and a median share {:.2} of them on a patch)",
                med_or_nan(&stats::finite(&res)),
                median(&alive),
                med_or_nan(&on)
            )
        })
        .collect::<Vec<_>>()
        .join(", ")
}

// ---------------------------------------------------------------- forgetting

const RATES: [f64; 3] = [0.25, 0.5, 1.0];
const SPANS: [u32; 6] = [10, 25, 50, 100, 200, 400];

/// The `mem-span-*` grid: per rate, per span, per seed advantages.
type Grid = Vec<Vec<Vec<f64>>>;

static GRIDS: Mutex<Vec<(Belief, Vec<u64>, Grid)>> = Mutex::new(Vec::new());

/// The `mem-span-recall` or `mem-span-project` sweep's metric per seed:
/// `mem-open` under `belief` at each growback rate and span. Cached, since
/// two claims share the recall grid.
fn grid(belief: Belief, seeds: &[u64]) -> Grid {
    if let Some((_, _, g)) = GRIDS
        .lock()
        .unwrap()
        .iter()
        .find(|(b, s, _)| *b == belief && s == seeds)
    {
        return g.clone();
    }
    let g: Grid = RATES
        .iter()
        .map(|&rate| {
            SPANS
                .iter()
                .map(|&span| {
                    let mut c = with_belief(preset("mem-open"), belief);
                    c.growback.rate = rate;
                    c.memory.span = span;
                    after(&c, seeds, 500, |w| {
                        window_mean(&series(w, "wealth_advantage"), 200, 500)
                    })
                })
                .collect()
        })
        .collect();
    GRIDS
        .lock()
        .unwrap()
        .push((belief, seeds.to_vec(), g.clone()));
    g
}

/// Per rate, per seed: the span with the largest advantage (the shortest on
/// a tie).
fn best_spans(g: &Grid) -> Vec<Vec<f64>> {
    g.iter()
        .map(|by_span| {
            (0..by_span[0].len())
                .map(|i| {
                    let mut best = 0;
                    for k in 1..SPANS.len() {
                        if by_span[k][i] > by_span[best][i] {
                            best = k;
                        }
                    }
                    f64::from(SPANS[best])
                })
                .collect()
        })
        .collect()
}

/// What the best span means when memory never pays: if every median
/// advantage in the grid is negative, the "best" span is only the least
/// harmful one, and a best span stuck at the grid's shortest can't show
/// forgetting tracking regrowth. Empty otherwise.
fn least_harmful(g: &Grid, best: &[Vec<f64>]) -> String {
    let all_negative = g.iter().flatten().all(|v| median(v) < 0.0);
    if !all_negative {
        return String::new();
    }
    let floor = f64::from(SPANS[0]);
    let fast = best.last().expect("a rate");
    let at_floor = if stats::quantile(fast, 0.25) == floor && stats::quantile(fast, 0.75) == floor {
        format!(
            " At growback {} it sits at the shortest span tested ({floor:.0}, IQR {floor:.0}–{floor:.0}), the floor of the grid, so the comparison can't show forgetting tracking regrowth: it shows only that memory hurts least when it's shortest.",
            RATES[RATES.len() - 1]
        )
    } else {
        String::new()
    };
    format!(
        "The median advantage is negative at every span and rate, so the \"best\" span is the least-harmful span, not one where memory pays.{at_floor}"
    )
}

fn grid_summary(g: &Grid) -> String {
    RATES
        .iter()
        .zip(g)
        .map(|(rate, by_span)| {
            let cells: Vec<String> = SPANS
                .iter()
                .zip(by_span)
                .map(|(s, v)| format!("{s}: {:.1}", median(v)))
                .collect();
            format!("growback {rate}: {}", cells.join(", "))
        })
        .collect::<Vec<_>>()
        .join("; ")
}

fn best_summary(best: &[Vec<f64>]) -> String {
    RATES
        .iter()
        .zip(best)
        .map(|(r, b)| {
            format!(
                "{r}: {:.0} (IQR {:.0}–{:.0})",
                median(b),
                stats::quantile(b, 0.25),
                stats::quantile(b, 0.75)
            )
        })
        .collect::<Vec<_>>()
        .join(", ")
}

// ------------------------------------------------------------------ catchment

struct Catch {
    adv: f64,
    rem: f64,
    oth: f64,
    /// Median first tick on a patch per group (never reached counts as 501).
    first_rem: f64,
    first_oth: f64,
    never_rem: usize,
    never_oth: usize,
    /// Share of each group on a patch, over ticks 200–500.
    on_rem: f64,
    on_oth: f64,
}

fn catchment(c: &Config, seeds: &[u64]) -> Vec<Catch> {
    let Map::Peaks { peaks } = &c.goods[0].map else {
        unreachable!("mem-catchment is a peaks map")
    };
    let peaks = peaks.clone();
    let (gw, gh) = (c.width, c.height);
    each_seed(c, seeds, move |mut w| {
        let on = |x: u32, y: u32| patch_of(&peaks, x, y, gw, gh).is_some();
        let mut first: BTreeMap<AgentId, (bool, Option<u64>)> = w
            .agents()
            .map(|a| (a.id, (a.remembers, on(a.pos.x, a.pos.y).then_some(0))))
            .collect();
        let (mut on_r, mut n_r, mut on_o, mut n_o) = (0.0, 0.0, 0.0, 0.0);
        for _ in 0..500 {
            w.step();
            let t = w.tick;
            for a in w.agents() {
                let here = on(a.pos.x, a.pos.y);
                let f = first.get_mut(&a.id).expect("nobody is born");
                if here && f.1.is_none() {
                    f.1 = Some(t);
                }
                if t >= 200 {
                    let v = if here { 1.0 } else { 0.0 };
                    if a.remembers {
                        on_r += v;
                        n_r += 1.0;
                    } else {
                        on_o += v;
                        n_o += 1.0;
                    }
                }
            }
        }
        let times = |r: bool| -> Vec<f64> {
            first
                .values()
                .filter(|(m, _)| *m == r)
                .map(|(_, t)| t.map_or(501.0, |t| t as f64))
                .collect()
        };
        let never = |r: bool| {
            first
                .values()
                .filter(|(m, t)| *m == r && t.is_none())
                .count()
        };
        Catch {
            adv: window_mean(&series(&w, "wealth_advantage"), 200, 500),
            rem: window_mean(&series(&w, "wealth_rememberers"), 200, 500),
            oth: window_mean(&series(&w, "wealth_others"), 200, 500),
            first_rem: med_or_nan(&times(true)),
            first_oth: med_or_nan(&times(false)),
            never_rem: never(true),
            never_oth: never(false),
            on_rem: on_r / n_r,
            on_oth: on_o / n_o,
        }
    })
}

/// Mean population over ticks 300–500.
fn capacity(c: &Config, seeds: &[u64]) -> Vec<f64> {
    after(c, seeds, 500, |w| {
        window_mean(&series(w, "population"), 300, 500)
    })
}

fn trapline_mixed() -> Config {
    with_share(preset("mem-trapline"), 0.5)
}

const TRAP_TICKS: u32 = 1000;

pub fn claims() -> Vec<Claim> {
    vec![
        Claim {
            id: "mem-open.about-zero",
            item: "mem-open",
            source: Source::Comment,
            citation: SPEC,
            text: "Memory is worth about nothing on the open scape: the rememberers' mean wealth is within 10 % of the others', seed by seed (ticks 200–500)",
            check: |seeds| {
                let g = groups(&preset("mem-open"), seeds);
                let rel: Vec<f64> = g.iter().map(|x| x.adv / x.oth).collect();
                range(&rel, -0.1, 0.1, false).with(&format!(
                    "Per-seed advantage ÷ the others' wealth. {}",
                    describe(&g)
                ))
            },
        },
        Claim {
            id: "mem-walled.advantage",
            item: "mem-walled",
            source: Source::Comment,
            citation: SPEC,
            text: "Memory pays behind a wall: rememberers are wealthier than others (ticks 200–500), seed by seed",
            check: |seeds| {
                let recall = groups(&with_belief(preset("mem-walled"), Belief::Recall), seeds);
                let losing = recall.iter().filter(|x| x.adv < 0.0).count();
                richer(&preset("mem-walled"), seeds).with(&format!(
                    "Under recall: median advantage {:.2}, negative in {losing} of {} seeds; alive at tick 500: rememberers {:.3}, others {:.3}.",
                    median(&col(&recall, |x| x.adv)),
                    recall.len(),
                    median(&col(&recall, |x| x.rem_alive)),
                    median(&col(&recall, |x| x.oth_alive)),
                ))
            },
        },
        Claim {
            id: "mem-seasons.advantage",
            item: "mem-seasons",
            source: Source::Comment,
            citation: SPEC,
            text: "Memory pays under seasons: rememberers are wealthier than others (ticks 200–500), seed by seed",
            check: |seeds| richer(&preset("mem-seasons"), seeds),
        },
        Claim {
            id: "mem-truffles.advantage",
            item: "mem-truffles",
            source: Source::Comment,
            citation: SPEC,
            text: "Memory pays with hidden truffles: rememberers are wealthier than others (ticks 200–500), seed by seed",
            check: |seeds| richer(&preset("mem-truffles"), seeds),
        },
        Claim {
            id: "mem-trapline.advantage",
            item: "mem-trapline",
            source: Source::Comment,
            citation: SPEC,
            text: "Memory pays on the trapline world: with half the Flumps remembering (share 0.5), rememberers are wealthier than others (ticks 200–500), seed by seed",
            check: |seeds| richer(&trapline_mixed(), seeds),
        },
        Claim {
            id: "mem-catchment.advantage",
            item: "mem-catchment",
            source: Source::Comment,
            citation: SPEC,
            text: "Memory pays when patches fall out of sight: rememberers are wealthier than others (ticks 200–500), seed by seed; the first tick each group reaches a patch is reported",
            check: |seeds| {
                let c = catchment(&preset("mem-catchment"), seeds);
                let f = |g: fn(&Catch) -> f64| c.iter().map(g).collect::<Vec<f64>>();
                let never_r: usize = c.iter().map(|x| x.never_rem).sum();
                let never_o: usize = c.iter().map(|x| x.never_oth).sum();
                paired_greater(&f(|x| x.rem), &f(|x| x.oth), "rememberers", "others").with(&format!(
                    "Medians over seeds: advantage {:.2}; first tick on a patch (per seed, the median Flump; never reached by tick 500 counts as 501): rememberers {:.1}, others {:.1}; never reached, over all seeds: rememberers {never_r}, others {never_o}; share on a patch over ticks 200–500: rememberers {:.3}, others {:.3}.",
                    median(&f(|x| x.adv)),
                    median(&f(|x| x.first_rem)),
                    median(&f(|x| x.first_oth)),
                    median(&f(|x| x.on_rem)),
                    median(&f(|x| x.on_oth)),
                ))
            },
        },
        Claim {
            id: "mem-truffles.intake",
            item: "mem-truffles",
            source: Source::Comment,
            citation: SPEC,
            text: "Rememberers gather more truffles per head than others (ticks 1–500, per Flump-tick alive)",
            check: |seeds| {
                let r = truffle_rates(&preset("mem-truffles"), seeds, 500);
                let (a, b): (Vec<f64>, Vec<f64>) = r.into_iter().unzip();
                paired_greater(&a, &b, "rememberers", "others")
            },
        },
        Claim {
            id: "mem-trapline.index",
            item: "mem-trapline",
            source: Source::Book,
            citation: "Thomson, Slatkin & Thomson 1997",
            text: "Rememberers trapline: per seed, the median index of return variability (variance of return lengths over its mean in 999 shuffles; 0 a perfect trapliner, 1 the null) is below 0.8 in at least 80 % of seeds (ticks 1–1000, sequences of at least 10 spot visits)",
            check: |seeds| {
                let t = trapline(&preset("mem-trapline"), seeds, TRAP_TICKS);
                let meds: Vec<f64> = t.iter().map(|x| med_or_nan(&x.idx_rem)).collect();
                let below1 = meds.iter().filter(|&&m| m < 1.0).count();
                let n: usize = t.iter().map(|x| x.idx_rem.len()).sum();
                let mixed = trapline(&trapline_mixed(), seeds, TRAP_TICKS);
                let mr: Vec<f64> = stats::finite(&mixed.iter().map(|x| med_or_nan(&x.idx_rem)).collect::<Vec<_>>());
                let mo: Vec<f64> = stats::finite(&mixed.iter().map(|x| med_or_nan(&x.idx_oth)).collect::<Vec<_>>());
                range(&meds, 0.0, 0.8, false).with(&format!(
                    "Every Flump remembers (the preset); {n} qualifying sequences over all seeds; the per-seed median is below 1 in {below1} of {} seeds. Control, share 0.5: median of per-seed medians, rememberers {:.3} ({} seeds), non-rememberers {:.3} ({} seeds).",
                    meds.len(),
                    med_or_nan(&mr),
                    mr.len(),
                    med_or_nan(&mo),
                    mo.len(),
                ))
            },
        },
        Claim {
            id: "mem-trapline.gill",
            item: "mem-trapline",
            source: Source::Book,
            citation: "Gill 1988",
            text: "Competition shortens revisits: the median interval between a rememberer's visits to the same spot is shorter with 20 Flumps than with 5 (ticks 1–1000), seed by seed",
            check: |seeds| {
                let at = |pop: u32| {
                    let mut c = preset("mem-trapline");
                    c.population = pop;
                    trapline(&c, seeds, TRAP_TICKS)
                };
                let (five, twenty) = (at(5), at(20));
                let med = |t: &[Trap]| t.iter().map(|x| med_or_nan(&x.intervals)).collect::<Vec<f64>>();
                let (m5, m20) = (med(&five), med(&twenty));
                let under = |t: &[Trap]| {
                    let all: Vec<f64> = t.iter().flat_map(|x| x.intervals.clone()).collect();
                    all.iter().filter(|&&i| i < 40.0).count() as f64 / all.len() as f64
                };
                paired_greater(&m5, &m20, "5 Flumps", "20 Flumps").with(&format!(
                    "Against regrow 40: median of per-seed median intervals, 5 Flumps {:.1}, 20 Flumps {:.1}; share of all revisits sooner than 40 ticks, 5 Flumps {:.3}, 20 Flumps {:.3}.",
                    median(&stats::finite(&m5)),
                    median(&stats::finite(&m20)),
                    under(&five),
                    under(&twenty),
                ))
            },
        },
        Claim {
            id: "mem-trapline.payoff",
            item: "mem-trapline",
            source: Source::Book,
            citation: "Ohashi & Thomson 2005",
            text: "Traplining is more competitive: with half the Flumps remembering (share 0.5), rememberers gather more truffles per head than others (ticks 1–1000, per Flump-tick alive), seed by seed",
            check: |seeds| {
                let r = truffle_rates(&trapline_mixed(), seeds, TRAP_TICKS);
                let (a, b): (Vec<f64>, Vec<f64>) = r.into_iter().unzip();
                paired_greater(&a, &b, "rememberers", "others")
            },
        },
        Claim {
            id: "mem-mvt.travel",
            item: "mem-mvt",
            source: Source::Book,
            citation: "Charnov 1976; Stephens & Krebs 1986, Fig. 2.2",
            text: "Longer travel, longer stays: under the utility mind with travel, mean patch residence rises with the patches' spacing (the per-seed slope over spacings 12, 16, 20 and 24 is above 0 in at least 80 % of seeds); under rule M the slope is reported",
            check: |seeds| {
                let (u, ur) = residence_slopes(seeds, |s| mvt(s, true));
                let (m, mr) = residence_slopes(seeds, |s| mvt(s, false));
                let pop = preset("mem-mvt").population;
                range(&u, f64::MIN_POSITIVE, f64::INFINITY, false).with(&format!(
                    "Nine peaks on a 3s × 3s torus at spacing s, vision 1–18 throughout (s = 20 is the preset but for vision); completed visits over ticks 1–{MVT_TICKS} (visits under way at tick 0 or at the end, or cut short by death, are dropped); residences are medians over seeds of per-seed means. A slope needs at least 3 spacings with a completed visit: finite for utility with travel in {} of {} seeds, for rule M in {} of {}, positive in {}. Median residence by spacing, utility with travel: {}. Rule M: slope median {:.4} (IQR {:.4}–{:.4}); residence by spacing {}.",
                    stats::finite(&u).len(),
                    u.len(),
                    stats::finite(&m).len(),
                    m.len(),
                    m.iter().filter(|&&x| x > 0.0).count(),
                    residence_medians(&ur, pop),
                    med_or_nan(&m),
                    q_or_nan(&m, 0.25),
                    q_or_nan(&m, 0.75),
                    residence_medians(&mr, pop),
                ))
                .with(MVT_REGIMES)
            },
        },
        Claim {
            id: "mem-mvt.overstay",
            item: "mem-mvt",
            source: Source::Book,
            citation: "Nonacs 2001; Hayden, Pearson & Platt 2011",
            text: "Flumps overstay: at over half of departures, the sugar gathered on the last tick in the patch is below the Flump's mean gathered per tick so far, in at least 80 % of seeds",
            check: |seeds| {
                let v = visits(&preset("mem-mvt"), seeds);
                let share: Vec<f64> = v.iter().map(Visits::overstay_share).collect();
                let deps: usize = v.iter().map(|x| x.departures).sum();
                let on: Vec<f64> = v.iter().map(|x| x.on_patch).collect();
                let alive: Vec<f64> = v.iter().map(|x| x.alive).collect();
                let pop = preset("mem-mvt").population;
                let alive_list = alive
                    .iter()
                    .map(|a| format!("{a:.0}"))
                    .collect::<Vec<_>>()
                    .join(" ");
                let rule_m = |vision: u32| {
                    let mut c = preset("mem-mvt");
                    c.decision.rule = DecisionRule::Book;
                    c.decision.travel = 0.0;
                    c.vision = URange::new(1, vision);
                    visits(&c, seeds)
                        .iter()
                        .filter(|x| x.departures > 0)
                        .map(|x| x.overstays as f64 / x.departures as f64)
                        .collect::<Vec<f64>>()
                };
                let (m20, m18) = (rule_m(20), rule_m(18));
                range(&share, 0.5 + f64::EPSILON, 1.0, false).with(&format!(
                    "Per-seed share of departures that overstay, mem-mvt (utility with travel), ticks 1–{MVT_TICKS}; {deps} departures over all seeds; at tick {MVT_TICKS}, a median {:.1} of the {pop} Flumps alive (per seed: {alive_list}), a median share {:.2} of them on a patch. Rule M's figures, on mem-mvt's world: median share {:.3} at the preset's vision 1–20 ({} seeds with departures), {:.3} at vision 1–18 as in the spacing runs ({} seeds).",
                    median(&alive),
                    med_or_nan(&on),
                    med_or_nan(&m20),
                    m20.len(),
                    med_or_nan(&m18),
                    m18.len()
                ))
                .with(MVT_REGIMES)
            },
        },
        Claim {
            id: "mem-span-recall.bracis",
            item: "mem-span-recall",
            source: Source::Book,
            citation: "Bracis et al. 2015",
            text: "Forgetting tracks regrowth: under recall, the span that maximizes the rememberers' advantage is shorter at growback 1 than at 0.25, seed by seed",
            check: |seeds| {
                let g = grid(Belief::Recall, seeds);
                let best = best_spans(&g);
                let out = paired_greater(&best[0], &best[2], "best span at 0.25", "best span at 1")
                    .with(&format!(
                        "Best span per seed, median by growback rate: {}. Median advantage by rate and span: {}.",
                        best_summary(&best),
                        grid_summary(&g)
                    ));
                match least_harmful(&g, &best) {
                    note if note.is_empty() => out,
                    note => out.with(&note),
                }
            },
        },
        Claim {
            id: "mem-span-project.longer",
            item: "mem-span-project",
            source: Source::Comment,
            citation: SPEC,
            text: "Projection misleads less: under project, the best span is longer than under recall, seed by seed, at each growback rate",
            check: |seeds| {
                let (gp, gr) = (grid(Belief::Project, seeds), grid(Belief::Recall, seeds));
                let (bp, br) = (best_spans(&gp), best_spans(&gr));
                all_of(
                    RATES
                        .iter()
                        .enumerate()
                        .map(|(k, rate)| {
                            (
                                format!("growback {rate}"),
                                paired_greater(&bp[k], &br[k], "project", "recall"),
                            )
                        })
                        .collect(),
                )
                .with(&format!(
                    "Best span per seed under project, median by rate: {}. Median advantage under project by rate and span: {}.",
                    best_summary(&bp),
                    grid_summary(&gp)
                ))
            },
        },
        Claim {
            id: "mem-open.hornvale",
            item: "mem-open",
            source: Source::Comment,
            citation: "Hornvale M2c1",
            text: "Projection beats recall on the open scape: under project the rememberers' advantage is larger and belief_error smaller than under recall, seed by seed",
            check: |seeds| hornvale("mem-open", seeds),
        },
        Claim {
            id: "mem-truffles.hornvale",
            item: "mem-truffles",
            source: Source::Comment,
            citation: "Hornvale M2c1",
            text: "Projection beats recall with truffles: under project the rememberers' advantage is larger and belief_error smaller than under recall, seed by seed",
            check: |seeds| hornvale("mem-truffles", seeds),
        },
        Claim {
            id: "mem-open.travel",
            item: "mem-open",
            source: Source::Comment,
            citation: SPEC,
            text: "Memory pays only when travel is priced: in mem-open's world (memory, belief and share as in the preset), under the utility mind with travel 0.5 the rememberers' advantage exceeds rule M's, and rememberers are wealthier than others (ticks 200–500), seed by seed",
            check: |seeds| {
                let rule_m = groups(&preset("mem-open"), seeds);
                let priced = groups(&priced_travel(preset("mem-open"), 0.5), seeds);
                let other = |id: &str| {
                    let m = groups(&preset(id), seeds);
                    let t = groups(&priced_travel(preset(id), 0.5), seeds);
                    format!(
                        "{id}: rule M median advantage {:.2}, travel 0.5 {:.2} (travel 0.5 higher in {} of {} seeds); remembered_moves rule M {:.3}, travel 0.5 {:.3}",
                        median(&col(&m, |x| x.adv)),
                        median(&col(&t, |x| x.adv)),
                        t.iter().zip(&m).filter(|(a, b)| a.adv > b.adv).count(),
                        seeds.len(),
                        median(&stats::finite(&col(&m, |x| x.moves))),
                        median(&stats::finite(&col(&t, |x| x.moves))),
                    )
                };
                all_of(vec![
                    (
                        "travel 0.5 against rule M".into(),
                        paired_greater(
                            &col(&priced, |x| x.adv),
                            &col(&rule_m, |x| x.adv),
                            "travel 0.5",
                            "rule M",
                        ),
                    ),
                    (
                        "rememberers against others under travel 0.5".into(),
                        paired_greater(
                            &col(&priced, |x| x.rem),
                            &col(&priced, |x| x.oth),
                            "rememberers",
                            "others",
                        ),
                    ),
                ])
                .with(&format!(
                    "Rule M: {} Travel 0.5: {} Same switch elsewhere: {}; {}.",
                    describe(&rule_m),
                    describe(&priced),
                    other("mem-walled"),
                    other("mem-truffles")
                ))
            },
        },
        Claim {
            id: "mem-open.capacity",
            item: "mem-open",
            source: Source::Comment,
            citation: SPEC,
            text: "Memory for everyone restores walking's lost capacity: walk-capacity with memory (span 100, share 1) holds a larger population than without (mean over ticks 300–500), seed by seed",
            check: |seeds| {
                let base = preset("walk-capacity");
                let mut with = base.clone();
                with.memory.span = 100;
                with.memory.share = 1.0;
                assert_eq!(with.movement.mode, MoveMode::Walk);
                let off = capacity(&base, seeds);
                let recall = capacity(&with_belief(with.clone(), Belief::Recall), seeds);
                let jump = capacity(&preset("ii-2-unit"), seeds);
                paired_greater(&capacity(&with, seeds), &off, "memory (project)", "no memory").with(&format!(
                    "Medians: no memory {:.1}; memory under recall {:.1}; jump (ii-2-unit) {:.1}.",
                    median(&off),
                    median(&recall),
                    median(&jump)
                ))
            },
        },
    ]
}

/// `project` against `recall` in `id`, paired on the advantage and on
/// `belief_error`.
fn hornvale(id: &str, seeds: &[u64]) -> Outcome {
    let p = groups(&with_belief(preset(id), Belief::Project), seeds);
    let r = groups(&with_belief(preset(id), Belief::Recall), seeds);
    all_of(vec![
        (
            "advantage".into(),
            paired_greater(
                &col(&p, |x| x.adv),
                &col(&r, |x| x.adv),
                "project",
                "recall",
            ),
        ),
        (
            "belief_error".into(),
            paired_greater(
                &col(&r, |x| x.err),
                &col(&p, |x| x.err),
                "recall",
                "project",
            ),
        ),
    ])
    .with(&format!(
        "Project: {} Recall: {}",
        describe(&p),
        describe(&r)
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn return_lengths_count_visits_since_the_same_spot() {
        assert_eq!(
            return_lengths(&[1, 2, 3, 1, 2, 3, 1]),
            vec![3.0, 3.0, 3.0, 3.0]
        );
        assert_eq!(return_lengths(&[1, 1, 2, 1]), vec![1.0, 2.0]);
    }

    #[test]
    fn a_perfect_trapline_scores_zero_and_shuffles_are_reproducible() {
        let seq: Vec<u32> = (0..30).map(|i| i % 5).collect();
        assert_eq!(trapline_index(&seq, &mut shuffle_rng(1, 7)), 0.0);
        let messy = [1, 2, 1, 3, 3, 2, 4, 1, 2, 4, 3, 1, 1, 2];
        let a = trapline_index(&messy, &mut shuffle_rng(1, 7));
        assert_eq!(a, trapline_index(&messy, &mut shuffle_rng(1, 7)));
        assert!(a > 0.0 && a.is_finite(), "{a}");
    }

    #[test]
    fn spacing_20_is_the_preset_but_for_vision() {
        let (a, mut b) = (mvt(20, true), preset("mem-mvt"));
        b.vision = URange::new(1, 18);
        assert_eq!(a, b, "spacing 20 is the preset at vision 1–18");
        let m = mvt(12, false);
        assert_eq!((m.width, m.decision.rule), (36, DecisionRule::Book));
    }
}
