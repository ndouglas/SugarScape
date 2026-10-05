use super::state::Agent;
use super::{Pos, Resource, Setup};
use crate::config::FieldError;
use crate::foraging::FindRecord;

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub enum ResourceState {
    Available,
    Assigned { agent: u32 },
    Delivered,
}
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub struct ResourceView {
    pub resource: Resource,
    pub state: ResourceState,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
pub struct Inventory {
    pub initial: u32,
    pub available: u32,
    pub assigned: u32,
    pub delivered: u32,
}
// Temporary staging allowance: world/controller consumers land in Task 4.
#[allow(dead_code)]
#[derive(Clone, Debug, PartialEq)]
pub(super) struct Ledger {
    resources: Vec<ResourceView>,
}
// Temporary staging allowance: world/controller consumers land in Task 4.
#[allow(dead_code)]
impl Ledger {
    pub(super) fn new(setup: &Setup) -> Self {
        let mut resources: Vec<_> = setup
            .resources
            .iter()
            .cloned()
            .map(|resource| ResourceView {
                resource,
                state: ResourceState::Available,
            })
            .collect();
        resources.sort_by_key(|view| view.resource.id);
        Self { resources }
    }

    pub(super) fn claim(
        &mut self,
        setup: &Setup,
        cell: Pos,
        agent: u32,
    ) -> Result<Option<(u64, FindRecord)>, Vec<FieldError>> {
        let Some(index) = self
            .resources
            .iter()
            .position(|view| view.resource.pos == cell && view.state == ResourceState::Available)
        else {
            return Ok(None);
        };
        if self
            .resources
            .iter()
            .any(|view| view.state == (ResourceState::Assigned { agent }))
        {
            return Err(vec![FieldError::new(
                format!("agents[{agent}].cargo"),
                "cannot claim another resource while carrying one",
            )]);
        }
        // Count before assignment: the center contributes exactly once and only
        // currently available Moore neighbors contribute to the frozen find.
        let count = self
            .resources
            .iter()
            .filter(|view| {
                view.state == ResourceState::Available
                    && view.resource.pos.x.abs_diff(cell.x) <= 1
                    && view.resource.pos.y.abs_diff(cell.y) <= 1
            })
            .count() as u32;
        let id = self.resources[index].resource.id;
        self.resources[index].state = ResourceState::Assigned { agent };
        Ok(Some((
            id,
            FindRecord {
                site: setup.site(cell),
                count,
            },
        )))
    }

    pub(super) fn deposit(&mut self, resource: u64, agent: u32) -> Result<(), Vec<FieldError>> {
        let index = self
            .resources
            .binary_search_by_key(&resource, |view| view.resource.id)
            .map_err(|_| {
                vec![FieldError::new(
                    format!("resources[{resource}]"),
                    "unknown resource identity",
                )]
            })?;
        if self.resources[index].state != (ResourceState::Assigned { agent }) {
            return Err(vec![FieldError::new(
                format!("resources[{resource}].state"),
                format!("deposit requires assignment to agent {agent}"),
            )]);
        }
        self.resources[index].state = ResourceState::Delivered;
        Ok(())
    }

    pub(super) fn inventory(&self) -> Inventory {
        let mut inventory = Inventory {
            initial: self.resources.len() as u32,
            available: 0,
            assigned: 0,
            delivered: 0,
        };
        for view in &self.resources {
            match view.state {
                ResourceState::Available => inventory.available += 1,
                ResourceState::Assigned { .. } => inventory.assigned += 1,
                ResourceState::Delivered => inventory.delivered += 1,
            }
        }
        inventory
    }

    pub(super) fn views(&self) -> &[ResourceView] {
        &self.resources
    }

    pub(super) fn check(&self, agents: &[Agent]) -> Result<(), Vec<FieldError>> {
        let mut errors = Vec::new();
        let inventory = self.inventory();
        if inventory.available + inventory.assigned + inventory.delivered != inventory.initial {
            errors.push(FieldError::new(
                "inventory",
                "resource counts must sum to initial",
            ));
        }
        for (index, agent) in agents.iter().enumerate() {
            if agents[..index].iter().any(|other| other.id == agent.id) {
                errors.push(FieldError::new(
                    format!("agents[{}].id", agent.id),
                    "duplicate agent identity",
                ));
            }
            if let Some(resource) = agent.cargo {
                let matches = self.resources.iter().any(|view| {
                    view.resource.id == resource
                        && view.state == (ResourceState::Assigned { agent: agent.id })
                });
                if !matches {
                    errors.push(FieldError::new(
                        format!("agents[{}].cargo", agent.id),
                        format!("resource {resource} must be assigned to this agent"),
                    ));
                }
            }
        }
        for view in &self.resources {
            if let ResourceState::Assigned { agent } = view.state {
                let owners = agents
                    .iter()
                    .filter(|worker| worker.id == agent && worker.cargo == Some(view.resource.id))
                    .count();
                if owners != 1 {
                    errors.push(FieldError::new(
                        format!("resources[{}].state", view.resource.id),
                        "assigned resource must have exactly one matching cargo owner",
                    ));
                }
            }
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}
