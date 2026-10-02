//! Tick means and terminal policy statistics.
use crate::stats::Series;
use serde::Serialize;
pub const SERIES: [&str; 20] = [
    "bid_1",
    "bid_2",
    "bid_3",
    "greedy_1",
    "greedy_2",
    "greedy_3",
    "revenue",
    "profit_1",
    "profit_2",
    "profit_3",
    "epsilon",
    "explored",
    "downward",
    "greedy_changes",
    "stable",
    "converged",
    "terminal_revenue",
    "terminal_deviation_gain",
    "top_profile",
    "periods",
];
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct AuctionsSnapshot {
    pub tick: u64,
    pub periods_in_tick: u32,
    pub bid_1: f64,
    pub bid_2: f64,
    pub bid_3: f64,
    pub greedy_1: f64,
    pub greedy_2: f64,
    pub greedy_3: f64,
    pub revenue: f64,
    pub profit_1: f64,
    pub profit_2: f64,
    pub profit_3: f64,
    pub epsilon: f64,
    pub explored: f64,
    pub downward: f64,
    pub greedy_changes: f64,
    pub stable: f64,
    pub converged: f64,
    pub terminal_revenue: f64,
    pub terminal_deviation_gain: f64,
    pub top_profile: f64,
    pub periods: f64,
}
impl Default for AuctionsSnapshot {
    fn default() -> Self {
        Self {
            tick: 0,
            periods_in_tick: 0,
            bid_1: f64::NAN,
            bid_2: f64::NAN,
            bid_3: f64::NAN,
            greedy_1: f64::NAN,
            greedy_2: f64::NAN,
            greedy_3: f64::NAN,
            revenue: f64::NAN,
            profit_1: f64::NAN,
            profit_2: f64::NAN,
            profit_3: f64::NAN,
            epsilon: f64::NAN,
            explored: f64::NAN,
            downward: f64::NAN,
            greedy_changes: f64::NAN,
            stable: f64::NAN,
            converged: f64::NAN,
            terminal_revenue: f64::NAN,
            terminal_deviation_gain: f64::NAN,
            top_profile: f64::NAN,
            periods: f64::NAN,
        }
    }
}
impl Series for AuctionsSnapshot {
    fn tick(&self) -> u64 {
        self.tick
    }
    fn value(&self, name: &str) -> Option<f64> {
        Some(match name {
            "tick" => self.tick as f64,
            "bid_1" => self.bid_1,
            "bid_2" => self.bid_2,
            "bid_3" => self.bid_3,
            "greedy_1" => self.greedy_1,
            "greedy_2" => self.greedy_2,
            "greedy_3" => self.greedy_3,
            "revenue" => self.revenue,
            "profit_1" => self.profit_1,
            "profit_2" => self.profit_2,
            "profit_3" => self.profit_3,
            "epsilon" => self.epsilon,
            "explored" => self.explored,
            "downward" => self.downward,
            "greedy_changes" => self.greedy_changes,
            "stable" => self.stable,
            "converged" => self.converged,
            "terminal_revenue" => self.terminal_revenue,
            "terminal_deviation_gain" => self.terminal_deviation_gain,
            "top_profile" => self.top_profile,
            "periods" => self.periods,
            _ => return None,
        })
    }
}

pub fn series_names(bidders: u32) -> Vec<String> {
    SERIES
        .iter()
        .filter(|s| bidders == 3 || !["bid_3", "greedy_3", "profit_3"].contains(s))
        .map(|s| s.to_string())
        .collect()
}
