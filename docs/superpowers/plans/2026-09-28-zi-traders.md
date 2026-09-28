# Zero-Intelligence Traders Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add Gode and Sunder's double auction with zero-intelligence traders (JPE 1993), with Cliff's critique, simulator and ZIP traders (HP Labs 1997), as one model kind, `zi` ("Zero-Intelligence Traders"): Gode and Sunder's five markets recovered from their scanned figures, Cliff's four markets, shifts and retail market, both mechanisms, every unstated rule a switch; eighteen titled presets, eight measured sweeps and a 16-claim survey, in every playground surface, without changing any existing run.

**Architecture:** A new core module `crates/sugarscape-core/src/zi/` — `config.rs` (parameters, seven reading enums, validation, schema), `market.rs` (the built-in schedules, their shifts, equilibrium and equilibrium profits), `stats.rs`, `view.rs` (schedules, prices across periods, a trader strip), `world.rs` (`ZiWorld`: one shout a tick, the book and Cliff's mechanism, NYSE barring, turns, periods, ZIP's update, statistics, rendering, Inspect), `presets.rs`, `mod.rs` — wired into `ModelConfig`/`ModelWorld` like the other models, with titles in `titles.rs`. The page adds the model's types, color modes, charts, Inspect rows, two Compare entries and an Experiments default.

**Tech Stack:** Rust core, `wasm-bindgen`, the `sugarscape` CLI, TypeScript + Vite + uPlot + Vitest, the standalone `survey` crate. No new dependencies.

**Spec:** `docs/superpowers/specs/2026-09-28-zi-traders-design.md` (binding, as amended in Task 5). Sources: `papers/zi-traders/gode-sunder-1993-jpe-zero-intelligence-traders.pdf` (a scan, read by OCR) and `papers/zi-traders/cliff-1997-hp-minimal-intelligence-agents-for-bargaining.pdf` (with its C source in the appendices).

## Global Constraints

- **Existing runs unchanged:** every existing `GOLDEN` and `MODEL_GOLDEN` entry and legacy fixture stays green and unedited (`MODEL_GOLDEN` gains eighteen `gs-*`, `cliff-*` and `zip-*` entries).
- **One engine path; deterministic; portable:** native and WASM fingerprints identical (verified in planning by `wasm-pack test`: six presets across both mechanisms, ZI-U, ZI-C and ZIP). Draws use `u32` ranges and `f64` samples only; prices are integers; ZIP margins use `+ − × ÷` only. The fingerprint hashes an empty book slot as `u64::MAX`, never `usize::MAX` (which differs on wasm32 — found in planning).
- **Literal defaults, named departures, honest descriptions and titles:** Gode and Sunder's book, random turns and 2 000 shouts a period by default (their "30 seconds" is never given in shouts); Cliff's code over his text (momentum U[0, 0.1]); NYSE rules on for ZI-C and off for ZIP in his presets, as his runs had them; every description and title says what was measured.
- **Copy (verbatim):** model label **Zero-Intelligence Traders**; preset ids `gs-1`–`gs-5`, `gs-1-u`, `gs-4-u`, `cliff-symmetric`, `cliff-flat`, `cliff-excess-demand`, `cliff-excess-supply`, `zip-symmetric`, `zip-flat`, `zip-excess-demand`, `zip-excess-supply`, `zip-demand-shift`, `zip-supply-shift`, `zip-retail`; Compare entries **With vs without the budget constraint — Zero-Intelligence Traders (Compare)** (id `gs-1-vs-u`) and **ZI-C vs ZIP in a box market — Zero-Intelligence Traders (Compare)** (id `zi-c-vs-zip`); color modes **Side**, **Profit**, **Margin**; schema groups **Market**, **Traders**, **Mechanism**, **Periods**, **Stopping**; charts **Prices**, **Efficiency**, **Convergence**, **Profit dispersion**, **Volume**; time axis **Shouts**; sweeps `gs-efficiency`, `gs-dispersion`, `gs-shouts`, `gs-mechanism`, `cliff-prices`, `zip-days`, `zip-momentum`, `zip-shift`; series `price, mean_price, volume, efficiency, rmsd, alpha, dispersion, period, p0, last_price, last_efficiency, last_alpha, last_dispersion, avg_price, avg_efficiency, avg_dispersion`; notice `This run has reached its last period — Reset to run it again`; CLI `(its last period)`.
- Every commit message ends with a blank line and `Claude-Session: https://claude.ai/code/session_01CDy35L1s2fYHpcryzQj8w4`. Stage only the task's files; never `.claude/` or `papers/`.
- Rust: `cargo fmt --all && cargo clippy --all-targets -- -D warnings`. In `survey/`, format only `survey/src/claims/zi.rs`; do not commit `survey/out/results-*.json`.
- Web: `(cd web && npm run build && npm test)` (run `npm ci` and `npm run wasm` first in a fresh worktree).
- **Browser checks are the controller's** (Task 3's Step 6; the full pass in Task 5).

## Review Focus

1. **The book's prices and clearing** — a crossing shout trades at the *standing* order's price; only a better quote replaces the standing one; a trade clears both sides; units trade in order. Pinned in Task 1 by `the_book_trades_at_the_earlier_price_and_clears`.
2. **Cliff's mechanism and NYSE** — ZI counterparts redraw and must strictly beat the shout, ZIP counterparts cross it; the trade is at the shout's price; NYSE bars ZI traders by limit and ZIP traders by price, and resets with each trade; `sellers_only` lets only sellers shout. Pinned by `cliffs_mechanism_trades_at_the_shouts_price_with_a_willing_trader`, `nyse_bars_shouts_that_cannot_beat_the_quote`, `sellers_only_means_sellers_shout`.
3. **Periods** — a period ends after its shouts or (Cliff) after 100 failures in a row or a side with no one able; every trader's units come back, ZIP margins persist, a shift applies from `shift_at`. Pinned by `a_period_ends_after_its_shouts_and_restores_the_units`, `failures_end_a_day`, `a_shift_moves_p0_from_its_period`.
4. **The markets' anchors** — Gode and Sunder's P₀ of 69 and 170, their volumes (the zero-surplus units aside), Table 2's ZI-U efficiencies, Cliff's P₀ 200 and 225. Pinned by `gode_and_sunders_markets_meet_the_texts_anchors`, `cliffs_markets_clear_at_200_and_retail_at_225`.
5. **Portability** — native and WASM fingerprints agree, empty book included. Pinned in Task 2 by `zi_sims_match_the_native_golden_entries`.

## Decisions (where the spec leaves room, or planning changed it)

All code here was implemented in a scratch copy during planning and passed `cargo test --workspace` (1 057), `cargo clippy --all-targets -D warnings`, `wasm-pack test --node crates/sugarscape-wasm` (52), `npm run build && npm test` (733, 49 files) and the survey (16 claims, about a second).

1. **ZIP converges far better than the prototype showed** (amends the spec's planning numbers): within 2 of P₀ by day 10 in all four markets; the momentum readings differ by a day or two.
2. **NYSE off for ZIP** (Cliff's ZIP control file); on for ZI-C.
3. **`sellers_only` under the book** is allowed (nothing trades), since every live field must accept a change on its own.
4. **The fingerprint's empty-book sentinel** is `u64::MAX` (the WASM test caught `usize::MAX`).
5. **Planning's findings** (the survey reproduces them): Tables 1–3 reproduce with 2 000 shouts a period, but efficiency is 44–86 % at 100; Cliff's predictions miss the box markets (138, 250 against 125, 260) and his 233⅓ is not his formula's; his critique holds in Gode and Sunder's own mechanism; ZIP converges, re-converges after shifts, cuts dispersion tenfold, and stays below P₀ in the retail market.

---

### Task 1: The zi model in the core

**Files:**
- Create: `crates/sugarscape-core/src/zi/{config,market,stats,view,world,presets,mod}.rs`
- Modify: `crates/sugarscape-core/src/lib.rs`, `crates/sugarscape-core/src/model.rs`, `crates/sugarscape-core/src/presets.rs`, `crates/sugarscape-core/src/titles.rs`, `crates/sugarscape-core/tests/golden.rs`

**Interfaces:**
- Consumes: `crate::model::{Model, ModelConfig, ModelKind, wrong_model}`, `crate::stats::{Series, Stats}`, `crate::export::history_csv`, `crate::render::{lerp, Rgb}`, `crate::rng::{self, SimRng}`, `crate::schema::{Apply, Param}`, `crate::presets::ModelPreset`, `crate::opinions::Canvas`.
- Produces: `zi::{ZiConfig, Market, Strategy, Mechanism, Turns, PeriodEnd, Momentum, Shift, MAX_FAILS, SHIFT, schema, presets, schedules, schedules_at, equilibrium, equilibrium_profits, Equilibrium, ZiSnapshot, SERIES, GAP, PRICES_W, SCHED, SHOWN, STRIP, TALL, WIDE, Period, Trade, Trader, TraderView, ZiCell, ZiInspection, ZiMode, ZiWorld, Zip}`; `ZiWorld::{new, step, run, run_periods, traders, equilibrium, periods, trades, period_trades, is_finished, inspect}` and `pub tick`; `ModelKind::Zi` (`"zi"`), `ModelConfig::Zi`, `ModelWorld::Zi`; eighteen titles.

- [ ] **Step 1: Write the module**

The tests are in each file (config: defaults, validation, reset fields, the strategies' names, the schema; market: the texts' anchors, Cliff's P₀, shifts, equilibrium profits; view: the panels; world: fifteen, listed in the Review Focus and below).

Create `crates/sugarscape-core/src/zi/config.rs` with exactly this content:

````rust
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
````

Create `crates/sugarscape-core/src/zi/market.rs` with exactly this content:

````rust
//! The markets' schedules — Gode and Sunder's five, read from their figures
//! (the text's anchors: P₀ 69 and 170 in markets 2 and 4, volumes 24 and 6
//! in markets 1 and 3, and Table 2's ZI-U efficiencies, which follow from the
//! schedules alone), and Cliff's five, read from his figures — and their
//! competitive equilibrium.

use super::config::{Market, Shift, ZiConfig, SHIFT};

/// Six traders holding the same units.
fn six(units: &[u32]) -> Vec<Vec<u32>> {
    vec![units.to_vec(); 6]
}

/// One unit each at these limits.
fn one_each(limits: impl IntoIterator<Item = u32>) -> Vec<Vec<u32>> {
    limits.into_iter().map(|v| vec![v]).collect()
}

/// Market 5's staircase: 18 intramarginal units a side, evenly from `from`
/// to `to`, dealt in turn to three traders; three more traders hold seven
/// extramarginal units each at `beyond`.
fn staircase(from: u32, to: u32, beyond: u32) -> Vec<Vec<u32>> {
    let steps: Vec<u32> = (0..18u32)
        .map(|i| {
            let (a, b) = (f64::from(from), f64::from(to));
            (a + (b - a) * f64::from(i) / 17.0).round() as u32
        })
        .collect();
    let mut out: Vec<Vec<u32>> = (0..3)
        .map(|k| steps.iter().skip(k).step_by(3).copied().collect())
        .collect();
    out.extend(vec![vec![beyond; 7]; 3]);
    out
}

/// Each buyer's values and each seller's costs, in trading order, before any shift.
pub fn schedules(c: &ZiConfig) -> (Vec<Vec<u32>>, Vec<Vec<u32>>) {
    match c.market {
        Market::Gs1 => (
            six(&[102, 97, 92, 87, 82, 77]),
            six(&[34, 46, 58, 70, 82, 94]),
        ),
        Market::Gs2 => (
            six(&[117, 105, 93, 81, 69, 57]),
            six(&[49, 54, 59, 64, 69, 74]),
        ),
        Market::Gs3 => (six(&[133, 95, 90]), six(&[90, 95, 100])),
        Market::Gs4 => (
            six(&[180, 175, 170, 165, 160]),
            six(&[90, 142, 170, 190, 198]),
        ),
        Market::Gs5 => (staircase(159, 131, 127), staircase(96, 124, 131)),
        Market::Symmetric => (
            one_each((0..11).map(|i| 75 + 25 * i)),
            one_each((0..11).map(|i| 75 + 25 * i)),
        ),
        Market::FlatSupply => (one_each((0..11).map(|i| 75 + 25 * i)), one_each([200; 11])),
        Market::ExcessDemand => (one_each([200; 11]), one_each([50; 6])),
        Market::ExcessSupply => (one_each([320; 6]), one_each([200; 11])),
        Market::Retail => (
            one_each((0..12).map(|i| 100 + 25 * i)),
            one_each((0..11).map(|i| 75 + 25 * i)),
        ),
        Market::Custom => (c.buyers.clone(), c.sellers.clone()),
    }
}

/// The schedules in force in `period` (1-based): shifted from `shift_at`.
pub fn schedules_at(c: &ZiConfig, period: u64) -> (Vec<Vec<u32>>, Vec<Vec<u32>>) {
    let (mut b, mut s) = schedules(c);
    if period >= u64::from(c.shift_at) {
        match c.shift {
            Shift::None => {}
            Shift::Demand => b.iter_mut().flatten().for_each(|v| *v += SHIFT),
            Shift::Supply => s.iter_mut().flatten().for_each(|v| *v -= SHIFT),
        }
    }
    (b, s)
}

/// The competitive equilibrium of a market.
#[derive(Clone, Debug, PartialEq)]
pub struct Equilibrium {
    /// Units that can trade without loss (value ≥ cost), Cliff's Q₀; Gode
    /// and Sunder's volumes leave out the units whose value equals their cost.
    pub quantity: u32,
    /// The price: the middle of the range that clears `quantity`.
    pub price: f64,
    /// The largest total profit (consumer plus producer surplus).
    pub surplus: i64,
}

pub fn equilibrium(buyers: &[Vec<u32>], sellers: &[Vec<u32>]) -> Equilibrium {
    let mut d: Vec<u32> = buyers.iter().flatten().copied().collect();
    let mut c: Vec<u32> = sellers.iter().flatten().copied().collect();
    d.sort_unstable_by(|a, b| b.cmp(a));
    c.sort_unstable();
    let mut q = 0;
    while q < d.len().min(c.len()) && d[q] >= c[q] {
        q += 1;
    }
    let surplus = (0..q).map(|i| i64::from(d[i]) - i64::from(c[i])).sum();
    let price = if q == 0 {
        (f64::from(d.first().copied().unwrap_or(0)) + f64::from(c.first().copied().unwrap_or(0)))
            / 2.0
    } else {
        let lo = c[q - 1].max(d.get(q).copied().unwrap_or(0));
        let hi = d[q - 1].min(c.get(q).copied().unwrap_or(u32::MAX));
        (f64::from(lo) + f64::from(hi)) / 2.0
    };
    Equilibrium {
        quantity: q as u32,
        price,
        surplus,
    }
}

/// Each trader's profit at the equilibrium price (buyers first).
pub fn equilibrium_profits(buyers: &[Vec<u32>], sellers: &[Vec<u32>], price: f64) -> Vec<f64> {
    buyers
        .iter()
        .map(|t| t.iter().map(|&v| (f64::from(v) - price).max(0.0)).sum())
        .chain(
            sellers
                .iter()
                .map(|t| t.iter().map(|&c| (price - f64::from(c)).max(0.0)).sum()),
        )
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn market(m: Market) -> (Vec<Vec<u32>>, Vec<Vec<u32>>) {
        schedules(&ZiConfig {
            market: m,
            ..ZiConfig::default()
        })
    }

    /// ZI-U trades every unit: its efficiency is the whole schedules' surplus
    /// over the equilibrium's.
    fn all_units(b: &[Vec<u32>], s: &[Vec<u32>]) -> f64 {
        let total: i64 = b.iter().flatten().map(|&v| i64::from(v)).sum::<i64>()
            - s.iter().flatten().map(|&v| i64::from(v)).sum::<i64>();
        100.0 * total as f64 / equilibrium(b, s).surplus as f64
    }

    #[test]
    fn gode_and_sunders_markets_meet_the_texts_anchors() {
        let (b, s) = market(Market::Gs2);
        assert_eq!(equilibrium(&b, &s).price, 69.0);
        let (b, s) = market(Market::Gs4);
        assert_eq!(equilibrium(&b, &s).price, 170.0);
        // Volumes 24 and 6 leave out the zero-surplus units (value = cost).
        let (b, s) = market(Market::Gs1);
        assert_eq!(equilibrium(&b, &s).quantity, 30);
        let (b, s) = market(Market::Gs3);
        assert_eq!(equilibrium(&b, &s).quantity, 12);
        // Table 2's ZI-U efficiencies.
        for (m, want) in [
            (Market::Gs1, 90.0),
            (Market::Gs2, 90.0),
            (Market::Gs3, 76.7),
            (Market::Gs4, 48.8),
        ] {
            let (b, s) = market(m);
            assert!(
                (all_units(&b, &s) - want).abs() < 0.05,
                "{m:?}: {}",
                all_units(&b, &s)
            );
        }
        let (b, s) = market(Market::Gs5);
        assert_eq!((b.len(), s.len()), (6, 6));
        assert_eq!(equilibrium(&b, &s).price, 129.0);
        assert!((all_units(&b, &s) - 86.0).abs() < 1.0);
    }

    #[test]
    fn cliffs_markets_clear_at_200_and_retail_at_225() {
        for m in [
            Market::Symmetric,
            Market::FlatSupply,
            Market::ExcessDemand,
            Market::ExcessSupply,
        ] {
            let (b, s) = market(m);
            let e = equilibrium(&b, &s);
            assert_eq!((e.price, e.quantity), (200.0, 6), "{m:?}");
        }
        let (b, s) = market(Market::Retail);
        assert_eq!((b.len(), s.len()), (12, 11));
        let e = equilibrium(&b, &s);
        assert_eq!((e.price, e.quantity), (225.0, 7));
    }

    #[test]
    fn shifts_move_one_side_by_50() {
        let c = ZiConfig {
            market: Market::Symmetric,
            price_max: 400,
            shift: Shift::Demand,
            ..ZiConfig::default()
        };
        let (b, s) = schedules_at(&c, 11);
        assert_eq!(equilibrium(&b, &s).price, 225.0);
        let (b, s) = schedules_at(&c, 10);
        assert_eq!(equilibrium(&b, &s).price, 200.0);
        let c = ZiConfig {
            shift: Shift::Supply,
            ..c
        };
        let (b, s) = schedules_at(&c, 11);
        assert_eq!(equilibrium(&b, &s).price, 175.0);
    }

    #[test]
    fn equilibrium_profits_sum_to_the_surplus() {
        let (b, s) = market(Market::Gs1);
        let e = equilibrium(&b, &s);
        let p = equilibrium_profits(&b, &s, e.price);
        assert_eq!(p.iter().sum::<f64>() as i64, e.surplus);
    }
}
````

Create `crates/sugarscape-core/src/zi/stats.rs` with exactly this content:

````rust
//! Zero-Intelligence Traders' statistics: this period's trades so far and the
//! last completed period's, with Smith's convergence coefficient α and Gode and
//! Sunder's efficiency and profit dispersion, and averages over the periods.

use serde::Serialize;

use crate::stats::Series;

/// The statistics series, in the order the CSV and the page list them.
pub const SERIES: [&str; 16] = [
    "price",
    "mean_price",
    "volume",
    "efficiency",
    "rmsd",
    "alpha",
    "dispersion",
    "period",
    "p0",
    "last_price",
    "last_efficiency",
    "last_alpha",
    "last_dispersion",
    "avg_price",
    "avg_efficiency",
    "avg_dispersion",
];

/// One shout's statistics.
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct ZiSnapshot {
    pub tick: u64,
    /// This shout's trade price; NaN (null) if it did not trade.
    pub price: f64,
    /// This period so far: the mean price (NaN before a trade), units traded,
    /// profit over the maximum surplus (%), the RMS deviation of prices from
    /// P₀, Smith's α (100 × rmsd / P₀), and the RMS of each trader's profit
    /// minus its equilibrium profit.
    pub mean_price: f64,
    pub volume: u32,
    pub efficiency: f64,
    pub rmsd: f64,
    pub alpha: f64,
    pub dispersion: f64,
    /// The period under way (1-based) and its equilibrium price.
    pub period: u64,
    pub p0: f64,
    /// The last completed period's values (NaN before one completes).
    pub last_price: f64,
    pub last_efficiency: f64,
    pub last_alpha: f64,
    pub last_dispersion: f64,
    /// Means over the completed periods (NaN before one completes).
    pub avg_price: f64,
    pub avg_efficiency: f64,
    pub avg_dispersion: f64,
}

impl Series for ZiSnapshot {
    fn tick(&self) -> u64 {
        self.tick
    }

    fn value(&self, name: &str) -> Option<f64> {
        Some(match name {
            "tick" => self.tick as f64,
            "price" => self.price,
            "mean_price" => self.mean_price,
            "volume" => f64::from(self.volume),
            "efficiency" => self.efficiency,
            "rmsd" => self.rmsd,
            "alpha" => self.alpha,
            "dispersion" => self.dispersion,
            "period" => self.period as f64,
            "p0" => self.p0,
            "last_price" => self.last_price,
            "last_efficiency" => self.last_efficiency,
            "last_alpha" => self.last_alpha,
            "last_dispersion" => self.last_dispersion,
            "avg_price" => self.avg_price,
            "avg_efficiency" => self.avg_efficiency,
            "avg_dispersion" => self.avg_dispersion,
            _ => return None,
        })
    }
}
````

Create `crates/sugarscape-core/src/zi/view.rs` with exactly this content:

````rust
//! The frame: the market's schedules, the trade prices across periods (Gode
//! and Sunder's panels), and a strip of traders.

use crate::render::{lerp, Rgb};

/// The schedules panel's side, the prices panel's width, the gap, and the
/// traders strip's height.
pub const SCHED: usize = 200;
pub const PRICES_W: usize = 600;
pub const GAP: usize = 8;
pub const STRIP: usize = 60;
/// The frame.
pub const WIDE: usize = SCHED + GAP + PRICES_W;
pub const TALL: usize = SCHED + GAP + STRIP;
/// Periods the prices panel shows.
pub const SHOWN: usize = 10;

pub const BUYER: Rgb = [0x4a, 0x7c, 0xd8];
pub const SELLER: Rgb = [0xe0, 0x3c, 0x31];
pub const TRADE: Rgb = [0xf2, 0xc1, 0x4e];
pub const MARK: Rgb = [0xd8, 0xd4, 0xca];
pub const DIM: Rgb = [0x5a, 0x55, 0x4c];
pub const AHEAD: Rgb = [0x3c, 0xa8, 0x5a];
pub const BEHIND: Rgb = [0xf2, 0x9a, 0x3a];
pub const LOW: Rgb = [0x2a, 0x26, 0x20];
pub const HIGH: Rgb = [0x6a, 0xd8, 0xf6];

/// The row of `price` in a panel `tall` pixels high, `price_max` at the top.
pub fn row(price: f64, price_max: u32, tall: usize) -> usize {
    let t = (price / f64::from(price_max)).clamp(0.0, 1.0);
    ((1.0 - t) * (tall - 1) as f64).round() as usize
}

pub fn scale(t: f64, from: Rgb, to: Rgb) -> Rgb {
    lerp(from, to, t.clamp(0.0, 1.0))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn panels_line_up() {
        assert_eq!((WIDE, TALL), (808, 268));
        assert_eq!(
            (row(200.0, 200, SCHED), row(0.0, 200, SCHED)),
            (0, SCHED - 1)
        );
        assert_eq!(row(100.0, 200, 201), 100);
    }
}
````

Create `crates/sugarscape-core/src/zi/world.rs` with exactly this content:

````rust
//! The Zero-Intelligence Traders world. Each tick one trader shouts a price
//! for its next unit: at random (ZI-U over the whole range; ZI-C never at a
//! loss) or at its limit times a learned margin (ZIP). Under Gode and
//! Sunder's book a shout that crosses the standing quote trades at the
//! quote's price; under Cliff's mechanism a random willing trader on the other
//! side takes it at the shout's price. A trade clears the book. When a period
//! ends every trader gets its units back (ZIP margins persist).

use std::collections::VecDeque;
use std::fmt::Write;

use rand::Rng;
use serde::Serialize;

use super::config::{Mechanism, Momentum, PeriodEnd, Strategy, Turns, ZiConfig, MAX_FAILS};
use super::market::{equilibrium, equilibrium_profits, schedules_at, Equilibrium};
use super::stats::ZiSnapshot;
use super::view::{
    row, scale, AHEAD, BEHIND, BUYER, DIM, GAP, HIGH, LOW, MARK, PRICES_W, SCHED, SELLER, SHOWN,
    STRIP, TALL, TRADE, WIDE,
};
use crate::config::FieldError;
use crate::export;
use crate::model::{wrong_model, Model, ModelConfig, ModelKind};
use crate::opinions::Canvas;
use crate::rng::{self, SimRng};
use crate::stats::{Series, Stats};

/// Trades kept for the view.
const KEPT: usize = 4000;

/// A ZIP trader's learning state (Cliff's `agent.c`).
#[derive(Clone, Debug, PartialEq)]
pub struct Zip {
    /// μ: sellers ≥ 0, buyers ≤ 0; price = limit × (1 + μ).
    pub margin: f64,
    pub beta: f64,
    pub gamma: f64,
    /// Γ: the last change (momentum).
    pub last: f64,
}

#[derive(Clone, Debug)]
pub struct Trader {
    pub buyer: bool,
    /// Values (buyers) or costs (sellers), in trading order.
    pub limits: Vec<u32>,
    /// The next unit to trade.
    pub next: usize,
    /// Profit this period.
    pub profit: i64,
    pub zip: Zip,
}

impl Trader {
    fn active(&self) -> bool {
        self.next < self.limits.len()
    }

    /// The limit of the next unit (the last one's once all are traded).
    fn limit(&self) -> u32 {
        self.limits[self.next.min(self.limits.len() - 1)]
    }

    /// A ZIP trader's price, before rounding.
    fn zip_price(&self) -> f64 {
        f64::from(self.limit()) * (1.0 + self.zip.margin)
    }
}

/// A trade.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct Trade {
    pub period: u64,
    pub tick: u64,
    pub price: u32,
    pub buyer: u32,
    pub seller: u32,
    /// The buyer's value and the seller's cost for the units traded.
    pub value: u32,
    pub cost: u32,
}

/// A completed period.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Period {
    pub period: u64,
    pub p0: f64,
    pub volume: u32,
    pub mean_price: f64,
    pub efficiency: f64,
    pub rmsd: f64,
    pub alpha: f64,
    pub dispersion: f64,
}

/// The frame's color modes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ZiMode {
    Side,
    Profit,
    Margin,
}

