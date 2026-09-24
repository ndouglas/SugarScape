//! Per-tick summary statistics (Chapter II's Gini coefficient and Lorenz
//! curve, plus population and trait means).

use serde::Serialize;

use crate::config::Config;
use crate::world::World;

pub const SERIES: [&str; 24] = [
    "population",
    "gini",
    "mean_wealth",
    "mean_vision",
    "mean_metabolism",
    "blue_fraction",
    "births",
    "deaths",
    "mean_log_price",
    "sd_log_price",
    "trade_volume",
    "sugar_traded",
    "loans_made",
    "amount_lent",
    "defaults",
    "debt_outstanding",
    "mean_foresight",
    "mean_spice",
    "mean_spice_metabolism",
    "infected_fraction",
    "mean_diseases",
    "diseases_in_circulation",
    "new_infections",
    "trade_pairs",
];

/// Per-good statistics for one tick.
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct GoodStats {
    pub mean_holding: f64,
    /// Mean genetic metabolism of the good (not counting disease fees).
    pub mean_metabolism: f64,
    /// Units exchanged this tick: `amount` where it was bought, `amount ×
    /// price` where it paid.
    pub traded: f64,
}

/// `SERIES`, then `mean_holding_I`, `mean_metabolism_I`, `traded_I` for
/// each good, then `mean_pollution_K` for each pollutant, then
/// `group_share_K` for each group.
pub fn series_names(config: &Config) -> Vec<String> {
    let mut names: Vec<String> = SERIES.iter().map(|s| s.to_string()).collect();
    for i in 0..config.goods.len() {
        names.push(format!("mean_holding_{i}"));
        names.push(format!("mean_metabolism_{i}"));
        names.push(format!("traded_{i}"));
    }
    for k in 0..config.pollution.pollutants.len() {
        names.push(format!("mean_pollution_{k}"));
    }
    for k in 0..config.culture.groups.len() {
        names.push(format!("group_share_{k}"));
    }
    names
}

#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct Snapshot {
    pub tick: u64,
    pub population: u32,
    pub gini: f64,
    pub mean_wealth: f64,
    pub mean_vision: f64,
    pub mean_metabolism: f64,
    pub blue_fraction: f64,
    /// Sexual births this tick; replacements (rule R) are not births.
    pub births: u32,
    pub deaths: u32,
    pub mean_log_price: f64,
    pub sd_log_price: f64,
    pub trade_volume: u32,
    pub sugar_traded: f64,
    pub loans_made: u32,
    pub amount_lent: f64,
    pub defaults: u32,
    pub debt_outstanding: f64,
    pub mean_foresight: f64,
    pub mean_spice: f64,
    pub mean_spice_metabolism: f64,
    /// Share of living agents carrying at least one disease.
    pub infected_fraction: f64,
    pub mean_diseases: f64,
    /// Distinct diseases carried by anyone.
    pub diseases_in_circulation: u32,
    /// Infections this tick (transmissions and outbreaks).
    pub new_infections: u32,
    /// Distinct pairs of goods exchanged this tick.
    pub trade_pairs: u32,
    pub goods: Vec<GoodStats>,
    /// Mean level of each pollutant over all sites.
    pub pollution: Vec<f64>,
    /// Share of living agents in each group (`culture.groups`); 0 with no
    /// agents.
    pub groups: Vec<f64>,
}

