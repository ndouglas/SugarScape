//! The hoarding world's statistics (spec amendments, item 13).
//!
//! **Per bout.** One small snapshot per bout, tick 0 included, carrying the
//! current generation's running values. A keyframe restore cuts the history
//! by tick, so the history must hold exactly one entry per tick. A 60-
//! generation run is 120 000 bouts, so a snapshot holds only the tick and
//! nine numbers. The trait means are constant within a season (traits don't
//! change while it runs); survivors and the larder share are live; the loss
//! rates are the season's pooled rates so far. The snapshot at a season's
//! last tick is that season's end, so it equals the season's record.
//!
//! **Per generation.** The page charts mean L, mean D, the loss rates,
//! survivors and the larder share by generation. Those come from the world's
//! per-generation records (`HoardWorld::seasons`) through
//! [`by_generation`], one value per finished season under the same names as
//! the per-bout series, rather than from the 120 000-entry bout history.

use serde::Serialize;

use super::world::Season;
use crate::stats::Series;

/// The per-bout series, in the order the CSV and the page list them.
pub const SERIES: [&str; 9] = [
    "generation",
    "mean_larder_prob",
    "hoarder_larder_prob",
    "mean_defense",
    "survivors",
    "larder_share",
    "larder_loss_rate",
    "scatter_loss_rate",
    "takeover",
];

/// The per-generation series ([`by_generation`]): `SERIES` less the
/// takeover flag, which belongs to the run.
pub const GENERATION_SERIES: [&str; 8] = [
    "generation",
    "mean_larder_prob",
    "hoarder_larder_prob",
    "mean_defense",
    "survivors",
    "larder_share",
    "larder_loss_rate",
    "scatter_loss_rate",
];

/// One bout's statistics.
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct HoardSnapshot {
    pub tick: u64,
    /// The generation (1-based).
    pub generation: u64,
    /// The mean probability of larder hoarding (L) and of burrow defense
    /// (D) among the agents born into the generation.
    pub mean_larder_prob: f64,
    /// Mean L among the generation's hoarders only (NaN when all are
    /// cheaters): a cheater's L is never expressed, so this is the mean the
    /// takeover is classified on (item 11, with item 14's cheaters).
    pub hoarder_larder_prob: f64,
    pub mean_defense: f64,
    /// Agents alive now.
    pub survivors: u32,
    /// Larder items ÷ all items the living agents hold (NaN when none).
    pub larder_share: f64,
    /// The generation's pooled per-item daily loss rates so far: items taken
    /// from living owners ÷ (bout-start stock ÷ bouts a day), summed over its
    /// agents (NaN before any stock).
    pub larder_loss_rate: f64,
    pub scatter_loss_rate: f64,
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
            "mean_larder_prob" => self.mean_larder_prob,
            "hoarder_larder_prob" => self.hoarder_larder_prob,
            "mean_defense" => self.mean_defense,
            "survivors" => f64::from(self.survivors),
            "larder_share" => self.larder_share,
            "larder_loss_rate" => self.larder_loss_rate,
            "scatter_loss_rate" => self.scatter_loss_rate,
            "takeover" => self.takeover,
            _ => return None,
        })
    }
}

/// One value per finished season for a name in [`GENERATION_SERIES`] (NaN
/// where the season has none), or `None` for any other name. Each value
/// equals that season's last per-bout snapshot of the same name.
pub fn by_generation(seasons: &[Season], name: &str) -> Option<Vec<f64>> {
    let value: fn(&Season) -> f64 = match name {
        "generation" => |s| f64::from(s.generation),
        "mean_larder_prob" => |s| s.summary.mean_l,
        "hoarder_larder_prob" => |s| s.summary.hoarder_mean_l.unwrap_or(f64::NAN),
        "mean_defense" => |s| s.summary.mean_d,
        "survivors" => |s| f64::from(s.summary.survivors),
        "larder_share" => |s| s.summary.larder_share.unwrap_or(f64::NAN),
        "larder_loss_rate" => |s| s.summary.pooled_larder_rate.unwrap_or(f64::NAN),
        "scatter_loss_rate" => |s| s.summary.pooled_scatter_rate.unwrap_or(f64::NAN),
        _ => return None,
    };
    Some(seasons.iter().map(value).collect())
}
