//! Strict wire records for completed settlements and failed attempts.

use super::{
    config::Side,
    math::{NumericIssue, Probability},
};
use serde::{Deserialize, Serialize};

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
