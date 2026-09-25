//! The spatial games' statistics.

use serde::Serialize;

use crate::stats::Series;

/// The statistics series, in the order the CSV and the page list them.
pub const SERIES: [&str; 7] = [
    "fraction_c",
    "changed",
    "c_to_d",
    "d_to_c",
    "mean_payoff_c",
    "mean_payoff_d",
    "players",
];

/// One generation's statistics.
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct SpatialSnapshot {
    pub tick: u64,
    /// The share of players cooperating.
    pub fraction_c: f64,
    /// The share of players whose strategy changed this generation.
    pub changed: f64,
    /// Players that switched from C to D, and from D to C.
    pub c_to_d: u32,
    pub d_to_c: u32,
    /// Mean score of the cooperators and of the defectors (NaN with none;
    /// JSON null).
    pub mean_payoff_c: f64,
    pub mean_payoff_d: f64,
    pub players: u32,
}

impl Series for SpatialSnapshot {
    fn tick(&self) -> u64 {
        self.tick
    }

    fn value(&self, name: &str) -> Option<f64> {
        Some(match name {
            "tick" => self.tick as f64,
            "fraction_c" => self.fraction_c,
            "changed" => self.changed,
            "c_to_d" => f64::from(self.c_to_d),
            "d_to_c" => f64::from(self.d_to_c),
            "mean_payoff_c" => self.mean_payoff_c,
            "mean_payoff_d" => self.mean_payoff_d,
            "players" => f64::from(self.players),
            _ => return None,
        })
    }
}
