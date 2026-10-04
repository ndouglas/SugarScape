//! Full economic periods share one seeded portable engine on native and WASM.
use super::{
    analysis::*,
    config::*,
    decision::{Front, FrontKey},
    resources,
    stats::PolaritySnapshot,
    territory::{self, Cell},
};
use crate::{
    config::FieldError,
    rng::{self, SimRng},
    stats::Stats,
};
use rand::Rng;
use std::collections::{BTreeMap, BTreeSet};
#[derive(Clone)]
pub struct PolarityWorld {
    pub config: PolarityConfig,
    pub tick: u64,
    pub stats: Stats<PolaritySnapshot>,
    pub(crate) period: u64,
    completed: u64,
    period_ledger_accumulated: bool,
    #[cfg(test)]
    pub(super) panic_on_period: Option<u64>,
    pub(crate) cells: Vec<Cell>,
    pub(crate) fronts: BTreeMap<FrontKey, Front>,
    pub(super) seed: u64,
    pub(super) rng: SimRng,
    pub(super) trust: BTreeMap<(usize, usize), f64>,
    pub(super) threats: BTreeMap<usize, usize>,
    pub(super) coalitions: BTreeMap<usize, Vec<usize>>,
    pub(super) aggression: BTreeSet<usize>,
    pub(super) dyadic_aggression: BTreeSet<(usize, usize)>,
    pub(super) pending: BTreeSet<(usize, usize)>,
    pub(super) totals: Ledger,
    pub(super) episodes: Vec<Episode>,
    pub(super) events: Vec<Event>,
    pub(super) events_dropped: u64,
    pub(super) next_event: u64,
    pub(super) next_episode: u64,
    pub(super) initial_predator_share: f64,
    pub(super) outcome: Option<Outcome>,
    pub(super) last_event: Option<Event>,
    pub(super) period_locks: BTreeSet<usize>,
    pub(super) resolved_fronts: BTreeSet<FrontKey>,
}
impl PolarityWorld {
    pub fn new(config: PolarityConfig, seed: u64) -> Result<Self, Vec<FieldError>> {
        config.validate()?;
        let mut rng = rng::seeded(seed);
        let n = (config.width * config.height) as usize;
        let mut cells: Vec<Cell> = (0..n)
            .map(|id| Cell {
                id,
                capital: id,
                predator: false,
                stock: draw(&config, &mut rng, config.initial_mean, config.initial_sd),
            })
            .collect();
        match config.placement {
            Placement::ExactCount => {
                let mut order: Vec<usize> = (0..n).collect();
                shuffle(&mut order, &mut rng);
                let count = (config.predator_share * n as f64 + 0.5).floor() as usize;
                for &i in order.iter().take(count) {
                    cells[i].predator = true;
                }
            }
            Placement::Bernoulli => {
                for c in &mut cells {
                    c.predator = rng.gen::<f64>() < config.predator_share;
                }
            }
        }
        let initial_predator_share = cells.iter().filter(|c| c.predator).count() as f64 / n as f64;
        let mut w = Self {
            config,
            tick: 0,
            stats: Stats::default(),
            period: 0,
            completed: 0,
            period_ledger_accumulated: false,
            #[cfg(test)]
            panic_on_period: None,
            cells,
            fronts: BTreeMap::new(),
            seed,
            rng,
            trust: BTreeMap::new(),
            threats: BTreeMap::new(),
            coalitions: BTreeMap::new(),
            aggression: BTreeSet::new(),
            dyadic_aggression: BTreeSet::new(),
            pending: BTreeSet::new(),
            totals: Ledger::default(),
            episodes: vec![],
            events: vec![],
            events_dropped: 0,
            next_event: 0,
            next_episode: 0,
            initial_predator_share,
            outcome: None,
            last_event: None,
            period_locks: BTreeSet::new(),
            resolved_fronts: BTreeSet::new(),
        };
        w.rebuild();
        let mut e = Ledger::default();
        w.policy(&mut e);
        w.accumulate(&e);
        w.record(e, 0);
        Ok(w)
    }
    pub fn completed_periods(&self) -> u64 {
        self.outcome
            .as_ref()
            .map(|o| o.periods)
            .unwrap_or(self.completed)
    }
    pub fn outcome(&self) -> Option<&Outcome> {
        self.outcome.as_ref()
    }
    pub fn run(&mut self, ticks: u32) {
        for _ in 0..ticks {
            if self.outcome.is_some() {
                break;
            }
            let mut e = Ledger::default();
            let mut count = 0;
            for _ in 0..self.config.periods_per_tick {
                if self.outcome.is_some() {
                    break;
                }
                let completed = self.completed_periods();
                let mut one = Ledger::default();
                self.period_ledger_accumulated = false;
                let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    self.period(&mut one)
                }));
                if let Err(panic) = result {
                    if !self.period_ledger_accumulated {
                        self.accumulate(&one);
                    }
                    self.invalidate_after_panic(panic_message(&*panic));
                } else if self.outcome.as_ref().is_none_or(|o| o.valid) {
                    self.completed = self.period;
                }
                e.add(&one);
                count += (self.completed_periods() - completed) as u32;
            }
            self.tick += 1;
            self.record(e, count);
        }
    }
    pub(super) fn accumulate(&mut self, e: &Ledger) {
        self.period_ledger_accumulated = true;
        self.totals.add(e);
        if !self.totals.all_finite() && self.outcome.is_none() {
            self.finish(
                "invalid",
                Some(format!(
                    "period {}: nonfinite cumulative ledger {:?}; config {:?}",
                    self.period, self.totals, self.config
                )),
            );
        }
        if let Some(o) = &mut self.outcome {
            o.events = self.totals.clone();
            o.destruction = self.totals.destruction;
            o.signed_creation = self.totals.signed_creation;
        }
    }
    pub(super) fn log(&mut self, kind: &str, cells: Vec<usize>, front: Option<[usize; 2]>) {
        let event = Event {
            id: self.next_event,
            period: self.period,
            kind: kind.into(),
            cells,
            front,
        };
        self.next_event += 1;
        self.last_event = Some(event.clone());
        if self.config.event_log {
            if self.events.len() >= self.config.event_log_limit as usize {
                self.events.remove(0);
                self.events_dropped += 1;
            }
            self.events.push(event);
        }
    }
    /// Preserve live accounting and clocks when an external host catches a panic.
    pub fn invalidate_after_panic(&mut self, message: String) {
        self.finish("panic", Some(message));
    }
    pub(super) fn finish(&mut self, reason: &str, invalid: Option<String>) {
        if self.outcome.is_some() {
            return;
        }
        let caps = territory::capitals(&self.cells);
        let mut episodes = self.episodes.clone();
        for front in self.fronts.values() {
            if let Some(mut ep) = front.episode.clone() {
                ep.censored = true;
                episodes.push(ep);
            }
        }
        self.outcome = Some(Outcome {
            config: self.config.clone(),
            seed: self.seed,
            periods: if invalid.is_some() {
                self.completed
            } else {
                self.period
            },
            attempted_period: self.period,
            finish_reason: reason.into(),
            valid: invalid.is_none(),
            invalid_reason: invalid.clone(),
            sovereign_count: caps.len() as u32,
            terminal_category: if invalid.is_none() {
                super::analysis::category(caps.len() as u32)
            } else {
                None
            },
            initial_predator_share: self.initial_predator_share,
            predator_capital_share: caps.iter().filter(|&&c| self.cells[c].predator).count() as f64
                / caps.len() as f64,
            destruction: self.totals.destruction,
            signed_creation: self.totals.signed_creation,
            events: self.totals.clone(),
            episodes,
        });
    }
    pub(super) fn rebuild(&mut self) {
        let mut wanted = BTreeSet::new();
        let caps = territory::capitals(&self.cells);
        for &a in &caps {
            for b in territory::neighbors(&self.config, &self.cells, a) {
                wanted.insert(FrontKey::foreign(a, b));
            }
        }
        if self.config.provincial() {
            for c in &self.cells {
                if c.capital != c.id {
                    wanted.insert(FrontKey {
                        domestic: true,
                        a: c.capital,
                        b: c.id,
                    });
                }
            }
        }
        let old = std::mem::take(&mut self.fronts);
        for (key, mut f) in old {
            let valid = wanted.contains(&key) && f.path.is_none_or(|p| self.valid_path(key, p));
            if valid {
                self.fronts.insert(key, f);
            } else if let Some(mut ep) = f.episode.take() {
                ep.end = Some(self.period);
                ep.end_cause = Some(
                    if wanted.contains(&key) {
                        "path_loss"
                    } else if self.cells[key.a].capital != key.a
                        || (!key.domestic && self.cells[key.b].capital != key.b)
                    {
                        "sovereignty_loss"
                    } else {
                        "border_loss"
                    }
                    .into(),
                );
                self.episodes.push(ep);
            }
        }
        for key in wanted {
            if !self.fronts.contains_key(&key) {
                let na = self.front_count(key.a, key.domestic, false);
                let nb = if key.domestic {
                    1
                } else {
                    self.front_count(key.b, false, false)
                };
                let old_commitments = [
                    self.cells[key.a].stock / na.max(1) as f64,
                    self.cells[key.b].stock / nb.max(1) as f64,
                ];
                self.fronts.insert(
                    key,
                    Front {
                        key,
                        previous: [false, false],
                        actions: [false, false],
                        old_commitments,
                        commitments: old_commitments,
                        path: None,
                        initiated: [false, false],
                        episode: None,
                    },
                );
            }
        }
        self.trust.retain(|(a, b), _| {
            self.cells[*a].capital == *a
                && self.cells[*b].capital == *b
                && territory::neighbors(&self.config, &self.cells, *a).contains(b)
        });
        for f in self.fronts.values().filter(|f| !f.key.domestic) {
            self.trust
                .entry((f.key.a, f.key.b))
                .or_insert(self.config.trust_initial);
            self.trust
                .entry((f.key.b, f.key.a))
                .or_insert(self.config.trust_initial);
        }
    }
    fn period(&mut self, e: &mut Ledger) {
        self.period += 1;
        self.period_locks.clear();
        self.resolved_fronts.clear();
        self.update_trust();
        self.prepare();
        let caps = territory::capitals(&self.cells);
        self.aggression.clear();
        self.dyadic_aggression.clear();
        if self.config.update == Update::Sequential {
            self.sequential(caps, e);
            if self.outcome.is_some() {
                self.accumulate(e);
                return;
            }
        } else {
            for actor in caps {
                self.decide_actor(actor, e);
                if self.outcome.is_some() {
                    self.accumulate(e);
                    return;
                }
            }
            self.obligations();
            self.paths(e);
            let stocks: Vec<f64> = self.cells.iter().map(|c| c.stock).collect();
            let frozen = self.allocate(&stocks);
            for (&k, &v) in &frozen {
                self.fronts.get_mut(&k).unwrap().commitments = v;
            }
            if let Some((k, _)) = frozen
                .iter()
                .find(|(_, v)| v.iter().any(|x| !x.is_finite()))
            {
                self.finish(
                    "invalid",
                    Some(format!(
                        "period {} front {:?}: nonfinite allocation; config {:?}",
                        self.period, k, self.config
                    )),
                );
                self.accumulate(e);
                return;
            }
            let keys: Vec<FrontKey> = self.fronts.keys().copied().collect();
            let mut damage = vec![0.0; self.cells.len()];
            for &key in &keys {
                let f = &self.fronts[&key];
                let losses = resources::damage(
                    f.actions,
                    f.commitments,
                    self.config.damage_rate,
                    self.config.asymmetric_losses(),
                );
                damage[key.a] += losses[0];
                damage[key.b] += losses[1];
                if f.actions == [true, true] {
                    e.dd_encounters += 1;
                }
                for loss in losses {
                    if loss > 0.0 {
                        e.destruction += loss;
                    } else {
                        e.signed_creation -= loss;
                    }
                }
                self.episode_period(key, losses);
            }
            let mut victories = BTreeMap::new();
            if self.config.victory_timing == VictoryTiming::BeforeDamage {
                for &key in &keys {
                    if self.fronts[&key].actions.iter().any(|x| *x) {
                        let win = self.victory(key, frozen[&key], e);
                        victories.insert(key, win);
                        if self.outcome.is_some() {
                            self.accumulate(e);
                            return;
                        }
                    }
                }
            }
            for (cell, loss) in self.cells.iter_mut().zip(damage) {
                cell.stock -= loss;
            }
            if !self.intermediate_valid() {
                self.accumulate(e);
                if let Some(o) = &mut self.outcome {
                    o.events = self.totals.clone();
                    o.destruction = self.totals.destruction;
                    o.signed_creation = self.totals.signed_creation;
                }
                return;
            }
            if self.config.victory_timing == VictoryTiming::AfterDamage {
                let stocks = self.cells.iter().map(|c| c.stock).collect::<Vec<_>>();
                let values = self.allocate(&stocks);
                for &key in &keys {
                    if self.fronts[&key].actions.iter().any(|x| *x) {
                        let win = self.victory(key, values[&key], e);
                        victories.insert(key, win);
                        if self.outcome.is_some() {
                            break;
                        }
                    }
                }
            }
            if self.outcome.is_some() {
                self.accumulate(e);
                return;
            }
            self.harvest(e);
            #[cfg(test)]
            if self.panic_on_period == Some(self.period) {
                panic!("injected period panic after harvest");
            }
            if !self.intermediate_valid() {
                self.accumulate(e);
                if let Some(o) = &mut self.outcome {
                    o.events = self.totals.clone();
                }
                return;
            }
            if self.config.victory_timing == VictoryTiming::AfterHarvest {
                let stocks = self.cells.iter().map(|c| c.stock).collect::<Vec<_>>();
                let values = self.allocate(&stocks);
                for &key in &keys {
                    if self.fronts[&key].actions.iter().any(|x| *x) {
                        let win = self.victory(key, values[&key], e);
                        victories.insert(key, win);
                        if self.outcome.is_some() {
                            break;
                        }
                    }
                }
            }
            if self.outcome.is_some() {
                self.accumulate(e);
                return;
            }
            let mut claims = Vec::new();
            for (key, win) in victories {
                if let Some(side) = win {
                    if let Some(claim) = self.claim(key, side) {
                        claims.push(claim);
                    }
                    let winner = if side == 0 { key.a } else { key.b };
                    self.end_episode(key, "victory", Some(winner));
                }
            }
            self.note_aggression();
            shuffle(&mut claims, &mut self.rng);
            self.structural(&claims, e);
        }
        for f in self.fronts.values_mut() {
            f.previous = f.actions;
            f.old_commitments = f.commitments;
            f.initiated = [false, false];
        }
        self.rebuild();
        self.form_coalitions();
        self.policy(e);
        self.accumulate(e);
        // Policy invalidation happens before totals were added; retain complete ledgers.
        if let Some(o) = &mut self.outcome {
            o.events = self.totals.clone();
            o.destruction = self.totals.destruction;
            o.signed_creation = self.totals.signed_creation;
        }
        if self.outcome.is_none() {
            if self.config.stop_at_hegemony && territory::capitals(&self.cells).len() == 1 {
                self.finish("hegemony", None);
            } else if self.period >= self.config.horizon {
                self.finish("horizon", None);
            }
        }
    }
    fn record(&mut self, events: Ledger, last_tick_periods: u32) {
        let caps = territory::capitals(&self.cells);
        let mut sizes: Vec<usize> = caps
            .iter()
            .map(|&c| territory::members(&self.cells, c).len())
            .collect();
        sizes.sort_unstable_by(|a, b| b.cmp(a));
        let capital_stock = caps.iter().map(|&i| self.cells[i].stock).sum::<f64>();
        let total_stock = self.cells.iter().map(|c| c.stock).sum::<f64>();
        let s = PolaritySnapshot {
            tick: self.tick,
            period: self.completed_periods(),
            periods: self.completed_periods(),
            attempted_period: self.period,
            last_tick_periods,
            sovereign_count: caps.len() as u32,
            sovereigns: caps.len() as u32,
            category: self.outcome.as_ref().and_then(|o| o.terminal_category),
            largest_territory: sizes.first().copied().unwrap_or(0),
            second_largest_territory: sizes.get(1).copied().unwrap_or(0),
            predator_capital_share: caps.iter().filter(|&&i| self.cells[i].predator).count() as f64
                / caps.len() as f64,
            total_stock,
            capital_stock,
            province_stock: total_stock - capital_stock,
            nonpositive_stocks: self
                .cells
                .iter()
                .filter(|c| c.stock <= 0.0 && (self.config.provincial() || c.id == c.capital))
                .count(),
            coalitions: self.coalitions.len(),
            open_episodes: self.fronts.values().filter(|f| f.episode.is_some()).count(),
            finish_reason: self.outcome.as_ref().map(|o| o.finish_reason.clone()),
            invalidity: self.outcome.as_ref().and_then(|o| o.invalid_reason.clone()),
            events_dropped: self.events_dropped,
            events,
        };
        if self.stats.history().len() >= 1_000_001 {
            self.stats.truncate(1_000_000);
        }
        self.stats.push(s);
    }
    pub fn economic_fingerprint(&self) -> u64 {
        let state = serde_json::json!({"period":self.period,"cells":self.cells,"fronts":self.fronts.values().collect::<Vec<_>>(),"trust":self.trust.iter().collect::<Vec<_>>(),"coalitions":self.coalitions,"pending":self.pending,"aggression":self.aggression,"dyadic":self.dyadic_aggression,"totals":self.totals,"episodes":self.episodes,"threats":self.threats,"next_episode":self.next_episode,"next_event":self.next_event,"last_event":self.last_event});
        let mut h = hash(&serde_json::to_vec(&state).unwrap());
        let mut r = self.rng.clone();
        for _ in 0..2 {
            for b in r.gen::<u64>().to_le_bytes() {
                h ^= u64::from(b);
                h = h.wrapping_mul(0x0100_0000_01b3);
            }
        }
        h
    }
}
pub(super) fn hash(bytes: &[u8]) -> u64 {
    let mut h = 0xcbf2_9ce4_8422_2325u64;
    for &b in bytes {
        h ^= u64::from(b);
        h = h.wrapping_mul(0x0100_0000_01b3);
    }
    h
}
pub(super) fn draw(c: &PolarityConfig, r: &mut SimRng, mean: f64, sd: f64) -> f64 {
    match c.resource_distribution {
        ResourceDistribution::Normal => mean + sd * crate::anasazi::random::normal(r),
        ResourceDistribution::BoundedUniform => {
            mean + 3.0_f64.sqrt() * sd * (2.0 * r.gen::<f64>() - 1.0)
        }
    }
}
pub(super) fn index(r: &mut SimRng, len: usize) -> usize {
    r.gen_range(0..len as u32) as usize
}
pub(super) fn shuffle<T>(items: &mut [T], r: &mut SimRng) {
    for i in (1..items.len()).rev() {
        let j = index(r, i + 1);
        items.swap(i, j);
    }
}
pub(super) fn discount(base: f64, n: u32) -> f64 {
    let mut value = 1.0;
    for _ in 0..n {
        value *= base;
    }
    value
}

fn panic_message(panic: &(dyn std::any::Any + Send)) -> String {
    panic
        .downcast_ref::<String>()
        .cloned()
        .or_else(|| panic.downcast_ref::<&str>().map(|s| s.to_string()))
        .unwrap_or_else(|| "unknown panic".into())
}
