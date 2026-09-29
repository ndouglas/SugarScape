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
    /// The period under way (1-based), its shouts, its completed sessions,
    /// and failures in a row.
    period: u64,
    shouts: u32,
    sessions: u32,
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
            sessions: 0,
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
        // Cliff: ZIP traders are barred unless their price beats the quote;
        // ZI traders only when their limit cannot reach it (`limit > best_offer`
        // bars a seller, so a limit at the quote may still shout).
        if self.config.strategy == Strategy::Zip {
            let own = t.zip_price();
            if t.buyer {
                self.bid.is_none_or(|(p, _)| own > f64::from(p))
            } else {
                self.ask.is_none_or(|(p, _)| own < f64::from(p))
            }
        } else if t.buyer {
            self.bid.is_none_or(|(p, _)| t.limit() >= p)
        } else {
            self.ask.is_none_or(|(p, _)| t.limit() <= p)
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
        // A session ends with a trade or 100 failures in a row (Cliff's code).
        let session_over = traded.is_some() || self.fails + 1 >= MAX_FAILS;
        if traded.is_some() {
            self.fails = 0;
        } else {
            self.fails += 1;
        }
        let over = match self.config.period_end {
            PeriodEnd::Shouts => self.shouts >= self.config.shouts,
            PeriodEnd::Failures => blocked || self.fails >= MAX_FAILS,
            PeriodEnd::Sessions => {
                if session_over && !blocked {
                    self.sessions += 1;
                    if traded.is_none() {
                        self.fails = 0;
                    }
                }
                blocked || self.sessions >= self.config.sessions
            }
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
        self.sessions = 0;
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
        // Buyers valued below 300 may not bid; those at 300 and 325 may.
        let able: Vec<usize> = (0..w.buyers).filter(|&i| w.able(i)).collect();
        assert_eq!(able, [9, 10]);
        w.config.nyse = false;
        assert!((0..w.buyers).all(|i| w.able(i)));
    }

    #[test]
    fn a_day_runs_its_sessions_each_ending_in_a_trade_or_100_failures() {
        // Cliff's code: a session ends with a trade or 100 failed shouts; a
        // day with `sessions` of them (or when the side to shout has no one
        // able).
        let mut w = world(|c| {
            cliff(c);
            c.market = Market::Symmetric;
            c.period_end = PeriodEnd::Sessions;
            c.sessions = 3;
        });
        w.run_periods(1);
        assert!(w.periods[0].volume <= 3, "{}", w.periods[0].volume);
        // A market where nothing can trade: three sessions of 100 failures.
        let mut dead = world(|c| {
            cliff(c);
            c.market = Market::Custom;
            c.buyers = vec![vec![100]];
            c.sellers = vec![vec![300]];
            c.nyse = false;
            c.period_end = PeriodEnd::Sessions;
            c.sessions = 3;
        });
        dead.run_periods(1);
        assert_eq!(dead.tick, 300);
    }

    #[test]
    fn nyse_lets_a_zi_trader_at_the_quote_shout() {
        // Cliff bars a ZI seller only when its limit is above the best offer.
        let mut w = world(|c| {
            cliff(c);
            c.market = Market::FlatSupply;
        });
        w.ask = Some((200, w.buyers));
        assert!((w.buyers..w.traders.len()).all(|i| w.able(i)));
        w.ask = Some((199, w.buyers));
        assert!((w.buyers..w.traders.len()).all(|i| !w.able(i)));
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
