//! Strict wire records for completed settlements and failed attempts.

use super::{
    config::Side,
    math::{NumericIssue, Probability},
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize)]
pub struct Capture {
    pub retain_steps: Vec<u64>,
}

#[derive(Clone, Debug, Serialize)]
pub struct EqualityBasis {
    pub counts: Option<bool>,
    pub rates: Option<bool>,
    pub resources: Option<bool>,
}

#[derive(Clone, Debug, Serialize)]
pub struct RunHeader {
    pub input: super::config::StudyInput,
    pub equality: EqualityBasis,
    pub unavailable: Vec<UnavailableObservation>,
    pub force_unit: String,
    pub clock_unit: String,
}

/// The stationary graph supplies aggregate exposure, never a unique killer.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum GraphCasualtyCause {
    BenchmarkExposure,
}

#[derive(Clone, Debug, Serialize)]
#[serde(tag = "kind", content = "data", rename_all = "snake_case")]
#[allow(clippy::large_enum_variant)] // Public record contract keeps the observation inline.
pub enum ObservedFrame {
    Initial {
        counts: Option<[u32; 2]>,
    },
    Graph {
        cause: GraphCasualtyCause,
        frame: Frame,
        reference: Option<super::reference::ReferencePoint>,
        reference_error: Option<super::reference::ReferenceFailure>,
    },
    Book {
        frame: BookFrame,
    },
}

#[derive(Clone, Debug, Serialize)]
#[serde(tag = "kind", content = "data", rename_all = "snake_case")]
#[allow(clippy::large_enum_variant)] // Public record contract owns its resolved header.
pub enum RunPayload {
    Header { header: RunHeader },
    Observed { frame: ObservedFrame },
    Terminal { summary: RunSummary },
}

#[derive(Clone, Debug, Serialize)]
pub struct RunRecord {
    pub schema: String,
    pub input_identity: String,
    pub payload: RunPayload,
}

#[derive(Clone, Debug, Serialize)]
pub struct BookFrame {
    pub tick: u64,
    pub fingerprint: String,
    pub rng_state: String,
    pub snapshot: crate::stats::Snapshot,
    pub combat_enabled: bool,
    pub deaths: Vec<BookDeath>,
    pub kills: Vec<BookKill>,
    /// Living holdings, in the current World configuration's good order.
    pub agent_stores: Vec<f64>,
    /// Site resources, in the current World configuration's good order.
    pub site_stores: Vec<f64>,
    pub unavailable: Vec<UnavailableObservation>,
}

#[derive(Clone, Debug, Serialize)]
pub struct UnavailableObservation {
    pub quantity: String,
    pub reason: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct BookDeath {
    pub id: u64,
    pub tribe: String,
    pub cause: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct BookKill {
    pub attacker: u64,
    pub victim: u64,
    pub loot: f64,
}

#[derive(Clone, Debug, Serialize)]
#[serde(tag = "kind", content = "data", rename_all = "snake_case")]
#[allow(clippy::large_enum_variant)] // Only explicitly requested nonterminal frames are retained.
pub enum CapturedFrame {
    Available {
        input_identity: String,
        frame: ObservedFrame,
    },
    Unavailable {
        step: u64,
        reason: String,
    },
}

#[derive(Clone, Debug, Serialize)]
pub struct RunSummary {
    pub completed_steps: u64,
    pub ending: Option<Ending>,
    pub capture: Vec<CapturedFrame>,
}

#[derive(Clone, Debug, Serialize)]
pub struct RunFailure {
    pub kind: String,
    pub detail: String,
    pub attempted_step: Option<u64>,
    pub completed_steps: u64,
    /// Successfully emitted settlement records; header/initial/terminal do not count.
    pub emitted_steps: u64,
    pub checkpoint: Option<super::checkpoint::Checkpoint>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EndReason {
    DoubleExtinction,
    OneSideExtinction,
    RateZero,
    Horizon,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Ending {
    pub reason: EndReason,
    pub winner: Option<Side>,
    pub censored: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Casualty {
    pub id: u32,
    pub side: Side,
}

/// Arrays are Blue then Red; rates/integrals describe outgoing contributions,
/// while exposed counts and probabilities describe incoming target exposure.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Exposure {
    pub contact_pairs: u64,
    pub exposed: [u32; 2],
    pub contributed_rate: [f64; 2],
    pub integrated: [f64; 2],
    pub target_probability: [Option<Probability>; 2],
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Frame {
    pub step: u64,
    pub start: [u32; 2],
    pub survivors: [u32; 2],
    pub active_steps: u64,
    pub calendar_time: f64,
    pub active_time: f64,
    pub exposure: Exposure,
    pub casualties: Vec<Casualty>,
    pub ending: Option<Ending>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StepFailure {
    pub attempted_step: u64,
    pub issue: NumericIssue,
}

pub(super) fn ending_for(
    counts: [u32; 2],
    rates: [f64; 2],
    step: u64,
    horizon: u64,
) -> Option<Ending> {
    let (reason, winner, censored) = match counts {
        [0, 0] => (EndReason::DoubleExtinction, None, false),
        [0, _] => (EndReason::OneSideExtinction, Some(Side::Red), false),
        [_, 0] => (EndReason::OneSideExtinction, Some(Side::Blue), false),
        _ if rates == [0.0, 0.0] => (EndReason::RateZero, None, false),
        _ if step == horizon => (EndReason::Horizon, None, true),
        _ => return None,
    };
    Some(Ending {
        reason,
        winner,
        censored,
    })
}
