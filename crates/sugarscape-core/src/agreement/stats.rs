//! The Relative Agreement model's statistics: Deffuant et al.'s indicator y
//! and the outcome it stands for, clusters with and without DNAW's wings,
//! Weisbuch's dispersion and unmoved agents, and stability.

use serde::Serialize;

use crate::opinions::groups;
use crate::stats::Series;

/// A period in which no opinion or uncertainty moved more than this is
/// stable (planning: 10⁻⁴, 10⁻⁶ and 10⁻⁸ give the same y).
pub const STILL: f64 = 1e-6;

/// Sorted opinions this close are one cluster: pairwise meetings close
/// clusters only geometrically.
pub const GAP: f64 = 1e-3;

/// The statistics series, in the order the CSV and the page list them.
pub const SERIES: [&str; 15] = [
    "y",
    "p_plus",
    "p_minus",
    "outcome",
    "clusters",
    "major",
    "isolated",
    "largest",
    "second",
    "dispersion",
    "unmoved",
    "mean_opinion",
    "mean_uncertainty",
    "max_change",
    "stable_at",
];

/// How a run ended up, from p′₊ and p′₋.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    /// Both below 0.15.
    Central = 0,
    /// Both at least 0.25.
    BothExtremes = 1,
    /// One at least 0.7, the other below 0.1.
    SingleExtreme = 2,
    Intermediate = 3,
}

impl Outcome {
    pub fn of(p_plus: f64, p_minus: f64) -> Self {
        let (hi, lo) = (p_plus.max(p_minus), p_plus.min(p_minus));
        if hi < 0.15 {
            Outcome::Central
        } else if lo >= 0.25 {
            Outcome::BothExtremes
        } else if hi >= 0.7 && lo < 0.1 {
            Outcome::SingleExtreme
        } else {
            Outcome::Intermediate
        }
    }
}

/// One period's statistics.
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct AgreementSnapshot {
    pub tick: u64,
    /// DAWF's indicator y = p′₊² + p′₋².
    pub y: f64,
    /// Shares of the initial moderates that became extremists on each side
    /// (0 on a side without extremists).
    pub p_plus: f64,
    pub p_minus: f64,
    /// 0 central, 1 both extremes, 2 single extreme, 3 intermediate.
    pub outcome: u32,
    /// Groups of at least two sorted opinions with gaps at most `GAP`.
    pub clusters: u32,
    /// Groups holding at least 1 % of agents (and two): DNAW's peaks
    /// without the wings.
    pub major: u32,
    /// Groups of one.
    pub isolated: u32,
    /// Shares of agents in the biggest and second-biggest groups.
    pub largest: f64,
    pub second: f64,
    /// Weisbuch's Σ sᵢ² / N² over every group.
    pub dispersion: f64,
    /// The share of agents whose opinion never changed.
    pub unmoved: f64,
    pub mean_opinion: f64,
    pub mean_uncertainty: f64,
    /// The largest move of an opinion or uncertainty this period.
    pub max_change: f64,
    /// The first stable period, else this one.
    pub stable_at: u64,
}

impl Series for AgreementSnapshot {
    fn tick(&self) -> u64 {
        self.tick
    }

    fn value(&self, name: &str) -> Option<f64> {
        Some(match name {
            "tick" => self.tick as f64,
            "y" => self.y,
            "p_plus" => self.p_plus,
            "p_minus" => self.p_minus,
            "outcome" => f64::from(self.outcome),
            "clusters" => f64::from(self.clusters),
            "major" => f64::from(self.major),
            "isolated" => f64::from(self.isolated),
            "largest" => self.largest,
            "second" => self.second,
            "dispersion" => self.dispersion,
            "unmoved" => self.unmoved,
            "mean_opinion" => self.mean_opinion,
            "mean_uncertainty" => self.mean_uncertainty,
            "max_change" => self.max_change,
            "stable_at" => self.stable_at as f64,
            _ => return None,
        })
    }
}

/// The group measures of a sorted profile: clusters, major, isolated,
/// largest and second shares, dispersion.
pub fn grouping(sorted: &[f64]) -> (u32, u32, u32, f64, f64, f64) {
    let n = sorted.len();
    let mut sizes = groups(sorted, GAP);
    sizes.sort_unstable_by(|a, b| b.cmp(a));
    let major_at = (0.01 * n as f64).max(2.0);
    let share = |k: Option<&usize>| k.map_or(0.0, |&s| s as f64 / n as f64);
    (
        sizes.iter().filter(|&&s| s >= 2).count() as u32,
        sizes.iter().filter(|&&s| s as f64 >= major_at).count() as u32,
        sizes.iter().filter(|&&s| s == 1).count() as u32,
        share(sizes.first()),
        share(sizes.get(1)),
        sizes.iter().map(|&s| (s * s) as f64).sum::<f64>() / (n * n) as f64,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn outcomes_read_both_shares() {
        assert_eq!(Outcome::of(0.0, 0.04), Outcome::Central);
        assert_eq!(Outcome::of(0.43, 0.56), Outcome::BothExtremes);
        assert_eq!(Outcome::of(0.0, 0.9833), Outcome::SingleExtreme);
        // y = 0.49 looks like both extremes; the shares say single (M&C ¶4.10).
        assert_eq!(Outcome::of(0.7, 0.0), Outcome::SingleExtreme);
        assert_eq!(Outcome::of(0.3, 0.1), Outcome::Intermediate);
    }

    #[test]
    fn grouping_counts_clusters_wings_and_dispersion() {
        let mut p = vec![0.0; 150];
        p.extend([0.5; 48]);
        p.extend([0.9, 0.95]);
        p.sort_by(f64::total_cmp);
        let (clusters, major, isolated, largest, second, dispersion) = grouping(&p);
        assert_eq!((clusters, major, isolated), (2, 2, 2));
        assert_eq!((largest, second), (0.75, 0.24));
        let want = (150.0f64.powi(2) + 48.0f64.powi(2) + 2.0) / 200.0f64.powi(2);
        assert!((dispersion - want).abs() < 1e-12);
        let (c, m, i, l, _, d) = grouping(&[0.2, 0.2 + 5e-4, 0.2 + 1e-3]);
        assert_eq!((c, m, i, l, d), (1, 1, 0, 1.0, 1.0), "gaps up to 10⁻³ join");
    }
}
