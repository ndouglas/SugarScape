//! Sequential auctions and the fixed-horizon observation protocol.
use super::{analysis, config::*, learner::Learner, mechanism, stats::AuctionsSnapshot};
use crate::{
    config::FieldError,
    model::{wrong_model, Model, ModelConfig, ModelKind},
    portable::exp_neg,
    rng::{self, SimRng},
    stats::{Series, Stats},
};
use rand::Rng;
use serde::Serialize;
use std::fmt::Write;
/// Retains full page history, then the first million snapshots plus latest for longer native runs.
pub const MAX_HISTORY: usize = 1_000_001;
fn bounded_push(stats: &mut Stats<AuctionsSnapshot>, snapshot: AuctionsSnapshot, cap: usize) {
    if stats.history().len() >= cap {
        stats.truncate(cap - 1);
    }
    stats.push(snapshot);
}
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Outcome {
    pub config: AuctionsConfig,
    pub seed: u64,
    pub equilibria: Vec<Vec<f64>>,
    pub periods: u64,
    pub converged: bool,
    pub greedy: Vec<f64>,
    pub greedy_actions: Vec<usize>,
    pub terminal_revenue: f64,
    pub terminal_profits: Vec<f64>,
    pub deviation_gain: Vec<f64>,
    pub top_profile: bool,
    pub below_top: bool,
    pub low_non_nash: bool,
    pub whole_revenue: f64,
    pub late_revenue: f64,
    pub whole_profits: Vec<f64>,
    pub late_profits: Vec<f64>,
    pub occupancy: Vec<u64>,
    pub late_occupancy: Vec<u64>,
    pub grid: Vec<f64>,
    pub whole_count: u64,
    pub late_count: u64,
    pub stable: u64,
    pub activation: Option<u64>,
    pub q: Vec<Vec<f64>>,
    pub chosen: Vec<Vec<u64>>,
    pub updated: Vec<Vec<u64>>,
    pub win_shares: Vec<f64>,
}
#[derive(Clone)]
pub struct AuctionsWorld {
    pub config: AuctionsConfig,
    pub tick: u64,
    seed: u64,
    period: u64,
    rng: SimRng,
    grid: Vec<f64>,
    equilibria: Vec<Vec<f64>>,
    learners: Vec<Learner>,
    stable: u64,
    observed: Option<Vec<usize>>,
    activation: Option<u64>,
    occupancy: Vec<u64>,
    late_occupancy: Vec<u64>,
    revenue: f64,
    late_revenue: f64,
    profits: Vec<f64>,
    late_profits: Vec<f64>,
    wins: Vec<f64>,
    late_count: u64,
    played: Vec<usize>,
    last_bids: Vec<f64>,
    priorities: Vec<f64>,
    shares: Vec<f64>,
    payment: f64,
    epsilon: f64,
    outcome: Option<Outcome>,
    pub stats: Stats<AuctionsSnapshot>,
}
impl AuctionsWorld {
    pub fn new(config: AuctionsConfig, seed: u64) -> Result<Self, Vec<FieldError>> {
        config.validate()?;
        let grid = mechanism::grid(&config);
        let equilibria = analysis::equilibria(&config);
        let mut rng = rng::seeded(seed);
        let n = config.bidders as usize;
        let learners = (0..n)
            .map(|_| Learner::new(&config, &grid, rng.gen::<f64>()))
            .collect();
        let m = grid.len();
        let last_bids = vec![grid[0]; n];
        let mut out = Self {
            config,
            tick: 0,
            seed,
            period: 0,
            rng,
            grid,
            equilibria,
            learners,
            stable: 0,
            observed: None,
            activation: None,
            occupancy: vec![0; m * m],
            late_occupancy: vec![0; m * m],
            revenue: 0.0,
            late_revenue: 0.0,
            profits: vec![0.0; n],
            late_profits: vec![0.0; n],
            wins: vec![0.0; n],
            late_count: 0,
            played: vec![0; n],
            last_bids,
            priorities: vec![0.0; n + 1],
            shares: vec![0.0; n],
            payment: 0.0,
            epsilon: 0.0,
            outcome: None,
            stats: Stats::default(),
        };
        let mut s = AuctionsSnapshot {
            periods: 0.0,
            stable: 0.0,
            ..Default::default()
        };
        out.policies(&mut s);
        out.stats.push(s);
        Ok(out)
    }
    pub fn period(&self) -> u64 {
        self.period
    }
    pub fn grid(&self) -> &[f64] {
        &self.grid
    }
    pub fn learners(&self) -> &[Learner] {
        &self.learners
    }
    pub fn outcome(&self) -> Option<&Outcome> {
        self.outcome.as_ref()
    }
    pub fn is_finished(&self) -> bool {
        self.period >= u64::from(self.config.horizon)
    }
    fn policies(&self, s: &mut AuctionsSnapshot) {
        s.greedy_1 = self.grid[self.learners[0].greedy];
        s.greedy_2 = self.grid[self.learners[1].greedy];
        s.greedy_3 = self
            .learners
            .get(2)
            .map_or(f64::NAN, |f| self.grid[f.greedy]);
    }
    fn one_period(&mut self) -> (Vec<f64>, Vec<f64>, f64, f64, u32) {
        let c = &self.config;
        let n = self.learners.len();
        if self.activation.is_none()
            && match c.downward_trigger {
                DownwardTrigger::Off => false,
                DownwardTrigger::Stable => self.stable >= u64::from(c.window),
                DownwardTrigger::Period => self.period >= u64::from(c.downward_at),
            }
        {
            self.activation = Some(self.period);
        }
        let clock = self.period + u64::from(c.period_origin == PeriodOrigin::One);
        let epsilon = if c.exploration == Exploration::Constant {
            c.epsilon
        } else {
            c.epsilon * exp_neg(-c.beta * clock as f64)
        };
        self.epsilon = epsilon;
        let chi = self.activation.map_or(0.0, |tau| {
            c.downward_chi
                * exp_neg(
                    -c.downward_beta
                        * if c.downward_clock == DownwardClock::Activation {
                            (self.period - tau) as f64
                        } else {
                            clock as f64
                        },
                )
        });
        let previous: Vec<usize> = self.learners.iter().map(|f| f.greedy).collect();
        let mut tie_draws = vec![0.0; n];
        let mut explored = 0.0;
        let mut downward = 0.0;
        for (i, f) in self.learners.iter().enumerate() {
            let down = self.rng.gen::<f64>();
            let explore = self.rng.gen::<f64>();
            let action = self.rng.gen::<f64>();
            tie_draws[i] = self.rng.gen::<f64>();
            self.played[i] = if down < chi {
                downward += 1.0;
                let max = f.maximum();
                f.q.iter().position(|q| max - q <= c.downward_gap).unwrap()
            } else if explore < epsilon {
                explored += 1.0;
                f.explore(c, action)
            } else {
                f.greedy
            };
        }
        let fringe = self.rng.gen::<f64>();
        for priority in &mut self.priorities {
            *priority = self.rng.gen::<f64>();
        }
        self.last_bids = self.played.iter().map(|a| self.grid[*a]).collect();
        if c.fringe == Fringe::Uniform {
            self.last_bids.push(fringe);
        }
        let result = mechanism::clear(
            c,
            &self.last_bids,
            &self.priorities,
            c.auction_ties == AuctionTies::Expected,
        );
        self.payment = result.revenue;
        self.shares = result.shares.clone();
        for (i, f) in self.learners.iter_mut().enumerate() {
            let mut rewards = vec![0.0; self.grid.len()];
            if c.update == Update::Chosen {
                rewards[self.played[i]] = result.rewards[i];
            } else {
                for (a, r) in rewards.iter_mut().enumerate() {
                    *r = mechanism::hypothetical(
                        c,
                        &self.last_bids,
                        &self.priorities,
                        i,
                        self.grid[a],
                    );
                }
            }
            f.update(c, self.played[i], &rewards, tie_draws[i]);
        }
        let current: Vec<usize> = self.learners.iter().map(|f| f.greedy).collect();
        let changes = current
            .iter()
            .zip(&previous)
            .filter(|(a, b)| a != b)
            .count() as u32;
        let observed = if c.convergence_phase == ConvergencePhase::PostUpdate {
            current
        } else {
            previous
        };
        if self.observed.as_ref() == Some(&observed) {
            self.stable += 1;
        } else {
            self.stable = 1;
            self.observed = Some(observed);
        }
        let cell = self.played[0] * self.grid.len() + self.played[1];
        self.occupancy[cell] += 1;
        self.revenue += result.revenue;
        for i in 0..n {
            self.profits[i] += result.rewards[i];
            self.wins[i] += result.shares[i];
        }
        self.period += 1;
        if self.period > u64::from(c.horizon) * 4 / 5 {
            self.late_count += 1;
            self.late_occupancy[cell] += 1;
            self.late_revenue += result.revenue;
            for i in 0..n {
                self.late_profits[i] += result.rewards[i];
            }
        }
        (
            self.last_bids[..n].to_vec(),
            result.rewards[..n].to_vec(),
            explored / n as f64,
            downward / n as f64,
            changes,
        )
    }
    pub fn step(&mut self) {
        if self.is_finished() {
            return;
        }
        let count = u64::from(self.config.periods_per_tick)
            .min(u64::from(self.config.horizon) - self.period);
        let n = self.learners.len();
        let mut bids = vec![0.0; n];
        let mut profits = vec![0.0; n];
        let mut revenue = 0.0;
        let mut explored = 0.0;
        let mut downward = 0.0;
        let mut eps = 0.0;
        let mut changes = 0u64;
        for _ in 0..count {
            let (b, p, e, d, g) = self.one_period();
            for i in 0..n {
                bids[i] += b[i];
                profits[i] += p[i];
            }
            revenue += self.payment;
            eps += self.epsilon;
            explored += e;
            downward += d;
            changes += u64::from(g);
        }
        self.tick += 1;
        let div = count as f64;
        let mut s = AuctionsSnapshot {
            tick: self.tick,
            periods_in_tick: count as u32,
            bid_1: bids[0] / div,
            bid_2: bids[1] / div,
            bid_3: bids.get(2).map_or(f64::NAN, |b| b / div),
            profit_1: profits[0] / div,
            profit_2: profits[1] / div,
            profit_3: profits.get(2).map_or(f64::NAN, |p| p / div),
            revenue: revenue / div,
            epsilon: eps / div,
            explored: explored / div,
            downward: downward / div,
            greedy_changes: changes as f64,
            stable: self.stable as f64,
            periods: self.period as f64,
            ..Default::default()
        };
        self.policies(&mut s);
        if self.is_finished() {
            self.outcome = Some(self.analyze());
            let o = self.outcome.as_ref().unwrap();
            s.converged = f64::from(o.converged);
            s.terminal_revenue = o.terminal_revenue;
            s.terminal_deviation_gain = o.deviation_gain.iter().copied().fold(0.0, f64::max);
            s.top_profile = f64::from(o.top_profile);
        }
        bounded_push(&mut self.stats, s, MAX_HISTORY);
    }
    pub fn run(&mut self, ticks: u32) {
        for _ in 0..ticks {
            if self.is_finished() {
                break;
            }
            self.step();
        }
    }
    fn analyze(&self) -> Outcome {
        let greedy_actions: Vec<usize> = self.learners.iter().map(|f| f.greedy).collect();
        let greedy: Vec<f64> = greedy_actions.iter().map(|a| self.grid[*a]).collect();
        let evaluated = analysis::evaluate(&self.config, &greedy);
        let top = *self.grid.last().unwrap();
        let below_top = greedy.iter().all(|b| *b < top);
        Outcome {
            config: self.config.clone(),
            seed: self.seed,
            equilibria: self.equilibria.clone(),
            periods: self.period,
            converged: self.stable >= u64::from(self.config.window),
            top_profile: greedy.iter().all(|b| *b == top),
            below_top,
            low_non_nash: below_top && evaluated.deviation_gain.iter().any(|g| *g > 1e-12),
            greedy,
            greedy_actions,
            terminal_revenue: evaluated.revenue,
            terminal_profits: evaluated.profits,
            deviation_gain: evaluated.deviation_gain,
            whole_revenue: self.revenue / self.period as f64,
            late_revenue: self.late_revenue / self.late_count as f64,
            whole_profits: self
                .profits
                .iter()
                .map(|p| p / self.period as f64)
                .collect(),
            late_profits: self
                .late_profits
                .iter()
                .map(|p| p / self.late_count as f64)
                .collect(),
            occupancy: self.occupancy.clone(),
            late_occupancy: self.late_occupancy.clone(),
            grid: self.grid.clone(),
            whole_count: self.period,
            late_count: self.late_count,
            stable: self.stable,
            activation: self.activation,
            q: self.learners.iter().map(|f| f.q.clone()).collect(),
            chosen: self.learners.iter().map(|f| f.chosen.clone()).collect(),
            updated: self.learners.iter().map(|f| f.updated.clone()).collect(),
            win_shares: self.wins.clone(),
        }
    }
}
impl Model for AuctionsWorld {
    fn config(&self) -> ModelConfig {
        ModelConfig::Auctions(self.config.clone())
    }
    fn run(&mut self, ticks: u32) {
        AuctionsWorld::run(self, ticks)
    }
    fn tick(&self) -> u64 {
        self.tick
    }
    fn population(&self) -> usize {
        self.learners.len()
    }
    fn fingerprint(&self) -> u64 {
        let state = serde_json::json!({"config":self.config,"seed":self.seed,"tick":self.tick,"period":self.period,"grid":self.grid,"learners":self.learners,"stable":self.stable,"observed":self.observed,"activation":self.activation,"occupancy":self.occupancy,"late_occupancy":self.late_occupancy,"revenue":self.revenue,"late_revenue":self.late_revenue,"profits":self.profits,"late_profits":self.late_profits,"wins":self.wins,"late_count":self.late_count,"played":self.played,"last_bids":self.last_bids,"priorities":self.priorities,"shares":self.shares,"payment":self.payment,"epsilon":self.epsilon,"outcome":self.outcome});
        let bytes = serde_json::to_vec(&state).unwrap();
        let mut h = 0xcbf2_9ce4_8422_2325u64;
        for byte in bytes {
            h ^= u64::from(byte);
            h = h.wrapping_mul(0x0100_0000_01b3);
        }
        let mut stream = self.rng.clone();
        for _ in 0..2 {
            for byte in stream.gen::<u64>().to_le_bytes() {
                h ^= u64::from(byte);
                h = h.wrapping_mul(0x0100_0000_01b3);
            }
        }
        h
    }

