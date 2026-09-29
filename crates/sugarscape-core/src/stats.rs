//! Per-tick summary statistics (Chapter II's Gini coefficient and Lorenz
//! curve, plus population and trait means).

use serde::Serialize;

use std::ops::Range;

use crate::config::Config;
use crate::world::World;

pub const SERIES: [&str; 25] = [
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
    "gini_total",
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

/// goods[0]'s peaks when there are at least two (the patch series' condition).
fn patches(config: &Config) -> Option<&[crate::config::Peak]> {
    match &config.goods[0].map {
        crate::config::Map::Peaks { peaks } if peaks.len() >= 2 => Some(peaks),
        _ => None,
    }
}

/// `SERIES`, then `mean_holding_I`, `mean_metabolism_I`, `traded_I` for
/// each good, then `mean_pollution_K` for each pollutant, then
/// `group_share_K` for each group, then (under Axelrod) `distinct_cultures`
/// and `settled`, then (on a two-or-more-peak map) the patch series.
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
    if config.culture.rule == crate::config::CultureKind::Axelrod {
        names.push("distinct_cultures".into());
        names.push("settled".into());
    }
    if patches(config).is_some() {
        for s in [
            "on_first_patch",
            "on_other_patches",
            "off_patch",
            "first_patch_share",
        ] {
            names.push(s.into());
        }
    }
    if config.memory.span > 0 {
        for s in [
            "remembering",
            "remembered_moves",
            "belief_error",
            "stale_choices",
            "wealth_rememberers",
            "wealth_others",
            "wealth_advantage",
        ] {
            names.push(s.into());
        }
    }
    if config.truffles.share > 0.0 {
        for s in ["truffles_found", "truffles_by_rememberers"] {
            names.push(s.into());
        }
    }
    if config.decision.rule == crate::config::DecisionRule::Goap {
        for s in [
            "replans",
            "mean_plan_length",
            "fallbacks",
            "plans_remembered",
        ] {
            names.push(s.into());
        }
    }
    if config.decision.rule == crate::config::DecisionRule::Mvt {
        for s in ["replans", "mean_rate"] {
            names.push(s.into());
        }
    }
    if config.caching.is_on() {
        for s in ["cached", "buried", "dug", "recovery", "mean_cache_age"] {
            names.push(s.into());
        }
    }
    if config.central.enabled {
        for s in ["mean_load", "trips"] {
            names.push(s.into());
        }
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
    /// Gini coefficient of total wealth (every good's holdings summed);
    /// equals `gini` in a one-good world.
    pub gini_total: f64,
    pub goods: Vec<GoodStats>,
    /// Mean level of each pollutant over all sites.
    pub pollution: Vec<f64>,
    /// Share of living agents in each group (`culture.groups`); 0 with no
    /// agents.
    pub groups: Vec<f64>,
    /// Under Axelrod's rule: distinct cultures among living agents, and
    /// whether every two share nothing (1) or not (0). Absent otherwise.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub axelrod: Option<AxelrodStats>,
    /// Minds 1's patch counts, present when goods[0]'s map is `peaks` with
    /// at least two peaks.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub patches: Option<PatchStats>,
    /// Minds 3's memory series, present when `memory.span > 0`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memory: Option<MemoryStats>,
    /// Minds 3's truffle series, present when `truffles.share > 0`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub truffles: Option<TruffleStats>,
    /// Minds 4's GOAP series, present when `decision.rule` is `goap`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub goap: Option<GoapStats>,
    /// Minds 4's marginal-value series, present when `decision.rule` is `mvt`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mvt: Option<MvtStats>,
    /// Minds 5's caching series, present when caching is on (`caching.rule
    /// != none` or `caching.capacity > 0`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub caching: Option<CachingStats>,
    /// Minds 5's central-place foraging series, present when
    /// `central.enabled`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub central: Option<CentralStats>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize)]
pub struct AxelrodStats {
    pub distinct_cultures: u32,
    pub settled: bool,
}

/// Minds 1's patch counts: agents on the first peak's patch, on any other's,
/// and on none (`landscape::patch_of`). Present when goods[0]'s map is
/// `peaks` with at least two peaks.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize)]
pub struct PatchStats {
    pub on_first: u32,
    pub on_other: u32,
    pub off: u32,
}

/// Minds 3's memory series (see the module's series list). Shares and means
/// are 0 (not NaN) when their group or denominator is empty.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize)]
pub struct MemoryStats {
    /// Share of living agents who remember.
    pub remembering: f64,
    /// Share of rememberers' choices this tick that targeted a remembered
    /// out-of-sight site.
    pub remembered_moves: f64,
    /// Mean |believed − true| welfare over those remembered choices.
    pub belief_error: f64,
    /// Share of those remembered choices whose true value was below the
    /// believed value when chosen.
    pub stale_choices: f64,
    /// Mean sugar held by rememberers (0 if none are alive).
    pub wealth_rememberers: f64,
    /// Mean sugar held by non-rememberers (0 if none are alive).
    pub wealth_others: f64,
    /// `wealth_rememberers − wealth_others`, 0 unless both groups exist.
    pub wealth_advantage: f64,
}

/// Minds 3's truffle series (see the module's series list).
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize)]
pub struct TruffleStats {
    pub truffles_found: u32,
    pub truffles_by_rememberers: u32,
}

/// Minds 4's GOAP series (see the module's series list). Every ratio is 0
/// when its denominator (living agents, or this tick's plans) is 0.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize)]
pub struct GoapStats {
    /// Plans made this tick per living agent.
    pub replans: f64,
    /// Mean steps over this tick's plans.
    pub mean_plan_length: f64,
    /// Share of living agents who took the rate-choice fallback this tick
    /// (short of `G`, or the search hit `PLAN_LIMIT`).
    pub fallbacks: f64,
    /// Share of this tick's plans with any target from the agent's
    /// remembered entries out of sight.
    pub plans_remembered: f64,
}

/// Minds 4's marginal-value series (see the module's series list).
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize)]
pub struct MvtStats {
    /// Agents that set `leaving` this tick, per living agent.
    pub replans: f64,
    /// Mean ρ over the living.
    pub mean_rate: f64,
}

/// Minds 5's caching series (see the module's series list). Every ratio is 0
/// when its denominator (sugar buried since tick 0, or this tick's digs) is
/// 0. `buried_total` and `dug_total` carry `recovery`'s running sums forward
/// from the previous snapshot (`Stats::latest`); they aren't series
/// themselves.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize)]
pub struct CachingStats {
    /// Σ over living agents' caches, read from the world (not this tick's
    /// events).
    pub cached: f64,
    /// Sugar buried into caches this tick.
    pub buried: f64,
    /// Sugar dug out of caches this tick.
    pub dug: f64,
    /// Cumulative Σ dug ÷ Σ buried since tick 0.
    pub recovery: f64,
    /// Mean age (ticks since a cache's first burial) of this tick's digs.
    pub mean_cache_age: f64,
    buried_total: f64,
    dug_total: f64,
}

