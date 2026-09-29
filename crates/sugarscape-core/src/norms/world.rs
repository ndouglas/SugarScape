//! The norms world: each generation every agent gets a few chances to defect
//! under a random chance of being seen; others punish what they see (and,
//! with metanorms, those seen letting it pass); then the generation is
//! replaced by the offspring of its more successful members.

use std::collections::VecDeque;
use std::fmt::Write;

use rand::Rng;
use serde::Serialize;

use super::config::{AllEqual, NormsConfig, Refill, Selection};
use super::stats::{collapsed, established, NormsSnapshot};
use super::view::{
    agent_at_row, frame, level_at, level_cell, mean_pixel, row_top, Canvas, BOLD_BAR, COLLAPSED,
    DENSE, ESTABLISHED, GRID, LEVEL, PLANE, PLANE_BG, ROW, SPARSE, STRIP, STRIP_X, STRONG, TRAIL,
    TRAIL_NEW, TRAIL_OLD, VENGE_BAR, WEAK,
};
use crate::config::FieldError;
use crate::export;
use crate::model::{wrong_model, Model, ModelConfig, ModelKind};
use crate::render::lerp;
use crate::rng::{self, SimRng};
use crate::stats::{Series, Stats};

/// An agent: its strategy (levels 0–7), its group, and what happened to it
/// in the generation it played.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Agent {
    pub boldness: u8,
    pub vengefulness: u8,
    /// 0 strong (or the only group), 1 weak.
    pub group: u8,
    /// The previous generation's agent this one copies.
    pub parent: Option<u32>,
    pub payoff: f64,
    pub defections: u32,
    pub punished: u32,
    pub punishments: u32,
    pub metapunishments: u32,
    pub metapunished: u32,
}

impl Agent {
    /// The six bits: boldness, then vengefulness, most significant first.
    pub fn bits(&self) -> String {
        format!("{:03b}{:03b}", self.boldness, self.vengefulness)
    }
}

/// The frame's color modes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NormsMode {
    /// Plane cells by how many agents hold each strategy.
    Agents,
    /// Plane cells and strip rows by this generation's payoff.
    Payoff,
    /// Plane cells and strip rows by group.
    Group,
}

impl std::str::FromStr for NormsMode {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, String> {
        Ok(match s {
            "agents" => Self::Agents,
            "payoff" => Self::Payoff,
            "group" => Self::Group,
            _ => return Err(format!("unknown color mode {s:?}")),
        })
    }
}

/// What Inspect shows.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct NormsInspection {
    pub site: NormsCell,
    /// The plane cell's (boldness, vengefulness) levels, null off the plane.
    pub level: Option<[u8; 2]>,
    /// The agents holding that strategy.
    pub agents: Vec<NormAgentView>,
    /// The agent in the strip row clicked.
    pub agent: Option<NormAgentView>,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct NormsCell {
    pub x: u32,
    pub y: u32,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct NormAgentView {
    pub id: u64,
    /// `"strong"` or `"weak"`; null without groups.
    pub group: Option<&'static str>,
    pub bits: String,
    pub boldness: u8,
    pub vengefulness: u8,
    pub payoff: f64,
    pub defections: u32,
    pub punished: u32,
    pub punishments: u32,
    pub metapunishments: u32,
    pub metapunished: u32,
    /// The previous generation's agent it copies.
    pub parent: Option<u64>,
}

#[derive(Clone)]
pub struct NormsWorld {
    pub config: NormsConfig,
    /// Completed generations.
    pub tick: u64,
    /// The generation that plays next.
    agents: Vec<Agent>,
    /// The generation that played last (with payoffs and counts).
    played: Vec<Agent>,
    rng: SimRng,
    /// The mean (B, V) of recent generations, newest last.
    trail: VecDeque<(f64, f64)>,
    copied_equal: bool,
    /// The last generation's events, when recording them (the studio's shots).
    events: Option<NormsEvents>,
    pub stats: Stats<NormsSnapshot>,
}

/// What a generation's play did, in order, by places in the agent list:
/// each cheat, each punishment (punisher, cheat), and each metapunishment
/// (metapunisher, the one who looked away). Recording draws nothing.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct NormsEvents {
    pub cheats: Vec<u32>,
    pub punishments: Vec<(u32, u32)>,
    pub metapunishments: Vec<(u32, u32)>,
}

