use super::config::*;
use serde::{Deserialize, Serialize};
pub type AgentId = u16;
pub type CapabilityId = u16;
pub type Round = u32;
pub const PROTOCOL_VERSION: u16 = 1;
pub const RULES_VERSION: u16 = 1;
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Phase {
    Attention,
    Act,
    Discuss,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    Active,
    Inactive,
    Silenced,
}
impl Status {
    pub fn is_active(self) -> bool {
        self != Self::Inactive
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ObjectiveTeam {
    Threat,
    Accuser,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Action {
    Pass,
    Watch {
        agents: Vec<AgentId>,
    },
    Use {
        capability: CapabilityId,
        target: AgentId,
    },
    Say {
        claim: Claim,
    },
    Accuse {
        target: AgentId,
    },
}
// A zero-field struct variant enforces unknown-field rejection; serde's unit
// variant otherwise ignores extra fields in an internally tagged object.
impl<'de> Deserialize<'de> for Action {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
        enum WireAction {
            Pass {},
            Watch {
                agents: Vec<AgentId>,
            },
            Use {
                capability: CapabilityId,
                target: AgentId,
            },
            Say {
                claim: Claim,
            },
            Accuse {
                target: AgentId,
            },
        }
        Ok(match WireAction::deserialize(deserializer)? {
            WireAction::Pass {} => Self::Pass,
            WireAction::Watch { agents } => Self::Watch { agents },
            WireAction::Use { capability, target } => Self::Use { capability, target },
            WireAction::Say { claim } => Self::Say { claim },
            WireAction::Accuse { target } => Self::Accuse { target },
        })
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Claim {
    Suspect {
        agent: AgentId,
    },
    SawUse {
        source: AgentId,
        target: AgentId,
        round: Round,
    },
    DenyUse {
        round: Round,
    },
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TurnResponse {
    pub request_id: u64,
    pub actor: AgentId,
    pub action: Action,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TurnRequest {
    pub protocol_version: u16,
    pub request_id: u64,
    pub actor: AgentId,
    pub round: Round,
    pub phase: Phase,
    pub observation: Observation,
    pub legal: LegalActions,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RosterEntry {
    pub id: AgentId,
    pub status: Status,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Observation {
    pub roster: Vec<RosterEntry>,
    pub grants: Vec<CapabilityId>,
    pub accusation_budget: u16,
    pub events: Vec<VisibleEvent>,
    pub capabilities: Vec<CapabilitySpec>,
    pub objectives: ObjectiveRules,
    pub rules: PublicRules,
    pub objective: ObjectiveTeam,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PublicRules {
    pub display_name: String,
    pub max_rounds: Round,
    pub observation: ObservationRules,
    pub memory: MemoryRules,
    pub accusation: AccusationRules,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LegalActions {
    pub targets: Vec<AgentId>,
    pub capabilities: Vec<CapabilityId>,
    pub attention_capacity: u16,
    pub accusation_budget: u16,
    pub can_watch: bool,
    pub can_use: bool,
    pub can_say: bool,
    pub can_accuse: bool,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VisibleEvent {
    pub sequence: u64,
    pub round: Round,
    pub content: EventContent,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum EventContent {
    Use {
        source: AgentId,
        target: AgentId,
        capability: CapabilityId,
    },
    RecipientNotice {
        target: AgentId,
        capability: CapabilityId,
    },
    StatusChange {
        agent: AgentId,
        status: Status,
    },
    Claim {
        source: AgentId,
        claim: Claim,
    },
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OutcomeReason {
    NoActiveThreatHolders,
    SurvivorThreshold,
    Horizon,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Outcome {
    pub winner: ObjectiveTeam,
    pub reason: OutcomeReason,
    pub completed_rounds: Round,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ActionError {
    InvalidResponse,
}
impl std::fmt::Display for ActionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("invalid response")
    }
}
impl std::error::Error for ActionError {}
pub trait Controller {
    fn respond(&mut self, request: &TurnRequest) -> TurnResponse;
}
