//! The Norms and Metanorms model's statistics and Galán & Izquierdo's regions.

use serde::Serialize;

use crate::stats::Series;

/// The statistics series, in the order the CSV and the page list them.
pub const SERIES: [&str; 13] = [
    "mean_boldness",
    "mean_vengefulness",
    "mean_payoff",
    "defections",
    "punishments",
    "metapunishments",
    "established",
    "collapsed",
    "strong_boldness",
    "weak_boldness",
    "strong_vengefulness",
    "weak_vengefulness",
    "copied_equal",
];

/// One generation's statistics: the generation that played this period
/// (period 0: the starting population, before any play).
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct NormsSnapshot {
    pub tick: u64,
    /// Mean boldness and vengefulness (levels ÷ 7).
    pub mean_boldness: f64,
    pub mean_vengefulness: f64,
    pub mean_payoff: f64,
    /// This generation's counts.
    pub defections: u32,
    pub punishments: u32,
    pub metapunishments: u32,
    /// G&I's regions, 1 when in them.
    pub established: u8,
    pub collapsed: u8,
    /// Under `groups`, each group's means (0 otherwise).
    pub strong_boldness: f64,
    pub weak_boldness: f64,
    pub strong_vengefulness: f64,
    pub weak_vengefulness: f64,
    /// 1 when every payoff tied, so the `all_equal` reading applied.
    pub copied_equal: u8,
}

impl Series for NormsSnapshot {
    fn tick(&self) -> u64 {
        self.tick
    }

    fn value(&self, name: &str) -> Option<f64> {
        Some(match name {
            "tick" => self.tick as f64,
            "mean_boldness" => self.mean_boldness,
            "mean_vengefulness" => self.mean_vengefulness,
            "mean_payoff" => self.mean_payoff,
            "defections" => f64::from(self.defections),
            "punishments" => f64::from(self.punishments),
            "metapunishments" => f64::from(self.metapunishments),
            "established" => f64::from(self.established),
            "collapsed" => f64::from(self.collapsed),
            "strong_boldness" => self.strong_boldness,
            "weak_boldness" => self.weak_boldness,
            "strong_vengefulness" => self.strong_vengefulness,
            "weak_vengefulness" => self.weak_vengefulness,
            "copied_equal" => f64::from(self.copied_equal),
            _ => return None,
        })
    }
}

/// G&I §5.12: the norm is established at mean boldness ≤ 2/7 and mean
/// vengefulness ≥ 5/7 …
pub fn established(boldness: f64, vengefulness: f64) -> bool {
    boldness <= 2.0 / 7.0 + 1e-12 && vengefulness >= 5.0 / 7.0 - 1e-12
}

/// … and has collapsed at mean boldness ≥ 6/7 and mean vengefulness ≤ 1/7.
pub fn collapsed(boldness: f64, vengefulness: f64) -> bool {
    boldness >= 6.0 / 7.0 - 1e-12 && vengefulness <= 1.0 / 7.0 + 1e-12
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_regions_are_gis() {
        assert!(established(2.0 / 7.0, 5.0 / 7.0) && !established(3.0 / 7.0, 1.0));
        assert!(collapsed(6.0 / 7.0, 1.0 / 7.0) && !collapsed(1.0, 2.0 / 7.0));
        assert!(!established(0.5, 0.5) && !collapsed(0.5, 0.5));
    }
}
