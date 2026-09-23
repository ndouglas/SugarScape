//! Interactive edits and inspection used by the playground UI.

use serde::{Deserialize, Serialize};

use crate::agent::{Agent, AgentId, Sex, Tribe};
use crate::config::{Config, FieldError};
use crate::geometry::Pos;
use crate::world::{LoanId, World};

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
pub struct AgentOverrides {
    pub vision: Option<u32>,
    pub metabolism: Option<u32>,
    pub sugar: Option<f64>,
    pub sex: Option<Sex>,
    pub tribe: Option<Tribe>,
}

#[derive(Clone, Debug, Serialize)]
pub struct SiteView {
    pub x: u32,
    pub y: u32,
    pub sugar: f64,
    pub capacity: f64,
    pub pollution: f64,
    pub spice: f64,
    pub spice_capacity: f64,
}

#[derive(Clone, Debug, Serialize)]
pub struct LinkView {
    pub id: AgentId,
    pub alive: bool,
}

#[derive(Clone, Debug, Serialize)]
pub struct LoanView {
    pub id: LoanId,
    pub role: &'static str,
    pub counterparty: LinkView,
    pub due: f64,
    pub due_tick: u64,
}

#[derive(Clone, Debug, Serialize)]
pub struct AgentView {
    pub id: AgentId,
    pub x: u32,
    pub y: u32,
    pub sex: Sex,
    pub tribe: Tribe,
    pub tags: String,
    pub vision: u32,
    pub metabolism: u32,
    pub sugar: f64,
    pub initial_sugar: f64,
    pub age: u32,
    pub max_age: u32,
    pub fertile: bool,
    pub fertility_onset: u32,
    pub fertility_end: u32,
    pub born: u64,
    pub parents: Vec<LinkView>,
    pub children: Vec<LinkView>,
    pub spice: f64,
    pub initial_spice: f64,
    pub spice_metabolism: u32,
    pub foresight: u32,
    pub loans: Vec<LoanView>,
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

    /// Sets capacity to `value` on every site within Euclidean `radius` of
    /// (x, y) (wrapping), clamping sugar to the new capacity.
    pub fn paint_capacity(
        &mut self,
        x: u32,
        y: u32,
        radius: u32,
        value: f64,
    ) -> Result<(), String> {
        let center = self.checked_pos(x, y)?;
        if !(value.is_finite() && value >= 0.0) {
            return Err("capacity must be ≥ 0".into());
        }
        let r = radius as i32;
        for dy in -r..=r {
            for dx in -r..=r {
                if dx * dx + dy * dy > r * r {
                    continue;
                }
                let site = self.site_mut(self.torus.offset(center, dx, dy));
                site.capacity = value;
                site.sugar = site.sugar.min(value);
            }
        }
        self.landscape_edited = true;
        Ok(())
    }

