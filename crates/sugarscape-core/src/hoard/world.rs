//! The hoarding world: Vander Wall and Jenkins's (2003) genetic algorithm,
//! exactly as the spec's amendments fix it
//! (docs/superpowers/specs/2026-09-30-minds-7-hoarding-evolution-design.md,
//! "Amendments (implementation)"). One tick is one foraging bout: 20 bouts a
//! day, 100 days a season, one season a generation, `generations` seasons a
//! run. The switches `owner_recovery` and `cheaters` are Task 5 (read, but
//! their defaults, free recovery and no cheaters, are what runs here).
//!
//! **A day** (amendments "Methods as read", items 2, 5 and 7). At bout 1 the
//! day's production is added to the public pool (days 1–`food_days`). On
//! days 1–`nonstorable_days` every living agent is satiated all day (fed by
//! nonstorable food); from the next day on every living agent starts hungry.
//! At the end of each day after the nonstorable days, every living agent
//! still hungry starves. Raids end with the day.
//!
//! **A bout** (item 1), in this order:
//! 1. bout-1 eating (bout 1 only, days after the nonstorable days, or days
//!    2–5 under `early_bout1_eats`), agents in index order: an agent with
//!    stores eats one, larder first; a larder eater defends this bout with
//!    probability 1, a scatter eater with probability 0 (no draw either way);
//!    under `early_bout1_eats` an agent without stores on days 2–5 is flagged
//!    hungry, so its first find that day is eaten (nobody starves then);
//! 2. a defense draw for every other living, satiated agent, in index order;
//! 3. a predation draw for every living agent, in index order;
//! 4. a fresh shuffle of the living agents that aren't defending; each takes
//!    one foraging turn: a raid continuation if it took a larder item last
//!    bout (item 4), otherwise (or if that fails) a search (item 5).
//!
//! **A generation** (items 8, 9 and 13). A season ends after its last bout,
//! or at the end of the bout in which its last agent dies. Its per-agent
//! records and measurements are kept then, in `seasons`, before anything is
//! reset. The next bout, if the run goes on, first breeds `n` offspring:
//! mother and father drawn independently, with replacement, in proportion to
//! the survivors' leftover stores (uniformly among the survivors if they hold
//! none); L and D inherited on the logit scale, forage drawn fresh. The new
//! generation starts with empty stores and an empty public pool. The run
//! ends when the population dies out (it is recorded as extinct) or when the
//! season of generation `generations` (read live) is over; lowering
//! `generations` below the current generation ends the run at the end of the
//! current season.
//!
//! **Draw order** (one seeded RNG): founders draw, per agent in index order,
//! logit L, logit D and forage efficiency (each a standard normal). Each
//! bout then draws, in the order above: one uniform per defense draw, one
//! uniform per living agent for predation, the shuffle, and within each
//! turn: a search draws one uniform for detection (always, even with no food
//! available) and, on a find, one uniform for the item; a stored item draws
//! one uniform against L. Eating draws nothing. Breeding draws, per offspring
//! in index order, one uniform for the mother, one for the father, then a
//! standard normal each for logit L, logit D and forage.
//!
//! **Statistics** are one snapshot per bout, and tick 0 (item 13).

use std::fmt::Write;

use rand::seq::SliceRandom;
use rand::Rng;
use serde::Serialize;

use crate::anasazi::random::normal;
use crate::config::FieldError;
use crate::export;
use crate::model::{wrong_model, Model, ModelConfig, ModelKind};
use crate::portable::{exp_neg, ln};
use crate::rng::{self, SimRng};
use crate::stats::Stats;

use super::config::{DeadStores, DefendedInPool, HoardConfig, LarderWeight};
use super::stats::HoardSnapshot;

/// The frame's size in cells.
pub const WIDE: usize = 64;
pub const TALL: usize = 32;

const BACKGROUND: [u8; 4] = [16, 18, 24, 255];

/// How an agent died.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Cause {
    /// Taken by a predator (0.0001 a bout).
    Predation,
    /// Still hungry at the end of a day after the nonstorable days.
    Starvation,
}

/// When and how an agent died.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct Death {
    pub day: u32,
    pub bout: u32,
    pub cause: Cause,
}

/// One agent's season, as the loss rates (amendments item 10), the survey
/// and Task 4 need it. Losses and stocks count only while the agent lives.
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct Record {
    /// Items other agents took from its larder and its scattered caches.
    pub larder_lost: u64,
    pub scatter_lost: u64,
    /// The sums, over the bouts it began alive, of its larder and scatter
    /// stock at the bout's start.
    pub larder_stock: u64,
    pub scatter_stock: u64,
    /// Bouts it began alive.
    pub bouts_alive: u64,
    /// Items it ate (from its stores or on finding them).
    pub eaten: u64,
    /// Items it harvested: from the public pool, from others' scattered
    /// caches, from others' larders (search and raid continuations).
    pub took_public: u64,
    pub took_scatter: u64,
    pub took_larder: u64,
}

impl Record {
    /// The per-item daily loss rate (amendments item 10): items lost ÷
    /// (Σ over bouts of the stock at the bout's start ÷ `bouts`). `None` when
    /// it never held any at a bout's start.
    fn rate(lost: u64, stock: u64, bouts: u32) -> Option<f64> {
        (stock > 0).then(|| lost as f64 / (stock as f64 / f64::from(bouts)))
    }

    pub fn larder_rate(&self, bouts: u32) -> Option<f64> {
        Self::rate(self.larder_lost, self.larder_stock, bouts)
    }

    pub fn scatter_rate(&self, bouts: u32) -> Option<f64> {
        Self::rate(self.scatter_lost, self.scatter_stock, bouts)
    }

    /// The mean larder stock at bout starts while alive (claim 4's exposure
    /// floor is ≥ 1).
    pub fn mean_larder_stock(&self) -> f64 {
        if self.bouts_alive == 0 {
            0.0
        } else {
            self.larder_stock as f64 / self.bouts_alive as f64
        }
    }

    pub fn mean_scatter_stock(&self) -> f64 {
        if self.bouts_alive == 0 {
            0.0
        } else {
            self.scatter_stock as f64 / self.bouts_alive as f64
        }
    }

    /// Days alive: bouts begun alive ÷ bouts a day.
    pub fn days_alive(&self, bouts: u32) -> f64 {
        self.bouts_alive as f64 / f64::from(bouts)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Agent {
    /// Probability of larder hoarding a stored item, and propensity to
    /// defend (where the defense target sits between min and max).
    pub l: f64,
    pub d: f64,
    /// Foraging efficiency: N(1, `forage_sd`), floored at 0.
    pub forage: f64,
    pub larder: u32,
    pub scatter: u32,
    pub alive: bool,
    /// Satiated today (state 0) or hungry (state −1).
    pub fed: bool,
    /// Defending its burrow this bout.
    pub defending: bool,
    /// The burrow it took a larder item from last bout, today.
    pub raid: Option<usize>,
    pub death: Option<Death>,
    pub record: Record,
}

/// A season's summary numbers.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct SeasonSummary {
    pub survivors: u32,
    pub starved: u32,
    pub preyed: u32,
    /// Larder items ÷ all items stored by the survivors (their fitness), or
    /// `None` when they hold nothing.
    pub larder_share: Option<f64>,
    /// The mean per-agent loss rates over agents with a rate, and how many
    /// had one.
    pub mean_larder_rate: Option<f64>,
    pub mean_scatter_rate: Option<f64>,
    pub larder_rated: u32,
    pub scatter_rated: u32,
    /// Agents that lost items of the type but never held any at a bout's
    /// start (counted, not dropped silently; item 10).
    pub larder_unrated_losses: u32,
    pub scatter_unrated_losses: u32,
    /// The pooled rates: Σ lost ÷ (Σ bout-start stock ÷ bouts a day) over
    /// all agents, or `None` when none held any.
    pub pooled_larder_rate: Option<f64>,
    pub pooled_scatter_rate: Option<f64>,
    /// Claim 4's exposure floor (item 10): agents whose mean larder stock at
    /// bout starts while alive is at least `EXPOSURE_FLOOR`, and the minimum
    /// larder rate among them and among every agent with a rate.
    pub larder_exposed: u32,
    pub min_exposed_larder_rate: Option<f64>,
    pub min_larder_rate: Option<f64>,
    /// Means over the agents born into the generation.
    pub mean_l: f64,
    pub mean_d: f64,
}

/// Claim 4's exposure floor: a mean larder stock of at least 1 item at bout
/// starts while alive (amendments item 10, fixed before any run).
pub const EXPOSURE_FLOOR: f64 = 1.0;

/// Traits are clamped to (ε, 1 − ε) before the logit (item 8; Review Focus 3).
pub const EPSILON: f64 = 1e-6;

/// Item 11's thresholds and window.
pub const TAKEOVER_L: f64 = 0.95;
pub const LOW_L: f64 = 0.2;
pub const WINDOW: usize = 10;

/// One finished season, kept before breeding resets the agents.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Season {
    /// The generation (1-based).
    pub generation: u32,
    pub summary: SeasonSummary,
    /// Every agent born into it, as the season left them: traits, stores
    /// left, death and loss records.
    pub agents: Vec<Agent>,
}

/// How a finished run ended (item 11).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Fate {
    /// Mean L above 0.95, averaged over the window.
    Takeover,
    /// Mean L below 0.2 in every generation of the window.
    StayedLow,
    /// Neither.
    Intermediate,
    /// Every agent died; never a takeover.
    Extinct,
}

/// A finished run's outcome (item 11). The window is the last `WINDOW`
/// completed generations (51–60 at the default 60).
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Outcome {
    pub fate: Fate,
    /// The first and last generation of the window.
    pub window: (u32, u32),
    /// Mean L averaged over the window.
    pub window_mean_l: f64,
    /// The first generation whose mean L is above 0.95.
    pub rise: Option<u32>,
    /// The generation in which the population died out.
    pub extinct: Option<u32>,
}

