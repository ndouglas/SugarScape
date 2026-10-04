//! Stable engine identities, finite diagnostic records and scientific census.
use serde::{Deserialize, Serialize};
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct StateId {
    pub capital_cell: usize,
    pub sovereignty_generation: u64,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Regime {
    Democratic,
    Predatory,
}
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Cell {
    pub id: usize,
    pub owner: StateId,
    pub initial_regime: Regime,
    pub latent_regime: Regime,
    pub next_generation: u64,
}
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct State {
    pub id: StateId,
    pub regime: Regime,
    pub resources: f64,
    pub members: Vec<usize>,
}
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct Ratio {
    pub numerator: f64,
    pub denominator: f64,
    pub tag: String,
    pub value: Option<f64>,
}
impl Ratio {
    pub(crate) fn new(n: f64, d: f64) -> Self {
        Self {
            numerator: n,
            denominator: d,
            tag: if d == 0. {
                if n == 0. {
                    "equal_zero"
                } else {
                    "positive_over_zero"
                }
            } else {
                "finite"
            }
            .into(),
            value: (d > 0. && (n / d).is_finite()).then_some(n / d),
        }
    }
}
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct Allocation {
    pub fixed: f64,
    pub mobile_pool: f64,
    pub eligible_fronts: usize,
    pub old_opposing: f64,
    pub enemy_total: f64,
    pub inactive_term: f64,
    pub active: bool,
}
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Front {
    pub states: [StateId; 2],
    pub previous: [bool; 2],
    pub actions: [bool; 2],
    pub old_commitments: [f64; 2],
    pub commitments: [f64; 2],
    pub allocations: [Allocation; 2],
    pub initiations: [bool; 2],
    pub previous_initiations: [bool; 2],
    pub obligations: [Vec<String>; 2],
    pub path: Option<[usize; 2]>,
    pub path_proposer: Option<StateId>,
    pub attack_probabilities: [Option<f64>; 2],
    pub attack_ratios: [Option<Ratio>; 2],
    pub victory_probabilities: [Option<f64>; 2],
    pub victory_ratios: [Option<Ratio>; 2],
    pub claims: [bool; 2],
}
impl Front {
    pub(crate) fn new(states: [StateId; 2]) -> Self {
        Self {
            states,
            previous: [false; 2],
            actions: [false; 2],
            old_commitments: [0.; 2],
            commitments: [0.; 2],
            allocations: Default::default(),
            initiations: [false; 2],
            previous_initiations: [false; 2],
            obligations: Default::default(),
            path: None,
            path_proposer: None,
            attack_probabilities: [None; 2],
            attack_ratios: Default::default(),
            victory_probabilities: [None; 2],
            victory_ratios: Default::default(),
            claims: [false; 2],
        }
    }
}
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Alliance {
    pub threat_id: StateId,
    pub creation_period: u64,
    pub serial: u64,
    pub members: Vec<StateId>,
    pub pooled_resources: f64,
}
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct Counters {
    pub initiated_fronts: u64,
    pub mutual_d_front_periods: u64,
    pub completed_victory_battles: u64,
    pub completed_stalemate_battles: u64,
    pub opposing_claims: u64,
    pub successful_claims: u64,
    pub stale_claims: u64,
    pub locked_claims: u64,
    pub retired_states: u64,
    pub released_states: u64,
}
pub(crate) fn bump(n: &mut u64) -> Result<(), String> {
    *n = n.checked_add(1).ok_or("census counter overflow")?;
    Ok(())
}
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct SetupCensus {
    pub initial_democratic_cells: u32,
    pub initial_resourced_cells: u32,
    pub total_cells: u32,
}
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct Metrics {
    pub democratic_cells: u32,
    pub total_cells: u32,
    pub democratic_share: f64,
    pub sovereign_count: u32,
    pub democratic_states: u32,
    pub predatory_states: u32,
    pub democratic_mean_size: Option<f64>,
    pub predatory_mean_size: Option<f64>,
    pub democratic_max_size: Option<u32>,
    pub predatory_max_size: Option<u32>,
    pub democratic_size_reason: Option<String>,
    pub predatory_size_reason: Option<String>,
    pub democratic_exposure: Option<f64>,
    pub clustering_ratio: Option<f64>,
    pub clustering_reason: Option<String>,
    pub conflict_fronts: u32,
    pub alliance_count: u32,
    pub pariah_count: u32,
    pub democratic_extinction: bool,
    pub all_democratic: bool,
    pub first_extinction_period: Option<u64>,
    pub first_all_democratic_period: Option<u64>,
}
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Outcome {
    pub valid: bool,
    pub finish_reason: String,
    pub invalid_reason: Option<String>,
    pub invalid_phase: Option<String>,
    pub attempted_period: u64,
    pub completed_periods: u64,
    pub final_metrics: Option<Metrics>,
    pub census: Counters,
}
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Event {
    pub period: u64,
    pub kind: String,
    pub states: Vec<StateId>,
    pub cells: Vec<usize>,
}
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Extraction {
    pub state: StateId,
    pub resources_before: f64,
    pub resources_after: f64,
    pub province_terms: Vec<(usize, f64, f64)>,
}
#[derive(Clone, Debug)]
pub(crate) struct Claim {
    pub states: [StateId; 2],
    pub side: usize,
    pub path: [usize; 2],
}
