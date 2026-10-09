//! Closed configuration and authoritative food-task wire state.
use super::runtime::{Status, TreeState};
use crate::geometry::Pos;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Profile {
    BookLeaf,
    UtilityLeaf,
    GuardedRate,
    UnguardedRate,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Settings {
    pub profile: Profile,
    pub visits: u16,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            profile: Profile::BookLeaf,
            visits: 64,
        }
    }
}
impl Settings {
    pub fn is_default(&self) -> bool {
        self == &Self::default()
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Controller {
    ReactiveUtility,
    GuardedTree,
    MatchedFsm,
    UnguardedTree,
    TaskGoap,
    LegacyGoap,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Scenario {
    Stable,
    BetterAlternative,
    DepletedTarget,
    TemporaryObstacle,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LabConfig {
    pub controller: Controller,
    pub scenario: Scenario,
    pub quota: u32,
    pub mirrored: bool,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Candidate {
    pub site: u32,
    pub pos: Pos,
    pub distance: u32,
    pub value: f64,
    pub remembered: bool,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Observation {
    pub action_tick: u64,
    pub origin: Pos,
    pub quota: u32,
    pub gross: f64,
    pub candidates: Vec<Candidate>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TaskPlan {
    pub goal: f64,
    pub steps: Vec<(Pos, f64)>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TaskState {
    pub quota: u32,
    pub gross: f64,
    pub first_completion: Option<u64>,
    pub target: Option<u32>,
    pub failed_until: BTreeMap<u32, u64>,
    pub tree: TreeState,
    pub fsm: FsmState,
    pub task_plan: Option<TaskPlan>,
}
impl TaskState {
    pub fn new(quota: u32) -> Self {
        Self {
            quota,
            gross: 0.0,
            first_completion: None,
            target: None,
            failed_until: BTreeMap::new(),
            tree: TreeState::default(),
            fsm: FsmState {
                phase: FsmPhase::Selecting,
            },
            task_plan: None,
        }
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
// Phase is a redundant projection of valid target/gross/failure control state.
// Saved-data validation must check phase coherence, not treat phase alone as authority.
pub struct FsmState {
    pub phase: FsmPhase,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FsmPhase {
    Selecting,
    Moving,
    Deferred,
    Finished,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PhysicalReceipt {
    pub action_tick: u64,
    pub actor: u64,
    pub origin: Pos,
    pub target: Pos,
    pub destination: Pos,
    pub gathered: f64,
    pub route_failed: bool,
}
#[derive(Clone, Debug, PartialEq)]
pub struct Turn {
    pub harvest: crate::rules::Harvest,
    pub receipt: Option<PhysicalReceipt>,
    pub status: Status,
    pub visits: u16,
    pub exhausted: bool,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PolicyError {
    pub message: String,
}
impl std::fmt::Display for PolicyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}
impl std::error::Error for PolicyError {}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LabRuntime {
    pub task: TaskState,
    pub living_ticks: u64,
    pub external_added: f64,
    pub external_removed: f64,
    pub consumed: f64,
    pub death_loss: f64,
    pub diagnostics: bool,
    pub controller_timing: bool,
    pub fatal_error: Option<String>,
    pub errors: Vec<String>,
    pub(crate) observation: Option<Observation>,
    pub(crate) receipt: Option<PhysicalReceipt>,
    pub(crate) controller_seconds: Option<f64>,
}
