//! Interactive edits and inspection used by the playground UI.

use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

use crate::agent::{Agent, AgentId, DiseaseId, Sex, Tribe};
use crate::config::{Config, FieldError};
use crate::geometry::Pos;
use crate::minds::memory::believed_ripe;
use crate::rules;
use crate::world::{LoanId, World};

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
pub struct AgentOverrides {
    pub vision: Option<u32>,
    pub metabolism: Option<u32>,
    pub sugar: Option<f64>,
    pub sex: Option<Sex>,
    pub tribe: Option<Tribe>,
    /// Good 1 (spice) held and needed; two-good worlds only.
    pub spice: Option<f64>,
    pub spice_metabolism: Option<u32>,
    /// Age at placement, and a birth endowment of sugar other than what it
    /// holds (credit's borrowers are those short of theirs).
    pub age: Option<u32>,
    pub endowment: Option<f64>,
}

#[derive(Clone, Debug, Serialize)]
pub struct SiteView {
    pub x: u32,
    pub y: u32,
    /// Level of each good.
    pub resources: Vec<f64>,
    /// Capacity of each good.
    pub capacities: Vec<f64>,
    /// Level of each pollutant.
    pub pollution: Vec<f64>,
}

#[derive(Clone, Debug, Serialize)]
pub struct LinkView {
    pub id: AgentId,
    pub alive: bool,
}

#[derive(Clone, Debug, Serialize)]
pub struct DiseaseView {
    pub id: DiseaseId,
    pub bits: String,
    /// Smallest Hamming distance between the disease and a window of the
    /// agent's immune string.
    pub distance: u32,
}

#[derive(Clone, Debug, Serialize)]
pub struct LoanView {
    pub id: LoanId,
    pub role: &'static str,
    pub good: usize,
    pub counterparty: LinkView,
    pub due: f64,
    pub due_tick: u64,
}

/// Minds 2: an agent's plan, for display.
#[derive(Clone, Debug, Serialize)]
pub struct PlanView {
    pub target_x: u32,
    pub target_y: u32,
    pub path: Vec<[u32; 2]>,
    /// The agent walked (or tried to) rather than jumped.
    pub walked: bool,
}

#[derive(Clone, Debug, Serialize)]
pub struct DiseaseEntry {
    pub id: DiseaseId,
    pub bits: String,
    /// Living agents carrying it.
    pub carriers: u32,
}

#[derive(Clone, Debug, Serialize)]
pub struct AgentView {
    pub id: AgentId,
    pub x: u32,
    pub y: u32,
    pub sex: Sex,
    pub tribe: Tribe,
    /// Index of the agent's group in `culture.groups`.
    pub group: usize,
    pub tags: String,
    pub vision: u32,
    /// Holdings, birth endowment and metabolism of each good.
    pub holdings: Vec<f64>,
    pub initial: Vec<f64>,
    pub metabolism: Vec<u32>,
    pub age: u32,
    pub max_age: u32,
    pub fertile: bool,
    pub fertility_onset: u32,
    pub fertility_end: u32,
    pub born: u64,
    pub parents: Vec<LinkView>,
    pub children: Vec<LinkView>,
    pub foresight: u32,
    pub loans: Vec<LoanView>,
    pub immune: String,
    pub immune_genome: String,
    pub diseases: Vec<DiseaseView>,
    pub infected_by: Option<LinkView>,
    /// Minds 2: where the agent is walking and the path left to it. `None`
    /// until the agent first moves; its path is empty under `jump` or once
    /// the agent has arrived.
    pub plan: Option<PlanView>,
    /// Minds 3: whether the agent remembers, and how many sites and truffle
    /// spots it holds in memory. `None` while memory is off (`span` 0).
    pub memory: Option<MemoryView>,
}

/// Minds 3: what an agent remembers, for display.
#[derive(Clone, Debug, Serialize)]
pub struct MemoryView {
    pub remembers: bool,
    /// Sites in its memory (forgotten ones may linger until the next sweep).
    pub sites: u32,
    /// Of those, the ones where it knows a truffle spot.
    pub spots: u32,
}

#[derive(Clone, Debug, Serialize)]
pub struct Inspection {
    pub site: SiteView,
    pub agent: Option<AgentView>,
}

impl World {
    fn checked_pos(&self, x: u32, y: u32) -> Result<Pos, String> {
        if x < self.torus.width && y < self.torus.height {
            Ok(Pos::new(x, y))
        } else {
            Err(format!("({x}, {y}) is outside the grid"))
        }
    }

