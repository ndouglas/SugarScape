use super::*;
use crate::stats::Series;
use serde::Serialize;
pub const SERIES: &[&str] = &[
    "period",
    "periods",
    "completed_periods",
    "attempted_period",
    "last_tick_periods",
    "democratic_share",
    "clustering_ratio",
    "sovereign_count",
    "democratic_states",
    "predatory_states",
    "conflict_fronts",
    "alliance_count",
    "pariah_count",
];
pub fn series_names() -> Vec<String> {
    SERIES.iter().map(|s| s.to_string()).collect()
}
#[derive(Clone, Debug, Default, Serialize)]
pub struct DemocraticPeaceSnapshot {
    pub tick: u64,
    pub period: u64,
    pub periods: u64,
    pub completed_periods: u64,
    pub attempted_period: u64,
    pub last_tick_periods: u32,
    pub democratic_share: f64,
    pub clustering_ratio: Option<f64>,
    pub clustering_reason: Option<String>,
    pub sovereign_count: u32,
    pub democratic_states: u32,
    pub predatory_states: u32,
    pub conflict_fronts: u32,
    pub alliance_count: u32,
    pub pariah_count: u32,
    pub finish_reason: Option<String>,
    pub invalidity: Option<String>,
    pub invalid_phase: Option<String>,
    pub metrics: Metrics,
}
impl Series for DemocraticPeaceSnapshot {
    fn tick(&self) -> u64 {
        self.tick
    }
    fn value(&self, n: &str) -> Option<f64> {
        Some(match n {
            "tick" => self.tick as f64,
            "period" | "periods" | "completed_periods" => self.period as f64,
            "attempted_period" => self.attempted_period as f64,
            "last_tick_periods" => self.last_tick_periods as f64,
            "democratic_share" => self.democratic_share,
            "clustering_ratio" => return self.clustering_ratio,
            "sovereign_count" => self.sovereign_count as f64,
            "democratic_states" => self.democratic_states as f64,
            "predatory_states" => self.predatory_states as f64,
            "conflict_fronts" => self.conflict_fronts as f64,
            "alliance_count" => self.alliance_count as f64,
            "pariah_count" => self.pariah_count as f64,
            _ => return None,
        })
    }
}
