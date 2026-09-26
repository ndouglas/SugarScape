//! Image scoring's statistics.

use serde::Serialize;

use crate::stats::Series;

/// The statistics series, in the order the CSV and the page list them.
pub const SERIES: [&str; 17] = [
    "help_rate",
    "mean_k",
    "cooperative",
    "mean_payoff",
    "mean_score",
    "k_cooperative",
    "k_defective",
    "h",
    "own_only",
    "and",
    "or",
    "standing",
    "binary_c",
    "binary_x",
    "binary_d",
    "q",
    "helps",
];

/// One generation's statistics: the rounds it played and the strategies
/// that played them (tick 0: the first generation before it plays). A
/// share or mean with nobody to count is NaN (JSON null).
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct ImageSnapshot {
    pub tick: u64,
    /// Helps ÷ rounds played (all groups).
    pub help_rate: f64,
    /// The mean k over agents whose strategy has one (k, AND, OR, binary).
    pub mean_k: f64,
    /// The share whose strategy helps at a generation's start
    /// (`Strategy::cooperative`: k ≤ 0 for the k strategies).
    pub cooperative: f64,
    pub mean_payoff: f64,
    pub mean_score: f64,
    /// Shares: k strategies with k ≤ 0 and k > 0; each other class; the
    /// binary cooperators, discriminators and defectors.
    pub k_cooperative: f64,
    pub k_defective: f64,
    pub h: f64,
    pub own_only: f64,
    pub and: f64,
    pub or: f64,
    pub standing: f64,
    pub binary_c: f64,
    pub binary_x: f64,
    pub binary_d: f64,
    pub q: f64,
    /// Helps given this generation.
    pub helps: u64,
}

impl Series for ImageSnapshot {
    fn tick(&self) -> u64 {
        self.tick
    }

    fn value(&self, name: &str) -> Option<f64> {
        Some(match name {
            "tick" => self.tick as f64,
            "help_rate" => self.help_rate,
            "mean_k" => self.mean_k,
            "cooperative" => self.cooperative,
            "mean_payoff" => self.mean_payoff,
            "mean_score" => self.mean_score,
            "k_cooperative" => self.k_cooperative,
            "k_defective" => self.k_defective,
            "h" => self.h,
            "own_only" => self.own_only,
            "and" => self.and,
            "or" => self.or,
            "standing" => self.standing,
            "binary_c" => self.binary_c,
            "binary_x" => self.binary_x,
            "binary_d" => self.binary_d,
            "q" => self.q,
            "helps" => self.helps as f64,
            _ => return None,
        })
    }
}

/// `a / b`, or NaN when `b` is 0.
pub fn ratio(a: f64, b: u64) -> f64 {
    if b == 0 {
        f64::NAN
    } else {
        a / b as f64
    }
}