/// Minds 5's central-place foraging series (see the module's series list).
/// `delivered`, and so `mean_load`, counts a trip's gross intake, not net of
/// what the agent ate on the way (`minds::central`'s `at_home`). `mean_load`
/// is 0 before the first delivery; `trips` is 0 when nobody is alive.
/// `delivered_total` and `deliveries_total` carry `mean_load`'s running sums
/// forward from the previous snapshot (`Stats::latest`), the way `recovery`
/// does; they aren't series themselves.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize)]
pub struct CentralStats {
    /// Cumulative Σ delivered ÷ Σ deliveries since tick 0.
    pub mean_load: f64,
    /// `deliveries ÷ alive` this tick.
    pub trips: f64,
    delivered_total: f64,
    deliveries_total: f64,
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
        let gini_value = gini(&w);

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
            gini: gini_value,
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
            gini_total: if world.config.goods.len() == 1 {
                gini_value
            } else {
                gini(&total_wealths(world))
            },
            goods,
            pollution,
            groups: group_shares,
            axelrod: (world.config.culture.rule == crate::config::CultureKind::Axelrod).then(
                || {
                    let cultures: Vec<&[u8]> =
                        world.agents().map(|a| a.culture.as_slice()).collect();
                    let (distinct_cultures, settled) = crate::culture::settle(&cultures);
                    AxelrodStats {
                        distinct_cultures,
                        settled,
                    }
                },
            ),
            patches: patches(&world.config).map(|peaks| {
                let (w, h) = (world.config.width, world.config.height);
                let mut p = PatchStats::default();
                for a in world.agents() {
                    match crate::landscape::patch_of(peaks, a.pos.x, a.pos.y, w, h) {
                        Some(0) => p.on_first += 1,
                        Some(_) => p.on_other += 1,
                        None => p.off += 1,
                    }
                }
                p
            }),
            memory: (world.config.memory.span > 0).then(|| {
                let remembering = mean(&|a| if a.remembers { 1.0 } else { 0.0 });
                let moves = events.moves;
                let remembered = events.remembered_moves;
                let remembered_moves = if moves == 0 {
                    0.0
                } else {
                    f64::from(remembered) / f64::from(moves)
                };
                let belief_error = if remembered == 0 {
                    0.0
                } else {
                    events.belief_error_sum / f64::from(remembered)
                };
                let stale_choices = if remembered == 0 {
                    0.0
                } else {
                    f64::from(events.stale_choices) / f64::from(remembered)
                };
                let (mut rem_sum, mut rem_n, mut oth_sum, mut oth_n) = (0.0, 0u32, 0.0, 0u32);
                for a in world.agents() {
                    if a.remembers {
                        rem_sum += a.holdings[0];
                        rem_n += 1;
                    } else {
                        oth_sum += a.holdings[0];
                        oth_n += 1;
                    }
                }
                let wealth_rememberers = if rem_n == 0 {
                    0.0
                } else {
                    rem_sum / f64::from(rem_n)
                };
                let wealth_others = if oth_n == 0 {
                    0.0
                } else {
                    oth_sum / f64::from(oth_n)
                };
                let wealth_advantage = if rem_n == 0 || oth_n == 0 {
                    0.0
                } else {
                    wealth_rememberers - wealth_others
                };
                MemoryStats {
                    remembering,
                    remembered_moves,
                    belief_error,
                    stale_choices,
                    wealth_rememberers,
                    wealth_others,
                    wealth_advantage,
                }
            }),
            truffles: (world.config.truffles.share > 0.0).then_some(TruffleStats {
                truffles_found: events.truffles_found,
                truffles_by_rememberers: events.truffles_by_rememberers,
            }),
            goap: (world.config.decision.rule == crate::config::DecisionRule::Goap).then(|| {
                let alive = n as u32;
                let plans = events.plans;
                let replans = if alive == 0 {
                    0.0
                } else {
                    f64::from(plans) / f64::from(alive)
                };
                let mean_plan_length = if plans == 0 {
                    0.0
                } else {
                    f64::from(events.plan_steps_sum) / f64::from(plans)
                };
                let fallbacks = if alive == 0 {
                    0.0
                } else {
                    f64::from(events.fallback_short + events.fallback_limit) / f64::from(alive)
                };
                let plans_remembered = if plans == 0 {
                    0.0
                } else {
                    f64::from(events.plans_with_remembered) / f64::from(plans)
                };
                GoapStats {
                    replans,
                    mean_plan_length,
                    fallbacks,
                    plans_remembered,
                }
            }),
            mvt: (world.config.decision.rule == crate::config::DecisionRule::Mvt).then(|| {
                let alive = n as u32;
                let replans = if alive == 0 {
                    0.0
                } else {
                    f64::from(events.leaves) / f64::from(alive)
                };
                MvtStats {
                    replans,
                    mean_rate: mean(&|a| a.rate),
                }
            }),
            caching: world.config.caching.is_on().then(|| {
                let cached: f64 = world.agents().flat_map(|a| a.caches.values()).sum();
                let (prev_buried, prev_dug) = world
                    .stats
                    .latest()
                    .and_then(|s| s.caching)
                    .map(|c| (c.buried_total, c.dug_total))
                    .unwrap_or((0.0, 0.0));
                let buried_total = prev_buried + events.buried;
                let dug_total = prev_dug + events.dug;
                let recovery = if buried_total == 0.0 {
                    0.0
                } else {
                    dug_total / buried_total
                };
                let mean_cache_age = if events.digs == 0 {
                    0.0
                } else {
                    events.dig_ages_sum as f64 / f64::from(events.digs)
                };
                CachingStats {
                    cached,
                    buried: events.buried,
                    dug: events.dug,
                    recovery,
                    mean_cache_age,
                    buried_total,
                    dug_total,
                }
            }),
            central: world.config.central.enabled.then(|| {
                let (prev_delivered, prev_deliveries) = world
                    .stats
                    .latest()
                    .and_then(|s| s.central)
                    .map(|c| (c.delivered_total, c.deliveries_total))
                    .unwrap_or((0.0, 0.0));
                let delivered_total = prev_delivered + events.delivered;
                let deliveries_total = prev_deliveries + f64::from(events.deliveries);
                let mean_load = if deliveries_total == 0.0 {
                    0.0
                } else {
                    delivered_total / deliveries_total
                };
                let trips = if n == 0 {
                    0.0
                } else {
                    f64::from(events.deliveries) / n as f64
                };
                CentralStats {
                    mean_load,
                    trips,
                    delivered_total,
                    deliveries_total,
                }
            }),
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
            "gini_total" => self.gini_total,
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
                if let Some(a) = self.axelrod {
                    match name {
                        "distinct_cultures" => return Some(f64::from(a.distinct_cultures)),
                        "settled" => return Some(f64::from(u8::from(a.settled))),
                        _ => {}
                    }
                }
                if let Some(p) = self.patches {
                    match name {
                        "on_first_patch" => return Some(f64::from(p.on_first)),
                        "on_other_patches" => return Some(f64::from(p.on_other)),
                        "off_patch" => return Some(f64::from(p.off)),
                        "first_patch_share" => {
                            let on = p.on_first + p.on_other;
                            return Some(if on == 0 {
                                0.0
                            } else {
                                f64::from(p.on_first) / f64::from(on)
                            });
                        }
                        _ => {}
                    }
                }
                if let Some(m) = self.memory {
                    match name {
                        "remembering" => return Some(m.remembering),
                        "remembered_moves" => return Some(m.remembered_moves),
                        "belief_error" => return Some(m.belief_error),
                        "stale_choices" => return Some(m.stale_choices),
                        "wealth_rememberers" => return Some(m.wealth_rememberers),
                        "wealth_others" => return Some(m.wealth_others),
                        "wealth_advantage" => return Some(m.wealth_advantage),
                        _ => {}
                    }
                }
                if let Some(t) = self.truffles {
                    match name {
                        "truffles_found" => return Some(f64::from(t.truffles_found)),
                        "truffles_by_rememberers" => {
                            return Some(f64::from(t.truffles_by_rememberers))
                        }
                        _ => {}
                    }
                }
                if let Some(g) = self.goap {
                    match name {
                        "replans" => return Some(g.replans),
                        "mean_plan_length" => return Some(g.mean_plan_length),
                        "fallbacks" => return Some(g.fallbacks),
                        "plans_remembered" => return Some(g.plans_remembered),
                        _ => {}
                    }
                }
                if let Some(m) = self.mvt {
                    match name {
                        "replans" => return Some(m.replans),
                        "mean_rate" => return Some(m.mean_rate),
                        _ => {}
                    }
                }
                if let Some(c) = self.caching {
                    match name {
                        "cached" => return Some(c.cached),
                        "buried" => return Some(c.buried),
                        "dug" => return Some(c.dug),
                        "recovery" => return Some(c.recovery),
                        "mean_cache_age" => return Some(c.mean_cache_age),
                        _ => {}
                    }
                }
                if let Some(c) = self.central {
                    match name {
                        "mean_load" => return Some(c.mean_load),
                        "trips" => return Some(c.trips),
                        _ => {}
                    }
                }
                return None;
            }
        })
    }
}

