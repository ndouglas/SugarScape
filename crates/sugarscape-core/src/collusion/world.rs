//! One session: n firms learning to price together until their strategies
//! settle (CCDP §II), then what they learned.

use std::collections::VecDeque;
use std::fmt::Write;
use std::sync::Arc;

use serde::Serialize;

use super::analysis::{
    best_response_deviation, deviate, equilibrium, every_deviation, invitation, limit_cycle,
    rp_complete, Cycle, Equilibrium, Response, Strategies, HORIZON,
};
use super::config::{CollusionConfig, Exploration, Impulse, QInit, Update};
use super::demand::Game;
use super::learner::{initial_prices, Draws, Firm, Space};
use super::stats::CollusionSnapshot;
use super::view::{
    cell, price_color, row, side, visit_color, AXIS, BACK, FIRMS, GAP, MARK, MONOPOLY, NASH,
    PANEL_H, SHOWN, STEP,
};
use crate::config::FieldError;
use crate::export;
use crate::model::{wrong_model, Model, ModelConfig, ModelKind};
use crate::opinions::Canvas;
use crate::portable::exp_neg;
use crate::stats::{Series, Stats};

/// What a finished session learned.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Outcome {
    /// Whether the strategies settled (or the cap stopped the session).
    pub converged: bool,
    /// Periods to convergence, excluding the window (the code's measure).
    pub periods: u64,
    #[serde(skip)]
    pub strategies: Strategies,
    pub cycle: Cycle,
    pub gains: Vec<f64>,
    pub gain: f64,
    pub window_gain: f64,
    pub discounted_gain: f64,
    pub equilibrium: Equilibrium,
    /// The deviations `impulse` asks for and their responses.
    #[serde(skip)]
    pub responses: Vec<Response>,
    pub punishment_like: f64,
    pub rp_complete: bool,
    /// Off-path states whose greedy price is still the starting one.
    pub stale_greedy: f64,
    /// Periods from ε falling below 0.01 to the last greedy change.
    pub fumbling: Option<u64>,
    /// The share of Q-values ever updated.
    pub touched: f64,
}

#[derive(Clone)]
pub struct CollusionWorld {
    pub config: CollusionConfig,
    /// Completed ticks (`periods_per_tick` periods each, the last perhaps
    /// cut short when the session finished).
    pub tick: u64,
    /// Completed periods.
    period: u64,
    game: Arc<Game>,
    space: Space,
    firms: Vec<Firm>,
    draws: Draws,
    /// The state the next period's prices are chosen in.
    state: usize,
    decay: f64,
    stable: u32,
    /// The state at convergence (the code's `stateFix`).
    fixed: Option<usize>,
    converged: bool,
    initial: Strategies,
    touched: Vec<Vec<bool>>,
    touched_count: u64,
    visits: Vec<u32>,
    streak_profit: Vec<f64>,
    streak_len: u64,
    discounted: Vec<f64>,
    weight: f64,
    horizon: u32,
    last_change: u64,
    quiet_from: Option<u64>,
    recent: VecDeque<Vec<u8>>,
    explored: u32,
    changes: u32,
    /// This tick's sums: the profit gain, the share of firms exploring, and
    /// greedy changes, over `tick_periods` periods.
    tick_gain: f64,
    tick_explored: f64,
    tick_changes: u32,
    tick_periods: u32,
    outcome: Option<Outcome>,
    pub stats: Stats<CollusionSnapshot>,
}

/// One state of the Inspect panel.
#[derive(Clone, Debug, Serialize)]
pub struct StateView {
    pub state: usize,
    /// The prices the state records, latest first: `prices[d][i]`.
    pub prices: Vec<Vec<f64>>,
    /// Each firm's Q-values and greedy price there.
    pub q: Vec<Vec<f64>>,
    pub greedy: Vec<f64>,
    pub visits: u32,
}

#[derive(Clone, Debug, Serialize)]
pub struct CollusionInspection {
    pub x: u32,
    pub y: u32,
    pub panel: Option<&'static str>,
    pub firm: Option<usize>,
    pub state: Option<StateView>,
    pub tick: u64,
    pub period: u64,
    pub nash: Vec<f64>,
    pub monopoly: Vec<f64>,
    pub outcome: Option<Outcome>,
    /// Always null: there are no agents to follow, only firms.
    pub agent: Option<u8>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CollusionMode {
    Price,
    Visits,
}

impl std::str::FromStr for CollusionMode {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "price" | "" => Ok(CollusionMode::Price),
            "visits" => Ok(CollusionMode::Visits),
            other => Err(format!(
                "unknown color mode {other:?} (expected price or visits)"
            )),
        }
    }
}

