//! Gode and Sunder's five markets with and without the budget constraint, and
//! Cliff's markets with ZI-C and ZIP traders.

use super::config::{Market, Mechanism, PeriodEnd, Shift, Strategy, Turns, ZiConfig};
use crate::model::ModelConfig;
use crate::presets::ModelPreset;

const GS: &str = "Gode & Sunder 1993, JPE 101: 119";
const CLIFF: &str = "Cliff 1997, HP Labs HPL-97-91";

fn preset(
    id: &'static str,
    name: &'static str,
    source: &'static str,
    description: &'static str,
    edit: impl FnOnce(&mut ZiConfig),
) -> ModelPreset {
    let mut c = ZiConfig::default();
    edit(&mut c);
    ModelPreset {
        id,
        name,
        source,
        description,
        config: ModelConfig::Zi(c),
    }
}

/// Cliff's simulator: his mechanism, side-first turns, days ended by 100
/// failures, prices in cents, ten days. NYSE rules on for ZI-C (his Section
/// 5.3 runs), off for ZIP (his ZIP control file).
fn cliff(c: &mut ZiConfig, market: Market) {
    c.market = market;
    c.price_max = 400;
    c.mechanism = Mechanism::Cliff;
    c.turns = Turns::Side;
    c.period_end = PeriodEnd::Failures;
    c.stop_at = 10;
}

