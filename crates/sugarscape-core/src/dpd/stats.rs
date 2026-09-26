//! The demographic Prisoner's Dilemma's statistics.

use serde::Serialize;

use crate::stats::Series;

/// The statistics series, in the order the CSV and the page list them.
pub const SERIES: [&str; 9] = [
    "cooperators",
    "defectors",
    "population",
    "cooperator_share",
    "surrounded",
    "wealth_c",
    "wealth_d",
    "births",
    "deaths",
];

/// One cycle's statistics, of the agents alive at its end. A share or mean
/// with nobody to count is NaN (JSON null).
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct DpdSnapshot {
    pub tick: u64,
    pub cooperators: u32,
    pub defectors: u32,
    pub population: u32,
    pub cooperator_share: f64,
    /// Cooperators all eight of whose Moore neighbours are cooperators.
    pub surrounded: u32,
    /// Mean wealth of cooperators and of defectors.
    pub wealth_c: f64,
    pub wealth_d: f64,
    /// Offspring born and agents dead this cycle.
    pub births: u32,
    pub deaths: u32,
}

impl Series for DpdSnapshot {
    fn tick(&self) -> u64 {
        self.tick
    }

    fn value(&self, name: &str) -> Option<f64> {
        Some(match name {
            "tick" => self.tick as f64,
            "cooperators" => f64::from(self.cooperators),
            "defectors" => f64::from(self.defectors),
            "population" => f64::from(self.population),
            "cooperator_share" => self.cooperator_share,
            "surrounded" => f64::from(self.surrounded),
            "wealth_c" => self.wealth_c,
            "wealth_d" => self.wealth_d,
            "births" => f64::from(self.births),
            "deaths" => f64::from(self.deaths),
            _ => return None,
        })
    }
}

/// `a / b`, or NaN when `b` is 0.
pub fn ratio(a: f64, b: u32) -> f64 {
    if b == 0 {
        f64::NAN
    } else {
        a / f64::from(b)
    }
}
