//! Zero-Intelligence Traders' statistics: this period's trades so far and the
//! last completed period's, with Smith's convergence coefficient α and Gode and
//! Sunder's efficiency and profit dispersion, and averages over the periods.

use serde::Serialize;

use crate::stats::Series;

/// The statistics series, in the order the CSV and the page list them.
pub const SERIES: [&str; 16] = [
    "price",
    "mean_price",
    "volume",
    "efficiency",
    "rmsd",
    "alpha",
    "dispersion",
    "period",
    "p0",
    "last_price",
    "last_efficiency",
    "last_alpha",
    "last_dispersion",
    "avg_price",
    "avg_efficiency",
    "avg_dispersion",
];

/// One shout's statistics.
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct ZiSnapshot {
    pub tick: u64,
    /// This shout's trade price; NaN (null) if it did not trade.
    pub price: f64,
    /// This period so far: the mean price (NaN before a trade), units traded,
    /// profit over the maximum surplus (%), the RMS deviation of prices from
    /// P₀, Smith's α (100 × rmsd / P₀), and the RMS of each trader's profit
    /// minus its equilibrium profit.
    pub mean_price: f64,
    pub volume: u32,
    pub efficiency: f64,
    pub rmsd: f64,
    pub alpha: f64,
    pub dispersion: f64,
    /// The period under way (1-based) and its equilibrium price.
    pub period: u64,
    pub p0: f64,
    /// The last completed period's values (NaN before one completes).
    pub last_price: f64,
    pub last_efficiency: f64,
    pub last_alpha: f64,
    pub last_dispersion: f64,
    /// Means over the completed periods (NaN before one completes).
    pub avg_price: f64,
    pub avg_efficiency: f64,
    pub avg_dispersion: f64,
}

impl Series for ZiSnapshot {
    fn tick(&self) -> u64 {
        self.tick
    }

    fn value(&self, name: &str) -> Option<f64> {
        Some(match name {
            "tick" => self.tick as f64,
            "price" => self.price,
            "mean_price" => self.mean_price,
            "volume" => f64::from(self.volume),
            "efficiency" => self.efficiency,
            "rmsd" => self.rmsd,
            "alpha" => self.alpha,
            "dispersion" => self.dispersion,
            "period" => self.period as f64,
            "p0" => self.p0,
            "last_price" => self.last_price,
            "last_efficiency" => self.last_efficiency,
            "last_alpha" => self.last_alpha,
            "last_dispersion" => self.last_dispersion,
            "avg_price" => self.avg_price,
            "avg_efficiency" => self.avg_efficiency,
            "avg_dispersion" => self.avg_dispersion,
            _ => return None,
        })
    }
}
