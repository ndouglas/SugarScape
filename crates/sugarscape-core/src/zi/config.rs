//! Zero-Intelligence Traders' parameters: Gode and Sunder's (1993) double
//! auction with budget-constrained and unconstrained random traders, and
//! Cliff's (1997) critique, mechanism and ZIP traders, with every detail the
//! texts leave open as a named switch.

use serde::{Deserialize, Serialize};

use crate::config::FieldError;
use crate::schema::{Apply, Param};

/// Whose limits the traders hold.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Market {
    /// Gode and Sunder's five markets, read from their figures.
    Gs1,
    Gs2,
    Gs3,
    Gs4,
    Gs5,
    /// Cliff's four markets (cents; P₀ 200).
    Symmetric,
    FlatSupply,
    ExcessDemand,
    ExcessSupply,
    /// Cliff's Fig. 50: Smith's retail market, 12 buyers and 11 sellers.
    Retail,
    /// `buyers` and `sellers` from the config.
    Custom,
}

/// How traders shout.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Strategy {
    /// Uniform over the whole price range.
    #[serde(rename = "zi_u")]
    ZiU,
    /// Uniform between the limit and the edge of the range.
    #[serde(rename = "zi_c")]
    ZiC,
    /// Cliff's zero-intelligence-plus: limit × (1 + a learned margin).
    #[serde(rename = "zip")]
    Zip,
}

/// How shouts become trades.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Mechanism {
    /// Gode and Sunder: a standing best bid and ask; a crossing shout trades
    /// at the earlier order's price; a trade clears the book.
    Book,
    /// Cliff's code: the other side's willing traders, one drawn at random,
    /// trade at the shout's price.
    Cliff,
}

/// Who shouts next.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Turns {
    /// A random trader able to shout.
    Trader,
    /// Cliff: a side, weighted by its active traders, then a trader on it.
    Side,
}

/// When a period (Cliff's day) ends.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PeriodEnd {
    /// After a fixed number of shouts.
    Shouts,
    /// Cliff: after 100 failed shouts in a row, or when the side to shout
    /// has no one able.
    Failures,
}

/// ZIP's momentum coefficient γ.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Momentum {
    /// U[0, 0.1]: what Cliff's code ran (it overwrites the text's draw).
    Code,
    /// U[0.2, 0.8]: what his text says.
    Text,
}

/// Cliff's shifts of demand or supply.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Shift {
    None,
    /// Every buyer's limits rise by `SHIFT` from period `shift_at`.
    Demand,
    /// Every seller's limits fall by `SHIFT` from period `shift_at`.
    Supply,
}

/// Cliff's shift: $0.50.
pub const SHIFT: u32 = 50;
/// Cliff's failed shouts in a row that end a day.
pub const MAX_FAILS: u32 = 100;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ZiConfig {
    pub market: Market,
    /// `Market::Custom`: each trader's unit limits, in trading order.
    pub buyers: Vec<Vec<u32>>,
    pub sellers: Vec<Vec<u32>>,
    /// Prices run from 1 to this.
    pub price_max: u32,
    pub strategy: Strategy,
    pub mechanism: Mechanism,
    /// `Mechanism::Cliff`: shouts that cannot beat the best quote are
    /// barred, and the book resets on each trade.
    pub nyse: bool,
    pub turns: Turns,
    /// Only sellers shout (Cliff's retail market). Under `Mechanism::Book` no
    /// one then bids, so nothing trades: buyers accept only under Cliff's.
    pub sellers_only: bool,
    pub period_end: PeriodEnd,
    /// Shouts a period under `PeriodEnd::Shouts`.
    pub shouts: u32,
    pub momentum: Momentum,
    pub shift: Shift,
    /// The first shifted period.
    pub shift_at: u32,
    /// Stop after this many periods (0: never).
    pub stop_at: u32,
}

impl Default for ZiConfig {
    /// Gode and Sunder's market 1 with ZI-C traders, six periods of 2 000
    /// shouts (their "30 seconds" is never translated into shouts).
    fn default() -> Self {
        ZiConfig {
            market: Market::Gs1,
            buyers: Vec::new(),
            sellers: Vec::new(),
            price_max: 200,
            strategy: Strategy::ZiC,
            mechanism: Mechanism::Book,
            nyse: true,
            turns: Turns::Trader,
            sellers_only: false,
            period_end: PeriodEnd::Shouts,
            shouts: 2000,
            momentum: Momentum::Code,
            shift: Shift::None,
            shift_at: 11,
            stop_at: 6,
        }
    }
}