/// One entry of the food a searcher sees (amendments item 5).
#[derive(Clone, Copy, Debug, PartialEq)]
enum Pool {
    Public,
    Scatter(usize),
    Larder(usize),
}

#[derive(Clone)]
pub struct HoardWorld {
    pub config: HoardConfig,
    /// Bouts run.
    pub tick: u64,
    rng: SimRng,
    pub stats: Stats<HoardSnapshot>,
    agents: Vec<Agent>,
    /// The generation (1-based), the day (1-based) and the next bout
    /// (1-based) to run.
    generation: u32,
    day: u32,
    bout: u32,
    season_over: bool,
    /// Public food.
    public: u64,
    /// Items produced, eaten, and removed with the dead (`dead_stores =
    /// remove`), this season.
    produced: u64,
    eaten: u64,
    removed: u64,
    /// −ln(`search_miss`) ÷ (`bouts` × `search_items`^1.5).
    calibration: f64,
    /// Every finished season, in order.
    seasons: Vec<Season>,
    /// The generation in which every agent died.
    extinct: Option<u32>,
}

/// A newborn agent: no stores, alive, satiated.
fn newborn(l: f64, d: f64, forage: f64) -> Agent {
    Agent {
        l,
        d,
        forage,
        larder: 0,
        scatter: 0,
        alive: true,
        fed: true,
        defending: false,
        raid: None,
        death: None,
        record: Record::default(),
    }
}

/// The logit of a trait clamped to (ε, 1 − ε).
fn clamped_logit(p: f64) -> f64 {
    logit(p.clamp(EPSILON, 1.0 - EPSILON))
}

/// The parent a uniform `u` picks from `weights` (agent, leftover stores),
/// the survivors in index order: in proportion to the stores, or uniformly
/// when they hold none (Review Focus 2).
fn pick(weights: &[(usize, u64)], u: f64) -> usize {
    let total: u64 = weights.iter().map(|&(_, w)| w).sum();
    if total == 0 {
        let k = ((u * weights.len() as f64) as usize).min(weights.len() - 1);
        return weights[k].0;
    }
    let mut x = u * total as f64;
    let mut last = weights[0].0;
    for &(i, w) in weights {
        if w == 0 {
            continue;
        }
        last = i;
        if x < w as f64 {
            break;
        }
        x -= w as f64;
    }
    last
}

/// The logit and its inverse.
fn logit(p: f64) -> f64 {
    ln(p / (1.0 - p))
}

fn inverse_logit(x: f64) -> f64 {
    if x >= 0.0 {
        1.0 / (1.0 + exp_neg(-x))
    } else {
        let e = exp_neg(x);
        e / (1.0 + e)
    }
}

impl HoardWorld {
    pub fn new(config: HoardConfig, seed: u64) -> Result<Self, Vec<FieldError>> {
        config.validate()?;
        let mut rng = rng::seeded(seed);
        let sd = config.v_seg.sqrt();
        let (lc, dc) = (logit(config.l_mean), logit(config.d_mean));
        let agents = (0..config.n)
            .map(|_| {
                let l = inverse_logit(lc + sd * normal(&mut rng));
                let d = inverse_logit(dc + sd * normal(&mut rng));
                let forage = (1.0 + config.forage_sd * normal(&mut rng)).max(0.0);
                newborn(l, d, forage)
            })
            .collect();
        let items = f64::from(config.search_items);
        let calibration =
            -ln(config.search_miss) / (f64::from(config.bouts) * items * items.sqrt());
        let mut w = HoardWorld {
            config,
            tick: 0,
            rng,
            stats: Stats::default(),
            agents,
            generation: 1,
            day: 1,
            bout: 1,
            season_over: false,
            public: 0,
            produced: 0,
            eaten: 0,
            removed: 0,
            calibration,
            seasons: Vec::new(),
            extinct: None,
        };
        w.record();
        Ok(w)
    }

    pub fn agents(&self) -> &[Agent] {
        &self.agents
    }

    pub fn generation(&self) -> u32 {
        self.generation
    }

    /// The day and the next bout to run (both 1-based).
    pub fn day(&self) -> u32 {
        self.day
    }

    pub fn bout(&self) -> u32 {
        self.bout
    }

    pub fn public(&self) -> u64 {
        self.public
    }

    /// Items produced, eaten and removed with the dead this season.
    pub fn produced(&self) -> u64 {
        self.produced
    }

    pub fn eaten(&self) -> u64 {
        self.eaten
    }

    pub fn removed(&self) -> u64 {
        self.removed
    }

    /// Items held in stores, living and dead.
    pub fn stored(&self) -> u64 {
        self.agents
            .iter()
            .map(|a| u64::from(a.larder) + u64::from(a.scatter))
            .sum()
    }

    pub fn living(&self) -> usize {
        self.agents.iter().filter(|a| a.alive).count()
    }

    /// Whether the current season has ended (its record is kept; the next
    /// bout breeds, if the run goes on).
    pub fn season_over(&self) -> bool {
        self.season_over
    }

    /// Whether the run has ended: its population died out, or the season of
    /// generation `generations` (read now) is over.
    pub fn is_finished(&self) -> bool {
        self.season_over && (self.extinct.is_some() || self.generation >= self.config.generations)
    }

    /// Every finished season, in order.
    pub fn seasons(&self) -> &[Season] {
        &self.seasons
    }

    /// The generation in which every agent died, if it has happened.
    pub fn extinct(&self) -> Option<u32> {
        self.extinct
    }

    /// The run's outcome (item 11), once it is finished.
    pub fn outcome(&self) -> Option<Outcome> {
        if !self.is_finished() || self.seasons.is_empty() {
            return None;
        }
        let window = &self.seasons[self.seasons.len().saturating_sub(WINDOW)..];
        let window_mean_l =
            window.iter().map(|s| s.summary.mean_l).sum::<f64>() / window.len() as f64;
        let fate = if self.extinct.is_some() {
            Fate::Extinct
        } else if window_mean_l > TAKEOVER_L {
            Fate::Takeover
        } else if window.iter().all(|s| s.summary.mean_l < LOW_L) {
            Fate::StayedLow
        } else {
            Fate::Intermediate
        };
        Some(Outcome {
            fate,
            window: (window[0].generation, window[window.len() - 1].generation),
            window_mean_l,
            rise: self
                .seasons
                .iter()
                .find(|s| s.summary.mean_l > TAKEOVER_L)
                .map(|s| s.generation),
            extinct: self.extinct,
        })
    }

    /// Public food produced on day `d` (1-based): ⌊first − step·(d − 1) +
    /// 0.5⌋ for d up to `food_days`, and none after.
    pub fn production(config: &HoardConfig, d: u32) -> u64 {
        if d == 0 || d > config.food_days {
            return 0;
        }
        (config.food_first - config.food_step * f64::from(d - 1) + 0.5)
            .floor()
            .max(0.0) as u64
    }

    /// λ for an agent of efficiency `forage` seeing `available` weighted items.
    pub fn search_rate(&self, forage: f64, available: f64) -> f64 {
        forage * self.calibration * available * available.sqrt()
    }

    /// The chance of detecting an item in a bout at rate `k`.
    pub fn detection(k: f64) -> f64 {
        1.0 - exp_neg(-k)
    }

    /// The defense target on day `d`, bout `b` for propensity `dp`
    /// (amendments item 3): max(1, min + D·(max − min)), with min = min(days −
    /// d, days − nonstorable_days) and max the bouts left, this one included.
    pub fn defense_target(&self, dp: f64, d: u32, b: u32) -> f64 {
        let c = &self.config;
        let min = f64::from((c.days - d).min(c.days - c.nonstorable_days));
        let max = f64::from(c.bouts - b + 1) + f64::from(c.bouts) * f64::from(c.days - d);
        (min + dp * (max - min)).max(1.0)
    }

    /// The chance of defending with `larder` items against `target`:
    /// 1 / (1 + e^(−s(x − 0.5))), x = larder / target.
    pub fn defense_chance(&self, larder: u32, target: f64) -> f64 {
        let x = f64::from(larder) / target;
        inverse_logit(self.config.defense_slope * (x - 0.5))
    }

    pub fn run(&mut self, ticks: u32) {
        for _ in 0..ticks {
            if self.is_finished() {
                break;
            }
            self.step();
        }
    }

