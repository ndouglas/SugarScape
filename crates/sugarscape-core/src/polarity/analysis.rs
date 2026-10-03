//! Session categories and conflict episodes are distinct from battle counts.
use super::config::PolarityConfig;
use serde::Serialize;
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Category {
    One,
    Two,
    ThreeToTen,
    ElevenToNinety,
    NinetyOneToHundred,
}
pub fn category(n: u32) -> Option<Category> {
    if !(1..=100).contains(&n) {
        return None;
    }
    Some(match n {
        1 => Category::One,
        2 => Category::Two,
        3..=10 => Category::ThreeToTen,
        11..=90 => Category::ElevenToNinety,
        _ => Category::NinetyOneToHundred,
    })
}
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct Ledger {
    pub attacks: u64,
    pub dd_encounters: u64,
    pub conquests: u64,
    pub capital_collapses: u64,
    pub disconnections: u64,
    pub revolts: u64,
    pub stale_claims: u64,
    pub locked_claims: u64,
    pub path_collisions: u64,
    pub double_successes: u64,
    pub destruction: f64,
    pub signed_creation: f64,
    pub harvest: f64,
    pub taxes: f64,
    pub transfers: f64,
    pub clipping: f64,
}
impl Ledger {
    pub fn all_finite(&self) -> bool {
        [
            self.destruction,
            self.signed_creation,
            self.harvest,
            self.taxes,
            self.transfers,
            self.clipping,
        ]
        .iter()
        .all(|x| x.is_finite())
    }

    pub fn add(&mut self, b: &Self) {
        self.attacks += b.attacks;
        self.dd_encounters += b.dd_encounters;
        self.conquests += b.conquests;
        self.capital_collapses += b.capital_collapses;
        self.disconnections += b.disconnections;
        self.revolts += b.revolts;
        self.stale_claims += b.stale_claims;
        self.locked_claims += b.locked_claims;
        self.path_collisions += b.path_collisions;
        self.double_successes += b.double_successes;
        self.destruction += b.destruction;
        self.signed_creation += b.signed_creation;
        self.harvest += b.harvest;
        self.taxes += b.taxes;
        self.transfers += b.transfers;
        self.clipping += b.clipping;
    }
}
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Episode {
    pub id: u64,
    pub domestic: bool,
    pub capitals: [usize; 2],
    pub initiator: usize,
    pub start: u64,
    pub end: Option<u64>,
    pub duration: u64,
    pub path: [usize; 2],
    pub actions: [bool; 2],
    pub initial_sizes: [usize; 2],
    pub positive_loss: f64,
    pub signed_creation: f64,
    pub end_cause: Option<String>,
    pub winner: Option<usize>,
    pub censored: bool,
}
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Event {
    pub id: u64,
    pub period: u64,
    pub kind: String,
    pub cells: Vec<usize>,
    pub front: Option<[usize; 2]>,
}
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Outcome {
    pub config: PolarityConfig,
    pub seed: u64,
    pub periods: u64,
    pub attempted_period: u64,
    pub finish_reason: String,
    pub valid: bool,
    pub invalid_reason: Option<String>,
    pub sovereign_count: u32,
    pub terminal_category: Option<Category>,
    pub initial_predator_share: f64,
    pub predator_capital_share: f64,
    pub destruction: f64,
    pub signed_creation: f64,
    pub events: Ledger,
    pub episodes: Vec<Episode>,
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn source_category_partition_stops_at_one_hundred() {
        assert_eq!(category(1), Some(Category::One));
        assert_eq!(category(2), Some(Category::Two));
        assert_eq!(category(3), Some(Category::ThreeToTen));
        assert_eq!(category(10), Some(Category::ThreeToTen));
        assert_eq!(category(11), Some(Category::ElevenToNinety));
        assert_eq!(category(90), Some(Category::ElevenToNinety));
        assert_eq!(category(91), Some(Category::NinetyOneToHundred));
        assert_eq!(category(100), Some(Category::NinetyOneToHundred));
        assert_eq!(category(101), None);
    }
}