impl ZiConfig {
    pub fn validate(&self) -> Result<(), Vec<FieldError>> {
        let mut e = Vec::new();
        let mut check = |ok: bool, field: &str, message: &str| {
            if !ok {
                e.push(FieldError::new(field, message));
            }
        };
        check(
            (2..=10_000).contains(&self.price_max),
            "price_max",
            "must be between 2 and 10000",
        );
        let custom = self.market == Market::Custom;
        let fine = |side: &Vec<Vec<u32>>| {
            !side.is_empty()
                && side.len() <= 64
                && side.iter().all(|t| !t.is_empty() && t.len() <= 32)
        };
        check(
            !custom || (fine(&self.buyers) && fine(&self.sellers)),
            "buyers",
            "a custom market needs 1–64 buyers and sellers, each with 1–32 units",
        );
        let (buyers, sellers) = super::market::schedules(self);
        let top = buyers
            .iter()
            .chain(&sellers)
            .flatten()
            .copied()
            .max()
            .unwrap_or(1);
        let bottom = buyers
            .iter()
            .chain(&sellers)
            .flatten()
            .copied()
            .min()
            .unwrap_or(1);
        let up = if self.shift == Shift::Demand {
            SHIFT
        } else {
            0
        };
        let down = if self.shift == Shift::Supply {
            SHIFT
        } else {
            0
        };
        check(
            top + up <= self.price_max && bottom > down,
            "price_max",
            "every limit (after a shift) must lie between 1 and the price range's top",
        );
        check(
            (1..=1_000_000).contains(&self.shouts),
            "shouts",
            "must be between 1 and 1000000",
        );
        check(self.shift_at >= 1, "shift_at", "must be at least 1");
        check(
            self.stop_at <= 1_000_000,
            "stop_at",
            "must be at most 1000000",
        );
        if e.is_empty() {
            Ok(())
        } else {
            Err(e)
        }
    }

    /// Fields that change only on reset and differ in `next`.
    pub(crate) fn structural_changes(&self, next: &ZiConfig) -> Vec<FieldError> {
        let mut out = Vec::new();
        for (field, same) in [
            ("market", self.market == next.market),
            ("buyers", self.buyers == next.buyers),
            ("sellers", self.sellers == next.sellers),
            ("price_max", self.price_max == next.price_max),
            ("strategy", self.strategy == next.strategy),
            ("momentum", self.momentum == next.momentum),
        ] {
            if !same {
                out.push(FieldError::new(field, "changes only on reset"));
            }
        }
        out
    }
}