impl CollusionWorld {
    pub fn new(config: CollusionConfig, seed: u64) -> Result<Self, Vec<FieldError>> {
        config.validate()?;
        let game = Game::new(&config);
        let space = Space::of(&config);
        let (n, m) = (space.firms, space.prices);
        let mut draws = Draws::new(&config, seed);
        // Every firm's Q first, then the greedy prices (the code's order).
        let tables: Vec<Vec<f64>> = (0..n)
            .map(|i| match config.q_init {
                QInit::Calvano => {
                    let row: Vec<f64> = (0..m)
                        .map(|a| {
                            let mut sum = 0.0;
                            let mut count = 0u32;
                            for profile in 0..m.pow(n as u32) {
                                let own = profile / m.pow((n - 1 - i) as u32) % m;
                                if own == a {
                                    sum += game.payoff[profile * n + i];
                                    count += 1;
                                }
                            }
                            sum / (f64::from(count) * (1.0 - config.delta))
                        })
                        .collect();
                    row.repeat(space.states)
                }
                QInit::Zero => vec![0.0; space.states * m],
                QInit::Random => (0..space.states * m)
                    .map(|_| config.q_low + (config.q_high - config.q_low) * draws.tie())
                    .collect(),
            })
            .collect();
        let eps = match config.exploration {
            Exploration::Decaying | Exploration::TwoPhase => 1.0,
            Exploration::Constant => config.epsilon,
            Exploration::Boltzmann => config.temperature,
        };
        let firms: Vec<Firm> = tables
            .into_iter()
            .map(|q| Firm::new(q, m, config.ties, &mut draws, eps))
            .collect();
        let start = initial_prices(&config, seed, &mut draws);
        let state = space.encode(&start);
        let initial = firms.iter().map(|f| f.greedy.clone()).collect();
        let horizon = game.horizon(config.delta);
        let mut world = CollusionWorld {
            decay: exp_neg(-config.beta),
            game: Arc::new(game),
            space,
            initial,
            touched: vec![vec![false; space.states * m]; n],
            touched_count: 0,
            visits: vec![0; space.states],
            streak_profit: vec![0.0; n],
            streak_len: 0,
            discounted: vec![0.0; n],
            weight: 1.0,
            horizon,
            last_change: 0,
            quiet_from: None,
            recent: VecDeque::new(),
            explored: 0,
            changes: 0,
            tick_gain: 0.0,
            tick_explored: 0.0,
            tick_changes: 0,
            tick_periods: 0,
            firms,
            draws,
            state,
            stable: 0,
            fixed: None,
            converged: false,
            outcome: None,
            config,
            tick: 0,
            period: 0,
            stats: Stats::default(),
        };
        world.record();
        Ok(world)
    }

    pub fn game(&self) -> &Game {
        &self.game
    }

    pub fn space(&self) -> &Space {
        &self.space
    }

    pub fn firms(&self) -> &[Firm] {
        &self.firms
    }

    /// Completed periods.
    pub fn period(&self) -> u64 {
        self.period
    }

    pub fn state(&self) -> usize {
        self.state
    }

    /// The state the strategies are read in: where the session settled, or
    /// the current one.
    pub fn current_state(&self) -> usize {
        self.fixed.unwrap_or(self.state)
    }

    pub fn is_finished(&self) -> bool {
        self.fixed.is_some()
    }

    pub fn outcome(&self) -> Option<&Outcome> {
        self.outcome.as_ref()
    }

    /// The greedy strategies now.
    pub fn strategies(&self) -> Strategies {
        self.firms.iter().map(|f| f.greedy.clone()).collect()
    }

    /// Each firm's price this period.
    fn choose(&mut self, u: &[[f64; 2]]) -> Vec<u8> {
        let m = self.space.prices;
        let s = self.state;
        let t = self.period + 1;
        let mut out = Vec::with_capacity(self.firms.len());
        self.explored = 0;
        for (i, f) in self.firms.iter_mut().enumerate() {
            let greedy = f.greedy[s];
            let a = match self.config.exploration {
                Exploration::Decaying | Exploration::Constant | Exploration::TwoPhase => {
                    let eps = match self.config.exploration {
                        Exploration::TwoPhase => {
                            if t <= u64::from(self.config.explore_for) {
                                1.0
                            } else {
                                0.0
                            }
                        }
                        _ => f.eps,
                    };
                    let a = if u[i][0] <= eps {
                        (m as f64 * u[i][1]) as u8
                    } else {
                        greedy
                    };
                    if self.config.exploration == Exploration::Decaying {
                        f.eps *= self.decay;
                    }
                    a
                }
                Exploration::Boltzmann => {
                    let row = f.row(s, m);
                    let top = f.best[s];
                    let probs: Vec<f64> = row
                        .iter()
                        .map(|q| exp_neg((q - top).min(0.0) / f.eps))
                        .collect();
                    let target = u[i][0] * probs.iter().sum::<f64>();
                    let mut cum = 0.0;
                    let mut pick = (m - 1) as u8;
                    for (a, p) in probs.iter().enumerate() {
                        cum += p;
                        if target <= cum {
                            pick = a as u8;
                            break;
                        }
                    }
                    f.eps *= 1.0 - self.config.cooling;
                    pick
                }
            };
            if a != greedy {
                self.explored += 1;
            }
            out.push(a);
        }
        out
    }