    fn size(&self) -> (u32, u32) {
        let m = self.grid.len();
        (
            (m * super::view::CELL) as u32,
            (m * super::view::CELL).max(self.learners.len() * super::view::BIDDER_HEIGHT) as u32,
        )
    }
    fn render(&self, mode: &str, _layer: &str, buf: &mut Vec<u8>) -> Result<(), String> {
        if !["", "bids", "late", "values"].contains(&mode) {
            return Err(format!(
                "unknown color mode {mode:?} (expected bids, late or values)"
            ));
        }
        let (w, h) = self.size();
        let mut canvas = crate::opinions::Canvas { buf, wide: 0 };
        canvas.clear(w as usize, h as usize);
        let m = self.grid.len();
        let cell = super::view::CELL;
        if mode == "values" {
            for (i, f) in self.learners.iter().enumerate() {
                let max = f.maximum();
                for a in 0..m {
                    for row in 0..3 {
                        let value = match row {
                            0 => {
                                if max.abs() > 0.0 {
                                    f.q[a] / max
                                } else {
                                    0.0
                                }
                            }
                            1 => f.chosen[a] as f64 / self.period.max(1) as f64,
                            _ => f.updated[a] as f64 / self.period.max(1) as f64,
                        }
                        .clamp(0.0, 1.0);
                        let color = [
                            (30.0 + 190.0 * value) as u8,
                            (40.0 + 110.0 * value) as u8,
                            100,
                        ];
                        for dy in 0..cell {
                            for dx in 0..cell {
                                let mark = a == f.greedy && (dx == 0 || dy == 0);
                                canvas.put(
                                    a * cell + dx,
                                    i * 36 + row * cell + dy,
                                    if mark {
                                        [240, 210, 70]
                                    } else if self.period > 0
                                        && self.played[i] == a
                                        && (dx + 1 == cell || dy + 1 == cell)
                                    {
                                        [70, 220, 230]
                                    } else {
                                        color
                                    },
                                );
                            }
                        }
                    }
                }
            }
        } else {
            let counts = if mode == "late" {
                &self.late_occupancy
            } else {
                &self.occupancy
            };
            let most = counts.iter().copied().max().unwrap_or(1).max(1) as f64;
            for a in 0..m {
                for b in 0..m {
                    let value = counts[a * m + b] as f64 / most;
                    let color = [
                        (20.0 + 210.0 * value) as u8,
                        (35.0 + 120.0 * value) as u8,
                        (60.0 + 80.0 * value) as u8,
                    ];
                    for dy in 0..cell {
                        for dx in 0..cell {
                            let mark = self.period > 0
                                && self.played[0] == a
                                && self.played[1] == b
                                && (dx == 0 || dy == 0 || dx + 1 == cell || dy + 1 == cell);
                            canvas.put(
                                a * cell + dx,
                                (m - 1 - b) * cell + dy,
                                if mark { [240, 210, 70] } else { color },
                            );
                        }
                    }
                }
            }
        }
        Ok(())
    }
    fn latest_json(&self) -> String {
        serde_json::to_string(&self.stats.latest()).unwrap()
    }
    fn series_names(&self) -> Vec<String> {
        super::series_names(self.config.bidders)
    }
    fn series(&self, name: &str) -> Option<Vec<f64>> {
        if self.config.bidders == 2 && ["bid_3", "greedy_3", "profit_3"].contains(&name) {
            None
        } else {
            self.stats.series(name)
        }
    }
    fn latest_value(&self, name: &str) -> Option<f64> {
        if self.config.bidders == 2 && ["bid_3", "greedy_3", "profit_3"].contains(&name) {
            None
        } else {
            self.stats.latest().and_then(|s| s.value(name))
        }
    }
    fn series_csv(&self) -> String {
        crate::export::history_csv(&self.series_names(), self.stats.history())
    }
    fn agents_csv(&self) -> String {
        let mut out = "bidder,action,bid,q,chosen,updated,greedy\n".to_string();
        for (i, f) in self.learners.iter().enumerate() {
            for (a, q) in f.q.iter().enumerate() {
                writeln!(
                    out,
                    "{},{},{},{},{},{},{}",
                    i + 1,
                    a,
                    self.grid[a],
                    q,
                    f.chosen[a],
                    f.updated[a],
                    a == f.greedy
                )
                .unwrap();
            }
        }
        out
    }
    fn inspect_json(&self, x: u32, y: u32) -> Result<String, String> {
        let (w, h) = self.size();
        if x >= w || y >= h {
            return Err(format!("({x}, {y}) is not in the frame"));
        }
        let action = x as usize / 12;
        let bidder = y as usize / 36;
        let rival = self.grid.len().checked_sub(1 + y as usize / 12);
        let cell = rival.map(|r| action * self.grid.len() + r);
        let f = self.learners.get(bidder);
        let hypothetical_reward = f.map(|_| {
            mechanism::hypothetical(
                &self.config,
                &self.last_bids,
                &self.priorities,
                bidder,
                self.grid[action],
            )
        });
        Ok(serde_json::json!({"x":x,"y":y,"panel":"bids","bidder":f.map(|_|bidder),"action":action,"count":cell.map(|k|self.occupancy[k]),"frequency":cell.map(|k|self.occupancy[k] as f64/self.period.max(1) as f64),"q":f.map(|f|f.q[action]),"chosen":f.map(|f|f.chosen[action]),"updated":f.map(|f|f.updated[action]),"hypothetical_reward":hypothetical_reward,"tick":self.tick,"period":self.period,"horizon":self.config.horizon,"equilibria":self.equilibria,"grid":self.grid,"greedy":self.learners.iter().map(|f|self.grid[f.greedy]).collect::<Vec<_>>(),"played":self.played.iter().map(|a|self.grid[*a]).collect::<Vec<_>>(),"shares":self.shares,"fringe_bid":self.last_bids.get(self.learners.len()).copied(),"fringe_share":self.shares.get(self.learners.len()).copied(),"payment":self.payment,"epsilon":self.epsilon,"stable":self.stable,"occupancy":self.occupancy,"late_occupancy":self.late_occupancy,"whole_count":self.period,"late_count":self.late_count,"learners":self.learners,"outcome":self.outcome,"agent":null}).to_string())
    }
    fn locate(&self, _id: u64) -> Option<(u32, u32)> {
        None
    }
    fn set_config(&mut self, next: ModelConfig) -> Result<(), Vec<FieldError>> {
        let ModelConfig::Auctions(next) = next else {
            return Err(wrong_model(ModelKind::Auctions, &next));
        };
        next.validate()?;
        let old = serde_json::to_value(&self.config).unwrap();
        let new = serde_json::to_value(&next).unwrap();
        let errors: Vec<FieldError> = old
            .as_object()
            .unwrap()
            .iter()
            .filter(|(key, value)| new[*key] != **value)
            .map(|(key, _)| FieldError::new(key, "changes only on reset"))
            .collect();
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
    fn finished(&self) -> bool {
        self.is_finished()
    }
    fn holds_when_finished(&self) -> bool {
        true
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn short(ppt: u32) -> AuctionsWorld {
        AuctionsWorld::new(
            AuctionsConfig {
                horizon: 23,
                window: 5,
                periods_per_tick: ppt,
                ..Default::default()
            },
            1,
        )
        .unwrap()
    }
    #[test]
    fn values_view_marks_played_and_greedy_actions_with_distinct_edges() {
        let mut w = short(1);
        w.step();
        let mut buf = vec![];
        w.render("values", "", &mut buf).unwrap();
        let width = w.size().0 as usize;
        let at = (11 * width + 11) * 4;
        assert_eq!(&buf[at..at + 3], &[70, 220, 230]);
        let greedy = w.learners[0].greedy * 12 * 4;
        assert_eq!(&buf[greedy..greedy + 3], &[240, 210, 70]);
        let mut w = AuctionsWorld::new(
            AuctionsConfig {
                q_init: QInit::Constant,
                q_level: 0.0,
                epsilon: 0.0,
                horizon: 2,
                window: 1,
                periods_per_tick: 1,
                ..Default::default()
            },
            1,
        )
        .unwrap();
        w.step();
        w.render("values", "", &mut buf).unwrap();
        assert_eq!(&buf[..3], &[240, 210, 70]);
        assert_eq!(&buf[at..at + 3], &[70, 220, 230]);
    }
    #[test]
    fn inspection_exposes_the_actual_fringe_competitor_and_its_share() {
        let mut w = AuctionsWorld::new(
            AuctionsConfig {
                fringe: Fringe::Uniform,
                horizon: 2,
                window: 1,
                periods_per_tick: 1,
                ..Default::default()
            },
            1,
        )
        .unwrap();
        let before: serde_json::Value =
            serde_json::from_str(&w.inspect_json(0, 0).unwrap()).unwrap();
        assert!(before["fringe_bid"].is_null());
        w.step();
        let after: serde_json::Value =
            serde_json::from_str(&w.inspect_json(0, 0).unwrap()).unwrap();
        assert_eq!(after["fringe_bid"].as_f64(), Some(w.last_bids[2]));
        assert_eq!(after["fringe_share"].as_f64(), Some(w.shares[2]));
    }
    #[test]
    fn bounded_history_keeps_prefix_and_latest_with_actual_tick_stamps() {
        let mut stats = Stats::default();
        for tick in 0..5 {
            bounded_push(
                &mut stats,
                AuctionsSnapshot {
                    tick,
                    ..Default::default()
                },
                3,
            );
        }
        assert_eq!(
            stats.history().iter().map(|s| s.tick).collect::<Vec<_>>(),
            vec![0, 1, 4]
        );
    }
    #[test]
    fn clock_origin_and_fixed_draw_budget_do_not_depend_on_treatment_branches() {
        let base = AuctionsConfig {
            horizon: 2,
            window: 1,
            periods_per_tick: 1,
            epsilon: 0.5,
            beta: 1.0,
            ..Default::default()
        };
        let mut zero = AuctionsWorld::new(base.clone(), 7).unwrap();
        let mut one = AuctionsWorld::new(
            AuctionsConfig {
                period_origin: PeriodOrigin::One,
                auction_ties: AuctionTies::Expected,
                fringe: Fringe::Uniform,
                ..base
            },
            7,
        )
        .unwrap();
        zero.step();
        one.step();
        assert_eq!(zero.epsilon, 0.5);
        assert_eq!(one.epsilon, 0.5 * exp_neg(-1.0));
        assert_eq!(zero.rng.clone().gen::<f64>(), one.rng.clone().gen::<f64>());
        let mut expected = rng::seeded(7);
        for _ in 0..14 {
            expected.gen::<f64>();
        }
        assert_eq!(zero.rng.gen::<f64>(), expected.gen::<f64>());
    }
    #[test]
    fn stability_phase_observes_cached_or_updated_profile_without_changing_learning() {
        let base = AuctionsConfig {
            horizon: 2,
            window: 1,
            periods_per_tick: 1,
            epsilon: 0.0,
            ..Default::default()
        };
        let mut post = AuctionsWorld::new(base.clone(), 1).unwrap();
        let mut pre = AuctionsWorld::new(
            AuctionsConfig {
                convergence_phase: ConvergencePhase::PreUpdate,
                ..base
            },
            1,
        )
        .unwrap();
        post.step();
        pre.step();
        assert_eq!(pre.observed, Some(vec![0, 0]));
        assert_eq!(post.observed, Some(vec![1, 1]));
        assert_eq!(pre.learners, post.learners);
    }
    #[test]
    fn third_bidder_series_are_unavailable_in_two_bidder_runs() {
        let w = short(7);
        assert!(w.series("bid_3").is_none());
        assert!(w.latest_value("greedy_3").is_none());
        assert!(!w.series_names().contains(&"profit_3".to_string()));
        let w = AuctionsWorld::new(
            AuctionsConfig {
                bidders: 3,
                ..Default::default()
            },
            1,
        )
        .unwrap();
        assert!(w.latest_value("greedy_3").unwrap().is_finite());
    }
    #[test]
    fn tiny_three_bidder_views_and_inspection_cover_all_value_rows() {
        let w = AuctionsWorld::new(
            AuctionsConfig {
                bids: 2,
                bidders: 3,
                horizon: 2,
                window: 1,
                ..Default::default()
            },
            1,
        )
        .unwrap();
        assert_eq!(w.size(), (24, 108));
        for mode in ["bids", "late", "values"] {
            let mut buf = vec![];
            w.render(mode, "", &mut buf).unwrap();
            assert_eq!(buf.len(), 24 * 108 * 4);
        }
        let inspection: serde_json::Value =
            serde_json::from_str(&w.inspect_json(13, 85).unwrap()).unwrap();
        assert_eq!(inspection["bidder"], 2);
        assert!(inspection["q"].is_number());
        assert!(inspection["count"].is_null());
    }
    #[test]
    fn short_native_fingerprints() {
        let base = AuctionsConfig {
            horizon: 23,
            window: 5,
            periods_per_tick: 7,
            ..Default::default()
        };
        for (c, want) in [
            base.clone(),
            AuctionsConfig {
                auction: Auction::SecondPrice,
                ..base.clone()
            },
            AuctionsConfig {
                feedback: Feedback::RivalBids,
                update: Update::All,
                ..base.clone()
            },
            AuctionsConfig {
                fringe: Fringe::Uniform,
                ..base.clone()
            },
            AuctionsConfig { bidders: 3, ..base },
        ]
        .into_iter()
        .zip([
            0x8736a8664d834a2du64,
            0x1ba7cb1e9d3ff993,
            0x949a7d400b127f20,
            0x5f621ad6482502e7,
            0x5c5c0013fee7f42b,
        ]) {
            let mut w = AuctionsWorld::new(c, 1).unwrap();
            w.run(4);
            assert_eq!(w.fingerprint(), want);
        }
    }
    #[test]
    fn batching_preserves_economic_state_and_finishes_partial_tick() {
        let mut a = short(1);
        let mut b = short(7);
        let mut c = short(1000);
        a.run(23);
        b.run(4);
        c.run(1);
        assert_eq!(a.learners, b.learners);
        assert_eq!(a.learners, c.learners);
        assert_eq!(a.occupancy, b.occupancy);
        assert_eq!(a.stable, b.stable);
        assert_eq!((b.period, b.tick, b.late_count), (23, 4, 5));
        let before = b.fingerprint();
        b.run(20);
        assert_eq!(b.fingerprint(), before);
    }
    #[test]
    fn unused_feedback_cannot_change_learning() {
        for seed in 1..=10 {
            let c = AuctionsConfig {
                horizon: 101,
                window: 5,
                periods_per_tick: 7,
                ..Default::default()
            };
            let mut a = AuctionsWorld::new(c.clone(), seed).unwrap();
            let mut b = AuctionsWorld::new(
                AuctionsConfig {
                    feedback: Feedback::RivalBids,
                    ..c
                },
                seed,
            )
            .unwrap();
            a.run(15);
            b.run(15);
            assert_eq!(a.learners, b.learners);
            assert_eq!(a.occupancy, b.occupancy);
            assert_eq!(a.profits, b.profits);
        }
    }
    #[test]
    fn transient_stability_does_not_finish_session() {
        let mut w = AuctionsWorld::new(
            AuctionsConfig {
                q_init: QInit::Biased,
                bias_bid: 0.4,
                bias_q: 30.0,
                epsilon: 0.0,
                horizon: 20,
                window: 2,
                periods_per_tick: 1,
                ..Default::default()
            },
            1,
        )
        .unwrap();
        w.run(5);
        assert!(w.stable >= 2);
        assert!(!w.is_finished());
        assert!(w.outcome().is_none());
    }
    #[test]
    fn period_downward_activation_uses_next_period_and_activation_clock() {
        let mut w = AuctionsWorld::new(
            AuctionsConfig {
                horizon: 5,
                window: 1,
                periods_per_tick: 1,
                downward_trigger: DownwardTrigger::Period,
                downward_at: 2,
                downward_chi: 1.0,
                downward_beta: 1.0,
                ..Default::default()
            },
            1,
        )
        .unwrap();
        w.run(2);
        assert_eq!(w.activation, None);
        w.step();
        assert_eq!(w.activation, Some(2));
        assert_eq!(w.stats.latest().unwrap().downward, 1.0);
    }
}
