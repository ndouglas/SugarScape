//! The Social Structure world: each period every agent plays short iterated
//! Prisoner's Dilemmas with the partners its social structure gives it, then
//! copies its best partner if that partner did strictly better, with CRA's
//! two kinds of error.

use std::collections::VecDeque;
use std::fmt::Write;
use std::sync::Arc;

use rand::Rng;
use serde::Serialize;

use super::config::{NoiseOn, Start, Structure, StructureConfig};
use super::graph::{other, Graph};
use super::stats::{payoff, regression, StructureSnapshot};
use super::view::{
    block_side, cell, class, class_color, frame, plane_cell, plane_x, scale, Canvas, DOT, DOT_MANY,
    PLANE, PLANE_BG, TRAIL, TRAIL_NEW, TRAIL_OLD,
};
use crate::anasazi::random::normal;
use crate::config::FieldError;
use crate::export;
use crate::model::{wrong_model, Model, ModelConfig, ModelKind};
use crate::render::lerp;
use crate::rng::{self, SimRng};
use crate::stats::{Series, Stats};

/// A strategy (Nowak & Sigmund's): cooperate first with probability y,
/// after the other's cooperation with p, after its defection with q.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct Strategy {
    pub y: f64,
    pub p: f64,
    pub q: f64,
}

/// The frame's color modes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StructureMode {
    /// p on a red–green scale.
    Friendliness,
    /// 1 − q on a red–green scale.
    Provocability,
    /// This period's payoff per move (0–5 scaled to 0–3).
    Payoff,
    /// The nearest of TFT, ALLD, ALLC, or other.
    Strategy,
}

impl std::str::FromStr for StructureMode {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, String> {
        Ok(match s {
            "friendliness" => Self::Friendliness,
            "provocability" => Self::Provocability,
            "payoff" => Self::Payoff,
            "strategy" => Self::Strategy,
            _ => return Err(format!("unknown color mode {s:?}")),
        })
    }
}

