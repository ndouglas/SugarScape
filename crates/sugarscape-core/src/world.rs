//! The world: lattice, agents, and the tick loop.

use std::collections::BTreeMap;

use rand::seq::SliceRandom;

use crate::agent::{Agent, AgentId, DiseaseId, Tribe};
use crate::bits::Bits;
use crate::config::{Config, FieldError, Placement, MAX_GOODS};
use crate::geometry::{Pos, Torus};
use crate::landscape::{self, Site};
use crate::rng::{self, SimRng};
use crate::rules;
use crate::stats::{Snapshot, Stats};

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

/// One exchange under rule T: `buyer` received `sugar` sugar and paid
/// `sugar × price` spice to `seller`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Trade {
    pub buyer: AgentId,
    pub seller: AgentId,
    pub price: f64,
    pub sugar: f64,
}

/// One infection: `infector` gave `disease` to `infected` (`None` for an
/// outbreak).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Infection {
    pub infector: Option<AgentId>,
    pub infected: AgentId,
    pub disease: DiseaseId,
}

pub type LoanId = u64;

/// A sugar loan under rule L: `due` sugar owed at `due_tick`, written for
/// `duration` ticks at `rate` percent per tick (the terms travel with it).
#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize)]
pub struct Loan {
    pub id: LoanId,
    pub lender: AgentId,
    pub borrower: AgentId,
    pub principal: f64,
    pub due: f64,
    pub due_tick: u64,
    pub duration: u32,
    pub rate: f64,
}

/// What happened during the current (or last completed) tick.
#[derive(Clone, Debug, Default)]
pub struct TickEvents {
    pub births: u32,
    pub deaths: Vec<Death>,
    pub trades: Vec<Trade>,
    pub loans_made: u32,
    pub amount_lent: f64,
    pub defaults: u32,
    pub infections: Vec<Infection>,
}

pub struct World {
    pub config: Config,
    pub torus: Torus,
    /// Completed ticks.
    pub tick: u64,
    pub sites: Vec<Site>,
    /// True when capacities were supplied or painted rather than generated from the configured landscape.
    pub landscape_edited: bool,
    /// Chapter V's master list of diseases; a disease's id is its index.
    pub diseases: Vec<Bits>,
    agents: BTreeMap<AgentId, Agent>,
    occupancy: Vec<Option<AgentId>>,
    pub(crate) rng: SimRng,
    next_id: AgentId,
    pub(crate) events: TickEvents,
    pub stats: Stats,
    loans: BTreeMap<LoanId, Loan>,
    next_loan_id: LoanId,
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
        let n = config.goods.len();
        let mut maps: Vec<Vec<f64>> = config
            .goods
            .iter()
            .map(|g| landscape::generate(&g.map, config.width, config.height))
            .collect();
        match capacities {
            Some(c) if c.len() != torus.len() => {
                return Err(vec![FieldError::new(
                    "landscape",
                    format!("expected {} capacities, got {}", torus.len(), c.len()),
                )])
            }
            Some(c) => maps[0] = c.to_vec(),
            None => {}
        }
        let sites = (0..torus.len())
            .map(|s| {
                let mut caps = [0.0; MAX_GOODS];
                for (slot, map) in caps.iter_mut().zip(&maps) {
                    *slot = map[s];
                }
                Site::full(&caps[..n])
            })
            .collect();
        let mut world = World {
            torus,
            tick: 0,
            sites,
            landscape_edited: capacities.is_some(),
            diseases: Vec::new(),
            agents: BTreeMap::new(),
            occupancy: vec![None; torus.len()],
            rng: rng::seeded(seed),
            next_id: 1,
            events: TickEvents::default(),
            stats: Stats::default(),
            loans: BTreeMap::new(),
            next_loan_id: 1,
            config,
        };
        if world.config.disease.enabled {
            world.diseases = rules::disease::initial_list(&world.config.disease, &mut world.rng);
        }
        world.populate();
        world.stats.push(Snapshot::of(&world));
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
            rules::disease::endow(self, &mut agent);
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

