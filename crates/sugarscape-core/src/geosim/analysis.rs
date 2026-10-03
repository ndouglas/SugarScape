//! Complete source census, clocks and resource recurrence accounting.
use super::{Cell, GeosimConfig, Merge, State, StateId, War};
use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Ledger {
    pub attacks: u64,
    pub fighting_front_periods: u64,
    pub mutual_front_periods: u64,
    pub conquests: u64,
    pub collapses: u64,
    pub disconnections: u64,
    pub stale_claims: u64,
    pub locked_claims: u64,
    pub double_successes: u64,
    pub path_collisions: u64,
    pub shocks: u64,
    pub damage: f64,
    pub measured_damage: f64,
    pub capacity_increase: f64,
    pub capacity_decrease: f64,
    pub clipping: f64,
    pub retirement_capacity: f64,
    pub reemergence_capacity: f64,
    pub recurrence_residual: f64,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Event {
    pub id: u64,
    pub period: u64,
    pub kind: String,
    pub states: Vec<StateId>,
    pub cells: Vec<usize>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Outcome {
    pub config: GeosimConfig,
    pub seed: u64,
    pub rng_mode: String,
    pub periods: u64,
    pub attempted_period: u64,
    pub counting_start: u64,
    pub valid: bool,
    pub state_available: bool,
    pub finish_reason: String,
    pub invalid_reason: Option<String>,
    pub completed_wars: Vec<War>,
    pub censored_wars: Vec<War>,
    pub legacy_visible_wars: Vec<War>,
    pub exporter_backlog: Vec<War>,
    pub merges: Vec<Merge>,
    pub retired_states: Vec<StateId>,
    pub sovereign_count: usize,
    pub states: Vec<State>,
    pub cells: Vec<Cell>,
    pub ledger: Ledger,
    pub fronts: Vec<super::Front>,
    pub resource_updates: Vec<ResourceUpdate>,
    pub partial_period_fights: Vec<(StateId, StateId, f64)>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ResourceUpdate {
    pub state: StateId,
    pub period: u64,
    pub old_capacity: Option<f64>,
    pub extracted_yield: Option<f64>,
    pub applied_damage: Option<f64>,
    pub target_capacity: Option<f64>,
    pub new_capacity: Option<f64>,
    pub clipping: f64,
    pub residual: Option<f64>,
    pub reset: bool,
}
