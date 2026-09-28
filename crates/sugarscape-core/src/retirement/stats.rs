//! The Timing of Retirement's statistics: how many eligible agents have
//! retired, by group, when the norm set in, and at what ages people retire.

use serde::Serialize;

use crate::stats::Series;

/// Periods the retirement ages look back over.
pub const AGES_WINDOW: usize = 10;

/// The statistics series, in the order the CSV and the page list them.
pub const SERIES: [&str; 11] = [
    "retired",
    "retired_a",
    "retired_b",
    "transition",
    "transition_new",
    "transition_a",
    "transition_b",
    "modal_age",
    "mean_age",
    "rational_share",
    "eligibility",
];

/// One period's statistics.
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct RetirementSnapshot {
    pub tick: u64,
    /// The share of eligible agents retired.
    pub retired: f64,
    /// The same in the first and second group (both `retired` without groups).
    pub retired_a: f64,
    pub retired_b: f64,
    /// The period the norm was reached; NaN (null) before.
    pub transition: f64,
    /// Periods from the policy switch to the new norm; NaN (null) before.
    pub transition_new: f64,
    /// The period each group's eligible agents reached the norm (both
    /// `transition` without groups); NaN (null) before.
    pub transition_a: f64,
    pub transition_b: f64,
    /// The most common and the mean age of retirement over the last
    /// `AGES_WINDOW` periods; NaN (null) with no retirements.
    pub modal_age: f64,
    pub mean_age: f64,
    /// Rationals among the living.
    pub rational_share: f64,
    /// The eligibility age now (it drops at the policy switch).
    pub eligibility: u32,
}

impl Series for RetirementSnapshot {
    fn tick(&self) -> u64 {
        self.tick
    }

    fn value(&self, name: &str) -> Option<f64> {
        Some(match name {
            "tick" => self.tick as f64,
            "retired" => self.retired,
            "retired_a" => self.retired_a,
            "retired_b" => self.retired_b,
            "transition" => self.transition,
            "transition_new" => self.transition_new,
            "transition_a" => self.transition_a,
            "transition_b" => self.transition_b,
            "modal_age" => self.modal_age,
            "mean_age" => self.mean_age,
            "rational_share" => self.rational_share,
            "eligibility" => f64::from(self.eligibility),
            _ => return None,
        })
    }
}
