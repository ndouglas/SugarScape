//! State series are endpoint observations; event series sum complete periods in a tick.
use super::analysis::Ledger;
use crate::stats::Series;
use serde::Serialize;
pub const SERIES: &[&str] = &[
    "period",
    "periods",
    "last_tick_periods",
    "sovereign_count",
    "category_1",
    "category_2",
    "category_3_10",
    "category_11_90",
    "category_91_100",
    "largest_territory",
    "second_largest_territory",
    "predator_capital_share",
    "total_stock",
    "capital_stock",
    "province_stock",
    "nonpositive_stocks",
    "destruction",
    "signed_creation",
    "attacks",
    "dd_encounters",
    "conquests",
    "capital_collapses",
    "disconnections",
    "revolts",
    "coalitions",
    "open_episodes",
    "harvest",
    "taxes",
    "transfers",
    "clipping",
    "stale_claims",
    "locked_claims",
    "double_successes",
    "path_collisions",
];
pub fn series_names() -> Vec<String> {
    SERIES.iter().map(|s| s.to_string()).collect()
}
#[derive(Clone, Debug, Default, Serialize)]
pub struct PolaritySnapshot {
    pub tick: u64,
    pub period: u64,
    pub periods: u64,
    pub attempted_period: u64,
    pub last_tick_periods: u32,
    pub sovereign_count: u32,
    pub sovereigns: u32,
    pub category: Option<super::analysis::Category>,
    pub largest_territory: usize,
    pub second_largest_territory: usize,
    pub predator_capital_share: f64,
    pub total_stock: f64,
    pub capital_stock: f64,
    pub province_stock: f64,
    pub nonpositive_stocks: usize,
    pub coalitions: usize,
    pub open_episodes: usize,
    pub finish_reason: Option<String>,
    pub invalidity: Option<String>,
    pub events_dropped: u64,
    #[serde(flatten)]
    pub events: Ledger,
}
impl Series for PolaritySnapshot {
    fn tick(&self) -> u64 {
        self.tick
    }
    fn value(&self, name: &str) -> Option<f64> {
        Some(match name {
            "tick" => self.tick as f64,
            "period" | "periods" => self.period as f64,
            "last_tick_periods" => self.last_tick_periods as f64,
            "sovereign_count" => self.sovereign_count as f64,
            "category_1" | "category_2" | "category_3_10" | "category_11_90"
            | "category_91_100" => match self.category {
                None => f64::NAN,
                Some(c) => {
                    let wanted = match name {
                        "category_1" => super::Category::One,
                        "category_2" => super::Category::Two,
                        "category_3_10" => super::Category::ThreeToTen,
                        "category_11_90" => super::Category::ElevenToNinety,
                        _ => super::Category::NinetyOneToHundred,
                    };
                    f64::from(u8::from(c == wanted))
                }
            },
            "largest_territory" => self.largest_territory as f64,
            "second_largest_territory" => self.second_largest_territory as f64,
            "predator_capital_share" => self.predator_capital_share,
            "total_stock" => self.total_stock,
            "capital_stock" => self.capital_stock,
            "province_stock" => self.province_stock,
            "nonpositive_stocks" => self.nonpositive_stocks as f64,
            "coalitions" => self.coalitions as f64,
            "open_episodes" => self.open_episodes as f64,
            "destruction" => self.events.destruction,
            "signed_creation" => self.events.signed_creation,
            "harvest" => self.events.harvest,
            "taxes" => self.events.taxes,
            "transfers" => self.events.transfers,
            "clipping" => self.events.clipping,
            "attacks" => self.events.attacks as f64,
            "dd_encounters" => self.events.dd_encounters as f64,
            "conquests" => self.events.conquests as f64,
            "capital_collapses" => self.events.capital_collapses as f64,
            "disconnections" => self.events.disconnections as f64,
            "revolts" => self.events.revolts as f64,
            "stale_claims" => self.events.stale_claims as f64,
            "locked_claims" => self.events.locked_claims as f64,
            "double_successes" => self.events.double_successes as f64,
            "path_collisions" => self.events.path_collisions as f64,
            _ => return None,
        })
    }
}