/// One tick's statistics of any model: its tick and its series by name.
pub trait Series {
    fn tick(&self) -> u64;
    /// Series `name` (or `"tick"`), or `None` if the model has no such series.
    fn value(&self, name: &str) -> Option<f64>;
}

impl Series for Snapshot {
    fn tick(&self) -> u64 {
        self.tick
    }

    fn value(&self, name: &str) -> Option<f64> {
        Snapshot::value(self, name)
    }
}

/// A model's statistics history, one snapshot per tick from 0.
#[derive(Clone, Debug)]
pub struct Stats<S = Snapshot> {
    history: Vec<S>,
}

impl<S> Default for Stats<S> {
    fn default() -> Self {
        Self {
            history: Vec::new(),
        }
    }
}

impl<S: Series + Default> Stats<S> {
    pub fn push(&mut self, s: S) {
        self.history.push(s);
    }

    pub fn latest(&self) -> Option<&S> {
        self.history.last()
    }

    pub fn history(&self) -> &[S] {
        &self.history
    }

    /// The full history of one series (or `"tick"`), or `None` if unknown.
    pub fn series(&self, name: &str) -> Option<Vec<f64>> {
        if self.history.is_empty() {
            return S::default().value(name).map(|_| Vec::new());
        }
        self.history.iter().map(|s| s.value(name)).collect()
    }

    /// Keeps the first `len` snapshots (a restored keyframe's history: ticks 0 to its tick).
    pub fn truncate(&mut self, len: usize) {
        self.history.truncate(len);
    }
}

pub fn wealths(world: &World) -> Vec<f64> {
    good_wealths(world, 0)
}

/// Every living agent's holding of good `good`, in id order.
pub fn good_wealths(world: &World, good: usize) -> Vec<f64> {
    world.agents().map(|a| a.holdings[good]).collect()
}