    /// Sets good `good`'s capacity to `value` on every site within Euclidean
    /// `radius` of (x, y) (wrapping), clamping its level to the new capacity.
    pub fn paint_capacity(
        &mut self,
        x: u32,
        y: u32,
        radius: u32,
        value: f64,
        good: usize,
    ) -> Result<(), String> {
        let center = self.checked_pos(x, y)?;
        if good >= self.config.goods.len() {
            return Err(format!("there is no good {good}"));
        }
        if !(value.is_finite() && value >= 0.0) {
            return Err("capacity must be ≥ 0".into());
        }
        let r = radius as i32;
        for dy in -r..=r {
            for dx in -r..=r {
                if dx * dx + dy * dy > r * r {
                    continue;
                }
                let pos = self.torus.offset(center, dx, dy);
                if self.is_wall(pos) {
                    continue;
                }
                let site = self.site_mut(pos);
                site.capacity[good] = value;
                site.resource[good] = site.resource[good].min(value);
            }
        }
        Ok(())
    }

    /// Replaces good `good`'s capacities with `capacities` (row-major, one per
    /// site, each 0–10 — an imported image), clamping each site's level to its
    /// new capacity as painting does. The good then counts as edited
    /// (`landscape_edited`), so share links, export and reset keep it.
    pub fn set_capacities(&mut self, good: usize, capacities: &[f64]) -> Result<(), String> {
        if good >= self.config.goods.len() {
            return Err(format!("there is no good {good}"));
        }
        if capacities.len() != self.sites.len() {
            return Err(format!(
                "expected {} capacities, got {}",
                self.sites.len(),
                capacities.len()
            ));
        }
        if let Some(bad) = capacities.iter().find(|c| !(0.0..=10.0).contains(*c)) {
            return Err(format!("capacities must be between 0 and 10 (got {bad})"));
        }
        let walls = &self.walls;
        for ((i, site), &c) in self.sites.iter_mut().enumerate().zip(capacities) {
            if walls[i] != 0 {
                continue;
            }
            site.capacity[good] = c;
            site.resource[good] = site.resource[good].min(c);
        }
        Ok(())
    }

    pub fn place_agent(&mut self, x: u32, y: u32, o: &AgentOverrides) -> Result<AgentId, String> {
        let pos = self.checked_pos(x, y)?;
        if (o.spice.is_some() || o.spice_metabolism.is_some()) && self.config.goods.len() < 2 {
            return Err("spice needs a second good".into());
        }
        let mut agent = Agent::random(&self.config, pos, self.tick, &mut self.rng);
        if let Some(v) = o.vision {
            agent.vision = v;
        }
        if let Some(m) = o.metabolism {
            agent.metabolism[0] = m;
        }
        if let Some(s) = o.sugar {
            agent.holdings[0] = s;
            agent.initial[0] = s;
        }
        if let Some(s) = o.spice {
            agent.holdings[1] = s;
            agent.initial[1] = s;
        }
        if let Some(m) = o.spice_metabolism {
            agent.metabolism[1] = m;
        }
        if let Some(age) = o.age {
            agent.age = age;
        }
        if let Some(e) = o.endowment {
            agent.initial[0] = e;
        }
        if let Some(sex) = o.sex {
            agent.sex = sex;
            agent.fertility_end = self.config.sex.end_for(sex).sample(&mut self.rng);
        }
        if let Some(t) = o.tribe {
            agent.tags = agent.tags.forced_to(t);
        }
        crate::rules::disease::endow(self, &mut agent);
        self.insert_agent(agent)
    }

    /// Removes the agent at (x, y) without counting a death or bequeathing
    /// its sugar.
    pub fn remove_agent(&mut self, x: u32, y: u32) -> Result<(), String> {
        let pos = self.checked_pos(x, y)?;
        let id = self
            .occupant(pos)
            .ok_or_else(|| format!("no agent at ({x}, {y})"))?;
        self.remove(id);
        Ok(())
    }

    pub fn locate(&self, id: AgentId) -> Option<Pos> {
        self.agent(id).map(|a| a.pos)
    }