impl std::str::FromStr for ZiMode {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, String> {
        Ok(match s {
            "side" => Self::Side,
            "profit" => Self::Profit,
            "margin" => Self::Margin,
            _ => return Err(format!("unknown color mode {s:?}")),
        })
    }
}

/// What Inspect shows for a cell.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ZiInspection {
    pub site: ZiCell,
    /// `schedules`, `prices` or `traders`; null between panels.
    pub panel: Option<&'static str>,
    /// Schedules: the unit's rank, its value on the demand curve and its cost
    /// on the supply curve (null past a curve's end).
    pub unit: Option<u32>,
    pub demand: Option<u32>,
    pub supply: Option<u32>,
    /// Prices: the trade nearest the cell.
    pub trade: Option<Trade>,
    /// Traders: the trader.
    pub trader: Option<TraderView>,
    /// Always null: cells are read where they are.
    pub agent: Option<TraderView>,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct ZiCell {
    pub x: u32,
    pub y: u32,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct TraderView {
    pub id: u32,
    pub buyer: bool,
    pub limits: Vec<u32>,
    pub traded: u32,
    pub profit: i64,
    pub equilibrium_profit: f64,
    /// ZIP: the margin μ (null for ZI traders).
    pub margin: Option<f64>,
}

#[derive(Clone)]
pub struct ZiWorld {
    pub config: ZiConfig,
    /// Shouts made.
    pub tick: u64,
    rng: SimRng,
    /// Buyers, then sellers.
    traders: Vec<Trader>,
    buyers: usize,
    /// The period under way (1-based), its shouts and failures in a row.
    period: u64,
    shouts: u32,
    fails: u32,
    /// The standing bid and ask: (price, trader).
    bid: Option<(u32, usize)>,
    ask: Option<(u32, usize)>,
    eq: Equilibrium,
    eq_profits: Vec<f64>,
    /// This period's trades, and the recent ones for the view.
    period_trades: Vec<Trade>,
    trades: VecDeque<Trade>,
    /// Completed periods, and each trader's profit in the last one.
    periods: Vec<Period>,
    last_profits: Vec<i64>,
    pub stats: Stats<ZiSnapshot>,
}

impl ZiWorld {
    pub fn new(config: ZiConfig, seed: u64) -> Result<Self, Vec<FieldError>> {
        config.validate()?;
        let mut rng = rng::seeded(seed);
        let (b, s) = schedules_at(&config, 1);
        let zip = config.strategy == Strategy::Zip;
        let mut traders = Vec::new();
        for (limits, buyer) in b
            .iter()
            .map(|l| (l, true))
            .chain(s.iter().map(|l| (l, false)))
        {
            let z = if zip {
                // Cliff: β ~ U[0.1, 0.5]; μ₀ ~ U[0.05, 0.35] (buyers negative);
                // γ ~ U[0, 0.1] in his code, U[0.2, 0.8] in his text.
                let beta = 0.1 + 0.4 * rng.gen::<f64>();
                let m = 0.05 + 0.3 * rng.gen::<f64>();
                let gamma = match config.momentum {
                    Momentum::Code => 0.1 * rng.gen::<f64>(),
                    Momentum::Text => 0.2 + 0.6 * rng.gen::<f64>(),
                };
                Zip {
                    margin: if buyer { -m } else { m },
                    beta,
                    gamma,
                    last: 0.0,
                }
            } else {
                Zip {
                    margin: 0.0,
                    beta: 0.0,
                    gamma: 0.0,
                    last: 0.0,
                }
            };
            traders.push(Trader {
                buyer,
                limits: limits.clone(),
                next: 0,
                profit: 0,
                zip: z,
            });
        }
        let eq = equilibrium(&b, &s);
        let eq_profits = equilibrium_profits(&b, &s, eq.price);
        let mut world = ZiWorld {
            config,
            tick: 0,
            rng,
            traders,
            buyers: b.len(),
            period: 1,
            shouts: 0,
            fails: 0,
            bid: None,
            ask: None,
            eq,
            eq_profits,
            period_trades: Vec::new(),
            trades: VecDeque::new(),
            periods: Vec::new(),
            last_profits: Vec::new(),
            stats: Stats::default(),
        };
        world.record(None);
        Ok(world)
    }

    pub fn traders(&self) -> &[Trader] {
        &self.traders
    }

    pub fn equilibrium(&self) -> &Equilibrium {
        &self.eq
    }

    /// Completed periods.
    pub fn periods(&self) -> &[Period] {
        &self.periods
    }

    /// The last trades (up to 4 000), oldest first.
    pub fn trades(&self) -> impl Iterator<Item = &Trade> {
        self.trades.iter()
    }

    /// This period's trades.
    pub fn period_trades(&self) -> &[Trade] {
        &self.period_trades
    }

    pub fn is_finished(&self) -> bool {
        self.config.stop_at > 0 && self.periods.len() as u64 >= u64::from(self.config.stop_at)
    }

    pub fn run(&mut self, ticks: u32) {
        for _ in 0..ticks {
            if self.is_finished() {
                break;
            }
            self.step();
        }
    }

    /// Runs until `n` more periods are complete (or the run finishes).
    pub fn run_periods(&mut self, n: u32) {
        let until = self.periods.len() + n as usize;
        while self.periods.len() < until && !self.is_finished() {
            self.step();
        }
    }

    /// The price range of a random shout by trader `i` for its next unit.
    fn draw(&mut self, i: usize) -> u32 {
        let t = &self.traders[i];
        let top = self.config.price_max;
        match self.config.strategy {
            Strategy::ZiU => self.rng.gen_range(1..=top),
            Strategy::ZiC if t.buyer => self.rng.gen_range(1..=t.limit()),
            Strategy::ZiC => self.rng.gen_range(t.limit()..=top),
            Strategy::Zip => (t.zip_price().round() as i64).clamp(1, i64::from(top)) as u32,
        }
    }

    /// Whether trader `i` may shout under Cliff's NYSE rule.
    fn able(&self, i: usize) -> bool {
        let t = &self.traders[i];
        if !t.active() {
            return false;
        }
        if self.config.mechanism != Mechanism::Cliff || !self.config.nyse {
            return true;
        }
        // Cliff: ZI traders are barred by their limit, ZIP traders by their price.
        let own = match self.config.strategy {
            Strategy::Zip => t.zip_price(),
            _ => f64::from(t.limit()),
        };
        if t.buyer {
            self.bid.is_none_or(|(p, _)| own > f64::from(p))
        } else {
            self.ask.is_none_or(|(p, _)| own < f64::from(p))
        }
    }

    /// Who shouts: `None` when no one on the side to shout is able.
    fn choose(&mut self) -> Option<usize> {
        let n = self.traders.len();
        let sellers_only = self.config.sellers_only;
        let side_ok = |t: &Trader| !sellers_only || !t.buyer;
        match self.config.turns {
            Turns::Trader => {
                let pool: Vec<usize> = (0..n)
                    .filter(|&i| side_ok(&self.traders[i]) && self.able(i))
                    .collect();
                if pool.is_empty() {
                    return None;
                }
                Some(pool[self.rng.gen_range(0..pool.len() as u32) as usize])
            }
            Turns::Side => {
                let active_b = self.traders[..self.buyers]
                    .iter()
                    .filter(|t| t.active())
                    .count();
                let active_s = self.traders[self.buyers..]
                    .iter()
                    .filter(|t| t.active())
                    .count();
                if active_b + active_s == 0 {
                    return None;
                }
                let sellers = sellers_only
                    || self.rng.gen::<f64>() < active_s as f64 / (active_b + active_s) as f64;
                let range = if sellers {
                    self.buyers..n
                } else {
                    0..self.buyers
                };
                let pool: Vec<usize> = range.filter(|&i| self.able(i)).collect();
                if pool.is_empty() {
                    return None;
                }
                Some(pool[self.rng.gen_range(0..pool.len() as u32) as usize])
            }
        }
    }

    pub fn step(&mut self) {
        self.tick += 1;
        self.shouts += 1;
        let mut traded = None;
        let mut blocked = false;
        match self.choose() {
            None => blocked = true,
            Some(who) => {
                let price = self.draw(who);
                let buyer = self.traders[who].buyer;
                let counter = match self.config.mechanism {
                    Mechanism::Book => self.book(who, price),
                    Mechanism::Cliff => self.cliff(who, price),
                };
                let q = counter.map_or(price, |(_, p)| p);
                if self.config.strategy == Strategy::Zip {
                    self.learn(buyer, q, counter.is_some());
                }
                if let Some((other, p)) = counter {
                    let (b, s) = if buyer { (who, other) } else { (other, who) };
                    traded = Some(self.trade(b, s, p));
                }
            }
        }
        if traded.is_some() {
            self.fails = 0;
        } else {
            self.fails += 1;
        }
        let over = match self.config.period_end {
            PeriodEnd::Shouts => self.shouts >= self.config.shouts,
            PeriodEnd::Failures => blocked || self.fails >= MAX_FAILS,
        };
        if over {
            self.close_period();
        }
        self.record(traded);
    }

    /// Gode and Sunder's book: the counterparty and price if `price` crosses
    /// the standing quote; otherwise it stands if better.
    fn book(&mut self, who: usize, price: u32) -> Option<(usize, u32)> {
        if self.traders[who].buyer {
            if let Some((ask, seller)) = self.ask {
                if price >= ask {
                    return Some((seller, ask));
                }
            }
            if self.bid.is_none_or(|(b, _)| price > b) {
                self.bid = Some((price, who));
            }
        } else {
            if let Some((bid, buyer)) = self.bid {
                if price <= bid {
                    return Some((buyer, bid));
                }
            }
            if self.ask.is_none_or(|(a, _)| price < a) {
                self.ask = Some((price, who));
            }
        }
        None
    }

    /// Cliff's mechanism: each active trader on the other side decides
    /// whether it is willing (ZI traders draw a fresh price and must strictly
    /// beat the shout; ZIP traders' prices must cross it); a random willing
    /// one takes it at the shout's price.
    fn cliff(&mut self, who: usize, price: u32) -> Option<(usize, u32)> {
        let buyer = self.traders[who].buyer;
        if buyer {
            self.bid =
                Some(self.bid.map_or(
                    (price, who),
                    |(b, w)| if price > b { (price, who) } else { (b, w) },
                ));
        } else {
            self.ask =
                Some(self.ask.map_or(
                    (price, who),
                    |(a, w)| if price < a { (price, who) } else { (a, w) },
                ));
        }
        let others: Vec<usize> = if buyer {
            (self.buyers..self.traders.len()).collect()
        } else {
            (0..self.buyers).collect()
        };
        let mut willing = Vec::new();
        for j in others {
            if !self.traders[j].active() {
                continue;
            }
            let ok = match self.config.strategy {
                Strategy::Zip => {
                    let p = self.traders[j].zip_price().round();
                    if buyer {
                        p <= f64::from(price)
                    } else {
                        p >= f64::from(price)
                    }
                }
                _ => {
                    let d = self.draw(j);
                    if buyer {
                        d < price
                    } else {
                        d > price
                    }
                }
            };
            if ok {
                willing.push(j);
            }
        }
        if willing.is_empty() {
            return None;
        }
        Some((
            willing[self.rng.gen_range(0..willing.len() as u32) as usize],
            price,
        ))
    }

    /// Cliff's ZIP update after a shout at `q` (Cliff's code, run before the
    /// traders who traded are banked).
    fn learn(&mut self, bid: bool, q: u32, accepted: bool) {
        let qf = f64::from(q);
        // A: $0.05 on Cliff's $4 range, scaled to this one.
        let a = 0.05 * f64::from(self.config.price_max) / 4.0;
        for i in 0..self.traders.len() {
            let t = &self.traders[i];
            let p = t.zip_price().round();
            let active = t.active();
            // `Some(true)`: toward a higher price; `Some(false)`: lower.
            let toward = if t.buyer {
                if accepted {
                    if p >= qf {
                        Some(false)
                    } else if !bid && active {
                        Some(true)
                    } else {
                        None
                    }
                } else if bid && active && p <= qf {
                    Some(true)
                } else {
                    None
                }
            } else if accepted {
                if p <= qf {
                    Some(true)
                } else if bid && active {
                    Some(false)
                } else {
                    None
                }
            } else if !bid && active && p >= qf {
                Some(false)
            } else {
                None
            };
            let Some(up) = toward else { continue };
            let (r, add) = (self.rng.gen::<f64>(), self.rng.gen::<f64>());
            let target = if up {
                qf * (1.0 + 0.05 * r) + a * add
            } else {
                qf * (1.0 - 0.05 * r) - a * add
            };
            let t = &mut self.traders[i];
            let change = (1.0 - t.zip.gamma) * t.zip.beta * (target - p) + t.zip.gamma * t.zip.last;
            t.zip.last = change;
            let margin = (p + change) / f64::from(t.limit()) - 1.0;
            if (t.buyer && margin < 0.0) || (!t.buyer && margin > 0.0) {
                t.zip.margin = margin;
            }
        }
    }

    fn trade(&mut self, b: usize, s: usize, price: u32) -> Trade {
        let value = self.traders[b].limit();
        let cost = self.traders[s].limit();
        self.traders[b].profit += i64::from(value) - i64::from(price);
        self.traders[s].profit += i64::from(price) - i64::from(cost);
        self.traders[b].next += 1;
        self.traders[s].next += 1;
        self.bid = None;
        self.ask = None;
        let t = Trade {
            period: self.period,
            tick: self.tick,
            price,
            buyer: b as u32,
            seller: s as u32,
            value,
            cost,
        };
        self.period_trades.push(t);
        if self.trades.len() == KEPT {
            self.trades.pop_front();
        }
        self.trades.push_back(t);
        t
    }

    /// This period's figures so far.
    fn current(&self) -> Period {
        let n = self.period_trades.len();
        let p0 = self.eq.price;
        let (mean_price, rmsd) = if n == 0 {
            (f64::NAN, f64::NAN)
        } else {
            let mean = self
                .period_trades
                .iter()
                .map(|t| f64::from(t.price))
                .sum::<f64>()
                / n as f64;
            let ms = self
                .period_trades
                .iter()
                .map(|t| (f64::from(t.price) - p0) * (f64::from(t.price) - p0))
                .sum::<f64>()
                / n as f64;
            (mean, ms.sqrt())
        };
        let profit: i64 = self.traders.iter().map(|t| t.profit).sum();
        let efficiency = if self.eq.surplus > 0 {
            100.0 * profit as f64 / self.eq.surplus as f64
        } else {
            f64::NAN
        };
        let dispersion = (self
            .traders
            .iter()
            .zip(&self.eq_profits)
            .map(|(t, e)| (t.profit as f64 - e) * (t.profit as f64 - e))
            .sum::<f64>()
            / self.traders.len() as f64)
            .sqrt();
        Period {
            period: self.period,
            p0,
            volume: n as u32,
            mean_price,
            efficiency,
            rmsd,
            alpha: 100.0 * rmsd / p0,
            dispersion,
        }
    }

    fn close_period(&mut self) {
        let done = self.current();
        self.periods.push(done);
        self.last_profits = self.traders.iter().map(|t| t.profit).collect();
        self.period += 1;
        self.shouts = 0;
        self.fails = 0;
        self.bid = None;
        self.ask = None;
        self.period_trades.clear();
        let (b, s) = schedules_at(&self.config, self.period);
        for (t, limits) in self.traders.iter_mut().zip(b.iter().chain(&s)) {
            t.limits = limits.clone();
            t.next = 0;
            t.profit = 0;
        }
        self.eq = equilibrium(&b, &s);
        self.eq_profits = equilibrium_profits(&b, &s, self.eq.price);
    }

    fn record(&mut self, traded: Option<Trade>) {
        let now = self.current();
        let last = self.periods.last();
        let avg = |f: fn(&Period) -> f64| {
            let v: Vec<f64> = self
                .periods
                .iter()
                .map(f)
                .filter(|x| x.is_finite())
                .collect();
            if v.is_empty() {
                f64::NAN
            } else {
                v.iter().sum::<f64>() / v.len() as f64
            }
        };
        self.stats.push(ZiSnapshot {
            tick: self.tick,
            price: traded.map_or(f64::NAN, |t| f64::from(t.price)),
            mean_price: now.mean_price,
            volume: now.volume,
            efficiency: now.efficiency,
            rmsd: now.rmsd,
            alpha: now.alpha,
            dispersion: now.dispersion,
            period: self.period,
            p0: self.eq.price,
            last_price: last.map_or(f64::NAN, |p| p.mean_price),
            last_efficiency: last.map_or(f64::NAN, |p| p.efficiency),
            last_alpha: last.map_or(f64::NAN, |p| p.alpha),
            last_dispersion: last.map_or(f64::NAN, |p| p.dispersion),
            avg_price: avg(|p| p.mean_price),
            avg_efficiency: avg(|p| p.efficiency),
            avg_dispersion: avg(|p| p.dispersion),
        });
    }

    fn view(&self, i: usize) -> TraderView {
        let t = &self.traders[i];
        TraderView {
            id: i as u32 + 1,
            buyer: t.buyer,
            limits: t.limits.clone(),
            traded: t.next as u32,
            profit: t.profit,
            equilibrium_profit: self.eq_profits[i],
            margin: (self.config.strategy == Strategy::Zip).then_some(t.zip.margin),
        }
    }

    /// The demand (descending) and supply (ascending) curves.
    fn curves(&self) -> (Vec<u32>, Vec<u32>) {
        let mut d: Vec<u32> = self.traders[..self.buyers]
            .iter()
            .flat_map(|t| t.limits.iter().copied())
            .collect();
        let mut s: Vec<u32> = self.traders[self.buyers..]
            .iter()
            .flat_map(|t| t.limits.iter().copied())
            .collect();
        d.sort_unstable_by(|a, b| b.cmp(a));
        s.sort_unstable();
        (d, s)
    }

    /// The periods the prices panel shows: the last `SHOWN`, the current one
    /// last (a finished run's last completed one).
    fn shown(&self) -> Vec<u64> {
        let last = if self.is_finished() {
            self.periods.len() as u64
        } else {
            self.period
        };
        let first = last.saturating_sub(SHOWN as u64 - 1).max(1);
        (first..=last).collect()
    }

    fn column_color(&self, mode: ZiMode, i: usize) -> [u8; 3] {
        let t = &self.traders[i];
        match mode {
            ZiMode::Side => {
                if t.buyer {
                    BUYER
                } else {
                    SELLER
                }
            }
            ZiMode::Profit => {
                if t.profit as f64 >= self.eq_profits[i] {
                    AHEAD
                } else {
                    BEHIND
                }
            }
            ZiMode::Margin => scale(t.zip.margin.abs() / 0.5, LOW, HIGH),
        }
    }

    pub fn inspect(&self, x: u32, y: u32) -> Result<ZiInspection, String> {
        let (fw, fh) = Model::size(self);
        if x >= fw || y >= fh {
            return Err(format!("({x}, {y}) is not in the frame"));
        }
        let (cx, cy) = (x as usize, y as usize);
        let mut out = ZiInspection {
            site: ZiCell { x, y },
            panel: None,
            unit: None,
            demand: None,
            supply: None,
            trade: None,
            trader: None,
            agent: None,
        };
        if cy < SCHED && cx < SCHED {
            let (d, s) = self.curves();
            let units = d.len().max(s.len()).max(1);
            let k = cx * units / SCHED;
            out.panel = Some("schedules");
            out.unit = Some(k as u32 + 1);
            out.demand = d.get(k).copied();
            out.supply = s.get(k).copied();
        } else if cy < SCHED && cx >= SCHED + GAP {
            let shown = self.shown();
            let slot = PRICES_W / shown.len();
            let k = ((cx - SCHED - GAP) / slot).min(shown.len() - 1);
            let period = shown[k];
            let trades: Vec<&Trade> = self.trades.iter().filter(|t| t.period == period).collect();
            out.panel = Some("prices");
            if !trades.is_empty() {
                let within = (cx - SCHED - GAP - k * slot) * trades.len() / slot;
                out.trade = Some(*trades[within.min(trades.len() - 1)]);
            }
        } else if cy >= SCHED + GAP {
            let n = self.traders.len();
            let i = (cx * n / WIDE).min(n - 1);
            out.panel = Some("traders");
            out.trader = Some(self.view(i));
        }
        Ok(out)
    }
}

impl Model for ZiWorld {
    fn config(&self) -> ModelConfig {
        ModelConfig::Zi(self.config.clone())
    }

    fn run(&mut self, ticks: u32) {
        ZiWorld::run(self, ticks);
    }

    fn tick(&self) -> u64 {
        self.tick
    }

    fn population(&self) -> usize {
        self.traders.len()
    }

    /// FNV-1a over the tick, the period, the book and every trader's state.
    fn fingerprint(&self) -> u64 {
        let mut h: u64 = 0xcbf2_9ce4_8422_2325;
        let mut eat = |bytes: &[u8]| {
            for &b in bytes {
                h ^= u64::from(b);
                h = h.wrapping_mul(0x0100_0000_01b3);
            }
        };
        eat(&self.tick.to_le_bytes());
        eat(&self.period.to_le_bytes());
        for q in [self.bid, self.ask] {
            // A fixed-width sentinel: usize::MAX differs between native and wasm32.
            let (p, w) = q.map_or((0, u64::MAX), |(p, w)| (p, w as u64));
            eat(&p.to_le_bytes());
            eat(&w.to_le_bytes());
        }
        for t in &self.traders {
            eat(&(t.next as u64).to_le_bytes());
            eat(&t.profit.to_le_bytes());
            eat(&t.zip.margin.to_bits().to_le_bytes());
            eat(&t.zip.last.to_bits().to_le_bytes());
        }
        h
    }

    fn size(&self) -> (u32, u32) {
        (WIDE as u32, TALL as u32)
    }

    fn render(&self, mode: &str, _layer: &str, buf: &mut Vec<u8>) -> Result<(), String> {
        let mode: ZiMode = mode.parse()?;
        let mut c = Canvas { buf, wide: 0 };
        c.clear(WIDE, TALL);
        let top = self.config.price_max;
        // The schedules: demand and supply steps, P₀ dashed.
        let (d, s) = self.curves();
        let units = d.len().max(s.len()).max(1);
        let p0 = row(self.eq.price, top, SCHED);
        for x in (0..SCHED).step_by(4) {
            c.put(x, p0, DIM);
        }
        // Each curve a step line two pixels thick, its risers joined.
        let (mut pd, mut ps): (Option<usize>, Option<usize>) = (None, None);
        for x in 0..SCHED {
            let k = x * units / SCHED;
            for (curve, prev, color) in [(&d, &mut pd, BUYER), (&s, &mut ps, SELLER)] {
                if let Some(&v) = curve.get(k) {
                    let y = row(f64::from(v), top, SCHED);
                    c.column(x, y, prev.unwrap_or(y), color);
                    c.put(x, (y + 1).min(SCHED - 1), color);
                    *prev = Some(y);
                }
            }
        }
        // The prices: each shown period a slot, its trades in order; P₀ a line.
        let shown = self.shown();
        let slot = PRICES_W / shown.len();
        let left = SCHED + GAP;
        for x in 0..PRICES_W {
            c.put(left + x, row(self.eq.price, top, SCHED), MARK);
        }
        for (k, &period) in shown.iter().enumerate() {
            if k > 0 {
                c.column(left + k * slot, 0, SCHED - 1, DIM);
            }
            let trades: Vec<&Trade> = self.trades.iter().filter(|t| t.period == period).collect();
            let n = trades.len().max(1);
            let mut prev: Option<usize> = None;
            for (j, t) in trades.iter().enumerate() {
                let x = left + k * slot + (2 * j + 1) * slot / (2 * n);
                let y = row(f64::from(t.price), top, SCHED);
                c.column(x, y, prev.unwrap_or(y), TRADE);
                prev = Some(y);
            }
        }
        // The traders: a column each, its height this period's profit (a
        // finished run's last period) against a mark at its equilibrium
        // profit; under Margin, its height the margin |μ| (0.5 at the top).
        let n = self.traders.len();
        let finished = self.is_finished() && self.last_profits.len() == n;
        let most = self.eq_profits.iter().copied().fold(1.0, f64::max) * 1.5;
        let base = SCHED + GAP + STRIP - 1;
        for i in 0..n {
            let (x0, x1) = (i * WIDE / n, ((i + 1) * WIDE / n).max(i * WIDE / n + 1));
            let profit = if finished {
                self.last_profits[i]
            } else {
                self.traders[i].profit
            };
            let h = if mode == ZiMode::Margin {
                ((self.traders[i].zip.margin.abs() / 0.5).min(1.0) * (STRIP - 1) as f64).round()
                    as usize
            } else {
                ((profit.max(0) as f64 / most) * (STRIP - 1) as f64).round() as usize
            };
            let mark = ((self.eq_profits[i] / most) * (STRIP - 1) as f64).round() as usize;
            let color = self.column_color(mode, i);
            for x in x0..x1.saturating_sub(1).max(x0 + 1) {
                c.column(x, base - h.min(STRIP - 1), base, color);
                c.put(x, base - mark.min(STRIP - 1), MARK);
            }
        }
        Ok(())
    }

    fn latest_json(&self) -> String {
        serde_json::to_string(&self.stats.latest()).expect("snapshot serializes")
    }

    fn series_names(&self) -> Vec<String> {
        super::SERIES.iter().map(|s| s.to_string()).collect()
    }

    fn series(&self, name: &str) -> Option<Vec<f64>> {
        self.stats.series(name)
    }

    fn latest_value(&self, name: &str) -> Option<f64> {
        self.stats.latest().and_then(|s| s.value(name))
    }

    fn series_csv(&self) -> String {
        export::history_csv(&self.series_names(), self.stats.history())
    }

    fn agents_csv(&self) -> String {
        let mut out = String::from("id,side,limits,traded,profit,equilibrium_profit,margin\n");
        for i in 0..self.traders.len() {
            let v = self.view(i);
            let limits: Vec<String> = v.limits.iter().map(u32::to_string).collect();
            writeln!(
                out,
                "{},{},{},{},{},{},{}",
                v.id,
                if v.buyer { "buyer" } else { "seller" },
                limits.join(" "),
                v.traded,
                v.profit,
                v.equilibrium_profit,
                v.margin.map_or(String::new(), |m| m.to_string())
            )
            .unwrap();
        }
        out
    }

    fn inspect_json(&self, x: u32, y: u32) -> Result<String, String> {
        let inspection = self.inspect(x, y)?;
        Ok(serde_json::to_string(&inspection).expect("inspection serializes"))
    }

    /// A trader's column in the strip.
    fn locate(&self, id: u64) -> Option<(u32, u32)> {
        let i = usize::try_from(id.checked_sub(1)?).ok()?;
        let n = self.traders.len();
        (i < n).then(|| ((i * WIDE / n) as u32, (SCHED + GAP + STRIP / 2) as u32))
    }

    fn set_config(&mut self, next: ModelConfig) -> Result<(), Vec<FieldError>> {
        let ModelConfig::Zi(next) = next else {
            return Err(wrong_model(ModelKind::Zi, &next));
        };
        next.validate()?;
        let changes = self.config.structural_changes(&next);
        if !changes.is_empty() {
            return Err(changes);
        }
        self.config = next;
        Ok(())
    }

    fn finished(&self) -> bool {
        self.is_finished()
    }

    /// Stopped after its last period: a sweep reads it there.
    fn holds_when_finished(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::zi::config::{Market, Shift};

    fn config(edit: impl FnOnce(&mut ZiConfig)) -> ZiConfig {
        let mut c = ZiConfig::default();
        edit(&mut c);
        c
    }

    fn world(edit: impl FnOnce(&mut ZiConfig)) -> ZiWorld {
        ZiWorld::new(config(edit), 1).unwrap()
    }

    fn cliff(c: &mut ZiConfig) {
        c.price_max = 400;
        c.mechanism = Mechanism::Cliff;
        c.turns = Turns::Side;
        c.period_end = PeriodEnd::Failures;
        c.stop_at = 10;
    }

    #[test]
    fn shouts_stay_in_their_ranges() {
        for strategy in [Strategy::ZiU, Strategy::ZiC, Strategy::Zip] {
            let mut w = world(|c| c.strategy = strategy);
            for _ in 0..2000 {
                for i in 0..w.traders.len() {
                    let p = w.draw(i);
                    assert!((1..=200).contains(&p));
                    if strategy == Strategy::ZiC {
                        let t = &w.traders[i];
                        assert!(if t.buyer {
                            p <= t.limit()
                        } else {
                            p >= t.limit()
                        });
                    }
                }
            }
        }
    }

    #[test]
    fn the_book_trades_at_the_earlier_price_and_clears() {
        let mut w = world(|_| {});
        // A seller asks 90; a buyer bidding 95 trades at 90.
        assert_eq!(w.book(6, 90), None);
        assert_eq!(w.ask, Some((90, 6)));
        assert_eq!(w.book(7, 95), None, "a worse ask does not stand");
        assert_eq!(w.ask, Some((90, 6)));
        assert_eq!(w.book(0, 80), None);
        assert_eq!(w.bid, Some((80, 0)));
        assert_eq!(w.book(1, 70), None, "a worse bid does not stand");
        assert_eq!(w.book(2, 95), Some((6, 90)));
        let t = w.trade(2, 6, 90);
        assert_eq!((t.value, t.cost, t.price), (102, 34, 90));
        assert_eq!((w.bid, w.ask), (None, None));
        assert_eq!((w.traders[2].profit, w.traders[6].profit), (12, 56));
        assert_eq!((w.traders[2].next, w.traders[6].next), (1, 1));
        // A seller crossing the standing bid trades at the bid.
        w.book(0, 85);
        assert_eq!(w.book(8, 60), Some((0, 85)));
    }

    #[test]
    fn cliffs_mechanism_trades_at_the_shouts_price_with_a_willing_trader() {
        let mut w = world(|c| {
            cliff(c);
            c.market = Market::ExcessDemand;
        });
        // Every buyer values 200; a seller asking 1 finds all of them willing
        // to take it (ZI-C bids 1–200 draw above 1 almost surely).
        let seller = w.buyers;
        let (b, p) = w.cliff(seller, 1).unwrap_or((0, 0));
        assert!(b < w.buyers && p == 1);
        // An ask at the buyers' value can never be taken (bids ≤ 200, strictly above).
        assert_eq!(w.cliff(seller, 200), None);
    }

    #[test]
    fn nyse_bars_shouts_that_cannot_beat_the_quote() {
        let mut w = world(|c| {
            cliff(c);
            c.market = Market::Symmetric;
        });
        w.bid = Some((300, 10));
        // Buyers with values ≤ 300 may not bid; the one valued 325 may.
        let able: Vec<usize> = (0..w.buyers).filter(|&i| w.able(i)).collect();
        assert_eq!(able, [10]);
        w.config.nyse = false;
        assert!((0..w.buyers).all(|i| w.able(i)));
    }

    #[test]
    fn sellers_only_means_sellers_shout() {
        let mut w = world(|c| {
            cliff(c);
            c.market = Market::Retail;
            c.sellers_only = true;
            c.strategy = Strategy::Zip;
        });
        for _ in 0..200 {
            if let Some(i) = w.choose() {
                assert!(!w.traders[i].buyer);
            }
        }
    }

    #[test]
    fn a_period_ends_after_its_shouts_and_restores_the_units() {
        let mut w = world(|c| c.shouts = 500);
        w.run(499);
        assert_eq!(w.periods.len(), 0);
        let traded = w.traders.iter().map(|t| t.next).sum::<usize>();
        assert!(traded > 0);
        w.run(1);
        assert_eq!(w.periods.len(), 1);
        assert_eq!(w.period, 2);
        assert!(w.traders.iter().all(|t| t.next == 0 && t.profit == 0));
        assert_eq!(
            w.periods[0].volume as usize,
            traded / 2 + usize::from(w.stats.history()[500].price.is_finite())
        );
    }

    #[test]
    fn failures_end_a_day() {
        let mut w = world(|c| {
            cliff(c);
            c.market = Market::Symmetric;
        });
        w.run_periods(1);
        assert_eq!(w.periods.len(), 1);
        assert!(w.periods[0].volume >= 1);
    }

    #[test]
    fn efficiency_reproduces_the_budget_constraints_effect() {
        let mut c = world(|_| {});
        let mut u = world(|c| c.strategy = Strategy::ZiU);
        c.run_periods(6);
        u.run_periods(6);
        let (ec, eu) = (
            c.latest_value("avg_efficiency").unwrap(),
            u.latest_value("avg_efficiency").unwrap(),
        );
        assert!(ec > 97.0, "ZI-C {ec}");
        assert!((eu - 90.0).abs() < 0.01, "ZI-U {eu}: every unit trades");
        assert!(c.is_finished() && u.is_finished());
    }

    #[test]
    fn zip_updates_move_toward_the_target_and_reject_crossings() {
        let mut w = world(|c| {
            cliff(c);
            c.market = Market::Symmetric;
            c.strategy = Strategy::Zip;
        });
        // A seller's margin rises after an accepted shout above its price.
        let s = w.buyers;
        let before = w.traders[s].zip.margin;
        let high = (w.traders[s].zip_price().round() + 40.0) as u32;
        w.learn(true, high, true);
        assert!(w.traders[s].zip.margin > before);
        // A margin never crosses zero: a seller pushed far down keeps a positive margin.
        for _ in 0..50 {
            w.learn(true, 1, true);
        }
        assert!(w.traders.iter().all(|t| if t.buyer {
            t.zip.margin <= 0.0
        } else {
            t.zip.margin >= 0.0
        }));
    }

    #[test]
    fn momentum_follows_the_code_or_the_text() {
        let code = world(|c| {
            cliff(c);
            c.market = Market::Symmetric;
            c.strategy = Strategy::Zip;
        });
        assert!(code
            .traders
            .iter()
            .all(|t| (0.0..0.1).contains(&t.zip.gamma)));
        let text = world(|c| {
            cliff(c);
            c.market = Market::Symmetric;
            c.strategy = Strategy::Zip;
            c.momentum = Momentum::Text;
        });
        assert!(text
            .traders
            .iter()
            .all(|t| (0.2..0.8).contains(&t.zip.gamma)));
        assert!(code
            .traders
            .iter()
            .all(|t| (0.1..0.5).contains(&t.zip.beta)));
    }

    #[test]
    fn a_shift_moves_p0_from_its_period() {
        let mut w = world(|c| {
            cliff(c);
            c.market = Market::Symmetric;
            c.strategy = Strategy::Zip;
            c.shift = Shift::Demand;
            c.shift_at = 3;
            c.stop_at = 4;
        });
        w.run_periods(2);
        assert_eq!(w.eq.price, 225.0);
        assert_eq!(w.periods[1].p0, 200.0);
        assert_eq!(w.traders[0].limits, [125]);
    }

    #[test]
    fn statistics_match_hand_built_trades() {
        let mut w = world(|_| {});
        w.trade(0, 6, 82);
        w.trade(1, 7, 92);
        let p = w.current();
        assert_eq!((p.volume, p.mean_price), (2, 87.0));
        assert!((p.rmsd - (50.0f64).sqrt()).abs() < 1e-12);
        assert!((p.alpha - 100.0 * (50.0f64).sqrt() / 82.0).abs() < 1e-12);
        // Profits: 20 + 10 buyers, 48 + 58 sellers, of a 1020 surplus.
        assert!((p.efficiency - 100.0 * 136.0 / 1020.0).abs() < 1e-9);
    }

    #[test]
    fn the_view_and_inspect_read_schedules_trades_and_traders() {
        let mut w = world(|c| c.shouts = 200);
        w.run(450);
        let mut buf = Vec::new();
        for mode in ["side", "profit", "margin"] {
            w.render(mode, "", &mut buf).unwrap();
        }
        assert!(w.render("wealth", "", &mut buf).is_err());
        assert_eq!(Model::size(&w), (808, 268));
        let s = w.inspect(0, 100).unwrap();
        assert_eq!(
            (s.panel, s.unit, s.demand, s.supply),
            (Some("schedules"), Some(1), Some(102), Some(34))
        );
        let p = w.inspect((SCHED + GAP + 5) as u32, 50).unwrap();
        assert_eq!(p.panel, Some("prices"));
        assert_eq!(p.trade.unwrap().period, 1);
        let t = w.inspect(0, (SCHED + GAP + 10) as u32).unwrap();
        assert_eq!(
            (t.panel, t.trader.as_ref().unwrap().id),
            (Some("traders"), 1)
        );
        assert_eq!(
            Model::locate(&w, 12),
            Some(((11 * WIDE / 12) as u32, (SCHED + GAP + STRIP / 2) as u32))
        );
        assert_eq!(Model::locate(&w, 13), None);
    }

    #[test]
    fn live_edits_apply_and_the_traders_wait_for_reset() {
        let mut w = world(|_| {});
        let mut next = w.config.clone();
        next.shouts = 300;
        next.mechanism = Mechanism::Cliff;
        Model::set_config(&mut w, ModelConfig::Zi(next.clone())).unwrap();
        assert_eq!(w.config.shouts, 300);
        next.strategy = Strategy::Zip;
        assert!(Model::set_config(&mut w, ModelConfig::Zi(next)).is_err());
    }

    #[test]
    fn degenerate_configs_run_without_panicking() {
        for edit in [
            (|c: &mut ZiConfig| {
                c.market = Market::Custom;
                c.buyers = vec![vec![10]];
                c.sellers = vec![vec![20]];
            }) as fn(&mut ZiConfig),
            |c| {
                c.market = Market::Custom;
                c.buyers = vec![vec![50, 50]];
                c.sellers = vec![vec![50]];
                c.strategy = Strategy::Zip;
            },
            |c| {
                cliff(c);
                c.market = Market::Retail;
                c.sellers_only = true;
                c.strategy = Strategy::Zip;
            },
            |c| {
                c.shouts = 1;
                c.turns = Turns::Side;
            },
            |c| {
                cliff(c);
                c.market = Market::ExcessSupply;
                c.nyse = false;
                c.strategy = Strategy::ZiU;
            },
        ] {
            let mut w = world(|c| {
                c.stop_at = 3;
                edit(c);
            });
            w.run(20_000);
            assert!(w.is_finished() || w.tick == 20_000);
        }
    }
}
````

Create `crates/sugarscape-core/src/zi/presets.rs` with exactly this content:

````rust
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
````

Create `crates/sugarscape-core/src/zi/mod.rs` with exactly this content:

````rust
//! Zero-Intelligence Traders (milestone 28): Gode and Sunder, "Allocative
//! Efficiency of Markets with Zero-Intelligence Traders" (JPE 101: 119–137,
//! 1993), with Cliff's critique and ZIP traders, "Minimal-Intelligence Agents
//! for Bargaining Behaviours in Market-Based Environments" (HP Labs
//! HPL-97-91, 1997). See docs/superpowers/specs/2026-09-28-zi-traders-design.md.

mod config;
mod market;
mod presets;
mod stats;
mod view;
mod world;

pub use config::{
    schema, Market, Mechanism, Momentum, PeriodEnd, Shift, Strategy, Turns, ZiConfig, MAX_FAILS,
    SHIFT,
};
pub use market::{equilibrium, equilibrium_profits, schedules, schedules_at, Equilibrium};
pub use presets::presets;
pub use stats::{ZiSnapshot, SERIES};
pub use view::{GAP, PRICES_W, SCHED, SHOWN, STRIP, TALL, WIDE};
pub use world::{Period, Trade, Trader, TraderView, ZiCell, ZiInspection, ZiMode, ZiWorld, Zip};
````

- [ ] **Step 2: Wire the model kind and title its presets**

Modify `crates/sugarscape-core/src/lib.rs` (a unified diff against the branch's base: save it and `git apply` it, or make the edits by hand):

````diff
diff --git a/crates/sugarscape-core/src/lib.rs b/crates/sugarscape-core/src/lib.rs
index a0159e3..80553f8 100644
--- a/crates/sugarscape-core/src/lib.rs
+++ b/crates/sugarscape-core/src/lib.rs
@@ -48,6 +48,7 @@ pub mod tags;
 pub mod thresholds;
 pub mod titles;
 pub mod world;
+pub mod zi;
 
 #[cfg(test)]
 pub(crate) mod testkit;
````

Modify `crates/sugarscape-core/src/model.rs` — every match gains `Zi`; the reader gains its `"zi"` arm; a round-trip test pins it (a unified diff against the branch's base: save it and `git apply` it, or make the edits by hand):

````diff
diff --git a/crates/sugarscape-core/src/model.rs b/crates/sugarscape-core/src/model.rs
index 5973f19..86d7d3f 100644
--- a/crates/sugarscape-core/src/model.rs
+++ b/crates/sugarscape-core/src/model.rs
@@ -29,9 +29,11 @@ use crate::structure::{StructureConfig, StructureWorld};
 use crate::tags::{TagsConfig, TagsWorld};
 use crate::thresholds::{ThresholdsConfig, ThresholdsWorld};
 use crate::world::World;
+use crate::zi::{ZiConfig, ZiWorld};
 use crate::{
     agreement, anasazi, ants, civil, classes, culture, dpd, ethno, export, farol, image, norms,
     opinions, punishment, retirement, ring, schelling, spatial, stats, structure, tags, thresholds,
+    zi,
 };
 
 /// Which model a config or world is.
@@ -59,10 +61,11 @@ pub enum ModelKind {
     Thresholds,
     Retirement,
     Punishment,
+    Zi,
 }
 
 impl ModelKind {
-    pub const ALL: [ModelKind; 21] = [
+    pub const ALL: [ModelKind; 22] = [
         ModelKind::Sugarscape,
         ModelKind::Schelling,
         ModelKind::Ring,
@@ -84,6 +87,7 @@ impl ModelKind {
         ModelKind::Thresholds,
         ModelKind::Retirement,
         ModelKind::Punishment,
+        ModelKind::Zi,
     ];
 
     pub fn as_str(self) -> &'static str {
@@ -109,6 +113,7 @@ impl ModelKind {
             ModelKind::Thresholds => "thresholds",
             ModelKind::Retirement => "retirement",
             ModelKind::Punishment => "punishment",
+            ModelKind::Zi => "zi",
         }
     }
 
@@ -137,6 +142,7 @@ impl ModelKind {
             ModelKind::Thresholds => thresholds::schema(),
             ModelKind::Retirement => retirement::schema(),
             ModelKind::Punishment => punishment::schema(),
+            ModelKind::Zi => zi::schema(),
         }
     }
 }
@@ -171,6 +177,7 @@ pub enum ModelConfig {
     Thresholds(ThresholdsConfig),
     Retirement(RetirementConfig),
     Punishment(PunishmentConfig),
+    Zi(ZiConfig),
 }
 
 /// Another model's config on the wire: its fields and `"model": "<kind>"`.
@@ -197,6 +204,7 @@ enum Tagged<'a> {
     Thresholds(&'a ThresholdsConfig),
     Retirement(&'a RetirementConfig),
     Punishment(&'a PunishmentConfig),
+    Zi(&'a ZiConfig),
 }
 
 impl From<Config> for ModelConfig {
@@ -230,6 +238,7 @@ impl Serialize for ModelConfig {
             ModelConfig::Thresholds(c) => Tagged::Thresholds(c).serialize(s),
             ModelConfig::Retirement(c) => Tagged::Retirement(c).serialize(s),
             ModelConfig::Punishment(c) => Tagged::Punishment(c).serialize(s),
+            ModelConfig::Zi(c) => Tagged::Zi(c).serialize(s),
         }
     }
 }
@@ -258,6 +267,7 @@ impl ModelConfig {
             ModelConfig::Thresholds(_) => ModelKind::Thresholds,
             ModelConfig::Retirement(_) => ModelKind::Retirement,
             ModelConfig::Punishment(_) => ModelKind::Punishment,
+            ModelConfig::Zi(_) => ModelKind::Zi,
         }
     }
 
@@ -355,10 +365,13 @@ impl ModelConfig {
             "punishment" => serde_json::from_value(value)
                 .map(ModelConfig::Punishment)
                 .map_err(|e| FieldError::new("config", e.to_string())),
+            "zi" => serde_json::from_value(value)
+                .map(ModelConfig::Zi)
+                .map_err(|e| FieldError::new("config", e.to_string())),
             _ => Err(FieldError::new(
                 "model",
                 format!(
-                    "unknown model {tag:?} (expected sugarscape, schelling, ring, anasazi, civil, spatial, tags, culture, classes, ethno, opinions, structure, dpd, norms, agreement, image, farol, ants, thresholds, retirement or punishment)"
+                    "unknown model {tag:?} (expected sugarscape, schelling, ring, anasazi, civil, spatial, tags, culture, classes, ethno, opinions, structure, dpd, norms, agreement, image, farol, ants, thresholds, retirement, punishment or zi)"
                 ),
             )),
         }
@@ -387,6 +400,7 @@ impl ModelConfig {
             ModelConfig::Thresholds(c) => c.validate(),
             ModelConfig::Retirement(c) => c.validate(),
             ModelConfig::Punishment(c) => c.validate(),
+            ModelConfig::Zi(c) => c.validate(),
         }
     }
 
@@ -415,6 +429,7 @@ impl ModelConfig {
             ModelConfig::Thresholds(c) => set_path(c, path, value).map(ModelConfig::Thresholds),
             ModelConfig::Retirement(c) => set_path(c, path, value).map(ModelConfig::Retirement),
             ModelConfig::Punishment(c) => set_path(c, path, value).map(ModelConfig::Punishment),
+            ModelConfig::Zi(c) => set_path(c, path, value).map(ModelConfig::Zi),
         }
     }
 
@@ -442,7 +457,8 @@ impl ModelConfig {
             | ModelConfig::Ants(_)
             | ModelConfig::Thresholds(_)
             | ModelConfig::Retirement(_)
-            | ModelConfig::Punishment(_) => None,
+            | ModelConfig::Punishment(_)
+            | ModelConfig::Zi(_) => None,
         }
     }
 
@@ -476,6 +492,7 @@ impl ModelConfig {
             ModelConfig::Punishment(_) => {
                 punishment::SERIES.iter().map(|s| s.to_string()).collect()
             }
+            ModelConfig::Zi(_) => zi::SERIES.iter().map(|s| s.to_string()).collect(),
         }
     }
 }