    /// One tick: up to `periods_per_tick` periods, fewer if the session
    /// finishes; then a statistics snapshot.
    pub fn step(&mut self) {
        if self.is_finished() {
            return;
        }
        for _ in 0..self.config.periods_per_tick {
            self.step_period();
            if self.is_finished() {
                break;
            }
        }
        self.tick += 1;
        self.record();
    }

    /// One period: prices, profits, the firms' updates, convergence.
    pub fn step_period(&mut self) {
        if self.is_finished() {
            return;
        }
        let n = self.space.firms;
        let m = self.space.prices;
        let mut u = vec![[0.0; 2]; n];
        self.draws.explore(&mut u);
        let actions = self.choose(&u);
        let s = self.state;
        let next = self.space.next(s, &actions);
        let profile = self.game.profile(&actions);
        let before: Vec<u8> = self.firms.iter().map(|f| f.greedy[s]).collect();
        let (alpha, delta, ties) = (self.config.alpha, self.config.delta, self.config.ties);
        for i in 0..n {
            match self.config.update {
                Update::Asynchronous => {
                    let a = actions[i];
                    let f = &mut self.firms[i];
                    let old = f.q[s * m + usize::from(a)];
                    let target = self.game.payoff[profile * n + i] + delta * f.best[next];
                    f.set(s, a, old + alpha * (target - old), m, ties, &mut self.draws);
                    let k = s * m + usize::from(a);
                    if !self.touched[i][k] {
                        self.touched[i][k] = true;
                        self.touched_count += 1;
                    }
                }
                Update::Synchronous => {
                    let mut a = actions.clone();
                    let targets: Vec<f64> = (0..m as u8)
                        .map(|p| {
                            a[i] = p;
                            let r = self.game.profit(&a, i);
                            r + delta * self.firms[i].best[self.space.with_own(next, i, p)]
                        })
                        .collect();
                    let f = &mut self.firms[i];
                    for (p, target) in targets.into_iter().enumerate() {
                        let old = f.q[s * m + p];
                        f.set(
                            s,
                            p as u8,
                            old + alpha * (target - old),
                            m,
                            ties,
                            &mut self.draws,
                        );
                        if !self.touched[i][s * m + p] {
                            self.touched[i][s * m + p] = true;
                            self.touched_count += 1;
                        }
                    }
                }
            }
        }
        let t = self.period + 1;
        self.changes = (0..n)
            .filter(|&i| self.firms[i].greedy[s] != before[i])
            .count() as u32;
        let profits: Vec<f64> = (0..n).map(|i| self.game.payoff[profile * n + i]).collect();
        if self.changes == 0 {
            self.stable += 1;
            self.streak_len += 1;
            for (sum, p) in self.streak_profit.iter_mut().zip(&profits) {
                *sum += p;
            }
        } else {
            self.stable = 1;
            self.last_change = t;
            self.streak_len = 1;
            self.streak_profit.clone_from(&profits);
        }
        if t <= u64::from(self.horizon) {
            for (d, p) in self.discounted.iter_mut().zip(&profits) {
                *d += self.weight * p;
            }
            self.weight *= delta;
        }
        if self.quiet_from.is_none()
            && self.config.exploration == Exploration::Decaying
            && self.firms[0].eps < 0.01
        {
            self.quiet_from = Some(t);
        }
        self.visits[s] = self.visits[s].saturating_add(1);
        if self.recent.len() == SHOWN {
            self.recent.pop_front();
        }
        self.tick_gain += profits
            .iter()
            .enumerate()
            .map(|(i, &p)| self.game.gain(i, p))
            .sum::<f64>()
            / n as f64;
        self.tick_explored += f64::from(self.explored) / n as f64;
        self.tick_changes += self.changes;
        self.tick_periods += 1;
        self.recent.push_back(actions);
        self.period = t;
        if t > u64::from(self.config.cap) {
            // The code keeps the strategy from before this period's update.
            for (f, &b) in self.firms.iter_mut().zip(&before) {
                f.greedy[s] = b;
            }
            self.converged = false;
            self.fixed = Some(s);
        } else if self.stable == self.config.window {
            self.converged = true;
            self.fixed = Some(s);
        } else {
            self.state = next;
        }
        if self.fixed.is_some() {
            self.outcome = Some(self.analyze());
        }
    }

