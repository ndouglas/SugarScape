//! The El Farol and minority game world. Each round every agent decides by its
//! best predictor (El Farol) or strategy (the minority game), attendance is
//! counted, and every predictor or strategy is scored as if it had been used.

use std::collections::VecDeque;
use std::fmt::Write;
use std::sync::Arc;

use rand::Rng;
use serde::Serialize;

use super::config::{
    AtCapacity, Behavior, FarolConfig, Game, Information, Payoff, Rounding, Scoring,
};
use super::predictors::{library, Family, Predictor, LOOKBACK};
use super::stats::{random_fluctuation, FarolSnapshot, WINDOW};
use super::view::{
    attendance_at, gain_color, grid, memory_color, row, BAR, CAPACITY, CROWDED, FAMILIES, GRID_X,
    HIST_W, HIST_X, LINE, SHOWN, STAYED, STEADY, SWITCHED, TALL, TIME_W, WENT,
};
use crate::config::FieldError;
use crate::export;
use crate::model::{wrong_model, Model, ModelConfig, ModelKind};
use crate::opinions::Canvas;
use crate::rng::{self, SimRng};
use crate::stats::{Series, Stats};

/// A strategy table: 2^M bits, true for side A (attend).
pub type Table = Arc<Vec<u64>>;

#[derive(Clone, Debug)]
pub struct Agent {
    /// M (minority game).
    pub memory: u32,
    /// Indexes into the library (El Farol).
    pub predictors: Vec<u8>,
    /// Strategy tables and their virtual points (minority game).
    pub tables: Vec<Table>,
    pub points: Vec<f64>,
    /// The strategy or predictor acted on last (index into its own list).
    pub active: Option<usize>,
    pub went: bool,
    pub switched: bool,
    pub switches: u32,
    /// Winnings so far, and since the last replacement.
    pub gain: f64,
    pub window: f64,
}

/// The frame's color modes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FarolMode {
    Choice,
    Gain,
    Strategy,
    Memory,
}

impl std::str::FromStr for FarolMode {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, String> {
        Ok(match s {
            "choice" => Self::Choice,
            "gain" => Self::Gain,
            "strategy" => Self::Strategy,
            "memory" => Self::Memory,
            _ => return Err(format!("unknown color mode {s:?}")),
        })
    }
}

