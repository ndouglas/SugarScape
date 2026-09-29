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