    pub fn run(&mut self, ticks: u32) {
        for _ in 0..ticks {
            if self.is_finished() {
                break;
            }
            self.step();
        }
    }

    /// The deviations `impulse` asks for, from every cycle position.
    fn responses(&self, strategies: &Strategies, cycle: &Cycle) -> Vec<Response> {
        let (game, space) = (&*self.game, &self.space);
        let mut out = Vec::new();
        match self.config.impulse {
            Impulse::BestResponseDown => {
                for firm in 0..space.firms {
                    for k in 0..cycle.len() {
                        out.push(best_response_deviation(
                            game,
                            space,
                            strategies,
                            cycle,
                            k,
                            firm,
                            self.config.best_response_to,
                        ));
                    }
                }
            }
            Impulse::EveryPrice => out = every_deviation(space, strategies, cycle),
            Impulse::Up | Impulse::Invitation => {
                for firm in 0..space.firms {
                    for k in 0..cycle.len() {
                        let own = strategies[firm][cycle.states[k]];
                        if usize::from(own) + 1 < space.prices {
                            out.push(deviate(space, strategies, cycle, k, firm, own + 1));
                        }
                    }
                }
            }
        }
        out
    }

    fn analyze(&self) -> Outcome {
        let fixed = self.fixed.expect("analyzed when finished");
        let (game, space) = (&*self.game, &self.space);
        let strategies = self.strategies();
        let cycle = limit_cycle(game, space, &strategies, fixed);
        let gains = cycle.gains(game);
        let gain = gains.iter().sum::<f64>() / gains.len() as f64;
        let mean_gain = |profits: &[f64], scale: f64| {
            (0..space.firms)
                .map(|i| game.gain(i, profits[i] * scale))
                .sum::<f64>()
                / space.firms as f64
        };
        let window_gain = mean_gain(&self.streak_profit, 1.0 / self.streak_len.max(1) as f64);
        let discounted_gain = self.discounted_gain();
        let eq = equilibrium(
            game,
            space,
            &strategies,
            &cycle,
            self.config.delta,
            self.config.equilibrium_check,
        );
        let (responses, punishment_like) = if self.config.impulse == Impulse::Invitation {
            let results: Vec<(u8, u8)> = (0..space.firms)
                .filter_map(|f| {
                    invitation(
                        game,
                        space,
                        &strategies,
                        &cycle,
                        f,
                        self.config.invitation_hold,
                    )
                })
                .collect();
            let share = if results.is_empty() {
                f64::NAN
            } else {
                results
                    .iter()
                    .filter(|(back, before)| back < before)
                    .count() as f64
                    / results.len() as f64
            };
            (self.responses(&strategies, &cycle), share)
        } else {
            let r = self.responses(&strategies, &cycle);
            let share = if r.is_empty() {
                f64::NAN
            } else {
                r.iter().filter(|x| x.punishment_like()).count() as f64 / r.len() as f64
            };
            (r, share)
        };
        let off: Vec<usize> = (0..space.states)
            .filter(|s| !cycle.states.contains(s))
            .collect();
        let stale_greedy = if off.is_empty() {
            0.0
        } else {
            off.iter()
                .map(|&s| {
                    (0..space.firms)
                        .filter(|&i| strategies[i][s] == self.initial[i][s])
                        .count()
                })
                .sum::<usize>() as f64
                / (off.len() * space.firms) as f64
        };
        Outcome {
            converged: self.converged,
            periods: self.period.saturating_sub(u64::from(self.config.window)),
            rp_complete: rp_complete(space, &strategies, &cycle),
            strategies,
            gains,
            gain,
            window_gain,
            discounted_gain,
            equilibrium: eq,
            responses,
            punishment_like,
            stale_greedy,
            fumbling: self.quiet_from.map(|q| self.last_change.saturating_sub(q)),
            touched: self.touched_count as f64 / (space.firms * space.states * space.prices) as f64,
            cycle,
        }
    }

    /// den Boer, Meylahn & Schinkel's Δ̃: the discounted profit over the
    /// first T_δ periods, normalized by the weights' sum (NaN before then,
    /// unless the session ended first).
    pub fn discounted_gain(&self) -> f64 {
        let t_d = u64::from(self.horizon).min(self.period);
        if t_d == 0 || (self.period < u64::from(self.horizon) && !self.is_finished()) {
            return f64::NAN;
        }
        let d = self.config.delta;
        let sum_w = if d > 0.0 {
            (1.0 - d.powi(t_d as i32)) / (1.0 - d)
        } else {
            1.0
        };
        (0..self.space.firms)
            .map(|i| self.game.gain(i, self.discounted[i] / sum_w))
            .sum::<f64>()
            / self.space.firms as f64
    }