@@ -666,6 +683,7 @@ pub enum ModelWorld {
     Thresholds(Box<ThresholdsWorld>),
     Retirement(Box<RetirementWorld>),
     Punishment(Box<PunishmentWorld>),
+    Zi(Box<ZiWorld>),
 }
 
 impl ModelWorld {
@@ -718,6 +736,7 @@ impl ModelWorld {
             ModelConfig::Punishment(c) => {
                 ModelWorld::Punishment(Box::new(PunishmentWorld::new(c, seed)?))
             }
+            ModelConfig::Zi(c) => ModelWorld::Zi(Box::new(ZiWorld::new(c, seed)?)),
         })
     }
 
@@ -744,6 +763,7 @@ impl ModelWorld {
             ModelWorld::Thresholds(_) => ModelKind::Thresholds,
             ModelWorld::Retirement(_) => ModelKind::Retirement,
             ModelWorld::Punishment(_) => ModelKind::Punishment,
+            ModelWorld::Zi(_) => ModelKind::Zi,
         }
     }
 
@@ -770,6 +790,7 @@ impl ModelWorld {
             ModelWorld::Thresholds(w) => w.as_ref(),
             ModelWorld::Retirement(w) => w.as_ref(),
             ModelWorld::Punishment(w) => w.as_ref(),
+            ModelWorld::Zi(w) => w.as_ref(),
         }
     }
 
