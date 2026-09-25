//! The tags model's statistics: donation, tolerance, and the dominant tag
//! cluster RCA describe (its share, relatedness and takeovers).

use serde::Serialize;

use crate::stats::Series;

/// The statistics series, in the order the CSV and the page list them.
pub const SERIES: [&str; 8] = [
    "donation_rate",
    "mean_tolerance",
    "cluster_share",
    "relatedness",
    "cluster_tolerance",
    "zero_tolerance_share",
    "distinct_tags",
    "takeovers",
];

/// Half-width of a cluster around its modal tag: members lie within 0.02
/// of each other, about a dominant cluster's mean tolerance, and 0.01 is
/// the diagram's bin width. RCA do not define a cluster; this is ours.
pub const CLUSTER_RADIUS: f64 = 0.01;

/// One generation's statistics.
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct TagsSnapshot {
    pub tick: u64,
    pub population: u32,
    /// Donations ÷ pairings this generation (0 with no pairings).
    pub donation_rate: f64,
    pub mean_tolerance: f64,
    /// The share of agents within `CLUSTER_RADIUS` of the modal tag.
    pub cluster_share: f64,
    /// The share of that cluster holding the modal tag exactly (RCA's relatedness).
    pub relatedness: f64,
    /// The cluster's mean tolerance.
    pub cluster_tolerance: f64,
    /// The share of agents with tolerance ≤ 0.
    pub zero_tolerance_share: f64,
    /// Distinct exact tags.
    pub distinct_tags: u32,
    /// Changes of dominant cluster so far.
    pub takeovers: u32,
}

impl Series for TagsSnapshot {
    fn tick(&self) -> u64 {
        self.tick
    }

    fn value(&self, name: &str) -> Option<f64> {
        Some(match name {
            "tick" => self.tick as f64,
            "donation_rate" => self.donation_rate,
            "mean_tolerance" => self.mean_tolerance,
            "cluster_share" => self.cluster_share,
            "relatedness" => self.relatedness,
            "cluster_tolerance" => self.cluster_tolerance,
            "zero_tolerance_share" => self.zero_tolerance_share,
            "distinct_tags" => f64::from(self.distinct_tags),
            "takeovers" => f64::from(self.takeovers),
            _ => return None,
        })
    }
}

/// The modal cluster of `(tag, tolerance)` pairs sorted by tag.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Cluster {
    /// The most common exact tag (ties to the lowest).
    pub modal: f64,
    /// Agents holding it.
    pub modal_count: u32,
    /// Agents within `CLUSTER_RADIUS` of it.
    pub size: u32,
    pub mean_tolerance: f64,
}

/// The modal cluster of `sorted` (by tag, ascending), or `None` if empty.
pub fn cluster(sorted: &[(f64, f64)]) -> Option<Cluster> {
    let mut best: Option<(f64, u32)> = None;
    let mut i = 0;
    while i < sorted.len() {
        let tag = sorted[i].0;
        let run = sorted[i..].iter().take_while(|p| p.0 == tag).count();
        if best.is_none_or(|(_, n)| run as u32 > n) {
            best = Some((tag, run as u32));
        }
        i += run;
    }
    let (modal, modal_count) = best?;
    let members = sorted
        .iter()
        .filter(|p| (p.0 - modal).abs() <= CLUSTER_RADIUS);
    let (size, sum) = members.fold((0u32, 0.0), |(n, s), p| (n + 1, s + p.1));
    Some(Cluster {
        modal,
        modal_count,
        size,
        mean_tolerance: sum / f64::from(size),
    })
}

/// Counts takeovers: a dominant cluster (more than half the agents) whose
/// modal tag lies more than `CLUSTER_RADIUS` from the last dominant one's.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Takeovers {
    last: Option<f64>,
    pub count: u32,
}

impl Takeovers {
    /// Notes one generation's modal cluster in a population of `n`.
    pub fn see(&mut self, c: &Cluster, n: usize) {
        if 2 * c.size as usize <= n {
            return;
        }
        if let Some(last) = self.last {
            if (c.modal - last).abs() > CLUSTER_RADIUS {
                self.count += 1;
            }
        }
        self.last = Some(c.modal);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_modal_tag_is_the_most_common_exact_tag_ties_to_the_lowest() {
        assert_eq!(cluster(&[]), None);
        let c = cluster(&[
            (0.1, 0.0),
            (0.2, 0.0),
            (0.2, 0.02),
            (0.205, 0.04),
            (0.5, 0.0),
        ])
        .unwrap();
        assert_eq!((c.modal, c.modal_count, c.size), (0.2, 2, 3));
        assert!((c.mean_tolerance - 0.02).abs() < 1e-12);
        let tie = cluster(&[(0.3, 0.0), (0.7, 0.0)]).unwrap();
        assert_eq!((tie.modal, tie.size), (0.3, 1));
    }

    #[test]
    fn a_takeover_is_a_new_dominant_cluster_far_from_the_last() {
        let at = |modal: f64, size: u32| Cluster {
            modal,
            modal_count: size,
            size,
            mean_tolerance: 0.0,
        };
        let mut t = Takeovers::default();
        t.see(&at(0.3, 60), 100);
        assert_eq!(t.count, 0, "the first dominant cluster is no takeover");
        t.see(&at(0.305, 70), 100);
        assert_eq!(t.count, 0, "within 0.01: the same cluster, drifting");
        t.see(&at(0.8, 50), 100);
        assert_eq!(t.count, 0, "half is not dominant");
        t.see(&at(0.8, 51), 100);
        assert_eq!(t.count, 1);
        t.see(&at(0.3, 40), 100);
        t.see(&at(0.8, 90), 100);
        assert_eq!(t.count, 1, "a dip below half and back is no takeover");
    }
}
