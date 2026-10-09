//! Source records; controller inputs never read these research DTOs.
use super::{state::*, WorkCounters};
use crate::geometry::Pos;
use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Cell {
    pub site: u32,
    pub food: f64,
    pub capacity: f64,
    pub wall: u8,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MemoryRecord {
    pub site: u32,
    pub levels: Vec<f64>,
    pub most: Vec<f64>,
    pub tick: u64,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MotionPlan {
    pub target: Option<Pos>,
    pub path: Vec<Pos>,
    pub walked: bool,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Actor {
    pub id: u64,
    pub pos: Pos,
    pub holdings: f64,
    pub metabolism: u32,
    pub vision: u32,
    pub remembers: bool,
    pub age: u32,
    pub max_age: u32,
    pub memory: Vec<MemoryRecord>,
    pub motion_plan: MotionPlan,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LegacyPlan {
    pub steps: Vec<(Pos, f64)>,
    pub goal: f64,
    pub gathers: f64,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Frame {
    pub tick: u64,
    pub fingerprint: String,
    pub rng_state_json: String,
    pub actor: Option<Actor>,
    pub cells: Vec<Cell>,
    pub task: TaskState,
    pub legacy_plan: Option<LegacyPlan>,
    pub observation: Option<Observation>,
    pub receipt: Option<PhysicalReceipt>,
    pub work: Option<WorkCounters>,
    pub controller_seconds: Option<f64>,
    pub external_added: f64,
    pub external_removed: f64,
    pub consumed: f64,
    pub death_loss: f64,
    pub living_ticks: u64,
    pub errors: Vec<String>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EpisodeRecord {
    pub schema: String,
    pub lab: LabConfig,
    pub seed: u64,
    pub completed_ticks: u64,
    pub frames: Vec<Frame>,
    pub quota_attained: bool,
    pub first_completion: Option<u64>,
    pub restricted_completion_ticks: u64,
    pub right_censored: bool,
    pub unattained_reason: Option<UnattainedReason>,
    pub gross_gathered: f64,
    pub living_ticks: u64,
    pub alive_at_horizon: bool,
    pub errors: Vec<String>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EpisodeFailure {
    pub message: String,
    pub partial: Option<Box<EpisodeRecord>>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RunOptions {
    pub diagnostics: bool,
    pub controller_timing: bool,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum UnattainedReason {
    DiedBeforeQuota,
    HorizonWithoutQuota,
}