    pub fn inspect(&self, x: u32, y: u32) -> Result<Inspection, String> {
        let pos = self.checked_pos(x, y)?;
        let s = self.site(pos);
        let (n, m) = (
            self.config.goods.len(),
            self.config.pollution.pollutants.len(),
        );
        let link = |id: AgentId| LinkView {
            id,
            alive: self.agent(id).is_some(),
        };
        let agent = self.agent_at(pos).map(|a| AgentView {
            id: a.id,
            x: a.pos.x,
            y: a.pos.y,
            sex: a.sex,
            tribe: a.tribe(),
            group: a.group(&self.config.culture.groups),
            tags: a.tags.to_bit_string(),
            vision: a.vision,
            holdings: a.holdings[..n].to_vec(),
            initial: a.initial[..n].to_vec(),
            metabolism: a.metabolism[..n].to_vec(),
            age: a.age,
            max_age: a.max_age,
            fertile: a.is_fertile(self.config.sex.fertile_wealth, self.config.goods.len()),
            fertility_onset: a.fertility_onset,
            fertility_end: a.fertility_end,
            born: a.born,
            parents: a
                .parents
                .map(|p| p.iter().map(|&id| link(id)).collect())
                .unwrap_or_default(),
            children: a.children.iter().map(|&id| link(id)).collect(),
            foresight: a.foresight,
            loans: self
                .loans()
                .filter(|l| l.lender == a.id || l.borrower == a.id)
                .map(|l| {
                    let lender = l.lender == a.id;
                    LoanView {
                        id: l.id,
                        role: if lender { "lender" } else { "borrower" },
                        good: l.good,
                        counterparty: link(if lender { l.borrower } else { l.lender }),
                        due: l.due,
                        due_tick: l.due_tick,
                    }
                })
                .collect(),
            immune: a.immune.to_bit_string(),
            immune_genome: a.immune_genome.to_bit_string(),
            diseases: a
                .diseases
                .iter()
                .map(|&id| {
                    let d = self.diseases[id as usize];
                    DiseaseView {
                        id,
                        bits: d.to_bit_string(),
                        distance: a
                            .immune
                            .closest_window(&d)
                            .map_or(d.len(), |(_, distance)| distance),
                    }
                })
                .collect(),
            infected_by: a.infected_by.map(link),
            plan: a.plan.target.map(|t| PlanView {
                target_x: t.x,
                target_y: t.y,
                path: a.plan.path.iter().map(|p| [p.x, p.y]).collect(),
                walked: a.plan.walked,
            }),
            memory: (self.config.memory.span > 0).then(|| MemoryView {
                remembers: a.remembers,
                sites: a.memory.sites.len() as u32,
                spots: a
                    .memory
                    .sites
                    .values()
                    .filter(|s| s.truffle.is_some())
                    .count() as u32,
            }),
        });
        Ok(Inspection {
            site: SiteView {
                x,
                y,
                resources: s.resource[..n].to_vec(),
                capacities: s.capacity[..n].to_vec(),
                pollution: s.pollution[..m].to_vec(),
            },
            agent,
        })
    }

    /// Minds 3: the Flump at `pos`'s remembered sites, in the memory's own
    /// (site-index) order: each site's `(x, y)`, its age in ticks since last
    /// seen (capped at `u32::MAX`), and its `spot` — 0 for no known truffle
    /// spot, 1 for one believed unripe, 2 for one believed ripe
    /// (`minds::memory::believed_ripe`). Empty when there's no Flump there,
    /// it doesn't remember, or memory is off (`span` 0).
    pub fn memory_view(&self, pos: Pos) -> Vec<[u32; 4]> {
        if self.config.memory.span == 0 {
            return Vec::new();
        }
        let Some(agent) = self.agent_at(pos) else {
            return Vec::new();
        };
        if !agent.remembers {
            return Vec::new();
        }
        let now = self.tick;
        agent
            .memory
            .sites
            .iter()
            .map(|(&idx, seen)| {
                let p = self.torus.pos(idx as usize);
                let age = now.saturating_sub(seen.tick).min(u64::from(u32::MAX)) as u32;
                let spot = match &seen.truffle {
                    None => 0,
                    Some(t) => {
                        if believed_ripe(
                            t,
                            now,
                            self.config.truffles.regrow,
                            self.config.memory.belief,
                        ) {
                            2
                        } else {
                            1
                        }
                    }
                };
                [p.x, p.y, age, spot]
            })
            .collect()
    }

    /// Swaps in a new config mid-run. Rule toggles and parameters take effect
    /// on the next tick; grid size, tag length and landscape need a reset.
    /// Only schedule entries that have not fired yet are validated.
    pub fn set_config(&mut self, next: Config) -> Result<(), Vec<FieldError>> {
        next.validate_with_schedule_from(self.tick)?;
        let structural = self.config.structural_changes(&next);
        if !structural.is_empty() {
            return Err(structural);
        }
        self.config = next;
        Ok(())
    }

    fn disease_on(&self) -> Result<(), String> {
        if self.config.disease.enabled {
            Ok(())
        } else {
            Err("disease is off".into())
        }
    }

