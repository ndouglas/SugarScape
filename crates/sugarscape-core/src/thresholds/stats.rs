//! The Threshold Models' statistics: how many act now, how each episode
//! ends, and how widely participation swings.

use serde::Serialize;

use crate::stats::Series;

/// Steps the recent mean and swing look back over.
pub const RECENT: usize = 100;

/// The statistics series, in the order the CSV and the page list them.
pub const SERIES: [&str; 9] = [
    "acting",
    "step",
    "episodes",
    "last_size",
    "mean_size",
    "global_share",
    "theory",
    "recent_mean",
    "swing",
];

/// One step's statistics.
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct ThresholdsSnapshot {
    pub tick: u64,
    /// The share acting now.
    pub acting: f64,
    /// Steps into the current episode.
    pub step: u32,
    /// Episodes finished so far.
    pub episodes: u32,
    /// The last finished episode's final share (0 before the first).
    pub last_size: f64,
    /// The mean final share over finished episodes (0 before the first).
    pub mean_size: f64,
    /// The share of finished episodes that were global.
    pub global_share: f64,
    /// Granovetter's continuous equilibrium share (a normal crowd seen by
    /// everyone); NaN (null) otherwise.
    pub theory: f64,
    /// The mean share acting over the last `RECENT` steps.
    pub recent_mean: f64,
    /// Its range (largest − smallest) over the last `RECENT` steps.
    pub swing: f64,
}

impl Series for ThresholdsSnapshot {
    fn tick(&self) -> u64 {
        self.tick
    }

    fn value(&self, name: &str) -> Option<f64> {
        Some(match name {
            "tick" => self.tick as f64,
            "acting" => self.acting,
            "step" => f64::from(self.step),
            "episodes" => f64::from(self.episodes),
            "last_size" => self.last_size,
            "mean_size" => self.mean_size,
            "global_share" => self.global_share,
            "theory" => self.theory,
            "recent_mean" => self.recent_mean,
            "swing" => self.swing,
            _ => return None,
        })
    }
}
