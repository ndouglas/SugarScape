//! The Social Structure model's statistics, and the pieces of the game and
//! regression they rest on.

use serde::Serialize;

use crate::stats::Series;

/// The statistics series, in the order the CSV and the page list them.
pub const SERIES: [&str; 10] = [
    "mean_payoff",
    "cooperation",
    "mean_y",
    "mean_p",
    "mean_q",
    "high",
    "attained_high",
    "share_high_since",
    "copied",
    "partner_p_slope",
];

/// One period's statistics (period 0: the starting population, before any
/// game).
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct StructureSnapshot {
    pub tick: u64,
    /// Payoff per move over every game this period.
    pub mean_payoff: f64,
    /// The share of moves that cooperated.
    pub cooperation: f64,
    /// Population means after this period's adaptation.
    pub mean_y: f64,
    pub mean_p: f64,
    pub mean_q: f64,
    /// 1 when `mean_payoff` reached the `high` threshold.
    pub high: u8,
    /// The first high period, else −1.
    pub attained_high: i64,
    /// The share of periods from `attained_high` on that were high (0 before).
    pub share_high_since: f64,
    /// The share of agents that copied a partner this period.
    pub copied: f64,
    /// The least-squares slope of partners' mean p on the agent's own p, as
    /// played this period (CRA's Figs. 5–6).
    pub partner_p_slope: f64,
}

impl Series for StructureSnapshot {
    fn tick(&self) -> u64 {
        self.tick
    }

    fn value(&self, name: &str) -> Option<f64> {
        Some(match name {
            "tick" => self.tick as f64,
            "mean_payoff" => self.mean_payoff,
            "cooperation" => self.cooperation,
            "mean_y" => self.mean_y,
            "mean_p" => self.mean_p,
            "mean_q" => self.mean_q,
            "high" => f64::from(self.high),
            "attained_high" => self.attained_high as f64,
            "share_high_since" => self.share_high_since,
            "copied" => self.copied,
            "partner_p_slope" => self.partner_p_slope,
            _ => return None,
        })
    }
}

/// The Prisoner's Dilemma payoff to a player (Table 1: R 3, S 0, T 5, P 1).
pub fn payoff(me: bool, them: bool) -> u32 {
    match (me, them) {
        (true, true) => 3,
        (true, false) => 0,
        (false, true) => 5,
        (false, false) => 1,
    }
}

/// The least-squares slope of `y` on `x` (0 when `x` does not vary), and
/// the regression's F statistic (0 with fewer than three points).
pub fn regression(points: &[(f64, f64)]) -> (f64, f64) {
    let n = points.len() as f64;
    if points.len() < 3 {
        return (0.0, 0.0);
    }
    let (mx, my) = points
        .iter()
        .fold((0.0, 0.0), |(a, b), &(x, y)| (a + x / n, b + y / n));
    let (mut sxx, mut sxy, mut syy) = (0.0, 0.0, 0.0);
    for &(x, y) in points {
        sxx += (x - mx) * (x - mx);
        sxy += (x - mx) * (y - my);
        syy += (y - my) * (y - my);
    }
    if sxx == 0.0 {
        return (0.0, 0.0);
    }
    let slope = sxy / sxx;
    let explained = slope * sxy;
    let residual = (syy - explained).max(0.0);
    let f = if residual == 0.0 {
        f64::INFINITY
    } else {
        explained / (residual / (n - 2.0))
    };
    (slope, f)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn payoffs_are_table_1() {
        assert_eq!(
            [
                payoff(true, true),
                payoff(true, false),
                payoff(false, true),
                payoff(false, false)
            ],
            [3, 0, 5, 1]
        );
    }

    #[test]
    fn the_regression_recovers_a_line() {
        let pts: Vec<(f64, f64)> = (0..10).map(|i| (i as f64, 2.0 * i as f64 + 1.0)).collect();
        let (slope, f) = regression(&pts);
        assert!((slope - 2.0).abs() < 1e-12 && f.is_infinite());
        assert_eq!(
            regression(&[(1.0, 2.0), (1.0, 3.0), (1.0, 4.0)]),
            (0.0, 0.0)
        );
        let noisy = [(0.0, 0.0), (1.0, 2.0), (2.0, 1.0), (3.0, 3.0)];
        let (s, f) = regression(&noisy);
        assert!((s - 0.8).abs() < 1e-12 && f > 0.0, "{s} {f}");
    }
}