    pub fn loans(&self) -> impl Iterator<Item = &Loan> {
        self.loans.values()
    }

    /// Records a loan of `principal` on the current credit terms (no transfer).
    pub(crate) fn originate_loan(
        &mut self,
        lender: AgentId,
        borrower: AgentId,
        principal: f64,
    ) -> LoanId {
        let c = self.config.credit;
        self.originate_loan_on(lender, borrower, principal, c.duration, c.rate)
    }

    /// Records a loan of `principal` for `duration` ticks at `rate` percent.
    pub(crate) fn originate_loan_on(
        &mut self,
        lender: AgentId,
        borrower: AgentId,
        principal: f64,
        duration: u32,
        rate: f64,
    ) -> LoanId {
        let id = self.next_loan_id;
        self.next_loan_id += 1;
        let factor = 1.0 + rate / 100.0 * f64::from(duration);
        self.loans.insert(
            id,
            Loan {
                id,
                lender,
                borrower,
                principal,
                due: principal * factor,
                due_tick: self.tick + u64::from(duration),
                duration,
                rate,
            },
        );
        id
    }

    pub(crate) fn remove_loan(&mut self, id: LoanId) -> Option<Loan> {
        self.loans.remove(&id)
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
        let n = self.config.goods.len();
        let m = self.config.pollution.pollutants.len();
        let foresight = self.config.foresight.enabled;
        let disease = self.config.disease.enabled;
        eat(self.tick);
        for s in &self.sites {
            eat(s.resource[0].to_bits());
            eat(s.capacity[0].to_bits());
            eat(s.pollution[0].to_bits());
            for i in 1..n {
                eat(s.resource[i].to_bits());
                eat(s.capacity[i].to_bits());
            }
            for k in 1..m {
                eat(s.pollution[k].to_bits());
            }
        }
        for a in self.agents.values() {
            eat(a.id);
            eat((u64::from(a.pos.x) << 32) | u64::from(a.pos.y));
            eat(a.holdings[0].to_bits());
            eat(u64::from(a.age));
            eat(a.tags.bits());
            for i in 1..n {
                eat(a.holdings[i].to_bits());
            }
            if foresight {
                eat(u64::from(a.foresight));
            }
            if disease {
                eat(u64::from(a.immune.len()));
                eat(a.immune.bits());
                eat(u64::from(a.immune_genome.len()));
                eat(a.immune_genome.bits());
                eat(a.diseases.len() as u64);
                for &d in &a.diseases {
                    eat(u64::from(d));
                }
            }
        }
        if disease {
            for d in &self.diseases {
                eat(u64::from(d.len()));
                eat(d.bits());
            }
        }
        for l in self.loans.values() {
            eat(l.id);
            eat(l.due.to_bits());
            eat(l.due_tick);
            eat(u64::from(l.duration));
            eat(l.rate.to_bits());
        }
        h
    }

    /// Takes an agent off the grid with no death event and no inheritance.
    pub(crate) fn remove(&mut self, id: AgentId) -> Option<Agent> {
        let agent = self.agents.remove(&id)?;
        let i = self.torus.index(agent.pos);
        self.occupancy[i] = None;
        if !self.loans.is_empty() {
            self.loans.retain(|_, l| l.lender != id && l.borrower != id);
        }
        Some(agent)
    }

    /// Removes an agent from play and records its death. With rule I on, its
    /// remaining sugar is split equally among its living children. A dead
    /// lender's outstanding claims pass to its living children as well.
    pub(crate) fn kill(&mut self, id: AgentId, cause: DeathCause) -> Option<Agent> {
        let claims: Vec<Loan> = if self.config.inheritance.enabled {
            self.loans
                .values()
                .filter(|l| l.lender == id)
                .copied()
                .collect()
        } else {
            Vec::new()
        };
        let agent = self.remove(id)?;
        self.events.deaths.push(Death {
            id,
            tribe: agent.tribe(),
            cause,
        });
        if self.config.inheritance.enabled {
            self.bequeath(&agent);
        }
        if !claims.is_empty() {
            self.pass_on_claims(&agent, claims);
        }
        Some(agent)
    }