/// The Rules panel's fields.
pub fn schema() -> Vec<Param> {
    use Apply::{Live, Reset};
    vec![
        Param::choice(
            "Market",
            "market",
            "Market",
            &[
                ("gs1", "Gode & Sunder's market 1"),
                ("gs2", "Gode & Sunder's market 2"),
                ("gs3", "Gode & Sunder's market 3"),
                ("gs4", "Gode & Sunder's market 4"),
                ("gs5", "Gode & Sunder's market 5"),
                ("symmetric", "Cliff: symmetric"),
                ("flat_supply", "Cliff: flat supply"),
                ("excess_demand", "Cliff: excess demand"),
                ("excess_supply", "Cliff: excess supply"),
                ("retail", "Cliff: Smith's retail market"),
                ("custom", "Custom"),
            ],
            Reset,
        ),
        Param::integer("Market", "price_max", "Highest price", (2, 10_000), Reset)
            .with_help("Prices run from 1. Gode & Sunder: 200; Cliff: 400 (cents)."),
        Param::choice(
            "Traders",
            "strategy",
            "Traders",
            &[
                ("zi_c", "ZI-C: random, never at a loss"),
                ("zi_u", "ZI-U: random, unconstrained"),
                ("zip", "ZIP: learned margins (Cliff)"),
            ],
            Reset,
        ),
        Param::choice(
            "Traders",
            "momentum",
            "ZIP momentum",
            &[
                ("code", "U[0, 0.1] (Cliff's code)"),
                ("text", "U[0.2, 0.8] (Cliff's text)"),
            ],
            Reset,
        )
        .shown_if("strategy", "zip"),
        Param::choice(
            "Mechanism",
            "mechanism",
            "Trades happen",
            &[
                ("book", "Against the standing quote (Gode & Sunder)"),
                ("cliff", "With a random willing trader (Cliff)"),
            ],
            Live,
        ),
        Param::bool("Mechanism", "nyse", "NYSE rules", Live)
            .shown_if("mechanism", "cliff")
            .with_help("Shouts must be able to beat the best quote; each trade resets it."),
        Param::choice(
            "Mechanism",
            "turns",
            "Who shouts",
            &[
                ("trader", "A random trader"),
                ("side", "A side, then a trader (Cliff)"),
            ],
            Live,
        ),
        Param::bool("Mechanism", "sellers_only", "Only sellers shout", Live)
            .shown_if("mechanism", "cliff"),
        Param::choice(
            "Periods",
            "period_end",
            "A period ends",
            &[
                ("shouts", "After a number of shouts"),
                ("failures", "After 100 failures in a row (Cliff)"),
            ],
            Live,
        ),
        Param::integer("Periods", "shouts", "Shouts a period", (1, 1_000_000), Live)
            .shown_if("period_end", "shouts")
            .with_help("Gode & Sunder: 30 seconds, never given in shouts."),
        Param::choice(
            "Periods",
            "shift",
            "Shift",
            &[
                ("none", "None"),
                ("demand", "Demand up 50"),
                ("supply", "Supply down 50"),
            ],
            Live,
        ),
        Param::integer(
            "Periods",
            "shift_at",
            "Shift from period",
            (1, 1_000_000),
            Live,
        )
        .with_help("Cliff: 11, after ten days."),
        Param::integer(
            "Stopping",
            "stop_at",
            "Stop after period",
            (0, 1_000_000),
            Live,
        )
        .with_help("0: never."),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{ModelConfig, ModelWorld};

    #[test]
    fn defaults_are_gode_and_sunders_market_1() {
        let c = ZiConfig::default();
        assert_eq!(
            (c.market, c.strategy, c.mechanism),
            (Market::Gs1, Strategy::ZiC, Mechanism::Book)
        );
        assert_eq!((c.price_max, c.shouts, c.stop_at), (200, 2000, 6));
        assert!(c.validate().is_ok());
    }

    #[test]
    fn validation_names_fields() {
        let bad = ZiConfig {
            price_max: 1,
            market: Market::Custom,
            shouts: 0,
            shift_at: 0,
            stop_at: 2_000_000,
            ..ZiConfig::default()
        };
        let fields: Vec<String> = bad
            .validate()
            .unwrap_err()
            .into_iter()
            .map(|e| e.field)
            .collect();
        assert_eq!(
            fields,
            ["price_max", "buyers", "shouts", "shift_at", "stop_at"]
        );
        let cliff_at_200 = ZiConfig {
            market: Market::Symmetric,
            ..ZiConfig::default()
        };
        assert_eq!(cliff_at_200.validate().unwrap_err()[0].field, "price_max");
        let shifted = ZiConfig {
            market: Market::Symmetric,
            price_max: 400,
            shift: Shift::Demand,
            ..ZiConfig::default()
        };
        assert!(shifted.validate().is_ok());
    }

    #[test]
    fn the_traders_change_only_on_reset() {
        let next = ZiConfig {
            strategy: Strategy::Zip,
            shouts: 500,
            ..ZiConfig::default()
        };
        let changes = ZiConfig::default().structural_changes(&next);
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].field, "strategy");
    }

    #[test]
    fn strategies_read_as_zi_u_zi_c_and_zip() {
        let c: ZiConfig = serde_json::from_str(
            r#"{"strategy": "zi_u", "market": "flat_supply", "price_max": 400}"#,
        )
        .unwrap();
        assert_eq!((c.strategy, c.market), (Strategy::ZiU, Market::FlatSupply));
        assert!(serde_json::to_string(&c)
            .unwrap()
            .contains(r#""strategy":"zi_u""#));
    }

    #[test]
    fn schema_paths_exist_and_match_what_set_config_allows() {
        let config = ModelConfig::Zi(ZiConfig::default());
        crate::schema::check_schema(&schema(), &config, || {
            ModelWorld::new(config.clone(), 1).unwrap()
        });
    }
}