    /// The master disease list with each disease's carriers.
    pub fn disease_list(&self) -> Vec<DiseaseEntry> {
        let mut carriers = vec![0u32; self.diseases.len()];
        for a in self.agents() {
            for &d in &a.diseases {
                carriers[d as usize] += 1;
            }
        }
        self.diseases
            .iter()
            .zip(carriers)
            .enumerate()
            .map(|(i, (d, carriers))| DiseaseEntry {
                id: i as DiseaseId,
                bits: d.to_bit_string(),
                carriers,
            })
            .collect()
    }

    /// Infects the agent at (x, y) with listed `disease`, or — when `disease`
    /// is negative — with a brand-new random disease (appended to the list even
    /// if the agent resists it). No effect if the agent is immune or already
    /// carries it. Returns whether it was infected.
    pub fn infect(&mut self, x: u32, y: u32, disease: i64) -> Result<bool, String> {
        self.disease_on()?;
        let pos = self.checked_pos(x, y)?;
        let id = self
            .occupant(pos)
            .ok_or_else(|| format!("no agent at ({x}, {y})"))?;
        let d = if disease < 0 {
            rules::disease::new_random(self)
        } else {
            DiseaseId::try_from(disease)
                .ok()
                .filter(|&d| (d as usize) < self.diseases.len())
                .ok_or_else(|| format!("unknown disease {disease}"))?
        };
        Ok(rules::disease::infect(self, id, d))
    }

