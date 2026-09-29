//! Balinese Water Temples' statistics: each year's harvest and its spread,
//! the plans' changes, water stress and pest loss, and how the patches of
//! shared plans compare with the temples.

use serde::Serialize;

use crate::stats::Series;

/// The statistics series, in the order the CSV and the page list them.
pub const SERIES: [&str; 11] = [
    "harvest",
    "spread",
    "scored",
    "changing",
    "water_stress",
    "pest_loss",
    "patches",
    "strategies",
    "temple_match",
    "network_match",
    "year",
];

/// One month's statistics (the yearly ones held from the last year's end).
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct BaliSnapshot {
    pub tick: u64,
    /// The last year's harvest, t/ha (area-weighted), and its standard
    /// deviation across subaks (Janssen's "inequality"); two nodes: their total.
    pub harvest: f64,
    pub spread: f64,
    /// The mean harvest over the scored years so far (NaN before them).
    pub scored: f64,
    /// Subaks that changed plans at the last year's end.
    pub changing: u32,
    /// The last year's mean water shortfall over growing months, and the
    /// share of the potential harvest lost to pests.
    pub water_stress: f64,
    pub pest_loss: f64,
    /// Connected groups of subaks sharing a plan and start month, and the
    /// distinct plans and starts in use (NaN for adaptive subaks).
    pub patches: f64,
    pub strategies: f64,
    /// The adjusted Rand index of the patches, and of the pest network's own
    /// components, against the 14 masceti temples.
    pub temple_match: f64,
    pub network_match: f64,
    /// Completed years.
    pub year: u64,
}

impl Series for BaliSnapshot {
    fn tick(&self) -> u64 {
        self.tick
    }

    fn value(&self, name: &str) -> Option<f64> {
        Some(match name {
            "tick" => self.tick as f64,
            "harvest" => self.harvest,
            "spread" => self.spread,
            "scored" => self.scored,
            "changing" => f64::from(self.changing),
            "water_stress" => self.water_stress,
            "pest_loss" => self.pest_loss,
            "patches" => self.patches,
            "strategies" => self.strategies,
            "temple_match" => self.temple_match,
            "network_match" => self.network_match,
            "year" => self.year as f64,
            _ => return None,
        })
    }
}

/// The adjusted Rand index of two partitions (labels per item).
pub fn adjusted_rand(a: &[usize], b: &[usize]) -> f64 {
    use std::collections::HashMap;
    let n = a.len() as f64;
    let pairs = |k: f64| k * (k - 1.0) / 2.0;
    let mut joint: HashMap<(usize, usize), f64> = HashMap::new();
    let mut ca: HashMap<usize, f64> = HashMap::new();
    let mut cb: HashMap<usize, f64> = HashMap::new();
    for (&x, &y) in a.iter().zip(b) {
        *joint.entry((x, y)).or_default() += 1.0;
        *ca.entry(x).or_default() += 1.0;
        *cb.entry(y).or_default() += 1.0;
    }
    // Sum in a fixed order (HashMap iteration order is not portable).
    let mut j: Vec<f64> = joint.values().copied().collect();
    let mut sa: Vec<f64> = ca.values().copied().collect();
    let mut sb: Vec<f64> = cb.values().copied().collect();
    for v in [&mut j, &mut sa, &mut sb] {
        v.sort_by(f64::total_cmp);
    }
    let index: f64 = j.iter().map(|&v| pairs(v)).sum();
    let (xa, xb): (f64, f64) = (
        sa.iter().map(|&v| pairs(v)).sum(),
        sb.iter().map(|&v| pairs(v)).sum(),
    );
    let expected = xa * xb / pairs(n);
    let max = (xa + xb) / 2.0;
    if (max - expected).abs() < 1e-12 {
        0.0
    } else {
        (index - expected) / (max - expected)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_rand_index_is_one_for_equal_partitions_and_near_zero_for_unrelated() {
        let a = [0, 0, 1, 1, 2, 2];
        assert!((adjusted_rand(&a, &[5, 5, 7, 7, 9, 9]) - 1.0).abs() < 1e-12);
        assert!(adjusted_rand(&a, &[0, 1, 0, 1, 0, 1]) < 0.0);
    }
}
