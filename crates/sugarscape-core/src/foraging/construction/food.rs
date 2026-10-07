use super::{
    state::{Agent, Cargo},
    terrain::Terrain,
    Checked, Pos, Resource, Setup,
};
use std::collections::BTreeSet;
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub enum FoodState {
    Hidden,
    Available,
    Carried { agent: u32 },
    Delivered,
}
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub struct FoodView {
    pub resource: Resource,
    pub state: FoodState,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
pub struct FoodInventory {
    pub initial: u32,
    pub hidden: u32,
    pub available: u32,
    pub carried: u32,
    pub delivered: u32,
}
// Staged Task 4 world material transactions.
#[allow(dead_code)]
#[derive(Clone, Debug, PartialEq)]
pub(super) struct FoodLedger {
    records: Vec<FoodView>,
}
// Staged Task 4 world material transactions.
#[allow(dead_code)]
impl FoodLedger {
    pub(super) fn new(setup: &Setup, terrain: &Terrain) -> Checked<Self> {
        setup.validate()?;
        terrain.check()?;
        if terrain.dimensions() != (setup.width, setup.height) {
            return Err(error("food.terrain", "dimensions disagree with setup"));
        }
        let mut records = Vec::with_capacity(setup.food.len());
        for resource in &setup.food {
            records.push(FoodView {
                resource: resource.clone(),
                state: if terrain.is_open(resource.pos)? {
                    FoodState::Available
                } else {
                    FoodState::Hidden
                },
            });
        }
        records.sort_by_key(|v| v.resource.id);
        Ok(Self { records })
    }
    pub(super) fn expose(&mut self, pos: Pos) -> Checked<Option<u64>> {
        let Some(record) = self.records.iter_mut().find(|v| v.resource.pos == pos) else {
            return Ok(None);
        };
        if record.state != FoodState::Hidden {
            return Err(error("food.expose", "exposure requires hidden food"));
        }
        record.state = FoodState::Available;
        Ok(Some(record.resource.id))
    }
    pub(super) fn claim(&mut self, pos: Pos, agent: u32) -> Checked<Option<u64>> {
        if self
            .records
            .iter()
            .any(|r| r.state == FoodState::Carried { agent })
        {
            return Err(error(
                format!("agents[{agent}].cargo"),
                "cannot claim another food token while carrying one",
            ));
        }
        let Some(record) = self
            .records
            .iter_mut()
            .find(|r| r.resource.pos == pos && r.state == FoodState::Available)
        else {
            return Ok(None);
        };
        record.state = FoodState::Carried { agent };
        Ok(Some(record.resource.id))
    }
    pub(super) fn deposit(&mut self, id: u64, agent: u32) -> Checked<()> {
        let index = self
            .records
            .binary_search_by_key(&id, |v| v.resource.id)
            .map_err(|_| error(format!("food[{id}]"), "unknown food identity"))?;
        if self.records[index].state != (FoodState::Carried { agent }) {
            return Err(error(
                format!("food[{id}].state"),
                format!("deposit requires carriage by agent {agent}"),
            ));
        }
        self.records[index].state = FoodState::Delivered;
        Ok(())
    }
    /// Authoritative availability belongs only to observation construction.
    pub(super) fn available(&self) -> BTreeSet<Pos> {
        self.records
            .iter()
            .filter(|r| r.state == FoodState::Available)
            .map(|r| r.resource.pos)
            .collect()
    }
    pub(super) fn inventory(&self) -> FoodInventory {
        let mut inventory = FoodInventory {
            initial: self.records.len() as u32,
            hidden: 0,
            available: 0,
            carried: 0,
            delivered: 0,
        };
        for r in &self.records {
            match r.state {
                FoodState::Hidden => inventory.hidden += 1,
                FoodState::Available => inventory.available += 1,
                FoodState::Carried { .. } => inventory.carried += 1,
                FoodState::Delivered => inventory.delivered += 1,
            }
        }
        inventory
    }
    pub(super) fn views(&self) -> &[FoodView] {
        &self.records
    }
    pub(super) fn check(&self, terrain: &Terrain, agents: &[Agent]) -> Checked<()> {
        terrain.check()?;
        let mut ids = BTreeSet::new();
        let mut cells = BTreeSet::new();
        let mut owners = BTreeSet::new();
        if self.records.len() > 256
            || self
                .records
                .windows(2)
                .any(|w| w[0].resource.id >= w[1].resource.id)
        {
            return Err(error(
                "food",
                "inventory must fit 256 records sorted by unique identity",
            ));
        }
        for record in &self.records {
            if !ids.insert(record.resource.id) || !cells.insert(record.resource.pos) {
                return Err(error("food", "duplicate identity or original cell"));
            }
            if terrain.is_open(record.resource.pos)? == (record.state == FoodState::Hidden) {
                return Err(error(format!("food[{}].state",record.resource.id),"hidden food requires solid terrain; all other states require its original cell open"));
            }
            if let FoodState::Carried { agent } = record.state {
                if !owners.insert(agent)
                    || agents
                        .iter()
                        .filter(|a| {
                            a.id == agent && a.cargo == Some(Cargo::Food(record.resource.id))
                        })
                        .count()
                        != 1
                {
                    return Err(error(
                        format!("food[{}].state", record.resource.id),
                        "carried food must have exactly one matching tagged cargo owner",
                    ));
                }
            }
        }
        let mut agent_ids = BTreeSet::new();
        for agent in agents {
            if !agent_ids.insert(agent.id) {
                return Err(error("agents", "duplicate agent identity"));
            }
            if let Some(Cargo::Food(id)) = agent.cargo {
                if !self.records.iter().any(|r| {
                    r.resource.id == id && r.state == FoodState::Carried { agent: agent.id }
                }) {
                    return Err(error(
                        format!("agents[{}].cargo", agent.id),
                        "food cargo must match this agent's ledger assignment",
                    ));
                }
            }
        }
        Ok(())
    }
}
fn error(field: impl Into<String>, message: impl Into<String>) -> Vec<crate::config::FieldError> {
    vec![crate::config::FieldError::new(field, message)]
}

#[cfg(test)]
mod material_access {
    use super::super::tests::{pos, setup};
    use super::*;
    #[test]
    fn food_identity_order_supports_binary_search_deposit() {
        let mut s = setup();
        s.food.push(Resource {
            id: 2,
            pos: pos(2, 0),
        });
        let t = Terrain::new(&s).unwrap();
        let mut f = FoodLedger::new(&s, &t).unwrap();
        f.check(&t, &[]).unwrap();
        f.records.reverse();
        assert!(f.check(&t, &[]).is_err());
    }
}
