//! Fully resolved APSR defaults and named source/artifact interpretations.
use crate::{
    config::FieldError,
    schema::{Apply, Param},
};
use serde::{Deserialize, Serialize};
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Topology {
    Bounded,
    Torus,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FounderGrowth {
    OrderedRoundRobin,
    ShuffledRoundRobin,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InitialCapacity {
    ExtractedCapacity,
    #[serde(rename = "artifact_random_100_1")]
    ArtifactRandom1001,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DistanceMetric {
    Euclidean,
    Manhattan,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DistanceFormula {
    Decreasing,
    PrintedIncreasing,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EnemyTotal {
    ActiveFronts,
    AllFronts,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InitiationGuard {
    LiteralPrecedence,
    GlobalNoAction,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CampaignDropTiming {
    EachDecision,
    AfterBattle,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PathSampling {
    TargetFirst,
    AttackerFirst,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AttackProjection {
    RespectiveStates,
    InitiatorCurve,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DamageBasis {
    OpponentProjected,
    OwnCommitment,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DamageFeedback {
    SubtractLosses,
    AddLosses,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SeverityDamage {
    AllDamagedFronts,
    MutualOnly,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VictoryDraws {
    IndependentDefenderPriority,
    Exclusive,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CapitalCapture {
    CaptureAndFragment,
    CollapseOnly,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Locking {
    AffectedCells,
    AffectedStates,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TechnologyInheritance {
    ResetOnReemergence,
    RetainCellThreshold,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClusterLinkage {
    ConflictEdges,
    AdjacentActiveStates,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RetiredParticipants {
    RetainShadow,
    DropImmediately,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CountBoundary {
    AfterInitialization,
    AtInitialization,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SeverityExport {
    RawDamage,
    JavaInt100,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CompletedExport {
    AllCompleted,
    OnePerPeriod,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NumericalPolicy {
    RejectNonpositive,
    FloorZero,
}
/// The p.148 cost wording does not uniquely determine unilateral incidence.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DamageIncidence {
    AttackedParty,
    ActingParty,
}
/// APSR p.148 explicitly grants the defender the reciprocal threshold.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DefenderThreshold {
    Reciprocal,
    SameThreshold,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct GeosimConfig {
    pub width: u32,
    pub height: u32,
    pub initial_states: u32,
    pub initialization_periods: u64,
    pub observation_periods: u64,
    pub periods_per_tick: u32,
    pub resource_adjustment: f64,
    pub mobile_share: f64,
    pub campaign_drop_probability: f64,
    pub attack_probability: f64,
    pub deactivation_probability: f64,
    pub superiority_threshold: f64,
    pub victory_threshold: f64,
    pub defender_threshold: DefenderThreshold,
    pub superiority_exponent: u32,
    pub victory_exponent: u32,
    pub damage_fraction: f64,
    pub distance_offset: f64,
    pub distance_threshold: f64,
    pub distance_exponent: f64,
    pub shock_probability: f64,
    pub shock_shift: f64,
    pub war_shadow: u64,
    pub context_activation: bool,
    pub event_log: bool,
    pub event_log_limit: usize,
    pub topology: Topology,
    pub founder_growth: FounderGrowth,
    pub initial_capacity: InitialCapacity,
    pub distance_metric: DistanceMetric,
    pub distance_formula: DistanceFormula,
    pub enemy_total: EnemyTotal,
    pub initiation_guard: InitiationGuard,
    pub campaign_drop_timing: CampaignDropTiming,
    pub path_sampling: PathSampling,
    pub attack_projection: AttackProjection,
    pub damage_basis: DamageBasis,
    pub damage_incidence: DamageIncidence,
    pub damage_feedback: DamageFeedback,
    pub severity_damage: SeverityDamage,
    pub victory_draws: VictoryDraws,
    pub capital_capture: CapitalCapture,
    pub locking: Locking,
    pub technology_inheritance: TechnologyInheritance,
    pub cluster_linkage: ClusterLinkage,
    pub retired_participants: RetiredParticipants,
    pub count_boundary: CountBoundary,
    pub severity_export: SeverityExport,
    pub completed_export: CompletedExport,
    pub numerical_policy: NumericalPolicy,
}
impl Default for GeosimConfig {
    fn default() -> Self {
        Self {
            width: 50,
            height: 50,
            initial_states: 200,
            initialization_periods: 500,
            observation_periods: 10000,
            periods_per_tick: 1,
            resource_adjustment: 0.01,
            mobile_share: 0.5,
            campaign_drop_probability: 0.2,
            attack_probability: 0.01,
            deactivation_probability: 0.1,
            superiority_threshold: 3.0,
            victory_threshold: 3.0,
            defender_threshold: DefenderThreshold::Reciprocal,
            superiority_exponent: 20,
            victory_exponent: 20,
            damage_fraction: 0.1,
            distance_offset: 0.1,
            distance_threshold: 2.0,
            distance_exponent: 3.0,
            shock_probability: 0.0001,
            shock_shift: 20.0,
            war_shadow: 20,
            context_activation: true,
            event_log: false,
            event_log_limit: 1000,
            topology: Topology::Bounded,
            founder_growth: FounderGrowth::OrderedRoundRobin,
            initial_capacity: InitialCapacity::ExtractedCapacity,
            distance_metric: DistanceMetric::Euclidean,
            distance_formula: DistanceFormula::Decreasing,
            enemy_total: EnemyTotal::ActiveFronts,
            initiation_guard: InitiationGuard::LiteralPrecedence,
            campaign_drop_timing: CampaignDropTiming::EachDecision,
            path_sampling: PathSampling::TargetFirst,
            attack_projection: AttackProjection::RespectiveStates,
            damage_basis: DamageBasis::OpponentProjected,
            damage_incidence: DamageIncidence::AttackedParty,
            damage_feedback: DamageFeedback::SubtractLosses,
            severity_damage: SeverityDamage::AllDamagedFronts,
            victory_draws: VictoryDraws::IndependentDefenderPriority,
            capital_capture: CapitalCapture::CaptureAndFragment,
            locking: Locking::AffectedCells,
            technology_inheritance: TechnologyInheritance::ResetOnReemergence,
            cluster_linkage: ClusterLinkage::ConflictEdges,
            retired_participants: RetiredParticipants::RetainShadow,
            count_boundary: CountBoundary::AfterInitialization,
            severity_export: SeverityExport::RawDamage,
            completed_export: CompletedExport::AllCompleted,
            numerical_policy: NumericalPolicy::RejectNonpositive,
        }
    }
}
impl GeosimConfig {
    pub fn horizon(&self) -> u64 {
        self.initialization_periods
            .saturating_add(self.observation_periods)
    }
    pub fn artifact_2017() -> Self {
        Self {
            count_boundary: CountBoundary::AtInitialization,
            technology_inheritance: TechnologyInheritance::RetainCellThreshold,
            initiation_guard: InitiationGuard::GlobalNoAction,
            numerical_policy: NumericalPolicy::FloorZero,
            initial_capacity: InitialCapacity::ArtifactRandom1001,
            attack_projection: AttackProjection::InitiatorCurve,
            damage_feedback: DamageFeedback::AddLosses,
            severity_damage: SeverityDamage::MutualOnly,
            retired_participants: RetiredParticipants::DropImmediately,
            severity_export: SeverityExport::JavaInt100,
            completed_export: CompletedExport::OnePerPeriod,
            ..Self::default()
        }
    }
    pub fn validate(&self) -> Result<(), Vec<FieldError>> {
        let mut errors = Vec::new();
        if self.width < 2 || self.width > 100 {
            errors.push(FieldError::new("width", "outside supported finite range"));
        }
        if self.height < 2 || self.height > 100 {
            errors.push(FieldError::new("height", "outside supported finite range"));
        }
        if self.initial_states < 1 || self.initial_states > 10000 {
            errors.push(FieldError::new(
                "initial_states",
                "outside supported finite range",
            ));
        }
        if self.initialization_periods > 1000000 {
            errors.push(FieldError::new(
                "initialization_periods",
                "outside supported finite range",
            ));
        }
        if self.observation_periods < 1 || self.observation_periods > 1000000000 {
            errors.push(FieldError::new(
                "observation_periods",
                "outside supported finite range",
            ));
        }
        if self.periods_per_tick < 1 || self.periods_per_tick > 10000 {
            errors.push(FieldError::new(
                "periods_per_tick",
                "outside supported finite range",
            ));
        }
        if !self.resource_adjustment.is_finite()
            || self.resource_adjustment < 0.0
            || self.resource_adjustment > 1.0
        {
            errors.push(FieldError::new(
                "resource_adjustment",
                "outside supported finite range",
            ));
        }
        if !self.mobile_share.is_finite() || self.mobile_share < 0.0 || self.mobile_share > 1.0 {
            errors.push(FieldError::new(
                "mobile_share",
                "outside supported finite range",
            ));
        }
        if !self.campaign_drop_probability.is_finite()
            || self.campaign_drop_probability < 0.0
            || self.campaign_drop_probability > 1.0
        {
            errors.push(FieldError::new(
                "campaign_drop_probability",
                "outside supported finite range",
            ));
        }
        if !self.attack_probability.is_finite()
            || self.attack_probability < 0.0
            || self.attack_probability > 1.0
        {
            errors.push(FieldError::new(
                "attack_probability",
                "outside supported finite range",
            ));
        }
        if !self.deactivation_probability.is_finite()
            || self.deactivation_probability < 0.0
            || self.deactivation_probability > 1.0
        {
            errors.push(FieldError::new(
                "deactivation_probability",
                "outside supported finite range",
            ));
        }
        if !self.superiority_threshold.is_finite() || self.superiority_threshold <= 0.0 {
            errors.push(FieldError::new(
                "superiority_threshold",
                "outside supported finite range",
            ));
        }
        if !self.victory_threshold.is_finite() || self.victory_threshold <= 0.0 {
            errors.push(FieldError::new(
                "victory_threshold",
                "outside supported finite range",
            ));
        }
        if self.superiority_exponent < 1 || self.superiority_exponent > 100 {
            errors.push(FieldError::new(
                "superiority_exponent",
                "outside supported finite range",
            ));
        }
        if self.victory_exponent < 1 || self.victory_exponent > 100 {
            errors.push(FieldError::new(
                "victory_exponent",
                "outside supported finite range",
            ));
        }
        if !self.damage_fraction.is_finite()
            || self.damage_fraction < 0.0
            || self.damage_fraction > 1.0
        {
            errors.push(FieldError::new(
                "damage_fraction",
                "outside supported finite range",
            ));
        }
        if !self.distance_offset.is_finite()
            || self.distance_offset < 0.0
            || self.distance_offset > 1.0
        {
            errors.push(FieldError::new(
                "distance_offset",
                "outside supported finite range",
            ));
        }
        if !self.distance_threshold.is_finite() || self.distance_threshold <= 0.0 {
            errors.push(FieldError::new(
                "distance_threshold",
                "outside supported finite range",
            ));
        }
        if !self.distance_exponent.is_finite() || self.distance_exponent <= 0.0 {
            errors.push(FieldError::new(
                "distance_exponent",
                "outside supported finite range",
            ));
        }
        if !self.shock_probability.is_finite()
            || self.shock_probability < 0.0
            || self.shock_probability > 1.0
        {
            errors.push(FieldError::new(
                "shock_probability",
                "outside supported finite range",
            ));
        }
        if !self.shock_shift.is_finite() || self.shock_shift < 0.0 {
            errors.push(FieldError::new(
                "shock_shift",
                "outside supported finite range",
            ));
        }
        if self.war_shadow > 10000 {
            errors.push(FieldError::new(
                "war_shadow",
                "outside supported finite range",
            ));
        }
        if self.event_log_limit > 1000000 {
            errors.push(FieldError::new(
                "event_log_limit",
                "outside supported finite range",
            ));
        }
        if u64::from(self.width) * u64::from(self.height) > 10000 {
            errors.push(FieldError::new("width", "grid exceeds10000 cells"));
        }
        if u64::from(self.initial_states) > u64::from(self.width) * u64::from(self.height) {
            errors.push(FieldError::new(
                "initial_states",
                "must not exceed cell count",
            ));
        }
        if self
            .initialization_periods
            .checked_add(self.observation_periods)
            .is_none()
        {
            errors.push(FieldError::new("observation_periods", "horizon overflows"));
        }
        if self.event_log && self.event_log_limit == 0 {
            errors.push(FieldError::new(
                "event_log_limit",
                "must be positive when logging",
            ));
        }
        if self.defender_threshold == DefenderThreshold::Reciprocal
            && !(1.0 / self.victory_threshold).is_finite()
        {
            errors.push(FieldError::new(
                "victory_threshold",
                "reciprocal defender threshold must remain finite",
            ));
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}
pub fn schema() -> Vec<Param> {
    vec![
        Param::choice(
            "Readings",
            "damage_incidence",
            "Damage recipient",
            &[
                ("attacked_party", "Attacked party (selected reconstruction)"),
                ("acting_party", "Acting party (unmeasured p.148 reading)"),
            ],
            Apply::Reset,
        ),
        Param::choice(
            "Readings",
            "defender_threshold",
            "Defender threshold",
            &[
                ("reciprocal", "Source reciprocal threshold"),
                ("same_threshold", "Previous same-threshold reconstruction"),
            ],
            Apply::Reset,
        ),
        Param::integer("Model", "width", "Width", (2, 100), Apply::Reset),
        Param::integer("Model", "height", "Height", (2, 100), Apply::Reset),
        Param::integer(
            "Model",
            "initial_states",
            "Initial States",
            (1, 10000),
            Apply::Reset,
        ),
        Param::integer(
            "Model",
            "initialization_periods",
            "Initialization Periods",
            (0, 1000000),
            Apply::Reset,
        ),
        Param::integer(
            "Model",
            "observation_periods",
            "Observation Periods",
            (1, 1000000000),
            Apply::Reset,
        ),
        Param::integer(
            "Model",
            "periods_per_tick",
            "Periods Per Tick",
            (1, 10000),
            Apply::Reset,
        ),
        Param::number(
            "Model",
            "resource_adjustment",
            "Resource Adjustment",
            (0.0, 1.0, 0.01),
            Apply::Reset,
        ),
        Param::number(
            "Model",
            "mobile_share",
            "Mobile Share",
            (0.0, 1.0, 0.01),
            Apply::Reset,
        ),
        Param::number(
            "Model",
            "campaign_drop_probability",
            "Campaign Drop Probability",
            (0.0, 1.0, 0.01),
            Apply::Reset,
        ),
        Param::number(
            "Model",
            "attack_probability",
            "Attack Probability",
            (0.0, 1.0, 0.01),
            Apply::Reset,
        ),
        Param::number(
            "Model",
            "deactivation_probability",
            "Deactivation Probability",
            (0.0, 1.0, 0.01),
            Apply::Reset,
        ),
        Param::number(
            "Model",
            "superiority_threshold",
            "Superiority Threshold",
            (0.0, 1000000.0, 0.01),
            Apply::Reset,
        ),
        Param::number(
            "Model",
            "victory_threshold",
            "Victory Threshold",
            (0.0, 1000000.0, 0.01),
            Apply::Reset,
        ),
        Param::integer(
            "Model",
            "superiority_exponent",
            "Superiority Exponent",
            (1, 100),
            Apply::Reset,
        ),
        Param::integer(
            "Model",
            "victory_exponent",
            "Victory Exponent",
            (1, 100),
            Apply::Reset,
        ),
        Param::number(
            "Model",
            "damage_fraction",
            "Damage Fraction",
            (0.0, 1.0, 0.01),
            Apply::Reset,
        ),
        Param::number(
            "Model",
            "distance_offset",
            "Distance Offset",
            (0.0, 1.0, 0.01),
            Apply::Reset,
        ),
        Param::number(
            "Model",
            "distance_threshold",
            "Distance Threshold",
            (0.0, 1000000.0, 0.01),
            Apply::Reset,
        ),
        Param::number(
            "Model",
            "distance_exponent",
            "Distance Exponent",
            (0.0, 1000000.0, 0.01),
            Apply::Reset,
        ),
        Param::number(
            "Model",
            "shock_probability",
            "Shock Probability",
            (0.0, 1.0, 0.01),
            Apply::Reset,
        ),
        Param::number(
            "Model",
            "shock_shift",
            "Shock Shift",
            (0.0, 1000000.0, 0.01),
            Apply::Reset,
        ),
        Param::integer(
            "Model",
            "war_shadow",
            "War Shadow",
            (0, 10000),
            Apply::Reset,
        ),
        Param::bool(
            "Model",
            "context_activation",
            "Context Activation",
            Apply::Reset,
        ),
        Param::bool("Model", "event_log", "Event Log", Apply::Reset),
        Param::integer(
            "Model",
            "event_log_limit",
            "Event Log Limit",
            (0, 1000000),
            Apply::Reset,
        ),
        Param::choice(
            "Readings",
            "topology",
            "Topology",
            &[("bounded", "bounded"), ("torus", "torus")],
            Apply::Reset,
        ),
        Param::choice(
            "Readings",
            "founder_growth",
            "Founder Growth",
            &[
                ("ordered_round_robin", "ordered round robin"),
                ("shuffled_round_robin", "shuffled round robin"),
            ],
            Apply::Reset,
        ),
        Param::choice(
            "Readings",
            "initial_capacity",
            "Initial Capacity",
            &[
                ("extracted_capacity", "extracted capacity"),
                ("artifact_random_100_1", "artifact random 100 1"),
            ],
            Apply::Reset,
        ),
        Param::choice(
            "Readings",
            "distance_metric",
            "Distance Metric",
            &[("euclidean", "euclidean"), ("manhattan", "manhattan")],
            Apply::Reset,
        ),
        Param::choice(
            "Readings",
            "distance_formula",
            "Distance Formula",
            &[
                ("decreasing", "decreasing"),
                ("printed_increasing", "printed increasing"),
            ],
            Apply::Reset,
        ),
        Param::choice(
            "Readings",
            "enemy_total",
            "Enemy Total",
            &[
                ("active_fronts", "active fronts"),
                ("all_fronts", "all fronts"),
            ],
            Apply::Reset,
        ),
        Param::choice(
            "Readings",
            "initiation_guard",
            "Initiation Guard",
            &[
                ("literal_precedence", "literal precedence"),
                ("global_no_action", "global no action"),
            ],
            Apply::Reset,
        ),
        Param::choice(
            "Readings",
            "campaign_drop_timing",
            "Campaign Drop Timing",
            &[
                ("each_decision", "each decision"),
                ("after_battle", "after battle"),
            ],
            Apply::Reset,
        ),
        Param::choice(
            "Readings",
            "path_sampling",
            "Path Sampling",
            &[
                ("target_first", "target first"),
                ("attacker_first", "attacker first"),
            ],
            Apply::Reset,
        ),
        Param::choice(
            "Readings",
            "attack_projection",
            "Attack Projection",
            &[
                ("respective_states", "respective states"),
                ("initiator_curve", "initiator curve"),
            ],
            Apply::Reset,
        ),
        Param::choice(
            "Readings",
            "damage_basis",
            "Damage Basis",
            &[
                ("opponent_projected", "opponent projected"),
                ("own_commitment", "own commitment"),
            ],
            Apply::Reset,
        ),
        Param::choice(
            "Readings",
            "damage_feedback",
            "Damage Feedback",
            &[
                ("subtract_losses", "subtract losses"),
                ("add_losses", "add losses"),
            ],
            Apply::Reset,
        ),
        Param::choice(
            "Readings",
            "severity_damage",
            "Severity Damage",
            &[
                ("all_damaged_fronts", "all damaged fronts"),
                ("mutual_only", "mutual only"),
            ],
            Apply::Reset,
        ),
        Param::choice(
            "Readings",
            "victory_draws",
            "Victory Draws",
            &[
                (
                    "independent_defender_priority",
                    "independent defender priority",
                ),
                ("exclusive", "exclusive"),
            ],
            Apply::Reset,
        ),
        Param::choice(
            "Readings",
            "capital_capture",
            "Capital Capture",
            &[
                ("capture_and_fragment", "capture and fragment"),
                ("collapse_only", "collapse only"),
            ],
            Apply::Reset,
        ),
        Param::choice(
            "Readings",
            "locking",
            "Locking",
            &[
                ("affected_cells", "affected cells"),
                ("affected_states", "affected states"),
            ],
            Apply::Reset,
        ),
        Param::choice(
            "Readings",
            "technology_inheritance",
            "Technology Inheritance",
            &[
                ("reset_on_reemergence", "reset on reemergence"),
                ("retain_cell_threshold", "retain cell threshold"),
            ],
            Apply::Reset,
        ),
        Param::choice(
            "Readings",
            "cluster_linkage",
            "Cluster Linkage",
            &[
                ("conflict_edges", "conflict edges"),
                ("adjacent_active_states", "adjacent active states"),
            ],
            Apply::Reset,
        ),
        Param::choice(
            "Readings",
            "retired_participants",
            "Retired Participants",
            &[
                ("retain_shadow", "retain shadow"),
                ("drop_immediately", "drop immediately"),
            ],
            Apply::Reset,
        ),
        Param::choice(
            "Readings",
            "count_boundary",
            "Count Boundary",
            &[
                ("after_initialization", "after initialization"),
                ("at_initialization", "at initialization"),
            ],
            Apply::Reset,
        ),
        Param::choice(
            "Readings",
            "severity_export",
            "Severity Export",
            &[("raw_damage", "raw damage"), ("java_int100", "java int100")],
            Apply::Reset,
        ),
        Param::choice(
            "Readings",
            "completed_export",
            "Completed Export",
            &[
                ("all_completed", "all completed"),
                ("one_per_period", "one per period"),
            ],
            Apply::Reset,
        ),
        Param::choice(
            "Readings",
            "numerical_policy",
            "Numerical Policy",
            &[
                ("reject_nonpositive", "reject nonpositive"),
                ("floor_zero", "floor zero"),
            ],
            Apply::Reset,
        ),
    ]
}