@@ -796,6 +817,7 @@ impl ModelWorld {
             ModelWorld::Thresholds(w) => w.as_mut(),
             ModelWorld::Retirement(w) => w.as_mut(),
             ModelWorld::Punishment(w) => w.as_mut(),
+            ModelWorld::Zi(w) => w.as_mut(),
         }
     }
 
@@ -891,6 +913,7 @@ impl ModelWorld {
             ModelWorld::Thresholds(w) => copy_without_history!(Thresholds, w),
             ModelWorld::Retirement(w) => copy_without_history!(Retirement, w),
             ModelWorld::Punishment(w) => copy_without_history!(Punishment, w),
+            ModelWorld::Zi(w) => copy_without_history!(Zi, w),
             _ => return None,
         };
         Some(Checkpoint { world, tick })
@@ -935,6 +958,9 @@ impl ModelWorld {
             (ModelWorld::Punishment(live), ModelWorld::Punishment(kept)) => {
                 restore_into!(live, kept)
             }
+            (ModelWorld::Zi(live), ModelWorld::Zi(kept)) => {
+                restore_into!(live, kept)
+            }
             _ => return Err("the keyframe is of another model".into()),
         }
         Ok(())
@@ -1359,6 +1385,30 @@ mod tests {
         assert_eq!(w.model().tick(), 0);
     }
 
