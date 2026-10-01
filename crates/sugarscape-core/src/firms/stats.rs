//! The Emergence of Firms' statistics: each period's firms, births and
//! deaths, sizes, effort, output, pay and utility, and the running scaling
//! exponents and mean lifetime from the records since the burn-in.

use serde::Serialize;

use crate::stats::Series;

/// The statistics series, in the order the CSV and the page list them.
pub const SERIES: [&str; 15] = [
    "firms",
    "births",
    "deaths",
    "mean_size",
    "largest",
    "singletons",
    "effort",
    "output",
    "income",
    "utility",
    "largest_output_share",
    "mu",
    "mu_mle",
    "lifetime",
    "period",
];

/// One period's statistics.
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct FirmsSnapshot {
    pub tick: u64,
    /// Firms alive at the period's end, founded and dissolved during it.
    pub firms: u32,
    pub births: u32,
    pub deaths: u32,
    /// Agents per firm, the largest firm, and the share of firms of one.
    pub mean_size: f64,
    pub largest: u32,
    pub singletons: f64,
    /// Means over agents at the period's production.
    pub effort: f64,
    /// Total output of all firms.
    pub output: f64,
    pub income: f64,
    pub utility: f64,
    /// The largest firm's share of total output.
    pub largest_output_share: f64,
    /// The size exponent by A99's OLS and by maximum likelihood, over the
    /// sizes sampled since the burn-in (NaN before).
    pub mu: f64,
    pub mu_mle: f64,
    /// Mean lifetime of firms dissolved since the burn-in (NaN before).
    pub lifetime: f64,
    pub period: u64,
}

impl Series for FirmsSnapshot {
    fn tick(&self) -> u64 {
        self.tick
    }

    fn value(&self, name: &str) -> Option<f64> {
        Some(match name {
            "tick" => self.tick as f64,
            "firms" => f64::from(self.firms),
            "births" => f64::from(self.births),
            "deaths" => f64::from(self.deaths),
            "mean_size" => self.mean_size,
            "largest" => f64::from(self.largest),
            "singletons" => self.singletons,
            "effort" => self.effort,
            "output" => self.output,
            "income" => self.income,
            "utility" => self.utility,
            "largest_output_share" => self.largest_output_share,
            "mu" => self.mu,
            "mu_mle" => self.mu_mle,
            "lifetime" => self.lifetime,
            "period" => self.period as f64,
            _ => return None,
        })
    }
}
