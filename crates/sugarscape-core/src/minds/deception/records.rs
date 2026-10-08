//! Research records. These DTOs are never inputs to a controller or receiver.
use super::{
    controller::BoutResult,
    observation::Observation,
    state::{LabConfig, SenderState},
};
use crate::{geometry::Pos, minds::protection::ledger::Ledger};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EpisodeRecord {
    pub schema: String,
    pub lab: LabConfig,
    pub seed: u64,
    pub requested_ticks: u64,
    pub completed_ticks: u64,
    pub owner_ticks_alive: u64,
    pub owner_alive: bool,
    pub frames: Vec<FrameRecord>,
    pub fixture_errors: Vec<String>,
    pub ledger_errors: Vec<String>,
    pub cohorts: Option<Ledger>,
    pub thief_transferred: Option<f64>,
    pub diagnostics_enabled: bool,
    pub lineage_unavailable_reason: Option<String>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FrameRecord {
    pub tick: u64,
    pub fingerprint: String,
    pub roles: Vec<RoleRecord>,
    pub actions: Vec<ActionRecord>,
    pub observations: Vec<ObservedRecord>,
    pub choices: Vec<ChoiceRecord>,
    pub deaths: Vec<DeathRecord>,
    pub restrictions: BTreeMap<u64, u64>,
    pub stocks: Vec<StockRecord>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RoleRecord {
    pub id: u64,
    pub pos: Pos,
    pub holdings: f64,
    pub caches: BTreeMap<u32, f64>,
    pub sender: Option<SenderState>,
    pub seen: Vec<SeenRecord>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SeenRecord {
    pub site: u32,
    pub owner: u64,
    pub amount: f64,
    pub tick: u64,
}
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ActionRecord {
    pub actor: u64,
    pub phase: String,
    pub action: String,
    pub target: Option<Pos>,
    pub harvest: f64,
    pub dug: f64,
    pub buried: f64,
    pub effort: f64,
    pub metabolic_demand: f64,
    pub metabolic_consumed: f64,
    pub cancellation: Option<BoutResult>,
    pub pos: Option<Pos>,
    pub walk_outcome: Option<String>,
    pub target_occupant: Option<u64>,
    pub source_recovered: bool,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ObservedRecord {
    pub receiver: u64,
    pub public: Observation,
    pub actual_transfer: f64,
    pub actual_stock: f64,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ChoiceRecord {
    pub actor: u64,
    pub target: Pos,
    pub remembered_value: f64,
    pub actual_value: f64,
    pub arrived: bool,
    pub raid_amount: f64,
    pub wasted: bool,
    pub inspection: Option<Pos>,
    pub inspected_stock: Option<f64>,
    pub target_occupant: Option<u64>,
    pub source_recovered: bool,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DeathRecord {
    pub actor: u64,
    pub pos: Pos,
    pub cause: String,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct StockRecord {
    pub site: u32,
    pub amount: f64,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EpisodeFailure {
    pub message: String,
    pub partial: Option<EpisodeRecord>,
}