    /// One bout; after a finished season (and if the run goes on), the next
    /// generation is bred first.
    pub fn step(&mut self) {
        if self.is_finished() {
            return;
        }
        if self.season_over {
            self.breed();
        }
        let (d, b) = (self.day, self.bout);
        let early = d <= self.config.nonstorable_days;
        if b == 1 {
            self.start_day();
        }
        for a in self.agents.iter_mut().filter(|a| a.alive) {
            a.record.bouts_alive += 1;
            a.record.larder_stock += u64::from(a.larder);
            a.record.scatter_stock += u64::from(a.scatter);
        }
        // 1. Bout-1 eating: Some(defends) for the agents it assigns.
        let mut assigned: Vec<Option<bool>> = vec![None; self.agents.len()];
        if b == 1 && (!early || (self.config.early_bout1_eats && d >= 2)) {
            for (i, a) in self.agents.iter_mut().enumerate() {
                if !a.alive || (!early && a.fed) {
                    continue;
                }
                if a.larder > 0 {
                    a.larder -= 1;
                    assigned[i] = Some(true);
                } else if a.scatter > 0 {
                    a.scatter -= 1;
                    assigned[i] = Some(false);
                } else {
                    // The printed flag: its first find today is eaten. After
                    // the nonstorable days it is hungry already; on them, it
                    // can't starve (item 7).
                    a.fed = false;
                    continue;
                }
                a.fed = true;
                a.record.eaten += 1;
                self.eaten += 1;
            }
        }
        // 2. Defense.
        for (i, &fixed) in assigned.iter().enumerate() {
            let a = &self.agents[i];
            if !a.alive {
                continue;
            }
            let defends = if !a.fed {
                false
            } else if let Some(x) = fixed {
                x
            } else {
                let p = self.defense_chance(a.larder, self.defense_target(a.d, d, b));
                self.rng.gen::<f64>() < p
            };
            let a = &mut self.agents[i];
            a.defending = defends;
            if defends {
                a.raid = None;
            }
        }
        // 3. Predation.
        for i in 0..self.agents.len() {
            if self.agents[i].alive && self.rng.gen::<f64>() < self.config.predation {
                self.kill(i, Cause::Predation);
            }
        }
        // 4. Foraging, in a fresh random order.
        let mut order: Vec<usize> = (0..self.agents.len())
            .filter(|&i| self.agents[i].alive && !self.agents[i].defending)
            .collect();
        order.shuffle(&mut self.rng);
        for i in order {
            self.turn(i);
        }
        self.tick += 1;
        if self.bout == self.config.bouts {
            self.end_day();
        } else {
            self.bout += 1;
        }
        if !self.season_over && self.living() == 0 {
            self.end_season();
        }
        self.record();
    }

    fn start_day(&mut self) {
        let add = Self::production(&self.config, self.day);
        self.public += add;
        self.produced += add;
        let fed = self.day <= self.config.nonstorable_days;
        for a in self.agents.iter_mut().filter(|a| a.alive) {
            a.fed = fed;
            a.raid = None;
        }
    }

    fn end_day(&mut self) {
        if self.day > self.config.nonstorable_days {
            for i in 0..self.agents.len() {
                if self.agents[i].alive && !self.agents[i].fed {
                    self.kill(i, Cause::Starvation);
                }
            }
        }
        for a in &mut self.agents {
            a.raid = None;
            a.defending = false;
        }
        self.bout = 1;
        self.day += 1;
        if self.day > self.config.days {
            self.end_season();
        }
    }

    /// Ends the season: its record is kept before anything is reset, and a
    /// season with no survivors marks the run extinct (Review Focus 1).
    fn end_season(&mut self) {
        self.season_over = true;
        for a in &mut self.agents {
            a.raid = None;
            a.defending = false;
        }
        if self.living() == 0 {
            self.extinct = Some(self.generation);
        }
        self.seasons.push(Season {
            generation: self.generation,
            summary: self.summary(),
            agents: self.agents.clone(),
        });
    }

    /// Breeds the next generation from the finished season (items 8 and 9)
    /// and starts its season: empty stores, an empty public pool, day 1.
    fn breed(&mut self) {
        let weights: Vec<(usize, u64)> = self
            .agents
            .iter()
            .enumerate()
            .filter(|(_, a)| a.alive)
            .map(|(i, a)| (i, u64::from(a.larder) + u64::from(a.scatter)))
            .collect();
        assert!(!weights.is_empty(), "an extinct run doesn't breed");
        let n = self.agents.len() as f64;
        let l_bar = self.agents.iter().map(|a| clamped_logit(a.l)).sum::<f64>() / n;
        let d_bar = self.agents.iter().map(|a| clamped_logit(a.d)).sum::<f64>() / n;
        let next = (0..self.agents.len())
            .map(|_| {
                let m = pick(&weights, self.rng.gen::<f64>());
                let f = pick(&weights, self.rng.gen::<f64>());
                self.child(m, f, l_bar, d_bar)
            })
            .collect();
        self.agents = next;
        self.generation += 1;
        self.day = 1;
        self.bout = 1;
        self.season_over = false;
        self.public = 0;
        self.produced = 0;
        self.eaten = 0;
        self.removed = 0;
    }

    /// A child of agents `m` and `f`: on the logit scale, each trait is
    /// N(h²·(midparent) + (1 − h²)·mean, `v_seg`), where the mean is over the
    /// whole parental generation, dead included; forage is drawn fresh.
    fn child(&mut self, m: usize, f: usize, l_bar: f64, d_bar: f64) -> Agent {
        let h2 = self.config.heritability;
        let sd = self.config.v_seg.sqrt();
        let (pm, pf) = (&self.agents[m], &self.agents[f]);
        let l_mid = (clamped_logit(pm.l) + clamped_logit(pf.l)) / 2.0;
        let d_mid = (clamped_logit(pm.d) + clamped_logit(pf.d)) / 2.0;
        let l = inverse_logit(h2 * l_mid + (1.0 - h2) * l_bar + sd * normal(&mut self.rng));
        let d = inverse_logit(h2 * d_mid + (1.0 - h2) * d_bar + sd * normal(&mut self.rng));
        let forage = (1.0 + self.config.forage_sd * normal(&mut self.rng)).max(0.0);
        newborn(l, d, forage)
    }

    /// Pushes this bout's snapshot (item 13).
    fn record(&mut self) {
        let n = self.agents.len() as f64;
        let (mut sl, mut ss) = (0u64, 0u64);
        let (mut ll, mut ls, mut sk, mut kk) = (0u64, 0u64, 0u64, 0u64);
        for a in &self.agents {
            if a.alive {
                sl += u64::from(a.larder);
                ss += u64::from(a.scatter);
            }
            ll += a.record.larder_lost;
            ls += a.record.larder_stock;
            sk += a.record.scatter_lost;
            kk += a.record.scatter_stock;
        }
        let rate = |lost: u64, stock: u64| {
            Record::rate(lost, stock, self.config.bouts).unwrap_or(f64::NAN)
        };
        let snapshot = HoardSnapshot {
            tick: self.tick,
            generation: u64::from(self.generation),
            mean_l: self.agents.iter().map(|a| a.l).sum::<f64>() / n,
            mean_d: self.agents.iter().map(|a| a.d).sum::<f64>() / n,
            survivors: self.living() as u32,
            larder_share: if sl + ss > 0 {
                sl as f64 / (sl + ss) as f64
            } else {
                f64::NAN
            },
            larder_rate: rate(ll, ls),
            scatter_rate: rate(sk, kk),
            takeover: self
                .outcome()
                .map_or(f64::NAN, |o| f64::from(u8::from(o.fate == Fate::Takeover))),
        };
        self.stats.push(snapshot);
    }

    fn kill(&mut self, i: usize, cause: Cause) {
        let (day, bout) = (self.day, self.bout);
        let remove = self.config.dead_stores == DeadStores::Remove;
        let a = &mut self.agents[i];
        a.alive = false;
        a.defending = false;
        a.raid = None;
        a.death = Some(Death { day, bout, cause });
        if remove {
            self.removed += u64::from(a.larder) + u64::from(a.scatter);
            a.larder = 0;
            a.scatter = 0;
        }
    }

    /// Agent `i`'s foraging turn: a raid continuation, or a search.
    fn turn(&mut self, i: usize) {
        if let Some(j) = self.agents[i].raid {
            let owner = &self.agents[j];
            if owner.larder > 0 && !owner.defending {
                self.take(i, Pool::Larder(j));
                return;
            }
            self.agents[i].raid = None;
        }
        self.search(i);
    }

    /// The food agent `i` sees, in the draw's order: public food, then
    /// others' scattered caches, then others' larders, each in index order.
    fn pool(&self, i: usize) -> Vec<(Pool, f64)> {
        let c = &self.config;
        let mut out = vec![(Pool::Public, self.public as f64)];
        for (j, a) in self.agents.iter().enumerate() {
            if j != i && a.scatter > 0 {
                out.push((Pool::Scatter(j), c.app_scat * f64::from(a.scatter)));
            }
        }
        for (j, a) in self.agents.iter().enumerate() {
            if j == i || a.larder == 0 {
                continue;
            }
            if a.defending && c.defended_in_pool == DefendedInPool::Excluded {
                continue;
            }
            let w = match c.larder_weight {
                LarderWeight::PerItem => c.app_lard * f64::from(a.larder),
                LarderWeight::PerBurrow => c.app_lard,
            };
            out.push((Pool::Larder(j), w));
        }
        out
    }

    /// The weighted food available to agent `i`.
    pub fn available(&self, i: usize) -> f64 {
        self.pool(i).iter().map(|&(_, w)| w).sum()
    }

    fn search(&mut self, i: usize) {
        let pool = self.pool(i);
        let available: f64 = pool.iter().map(|&(_, w)| w).sum();
        let k = self.search_rate(self.agents[i].forage, available);
        let p = Self::detection(k);
        if self.rng.gen::<f64>() >= p {
            return;
        }
        let mut x = self.rng.gen::<f64>() * available;
        let mut found = None;
        for &(what, w) in &pool {
            if w <= 0.0 {
                continue;
            }
            found = Some(what);
            if x < w {
                break;
            }
            x -= w;
        }
        let Some(what) = found else {
            return;
        };
        if let Pool::Larder(j) = what {
            if self.agents[j].defending {
                return;
            }
        }
        self.take(i, what);
    }

    /// Agent `i` harvests one item from `what`, then eats or stores it.
    fn take(&mut self, i: usize, what: Pool) {
        match what {
            Pool::Public => {
                self.public -= 1;
                self.agents[i].record.took_public += 1;
            }
            Pool::Scatter(j) => {
                let o = &mut self.agents[j];
                o.scatter -= 1;
                if o.alive {
                    o.record.scatter_lost += 1;
                }
                self.agents[i].record.took_scatter += 1;
            }
            Pool::Larder(j) => {
                let o = &mut self.agents[j];
                o.larder -= 1;
                if o.alive {
                    o.record.larder_lost += 1;
                }
                let a = &mut self.agents[i];
                a.record.took_larder += 1;
                a.raid = Some(j);
            }
        }
        let a = &self.agents[i];
        if !a.fed {
            let a = &mut self.agents[i];
            a.fed = true;
            a.record.eaten += 1;
            self.eaten += 1;
        } else {
            let larder = self.rng.gen::<f64>() < a.l;
            let a = &mut self.agents[i];
            if larder {
                a.larder += 1;
            } else {
                a.scatter += 1;
            }
        }
    }