    fn bequeath(&mut self, agent: &Agent) {
        let heirs: Vec<AgentId> = agent
            .children
            .iter()
            .copied()
            .filter(|c| self.agents.contains_key(c))
            .collect();
        let n = heirs.len() as f64;
        let sugar = if agent.holdings[0] > 0.0 {
            agent.holdings[0] / n
        } else {
            0.0
        };
        let spice = if agent.holdings[1] > 0.0 {
            agent.holdings[1] / n
        } else {
            0.0
        };
        if heirs.is_empty() || (sugar == 0.0 && spice == 0.0) {
            return;
        }
        for heir in heirs {
            let h = self.agents.get_mut(&heir).expect("living heir");
            h.holdings[0] += sugar;
            h.holdings[1] += spice;
        }
    }

    /// Splits a dead lender's claims equally among its living children; a
    /// child who is the borrower has its own share forgiven.
    fn pass_on_claims(&mut self, lender: &Agent, claims: Vec<Loan>) {
        let heirs: Vec<AgentId> = lender
            .children
            .iter()
            .copied()
            .filter(|c| self.agents.contains_key(c))
            .collect();
        if heirs.is_empty() {
            return;
        }
        let n = heirs.len() as f64;
        for claim in claims {
            if !self.agents.contains_key(&claim.borrower) {
                continue;
            }
            for &heir in &heirs {
                if heir == claim.borrower {
                    // A claim on oneself is forgiven.
                    continue;
                }
                let id = self.next_loan_id;
                self.next_loan_id += 1;
                self.loans.insert(
                    id,
                    Loan {
                        id,
                        lender: heir,
                        principal: claim.principal / n,
                        due: claim.due / n,
                        ..claim
                    },
                );
            }
        }
    }

    /// One tick: every living agent takes a turn in a fresh random order
    /// (agents born or killed during the tick are skipped), then the
    /// environment updates and everyone ages.
    pub fn step(&mut self) {
        self.events = TickEvents::default();
        self.apply_schedule();
        if self.config.disease.enabled {
            rules::disease::outbreaks(self);
        }
        let mut order = self.agent_ids();
        order.shuffle(&mut self.rng);
        for id in order {
            if self.agents.contains_key(&id) {
                rules::agent_turn(self, id);
            }
        }
        if !self.loans.is_empty() {
            rules::credit::settle(self);
        }
        rules::growback::apply(self);
        rules::pollution::diffuse(self);
        rules::replacement::apply(self);
        for agent in self.agents.values_mut() {
            agent.age += 1;
        }
        self.tick += 1;
        let snapshot = Snapshot::of(self);
        self.stats.push(snapshot);
    }

    pub fn run(&mut self, ticks: u32) {
        for _ in 0..ticks {
            self.step();
        }
    }

