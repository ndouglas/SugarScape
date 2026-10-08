//! Strict saved observations; these types cannot restore a core World.
use super::wire::{WirePos, WireResource};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub(super) struct WireWorkCounts {
    pub(super) opportunities: u64,
    pub(super) moves: u64,
    pub(super) digs: u64,
    pub(super) pickups: u64,
    pub(super) deposits: u64,
    pub(super) disposals: u64,
    pub(super) waits: u64,
    pub(super) departure_moves: u64,
    pub(super) search_moves: u64,
    pub(super) empty_return_moves: u64,
    pub(super) food_moves: u64,
    pub(super) spoil_moves: u64,
    pub(super) transition_waits: u64,
    pub(super) empty_arrival_waits: u64,
    pub(super) no_neighbor_waits: u64,
    pub(super) empty_congestion_waits: u64,
    pub(super) food_congestion_waits: u64,
    pub(super) spoil_congestion_waits: u64,
    pub(super) search_entries: u64,
    pub(super) empty_returns: u64,
    pub(super) fidelity_departures: u64,
    pub(super) recruited_departures: u64,
    pub(super) uninformed_departures: u64,
    pub(super) publications: u64,
    pub(super) abandoned_targets: u64,
    pub(super) spoil_hauls: u64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) enum WireCargo {
    Food(u64),
    Spoil(u64),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) enum WireFoodPhase {
    Departing,
    Searching,
    Returning,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) enum WireMode {
    Departing,
    Searching,
    EmptyReturning,
    FoodReturning,
    SpoilHauling,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub(super) struct WireComputeCounts {
    pub(super) observations: u64,
    pub(super) cells_inspected: u64,
    pub(super) cells_learned: u64,
    pub(super) observed_revisions: u64,
    pub(super) dig_confirmations: u64,
    pub(super) face_scans: u64,
    pub(super) route_calls: u64,
    pub(super) route_visits: u64,
    pub(super) peak_queue: u64,
    pub(super) frontier_scans: u64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub(super) struct WireAccessCompute {
    pub(super) calls: u64,
    pub(super) visits: u64,
    pub(super) peak_queue: u64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub(super) struct WireTerrainInventory {
    pub(super) initial_open: u32,
    pub(super) open: u32,
    pub(super) excavated: u32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) enum WireFoodState {
    Hidden,
    Available,
    Carried { agent: u32 },
    Delivered,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct WireFoodView {
    pub(super) resource: WireResource,
    pub(super) state: WireFoodState,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub(super) struct WireFoodInventory {
    pub(super) initial: u32,
    pub(super) hidden: u32,
    pub(super) available: u32,
    pub(super) carried: u32,
    pub(super) delivered: u32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) enum WireSpoilState {
    Carried { agent: u32 },
    Disposed { tick: u32 },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct WireSpoilView {
    pub(super) id: u64,
    pub(super) origin: WirePos,
    pub(super) creator: u32,
    pub(super) born_tick: u32,
    pub(super) state: WireSpoilState,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub(super) struct WireSpoilInventory {
    pub(super) excavated: u32,
    pub(super) carried: u32,
    pub(super) disposed: u32,
}

impl WireWorkCounts {
    pub(super) fn fields(&self) -> [(&'static str, u64); 26] {
        [
            ("opportunities", self.opportunities),
            ("moves", self.moves),
            ("digs", self.digs),
            ("pickups", self.pickups),
            ("deposits", self.deposits),
            ("disposals", self.disposals),
            ("waits", self.waits),
            ("departure_moves", self.departure_moves),
            ("search_moves", self.search_moves),
            ("empty_return_moves", self.empty_return_moves),
            ("food_moves", self.food_moves),
            ("spoil_moves", self.spoil_moves),
            ("transition_waits", self.transition_waits),
            ("empty_arrival_waits", self.empty_arrival_waits),
            ("no_neighbor_waits", self.no_neighbor_waits),
            ("empty_congestion_waits", self.empty_congestion_waits),
            ("food_congestion_waits", self.food_congestion_waits),
            ("spoil_congestion_waits", self.spoil_congestion_waits),
            ("search_entries", self.search_entries),
            ("empty_returns", self.empty_returns),
            ("fidelity_departures", self.fidelity_departures),
            ("recruited_departures", self.recruited_departures),
            ("uninformed_departures", self.uninformed_departures),
            ("publications", self.publications),
            ("abandoned_targets", self.abandoned_targets),
            ("spoil_hauls", self.spoil_hauls),
        ]
    }
}
impl WireComputeCounts {
    pub(super) fn fields(&self) -> [(&'static str, u64); 10] {
        [
            ("observations", self.observations),
            ("cells_inspected", self.cells_inspected),
            ("cells_learned", self.cells_learned),
            ("observed_revisions", self.observed_revisions),
            ("dig_confirmations", self.dig_confirmations),
            ("face_scans", self.face_scans),
            ("route_calls", self.route_calls),
            ("route_visits", self.route_visits),
            ("peak_queue", self.peak_queue),
            ("frontier_scans", self.frontier_scans),
        ]
    }
}