impl Snapshot {
    pub fn of(world: &World) -> Self {
        let n = world.population();
        let mean = |f: &dyn Fn(&crate::agent::Agent) -> f64| {
            if n == 0 {
                0.0
            } else {
                world.agents().map(f).sum::<f64>() / n as f64
            }
        };
        let w = wealths(world);

        // Calculate mean and sd of log prices from trades
        let events = world.events();
        // Chapter IV's price and quantity series are for the pair (0, 1).
        let first_pair: Vec<&crate::world::Trade> =
            events.trades.iter().filter(|t| t.goods == (0, 1)).collect();
        let logs: Vec<f64> = first_pair.iter().map(|t| t.price.ln()).collect();
        let (mean_log_price, sd_log_price) = if logs.is_empty() {
            (0.0, 0.0)
        } else {
            let m = logs.iter().sum::<f64>() / logs.len() as f64;
            let var = logs.iter().map(|x| (x - m).powi(2)).sum::<f64>() / logs.len() as f64;
            (m, var.sqrt())
        };

        let mut goods: Vec<GoodStats> = (0..world.config.goods.len())
            .map(|i| GoodStats {
                mean_holding: mean(&|a| a.holdings[i]),
                mean_metabolism: mean(&|a| f64::from(a.metabolism[i])),
                traded: 0.0,
            })
            .collect();
        for t in &events.trades {
            goods[t.goods.0].traded += t.amount;
            goods[t.goods.1].traded += t.amount * t.price;
        }
        let trade_pairs = events
            .trades
            .iter()
            .map(|t| t.goods)
            .collect::<std::collections::BTreeSet<_>>()
            .len() as u32;
        let sites = world.sites.len() as f64;
        let pollution = (0..world.config.pollution.pollutants.len())
            .map(|k| world.sites.iter().map(|s| s.pollution[k]).sum::<f64>() / sites)
            .collect();
        let groups = &world.config.culture.groups;
        let mut members = vec![0usize; groups.len()];
        for a in world.agents() {
            if let Some(m) = members.get_mut(a.group(groups)) {
                *m += 1;
            }
        }
        let group_shares = members
            .iter()
            .map(|&m| if n == 0 { 0.0 } else { m as f64 / n as f64 })
            .collect();

        Self {
            tick: world.tick,
            population: n as u32,
            gini: gini(&w),
            mean_wealth: mean(&|a| a.holdings[0]),
            mean_vision: mean(&|a| f64::from(a.vision)),
            mean_metabolism: mean(&|a| f64::from(a.metabolism[0])),
            // Group 0 is Blue under the default groups (Decision 6).
            blue_fraction: mean(&|a| if a.group(groups) == 0 { 1.0 } else { 0.0 }),
            births: world.events().births,
            deaths: world.events().deaths.len() as u32,
            mean_log_price,
            sd_log_price,
            trade_volume: events.trades.len() as u32,
            sugar_traded: first_pair.iter().map(|t| t.amount).sum(),
            loans_made: events.loans_made,
            amount_lent: events.amount_lent,
            defaults: events.defaults,
            debt_outstanding: world.loans().map(|l| l.due).sum(),
            mean_foresight: mean(&|a| f64::from(a.foresight)),
            mean_spice: mean(&|a| a.holdings[1]),
            mean_spice_metabolism: mean(&|a| f64::from(a.metabolism[1])),
            infected_fraction: mean(&|a| if a.diseases.is_empty() { 0.0 } else { 1.0 }),
            mean_diseases: mean(&|a| a.diseases.len() as f64),
            diseases_in_circulation: world
                .agents()
                .flat_map(|a| a.diseases.iter().copied())
                .collect::<std::collections::BTreeSet<_>>()
                .len() as u32,
            new_infections: events.infections.len() as u32,
            trade_pairs,
            goods,
            pollution,
            groups: group_shares,
        }
    }

    pub fn value(&self, name: &str) -> Option<f64> {
        Some(match name {
            "tick" => self.tick as f64,
            "population" => f64::from(self.population),
            "gini" => self.gini,
            "mean_wealth" => self.mean_wealth,
            "mean_vision" => self.mean_vision,
            "mean_metabolism" => self.mean_metabolism,
            "blue_fraction" => self.blue_fraction,
            "births" => f64::from(self.births),
            "deaths" => f64::from(self.deaths),
            "mean_log_price" => self.mean_log_price,
            "sd_log_price" => self.sd_log_price,
            "trade_volume" => f64::from(self.trade_volume),
            "sugar_traded" => self.sugar_traded,
            "loans_made" => f64::from(self.loans_made),
            "amount_lent" => self.amount_lent,
            "defaults" => f64::from(self.defaults),
            "debt_outstanding" => self.debt_outstanding,
            "mean_foresight" => self.mean_foresight,
            "mean_spice" => self.mean_spice,
            "mean_spice_metabolism" => self.mean_spice_metabolism,
            "infected_fraction" => self.infected_fraction,
            "mean_diseases" => self.mean_diseases,
            "diseases_in_circulation" => f64::from(self.diseases_in_circulation),
            "new_infections" => f64::from(self.new_infections),
            "trade_pairs" => f64::from(self.trade_pairs),
            _ => {
                let index = |prefix: &str| name.strip_prefix(prefix)?.parse::<usize>().ok();
                if let Some(i) = index("mean_holding_") {
                    return self.goods.get(i).map(|g| g.mean_holding);
                }
                if let Some(i) = index("mean_metabolism_") {
                    return self.goods.get(i).map(|g| g.mean_metabolism);
                }
                if let Some(i) = index("traded_") {
                    return self.goods.get(i).map(|g| g.traded);
                }
                if let Some(k) = index("mean_pollution_") {
                    return self.pollution.get(k).copied();
                }
                if let Some(k) = index("group_share_") {
                    return self.groups.get(k).copied();
                }
                return None;
            }
        })
    }
}

