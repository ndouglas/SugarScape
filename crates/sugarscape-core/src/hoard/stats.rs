//! The hoarding world's statistics: one snapshot per bout (tick 0 included),
//! carrying the current generation's running values. A keyframe restore cuts
//! the history by tick, so the history must hold exactly one entry per tick
//! (spec amendments, item 13). The per-generation measurements themselves
//! live in the world (`HoardWorld::seasons`).

use serde::Serialize;

use crate::stats::Series;

/// The statistics series, in the order the CSV and the page list them.
pub const SERIES: [&str; 8] = [
    "generation",
    "mean_l",
    "mean_d",
    "survivors",
    "larder_share",
    "larder_rate",
    "scatter_rate",
    "takeover",
];

/// One bout's statistics.
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct HoardSnapshot {
    pub tick: u64,
    /// The generation (1-based).
    pub generation: u64,
    /// The mean probability of larder hoarding and of burrow defense among
    /// the agents born into the generation.
    pub mean_l: f64,
    pub mean_d: f64,
    /// Agents alive now.
    pub survivors: u32,
    /// Larder items ÷ all items the living agents hold (NaN when none).
    pub larder_share: f64,
    /// The generation's pooled per-item daily loss rates so far: items taken
    /// from living owners ÷ (bout-start stock ÷ bouts a day), summed over its
    /// agents (NaN before any stock).
    pub larder_rate: f64,
    pub scatter_rate: f64,
    /// Item 11's takeover: NaN until the run is finished, then 1 or 0.
    pub takeover: f64,
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
            "larder_share" => self.larder_share,
            "larder_rate" => self.larder_rate,
            "scatter_rate" => self.scatter_rate,
            "takeover" => self.takeover,
            _ => return None,
        })
    }
}
