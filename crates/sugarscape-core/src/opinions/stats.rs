//! The Bounded Confidence model's statistics: surviving opinions, camps,
//! splits and stability, read from the sorted opinion profile.

use serde::Serialize;

use crate::stats::Series;

/// A period whose largest move is this small is stable. Averaging k equal
/// doubles need not return the same double, so exact stillness is not
/// guaranteed.
pub const STILL: f64 = 1e-10;

/// Opinions this close are one surviving opinion: serial updating closes
/// camps geometrically, leaving spreads far above `STILL` but far below any
/// confidence the paper uses.
pub const SAME: f64 = 1e-6;

/// The statistics series, in the order the CSV and the page list them.
pub const SERIES: [&str; 10] = [
    "clusters",
    "largest",
    "second",
    "mean_opinion",
    "median_opinion",
    "range",
    "splits",
    "one_sided_splits",
    "max_change",
    "stable_at",
];

/// One period's statistics.
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct OpinionsSnapshot {
    pub tick: u64,
    /// Surviving opinions: maximal runs of the sorted profile whose gaps
    /// are at most `SAME`.
    pub clusters: u32,
    /// Shares of agents in the biggest and second-biggest clusters.
    pub largest: f64,
    pub second: f64,
    pub mean_opinion: f64,
    pub median_opinion: f64,
    /// The largest opinion minus the smallest.
    pub range: f64,
    /// Gaps between sorted neighbors beyond both reaches (the lower one's
    /// εr, the upper one's εl), and gaps beyond exactly one of them.
    pub splits: u32,
    pub one_sided_splits: u32,
    /// The largest move this period (0 at the start).
    pub max_change: f64,
    /// The first stable period, else this one.
    pub stable_at: u64,
}

impl Series for OpinionsSnapshot {
    fn tick(&self) -> u64 {
        self.tick
    }

    fn value(&self, name: &str) -> Option<f64> {
        Some(match name {
            "tick" => self.tick as f64,
            "clusters" => f64::from(self.clusters),
            "largest" => self.largest,
            "second" => self.second,
            "mean_opinion" => self.mean_opinion,
            "median_opinion" => self.median_opinion,
            "range" => self.range,
            "splits" => f64::from(self.splits),
            "one_sided_splits" => f64::from(self.one_sided_splits),
            "max_change" => self.max_change,
            "stable_at" => self.stable_at as f64,
            _ => return None,
        })
    }
}

/// The sizes of the clusters of a sorted profile, in order.
pub fn clusters(sorted: &[f64]) -> Vec<usize> {
    let mut out = Vec::new();
    let mut run = 0;
    for (k, &x) in sorted.iter().enumerate() {
        if k > 0 && x - sorted[k - 1] > SAME {
            out.push(run);
            run = 0;
        }
        run += 1;
    }
    if run > 0 {
        out.push(run);
    }
    out
}

/// Two-sided and one-sided splits of a sorted profile, given each opinion's
/// reach `(εl, εr)`.
pub fn splits(sorted: &[f64], reach: impl Fn(f64) -> (f64, f64)) -> (u32, u32) {
    let (mut two, mut one) = (0, 0);
    for w in sorted.windows(2) {
        let gap = w[1] - w[0];
        if gap <= SAME {
            continue;
        }
        let up = gap <= reach(w[0]).1;
        let down = gap <= reach(w[1]).0;
        match (up, down) {
            (false, false) => two += 1,
            (true, true) => {}
            _ => one += 1,
        }
    }
    (two, one)
}

/// The median of a sorted, nonempty profile.
pub fn median(sorted: &[f64]) -> f64 {
    let n = sorted.len();
    if !n.is_multiple_of(2) {
        sorted[n / 2]
    } else {
        (sorted[n / 2 - 1] + sorted[n / 2]) / 2.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clusters_join_opinions_within_the_tolerance() {
        assert_eq!(
            clusters(&[0.1, 0.1, 0.1 + 1e-7, 0.5, 0.9, 0.9 + 1e-5]),
            [3, 1, 1, 1]
        );
        assert_eq!(clusters(&[0.3]), [1]);
        assert!(clusters(&[]).is_empty());
    }

    #[test]
    fn splits_count_gaps_beyond_one_or_both_reaches() {
        let sym = |_| (0.1, 0.1);
        assert_eq!(splits(&[0.0, 0.05, 0.3, 0.3, 0.6], sym), (2, 0));
        // Reaching 0.2 right but 0.05 left: a 0.1 gap is one-sided.
        let asym = |_| (0.05, 0.2);
        assert_eq!(splits(&[0.0, 0.1, 0.5], asym), (1, 1));
    }

    #[test]
    fn medians_of_odd_and_even_profiles() {
        assert_eq!(median(&[0.1, 0.2, 0.9]), 0.2);
        assert!((median(&[0.1, 0.2, 0.4, 0.9]) - 0.3).abs() < 1e-12);
    }
}
