//! The world: lattice, agents, and the tick loop.

use std::collections::BTreeMap;

use rand::seq::SliceRandom;

use crate::agent::{Agent, AgentId, Tribe};
use crate::config::{Config, FieldError, Placement};
use crate::geometry::{Pos, Torus};
use crate::landscape::{self, Site};
use crate::rng::{self, SimRng};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DeathCause {
    Starvation,
    OldAge,
    Combat,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Death {
    pub id: AgentId,
    pub tribe: Tribe,
    pub cause: DeathCause,
}

/// What happened during the current (or last completed) tick.
#[derive(Clone, Debug, Default)]
pub struct TickEvents {
    pub births: u32,
    pub deaths: Vec<Death>,
}

pub struct World {
    pub config: Config,
    pub torus: Torus,
    /// Completed ticks.
    pub tick: u64,
    pub sites: Vec<Site>,
    /// True once the capacities differ from the configured landscape.
    pub landscape_edited: bool,
    agents: BTreeMap<AgentId, Agent>,
    occupancy: Vec<Option<AgentId>>,
    pub(crate) rng: SimRng,
    next_id: AgentId,
    pub(crate) events: TickEvents,
}

impl World {
    pub fn new(config: Config, seed: u64) -> Result<Self, Vec<FieldError>> {
        Self::with_capacities(config, seed, None)
    }

    /// Like `new`, but with explicit row-major capacities (a painted map).
    pub fn with_capacities(
        config: Config,
        seed: u64,
        capacities: Option<&[f64]>,
    ) -> Result<Self, Vec<FieldError>> {
        config.validate()?;
        let torus = Torus::new(config.width, config.height);
        let caps = match capacities {
            Some(c) if c.len() != torus.len() => {
                return Err(vec![FieldError::new(
                    "landscape",
                    format!("expected {} capacities, got {}", torus.len(), c.len()),
                )])
            }
            Some(c) => c.to_vec(),
            None => landscape::capacities(&config.landscape, config.width, config.height),
        };
        let mut world = World {
            torus,
            tick: 0,
            sites: caps.into_iter().map(Site::full).collect(),
            landscape_edited: capacities.is_some(),
            agents: BTreeMap::new(),
            occupancy: vec![None; torus.len()],
            rng: rng::seeded(seed),
            next_id: 1,
            events: TickEvents::default(),
            config,
        };
        world.populate();
        Ok(world)
    }

    fn populate(&mut self) {
        let n = self.config.population as usize;
        let (w, h) = (self.config.width, self.config.height);
        match self.config.placement {
            Placement::Random => {
                let cells = (0..self.torus.len()).map(|i| self.torus.pos(i)).collect();
                self.place(cells, n, None);
            }
            Placement::Block {
                x,
                y,
                width,
                height,
            } => {
                self.place(rect(x, y, width, height), n, None);
            }
            Placement::Tribes { size } => {
                let blues = n.div_ceil(2);
                self.place(rect(0, h - size, size, size), blues, Some(Tribe::Blue));
                self.place(rect(w - size, 0, size, size), n - blues, Some(Tribe::Red));
            }
        }
    }

    fn place(&mut self, mut cells: Vec<Pos>, n: usize, tribe: Option<Tribe>) {
        cells.shuffle(&mut self.rng);
        for pos in cells.into_iter().take(n) {
            let mut agent = Agent::random(&self.config, pos, self.tick, &mut self.rng);
            if let Some(t) = tribe {
                agent.tags = agent.tags.forced_to(t);
            }
            self.insert_agent(agent)
                .expect("placement cells are distinct and empty");
        }
    }

    pub fn agent(&self, id: AgentId) -> Option<&Agent> {
        self.agents.get(&id)
    }

    pub fn agent_mut(&mut self, id: AgentId) -> Option<&mut Agent> {
        self.agents.get_mut(&id)
    }

    /// Living agents in id order.
    pub fn agents(&self) -> impl Iterator<Item = &Agent> {
        self.agents.values()
    }

    #[allow(dead_code)]
    pub(crate) fn agent_ids(&self) -> Vec<AgentId> {
        self.agents.keys().copied().collect()
    }

    pub fn population(&self) -> usize {
        self.agents.len()
    }

    pub fn occupant(&self, pos: Pos) -> Option<AgentId> {
        self.occupancy[self.torus.index(pos)]
    }

    pub fn agent_at(&self, pos: Pos) -> Option<&Agent> {
        self.occupant(pos).and_then(|id| self.agents.get(&id))
    }

    pub fn is_occupied(&self, pos: Pos) -> bool {
        self.occupant(pos).is_some()
    }

    pub fn site(&self, pos: Pos) -> &Site {
        &self.sites[self.torus.index(pos)]
    }

    pub fn site_mut(&mut self, pos: Pos) -> &mut Site {
        let i = self.torus.index(pos);
        &mut self.sites[i]
    }

    pub fn empty_sites(&self) -> Vec<Pos> {
        (0..self.torus.len())
            .filter(|&i| self.occupancy[i].is_none())
            .map(|i| self.torus.pos(i))
            .collect()
    }

    /// Adds `agent` at its position with a fresh id.
    pub fn insert_agent(&mut self, mut agent: Agent) -> Result<AgentId, String> {
        let i = self.torus.index(agent.pos);
        if self.occupancy[i].is_some() {
            return Err(format!(
                "site ({}, {}) is occupied",
                agent.pos.x, agent.pos.y
            ));
        }
        let id = self.next_id;
        self.next_id += 1;
        agent.id = id;
        self.occupancy[i] = Some(id);
        self.agents.insert(id, agent);
        Ok(id)
    }

    #[allow(dead_code)]
    pub(crate) fn move_agent(&mut self, id: AgentId, to: Pos) {
        let from = self.agents[&id].pos;
        if from == to {
            return;
        }
        let (fi, ti) = (self.torus.index(from), self.torus.index(to));
        assert!(self.occupancy[ti].is_none(), "move onto occupied site");
        self.occupancy[fi] = None;
        self.occupancy[ti] = Some(id);
        self.agents.get_mut(&id).expect("live agent").pos = to;
    }

    pub fn events(&self) -> &TickEvents {
        &self.events
    }

    /// FNV-1a hash of the full dynamic state, for determinism checks.
    pub fn fingerprint(&self) -> u64 {
        let mut h: u64 = 0xcbf2_9ce4_8422_2325;
        let mut eat = |v: u64| {
            for b in v.to_le_bytes() {
                h ^= u64::from(b);
                h = h.wrapping_mul(0x0100_0000_01b3);
            }
        };
        eat(self.tick);
        for s in &self.sites {
            eat(s.sugar.to_bits());
            eat(s.capacity.to_bits());
            eat(s.pollution.to_bits());
        }
        for a in self.agents.values() {
            eat(a.id);
            eat((u64::from(a.pos.x) << 32) | u64::from(a.pos.y));
            eat(a.sugar.to_bits());
            eat(u64::from(a.age));
            eat(a.tags.bits());
        }
        h
    }
}

fn rect(x: u32, y: u32, width: u32, height: u32) -> Vec<Pos> {
    (y..y + height)
        .flat_map(|yy| (x..x + width).map(move |xx| Pos::new(xx, yy)))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{Config, Placement};

    #[test]
    fn default_world_places_400_agents_on_distinct_sites() {
        let w = World::new(Config::default(), 1).unwrap();
        assert_eq!(w.population(), 400);
        let mut positions: Vec<Pos> = w.agents().map(|a| a.pos).collect();
        positions.sort();
        positions.dedup();
        assert_eq!(positions.len(), 400);
        for a in w.agents() {
            assert_eq!(w.occupant(a.pos), Some(a.id));
            assert!((1..=6).contains(&a.vision));
            assert!((1..=4).contains(&a.metabolism));
            assert!((5.0..=25.0).contains(&a.sugar));
            assert_eq!(a.sugar, a.initial_sugar);
        }
        assert_eq!(
            w.site(Pos::new(37, 5)).sugar,
            4.0,
            "sugar starts at capacity"
        );
    }

    #[test]
    fn tribes_placement_puts_blues_southwest_and_reds_northeast() {
        let c = Config {
            placement: Placement::Tribes { size: 20 },
            ..Config::default()
        };
        let w = World::new(c, 3).unwrap();
        assert_eq!(w.population(), 400);
        for a in w.agents() {
            match a.tribe() {
                Tribe::Blue => assert!(a.pos.x < 20 && a.pos.y >= 30),
                Tribe::Red => assert!(a.pos.x >= 30 && a.pos.y < 20),
            }
        }
    }

    #[test]
    fn same_seed_same_world_different_seed_different_world() {
        let a = World::new(Config::default(), 9).unwrap();
        let b = World::new(Config::default(), 9).unwrap();
        let c = World::new(Config::default(), 10).unwrap();
        assert_eq!(a.fingerprint(), b.fingerprint());
        assert_ne!(a.fingerprint(), c.fingerprint());
    }

    #[test]
    fn custom_capacities_must_match_grid() {
        let err = World::with_capacities(Config::default(), 1, Some(&[1.0; 10]))
            .err()
            .unwrap();
        assert_eq!(err[0].field, "landscape");
        let w = World::with_capacities(Config::default(), 1, Some(&[2.0; 2500])).unwrap();
        assert!(w.landscape_edited);
        assert_eq!(w.site(Pos::new(0, 0)).capacity, 2.0);
    }

    #[test]
    fn invalid_config_is_rejected() {
        let c = Config {
            population: 10_000,
            ..Config::default()
        };
        assert!(World::new(c, 1).is_err());
    }

    #[test]
    fn insert_rejects_occupied_site() {
        let mut w = crate::testkit::blank_world(5, 5);
        crate::testkit::spawn(&mut w, 2, 2);
        let clone = w.agent_at(Pos::new(2, 2)).unwrap().clone();
        assert!(w.insert_agent(clone).is_err());
    }
}