#[derive(Clone, Debug, Default)]
pub struct Stats {
    history: Vec<Snapshot>,
}

impl Stats {
    pub fn push(&mut self, s: Snapshot) {
        self.history.push(s);
    }

    pub fn latest(&self) -> Option<&Snapshot> {
        self.history.last()
    }

    pub fn history(&self) -> &[Snapshot] {
        &self.history
    }

    /// The full history of one series (or `"tick"`), or `None` if unknown.
    pub fn series(&self, name: &str) -> Option<Vec<f64>> {
        if self.history.is_empty() {
            return Snapshot::default().value(name).map(|_| Vec::new());
        }
        self.history.iter().map(|s| s.value(name)).collect()
    }
}

pub fn wealths(world: &World) -> Vec<f64> {
    world.agents().map(|a| a.holdings[0]).collect()
}

/// G = 2·Σ i·x₍ᵢ₎ / (n·Σx) − (n+1)/n over ascending wealth, i from 1.
pub fn gini(values: &[f64]) -> f64 {
    let n = values.len();
    let total: f64 = values.iter().sum();
    if n == 0 || total <= 0.0 {
        return 0.0;
    }
    let mut v = values.to_vec();
    v.sort_by(f64::total_cmp);
    let weighted: f64 = v.iter().enumerate().map(|(i, x)| (i + 1) as f64 * x).sum();
    let n = n as f64;
    (2.0 * weighted / (n * total) - (n + 1.0) / n).max(0.0)
}

/// Share of total wealth held by the poorest k/(points−1) of agents, for
/// k = 0..points.
pub fn lorenz(values: &[f64], points: usize) -> Vec<f64> {
    assert!(points >= 2);
    let mut v = values.to_vec();
    v.sort_by(f64::total_cmp);
    let total: f64 = v.iter().sum();
    let mut cumulative = vec![0.0];
    for x in &v {
        cumulative.push(cumulative.last().unwrap() + x);
    }
    (0..points)
        .map(|k| {
            if total <= 0.0 {
                return k as f64 / (points - 1) as f64;
            }
            let m = (k as f64 / (points - 1) as f64 * v.len() as f64).round() as usize;
            cumulative[m] / total
        })
        .collect()
}

/// `bins` equal-width bins over [0, max]; returns (bin width, counts).
pub fn histogram(values: &[f64], bins: usize) -> (f64, Vec<f64>) {
    assert!(bins >= 1);
    let max = values.iter().copied().fold(0.0, f64::max);
    let width = if max > 0.0 { max / bins as f64 } else { 1.0 };
    let mut counts = vec![0.0; bins];
    for &x in values {
        let i = ((x.max(0.0) / width) as usize).min(bins - 1);
        counts[i] += 1.0;
    }
    (width, counts)
}

/// Aggregate sugar supply and demand over 41 log-spaced prices in [0.1, 10]
/// (spice per sugar), the interpolated market-clearing point, and the tick's
/// actual geometric-mean price and sugar traded (NaN where undefined).
#[derive(Clone, Debug, PartialEq)]
pub struct SupplyDemand {
    pub prices: Vec<f64>,
    pub demand: Vec<f64>,
    pub supply: Vec<f64>,
    pub equilibrium_price: f64,
    pub equilibrium_quantity: f64,
    pub actual_price: f64,
    pub actual_quantity: f64,
}