/// Every living agent's total wealth: its holdings of all the world's goods
/// summed in good order (the book never defines it for two goods; this is
/// the natural reading of VI-1's "Lorenz curve and Gini coefficient for
/// total wealth"). With one good it is `wealths`.
pub fn total_wealths(world: &World) -> Vec<f64> {
    let n = world.config.goods.len();
    world
        .agents()
        .map(|a| {
            a.holdings[1..n]
                .iter()
                .fold(a.holdings[0], |sum, h| sum + h)
        })
        .collect()
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

/// Animation III-1's age histogram: living agents' ages in `bin`-tick bins
/// from 0. The last bin reaches the largest configured maximum lifetime plus
/// one (an agent dies at its first turn with age > its maximum, so it lives
/// through the end-of-tick aging that makes it one older) and also holds any
/// older agent. With nobody alive every count is 0.
pub fn age_histogram(world: &World, bin: u32) -> Vec<f64> {
    assert!(bin >= 1);
    let bins = ((world.config.lifespan.max_age.max + 1) / bin + 1) as usize;
    let mut counts = vec![0.0; bins];
    for a in world.agents() {
        counts[((a.age / bin) as usize).min(bins - 1)] += 1.0;
    }
    counts
}

/// Animation III-7's cultural tag histogram: "one bin for each tag position.
/// The height of the bin gives the percentage of agents having a 0 at that
/// position" (position 0 first). With nobody alive every bin is 0.
pub fn tag_histogram(world: &World) -> Vec<f64> {
    let mut zeros = vec![0u32; world.config.tag_length as usize];
    for a in world.agents() {
        for (i, z) in zeros.iter_mut().enumerate() {
            if !a.tags.get(i as u32) {
                *z += 1;
            }
        }
    }
    let n = world.population();
    zeros
        .into_iter()
        .map(|z| {
            if n == 0 {
                0.0
            } else {
                100.0 * f64::from(z) / n as f64
            }
        })
        .collect()
}

/// Largest-Triangle-Three-Buckets: at most `max` of `values`' points as
/// `(index, value)` — the index is the tick, since the history holds one
/// snapshot per tick from 0 — chosen so the line keeps its shape. The first
/// and last points are always kept, and `values.len() <= max` keeps every
/// point. Non-finite values (NaN: no trades, no agents) are never a bucket's
/// choice, but a bucket holding nothing else keeps its first index, so a gap
/// stays a gap. `max < 3` keeps the two endpoints only.
pub fn downsample(values: &[f64], max: usize) -> Vec<(u32, f64)> {
    let n = values.len();
    let point = |i: usize| (i as u32, values[i]);
    if n <= max || n <= 2 {
        return (0..n).map(point).collect();
    }
    if max < 3 {
        return vec![point(0), point(n - 1)];
    }
    let buckets = max - 2;
    // Bucket b holds indices start(b)..start(b + 1) of the interior 1..n - 1;
    // each holds at least one, since n - 2 > buckets.
    let start = |b: usize| bucket_start(b, n, buckets);
    let mut out = Vec::with_capacity(max);
    out.push(point(0));
    // The triangle's first corner: the last finite point kept.
    let mut anchor = values[0].is_finite().then_some((0.0, values[0]));
    for b in 0..buckets {
        // Its third corner: the mean of the next bucket (the last point after the final one).
        let next = if b + 1 < buckets {
            start(b + 1)..start(b + 2)
        } else {
            n - 1..n
        };
        let third = finite_mean(values, next);
        let mut best: Option<(usize, f64)> = None;
        for (i, &y) in values.iter().enumerate().take(start(b + 1)).skip(start(b)) {
            if !y.is_finite() {
                continue;
            }
            let x = i as f64;
            let area = match (anchor, third) {
                (Some((ax, ay)), Some((cx, cy))) => {
                    ((ax - cx) * (y - ay) - (ax - x) * (cy - ay)).abs()
                }
                (Some((_, ay)), None) => (y - ay).abs(),
                (None, Some((_, cy))) => (y - cy).abs(),
                (None, None) => 0.0,
            };
            if best.is_none_or(|(_, a)| area > a) {
                best = Some((i, area));
            }
        }
        match best {
            Some((i, _)) => {
                out.push(point(i));
                anchor = Some((i as f64, values[i]));
            }
            // Nothing finite here: keep the gap.
            None => out.push(point(start(b))),
        }
    }
    out.push(point(n - 1));
    out
}

/// Where LTTB bucket `b` of `buckets` begins in an `n`-point series: the
/// interior 1..n - 1 split evenly. The product is taken in u64, since
/// `b * (n - 2)` overflows wasm32's 32-bit `usize` past about 2.15 M points.
fn bucket_start(b: usize, n: usize, buckets: usize) -> usize {
    1 + (b as u64 * (n as u64 - 2) / buckets as u64) as usize
}

/// The mean position of the finite values in `range`, or `None` if it has none.
fn finite_mean(values: &[f64], range: Range<usize>) -> Option<(f64, f64)> {
    let (mut sx, mut sy, mut k) = (0.0, 0.0, 0.0);
    for (i, &y) in range.clone().zip(&values[range]) {
        if y.is_finite() {
            sx += i as f64;
            sy += y;
            k += 1.0;
        }
    }
    if k > 0.0 {
        Some((sx / k, sy / k))
    } else {
        None
    }
}

/// The sorted union of each column's `downsample(column, max)` indices: one
/// x axis on which every column keeps its own shape (a multi-line chart).
pub fn downsample_union(columns: &[Vec<f64>], max: usize) -> Vec<usize> {
    let mut keep: Vec<usize> = columns
        .iter()
        .flat_map(|c| downsample(c, max).into_iter().map(|(i, _)| i as usize))
        .collect();
    keep.sort_unstable();
    keep.dedup();
    keep
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
    use crate::config::CachingRule;
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

    #[test]
    fn downsample_keeps_short_series_and_both_endpoints() {
        let v: Vec<f64> = (0..10).map(f64::from).collect();
        let all: Vec<(u32, f64)> = (0..10u32).map(|i| (i, f64::from(i))).collect();
        assert_eq!(downsample(&v, 10), all);
        assert_eq!(downsample(&v, 50), all);
        assert_eq!(downsample(&v, 2), vec![(0, 0.0), (9, 9.0)]);
        assert_eq!(downsample(&v, 0), vec![(0, 0.0), (9, 9.0)]);
        assert!(downsample(&[], 5).is_empty());
        assert_eq!(downsample(&[4.0], 0), vec![(0, 4.0)]);
        let long: Vec<f64> = (0..10_000).map(|i| (f64::from(i) / 50.0).sin()).collect();
        for max in [3, 7, 100, 2000] {
            let d = downsample(&long, max);
            assert_eq!(d.len(), max, "max {max}");
            assert_eq!(d[0], (0, long[0]));
            assert_eq!(d[max - 1], (9_999, long[9_999]));
            assert!(d.windows(2).all(|w| w[0].0 < w[1].0), "indices ascend");
            assert!(
                d.iter().all(|&(i, y)| long[i as usize] == y),
                "points are real"
            );
        }
    }

    #[test]
    fn bucket_bounds_do_not_overflow_32_bits() {
        // A 3 M-tick history cut to 2000 points: b * (n - 2) passes u32::MAX,
        // which wrapped (then panicked) on wasm32.
        let (n, buckets) = (3_000_000usize, 1998usize);
        assert!((buckets as u64 - 1) * (n as u64 - 2) > u64::from(u32::MAX));
        assert_eq!(bucket_start(0, n, buckets), 1);
        assert_eq!(bucket_start(buckets, n, buckets), n - 1);
        assert_eq!(
            bucket_start(buckets - 1, n, buckets),
            1 + 1997 * 2_999_998 / 1998
        );
        let mut last = 0;
        for b in 0..=buckets {
            let s = bucket_start(b, n, buckets);
            assert!(s > last && s < n);
            last = s;
        }
    }

    #[test]
    fn downsample_keeps_a_single_sharp_spike() {
        let mut v = vec![1.0; 10_000];
        v[4_321] = 50.0;
        let d = downsample(&v, 100);
        assert!(d.len() <= 100);
        assert!(d.contains(&(4_321, 50.0)), "{d:?}");
    }

    #[test]
    fn downsample_skips_nan_but_keeps_gaps() {
        let v: Vec<f64> = (0..1000)
            .map(|i: i32| {
                if (400..600).contains(&i) {
                    f64::NAN
                } else {
                    f64::from(i % 7)
                }
            })
            .collect();
        let d = downsample(&v, 50);
        assert_eq!(d.len(), 50);
        assert_eq!((d[0].0, d[49].0), (0, 999));
        let nan: Vec<u32> = d.iter().filter(|p| p.1.is_nan()).map(|p| p.0).collect();
        assert!(!nan.is_empty(), "the gap survives");
        assert!(nan.iter().all(|i| (400..600).contains(i)), "{nan:?}");
        assert!(d.iter().any(|p| p.0 < 400 && p.1.is_finite()));
        assert!(d.iter().any(|p| p.0 >= 600 && p.1.is_finite()));
    }

    #[test]
    fn downsample_union_keeps_each_columns_spike() {
        let mut a = vec![0.0; 5_000];
        let mut b = vec![0.0; 5_000];
        a[1_000] = 9.0;
        b[3_000] = -9.0;
        let keep = downsample_union(&[a, b], 50);
        assert!(keep.contains(&1_000) && keep.contains(&3_000), "{keep:?}");
        assert!(
            keep.windows(2).all(|w| w[0] < w[1]),
            "sorted, no duplicates"
        );
        assert_eq!((keep[0], keep[keep.len() - 1]), (0, 4_999));
        assert!(keep.len() <= 100);
    }

    #[test]
    fn age_histogram_bins_ages_up_to_the_largest_maximum_lifetime() {
        use crate::testkit::*;
        let mut w = blank_world(10, 10);
        assert_eq!(
            age_histogram(&w, 5),
            vec![0.0; 21],
            "(100 + 1) / 5 + 1 bins, all empty"
        );
        for (x, age) in [
            (0, 0),
            (1, 4),
            (2, 5),
            (3, 99),
            (4, 100),
            (5, 101),
            (6, 250),
        ] {
            let id = spawn(&mut w, x, 0);
            w.agent_mut(id).unwrap().age = age;
        }
        let h = age_histogram(&w, 5);
        assert_eq!(h.len(), 21);
        assert_eq!((h[0], h[1], h[19], h[20]), (2.0, 1.0, 1.0, 3.0));
        assert_eq!(h.iter().sum::<f64>(), 7.0);
        w.config.lifespan.max_age = crate::config::URange::new(60, 64);
        assert_eq!(age_histogram(&w, 5).len(), 14, "(64 + 1) / 5 + 1");
    }

    #[test]
    fn tag_histogram_is_the_percentage_of_zeros_at_each_position() {
        use crate::agent::Tags;
        use crate::testkit::*;
        let mut w = blank_world(10, 10);
        assert_eq!(tag_histogram(&w), vec![0.0; 11], "nobody alive");
        let a = spawn(&mut w, 0, 0);
        let b = spawn(&mut w, 1, 0);
        w.agent_mut(a).unwrap().tags = Tags::new(0b000_0000_0011, 11);
        w.agent_mut(b).unwrap().tags = Tags::new(0b100_0000_0001, 11);
        let h = tag_histogram(&w);
        assert_eq!(h.len(), 11);
        assert_eq!((h[0], h[1], h[2], h[10]), (0.0, 50.0, 100.0, 50.0));
    }

    #[test]
    fn per_good_and_total_wealth() {
        use crate::testkit::*;
        let mut w = blank_world(10, 10);
        let a = spawn(&mut w, 0, 0);
        let b = spawn(&mut w, 1, 0);
        w.agent_mut(a).unwrap().holdings[..3].copy_from_slice(&[1.0, 4.0, 2.0]);
        w.agent_mut(b).unwrap().holdings[..3].copy_from_slice(&[3.0, 0.0, 9.0]);
        assert_eq!(wealths(&w), vec![1.0, 3.0]);
        assert_eq!(total_wealths(&w), vec![1.0, 3.0], "one good: sugar only");
        add_goods(&mut w.config, 3);
        assert_eq!(good_wealths(&w, 1), vec![4.0, 0.0]);
        assert_eq!(good_wealths(&w, 2), vec![2.0, 9.0]);
        assert_eq!(total_wealths(&w), vec![7.0, 12.0]);
        assert_eq!(wealths(&w), vec![1.0, 3.0], "the sugar views are unchanged");
    }

    #[test]
    fn gini_total_is_recorded_every_tick_and_equals_gini_with_one_good() {
        let mut one = World::new(Config::default(), 3).unwrap();
        one.run(5);
        for s in one.stats.history() {
            assert_eq!(s.gini_total, s.gini);
        }
        let mut c = Config::default();
        c.add_good(crate::config::Good::spice());
        let mut two = World::new(c, 3).unwrap();
        two.run(5);
        let s = two.stats.latest().unwrap();
        assert_eq!(s.gini_total, gini(&total_wealths(&two)));
        assert_ne!(s.gini_total, s.gini);
        assert_eq!(two.stats.series("gini_total").unwrap().len(), 6);
        assert_eq!(series_names(&two.config)[SERIES.len() - 1], "gini_total");
    }

    fn two_patch_world() -> World {
        let mut c = Config {
            width: 60,
            height: 40,
            population: 0,
            ..Config::default()
        };
        c.goods[0].map = crate::config::Map::Peaks {
            peaks: vec![
                crate::config::Peak {
                    x: 15,
                    y: 20,
                    radius: 10.0,
                    height: 4.0,
                },
                crate::config::Peak {
                    x: 42,
                    y: 20,
                    radius: 7.0,
                    height: 4.0,
                },
            ],
        };
        World::new(c, 1).unwrap()
    }

    #[test]
    fn patch_series_exist_only_on_maps_of_two_or_more_peaks() {
        let names = |c: &Config| series_names(c);
        let two = two_patch_world().config.clone();
        for s in [
            "on_first_patch",
            "on_other_patches",
            "off_patch",
            "first_patch_share",
        ] {
            assert!(names(&two).contains(&s.to_string()), "{s}");
            assert!(
                !names(&Config::default()).contains(&s.to_string()),
                "{s} on two_peaks"
            );
        }
        let mut one = two.clone();
        if let crate::config::Map::Peaks { peaks } = &mut one.goods[0].map {
            peaks.truncate(1);
        }
        assert!(!names(&one).contains(&"off_patch".to_string()));
        let snap = Snapshot::of(&World::new(Config::default(), 1).unwrap());
        assert!(snap.patches.is_none());
        assert_eq!(snap.value("off_patch"), None);
    }

    #[test]
    fn patch_series_count_agents_by_patch() {
        let mut w = two_patch_world();
        assert_eq!(Snapshot::of(&w).value("first_patch_share"), Some(0.0)); // nobody: 0, not NaN
        for (x, y) in [(15, 20), (16, 20), (42, 20), (30, 5)] {
            crate::testkit::spawn(&mut w, x, y);
        }
        let s = Snapshot::of(&w);
        assert_eq!(s.value("on_first_patch"), Some(2.0));
        assert_eq!(s.value("on_other_patches"), Some(1.0));
        assert_eq!(s.value("off_patch"), Some(1.0));
        assert_eq!(s.value("first_patch_share"), Some(2.0 / 3.0));
    }

    const MEMORY_SERIES: [&str; 7] = [
        "remembering",
        "remembered_moves",
        "belief_error",
        "stale_choices",
        "wealth_rememberers",
        "wealth_others",
        "wealth_advantage",
    ];
    const TRUFFLE_SERIES: [&str; 2] = ["truffles_found", "truffles_by_rememberers"];

    #[test]
    fn memory_and_truffle_series_exist_only_under_their_switches() {
        let off = Config::default();
        let names = series_names(&off);
        for s in MEMORY_SERIES.iter().chain(TRUFFLE_SERIES.iter()) {
            assert!(!names.contains(&s.to_string()), "{s}");
        }
        let snap = Snapshot::of(&World::new(off, 1).unwrap());
        assert!(snap.memory.is_none());
        assert!(snap.truffles.is_none());
        for s in MEMORY_SERIES.iter().chain(TRUFFLE_SERIES.iter()) {
            assert_eq!(snap.value(s), None, "{s}");
        }

        let mut on = Config::default();
        on.memory.span = 50;
        on.truffles.share = 0.2;
        let names = series_names(&on);
        for s in MEMORY_SERIES.iter().chain(TRUFFLE_SERIES.iter()) {
            assert!(names.contains(&s.to_string()), "{s}");
        }
    }

    #[test]
    fn memory_series_shares_and_means_with_empty_groups_as_zero() {
        use crate::testkit::*;
        let mut w = blank_world(5, 5);
        w.config.memory.span = 50;
        // Nobody alive yet: shares and means are 0, not NaN.
        let empty = Snapshot::of(&w);
        let m = empty.memory.expect("memory stats present");
        assert_eq!(
            (
                m.remembering,
                m.remembered_moves,
                m.belief_error,
                m.stale_choices,
                m.wealth_rememberers,
                m.wealth_others,
                m.wealth_advantage
            ),
            (0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0)
        );

        // Three rememberers, one non-rememberer.
        let a = spawn(&mut w, 0, 0);
        let b = spawn(&mut w, 1, 0);
        let c = spawn(&mut w, 2, 0);
        let d = spawn(&mut w, 3, 0);
        for id in [a, b, c] {
            w.agent_mut(id).unwrap().remembers = true;
        }
        w.agent_mut(a).unwrap().holdings[0] = 10.0;
        w.agent_mut(b).unwrap().holdings[0] = 20.0;
        w.agent_mut(c).unwrap().holdings[0] = 30.0;
        w.agent_mut(d).unwrap().holdings[0] = 100.0;
        w.events.moves = 5;
        w.events.remembered_moves = 2;
        w.events.belief_error_sum = 3.0;
        w.events.stale_choices = 1;
        let s = Snapshot::of(&w);
        let m = s.memory.expect("memory stats present");
        assert_eq!(m.remembering, 0.75, "3 of 4 remember");
        assert_eq!(m.remembered_moves, 2.0 / 5.0);
        assert_eq!(m.belief_error, 3.0 / 2.0);
        assert_eq!(m.stale_choices, 1.0 / 2.0);
        assert_eq!(m.wealth_rememberers, 20.0, "mean of 10, 20, 30");
        assert_eq!(m.wealth_others, 100.0);
        assert_eq!(m.wealth_advantage, 20.0 - 100.0);
        assert!(s.truffles.is_none(), "truffles.share is still 0");
        for name in MEMORY_SERIES {
            assert!(s.value(name).is_some(), "{name}");
        }

        // Everyone remembers: `wealth_others` has no members, so both it and
        // the advantage are 0 (not the rememberers' own mean, not NaN).
        w.agent_mut(d).unwrap().remembers = true;
        let s = Snapshot::of(&w);
        let m = s.memory.unwrap();
        assert_eq!(m.wealth_others, 0.0);
        assert_eq!(m.wealth_advantage, 0.0);
        assert_eq!(m.wealth_rememberers, (10.0 + 20.0 + 30.0 + 100.0) / 4.0);
    }

    #[test]
    fn memory_series_zero_denominators_are_zero_not_nan() {
        use crate::testkit::*;
        let mut w = blank_world(5, 5);
        w.config.memory.span = 50;
        spawn(&mut w, 0, 0);
        // No moves at all this tick.
        let s = Snapshot::of(&w);
        let m = s.memory.unwrap();
        assert_eq!(
            (m.remembered_moves, m.belief_error, m.stale_choices),
            (0.0, 0.0, 0.0)
        );

        // Moves, but none remembered.
        w.events.moves = 4;
        let s = Snapshot::of(&w);
        let m = s.memory.unwrap();
        assert_eq!(m.remembered_moves, 0.0);
        assert_eq!((m.belief_error, m.stale_choices), (0.0, 0.0));
    }

    #[test]
    fn truffle_series_read_the_tick_events() {
        use crate::testkit::*;
        let mut w = blank_world(5, 5);
        w.config.truffles.share = 0.1;
        let empty = Snapshot::of(&w);
        let t = empty.truffles.expect("truffle stats present");
        assert_eq!((t.truffles_found, t.truffles_by_rememberers), (0, 0));
        assert!(empty.memory.is_none(), "memory.span is still 0");

        w.events.truffles_found = 7;
        w.events.truffles_by_rememberers = 3;
        let s = Snapshot::of(&w);
        let t = s.truffles.unwrap();
        assert_eq!((t.truffles_found, t.truffles_by_rememberers), (7, 3));
        for name in TRUFFLE_SERIES {
            assert!(s.value(name).is_some(), "{name}");
        }
        assert_eq!(s.value("truffles_found"), Some(7.0));
        assert_eq!(s.value("truffles_by_rememberers"), Some(3.0));
    }

    #[test]
    fn memory_and_truffle_series_names_come_after_the_patch_series() {
        let mut c = Config::default();
        c.memory.span = 50;
        c.truffles.share = 0.2;
        let names = series_names(&c);
        let base = series_names(&Config::default()).len();
        assert_eq!(
            names.len(),
            base + MEMORY_SERIES.len() + TRUFFLE_SERIES.len()
        );
        assert_eq!(
            &names[base..base + MEMORY_SERIES.len()],
            MEMORY_SERIES.as_slice()
        );
        assert_eq!(
            &names[base + MEMORY_SERIES.len()..],
            TRUFFLE_SERIES.as_slice()
        );
    }

    const GOAP_SERIES: [&str; 4] = [
        "replans",
        "mean_plan_length",
        "fallbacks",
        "plans_remembered",
    ];
    const MVT_SERIES: [&str; 2] = ["replans", "mean_rate"];
    // `replans` names both rules' leave/plan-count series (never together,
    // since `decision.rule` is one value); the other names are exclusive.
    const GOAP_ONLY: [&str; 3] = ["mean_plan_length", "fallbacks", "plans_remembered"];
    const MVT_ONLY: [&str; 1] = ["mean_rate"];

    #[test]
    fn goap_and_mvt_series_exist_only_under_their_rule() {
        let book = Config::default();
        let names = series_names(&book);
        assert!(!names.contains(&"replans".to_string()));
        for s in GOAP_ONLY.iter().chain(MVT_ONLY.iter()) {
            assert!(!names.contains(&s.to_string()), "{s}");
        }
        let snap = Snapshot::of(&World::new(book, 1).unwrap());
        assert!(snap.goap.is_none());
        assert!(snap.mvt.is_none());
        for s in GOAP_SERIES.iter().chain(MVT_SERIES.iter()) {
            assert_eq!(snap.value(s), None, "{s}");
        }

        let mut goap = Config::default();
        goap.decision.rule = crate::config::DecisionRule::Goap;
        let names = series_names(&goap);
        for s in GOAP_SERIES {
            assert!(names.contains(&s.to_string()), "{s}");
        }
        for s in MVT_ONLY {
            assert!(!names.contains(&s.to_string()), "{s}");
        }

        let mut mvt = Config::default();
        mvt.decision.rule = crate::config::DecisionRule::Mvt;
        let names = series_names(&mvt);
        for s in MVT_SERIES {
            assert!(names.contains(&s.to_string()), "{s}");
        }
        for s in GOAP_ONLY {
            assert!(!names.contains(&s.to_string()), "{s}");
        }
    }

    #[test]
    fn goap_series_reads_the_tick_events() {
        use crate::testkit::*;
        let mut w = blank_world(5, 5);
        w.config.decision.rule = crate::config::DecisionRule::Goap;
        spawn(&mut w, 0, 0);
        spawn(&mut w, 1, 0);
        w.events.plans = 3;
        w.events.plan_steps_sum = 9;
        w.events.plans_with_remembered = 1;
        w.events.fallback_short = 1;
        w.events.fallback_limit = 1;
        let s = Snapshot::of(&w);
        let g = s.goap.expect("goap stats present");
        assert_eq!(g.replans, 1.5, "3 plans over 2 alive");
        assert_eq!(g.mean_plan_length, 3.0, "9 steps over 3 plans");
        assert_eq!(g.fallbacks, 1.0, "2 fallbacks over 2 alive");
        assert_eq!(g.plans_remembered, 1.0 / 3.0);
        assert!(s.mvt.is_none());
        for name in GOAP_SERIES {
            assert!(s.value(name).is_some(), "{name}");
        }
    }

    #[test]
    fn goap_series_zero_denominators_are_zero_not_nan() {
        use crate::testkit::*;
        let mut w = blank_world(5, 5);
        w.config.decision.rule = crate::config::DecisionRule::Goap;
        // Nobody alive: the per-alive ratios are 0, not NaN.
        let s = Snapshot::of(&w);
        let g = s.goap.unwrap();
        assert_eq!((g.replans, g.fallbacks), (0.0, 0.0));
        assert_eq!((g.mean_plan_length, g.plans_remembered), (0.0, 0.0));

        // Alive, but no plans made this tick.
        spawn(&mut w, 0, 0);
        let s = Snapshot::of(&w);
        let g = s.goap.unwrap();
        assert_eq!(g.replans, 0.0);
        assert_eq!((g.mean_plan_length, g.plans_remembered), (0.0, 0.0));
    }

    #[test]
    fn mvt_series_reads_the_tick_events_and_mean_rate() {
        use crate::testkit::*;
        let mut w = blank_world(5, 5);
        w.config.decision.rule = crate::config::DecisionRule::Mvt;
        let a = spawn(&mut w, 0, 0);
        let b = spawn(&mut w, 1, 0);
        w.agent_mut(a).unwrap().rate = 1.0;
        w.agent_mut(b).unwrap().rate = 3.0;
        w.events.leaves = 1;
        let s = Snapshot::of(&w);
        let m = s.mvt.expect("mvt stats present");
        assert_eq!(m.replans, 0.5, "1 leave over 2 alive");
        assert_eq!(m.mean_rate, 2.0, "mean of 1.0 and 3.0");
        assert!(s.goap.is_none());
        for name in MVT_SERIES {
            assert!(s.value(name).is_some(), "{name}");
        }
    }

    #[test]
    fn mvt_series_zero_denominators_are_zero_not_nan() {
        let mut w = crate::testkit::blank_world(5, 5);
        w.config.decision.rule = crate::config::DecisionRule::Mvt;
        // Nobody alive: 0, not NaN.
        let s = Snapshot::of(&w);
        let m = s.mvt.unwrap();
        assert_eq!((m.replans, m.mean_rate), (0.0, 0.0));
    }

    #[test]
    fn goap_and_mvt_series_names_come_after_truffles() {
        let mut goap = Config::default();
        goap.decision.rule = crate::config::DecisionRule::Goap;
        let names = series_names(&goap);
        let base = series_names(&Config::default()).len();
        assert_eq!(names.len(), base + GOAP_SERIES.len());
        assert_eq!(&names[base..], GOAP_SERIES.as_slice());

        let mut mvt = Config::default();
        mvt.decision.rule = crate::config::DecisionRule::Mvt;
        let names = series_names(&mvt);
        assert_eq!(names.len(), base + MVT_SERIES.len());
        assert_eq!(&names[base..], MVT_SERIES.as_slice());
    }

    const CACHING_SERIES: [&str; 5] = ["cached", "buried", "dug", "recovery", "mean_cache_age"];
    const CENTRAL_SERIES: [&str; 2] = ["mean_load", "trips"];

    #[test]
    fn caching_and_central_series_exist_only_under_their_switch() {
        let book = Config::default();
        let names = series_names(&book);
        for s in CACHING_SERIES.iter().chain(CENTRAL_SERIES.iter()) {
            assert!(!names.contains(&s.to_string()), "{s}");
        }
        let snap = Snapshot::of(&World::new(book, 1).unwrap());
        assert!(snap.caching.is_none());
        assert!(snap.central.is_none());
        for s in CACHING_SERIES.iter().chain(CENTRAL_SERIES.iter()) {
            assert_eq!(snap.value(s), None, "{s}");
        }

        // A carrying limit alone (no named rule) still turns caching on.
        let mut capacity_only = Config::default();
        capacity_only.caching.capacity = 5;
        let names = series_names(&capacity_only);
        for s in CACHING_SERIES {
            assert!(names.contains(&s.to_string()), "{s}");
        }
        for s in CENTRAL_SERIES {
            assert!(!names.contains(&s.to_string()), "{s}");
        }

        let mut central = Config::default();
        central.central.enabled = true;
        let names = series_names(&central);
        for s in CENTRAL_SERIES {
            assert!(names.contains(&s.to_string()), "{s}");
        }
        for s in CACHING_SERIES {
            assert!(!names.contains(&s.to_string()), "{s}");
        }
    }

    #[test]
    fn caching_series_reads_the_tick_events_and_world_caches() {
        use crate::testkit::*;
        let mut w = blank_world(5, 5);
        w.config.caching.rule = CachingRule::Even;
        let a = spawn(&mut w, 0, 0);
        let b = spawn(&mut w, 1, 0);
        w.agent_mut(a).unwrap().caches.insert(2, 3.0);
        w.agent_mut(b).unwrap().caches.insert(7, 4.5);
        w.events.buried = 6.0;
        w.events.dug = 2.0;
        w.events.digs = 2;
        w.events.dig_ages_sum = 9;
        let s = Snapshot::of(&w);
        let c = s.caching.expect("caching stats present");
        assert_eq!(c.cached, 7.5, "Σ over living agents' caches");
        assert_eq!(c.buried, 6.0);
        assert_eq!(c.dug, 2.0);
        assert_eq!(c.mean_cache_age, 4.5, "9 ticks over 2 digs");
        assert_eq!(c.recovery, 2.0 / 6.0, "first tick's Σ dug ÷ Σ buried");
        assert!(s.central.is_none());
        for name in CACHING_SERIES {
            assert!(s.value(name).is_some(), "{name}");
        }
    }

    #[test]
    fn caching_recovery_is_cumulative_since_tick_zero() {
        use crate::testkit::*;
        use crate::world::TickEvents;
        let mut w = blank_world(5, 5);
        w.config.caching.rule = CachingRule::Even;

        w.events.buried = 10.0;
        let s1 = Snapshot::of(&w);
        assert_eq!(s1.caching.unwrap().recovery, 0.0, "nothing dug yet");
        w.stats.push(s1);

        w.events = TickEvents::default();
        w.events.dug = 4.0;
        let s2 = Snapshot::of(&w);
        assert_eq!(
            s2.caching.unwrap().recovery,
            0.4,
            "4 of the 10 buried so far"
        );
        w.stats.push(s2);

        w.events = TickEvents::default();
        w.events.buried = 10.0;
        w.events.dug = 1.0;
        let s3 = Snapshot::of(&w);
        assert_eq!(
            s3.caching.unwrap().recovery,
            5.0 / 20.0,
            "5 dug of 20 buried, cumulative since tick 0"
        );
    }

    #[test]
    fn caching_series_zero_denominators_are_zero_not_nan() {
        let mut w = crate::testkit::blank_world(5, 5);
        w.config.caching.rule = CachingRule::Even;
        // Nothing buried yet, no digs, nobody caching: 0, not NaN.
        let s = Snapshot::of(&w);
        let c = s.caching.unwrap();
        assert_eq!((c.recovery, c.mean_cache_age, c.cached), (0.0, 0.0, 0.0));
    }

    #[test]
    fn central_series_reads_the_tick_events() {
        use crate::testkit::*;
        let mut w = blank_world(5, 5);
        w.config.central.enabled = true;
        spawn(&mut w, 0, 0);
        spawn(&mut w, 1, 0);
        w.events.deliveries = 3;
        w.events.delivered = 12.0;
        let s = Snapshot::of(&w);
        let c = s.central.expect("central stats present");
        assert_eq!(c.mean_load, 4.0, "12 delivered over 3 deliveries so far");
        assert_eq!(c.trips, 1.5, "3 deliveries over 2 living agents");
        assert!(s.caching.is_none());
        for name in CENTRAL_SERIES {
            assert!(s.value(name).is_some(), "{name}");
        }
    }

    #[test]
    fn central_series_zero_denominators_are_zero_not_nan() {
        let mut w = crate::testkit::blank_world(5, 5);
        w.config.central.enabled = true;
        // Nobody alive, nothing delivered: 0, not NaN.
        let s = Snapshot::of(&w);
        let c = s.central.unwrap();
        assert_eq!((c.mean_load, c.trips), (0.0, 0.0));
    }

    #[test]
    fn central_mean_load_is_cumulative_since_tick_zero() {
        use crate::testkit::*;
        use crate::world::TickEvents;
        let mut w = blank_world(5, 5);
        w.config.central.enabled = true;
        spawn(&mut w, 0, 0);
        spawn(&mut w, 1, 0);

        w.events.deliveries = 3;
        w.events.delivered = 12.0;
        let s1 = Snapshot::of(&w);
        assert_eq!(s1.central.unwrap().mean_load, 4.0, "12 over 3, tick 1");
        w.stats.push(s1);

        w.events = TickEvents::default();
        w.events.deliveries = 1;
        w.events.delivered = 2.0;
        let s2 = Snapshot::of(&w);
        assert_eq!(
            s2.central.unwrap().mean_load,
            14.0 / 4.0,
            "14 delivered of 4 deliveries, cumulative since tick 0"
        );
    }

    #[test]
    fn caching_and_central_series_names_come_after_mvt() {
        let base = series_names(&Config::default()).len();

        let mut caching = Config::default();
        caching.caching.rule = CachingRule::Even;
        let names = series_names(&caching);
        assert_eq!(names.len(), base + CACHING_SERIES.len());
        assert_eq!(&names[base..], CACHING_SERIES.as_slice());

        let mut central = Config::default();
        central.central.enabled = true;
        let names = series_names(&central);
        assert_eq!(names.len(), base + CENTRAL_SERIES.len());
        assert_eq!(&names[base..], CENTRAL_SERIES.as_slice());

        let mut both = Config::default();
        both.caching.rule = CachingRule::Even;
        both.central.enabled = true;
        let names = series_names(&both);
        assert_eq!(
            names.len(),
            base + CACHING_SERIES.len() + CENTRAL_SERIES.len()
        );
        assert_eq!(
            &names[base..base + CACHING_SERIES.len()],
            CACHING_SERIES.as_slice()
        );
        assert_eq!(
            &names[base + CACHING_SERIES.len()..],
            CENTRAL_SERIES.as_slice()
        );
    }
}