/// What Inspect shows.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct StructureInspection {
    pub site: StructureCell,
    /// The block cell clicked, null off the block.
    pub block: Option<StructureCell>,
    /// The (p, q) of the plane cell clicked, null off the plane.
    pub plane: Option<[f64; 2]>,
    /// The agents at the plane cell (none on the block).
    pub agents: Vec<AgentView>,
    /// The agent in the block cell, with its partners.
    pub agent: Option<AgentView>,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct StructureCell {
    pub x: u32,
    pub y: u32,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct AgentView {
    pub id: u64,
    pub y: f64,
    pub p: f64,
    pub q: f64,
    /// `"tft"`, `"alld"`, `"allc"` or `"other"`.
    pub class: &'static str,
    /// This period's payoff per move.
    pub score: f64,
    /// The partner copied this period.
    pub copied: Option<u64>,
    /// The agents played this period (empty in plane lists).
    pub partners: Vec<PartnerView>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct PartnerView {
    pub id: u64,
    /// The strategy's p as played this period.
    pub p: f64,
    pub score: f64,
    pub games: u32,
}

#[derive(Clone)]
pub struct StructureWorld {
    pub config: StructureConfig,
    /// Completed periods.
    pub tick: u64,
    strategies: Vec<Strategy>,
    graph: Arc<Graph>,
    rng: SimRng,
    /// This period's: strategies as played, scores, the distinct partners
    /// met with game counts, and whom each agent copied.
    played: Vec<Strategy>,
    scores: Vec<f64>,
    met: Vec<Vec<(u32, u32)>>,
    copied: Vec<Option<u32>>,
    /// The population's mean (p, q), newest last.
    trail: VecDeque<(f64, f64)>,
    attained_high: Option<u64>,
    high_since: u64,
    pub stats: Stats<StructureSnapshot>,
}

impl StructureWorld {
    pub fn new(config: StructureConfig, seed: u64) -> Result<Self, Vec<FieldError>> {
        config.validate()?;
        let mut rng = rng::seeded(seed);
        let n = config.agents as usize;
        let strategies: Vec<Strategy> = match config.start {
            Start::Grid => {
                let k = block_side(n);
                (0..n)
                    .map(|i| {
                        let p = ((i % k) as f64 + 0.5) / k as f64;
                        let q = ((i / k) as f64 + 0.5) / k as f64;
                        Strategy { y: p, p, q }
                    })
                    .collect()
            }
            Start::Random => (0..n)
                .map(|_| {
                    let p = rng.gen::<f64>();
                    let q = rng.gen::<f64>();
                    Strategy { y: p, p, q }
                })
                .collect(),
        };
        let graph = Arc::new(Graph::new(&config, &mut rng));
        let mut world = StructureWorld {
            config,
            tick: 0,
            played: strategies.clone(),
            strategies,
            graph,
            rng,
            scores: vec![0.0; n],
            met: vec![Vec::new(); n],
            copied: vec![None; n],
            trail: VecDeque::with_capacity(TRAIL),
            attained_high: None,
            high_since: 0,
            stats: Stats::default(),
        };
        world.record(0.0, 0.0);
        Ok(world)
    }

    pub fn strategies(&self) -> &[Strategy] {
        &self.strategies
    }

    pub fn graph(&self) -> &Graph {
        &self.graph
    }

    /// Whether the run has stopped at `stop_at`.
    pub fn is_finished(&self) -> bool {
        self.config.stop_at > 0 && self.tick >= u64::from(self.config.stop_at)
    }

    /// This period's partners chosen by each agent.
    fn partners(&mut self) -> Vec<Vec<u32>> {
        let n = self.strategies.len();
        let k = self.config.partners;
        let x = self.config.substitution;
        match self.config.structure {
            Structure::Rwr => (0..n)
                .map(|a| (0..k).map(|_| other(&mut self.rng, n, a)).collect())
                .collect(),
            _ => {
                let graph = self.graph.clone();
                (0..n)
                    .map(|a| {
                        graph.chosen[a]
                            .iter()
                            .map(|&b| {
                                if x > 0.0 && self.rng.gen::<f64>() < x {
                                    other(&mut self.rng, n, a)
                                } else {
                                    b
                                }
                            })
                            .collect()
                    })
                    .collect()
            }
        }
    }

    /// One game of `moves` moves: each player's payoff and cooperations.
    fn game(&mut self, a: Strategy, b: Strategy) -> ([u32; 2], [u32; 2]) {
        let (mut pay, mut coop) = ([0; 2], [0; 2]);
        let (mut last_a, mut last_b) = (true, true);
        for m in 0..self.config.moves {
            let (pa, pb) = if m == 0 {
                (a.y, b.y)
            } else {
                (
                    if last_b { a.p } else { a.q },
                    if last_a { b.p } else { b.q },
                )
            };
            let ca = self.rng.gen::<f64>() < pa;
            let cb = self.rng.gen::<f64>() < pb;
            pay[0] += payoff(ca, cb);
            pay[1] += payoff(cb, ca);
            coop[0] += u32::from(ca);
            coop[1] += u32::from(cb);
            (last_a, last_b) = (ca, cb);
        }
        (pay, coop)
    }

    /// One period: games, scores, adaptation.
    // `a` indexes met, scores, played and copied as well as next.
    #[allow(clippy::needless_range_loop)]
    pub fn step(&mut self) {
        let n = self.strategies.len();
        let chosen = self.partners();
        self.played = self.strategies.clone();
        let mut payoffs = vec![0u64; n];
        let mut moves = vec![0u64; n];
        let (mut coop_total, mut move_total) = (0u64, 0u64);
        let mut met: Vec<Vec<(u32, u32)>> = vec![Vec::new(); n];
        let meet = |met: &mut Vec<Vec<(u32, u32)>>, a: usize, b: u32| match met[a]
            .iter_mut()
            .find(|e| e.0 == b)
        {
            Some(e) => e.1 += 1,
            None => met[a].push((b, 1)),
        };
        let m = u64::from(self.config.moves);
        for (a, list) in chosen.iter().enumerate() {
            for &b in list {
                let (pay, coop) = self.game(self.played[a], self.played[b as usize]);
                payoffs[a] += u64::from(pay[0]);
                payoffs[b as usize] += u64::from(pay[1]);
                moves[a] += m;
                moves[b as usize] += m;
                coop_total += u64::from(coop[0] + coop[1]);
                move_total += 2 * m;
                meet(&mut met, a, b);
                meet(&mut met, b as usize, a as u32);
            }
        }
        self.scores = (0..n)
            .map(|a| {
                if moves[a] == 0 {
                    0.0
                } else {
                    payoffs[a] as f64 / moves[a] as f64
                }
            })
            .collect();
        self.met = met;
        // Adaptation, from this period's strategies and scores.
        let mut next = self.played.clone();
        self.copied = vec![None; n];
        for a in 0..n {
            let mut best: Vec<u32> = Vec::new();
            let mut top = f64::NEG_INFINITY;
            for &(b, _) in &self.met[a] {
                let s = self.scores[b as usize];
                if s > top {
                    top = s;
                    best.clear();
                    best.push(b);
                } else if s == top {
                    best.push(b);
                }
            }
            let mut copy = false;
            if !best.is_empty() {
                let pick = best[self.rng.gen_range(0..best.len() as u32) as usize];
                let better = self.scores[pick as usize] > self.scores[a];
                let wrong = self.rng.gen::<f64>() < self.config.judge_error;
                if better != wrong {
                    next[a] = self.played[pick as usize];
                    self.copied[a] = Some(pick);
                    copy = true;
                }
            }
            if self.config.noise_on == NoiseOn::Always || copy {
                let s = &mut next[a];
                for v in [&mut s.y, &mut s.p, &mut s.q] {
                    if self.rng.gen::<f64>() < self.config.mutation {
                        *v = (*v + normal(&mut self.rng) * self.config.mutation_sd).clamp(0.0, 1.0);
                    }
                }
            }
        }
        self.strategies = next;
        self.tick += 1;
        let mean_payoff = if move_total == 0 {
            0.0
        } else {
            payoffs.iter().sum::<u64>() as f64 / moves.iter().sum::<u64>() as f64
        };
        let cooperation = if move_total == 0 {
            0.0
        } else {
            coop_total as f64 / move_total as f64
        };
        self.record(mean_payoff, cooperation);
    }

    pub fn run(&mut self, ticks: u32) {
        for _ in 0..ticks {
            if self.is_finished() {
                break;
            }
            self.step();
        }
    }

    /// (own p, partners' mean p) for every agent that played this period,
    /// with strategies as played (CRA's Figs. 5–6).
    pub fn partner_p_pairs(&self) -> Vec<(f64, f64)> {
        (0..self.played.len())
            .filter(|&a| !self.met[a].is_empty())
            .map(|a| {
                let (sum, count) = self.met[a].iter().fold((0.0, 0u32), |(s, c), &(b, g)| {
                    (s + self.played[b as usize].p * f64::from(g), c + g)
                });
                (self.played[a].p, sum / f64::from(count))
            })
            .collect()
    }

    fn record(&mut self, mean_payoff: f64, cooperation: f64) {
        let n = self.strategies.len() as f64;
        let mean = |f: fn(&Strategy) -> f64| self.strategies.iter().map(f).sum::<f64>() / n;
        let (mean_y, mean_p, mean_q) = (mean(|s| s.y), mean(|s| s.p), mean(|s| s.q));
        let high = self.tick > 0 && mean_payoff >= self.config.high;
        if high && self.attained_high.is_none() {
            self.attained_high = Some(self.tick);
        }
        if high && self.attained_high.is_some() {
            self.high_since += 1;
        }
        let share_high_since = match self.attained_high {
            Some(t) => self.high_since as f64 / (self.tick - t + 1) as f64,
            None => 0.0,
        };
        if self.trail.len() == TRAIL {
            self.trail.pop_front();
        }
        self.trail.push_back((mean_p, mean_q));
        let copied = self.copied.iter().filter(|c| c.is_some()).count() as f64 / n;
        let partner_p_slope = regression(&self.partner_p_pairs()).0;
        self.stats.push(StructureSnapshot {
            tick: self.tick,
            mean_payoff,
            cooperation,
            mean_y,
            mean_p,
            mean_q,
            high: u8::from(high),
            attained_high: self.attained_high.map_or(-1, |t| t as i64),
            share_high_since,
            copied,
            partner_p_slope,
        });
    }

    /// The block cell of agent `a`: its torus site, else its index.
    fn block_of(&self, a: usize) -> (usize, usize) {
        let k = if self.graph.site.is_empty() {
            a
        } else {
            self.graph.site[a] as usize
        };
        let s = block_side(self.strategies.len());
        (k % s, k / s)
    }

    /// The agent at block cell (bx, by), if any.
    fn agent_at(&self, bx: usize, by: usize) -> Option<usize> {
        let s = block_side(self.strategies.len());
        let k = by * s + bx;
        if k >= self.strategies.len() {
            return None;
        }
        Some(if self.graph.agent_at.is_empty() {
            k
        } else {
            self.graph.agent_at[k] as usize
        })
    }

    fn view(&self, a: usize, with_partners: bool) -> AgentView {
        let s = self.strategies[a];
        AgentView {
            id: a as u64 + 1,
            y: s.y,
            p: s.p,
            q: s.q,
            class: class(s.y, s.p, s.q),
            score: self.scores[a],
            copied: self.copied[a].map(|b| u64::from(b) + 1),
            partners: if with_partners {
                self.met[a]
                    .iter()
                    .map(|&(b, g)| PartnerView {
                        id: u64::from(b) + 1,
                        p: self.played[b as usize].p,
                        score: self.scores[b as usize],
                        games: g,
                    })
                    .collect()
            } else {
                Vec::new()
            },
        }
    }

    pub fn inspect(&self, x: u32, y: u32) -> Result<StructureInspection, String> {
        let n = self.strategies.len();
        let (fw, fh) = frame(n);
        if x as usize >= fw || y as usize >= fh {
            return Err(format!("({x}, {y}) is not in the frame"));
        }
        let mut out = StructureInspection {
            site: StructureCell { x, y },
            block: None,
            plane: None,
            agents: Vec::new(),
            agent: None,
        };
        let (cx, cy) = (x as usize, y as usize);
        let c = cell(n);
        let side = block_side(n);
        if cx < side * c && cy < side * c {
            let (bx, by) = (cx / c, cy / c);
            out.block = Some(StructureCell {
                x: bx as u32,
                y: by as u32,
            });
            out.agent = self.agent_at(bx, by).map(|a| self.view(a, true));
        } else if cx >= plane_x(n) && cy < PLANE {
            let (px, py) = (cx - plane_x(n), cy);
            let p = px as f64 / (PLANE - 1) as f64;
            let q = (PLANE - 1 - py) as f64 / (PLANE - 1) as f64;
            out.plane = Some([p, q]);
            out.agents = (0..n)
                .filter(|&a| plane_cell(self.strategies[a].p, self.strategies[a].q) == (px, py))
                .map(|a| self.view(a, false))
                .collect();
        }
        Ok(out)
    }
}

impl Model for StructureWorld {
    fn config(&self) -> ModelConfig {
        ModelConfig::Structure(self.config.clone())
    }

    fn run(&mut self, ticks: u32) {
        StructureWorld::run(self, ticks);
    }

    fn tick(&self) -> u64 {
        self.tick
    }

    fn population(&self) -> usize {
        self.strategies.len()
    }

    /// FNV-1a over the tick and every strategy's bits.
    fn fingerprint(&self) -> u64 {
        let mut h: u64 = 0xcbf2_9ce4_8422_2325;
        let mut eat = |bytes: [u8; 8]| {
            for b in bytes {
                h ^= u64::from(b);
                h = h.wrapping_mul(0x0100_0000_01b3);
            }
        };
        eat(self.tick.to_le_bytes());
        for s in &self.strategies {
            for v in [s.y, s.p, s.q] {
                eat(v.to_bits().to_le_bytes());
            }
        }
        h
    }

    fn size(&self) -> (u32, u32) {
        let (w, h) = frame(self.strategies.len());
        (w as u32, h as u32)
    }

    fn render(&self, mode: &str, _layer: &str, buf: &mut Vec<u8>) -> Result<(), String> {
        let mode: StructureMode = mode.parse()?;
        let n = self.strategies.len();
        let (fw, fh) = frame(n);
        let mut c = Canvas { buf, wide: 0 };
        c.clear(fw, fh);
        let size = cell(n);
        for a in 0..n {
            let s = self.strategies[a];
            let color = match mode {
                StructureMode::Friendliness => scale(s.p),
                StructureMode::Provocability => scale(1.0 - s.q),
                StructureMode::Payoff => scale(self.scores[a] / 3.0),
                StructureMode::Strategy => class_color(class(s.y, s.p, s.q)),
            };
            let (bx, by) = self.block_of(a);
            for dy in 0..size {
                for dx in 0..size {
                    c.put(bx * size + dx, by * size + dy, color);
                }
            }
        }
        let ox = plane_x(n);
        for y in 0..PLANE {
            for x in 0..PLANE {
                c.put(ox + x, y, PLANE_BG);
            }
        }
        let len = self.trail.len();
        for (i, &(p, q)) in self.trail.iter().enumerate() {
            let (x, y) = plane_cell(p, q);
            c.put(
                ox + x,
                y,
                lerp(TRAIL_OLD, TRAIL_NEW, (i + 1) as f64 / len as f64),
            );
        }
        let mut counts = vec![0u32; PLANE * PLANE];
        for s in &self.strategies {
            let (x, y) = plane_cell(s.p, s.q);
            counts[y * PLANE + x] += 1;
        }
        for (k, &count) in counts.iter().enumerate() {
            if count > 0 {
                let t = (f64::from(count - 1) / 8.0).min(1.0);
                c.put(ox + k % PLANE, k / PLANE, lerp(DOT, DOT_MANY, t));
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
        let mut out = String::from("id,y,p,q,class,score,copied,partners\n");
        for a in 0..self.strategies.len() {
            let v = self.view(a, false);
            writeln!(
                out,
                "{},{},{},{},{},{},{},{}",
                v.id,
                v.y,
                v.p,
                v.q,
                v.class,
                v.score,
                v.copied.map_or(String::new(), |c| c.to_string()),
                self.met[a].len()
            )
            .unwrap();
        }
        out
    }

    fn inspect_json(&self, x: u32, y: u32) -> Result<String, String> {
        let inspection = self.inspect(x, y)?;
        Ok(serde_json::to_string(&inspection).expect("inspection serializes"))
    }

    /// An agent's block cell (its center).
    fn locate(&self, id: u64) -> Option<(u32, u32)> {
        let a = usize::try_from(id.checked_sub(1)?).ok()?;
        if a >= self.strategies.len() {
            return None;
        }
        let (bx, by) = self.block_of(a);
        let c = cell(self.strategies.len());
        Some(((bx * c + c / 2) as u32, (by * c + c / 2) as u32))
    }

    fn set_config(&mut self, next: ModelConfig) -> Result<(), Vec<FieldError>> {
        let ModelConfig::Structure(next) = next else {
            return Err(wrong_model(ModelKind::Structure, &next));
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

    fn config(edit: impl FnOnce(&mut StructureConfig)) -> StructureConfig {
        let mut c = StructureConfig::default();
        edit(&mut c);
        c
    }

    fn world(edit: impl FnOnce(&mut StructureConfig)) -> StructureWorld {
        StructureWorld::new(config(edit), 1).unwrap()
    }

    const TFT: Strategy = Strategy {
        y: 1.0,
        p: 1.0,
        q: 0.0,
    };
    const ALLD: Strategy = Strategy {
        y: 0.0,
        p: 0.0,
        q: 0.0,
    };
    const ALLC: Strategy = Strategy {
        y: 1.0,
        p: 1.0,
        q: 1.0,
    };

    #[test]
    fn games_follow_the_strategies() {
        let mut w = world(|_| {});
        assert_eq!(w.game(TFT, TFT), ([12, 12], [4, 4]));
        assert_eq!(w.game(ALLD, ALLC), ([20, 0], [0, 4]));
        // TFT against ALLD: cooperates once, then defects: 0 + 1 + 1 + 1.
        assert_eq!(w.game(TFT, ALLD), ([3, 8], [1, 0]));
    }

    #[test]
    fn starts_spread_evenly_or_at_random_with_y_equal_to_p() {
        let g = world(|_| {});
        assert_eq!(
            g.strategies()[0],
            Strategy {
                y: 1.0 / 32.0,
                p: 1.0 / 32.0,
                q: 1.0 / 32.0
            }
        );
        assert_eq!(g.strategies()[17].p, 1.5 / 16.0);
        assert_eq!(g.strategies()[17].q, 1.5 / 16.0);
        let r = world(|c| c.start = Start::Random);
        assert!(r.strategies().iter().all(|s| s.y == s.p));
        assert_ne!(r.strategies()[0], g.strategies()[0]);
    }

    #[test]
    fn a_period_plays_every_chosen_game_in_both_roles() {
        let mut w = world(|c| {
            c.mutation = 0.0;
            c.judge_error = 0.0;
        });
        w.step();
        let games: u32 = w.met.iter().flatten().map(|e| e.1).sum();
        assert_eq!(games, 2 * 256 * 4, "each game counted for both players");
        assert!(w
            .met
            .iter()
            .enumerate()
            .all(|(a, l)| l.iter().all(|e| e.0 as usize != a)));
        let s = w.stats.latest().unwrap();
        // Fig. 1: a spread-out start realizes each cell about a quarter of the time.
        assert!((s.mean_payoff - 2.25).abs() < 0.1, "{}", s.mean_payoff);
        let mut t = world(|c| c.structure = Structure::Torus);
        t.step();
        assert!(
            t.met
                .iter()
                .all(|l| l.len() == 4 && l.iter().all(|e| e.1 == 2)),
            "torus pairs play twice"
        );
    }

    #[test]
    fn agents_copy_only_a_strictly_better_partner_unless_they_misjudge() {
        let mut w = world(|c| {
            c.agents = 4;
            c.partners = 1;
            c.structure = Structure::Frn;
            c.mutation = 0.0;
            c.judge_error = 0.0;
        });
        w.strategies = vec![ALLD, ALLC, ALLC, ALLC];
        w.step();
        // ALLD beats every ALLC it meets; ALLCs that met ALLD copy it.
        for a in 0..4 {
            match w.copied[a] {
                Some(b) => {
                    assert!(w.scores[b as usize] > w.scores[a]);
                    assert_eq!(w.strategies[a], w.played[b as usize]);
                }
                None => assert_eq!(w.strategies[a], w.played[a]),
            }
        }
        assert_eq!(w.copied[0], None, "the best copies no one");
        let mut e = world(|c| {
            c.agents = 4;
            c.partners = 1;
            c.structure = Structure::Frn;
            c.mutation = 0.0;
            c.judge_error = 1.0;
        });
        e.strategies = vec![ALLD, ALLC, ALLC, ALLC];
        e.step();
        assert!(
            e.copied[0].is_some(),
            "always misjudging, the best copies a worse partner"
        );
    }

    #[test]
    fn noise_hits_everyone_or_only_copiers() {
        let run = |noise_on| {
            let mut w = world(|c| {
                c.mutation = 1.0;
                c.judge_error = 0.0;
                c.noise_on = noise_on;
            });
            w.step();
            (0..256)
                .filter(|&a| w.copied[a].is_none() && w.strategies[a] != w.played[a])
                .count()
        };
        assert!(run(NoiseOn::Always) > 0);
        assert_eq!(run(NoiseOn::Copy), 0);
        let mut w = world(|c| {
            c.mutation = 1.0;
            c.mutation_sd = 2.0;
        });
        w.run(3);
        assert!(w
            .strategies()
            .iter()
            .all(|s| [s.y, s.p, s.q].iter().all(|v| (0.0..=1.0).contains(v))));
    }

    #[test]
    fn substitution_replaces_fixed_links_only_for_the_period() {
        let mut w = world(|c| {
            c.structure = Structure::Frn;
            c.substitution = 1.0;
        });
        let fixed = w.graph.chosen.clone();
        let chosen = w.partners();
        let same = chosen.iter().zip(&fixed).filter(|(a, b)| a == b).count();
        assert!(same < 5, "{same} agents kept every link");
        assert_eq!(w.graph.chosen, fixed, "the network itself is untouched");
        let mut none = world(|c| c.structure = Structure::Frn);
        assert_eq!(none.partners(), none.graph.chosen);
    }

    #[test]
    fn high_cooperation_is_attained_and_remembered() {
        let mut w = world(|c| {
            c.mutation = 0.0;
            c.judge_error = 0.0;
            c.structure = Structure::Frne;
            c.high = 2.9;
        });
        w.strategies = vec![TFT; 256];
        w.step();
        let s = w.stats.latest().unwrap().clone();
        assert_eq!(
            (s.mean_payoff, s.cooperation, s.high, s.attained_high),
            (3.0, 1.0, 1, 1)
        );
        w.config.high = 3.5;
        w.step();
        let s = w.stats.latest().unwrap();
        assert_eq!((s.high, s.attained_high, s.share_high_since), (0, 1, 0.5));
        assert_eq!(
            w.stats.history()[0].mean_payoff,
            0.0,
            "no games before period 1"
        );
    }

    #[test]
    fn the_partner_regression_reads_the_strategies_as_played() {
        let mut w = world(|c| {
            c.structure = Structure::Frne;
            c.mutation = 0.0;
            c.judge_error = 0.0;
        });
        w.step();
        let pairs = w.partner_p_pairs();
        assert_eq!(pairs.len(), 256);
        let a = 7;
        let mean: f64 = w.met[a]
            .iter()
            .map(|&(b, g)| w.played[b as usize].p * f64::from(g))
            .sum::<f64>()
            / w.met[a].iter().map(|e| f64::from(e.1)).sum::<f64>();
        assert_eq!(pairs[a], (w.played[a].p, mean));
        assert_eq!(
            w.stats.latest().unwrap().partner_p_slope,
            regression(&pairs).0
        );
    }

    #[test]
    fn the_view_draws_block_and_plane_and_inspect_finds_both() {
        let mut w = world(|c| c.structure = Structure::Torus);
        w.run(3);
        let (fw, fh) = frame(256);
        assert_eq!(Model::size(&w), (fw as u32, fh as u32));
        let mut buf = Vec::new();
        for mode in ["friendliness", "provocability", "payoff", "strategy"] {
            w.render(mode, "", &mut buf).unwrap();
            assert_eq!(buf.len(), fw * fh * 4);
        }
        assert!(w.render("wealth", "", &mut buf).is_err());
        let agent = w.graph.agent_at[17] as usize;
        let v = w.inspect((6 + 1) as u32, (6 + 1) as u32).unwrap();
        assert_eq!(v.block, Some(StructureCell { x: 1, y: 1 }));
        let seen = v.agent.unwrap();
        assert_eq!(seen.id, agent as u64 + 1);
        assert_eq!(seen.partners.len(), 4);
        assert_eq!(Model::locate(&w, seen.id), Some((6 + 3, 6 + 3)));
        let s = w.strategies()[0];
        let (px, py) = plane_cell(s.p, s.q);
        let on = w.inspect((plane_x(256) + px) as u32, py as u32).unwrap();
        assert!(on.plane.is_some() && on.agents.iter().any(|a| a.id == 1));
        let gap = w.inspect(97, 0).unwrap();
        assert!(gap.block.is_none() && gap.plane.is_none());
        assert!(w.inspect(fw as u32, 0).is_err());
    }

    #[test]
    fn keyframes_restore_strategies_and_the_trail() {
        let mut any = crate::model::ModelWorld::new(
            ModelConfig::Structure(config(|c| c.structure = Structure::Frn)),
            6,
        )
        .unwrap();
        any.model_mut().run(5);
        let cp = any.checkpoint().unwrap();
        let print = any.model().fingerprint();
        let mut before = Vec::new();
        any.model().render("friendliness", "", &mut before).unwrap();
        any.model_mut().run(10);
        any.restore(&cp).unwrap();
        assert_eq!(any.model().fingerprint(), print);
        let mut after = Vec::new();
        any.model().render("friendliness", "", &mut after).unwrap();
        assert_eq!(before, after);
        assert_eq!(any.model().series("mean_payoff").unwrap().len(), 6);
    }

    #[test]
    fn live_edits_apply_and_the_population_waits_for_reset() {
        let mut w = world(|_| {});
        let next = config(|c| {
            c.substitution = 0.3;
            c.judge_error = 0.0;
            c.noise_on = NoiseOn::Copy;
            c.high = 2.5;
        });
        Model::set_config(&mut w, ModelConfig::Structure(next.clone())).unwrap();
        w.step();
        let e = Model::set_config(
            &mut w,
            ModelConfig::Structure(StructureConfig {
                structure: Structure::Frn,
                ..next
            }),
        )
        .unwrap_err();
        assert_eq!(e[0].field, "structure");
    }

    #[test]
    fn the_stop_ends_the_run() {
        let mut w = world(|c| c.stop_at = 5);
        w.run(100);
        assert_eq!(w.tick, 5);
        assert!(Model::finished(&w));
    }

    #[test]
    fn degenerate_configs_run_without_panicking() {
        for c in [
            config(|c| {
                c.agents = 4;
                c.partners = 3;
            }),
            config(|c| {
                c.agents = 9;
                c.structure = Structure::Torus;
            }),
            config(|c| {
                c.agents = 4;
                c.partners = 2;
                c.structure = Structure::Frne;
            }),
            config(|c| c.judge_error = 1.0),
            config(|c| c.mutation = 1.0),
            config(|c| {
                c.structure = Structure::Frn;
                c.substitution = 1.0;
            }),
            config(|c| c.moves = 1),
            config(|c| c.agents = 250),
        ] {
            let mut w = StructureWorld::new(c.clone(), 1).unwrap();
            w.run(20);
            let s = w.stats.latest().unwrap();
            assert!((0.0..=5.0).contains(&s.mean_payoff), "{c:?}");
            let mut buf = Vec::new();
            w.render("strategy", "", &mut buf).unwrap();
        }
    }
}