    /// T_δ, the periods `discounted_gain` covers.
    pub fn horizon(&self) -> u32 {
        self.horizon
    }

    fn record(&mut self) {
        let n = self.space.firms;
        let mut s = CollusionSnapshot {
            tick: self.tick,
            stable: self.stable,
            greedy_changes: self.tick_changes,
            ..CollusionSnapshot::default()
        };
        if let Some(a) = self.recent.back() {
            s.price_1 = self.game.grid[0][usize::from(a[0])];
            s.price_2 = self.game.grid[1][usize::from(a[1])];
        }
        if self.tick_periods > 0 {
            let k = f64::from(self.tick_periods);
            s.profit_gain = self.tick_gain / k;
            s.explored = self.tick_explored / k;
        }
        (
            self.tick_gain,
            self.tick_explored,
            self.tick_changes,
            self.tick_periods,
        ) = (0.0, 0.0, 0, 0);
        let st = self.fixed.unwrap_or(self.state);
        s.greedy_price = (0..n)
            .map(|i| self.game.grid[i][usize::from(self.firms[i].greedy[st])])
            .sum::<f64>()
            / n as f64;
        s.epsilon = match self.config.exploration {
            Exploration::TwoPhase => {
                if self.period < u64::from(self.config.explore_for) {
                    1.0
                } else {
                    0.0
                }
            }
            _ => self.firms[0].eps,
        };
        self.results(&mut s);
        self.stats.push(s);
    }

    /// The session's results into snapshot `s` (NaN while it learns).
    fn results(&self, s: &mut CollusionSnapshot) {
        s.discounted_gain = self.discounted_gain();
        if let Some(o) = &self.outcome {
            s.converged = if o.converged { 1.0 } else { 0.0 };
            s.cycle_length = o.cycle.len() as f64;
            s.cycle_gain = o.gain;
            s.window_gain = o.window_gain;
            s.equilibrium_on_path = if o.equilibrium.on_path { 1.0 } else { 0.0 };
            s.punishment_like = o.punishment_like;
            s.rp_complete = if o.rp_complete { 1.0 } else { 0.0 };
            s.periods = o.periods as f64;
        }
    }

    /// The state firm `i`'s map cell (own price `own`, the next firm's
    /// `rival`) stands for: the current state with those two prices replaced.
    fn map_state(&self, i: usize, own: u8, rival: u8) -> usize {
        let sp = &self.space;
        if sp.memory == 0 {
            return 0;
        }
        let s = self.fixed.unwrap_or(self.state);
        let mut digits = vec![vec![0u8; sp.firms]; sp.memory];
        let mut rest = s;
        for d in (0..sp.memory).rev() {
            for j in (0..sp.firms).rev() {
                digits[d][j] = (rest % sp.prices) as u8;
                rest /= sp.prices;
            }
        }
        digits[0][i] = own;
        digits[0][(i + 1) % sp.firms] = rival;
        sp.encode(&digits)
    }

    fn view_state(&self, s: usize) -> StateView {
        let sp = &self.space;
        let m = sp.prices;
        let mut prices = vec![vec![0.0; sp.firms]; sp.memory];
        let mut rest = s;
        for d in (0..sp.memory).rev() {
            for j in (0..sp.firms).rev() {
                prices[d][j] = self.game.grid[j][rest % m];
                rest /= m;
            }
        }
        StateView {
            state: s,
            prices,
            q: self.firms.iter().map(|f| f.row(s, m).to_vec()).collect(),
            greedy: (0..sp.firms)
                .map(|i| self.game.grid[i][usize::from(self.firms[i].greedy[s])])
                .collect(),
            visits: self.visits[s],
        }
    }

    fn price_x(&self) -> usize {
        self.space.firms * (side(self.space.prices) + GAP)
    }

    pub fn inspect(&self, x: u32, y: u32) -> Result<CollusionInspection, String> {
        let (fw, fh) = Model::size(self);
        if x >= fw || y >= fh {
            return Err(format!("({x}, {y}) is not in the frame"));
        }
        let (cx, cy) = (x as usize, y as usize);
        let m = self.space.prices;
        let (side, cell) = (side(m), cell(m));
        let mut out = CollusionInspection {
            x,
            y,
            panel: None,
            firm: None,
            state: None,
            tick: self.tick,
            period: self.period,
            nash: self.game.nash.clone(),
            monopoly: self.game.monopoly.clone(),
            outcome: self.outcome.clone(),
            agent: None,
        };
        let i = cx / (side + GAP);
        if i < self.space.firms && cx % (side + GAP) < side && cy < side {
            let own = ((cx % (side + GAP)) / cell) as u8;
            let rival = (m - 1 - cy / cell) as u8;
            out.panel = Some("strategy");
            out.firm = Some(i);
            out.state = Some(self.view_state(self.map_state(i, own, rival)));
        } else if cx >= self.price_x() && cx < self.price_x() + SHOWN {
            out.panel = Some("prices");
            out.state = Some(self.view_state(self.fixed.unwrap_or(self.state)));
        } else if cx >= self.price_x() + SHOWN + GAP {
            out.panel = Some("response");
        }
        Ok(out)
    }
}