impl NormsWorld {
    pub fn new(config: NormsConfig, seed: u64) -> Result<Self, Vec<FieldError>> {
        config.validate()?;
        let mut rng = rng::seeded(seed);
        let groups: Vec<u8> = if config.groups.enabled {
            let (s, w) = (config.groups.strong as usize, config.groups.weak as usize);
            [vec![0; s], vec![1; w]].concat()
        } else {
            vec![0; config.agents as usize]
        };
        let agents: Vec<Agent> = groups
            .into_iter()
            .map(|group| Agent {
                boldness: rng.gen_range(0..8u8),
                vengefulness: rng.gen_range(0..8u8),
                group,
                ..Agent::default()
            })
            .collect();
        let mut world = NormsWorld {
            config,
            tick: 0,
            played: agents.clone(),
            agents,
            rng,
            trail: VecDeque::with_capacity(TRAIL),
            copied_equal: false,
            events: None,
            stats: Stats::default(),
        };
        world.record();
        Ok(world)
    }

    /// The generation that played last (the starting one before any play).
    pub fn played(&self) -> &[Agent] {
        &self.played
    }

    /// Starts or stops recording each generation's events (from the next
    /// generation played). Recording draws nothing.
    pub fn record_events(&mut self, on: bool) {
        self.events = on.then(NormsEvents::default);
    }

    /// The last generation's events, if recording.
    pub fn events(&self) -> Option<&NormsEvents> {
        self.events.as_ref()
    }

    /// The generation that plays next.
    pub fn agents(&self) -> &[Agent] {
        &self.agents
    }

    pub fn is_finished(&self) -> bool {
        self.config.stop_at > 0 && self.tick >= u64::from(self.config.stop_at)
    }

    /// Whether probability `p` comes up (a draw is spent either way).
    fn chance(&mut self, p: f64) -> bool {
        self.rng.gen::<f64>() < p
    }

    /// The cost of being punished for agent `i`.
    fn punishment_of(&self, i: usize) -> f64 {
        if self.config.groups.enabled && self.agents[i].group == 0 {
            self.config.groups.strong_punishment
        } else {
            self.config.punishment
        }
    }