    /// Applies scheduled changes due at the tick about to run. Entries not yet
    /// fired were validated against the config that reaches them (`World::new`
    /// checks the whole schedule; `set_config` checks entries with
    /// `tick >= self.tick`), so failures should not happen; any are ignored.
    fn apply_schedule(&mut self) {
        let due: Vec<_> = self
            .config
            .schedule
            .iter()
            .filter(|c| c.tick == self.tick)
            .cloned()
            .collect();
        for change in due {
            if let Ok(next) = self.config.apply_change(&change) {
                self.config = next;
            }
        }
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
    use crate::config::{Config, Good, Placement};

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
            assert!((1..=4).contains(&a.metabolism[0]));
            assert!((5.0..=25.0).contains(&a.holdings[0]));
            assert_eq!(a.holdings[0], a.initial[0]);
        }
        assert_eq!(
            w.site(Pos::new(37, 5)).resource[0],
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
        assert_eq!(w.site(Pos::new(0, 0)).capacity[0], 2.0);
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

    #[test]
    fn step_is_deterministic_for_a_seed() {
        let mut a = World::new(Config::default(), 42).unwrap();
        let mut b = World::new(Config::default(), 42).unwrap();
        a.run(50);
        b.run(50);
        assert_eq!(a.tick, 50);
        assert_eq!(a.fingerprint(), b.fingerprint());
    }

    #[test]
    fn population_declines_toward_carrying_capacity() {
        let mut w = World::new(Config::default(), 5).unwrap();
        w.run(100);
        assert!(
            w.population() < 400 && w.population() > 100,
            "got {}",
            w.population()
        );
        for a in w.agents() {
            assert!(a.holdings[0] > 0.0);
            assert_eq!(a.age, 100, "immortal first generation ages every tick");
        }
    }

    #[test]
    fn scheduled_changes_apply_when_their_tick_is_reached() {
        use crate::config::ScheduledChange;
        let mut c = crate::testkit::blank_config(10, 10);
        c.schedule = vec![ScheduledChange {
            tick: 2,
            set: [("pollution.enabled".to_string(), serde_json::json!(true))]
                .into_iter()
                .collect(),
        }];
        let mut w = World::new(c, 1).unwrap();
        w.step(); // tick 0 → 1
        assert!(!w.config.pollution.enabled);
        w.step(); // tick 1 → 2
        assert!(!w.config.pollution.enabled);
        w.step(); // starts at tick 2: applied
        assert!(w.config.pollution.enabled);
        assert_eq!(w.config.schedule.len(), 1, "the schedule itself is kept");
    }

    #[test]
    fn disease_worlds_draw_a_list_and_endow_agents() {
        let mut c = Config::default();
        c.disease.enabled = true;
        let w = World::new(c, 1).unwrap();
        assert_eq!(w.diseases.len(), 10);
        assert!(w.diseases.iter().all(|d| (1..=10).contains(&d.len())));
        let mut carried = 0;
        for a in w.agents() {
            assert_eq!(a.immune.len(), 50);
            assert!(a.diseases.len() <= 4);
            let mut ids = a.diseases.clone();
            ids.sort_unstable();
            ids.dedup();
            assert_eq!(ids.len(), a.diseases.len(), "distinct diseases");
            for &d in &a.diseases {
                assert!(
                    !a.immune.contains(&w.diseases[d as usize]),
                    "never starts with a disease it is immune to"
                );
            }
            carried += a.diseases.len();
        }
        assert!(carried > 0, "some agents start sick");
        assert!(World::new(Config::default(), 1)
            .unwrap()
            .diseases
            .is_empty());
    }

    #[test]
    fn fingerprint_covers_disease_state_only_when_disease_is_on() {
        let mut w = crate::testkit::blank_world(5, 5);
        let id = crate::testkit::spawn(&mut w, 1, 1);
        let before = w.fingerprint();
        w.agent_mut(id).unwrap().diseases.push(0);
        w.agent_mut(id).unwrap().immune.flip(0);
        assert_eq!(w.fingerprint(), before, "ignored while disease is off");
        w.config.disease.enabled = true;
        let on = w.fingerprint();
        w.agent_mut(id).unwrap().immune.flip(1);
        assert_ne!(w.fingerprint(), on, "immune strings are hashed");
        let on = w.fingerprint();
        w.diseases.push(crate::bits::Bits::parse("101").unwrap());
        assert_ne!(w.fingerprint(), on, "the disease list is hashed");
    }

    #[test]
    fn goods_are_stored_in_per_good_slots() {
        let mut c = Config::default();
        c.add_good(Good::spice());
        let w = World::new(c, 1).unwrap();
        assert_eq!(
            w.site(Pos::new(37, 5)).capacity[0],
            4.0,
            "sugar's northeast peak"
        );
        assert_eq!(
            w.site(Pos::new(12, 5)).capacity[1],
            4.0,
            "spice's northwest peak"
        );
        assert!(w
            .sites
            .iter()
            .all(|s| s.resource[2..].iter().all(|&x| x == 0.0)));
        for a in w.agents() {
            assert_eq!(a.holdings[..2], a.initial[..2]);
            assert!((1..=4).contains(&a.metabolism[1]));
            assert!(a.holdings[2..].iter().all(|&x| x == 0.0));
        }
    }
}
