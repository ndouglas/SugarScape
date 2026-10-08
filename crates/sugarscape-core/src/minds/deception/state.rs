//! Treatment configuration and authoritative sender state.
use serde::{Deserialize, Serialize};

pub const NOMINAL_AMOUNT: f64 = 12.0;
pub const OBSERVER_SPAN: u32 = 64;
pub const TICKS: u64 = 64;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SenderPolicy {
    #[default]
    Ordinary,
    MatchedNeutral,
    Sham,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum View {
    #[default]
    Ambiguous,
    Clear,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Layout {
    OnRoute,
    #[default]
    OffRoute,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct LabConfig {
    pub sender: SenderPolicy,
    pub view: View,
    pub display_seen: bool,
    pub layout: Layout,
    pub effort_cost: f64,
    pub mirrored: bool,
}
impl Default for LabConfig {
    fn default() -> Self {
        Self {
            sender: SenderPolicy::Ordinary,
            view: View::Ambiguous,
            display_seen: true,
            layout: Layout::OffRoute,
            effort_cost: 0.0,
            mirrored: false,
        }
    }
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Stage {
    #[default]
    Preparation,
    ToDisplay,
    Display,
    Return,
    Ordinary,
    Departure,
    Cancelled,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SenderState {
    pub stage: Stage,
    pub attempted: bool,
    pub pending_departure: bool,
}
impl Default for SenderState {
    fn default() -> Self {
        Self {
            stage: Stage::Preparation,
            attempted: false,
            pending_departure: false,
        }
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Runtime {
    pub source: u32,
    pub display: u32,
    pub prepared: bool,
    /// Diagnostics never affect decisions or fingerprints.
    pub diagnostics: bool,
}