+    #[test]
+    fn zi_configs_round_trip_with_their_tag() {
+        let c = ModelConfig::from_json(
+            r#"{"model": "zi", "market": "gs4", "strategy": "zi_u", "shouts": 500}"#,
+        )
+        .unwrap();
+        assert_eq!(c.kind(), ModelKind::Zi);
+        let json = serde_json::to_value(&c).unwrap();
+        assert_eq!(
+            (json["model"].as_str(), json["price_max"].as_u64()),
+            (Some("zi"), Some(200))
+        );
+        assert_eq!(ModelConfig::from_value(json).unwrap(), c);
+        assert_eq!(c.series_names()[..2], ["price", "mean_price"]);
+        let e = ModelConfig::from_json(r#"{"model": "zi", "shouts": 0}"#).unwrap_err();
+        assert_eq!(e[0].field, "shouts");
+        let mut w = ModelWorld::new(c, 1).unwrap();
+        assert_eq!(w.kind(), ModelKind::Zi);
+        let cp = w.checkpoint().expect("zi worlds have keyframes");
+        w.model_mut().run(3);
+        w.restore(&cp).unwrap();
+        assert_eq!(w.model().tick(), 0);
+    }
+
     #[test]
     fn only_the_anasazi_finishes() {
         let mut w = ModelWorld::new(
@@ -1403,7 +1453,8 @@ mod tests {
                 "ants",
                 "thresholds",
                 "retirement",
-                "punishment"
+                "punishment",
+                "zi"
             ]
         );
         assert!(ModelKind::Sugarscape.schema().is_empty());
````

Modify `crates/sugarscape-core/src/presets.rs` (a unified diff against the branch's base: save it and `git apply` it, or make the edits by hand):

````diff
diff --git a/crates/sugarscape-core/src/presets.rs b/crates/sugarscape-core/src/presets.rs
index 4037930..d1f8440 100644
--- a/crates/sugarscape-core/src/presets.rs
+++ b/crates/sugarscape-core/src/presets.rs
@@ -947,6 +947,7 @@ pub fn catalog() -> Vec<ModelPreset> {
     out.extend(crate::thresholds::presets());
     out.extend(crate::retirement::presets());
     out.extend(crate::punishment::presets());
+    out.extend(crate::zi::presets());
     out
 }
````

Modify `crates/sugarscape-core/src/titles.rs` — eighteen titles (a unified diff against the branch's base: save it and `git apply` it, or make the edits by hand):

````diff
diff --git a/crates/sugarscape-core/src/titles.rs b/crates/sugarscape-core/src/titles.rs
index 9398580..2aa3cf1 100644
--- a/crates/sugarscape-core/src/titles.rs
+++ b/crates/sugarscape-core/src/titles.rs
@@ -3,7 +3,7 @@
 //! paper (`source`) stay on the preset as its reference.
 
 /// Titles by preset id, in catalog order.
-pub const TITLES: [(&str, &str); 264] = [
+pub const TITLES: [(&str, &str); 282] = [
     (
         "ii-1-instant",
         "Sugar grows back at once: agents climb the best ridges and the poorly endowed starve",
@@ -1021,6 +1021,78 @@ pub const TITLES: [(&str, &str); 264] = [
         "bg-janssen",
         "Janssen's readings of the gaps together",
     ),
+    (
+        "gs-1",
+        "Random traders who never take a loss capture nearly all the surplus",
+    ),
+    (
+        "gs-2",
+        "Market 2: random traders who never lose reach 99.8 % efficiency",
+    ),
+    (
+        "gs-3",
+        "Market 3: six units worth trading, and random traders find them",
+    ),
+    (
+        "gs-4",
+        "Market 4: loss-averse random traders capture 99.5 %; free ones, half",
+    ),
+    (
+        "gs-5",
+        "Market 5: marginal traders crowd equilibrium, and efficiency dips to 97 %",
+    ),
+    (
+        "gs-1-u",
+        "Traders free to lose money: 90 % efficiency and prices everywhere",
+    ),
+    (
+        "gs-4-u",
+        "Free to lose money in market 4, traders capture less than half",
+    ),
+    (
+        "cliff-symmetric",
+        "Symmetric supply and demand: random prices center on equilibrium",
+    ),
+    (
+        "cliff-flat",
+        "Flat supply: random prices settle well above equilibrium",
+    ),
+    (
+        "cliff-excess-demand",
+        "Too many buyers: random prices sit far below equilibrium",
+    ),
+    (
+        "cliff-excess-supply",
+        "Too many sellers: random prices sit far above equilibrium",
+    ),
+    (
+        "zip-symmetric",
+        "Traders who learn a margin converge on the equilibrium price",
+    ),
+    (
+        "zip-flat",
+        "Learning traders reach equilibrium within days when supply is flat",
+    ),
+    (
+        "zip-excess-demand",
+        "Where buyers abound, learning traders climb to equilibrium from below",
+    ),
+    (
+        "zip-excess-supply",
+        "Where sellers abound, learning traders fall to equilibrium from above",
+    ),
+    (
+        "zip-demand-shift",
+        "Demand jumps after day 10, and learning traders follow it",
+    ),
+    (
+        "zip-supply-shift",
+        "Supply drops after day 10, and learning traders follow it",
+    ),
+    (
+        "zip-retail",
+        "Only sellers post prices, and trades stay below equilibrium",
+    ),
 ];
 
 /// The title of preset `id`, or "" if it has none.
````

- [ ] **Step 3: Run the model's tests**

Run: `cargo test -p sugarscape-core --lib zi::`
Expected: PASS (80, counting the other modules' tests whose names contain `zi::`).

- [ ] **Step 4: Watch the golden check fail, then record the entries**

Run: `cargo test --release -p sugarscape-core --test golden`
Expected: FAIL with `record a golden fingerprint for gs-1 (run print_golden)`.

Modify `crates/sugarscape-core/tests/golden.rs` (a unified diff against the branch's base: save it and `git apply` it, or make the edits by hand):

````diff
diff --git a/crates/sugarscape-core/tests/golden.rs b/crates/sugarscape-core/tests/golden.rs
index 283d16e..8e10b1e 100644
--- a/crates/sugarscape-core/tests/golden.rs
+++ b/crates/sugarscape-core/tests/golden.rs
@@ -282,6 +282,24 @@ const MODEL_GOLDEN: &[(&str, u64)] = &[
     ("bg-continuous", 0xf763b0b75f9a2298),
     ("bg-ring", 0xc4e75af13ebdeaed),
     ("bg-janssen", 0xe95bb859afd7e9b0),
+    ("gs-1", 0xe38d85243d149ae6),
+    ("gs-2", 0xd40f4b475f30e6fe),
+    ("gs-3", 0xd27a97984854b84e),
+    ("gs-4", 0x3c72fdcb2726a9e9),
+    ("gs-5", 0xdbd5cfd66c153ac1),
+    ("gs-1-u", 0x298dee37f6ba4950),
+    ("gs-4-u", 0xeaabdfdb9910b213),
+    ("cliff-symmetric", 0x7276d578521ce1d9),
+    ("cliff-flat", 0x20cc4c9d08469e1f),
+    ("cliff-excess-demand", 0xba8efd62a0b5f064),
+    ("cliff-excess-supply", 0x4b3767b49bf5430f),
+    ("zip-symmetric", 0x410e3a0938f8f88f),
+    ("zip-flat", 0xfee1bbb2ae8822b8),
+    ("zip-excess-demand", 0x677145266ced0690),
+    ("zip-excess-supply", 0x19e76ebc146f86e1),
+    ("zip-demand-shift", 0x410e3a0938f8f88f),
+    ("zip-supply-shift", 0x410e3a0938f8f88f),
+    ("zip-retail", 0x5925a52d82bfec49),
 ];
 
 fn fingerprint(id: &str) -> u64 {
````

Run: `cargo test --release -p sugarscape-core --test golden`
Expected: PASS.

- [ ] **Step 5: Format, lint, run everything, commit**

```bash
cargo fmt --all && cargo clippy --all-targets -- -D warnings && cargo test --workspace
git add crates/sugarscape-core/src/zi crates/sugarscape-core/src/lib.rs crates/sugarscape-core/src/model.rs crates/sugarscape-core/src/presets.rs crates/sugarscape-core/src/titles.rs crates/sugarscape-core/tests/golden.rs
```
```bash
git commit -m "Add Zero-Intelligence Traders (Gode & Sunder, with Cliff's ZIP) as a model kind

Claude-Session: https://claude.ai/code/session_01CDy35L1s2fYHpcryzQj8w4"
```

---

### Task 2: Sweeps, the CLI and WASM

**Files:**
- Create: `sweeps/{gs-efficiency,gs-dispersion,gs-shouts,gs-mechanism,cliff-prices,zip-days,zip-momentum,zip-shift}.json`
- Modify: `crates/sugarscape-core/src/sweep.rs`, `crates/sugarscape-cli/src/main.rs`, `crates/sugarscape-cli/tests/cli.rs`, `crates/sugarscape-wasm/tests/web.rs`

**Interfaces:**
- Consumes: Task 1's presets and series (`avg_efficiency`, `avg_dispersion`, `avg_price`, `last_price`).
- Produces: eight built-in sweeps; the CLI's stop `(its last period)`.

- [ ] **Step 1: Write the failing tests**

Modify `crates/sugarscape-cli/tests/cli.rs` (a unified diff against the branch's base: save it and `git apply` it, or make the edits by hand):

````diff
diff --git a/crates/sugarscape-cli/tests/cli.rs b/crates/sugarscape-cli/tests/cli.rs
index 1644894..173ad89 100644
--- a/crates/sugarscape-cli/tests/cli.rs
+++ b/crates/sugarscape-cli/tests/cli.rs
@@ -170,6 +170,14 @@ fn presets_and_sweeps_are_listed() {
         "bg-ring",
         "bg-cooney-fine",
         "bg-cooney-cost",
+        "gs-efficiency",
+        "gs-dispersion",
+        "gs-shouts",
+        "gs-mechanism",
+        "cliff-prices",
+        "zip-days",
+        "zip-momentum",
+        "zip-shift",
     ] {
         assert!(
             text.lines().any(|l| l.starts_with(&format!("{id}\t"))),
@@ -593,6 +601,22 @@ fn a_thresholds_run_stops_at_its_last_step() {
     assert_eq!(stderr(&out), "finished at tick 25 (its last step)\n");
 }
 
+#[test]
+fn a_zi_run_stops_at_its_last_period() {
+    let dir = scratch("zi");
+    let config = dir.join("zi.json");
+    std::fs::write(&config, r#"{"model": "zi", "shouts": 100, "stop_at": 3}"#).unwrap();
+    let out = sugarscape(&[
+        "run",
+        "--config",
+        config.to_str().unwrap(),
+        "--ticks",
+        "1000",
+    ]);
+    assert!(out.status.success(), "{}", stderr(&out));
+    assert_eq!(stderr(&out), "finished at tick 300 (its last period)\n");
+}
+
 #[test]
 fn a_punishment_run_stops_at_its_last_period() {
     let dir = scratch("punishment");
````

Modify `crates/sugarscape-wasm/tests/web.rs` (a unified diff against the branch's base: save it and `git apply` it, or make the edits by hand):

````diff
diff --git a/crates/sugarscape-wasm/tests/web.rs b/crates/sugarscape-wasm/tests/web.rs
index 9ae51c6..24c4315 100644
--- a/crates/sugarscape-wasm/tests/web.rs
+++ b/crates/sugarscape-wasm/tests/web.rs
@@ -380,7 +380,15 @@ fn builtins_and_series_names_are_listed() {
             "bg-continuous",
             "bg-ring",
             "bg-cooney-fine",
-            "bg-cooney-cost"
+            "bg-cooney-cost",
+            "gs-efficiency",
+            "gs-dispersion",
+            "gs-shouts",
+            "gs-mechanism",
+            "cliff-prices",
+            "zip-days",
+            "zip-momentum",
+            "zip-shift"
         ]
     );
     assert!(list[0]["sweep"]["name"]
@@ -1031,6 +1039,25 @@ fn punishment_sims_match_the_native_golden_entries() {
     assert_eq!(sim.fingerprint(), "0x41e7fa5fab5ba690");
 }
 
+#[wasm_bindgen_test]
+fn zi_sims_match_the_native_golden_entries() {
+    // crates/sugarscape-core/tests/golden.rs, MODEL_GOLDEN: the book with
+    // ZI-C and ZI-U, market 5, Cliff's mechanism, and ZIP's margins (f64).
+    for (id, fp) in [
+        ("gs-1", "0xe38d85243d149ae6"),
+        ("gs-4-u", "0xeaabdfdb9910b213"),
+        ("gs-5", "0xdbd5cfd66c153ac1"),
+        ("cliff-excess-demand", "0xba8efd62a0b5f064"),
+        ("zip-symmetric", "0x410e3a0938f8f88f"),
+        ("zip-retail", "0x5925a52d82bfec49"),
+    ] {
+        let mut sim = Sim::new(&preset_json(id), 1, JsValue::NULL).unwrap();
+        assert_eq!(sim.model_kind(), "zi");
+        sim.step(200);
+        assert_eq!(sim.fingerprint(), fp, "{id}");
+    }
+}
+
 #[wasm_bindgen_test]
 fn dpd_sims_match_the_native_golden_entries() {
     // crates/sugarscape-core/tests/golden.rs, MODEL_GOLDEN: wealth sums and
````

Run: `cargo test -p sugarscape-cli`
Expected: FAIL (`a_zi_run_stops_at_its_last_period`: `(its end year)`; `presets_and_sweeps_are_listed`: `gs-efficiency` missing).

- [ ] **Step 2: Write the sweeps, register them and name the stop**

Create `sweeps/gs-efficiency.json` with exactly this content:

````json
{
  "name": "Zero-Intelligence Traders: efficiency by market (Table 2)",
  "description": "Gode and Sunder's Table 2: efficiency (profit earned over the maximum surplus, averaged over six periods of 2 000 shouts) by market, with and without the budget constraint. Measured (release, seeds 1–20, recorded 2026-09-28): ZI-U 90.0, 90.0, 76.7, 48.8, 86.7 (Table 2: 90.0, 90.0, 76.7, 48.8, 86.0 — exact in markets 1–4, which were read from the figures with it; market 5 is approximate); ZI-C 99.9, 99.8, 99.7, 99.5, 97.1 (Table 2: 99.9, 99.2, 99.0, 98.2, 97.1). The budget constraint alone lifts efficiency to near 100 %, as stated.",
  "base": {
    "preset": "gs-1"
  },
  "x": {
    "label": "Market",
    "values": [
      {
        "at": 1,
        "name": "Market 1",
        "set": {
          "market": "gs1"
        }
      },
      {
        "at": 2,
        "name": "Market 2",
        "set": {
          "market": "gs2"
        }
      },
      {
        "at": 3,
        "name": "Market 3",
        "set": {
          "market": "gs3"
        }
      },
      {
        "at": 4,
        "name": "Market 4",
        "set": {
          "market": "gs4"
        }
      },
      {
        "at": 5,
        "name": "Market 5",
        "set": {
          "market": "gs5"
        }
      }
    ]
  },
  "series": {
    "label": "Traders",
    "values": [
      {
        "at": 0,
        "name": "ZI-U",
        "set": {
          "strategy": "zi_u"
        }
      },
      {
        "at": 1,
        "name": "ZI-C",
        "set": {
          "strategy": "zi_c"
        }
      }
    ]
  },
  "seeds": {
    "from": 1,
    "count": 20
  },
  "ticks": 12000,
  "metric": {
    "kind": "final",
    "series": "avg_efficiency"
  }
}
````

Create `sweeps/gs-dispersion.json` with exactly this content:

````json
{
  "name": "Zero-Intelligence Traders: profit dispersion by market (Table 3)",
  "description": "Gode and Sunder's Table 3: profit dispersion (the RMS of each trader's profit minus its equilibrium profit) by market. Measured (release, seeds 1–20, recorded 2026-09-28): ZI-U 177.5, 240.6, 102.1, 384.2, 234.0 (Table 3: 225.5, 253.1, 90.5, 363.8, 156.3); ZI-C 31.8, 49.0, 18.0, 64.8, 23.7 (Table 3: 28.5, 49.8, 15.9, 60.5, 19.1) — ZI-C close in every market, ZI-U the same order.",
  "base": {
    "preset": "gs-1"
  },
  "x": {
    "label": "Market",
    "values": [
      {
        "at": 1,
        "name": "Market 1",
        "set": {
          "market": "gs1"
        }
      },
      {
        "at": 2,
        "name": "Market 2",
        "set": {
          "market": "gs2"
        }
      },
      {
        "at": 3,
        "name": "Market 3",
        "set": {
          "market": "gs3"
        }
      },
      {
        "at": 4,
        "name": "Market 4",
        "set": {
          "market": "gs4"
        }
      },
      {
        "at": 5,
        "name": "Market 5",
        "set": {
          "market": "gs5"
        }
      }
    ]
  },
  "series": {
    "label": "Traders",
    "values": [
      {
        "at": 0,
        "name": "ZI-U",
        "set": {
          "strategy": "zi_u"
        }
      },
      {
        "at": 1,
        "name": "ZI-C",
        "set": {
          "strategy": "zi_c"
        }
      }
    ]
  },
  "seeds": {
    "from": 1,
    "count": 20
  },
  "ticks": 12000,
  "metric": {
    "kind": "final",
    "series": "avg_dispersion"
  }
}
````

Create `sweeps/gs-shouts.json` with exactly this content:

````json
{
  "name": "Zero-Intelligence Traders: efficiency against the period's length",
  "description": "Gode and Sunder ran 'six periods of … 30 seconds for machine traders' and never say how many shouts that was. ZI-C efficiency against shouts a period. Measured (release, seeds 1–10, recorded 2026-09-28): market 1: 18, 34, 58, 82, 98.5, 99.9, 99.9 at 25, 50, 100, 200, 500, 1 000, 2 000; market 4: 39, 64, 86, 97, 99.0, 99.4, 99.5; market 5: 12, 23, 44, 68, 92, 97, 97. Their near-100 % efficiency needs at least about 500–1 000 shouts a period: the traders capture the surplus because they get enough chances.",
  "base": {
    "preset": "gs-1"
  },
  "x": {
    "label": "Shouts a period",
    "path": "shouts",
    "values": [
      25,
      50,
      100,
      200,
      500,
      1000,
      2000
    ]
  },
  "series": {
    "label": "Market",
    "values": [
      {
        "at": 1,
        "name": "Market 1",
        "set": {
          "market": "gs1"
        }
      },
      {
        "at": 2,
        "name": "Market 2",
        "set": {
          "market": "gs2"
        }
      },
      {
        "at": 3,
        "name": "Market 3",
        "set": {
          "market": "gs3"
        }
      },
      {
        "at": 4,
        "name": "Market 4",
        "set": {
          "market": "gs4"
        }
      },
      {
        "at": 5,
        "name": "Market 5",
        "set": {
          "market": "gs5"
        }
      }
    ]
  },
  "seeds": {
    "from": 1,
    "count": 10
  },
  "ticks": 12000,
  "metric": {
    "kind": "final",
    "series": "avg_efficiency"
  }
}
````

Create `sweeps/gs-mechanism.json` with exactly this content:

````json
{
  "name": "Zero-Intelligence Traders: Gode and Sunder's book against Cliff's mechanism",
  "description": "Gode and Sunder's book (a crossing shout trades at the standing order's price) against Cliff's mechanism (a random willing trader on the other side takes the shout at its price), ZI-C. Measured (release, seeds 1–20, recorded 2026-09-28): book 99.9, 99.8, 99.7, 99.5, 97.1; Cliff's 99.4, 99.5, 98.2, 99.2, 96.7 in markets 1–5. Efficiency survives the change of mechanism.",
  "base": {
    "preset": "gs-1"
  },
  "x": {
    "label": "Market",
    "values": [
      {
        "at": 1,
        "name": "Market 1",
        "set": {
          "market": "gs1"
        }
      },
      {
        "at": 2,
        "name": "Market 2",
        "set": {
          "market": "gs2"
        }
      },
      {
        "at": 3,
        "name": "Market 3",
        "set": {
          "market": "gs3"
        }
      },
      {
        "at": 4,
        "name": "Market 4",
        "set": {
          "market": "gs4"
        }
      },
      {
        "at": 5,
        "name": "Market 5",
        "set": {
          "market": "gs5"
        }
      }
    ]
  },
  "series": {
    "label": "Mechanism",
    "values": [
      {
        "at": 0,
        "name": "Book (Gode & Sunder)",
        "set": {
          "mechanism": "book"
        }
      },
      {
        "at": 1,
        "name": "Cliff's",
        "set": {
          "mechanism": "cliff"
        }
      }
    ]
  },
  "seeds": {
    "from": 1,
    "count": 20
  },
  "ticks": 12000,
  "metric": {
    "kind": "final",
    "series": "avg_efficiency"
  }
}
````

Create `sweeps/cliff-prices.json` with exactly this content:

````json
{
  "name": "Zero-Intelligence Traders: ZI-C mean prices in Cliff's markets",
  "description": "Cliff's critique: ZI-C mean prices in his four markets (P₀ 200 cents), in his simulator and in Gode and Sunder's book. Measured (release, seeds 1–50, 10 days, recorded 2026-09-28): his mechanism 200.6, 235.8, 138.1, 250.3; the book 200.6, 216.6, 161.8, 232.9 (symmetric, flat supply, excess demand, excess supply). His predictions — 200, 233⅓, 125, 260 — hold for the symmetric and flat markets in his simulator but miss the box markets by 13 and 10; his 233⅓ is not his formula's (241⅔). In Gode and Sunder's own mechanism the prices sit about half as far from P₀, but still far: the critique's direction holds.",
  "base": {
    "preset": "cliff-symmetric"
  },
  "x": {
    "label": "Market",
    "values": [
      {
        "at": 1,
        "name": "Symmetric",
        "set": {
          "market": "symmetric"
        }
      },
      {
        "at": 2,
        "name": "Flat supply",
        "set": {
          "market": "flat_supply"
        }
      },
      {
        "at": 3,
        "name": "Excess demand",
        "set": {
          "market": "excess_demand"
        }
      },
      {
        "at": 4,
        "name": "Excess supply",
        "set": {
          "market": "excess_supply"
        }
      }
    ]
  },
  "series": {
    "label": "Mechanism",
    "values": [
      {
        "at": 0,
        "name": "Cliff's mechanism",
        "set": {}
      },
      {
        "at": 1,
        "name": "Gode & Sunder's book",
        "set": {
          "mechanism": "book",
          "turns": "trader",
          "period_end": "shouts",
          "shouts": 2000
        }
      }
    ]
  },
  "seeds": {
    "from": 1,
    "count": 50
  },
  "ticks": 40000,
  "metric": {
    "kind": "final",
    "series": "avg_price"
  }
}
````

Create `sweeps/zip-days.json` with exactly this content:

````json
{
  "name": "Zero-Intelligence Traders: ZIP's daily mean price",
  "description": "ZIP traders' mean price on each of ten days in Cliff's four markets. Measured (release, seeds 1–50, recorded 2026-09-28): symmetric 184, 191, 194, 196, 198, 199, 199, 200, 200, 200; flat supply 225, 208, 204, 202, 202, 201 …; excess demand 124, 144, 167, 182, 188, 194, 196, 197, 198, 198; excess supply 244, 225, 213, 208, 204, 203, 202 … 202. All converge on P₀ 200 — the box markets from below and above, within a few days more than the others.",
  "base": {
    "preset": "zip-symmetric"
  },
  "x": {
    "label": "Day",
    "path": "stop_at",
    "values": [
      1,
      2,
      3,
      4,
      5,
      6,
      7,
      8,
      9,
      10
    ]
  },
  "series": {
    "label": "Market",
    "values": [
      {
        "at": 1,
        "name": "Symmetric",
        "set": {
          "market": "symmetric"
        }
      },
      {
        "at": 2,
        "name": "Flat supply",
        "set": {
          "market": "flat_supply"
        }
      },
      {
        "at": 3,
        "name": "Excess demand",
        "set": {
          "market": "excess_demand"
        }
      },
      {
        "at": 4,
        "name": "Excess supply",
        "set": {
          "market": "excess_supply"
        }
      }
    ]
  },
  "seeds": {
    "from": 1,
    "count": 50
  },
  "ticks": 100000,
  "metric": {
    "kind": "final",
    "series": "last_price"
  }
}
````

Create `sweeps/zip-momentum.json` with exactly this content:

````json
{
  "name": "Zero-Intelligence Traders: ZIP's momentum, Cliff's code against his text",
  "description": "Cliff's text draws ZIP's momentum γ from U[0.2, 0.8]; his code overwrites it with U[0, 0.1], which produced his results. Measured (release, seeds 1–50, recorded 2026-09-28), excess demand: code 124, 144, 167, 182, 188 … 198 by day 10; text 127, 156, 179, 189, 195 … 199. Excess supply: code 244, 225, 213, 208, 204 … 202; text 242, 224, 209, 204, 201 … 201. The text's reading converges a day or two sooner; both reach P₀.",
  "base": {
    "preset": "zip-excess-demand"
  },
  "x": {
    "label": "Day",
    "path": "stop_at",
    "values": [
      1,
      2,
      3,
      4,
      5,
      6,
      7,
      8,
      9,
      10,
      11,
      12,
      13,
      14,
      15,
      16,
      17,
      18,
      19,
      20
    ]
  },
  "series": {
    "label": "Market and momentum",
    "values": [
      {
        "at": 1,
        "name": "Excess demand, code",
        "set": {}
      },
      {
        "at": 2,
        "name": "Excess demand, text",
        "set": {
          "momentum": "text"
        }
      },
      {
        "at": 3,
        "name": "Excess supply, code",
        "set": {
          "market": "excess_supply"
        }
      },
      {
        "at": 4,
        "name": "Excess supply, text",
        "set": {
          "market": "excess_supply",
          "momentum": "text"
        }
      }
    ]
  },
  "seeds": {
    "from": 1,
    "count": 50
  },
  "ticks": 100000,
  "metric": {
    "kind": "final",
    "series": "last_price"
  }
}
````

Create `sweeps/zip-shift.json` with exactly this content:

````json
{
  "name": "Zero-Intelligence Traders: ZIP after a shift of demand or supply",
  "description": "ZIP traders in the symmetric market, shifted after day 10: demand up 50 cents (P₀ 225) or supply down 50 (P₀ 175). Measured (release, seeds 1–50, recorded 2026-09-28): demand 200 on day 10, then 221, 222, 223, 224, 225, 226, 225, 225, 225, 225; supply 161, 168, 170, 172, 173, 174, 175, 175, 175, 175. They re-converge within about five days, as Cliff shows.",
  "base": {
    "preset": "zip-demand-shift"
  },
  "x": {
    "label": "Day",
    "path": "stop_at",
    "values": [
      1,
      2,
      3,
      4,
      5,
      6,
      7,
      8,
      9,
      10,
      11,
      12,
      13,
      14,
      15,
      16,
      17,
      18,
      19,
      20
    ]
  },
  "series": {
    "label": "Shift",
    "values": [
      {
        "at": 1,
        "name": "Demand up $0.50",
        "set": {}
      },
      {
        "at": 2,
        "name": "Supply down $0.50",
        "set": {
          "shift": "supply"
        }
      }
    ]
  },
  "seeds": {
    "from": 1,
    "count": 50
  },
  "ticks": 100000,
  "metric": {
    "kind": "final",
    "series": "last_price"
  }
}
````

Modify `crates/sugarscape-core/src/sweep.rs` (a unified diff against the branch's base: save it and `git apply` it, or make the edits by hand):

````diff
diff --git a/crates/sugarscape-core/src/sweep.rs b/crates/sugarscape-core/src/sweep.rs
index 2b19752..a4679fd 100644
--- a/crates/sugarscape-core/src/sweep.rs
+++ b/crates/sugarscape-core/src/sweep.rs
@@ -965,7 +965,7 @@ pub struct Builtin {
     pub json: &'static str,
 }
 
-const BUILTINS: [Builtin; 126] = [
+const BUILTINS: [Builtin; 134] = [
     Builtin {
         id: "fig-ii-5",
         json: include_str!("../../../sweeps/fig-ii-5.json"),
@@ -1470,6 +1470,38 @@ const BUILTINS: [Builtin; 126] = [
         id: "bg-cooney-cost",
         json: include_str!("../../../sweeps/bg-cooney-cost.json"),
     },
+    Builtin {
+        id: "gs-efficiency",
+        json: include_str!("../../../sweeps/gs-efficiency.json"),
+    },
+    Builtin {
+        id: "gs-dispersion",
+        json: include_str!("../../../sweeps/gs-dispersion.json"),
+    },
+    Builtin {
+        id: "gs-shouts",
+        json: include_str!("../../../sweeps/gs-shouts.json"),
+    },
+    Builtin {
+        id: "gs-mechanism",
+        json: include_str!("../../../sweeps/gs-mechanism.json"),
+    },
+    Builtin {
+        id: "cliff-prices",
+        json: include_str!("../../../sweeps/cliff-prices.json"),
+    },
+    Builtin {
+        id: "zip-days",
+        json: include_str!("../../../sweeps/zip-days.json"),
+    },
+    Builtin {
+        id: "zip-momentum",
+        json: include_str!("../../../sweeps/zip-momentum.json"),
+    },
+    Builtin {
+        id: "zip-shift",
+        json: include_str!("../../../sweeps/zip-shift.json"),
+    },
 ];
 
 /// The built-in sweeps, in display order.
@@ -2418,7 +2450,15 @@ mod tests {
                 "bg-continuous",
                 "bg-ring",
                 "bg-cooney-fine",
-                "bg-cooney-cost"
+                "bg-cooney-cost",
+                "gs-efficiency",
+                "gs-dispersion",
+                "gs-shouts",
+                "gs-mechanism",
+                "cliff-prices",
+                "zip-days",
+                "zip-momentum",
+                "zip-shift"
             ]
         );
         for b in builtins() {
````

Modify `crates/sugarscape-cli/src/main.rs` (a unified diff against the branch's base: save it and `git apply` it, or make the edits by hand):

````diff
diff --git a/crates/sugarscape-cli/src/main.rs b/crates/sugarscape-cli/src/main.rs
index 8b6764f..e4afcf7 100644
--- a/crates/sugarscape-cli/src/main.rs
+++ b/crates/sugarscape-cli/src/main.rs
@@ -236,7 +236,7 @@ fn run_world(args: RunArgs) -> Result<(), Failure> {
             ModelKind::Image => "its last generation",
             ModelKind::Farol => "its last round",
             ModelKind::Ants | ModelKind::Thresholds => "its last step",
-            ModelKind::Punishment => "its last period",
+            ModelKind::Punishment | ModelKind::Zi => "its last period",
             ModelKind::Retirement => match &config {
                 ModelConfig::Retirement(c)
                     if c.stop_at_norm
````

- [ ] **Step 3: Run the tests and measure the sweeps**

Run: `cargo test --workspace && wasm-pack test --node crates/sugarscape-wasm`, then `cargo build --release -p sugarscape-cli` and each `./target/release/sugarscape sweep --builtin <id> --quiet --summary-csv /tmp/<id>.csv --out /dev/null`.
Expected: PASS (WASM 52, including `zi_sims_match_the_native_golden_entries`); each summary's means as its description records (a second or two each).

- [ ] **Step 4: Format, lint, commit**

```bash
cargo fmt --all && cargo clippy --all-targets -- -D warnings
git add sweeps/gs-efficiency.json sweeps/gs-dispersion.json sweeps/gs-shouts.json sweeps/gs-mechanism.json sweeps/cliff-prices.json sweeps/zip-days.json sweeps/zip-momentum.json sweeps/zip-shift.json crates/sugarscape-core/src/sweep.rs crates/sugarscape-cli crates/sugarscape-wasm/tests/web.rs
```
```bash
git commit -m "Measure Zero-Intelligence Traders: eight sweeps, the CLI's stop and WASM agreement

Claude-Session: https://claude.ai/code/session_01CDy35L1s2fYHpcryzQj8w4"
```

---

### Task 3: The page

**Files:**
- Modify: `web/src/types.ts`, `web/src/models.ts`, `web/src/engine.ts`, `web/src/compare-presets.ts`, `web/src/experiments/form.ts`, `web/src/ui/series-data.ts`, `web/src/ui/inspect-panel.ts`
- Test: `web/src/models.test.ts`, `web/src/compare-presets.test.ts`, `web/src/engine.test.ts`, `web/src/experiments/form.test.ts`, `web/src/ui/series-data.test.ts`, `web/src/determinism.test.ts`

**Interfaces:**
- Consumes: the WASM build of Tasks 1–2.
- Produces: `ZiConfig`, `ZiStats`, `ZiTrade`, `ZiTraderView`, `ZiInspection` (types.ts); `isZiView` (checked first: it tests `trade` and `supply`); `MODEL_CHARTS.zi`; the Compare entries `gs-1-vs-u` and `zi-c-vs-zip`; the color modes `side`, `profit`, `margin`.

- [ ] **Step 1: Write the failing tests**

Modify `web/src/models.test.ts` (a unified diff against the branch's base: save it and `git apply` it, or make the edits by hand):

````diff
diff --git a/web/src/models.test.ts b/web/src/models.test.ts
index 84cbac3..db444ba 100644
--- a/web/src/models.test.ts
+++ b/web/src/models.test.ts
@@ -7,6 +7,7 @@ import {
   isAntsView,
   isThresholdsView,
   isPunishmentView,
+  isZiView,
   isRetirementView,
   isFarolView,
   isCivilView,
@@ -121,6 +122,26 @@ describe('the presets menu', () => {
   });
 });
 
+describe('the zi model', () => {
+  it('is read by its tag, and its inspections by `trade` and `supply`, before the others with a panel', () => {
+    expect(modelOf({ model: 'zi' } as unknown as ModelConfig)).toBe('zi');
+    const cell = { site: { x: 1, y: 2 }, panel: 'schedules', unit: 1, demand: 102, supply: 34, trade: null, trader: null, agent: null } as unknown as AnyInspection;
+    const pun = { site: { x: 1, y: 2 }, panel: 'groups', group: null, agent: null, period: null, cooperation: null, punishment: null } as unknown as AnyInspection;
+    expect([cell, pun].map(isZiView)).toEqual([true, false]);
+    expect([isPunishmentView(cell), isRetirementView(cell), isThresholdsView(cell)]).toEqual([false, false, false]);
+  });
+
+  it('colors three ways, has no overlays, and ends after its periods', () => {
+    expect(COLOR_MODES.zi).toEqual([
+      ['side', 'Side'],
+      ['profit', 'Profit'],
+      ['margin', 'Margin'],
+    ]);
+    expect(MODEL_OVERLAYS.zi).toEqual([]);
+    expect(finishesUnpredictably({ model: 'zi', stop_at: 6 } as unknown as ModelConfig)).toBe(false);
+  });
+});
+
 describe('the punishment model', () => {
   it('is read by its tag, and its inspections by `punishment`, before the others with a panel', () => {
     const c = { model: 'punishment', stop_at: 2000 } as unknown as ModelConfig;
````

Modify `web/src/compare-presets.test.ts` (a unified diff against the branch's base: save it and `git apply` it, or make the edits by hand):

````diff
diff --git a/web/src/compare-presets.test.ts b/web/src/compare-presets.test.ts
index fefc285..75cf410 100644
--- a/web/src/compare-presets.test.ts
+++ b/web/src/compare-presets.test.ts
@@ -43,6 +43,12 @@ describe('compare presets', () => {
     expect([states.aSeed, states.b.seed]).toEqual([9, 9]);
   });
 
+  it('pairs the budget constraint with its absence, and ZI-C with ZIP', () => {
+    const ids = COMPARE_PRESETS.map((c) => [c.id, c.a, c.b, c.label]);
+    expect(ids).toContainEqual(['gs-1-vs-u', 'gs-1', 'gs-1-u', 'With vs without the budget constraint — Zero-Intelligence Traders (Compare)']);
+    expect(ids).toContainEqual(['zi-c-vs-zip', 'cliff-excess-demand', 'zip-excess-demand', 'ZI-C vs ZIP in a box market — Zero-Intelligence Traders (Compare)']);
+  });
+
   it('pairs punishment with its absence', () => {
     const ids = COMPARE_PRESETS.map((c) => [c.id, c.a, c.b, c.label]);
     expect(ids).toContainEqual(['bg-base-vs-none', 'bg-base', 'bg-none', 'With vs without punishment — Altruistic Punishment (Compare)']);
````

Modify `web/src/engine.test.ts` (a unified diff against the branch's base: save it and `git apply` it, or make the edits by hand):

````diff
diff --git a/web/src/engine.test.ts b/web/src/engine.test.ts
index 131e92e..42eb3a6 100644
--- a/web/src/engine.test.ts
+++ b/web/src/engine.test.ts
@@ -1154,6 +1154,7 @@ describe('Engine with other models', () => {
     expect(finishedNotice({ model: 'farol', stop_at: 100 } as unknown as ModelConfig, 100)).toBe('This run has reached its last round (100) — Reset to run it again');
     expect(finishedNotice({ model: 'ants', stop_at: 2000 } as unknown as ModelConfig, 2000)).toBe('This run has reached its last step (2000) — Reset to run it again');
     expect(finishedNotice({ model: 'thresholds', stop_at: 50 } as unknown as ModelConfig, 50)).toBe('This run has reached its last step (50) — Reset to run it again');
+    expect(finishedNotice({ model: 'zi', stop_at: 6 } as unknown as ModelConfig, 12000)).toBe('This run has reached its last period — Reset to run it again');
     expect(finishedNotice({ model: 'punishment', stop_at: 2000 } as unknown as ModelConfig, 2000)).toBe('This run has reached its last period (2000) — Reset to run it again');
     expect(finishedNotice({ model: 'retirement', stop_at: 50 } as unknown as ModelConfig, 50)).toBe('This run has reached its last period (50) — Reset to run it again');
     expect(finishedNotice({ model: 'retirement', stop_at_norm: true } as unknown as ModelConfig, 16)).toBe('The retirement norm has set in at t = 16 — Reset to run it again');
````

Modify `web/src/experiments/form.test.ts` (a unified diff against the branch's base: save it and `git apply` it, or make the edits by hand):

````diff
diff --git a/web/src/experiments/form.test.ts b/web/src/experiments/form.test.ts
index ea9d819..3342c38 100644
--- a/web/src/experiments/form.test.ts
+++ b/web/src/experiments/form.test.ts
@@ -147,6 +147,11 @@ describe('sweeps over other models', () => {
       ticks: 550,
       metric: { kind: 'final', series: 'fit' },
     });
+    expect(defaultForm('zi')).toMatchObject({
+      x: { path: 'shouts', values: '25,50,100,200,500,1000,2000' },
+      ticks: 12000,
+      metric: { kind: 'final', series: 'avg_efficiency' },
+    });
     expect(defaultForm('punishment')).toMatchObject({
       x: { path: 'size', values: '4,8,16,32,64,128,256' },
       ticks: 2000,
````

Modify `web/src/ui/series-data.test.ts` (a unified diff against the branch's base: save it and `git apply` it, or make the edits by hand):

````diff
diff --git a/web/src/ui/series-data.test.ts b/web/src/ui/series-data.test.ts
index 4d0c3e8..ab689c3 100644
--- a/web/src/ui/series-data.test.ts
+++ b/web/src/ui/series-data.test.ts
@@ -227,6 +227,13 @@ describe('the anasazi’s charts', () => {
   });
 });
 
+describe('zi charts', () => {
+  it('chart prices, efficiency, convergence, dispersion and volume over shouts', () => {
+    expect(MODEL_CHARTS.zi.map((c) => c.title)).toEqual(['Prices', 'Efficiency', 'Convergence', 'Profit dispersion', 'Volume']);
+    expect(timeAxisLabel('zi')).toBe('Shouts');
+  });
+});
+
 describe('punishment charts', () => {
   it('chart the types, cooperation with its long-run average, payoff and conflict over periods', () => {
     expect(MODEL_CHARTS.punishment.map((c) => c.title)).toEqual(['Types', 'Cooperation', 'Payoff', 'Conflict']);
````

Modify `web/src/determinism.test.ts` (a unified diff against the branch's base: save it and `git apply` it, or make the edits by hand):

````diff
diff --git a/web/src/determinism.test.ts b/web/src/determinism.test.ts
index da3ad3f..9c55851 100644
--- a/web/src/determinism.test.ts
+++ b/web/src/determinism.test.ts
@@ -13,6 +13,9 @@ import { InlineTransport } from './transport';
 import { decodeShare, encodeShare } from './share';
 import type {
   AgreementConfig,
+  ZiConfig,
+  ZiInspection,
+  ZiStats,
   PunishmentConfig,
   PunishmentInspection,
   PunishmentStats,
@@ -730,6 +733,26 @@ describe('the social-structure model through the engine', () => {
   });
 });
 
+describe('the zi model through the engine', () => {
+  it('stops after its last period and inspects a step, a trade and a trader', async () => {
+    const r = presets.find((p) => p.id === 'gs-1')!;
+    const config = { ...structuredClone(r.config as ZiConfig), shouts: 200, stop_at: 3 };
+    const e = await Engine.create({ config, seed: 1 }, { presets, transport: inline() });
+    e.setDisplay({ colorMode: 'profit' });
+    let ends = 0;
+    e.on('finished', () => ends++);
+    await e.advance(1_000_000);
+    const s = e.latest as ZiStats;
+    expect([e.finished, ends, e.tick, s.period]).toEqual([true, 1, 600, 4]);
+    expect(s.last_efficiency).toBeGreaterThan(50);
+    await e.select(0, 100);
+    const v = e.inspection!.view as ZiInspection;
+    expect([v.panel, v.unit, v.demand, v.supply]).toEqual(['schedules', 1, 102, 34]);
+    await e.select(5, 240);
+    expect((e.inspection!.view as ZiInspection).trader?.id).toBe(1);
+  });
+});
+
 describe('the punishment model through the engine', () => {
   it('stops at its last period and inspects an agent, its group and a period', async () => {
     const r = presets.find((p) => p.id === 'bg-base')!;
````

- [ ] **Step 2: Run them to see them fail**

Run: `cd web && npm ci && npm run wasm && npx vitest run`
Expected: failures in the six files above.

- [ ] **Step 3: Carry the model through the page**

Modify `web/src/types.ts` (a unified diff against the branch's base: save it and `git apply` it, or make the edits by hand):

````diff
diff --git a/web/src/types.ts b/web/src/types.ts
index 8c8a5ca..f32f434 100644
--- a/web/src/types.ts
+++ b/web/src/types.ts
@@ -112,7 +112,7 @@ export interface Config {
 }
 
 /** The models the playground runs (milestones 9–13). */
-export type ModelKind = 'sugarscape' | 'schelling' | 'ring' | 'anasazi' | 'civil' | 'spatial' | 'tags' | 'culture' | 'classes' | 'ethno' | 'opinions' | 'structure' | 'dpd' | 'norms' | 'agreement' | 'image' | 'farol' | 'ants' | 'thresholds' | 'retirement' | 'punishment';
+export type ModelKind = 'sugarscape' | 'schelling' | 'ring' | 'anasazi' | 'civil' | 'spatial' | 'tags' | 'culture' | 'classes' | 'ethno' | 'opinions' | 'structure' | 'dpd' | 'norms' | 'agreement' | 'image' | 'farol' | 'ants' | 'thresholds' | 'retirement' | 'punishment' | 'zi';
 
 /** A fraction range (Schelling's preferences). */
 export interface FRange { min: number; max: number }
@@ -511,7 +511,7 @@ export interface AgreementConfig {
   stop_at: number;
 }
 
-export type ModelConfig = Config | SchellingConfig | RingConfig | AnasaziConfig | CivilConfig | TagsConfig | SpatialConfig | CultureConfig | ClassesConfig | EthnoConfig | OpinionsConfig | StructureConfig | DpdConfig | NormsConfig | AgreementConfig | ImageConfig | FarolConfig | AntsConfig | ThresholdsConfig | RetirementConfig | PunishmentConfig;
+export type ModelConfig = Config | SchellingConfig | RingConfig | AnasaziConfig | CivilConfig | TagsConfig | SpatialConfig | CultureConfig | ClassesConfig | EthnoConfig | OpinionsConfig | StructureConfig | DpdConfig | NormsConfig | AgreementConfig | ImageConfig | FarolConfig | AntsConfig | ThresholdsConfig | RetirementConfig | PunishmentConfig | ZiConfig;
 
 /**
  * Arthur's El Farol bar and Challet and Zhang's minority game (milestone 23), with Challet, Marsili
@@ -849,6 +849,77 @@ export interface PunishmentInspection {
   punishment: number | null;
 }
 
+/**
+ * Gode and Sunder's zero-intelligence traders (milestone 28), with Cliff's critique, mechanism and
+ * ZIP traders.
+ */
+export interface ZiConfig {
+  model: 'zi';
+  market: 'gs1' | 'gs2' | 'gs3' | 'gs4' | 'gs5' | 'symmetric' | 'flat_supply' | 'excess_demand' | 'excess_supply' | 'retail' | 'custom';
+  buyers: number[][];
+  sellers: number[][];
+  price_max: number;
+  strategy: 'zi_u' | 'zi_c' | 'zip';
+  mechanism: 'book' | 'cliff';
+  nyse: boolean;
+  turns: 'trader' | 'side';
+  sellers_only: boolean;
+  period_end: 'shouts' | 'failures';
+  shouts: number;
+  momentum: 'code' | 'text';
+  shift: 'none' | 'demand' | 'supply';
+  shift_at: number;
+  stop_at: number;
+}
+
+export interface ZiStats {
+  tick: number;
+  /** This shout's trade price (null if it did not trade). */
+  price: number | null;
+  /** This period so far (the mean price and rmsd null before a trade). */
+  mean_price: number | null;
+  volume: number;
+  efficiency: number | null;
+  rmsd: number | null;
+  alpha: number | null;
+  dispersion: number;
+  period: number;
+  p0: number;
+  /** The last completed period, and means over the completed periods (null before one). */
+  last_price: number | null;
+  last_efficiency: number | null;
+  last_alpha: number | null;
+  last_dispersion: number | null;
+  avg_price: number | null;
+  avg_efficiency: number | null;
+  avg_dispersion: number | null;
+}
+
+export interface ZiTrade { period: number; tick: number; price: number; buyer: number; seller: number; value: number; cost: number }
+
+export interface ZiTraderView {
+  id: number;
+  buyer: boolean;
+  limits: number[];
+  traded: number;
+  profit: number;
+  equilibrium_profit: number;
+  margin: number | null;
+}
+
+/** A cell of the zi frame: a step of the schedules, a trade, or a trader. */
+export interface ZiInspection {
+  site: { x: number; y: number };
+  panel: 'schedules' | 'prices' | 'traders' | null;
+  unit: number | null;
+  demand: number | null;
+  supply: number | null;
+  trade: ZiTrade | null;
+  trader: ZiTraderView | null;
+  /** Always null: cells are read where they are. */
+  agent: null;
+}
+
 /** A preset: `title` is the menu's plain headline; `source` and `name` are its figure or paper and its rules. */
 export interface Preset { id: string; title: string; name: string; source: string; description: string; config: ModelConfig }
 
@@ -1158,7 +1229,7 @@ export interface AgreementStats {
   stable_at: number;
 }
 
-export type ModelStats = Snapshot | SchellingStats | RingStats | AnasaziStats | CivilStats | TagsStats | SpatialStats | CultureStats | ClassesStats | EthnoStats | OpinionsStats | StructureStats | DpdStats | NormsStats | AgreementStats | ImageStats | FarolStats | AntsStats | ThresholdsStats | RetirementStats | PunishmentStats;
+export type ModelStats = Snapshot | SchellingStats | RingStats | AnasaziStats | CivilStats | TagsStats | SpatialStats | CultureStats | ClassesStats | EthnoStats | OpinionsStats | StructureStats | DpdStats | NormsStats | AgreementStats | ImageStats | FarolStats | AntsStats | ThresholdsStats | RetirementStats | PunishmentStats | ZiStats;
 
 export interface SiteView { x: number; y: number; resources: number[]; capacities: number[]; pollution: number[] }
 export interface LinkView { id: number; alive: boolean }
@@ -1488,7 +1559,7 @@ export interface AgreementInspection {
   agent: null;
 }
 
-export type AnyInspection = Inspection | SchellingInspection | RingInspection | AnasaziInspection | CivilInspection | TagsInspection | SpatialInspection | CultureInspection | ClassesInspection | EthnoInspection | OpinionsInspection | StructureInspection | DpdInspection | NormsInspection | AgreementInspection | ImageInspection | FarolInspection | AntsInspection | ThresholdsInspection | RetirementInspection | PunishmentInspection;
+export type AnyInspection = Inspection | SchellingInspection | RingInspection | AnasaziInspection | CivilInspection | TagsInspection | SpatialInspection | CultureInspection | ClassesInspection | EthnoInspection | OpinionsInspection | StructureInspection | DpdInspection | NormsInspection | AgreementInspection | ImageInspection | FarolInspection | AntsInspection | ThresholdsInspection | RetirementInspection | PunishmentInspection | ZiInspection;
 
 /**
  * A sugarscape color mode, or (Schelling) `color`, `satisfaction`, `preference`, or (the anasazi)
@@ -1550,7 +1621,10 @@ export type ColorMode =
   | 'status'
   | 'type'
   | 'group'
-  | 'acts';
+  | 'acts'
+  | 'side'
+  | 'profit'
+  | 'margin';
 export type Layer = `resource:${number}` | `capacity:${number}` | `pollution:${number}` | `slice:${number}`;
 
 /** WASM calls throw a JSON string of FieldError[]; anything else becomes one error. */
````

Modify `web/src/models.ts` (a unified diff against the branch's base: save it and `git apply` it, or make the edits by hand):

````diff
diff --git a/web/src/models.ts b/web/src/models.ts
index dcd4c6f..3671c2d 100644
--- a/web/src/models.ts
+++ b/web/src/models.ts
@@ -1,6 +1,7 @@
 // Which model a config is (milestones 9–21), and what each model offers the page.
 import { NETWORKS, VALLEY_OVERLAYS, type Overlay } from './protocol';
 import type {
+  ZiInspection,
   PunishmentConfig,
   PunishmentInspection,
   RetirementConfig,
@@ -45,7 +46,7 @@ import type {
   TagsInspection,
 } from './types';
 
-export const MODELS: ModelKind[] = ['sugarscape', 'schelling', 'ring', 'anasazi', 'civil', 'tags', 'spatial', 'culture', 'classes', 'ethno', 'opinions', 'structure', 'dpd', 'norms', 'agreement', 'image', 'farol', 'ants', 'thresholds', 'retirement', 'punishment'];
+export const MODELS: ModelKind[] = ['sugarscape', 'schelling', 'ring', 'anasazi', 'civil', 'tags', 'spatial', 'culture', 'classes', 'ethno', 'opinions', 'structure', 'dpd', 'norms', 'agreement', 'image', 'farol', 'ants', 'thresholds', 'retirement', 'punishment', 'zi'];
 
 /** The presets menu's group labels. */
 export const MODEL_LABELS: Record<ModelKind, string> = {
@@ -70,12 +71,13 @@ export const MODEL_LABELS: Record<ModelKind, string> = {
   thresholds: 'Threshold Models',
   retirement: 'The Timing of Retirement',
   punishment: 'Altruistic Punishment',
+  zi: 'Zero-Intelligence Traders',
 };
 
 /** A config without a `model` key (or with `"sugarscape"`) is a sugarscape config. */
 export function modelOf(c: ModelConfig): ModelKind {
   const tag = (c as { model?: unknown }).model;
-  return tag === 'schelling' || tag === 'ring' || tag === 'anasazi' || tag === 'civil' || tag === 'spatial' || tag === 'tags' || tag === 'culture' || tag === 'classes' || tag === 'ethno' || tag === 'opinions' || tag === 'structure' || tag === 'dpd' || tag === 'norms' || tag === 'agreement' || tag === 'image' || tag === 'farol' || tag === 'ants' || tag === 'thresholds' || tag === 'retirement' || tag === 'punishment'
+  return tag === 'schelling' || tag === 'ring' || tag === 'anasazi' || tag === 'civil' || tag === 'spatial' || tag === 'tags' || tag === 'culture' || tag === 'classes' || tag === 'ethno' || tag === 'opinions' || tag === 'structure' || tag === 'dpd' || tag === 'norms' || tag === 'agreement' || tag === 'image' || tag === 'farol' || tag === 'ants' || tag === 'thresholds' || tag === 'retirement' || tag === 'punishment' || tag === 'zi'
     ? tag
     : 'sugarscape';
 }
@@ -169,6 +171,11 @@ export function isImageView(v: AnyInspection): v is ImageInspection {
   return 'cell' in v && 'group' in v;
 }
 
+/** A cell of the zi frame (a panel, a `trade` and a step's `supply`); check it first. */
+export function isZiView(v: AnyInspection): v is ZiInspection {
+  return 'panel' in v && 'trade' in v && 'supply' in v;
+}
+
 /** A cell of the punishment frame (a panel, an agent and its group, and a period's `punishment`); check it first. */
 export function isPunishmentView(v: AnyInspection): v is PunishmentInspection {
   return 'panel' in v && 'punishment' in v;
@@ -393,6 +400,12 @@ export const COLOR_MODES: Record<ModelKind, [ColorMode, string][]> = {
     ['payoff', 'Payoff'],
     ['group', 'Group'],
   ],
+  // Buyers and sellers; each trader's profit against its equilibrium profit; ZIP margins.
+  zi: [
+    ['side', 'Side'],
+    ['profit', 'Profit'],
+    ['margin', 'Margin'],
+  ],
 };
 
 /** The overlays each model can draw: the sugarscape's networks, the valley's water, settlements and links. */
@@ -418,4 +431,5 @@ export const MODEL_OVERLAYS: Record<ModelKind, Overlay[]> = {
   thresholds: [],
   retirement: [],
   punishment: [],
+  zi: [],
 };
````

Modify `web/src/engine.ts` (a unified diff against the branch's base: save it and `git apply` it, or make the edits by hand):

````diff
diff --git a/web/src/engine.ts b/web/src/engine.ts
index 3fee776..5bf6fda 100644
--- a/web/src/engine.ts
+++ b/web/src/engine.ts
@@ -54,6 +54,7 @@ export function finishedNotice(config: ModelConfig, tick: number): string {
   if (modelOf(config) === 'civil') return `A group has died out at t = ${tick} — Reset to run it again`;
   if (modelOf(config) === 'farol') return `This run has reached its last round (${tick}) — Reset to run it again`;
   if (modelOf(config) === 'ants' || modelOf(config) === 'thresholds') return `This run has reached its last step (${tick}) — Reset to run it again`;
+  if (modelOf(config) === 'zi') return `This run has reached its last period — Reset to run it again`;
   if (modelOf(config) === 'punishment') return `This run has reached its last period (${tick}) — Reset to run it again`;
   if (modelOf(config) === 'retirement') {
     const c = config as { stop_at_norm?: boolean; stop_at?: number };
````

Modify `web/src/compare-presets.ts` (a unified diff against the branch's base: save it and `git apply` it, or make the edits by hand):

````diff
diff --git a/web/src/compare-presets.ts b/web/src/compare-presets.ts
index 906c128..7f4ab28 100644
--- a/web/src/compare-presets.ts
+++ b/web/src/compare-presets.ts
@@ -182,6 +182,18 @@ export const COMPARE_PRESETS: ComparePreset[] = [
     a: 'bg-base',
     b: 'bg-none',
   },
+  {
+    id: 'gs-1-vs-u',
+    label: 'With vs without the budget constraint — Zero-Intelligence Traders (Compare)',
+    a: 'gs-1',
+    b: 'gs-1-u',
+  },
+  {
+    id: 'zi-c-vs-zip',
+    label: 'ZI-C vs ZIP in a box market — Zero-Intelligence Traders (Compare)',
+    a: 'cliff-excess-demand',
+    b: 'zip-excess-demand',
+  },
 ];
 
 /**
````

Modify `web/src/experiments/form.ts` (a unified diff against the branch's base: save it and `git apply` it, or make the edits by hand):

````diff
diff --git a/web/src/experiments/form.ts b/web/src/experiments/form.ts
index 0689056..f0bf9cf 100644
--- a/web/src/experiments/form.ts
+++ b/web/src/experiments/form.ts
@@ -104,6 +104,10 @@ export function defaultForm(model: ModelKind = 'sugarscape', config?: ModelConfi
     // The built-in ef-predictors' axis: how far attendance swings against predictors per agent.
     return { ...form, x: { path: 'strategies', values: '2:24:2' }, ticks: 2000, metric: { ...form.metric, kind: 'final', series: 'fluctuation' } };
   }
+  if (model === 'zi') {
+    // The built-in gs-shouts' axis: efficiency against the period's length (Gode and Sunder's "30 seconds").
+    return { ...form, x: { path: 'shouts', values: '25,50,100,200,500,1000,2000' }, ticks: 12000, metric: { ...form.metric, kind: 'final', series: 'avg_efficiency' } };
+  }
   if (model === 'punishment') {
     // The built-in bg-fig1b's axis: the long-run cooperation (the last 1 000 of 2 000 periods) against group size.
     return { ...form, x: { path: 'size', values: '4,8,16,32,64,128,256' }, ticks: 2000, metric: { ...form.metric, kind: 'final', series: 'long_run' } };
````

Modify `web/src/ui/series-data.ts` (a unified diff against the branch's base: save it and `git apply` it, or make the edits by hand):

````diff
diff --git a/web/src/ui/series-data.ts b/web/src/ui/series-data.ts
index 3c1e284..266f1ee 100644
--- a/web/src/ui/series-data.ts
+++ b/web/src/ui/series-data.ts
@@ -627,6 +627,38 @@ export const MODEL_CHARTS: Record<Exclude<ModelKind, 'sugarscape'>, ModelChart[]
       shown: hasRetirementGroups,
     },
   ],
+  zi: [
+    {
+      title: 'Prices',
+      lines: [
+        { key: 'price', label: 'Trade', color: '--c3' },
+        { key: 'mean_price', label: 'Mean this period', color: '--c1' },
+        { key: 'p0', label: 'Equilibrium (P₀)', color: '--c4' },
+      ],
+    },
+    {
+      title: 'Efficiency',
+      lines: [
+        { key: 'efficiency', label: 'This period so far (%)', color: '--c1' },
+        { key: 'last_efficiency', label: 'Last period (%)', color: '--c2' },
+      ],
+    },
+    {
+      title: 'Convergence',
+      lines: [
+        { key: 'alpha', label: "Smith's α this period", color: '--c1' },
+        { key: 'last_alpha', label: 'α, last period', color: '--c2' },
+      ],
+    },
+    {
+      title: 'Profit dispersion',
+      lines: [
+        { key: 'dispersion', label: 'This period so far', color: '--c1' },
+        { key: 'last_dispersion', label: 'Last period', color: '--c2' },
+      ],
+    },
+    { title: 'Volume', lines: [{ key: 'volume', label: 'Units traded this period', color: '--c1' }] },
+  ],
   punishment: [
     {
       title: 'Types',
@@ -662,7 +694,7 @@ export const MODEL_CHARTS: Record<Exclude<ModelKind, 'sugarscape'>, ModelChart[]
  * periods (ethnocentrism, HA06's word), cycles (the demographic PD, Epstein's word) or ticks.
  */
 export function timeAxisLabel(model: ModelKind): string {
-  return model === 'farol' ? 'Rounds' : model === 'ants' || model === 'thresholds' ? 'Steps' : model === 'retirement' || model === 'punishment' ? 'Periods' : model === 'anasazi' ? 'Year' : model === 'tags' || model === 'image' ? 'Generation' : model === 'culture' ? 'Events per site' : model === 'classes' || model === 'opinions' || model === 'structure' || model === 'agreement' ? 'Periods' : model === 'ethno' ? 'Period' : model === 'dpd' ? 'Cycle' : model === 'norms' ? 'Generations' : 'Tick';
+  return model === 'farol' ? 'Rounds' : model === 'ants' || model === 'thresholds' ? 'Steps' : model === 'retirement' || model === 'punishment' ? 'Periods' : model === 'zi' ? 'Shouts' : model === 'anasazi' ? 'Year' : model === 'tags' || model === 'image' ? 'Generation' : model === 'culture' ? 'Events per site' : model === 'classes' || model === 'opinions' || model === 'structure' || model === 'agreement' ? 'Periods' : model === 'ethno' ? 'Period' : model === 'dpd' ? 'Cycle' : model === 'norms' ? 'Generations' : 'Tick';
 }
 
 /** A calendar-year axis's tick labels: plain years (`1000`, not `1,000`), up to 3 decimals when zoomed in. */
````

Modify `web/src/ui/inspect-panel.ts` (a unified diff against the branch's base: save it and `git apply` it, or make the edits by hand):

````diff
diff --git a/web/src/ui/inspect-panel.ts b/web/src/ui/inspect-panel.ts
index efde89a..29e17fd 100644
--- a/web/src/ui/inspect-panel.ts
+++ b/web/src/ui/inspect-panel.ts
@@ -3,12 +3,13 @@ import { dpdRows } from '../dpd';
 import type { Engine } from '../engine';
 import { ethnoRows } from '../ethno';
 import { imageRows } from '../image-scoring';
-import { isAgreementView, isAntsView, isPunishmentView, isRetirementView, isThresholdsView, isFarolView, isCivilView, isClassesView, isCultureView, isDpdView, isEthnoView, isImageView, isNormsView, isOpinionsView, isRingView, isStructureView, isSpatialView, isSugarView, isTagsView, isValleyView } from '../models';
+import { isAgreementView, isAntsView, isPunishmentView, isZiView, isRetirementView, isThresholdsView, isFarolView, isCivilView, isClassesView, isCultureView, isDpdView, isEthnoView, isImageView, isNormsView, isOpinionsView, isRingView, isStructureView, isSpatialView, isSugarView, isTagsView, isValleyView } from '../models';
 import { playerRows } from '../spatial';
 import type {
   AgentView,
   AntsInspection,
   PunishmentInspection,
+  ZiInspection,
   RetirementInspection,
   ThresholdsInspection,
   FarolInspection,
@@ -291,6 +292,32 @@ export class InspectPanel {
     return rows;
   }
 
+  /** A step of the schedules, a trade, or a trader. */
+  private ziRows(view: ZiInspection): HTMLElement[] {
+    const row = (k: string, v: string) => h('tr', {}, h('th', {}, k), h('td', {}, v));
+    if (view.panel === 'schedules')
+      return [row('Unit', String(view.unit)), row('Demand', view.demand === null ? '—' : String(view.demand)), row('Supply', view.supply === null ? '—' : String(view.supply))];
+    if (view.panel === 'prices') {
+      const t = view.trade;
+      if (!t) return [row('Trade', 'none here')];
+      return [
+        row('Trade', `period ${t.period}, shout ${t.tick}`),
+        row('Price', String(t.price)),
+        row('Buyer', `#${t.buyer + 1} (value ${t.value}, profit ${t.value - t.price})`),
+        row('Seller', `#${t.seller + 1} (cost ${t.cost}, profit ${t.price - t.cost})`),
+      ];
+    }
+    const a = view.trader;
+    if (!a) return [row('Point', 'between panels')];
+    const rows = [
+      row('Trader', `#${a.id} · ${a.buyer ? 'buyer' : 'seller'}`),
+      row(a.buyer ? 'Values' : 'Costs', a.limits.join(', ')),
+      row('This period', `${a.traded} traded · profit ${a.profit} (equilibrium ${fmt(a.equilibrium_profit)})`),
+    ];
+    if (a.margin !== null) rows.push(row('Margin (μ)', fmt(a.margin)));
+    return rows;
+  }
+
   /** An agent and its group, or a period of the time strip. */
   private punishmentRows(view: PunishmentInspection): HTMLElement[] {
     const row = (k: string, v: string) => h('tr', {}, h('th', {}, k), h('td', {}, v));
@@ -548,6 +575,8 @@ export class InspectPanel {
             ? this.normsRows(view)
           : isAgreementView(view)
             ? this.agreementRows(view)
+          : isZiView(view)
+            ? this.ziRows(view)
           : isPunishmentView(view)
             ? this.punishmentRows(view)
           : isRetirementView(view)
````

- [ ] **Step 4: Run the page's build and tests**

Run: `cd web && npm run build && npm test`
Expected: the build succeeds; 733 tests pass (49 files).

- [ ] **Step 5: Commit**

```bash
git add web/src
```
```bash
git commit -m "Carry Zero-Intelligence Traders through the page

Claude-Session: https://claude.ai/code/session_01CDy35L1s2fYHpcryzQj8w4"
```

- [ ] **Step 6 (controller): check it in the browser**

`cd web && npm run build && npx vite preview`, then with `?debug`: `gs-1` in Profit colors — the schedules' steps, prices tightening around P₀ within each period, the strip against its equilibrium marks; `gs-4-u` in Side colors, prices everywhere; `cliff-flat` — prices well above P₀; `zip-excess-demand` in Margin colors, climbing toward P₀ over the days; `zip-retail` below P₀. Inspect a step, a trade and a trader. Rules panel groups as listed, ZIP momentum only for ZIP, NYSE and sellers-only only under Cliff's mechanism. Both Compare entries. Experiments with a zi preset: the default axis shouts a period against `avg_efficiency`; run the built-in `gs-shouts`.

---

### Task 4: The survey's zi claims

**Files:**
- Create: `survey/src/claims/zi.rs`
- Modify: `survey/src/claims/mod.rs`

**Interfaces:**
- Consumes: `sugarscape_core::zi::{Market, Mechanism, Momentum, PeriodEnd, Shift, Strategy, Turns, ZiConfig, ZiWorld, Period}`, `sugarscape_core::model::{ModelConfig, ModelWorld}`, `crate::runner::model_after`, `crate::claim::{all_of, greater, Claim, Outcome, Source, Verdict}`.
- Produces: 16 claims (`zi.gs.*`, `zi.cliff.*`, `zi.zip.*`).

- [ ] **Step 1: Write the claims**

Create `survey/src/claims/zi.rs` with exactly this content:

````rust
//! Zero-intelligence traders (milestone 28): Gode and Sunder (1993), with
//! Cliff's (1997) critique and ZIP traders. Gode and Sunder's markets run six
//! periods of 2 000 shouts (their "30 seconds" is never translated into
//! shouts); Cliff's run his simulator (his mechanism, side-first turns, days
//! ended by 100 failures; NYSE rules for ZI-C, not for ZIP) for ten days.

use sugarscape_core::model::{ModelConfig, ModelWorld};
use sugarscape_core::zi::{
    Market, Mechanism, Momentum, PeriodEnd, Shift, Strategy, Turns, ZiConfig, ZiWorld,
};

use crate::claim::{all_of, greater, Claim, Outcome, Source, Verdict};
use crate::runner::model_after;

const GS: &str = "Gode & Sunder 1993, JPE 101: 119";
const CLIFF: &str = "Cliff 1997, HP Labs HPL-97-91";

const GS_MARKETS: [Market; 5] = [
    Market::Gs1,
    Market::Gs2,
    Market::Gs3,
    Market::Gs4,
    Market::Gs5,
];
const CLIFF_MARKETS: [Market; 4] = [
    Market::Symmetric,
    Market::FlatSupply,
    Market::ExcessDemand,
    Market::ExcessSupply,
];
/// Table 2 (efficiency, %) and Table 3 (profit dispersion).
const T2_U: [f64; 5] = [90.0, 90.0, 76.7, 48.8, 86.0];
const T2_C: [f64; 5] = [99.9, 99.2, 99.0, 98.2, 97.1];
const T3_U: [f64; 5] = [225.48, 253.12, 90.54, 363.80, 156.28];
const T3_C: [f64; 5] = [28.53, 49.81, 15.90, 60.47, 19.07];

fn outcome(holds: bool, measured: String) -> Outcome {
    Outcome {
        verdict: if holds {
            Verdict::Holds
        } else {
            Verdict::Fails
        },
        measured,
        detail: String::new(),
    }
}

fn seeds(n: u64) -> Vec<u64> {
    (1..=n).collect()
}

fn gs(market: Market, strategy: Strategy) -> ZiConfig {
    ZiConfig {
        market,
        strategy,
        ..ZiConfig::default()
    }
}

/// Cliff's simulator in `market`.
fn cliff(market: Market, strategy: Strategy) -> ZiConfig {
    ZiConfig {
        market,
        strategy,
        price_max: 400,
        mechanism: Mechanism::Cliff,
        nyse: strategy != Strategy::Zip,
        turns: Turns::Side,
        period_end: PeriodEnd::Failures,
        stop_at: 10,
        ..ZiConfig::default()
    }
}

/// Each seed's finished world.
fn worlds(c: ZiConfig, n: u64) -> Vec<ZiWorld> {
    model_after(&ModelConfig::Zi(c), &seeds(n), 1_000_000, |w| match w {
        ModelWorld::Zi(z) => z.as_ref().clone(),
        _ => unreachable!(),
    })
}

fn mean(v: &[f64]) -> f64 {
    let f: Vec<f64> = v.iter().copied().filter(|x| x.is_finite()).collect();
    f.iter().sum::<f64>() / f.len().max(1) as f64
}

/// Each seed's mean over its periods of `f`.
fn per_seed(ws: &[ZiWorld], f: fn(&sugarscape_core::zi::Period) -> f64) -> Vec<f64> {
    ws.iter()
        .map(|w| mean(&w.periods().iter().map(f).collect::<Vec<_>>()))
        .collect()
}

/// The mean price on each day, across seeds.
fn daily(ws: &[ZiWorld], days: usize) -> Vec<f64> {
    (0..days)
        .map(|d| {
            mean(
                &ws.iter()
                    .filter_map(|w| w.periods().get(d).map(|p| p.mean_price))
                    .collect::<Vec<_>>(),
            )
        })
        .collect()
}

fn show(v: &[f64]) -> String {
    let parts: Vec<String> = v.iter().map(|x| format!("{x:.1}")).collect();
    format!("[{}]", parts.join(", "))
}

/// The Spearman correlation between each period's trade order and the order
/// of the trades' surplus (value − cost, largest first), averaged over periods.
fn rank_correlation(w: &ZiWorld) -> f64 {
    let mut rs = Vec::new();
    for p in 1..=w.periods().len() as u64 {
        let t: Vec<_> = w.trades().filter(|t| t.period == p).collect();
        let n = t.len();
        if n < 3 {
            continue;
        }
        // Ranks of surplus, largest first (ties by average rank).
        let s: Vec<f64> = t
            .iter()
            .map(|t| f64::from(t.value) - f64::from(t.cost))
            .collect();
        let mut rank = vec![0.0; n];
        for i in 0..n {
            let above = s.iter().filter(|&&x| x > s[i]).count() as f64;
            let same = s.iter().filter(|&&x| x == s[i]).count() as f64;
            rank[i] = above + (same + 1.0) / 2.0;
        }
        let order: Vec<f64> = (1..=n).map(|k| k as f64).collect();
        let (mr, mo) = (mean(&rank), mean(&order));
        let cov: f64 = (0..n).map(|i| (rank[i] - mr) * (order[i] - mo)).sum();
        let (vr, vo): (f64, f64) = (
            rank.iter().map(|r| (r - mr) * (r - mr)).sum(),
            order.iter().map(|o| (o - mo) * (o - mo)).sum(),
        );
        if vr > 0.0 {
            rs.push(cov / (vr * vo).sqrt());
        }
    }
    mean(&rs)
}

/// The slope of the RMS deviation from P₀ against the transaction number
/// (Gode and Sunder's Fig. 6 and Table 1), pooled over periods and seeds, over
/// the transaction numbers reached in at least half the periods.
fn rms_slope(ws: &[ZiWorld]) -> f64 {
    let mut sq: Vec<(f64, u32)> = Vec::new();
    let mut periods = 0;
    for w in ws {
        let p0 = w.equilibrium().price;
        for p in 1..=w.periods().len() as u64 {
            periods += 1;
            for (k, t) in w.trades().filter(|t| t.period == p).enumerate() {
                if sq.len() <= k {
                    sq.push((0.0, 0));
                }
                sq[k].0 += (f64::from(t.price) - p0).powi(2);
                sq[k].1 += 1;
            }
        }
    }
    let pts: Vec<(f64, f64)> = sq
        .iter()
        .enumerate()
        .filter(|(_, (_, n))| 2 * n >= periods)
        .map(|(k, (s, n))| ((k + 1) as f64, (s / f64::from(*n)).sqrt()))
        .collect();
    let (mx, my) = (
        mean(&pts.iter().map(|p| p.0).collect::<Vec<_>>()),
        mean(&pts.iter().map(|p| p.1).collect::<Vec<_>>()),
    );
    let num: f64 = pts.iter().map(|(x, y)| (x - mx) * (y - my)).sum();
    let den: f64 = pts.iter().map(|(x, _)| (x - mx) * (x - mx)).sum();
    num / den
}

pub fn claims() -> Vec<Claim> {
    vec![
        Claim {
            id: "zi.gs.table2-u",
            item: "gs-efficiency",
            source: Source::Book,
            citation: GS,
            text: "Table 2, ZI-U: efficiency 90.0, 90.0, 76.7, 48.8, 86.0 in markets 1–5 (every unit trades, so it follows from the schedules; within 1 point in every market, 10 seeds × 6 periods)",
            check: |_| {
                let v: Vec<f64> = GS_MARKETS.iter().map(|&m| mean(&per_seed(&worlds(gs(m, Strategy::ZiU), 10), |p| p.efficiency))).collect();
                let ok = v.iter().zip(T2_U).all(|(a, b)| (a - b).abs() <= 1.0);
                outcome(ok, format!("{} against {}", show(&v), show(&T2_U)))
                    .with("Markets 1–4 were digitized so that this holds exactly; market 5 is a fine staircase read as well as the scan allows.")
            },
        },
        Claim {
            id: "zi.gs.table2-c",
            item: "gs-efficiency",
            source: Source::Book,
            citation: GS,
            text: "Table 2, ZI-C: 'imposing a budget constraint … is sufficient to raise the allocative efficiency of these auctions close to 100 percent' — 99.9, 99.2, 99.0, 98.2, 97.1 (within 1.5 points in every market, 10 seeds × 6 periods of 2 000 shouts)",
            check: |_| {
                let v: Vec<f64> = GS_MARKETS.iter().map(|&m| mean(&per_seed(&worlds(gs(m, Strategy::ZiC), 10), |p| p.efficiency))).collect();
                let ok = v.iter().zip(T2_C).all(|(a, b)| (a - b).abs() <= 1.5);
                outcome(ok, format!("{} against {}", show(&v), show(&T2_C)))
            },
        },
        Claim {
            id: "zi.gs.table3",
            item: "gs-dispersion",
            source: Source::Book,
            citation: GS,
            text: "Table 3: profit dispersion — ZI-C 28.5, 49.8, 15.9, 60.5, 19.1 (within 25 % in every market) and ZI-U above ZI-C in every market",
            check: |_| {
                let c: Vec<f64> = GS_MARKETS.iter().map(|&m| mean(&per_seed(&worlds(gs(m, Strategy::ZiC), 10), |p| p.dispersion))).collect();
                let u: Vec<f64> = GS_MARKETS.iter().map(|&m| mean(&per_seed(&worlds(gs(m, Strategy::ZiU), 10), |p| p.dispersion))).collect();
                let close = c.iter().zip(T3_C).all(|(a, b)| (a - b).abs() <= 0.25 * b);
                let above = u.iter().zip(&c).all(|(a, b)| a > b);
                all_of(vec![
                    ("ZI-C within 25 %".into(), outcome(close, format!("{} against {}", show(&c), show(&T3_C)))),
                    ("ZI-U above".into(), outcome(above, format!("ZI-U {} (Table 3: {})", show(&u), show(&T3_U)))),
                ])
            },
        },
        Claim {
            id: "zi.gs.table1",
            item: "gs-1",
            source: Source::Book,
            citation: GS,
            text: "Fig. 6 and Table 1: ZI-C prices 'converge slowly toward equilibrium within each period' — the RMS deviation from P₀ falls with the transaction number (a negative slope in every market, 10 seeds × 6 periods)",
            check: |_| {
                let v: Vec<f64> = GS_MARKETS.iter().map(|&m| rms_slope(&worlds(gs(m, Strategy::ZiC), 10))).collect();
                outcome(v.iter().all(|&s| s < 0.0), format!("slopes {} (Table 1: −0.64, −0.61, −1.23, −3.59, −0.83)", show(&v)))
            },
        },
        Claim {
            id: "zi.gs.order",
            item: "gs-1",
            source: Source::Book,
            citation: GS,
            text: "Footnote 5: 'The Spearman rank correlation between the actual and the efficient order of surplus extracted is, on average, highest for ZI-C traders (.74), lowest for ZI-U traders (.42)' (ZI-C greater in market 1, 10 seeds)",
            check: |_| {
                let c: Vec<f64> = worlds(gs(Market::Gs1, Strategy::ZiC), 10).iter().map(rank_correlation).collect();
                let u: Vec<f64> = worlds(gs(Market::Gs1, Strategy::ZiU), 10).iter().map(rank_correlation).collect();
                greater(&c, &u, "ZI-C", "ZI-U").with(&format!("means: ZI-C {:.2}, ZI-U {:.2} (the footnote: .74, .42)", mean(&c), mean(&u)))
            },
        },
        Claim {
            id: "zi.gs.period",
            item: "gs-shouts",
            source: Source::Comment,
            citation: GS,
            text: "Ours: the period's unstated length decides the efficiency — 'six periods of specified duration (… 30 seconds for machine traders)' is never given in shouts; at 100 shouts a period ZI-C efficiency is below 90 in every market",
            check: |_| {
                let v: Vec<f64> = GS_MARKETS
                    .iter()
                    .map(|&m| mean(&per_seed(&worlds(ZiConfig { shouts: 100, ..gs(m, Strategy::ZiC) }, 10), |p| p.efficiency)))
                    .collect();
                outcome(v.iter().all(|&e| e < 90.0), format!("{} at 100 shouts", show(&v)))
            },
        },
        Claim {
            id: "zi.gs.mechanism",
            item: "gs-mechanism",
            source: Source::Comment,
            citation: GS,
            text: "Ours: Table 2's ZI-C efficiencies survive Cliff's mechanism (a random willing trader at the shout's price, instead of the standing quote at the earlier price) — within 1.5 points in every market",
            check: |_| {
                let v: Vec<f64> = GS_MARKETS
                    .iter()
                    .map(|&m| mean(&per_seed(&worlds(ZiConfig { mechanism: Mechanism::Cliff, nyse: false, ..gs(m, Strategy::ZiC) }, 10), |p| p.efficiency)))
                    .collect();
                outcome(v.iter().zip(T2_C).all(|(a, b)| (a - b).abs() <= 1.5), format!("{} against {}", show(&v), show(&T2_C)))
            },
        },
        Claim {
            id: "zi.cliff.predictions",
            item: "cliff-prices",
            source: Source::Comment,
            citation: CLIFF,
            text: "Cliff's E(P) for ZI-C: 'The mean transaction price in ZI-C markets can be predicted from the expected value E(P) of the pdf given by the intersection of the sellers' offer-price pdf and the buyers' bid-price pdf' — 200 (symmetric), 233⅓ (flat supply), 125 (excess demand), 260 (excess supply); each within 5 in his simulator (50 seeds × 10 days)",
            check: |_| {
                let want = [200.0, 233.3, 125.0, 260.0];
                let v: Vec<f64> = CLIFF_MARKETS.iter().map(|&m| mean(&per_seed(&worlds(cliff(m, Strategy::ZiC), 50), |p| p.mean_price))).collect();
                let parts = CLIFF_MARKETS
                    .iter()
                    .zip(v.iter().zip(want))
                    .map(|(m, (&a, b))| (format!("{m:?}"), outcome((a - b).abs() <= 5.0, format!("{a:.1} against {b:.1}"))))
                    .collect();
                all_of(parts).with("His 233⅓ is not his own formula's: Eq. 5 gives 200 + 125/3 = 241⅔ (summing his discrete pdf, 245⅓).")
            },
        },
        Claim {
            id: "zi.cliff.book",
            item: "cliff-prices",
            source: Source::Comment,
            citation: CLIFF,
            text: "Ours: Cliff's critique holds in Gode and Sunder's own mechanism — with the standing quote and the earlier price, ZI-C mean prices in the flat and box markets also sit more than 10 from P₀ = 200 (50 seeds × 10 periods of 2 000 shouts)",
            check: |_| {
                let v: Vec<f64> = CLIFF_MARKETS[1..]
                    .iter()
                    .map(|&m| {
                        let c = ZiConfig { market: m, price_max: 400, stop_at: 10, ..ZiConfig::default() };
                        mean(&per_seed(&worlds(c, 50), |p| p.mean_price))
                    })
                    .collect();
                outcome(v.iter().all(|&p| (p - 200.0).abs() > 10.0), format!("flat, excess demand, excess supply: {} (Cliff's mechanism: see zi.cliff.predictions)", show(&v)))
            },
        },
        Claim {
            id: "zi.zip.converge",
            item: "zip-days",
            source: Source::Comment,
            citation: CLIFF,
            text: "ZIP in the symmetric and flat markets: prices converge to $2.00 'typically within the first four trading days' (the mean price on day 4 within 5 of 200 in both, 50 seeds)",
            check: |_| {
                let s = daily(&worlds(cliff(Market::Symmetric, Strategy::Zip), 50), 10);
                let f = daily(&worlds(cliff(Market::FlatSupply, Strategy::Zip), 50), 10);
                all_of(vec![
                    ("symmetric".into(), outcome((s[3] - 200.0).abs() <= 5.0, format!("days 1–10 {}", show(&s)))),
                    ("flat supply".into(), outcome((f[3] - 200.0).abs() <= 5.0, format!("days 1–10 {}", show(&f)))),
                ])
            },
        },
        Claim {
            id: "zi.zip.below",
            item: "zip-days",
            source: Source::Comment,
            citation: CLIFF,
            text: "ZIP in the excess-demand market: a 'comparatively slow (yet steady) approach … from below' (the daily mean below 200 on each of days 1–10, and higher on day 10 than day 1)",
            check: |_| {
                let d = daily(&worlds(cliff(Market::ExcessDemand, Strategy::Zip), 50), 10);
                outcome(d.iter().all(|&p| p < 200.0) && d[9] > d[0], format!("days 1–10 {}", show(&d)))
            },
        },
        Claim {
            id: "zi.zip.efficiency",
            item: "zip-symmetric",
            source: Source::Comment,
            citation: CLIFF,
            text: "ZIP efficiency is 'typically very high (often averaging 100%)' (at least 99 % over days 2–10 in all four markets)",
            check: |_| {
                let v: Vec<f64> = CLIFF_MARKETS
                    .iter()
                    .map(|&m| {
                        let ws = worlds(cliff(m, Strategy::Zip), 50);
                        mean(&ws.iter().flat_map(|w| w.periods()[1..].iter().map(|p| p.efficiency)).collect::<Vec<_>>())
                    })
                    .collect();
                outcome(v.iter().all(|&e| e >= 99.0), format!("{} (symmetric, flat, excess demand, excess supply)", show(&v)))
            },
        },
        Claim {
            id: "zi.zip.dispersion",
            item: "zip-symmetric",
            source: Source::Comment,
            citation: CLIFF,
            text: "ZIP's profit dispersion is 'in some cases approximately a factor of ten less' than ZI-C's (a tenth or less in at least one of the four markets, days 5–10)",
            check: |_| {
                let ratio: Vec<f64> = CLIFF_MARKETS
                    .iter()
                    .map(|&m| {
                        let late = |ws: Vec<ZiWorld>| mean(&ws.iter().flat_map(|w| w.periods()[4..].iter().map(|p| p.dispersion)).collect::<Vec<_>>());
                        late(worlds(cliff(m, Strategy::Zip), 50)) / late(worlds(cliff(m, Strategy::ZiC), 50))
                    })
                    .collect();
                outcome(ratio.iter().any(|&r| r <= 0.1), format!("ZIP over ZI-C: {}", show(&ratio)))
            },
        },
        Claim {
            id: "zi.zip.shift",
            item: "zip-shift",
            source: Source::Comment,
            citation: CLIFF,
            text: "ZIP re-converges after a shift: demand up $0.50 (P₀ 225) or supply down $0.50 (P₀ 175) after day 10 — the day-20 mean within 10 of the new P₀ (50 seeds)",
            check: |_| {
                let run = |shift| {
                    let c = ZiConfig { shift, stop_at: 20, ..cliff(Market::Symmetric, Strategy::Zip) };
                    daily(&worlds(c, 50), 20)
                };
                let (d, s) = (run(Shift::Demand), run(Shift::Supply));
                all_of(vec![
                    ("demand".into(), outcome((d[19] - 225.0).abs() <= 10.0, format!("days 11–20 {}", show(&d[10..])))),
                    ("supply".into(), outcome((s[19] - 175.0).abs() <= 10.0, format!("days 11–20 {}", show(&s[10..])))),
                ])
            },
        },
        Claim {
            id: "zi.zip.retail",
            item: "zip-retail",
            source: Source::Comment,
            citation: CLIFF,
            text: "ZIP in Smith's retail market (only sellers shout): 'the average transaction prices are typically less than $2.00 (significantly below the theoretical equilibrium price of $2.25)' (the mean over days 5–10 at least 10 below 225)",
            check: |_| {
                let ws = worlds(ZiConfig { sellers_only: true, ..cliff(Market::Retail, Strategy::Zip) }, 50);
                let d = daily(&ws, 10);
                let late = mean(&d[4..]);
                outcome(late <= 215.0, format!("days 1–10 {} (days 5–10: {late:.1})", show(&d)))
            },
        },
        Claim {
            id: "zi.zip.momentum",
            item: "zip-momentum",
            source: Source::Comment,
            citation: CLIFF,
            text: "Ours: Cliff's text gives ZIP momentum γ ~ U[0.2, 0.8], his code U[0, 0.1]; the text's reading converges faster in the box markets (day-10 distance from P₀ smaller in both, Mann–Whitney)",
            check: |_| {
                let dist = |m, momentum| {
                    worlds(ZiConfig { momentum, ..cliff(m, Strategy::Zip) }, 50)
                        .iter()
                        .map(|w| (w.periods()[9].mean_price - 200.0).abs())
                        .collect::<Vec<f64>>()
                };
                all_of(vec![
                    ("excess demand".into(), greater(&dist(Market::ExcessDemand, Momentum::Code), &dist(Market::ExcessDemand, Momentum::Text), "code", "text")),
                    ("excess supply".into(), greater(&dist(Market::ExcessSupply, Momentum::Code), &dist(Market::ExcessSupply, Momentum::Text), "code", "text")),
                ])
            },
        },
    ]
}
````

Modify `survey/src/claims/mod.rs` (a unified diff against the branch's base: save it and `git apply` it, or make the edits by hand):

````diff
diff --git a/survey/src/claims/mod.rs b/survey/src/claims/mod.rs
index b4b7d4c..edcd585 100644
--- a/survey/src/claims/mod.rs
+++ b/survey/src/claims/mod.rs
@@ -22,6 +22,7 @@ mod spatial;
 mod structure;
 mod tags;
 mod thresholds;
+mod zi;
 
 use crate::claim::Claim;
 
@@ -51,6 +52,7 @@ pub fn all() -> Vec<Claim> {
         structure::claims(),
         tags::claims(),
         thresholds::claims(),
+        zi::claims(),
     ]
     .into_iter()
     .flatten()
````

- [ ] **Step 2: Run them**

Run: `cd survey && rustfmt --edition 2021 src/claims/zi.rs && cargo build --release && ./target/release/survey --only zi.`
Expected (about a second): 14 Holds (table2-u, table2-c, table3, table1, order — 0.91 against 0.85, period — 44–86 % at 100 shouts, mechanism, cliff.book, zip.converge, zip.below, zip.dispersion, zip.shift, zip.retail, zip.momentum) and 2 Fails (`zi.cliff.predictions`: the box markets at 138.1 against 125 and 250.3 against 260; `zi.zip.efficiency`: the symmetric market at 98.9).

- [ ] **Step 3: Commit**

```bash
git add survey/src/claims/zi.rs survey/src/claims/mod.rs
```
```bash
git commit -m "Survey Zero-Intelligence Traders: Gode and Sunder's tables, the period's length, Cliff's predictions and ZIP

Claude-Session: https://claude.ai/code/session_01CDy35L1s2fYHpcryzQj8w4"
```

---

### Task 5: README, roadmap, papers index, spec amendments and full verification

**Files:**
- Modify: `README.md`, `docs/roadmap.md`, `docs/papers.md`, `docs/superpowers/specs/2026-09-28-zi-traders-design.md`

- [ ] **Step 1: Write the docs**

Modify `README.md` (a unified diff against the branch's base: save it and `git apply` it, or make the edits by hand):

````diff
diff --git a/README.md b/README.md
index e7e4850..abb6823 100644
--- a/README.md
+++ b/README.md
@@ -1802,6 +1802,68 @@ Punishment with a PDE Model of Cultural Multilevel Selection," arXiv:2405.18419
 Biol.* 2025); Marco Janssen's replication, CoMSES Net 2223 (GPL-3.0, read for its readings only). See
 `docs/superpowers/specs/2026-09-28-punishment-design.md`.
 
+### Zero-Intelligence Traders (Gode & Sunder 1993; Cliff 1997)
+
+**The model.** Are markets efficient because traders are smart? Gode and Sunder replaced human traders
+in a double auction with programs that shout random prices. Unconstrained (ZI-U), they trade at a loss
+and waste surplus; merely forbidden to trade at a loss (ZI-C), they capture almost all of it — "the
+market as a partial substitute for individual rationality." Six buyers and six sellers trade units
+one at a time; a shout that crosses the standing bid or ask trades at the earlier order's price, and
+each trade clears the book. Cliff (1997) argued that ZI-C prices converge on equilibrium only when
+supply and demand are symmetric, and built traders that learn a profit margin (ZIP).
+
+**How the sources were read.** Gode and Sunder's paper is a scan (read by OCR); their five markets
+exist only as step curves in the figures. Because ZI-U traders trade every unit, a market's ZI-U
+efficiency follows arithmetically from its schedules — so Table 2's ZI-U numbers pin markets 1–4 down
+exactly (90.0, 90.0, 76.7, 48.8), alongside the text's P₀ of 69 and 170 and volumes of 24 and 6;
+market 5, a fine staircase, is read as well as the scan allows (86.7 against 86.0). Cliff's results
+come from the C code in his appendices, which differs from his text in places; the code is the
+default, the text a switch.
+
+Measured (the survey and the presets' descriptions):
+
+- **Gode and Sunder reproduce, given enough time.** ZI-C efficiency 99.9, 99.8, 99.7, 99.5, 97.1 in
+  markets 1–5 (their 99.9, 99.2, 99.0, 98.2, 97.1); ZI-U exactly theirs; profit dispersion close to Table
+  3; prices tightening within each period (Table 1's negative slopes). The rank correlation of the
+  trading order with the efficient one is higher for ZI-C than ZI-U, as their footnote says, though
+  higher than theirs (0.91 and 0.85 against 0.74 and 0.42).
+- **But "30 seconds" decides it.** Their periods lasted 30 seconds, never translated into shouts. At 100
+  shouts a period ZI-C efficiency is 44–86 %; it needs about 500–1 000 to reach their numbers. The
+  random traders capture the surplus because they get enough chances.
+- **Cliff's critique holds in direction, not in number.** In his simulator (a random willing trader at
+  the shout's price) ZI-C mean prices are 200.6, 235.8, 138.1, 250.3 in his four markets (P₀ 200): his
+  predictions hold for the symmetric and flat markets but miss the box markets by 13 and 10, and his
+  printed 233⅓ is not his own formula's (241⅔). In Gode and Sunder's own mechanism the prices sit
+  about half as far from P₀ (216.6, 161.8, 232.9) — still off, so the critique's direction survives.
+- **ZIP learns its way to equilibrium.** Daily mean prices converge on P₀ in all four markets — the
+  flat one within 4 days, the excess-demand box from below and steadily — and after a demand or supply
+  shift; profit dispersion falls to a tenth of ZI-C's or less. Efficiency averages 98.9–100 % (the
+  symmetric market just short of "often averaging 100 %"). In Smith's retail market, where only sellers
+  post prices, trades stay below P₀ as Cliff says, but rise past $2.00 by day 9.
+- **Cliff's momentum.** His text draws ZIP's momentum from U[0.2, 0.8]; his code overwrites it with
+  U[0, 0.1]. The text's reading converges a day or two sooner in the box markets.
+
+Switches: **Market** (Gode and Sunder's 1–5, Cliff's symmetric, flat supply, excess demand, excess supply
+and retail, or custom), **Highest price**, **Traders** (ZI-C, ZI-U, ZIP) with **ZIP momentum** (his code or
+his text), **Trades happen** (against the standing quote, or with a random willing trader) with **NYSE
+rules**, **Who shouts** (a random trader, or a side then a trader), **Only sellers shout**, **A period
+ends** (after a number of shouts, or after 100 failures in a row) with **Shouts a period**, **Shift**
+(demand up or supply down 50) with **Shift from period**, and **Stop after period**. The view follows Gode
+and Sunder's figures: the schedules, the trade prices across periods, and a strip of traders with
+their profits against their equilibrium profits. Color modes: **Side**, **Profit**, **Margin**. Charts:
+Prices; Efficiency; Convergence (Smith's α); Profit dispersion; Volume. Presets: `gs-1`–`gs-5`, `gs-1-u`,
+`gs-4-u`, `cliff-symmetric`, `cliff-flat`, `cliff-excess-demand`, `cliff-excess-supply`, `zip-symmetric`,
+`zip-flat`, `zip-excess-demand`, `zip-excess-supply`, `zip-demand-shift`, `zip-supply-shift`, `zip-retail`.
+**Compare** entries: "With vs without the budget constraint — Zero-Intelligence Traders (Compare)" and
+"ZI-C vs ZIP in a box market — Zero-Intelligence Traders (Compare)". Built-in sweeps: `gs-efficiency`,
+`gs-dispersion`, `gs-shouts`, `gs-mechanism`, `cliff-prices`, `zip-days`, `zip-momentum`, `zip-shift`.
+
+Credit: Dhananjay K. Gode and Shyam Sunder, "Allocative Efficiency of Markets with Zero-Intelligence
+Traders: Market as a Partial Substitute for Individual Rationality," *Journal of Political Economy*
+101(1): 119–137 (1993); Dave Cliff, "Minimal-Intelligence Agents for Bargaining Behaviours in
+Market-Based Environments," HP Laboratories HPL-97-91 (1997). See
+`docs/superpowers/specs/2026-09-28-zi-traders-design.md`.
+
 ## Experiments
 
 The header's **Experiments** switch replaces the grid with a sweep runner (the playground's
````

Modify `docs/roadmap.md` (a unified diff against the branch's base: save it and `git apply` it, or make the edits by hand):

````diff
diff --git a/docs/roadmap.md b/docs/roadmap.md
index 62d3db8..cae14c6 100644
--- a/docs/roadmap.md
+++ b/docs/roadmap.md
@@ -263,6 +263,17 @@ Figs. 1–4, the text's rate 2 (either group starting a conflict, a switch here,
 calibration is off by five; Cooney's payoff dip appears under every victory rule. See
 `docs/superpowers/specs/2026-09-28-punishment-design.md`.
 
+## Milestone 28: Zero-Intelligence Traders (done)
+
+Gode and Sunder's double auction with zero-intelligence traders (JPE 1993) as a model kind, with
+Cliff's critique, simulator and ZIP traders (HP Labs 1997): their five markets read from the scan
+(Table 2's ZI-U efficiencies pin four of them down exactly), Cliff's markets, shifts and retail
+market, and every unstated rule a switch. Gode and Sunder's efficiencies and dispersions reproduce —
+but only with enough shouts per period, which their "30 seconds" never gives; Cliff's price
+predictions miss the box markets and his 233⅓ is not his formula's, though his critique's direction
+holds even in Gode and Sunder's mechanism; ZIP converges, and his code's momentum is not his text's.
+See `docs/superpowers/specs/2026-09-28-zi-traders-design.md`.
+
 ## Experiments and science
 
 - **Parameter sweeps / batch runs**: done (Milestone 5).
@@ -286,6 +297,7 @@ calibration is off by five; Cooney's payoff dip appears under every victory rule
 - **Granovetter's threshold models** (and Watts's global cascades): done (Milestone 25).
 - **Axtell and Epstein's timing of retirement**: done (Milestone 26).
 - **Boyd, Gintis, Bowles and Richerson's altruistic punishment** (and Cooney's PDE critique): done (Milestone 27).
+- **Gode and Sunder's zero-intelligence traders** (and Cliff's critique and ZIP traders): done (Milestone 28).
 - **Minds 1: the utility mind and the ideal free distribution** (our experiment; docs/studies/2026-09-27-minds.md): done.
 - **Minds 2: A\* and walking; which of the book's results need the jump** (our experiment; docs/studies/2026-09-27-minds.md): done.
 - **Credit hierarchy view**: done (Milestone 6).
````

Modify `docs/papers.md` (a unified diff against the branch's base: save it and `git apply` it, or make the edits by hand):

````diff
diff --git a/docs/papers.md b/docs/papers.md
index ee7f105..55c4e26 100644
--- a/docs/papers.md
+++ b/docs/papers.md
@@ -35,6 +35,7 @@ read online or from another copy; add it when found. Scanned PDFs (no text layer
 | 25 | `thresholds` | `thresholds/granovetter-1978-ajs-threshold-models-of-collective-behavior.pdf` (*scan*, read by OCR); the follow-up `thresholds/watts-2002-pnas-simple-model-of-global-cascades-on-random-networks.pdf` (from the Internet Archive's copy of PNAS) | the crowds and Fig. 2's continuous jump reproduce, but a crowd of people tips at a σ set by rounding and sampled crowds do not jump; the city's riot of 100 comes 2 % of the time; Watts's upper edge depends on n, his Fig. 4b cannot be built, hubs help in both regimes |
 | 26 | `retirement` | `retirement/axtell-epstein-1999-coordination-in-transient-social-networks-retirement.pdf`; the revised text `demographic-pd/epstein-2006-generative-social-science.pdf` (ch. 7) | the realizations (a little slower) and network-size effects reproduce, extent only at 10 % rational; footnote 5 is false; Fig. 6-6's minimum rationality needs an unstated rule (friends replaced); the 65 → 62 switch takes 2 periods, not 20–35; coupling slows the rational group |
 | 27 | `punishment` | `punishment/boyd-gintis-bowles-richerson-2003-pnas-altruistic-punishment.pdf`; the critique `punishment/cooney-2024-arxiv-altruistic-punishment-pde-multilevel-selection.pdf` (published in *Bull. Math. Biol.* 2025); Janssen's NetLogo replication `punishment/janssen-comses-2223-netlogo/` (GPL-3.0: readings only) | the shapes hold but not the reach; the baseline is unstated and the caption contradicts the legend; the figures fit twice the stated conflict rate (14 of 14 curves; the text's rate 2); continuous traits are not similar; Cooney's dip appears under every victory rule |
+| 28 | `zi` | `zi-traders/gode-sunder-1993-jpe-zero-intelligence-traders.pdf` (*scan*, read by OCR); the critique `zi-traders/cliff-1997-hp-minimal-intelligence-agents-for-bargaining.pdf` (with its C source) | the five markets recovered from the figures (Table 2 pins four exactly); efficiencies and dispersions reproduce, but only with enough shouts — "30 seconds" is never translated; Cliff's predictions miss the box markets and his 233⅓ is not his formula's; ZIP converges; his code's momentum is not his text's |
 
 ## Queue
 
@@ -43,10 +44,9 @@ worth doing; "size" is a guess at the milestone's scale.
 
 | # | Model | Original | Critique or follow-up | Size | Shape |
 |---|---|---|---|---|---|
-| 1 | Zero-intelligence traders | `zi-traders/gode-sunder-1993-jpe-zero-intelligence-traders.pdf` (*scan*) | `zi-traders/cliff-1997-hp-minimal-intelligence-agents-for-bargaining.pdf` (Cliff's HP Labs report: where ZI-C fails, and ZIP traders) | medium | new kind: a double auction with values and costs (Sugarscape's `PriceRule::Random` is only the bilateral analog) |
-| 2 | Bali water temples | `bali/lansing-kremer-1993-am-anthropologist-balinese-water-temples.pdf` (*scan*) | `bali/janssen-2007-agricultural-systems-coordination-in-irrigation.pdf`; github.com/mars0i/bali (NetLogo, no license: reference only); Janssen's CoMSES model 2221 for the watershed data (check its license first) | large | new kind on a watershed: subaks, dams, rain, pests |
-| 3 | Emergence of firms | `firms/axtell-1999-emergence-of-firms.pdf` (108 pp) | — | large | new kind (team formation, power-law firm sizes) |
-| 4 | Emergent actors in world politics | `geopolitics/cederman-1997-emergent-actors-in-world-politics.pdf` (274-page book) | — | very large | new kind (states on a grid, conquest, nationalism) |
+| 1 | Bali water temples | `bali/lansing-kremer-1993-am-anthropologist-balinese-water-temples.pdf` (*scan*) | `bali/janssen-2007-agricultural-systems-coordination-in-irrigation.pdf`; github.com/mars0i/bali (NetLogo, no license: reference only); Janssen's CoMSES model 2221 for the watershed data (check its license first) | large | new kind on a watershed: subaks, dams, rain, pests |
+| 2 | Emergence of firms | `firms/axtell-1999-emergence-of-firms.pdf` (108 pp) | — | large | new kind (team formation, power-law firm sizes) |
+| 3 | Emergent actors in world politics | `geopolitics/cederman-1997-emergent-actors-in-world-politics.pdf` (274-page book) | — | very large | new kind (states on a grid, conquest, nationalism) |
 
 ## Wanted
````

Modify `docs/superpowers/specs/2026-09-28-zi-traders-design.md` (a unified diff against the branch's base: save it and `git apply` it, or make the edits by hand):

````diff
diff --git a/docs/superpowers/specs/2026-09-28-zi-traders-design.md b/docs/superpowers/specs/2026-09-28-zi-traders-design.md
index 33a4a01..e1dfa6b 100644
--- a/docs/superpowers/specs/2026-09-28-zi-traders-design.md
+++ b/docs/superpowers/specs/2026-09-28-zi-traders-design.md
@@ -122,3 +122,15 @@ The presets menu gains a **Zero-Intelligence Traders** group and the Compare ent
 ## Docs
 
 README: a Zero-Intelligence Traders section (the model, how the scans and the code were read, the stated choices, switches, presets, sweeps, and the findings). `docs/papers.md`: the milestone's row (with Cliff as the critique); the Queue's first entry removed; roadmap: Milestone 28 done.
+
+## Amendments (implementation planning)
+
+Found while implementing and measuring, in the plan `docs/superpowers/plans/2026-09-28-zi-traders.md`:
+
+- **ZIP converges far better than the prototype showed.** Implemented to Cliff's code, the daily mean prices (50 seeds) reach within 2 of P₀ by day 10 in all four markets — excess demand 124, 144, 167, 182, 188 … 198; excess supply 244 … 202 — where the throwaway prototype had 133 and 232 (it differed from the code in a detail not traced). The momentum discrepancy is smaller than planned: the text's reading converges a day or two sooner (excess demand: 179 against 167 on day 3).
+- **NYSE rules for ZIP off.** Cliff's ZIP control file has NYSE off (his ZI-C runs had it on); with it on, a ZIP day ends within a few dozen shouts (no one can beat the quote). The ZIP presets and claims turn it off.
+- **`sellers_only` under the book** is allowed: nothing trades (no one bids). Every live field must accept a change on its own.
+- **ZIP's A** — $0.05 on Cliff's $4 range — is scaled to the price range (5 at 400, 2.5 at 200).
+- **Statistics:** the per-period series are held as `last_*` (the last completed period) and `avg_*` (the mean over completed periods), and the sweeps read those; one tick is one shout; the inspection carries `agent: null` like the others.
+- **Market 5:** 86.7 % ZI-U efficiency against Table 2's 86.0 — approximate, as planned.
+- **Measured with the implementation** (the survey, 16 claims: 14 hold, 2 fail): Tables 1–3, footnote 5's ordering (magnitudes 0.91 and 0.85 against .74 and .42), the period length (44–86 % at 100 shouts), efficiency under Cliff's mechanism, the critique in Gode and Sunder's mechanism, ZIP convergence, the approach from below, dispersion a tenth, the shifts, the retail market and the momentum effect hold; Cliff's box-market predictions (138 against 125; 250 against 260) fail, and so does ZIP's "often averaging 100 %" in the symmetric market (98.9).
````

- [ ] **Step 2: Verify everything**

Run: `cargo fmt --all --check && cargo clippy --all-targets -- -D warnings && cargo test --workspace && wasm-pack test --node crates/sugarscape-wasm && (cd web && npm run build && npm test) && (cd survey && cargo build --release && ./target/release/survey --only zi.)`
Expected: all green (1 057 Rust tests, 52 WASM, 733 web); the survey's verdicts as in Task 4.

- [ ] **Step 3 (controller): the full browser pass** — Task 3's Step 6 list again, on the final build.

- [ ] **Step 4: Commit**

```bash
git add README.md docs/roadmap.md docs/papers.md docs/superpowers/specs/2026-09-28-zi-traders-design.md
```
```bash
git commit -m "Document Zero-Intelligence Traders, how the scan and Cliff's code were read, and what reproduces; mark milestone 28 done

Claude-Session: https://claude.ai/code/session_01CDy35L1s2fYHpcryzQj8w4"
```

## Self-review (planning)

- **Spec coverage:** Architecture, Config, Step, Statistics, Views, Presets, Compare, Experiments and CLI → Tasks 1–3; Survey → Task 4; Docs → Task 5; departures in the spec's Amendments.
- **Placeholders:** none; every file is given in full or as a diff against `ff712fb`.
- **Types:** `ZiInspection`, `ZiTraderView` and `ZiTrade` (types.ts) match `ZiInspection`, `TraderView` and `Trade` (world.rs); `isZiView` tests `trade` and `supply`, which no other inspection has.