    /// The season's summary numbers, so far.
    pub fn summary(&self) -> SeasonSummary {
        let bouts = self.config.bouts;
        let survivors = self.agents.iter().filter(|a| a.alive);
        let (sl, ss) = survivors.fold((0u64, 0u64), |(l, s), a| {
            (l + u64::from(a.larder), s + u64::from(a.scatter))
        });
        let dead = |cause| {
            self.agents
                .iter()
                .filter(|a| a.death.map(|d| d.cause) == Some(cause))
                .count() as u32
        };
        let mean = |v: Vec<f64>| (!v.is_empty()).then(|| v.iter().sum::<f64>() / v.len() as f64);
        let larder: Vec<f64> = self
            .agents
            .iter()
            .filter_map(|a| a.record.larder_rate(bouts))
            .collect();
        let scatter: Vec<f64> = self
            .agents
            .iter()
            .filter_map(|a| a.record.scatter_rate(bouts))
            .collect();
        let n = self.agents.len() as f64;
        let pooled = |lost: fn(&Record) -> u64, stock: fn(&Record) -> u64| {
            let (l, s) = self.agents.iter().fold((0, 0), |(l, s), a| {
                (l + lost(&a.record), s + stock(&a.record))
            });
            Record::rate(l, s, bouts)
        };
        let min = |v: &[f64]| v.iter().copied().reduce(f64::min);
        let exposed: Vec<f64> = self
            .agents
            .iter()
            .filter(|a| a.record.mean_larder_stock() >= EXPOSURE_FLOOR)
            .filter_map(|a| a.record.larder_rate(bouts))
            .collect();
        SeasonSummary {
            survivors: self.living() as u32,
            starved: dead(Cause::Starvation),
            preyed: dead(Cause::Predation),
            larder_share: (sl + ss > 0).then(|| sl as f64 / (sl + ss) as f64),
            larder_rated: larder.len() as u32,
            scatter_rated: scatter.len() as u32,
            pooled_larder_rate: pooled(|r| r.larder_lost, |r| r.larder_stock),
            pooled_scatter_rate: pooled(|r| r.scatter_lost, |r| r.scatter_stock),
            larder_exposed: exposed.len() as u32,
            min_exposed_larder_rate: min(&exposed),
            min_larder_rate: min(&larder),
            mean_larder_rate: mean(larder),
            mean_scatter_rate: mean(scatter),
            larder_unrated_losses: self
                .agents
                .iter()
                .filter(|a| a.record.larder_stock == 0 && a.record.larder_lost > 0)
                .count() as u32,
            scatter_unrated_losses: self
                .agents
                .iter()
                .filter(|a| a.record.scatter_stock == 0 && a.record.scatter_lost > 0)
                .count() as u32,
            mean_l: self.agents.iter().map(|a| a.l).sum::<f64>() / n,
            mean_d: self.agents.iter().map(|a| a.d).sum::<f64>() / n,
        }
    }
}

impl Model for HoardWorld {
    fn config(&self) -> ModelConfig {
        ModelConfig::Hoard(self.config.clone())
    }

    fn run(&mut self, ticks: u32) {
        HoardWorld::run(self, ticks);
    }

    fn tick(&self) -> u64 {
        self.tick
    }

    fn population(&self) -> usize {
        self.living()
    }

    /// FNV-1a over the clock, the public food and every agent's state.
    fn fingerprint(&self) -> u64 {
        let mut h: u64 = 0xcbf2_9ce4_8422_2325;
        let mut eat = |bytes: &[u8]| {
            for &b in bytes {
                h ^= u64::from(b);
                h = h.wrapping_mul(0x0100_0000_01b3);
            }
        };
        eat(&self.tick.to_le_bytes());
        eat(&self.generation.to_le_bytes());
        eat(&self.day.to_le_bytes());
        eat(&self.bout.to_le_bytes());
        eat(&self.public.to_le_bytes());
        for a in &self.agents {
            eat(&a.l.to_bits().to_le_bytes());
            eat(&a.d.to_bits().to_le_bytes());
            eat(&a.forage.to_bits().to_le_bytes());
            eat(&a.larder.to_le_bytes());
            eat(&a.scatter.to_le_bytes());
            eat(&[u8::from(a.alive), u8::from(a.fed), u8::from(a.defending)]);
            eat(&a.raid.map_or(u64::MAX, |j| j as u64).to_le_bytes());
        }
        h
    }

    fn size(&self) -> (u32, u32) {
        (WIDE as u32, TALL as u32)
    }

    fn render(&self, mode: &str, _layer: &str, buf: &mut Vec<u8>) -> Result<(), String> {
        if mode != "agents" {
            return Err(format!("unknown color mode {mode:?}"));
        }
        buf.clear();
        for _ in 0..WIDE * TALL {
            buf.extend_from_slice(&BACKGROUND);
        }
        Ok(())
    }

    fn latest_json(&self) -> String {
        serde_json::to_string(&self.stats.latest()).expect("snapshot serializes")
    }

    fn series_names(&self) -> Vec<String> {
        super::SERIES.iter().map(|s| s.to_string()).collect()
    }

    fn series(&self, name: &str) -> Option<Vec<f64>> {
        self.stats.series(name)
    }

    fn series_csv(&self) -> String {
        export::history_csv(&self.series_names(), self.stats.history())
    }

    fn agents_csv(&self) -> String {
        let mut out = String::from("id,larder_probability,defense_propensity,larder,scatter\n");
        for (i, a) in self.agents.iter().enumerate().filter(|(_, a)| a.alive) {
            writeln!(out, "{i},{},{},{},{}", a.l, a.d, a.larder, a.scatter)
                .expect("writing to a string");
        }
        out
    }

    fn inspect_json(&self, _x: u32, _y: u32) -> Result<String, String> {
        Err("there is nothing to inspect yet".into())
    }

    fn locate(&self, _id: u64) -> Option<(u32, u32)> {
        None
    }

    fn set_config(&mut self, next: ModelConfig) -> Result<(), Vec<FieldError>> {
        let ModelConfig::Hoard(next) = next else {
            return Err(wrong_model(ModelKind::Hoard, &next));
        };
        next.validate()?;
        let changes = self.config.structural_changes(&next);
        if !changes.is_empty() {
            return Err(changes);
        }
        self.config = next;
        Ok(())
    }