impl Model for CollusionWorld {
    fn config(&self) -> ModelConfig {
        ModelConfig::Collusion(self.config.clone())
    }

    fn run(&mut self, ticks: u32) {
        CollusionWorld::run(self, ticks);
    }

    fn tick(&self) -> u64 {
        self.tick
    }

    fn population(&self) -> usize {
        self.firms.len()
    }

    /// FNV-1a over the tick, the state, the counter, and every firm's
    /// Q-values, greedy prices and exploration rate.
    fn fingerprint(&self) -> u64 {
        let mut h: u64 = 0xcbf2_9ce4_8422_2325;
        let mut eat = |bytes: &[u8]| {
            for &b in bytes {
                h ^= u64::from(b);
                h = h.wrapping_mul(0x0100_0000_01b3);
            }
        };
        eat(&self.tick.to_le_bytes());
        eat(&self.period.to_le_bytes());
        eat(&(self.state as u64).to_le_bytes());
        eat(&self.stable.to_le_bytes());
        for f in &self.firms {
            eat(&f.eps.to_bits().to_le_bytes());
            eat(&f.greedy);
            for q in &f.q {
                eat(&q.to_bits().to_le_bytes());
            }
        }
        h
    }

    fn size(&self) -> (u32, u32) {
        let w = self.price_x() + SHOWN + GAP + HORIZON * STEP;
        let h = side(self.space.prices).max(PANEL_H);
        (w as u32, h as u32)
    }

