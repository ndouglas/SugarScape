//! Altruistic Punishment's statistics: the shares of each type, what agents
//! did this period, payoffs, conflict, how different the groups are, and the
//! long-run average Boyd and coauthors plot.

use serde::Serialize;

use crate::stats::Series;

/// The statistics series, in the order the CSV and the page list them.
pub const SERIES: [&str; 11] = [
    "cooperation",
    "contributors",
    "punishers",
    "defectors",
    "punishment",
    "acts",
    "payoff",
    "conflicts",
    "extinctions",
    "spread",
    "long_run",
];

/// One period's statistics.
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct PunishmentSnapshot {
    pub tick: u64,
    /// The mean cooperation trait: contributors plus punishers.
    pub cooperation: f64,
    /// The shares of each type (continuous traits: the means of x(1 − y),
    /// xy and 1 − x).
    pub contributors: f64,
    pub punishers: f64,
    pub defectors: f64,
    /// The mean punishment trait.
    pub punishment: f64,
    /// The share who cooperated this period.
    pub acts: f64,
    /// The mean payoff this period.
    pub payoff: f64,
    /// Conflicts fought, and groups replaced, this period.
    pub conflicts: u32,
    pub extinctions: u32,
    /// The standard deviation of the groups' cooperation.
    pub spread: f64,
    /// The mean `cooperation` over the long-run window so far; NaN (null)
    /// before it.
    pub long_run: f64,
}

impl Series for PunishmentSnapshot {
    fn tick(&self) -> u64 {
        self.tick
    }

    fn value(&self, name: &str) -> Option<f64> {
        Some(match name {
            "tick" => self.tick as f64,
            "cooperation" => self.cooperation,
            "contributors" => self.contributors,
            "punishers" => self.punishers,
            "defectors" => self.defectors,
            "punishment" => self.punishment,
            "acts" => self.acts,
            "payoff" => self.payoff,
            "conflicts" => f64::from(self.conflicts),
            "extinctions" => f64::from(self.extinctions),
            "spread" => self.spread,
            "long_run" => self.long_run,
            _ => return None,
        })
    }
}
