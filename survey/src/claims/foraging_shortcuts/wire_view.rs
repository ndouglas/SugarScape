//! Strict saved observations; these types cannot restore a core World.
use super::wire::{WireFloat, WirePos};
use super::wire_state::{
    WireAccessCompute, WireCargo, WireComputeCounts, WireFoodInventory, WireFoodPhase,
    WireFoodView, WireMode, WireSpoilInventory, WireSpoilView, WireTerrainInventory,
    WireWorkCounts,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct WireEventContext {
    pub(super) tick: u32,
    pub(super) opportunity: u64,
    pub(super) worker: u32,
    pub(super) excavated: u32,
    pub(super) spoil_disposed: u32,
    pub(super) food_delivered: u32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct WireEventMilestone {
    pub(super) context: WireEventContext,
    pub(super) pos: WirePos,
    pub(super) nest_distance: u32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct WireFoodAccessRecord {
    pub(super) id: u64,
    pub(super) initially_exposed: bool,
    pub(super) initially_accessible: bool,
    #[serde(deserialize_with = "super::wire::required_option")]
    pub(super) first_exposure: Option<WireEventMilestone>,
    #[serde(deserialize_with = "super::wire::required_option")]
    pub(super) first_access: Option<WireEventMilestone>,
    pub(super) accessible: bool,
    #[serde(deserialize_with = "super::wire::required_option")]
    pub(super) distance: Option<u32>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct WireAccessSummary {
    pub(super) initially_exposed: u32,
    pub(super) initially_accessible: u32,
    pub(super) accessible: u32,
    pub(super) records: Vec<WireFoodAccessRecord>,
    pub(super) compute: WireAccessCompute,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct WireMilestones {
    #[serde(deserialize_with = "super::wire::required_option")]
    pub(super) first_excavation: Option<WireEventMilestone>,
    #[serde(deserialize_with = "super::wire::required_option")]
    pub(super) first_exposure: Option<WireEventMilestone>,
    #[serde(deserialize_with = "super::wire::required_option")]
    pub(super) first_access: Option<WireEventMilestone>,
    #[serde(deserialize_with = "super::wire::required_option")]
    pub(super) first_disposal: Option<WireEventMilestone>,
    #[serde(deserialize_with = "super::wire::required_option")]
    pub(super) first_pickup_tick: Option<u32>,
    #[serde(deserialize_with = "super::wire::required_option")]
    pub(super) first_delivery_tick: Option<u32>,
    #[serde(deserialize_with = "super::wire::required_option")]
    pub(super) all_food_delivered_tick: Option<u32>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct WireFindView {
    pub(super) site: WirePos,
    pub(super) count: u32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct WireWaypointView {
    pub(super) id: u64,
    pub(super) site: WirePos,
    pub(super) created_tick: u32,
    pub(super) strength: WireFloat,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct WireAgentView {
    pub(super) id: u32,
    pub(super) pos: WirePos,
    pub(super) phase: WireFoodPhase,
    pub(super) mode: WireMode,
    #[serde(deserialize_with = "super::wire::required_option")]
    pub(super) cargo: Option<WireCargo>,
    #[serde(deserialize_with = "super::wire::required_option")]
    pub(super) find: Option<WireFindView>,
    #[serde(deserialize_with = "super::wire::required_option")]
    pub(super) site: Option<WirePos>,
    #[serde(deserialize_with = "super::wire::required_option")]
    pub(super) frontier: Option<WirePos>,
    #[serde(deserialize_with = "super::wire::required_option")]
    pub(super) face: Option<WirePos>,
    pub(super) known_open: u32,
    pub(super) known_solid: u32,
    pub(super) known_diggable: u32,
    pub(super) work: WireWorkCounts,
    pub(super) compute: WireComputeCounts,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct WireSummary {
    pub(super) completed_ticks: u32,
    pub(super) food: WireFoodInventory,
    pub(super) spoil: WireSpoilInventory,
    pub(super) terrain: WireTerrainInventory,
    pub(super) work: WireWorkCounts,
    pub(super) compute: WireComputeCounts,
    pub(super) per_agent_work: Vec<WireWorkCounts>,
    pub(super) per_agent_compute: Vec<WireComputeCounts>,
    pub(super) expired_records: u64,
    pub(super) access: WireAccessSummary,
    pub(super) milestones: WireMilestones,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct WireSnapshot {
    pub(super) summary: WireSummary,
    pub(super) open: Vec<WirePos>,
    pub(super) nest: Vec<WirePos>,
    pub(super) waste: WirePos,
    pub(super) agents: Vec<WireAgentView>,
    pub(super) food: Vec<WireFoodView>,
    pub(super) spoil: Vec<WireSpoilView>,
    pub(super) waypoints: Vec<WireWaypointView>,
}
