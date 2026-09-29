//! Janssen's (2007, §4) two-node model: an upstream and a downstream subak
//! sharing water and pests. With two periods a year each node plants in the
//! first, the second, or neither (a crop yields 1, water unlimited); with
//! twelve, it follows one of five three-month-crop patterns from any month,
//! the upstream node taking up to a unit of water a month and the downstream
//! node the rest. Pests follow his eq. 2; a crop yields (1 − min(1, P)) × its
//! water (eq. 3). The best pair of plans is found by trying them all.

use serde::Serialize;

/// The five monthly patterns (1 = a crop month), started from any month.
pub const PATTERNS: [[u8; 12]; 5] = [
    [1, 1, 1, 0, 1, 1, 1, 0, 1, 1, 1, 0],
    [1, 1, 1, 0, 1, 1, 1, 0, 0, 0, 0, 0],
    [1, 1, 1, 0, 0, 1, 1, 1, 0, 0, 0, 0],
    [1, 1, 1, 0, 0, 0, 1, 1, 1, 0, 0, 0],
    [1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0],
];

/// The pest floor (unstated for two nodes; as in the watershed).
const FLOOR: f64 = 0.01;

/// One node's plan: a pattern and a start month (or, with two periods, 0
/// none, 1 the first period, 2 the second).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct NodePlan {
    pub pattern: u8,
    pub start: u8,
}

/// Whether node plan `p` has a crop in period `t`.
fn crop(p: NodePlan, t: usize, periods: usize) -> bool {
    if periods == 2 {
        p.pattern as usize == t + 1
    } else {
        PATTERNS[p.pattern as usize][(t + 12 - p.start as usize) % 12] == 1
    }
}

/// Every plan of one node.
pub fn plans(periods: usize) -> Vec<NodePlan> {
    if periods == 2 {
        (0..3)
            .map(|k| NodePlan {
                pattern: k,
                start: 0,
            })
            .collect()
    } else {
        (0..5u8)
            .flat_map(|p| {
                (0..12u8).map(move |s| NodePlan {
                    pattern: p,
                    start: s,
                })
            })
            .collect()
    }
}

/// Two nodes running: their plans, pests, and the water each crop has had.
#[derive(Clone, Debug, PartialEq)]
pub struct Nodes {
    pub pair: [NodePlan; 2],
    pub periods: usize,
    pub g: f64,
    pub d: f64,
    pub rain: f64,
    pub pests: [f64; 2],
    water: [(f64, u32); 2],
    /// The period of the year, and this year's harvest so far.
    pub t: usize,
    pub harvest: f64,
}

impl Nodes {
    pub fn new(pair: [NodePlan; 2], g: f64, d: f64, rain: f64, periods: usize) -> Nodes {
        Nodes {
            pair,
            periods,
            g,
            d,
            rain,
            pests: [FLOOR; 2],
            water: [(0.0, 0); 2],
            t: 0,
            harvest: 0.0,
        }
    }

    /// Whether node `k` has a crop this period.
    pub fn growing(&self, k: usize) -> bool {
        crop(self.pair[k], self.t, self.periods)
    }

    /// One period; the year's harvest when a year ends. Eq. 2 as printed:
    /// the growth term mixes half the difference whatever d is.
    pub fn step(&mut self) -> Option<f64> {
        let (periods, t) = (self.periods, self.t);
        let grow = [self.growing(0), self.growing(1)];
        if periods == 12 {
            let up = if grow[0] { self.rain.min(1.0) } else { 0.0 };
            let down = if grow[1] {
                (self.rain - up).clamp(0.0, 1.0)
            } else {
                0.0
            };
            for (k, w) in [up, down].into_iter().enumerate() {
                if grow[k] {
                    self.water[k].0 += w;
                    self.water[k].1 += 1;
                }
            }
        }
        let rate = |k: usize| if grow[k] { self.g } else { 0.1 };
        let (pu, pd) = (self.pests[0], self.pests[1]);
        let nu = rate(0) * (pu + 0.5 * (pd - pu)) + 0.5 * self.d * (pd - pu);
        let nd = rate(1) * (pd + 0.5 * (pu - pd)) + 0.5 * self.d * (pu - pd);
        self.pests = [nu.max(FLOOR), nd.max(FLOOR)];
        for (k, &growing) in grow.iter().enumerate() {
            let ends = growing && (periods == 2 || !crop(self.pair[k], (t + 1) % 12, periods));
            if ends {
                let ws = if periods == 2 {
                    1.0
                } else {
                    self.water[k].0 / f64::from(self.water[k].1.max(1))
                };
                self.harvest += (1.0 - self.pests[k].min(1.0)) * ws;
                self.water[k] = (0.0, 0);
            }
        }
        self.t += 1;
        if self.t == periods {
            self.t = 0;
            let h = self.harvest;
            self.harvest = 0.0;
            Some(h)
        } else {
            None
        }
    }
}

/// The two nodes' harvest in each of `years` years.
pub fn simulate(
    pair: [NodePlan; 2],
    g: f64,
    d: f64,
    rain: f64,
    periods: usize,
    years: usize,
) -> Vec<f64> {
    let mut n = Nodes::new(pair, g, d, rain, periods);
    let mut out = Vec::with_capacity(years);
    while out.len() < years {
        if let Some(h) = n.step() {
            out.push(h);
        }
    }
    out
}

/// The pair of plans with the highest harvest in the last of `years` years.
pub fn best(g: f64, d: f64, rain: f64, periods: usize, years: usize) -> ([NodePlan; 2], f64) {
    let all = plans(periods);
    let mut top = ([all[0], all[0]], f64::MIN);
    for &a in &all {
        for &b in &all {
            let h = *simulate([a, b], g, d, rain, periods, years).last().unwrap();
            if h > top.1 + 1e-12 {
                top = ([a, b], h);
            }
        }
    }
    top
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn two_periods_plant_both_nodes_until_pests_explode() {
        // Janssen: with g ≤ 10 both nodes can plant (g·0.1 ≤ 1).
        let (pair, h) = best(3.0, 0.3, 2.0, 2, 100);
        assert!(pair.iter().all(|p| p.pattern != 0), "{pair:?}");
        assert!(h > 1.9, "{h}");
        let (_, high) = best(30.0, 0.3, 2.0, 2, 100);
        assert!(high < h);
    }

    #[test]
    fn monthly_below_the_cube_root_of_ten_pests_stay_small() {
        // Janssen: below g ≈ 2.14 (∛10) pests cannot grow exponentially, and
        // with water for both (2 units) three crops each are possible. Each
        // loses the pests grown from the floor in its three months (0.01 ×
        // 2³ = 0.08), so six crops yield 5.52, not 6.
        let (pair, low) = best(2.0, 0.3, 2.0, 12, 60);
        assert!(pair.iter().all(|p| p.pattern == 0), "{pair:?}");
        assert!((low - 6.0 * 0.92).abs() < 1e-9, "{low}");
        let (_, high) = best(2.4, 0.3, 2.0, 12, 60);
        assert!(high < low, "{high} against {low}");
    }

    #[test]
    fn with_one_unit_of_rain_only_one_node_is_watered() {
        let upper = NodePlan {
            pattern: 4,
            start: 0,
        };
        let lower = NodePlan {
            pattern: 4,
            start: 0,
        };
        let h = simulate([upper, lower], 1.0, 0.0, 1.0, 12, 3);
        // The upper crop gets its unit; the lower one nothing.
        assert!((h[2] - (1.0 - 0.01)).abs() < 0.05, "{:?}", h);
    }
}
