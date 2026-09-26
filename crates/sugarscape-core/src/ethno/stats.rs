//! The ethnocentrism model's statistics.

use serde::Serialize;

use crate::stats::Series;

/// The statistics series, in the order the CSV and the page list them.
pub const SERIES: [&str; 14] = [
    "population",
    "ethnocentric",
    "humanitarian",
    "selfish",
    "traitorous",
    "kin",
    "nonkin",
    "mixed",
    "cooperation",
    "same_tag",
    "relatives",
    "kin_help",
    "tag_given_relative",
    "relative_given_tag",
];

/// One period's statistics. Shares are of the population at the end of the
/// period; the interaction statistics are of this period's decisions and
/// neighboring pairs (at the interaction step). A zero denominator gives NaN
/// (JSON null).
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct EthnoSnapshot {
    pub tick: u64,
    pub population: u32,
    pub ethnocentric: f64,
    pub humanitarian: f64,
    pub selfish: f64,
    pub traitorous: f64,
    pub kin: f64,
    pub nonkin: f64,
    pub mixed: f64,
    /// Helps ÷ decisions (HA06's "percent cooperative behavior").
    pub cooperation: f64,
    /// The share of decisions toward an agent of the same tag.
    pub same_tag: f64,
    /// The share of neighboring pairs with a common founding immigrant.
    pub relatives: f64,
    /// The share of helps given to relatives.
    pub kin_help: f64,
    /// P(same tag | related pair) (J13 Table 4's p(i|r)).
    pub tag_given_relative: f64,
    /// P(related | same-tag pair) (J13 Table 4's p(r|i)).
    pub relative_given_tag: f64,
}

impl Series for EthnoSnapshot {
    fn tick(&self) -> u64 {
        self.tick
    }

    fn value(&self, name: &str) -> Option<f64> {
        Some(match name {
            "tick" => self.tick as f64,
            "population" => f64::from(self.population),
            "ethnocentric" => self.ethnocentric,
            "humanitarian" => self.humanitarian,
            "selfish" => self.selfish,
            "traitorous" => self.traitorous,
            "kin" => self.kin,
            "nonkin" => self.nonkin,
            "mixed" => self.mixed,
            "cooperation" => self.cooperation,
            "same_tag" => self.same_tag,
            "relatives" => self.relatives,
            "kin_help" => self.kin_help,
            "tag_given_relative" => self.tag_given_relative,
            "relative_given_tag" => self.relative_given_tag,
            _ => return None,
        })
    }
}

/// This period's interaction counts, from which the snapshot's ratios come.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Tally {
    pub decisions: u64,
    pub helps: u64,
    /// Decisions toward an agent of the same tag.
    pub same_tag: u64,
    /// Helps given to a relative.
    pub helps_to_relatives: u64,
    /// Ordered neighboring pairs (each unordered pair twice): all, related,
    /// same-tag, and both.
    pub pairs: u64,
    pub related: u64,
    pub same: u64,
    pub related_same: u64,
}

/// `a / b`, or NaN when `b` is 0.
pub fn ratio(a: u64, b: u64) -> f64 {
    if b == 0 {
        f64::NAN
    } else {
        a as f64 / b as f64
    }
}