    fn finished(&self) -> bool {
        self.is_finished()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A world of `n` agents with no predation, set to day `d`, bout `b`,
    /// no public food, every agent alive, satiated, with no stores.
    fn quiet(n: u32, d: u32, b: u32, edit: impl FnOnce(&mut HoardConfig)) -> HoardWorld {
        let mut c = HoardConfig {
            n,
            predation: 0.0,
            ..HoardConfig::default()
        };
        edit(&mut c);
        let mut w = HoardWorld::new(c, 7).unwrap();
        w.day = d;
        w.bout = b;
        for a in &mut w.agents {
            a.fed = true;
        }
        w
    }

    fn conserved(w: &HoardWorld) -> bool {
        w.produced == w.public + w.stored() + w.eaten + w.removed
    }

    #[test]
    fn the_schedule_sums_to_2100() {
        let c = HoardConfig::default();
        let day = |d| HoardWorld::production(&c, d);
        assert_eq!(
            (day(1), day(2), day(3), day(50), day(51)),
            (82, 81, 79, 2, 0)
        );
        assert!((1..50).all(|d| day(d) >= day(d + 1)));
        assert_eq!((1..=100).map(day).sum::<u64>(), 2100);
        let mut w = HoardWorld::new(c, 1).unwrap();
        w.run(2000);
        assert_eq!(w.produced(), 2100);
    }

    #[test]
    fn the_search_is_calibrated_as_printed() {
        let w = HoardWorld::new(HoardConfig::default(), 1).unwrap();
        let k = w.search_rate(1.0, 82.0);
        let p = HoardWorld::detection(k);
        assert!((k - 0.2303).abs() < 5e-5, "k = {k}");
        assert!((p - 0.2057).abs() < 5e-5, "P = {p}");
        assert!(((1.0 - p).powi(20) - 0.01).abs() < 1e-9);
    }

    /// Page 665's calibration: an agent of efficiency 1 searching 20 bouts
    /// with 82 items available (and no stores to pilfer) finds nothing with
    /// chance 0.01. Driven through the world's own search, the pool reset to
    /// 82 before each bout, over 10 000 seeds. The binomial SD is
    /// √(0.01 × 0.99 / 10 000) ≈ 0.001, so the tolerance ±0.003 is 3 SD.
    #[test]
    fn a_day_1_total_failure_is_near_1_percent() {
        let seeds = 10_000;
        let mut failures = 0;
        for seed in 0..seeds {
            let mut w = HoardWorld::new(HoardConfig::default(), seed).unwrap();
            w.agents[0].forage = 1.0;
            let found = (0..20).any(|_| {
                w.public = 82;
                w.search(0);
                w.agents[0].record.took_public > 0
            });
            if !found {
                failures += 1;
            }
        }
        let f = f64::from(failures) / seeds as f64;
        assert!((f - 0.01).abs() < 0.003, "day-1 failure {f}");
    }

    #[test]
    fn founders_are_logit_normal_around_the_printed_means() {
        let c = HoardConfig {
            n: 1000,
            ..HoardConfig::default()
        };
        let w = HoardWorld::new(c, 3).unwrap();
        let n = w.agents.len() as f64;
        let mean = |f: fn(&Agent) -> f64| w.agents.iter().map(f).sum::<f64>() / n;
        let (l, d, f) = (mean(|a| a.l), mean(|a| a.d), mean(|a| a.forage));
        assert!((0.15..0.19).contains(&l), "mean L {l}");
        assert!((0.47..0.53).contains(&d), "mean D {d}");
        assert!((0.99..1.01).contains(&f), "mean forage {f}");
        let sd = (w.agents.iter().map(|a| (a.forage - f).powi(2)).sum::<f64>() / n).sqrt();
        assert!((0.09..0.11).contains(&sd), "forage sd {sd}");
        assert!(w.agents.iter().all(|a| a.l > 0.0
            && a.l < 1.0
            && a.d > 0.0
            && a.d < 1.0
            && a.forage >= 0.0));
    }

    #[test]
    fn the_defense_logistic_and_target_are_the_amendments() {
        let w = HoardWorld::new(HoardConfig::default(), 1).unwrap();
        // Day 6, bout 1, D = 0.5: min 94, max 20 + 20 × 94 = 1900.
        assert_eq!(w.defense_target(0.5, 6, 1), 94.0 + 0.5 * (1900.0 - 94.0));
        // Days 1–5: min 95.
        assert_eq!(w.defense_target(0.0, 3, 7), 95.0);
        assert_eq!(w.defense_target(1.0, 3, 7), 14.0 + 20.0 * 97.0);
        // The last bout, D = 0: the floor of 1.
        assert_eq!(w.defense_target(0.0, 100, 20), 1.0);
        let t = 100.0;
        assert!((w.defense_chance(0, t) - 0.006_692_85).abs() < 1e-6);
        assert!((w.defense_chance(50, t) - 0.5).abs() < 1e-12);
        assert!((w.defense_chance(100, t) - 0.993_307_15).abs() < 1e-6);
        let s4 = quiet(2, 1, 1, |c| c.defense_slope = 4.0);
        assert!((s4.defense_chance(0, t) - 0.119_202_9).abs() < 1e-6);
    }

    #[test]
    fn items_are_conserved_every_bout() {
        for (seed, edit) in [
            (1, (|_: &mut HoardConfig| {}) as fn(&mut HoardConfig)),
            (2, |c| c.dead_stores = DeadStores::Remove),
            (3, |c| {
                c.dead_stores = DeadStores::Remove;
                c.predation = 0.002;
            }),
            (4, |c| c.early_bout1_eats = true),
            (5, |c| c.larder_weight = LarderWeight::PerBurrow),
            (6, |c| {
                c.defended_in_pool = DefendedInPool::Excluded;
                c.l_mean = 0.8;
            }),
        ] {
            let mut c = HoardConfig::default();
            edit(&mut c);
            let mut w = HoardWorld::new(c, seed).unwrap();
            while !w.season_over() {
                w.step();
                assert!(conserved(&w), "seed {seed}, tick {}", w.tick);
            }
            // A season ends early only if every agent has died.
            assert!(w.tick == 2000 || w.extinct() == Some(1), "seed {seed}");
        }
    }

    #[test]
    fn the_season_stops_at_its_end() {
        let one = HoardConfig {
            generations: 1,
            ..HoardConfig::default()
        };
        let mut w = HoardWorld::new(one, 9).unwrap();
        w.run(5000);
        assert!(w.season_over() && w.is_finished());
        assert_eq!((w.tick, w.day, w.bout), (2000, 101, 1));
        let f = Model::fingerprint(&w);
        w.run(10);
        assert_eq!(Model::fingerprint(&w), f);
        // With generations to go, the season still stops, and the next bout
        // breeds.
        let mut w = HoardWorld::new(HoardConfig::default(), 9).unwrap();
        w.run(2000);
        assert!(w.season_over() && !w.is_finished());
        assert_eq!((w.generation(), w.seasons().len()), (1, 1));
        w.step();
        assert_eq!((w.generation(), w.day(), w.bout(), w.tick), (2, 1, 2, 2001));
    }

    #[test]
    fn a_seed_gives_the_same_season() {
        let run = |seed| {
            let mut w = HoardWorld::new(HoardConfig::default(), seed).unwrap();
            w.run(2000);
            (Model::fingerprint(&w), w.agents.clone(), w.summary())
        };
        assert_eq!(run(11), run(11));
        assert_ne!(run(11).0, run(12).0);
    }

    /// Review Focus 4: a raid that empties a larder takes one item a bout,
    /// then ends; items are conserved throughout.
    #[test]
    fn a_raid_empties_a_larder_one_item_a_bout_then_ends() {
        // Agent 1 never finds anything (forage 0) and never defends (a steep
        // slope far below half its target); agent 0 raids it and scatters.
        let mut w = quiet(2, 10, 3, |c| c.defense_slope = 1000.0);
        w.agents[1].forage = 0.0;
        w.agents[1].larder = 3;
        w.agents[0].l = 0.0;
        w.agents[0].raid = Some(1);
        w.produced = 3;
        for left in [2, 1, 0] {
            w.step();
            assert_eq!(w.agents[1].larder, left);
            assert_eq!(w.agents[0].raid, Some(1));
            assert!(conserved(&w));
        }
        assert_eq!(w.agents[0].scatter, 3);
        assert_eq!(w.agents[0].record.took_larder, 3);
        assert_eq!(w.agents[1].record.larder_lost, 3);
        // Empty: the continuation fails, the raid ends and agent 0 searches
        // (there is nothing to find).
        w.step();
        assert_eq!(w.agents[0].raid, None);
        assert_eq!(w.agents[0].scatter, 3);
        assert!(conserved(&w));
    }

    /// Review Focus 4: the owner returns mid-raid. The raid ends that bout
    /// and doesn't resume when the owner leaves again.
    #[test]
    fn an_owner_returning_ends_the_raid() {
        // Day 100: D = 1 puts the target at the bouts left (far above the
        // larder), D = 0 puts it at 1 (far below), with a steep slope.
        let mut w = quiet(2, 100, 3, |c| c.defense_slope = 1000.0);
        w.agents[1].forage = 0.0;
        w.agents[1].d = 1.0;
        w.agents[1].larder = 5;
        w.agents[0].raid = Some(1);
        w.agents[0].forage = 0.0;
        w.produced = 5;
        w.step();
        assert_eq!(w.agents[1].larder, 4);
        assert_eq!(w.agents[0].raid, Some(1));
        // The owner returns.
        w.agents[1].d = 0.0;
        w.step();
        assert!(w.agents[1].defending);
        assert_eq!(w.agents[1].larder, 4);
        assert_eq!(w.agents[0].raid, None);
        assert!(conserved(&w));
        // It leaves again; the raider (forage 0) finds nothing by search.
        w.agents[1].d = 1.0;
        w.step();
        assert!(!w.agents[1].defending);
        assert_eq!(w.agents[1].larder, 4);
        assert_eq!(w.agents[0].raid, None);
        assert!(conserved(&w));
    }

    #[test]
    fn raids_end_with_the_day_and_with_the_raider() {
        let mut w = quiet(3, 3, 20, |c| c.defense_slope = 1000.0);
        w.agents[1].forage = 0.0;
        w.agents[1].larder = 5;
        w.agents[0].raid = Some(1);
        w.agents[0].forage = 0.0;
        w.agents[2].forage = 0.0;
        w.produced = 5;
        w.step();
        assert_eq!(w.agents[1].larder, 4);
        assert_eq!((w.day, w.bout), (4, 1));
        assert_eq!(w.agents[0].raid, None);
        // A raider that dies ends its raid.
        w.agents[2].raid = Some(1);
        w.kill(2, Cause::Predation);
        assert_eq!(w.agents[2].raid, None);
        // Several raiders at once each take one item.
        let mut w = quiet(3, 10, 3, |c| c.defense_slope = 1000.0);
        w.agents[2].forage = 0.0;
        w.agents[2].larder = 3;
        w.agents[0].raid = Some(2);
        w.agents[1].raid = Some(2);
        w.produced = 3;
        w.step();
        assert_eq!(w.agents[2].larder, 1);
        assert!(conserved(&w));
    }

    #[test]
    fn bout_1_eats_from_larders_first_and_defends() {
        let mut w = HoardWorld::new(HoardConfig::default(), 5).unwrap();
        w.run(100); // days 1–5
        assert_eq!(w.eaten(), 0, "nobody eats from stores on days 1–5");
        let before = w.agents.clone();
        w.step(); // day 6, bout 1
        for (a, b) in w.agents.iter().zip(&before).filter(|(_, b)| b.alive) {
            if !a.alive {
                continue;
            }
            if b.larder > 0 {
                assert!(a.fed && a.defending);
                // Its larder is guarded; its scattered caches aren't.
                assert_eq!(a.larder, b.larder - 1);
                assert!(a.scatter <= b.scatter);
            } else if b.scatter > 0 {
                assert!(a.fed && !a.defending);
                assert_eq!(a.record.eaten, 1);
            } else {
                let found = a.record.took_public + a.record.took_scatter + a.record.took_larder
                    > b.record.took_public + b.record.took_scatter + b.record.took_larder;
                assert_eq!(a.fed, found);
                assert!(!a.defending);
                assert_eq!((a.larder, a.scatter), (0, 0), "its first find is eaten");
            }
        }
    }

    #[test]
    fn the_hungry_starve_from_day_6_only() {
        let mut w = quiet(2, 3, 1, |_| {});
        w.agents[1].forage = 0.0;
        w.run(20);
        assert!(w.agents[1].alive, "no starvation on the nonstorable days");
        let mut w = quiet(2, 6, 1, |_| {});
        w.agents[1].forage = 0.0;
        w.run(20);
        assert!(!w.agents[1].alive);
        assert_eq!(
            w.agents[1].death,
            Some(Death {
                day: 6,
                bout: 20,
                cause: Cause::Starvation
            })
        );
        assert_eq!(w.agents[1].record.bouts_alive, 20);
    }

    #[test]
    fn a_hungry_agent_eats_its_first_find_and_stores_the_rest() {
        let mut w = quiet(2, 30, 2, |c| c.l_mean = 0.5);
        w.agents[0].fed = false;
        w.agents[0].forage = 1000.0;
        w.agents[1].forage = 0.0;
        w.public = 10;
        w.produced = 10;
        w.run(4);
        let a = &w.agents[0];
        assert_eq!(a.record.took_public, 4);
        assert_eq!(a.record.eaten, 1);
        assert_eq!(a.larder + a.scatter, 3);
        assert!(conserved(&w));
    }

    #[test]
    fn loss_rates_are_losses_per_mean_stock_day() {
        let r = Record {
            larder_lost: 6,
            larder_stock: 40, // 40 bout-items = 2 item-days
            scatter_lost: 0,
            scatter_stock: 0,
            bouts_alive: 40,
            ..Record::default()
        };
        assert_eq!(r.larder_rate(20), Some(3.0));
        assert_eq!(r.scatter_rate(20), None);
        assert_eq!((r.mean_larder_stock(), r.days_alive(20)), (1.0, 2.0));
    }

    #[test]
    fn early_bout1_eats_is_the_printed_bout_1_rule_on_days_2_to_5() {
        let mut w = HoardWorld::new(
            HoardConfig {
                early_bout1_eats: true,
                ..HoardConfig::default()
            },
            5,
        )
        .unwrap();
        w.run(20);
        let before = w.agents.clone();
        let holders = before
            .iter()
            .filter(|a| a.alive && a.larder + a.scatter > 0)
            .count();
        assert!(holders > 0);
        w.step();
        let mut finders = 0;
        for (a, b) in w.agents.iter().zip(&before).filter(|(a, _)| a.alive) {
            if b.larder > 0 {
                assert!(a.fed && a.defending);
                assert_eq!(a.larder, b.larder - 1);
            } else if b.scatter > 0 {
                assert!(a.fed && !a.defending);
            } else {
                // Flagged hungry: fed only if it found something, which it ate.
                let found = a.record.took_public + a.record.took_scatter + a.record.took_larder
                    > b.record.took_public + b.record.took_scatter + b.record.took_larder;
                assert_eq!(a.fed, found);
                assert_eq!((a.larder, a.scatter), (0, 0));
                finders += usize::from(found);
            }
        }
        assert_eq!(w.eaten() as usize, holders + finders);
        w.run(79);
        assert!(w.eaten() as usize > holders);
        let mut d = HoardWorld::new(HoardConfig::default(), 5).unwrap();
        d.run(100);
        assert_eq!(d.eaten(), 0);
    }

    /// Under `early_bout1_eats`, a storeless agent on days 2–5 is flagged
    /// hungry (it doesn't defend, and eats its first find), but nobody
    /// starves on those days. Day 1 flags nobody.
    #[test]
    fn early_bout1_eats_flags_the_storeless_hungry_without_starving_them() {
        let mut w = quiet(2, 1, 1, |c| c.early_bout1_eats = true);
        w.agents[1].forage = 0.0;
        w.step();
        assert!(w.agents[1].fed, "day 1 flags nobody");
        w.run(19);
        for _ in 2..=5 {
            w.step();
            assert!(!w.agents[1].fed && !w.agents[1].defending);
            w.run(19);
            assert!(w.agents[1].alive, "no starvation on day {}", w.day - 1);
        }
        assert_eq!(w.day, 6);
        // A storeless agent that finds food on day 2 eats it.
        let mut w = quiet(2, 2, 1, |c| c.early_bout1_eats = true);
        w.agents[0].forage = 1e6;
        w.public = 3;
        w.produced = 3;
        w.step();
        let a = &w.agents[0];
        assert!(a.fed);
        assert_eq!((a.record.eaten, a.larder + a.scatter), (1, 0));
        assert!(conserved(&w));
    }

    #[test]
    fn larder_weight_per_burrow_counts_each_larder_once() {
        for (weight, expect) in [
            (LarderWeight::PerItem, 20.0),
            (LarderWeight::PerBurrow, 2.0),
        ] {
            let mut w = quiet(3, 10, 3, |c| c.larder_weight = weight);
            w.agents[1].larder = 10;
            w.agents[2].scatter = 5;
            w.agents[0].larder = 50; // its own: never counted
            w.public = 7;
            assert_eq!(w.available(0), 7.0 + 0.44 * 5.0 + expect);
        }
    }

    #[test]
    fn defended_in_pool_counts_or_excludes_a_defended_larder() {
        for (mode, expect) in [
            (DefendedInPool::Counted, 20.0),
            (DefendedInPool::Excluded, 0.0),
        ] {
            let mut w = quiet(2, 10, 3, |c| c.defended_in_pool = mode);
            w.agents[1].larder = 10;
            w.agents[1].defending = true;
            assert_eq!(w.available(0), expect);
        }
        // Drawn while defended, it yields nothing (day 100, D = 0: the owner
        // defends with certainty; the searcher can't miss).
        let mut w = quiet(2, 100, 5, |_| {});
        w.agents[1].d = 0.0;
        w.agents[1].forage = 0.0;
        w.agents[1].larder = 5;
        w.agents[0].forage = 1e6;
        w.produced = 5;
        w.step();
        assert!(w.agents[1].defending);
        assert_eq!(w.agents[1].larder, 5);
        assert_eq!(w.agents[0].record.took_larder, 0);
        assert_eq!(w.agents[0].raid, None);
    }

    #[test]
    fn dead_stores_remain_pilferable_or_are_removed() {
        for (mode, left) in [(DeadStores::Remain, 4), (DeadStores::Remove, 0)] {
            // No scattered caches: the searcher's certain find is the larder.
            let mut w = quiet(2, 30, 3, |c| c.dead_stores = mode);
            w.agents[1].larder = 5;
            w.agents[1].scatter = 0;
            w.produced = 5;
            w.kill(1, Cause::Predation);
            w.agents[0].forage = 1e6;
            w.step();
            assert_eq!(w.agents[1].larder, left);
            assert!(conserved(&w));
            if mode == DeadStores::Remove {
                assert_eq!(w.removed(), 5);
            } else {
                assert_eq!(w.agents[0].raid, Some(1), "the dead never defend");
                assert_eq!(w.agents[1].record.larder_lost, 0, "no loss after death");
            }
        }
    }

    #[test]
    fn defense_slope_sets_how_often_an_empty_larder_is_guarded() {
        let share = |slope| {
            let mut w = quiet(20, 10, 2, |c| c.defense_slope = slope);
            let mut defended = 0;
            for _ in 0..200 {
                w.bout = 2;
                w.step();
                defended += w.agents.iter().filter(|a| a.defending).count();
            }
            defended as f64 / 4000.0
        };
        let (s10, s4) = (share(10.0), share(4.0));
        assert!(s10 < 0.02, "slope 10: {s10}");
        assert!(s4 > 0.08, "slope 4: {s4}");
    }

    /// One season at the defaults, and the mean over 20 seeds (printed with
    /// `--nocapture` for the task report).
    #[test]
    fn a_default_season_runs_end_to_end() {
        let mut totals = [0.0; 5];
        let mut day1_fail = (0u32, 0u32);
        for seed in 1..=20 {
            let mut w = HoardWorld::new(HoardConfig::default(), seed).unwrap();
            w.run(20);
            day1_fail.0 += w
                .agents
                .iter()
                .filter(|a| a.record.took_public == 0)
                .count() as u32;
            day1_fail.1 += w.agents.len() as u32;
            w.run(1980);
            assert!(w.season_over());
            let s = w.summary();
            assert!(
                s.mean_larder_rate.unwrap() > s.mean_scatter_rate.unwrap(),
                "seed {seed}: {s:?}"
            );
            assert!(s.survivors <= 20 && s.survivors + s.starved + s.preyed == 20);
            if seed == 1 {
                println!("seed 1: {s:?}");
            }
            totals[0] += f64::from(s.survivors);
            totals[1] += s.larder_share.unwrap_or(f64::NAN);
            totals[2] += s.mean_larder_rate.unwrap_or(f64::NAN);
            totals[3] += s.mean_scatter_rate.unwrap_or(f64::NAN);
            totals[4] += f64::from(s.starved);
        }
        let m: Vec<f64> = totals.iter().map(|t| t / 20.0).collect();
        println!(
            "20 seeds: survivors {:.2}, larder share {:.3}, larder rate {:.3}, scatter rate {:.3}, starved {:.2}; day-1 failures {}/{}",
            m[0], m[1], m[2], m[3], m[4], day1_fail.0, day1_fail.1
        );
    }

    /// Task 3 minor: the search draws each entry in proportion to its
    /// weight. Public 10, one scatter of 10 (weight 4.4) and one larder of 5
    /// (weight 10 per item, 2 per burrow); shares within 3 SD over 20 000
    /// certain finds. Under `defended_in_pool = excluded` a defended larder
    /// is never drawn.
    #[test]
    fn the_search_draws_in_proportion_to_the_weights() {
        let trials = 20_000;
        let shares = |edit: fn(&mut HoardConfig), defended: bool| {
            let mut w = quiet(3, 30, 3, edit);
            w.agents[0].forage = 1e6;
            let mut counts = [0u32; 3];
            for _ in 0..trials {
                w.public = 10;
                w.agents[1].scatter = 10;
                w.agents[2].larder = 5;
                w.agents[2].defending = defended;
                let r = w.agents[0].record.clone();
                w.search(0);
                let a = &mut w.agents[0];
                a.raid = None;
                (a.larder, a.scatter) = (0, 0);
                counts[0] += (a.record.took_public - r.took_public) as u32;
                counts[1] += (a.record.took_scatter - r.took_scatter) as u32;
                counts[2] += (a.record.took_larder - r.took_larder) as u32;
            }
            counts
        };
        let check = |counts: [u32; 3], weights: [f64; 3]| {
            let total: f64 = weights.iter().sum();
            for (c, w) in counts.iter().zip(weights) {
                let p = w / total;
                let sd = (p * (1.0 - p) / f64::from(trials)).sqrt();
                let got = f64::from(*c) / f64::from(trials);
                assert!((got - p).abs() <= 3.0 * sd, "{counts:?} {weights:?}");
            }
        };
        check(shares(|_| {}, false), [10.0, 4.4, 10.0]);
        check(
            shares(|c| c.larder_weight = LarderWeight::PerBurrow, false),
            [10.0, 4.4, 2.0],
        );
        let excluded = shares(|c| c.defended_in_pool = DefendedInPool::Excluded, true);
        assert_eq!(excluded[2], 0);
        check(excluded, [10.0, 4.4, 0.0]);
    }

    /// Sets agent `i`'s traits on the logit scale.
    fn set_logits(w: &mut HoardWorld, i: usize, l: f64, d: f64) {
        w.agents[i].l = inverse_logit(l);
        w.agents[i].d = inverse_logit(d);
    }

    /// Ends the world's season by hand, as `end_season` would.
    fn end(w: &mut HoardWorld) {
        w.end_season();
    }

    /// With V_seg = 0, a child's logit trait is exactly h²·midparent +
    /// (1 − h²)·mean, the mean over the whole parental generation (dead
    /// included).
    #[test]
    fn inheritance_regresses_to_the_generation_mean_on_the_logit_scale() {
        let mut w = quiet(4, 1, 1, |c| c.v_seg = 0.0);
        for (i, (l, d)) in [(-2.0, 0.5), (1.0, -1.0), (3.0, 2.0), (-4.0, 0.0)]
            .into_iter()
            .enumerate()
        {
            set_logits(&mut w, i, l, d);
        }
        let (l_bar, d_bar) = (-0.5, 0.375);
        let h2 = 0.8;
        let c = w.child(0, 1, l_bar, d_bar);
        assert!((logit(c.l) - (h2 * -0.5 + (1.0 - h2) * l_bar)).abs() < 1e-9);
        assert!((logit(c.d) - (h2 * -0.25 + (1.0 - h2) * d_bar)).abs() < 1e-9);
        // Breeding: agent 2 is the only survivor with stores, so it is both
        // parents of every child; agent 3 is dead but in the mean.
        w.agents[1].alive = false;
        w.agents[3].alive = false;
        w.agents[2].scatter = 4;
        end(&mut w);
        w.breed();
        assert_eq!(w.generation(), 2);
        for c in &w.agents {
            assert!((logit(c.l) - (h2 * 3.0 + (1.0 - h2) * l_bar)).abs() < 1e-9);
            assert!((logit(c.d) - (h2 * 2.0 + (1.0 - h2) * d_bar)).abs() < 1e-9);
            assert!(c.alive && c.fed && c.larder + c.scatter == 0);
            assert_eq!(c.record, Record::default());
        }
        assert_eq!((w.day, w.bout, w.public, w.produced), (1, 1, 0, 0));
    }

    /// Parents are drawn independently, with replacement, in proportion to
    /// the survivors' leftover stores; the dead are never drawn, whatever
    /// they hold. With h² = 1 and V_seg = 0 a child's logit L is its
    /// midparent, and logit Ls 0, 1 and 4 make every pair's sum distinct.
    #[test]
    fn selection_is_proportional_to_leftover_stores_in_expectation() {
        let mut w = quiet(4, 1, 1, |c| {
            c.v_seg = 0.0;
            c.heritability = 1.0;
        });
        for (i, (l, larder, scatter, alive)) in [
            (0.0, 1, 0, true),
            (1.0, 1, 2, true),
            (9.0, 10, 10, false),
            (4.0, 2, 4, true),
        ]
        .into_iter()
        .enumerate()
        {
            set_logits(&mut w, i, l, 0.0);
            let a = &mut w.agents[i];
            (a.larder, a.scatter, a.alive) = (larder, scatter, alive);
        }
        let parents = w.agents.clone();
        let pair = |sum: i64| match sum {
            0 => [0, 0],
            1 => [0, 1],
            4 => [0, 3],
            2 => [1, 1],
            5 => [1, 3],
            8 => [3, 3],
            _ => panic!("no pair sums to {sum}"),
        };
        let mut slots = [0u32; 4];
        let breeds = 2_000;
        for _ in 0..breeds {
            w.agents = parents.clone();
            end(&mut w);
            w.breed();
            for c in &w.agents {
                for p in pair((2.0 * logit(c.l)).round() as i64) {
                    slots[p] += 1;
                }
            }
        }
        let n = f64::from(breeds * 4 * 2);
        for (i, expect) in [0.1, 0.3, 0.0, 0.6].into_iter().enumerate() {
            let got = f64::from(slots[i]) / n;
            let sd = (expect * (1.0 - expect) / n).sqrt();
            assert!(
                (got - expect).abs() <= 3.0 * sd,
                "agent {i}: {got} {slots:?}"
            );
        }
        assert_eq!(slots[2], 0);
    }

    /// Review Focus 2: when every survivor holds nothing, parents are drawn
    /// uniformly among the survivors (never the dead, whatever they hold).
    #[test]
    fn survivors_with_nothing_left_are_drawn_uniformly() {
        let weights = [(0, 0), (2, 0), (5, 0)];
        assert_eq!(pick(&weights, 0.0), 0);
        assert_eq!(pick(&weights, 0.5), 2);
        assert_eq!(pick(&weights, 0.999_999), 5);
        assert_eq!(pick(&[(0, 0), (1, 3), (2, 0)], 0.999_999), 1);
        let mut w = quiet(3, 1, 1, |c| {
            c.v_seg = 0.0;
            c.heritability = 1.0;
        });
        set_logits(&mut w, 0, 1.0, 0.0);
        set_logits(&mut w, 1, 3.0, 0.0);
        set_logits(&mut w, 2, 9.0, 0.0);
        w.agents[2].alive = false;
        w.agents[2].larder = 50;
        let parents = w.agents.clone();
        let mut counts = [0u32; 3]; // midparent logit 1, 2, 3
        let breeds = 4_000;
        for _ in 0..breeds {
            w.agents = parents.clone();
            end(&mut w);
            w.breed();
            for c in &w.agents {
                counts[(logit(c.l).round() as usize) - 1] += 1;
            }
        }
        let n = f64::from(breeds * 3);
        for (c, p) in counts.iter().zip([0.25, 0.5, 0.25]) {
            let sd = (p * (1.0 - p) / n).sqrt();
            assert!((f64::from(*c) / n - p).abs() < 3.0 * sd, "{counts:?}");
        }
    }

    /// Review Focus 3: traits at exactly 0 or 1 are clamped to (ε, 1 − ε)
    /// before the logit, so breeding stays finite.
    #[test]
    fn traits_at_0_or_1_breed_finite_children() {
        assert_eq!(clamped_logit(0.0), logit(EPSILON));
        assert_eq!(clamped_logit(1.0), logit(1.0 - EPSILON));
        assert!(clamped_logit(0.0).is_finite() && clamped_logit(1.0).is_finite());
        let mut w = quiet(2, 1, 1, |c| c.v_seg = 0.0);
        (w.agents[0].l, w.agents[0].d) = (0.0, 1.0);
        (w.agents[1].l, w.agents[1].d) = (1.0, 0.0);
        w.agents[0].scatter = 1;
        w.agents[1].scatter = 1;
        end(&mut w);
        w.breed();
        for c in &w.agents {
            // Midparent 0 and generation mean 0, whatever the pairing, for
            // symmetric pairs; asymmetric pairs land at ±0.8·logit(1 − ε).
            assert!(c.l.is_finite() && c.d.is_finite());
            assert!(c.l > 0.0 && c.l < 1.0 && c.d > 0.0 && c.d < 1.0, "{c:?}");
        }
        // With the default V_seg, many generations from traits at 0 and 1.
        let mut w = quiet(20, 1, 1, |_| {});
        for (i, a) in w.agents.iter_mut().enumerate() {
            (a.l, a.d) = if i % 2 == 0 { (0.0, 1.0) } else { (1.0, 0.0) };
            a.larder = 1;
        }
        for _ in 0..50 {
            for a in &mut w.agents {
                a.larder = 1;
            }
            end(&mut w);
            w.breed();
            assert!(w.agents.iter().all(|a| (0.0..=1.0).contains(&a.l)
                && (0.0..=1.0).contains(&a.d)
                && a.l.is_finite()
                && a.d.is_finite()));
        }
    }

    /// Review Focus 1: when every agent dies, the season ends at the end of
    /// that bout, its record is kept, and the run ends as extinct: nothing is
    /// bred and further steps do nothing.
    #[test]
    fn a_generation_where_every_agent_dies_ends_the_run() {
        let c = HoardConfig {
            predation: 1.0,
            ..HoardConfig::default()
        };
        let mut w = HoardWorld::new(c, 1).unwrap();
        w.step();
        assert_eq!(w.living(), 0);
        assert!(w.season_over() && w.is_finished());
        assert_eq!((w.extinct(), w.seasons().len(), w.tick), (Some(1), 1, 1));
        let f = Model::fingerprint(&w);
        w.run(100);
        w.step();
        assert_eq!((Model::fingerprint(&w), w.tick), (f, 1));
        let o = w.outcome().unwrap();
        assert_eq!(
            (o.fate, o.extinct, o.window),
            (Fate::Extinct, Some(1), (1, 1))
        );
        assert_eq!(w.stats.latest().unwrap().takeover, 0.0);
        // Dying out in a later generation: the earlier seasons stay.
        let mut w = HoardWorld::new(HoardConfig::default(), 2).unwrap();
        w.run(2000);
        let mut next = w.config.clone();
        next.predation = 1.0;
        Model::set_config(&mut w, ModelConfig::Hoard(next)).unwrap();
        w.run(5000);
        assert_eq!((w.extinct(), w.seasons().len(), w.tick), (Some(2), 2, 2001));
        assert_eq!(w.outcome().unwrap().fate, Fate::Extinct);
    }

    /// Item 11's classification over the last 10 generations.
    #[test]
    fn the_outcome_classifies_the_last_10_generations() {
        let fate = |means: &[f64]| {
            let mut w = quiet(2, 1, 1, |c| c.generations = means.len() as u32);
            let summary = w.summary();
            w.seasons = means
                .iter()
                .enumerate()
                .map(|(g, &m)| Season {
                    generation: g as u32 + 1,
                    summary: SeasonSummary {
                        mean_l: m,
                        ..summary.clone()
                    },
                    agents: Vec::new(),
                })
                .collect();
            w.generation = means.len() as u32;
            w.season_over = true;
            w.outcome().unwrap()
        };
        let mut up = vec![0.15; 50];
        up.extend([0.97; 10]);
        up[20] = 0.96;
        let o = fate(&up);
        assert_eq!(
            (o.fate, o.window, o.rise),
            (Fate::Takeover, (51, 60), Some(21))
        );
        let mut low = vec![0.15; 60];
        low[55] = 0.19;
        assert_eq!(fate(&low).fate, Fate::StayedLow);
        low[55] = 0.21;
        assert_eq!(fate(&low).fate, Fate::Intermediate);
        let mut mid = vec![0.99; 60];
        mid[59] = 0.5; // window mean 0.941
        let o = fate(&mid);
        assert_eq!((o.fate, o.rise), (Fate::Intermediate, Some(1)));
        // Fewer than 10 generations: the window is all of them.
        assert_eq!(fate(&[0.1, 0.1, 0.1]).window, (1, 3));
        // Not finished: no outcome.
        let w = HoardWorld::new(HoardConfig::default(), 1).unwrap();
        assert_eq!(w.outcome(), None);
    }

    /// Lowering `generations` live below the current generation ends the run
    /// at the end of the current season; so does lowering it to the current
    /// one.
    #[test]
    fn lowering_generations_live_ends_the_run_at_the_seasons_end() {
        for to in [1, 3] {
            let mut w = HoardWorld::new(HoardConfig::default(), 4).unwrap();
            w.run(4500); // generation 3, mid-season
            assert_eq!(w.generation(), 3);
            let mut next = w.config.clone();
            next.generations = to;
            Model::set_config(&mut w, ModelConfig::Hoard(next)).unwrap();
            assert!(!w.is_finished());
            w.run(10_000);
            assert!(w.is_finished());
            assert_eq!((w.tick, w.seasons().len(), w.generation()), (6000, 3, 3));
            let o = w.outcome().unwrap();
            assert_eq!(o.window, (1, 3));
        }
    }

    /// Items are conserved within every season, across generations.
    #[test]
    fn items_are_conserved_within_each_season_across_generations() {
        let c = HoardConfig {
            generations: 4,
            ..HoardConfig::default()
        };
        let mut w = HoardWorld::new(c, 8).unwrap();
        while !w.is_finished() {
            w.step();
            assert!(conserved(&w), "tick {}", w.tick);
            if w.season_over() {
                assert_eq!(w.produced(), 2100);
            }
        }
        assert_eq!((w.tick, w.seasons().len()), (8000, 4));
    }

    /// Each finished season keeps its agents and measurements before
    /// breeding resets them; the history holds one snapshot per bout.
    #[test]
    fn seasons_keep_their_records_and_the_history_one_snapshot_a_bout() {
        let c = HoardConfig {
            generations: 3,
            ..HoardConfig::default()
        };
        let mut w = HoardWorld::new(c, 6).unwrap();
        assert_eq!(w.stats.history().len(), 1);
        w.run(2000);
        let kept = w.agents.clone();
        let summary = w.summary();
        w.run(10_000);
        assert_eq!(w.stats.history().len(), 6001);
        assert_eq!(w.seasons().len(), 3);
        let s = &w.seasons()[0];
        assert_eq!((s.generation, &s.agents, &s.summary), (1, &kept, &summary));
        let n = kept.len() as f64;
        assert_eq!(s.summary.mean_l, kept.iter().map(|a| a.l).sum::<f64>() / n);
        let exposed = kept
            .iter()
            .filter(|a| a.record.mean_larder_stock() >= EXPOSURE_FLOOR)
            .count() as u32;
        assert_eq!(s.summary.larder_exposed, exposed);
        let h = w.stats.history();
        assert_eq!((h[2000].generation, h[2001].generation), (1, 2));
        assert_eq!(h[2000].survivors, s.summary.survivors);
        assert_eq!(h[2000].mean_l, s.summary.mean_l);
        assert_eq!(
            h[2000].larder_rate.to_bits(),
            s.summary.pooled_larder_rate.unwrap_or(f64::NAN).to_bits()
        );
        assert!(h[..6000].iter().all(|x| x.takeover.is_nan()));
        assert!(!h[6000].takeover.is_nan());
        assert!(h.iter().enumerate().all(|(t, x)| x.tick == t as u64));
    }

    fn model(c: HoardConfig) -> crate::model::ModelWorld {
        crate::model::ModelWorld::new(ModelConfig::Hoard(c), 3).unwrap()
    }

    fn bits(w: &crate::model::ModelWorld) -> Vec<Vec<u64>> {
        let m = w.model();
        let mut names = m.series_names();
        names.push("tick".into());
        names
            .iter()
            .map(|n| m.series(n).unwrap().into_iter().map(f64::to_bits).collect())
            .collect()
    }

    /// A keyframe taken mid-season, at a season's end (before breeding) and
    /// just after breeding restores a world that runs on as the straight run.
    #[test]
    fn keyframes_restore_mid_season_and_at_generation_boundaries() {
        let c = HoardConfig {
            generations: 4,
            ..HoardConfig::default()
        };
        let end = 6500;
        let mut straight = model(c.clone());
        straight.model_mut().run(end);
        for at in [1000, 2000, 2001, 4000, 5555] {
            let mut w = model(c.clone());
            w.model_mut().run(at);
            let cp = w.checkpoint().unwrap();
            w.model_mut().run(1234);
            w.restore(&cp).unwrap();
            assert_eq!(w.model().tick(), u64::from(at));
            assert_eq!(
                w.model().series("tick").unwrap().len(),
                at as usize + 1,
                "{at}"
            );
            w.model_mut().run(end - at);
            assert_eq!(
                w.model().fingerprint(),
                straight.model().fingerprint(),
                "{at}"
            );
            assert_eq!(bits(&w), bits(&straight), "{at}");
            let (crate::model::ModelWorld::Hoard(a), crate::model::ModelWorld::Hoard(b)) =
                (&w, &straight)
            else {
                unreachable!()
            };
            assert_eq!(a.seasons(), b.seasons(), "{at}");
        }
    }

    /// A 60-generation run at the defaults is deterministic, and ends.
    #[test]
    fn a_60_generation_run_is_deterministic() {
        let run = |seed| {
            let mut w = HoardWorld::new(HoardConfig::default(), seed).unwrap();
            w.run(200_000);
            assert!(w.is_finished());
            (
                Model::fingerprint(&w),
                w.seasons().to_vec(),
                w.outcome(),
                Model::series_csv(&w),
            )
        };
        let a = run(21);
        assert_eq!(a, run(21));
        let seasons = a.1.len();
        assert!(seasons == 60 || a.2.as_ref().unwrap().extinct.is_some());
        assert_ne!(a.0, run(22).0);
    }

    /// The task report's observations: 60 generations at the defaults and at
    /// app_scat 0.1 and 0.8 (app_lard 2), 5 seeds each. Printed with
    /// `--ignored --nocapture`; observations only, never judged here.
    #[test]
    #[ignore]
    fn report_60_generations() {
        for app_scat in [0.44, 0.1, 0.8] {
            for seed in 1..=5 {
                let c = HoardConfig {
                    app_scat,
                    ..HoardConfig::default()
                };
                let mut w = HoardWorld::new(c, seed).unwrap();
                w.run(200_000);
                let o = w.outcome().unwrap();
                let last = &w.seasons().last().unwrap().summary;
                let early: Vec<&SeasonSummary> =
                    w.seasons().iter().take(10).map(|s| &s.summary).collect();
                let m = |f: fn(&SeasonSummary) -> Option<f64>| {
                    let v: Vec<f64> = early.iter().filter_map(|s| f(s)).collect();
                    v.iter().sum::<f64>() / v.len() as f64
                };
                println!(
                    "app_scat {app_scat} seed {seed}: gen-60 mean L {:.3}, window mean L {:.3}, mean D {:.3}, fate {:?}, rise {:?}, survivors(60) {}, gens 1-10 larder {:.2} scatter {:.2}",
                    last.mean_l,
                    o.window_mean_l,
                    last.mean_d,
                    o.fate,
                    o.rise,
                    last.survivors,
                    m(|s| s.mean_larder_rate),
                    m(|s| s.mean_scatter_rate),
                );
            }
        }
    }
}
