use crate::stats::PairedSummary;
use serde::{Deserialize, Serialize};
use sugarscape_core::minds::behavior_tree::records::UnattainedReason;
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Family {
    Primary,
    Secondary,
    Ablation,
    LegacyReference,
}
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Metric {
    QuotaAttained,
    RestrictedCompletionTicks,
    GrossGathered,
    LivingTicks,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Endpoint {
    pub condition: String,
    pub seed: u64,
    pub quota_attained: bool,
    pub first_completion: Option<u64>,
    pub restricted_completion_ticks: u64,
    pub right_censored: bool,
    pub unattained_reason: Option<UnattainedReason>,
    pub gross_gathered: f64,
    pub living_ticks: u64,
    pub alive_at_horizon: bool,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Estimate {
    pub id: String,
    pub family: Family,
    pub metric: Metric,
    pub denominator: usize,
    pub summary: PairedSummary,
}
use super::archive::{AttemptRef, Census};
use sugarscape_core::minds::behavior_tree::WorkCounters;
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CompletionSummary {
    pub id: String,
    pub denominator: usize,
    pub summary: Option<PairedSummary>,
    pub unavailable: Vec<(u64, String)>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CellDiagnostics {
    pub condition: String,
    pub seeds: Vec<u64>,
    pub completed_ticks: Vec<u64>,
    pub survival_at_64: Vec<bool>,
    pub work: Vec<Option<WorkCounters>>,
    pub controller_seconds_total: Vec<Option<f64>>,
    pub task_active_calls: Vec<u64>,
    pub hold_calls: Vec<u64>,
    pub physical_signature: Vec<String>,
    pub controller_seconds_per_invocation: Vec<Vec<Option<f64>>>,
    pub episode_seconds: Vec<f64>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MatchedCheck {
    pub stratum: String,
    pub seed: u64,
    pub equal_physical_task: bool,
    pub equal_rng: bool,
    pub first_difference_tick: Option<u64>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AliasGroup {
    pub conditions: Vec<String>,
    pub reason: String,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Analysis {
    pub schema: String,
    pub census: Census,
    pub endpoints: Vec<Endpoint>,
    pub estimates: Vec<Estimate>,
    pub matched_pairs: usize,
    pub duplicate_groups: Vec<Vec<(String, u64)>>,
    pub raw_frame_references: Vec<AttemptRef>,
    pub completion: Vec<CompletionSummary>,
    pub cells: Vec<CellDiagnostics>,
    pub matched: Vec<MatchedCheck>,
    pub aliases: Vec<AliasGroup>,
    pub timing_definition: String,
    pub rng_identities: Vec<(String, u64, Vec<String>)>,
}
