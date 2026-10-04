//! Endpoint state and source-period clocks for all model hosts.
use crate::stats::Series;
use serde::Serialize;
pub const SERIES: &[&str] = &[
    "period",
    "periods",
    "last_tick_periods",
    "sovereign_count",
    "total_capacity",
    "largest_territory",
    "alerted_states",
    "mean_threshold",
    "completed_wars",
    "active_wars",
    "collector_backlog",
    "damage",
    "conquests",
    "shocks",
];
pub fn series_names() -> Vec<String> {
    SERIES.iter().map(|s| s.to_string()).collect()
}
#[derive(Clone, Debug, Default, Serialize)]
pub struct GeosimSnapshot {
    pub tick: u64,
    pub period: u64,
    pub periods: u64,
    pub attempted_period: u64,
    pub last_tick_periods: u32,
    pub sovereign_count: usize,
    pub total_capacity: Option<f64>,
    pub largest_territory: usize,
    pub alerted_states: usize,
    pub mean_threshold: Option<f64>,
    pub completed_wars: usize,
    pub active_wars: usize,
    pub collector_backlog: usize,
    pub damage: f64,
    pub conquests: u64,
    pub shocks: u64,
    pub finish_reason: Option<String>,
    pub invalidity: Option<String>,
}
impl Series for GeosimSnapshot {
    fn tick(&self) -> u64 {
        self.tick
    }
    fn value(&self, n: &str) -> Option<f64> {
        Some(match n {
            "tick" => self.tick as f64,
            "period" | "periods" => self.period as f64,
            "last_tick_periods" => self.last_tick_periods as f64,
            "sovereign_count" => self.sovereign_count as f64,
            "total_capacity" => return self.total_capacity,
            "largest_territory" => self.largest_territory as f64,
            "alerted_states" => self.alerted_states as f64,
            "mean_threshold" => return self.mean_threshold,
            "completed_wars" => self.completed_wars as f64,
            "active_wars" => self.active_wars as f64,
            "collector_backlog" => self.collector_backlog as f64,
            "damage" => self.damage,
            "conquests" => self.conquests as f64,
            "shocks" => self.shocks as f64,
            _ => return None,
        })
    }
}