pub fn presets() -> Vec<ModelPreset> {
    vec![
        preset("gs-1", "Market 1, ZI-C", GS, "Gode and Sunder's market 1: six buyers and six sellers, each with six units to trade one at a time (values 102 down to 77, costs 34 up to 94), P₀ 82. ZI-C traders shout random prices, never at a loss (buyers 1 to their value, sellers their cost to 200); a shout that crosses the standing bid or ask trades at the standing order's price, and a trade clears the book. Their 'six periods of … 30 seconds' is never given in shouts; here 2 000 a period. Measured (20 seeds × 6 periods): 99.9 % of the surplus (Table 2: 99.9), prices tightening toward P₀ within each period; profit dispersion 31.8 (Table 3: 28.5). At 100 shouts a period, only 58 % (gs-shouts).", |_| {}),
        preset("gs-2", "Market 2, ZI-C", GS, "Gode and Sunder's market 2 (values 117 down to 57, costs 49 up to 74; P₀ 69, as the text says), ZI-C. Measured (20 seeds × 6 periods): 99.8 % efficiency (Table 2: 99.2), dispersion 49.0 (Table 3: 49.8).", |c| {
            c.market = Market::Gs2
        }),
        preset("gs-3", "Market 3, ZI-C", GS, "Gode and Sunder's market 3: each trader has three units (values 133, 95, 90; costs 90, 95, 100), so only six units carry surplus — the text's volume of 6. ZI-C. Measured (20 seeds × 6 periods): 99.7 % efficiency (Table 2: 99.0), dispersion 18.0 (Table 3: 15.9).", |c| {
            c.market = Market::Gs3
        }),
        preset("gs-4", "Market 4, ZI-C", GS, "Gode and Sunder's market 4 (values 180 down to 160; costs 90, 142, 170, 190, 198; P₀ 170, as the text says). ZI-C. Measured (20 seeds × 6 periods): 99.5 % efficiency (Table 2: 98.2), dispersion 64.8 (Table 3: 60.5) — while unconstrained traders capture less than half (gs-4-u).", |c| {
            c.market = Market::Gs4
        }),
        preset("gs-5", "Market 5, ZI-C", GS, "Gode and Sunder's market 5: a fine staircase of values and costs, and 'costs and redemption values of all the units of several buyers and sellers … placed just beyond the equilibrium point' (read from the scan as well as it allows: three buyers and three sellers with seven units each at 127 and 131, P₀ 129). ZI-C. Measured (20 seeds × 6 periods): 97.1 % efficiency (Table 2: 97.1), the lowest of the five, as those marginal traders displace intramarginal units; dispersion 23.7 (Table 3: 19.1).", |c| {
            c.market = Market::Gs5
        }),
        preset(
            "gs-1-u",
            "Market 1, ZI-U",
            GS,
            "Market 1 with ZI-U traders: random prices over the whole range 1–200, free to buy above their value and sell below their cost. Every unit trades, the losing ones too, so efficiency follows from the schedules: 90.0 % (Table 2: 90.0). Measured (20 seeds × 6 periods): prices wander over the whole range with no pull toward P₀; profit dispersion 177.5 (Table 3: 225.5).",
            |c| c.strategy = Strategy::ZiU,
        ),
        preset(
            "gs-4-u",
            "Market 4, ZI-U",
            GS,
            "Market 4 with ZI-U traders: every unit trades, and market 4's extramarginal units carry large losses. Measured (20 seeds × 6 periods): 48.8 % efficiency (Table 2: 48.8), the lowest of Gode and Sunder's baselines; dispersion 384.2 (Table 3: 363.8).",
            |c| {
                c.market = Market::Gs4;
                c.strategy = Strategy::ZiU;
            },
        ),
        preset(
            "cliff-symmetric",
            "Fig. 24: symmetric, ZI-C",
            CLIFF,
            "Cliff's critique of Gode and Sunder, in his own simulator: 11 buyers and 11 sellers with one unit each, limits 75 to 325 cents by 25, P₀ 200. His mechanism: after each shout, every trader on the other side draws a fresh random price, and a random one of those willing trades at the shout's price; days end after 100 failed shouts in a row. Symmetric schedules: his predicted E(P) is P₀. Measured (50 seeds × 10 days): a mean price of 200.6.",
            |c| cliff(c, Market::Symmetric),
        ),
        preset(
            "cliff-flat",
            "Fig. 26: flat supply, ZI-C",
            CLIFF,
            "Cliff's flat-supply market: 11 sellers all at 200 cents, buyers 75 to 325. His predicted mean price is '233⅓' — though his own Eq. 5 gives 241⅔ (and his discrete pdf 245⅓). Measured (50 seeds × 10 days): 235.8 — well above P₀ 200, as he argues, and near the number he printed rather than his formula's. In Gode and Sunder's own mechanism, 216.6 (cliff-prices).",
            |c| cliff(c, Market::FlatSupply),
        ),
        preset(
            "cliff-excess-demand",
            "Fig. 28: excess demand, ZI-C",
            CLIFF,
            "Cliff's excess-demand box: 11 buyers all valuing 200 cents, 6 sellers all costing 50, P₀ 200. Predicted E(P): 125. Measured (50 seeds × 10 days): 138.1 — far below P₀, as he argues, but 13 above his prediction; in Gode and Sunder's own mechanism, 161.8.",
            |c| cliff(c, Market::ExcessDemand),
        ),
        preset(
            "cliff-excess-supply",
            "Fig. 30: excess supply, ZI-C",
            CLIFF,
            "Cliff's excess-supply box: 6 buyers valuing 320 cents, 11 sellers costing 200, P₀ 200. Predicted E(P): 260. Measured (50 seeds × 10 days): 250.3 — far above P₀, but 10 below his prediction; in Gode and Sunder's own mechanism, 232.9.",
            |c| cliff(c, Market::ExcessSupply),
        ),
        preset(
            "zip-symmetric",
            "Fig. 36: symmetric, ZIP",
            CLIFF,
            "Cliff's zero-intelligence-plus traders in the symmetric market: each shouts its limit times a profit margin and nudges the margin toward each shout it sees (Widrow–Hoff with momentum), as his code does. Measured (50 seeds): daily mean prices 184, 191, 194, 196, 198 … 200 by day 10 — converging from below, as he notes; efficiency 98.9 % over days 2–10.",
            |c| {
                cliff(c, Market::Symmetric);
                c.strategy = Strategy::Zip;
            c.nyse = false;
            },
        ),
        preset(
            "zip-flat",
            "Fig. 37: flat supply, ZIP",
            CLIFF,
            "ZIP traders in the flat-supply market. 'Typically within the first four trading days' prices converge. Measured (50 seeds): 225, 208, 204, 202 … 201 — within 2 of P₀ by day 4, as stated; profit dispersion a tenth of ZI-C's or less.",
            |c| {
                cliff(c, Market::FlatSupply);
                c.strategy = Strategy::Zip;
            c.nyse = false;
            },
        ),
        preset(
            "zip-excess-demand",
            "Fig. 38: excess demand, ZIP",
            CLIFF,
            "ZIP traders in the excess-demand box. Cliff: 'a comparatively slow (yet steady) approach … from below'. Measured (50 seeds): 124, 144, 167, 182, 188, 194 … 198 by day 10 — from below and steady, as stated. With his text's momentum (U[0.2, 0.8]) instead of his code's (U[0, 0.1]), faster: 179 by day 3 against 167.",
            |c| {
                cliff(c, Market::ExcessDemand);
                c.strategy = Strategy::Zip;
            c.nyse = false;
            },
        ),
        preset(
            "zip-excess-supply",
            "Fig. 40: excess supply, ZIP",
            CLIFF,
            "ZIP traders in the excess-supply box. Measured (50 seeds): 244, 225, 213, 208, 204 … 202 by day 10 — converging from above.",
            |c| {
                cliff(c, Market::ExcessSupply);
                c.strategy = Strategy::Zip;
            c.nyse = false;
            },
        ),
        preset(
            "zip-demand-shift",
            "Fig. 46: demand up $0.50 after day 10, ZIP",
            CLIFF,
            "ZIP traders in the symmetric market, every buyer's value raised by 50 cents after day 10 (P₀ 225). Measured (50 seeds): 200 on day 10, then 221, 222, 223, 224, 225 … 225 by day 20 — re-converging, as Cliff shows.",
            |c| {
                cliff(c, Market::Symmetric);
                c.strategy = Strategy::Zip;
            c.nyse = false;
                c.shift = Shift::Demand;
                c.stop_at = 20;
            },
        ),
        preset(
            "zip-supply-shift",
            "Fig. 48: supply down $0.50 after day 10, ZIP",
            CLIFF,
            "ZIP traders in the symmetric market, every seller's cost lowered by 50 cents after day 10 (P₀ 175). Measured (50 seeds): 200 on day 10, then 161, 168, 170, 172, 173 … 175 by day 20.",
            |c| {
                cliff(c, Market::Symmetric);
                c.strategy = Strategy::Zip;
            c.nyse = false;
                c.shift = Shift::Supply;
                c.stop_at = 20;
            },
        ),
        preset(
            "zip-retail",
            "Fig. 50: only sellers shout, ZIP",
            CLIFF,
            "Smith's retail market in Cliff's Fig. 50: 12 buyers (375 down to 100 cents) and 11 sellers (75 up to 325), P₀ 225; only sellers shout, and buyers take or leave their offers. Cliff: prices 'typically less than $2.00 (significantly below the theoretical equilibrium price of $2.25)'. Measured (50 seeds): 184, 185, 188, 190, 192, 194, 196, 199, 201, 203 — below P₀ throughout, but rising past $2.00 by day 9.",
            |c| {
                cliff(c, Market::Retail);
                c.strategy = Strategy::Zip;
            c.nyse = false;
                c.sellers_only = true;
            },
        ),
    ]
}
