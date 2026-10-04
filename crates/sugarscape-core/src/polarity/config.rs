//! Source mechanics and explicit, serialized reconstruction readings.
use crate::{
    config::FieldError,
    schema::{Apply, Param},
};
use serde::{Deserialize, Serialize};
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Variant {
    Epm,
    TwoLevel,
    Overextension,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceProfile {
    Chapter4,
    Chapter5,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Topology {
    Bounded,
    Torus,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Placement {
    ExactCount,
    Bernoulli,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResourceDistribution {
    Normal,
    BoundedUniform,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AsymmetricDamage {
    Source,
    On,
    Off,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Allocation {
    Equal,
    Pra,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActionMemory {
    PreviousAction,
    WarUntilVictory,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SchlieffenGate {
    OwnCurrentDefections,
    PreviousHostilities,
    UnresolvedWar,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CombatPath {
    StoredEpisode,
    RedrawEachPeriod,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TieBreak {
    LowestId,
    Random,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PathCollision {
    LowestId,
    Random,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PraActive {
    EitherDefection,
    MutualDefection,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VictoryTiming {
    BeforeDamage,
    AfterDamage,
    AfterHarvest,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Update {
    Snapshot,
    Sequential,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Locking {
    AffectedCells,
    AffectedStates,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CapitalCapture {
    CollapseOnly,
    CaptureAndFragment,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProvinceTransfer {
    EqualShare,
    PrimitiveStockOnly,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResourcePolicy {
    Signed,
    FloorZero,
    RejectNonpositive,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ThreatObservation {
    NeighborAggression,
    DyadicAggression,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ObligationTiming {
    SamePeriod,
    NextPeriod,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PraAttackRule {
    Diagram,
    LiteralProse,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaxDistance {
    Manhattan,
    TerritorialPath,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StochasticResolution {
    SingleDraw,
    IndependentDraws,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PraAllianceSupport {
    FrontCommitments,
    Stocks,
}
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct PolarityConfig {
    pub variant: Variant,
    pub source_profile: SourceProfile,
    pub width: u32,
    pub height: u32,
    pub topology: Topology,
    pub predator_share: f64,
    pub placement: Placement,
    pub initial_mean: f64,
    pub initial_sd: f64,
    pub harvest_mean: f64,
    pub harvest_sd: f64,
    pub resource_distribution: ResourceDistribution,
    pub superiority: f64,
    pub victory: f64,
    pub damage_rate: f64,
    pub asymmetric_damage: AsymmetricDamage,
    pub allocation: Allocation,
    pub action_memory: ActionMemory,
    pub schlieffen_gate: SchlieffenGate,
    pub combat_path: CombatPath,
    pub tie_break: TieBreak,
    pub path_collision: PathCollision,
    pub pra_active: PraActive,
    pub victory_timing: VictoryTiming,
    pub update: Update,
    pub locking: Locking,
    pub capital_capture: CapitalCapture,
    pub province_transfer: ProvinceTransfer,
    pub resource_policy: ResourcePolicy,
    pub alliances: bool,
    pub pra_alliance_support: PraAllianceSupport,
    pub trust_initial: f64,
    pub threat_threshold: f64,
    pub negative_trust_rate: f64,
    pub positive_trust_rate: f64,
    pub threat_observation: ThreatObservation,
    pub obligation_timing: ObligationTiming,
    pub pra_attack_rule: PraAttackRule,
    pub tax_rate: f64,
    pub tax_discount: f64,
    pub tax_distance: TaxDistance,
    pub stochastic_threshold: f64,
    pub stochastic_exponent: f64,
    pub stochastic_resolution: StochasticResolution,
    pub horizon: u64,
    pub stop_at_hegemony: bool,
    pub periods_per_tick: u32,
    pub event_log: bool,
    pub event_log_limit: u32,
}
impl Default for PolarityConfig {
    fn default() -> Self {
        Self {
            variant: Variant::Epm,
            source_profile: SourceProfile::Chapter4,
            width: 10,
            height: 10,
            topology: Topology::Bounded,
            predator_share: 0.2,
            placement: Placement::ExactCount,
            initial_mean: 50.0,
            initial_sd: 10.0,
            harvest_mean: 2.0,
            harvest_sd: 5.0,
            resource_distribution: ResourceDistribution::Normal,
            superiority: 2.0,
            victory: 2.0,
            damage_rate: 0.05,
            asymmetric_damage: AsymmetricDamage::Source,
            allocation: Allocation::Equal,
            action_memory: ActionMemory::PreviousAction,
            schlieffen_gate: SchlieffenGate::OwnCurrentDefections,
            combat_path: CombatPath::StoredEpisode,
            tie_break: TieBreak::LowestId,
            path_collision: PathCollision::LowestId,
            pra_active: PraActive::EitherDefection,
            victory_timing: VictoryTiming::BeforeDamage,
            update: Update::Snapshot,
            locking: Locking::AffectedCells,
            capital_capture: CapitalCapture::CollapseOnly,
            province_transfer: ProvinceTransfer::EqualShare,
            resource_policy: ResourcePolicy::Signed,
            alliances: false,
            pra_alliance_support: PraAllianceSupport::FrontCommitments,
            trust_initial: 0.0,
            threat_threshold: 0.0,
            negative_trust_rate: 0.5,
            positive_trust_rate: 0.01,
            threat_observation: ThreatObservation::NeighborAggression,
            obligation_timing: ObligationTiming::SamePeriod,
            pra_attack_rule: PraAttackRule::Diagram,
            tax_rate: 0.4,
            tax_discount: 0.7,
            tax_distance: TaxDistance::Manhattan,
            stochastic_threshold: 3.0,
            stochastic_exponent: 5.0,
            stochastic_resolution: StochasticResolution::SingleDraw,
            horizon: 1000,
            stop_at_hegemony: true,
            periods_per_tick: 1,
            event_log: false,
            event_log_limit: 1000,
        }
    }
}
impl PolarityConfig {
    pub fn for_variant(variant: Variant) -> Self {
        let mut c = Self {
            variant,
            ..Self::default()
        };
        if variant != Variant::Epm {
            c.source_profile = SourceProfile::Chapter5;
            c.allocation = Allocation::Pra;
        }
        if variant == Variant::TwoLevel {
            c.tax_discount = 1.0;
        }
        if variant == Variant::Overextension {
            c.horizon = 4000;
            c.stop_at_hegemony = false;
        }
        c
    }
    pub fn provincial(&self) -> bool {
        self.variant != Variant::Epm
    }
    pub fn asymmetric_losses(&self) -> bool {
        match self.asymmetric_damage {
            AsymmetricDamage::On => true,
            AsymmetricDamage::Off => false,
            AsymmetricDamage::Source => self.source_profile == SourceProfile::Chapter4,
        }
    }
    pub fn validate(&self) -> Result<(), Vec<FieldError>> {
        let mut errors = Vec::new();
        let mut check = |ok: bool, field: &str, message: &str| {
            if !ok {
                errors.push(FieldError::new(field, message));
            }
        };
        check((2..=100).contains(&self.width), "width", "must be 2..100");
        check((2..=100).contains(&self.height), "height", "must be 2..100");
        check(
            u64::from(self.width) * u64::from(self.height) <= 10000,
            "width",
            "at most 10,000 cells",
        );
        check(
            (0.0..=1.0).contains(&self.predator_share),
            "predator_share",
            "must be finite and 0..1",
        );
        check(
            (0.0..=1.0).contains(&self.damage_rate),
            "damage_rate",
            "must be finite and 0..1",
        );
        check(
            (0.0..=1.0).contains(&self.negative_trust_rate),
            "negative_trust_rate",
            "must be finite and 0..1",
        );
        check(
            (0.0..=1.0).contains(&self.positive_trust_rate),
            "positive_trust_rate",
            "must be finite and 0..1",
        );
        check(
            (0.0..=1.0).contains(&self.tax_rate),
            "tax_rate",
            "must be finite and 0..1",
        );
        check(
            (0.0..=1.0).contains(&self.tax_discount),
            "tax_discount",
            "must be finite and 0..1",
        );
        check(
            self.initial_mean.is_finite(),
            "initial_mean",
            "must be finite",
        );
        check(
            self.harvest_mean.is_finite(),
            "harvest_mean",
            "must be finite",
        );
        check(
            self.initial_sd.is_finite() && self.initial_sd >= 0.0,
            "initial_sd",
            "must be finite and nonnegative",
        );
        check(
            self.harvest_sd.is_finite() && self.harvest_sd >= 0.0,
            "harvest_sd",
            "must be finite and nonnegative",
        );
        check(
            self.superiority.is_finite() && self.superiority > 1.0,
            "superiority",
            "must be finite and greater than 1",
        );
        check(
            self.victory.is_finite() && self.victory > 1.0,
            "victory",
            "must be finite and greater than 1",
        );
        check(
            self.stochastic_threshold.is_finite() && self.stochastic_threshold > 1.0,
            "stochastic_threshold",
            "must be finite and greater than 1",
        );
        check(
            (-1000.0..=1000.0).contains(&self.trust_initial),
            "trust_initial",
            "must be finite and -1000..1000",
        );
        check(
            (-1000.0..=1000.0).contains(&self.threat_threshold),
            "threat_threshold",
            "must be finite and -1000..1000",
        );
        check(
            self.stochastic_exponent.is_finite() && self.stochastic_exponent > 0.0,
            "stochastic_exponent",
            "must be finite and positive",
        );
        check(
            (1..=1_000_000_000).contains(&self.horizon),
            "horizon",
            "must be 1..1,000,000,000",
        );
        check(
            (1..=10000).contains(&self.periods_per_tick),
            "periods_per_tick",
            "must be 1..10,000",
        );
        check(
            self.event_log_limit <= 1_000_000 && (!self.event_log || self.event_log_limit > 0),
            "event_log_limit",
            "must be 0..1,000,000 and positive when logging",
        );
        check(
            !self.provincial() || self.source_profile == SourceProfile::Chapter5,
            "source_profile",
            "provincial variants require chapter5",
        );
        check(
            !self.provincial() || self.allocation == Allocation::Pra,
            "allocation",
            "provincial variants require pra",
        );
        check(
            self.variant != Variant::TwoLevel || self.tax_discount == 1.0,
            "tax_discount",
            "two_level uses constant tax: choose tax_discount=1 or overextension",
        );
        check(
            self.variant != Variant::Overextension || !self.stop_at_hegemony,
            "stop_at_hegemony",
            "overextension must continue through hegemony",
        );
        check(
            !(self.update == Update::Sequential
                && self.victory_timing == VictoryTiming::AfterHarvest),
            "victory_timing",
            "sequential requires before_damage or after_damage; choose snapshot for after_harvest",
        );
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}
impl<'de> Deserialize<'de> for PolarityConfig {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Partial {
            #[serde(default)]
            variant: Option<Variant>,
            #[serde(default)]
            source_profile: Option<SourceProfile>,
            #[serde(default)]
            width: Option<u32>,
            #[serde(default)]
            height: Option<u32>,
            #[serde(default)]
            topology: Option<Topology>,
            #[serde(default)]
            predator_share: Option<f64>,
            #[serde(default)]
            placement: Option<Placement>,
            #[serde(default)]
            initial_mean: Option<f64>,
            #[serde(default)]
            initial_sd: Option<f64>,
            #[serde(default)]
            harvest_mean: Option<f64>,
            #[serde(default)]
            harvest_sd: Option<f64>,
            #[serde(default)]
            resource_distribution: Option<ResourceDistribution>,
            #[serde(default)]
            superiority: Option<f64>,
            #[serde(default)]
            victory: Option<f64>,
            #[serde(default)]
            damage_rate: Option<f64>,
            #[serde(default)]
            asymmetric_damage: Option<AsymmetricDamage>,
            #[serde(default)]
            allocation: Option<Allocation>,
            #[serde(default)]
            action_memory: Option<ActionMemory>,
            #[serde(default)]
            schlieffen_gate: Option<SchlieffenGate>,
            #[serde(default)]
            combat_path: Option<CombatPath>,
            #[serde(default)]
            tie_break: Option<TieBreak>,
            #[serde(default)]
            path_collision: Option<PathCollision>,
            #[serde(default)]
            pra_active: Option<PraActive>,
            #[serde(default)]
            victory_timing: Option<VictoryTiming>,
            #[serde(default)]
            update: Option<Update>,
            #[serde(default)]
            locking: Option<Locking>,
            #[serde(default)]
            capital_capture: Option<CapitalCapture>,
            #[serde(default)]
            province_transfer: Option<ProvinceTransfer>,
            #[serde(default)]
            resource_policy: Option<ResourcePolicy>,
            #[serde(default)]
            alliances: Option<bool>,
            #[serde(default)]
            pra_alliance_support: Option<PraAllianceSupport>,
            #[serde(default)]
            trust_initial: Option<f64>,
            #[serde(default)]
            threat_threshold: Option<f64>,
            #[serde(default)]
            negative_trust_rate: Option<f64>,
            #[serde(default)]
            positive_trust_rate: Option<f64>,
            #[serde(default)]
            threat_observation: Option<ThreatObservation>,
            #[serde(default)]
            obligation_timing: Option<ObligationTiming>,
            #[serde(default)]
            pra_attack_rule: Option<PraAttackRule>,
            #[serde(default)]
            tax_rate: Option<f64>,
            #[serde(default)]
            tax_discount: Option<f64>,
            #[serde(default)]
            tax_distance: Option<TaxDistance>,
            #[serde(default)]
            stochastic_threshold: Option<f64>,
            #[serde(default)]
            stochastic_exponent: Option<f64>,
            #[serde(default)]
            stochastic_resolution: Option<StochasticResolution>,
            #[serde(default)]
            horizon: Option<u64>,
            #[serde(default)]
            stop_at_hegemony: Option<bool>,
            #[serde(default)]
            periods_per_tick: Option<u32>,
            #[serde(default)]
            event_log: Option<bool>,
            #[serde(default)]
            event_log_limit: Option<u32>,
        }
        let p = Partial::deserialize(d)?;
        let mut c = Self::for_variant(p.variant.unwrap_or(Variant::Epm));
        if let Some(v) = p.variant {
            c.variant = v;
        }
        if let Some(v) = p.source_profile {
            c.source_profile = v;
        }
        if let Some(v) = p.width {
            c.width = v;
        }
        if let Some(v) = p.height {
            c.height = v;
        }
        if let Some(v) = p.topology {
            c.topology = v;
        }
        if let Some(v) = p.predator_share {
            c.predator_share = v;
        }
        if let Some(v) = p.placement {
            c.placement = v;
        }
        if let Some(v) = p.initial_mean {
            c.initial_mean = v;
        }
        if let Some(v) = p.initial_sd {
            c.initial_sd = v;
        }
        if let Some(v) = p.harvest_mean {
            c.harvest_mean = v;
        }
        if let Some(v) = p.harvest_sd {
            c.harvest_sd = v;
        }
        if let Some(v) = p.resource_distribution {
            c.resource_distribution = v;
        }
        if let Some(v) = p.superiority {
            c.superiority = v;
        }
        if let Some(v) = p.victory {
            c.victory = v;
        }
        if let Some(v) = p.damage_rate {
            c.damage_rate = v;
        }
        if let Some(v) = p.asymmetric_damage {
            c.asymmetric_damage = v;
        }
        if let Some(v) = p.allocation {
            c.allocation = v;
        }
        if let Some(v) = p.action_memory {
            c.action_memory = v;
        }
        if let Some(v) = p.schlieffen_gate {
            c.schlieffen_gate = v;
        }
        if let Some(v) = p.combat_path {
            c.combat_path = v;
        }
        if let Some(v) = p.tie_break {
            c.tie_break = v;
        }
        if let Some(v) = p.path_collision {
            c.path_collision = v;
        }
        if let Some(v) = p.pra_active {
            c.pra_active = v;
        }
        if let Some(v) = p.victory_timing {
            c.victory_timing = v;
        }
        if let Some(v) = p.update {
            c.update = v;
        }
        if let Some(v) = p.locking {
            c.locking = v;
        }
        if let Some(v) = p.capital_capture {
            c.capital_capture = v;
        }
        if let Some(v) = p.province_transfer {
            c.province_transfer = v;
        }
        if let Some(v) = p.resource_policy {
            c.resource_policy = v;
        }
        if let Some(v) = p.alliances {
            c.alliances = v;
        }
        if let Some(v) = p.pra_alliance_support {
            c.pra_alliance_support = v;
        }
        if let Some(v) = p.trust_initial {
            c.trust_initial = v;
        }
        if let Some(v) = p.threat_threshold {
            c.threat_threshold = v;
        }
        if let Some(v) = p.negative_trust_rate {
            c.negative_trust_rate = v;
        }
        if let Some(v) = p.positive_trust_rate {
            c.positive_trust_rate = v;
        }
        if let Some(v) = p.threat_observation {
            c.threat_observation = v;
        }
        if let Some(v) = p.obligation_timing {
            c.obligation_timing = v;
        }
        if let Some(v) = p.pra_attack_rule {
            c.pra_attack_rule = v;
        }
        if let Some(v) = p.tax_rate {
            c.tax_rate = v;
        }
        if let Some(v) = p.tax_discount {
            c.tax_discount = v;
        }
        if let Some(v) = p.tax_distance {
            c.tax_distance = v;
        }
        if let Some(v) = p.stochastic_threshold {
            c.stochastic_threshold = v;
        }
        if let Some(v) = p.stochastic_exponent {
            c.stochastic_exponent = v;
        }
        if let Some(v) = p.stochastic_resolution {
            c.stochastic_resolution = v;
        }
        if let Some(v) = p.horizon {
            c.horizon = v;
        }
        if let Some(v) = p.stop_at_hegemony {
            c.stop_at_hegemony = v;
        }
        if let Some(v) = p.periods_per_tick {
            c.periods_per_tick = v;
        }
        if let Some(v) = p.event_log {
            c.event_log = v;
        }
        if let Some(v) = p.event_log_limit {
            c.event_log_limit = v;
        }
        Ok(c)
    }
}
pub fn schema() -> Vec<Param> {
    let mut p=vec![
Param::choice("Readings","pra_alliance_support","PRA alliance deterrence",&[("front_commitments","Coalition commitments"),("stocks","Coalition stocks")],Apply::Reset).with_help("Unreported alliance/PRA interaction (Cederman 1997 p.121 fn5); named reconstruction, deterrence only."),
Param::choice("Readings","variant","Variant",&[("epm","EPM"),("two_level","Two level"),("overextension","Overextension")],Apply::Reset).with_help("Cederman 1994 pp.507–517 / 1997 ch.5; explicit reconstruction reading, see approved design."),
Param::choice("Readings","source_profile","Source profile",&[("chapter4","Chapter 4"),("chapter5","Chapter 5")],Apply::Reset).with_help("Cederman 1994 pp.507–517 / 1997 ch.5; explicit reconstruction reading, see approved design."),
Param::integer(
        "Clock and grid",
        "width",
        "Width",
        (2, 100),
        Apply::Reset,
    ),
Param::integer(
        "Clock and grid",
        "height",
        "Height",
        (2, 100),
        Apply::Reset,
    ),
Param::choice("Readings","topology","Topology",&[("bounded","Bounded"),("torus","Torus")],Apply::Reset).with_help("Cederman 1994 pp.507–517 / 1997 ch.5; explicit reconstruction reading, see approved design."),
Param::number(
        "Resources and decisions",
        "predator_share",
        "Predator share",
        (0.0, 1.0, 0.01),
        Apply::Reset,
    ),
Param::choice("Readings","placement","Placement",&[("exact_count","Exact count"),("bernoulli","Bernoulli")],Apply::Reset).with_help("Cederman 1994 pp.507–517 / 1997 ch.5; explicit reconstruction reading, see approved design."),
Param::number(
        "Resources and decisions",
        "initial_mean",
        "Initial mean",
        (-1000.0, 1000.0, 1.0),
        Apply::Reset,
    ),
Param::number(
        "Resources and decisions",
        "initial_sd",
        "Initial sd",
        (0.0, 1000.0, 0.1),
        Apply::Reset,
    ),
Param::number(
        "Resources and decisions",
        "harvest_mean",
        "Harvest mean",
        (-1000.0, 1000.0, 1.0),
        Apply::Reset,
    ),
Param::number(
        "Resources and decisions",
        "harvest_sd",
        "Harvest sd",
        (0.0, 1000.0, 0.1),
        Apply::Reset,
    ),
Param::choice("Readings","resource_distribution","Resource distribution",&[("normal","Normal"),("bounded_uniform","Bounded uniform")],Apply::Reset).with_help("Cederman 1994 pp.507–517 / 1997 ch.5; explicit reconstruction reading, see approved design."),
Param::number(
        "Resources and decisions",
        "superiority",
        "Superiority",
        (1.01, 20.0, 0.01),
        Apply::Reset,
    ),
Param::number(
        "Resources and decisions",
        "victory",
        "Victory",
        (1.01, 20.0, 0.01),
        Apply::Reset,
    ),
Param::number(
        "Resources and decisions",
        "damage_rate",
        "Damage rate",
        (0.0, 1.0, 0.01),
        Apply::Reset,
    ),
Param::choice("Readings","asymmetric_damage","Asymmetric damage",&[("source","Source"),("on","On"),("off","Off")],Apply::Reset).with_help("Cederman 1994 pp.507–517 / 1997 ch.5; explicit reconstruction reading, see approved design."),
Param::choice("Readings","allocation","Allocation",&[("equal","Equal"),("pra","PRA")],Apply::Reset).with_help("Cederman 1994 pp.507–517 / 1997 ch.5; explicit reconstruction reading, see approved design."),
Param::choice("Readings","action_memory","Action memory",&[("previous_action","Previous action"),("war_until_victory","War until victory")],Apply::Reset).with_help("Cederman 1994 pp.507–517 / 1997 ch.5; explicit reconstruction reading, see approved design."),
Param::choice("Readings","schlieffen_gate","Schlieffen gate",&[("own_current_defections","Own current defections"),("previous_hostilities","Previous hostilities"),("unresolved_war","Unresolved war")],Apply::Reset).with_help("Cederman 1994 pp.507–517 / 1997 ch.5; explicit reconstruction reading, see approved design."),
Param::choice("Readings","combat_path","Combat path",&[("stored_episode","Stored episode"),("redraw_each_period","Redraw each period")],Apply::Reset).with_help("Cederman 1994 pp.507–517 / 1997 ch.5; explicit reconstruction reading, see approved design."),
Param::choice("Readings","tie_break","Tie break",&[("lowest_id","Lowest id"),("random","Random")],Apply::Reset).with_help("Cederman 1994 pp.507–517 / 1997 ch.5; explicit reconstruction reading, see approved design."),
Param::choice("Readings","path_collision","Path collision",&[("lowest_id","Lowest id"),("random","Random")],Apply::Reset).with_help("Cederman 1994 pp.507–517 / 1997 ch.5; explicit reconstruction reading, see approved design."),
Param::choice("Readings","pra_active","Pra active",&[("either_defection","Either defection"),("mutual_defection","Mutual defection")],Apply::Reset).with_help("Cederman 1994 pp.507–517 / 1997 ch.5; explicit reconstruction reading, see approved design."),
Param::choice("Readings","victory_timing","Victory timing",&[("before_damage","Before damage"),("after_damage","After damage"),("after_harvest","After harvest")],Apply::Reset).with_help("Cederman 1994 pp.507–517 / 1997 ch.5; explicit reconstruction reading, see approved design."),
Param::choice("Readings","update","Update",&[("snapshot","Snapshot"),("sequential","Sequential")],Apply::Reset).with_help("Cederman 1994 pp.507–517 / 1997 ch.5; explicit reconstruction reading, see approved design."),
Param::choice("Readings","locking","Locking",&[("affected_cells","Affected cells"),("affected_states","Affected states")],Apply::Reset).with_help("Cederman 1994 pp.507–517 / 1997 ch.5; explicit reconstruction reading, see approved design."),
Param::choice("Readings","capital_capture","Capital capture",&[("collapse_only","Collapse only"),("capture_and_fragment","Capture and fragment")],Apply::Reset).with_help("Cederman 1994 pp.507–517 / 1997 ch.5; explicit reconstruction reading, see approved design."),
Param::choice("Readings","province_transfer","Province transfer",&[("equal_share","Equal share"),("primitive_stock_only","Primitive stock only")],Apply::Reset).with_help("Cederman 1994 pp.507–517 / 1997 ch.5; explicit reconstruction reading, see approved design."),
Param::choice("Readings","resource_policy","Resource policy",&[("signed","Signed"),("floor_zero","Floor zero"),("reject_nonpositive","Reject nonpositive")],Apply::Reset).with_help("Cederman 1994 pp.507–517 / 1997 ch.5; explicit reconstruction reading, see approved design."),
Param::bool("Model", "alliances", "Alliances", Apply::Reset),
Param::number(
        "Resources and decisions",
        "trust_initial",
        "Trust initial",
        (-1000.0, 1000.0, 1.0),
        Apply::Reset,
    ),
Param::number(
        "Resources and decisions",
        "threat_threshold",
        "Threat threshold",
        (-1000.0, 1000.0, 1.0),
        Apply::Reset,
    ),
Param::number(
        "Resources and decisions",
        "negative_trust_rate",
        "Negative trust rate",
        (0.0, 1.0, 0.01),
        Apply::Reset,
    ),
Param::number(
        "Resources and decisions",
        "positive_trust_rate",
        "Positive trust rate",
        (0.0, 1.0, 0.01),
        Apply::Reset,
    ),
Param::choice("Readings","threat_observation","Threat observation",&[("neighbor_aggression","Neighbor aggression"),("dyadic_aggression","Dyadic aggression")],Apply::Reset).with_help("Cederman 1994 pp.507–517 / 1997 ch.5; explicit reconstruction reading, see approved design."),
Param::choice("Readings","obligation_timing","Obligation timing",&[("same_period","Same period"),("next_period","Next period")],Apply::Reset).with_help("Cederman 1994 pp.507–517 / 1997 ch.5; explicit reconstruction reading, see approved design."),
Param::choice("Readings","pra_attack_rule","Pra attack rule",&[("diagram","Diagram"),("literal_prose","Literal prose")],Apply::Reset).with_help("Cederman 1994 pp.507–517 / 1997 ch.5; explicit reconstruction reading, see approved design."),
Param::number(
        "Resources and decisions",
        "tax_rate",
        "Tax rate",
        (0.0, 1.0, 0.01),
        Apply::Reset,
    ),
Param::number(
        "Resources and decisions",
        "tax_discount",
        "Tax discount",
        (0.0, 1.0, 0.01),
        Apply::Reset,
    ),
Param::choice("Readings","tax_distance","Tax distance",&[("manhattan","Manhattan"),("territorial_path","Territorial path")],Apply::Reset).with_help("Cederman 1994 pp.507–517 / 1997 ch.5; explicit reconstruction reading, see approved design."),
Param::number(
        "Resources and decisions",
        "stochastic_threshold",
        "Stochastic threshold",
        (1.01, 20.0, 0.01),
        Apply::Reset,
    ),
Param::number(
        "Resources and decisions",
        "stochastic_exponent",
        "Stochastic exponent",
        (0.01, 100.0, 0.01),
        Apply::Reset,
    ),
Param::choice("Readings","stochastic_resolution","Stochastic resolution",&[("single_draw","Single draw"),("independent_draws","Independent draws")],Apply::Reset).with_help("Cederman 1994 pp.507–517 / 1997 ch.5; explicit reconstruction reading, see approved design."),
Param::integer(
        "Clock and grid",
        "horizon",
        "Horizon",
        (1, 1000000000),
        Apply::Reset,
    ),
Param::bool(
        "Model",
        "stop_at_hegemony",
        "Stop at hegemony",
        Apply::Reset,
    ),
Param::integer(
        "Clock and grid",
        "periods_per_tick",
        "Periods per tick",
        (1, 10000),
        Apply::Reset,
    ),
Param::bool("Model", "event_log", "Event log", Apply::Reset),
Param::integer(
        "Clock and grid",
        "event_log_limit",
        "Event log limit",
        (0, 1000000),
        Apply::Reset,
    )
];
    for param in &mut p {
        let help=match param.path {
            "variant"=>Some("Cederman 1994 base EPM; 1997 ch.5 adds province stocks/revolts and stochastic overextension. Extended variants require chapter5 and PRA."),
            "source_profile"=>Some("1994 Fig.8 damages CD/DC; 1997 p.109 fn1 suppresses asymmetric losses. Source selects that distinction; explicit on/off overrides remain visible."),
            "placement"=>Some("1994 p.517 supplies predator percentages; exact rounded count shuffled over cells is reconstruction. Bernoulli is an explicit alternative."),
            "resource_distribution"=>Some("1994 pp.511,514–515 normal draws remain untruncated. Variance-matched bounded uniform is our attributed sensitivity."),
            "action_memory"=>Some("1994 Fig.7 previous-opponent TFT is default. War until victory is a persistence reading from later GeoSim/Radax, not recovered Pascal behavior."),
            "schlieffen_gate"=>Some("1994 Fig.7 no-other-war guard is underspecified. Default checks current own defections; alternatives check any previous hostility or an unresolved episode."),
            "combat_path"=>Some("1994 pp.512–514 sample attacker border cell then defender cell. Stored episode path is reconstruction; redraw each period tests path dependence."),
            "allocation"=>Some("1994 equal division uses distinct sovereign neighbors. 1997 pp.117–120 PRA uses opposing old commitments, active/passive equations and zero fallbacks."),
            "pra_active"=>Some("1997 PRA active-front timing is reconstructed as either side previously D. Mutual D is an explicit alternative."),
            "pra_attack_rule"=>Some("1997 diagram maximizes ratio and uses strict greater-than. Literal prose minimizes and uses less-than, retaining the published inconsistency."),
            "victory_timing"=>Some("1994 Fig.8 does not establish loss/harvest timing. Frozen allocations before damage are default; alternatives recompute stocks with frozen graph/PRA weights."),
            "locking"=>Some("1994 p.511 fn7 locking scope is unreported. Default locks affected cells and changed-stock capitals; alternative locks both pre-event states."),
            "capital_capture"=>Some("1994 p.515 fn19 collapse-then-subsequent-capture supports collapse only. Capture and fragment annexes the capital, releasing provinces."),
            "province_transfer"=>Some("EPM captured-stock bookkeeping is unreported. Equal share moves pre-event per-cell corporate stock; primitive-stock-only moves no share for dependent provinces."),
            "resource_policy"=>Some("Sources specify no negative-stock policy. Signed preserves negative draws and damage creation; floor clips with a ledger; reject retains invalid sessions."),
            "update"=>Some("Snapshot follows decisions/interaction/structure phases. Sequential shuffles actors and resolves each dyad on first visit; after-harvest timing requires snapshot."),
            "topology"=>Some("1994 at most four neighbors supports bounded cardinal grid. Toroidal wrapping is a named sensitivity; both deduplicate neighbors."),
            "tie_break"=>Some("Equal strategic target and prime-threat ties are unreported. Lowest stable cell ID is default; random is a seeded alternative."),
            "path_collision"=>Some("Opposed initiations share one dyad. Default chooses lower-ID initiator path; random chooses between both sampled paths. This arbitration is reconstruction."),
            "alliances"=>Some("1994 pp.515–517 directed behavioral trust groups at least two states sharing a prime threat; never pools stocks or changes sovereignty."),
            "obligation_timing"=>Some("Coalition response timing is unreported. Same period expands voluntary attacks once, without recursive obligations; next period defers the response."),
            "threat_observation"=>Some("Aggression perception is reconstruction. Current neighbors observe prior initiation/conquest; dyadic observes only aggression against the observer."),
            "tax_distance"=>Some("1997 pp.126–134 distance decay uses tax_rate times tax_discount raised to distance. Manhattan is default; territorial path remains inside the state."),
            "stochastic_resolution"=>Some("1997 competing victories are unreported. Single draw uses adjacent success intervals; independent draws count double success as stalemate."),
            "stochastic_threshold"=>Some("Threshold 3 is an inferred Fig.5.9 overextension run setting, not a recovered narrative-run input."),
            "event_log"=>Some("Our bounded opt-in diagnostic event log; episode/counter summaries remain complete independently of retained log records."),
            _=>param.help,
        };
        param.help = help;
    }
    p
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn extended_partial_json_selects_consistent_source_defaults() {
        let c: PolarityConfig = serde_json::from_str(r#"{"variant":"overextension"}"#).unwrap();
        assert_eq!(
            (c.horizon, c.stop_at_hegemony, c.tax_discount),
            (4000, false, 0.7)
        );
        assert!(c.validate().is_ok());
    }
    #[test]
    fn provincial_equal_allocation_and_nonfinite_rates_are_rejected() {
        let mut c = PolarityConfig::for_variant(Variant::TwoLevel);
        c.allocation = Allocation::Equal;
        assert!(c
            .validate()
            .unwrap_err()
            .iter()
            .any(|e| e.field == "allocation"));
        c.allocation = Allocation::Pra;
        c.harvest_sd = f64::NAN;
        assert!(c
            .validate()
            .unwrap_err()
            .iter()
            .any(|e| e.field == "harvest_sd"));
    }
    #[test]
    fn explicit_inconsistent_extended_profile_is_not_silently_overwritten() {
        let c: PolarityConfig =
            serde_json::from_str(r#"{"variant":"two_level","source_profile":"chapter4"}"#).unwrap();
        assert!(c.validate().is_err());
    }
}
