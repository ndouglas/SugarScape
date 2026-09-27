//! The El Farol and Minority Game model's statistics: attendance, how far it
//! swings around the capacity against coin-flippers, who was right, and who
//! switched.

use serde::Serialize;

use crate::stats::Series;

/// Rounds the fluctuation averages over.
pub const WINDOW: usize = 100;

/// The statistics series, in the order the CSV and the page list them.
pub const SERIES: [&str; 9] = [
    "attendance",
    "crowded",
    "fluctuation",
    "random_fluctuation",
    "success",
    "mean_gain",
    "switching",
    "forecast_above",
    "mean_memory",
];

/// One round's statistics.
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct FarolSnapshot {
    pub tick: u64,
    /// A: how many went (or chose side A).
    pub attendance: u32,
    /// 1 when crowded (El Farol: A ≥ L; minority: A > L, so side A lost).
    pub crowded: u32,
    /// (A − c)² over the last `WINDOW` rounds, divided by N; c is L, or N/2
    /// in the plain minority game.
    pub fluctuation: f64,
    /// The same for agents attending at random with probability L/N.
    pub random_fluctuation: f64,
    /// The share of agents whose choice was right this round.
    pub success: f64,
    /// Gain per agent per round so far.
    pub mean_gain: f64,
    /// The share of agents whose active strategy changed this round.
    pub switching: f64,
    /// El Farol: the share of active predictors forecasting above L.
    pub forecast_above: f64,
    pub mean_memory: f64,
}

impl Series for FarolSnapshot {
    fn tick(&self) -> u64 {
        self.tick
    }

    fn value(&self, name: &str) -> Option<f64> {
        Some(match name {
            "tick" => self.tick as f64,
            "attendance" => f64::from(self.attendance),
            "crowded" => f64::from(self.crowded),
            "fluctuation" => self.fluctuation,
            "random_fluctuation" => self.random_fluctuation,
            "success" => self.success,
            "mean_gain" => self.mean_gain,
            "switching" => self.switching,
            "forecast_above" => self.forecast_above,
            "mean_memory" => self.mean_memory,
            _ => return None,
        })
    }
}

/// σ²/N for N agents each attending with probability p, around c (L, or
/// N/2 in the plain minority game, where random agents flip a fair coin).
pub fn random_fluctuation(n: u32, p: f64, c: f64) -> f64 {
    let n = f64::from(n);
    p * (1.0 - p) + (p * n - c).powi(2) / n
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn coin_flippers_fluctuate_by_p_times_one_minus_p() {
        assert!((random_fluctuation(100, 0.6, 60.0) - 0.24).abs() < 1e-12);
        // The plain minority game: a fair coin, around N/2.
        assert_eq!(random_fluctuation(1001, 0.5, 500.5), 0.25);
        // A bias away from the center adds its square.
        assert!(random_fluctuation(100, 0.5, 60.0) > 0.25 + 0.99);
    }
}