    pub fn place_agent(&mut self, x: u32, y: u32, o: &AgentOverrides) -> Result<AgentId, String> {
        let pos = self.checked_pos(x, y)?;
        let mut agent = Agent::random(&self.config, pos, self.tick, &mut self.rng);
        if let Some(v) = o.vision {
            agent.vision = v;
        }
        if let Some(m) = o.metabolism {
            agent.metabolism = m;
        }
        if let Some(s) = o.sugar {
            agent.sugar = s;
            agent.initial_sugar = s;
        }
        if let Some(sex) = o.sex {
            agent.sex = sex;
            agent.fertility_end = self.config.sex.end_for(sex).sample(&mut self.rng);
        }
        if let Some(t) = o.tribe {
            agent.tags = agent.tags.forced_to(t);
        }
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
            tags: a.tags.to_bit_string(),
            vision: a.vision,
            metabolism: a.metabolism,
            sugar: a.sugar,
            initial_sugar: a.initial_sugar,
            age: a.age,
            max_age: a.max_age,
            fertile: a.is_fertile(),
            fertility_onset: a.fertility_onset,
            fertility_end: a.fertility_end,
            born: a.born,
            parents: a
                .parents
                .map(|p| p.iter().map(|&id| link(id)).collect())
                .unwrap_or_default(),
            children: a.children.iter().map(|&id| link(id)).collect(),
            spice: a.spice,
            initial_spice: a.initial_spice,
            spice_metabolism: a.spice_metabolism,
            foresight: a.foresight,
            loans: self
                .loans()
                .filter(|l| l.lender == a.id || l.borrower == a.id)
                .map(|l| {
                    let lender = l.lender == a.id;
                    LoanView {
                        id: l.id,
                        role: if lender { "lender" } else { "borrower" },
                        counterparty: link(if lender { l.borrower } else { l.lender }),
                        due: l.due,
                        due_tick: l.due_tick,
                    }
                })
                .collect(),
        });
        Ok(Inspection {
            site: SiteView {
                x,
                y,
                sugar: s.sugar,
                capacity: s.capacity,
                pollution: s.pollution,
                spice: s.spice,
                spice_capacity: s.spice_capacity,
            },
            agent,
        })
    }

    /// Swaps in a new config mid-run. Rule toggles and parameters take effect
    /// on the next tick; grid size, tag length and landscape need a reset.
    pub fn set_config(&mut self, next: Config) -> Result<(), Vec<FieldError>> {
        next.validate()?;
        let structural = self.config.structural_changes(&next);
        if !structural.is_empty() {
            return Err(structural);
        }
        self.config = next;
        Ok(())
    }

    pub fn capacities(&self) -> Vec<f64> {
        self.sites.iter().map(|s| s.capacity).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testkit::*;

    #[test]
    fn painting_sets_capacity_in_a_disc_and_clamps_sugar() {
        let mut w = blank_world(20, 20);
        for i in 0..w.sites.len() {
            w.sites[i].capacity = 4.0;
            w.sites[i].sugar = 4.0;
        }
        w.paint_capacity(10, 10, 1, 1.0).unwrap();
        assert!(w.landscape_edited);
        for p in [(10, 10), (10, 9), (11, 10), (9, 10), (10, 11)] {
            let s = w.site(Pos::new(p.0, p.1));
            assert_eq!((s.capacity, s.sugar), (1.0, 1.0), "{p:?}");
        }
        assert_eq!(
            w.site(Pos::new(11, 11)).capacity,
            4.0,
            "radius 1 disc excludes diagonals"
        );
        assert!(w.paint_capacity(20, 0, 1, 1.0).is_err());
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
        w.agent_mut(parent).unwrap().sugar = 30.0;
        w.agent_mut(parent).unwrap().children = vec![child];
        let before = w.agent(child).unwrap().sugar;
        w.remove_agent(1, 1).unwrap();
        assert_eq!(w.agent(child).unwrap().sugar, before);
        assert_eq!(w.occupant(Pos::new(1, 1)), None);
        assert!(w.events().deaths.is_empty());
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
        bad.metabolism.min = 9;
        bad.metabolism.max = 1;
        assert_eq!(w.set_config(bad).unwrap_err()[0].field, "metabolism");
    }

    #[test]
    fn inspection_shows_spice_foresight_and_loans() {
        let mut w = blank_world(10, 10);
        let a = spawn(&mut w, 1, 1);
        let b = spawn(&mut w, 2, 1);
        w.agent_mut(a).unwrap().foresight = 3;
        w.originate_loan(a, b, 2.0);
        let view = w.inspect(1, 1).unwrap().agent.unwrap();
        assert_eq!((view.spice, view.foresight), (10.0, 3));
        assert_eq!(view.loans.len(), 1);
        assert_eq!(view.loans[0].role, "lender");
        assert_eq!(view.loans[0].counterparty.id, b);
        let other = w.inspect(2, 1).unwrap().agent.unwrap();
        assert_eq!(other.loans[0].role, "borrower");
        assert_eq!(w.inspect(1, 1).unwrap().site.spice_capacity, 0.0);
    }
}