    /// The generation's rounds of play.
    fn play(&mut self) {
        let n = self.agents.len();
        for a in &mut self.agents {
            (
                a.payoff,
                a.defections,
                a.punished,
                a.punishments,
                a.metapunishments,
                a.metapunished,
            ) = (0.0, 0, 0, 0, 0, 0);
        }
        let c = self.config.clone();
        if let Some(e) = &mut self.events {
            *e = NormsEvents::default();
        }
        for _ in 0..c.rounds {
            for i in 0..n {
                let seen = self.rng.gen::<f64>();
                if seen >= f64::from(self.agents[i].boldness) / 7.0 {
                    continue;
                }
                self.agents[i].payoff += c.temptation;
                self.agents[i].defections += 1;
                if let Some(e) = &mut self.events {
                    e.cheats.push(i as u32);
                }
                for j in (0..n).filter(|&j| j != i) {
                    self.agents[j].payoff += c.hurt;
                }
                for j in (0..n).filter(|&j| j != i) {
                    if !self.chance(seen) {
                        continue;
                    }
                    if self.chance(f64::from(self.agents[j].vengefulness) / 7.0) {
                        self.agents[i].payoff += self.punishment_of(i);
                        self.agents[i].punished += 1;
                        self.agents[j].payoff += c.enforcement;
                        self.agents[j].punishments += 1;
                        if let Some(e) = &mut self.events {
                            e.punishments.push((j as u32, i as u32));
                        }
                    } else if c.metanorms {
                        for k in (0..n).filter(|&k| k != i && k != j) {
                            if self.chance(seen)
                                && self.chance(f64::from(self.agents[k].vengefulness) / 7.0)
                            {
                                self.agents[j].payoff += c.meta_punishment;
                                self.agents[j].metapunished += 1;
                                self.agents[k].payoff += c.meta_enforcement;
                                self.agents[k].metapunishments += 1;
                                if let Some(e) = &mut self.events {
                                    e.metapunishments.push((k as u32, j as u32));
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    /// The parents (indices into `members`) of `members.len()` offspring.
    fn select(&mut self, members: &[usize]) -> Vec<usize> {
        let n = members.len();
        let pay: Vec<f64> = members.iter().map(|&i| self.agents[i].payoff).collect();
        let mean = pay.iter().sum::<f64>() / n as f64;
        let all_equal = pay.iter().all(|&p| p == pay[0]);
        let pick = |rng: &mut SimRng| rng.gen_range(0..n as u32) as usize;
        let tournament = |rng: &mut SimRng| -> Vec<usize> {
            (0..n)
                .map(|_| {
                    let (a, b) = (pick(rng), pick(rng));
                    if pay[a] > pay[b] || (pay[a] == pay[b] && rng.gen::<bool>()) {
                        a
                    } else {
                        b
                    }
                })
                .collect()
        };
        let offspring: Vec<usize> = match self.config.selection {
            Selection::Tournament => return tournament(&mut self.rng),
            Selection::Roulette => {
                let min = pay.iter().cloned().fold(f64::INFINITY, f64::min);
                let total: f64 = pay.iter().map(|p| p - min).sum();
                if total <= 0.0 {
                    return tournament(&mut self.rng);
                }
                return (0..n)
                    .map(|_| {
                        let mut r = self.rng.gen::<f64>() * total;
                        for (k, p) in pay.iter().enumerate() {
                            r -= p - min;
                            if r < 0.0 {
                                return k;
                            }
                        }
                        n - 1
                    })
                    .collect();
            }
            Selection::Average => (0..n)
                .flat_map(|k| {
                    let copies = if pay[k] >= mean { 2 } else { 0 };
                    std::iter::repeat_n(k, copies)
                })
                .collect(),
            Selection::Axelrod => {
                if all_equal {
                    self.copied_equal = true;
                    match self.config.all_equal {
                        AllEqual::Keep => return (0..n).collect(),
                        AllEqual::Drift => (0..n).flat_map(|k| [k, k]).collect(),
                    }
                } else {
                    let sd =
                        (pay.iter().map(|p| (p - mean).powi(2)).sum::<f64>() / n as f64).sqrt();
                    (0..n)
                        .flat_map(|k| {
                            let copies = if pay[k] >= mean + sd {
                                2
                            } else if pay[k] <= mean - sd {
                                0
                            } else {
                                1
                            };
                            std::iter::repeat_n(k, copies)
                        })
                        .collect()
                }
            }
        };
        self.refill(offspring, &pay, all_equal)
    }

    /// Brings `offspring` back to the size of `pay`.
    fn refill(&mut self, mut offspring: Vec<usize>, pay: &[f64], all_equal: bool) -> Vec<usize> {
        let n = pay.len();
        if offspring.is_empty() {
            return (0..n).collect();
        }
        if self.config.refill == Refill::Ranked && !all_equal && offspring.len() != n {
            // Worst parents first: remove from the front; copy the best first.
            offspring.sort_by(|&a, &b| pay[a].total_cmp(&pay[b]).then(a.cmp(&b)));
            while offspring.len() > n {
                offspring.remove(0);
            }
            let best: Vec<usize> = offspring.iter().rev().copied().collect();
            for k in 0..n.saturating_sub(offspring.len()) {
                offspring.push(best[k % best.len()]);
            }
            return offspring;
        }
        while offspring.len() > n {
            let k = self.rng.gen_range(0..offspring.len() as u32) as usize;
            offspring.remove(k);
        }
        while offspring.len() < n {
            let k = self.rng.gen_range(0..offspring.len() as u32) as usize;
            offspring.push(offspring[k]);
        }
        offspring
    }

    /// Flips each of a strategy's six bits with probability `mutation`.
    fn mutate(&mut self, level: u8) -> u8 {
        let mut out = level;
        for bit in 0..3 {
            if self.chance(self.config.mutation) {
                out ^= 1 << bit;
            }
        }
        out
    }

    /// One generation: play, record, then the next generation.
    pub fn step(&mut self) {
        self.play();
        self.played = self.agents.clone();
        self.tick += 1;
        self.copied_equal = false;
        let n = self.agents.len();
        let groups: Vec<Vec<usize>> = if self.config.groups.enabled {
            vec![
                (0..n).filter(|&i| self.agents[i].group == 0).collect(),
                (0..n).filter(|&i| self.agents[i].group == 1).collect(),
            ]
        } else {
            vec![(0..n).collect()]
        };
        let mut next = Vec::with_capacity(n);
        for members in groups {
            for k in self.select(&members) {
                let parent = members[k];
                let p = self.agents[parent].clone();
                let boldness = self.mutate(p.boldness);
                let vengefulness = self.mutate(p.vengefulness);
                next.push(Agent {
                    boldness,
                    vengefulness,
                    group: p.group,
                    parent: Some(parent as u32),
                    ..Agent::default()
                });
            }
        }
        self.agents = next;
        self.record();
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
        let g = &self.played;
        let mean = |sel: &dyn Fn(&Agent) -> bool, f: &dyn Fn(&Agent) -> f64| {
            let (s, c) = g
                .iter()
                .filter(|a| sel(a))
                .fold((0.0, 0u32), |(s, c), a| (s + f(a), c + 1));
            if c == 0 {
                0.0
            } else {
                s / f64::from(c)
            }
        };
        let b = |a: &Agent| f64::from(a.boldness) / 7.0;
        let v = |a: &Agent| f64::from(a.vengefulness) / 7.0;
        let (mb, mv) = (mean(&|_| true, &b), mean(&|_| true, &v));
        let groups = self.config.groups.enabled;
        let group = |k: u8| move |a: &Agent| groups && a.group == k;
        if self.trail.len() == TRAIL {
            self.trail.pop_front();
        }
        self.trail.push_back((mb, mv));
        let sum = |f: fn(&Agent) -> u32| g.iter().map(f).sum::<u32>();
        self.stats.push(NormsSnapshot {
            tick: self.tick,
            mean_boldness: mb,
            mean_vengefulness: mv,
            mean_payoff: mean(&|_| true, &|a| a.payoff),
            defections: sum(|a| a.defections),
            punishments: sum(|a| a.punishments),
            metapunishments: sum(|a| a.metapunishments),
            established: u8::from(established(mb, mv)),
            collapsed: u8::from(collapsed(mb, mv)),
            strong_boldness: mean(&group(0), &b),
            weak_boldness: mean(&group(1), &b),
            strong_vengefulness: mean(&group(0), &v),
            weak_vengefulness: mean(&group(1), &v),
            copied_equal: u8::from(self.copied_equal),
        });
    }

    /// The strong group's size under groups (for the strip's gap).
    fn strong(&self) -> Option<usize> {
        self.config
            .groups
            .enabled
            .then_some(self.config.groups.strong as usize)
    }

    fn view(&self, i: usize) -> NormAgentView {
        let a = &self.played[i];
        NormAgentView {
            id: i as u64 + 1,
            group: self.config.groups.enabled.then_some(if a.group == 0 {
                "strong"
            } else {
                "weak"
            }),
            bits: a.bits(),
            boldness: a.boldness,
            vengefulness: a.vengefulness,
            payoff: a.payoff,
            defections: a.defections,
            punished: a.punished,
            punishments: a.punishments,
            metapunishments: a.metapunishments,
            metapunished: a.metapunished,
            parent: a.parent.map(|p| u64::from(p) + 1),
        }
    }

    pub fn inspect(&self, x: u32, y: u32) -> Result<NormsInspection, String> {
        let n = self.played.len();
        let (fw, fh) = frame(n, self.config.groups.enabled);
        if x as usize >= fw || y as usize >= fh {
            return Err(format!("({x}, {y}) is not in the frame"));
        }
        let (cx, cy) = (x as usize, y as usize);
        let mut out = NormsInspection {
            site: NormsCell { x, y },
            level: None,
            agents: Vec::new(),
            agent: None,
        };
        if let Some((b, v)) = level_at(cx, cy) {
            out.level = Some([b, v]);
            out.agents = (0..n)
                .filter(|&i| self.played[i].boldness == b && self.played[i].vengefulness == v)
                .map(|i| self.view(i))
                .collect();
        } else if cx >= STRIP_X {
            out.agent = agent_at_row(cy, n, self.strong()).map(|i| self.view(i));
        }
        Ok(out)
    }

    /// The payoff scale's bounds: this generation's lowest and highest.
    fn payoff_range(&self) -> (f64, f64) {
        self.played
            .iter()
            .fold((f64::INFINITY, f64::NEG_INFINITY), |(lo, hi), a| {
                (lo.min(a.payoff), hi.max(a.payoff))
            })
    }
}

impl Model for NormsWorld {
    fn config(&self) -> ModelConfig {
        ModelConfig::Norms(self.config.clone())
    }

    fn run(&mut self, ticks: u32) {
        NormsWorld::run(self, ticks);
    }

    fn tick(&self) -> u64 {
        self.tick
    }

    fn population(&self) -> usize {
        self.agents.len()
    }

    /// FNV-1a over the tick and the next generation's strategies and groups.
    fn fingerprint(&self) -> u64 {
        let mut h: u64 = 0xcbf2_9ce4_8422_2325;
        let mut eat = |v: u8| {
            h ^= u64::from(v);
            h = h.wrapping_mul(0x0100_0000_01b3);
        };
        for b in self.tick.to_le_bytes() {
            eat(b);
        }
        for a in &self.agents {
            eat(a.boldness);
            eat(a.vengefulness);
            eat(a.group);
        }
        h
    }

    fn size(&self) -> (u32, u32) {
        let (w, h) = frame(self.played.len(), self.config.groups.enabled);
        (w as u32, h as u32)
    }

    fn render(&self, mode: &str, _layer: &str, buf: &mut Vec<u8>) -> Result<(), String> {
        let mode: NormsMode = mode.parse()?;
        let n = self.played.len();
        let (fw, fh) = frame(n, self.config.groups.enabled);
        let mut c = Canvas { buf, wide: 0 };
        c.clear(fw, fh);
        let (lo, hi) = self.payoff_range();
        let payoff_color = |p: f64| {
            let t = if hi > lo { (p - lo) / (hi - lo) } else { 0.5 };
            lerp(COLLAPSED, ESTABLISHED, t)
        };
        // The plane: a faint grid, the agents, G&I's regions, the trail.
        c.rect(0, 0, PLANE, PLANE, PLANE_BG);
        for b in 0..8u8 {
            for v in 0..8u8 {
                let (x, y) = level_cell(b, v);
                c.outline(x, y, LEVEL, LEVEL, GRID);
                let here: Vec<&Agent> = self
                    .played
                    .iter()
                    .filter(|a| a.boldness == b && a.vengefulness == v)
                    .collect();
                if here.is_empty() {
                    continue;
                }
                let color = match mode {
                    NormsMode::Agents => lerp(SPARSE, DENSE, (here.len() as f64 - 1.0) / 5.0),
                    NormsMode::Payoff => {
                        payoff_color(here.iter().map(|a| a.payoff).sum::<f64>() / here.len() as f64)
                    }
                    NormsMode::Group => {
                        let strong = here.iter().filter(|a| a.group == 0).count();
                        lerp(WEAK, STRONG, strong as f64 / here.len() as f64)
                    }
                };
                c.rect(x + 1, y + 1, LEVEL - 2, LEVEL - 2, color);
            }
        }
        c.outline(0, 0, 3 * LEVEL, 3 * LEVEL, ESTABLISHED);
        c.outline(6 * LEVEL, 6 * LEVEL, 2 * LEVEL, 2 * LEVEL, COLLAPSED);
        let len = self.trail.len();
        for (k, &(b, v)) in self.trail.iter().enumerate() {
            let (x, y) = mean_pixel(b, v);
            c.put(
                x,
                y,
                lerp(TRAIL_OLD, TRAIL_NEW, (k + 1) as f64 / len as f64),
            );
        }
        // The strip: boldness and vengefulness bars and a payoff swatch.
        let strong = self.strong();
        for (i, a) in self.played.iter().enumerate() {
            let y = row_top(i, strong);
            let h = ROW - 1;
            c.rect(STRIP_X, y, usize::from(a.boldness) * 4, h, BOLD_BAR);
            c.rect(
                STRIP_X + 30,
                y,
                usize::from(a.vengefulness) * 4,
                h,
                VENGE_BAR,
            );
            let swatch = match mode {
                NormsMode::Group if a.group == 0 => STRONG,
                NormsMode::Group => WEAK,
                _ => payoff_color(a.payoff),
            };
            c.rect(STRIP_X + 60, y, STRIP - 60, h, swatch);
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
        let mut out = String::from(
            "id,group,bits,boldness,vengefulness,payoff,defections,punished,punishments,metapunishments,metapunished,parent\n",
        );
        for i in 0..self.played.len() {
            let v = self.view(i);
            writeln!(
                out,
                "{},{},{},{},{},{},{},{},{},{},{},{}",
                v.id,
                v.group.unwrap_or(""),
                v.bits,
                v.boldness,
                v.vengefulness,
                v.payoff,
                v.defections,
                v.punished,
                v.punishments,
                v.metapunishments,
                v.metapunished,
                v.parent.map_or(String::new(), |p| p.to_string())
            )
            .unwrap();
        }
        out
    }

    fn inspect_json(&self, x: u32, y: u32) -> Result<String, String> {
        let inspection = self.inspect(x, y)?;
        Ok(serde_json::to_string(&inspection).expect("inspection serializes"))
    }

    /// An agent's strip row (its centre).
    fn locate(&self, id: u64) -> Option<(u32, u32)> {
        let i = usize::try_from(id.checked_sub(1)?).ok()?;
        if i >= self.played.len() {
            return None;
        }
        let y = row_top(i, self.strong()) + ROW / 2;
        Some(((STRIP_X + STRIP / 2) as u32, y as u32))
    }

    fn set_config(&mut self, next: ModelConfig) -> Result<(), Vec<FieldError>> {
        let ModelConfig::Norms(next) = next else {
            return Err(wrong_model(ModelKind::Norms, &next));
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

    /// A run stops only at its `stop_at` horizon, and a sweep reads it there.
    fn holds_when_finished(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::norms::config::GroupsConfig;

    fn config(edit: impl FnOnce(&mut NormsConfig)) -> NormsConfig {
        let mut c = NormsConfig::default();
        edit(&mut c);
        c
    }

    fn world(edit: impl FnOnce(&mut NormsConfig)) -> NormsWorld {
        NormsWorld::new(config(edit), 1).unwrap()
    }

    fn set(w: &mut NormsWorld, strategies: &[(u8, u8)]) {
        for (a, &(b, v)) in w.agents.iter_mut().zip(strategies) {
            (a.boldness, a.vengefulness) = (b, v);
        }
    }

    #[test]
    fn only_the_bold_defect_and_only_the_vengeful_punish() {
        let mut w = world(|c| {
            c.agents = 3;
            c.mutation = 0.0;
        });
        set(&mut w, &[(7, 0), (0, 0), (0, 0)]);
        w.step();
        let p = w.played();
        assert_eq!(p[0].defections, 4, "boldness 7/7 defects every round");
        assert_eq!((p[1].defections, p[0].punished), (0, 0));
        assert_eq!(p[0].payoff, 12.0);
        assert_eq!(p[1].payoff, -4.0);
        let mut v = world(|c| {
            c.agents = 3;
            c.mutation = 0.0;
        });
        set(&mut v, &[(7, 7), (0, 7), (0, 7)]);
        v.step();
        let q = v.played();
        assert_eq!(
            q[0].punished,
            q[1].punishments + q[2].punishments,
            "every seen defection punished"
        );
        assert_eq!(
            q[0].payoff,
            4.0 * 3.0 - 9.0 * f64::from(q[0].punished),
            "T per defection, P per punishment"
        );
    }

    #[test]
    fn table_1s_arithmetic() {
        // A hand-built world: agent 0 always defects; agents 1 and 2 have
        // full vengefulness (7/7), so `chance(vengefulness / 7)` always
        // succeeds and every noticed defection is punished. Table 1's
        // arithmetic then follows from the counts each agent actually
        // recorded: T and P for the defector, H (every round, since it is
        // hurt whether or not it notices) and E (once per punishment) for
        // the punishers.
        let c = NormsConfig::default();
        let mut w = world(|c| {
            c.agents = 3;
            c.mutation = 0.0;
        });
        set(&mut w, &[(7, 7), (0, 7), (0, 7)]);
        w.step();
        let p = w.played();
        assert_eq!(
            p[0].defections, c.rounds,
            "boldness 7/7 defects every round"
        );
        assert_eq!(
            p[0].punished,
            p[1].punishments + p[2].punishments,
            "every seen defection is punished (vengefulness 7/7)"
        );
        assert_eq!(
            p[0].payoff,
            f64::from(p[0].defections) * c.temptation + f64::from(p[0].punished) * c.punishment,
            "T per defection, P per punishment"
        );
        for punisher in [1, 2] {
            assert_eq!(
                p[punisher].payoff,
                f64::from(p[0].defections) * c.hurt
                    + f64::from(p[punisher].punishments) * c.enforcement,
                "H per round hurt, E per punishment"
            );
        }
    }

    #[test]
    fn metanorms_fall_only_on_those_who_saw_and_did_not_punish() {
        let mut w = world(|c| {
            c.agents = 4;
            c.mutation = 0.0;
            c.metanorms = true;
        });
        // Agent 0 always defects, and — critically for this test — has full
        // vengefulness (7/7): if the metapunishment loop's `k != i` exclusion
        // were ever dropped, the defector would deterministically metapunish
        // (or be metapunished) whenever it were wrongly admitted as a
        // witness of its own defection, instead of a boldness-0 defector
        // whose vengefulness-0 would mask the bug. Agent 1 never punishes;
        // 2 and 3 always punish.
        set(&mut w, &[(7, 7), (0, 0), (0, 7), (0, 7)]);
        w.step();
        let p = w.played();
        assert_eq!(
            p[0].metapunished, 0,
            "the defector is excluded as its own witness (k != i)"
        );
        assert!(p[1].metapunished > 0, "the non-punisher is metapunished");
        assert_eq!(
            p[2].metapunished + p[3].metapunished,
            0,
            "punishers are not metapunished"
        );
        assert_eq!(
            p[0].metapunishments, 0,
            "the defector is excluded as its own metapunisher (k != i)"
        );
        assert_eq!(
            p[2].metapunishments + p[3].metapunishments,
            p[1].metapunished
        );
        let mut off = world(|c| {
            c.agents = 4;
            c.mutation = 0.0;
        });
        set(&mut off, &[(7, 7), (0, 0), (0, 7), (0, 7)]);
        off.step();
        assert_eq!(off.played()[1].metapunished, 0);
    }

    #[test]
    fn axelrods_selection_follows_the_standard_deviation() {
        let mut w = world(|c| {
            c.agents = 5;
            c.refill = Refill::Ranked;
        });
        for (a, p) in w.agents.iter_mut().zip([10.0, 0.0, 0.0, 0.0, -10.0]) {
            a.payoff = p;
        }
        // Mean 0, s.d. √40 ≈ 6.3: agent 0 twice, agent 4 none, the rest once.
        let parents = w.select(&[0, 1, 2, 3, 4]);
        let count = |k| parents.iter().filter(|&&p| p == k).count();
        assert_eq!((count(0), count(1), count(4)), (2, 1, 0));
        assert_eq!(parents.len(), 5);
    }

    #[test]
    fn ranked_refill_removes_the_worst_and_duplicates_the_best() {
        // Average selection on [10, 10, 0, -20] (mean 0) copies the three
        // payoffs at or above the mean twice each: six offspring for a
        // population of four. Ranked refill must remove two of them, and
        // remove agent 2's copies (payoff 0, the worst that still
        // qualified) rather than agent 0's or agent 1's (payoff 10 each).
        let mut more = world(|c| {
            c.agents = 4;
            c.selection = Selection::Average;
            c.refill = Refill::Ranked;
        });
        for (a, p) in more.agents.iter_mut().zip([10.0, 10.0, 0.0, -20.0]) {
            a.payoff = p;
        }
        assert_eq!(
            more.select(&[0, 1, 2, 3]),
            [0, 0, 1, 1],
            "the worst qualifying parent's copies are removed first"
        );

        // Axelrod selection on the same payoffs (mean 0, s.d. ≈ 12.2) is
        // within one s.d. of the mean for agents 0–2, so each gets exactly
        // one offspring: three for a population of four. Ranked refill must
        // duplicate one, and duplicate the best (agent 1, payoff 10) rather
        // than agent 0 or agent 2.
        let mut fewer = world(|c| {
            c.agents = 4;
            c.refill = Refill::Ranked;
        });
        for (a, p) in fewer.agents.iter_mut().zip([10.0, 10.0, 0.0, -20.0]) {
            a.payoff = p;
        }
        assert_eq!(
            fewer.select(&[0, 1, 2, 3]),
            [2, 0, 1, 1],
            "the best surviving parent is duplicated to make up the rest"
        );
    }

    #[test]
    fn when_every_payoff_ties_the_reading_decides() {
        let mut keep = world(|c| {
            c.agents = 6;
            c.all_equal = AllEqual::Keep;
        });
        assert_eq!(keep.select(&[0, 1, 2, 3, 4, 5]), [0, 1, 2, 3, 4, 5]);
        let mut drift = world(|c| c.agents = 6);
        let mut changed = false;
        for _ in 0..20 {
            let p = drift.select(&[0, 1, 2, 3, 4, 5]);
            assert_eq!(p.len(), 6);
            changed |= p != [0, 1, 2, 3, 4, 5];
        }
        assert!(changed, "drift resamples");
        assert!(drift.copied_equal);
    }

    #[test]
    fn the_other_selection_rules_favor_higher_payoffs() {
        for selection in [
            Selection::Tournament,
            Selection::Roulette,
            Selection::Average,
        ] {
            let mut w = world(|c| {
                c.agents = 10;
                c.selection = selection;
            });
            for (k, a) in w.agents.iter_mut().enumerate() {
                a.payoff = k as f64;
            }
            let members: Vec<usize> = (0..10).collect();
            let mut top = 0;
            for _ in 0..200 {
                let p = w.select(&members);
                assert_eq!(p.len(), 10, "{selection:?}");
                top += p.iter().filter(|&&k| k >= 5).count();
            }
            assert!(
                top > 1200,
                "{selection:?}: {top} of 2000 from the better half"
            );
        }
        let mut avg = world(|c| {
            c.agents = 4;
            c.selection = Selection::Average;
            c.refill = Refill::Ranked;
        });
        for (a, p) in avg.agents.iter_mut().zip([1.0, 1.0, 0.0, 0.0]) {
            a.payoff = p;
        }
        assert_eq!(
            avg.select(&[0, 1, 2, 3]),
            [0, 0, 1, 1],
            "above the mean: two each"
        );
    }

    #[test]
    fn mutation_flips_bits() {
        let mut none = world(|c| c.mutation = 0.0);
        assert_eq!(none.mutate(5), 5);
        let mut all = world(|c| c.mutation = 1.0);
        assert_eq!(all.mutate(5), 2, "101 → 010");
    }

    #[test]
    fn groups_select_within_themselves_and_the_strong_are_punished_less() {
        let mut w = world(|c| {
            c.groups = GroupsConfig {
                enabled: true,
                ..GroupsConfig::default()
            };
        });
        assert_eq!(w.agents().len(), 30);
        assert_eq!((w.punishment_of(0), w.punishment_of(25)), (-3.0, -9.0));
        w.run(20);
        let strong = w.agents().iter().filter(|a| a.group == 0).count();
        assert_eq!(strong, 20);
        assert!(w.agents()[..20].iter().all(|a| a.group == 0));
        let s = w.stats.latest().unwrap();
        assert!(s.strong_boldness >= 0.0 && s.weak_boldness >= 0.0);
    }

    #[test]
    fn statistics_follow_the_generation_that_played() {
        let mut w = world(|c| c.mutation = 0.0);
        set(&mut w, &[(7, 0); 20]);
        w.step();
        let s = w.stats.latest().unwrap();
        assert_eq!((s.mean_boldness, s.mean_vengefulness), (1.0, 0.0));
        assert_eq!((s.collapsed, s.established, s.defections), (1, 0, 80));
        assert_eq!(s.punishments, 0);
        assert_eq!(
            w.stats.history()[0].defections,
            0,
            "no play before generation 1"
        );
    }

    #[test]
    fn the_view_draws_plane_and_strip_and_inspect_finds_both() {
        let mut w = world(|c| c.groups.enabled = true);
        w.run(3);
        let (fw, fh) = frame(30, true);
        assert_eq!(Model::size(&w), (fw as u32, fh as u32));
        let mut buf = Vec::new();
        for mode in ["agents", "payoff", "group"] {
            w.render(mode, "", &mut buf).unwrap();
            assert_eq!(buf.len(), fw * fh * 4);
        }
        assert!(w.render("wealth", "", &mut buf).is_err());
        let a = &w.played()[0];
        let (x, y) = level_cell(a.boldness, a.vengefulness);
        let v = w.inspect((x + 5) as u32, (y + 5) as u32).unwrap();
        assert_eq!(v.level, Some([a.boldness, a.vengefulness]));
        assert!(v.agents.iter().any(|g| g.id == 1));
        let row = w.inspect((STRIP_X + 1) as u32, 1).unwrap();
        assert_eq!(row.agent.as_ref().unwrap().id, 1);
        assert_eq!(row.agent.unwrap().group, Some("strong"));
        let gap = w.inspect((STRIP_X + 1) as u32, 81).unwrap();
        assert!(gap.agent.is_none() && gap.level.is_none());
        assert_eq!(
            Model::locate(&w, 21),
            Some(((STRIP_X + STRIP / 2) as u32, 86))
        );
        assert!(w.inspect(fw as u32, 0).is_err());
    }

    #[test]
    fn keyframes_restore_strategies_and_the_trail() {
        let mut any =
            crate::model::ModelWorld::new(ModelConfig::Norms(config(|c| c.metanorms = true)), 6)
                .unwrap();
        any.model_mut().run(5);
        let cp = any.checkpoint().unwrap();
        let print = any.model().fingerprint();
        let mut before = Vec::new();
        any.model().render("agents", "", &mut before).unwrap();
        any.model_mut().run(10);
        any.restore(&cp).unwrap();
        assert_eq!(any.model().fingerprint(), print);
        let mut after = Vec::new();
        any.model().render("agents", "", &mut after).unwrap();
        assert_eq!(before, after);
        assert_eq!(any.model().series("mean_boldness").unwrap().len(), 6);
    }

    #[test]
    fn live_edits_apply_and_the_population_waits_for_reset() {
        let mut w = world(|_| {});
        let next = config(|c| {
            c.metanorms = true;
            c.selection = Selection::Tournament;
            c.mutation = 0.001;
            c.meta_punishment = -0.9;
        });
        Model::set_config(&mut w, ModelConfig::Norms(next.clone())).unwrap();
        w.step();
        let e = Model::set_config(
            &mut w,
            ModelConfig::Norms(NormsConfig { agents: 21, ..next }),
        )
        .unwrap_err();
        assert_eq!(e[0].field, "agents");
    }

    #[test]
    fn the_stop_ends_the_run() {
        let mut w = world(|c| c.stop_at = 7);
        w.run(100);
        assert_eq!(w.tick, 7);
        assert!(Model::finished(&w));
    }

    #[test]
    fn degenerate_configs_run_without_panicking() {
        for c in [
            config(|c| c.agents = 2),
            config(|c| {
                c.agents = 2;
                c.metanorms = true;
            }),
            config(|c| c.rounds = 1),
            config(|c| c.mutation = 0.0),
            config(|c| c.mutation = 1.0),
            config(|c| {
                c.selection = Selection::Roulette;
                c.refill = Refill::Ranked;
            }),
            config(|c| {
                c.groups = GroupsConfig {
                    enabled: true,
                    strong: 2,
                    weak: 2,
                    ..GroupsConfig::default()
                };
                c.metanorms = true;
            }),
        ] {
            let mut w = NormsWorld::new(c.clone(), 1).unwrap();
            w.run(50);
            let s = w.stats.latest().unwrap();
            assert!((0.0..=1.0).contains(&s.mean_boldness), "{c:?}");
            let mut buf = Vec::new();
            w.render("payoff", "", &mut buf).unwrap();
        }
    }
}
