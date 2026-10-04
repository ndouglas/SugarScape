//! Complete source defaults and explicit reconstruction readings.
use crate::{
    config::FieldError,
    schema::{Apply, Param},
};
use serde::{Deserialize, Serialize};
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Mechanism {
    Tagging,
    Alliances,
    CollectiveSecurity,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProbabilityDirection {
    PrintedDecreasing,
    ProseIncreasing,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ZeroRatio {
    EqualZeroNeutral,
    RejectZeroDenominator,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Assignment {
    IndependentBernoulli,
    RoundedQuota,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DistanceMetric {
    Euclidean,
    Manhattan,
    TerritorialPath,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EnemyTotal {
    ActiveFronts,
    AllFronts,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InactiveCommitment {
    PerFrontOpponent,
    FirstInactiveOpponent,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LatentRegime {
    PersistentCellTags,
    OverwriteOnConquest,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CapitalCapture {
    CollapseOnly,
    CaptureAndFragment,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClaimLocking {
    AffectedCells,
    AffectedStates,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OpposingVictories {
    IndependentClaims,
    SingleDraw,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AllianceMaintenance {
    RebuildEachPeriod,
    PersistWhileThreatened,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ThreatTies {
    LowestStateId,
    RandomTie,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ObligationObservation {
    PriorActions,
    CurrentPlansOnce,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SecurityScope {
    AllDemocracies,
    SameAlliance,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClusteringExposure {
    UniqueStateNeighbors,
    BorderEdges,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClusteringWeights {
    TerritoryWeighted,
    EqualStates,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct DemocraticPeaceConfig {
    pub width: u32,
    pub height: u32,
    pub superiority_exponent: u32,
    pub victory_exponent: u32,
    pub horizon_periods: u32,
    pub periods_per_tick: u32,
    pub event_limit: u32,
    pub initial_democratic_share: f64,
    pub initial_resourced_share: f64,
    pub mobile_share: f64,
    pub superiority_threshold: f64,
    pub victory_threshold: f64,
    pub stalemate_probability: f64,
    pub tax_rate: f64,
    pub distance_gradient: f64,
    pub min_threat: f64,
    pub event_recording: bool,
    pub mechanism: Mechanism,
    pub probability_direction: ProbabilityDirection,
    pub zero_ratio: ZeroRatio,
    pub assignment: Assignment,
    pub distance_metric: DistanceMetric,
    pub enemy_total: EnemyTotal,
    pub inactive_commitment: InactiveCommitment,
    pub latent_regime: LatentRegime,
    pub capital_capture: CapitalCapture,
    pub claim_locking: ClaimLocking,
    pub opposing_victories: OpposingVictories,
    pub alliance_maintenance: AllianceMaintenance,
    pub threat_ties: ThreatTies,
    pub obligation_observation: ObligationObservation,
    pub security_scope: SecurityScope,
    pub clustering_exposure: ClusteringExposure,
    pub clustering_weights: ClusteringWeights,
}
impl Default for DemocraticPeaceConfig {
    fn default() -> Self {
        Self {
            width: 15,
            height: 15,
            superiority_exponent: 30,
            victory_exponent: 10,
            horizon_periods: 1000,
            periods_per_tick: 1,
            event_limit: 1000,
            initial_democratic_share: 0.1,
            initial_resourced_share: 0.05,
            mobile_share: 0.5,
            superiority_threshold: 2.2,
            victory_threshold: 2.2,
            stalemate_probability: 0.2,
            tax_rate: 1.0,
            distance_gradient: 0.95,
            min_threat: 2.0,
            event_recording: false,
            mechanism: Mechanism::CollectiveSecurity,
            probability_direction: ProbabilityDirection::PrintedDecreasing,
            zero_ratio: ZeroRatio::EqualZeroNeutral,
            assignment: Assignment::IndependentBernoulli,
            distance_metric: DistanceMetric::Euclidean,
            enemy_total: EnemyTotal::ActiveFronts,
            inactive_commitment: InactiveCommitment::PerFrontOpponent,
            latent_regime: LatentRegime::PersistentCellTags,
            capital_capture: CapitalCapture::CollapseOnly,
            claim_locking: ClaimLocking::AffectedCells,
            opposing_victories: OpposingVictories::IndependentClaims,
            alliance_maintenance: AllianceMaintenance::RebuildEachPeriod,
            threat_ties: ThreatTies::LowestStateId,
            obligation_observation: ObligationObservation::PriorActions,
            security_scope: SecurityScope::AllDemocracies,
            clustering_exposure: ClusteringExposure::UniqueStateNeighbors,
            clustering_weights: ClusteringWeights::TerritoryWeighted,
        }
    }
}
impl DemocraticPeaceConfig {
    pub fn horizon(&self) -> u64 {
        u64::from(self.horizon_periods)
    }
    pub fn validate(&self) -> Result<(), Vec<FieldError>> {
        let mut e = Vec::new();
        if !(2..=100).contains(&self.width) {
            e.push(FieldError::new("width", "outside validated integer range"));
        }
        if !(2..=100).contains(&self.height) {
            e.push(FieldError::new("height", "outside validated integer range"));
        }
        if !(1..=100).contains(&self.superiority_exponent) {
            e.push(FieldError::new(
                "superiority_exponent",
                "outside validated integer range",
            ));
        }
        if !(1..=100).contains(&self.victory_exponent) {
            e.push(FieldError::new(
                "victory_exponent",
                "outside validated integer range",
            ));
        }
        if !(1..=1000000).contains(&self.horizon_periods) {
            e.push(FieldError::new(
                "horizon_periods",
                "outside validated integer range",
            ));
        }
        if !(1..=10000).contains(&self.periods_per_tick) {
            e.push(FieldError::new(
                "periods_per_tick",
                "outside validated integer range",
            ));
        }
        if !(0..=100000).contains(&self.event_limit) {
            e.push(FieldError::new(
                "event_limit",
                "outside validated integer range",
            ));
        }
        if !self.initial_democratic_share.is_finite()
            || self.initial_democratic_share < 0.0
            || self.initial_democratic_share > 1.0
        {
            e.push(FieldError::new(
                "initial_democratic_share",
                "outside validated finite range",
            ));
        }
        if !self.initial_resourced_share.is_finite()
            || self.initial_resourced_share < 0.0
            || self.initial_resourced_share > 1.0
        {
            e.push(FieldError::new(
                "initial_resourced_share",
                "outside validated finite range",
            ));
        }
        if !self.mobile_share.is_finite() || self.mobile_share < 0.0 || self.mobile_share > 1.0 {
            e.push(FieldError::new(
                "mobile_share",
                "outside validated finite range",
            ));
        }
        if !self.superiority_threshold.is_finite() || self.superiority_threshold <= 0.0 {
            e.push(FieldError::new(
                "superiority_threshold",
                "outside validated finite range",
            ));
        }
        if !self.victory_threshold.is_finite() || self.victory_threshold <= 0.0 {
            e.push(FieldError::new(
                "victory_threshold",
                "outside validated finite range",
            ));
        }
        if !self.stalemate_probability.is_finite()
            || self.stalemate_probability < 0.0
            || self.stalemate_probability > 1.0
        {
            e.push(FieldError::new(
                "stalemate_probability",
                "outside validated finite range",
            ));
        }
        if !self.tax_rate.is_finite() || self.tax_rate < 0.0 || self.tax_rate > 1.0 {
            e.push(FieldError::new(
                "tax_rate",
                "outside validated finite range",
            ));
        }
        if !self.distance_gradient.is_finite()
            || self.distance_gradient <= 0.0
            || self.distance_gradient > 1.0
        {
            e.push(FieldError::new(
                "distance_gradient",
                "outside validated finite range",
            ));
        }
        if !self.min_threat.is_finite() || self.min_threat <= 0.0 {
            e.push(FieldError::new(
                "min_threat",
                "outside validated finite range",
            ));
        }
        if u64::from(self.width) * u64::from(self.height) > 10000 {
            e.push(FieldError::new("width", "grid exceeds10000 cells"));
        }
        if self.event_recording && self.event_limit == 0 {
            e.push(FieldError::new(
                "event_limit",
                "positive limit required while recording",
            ));
        }
        if e.is_empty() {
            Ok(())
        } else {
            Err(e)
        }
    }
}
pub fn schema() -> Vec<Param> {
    vec![
        Param::integer("Source", "width", "width", (2, 100), Apply::Reset),
        Param::integer("Source", "height", "height", (2, 100), Apply::Reset),
        Param::integer(
            "Source",
            "superiority_exponent",
            "superiority exponent",
            (1, 100),
            Apply::Reset,
        ),
        Param::integer(
            "Source",
            "victory_exponent",
            "victory exponent",
            (1, 100),
            Apply::Reset,
        ),
        Param::integer(
            "Source",
            "horizon_periods",
            "horizon periods",
            (1, 1000000),
            Apply::Reset,
        ),
        Param::integer(
            "Source",
            "periods_per_tick",
            "periods per tick",
            (1, 10000),
            Apply::Reset,
        ),
        Param::integer(
            "Source",
            "event_limit",
            "event limit",
            (0, 100000),
            Apply::Reset,
        ),
        Param::number(
            "Source",
            "initial_democratic_share",
            "initial democratic share",
            (0.0, 1.0, 0.01),
            Apply::Reset,
        ),
        Param::number(
            "Source",
            "initial_resourced_share",
            "initial resourced share",
            (0.0, 1.0, 0.01),
            Apply::Reset,
        ),
        Param::number(
            "Source",
            "mobile_share",
            "mobile share",
            (0.0, 1.0, 0.01),
            Apply::Reset,
        ),
        Param::number(
            "Source",
            "superiority_threshold",
            "superiority threshold",
            (0.0, 100.0, 0.01),
            Apply::Reset,
        ),
        Param::number(
            "Source",
            "victory_threshold",
            "victory threshold",
            (0.0, 100.0, 0.01),
            Apply::Reset,
        ),
        Param::number(
            "Source",
            "stalemate_probability",
            "stalemate probability",
            (0.0, 1.0, 0.01),
            Apply::Reset,
        ),
        Param::number(
            "Source",
            "tax_rate",
            "tax rate",
            (0.0, 1.0, 0.01),
            Apply::Reset,
        ),
        Param::number(
            "Source",
            "distance_gradient",
            "distance gradient",
            (0.0, 1.0, 0.01),
            Apply::Reset,
        ),
        Param::number(
            "Source",
            "min_threat",
            "min threat",
            (0.0, 100.0, 0.01),
            Apply::Reset,
        ),
        Param::bool(
            "Recording",
            "event_recording",
            "Record events",
            Apply::Reset,
        ),
        Param::choice(
            "Readings",
            "mechanism",
            "mechanism",
            &[
                ("tagging", "tagging"),
                ("alliances", "alliances"),
                ("collective_security", "collective security"),
            ],
            Apply::Reset,
        ),
        Param::choice(
            "Readings",
            "probability_direction",
            "probability direction",
            &[
                ("printed_decreasing", "printed decreasing"),
                ("prose_increasing", "prose increasing"),
            ],
            Apply::Reset,
        ),
        Param::choice(
            "Readings",
            "zero_ratio",
            "zero ratio",
            &[
                ("equal_zero_neutral", "equal zero neutral"),
                ("reject_zero_denominator", "reject zero denominator"),
            ],
            Apply::Reset,
        ),
        Param::choice(
            "Readings",
            "assignment",
            "assignment",
            &[
                ("independent_bernoulli", "independent bernoulli"),
                ("rounded_quota", "rounded quota"),
            ],
            Apply::Reset,
        ),
        Param::choice(
            "Readings",
            "distance_metric",
            "distance metric",
            &[
                ("euclidean", "euclidean"),
                ("manhattan", "manhattan"),
                ("territorial_path", "territorial path"),
            ],
            Apply::Reset,
        ),
        Param::choice(
            "Readings",
            "enemy_total",
            "enemy total",
            &[
                ("active_fronts", "active fronts"),
                ("all_fronts", "all fronts"),
            ],
            Apply::Reset,
        ),
        Param::choice(
            "Readings",
            "inactive_commitment",
            "inactive commitment",
            &[
                ("per_front_opponent", "per front opponent"),
                ("first_inactive_opponent", "first inactive opponent"),
            ],
            Apply::Reset,
        ),
        Param::choice(
            "Readings",
            "latent_regime",
            "latent regime",
            &[
                ("persistent_cell_tags", "persistent cell tags"),
                ("overwrite_on_conquest", "overwrite on conquest"),
            ],
            Apply::Reset,
        ),
        Param::choice(
            "Readings",
            "capital_capture",
            "capital capture",
            &[
                ("collapse_only", "collapse only"),
                ("capture_and_fragment", "capture and fragment"),
            ],
            Apply::Reset,
        ),
        Param::choice(
            "Readings",
            "claim_locking",
            "claim locking",
            &[
                ("affected_cells", "affected cells"),
                ("affected_states", "affected states"),
            ],
            Apply::Reset,
        ),
        Param::choice(
            "Readings",
            "opposing_victories",
            "opposing victories",
            &[
                ("independent_claims", "independent claims"),
                ("single_draw", "single draw"),
            ],
            Apply::Reset,
        ),
        Param::choice(
            "Readings",
            "alliance_maintenance",
            "alliance maintenance",
            &[
                ("rebuild_each_period", "rebuild each period"),
                ("persist_while_threatened", "persist while threatened"),
            ],
            Apply::Reset,
        ),
        Param::choice(
            "Readings",
            "threat_ties",
            "threat ties",
            &[
                ("lowest_state_id", "lowest state id"),
                ("random_tie", "random tie"),
            ],
            Apply::Reset,
        ),
        Param::choice(
            "Readings",
            "obligation_observation",
            "obligation observation",
            &[
                ("prior_actions", "prior actions"),
                ("current_plans_once", "current plans once"),
            ],
            Apply::Reset,
        ),
        Param::choice(
            "Readings",
            "security_scope",
            "security scope",
            &[
                ("all_democracies", "all democracies"),
                ("same_alliance", "same alliance"),
            ],
            Apply::Reset,
        ),
        Param::choice(
            "Readings",
            "clustering_exposure",
            "clustering exposure",
            &[
                ("unique_state_neighbors", "unique state neighbors"),
                ("border_edges", "border edges"),
            ],
            Apply::Reset,
        ),
        Param::choice(
            "Readings",
            "clustering_weights",
            "clustering weights",
            &[
                ("territory_weighted", "territory weighted"),
                ("equal_states", "equal states"),
            ],
            Apply::Reset,
        ),
    ]
}
