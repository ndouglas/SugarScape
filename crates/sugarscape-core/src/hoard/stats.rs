//! The hoarding world's statistics, one snapshot per generation.

use serde::Serialize;

use crate::stats::Series;

/// The statistics series, in the order the CSV and the page list them.
pub const SERIES: [&str; 4] = ["generation", "mean_l", "mean_d", "survivors"];

/// One generation's statistics.
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct HoardSnapshot {
    pub tick: u64,
    /// The generation (1-based).
    pub generation: u64,
    /// The mean probability of larder hoarding and of burrow defense among
    /// the agents born into the generation.
    pub mean_l: f64,
    pub mean_d: f64,
    /// Agents alive at the season's end.
    pub survivors: u32,
}

impl Series for HoardSnapshot {
    fn tick(&self) -> u64 {
        self.tick
    }

    fn value(&self, name: &str) -> Option<f64> {
        Some(match name {
            "tick" => self.tick as f64,
            "generation" => self.generation as f64,
            "mean_l" => self.mean_l,
            "mean_d" => self.mean_d,
            "survivors" => f64::from(self.survivors),
            _ => return None,
        })
    }
}