pub fn supply_demand(world: &World) -> SupplyDemand {
    let prices: Vec<f64> = (0..41)
        .map(|k| 10f64.powf(-1.0 + k as f64 / 20.0))
        .collect();
    let mut demand = vec![0.0; prices.len()];
    let mut supply = vec![0.0; prices.len()];
    let fee = world.config.disease.active_fee();
    for a in world.agents() {
        let (m1, m2) = (
            a.effective_metabolism(0, fee),
            a.effective_metabolism(1, fee),
        );
        for (k, &p) in prices.iter().enumerate() {
            let excess =
                crate::econ::sugar_demand(p, a.holdings[0], a.holdings[1], m1, m2) - a.holdings[0];
            if excess > 0.0 {
                demand[k] += excess
            } else {
                supply[k] -= excess
            }
        }
    }
    let (mut equilibrium_price, mut equilibrium_quantity) = (f64::NAN, f64::NAN);
    for k in 0..prices.len() - 1 {
        let (e0, e1) = (demand[k] - supply[k], demand[k + 1] - supply[k + 1]);
        if e0 >= 0.0 && e1 <= 0.0 && e0 != e1 {
            let t = e0 / (e0 - e1);
            equilibrium_price = (prices[k].ln() + t * (prices[k + 1].ln() - prices[k].ln())).exp();
            equilibrium_quantity = demand[k] + t * (demand[k + 1] - demand[k]);
            break;
        }
    }
    let trades: Vec<_> = world
        .events()
        .trades
        .iter()
        .filter(|t| t.goods == (0, 1))
        .collect();
    let (actual_price, actual_quantity) = if trades.is_empty() {
        (f64::NAN, f64::NAN)
    } else {
        let m = trades.iter().map(|t| t.price.ln()).sum::<f64>() / trades.len() as f64;
        (m.exp(), trades.iter().map(|t| t.amount).sum())
    };
    SupplyDemand {
        prices,
        demand,
        supply,
        equilibrium_price,
        equilibrium_quantity,
        actual_price,
        actual_quantity,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use crate::config::Pollutant;

    #[test]
    fn per_good_and_per_pollutant_series() {
        use crate::testkit::*;
        use crate::world::Trade;
        let mut w = blank_world(5, 5);
        add_goods(&mut w.config, 3);
        w.config.pollution.pollutants.push(Pollutant {
            name: "runoff".into(),
            production: vec![0.0; 3],
            consumption: vec![0.0; 3],
            devalues: vec![false; 3],
        });
        let a = spawn(&mut w, 0, 0);
        let b = spawn(&mut w, 1, 0);
        w.agent_mut(a).unwrap().holdings[2] = 4.0;
        w.agent_mut(b).unwrap().metabolism[2] = 3;
        w.sites[0].pollution[1] = 25.0;
        let t = |goods, price, amount| Trade {
            buyer: a,
            seller: b,
            goods,
            price,
            amount,
        };
        w.events.trades = vec![
            t((0, 1), 2.0, 1.0),
            t((1, 2), 0.5, 4.0),
            t((1, 2), 0.5, 2.0),
        ];
        let s = Snapshot::of(&w);
        assert_eq!(s.goods.len(), 3);
        assert_eq!(
            (s.goods[2].mean_holding, s.goods[2].mean_metabolism),
            (2.0, 1.5)
        );
        assert_eq!(
            s.goods.iter().map(|g| g.traded).collect::<Vec<_>>(),
            vec![1.0, 2.0 + 4.0 + 2.0, 2.0 + 1.0],
            "good j counts amount × price"
        );
        assert_eq!(s.trade_pairs, 2);
        assert_eq!(s.pollution, vec![0.0, 1.0]);
        assert_eq!(s.value("mean_holding_2"), Some(2.0));
        assert_eq!(s.value("traded_1"), Some(8.0));
        assert_eq!(s.value("mean_pollution_1"), Some(1.0));
        assert_eq!(s.value("mean_holding_3"), None);
        assert_eq!(s.value("mean_spice"), Some(10.0), "good 1, as before");
        let names = series_names(&w.config);
        assert_eq!(names.len(), SERIES.len() + 3 * 3 + 2 + 2);
        assert_eq!(
            &names[SERIES.len()..SERIES.len() + 3],
            ["mean_holding_0", "mean_metabolism_0", "traded_0"]
        );
        assert_eq!(names[names.len() - 3], "mean_pollution_1");
        assert_eq!(names.last().unwrap(), "group_share_1");
        for name in &names {
            assert!(s.value(name).is_some(), "{name}");
        }
    }

    #[test]
    fn gini_of_equal_wealth_is_zero_and_of_total_concentration_is_high() {
        assert_eq!(gini(&[5.0, 5.0, 5.0, 5.0]), 0.0);
        assert!((gini(&[0.0, 0.0, 0.0, 1.0]) - 0.75).abs() < 1e-12);
        assert_eq!(gini(&[]), 0.0);
        assert_eq!(
            gini(&[3.0, 1.0, 2.0]),
            gini(&[1.0, 2.0, 3.0]),
            "order-independent"
        );
    }

    #[test]
    fn lorenz_curve_of_equal_wealth_is_the_diagonal() {
        let l = lorenz(&[2.0, 2.0, 2.0, 2.0], 5);
        assert_eq!(l, vec![0.0, 0.25, 0.5, 0.75, 1.0]);
        assert_eq!(
            lorenz(&[0.0, 0.0, 0.0, 1.0], 5),
            vec![0.0, 0.0, 0.0, 0.0, 1.0]
        );
    }

    #[test]
    fn histogram_bins_from_zero_to_max() {
        let (width, counts) = histogram(&[0.5, 1.5, 3.9, 4.0], 4);
        assert_eq!(width, 1.0);
        assert_eq!(counts, vec![1.0, 1.0, 0.0, 2.0]);
    }

    #[test]
    fn world_records_a_snapshot_per_tick() {
        let mut w = World::new(Config::default(), 3).unwrap();
        assert_eq!(w.stats.history().len(), 1);
        assert_eq!(w.stats.latest().unwrap().population, 400);
        w.run(3);
        assert_eq!(w.stats.history().len(), 4);
        assert_eq!(w.stats.series("tick").unwrap(), vec![0.0, 1.0, 2.0, 3.0]);
        assert_eq!(w.stats.series("population").unwrap().len(), 4);
        assert!(w.stats.series("nonsense").is_none());
        let s = w.stats.latest().unwrap();
        assert_eq!(s.population as usize, w.population());
        assert!((1.0..=6.0).contains(&s.mean_vision));
        for name in SERIES {
            assert!(s.value(name).is_some(), "{name}");
        }
        assert!(
            w.stats.series("mean_holding_0").is_some()
                && w.stats.series("mean_holding_1").is_none()
        );
    }

    #[test]
    fn trade_and_credit_series_come_from_the_tick_events() {
        use crate::testkit::*;
        use crate::world::Trade;
        let mut w = blank_world(5, 5);
        add_goods(&mut w.config, 2);
        w.events.trades = vec![
            Trade {
                buyer: 1,
                seller: 2,
                goods: (0, 1),
                price: 2.0,
                amount: 1.0,
            },
            Trade {
                buyer: 1,
                seller: 2,
                goods: (0, 1),
                price: 0.5,
                amount: 2.0,
            },
        ];
        w.events.loans_made = 3;
        let s = Snapshot::of(&w);
        assert!(s.mean_log_price.abs() < 1e-12, "ln 2 and ln ½ average to 0");
        assert!((s.sd_log_price - 2f64.ln()).abs() < 1e-12);
        assert_eq!((s.trade_volume, s.sugar_traded, s.loans_made), (2, 3.0, 3));
        for name in SERIES {
            assert!(s.value(name).is_some(), "{name}");
        }
    }

    #[test]
    fn prices_come_from_the_first_pair_and_volume_from_every_pair() {
        use crate::testkit::*;
        use crate::world::Trade;
        let mut w = blank_world(5, 5);
        add_goods(&mut w.config, 3);
        let t = |goods, price, amount| Trade {
            buyer: 1,
            seller: 2,
            goods,
            price,
            amount,
        };
        w.events.trades = vec![
            t((0, 1), 2.0, 1.0),
            t((1, 2), 8.0, 1.0),
            t((0, 1), 2.0, 3.0),
        ];
        let s = Snapshot::of(&w);
        assert!((s.mean_log_price - 2f64.ln()).abs() < 1e-12);
        assert_eq!(s.sd_log_price, 0.0);
        assert_eq!((s.trade_volume, s.sugar_traded), (3, 4.0));
    }

    #[test]
    fn symmetric_market_clears_near_one() {
        use crate::testkit::*;
        let mut w = blank_world(5, 5);
        add_goods(&mut w.config, 2);
        for (x, sugar, spice) in [(0, 30.0, 10.0), (1, 10.0, 30.0)] {
            let id = spawn(&mut w, x, 0);
            let a = w.agent_mut(id).unwrap();
            (
                a.holdings[0],
                a.holdings[1],
                a.metabolism[0],
                a.metabolism[1],
            ) = (sugar, spice, 1, 1);
        }
        let sd = supply_demand(&w);
        assert_eq!(sd.prices.len(), 41);
        assert!(
            (sd.equilibrium_price - 1.0).abs() < 0.05,
            "{}",
            sd.equilibrium_price
        );
        assert!((sd.equilibrium_quantity - 10.0).abs() < 0.5);
        assert!(sd.actual_price.is_nan(), "no trades this tick");
    }

    #[test]
    fn supply_and_demand_use_effective_metabolism() {
        use crate::testkit::*;
        let mut w = blank_world(5, 5);
        add_goods(&mut w.config, 2);
        w.config.disease.enabled = true;
        for (x, sugar, spice) in [(0, 30.0, 10.0), (1, 10.0, 30.0)] {
            let id = spawn(&mut w, x, 0);
            let a = w.agent_mut(id).unwrap();
            (
                a.holdings[0],
                a.holdings[1],
                a.metabolism[0],
                a.metabolism[1],
            ) = (sugar, spice, 1, 3);
        }
        let healthy = supply_demand(&w);
        for a in w.agent_ids() {
            w.agent_mut(a).unwrap().diseases = vec![0, 1];
        }
        let sick = supply_demand(&w);
        assert_ne!(healthy.demand, sick.demand, "weights (1, 3) became (3, 5)");
    }

    #[test]
    fn group_shares_follow_the_configured_groups() {
        use crate::agent::Tags;
        use crate::testkit::*;
        let mut w = blank_world(5, 5);
        w.config.culture.groups = crate::config::three_tribes(11);
        for (x, bits) in [(0, 0u64), (1, 0), (2, 0b111_1111), (3, u64::MAX)] {
            let id = spawn(&mut w, x, 0);
            w.agent_mut(id).unwrap().tags = Tags::new(bits, 11);
        }
        // Zeros 11, 11 (Red 8–11), 4 (Green 4–7), 0 (Blue 0–3).
        let s = Snapshot::of(&w);
        assert_eq!(s.groups, vec![0.25, 0.25, 0.5]);
        assert_eq!(s.blue_fraction, 0.25, "the share of group 0");
        assert_eq!(s.value("group_share_2"), Some(0.5));
        assert_eq!(s.value("group_share_3"), None);
        let names = series_names(&w.config);
        assert_eq!(
            &names[names.len() - 3..],
            ["group_share_0", "group_share_1", "group_share_2"]
        );
        w.config.culture.groups = crate::config::default_groups(11);
        let s = Snapshot::of(&w);
        assert_eq!(s.groups, vec![0.5, 0.5]);
        assert_eq!(s.blue_fraction, 0.5, "Blue: zeros outnumber ones");
        let empty = Snapshot::of(&blank_world(5, 5));
        assert_eq!(empty.groups, vec![0.0, 0.0]);
    }

    #[test]
    fn disease_series_count_carriers_and_new_infections() {
        use crate::testkit::*;
        use crate::world::Infection;
        let mut w = blank_world(5, 5);
        let a = spawn(&mut w, 0, 0);
        let b = spawn(&mut w, 1, 0);
        spawn(&mut w, 2, 0);
        spawn(&mut w, 3, 0);
        w.agent_mut(a).unwrap().diseases = vec![0, 2];
        w.agent_mut(b).unwrap().diseases = vec![2];
        w.events.infections = vec![Infection {
            infector: Some(a),
            infected: b,
            disease: 2,
        }];
        let s = Snapshot::of(&w);
        assert_eq!(s.infected_fraction, 0.5);
        assert_eq!(s.mean_diseases, 0.75);
        assert_eq!(s.diseases_in_circulation, 2);
        assert_eq!(s.new_infections, 1);
        for name in SERIES {
            assert!(s.value(name).is_some(), "{name}");
        }
    }
}