    fn render(&self, mode: &str, _layer: &str, buf: &mut Vec<u8>) -> Result<(), String> {
        let mode: CollusionMode = mode.parse()?;
        let (fw, fh) = Model::size(self);
        let mut c = Canvas { buf, wide: 0 };
        c.clear(fw as usize, fh as usize);
        let sp = &self.space;
        let m = sp.prices;
        let (side, cell) = (side(m), cell(m));
        let current = self.fixed.unwrap_or(self.state);
        let most = self.visits.iter().copied().max().unwrap_or(0);
        for i in 0..sp.firms {
            let x0 = i * (side + GAP);
            let last = sp.last(current);
            for own in 0..m as u8 {
                for rival in 0..m as u8 {
                    let s = self.map_state(i, own, rival);
                    let color = match mode {
                        CollusionMode::Price => price_color(self.firms[i].greedy[s], m),
                        CollusionMode::Visits => visit_color(self.visits[s], most),
                    };
                    let (px, py) = (
                        x0 + usize::from(own) * cell,
                        (m - 1 - usize::from(rival)) * cell,
                    );
                    let here = last
                        .as_ref()
                        .is_some_and(|l| l[i] == own && l[(i + 1) % sp.firms] == rival);
                    for dy in 0..cell {
                        for dx in 0..cell {
                            let edge = dx == 0 || dy == 0 || dx == cell - 1 || dy == cell - 1;
                            c.put(px + dx, py + dy, if here && edge { MARK } else { color });
                        }
                    }
                }
            }
        }
        // The last periods' prices, on firm 1's grid's scale.
        let (lo, hi) = (self.game.grid[0][0], self.game.grid[0][m - 1]);
        let x0 = self.price_x();
        for x in 0..SHOWN {
            c.put(x0 + x, row(self.game.nash[0], lo, hi), NASH);
            c.put(x0 + x, row(self.game.monopoly[0], lo, hi), MONOPOLY);
        }
        for (k, a) in self.recent.iter().enumerate() {
            for (j, &p) in a.iter().enumerate() {
                c.put(
                    x0 + k,
                    row(self.game.grid[j][usize::from(p)], lo, hi),
                    FIRMS[j % 4],
                );
            }
        }
        // The response to the first deviation, once finished.
        let x1 = x0 + SHOWN + GAP;
        for x in 0..HORIZON * STEP {
            c.put(x1 + x, PANEL_H - 1, AXIS);
            c.put(x1 + x, row(self.game.nash[0], lo, hi), NASH);
            c.put(x1 + x, row(self.game.monopoly[0], lo, hi), MONOPOLY);
        }
        if let Some(r) = self.outcome.as_ref().and_then(|o| o.responses.first()) {
            for (t, a) in r.path.iter().enumerate() {
                for (j, &p) in a.iter().enumerate() {
                    let y = row(self.game.grid[j][usize::from(p)], lo, hi);
                    for dx in 0..STEP - 1 {
                        c.put(x1 + t * STEP + dx, y, FIRMS[j % 4]);
                    }
                }
            }
        } else {
            for y in 0..PANEL_H - 1 {
                c.put(x1, y, BACK);
            }
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

    fn latest_value(&self, name: &str) -> Option<f64> {
        self.stats.latest().and_then(|s| s.value(name))
    }

    fn series_csv(&self) -> String {
        export::history_csv(&self.series_names(), self.stats.history())
    }

    /// One row per firm: its price now, its greedy price in the current
    /// state, its exploration rate and, once finished, its Δ.
    fn agents_csv(&self) -> String {
        let mut out = String::from("firm,price,greedy_price,epsilon,gain\n");
        let s = self.fixed.unwrap_or(self.state);
        for (i, f) in self.firms.iter().enumerate() {
            let price = self
                .recent
                .back()
                .map_or(f64::NAN, |a| self.game.grid[i][usize::from(a[i])]);
            let gain = self.outcome.as_ref().map_or(f64::NAN, |o| o.gains[i]);
            writeln!(
                out,
                "{},{},{},{},{}",
                i + 1,
                price,
                self.game.grid[i][usize::from(f.greedy[s])],
                f.eps,
                gain
            )
            .unwrap();
        }
        out
    }

    fn inspect_json(&self, x: u32, y: u32) -> Result<String, String> {
        let inspection = self.inspect(x, y)?;
        Ok(serde_json::to_string(&inspection).expect("inspection serializes"))
    }

    fn locate(&self, _id: u64) -> Option<(u32, u32)> {
        None
    }

    fn set_config(&mut self, next: ModelConfig) -> Result<(), Vec<FieldError>> {
        let ModelConfig::Collusion(next) = next else {
            return Err(wrong_model(ModelKind::Collusion, &next));
        };
        next.validate()?;
        let changes = self.config.structural_changes(&next);
        if !changes.is_empty() {
            return Err(changes);
        }
        self.config = next;
        if self.is_finished() {
            self.outcome = Some(self.analyze());
            let mut last = self.stats.latest().cloned().unwrap_or_default();
            self.results(&mut last);
            self.stats.truncate(self.stats.history().len() - 1);
            self.stats.push(last);
        }
        Ok(())
    }

    fn finished(&self) -> bool {
        self.is_finished()
    }

    /// A finished session holds its results: a sweep reads them.
    fn holds_when_finished(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::collusion::config::{RngKind, Ties};

    fn world(edit: impl FnOnce(&mut CollusionConfig)) -> CollusionWorld {
        let mut c = CollusionConfig::default();
        edit(&mut c);
        CollusionWorld::new(c, 1).unwrap()
    }

    #[test]
    fn the_starting_q_is_equation_eight() {
        let w = world(|_| {});
        let want = [
            5.790, 6.008, 6.162, 6.252, 6.278, 6.244, 6.153, 6.010, 5.821, 5.593, 5.332, 5.047,
            4.744, 4.430, 4.111,
        ];
        for (a, v) in want.iter().enumerate() {
            assert!(
                (w.firms()[0].q[a] - v).abs() < 0.001,
                "{a}: {}",
                w.firms()[0].q[a]
            );
        }
        assert!(w.firms().iter().all(|f| f.greedy.iter().all(|&g| g == 4)));
        assert_eq!(w.stats.history().len(), 1);
    }

    #[test]
    fn a_short_session_converges_and_is_analyzed() {
        // Fast exploration decay and a short window: a session in seconds.
        let mut w = world(|c| {
            c.beta = 2e-4;
            c.window = 2_000;
        });
        w.run(2_000_000);
        assert!(w.is_finished());
        let o = w.outcome().unwrap();
        assert!(o.converged);
        assert_eq!(o.periods, w.period() - 2_000);
        assert_eq!(w.tick, w.period().div_ceil(1000));
        assert!(o.gain > -0.5 && o.gain <= 1.0, "{}", o.gain);
        assert!(!o.cycle.is_empty());
        let last = w.stats.latest().unwrap();
        assert_eq!(last.tick, w.tick);
        assert_eq!(last.cycle_gain, o.gain);
        assert!(!o.responses.is_empty());
        // Finished worlds do not move.
        let fp = w.fingerprint();
        w.run(10);
        assert_eq!(w.fingerprint(), fp);
    }

    #[test]
    fn the_cap_stops_a_session_that_never_settles() {
        let mut w = world(|c| {
            c.exploration = Exploration::Constant;
            c.epsilon = 1.0;
            c.cap = 5_000;
            c.window = 1_000;
        });
        w.run(10_000);
        assert!(w.is_finished());
        assert_eq!((w.period(), w.tick), (5_001, 6));
        assert!(!w.outcome().unwrap().converged);
    }

    #[test]
    fn exploration_decays_by_e_to_the_minus_beta() {
        let mut w = world(|c| c.periods_per_tick = 1);
        w.run(1000);
        let want = exp_neg(-4e-6 * 1000.0);
        assert!((w.firms()[0].eps - want).abs() < 1e-12);
        let mut two = world(|c| {
            c.exploration = Exploration::TwoPhase;
            c.explore_for = 10;
            c.periods_per_tick = 1;
        });
        two.run(10);
        assert_eq!(two.stats.latest().unwrap().epsilon, 0.0);
    }

    #[test]
    fn synchronous_updates_touch_every_price_in_the_state() {
        let mut w = world(|c| c.update = Update::Synchronous);
        let s = w.state();
        let before = w.firms()[0].row(s, 15).to_vec();
        w.step_period();
        let after = w.firms()[0].row(s, 15).to_vec();
        assert!(before.iter().zip(&after).all(|(a, b)| a != b));
        let mut a = world(|_| {});
        let s = a.state();
        let before = a.firms()[0].row(s, 15).to_vec();
        a.step_period();
        let changed = before
            .iter()
            .zip(a.firms()[0].row(s, 15))
            .filter(|(x, y)| x != y)
            .count();
        assert_eq!(changed, 1);
    }

    #[test]
    fn memoryless_firms_live_in_one_state_and_keep_delta() {
        let mut w = world(|c| {
            c.memory = 0;
            c.beta = 2e-4;
            c.window = 2_000;
        });
        w.run(2_000_000);
        let o = w.outcome().unwrap();
        assert_eq!(w.space().states, 1);
        assert!(o.cycle.is_point());
        assert_eq!(w.config.delta, 0.95);
    }

    #[test]
    fn the_authors_rng_seeds_by_session() {
        let a = world(|c| c.rng = RngKind::Calvano);
        let b = CollusionWorld::new(
            CollusionConfig {
                rng: RngKind::Calvano,
                ..CollusionConfig::default()
            },
            1,
        )
        .unwrap();
        assert_eq!(a.state(), b.state());
        let other = CollusionWorld::new(
            CollusionConfig {
                rng: RngKind::Calvano,
                ties: Ties::Random,
                ..CollusionConfig::default()
            },
            2,
        )
        .unwrap();
        // Session 1's first prices are the shared stream's first two draws:
        // ⌊15 · 0.2854⌋ = 4 and ⌊15 · 0.2534⌋ = 3.
        assert_eq!(a.space().last(a.state()), Some(vec![4, 3]));
        // Session 2's are its next two: ⌊15 · 0.0935⌋ = 1, then the fourth.
        assert_eq!(other.space().last(other.state()).unwrap()[0], 1);
    }

    #[test]
    fn boltzmann_and_three_firms_reach_their_pinned_fingerprints() {
        // crates/sugarscape-wasm/tests/web.rs repeats these: Boltzmann choice
        // runs on the portable exp, and no preset uses it.
        let mut b = world(|c| {
            c.exploration = Exploration::Boltzmann;
            c.temperature = 0.01;
            c.cooling = 1e-4;
        });
        b.run(500);
        assert_eq!(b.fingerprint(), 0xffb749a8db1a5481);
        let mut three = world(|c| {
            c.q_init = QInit::Random;
            c.firms = 3;
        });
        three.run(200);
        assert_eq!(three.fingerprint(), 0x0bab95a370db18eb);
    }

    #[test]
    fn inspect_reads_a_strategy_cell_and_live_edits_reanalyze() {
        let mut w = world(|c| {
            c.beta = 2e-4;
            c.window = 2_000;
        });
        w.run(2_000_000);
        let i = w.inspect(0, 0).unwrap();
        assert_eq!((i.panel, i.firm), (Some("strategy"), Some(0)));
        let st = i.state.unwrap();
        assert_eq!(st.q.len(), 2);
        assert_eq!(
            st.prices[0][1],
            w.game().grid[1][14],
            "top row: the rival at the top price"
        );
        let mut next = w.config.clone();
        next.impulse = Impulse::EveryPrice;
        w.set_config(ModelConfig::Collusion(next)).unwrap();
        let o = w.outcome().unwrap();
        assert_eq!(o.responses.len(), o.cycle.len() * 2 * 14);
        let mut bad = w.config.clone();
        bad.memory = 2;
        assert_eq!(
            w.set_config(ModelConfig::Collusion(bad)).unwrap_err()[0].field,
            "memory"
        );
        let mut buf = Vec::new();
        Model::render(&w, "price", "", &mut buf).unwrap();
        Model::render(&w, "visits", "", &mut buf).unwrap();
        assert!(Model::render(&w, "nope", "", &mut buf).is_err());
    }
}
