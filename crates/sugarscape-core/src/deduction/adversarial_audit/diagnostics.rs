//! Frozen arithmetic protocol; submitted data never determines expected settings.
use super::*;
use crate::deduction::strategic_reporting::{Config, Policy, Probability, UtilityTable};
use crate::deduction::strategy_inference::{Catalog, PolicyProvenance, Ratio, SignedRatio};
use serde::{Deserialize, Serialize};
mod protocol;
mod references;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DiagnosticReport {
    pub version: String,
    pub passed: bool,
    pub metadata: Metadata,
    pub environments: Vec<EnvironmentRow>,
    pub controller_snapshots: Vec<SnapshotRow>,
    pub fixed_only_models: Vec<FixedModelRow>,
    pub policy_provenance: Vec<PolicyProvenance>,
    pub fitness_tables: Vec<FitnessTable>,
    pub targeted_audits: Vec<TargetedAudit>,
    pub control_evaluations: Vec<ControlEvaluation>,
    pub nominal_evaluations: Vec<NominalEvaluation>,
    pub cross_target_evaluations: Vec<CrossTargetEvaluation>,
    pub bound_checks: Vec<BoundCheck>,
    pub checks: Vec<DiagnosticCheck>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Metadata {
    pub threat_model: String,
    pub trusted_components: String,
    pub private_view_boundary: String,
    pub prior_assumptions: String,
    pub named_control_order: [String; 5],
    pub uniform_prior: [u8; 5],
    pub optimization_informed_prior: [u8; 5],
    pub tie_rule: String,
    pub decomposition: String,
    pub query_count_per_controller: u16,
    pub raw_encodings: u32,
    pub canonical_behaviors: u16,
    pub aliases_per_behavior: u16,
    pub prior_published_benchmark_predictions: [Ratio; 2],
    pub guarantee_scope: String,
    pub clone_provenance: String,
    pub rng: bool,
    pub learning: bool,
    pub search: bool,
    pub oracle_source_sha256: String,
    pub reference_sha256: String,
    pub dto_extraction_source_sha256: String,
    pub dto_reference_sha256: String,
    pub diagnostic_fixture_sha256: String,
    pub attack_fixture_sha256: String,
    pub evaluation_fixture_sha256: String,
    pub policy_coordinate_source_sha256: String,
    pub prior_definition_source_sha256: String,
    pub provenance_source_report_sha256: String,
    pub champion_source_commit: String,
    pub champion_source_report_sha256: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EnvironmentRow {
    pub id: String,
    pub rules: Config,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SnapshotRow {
    pub environment_id: String,
    pub snapshot: ActionSnapshot,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FixedPosteriorRow {
    pub index: u8,
    pub view: FixedView,
    pub total_mass: u64,
    pub true_mass: u64,
    pub decision: FrozenDecision,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FixedModelRow {
    pub environment_id: String,
    pub rules: Config,
    pub denominator: u64,
    pub rows: Vec<FixedPosteriorRow>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FitnessTable {
    pub environment_id: String,
    pub controller: ControllerKind,
    pub denominator: u64,
    pub basis: [BasisRow; 4],
    pub rows: Vec<FitnessRow>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TargetedAudit {
    pub environment_id: String,
    pub controller: ControllerKind,
    pub witness: BestResponse,
    pub canonical_tie_count: u32,
    pub raw_tie_count: u32,
    pub evaluation: AuditEvaluation,
    pub guarantee_shortfall: Ratio,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ControlEvaluation {
    pub environment_id: String,
    pub controller: ControllerKind,
    pub policy_id: String,
    pub raw_bits: u32,
    pub canonical_bits: u32,
    pub evaluation: AuditEvaluation,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NominalEvaluation {
    pub environment_id: String,
    pub controller: ControllerKind,
    pub population: String,
    pub catalog: Catalog,
    pub evaluation: AuditEvaluation,
    pub nominal_minus_fixed_only: SignedRatio,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CrossTargetEvaluation {
    pub environment_id: String,
    pub controller: ControllerKind,
    pub target_controller: ControllerKind,
    pub witness_bits: u32,
    pub evaluation: AuditEvaluation,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BoundCheck {
    pub environment_id: String,
    pub fixed_guarantee: Ratio,
    pub constant_negative_informed_reference: Ratio,
    pub constant_positive_informed_reference: Ratio,
    pub upper_bound: Ratio,
    pub fixed_policy_count: u32,
    pub passive_policy_count: u32,
    pub passed: bool,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DiagnosticCheck {
    pub quantity: String,
    pub expected_numerator: u64,
    pub actual_numerator: u64,
    pub denominator: u64,
    pub passed: bool,
}

pub fn diagnose() -> Result<DiagnosticReport, Error> {
    protocol::rebuild()
}

/// Rebuild every frozen relation and independent check from trusted constants.
/// In particular, no submitted rules, priors, snapshots, caches or success flags
/// become inputs to expected values. Failed stored flags also invalidate a report.
pub fn report_integrity(report: &DiagnosticReport) -> Result<bool, Error> {
    let expected = protocol::rebuild()?;
    Ok(expected.passed && report == &expected)
}
