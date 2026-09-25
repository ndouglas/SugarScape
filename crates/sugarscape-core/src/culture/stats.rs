//! The culture model's statistics: regions, zones and cultures (Axelrod's
//! and Axtell et al.'s counts), and the connected-set labelling behind them.

use serde::Serialize;

use crate::stats::Series;

/// The statistics series, in the order the CSV and the page list them.
pub const SERIES: [&str; 8] = [
    "regions",
    "zones",
    "cultures",
    "largest_region",
    "mean_similarity",
    "active_bonds",
    "changes",
    "stable_at",
];

/// One tick's statistics.
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct CultureSnapshot {
    pub tick: u64,
    /// Connected sets of identical sites.
    pub regions: u32,
    /// Connected sets through pairs sharing at least one feature.
    pub zones: u32,
    /// Distinct cultures.
    pub cultures: u32,
    /// The largest region's share of the sites.
    pub largest_region: f64,
    /// The mean share of features neighboring pairs have in common.
    pub mean_similarity: f64,
    /// Neighboring pairs that can still interact (share some but not all
    /// features); with `soup`, the (feature, trait) values held by two or
    /// more distinct cultures. Zero means stable.
    pub active_bonds: u64,
    /// Traits changed this tick.
    pub changes: u64,
    /// The tick the world became stable, else this tick.
    pub stable_at: u64,
}

impl Series for CultureSnapshot {
    fn tick(&self) -> u64 {
        self.tick
    }

    fn value(&self, name: &str) -> Option<f64> {
        Some(match name {
            "tick" => self.tick as f64,
            "regions" => f64::from(self.regions),
            "zones" => f64::from(self.zones),
            "cultures" => f64::from(self.cultures),
            "largest_region" => self.largest_region,
            "mean_similarity" => self.mean_similarity,
            "active_bonds" => self.active_bonds as f64,
            "changes" => self.changes as f64,
            "stable_at" => self.stable_at as f64,
            _ => return None,
        })
    }
}

/// Distinct cultures among `cultures`, and whether they are settled: every
/// two distinct cultures share no feature (Axtell et al. 1996's global
/// criterion), i.e. every (feature, trait) value is held by one culture at most.
pub fn settle(cultures: &[&[u8]]) -> (u32, bool) {
    let mut distinct: Vec<&[u8]> = cultures.to_vec();
    distinct.sort_unstable();
    distinct.dedup();
    let features = distinct.first().map_or(0, |c| c.len());
    let settled = (0..features).all(|f| {
        let mut held = [false; 256];
        distinct
            .iter()
            .all(|c| !std::mem::replace(&mut held[c[f] as usize], true))
    });
    (distinct.len() as u32, settled)
}

/// Union–find over `n` items.
pub struct Sets {
    parent: Vec<u32>,
}

impl Sets {
    pub fn new(n: usize) -> Self {
        Sets {
            parent: (0..n as u32).collect(),
        }
    }

    pub fn find(&mut self, mut i: u32) -> u32 {
        while self.parent[i as usize] != i {
            let p = self.parent[i as usize];
            self.parent[i as usize] = self.parent[p as usize];
            i = p;
        }
        i
    }

    pub fn union(&mut self, a: u32, b: u32) {
        let (ra, rb) = (self.find(a), self.find(b));
        if ra != rb {
            let (lo, hi) = (ra.min(rb), ra.max(rb));
            self.parent[hi as usize] = lo;
        }
    }

    /// Each item's set as a dense label (sets numbered by their smallest
    /// member, ascending), and each set's size.
    pub fn labels(mut self) -> (Vec<u32>, Vec<u32>) {
        let n = self.parent.len();
        let mut label = vec![u32::MAX; n];
        let mut sizes = Vec::new();
        let mut of = vec![0u32; n];
        for i in 0..n as u32 {
            let r = self.find(i) as usize;
            if label[r] == u32::MAX {
                label[r] = sizes.len() as u32;
                sizes.push(0);
            }
            of[i as usize] = label[r];
            sizes[label[r] as usize] += 1;
        }
        (of, sizes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settled_means_every_two_distinct_cultures_share_nothing() {
        assert_eq!(settle(&[]), (0, true));
        assert_eq!(settle(&[&[1, 2], &[1, 2], &[3, 4]]), (2, true));
        assert_eq!(settle(&[&[1, 2], &[1, 3]]), (2, false));
        assert_eq!(settle(&[&[5, 5]]), (1, true));
    }

    #[test]
    fn sets_label_connected_items_by_their_smallest_member() {
        let mut s = Sets::new(6);
        s.union(4, 1);
        s.union(5, 3);
        s.union(3, 4);
        let (of, sizes) = s.labels();
        assert_eq!(of, [0, 1, 2, 1, 1, 1]);
        assert_eq!(sizes, [1, 4, 1]);
    }
}
