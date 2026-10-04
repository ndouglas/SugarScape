use super::types::*;
pub use crate::config::FieldError;
use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ScenarioConfig {
    pub version: u16,
    pub display_name: String,
    pub agents: Vec<AgentSpec>,
    pub capabilities: Vec<CapabilitySpec>,
    pub assignment: Option<SeededAssignment>,
    pub observation: ObservationRules,
    pub memory: MemoryRules,
    pub accusation: AccusationRules,
    pub objectives: ObjectiveRules,
    pub max_rounds: Round,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AgentSpec {
    pub id: AgentId,
    pub grants: Vec<CapabilityId>,
    pub objective: ObjectiveTeam,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CapabilitySpec {
    pub id: CapabilityId,
    pub label: String,
    pub effect: Effect,
    pub delay: Round,
    pub target: TargetRule,
    pub visibility: Visibility,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Effect {
    SetStatus { status: Status },
    Grant { capability: CapabilityId },
    Revoke { capability: CapabilityId },
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TargetRule {
    SelfOnly,
    OtherActive,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Visibility {
    Public,
    Recipient,
    Watched,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SeededAssignment {
    pub capability: CapabilityId,
    pub eligible: Vec<AgentId>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ObservationRules {
    pub attention_capacity: u16,
    pub detection_per_mille: u16,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MemoryRules {
    pub capacity: usize,
    pub retention_rounds: Round,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WrongCost {
    SpendOnly,
    DeactivateAccuser,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AccusationRules {
    pub enabled: bool,
    pub budget: u16,
    pub eligible: Vec<AgentId>,
    pub wrong_cost: WrongCost,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ObjectiveRules {
    pub capability: CapabilityId,
    pub accuser_team: Vec<AgentId>,
    pub non_holder_survivor_threshold: u16,
    pub horizon_winner: ObjectiveTeam,
}
fn unique<T: Ord + Copy>(values: &[T]) -> bool {
    values
        .iter()
        .copied()
        .collect::<std::collections::BTreeSet<_>>()
        .len()
        == values.len()
}
impl ScenarioConfig {
    pub fn validate(&self) -> Result<(), Vec<FieldError>> {
        let mut errors = vec![];
        let mut check = |ok: bool, field: &str| {
            if !ok {
                errors.push(FieldError::new(field, "invalid bound or reference"));
            }
        };
        let n = self.agents.len();
        let known_agent = |id: AgentId| usize::from(id) < n;
        let known_cap = |id: CapabilityId| self.capabilities.iter().any(|c| c.id == id);
        check(self.version == RULES_VERSION, "version");
        check((2..=32).contains(&n), "agents");
        check(
            self.agents
                .iter()
                .enumerate()
                .all(|(i, a)| usize::from(a.id) == i),
            "agents.id",
        );
        check((1..=32).contains(&self.capabilities.len()), "capabilities");
        check(
            unique(&self.capabilities.iter().map(|c| c.id).collect::<Vec<_>>()),
            "capabilities.id",
        );
        check((1..=1000).contains(&self.max_rounds), "max_rounds");
        check(
            usize::from(self.observation.attention_capacity) < n,
            "observation.attention_capacity",
        );
        check(
            self.observation.detection_per_mille <= 1000,
            "observation.detection_per_mille",
        );
        check(self.memory.capacity <= 4096, "memory.capacity");
        check(
            self.memory.retention_rounds <= 1000,
            "memory.retention_rounds",
        );
        check(self.accusation.budget <= 32, "accusation.budget");
        check(
            !self.accusation.enabled || !self.accusation.eligible.is_empty(),
            "accusation.eligible",
        );
        check(
            unique(&self.accusation.eligible)
                && self.accusation.eligible.iter().all(|id| known_agent(*id)),
            "accusation.eligible",
        );
        check(
            !self.objectives.accuser_team.is_empty()
                && unique(&self.objectives.accuser_team)
                && self
                    .objectives
                    .accuser_team
                    .iter()
                    .all(|id| known_agent(*id)),
            "objectives.accuser_team",
        );
        check(
            (self.objectives.non_holder_survivor_threshold as usize) < n,
            "objectives.non_holder_survivor_threshold",
        );
        check(
            known_cap(self.objectives.capability),
            "objectives.capability",
        );
        for a in &self.agents {
            check(
                unique(&a.grants) && a.grants.iter().all(|id| known_cap(*id)),
                "agents.grants",
            );
        }
        for c in &self.capabilities {
            check(
                c.delay <= 1000 && self.max_rounds.checked_add(c.delay).is_some(),
                "capabilities.delay",
            );
            check(
                match c.effect {
                    Effect::SetStatus { status } => status != Status::Active,
                    Effect::Grant { capability } | Effect::Revoke { capability } => {
                        known_cap(capability)
                    }
                },
                "capabilities.effect",
            );
        }
        if let Some(a) = &self.assignment {
            check(known_cap(a.capability), "assignment.capability");
            check(
                !a.eligible.is_empty()
                    && unique(&a.eligible)
                    && a.eligible.iter().all(|id| known_agent(*id)),
                "assignment.eligible",
            );
            check(
                a.capability == self.objectives.capability,
                "assignment.capability",
            );
        }
        check(
            self.assignment.is_some()
                || self
                    .agents
                    .iter()
                    .any(|a| a.grants.contains(&self.objectives.capability)),
            "objectives.initial_holder",
        );
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
    pub(crate) fn normalize(&mut self) {
        self.capabilities.sort_by_key(|c| c.id);
        for a in &mut self.agents {
            a.grants.sort_unstable();
        }
        self.accusation.eligible.sort_unstable();
        self.objectives.accuser_team.sort_unstable();
        if let Some(a) = &mut self.assignment {
            a.eligible.sort_unstable();
        }
    }
}