    /// Writes `disease` into the immune string of every agent within Euclidean
    /// `radius` of (x, y) (wrapping), over its closest window, then cures any
    /// carried disease the string now contains. The genome is untouched.
    /// Returns how many agents were vaccinated.
    pub fn vaccinate(
        &mut self,
        x: u32,
        y: u32,
        radius: u32,
        disease: DiseaseId,
    ) -> Result<u32, String> {
        self.disease_on()?;
        let center = self.checked_pos(x, y)?;
        let d = *self
            .diseases
            .get(disease as usize)
            .ok_or_else(|| format!("unknown disease {disease}"))?;
        let r = radius as i32;
        let mut ids = BTreeSet::new();
        for dy in -r..=r {
            for dx in -r..=r {
                if dx * dx + dy * dy <= r * r {
                    if let Some(id) = self.occupant(self.torus.offset(center, dx, dy)) {
                        ids.insert(id);
                    }
                }
            }
        }
        let mut vaccinated = 0;
        for id in ids {
            if self.agent_mut(id).expect("occupant").immune.imprint(&d) {
                rules::disease::cure_immune(self, id);
                vaccinated += 1;
            }
        }
        Ok(vaccinated)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{Good, Map, URange};
    use crate::testkit::*;

    #[test]
    fn painting_a_good_changes_only_that_good() {
        let mut w = blank_world(20, 20);
        add_goods(&mut w.config, 2);
        w.paint_capacity(3, 3, 0, 2.0, 1).unwrap();
        assert_eq!(w.site(Pos::new(3, 3)).capacity[..2], [0.0, 2.0]);
        assert!(!w.landscape_edited(0) && w.landscape_edited(1));
        assert!(w.paint_capacity(3, 3, 0, 2.0, 2).is_err(), "no good 2");
    }

    #[test]
    fn painting_sets_capacity_in_a_disc_and_clamps_sugar() {
        let mut w = blank_world(20, 20);
        for i in 0..w.sites.len() {
            w.sites[i].capacity[0] = 4.0;
            w.sites[i].resource[0] = 4.0;
        }
        w.paint_capacity(10, 10, 1, 1.0, 0).unwrap();
        assert!(w.landscape_edited(0));
        for p in [(10, 10), (10, 9), (11, 10), (9, 10), (10, 11)] {
            let s = w.site(Pos::new(p.0, p.1));
            assert_eq!((s.capacity[0], s.resource[0]), (1.0, 1.0), "{p:?}");
        }
        assert_eq!(
            w.site(Pos::new(11, 11)).capacity[0],
            4.0,
            "radius 1 disc excludes diagonals"
        );
        assert!(w.paint_capacity(20, 0, 1, 1.0, 0).is_err());
    }

    #[test]
    fn painting_and_setting_capacity_leaves_a_wall_at_zero() {
        let mut c = crate::testkit::blank_config(20, 20);
        c.walls = vec![crate::config::Wall {
            x: 10,
            y: 10,
            width: 1,
            height: 1,
            opaque: false,
        }];
        let mut w = World::new(c, 1).unwrap();
        let wall = Pos::new(10, 10);
        w.paint_capacity(10, 10, 2, 4.0, 0).unwrap();
        assert_eq!(w.site(wall).capacity[0], 0.0, "painting skips the wall");
        assert_eq!(
            w.site(Pos::new(11, 10)).capacity[0],
            4.0,
            "an open site in the disc is still painted"
        );
        let caps = vec![4.0; 400];
        w.set_capacities(0, &caps).unwrap();
        assert_eq!(
            w.site(wall).capacity[0],
            0.0,
            "set_capacities skips the wall"
        );
        assert_eq!(w.site(Pos::new(0, 0)).capacity[0], 4.0);
    }

    #[test]
    fn placing_an_agent_on_a_wall_is_refused() {
        let mut c = crate::testkit::blank_config(10, 10);
        c.walls = vec![crate::config::Wall {
            x: 3,
            y: 3,
            width: 1,
            height: 1,
            opaque: true,
        }];
        let mut w = World::new(c, 1).unwrap();
        let err = w.place_agent(3, 3, &AgentOverrides::default()).unwrap_err();
        assert!(err.contains("wall"), "{err}");
    }

    #[test]
    fn a_placement_can_set_spice_in_a_two_good_world() {
        let config = match crate::presets::find("iv-1-spice").unwrap().config {
            crate::model::ModelConfig::Sugarscape(c) => c,
            _ => unreachable!(),
        };
        let mut w = World::new(
            Config {
                population: 0,
                ..config
            },
            1,
        )
        .unwrap();
        let overrides = AgentOverrides {
            spice: Some(7.0),
            spice_metabolism: Some(4),
            ..Default::default()
        };
        let id = w.place_agent(2, 3, &overrides).unwrap();
        let a = w.agent(id).unwrap();
        assert_eq!(
            (a.holdings[1], a.initial[1], a.metabolism[1]),
            (7.0, 7.0, 4)
        );
        let mut one = blank_world(10, 10);
        assert!(
            one.place_agent(2, 3, &overrides).is_err(),
            "no spice in a one-good world"
        );
    }

    #[test]
    fn a_placement_can_set_its_age_and_birth_endowment() {
        let mut w = blank_world(10, 10);
        let overrides = AgentOverrides {
            sugar: Some(5.0),
            age: Some(70),
            endowment: Some(40.0),
            ..Default::default()
        };
        let id = w.place_agent(2, 3, &overrides).unwrap();
        let a = w.agent(id).unwrap();
        assert_eq!((a.age, a.holdings[0], a.initial[0]), (70, 5.0, 40.0));
    }

    #[test]
    fn placing_and_removing_agents() {
        let mut w = blank_world(10, 10);
        let overrides = AgentOverrides {
            vision: Some(3),
            sex: Some(Sex::Male),
            tribe: Some(Tribe::Red),
            ..Default::default()
        };
        let id = w.place_agent(2, 3, &overrides).unwrap();
        let a = w.agent(id).unwrap();
        assert_eq!((a.vision, a.sex, a.tribe()), (3, Sex::Male, Tribe::Red));
        assert!(
            w.place_agent(2, 3, &AgentOverrides::default()).is_err(),
            "occupied"
        );
        w.remove_agent(2, 3).unwrap();
        assert_eq!(w.population(), 0);
        assert!(w.events().deaths.is_empty(), "removal is not a death");
        assert!(w.remove_agent(2, 3).is_err());
    }

    #[test]
    fn erasing_a_parent_does_not_bequeath() {
        let mut w = blank_world(10, 10);
        w.config.inheritance.enabled = true;
        let parent = spawn(&mut w, 1, 1);
        let child = spawn(&mut w, 2, 1);
        w.agent_mut(parent).unwrap().holdings[0] = 30.0;
        w.agent_mut(parent).unwrap().children = vec![child];
        let before = w.agent(child).unwrap().holdings[0];
        w.remove_agent(1, 1).unwrap();
        assert_eq!(w.agent(child).unwrap().holdings[0], before);
        assert_eq!(w.occupant(Pos::new(1, 1)), None);
        assert!(w.events().deaths.is_empty());
    }

    /// `seen`, with a truffle spot there seen `ripe` (or not) at `tick`.
    fn with_spot(
        mut seen: crate::minds::memory::Seen,
        ripe: bool,
        tick: u64,
    ) -> crate::minds::memory::Seen {
        seen.truffle = Some(crate::minds::memory::TruffleSeen { ripe, tick });
        seen
    }

    #[test]
    fn inspect_shows_memory_only_when_memory_is_on() {
        let mut w = blank_world(10, 10);
        let id = spawn(&mut w, 2, 2);
        assert!(w.inspect(2, 2).unwrap().agent.unwrap().memory.is_none());
        w.config.movement.mode = crate::config::MoveMode::Walk;
        w.config.memory.span = 20;
        let seen = crate::minds::memory::Seen::new(&[0.0], &[0.0], 0);
        {
            let a = w.agent_mut(id).unwrap();
            a.remembers = true;
            a.memory.sites.insert(1, seen.clone());
            a.memory.sites.insert(2, with_spot(seen.clone(), true, 0));
        }
        let m = w.inspect(2, 2).unwrap().agent.unwrap().memory.unwrap();
        assert_eq!((m.remembers, m.sites, m.spots), (true, 2, 1));
    }

    #[test]
    fn memory_view_is_empty_with_memory_off_no_flump_or_a_non_rememberer() {
        let mut w = blank_world(10, 10);
        let id = spawn(&mut w, 2, 2);
        assert!(
            w.memory_view(Pos::new(2, 2)).is_empty(),
            "memory off (span 0)"
        );
        w.config.movement.mode = crate::config::MoveMode::Walk;
        w.config.memory.span = 20;
        assert!(
            w.memory_view(Pos::new(5, 5)).is_empty(),
            "no Flump at that site"
        );
        assert!(
            w.memory_view(Pos::new(2, 2)).is_empty(),
            "the Flump there doesn't remember"
        );
        w.agent_mut(id).unwrap().remembers = true;
        assert!(
            w.memory_view(Pos::new(2, 2)).is_empty(),
            "remembers, but nothing in memory yet"
        );
    }

    #[test]
    fn memory_view_lists_sites_in_memory_order_with_age_and_spot() {
        let mut w = blank_world(10, 10);
        let id = spawn(&mut w, 2, 2);
        w.config.movement.mode = crate::config::MoveMode::Walk;
        w.config.memory.span = 20;
        w.config.memory.belief = crate::config::Belief::Project;
        w.config.truffles.regrow = 10;
        w.tick = 20;
        let blank_seen = crate::minds::memory::Seen::new(&[0.0], &[0.0], 18);
        {
            let a = w.agent_mut(id).unwrap();
            a.remembers = true;
            // Site 1: no known spot, last seen 2 ticks ago.
            a.memory.sites.insert(1, blank_seen.clone());
            // Site 2: a spot seen unripe 5 ticks ago; regrow is 10, so it's
            // still believed unripe.
            a.memory
                .sites
                .insert(2, with_spot(blank_seen.clone(), false, 15));
            // Site 5: a spot seen unripe 15 ticks ago; past regrow, so it's
            // now believed ripe.
            a.memory
                .sites
                .insert(5, with_spot(blank_seen.clone(), false, 5));
        }
        let view = w.memory_view(Pos::new(2, 2));
        assert_eq!(
            view,
            vec![[1, 0, 2, 0], [2, 0, 2, 1], [5, 0, 2, 2],],
            "BTreeMap (site-index) order, each [x, y, age, spot]"
        );
    }

    #[test]
    fn inspect_reports_site_agent_and_lineage() {
        let mut w = blank_world(10, 10);
        let parent = spawn(&mut w, 1, 1);
        let child = spawn(&mut w, 2, 1);
        w.agent_mut(child).unwrap().parents = Some([parent, 999]);
        w.agent_mut(parent).unwrap().children = vec![child];
        let i = w.inspect(2, 1).unwrap();
        let a = i.agent.unwrap();
        assert_eq!(a.id, child);
        assert_eq!(a.tags, "00000000000");
        assert_eq!(a.parents.len(), 2);
        assert!(a.parents[0].alive && !a.parents[1].alive);
        assert!(w.inspect(5, 5).unwrap().agent.is_none());
        assert_eq!(w.locate(child), Some(Pos::new(2, 1)));
        assert_eq!(w.locate(999), None);
    }

    #[test]
    fn set_config_applies_rule_changes_but_rejects_structural_ones() {
        let mut w = blank_world(10, 10);
        let mut next = w.config.clone();
        next.culture.enabled = true;
        w.set_config(next.clone()).unwrap();
        assert!(w.config.culture.enabled);
        next.width = 12;
        let errs = w.set_config(next).unwrap_err();
        assert_eq!(errs[0].field, "width");
        let mut bad = w.config.clone();
        bad.goods[0].metabolism = URange::new(9, 1);
        assert_eq!(
            w.set_config(bad).unwrap_err()[0].field,
            "goods.0.metabolism"
        );
    }

    #[test]
    fn set_config_rejects_adding_a_good_mid_run() {
        let mut w = blank_world(10, 10);
        let mut next = w.config.clone();
        next.add_good(Good {
            map: Map::Flat { capacity: 0.0 },
            ..Good::spice()
        });
        let errs = w.set_config(next).unwrap_err();
        assert_eq!(errs[0].field, "goods");
        assert_eq!(w.config.goods.len(), 1);
    }

    #[test]
    fn set_config_ignores_schedule_entries_that_already_fired() {
        use crate::config::ScheduledChange;
        let mut c = blank_config(10, 10);
        c.lifespan.enabled = true;
        c.schedule = vec![ScheduledChange {
            tick: 1,
            set: [("replacement.enabled".to_string(), serde_json::json!(true))]
                .into_iter()
                .collect(),
        }];
        let mut w = World::new(c, 1).unwrap();
        w.run(2);
        assert!(w.config.replacement.enabled, "t=1 entry fired");
        // Re-applying the t=1 entry to this config would enable replacement
        // without lifespan, but that entry is in the past.
        let mut next = w.config.clone();
        next.replacement.enabled = false;
        next.lifespan.enabled = false;
        w.set_config(next).unwrap();
        assert!(!w.config.lifespan.enabled);
        // Entries still to come are validated.
        let mut next = w.config.clone();
        next.schedule.push(ScheduledChange {
            tick: 5,
            set: [("replacement.enabled".to_string(), serde_json::json!(true))]
                .into_iter()
                .collect(),
        });
        assert_eq!(w.set_config(next).unwrap_err()[0].field, "schedule");
    }

    #[test]
    fn inspection_shows_every_good_pollutant_and_loan() {
        let mut w = blank_world(10, 10);
        add_goods(&mut w.config, 2);
        let a = spawn(&mut w, 1, 1);
        let b = spawn(&mut w, 2, 1);
        {
            let x = w.agent_mut(a).unwrap();
            x.foresight = 3;
            x.metabolism[1] = 2;
        }
        w.originate_loan(a, b, 1, 2.0);
        let view = w.inspect(1, 1).unwrap().agent.unwrap();
        assert_eq!(
            (view.holdings.clone(), view.initial.clone()),
            (vec![10.0, 10.0], vec![10.0, 10.0])
        );
        assert_eq!((view.metabolism.clone(), view.foresight), (vec![0, 2], 3));
        assert_eq!(view.loans.len(), 1);
        assert_eq!((view.loans[0].role, view.loans[0].good), ("lender", 1));
        assert_eq!(view.loans[0].counterparty.id, b);
        let other = w.inspect(2, 1).unwrap().agent.unwrap();
        assert_eq!(other.loans[0].role, "borrower");
        w.site_mut(Pos::new(1, 1)).capacity[1] = 3.0;
        let site = w.inspect(1, 1).unwrap().site;
        assert_eq!(
            (site.resources.clone(), site.capacities.clone()),
            (vec![0.0, 0.0], vec![0.0, 3.0])
        );
        assert_eq!(site.pollution, vec![0.0]);
    }

    #[test]
    fn placed_agents_get_immune_systems_when_disease_is_on() {
        let mut w = blank_world(10, 10);
        w.config.disease.enabled = true;
        w.diseases = vec![crate::bits::Bits::parse("1111111111").unwrap()];
        let id = w.place_agent(3, 3, &AgentOverrides::default()).unwrap();
        let a = w.agent(id).unwrap();
        assert_eq!(a.immune.len(), 50);
        assert_eq!(a.immune, a.immune_genome);
        assert!(
            a.diseases.len() <= 1,
            "initial 4 is capped by the list's length"
        );
    }

    #[test]
    fn inspection_shows_immune_strings_diseases_and_infector() {
        use crate::bits::Bits;
        let mut w = blank_world(10, 10);
        w.config.disease.enabled = true;
        w.diseases = vec![Bits::parse("111").unwrap()];
        let a = spawn(&mut w, 1, 1);
        let b = spawn(&mut w, 2, 1);
        {
            let x = w.agent_mut(a).unwrap();
            x.immune = Bits::parse(&format!("0110{}", "0".repeat(46))).unwrap();
            x.diseases = vec![0];
            x.infected_by = Some(b);
        }
        let v = w.inspect(1, 1).unwrap().agent.unwrap();
        assert_eq!(v.immune, format!("0110{}", "0".repeat(46)));
        assert_eq!(v.immune_genome, "0".repeat(50));
        assert_eq!(v.diseases.len(), 1);
        let d = &v.diseases[0];
        assert_eq!((d.id, d.bits.as_str(), d.distance), (0, "111", 1));
        assert_eq!(v.infected_by.unwrap().id, b);
        assert!(w
            .inspect(2, 1)
            .unwrap()
            .agent
            .unwrap()
            .infected_by
            .is_none());
    }

    #[test]
    fn inspection_gives_the_agents_group() {
        let mut w = blank_world(5, 5);
        let id = spawn(&mut w, 1, 1); // all zeros: Blue (6–11 zeros)
        let group = |w: &World| w.inspect(1, 1).unwrap().agent.unwrap().group;
        assert_eq!(group(&w), 0);
        w.agent_mut(id).unwrap().tags = crate::agent::Tags::new(u64::MAX, 11);
        assert_eq!(group(&w), 1, "no zeros: Red");
        w.config.culture.groups = crate::config::three_tribes(11);
        assert_eq!(group(&w), 0, "no zeros: Blue (0–3)");
    }

    #[test]
    fn infect_tool_gives_a_listed_or_new_disease() {
        use crate::bits::Bits;
        let mut w = blank_world(10, 10);
        w.config.disease.enabled = true;
        w.diseases = vec![Bits::parse("101").unwrap()];
        let a = spawn(&mut w, 1, 1);
        assert!(w.infect(1, 1, 0).unwrap());
        assert!(!w.infect(1, 1, 0).unwrap(), "already carries it");
        assert_eq!(w.agent(a).unwrap().diseases, vec![0]);
        assert!(
            w.agent(a).unwrap().infected_by.is_none(),
            "a tool is not an infector"
        );
        w.infect(1, 1, -1).unwrap();
        assert_eq!(w.diseases.len(), 2, "a new disease is appended");
        assert_ne!(w.diseases[1], w.diseases[0]);
        assert!(w.infect(1, 1, 7).is_err(), "unknown disease");
        assert!(w.infect(5, 5, 0).is_err(), "no agent");
        assert!(
            w.events().infections.is_empty(),
            "edits are not counted as infections"
        );
        w.config.disease.enabled = false;
        assert_eq!(w.infect(1, 1, 0).unwrap_err(), "disease is off");
    }

    #[test]
    fn vaccination_writes_the_disease_into_immune_strings_in_the_brush() {
        use crate::bits::Bits;
        let mut w = blank_world(10, 10);
        w.config.disease.enabled = true;
        w.diseases = vec![Bits::parse("111").unwrap()];
        let near = spawn(&mut w, 5, 5);
        let edge = spawn(&mut w, 5, 6);
        let far = spawn(&mut w, 8, 8);
        for id in [near, edge, far] {
            w.agent_mut(id).unwrap().diseases = vec![0];
        }
        assert_eq!(w.vaccinate(5, 5, 1, 0).unwrap(), 2);
        for id in [near, edge] {
            let a = w.agent(id).unwrap();
            assert!(a.diseases.is_empty(), "cured");
            assert!(
                a.immune.to_bit_string().starts_with("111"),
                "leftmost closest window"
            );
            assert_eq!(
                a.immune_genome.to_bit_string(),
                "0".repeat(50),
                "genome untouched"
            );
        }
        assert_eq!(w.agent(far).unwrap().diseases, vec![0]);
        assert!(w.vaccinate(5, 5, 1, 3).is_err(), "unknown disease");
        let list = w.disease_list();
        assert_eq!(list.len(), 1);
        assert_eq!(
            (list[0].id, list[0].bits.as_str(), list[0].carriers),
            (0, "111", 1)
        );
    }

    #[test]
    fn set_capacities_replaces_a_goods_map_and_clamps_levels() {
        let mut w = blank_world(5, 5);
        add_goods(&mut w.config, 2);
        for s in &mut w.sites {
            s.capacity[1] = 5.0;
            s.resource[1] = 5.0;
        }
        w.sites[0].resource[1] = 0.0;
        let caps: Vec<f64> = (0..25).map(|i| f64::from(i % 3)).collect();
        w.set_capacities(1, &caps).unwrap();
        assert_eq!(w.capacities(1), caps);
        let levels: Vec<f64> = w.sites.iter().map(|s| s.resource[1]).collect();
        assert_eq!(levels, caps, "levels above the new capacity are clamped");
        assert!(w.landscape_edited(1) && !w.landscape_edited(0));
        let err = |r: Result<(), String>| r.unwrap_err();
        assert_eq!(err(w.set_capacities(2, &caps)), "there is no good 2");
        assert_eq!(
            err(w.set_capacities(1, &caps[..24])),
            "expected 25 capacities, got 24"
        );
        for bad in [10.5, -1.0, f64::NAN, f64::INFINITY] {
            let e = err(w.set_capacities(1, &[bad; 25]));
            assert!(e.starts_with("capacities must be between 0 and 10"), "{e}");
        }
        assert_eq!(w.capacities(1), caps, "a rejected call changes nothing");
    }
}