/// What Inspect shows for a cell.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct FarolInspection {
    pub site: FarolCell,
    /// `time`, `histogram` or `agents`; null between panels.
    pub panel: Option<&'static str>,
    /// The round of a time-panel column.
    pub round: Option<u64>,
    /// The attendance of that round, or the histogram row's.
    pub attendance: Option<u32>,
    /// Rounds with that attendance (histogram).
    pub count: Option<u32>,
    /// The agent at a grid cell.
    pub member: Option<FarolAgent>,
    /// Always null: cells are read where they are.
    pub agent: Option<FarolAgent>,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct FarolCell {
    pub x: u32,
    pub y: u32,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct FarolAgent {
    pub id: u64,
    pub memory: u32,
    pub went: bool,
    pub gain: f64,
    pub switches: u32,
    pub strategies: Vec<StrategyView>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct StrategyView {
    pub label: String,
    /// El Farol: the error (lower is better) or payoff score; minority: points.
    pub score: f64,
    /// El Farol: what it forecasts for next round.
    pub forecast: Option<u32>,
    /// Minority: whether it would attend next round.
    pub attend: Option<bool>,
    pub active: bool,
}

/// A decision made in a real native round, before scores and history update.
#[derive(Clone, Debug, Serialize)]
pub struct FarolDecisionAgent {
    pub id: u64,
    pub memory: u32,
    pub went: Option<bool>,
    pub selected: Option<usize>,
    pub strategies: Vec<StrategyView>,
}

#[derive(Clone, Debug, Serialize)]
pub struct FarolDecision {
    pub tick: u64,
    pub attendance: u32,
    pub capacity: u32,
    pub crowded: Option<bool>,
    pub winning_attend: Option<bool>,
    /// Attendance input before this decision, oldest first (last 12).
    pub attendance_history: Vec<u32>,
    /// The actual minority lookup input, newest winning bit in bit zero.
    pub history_bits: Option<u64>,
    pub agents: Vec<FarolDecisionAgent>,
}

/// The payoff to each of `x` winners among `n`.
pub fn payoff(c: &FarolConfig, n: u32, x: u32) -> f64 {
    match c.payoff {
        Payoff::Step => 1.0,
        Payoff::Inverse if x == 0 => 0.0,
        Payoff::Inverse => {
            let v = f64::from(n) / f64::from(x) - 2.0;
            match c.rounding {
                Rounding::Nearest => v.round(),
                Rounding::Exact => v,
            }
        }
    }
}

/// A table of 2^m entries, each attend with probability `bias`.
fn draw_table(m: u32, bias: f64, rng: &mut SimRng) -> Table {
    let entries = 1usize << m;
    let mut words = vec![0u64; entries.div_ceil(64)];
    for e in 0..entries {
        if rng.gen::<f64>() < bias {
            words[e / 64] |= 1 << (e % 64);
        }
    }
    Arc::new(words)
}

fn entry(t: &Table, mu: usize) -> bool {
    t[mu / 64] >> (mu % 64) & 1 == 1
}

/// A uniform index below `k` (k ≥ 1), drawing only when there is a choice.
fn pick(k: usize, rng: &mut SimRng) -> usize {
    if k == 1 {
        0
    } else {
        rng.gen_range(0..k as u32) as usize
    }
}

#[derive(Clone)]
pub struct FarolWorld {
    pub config: FarolConfig,
    /// Completed rounds.
    pub tick: u64,
    rng: SimRng,
    library: Arc<Vec<Predictor>>,
    /// El Farol: each library predictor's score (the same for every agent
    /// holding it: all read the same history).
    scores: Vec<f64>,
    agents: Vec<Agent>,
    /// The last rounds' attendance (El Farol: after 12 seeded weeks), newest last.
    recent: VecDeque<u32>,
    /// Minority: the winning sides, newest in bit 0 (1: side A won).
    outcomes: u64,
    /// Rounds by attendance, 0..=N.
    counts: Vec<u32>,
    squares: VecDeque<f64>,
    total_gain: f64,
    round: FarolSnapshot,
    pub stats: Stats<FarolSnapshot>,
}

impl FarolWorld {
    pub fn new(config: FarolConfig, seed: u64) -> Result<Self, Vec<FieldError>> {
        config.validate()?;
        let mut rng = rng::seeded(seed);
        let n = config.agents;
        let lib = library();
        let mut recent = VecDeque::new();
        let mut agents = Vec::with_capacity(n as usize);
        match config.game {
            Game::ElFarol => {
                for _ in 0..LOOKBACK {
                    recent.push_back(rng.gen_range(0..=n));
                }
                for _ in 0..n {
                    let predictors: Vec<u8> = if config.shared {
                        (0..lib.len() as u8).collect()
                    } else {
                        // k distinct predictors: a partial Fisher–Yates shuffle.
                        let mut idx: Vec<u8> = (0..lib.len() as u8).collect();
                        for i in 0..config.strategies as usize {
                            let j = i + rng.gen_range(0..(idx.len() - i) as u32) as usize;
                            idx.swap(i, j);
                        }
                        idx.truncate(config.strategies as usize);
                        idx
                    };
                    agents.push(Agent::new(0, predictors, Vec::new()));
                }
            }
            Game::Minority => {
                for _ in 0..n {
                    let m = if config.mixed_memory.enabled {
                        rng.gen_range(config.mixed_memory.min..=config.mixed_memory.max)
                    } else {
                        config.memory
                    };
                    let tables = (0..config.strategies)
                        .map(|_| draw_table(m, config.bias, &mut rng))
                        .collect();
                    agents.push(Agent::new(m, Vec::new(), tables));
                }
            }
        }
        let outcomes = if config.game == Game::Minority {
            rng.gen()
        } else {
            0
        };
        let mut world = FarolWorld {
            scores: vec![0.0; lib.len()],
            library: Arc::new(lib),
            counts: vec![0; n as usize + 1],
            config,
            tick: 0,
            rng,
            agents,
            recent,
            outcomes,
            squares: VecDeque::new(),
            total_gain: 0.0,
            round: FarolSnapshot::default(),
            stats: Stats::default(),
        };
        world.record();
        Ok(world)
    }

    pub fn agents(&self) -> &[Agent] {
        &self.agents
    }

    pub fn is_finished(&self) -> bool {
        self.config.stop_at > 0 && self.tick >= u64::from(self.config.stop_at)
    }

    /// The center fluctuations are measured around: L, or N/2 in the plain
    /// minority game.
    fn center(&self) -> f64 {
        if self.config.plain_minority() {
            f64::from(self.config.agents) / 2.0
        } else {
            f64::from(self.config.capacity())
        }
    }

    /// How often a random agent attends: L/N, or a fair coin in the plain
    /// minority game (Challet and Zhang's and Savit's random guessing).
    fn random_attendance(&self) -> f64 {
        if self.config.plain_minority() {
            0.5
        } else {
            f64::from(self.config.capacity()) / f64::from(self.config.agents)
        }
    }

    /// Each library predictor's forecast for next round.
    fn forecasts(&self) -> Vec<u32> {
        let h: Vec<u32> = self
            .recent
            .iter()
            .rev()
            .take(LOOKBACK)
            .rev()
            .copied()
            .collect();
        self.library
            .iter()
            .map(|p| p.forecast(&h, self.config.agents))
            .collect()
    }

    /// The agent's best predictor (index into its own list), ties at random.
    fn best_predictor(&mut self, i: usize) -> usize {
        let a = &self.agents[i];
        let better = |x: f64, y: f64| match self.config.scoring {
            Scoring::Error => x < y,
            Scoring::Payoff => x > y,
        };
        let mut best: Vec<usize> = Vec::new();
        let mut value = 0.0;
        for (k, &p) in a.predictors.iter().enumerate() {
            let s = self.scores[p as usize];
            if best.is_empty() || better(s, value) {
                best.clear();
                best.push(k);
                value = s;
            } else if s == value {
                best.push(k);
            }
        }
        best[pick(best.len(), &mut self.rng)]
    }

    fn best_strategy(&mut self, i: usize) -> usize {
        let a = &self.agents[i];
        let top = a.points.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        let best: Vec<usize> = (0..a.points.len())
            .filter(|&k| a.points[k] == top)
            .collect();
        best[pick(best.len(), &mut self.rng)]
    }

    fn set_active(&mut self, i: usize, k: usize) {
        let a = &mut self.agents[i];
        a.switched = a.active != Some(k);
        if a.switched {
            a.switches += 1;
        }
        a.active = Some(k);
    }

    /// One round.
    pub fn step(&mut self) {
        self.step_inner(false);
    }

    /// Film one actual step without drawing RNG or selecting strategies again.
    pub fn step_recorded(&mut self) -> FarolDecision {
        self.step_inner(true).expect("recording requested")
    }

    fn decision(&self, forecasts: &[u32], history: Option<u64>, played: bool) -> FarolDecision {
        let agents = self
            .agents
            .iter()
            .enumerate()
            .map(|(i, a)| {
                let selected = if played && self.config.behavior != Behavior::Random {
                    a.active
                } else {
                    None
                };
                let strategies = match self.config.game {
                    Game::ElFarol => a
                        .predictors
                        .iter()
                        .enumerate()
                        .map(|(k, &p)| StrategyView {
                            label: self.library[p as usize].describe(),
                            score: self.scores[p as usize],
                            forecast: Some(forecasts[p as usize]),
                            attend: None,
                            active: selected == Some(k),
                        })
                        .collect(),
                    Game::Minority => {
                        let mu =
                            (history.unwrap_or(self.outcomes) & ((1u64 << a.memory) - 1)) as usize;
                        a.tables
                            .iter()
                            .zip(&a.points)
                            .enumerate()
                            .map(|(k, (t, &score))| StrategyView {
                                label: format!("strategy {}", k + 1),
                                score,
                                forecast: None,
                                attend: history.map(|_| entry(t, mu)),
                                active: selected == Some(k),
                            })
                            .collect()
                    }
                };
                FarolDecisionAgent {
                    id: i as u64 + 1,
                    memory: a.memory,
                    went: played.then_some(a.went),
                    selected,
                    strategies,
                }
            })
            .collect();
        let attendance = if played {
            self.agents.iter().filter(|a| a.went).count() as u32
        } else {
            0
        };
        let crowded = match self.config.game {
            Game::ElFarol => attendance >= self.config.capacity(),
            Game::Minority => attendance > self.config.capacity(),
        };
        FarolDecision {
            tick: self.tick + u64::from(played),
            attendance,
            capacity: self.config.capacity(),
            crowded: played.then_some(crowded),
            winning_attend: played.then_some(!crowded),
            attendance_history: self
                .recent
                .iter()
                .rev()
                .take(LOOKBACK)
                .rev()
                .copied()
                .collect(),
            history_bits: history,
            agents,
        }
    }

    pub fn initial_decision(&self) -> FarolDecision {
        let forecasts = if self.config.game == Game::ElFarol {
            self.forecasts()
        } else {
            Vec::new()
        };
        self.decision(&forecasts, None, false)
    }

    fn step_inner(&mut self, recording: bool) -> Option<FarolDecision> {
        let mut decision = None;
        let n = self.config.agents;
        let p_random = self.random_attendance();
        let random = self.config.behavior == Behavior::Random;
        let (attendance, success, above) = match self.config.game {
            Game::ElFarol => {
                let forecasts = self.forecasts();
                let cap = self.config.capacity();
                let mut above = 0u32;
                for i in 0..n as usize {
                    let went = if random {
                        self.agents[i].switched = false;
                        self.rng.gen::<f64>() < p_random
                    } else {
                        let k = self.best_predictor(i);
                        self.set_active(i, k);
                        let f = forecasts[self.agents[i].predictors[k] as usize];
                        if f > cap {
                            above += 1;
                        }
                        match self.config.at_capacity {
                            AtCapacity::Stay => f < cap,
                            AtCapacity::Go => f <= cap,
                        }
                    };
                    self.agents[i].went = went;
                }
                let a = self.agents.iter().filter(|a| a.went).count() as u32;
                if recording {
                    decision = Some(self.decision(&forecasts, None, true));
                }
                let crowded = a >= cap;
                let mut right = 0u32;
                for ag in &mut self.agents {
                    if ag.went != crowded {
                        right += 1;
                    }
                    if ag.went && !crowded {
                        ag.gain += 1.0;
                        ag.window += 1.0;
                        self.total_gain += 1.0;
                    }
                }
                for (p, s) in self.scores.iter_mut().enumerate() {
                    let f = forecasts[p];
                    match self.config.scoring {
                        Scoring::Error => {
                            let d = self.config.decay;
                            *s = d * *s + (1.0 - d) * f64::from(f.abs_diff(a));
                        }
                        Scoring::Payoff => {
                            let predicts_crowding = match self.config.at_capacity {
                                AtCapacity::Stay => f >= cap,
                                AtCapacity::Go => f > cap,
                            };
                            if predicts_crowding == crowded {
                                *s += 1.0;
                            }
                        }
                    }
                }
                let above = if random {
                    0.0
                } else {
                    f64::from(above) / f64::from(n)
                };
                (a, right, above)
            }
            Game::Minority => {
                let history = match self.config.information {
                    Information::True => self.outcomes,
                    Information::Random => self.rng.gen(),
                };
                for i in 0..n as usize {
                    let went = if random {
                        self.agents[i].switched = false;
                        self.rng.gen::<f64>() < p_random
                    } else {
                        let k = self.best_strategy(i);
                        self.set_active(i, k);
                        let a = &self.agents[i];
                        let mu = (history & ((1u64 << a.memory) - 1)) as usize;
                        entry(&a.tables[k], mu)
                    };
                    self.agents[i].went = went;
                }
                let a = self.agents.iter().filter(|a| a.went).count() as u32;
                if recording {
                    decision = Some(self.decision(&[], Some(history), true));
                }
                let a_wins = a <= self.config.capacity();
                let winners = if a_wins { a } else { n - a };
                let pay = payoff(&self.config, n, winners);
                for ag in &mut self.agents {
                    if ag.went == a_wins {
                        ag.gain += pay;
                        ag.window += pay;
                        self.total_gain += pay;
                    }
                    let mu = (history & ((1u64 << ag.memory) - 1)) as usize;
                    for (t, pts) in ag.tables.iter().zip(ag.points.iter_mut()) {
                        if entry(t, mu) == a_wins {
                            *pts += pay;
                        }
                    }
                }
                self.outcomes = (self.outcomes << 1) | u64::from(a_wins);
                (a, winners, 0.0)
            }
        };
        self.tick += 1;
        if self.config.game == Game::Minority
            && self.config.evolution.enabled
            && self
                .tick
                .is_multiple_of(u64::from(self.config.evolution.every))
        {
            self.evolve();
        }
        let switched = self.agents.iter().filter(|a| a.switched).count() as u32;
        self.recent.push_back(attendance);
        while self.recent.len() > SHOWN + LOOKBACK {
            self.recent.pop_front();
        }
        self.counts[attendance as usize] += 1;
        let d = f64::from(attendance) - self.center();
        self.squares.push_back(d * d);
        if self.squares.len() > WINDOW {
            self.squares.pop_front();
        }
        let crowded = match self.config.game {
            Game::ElFarol => attendance >= self.config.capacity(),
            Game::Minority => attendance > self.config.capacity(),
        };
        let nf = f64::from(n);
        self.round = FarolSnapshot {
            tick: self.tick,
            attendance,
            crowded: u32::from(crowded),
            success: f64::from(success) / nf,
            switching: f64::from(switched) / nf,
            forecast_above: above,
            ..FarolSnapshot::default()
        };
        self.record();
        decision
    }

    /// Challet and Zhang's Darwinism: the worst of the last `every` rounds is
    /// replaced by a copy of the best, its scores reset, perhaps mutated.
    fn evolve(&mut self) {
        let ev = self.config.evolution.clone();
        let by = |f: fn(f64, f64) -> bool, agents: &[Agent]| {
            let mut k = 0;
            for (i, a) in agents.iter().enumerate() {
                if f(a.window, agents[k].window) {
                    k = i;
                }
            }
            k
        };
        let worst = by(|a, b| a < b, &self.agents);
        let best = by(|a, b| a > b, &self.agents);
        if worst != best {
            let mut m = self.agents[best].memory;
            let mut tables = self.agents[best].tables.clone();
            if self.rng.gen::<f64>() < ev.memory_mutation {
                let up = self.rng.gen::<bool>();
                let next = if up { m + 1 } else { m.saturating_sub(1) };
                if (1..=self.config.memory_ceiling()).contains(&next) {
                    m = next;
                    tables = (0..tables.len())
                        .map(|_| draw_table(m, self.config.bias, &mut self.rng))
                        .collect();
                }
            }
            if self.rng.gen::<f64>() < ev.strategy_mutation {
                let k = pick(tables.len(), &mut self.rng);
                tables[k] = draw_table(m, self.config.bias, &mut self.rng);
            }
            let s = tables.len();
            self.agents[worst] = Agent {
                memory: m,
                tables,
                points: vec![0.0; s],
                ..Agent::new(m, Vec::new(), Vec::new())
            };
        }
        for a in &mut self.agents {
            a.window = 0.0;
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

    fn record(&mut self) {
        let n = self.config.agents;
        let nf = f64::from(n);
        let p = self.random_attendance();
        let mut s = self.round.clone();
        s.tick = self.tick;
        s.fluctuation = if self.squares.is_empty() {
            0.0
        } else {
            self.squares.iter().sum::<f64>() / self.squares.len() as f64 / nf
        };
        s.random_fluctuation = random_fluctuation(n, p, self.center());
        s.mean_gain = if self.tick == 0 {
            0.0
        } else {
            self.total_gain / nf / self.tick as f64
        };
        s.mean_memory = self.agents.iter().map(|a| f64::from(a.memory)).sum::<f64>() / nf;
        self.stats.push(s);
    }

    /// The rounds shown, oldest first (El Farol's seeded weeks excluded).
    fn shown(&self) -> Vec<u32> {
        let played = (self.tick as usize).min(SHOWN);
        self.recent
            .iter()
            .skip(self.recent.len() - played)
            .copied()
            .collect()
    }

    fn color(&self, mode: FarolMode, i: usize, top: f64) -> [u8; 3] {
        let a = &self.agents[i];
        match mode {
            FarolMode::Choice => {
                if a.went {
                    WENT
                } else {
                    STAYED
                }
            }
            FarolMode::Gain => gain_color(a.gain, top),
            FarolMode::Strategy => match (self.config.game, a.active) {
                (Game::ElFarol, Some(k)) => {
                    let f = self.library[a.predictors[k] as usize].family();
                    FAMILIES[match f {
                        Family::Same => 0,
                        Family::Mirror => 1,
                        Family::Mean => 2,
                        Family::Trend => 3,
                    }]
                }
                (Game::Minority, Some(_)) if a.switched => SWITCHED,
                _ => STEADY,
            },
            FarolMode::Memory => memory_color(a.memory),
        }
    }

    fn view(&self, i: usize) -> FarolAgent {
        let a = &self.agents[i];
        let strategies = match self.config.game {
            Game::ElFarol => {
                let f = self.forecasts();
                a.predictors
                    .iter()
                    .enumerate()
                    .map(|(k, &p)| StrategyView {
                        label: self.library[p as usize].describe(),
                        score: self.scores[p as usize],
                        forecast: Some(f[p as usize]),
                        attend: None,
                        active: a.active == Some(k),
                    })
                    .collect()
            }
            Game::Minority => {
                let mu = (self.outcomes & ((1u64 << a.memory) - 1)) as usize;
                a.tables
                    .iter()
                    .zip(&a.points)
                    .enumerate()
                    .map(|(k, (t, &pts))| StrategyView {
                        label: format!("strategy {}", k + 1),
                        score: pts,
                        forecast: None,
                        attend: Some(entry(t, mu)),
                        active: a.active == Some(k),
                    })
                    .collect()
            }
        };
        FarolAgent {
            id: i as u64 + 1,
            memory: a.memory,
            went: a.went,
            gain: a.gain,
            switches: a.switches,
            strategies,
        }
    }

    pub fn inspect(&self, x: u32, y: u32) -> Result<FarolInspection, String> {
        let (fw, fh) = Model::size(self);
        if x >= fw || y >= fh {
            return Err(format!("({x}, {y}) is not in the frame"));
        }
        let (cx, cy) = (x as usize, y as usize);
        let n = self.config.agents;
        let mut out = FarolInspection {
            site: FarolCell { x, y },
            panel: None,
            round: None,
            attendance: None,
            count: None,
            member: None,
            agent: None,
        };
        let shown = self.shown();
        if cx < TIME_W {
            if let Some(&a) = shown.get(cx) {
                out.panel = Some("time");
                out.round = Some(self.tick - shown.len() as u64 + cx as u64 + 1);
                out.attendance = Some(a);
            }
        } else if (HIST_X..HIST_X + HIST_W).contains(&cx) {
            let a = attendance_at(cy, n);
            out.panel = Some("histogram");
            out.attendance = Some(a);
            out.count = Some(
                (0..=n)
                    .filter(|&v| row(v, n) == cy)
                    .map(|v| self.counts[v as usize])
                    .sum(),
            );
        } else if cx >= GRID_X {
            let (side, cell) = grid(n);
            let (gx, gy) = ((cx - GRID_X) / cell, cy / cell);
            let i = gy * side + gx;
            if gx < side && i < n as usize {
                out.panel = Some("agents");
                out.member = Some(self.view(i));
            }
        }
        Ok(out)
    }
}

impl Agent {
    fn new(memory: u32, predictors: Vec<u8>, tables: Vec<Table>) -> Self {
        let points = vec![0.0; tables.len()];
        Agent {
            memory,
            predictors,
            tables,
            points,
            active: None,
            went: false,
            switched: false,
            switches: 0,
            gain: 0.0,
            window: 0.0,
        }
    }
}

impl Model for FarolWorld {
    fn config(&self) -> ModelConfig {
        ModelConfig::Farol(self.config.clone())
    }

    fn run(&mut self, ticks: u32) {
        FarolWorld::run(self, ticks);
    }

    fn tick(&self) -> u64 {
        self.tick
    }

    fn population(&self) -> usize {
        self.agents.len()
    }

    /// FNV-1a over the tick, the history, the scores and every agent's state.
    fn fingerprint(&self) -> u64 {
        let mut h: u64 = 0xcbf2_9ce4_8422_2325;
        let mut eat = |bytes: &[u8]| {
            for &b in bytes {
                h ^= u64::from(b);
                h = h.wrapping_mul(0x0100_0000_01b3);
            }
        };
        eat(&self.tick.to_le_bytes());
        eat(&self.outcomes.to_le_bytes());
        for a in &self.recent {
            eat(&a.to_le_bytes());
        }
        for s in &self.scores {
            eat(&s.to_bits().to_le_bytes());
        }
        for a in &self.agents {
            eat(&[u8::from(a.went), a.active.map_or(255, |k| k as u8)]);
            eat(&a.memory.to_le_bytes());
            eat(&a.gain.to_bits().to_le_bytes());
            for p in &a.points {
                eat(&p.to_bits().to_le_bytes());
            }
            for t in &a.tables {
                for w in t.iter() {
                    eat(&w.to_le_bytes());
                }
            }
        }
        h
    }

    fn size(&self) -> (u32, u32) {
        let (side, cell) = grid(self.config.agents);
        ((GRID_X + side * cell) as u32, TALL as u32)
    }

    fn render(&self, mode: &str, _layer: &str, buf: &mut Vec<u8>) -> Result<(), String> {
        let mode: FarolMode = mode.parse()?;
        let (fw, fh) = Model::size(self);
        let mut c = Canvas { buf, wide: 0 };
        c.clear(fw as usize, fh as usize);
        let n = self.config.agents;
        let cap_row = row(self.config.capacity(), n);
        let shown = self.shown();
        for (k, &a) in shown.iter().enumerate() {
            let crowded = match self.config.game {
                Game::ElFarol => a >= self.config.capacity(),
                Game::Minority => a > self.config.capacity(),
            };
            if crowded {
                c.column(k, 0, TALL - 1, CROWDED);
            }
        }
        for x in 0..TIME_W {
            c.put(x, cap_row, CAPACITY);
        }
        for (k, &a) in shown.iter().enumerate() {
            let to = shown.get(k + 1).map_or(row(a, n), |&b| row(b, n));
            c.column(k, row(a, n), to, LINE);
        }
        // The histogram on the same vertical scale: a bar per row.
        let mut per_row = vec![0u32; TALL];
        for v in 0..=n {
            per_row[row(v, n)] += self.counts[v as usize];
        }
        let top = per_row.iter().copied().max().unwrap_or(0).max(1);
        for x in 0..HIST_W {
            c.put(HIST_X + x, cap_row, CAPACITY);
        }
        for (y, &k) in per_row.iter().enumerate() {
            let len = (f64::from(k) / f64::from(top) * (HIST_W - 1) as f64).round() as usize;
            for x in 0..len {
                c.put(HIST_X + x, y, BAR);
            }
        }
        let (side, cell) = grid(n);
        let rich = self.agents.iter().map(|a| a.gain).fold(0.0, f64::max);
        for i in 0..self.agents.len() {
            let color = self.color(mode, i, rich);
            let (gx, gy) = (i % side, i / side);
            for dy in 0..cell {
                for dx in 0..cell {
                    c.put(GRID_X + gx * cell + dx, gy * cell + dy, color);
                }
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

    fn agents_csv(&self) -> String {
        let mut out = String::from("id,memory,went,gain,switches,active\n");
        for (i, a) in self.agents.iter().enumerate() {
            let active = match (self.config.game, a.active) {
                (Game::ElFarol, Some(k)) => self.library[a.predictors[k] as usize].describe(),
                (Game::Minority, Some(k)) => format!("strategy {}", k + 1),
                (_, None) => String::new(),
            };
            writeln!(
                out,
                "{},{},{},{},{},{}",
                i + 1,
                a.memory,
                u8::from(a.went),
                a.gain,
                a.switches,
                active
            )
            .unwrap();
        }
        out
    }

    fn inspect_json(&self, x: u32, y: u32) -> Result<String, String> {
        let inspection = self.inspect(x, y)?;
        Ok(serde_json::to_string(&inspection).expect("inspection serializes"))
    }

    /// Nothing to follow: Inspect reads a cell.
    fn locate(&self, _id: u64) -> Option<(u32, u32)> {
        None
    }

    fn set_config(&mut self, next: ModelConfig) -> Result<(), Vec<FieldError>> {
        let ModelConfig::Farol(next) = next else {
            return Err(wrong_model(ModelKind::Farol, &next));
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

    /// Stopped at `stop_at`: a sweep reads the run at its last round.
    fn holds_when_finished(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::farol::config::{Evolution, MixedMemory};

    fn config(edit: impl FnOnce(&mut FarolConfig)) -> FarolConfig {
        let mut c = FarolConfig::default();
        edit(&mut c);
        c
    }

    fn world(edit: impl FnOnce(&mut FarolConfig)) -> FarolWorld {
        FarolWorld::new(config(edit), 1).unwrap()
    }

    fn minority(n: u32, s: u32, m: u32) -> impl FnOnce(&mut FarolConfig) {
        move |c| {
            c.game = Game::Minority;
            c.agents = n;
            c.strategies = s;
            c.memory = m;
            c.capacity = None;
        }
    }

    #[test]
    fn el_farol_agents_hold_distinct_predictors_or_the_whole_library() {
        let w = world(|_| {});
        assert_eq!(w.recent.len(), LOOKBACK, "twelve seeded weeks");
        for a in w.agents() {
            assert_eq!(a.predictors.len(), 12);
            let mut p = a.predictors.clone();
            p.sort_unstable();
            p.dedup();
            assert_eq!(p.len(), 12);
        }
        let s = world(|c| c.shared = true);
        assert!(s.agents().iter().all(|a| a.predictors.len() == 48));
    }

    #[test]
    fn a_round_goes_by_the_best_predictor_and_rescores_them_all() {
        let mut w = world(|c| c.decay = 0.5);
        let f = w.forecasts();
        w.step();
        let a = w.round.attendance;
        // Every agent acted on its predictor with the lowest error (all 0 at
        // first: a random one) and went when it forecast below 60.
        for ag in w.agents() {
            let p = ag.predictors[ag.active.unwrap()] as usize;
            assert_eq!(ag.went, f[p] < 60);
        }
        assert_eq!(a, w.agents().iter().filter(|x| x.went).count() as u32);
        for (p, s) in w.scores.iter().enumerate() {
            assert_eq!(*s, 0.5 * f64::from(f[p].abs_diff(a)));
        }
        let before = w.scores.clone();
        w.step();
        // The second round's choice is the lowest error going into it.
        for ag in w.agents() {
            let chosen = before[ag.predictors[ag.active.unwrap()] as usize];
            assert!(ag.predictors.iter().all(|&p| before[p as usize] >= chosen));
        }
    }

    #[test]
    fn crowding_starts_at_the_capacity_and_forecasts_of_exactly_l_stay_home() {
        let three = |c: &mut FarolConfig| {
            c.agents = 3;
            c.capacity = Some(2);
        };
        let mut w = world(three);
        w.recent = VecDeque::from(vec![2; LOOKBACK]);
        // Every predictor that repeats a past week forecasts exactly 2.
        for a in &mut w.agents {
            a.predictors = vec![0];
        }
        w.step();
        assert_eq!(w.round.attendance, 0, "a forecast of exactly L stays home");
        let mut g = world(three);
        g.config.at_capacity = AtCapacity::Go;
        g.recent = VecDeque::from(vec![2; LOOKBACK]);
        for a in &mut g.agents {
            a.predictors = vec![0];
        }
        g.step();
        assert_eq!(g.round.attendance, 3);
        assert_eq!(g.round.crowded, 1, "A ≥ L is crowded");
        assert_eq!(g.round.success, 0.0);
    }

    #[test]
    fn payoff_scoring_rewards_the_right_advice() {
        let mut w = world(|c| c.scoring = Scoring::Payoff);
        let f = w.forecasts();
        w.step();
        let crowded = w.round.attendance >= 60;
        for (p, s) in w.scores.iter().enumerate() {
            assert_eq!(*s, f64::from(u8::from((f[p] >= 60) == crowded)));
        }
    }

    #[test]
    fn payoff_scoring_does_not_reward_equal_forecasts_that_advise_go_into_a_crowd() {
        let mut w = world(|c| {
            c.agents = 3;
            c.capacity = Some(2);
            c.scoring = Scoring::Payoff;
            c.at_capacity = AtCapacity::Go;
        });
        w.recent = VecDeque::from(vec![2; LOOKBACK]);
        for a in &mut w.agents {
            a.predictors = vec![0];
        }
        w.step();
        assert_eq!(w.round.attendance, 3);
        assert_eq!(w.round.crowded, 1);
        assert_eq!(w.scores[0], 0.0);
    }

    #[test]
    fn payoff_scoring_does_not_reward_equal_forecasts_that_advise_stay_when_uncrowded() {
        let mut w = world(|c| {
            c.agents = 3;
            c.capacity = Some(2);
            c.scoring = Scoring::Payoff;
        });
        w.recent = VecDeque::from(vec![2; LOOKBACK]);
        for a in &mut w.agents {
            a.predictors = vec![0];
        }
        w.step();
        assert_eq!(w.scores[0], 0.0);
    }

    #[test]
    fn minority_agents_play_their_best_table_and_score_every_one() {
        let mut w = world(minority(11, 3, 2));
        let before = w.outcomes;
        w.step();
        let a = w.round.attendance;
        let a_wins = a <= 5;
        assert_eq!(w.outcomes, (before << 1) | u64::from(a_wins));
        let mu = (before & 3) as usize;
        for ag in w.agents() {
            assert_eq!(ag.went, entry(&ag.tables[ag.active.unwrap()], mu));
            for (t, &pts) in ag.tables.iter().zip(&ag.points) {
                assert_eq!(pts, f64::from(u8::from(entry(t, mu) == a_wins)));
            }
            assert_eq!(ag.gain, f64::from(u8::from(ag.went == a_wins)));
        }
        assert_eq!(
            w.round.success,
            f64::from(if a_wins { a } else { 11 - a }) / 11.0
        );
    }

    #[test]
    fn tables_follow_the_bias() {
        let all = world(|c| {
            minority(101, 2, 6)(c);
            c.bias = 1.0;
        });
        assert!(all
            .agents()
            .iter()
            .all(|a| a.tables.iter().all(|t| t[0] == u64::MAX)));
        let none = world(|c| {
            minority(101, 2, 6)(c);
            c.bias = 0.0;
        });
        assert!(none
            .agents()
            .iter()
            .all(|a| a.tables.iter().all(|t| t[0] == 0)));
    }

    #[test]
    fn the_inverse_payoff_rounds_or_not() {
        let mut c = config(minority(1001, 5, 4));
        c.payoff = Payoff::Inverse;
        assert_eq!(payoff(&c, 1001, 500), 0.0, "1001/500 − 2 rounds to 0");
        assert_eq!(payoff(&c, 1001, 200), 3.0);
        c.rounding = Rounding::Exact;
        assert!((payoff(&c, 1001, 500) - 0.002).abs() < 1e-12);
        assert_eq!(payoff(&c, 1001, 0), 0.0);
        c.payoff = Payoff::Step;
        assert_eq!(payoff(&c, 1001, 500), 1.0);
    }

    #[test]
    fn mixed_memories_read_their_own_bits() {
        let w = world(|c| {
            minority(101, 2, 3)(c);
            c.mixed_memory = MixedMemory {
                enabled: true,
                min: 1,
                max: 10,
            };
        });
        let mems: Vec<u32> = w.agents().iter().map(|a| a.memory).collect();
        assert!(mems.iter().all(|&m| (1..=10).contains(&m)));
        assert!(mems.contains(&1) && mems.contains(&10));
        for a in w.agents() {
            assert!(a
                .tables
                .iter()
                .all(|t| t.len() == (1usize << a.memory).div_ceil(64)));
        }
        assert!((w.stats.latest().unwrap().mean_memory - 5.5).abs() < 1.5);
    }

    #[test]
    fn random_information_ignores_the_history() {
        let mut a = world(minority(101, 2, 3));
        let mut b = world(|c| {
            minority(101, 2, 3)(c);
            c.information = Information::Random;
        });
        a.run(50);
        b.run(50);
        assert_ne!(Model::fingerprint(&a), Model::fingerprint(&b));
        assert!(b.stats.latest().unwrap().success > 0.0);
    }

    #[test]
    fn random_agents_attend_with_probability_l_over_n() {
        let mut w = world(|c| c.behavior = Behavior::Random);
        w.run(2000);
        let s = w.series("attendance").unwrap();
        let mean = s[1..].iter().sum::<f64>() / 2000.0;
        assert!((mean - 60.0).abs() < 1.0, "{mean}");
        let f = w.stats.latest().unwrap();
        assert!((f.fluctuation - f.random_fluctuation).abs() < 0.1, "{f:?}");
        assert!(w.agents().iter().all(|a| a.active.is_none()));
    }

    #[test]
    fn evolution_replaces_the_worst_by_a_copy_of_the_best() {
        let mut w = world(|c| {
            minority(21, 3, 2)(c);
            c.evolution = Evolution {
                enabled: true,
                every: 10,
                strategy_mutation: 0.0,
                memory_mutation: 0.0,
            };
        });
        w.run(9);
        // Ties go to the lower index.
        let window: Vec<f64> = w.agents().iter().map(|a| a.window).collect();
        let worst = (0..21).fold(0, |k, i| if window[i] < window[k] { i } else { k });
        let best = (0..21).fold(0, |k, i| if window[i] > window[k] { i } else { k });
        assert_ne!(worst, best);
        let (tables, memory) = (w.agents()[best].tables.clone(), w.agents()[best].memory);
        w.evolve();
        let a = &w.agents()[worst];
        assert_eq!(
            (&a.tables, a.memory),
            (&tables, memory),
            "a copy of the best"
        );
        assert!(a.points.iter().all(|&p| p == 0.0) && a.gain == 0.0 && a.active.is_none());
        assert!(w.agents().iter().all(|a| a.window == 0.0));
        // In the run, replacement comes every tenth round.
        let mut r = world(|c| {
            minority(21, 3, 2)(c);
            c.evolution.enabled = true;
            c.evolution.every = 10;
        });
        r.run(10);
        assert!(r.agents().iter().all(|a| a.window == 0.0));
    }

    #[test]
    fn memory_mutation_moves_memory_within_bounds() {
        let mut w = world(|c| {
            minority(21, 2, 1)(c);
            c.evolution = Evolution {
                enabled: true,
                every: 1,
                strategy_mutation: 1.0,
                memory_mutation: 1.0,
            };
        });
        w.run(400);
        let mems: Vec<u32> = w.agents().iter().map(|a| a.memory).collect();
        assert!(mems.iter().all(|&m| (1..=16).contains(&m)));
        assert!(mems.iter().any(|&m| m > 1), "{mems:?}");
        for a in w.agents() {
            assert!(a
                .tables
                .iter()
                .all(|t| t.len() == (1usize << a.memory).div_ceil(64)));
        }
    }

    #[test]
    fn statistics_track_fluctuation_success_and_switching() {
        let mut w = world(minority(101, 2, 6));
        w.run(300);
        let s = w.stats.latest().unwrap();
        let att = w.series("attendance").unwrap();
        let want = att[att.len() - WINDOW..]
            .iter()
            .map(|a| (a - 50.5).powi(2))
            .sum::<f64>()
            / WINDOW as f64
            / 101.0;
        assert!((s.fluctuation - want).abs() < 1e-9);
        assert!((s.random_fluctuation - 0.25).abs() < 1e-3);
        assert!((0.0..=1.0).contains(&s.switching));
        assert!(s.mean_gain > 0.3 && s.mean_gain < 0.5, "{}", s.mean_gain);
        assert_eq!(s.forecast_above, 0.0);
        let mut e = world(|_| {});
        e.run(20);
        assert!(e.stats.latest().unwrap().forecast_above > 0.0);
    }

    #[test]
    fn the_frame_draws_time_histogram_and_agents() {
        let mut w = world(|_| {});
        w.run(30);
        let mut buf = Vec::new();
        w.render("choice", "", &mut buf).unwrap();
        let (fw, fh) = Model::size(&w);
        assert_eq!((fw as usize, fh as usize), (GRID_X + 200, TALL));
        let px = |b: &[u8], x: usize, y: usize| {
            let k = (y * fw as usize + x) * 4;
            [b[k], b[k + 1], b[k + 2]]
        };
        let last = *w.shown().last().unwrap();
        assert_eq!(px(&buf, 29, row(last, 100)), LINE);
        assert_eq!(
            px(&buf, GRID_X, 0),
            if w.agents()[0].went { WENT } else { STAYED }
        );
        for mode in ["gain", "strategy", "memory"] {
            w.render(mode, "", &mut buf).unwrap();
        }
        assert!(w.render("wealth", "", &mut buf).is_err());
    }

    #[test]
    fn inspect_reads_rounds_histogram_rows_and_agents() {
        let mut w = world(|_| {});
        w.run(5);
        let v = w.inspect(4, 0).unwrap();
        assert_eq!((v.panel, v.round), (Some("time"), Some(5)));
        assert_eq!(v.attendance, Some(w.round.attendance));
        assert!(w.inspect(5, 0).unwrap().panel.is_none(), "no round yet");
        let a = w.round.attendance;
        let h = w.inspect(HIST_X as u32, row(a, 100) as u32).unwrap();
        assert_eq!(h.panel, Some("histogram"));
        assert!(h.count.unwrap() >= 1);
        let g = w.inspect(GRID_X as u32, 0).unwrap();
        let m = g.member.unwrap();
        assert_eq!((m.id, m.strategies.len()), (1, 12));
        assert_eq!(m.strategies.iter().filter(|s| s.active).count(), 1);
        assert!(g.agent.is_none());
        let mut mg = world(minority(101, 2, 3));
        mg.run(3);
        let g = mg.inspect(GRID_X as u32, 0).unwrap().member.unwrap();
        assert!(g.strategies.iter().all(|s| s.attend.is_some()));
        assert_eq!(Model::locate(&mg, 1), None);
    }

    #[test]
    fn keyframes_restore_the_world_and_its_view() {
        let mut any =
            crate::model::ModelWorld::new(ModelConfig::Farol(config(minority(101, 2, 3))), 6)
                .unwrap();
        any.model_mut().run(20);
        let cp = any.checkpoint().unwrap();
        let print = any.model().fingerprint();
        let mut before = Vec::new();
        any.model().render("gain", "", &mut before).unwrap();
        any.model_mut().run(30);
        any.restore(&cp).unwrap();
        assert_eq!(any.model().fingerprint(), print);
        let mut after = Vec::new();
        any.model().render("gain", "", &mut after).unwrap();
        assert_eq!(before, after);
        any.model_mut().run(30);
        let mut fresh =
            crate::model::ModelWorld::new(ModelConfig::Farol(config(minority(101, 2, 3))), 6)
                .unwrap();
        fresh.model_mut().run(50);
        assert_eq!(
            any.model().fingerprint(),
            fresh.model().fingerprint(),
            "replays the same"
        );
    }

    #[test]
    fn live_edits_apply_and_the_population_waits_for_reset() {
        let mut w = world(|_| {});
        let next = config(|c| {
            c.at_capacity = AtCapacity::Go;
            c.behavior = Behavior::Random;
            c.stop_at = 10;
        });
        Model::set_config(&mut w, ModelConfig::Farol(next)).unwrap();
        w.run(100);
        assert_eq!(w.tick, 10);
        for (field, edit) in [
            ("agents", config(|c| c.agents = 101)),
            ("game", config(|c| c.game = Game::Minority)),
            ("strategies", config(|c| c.strategies = 6)),
            // Error scores and payoff points share one tally: a switch
            // mid-run would act on the least accurate predictor.
            ("scoring", config(|c| c.scoring = Scoring::Payoff)),
        ] {
            let e = Model::set_config(&mut w, ModelConfig::Farol(edit)).unwrap_err();
            assert_eq!(e[0].field, field);
        }
    }

    #[test]
    fn degenerate_configs_run_without_panicking() {
        for c in [
            config(|c| {
                c.agents = 3;
                c.capacity = Some(1);
            }),
            config(|c| c.strategies = 1),
            config(|c| c.strategies = 48),
            config(|c| c.capacity = Some(0)),
            config(|c| c.capacity = Some(100)),
            config(|c| c.decay = 0.0),
            config(|c| c.decay = 1.0),
            config(minority(3, 1, 1)),
            config(|c| {
                minority(101, 16, 12)(c);
                c.payoff = Payoff::Inverse;
                c.rounding = Rounding::Exact;
            }),
            config(|c| {
                minority(101, 2, 3)(c);
                c.capacity = Some(0);
            }),
            config(|c| {
                minority(101, 2, 3)(c);
                c.behavior = Behavior::Random;
                c.evolution.enabled = true;
            }),
        ] {
            let mut w = FarolWorld::new(c.clone(), 1).unwrap();
            w.run(120);
            let s = w.stats.latest().unwrap();
            assert!(s.attendance <= c.agents, "{c:?}");
            assert!(
                s.fluctuation.is_finite() && s.mean_gain.is_finite(),
                "{c:?}"
            );
        }
    }
    #[test]
    fn recorded_decisions_keep_pre_update_scores_history_and_capacity_boundary() {
        for (at_capacity, expected_go) in [(AtCapacity::Stay, false), (AtCapacity::Go, true)] {
            let mut w = FarolWorld::new(
                FarolConfig {
                    agents: 3,
                    capacity: Some(1),
                    strategies: 1,
                    at_capacity,
                    ..FarolConfig::default()
                },
                9,
            )
            .unwrap();
            w.recent = VecDeque::from(vec![1; LOOKBACK]);
            for a in &mut w.agents {
                a.predictors = vec![0];
            }
            w.scores[0] = 7.0;
            let f = w.step_recorded();
            assert_eq!(f.attendance_history, vec![1; 12]);
            assert_eq!(f.agents[0].strategies[0].score, 7.0);
            assert_eq!(f.agents[0].strategies[0].forecast, Some(1));
            assert_eq!(f.agents[0].went, Some(expected_go));
            assert_ne!(w.scores[0], 7.0);
            assert_eq!(w.recent.back(), Some(&f.attendance));
        }
        let mut w = FarolWorld::new(
            FarolConfig {
                agents: 3,
                capacity: Some(1),
                strategies: 1,
                ..FarolConfig::default()
            },
            9,
        )
        .unwrap();
        w.recent = VecDeque::from(vec![1; LOOKBACK]);
        for a in &mut w.agents {
            a.predictors = vec![0];
        }
        w.agents[0].predictors = vec![0]; // forecast 1: stay
        w.agents[1].predictors = vec![1]; // two weeks ago 0: go
        w.recent[LOOKBACK - 2] = 0;
        let f = w.step_recorded();
        assert_eq!(f.attendance, 1);
        assert_eq!(f.crowded, Some(true));
        assert_eq!(f.winning_attend, Some(false));
    }

    #[test]
    fn recorded_minority_uses_actual_input_and_preserves_rng_and_all_series() {
        for game in [Game::ElFarol, Game::Minority] {
            for behavior in [Behavior::Inductive, Behavior::Random] {
                for information in [Information::True, Information::Random] {
                    let c = FarolConfig {
                        game,
                        behavior,
                        information,
                        ..FarolConfig::default()
                    };
                    let mut native = FarolWorld::new(c.clone(), 42).unwrap();
                    let mut recorded = FarolWorld::new(c, 42).unwrap();
                    for _ in 0..40 {
                        let before = recorded.outcomes;
                        native.step();
                        let f = recorded.step_recorded();
                        assert_eq!(native.fingerprint(), recorded.fingerprint());
                        assert_eq!(native.series_csv(), recorded.series_csv());
                        if game == Game::Minority {
                            let history = f.history_bits.unwrap();
                            if information == Information::True {
                                assert_eq!(history, before);
                            }
                            assert_eq!(
                                recorded.outcomes,
                                (before << 1) | u64::from(f.winning_attend.unwrap())
                            );
                            for (a, real) in f.agents.iter().zip(&recorded.agents) {
                                let mu = (history & ((1u64 << a.memory) - 1)) as usize;
                                for (v, t) in a.strategies.iter().zip(&real.tables) {
                                    assert_eq!(v.attend, Some(entry(t, mu)));
                                }
                                if behavior == Behavior::Inductive {
                                    assert_eq!(a.went, a.strategies[a.selected.unwrap()].attend);
                                }
                            }
                        }
                        if behavior == Behavior::Random {
                            assert!(f
                                .agents
                                .iter()
                                .all(|a| a.selected.is_none()
                                    && a.strategies.iter().all(|s| !s.active)));
                        }
                    }
                    // Future RNG remains identical after recording stops.
                    native.run(10);
                    recorded.run(10);
                    assert_eq!(native.fingerprint(), recorded.fingerprint());
                }
            }
        }
    }

    #[test]
    fn minority_exact_capacity_wins_a_and_keeps_pre_score() {
        let mut w = FarolWorld::new(
            FarolConfig {
                game: Game::Minority,
                agents: 3,
                strategies: 1,
                memory: 1,
                ..FarolConfig::default()
            },
            3,
        )
        .unwrap();
        w.outcomes = 0;
        for (i, a) in w.agents.iter_mut().enumerate() {
            a.tables = vec![Arc::new(vec![if i == 0 { 1 } else { 0 }])];
            a.points = vec![7.0];
        }
        let f = w.step_recorded();
        assert_eq!(f.attendance, 1);
        assert_eq!(f.crowded, Some(false));
        assert_eq!(f.winning_attend, Some(true));
        assert_eq!(f.history_bits, Some(0));
        assert_eq!(f.agents[0].strategies[0].score, 7.0);
        assert_eq!(w.agents[0].points[0], 8.0);
    }
}
