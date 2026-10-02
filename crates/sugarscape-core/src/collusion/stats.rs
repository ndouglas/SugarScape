//! Algorithmic Collusion's statistics: each charted period's prices, profit
//! gain and learning, and — once the session has finished — what it learned.

use serde::Serialize;

use crate::stats::Series;

/// The statistics series, in the order the CSV and the page list them.
pub const SERIES: [&str; 17] = [
    "price_1",
    "price_2",
    "profit_gain",
    "greedy_price",
    "epsilon",
    "explored",
    "greedy_changes",
    "stable",
    "converged",
    "cycle_length",
    "cycle_gain",
    "window_gain",
    "discounted_gain",
    "equilibrium_on_path",
    "punishment_like",
    "rp_complete",
    "periods",
];

/// One charted period. The session's results are NaN until it finishes.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct CollusionSnapshot {
    pub tick: u64,
    /// Firms 1 and 2's prices this period.
    pub price_1: f64,
    pub price_2: f64,
    /// Δ of this period's profits, the firms' mean.
    pub profit_gain: f64,
    /// The firms' mean greedy price at this period's state (Lambin's measure).
    pub greedy_price: f64,
    /// Firm 1's exploration rate (or temperature).
    pub epsilon: f64,
    /// The share of firms that explored this period.
    pub explored: f64,
    /// Greedy prices that changed this period.
    pub greedy_changes: u32,
    /// Periods the strategies have stayed the same.
    pub stable: u32,
    /// 1 converged, 0 stopped at the cap, NaN still learning.
    pub converged: f64,
    pub cycle_length: f64,
    /// Δ over the limit cycle (CCDP's).
    pub cycle_gain: f64,
    /// Δ of the realized profits over the last `window` periods.
    pub window_gain: f64,
    /// den Boer, Meylahn & Schinkel's Δ̃ over the first T_δ periods.
    pub discounted_gain: f64,
    /// 1 if every firm best responds on the cycle.
    pub equilibrium_on_path: f64,
    /// The share of deviations (as `impulse` says) answered by a
    /// punishment-like response.
    pub punishment_like: f64,
    /// 1 if every one-period deviation draws a punishment-like response
    /// (den Boer, Meylahn & Schinkel).
    pub rp_complete: f64,
    /// Periods to convergence (excluding the window), as the code reports.
    pub periods: f64,
}

impl Default for CollusionSnapshot {
    fn default() -> Self {
        CollusionSnapshot {
            tick: 0,
            price_1: f64::NAN,
            price_2: f64::NAN,
            profit_gain: f64::NAN,
            greedy_price: f64::NAN,
            epsilon: f64::NAN,
            explored: f64::NAN,
            greedy_changes: 0,
            stable: 0,
            converged: f64::NAN,
            cycle_length: f64::NAN,
            cycle_gain: f64::NAN,
            window_gain: f64::NAN,
            discounted_gain: f64::NAN,
            equilibrium_on_path: f64::NAN,
            punishment_like: f64::NAN,
            rp_complete: f64::NAN,
            periods: f64::NAN,
        }
    }
}

impl Series for CollusionSnapshot {
    fn tick(&self) -> u64 {
        self.tick
    }

    fn value(&self, name: &str) -> Option<f64> {
        Some(match name {
            "tick" => self.tick as f64,
            "price_1" => self.price_1,
            "price_2" => self.price_2,
            "profit_gain" => self.profit_gain,
            "greedy_price" => self.greedy_price,
            "epsilon" => self.epsilon,
            "explored" => self.explored,
            "greedy_changes" => f64::from(self.greedy_changes),
            "stable" => f64::from(self.stable),
            "converged" => self.converged,
            "cycle_length" => self.cycle_length,
            "cycle_gain" => self.cycle_gain,
            "window_gain" => self.window_gain,
            "discounted_gain" => self.discounted_gain,
            "equilibrium_on_path" => self.equilibrium_on_path,
            "punishment_like" => self.punishment_like,
            "rp_complete" => self.rp_complete,
            "periods" => self.periods,
            _ => return None,
        })
    }
}
