//! Declared measured Minds 9 campaign, independent of the Holds/Fails registry.
use crate::stats::{paired_summary, PairedSummary};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use sugarscape_core::config::{Scrounge, Who};
use sugarscape_core::minds::spatial_hoarding::{
    runner::{
        self, GenerationRecord, InitialCohortConfig, RunEnvelope, RunnerConfig, Selection,
        Terminal, DRAW_ORDER,
    },
    state::FounderTraits,
};

pub const SCHEMA: &str = "minds9-measured-v1";
pub const AMENDMENT: &str = "docs/superpowers/specs/2026-10-02-minds-9-judge-amendment.md";

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum CohortSpec {
    Fixed,
    Sampled,
    Contest { family: String, share: f64 },
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Condition {
    pub id: String,
    pub panel: String,
    pub config: RunnerConfig,
    pub cohort: CohortSpec,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Materialized {
    pub config: RunnerConfig,
    pub initial: Vec<FounderTraits>,
    pub initial_sampling: Option<InitialCohortConfig>,
}
impl Condition {
    pub fn watch(&self) -> u32 {
        if self.config.episode.watching.on {
            self.config.episode.watching.span
        } else {
            0
        }
    }
    pub fn materialize(&self, seed: u64) -> Result<Materialized, String> {
        let mut config = self.config.clone();
        config.episode_seed = seed;
        config.breeding_seed = seed;
        config.validate()?;
        let initial_sampling =
            matches!(self.cohort, CohortSpec::Sampled).then_some(InitialCohortConfig {
                seed,
                ..Default::default()
            });
        let initial = if let Some(init) = &initial_sampling {
            runner::sample_initial_cohort(&config.episode, init)?
        } else {
            (1..=config.founders as u64)
                .map(|id| {
                    let (cheater, watches) = match &self.cohort {
                        CohortSpec::Contest { family, share } => {
                            let mut assignment = config.episode.theft;
                            assignment.cheaters = *share;
                            let flagged = assignment.founder_cheats(id);
                            if family == "scrounger" {
                                (flagged, flagged && config.episode.watching.on)
                            } else {
                                (false, flagged)
                            }
                        }
                        _ => (false, config.episode.watching.on),
                    };
                    FounderTraits {
                        larder: config.episode.spatial_hoarding.larder,
                        defense: config.episode.spatial_hoarding.defense,
                        cheater,
                        watches,
                    }
                })
                .collect()
        };
        Ok(Materialized {
            config,
            initial,
            initial_sampling,
        })
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Contrast {
    pub id: String,
    pub a: String,
    /// Empty b is a declared zero reference for inherited share change.
    pub b: String,
    pub metric: String,
    pub primary: bool,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Manifest {
    pub schema: String,
    pub conditions: Vec<Condition>,
    pub seeds: Vec<u64>,
    pub timing_conditions: Vec<String>,
    pub timing_seeds: Vec<u64>,
    pub contrasts: Vec<Contrast>,
}
fn base(watch: u32) -> RunnerConfig {
    let mut c = RunnerConfig::default();
    c.episode.theft.cheaters = 0.0;
    c.episode.theft.find = 0.25;
    c.episode.watching.on = watch > 0;
    c.episode.watching.span = if watch == 0 { 2 } else { watch };
    c.episode.watching.who = Who::Share;
    c.episode.watching.watchers = if watch > 0 { 1.0 } else { 0.0 };
    c.episode.spatial_hoarding.larder = 0.15;
    c.episode.spatial_hoarding.defense = 0.5;
    c.episode.spatial_hoarding.guard = true;
    c.episode.spatial_hoarding.find_larder = 0.25;
    c
}
fn condition(id: String, panel: &str, config: RunnerConfig, cohort: CohortSpec) -> Condition {
    Condition {
        id,
        panel: panel.into(),
        config,
        cohort,
    }
}
fn contrast(m: &mut Manifest, id: String, a: String, b: String, metric: &str, primary: bool) {
    m.contrasts.push(Contrast {
        id,
        a,
        b,
        metric: metric.into(),
        primary,
    });
}
pub fn manifest() -> Manifest {
    let mut m = Manifest {
        schema: SCHEMA.into(),
        conditions: Vec::new(),
        seeds: (1..=40).collect(),
        timing_conditions: Vec::new(),
        timing_seeds: (1..=5).collect(),
        contrasts: Vec::new(),
    };
    for w in [0, 2, 7] {
        for l in [0, 50, 100] {
            for guard in [false, true] {
                let mut c = base(w);
                c.generations = 1;
                c.fixed_traits = true;
                c.episode.spatial_hoarding.larder = f64::from(l) / 100.0;
                c.episode.spatial_hoarding.guard = guard;
                let id = format!("fixed-l{l}-g{}-w{w}", u8::from(guard));
                m.timing_conditions.push(id.clone());
                m.conditions
                    .push(condition(id, "fixed", c, CohortSpec::Fixed));
            }
        }
        contrast(
            &mut m,
            format!("home-travel-w{w}"),
            format!("fixed-l100-g0-w{w}"),
            format!("fixed-l0-g0-w{w}"),
            "survival",
            true,
        );
        contrast(
            &mut m,
            format!("defense-w{w}"),
            format!("fixed-l100-g1-w{w}"),
            format!("fixed-l100-g0-w{w}"),
            "survival",
            true,
        );
        for (name, find) in [
            ("0", 0.0),
            ("002", 0.02),
            ("005", 0.05),
            ("025", 0.25),
            ("1", 1.0),
        ] {
            let mut c = base(w);
            c.episode.theft.find = find;
            m.conditions.push(condition(
                format!("discovery-f{name}-w{w}"),
                "discovery",
                c,
                CohortSpec::Sampled,
            ));
            if name != "025" {
                contrast(
                    &mut m,
                    format!("discovery-f{name}-w{w}"),
                    format!("discovery-f{name}-w{w}"),
                    format!("discovery-f025-w{w}"),
                    "mean_larder",
                    true,
                );
            }
        }
        for (name, selection) in [
            ("survival", Selection::Survival),
            ("neutral", Selection::Neutral),
            ("stores", Selection::Stores),
        ] {
            let mut c = base(w);
            c.selection = selection;
            let id = format!("continuous-{name}-w{w}");
            if selection == Selection::Survival {
                m.timing_conditions.push(id.clone());
            }
            m.conditions
                .push(condition(id, "continuous", c, CohortSpec::Sampled));
        }
        for metric in ["mean_larder", "mean_defense", "survival"] {
            contrast(
                &mut m,
                format!("selection-{metric}-w{w}"),
                format!("continuous-survival-w{w}"),
                format!("continuous-neutral-w{w}"),
                metric,
                false,
            );
            contrast(
                &mut m,
                format!("stores-{metric}-w{w}"),
                format!("continuous-stores-w{w}"),
                format!("continuous-survival-w{w}"),
                metric,
                false,
            );
        }
        for family in ["scrounger", "watcher"] {
            for (label, share) in [(10, 0.1), (50, 0.5), (90, 0.9)] {
                let mut c = base(w);
                c.fixed_traits = true;
                c.episode.spatial_hoarding.larder = 0.0;
                if family == "scrounger" {
                    c.episode.theft.cheaters = share;
                    c.episode.watching.who = Who::Cheaters;
                    c.episode.watching.scrounge = Scrounge::Forgo;
                } else {
                    c.episode.watching.watchers = share;
                }
                let id = format!("contest-{family}-s{label}-w{w}");
                m.conditions.push(condition(
                    id.clone(),
                    "contests",
                    c,
                    CohortSpec::Contest {
                        family: family.into(),
                        share,
                    },
                ));
                contrast(
                    &mut m,
                    format!("share-change-{id}"),
                    id.clone(),
                    String::new(),
                    if family == "scrounger" {
                        "cheater_change"
                    } else {
                        "watcher_change"
                    },
                    true,
                );
                if w > 0 {
                    contrast(
                        &mut m,
                        format!("group-survival-{id}"),
                        id,
                        format!("contest-{family}-s{label}-w0"),
                        "survival",
                        false,
                    );
                }
            }
        }
        for name in [
            "stores",
            "inheritance",
            "neutral-inheritance",
            "slope5",
            "slope20",
            "scatter-first",
            "guard-harvest",
        ] {
            let mut c = base(w);
            match name {
                "stores" => c.selection = Selection::Stores,
                "inheritance" => {
                    c.heritability = 1.0;
                    c.segregation_variance = 0.0;
                }
                "neutral-inheritance" => {
                    c.selection = Selection::Neutral;
                    c.heritability = 1.0;
                    c.segregation_variance = 0.0;
                }
                "slope5" => c.episode.spatial_hoarding.defense_slope = 5.0,
                "slope20" => c.episode.spatial_hoarding.defense_slope = 20.0,
                "scatter-first" => c.probe.scatter_first = true,
                "guard-harvest" => c.probe.guard_harvest = true,
                _ => unreachable!(),
            }
            let id = format!("sensitivity-{name}-w{w}");
            m.conditions
                .push(condition(id.clone(), "sensitivity", c, CohortSpec::Sampled));
            for metric in ["mean_larder", "mean_defense", "survival"] {
                contrast(
                    &mut m,
                    format!("{id}-{metric}"),
                    id.clone(),
                    format!("continuous-survival-w{w}"),
                    metric,
                    false,
                );
            }
        }
    }
    m
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RunRow {
    pub condition_id: String,
    pub seed: u64,
    pub envelope: RunEnvelope,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Distribution {
    pub n: usize,
    pub zero: usize,
    pub positive: usize,
    pub min: f64,
    pub median: f64,
    pub max: f64,
    pub sum: f64,
}
fn distribution(xs: &[f64]) -> Distribution {
    Distribution {
        n: xs.len(),
        zero: xs.iter().filter(|x| **x == 0.0).count(),
        positive: xs.iter().filter(|x| **x > 0.0).count(),
        min: crate::stats::quantile(xs, 0.0),
        median: crate::stats::median(xs),
        max: crate::stats::quantile(xs, 1.0),
        sum: xs.iter().sum(),
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MeasuredGeneration {
    pub generation: u32,
    pub terminal: Option<Terminal>,
    pub metrics: BTreeMap<String, Option<f64>>,
    pub parent_weights: Distribution,
    pub larder_traits: Distribution,
    pub defense_traits: Distribution,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SeedSummary {
    pub condition_id: String,
    pub seed: u64,
    pub terminal: Option<Terminal>,
    pub completed_generations: usize,
    pub completed_ticks: u64,
    pub live_agent_ticks: u64,
    pub initial_mean_larder: f64,
    pub initial_mean_defense: f64,
    pub initial_cheater_share: f64,
    pub initial_watcher_share: f64,
    pub trajectory: Vec<MeasuredGeneration>,
    pub endpoint: MeasuredGeneration,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MeasuredContrast {
    pub declaration: Contrast,
    pub summary: Option<PairedSummary>,
    pub missing_seeds: Vec<u64>,
    pub seed_differences: BTreeMap<u64, Option<f64>>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MetricDefinition {
    pub unit: String,
    pub denominator: String,
    pub missingness: String,
}
fn metric_definitions() -> BTreeMap<String, MetricDefinition> {
    let mut out = BTreeMap::new();
    let mut define = |name: String, unit: &str, denominator: &str, missingness: &str| {
        out.insert(
            name,
            MetricDefinition {
                unit: unit.into(),
                denominator: denominator.into(),
                missingness: missingness.into(),
            },
        );
    };
    for k in [
        "survival",
        "mean_larder",
        "mean_defense",
        "cheater_share",
        "watcher_share",
        "cheater_change",
        "watcher_flag_change",
    ] {
        define(k.into(), "fraction", "all_archived_founders", "never");
    }
    define(
        "watcher_change".into(),
        "fraction",
        "all_archived_founders",
        "mechanism_off",
    );
    define(
        "ticks_alive_per_founder".into(),
        "ticks",
        "all_archived_founders",
        "never",
    );
    define("completed_ticks".into(), "ticks", "none", "never");
    for kind in ["scatter", "larder"] {
        define(format!("closing_{kind}"), "food_units", "none", "never");
        for field in [
            "buried",
            "dug",
            "pilfered",
            "lost",
            "bury_cost",
            "loot_eaten",
        ] {
            define(format!("{kind}_{field}"), "food_units", "none", "never");
        }
        for field in ["digs", "pilfers", "pilfer_candidates", "caches_pilfered"] {
            define(
                format!("{kind}_{field}"),
                "events",
                "none",
                "never; pilfer_candidates is gate_dependent",
            );
        }
        define(
            format!("{kind}_cache_ticks"),
            "positive_cache_ticks",
            "pre_step_positive_stores_excluding_closing",
            "never",
        );
        define(
            format!("{kind}_stock_ticks"),
            "food_unit_ticks",
            "pre_step_positive_stocks_excluding_closing",
            "never",
        );
        define(
            format!("{kind}_recovery"),
            "fraction",
            "total_buried_food_units",
            "zero_denominator",
        );
        define(
            format!("{kind}_pilferage_rate"),
            "events_per_cache_tick",
            "pre_step_cache_ticks",
            "zero_denominator",
        );
        define(
            format!("{kind}_loss_rate"),
            "food_units_per_stock_tick",
            "pre_step_stock_ticks",
            "zero_denominator",
        );
    }
    define("closing_holdings".into(), "food_units", "none", "never");
    for prefix in ["delivery", "guard", "observation", "metabolism"] {
        let fields: &[(&str, &str)] = match prefix {
            "delivery" => &[
                ("starts", "events"),
                ("completions", "events"),
                ("cancellations", "events"),
                ("return_turns", "agent_turns"),
                ("delivered", "food_units"),
                ("bury_cost", "food_units"),
            ],
            "guard" => &[
                ("intended", "events"),
                ("executed", "agent_turns"),
                ("recovered", "food_units"),
                ("probe_harvest", "food_units"),
                ("blocked_raids", "events"),
                ("blocked_discoveries", "events"),
            ],
            "observation" => &[
                ("burials_seen", "events"),
                ("sightings", "events"),
                ("seen_entries", "entries"),
                ("seen_arrivals", "events"),
                ("contacts", "events"),
                ("discovery_draws", "draws"),
                ("discovery_hits", "events"),
                ("raid_attempts", "events"),
                ("raids", "events"),
                ("raided", "food_units"),
                ("raids_empty", "events"),
                ("raids_no_room", "events"),
                ("raids_blocked", "events"),
                ("discoveries_blocked", "events"),
                ("scatter_draws_skipped", "draws"),
            ],
            _ => &[("demand", "food_units"), ("consumed", "food_units")],
        };
        for (field, unit) in fields {
            define(
                format!("{prefix}_{field}"),
                unit,
                "none",
                "never; zero explicitly means unused or no events",
            );
        }
    }
    define(
        "larder_discovery_success".into(),
        "fraction",
        "observation_discovery_draws",
        "zero_denominator",
    );
    define(
        "guard_probe_harvest_per_guard".into(),
        "food_units_per_guard_turn",
        "guard_executed",
        "zero_denominator; ordinary_probe_harvest_is_zero",
    );
    for name in ["cheater", "watcher"] {
        define(
            format!("{name}_survival_gap"),
            "fraction_difference",
            "archived_founders_with_flag_and_without_flag_separately",
            "either_group_absent",
        );
    }
    out
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Analysis {
    pub schema: String,
    pub metric_definitions: BTreeMap<String, MetricDefinition>,
    pub manifest: Manifest,
    pub runs: Vec<SeedSummary>,
    pub contrasts: Vec<MeasuredContrast>,
}
fn close(x: f64, y: f64) -> bool {
    x.is_finite() && y.is_finite() && (x - y).abs() <= 1e-9 * x.abs().max(y.abs()).max(1.0)
}
/// serde_json represents nonfinite numbers as null: permit only the two actual optional fields.
fn check_record_numbers(v: &serde_json::Value, key: &str) -> Result<(), String> {
    match v {
        serde_json::Value::Null if key != "terminal" && key != "parents" => {
            Err(format!("nonfinite or missing measured field {key}"))
        }
        serde_json::Value::Object(o) => {
            for (k, x) in o {
                check_record_numbers(x, k)?;
            }
            Ok(())
        }
        serde_json::Value::Array(a) => {
            for x in a {
                check_record_numbers(x, key)?;
            }
            Ok(())
        }
        _ => Ok(()),
    }
}
fn ratio(n: f64, d: f64) -> Option<f64> {
    if d > 0.0 {
        Some(n / d)
    } else {
        None
    }
}
fn metrics(g: &GenerationRecord) -> BTreeMap<String, Option<f64>> {
    let mut m = BTreeMap::new();
    for (k, v) in [
        ("survival", g.survivors as f64 / g.founders.len() as f64),
        (
            "ticks_alive_per_founder",
            g.total_ticks_alive as f64 / g.founders.len() as f64,
        ),
        ("mean_larder", g.mean_larder),
        ("mean_defense", g.mean_defense),
        ("cheater_share", g.cheater_share),
        ("watcher_share", g.watcher_share),
        ("closing_holdings", g.closing_holdings),
        ("closing_scatter", g.closing_scatter),
        ("closing_larder", g.closing_larder),
        ("completed_ticks", g.completed_ticks as f64),
    ] {
        m.insert(k.into(), Some(v));
    }
    // Preserve every recorded counter with the exact runner field name, not a curated subset.
    fn flatten(v: &serde_json::Value, prefix: &str, m: &mut BTreeMap<String, Option<f64>>) {
        if let Some(o) = v.as_object() {
            for (k, x) in o {
                let name = if prefix.is_empty() {
                    k.clone()
                } else {
                    format!("{prefix}_{k}")
                };
                flatten(x, &name, m);
            }
        } else if let Some(n) = v.as_f64() {
            m.insert(prefix.into(), Some(n));
        }
    }
    flatten(
        &serde_json::to_value(g.events).expect("finite checked"),
        "",
        &mut m,
    );
    for (kind, flow, exposure) in [
        ("scatter", g.events.scatter, g.scatter_exposure),
        ("larder", g.events.larder, g.larder_exposure),
    ] {
        m.insert(
            format!("{kind}_cache_ticks"),
            Some(exposure.cache_ticks as f64),
        );
        m.insert(format!("{kind}_stock_ticks"), Some(exposure.stock_ticks));
        m.insert(format!("{kind}_recovery"), ratio(flow.dug, flow.buried));
        m.insert(
            format!("{kind}_pilferage_rate"),
            ratio(flow.caches_pilfered as f64, exposure.cache_ticks as f64),
        );
        m.insert(
            format!("{kind}_loss_rate"),
            ratio(flow.pilfered + flow.lost, exposure.stock_ticks),
        );
    }
    m.insert(
        "larder_discovery_success".into(),
        ratio(
            g.events.observation.discovery_hits as f64,
            g.events.observation.discovery_draws as f64,
        ),
    );
    m.insert(
        "guard_probe_harvest_per_guard".into(),
        ratio(g.events.guard.probe_harvest, g.events.guard.executed as f64),
    );
    for (flag, name) in [(true, "cheater"), (false, "watcher")] {
        let group: Vec<_> = g
            .founders
            .iter()
            .filter(|f| {
                if flag {
                    f.traits.cheater
                } else {
                    f.traits.watches
                }
            })
            .collect();
        let other: Vec<_> = g
            .founders
            .iter()
            .filter(|f| {
                if flag {
                    !f.traits.cheater
                } else {
                    !f.traits.watches
                }
            })
            .collect();
        let gap = if group.is_empty() || other.is_empty() {
            None
        } else {
            Some(
                group.iter().filter(|f| f.alive).count() as f64 / group.len() as f64
                    - other.iter().filter(|f| f.alive).count() as f64 / other.len() as f64,
            )
        };
        m.insert(format!("{name}_survival_gap"), gap);
    }
    m
}
pub fn summarize_run(m: &Manifest, row: &RunRow) -> Result<SeedSummary, String> {
    let c = m
        .conditions
        .iter()
        .find(|c| c.id == row.condition_id)
        .ok_or_else(|| format!("unknown condition {}", row.condition_id))?;
    if !m.seeds.contains(&row.seed) {
        return Err(format!("undeclared seed {}", row.seed));
    }
    let expected = c.materialize(row.seed)?;
    let e = &row.envelope;
    if e.config != expected.config
        || e.initial != expected.initial
        || e.initial_sampling != expected.initial_sampling
        || e.draw_order != DRAW_ORDER
    {
        return Err(format!(
            "config/cohort/sampler/draw-order mismatch: {} seed {}",
            c.id, row.seed
        ));
    }
    if e.generations.is_empty() || e.generations.len() > e.config.generations as usize {
        return Err("missing or extra generation rows".into());
    }
    let n = e.config.founders;
    let nf = n as f64;
    let initial_mean_larder = e.initial.iter().map(|t| t.larder).sum::<f64>() / nf;
    let initial_mean_defense = e.initial.iter().map(|t| t.defense).sum::<f64>() / nf;
    let initial_cheater_share = e.initial.iter().filter(|t| t.cheater).count() as f64 / nf;
    let initial_watcher_share = e.initial.iter().filter(|t| t.watches).count() as f64 / nf;
    let mut trajectory = Vec::new();
    for (i, g) in e.generations.iter().enumerate() {
        check_record_numbers(&serde_json::to_value(g).map_err(|e| e.to_string())?, "")?;
        if g.generation != i as u32
            || g.founders.len() != n
            || g.episode_seed != row.seed
            || g.breeding_seed != row.seed
            || g.requested_ticks != e.config.ticks
            || g.completed_ticks == 0
            || g.completed_ticks > g.requested_ticks
        {
            return Err("invalid generation sequence/cohort/seeds/ticks".into());
        }
        if g.terminal.is_some() && i + 1 != e.generations.len() {
            return Err("rows follow terminal generation".into());
        }
        let weights = runner::parent_weights(e.config.selection, &g.founders);
        if weights.as_ref().err().copied() != g.terminal {
            return Err("terminal/fitness mismatch".into());
        }
        if g.terminal != Some(Terminal::Extinct) && g.completed_ticks != g.requested_ticks {
            return Err("nonextinct season missing ticks".into());
        }
        for (slot, f) in g.founders.iter().enumerate() {
            if f.slot != slot
                || f.lineage.generation != g.generation
                || f.lineage.slot != slot
                || f.ticks_alive > g.completed_ticks
                || (f.alive && f.ticks_alive != g.completed_ticks)
                || !(0.0..=1.0).contains(&f.traits.larder)
                || !(0.0..=1.0).contains(&f.traits.defense)
                || f.holdings < 0.0
                || f.scatter < 0.0
                || f.larder < 0.0
                || (!f.alive && (f.holdings != 0.0 || f.scatter != 0.0 || f.larder != 0.0))
            {
                return Err("invalid archived founder".into());
            }
            if i == 0 {
                if f.traits != e.initial[slot] || f.parents.is_some() {
                    return Err("initial archive mismatch".into());
                }
            } else {
                let parents = f.parents.ok_or("missing parent lineage")?;
                if parents
                    .iter()
                    .any(|p| p.generation + 1 != g.generation || p.slot >= n)
                {
                    return Err("invalid parent lineage".into());
                }
                let previous = &e.generations[i - 1].founders;
                if parents
                    .iter()
                    .any(|p| previous[p.slot].parent_weight <= 0.0)
                {
                    return Err("parent lineage references zero weight".into());
                }
                let inherited = previous[parents[0].slot].traits;
                if f.traits.cheater != inherited.cheater
                    || f.traits.watches != inherited.watches
                    || (e.config.fixed_traits && f.traits != inherited)
                {
                    return Err("first-parent strategy/frozen-trait mismatch".into());
                }
            }
            if f.parent_weight != weights.as_ref().map_or(0.0, |w| w[slot]) {
                return Err("incorrect parent weight".into());
            }
        }
        let sums = [
            (
                g.survivors as f64,
                g.founders.iter().filter(|f| f.alive).count() as f64,
            ),
            (
                g.total_ticks_alive as f64,
                g.founders.iter().map(|f| f.ticks_alive as f64).sum(),
            ),
            (
                g.mean_larder,
                g.founders.iter().map(|f| f.traits.larder).sum::<f64>() / nf,
            ),
            (
                g.mean_defense,
                g.founders.iter().map(|f| f.traits.defense).sum::<f64>() / nf,
            ),
            (
                g.cheater_share,
                g.founders.iter().filter(|f| f.traits.cheater).count() as f64 / nf,
            ),
            (
                g.watcher_share,
                g.founders.iter().filter(|f| f.traits.watches).count() as f64 / nf,
            ),
            (
                g.closing_holdings,
                g.founders.iter().map(|f| f.holdings).sum(),
            ),
            (
                g.closing_scatter,
                g.founders.iter().map(|f| f.scatter).sum(),
            ),
            (g.closing_larder, g.founders.iter().map(|f| f.larder).sum()),
        ];
        if sums.iter().any(|(x, y)| !close(*x, *y)) {
            return Err("archive/summary denominator mismatch".into());
        }
        let mut values = metrics(g);
        values.insert(
            "cheater_change".into(),
            Some(g.cheater_share - initial_cheater_share),
        );
        values.insert(
            "watcher_flag_change".into(),
            Some(g.watcher_share - initial_watcher_share),
        );
        values.insert(
            "watcher_change".into(),
            e.config
                .episode
                .watching
                .on
                .then_some(g.watcher_share - initial_watcher_share),
        );
        if values.values().flatten().any(|x| !x.is_finite()) {
            return Err("nonfinite derived measurement".into());
        }
        trajectory.push(MeasuredGeneration {
            generation: g.generation,
            terminal: g.terminal,
            metrics: values,
            parent_weights: distribution(
                &g.founders
                    .iter()
                    .map(|f| f.parent_weight)
                    .collect::<Vec<_>>(),
            ),
            larder_traits: distribution(
                &g.founders
                    .iter()
                    .map(|f| f.traits.larder)
                    .collect::<Vec<_>>(),
            ),
            defense_traits: distribution(
                &g.founders
                    .iter()
                    .map(|f| f.traits.defense)
                    .collect::<Vec<_>>(),
            ),
        });
    }
    let endpoint = trajectory.last().expect("nonempty checked").clone();
    if endpoint.terminal.is_none() && trajectory.len() != e.config.generations as usize {
        return Err("missing generations or terminal row".into());
    }
    Ok(SeedSummary {
        condition_id: row.condition_id.clone(),
        seed: row.seed,
        terminal: endpoint.terminal,
        completed_generations: trajectory.len(),
        completed_ticks: e.generations.iter().map(|g| g.completed_ticks).sum(),
        live_agent_ticks: e.generations.iter().map(|g| g.total_ticks_alive).sum(),
        initial_mean_larder,
        initial_mean_defense,
        initial_cheater_share,
        initial_watcher_share,
        trajectory,
        endpoint,
    })
}
#[cfg(test)]
fn analyze_rows(m: &Manifest, rows: Vec<RunRow>) -> Result<Analysis, String> {
    analyze(
        m,
        rows.iter()
            .map(|r| summarize_run(m, r))
            .collect::<Result<_, _>>()?,
    )
}
pub fn analyze(m: &Manifest, mut runs: Vec<SeedSummary>) -> Result<Analysis, String> {
    validate_manifest(m)?;
    let mut index = BTreeMap::new();
    for r in &runs {
        if index.insert((r.condition_id.clone(), r.seed), r).is_some() {
            return Err(format!(
                "duplicate condition/seed: {} {}",
                r.condition_id, r.seed
            ));
        }
    }
    let expected = m.conditions.len() * m.seeds.len();
    if index.len() != expected {
        return Err(format!(
            "incomplete campaign: {} rows, expected {expected}",
            index.len()
        ));
    }
    for c in &m.conditions {
        for s in &m.seeds {
            if !index.contains_key(&(c.id.clone(), *s)) {
                return Err(format!("missing condition/seed {} {s}", c.id));
            }
        }
    }
    let mut contrasts = Vec::new();
    for c in &m.contrasts {
        let (mut a, mut b, mut missing, mut differences) = (
            BTreeMap::new(),
            BTreeMap::new(),
            Vec::new(),
            BTreeMap::new(),
        );
        for s in &m.seeds {
            let x = index
                .get(&(c.a.clone(), *s))
                .ok_or("missing comparison partner")?
                .endpoint
                .metrics
                .get(&c.metric)
                .ok_or("unknown metric")?;
            let y = if c.b.is_empty() {
                Some(0.0)
            } else {
                *index
                    .get(&(c.b.clone(), *s))
                    .ok_or("missing comparison partner")?
                    .endpoint
                    .metrics
                    .get(&c.metric)
                    .ok_or("unknown metric")?
            };
            if let (Some(x), Some(y)) = (x, y) {
                a.insert(*s, *x);
                b.insert(*s, y);
                differences.insert(*s, Some(*x - y));
            } else {
                missing.push(*s);
                differences.insert(*s, None);
            }
        }
        let summary = if missing.is_empty() {
            Some(paired_summary(&a, &b)?)
        } else {
            None
        };
        contrasts.push(MeasuredContrast {
            declaration: c.clone(),
            summary,
            missing_seeds: missing,
            seed_differences: differences,
        });
    }
    runs.sort_by(|a, b| (&a.condition_id, a.seed).cmp(&(&b.condition_id, b.seed)));
    Ok(Analysis {
        schema: SCHEMA.into(),
        metric_definitions: metric_definitions(),
        manifest: m.clone(),
        runs,
        contrasts,
    })
}

/// Construction and sampling only: no World is created or stepped here.
pub fn validate_manifest(m: &Manifest) -> Result<(), String> {
    if m.schema != SCHEMA
        || m.conditions.is_empty()
        || m.seeds.is_empty()
        || m.seeds.iter().collect::<BTreeSet<_>>().len() != m.seeds.len()
        || m.timing_seeds.iter().collect::<BTreeSet<_>>().len() != m.timing_seeds.len()
    {
        return Err("invalid manifest schema/seeds".into());
    }
    let mut ids = BTreeSet::new();
    for c in &m.conditions {
        if !ids.insert(c.id.clone())
            || c.id.is_empty()
            || !c.id.bytes().all(|x| x.is_ascii_alphanumeric() || x == b'-')
        {
            return Err("duplicate or invalid condition id".into());
        }
        for s in &m.seeds {
            c.materialize(*s)?;
        }
    }
    if m.timing_conditions.iter().collect::<BTreeSet<_>>().len() != m.timing_conditions.len()
        || m.timing_conditions.iter().any(|id| !ids.contains(id))
    {
        return Err("invalid timing conditions".into());
    }
    let mut contrasts = BTreeSet::new();
    for c in &m.contrasts {
        if !contrasts.insert(&c.id)
            || !ids.contains(&c.a)
            || (!c.b.is_empty() && !ids.contains(&c.b))
        {
            return Err("duplicate contrast or unknown partner".into());
        }
    }
    Ok(())
}
#[derive(Clone, Debug, PartialEq)]
pub enum Command {
    Help,
    Manifest,
    Analyze {
        index: String,
        out: String,
    },
    Run {
        amendment_revision: String,
        out: String,
    },
}
const HELP: &str = "Minds 9 measured route (independent of Holds/Fails):\n  --minds9                         print declared manifest; no simulations\n  --minds9 --manifest              same as default\n  --minds9 --help                  show this help; no simulations\n  --minds9 --analyze INDEX --out DIR  validate saved full campaign and render analysis\n  --minds9 --run --amendment-revision COMMIT --out NEW_DIR\n                                  execute declared campaign AFTER external user approval\n\n81 conditions x seeds 1..40; 21 timing conditions x seeds 1..5.\nThe revision records provenance, not approval. Do not run before the committed amendment is approved.\nRaw envelopes are saved per condition/seed before analysis. No seed/panel override.\n";
fn revision(s: &str) -> bool {
    s.len() == 40 && s.bytes().all(|c| c.is_ascii_hexdigit())
}
pub fn parse_command(args: &[String]) -> Result<Command, String> {
    if args.is_empty() || args == ["--manifest"] {
        return Ok(Command::Manifest);
    }
    if args == ["--help"] {
        return Ok(Command::Help);
    }
    let mut flags = BTreeMap::new();
    let mut run = false;
    let mut i = 0;
    while i < args.len() {
        let a = &args[i];
        if a == "--run" {
            if run {
                return Err("duplicate --run".into());
            }
            run = true;
            i += 1;
            continue;
        }
        if !["--analyze", "--out", "--amendment-revision"].contains(&a.as_str()) {
            return Err(format!("unknown or incompatible Minds 9 argument {a}"));
        }
        let v = args
            .get(i + 1)
            .filter(|v| !v.starts_with("--"))
            .ok_or_else(|| format!("{a} needs a value"))?;
        if flags.insert(a.as_str(), v.clone()).is_some() {
            return Err(format!("duplicate {a}"));
        }
        i += 2;
    }
    let out = flags.remove("--out").ok_or("--out is required")?;
    if run {
        let amendment_revision = flags
            .remove("--amendment-revision")
            .ok_or("--run needs --amendment-revision provenance")?;
        if !revision(&amendment_revision) || !flags.is_empty() {
            return Err(
                "--run needs a full 40-hex committed amendment revision and no --analyze".into(),
            );
        }
        Ok(Command::Run {
            amendment_revision,
            out,
        })
    } else {
        let index = flags
            .remove("--analyze")
            .ok_or("choose --analyze or --run")?;
        if !flags.is_empty() {
            return Err("unexpected analysis arguments".into());
        }
        Ok(Command::Analyze { index, out })
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Timing {
    pub tick_seconds: f64,
    pub runner_seconds: f64,
    pub completed_generations: usize,
    pub completed_ticks: u64,
    pub live_agent_ticks: u64,
}
impl Timing {
    pub fn per_agent_tick(&self) -> Option<f64> {
        ratio(self.tick_seconds, self.live_agent_ticks as f64)
    }
    pub fn per_completed_generation(&self) -> Option<f64> {
        ratio(self.runner_seconds, self.completed_generations as f64)
    }
}
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum Role {
    Comparison,
    Timing,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct RawFile {
    schema: String,
    code_revision: String,
    amendment_revision: String,
    detailed_ledger: String,
    role: Role,
    row: RunRow,
    timing: Option<Timing>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct RawRef {
    role: Role,
    condition_id: String,
    seed: u64,
    path: String,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct CampaignIndex {
    schema: String,
    code_revision: String,
    amendment_revision: String,
    amendment_path: String,
    manifest: Manifest,
    files: Vec<RawRef>,
}
#[derive(Serialize)]
struct CampaignAnalysis {
    schema: String,
    code_revision: String,
    amendment_revision: String,
    raw_index: String,
    measured: Analysis,
    timings: Vec<TimingSummary>,
}
#[derive(Serialize)]
struct TimingSummary {
    condition_id: String,
    seed: u64,
    terminal: Option<Terminal>,
    survival: f64,
    timing: Timing,
    seconds_per_live_agent_tick: Option<f64>,
    seconds_per_completed_generation: Option<f64>,
}
fn raw_name(role: &Role, id: &str, seed: u64) -> String {
    format!(
        "{}-{id}-seed{seed}.json",
        if *role == Role::Comparison {
            "comparison"
        } else {
            "timing"
        }
    )
}
fn save_json(path: &std::path::Path, value: &impl Serialize) -> Result<(), String> {
    use std::io::Write;
    let file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|e| format!("{}: {e}", path.display()))?;
    let mut out = std::io::BufWriter::new(file);
    serde_json::to_writer(&mut out, value).map_err(|e| e.to_string())?;
    out.flush().map_err(|e| e.to_string())?;
    out.get_ref().sync_all().map_err(|e| e.to_string())
}
fn save_index(dir: &std::path::Path, index: &CampaignIndex) -> Result<(), String> {
    let tmp = dir.join("index.next.json");
    save_json(&tmp, index)?;
    std::fs::rename(tmp, dir.join("index.json")).map_err(|e| e.to_string())
}
fn load_json<T: serde::de::DeserializeOwned>(path: &std::path::Path) -> Result<T, String> {
    let f = std::fs::File::open(path).map_err(|e| format!("{}: {e}", path.display()))?;
    serde_json::from_reader(std::io::BufReader::new(f))
        .map_err(|e| format!("{}: {e}", path.display()))
}
fn git_output(args: &[&str]) -> Result<String, String> {
    let o = std::process::Command::new("git")
        .args(args)
        .current_dir(concat!(env!("CARGO_MANIFEST_DIR"), "/.."))
        .output()
        .map_err(|e| e.to_string())?;
    if !o.status.success() {
        return Err(String::from_utf8_lossy(&o.stderr).into());
    }
    Ok(String::from_utf8_lossy(&o.stdout).trim().into())
}
fn execute(dir: &std::path::Path, amendment_revision: &str) -> Result<(), String> {
    let m = manifest();
    validate_manifest(&m)?;
    // This verifies provenance only. Scientific user approval is an external gate.
    let declared = git_output(&["show", &format!("{amendment_revision}:{AMENDMENT}")])?;
    let current = git_output(&["show", &format!("HEAD:{AMENDMENT}")])?;
    if declared != current {
        return Err("amendment revision differs from the committed current protocol".into());
    }
    if !git_output(&["status", "--porcelain", "--untracked-files=no"])?.is_empty() {
        return Err("campaign execution requires clean committed code".into());
    }
    let code_revision = git_output(&["rev-parse", "HEAD"])?;
    std::fs::create_dir(dir).map_err(|e| format!("new output directory {}: {e}", dir.display()))?;
    let mut index = CampaignIndex {
        schema: SCHEMA.into(),
        code_revision,
        amendment_revision: amendment_revision.into(),
        amendment_path: AMENDMENT.into(),
        manifest: m.clone(),
        files: Vec::new(),
    };
    save_index(dir, &index)?;
    for role in [Role::Comparison, Role::Timing] {
        for c in &m.conditions {
            if role == Role::Timing && !m.timing_conditions.contains(&c.id) {
                continue;
            }
            let seeds = if role == Role::Comparison {
                &m.seeds
            } else {
                &m.timing_seeds
            };
            for &seed in seeds {
                eprintln!("Minds 9 {role:?}: {} seed {seed}", c.id);
                let a = c.materialize(seed)?;
                let start = std::time::Instant::now();
                let mut envelope = runner::run(&a.config, &a.initial)?;
                let runner_seconds = start.elapsed().as_secs_f64();
                envelope.initial_sampling = a.initial_sampling;
                let row = RunRow {
                    condition_id: c.id.clone(),
                    seed,
                    envelope,
                };
                // Save the completed run before any analysis or timing replay.
                let name = raw_name(&role, &c.id, seed);
                let mut raw = RawFile {
                    schema: SCHEMA.into(),
                    code_revision: index.code_revision.clone(),
                    amendment_revision: amendment_revision.into(),
                    detailed_ledger: "not_collected".into(),
                    role: role.clone(),
                    row,
                    timing: None,
                };
                save_json(&dir.join(&name), &raw)?;
                if role == Role::Timing {
                    raw.timing = Some(time_ticks(&raw.row.envelope, runner_seconds)?);
                    let temporary = dir.join(format!("{name}.next"));
                    save_json(&temporary, &raw)?;
                    std::fs::rename(temporary, dir.join(&name)).map_err(|e| e.to_string())?;
                }
                index.files.push(RawRef {
                    role: role.clone(),
                    condition_id: c.id.clone(),
                    seed,
                    path: name,
                });
                save_index(dir, &index)?;
            }
        }
    }
    render_saved(&dir.join("index.json"), dir)
}
/// Replay archived seasonal cohorts, measuring only whole World::step calls.
/// Construction is outside the timer; elapsed death ticks remain in the denominator.
fn time_ticks(e: &RunEnvelope, runner_seconds: f64) -> Result<Timing, String> {
    let mut t = Timing {
        tick_seconds: 0.0,
        runner_seconds,
        completed_generations: e.generations.len(),
        completed_ticks: 0,
        live_agent_ticks: 0,
    };
    for g in &e.generations {
        let cohort: Vec<_> = g.founders.iter().map(|f| f.traits).collect();
        let mut w = e.config.episode_world(&cohort)?;
        let mut live_agent_ticks = 0;
        for _ in 0..g.completed_ticks {
            if w.population() == 0 {
                return Err("timing replay ended before recorded tick".into());
            }
            live_agent_ticks += w.population() as u64;
            let start = std::time::Instant::now();
            w.step();
            t.tick_seconds += start.elapsed().as_secs_f64();
        }
        if w.tick != g.completed_ticks
            || w.population() != g.survivors
            || live_agent_ticks != g.total_ticks_alive
        {
            return Err("timing replay denominator mismatch".into());
        }
        t.completed_ticks += g.completed_ticks;
        t.live_agent_ticks += live_agent_ticks;
    }
    Ok(t)
}
fn render_saved(index_path: &std::path::Path, out: &std::path::Path) -> Result<(), String> {
    let index: CampaignIndex = load_json(index_path)?;
    if index.schema != SCHEMA
        || index.manifest != manifest()
        || !revision(&index.code_revision)
        || !revision(&index.amendment_revision)
        || index.amendment_path != AMENDMENT
    {
        return Err("campaign provenance/manifest mismatch".into());
    }
    validate_manifest(&index.manifest)?;
    let dir = index_path.parent().unwrap_or(std::path::Path::new("."));
    let mut seen = BTreeSet::new();
    let mut summaries = Vec::new();
    let mut timings = Vec::new();
    for r in &index.files {
        if !seen.insert((r.role.clone(), r.condition_id.clone(), r.seed))
            || r.path != raw_name(&r.role, &r.condition_id, r.seed)
        {
            return Err("duplicate condition/seed or invalid raw reference".into());
        }
        let raw: RawFile = load_json(&dir.join(&r.path))?;
        if raw.schema != index.schema
            || raw.code_revision != index.code_revision
            || raw.amendment_revision != index.amendment_revision
            || raw.role != r.role
            || raw.row.condition_id != r.condition_id
            || raw.row.seed != r.seed
            || raw.detailed_ledger != "not_collected"
        {
            return Err("raw provenance/ledger mismatch".into());
        }
        let s = summarize_run(&index.manifest, &raw.row)?;
        match r.role {
            Role::Comparison => {
                if raw.timing.is_some() {
                    return Err("comparison has timing payload".into());
                }
                summaries.push(s);
            }
            Role::Timing => {
                if !index.manifest.timing_conditions.contains(&r.condition_id)
                    || !index.manifest.timing_seeds.contains(&r.seed)
                {
                    return Err("undeclared timing cell/seed".into());
                }
                let t = raw.timing.ok_or("missing timing payload")?;
                if !t.tick_seconds.is_finite()
                    || !t.runner_seconds.is_finite()
                    || t.tick_seconds < 0.0
                    || t.runner_seconds < 0.0
                    || t.completed_generations != s.completed_generations
                    || t.completed_ticks != s.completed_ticks
                    || t.live_agent_ticks != s.live_agent_ticks
                {
                    return Err("invalid timing or denominator mismatch".into());
                }
                timings.push(TimingSummary {
                    condition_id: r.condition_id.clone(),
                    seed: r.seed,
                    terminal: s.terminal,
                    survival: s.endpoint.metrics["survival"].expect("defined survival"),
                    seconds_per_live_agent_tick: t.per_agent_tick(),
                    seconds_per_completed_generation: t.per_completed_generation(),
                    timing: t,
                });
            }
        }
    }
    for id in &index.manifest.timing_conditions {
        for seed in &index.manifest.timing_seeds {
            if !seen.contains(&(Role::Timing, id.clone(), *seed)) {
                return Err(format!("missing timing row {id} {seed}"));
            }
        }
    }
    let analysis = analyze(&index.manifest, summaries)?;
    timings.sort_by(|a, b| (&a.condition_id, a.seed).cmp(&(&b.condition_id, b.seed)));
    let report = markdown(&analysis, &timings, index_path, &index);
    let result = CampaignAnalysis {
        schema: SCHEMA.into(),
        code_revision: index.code_revision,
        amendment_revision: index.amendment_revision,
        raw_index: index_path.display().to_string(),
        measured: analysis,
        timings,
    };
    std::fs::create_dir_all(out).map_err(|e| e.to_string())?;
    save_json(&out.join("analysis.json"), &result)?;
    std::fs::write(out.join("results.md"), report).map_err(|e| e.to_string())
}
fn markdown(
    a: &Analysis,
    t: &[TimingSummary],
    path: &std::path::Path,
    index: &CampaignIndex,
) -> String {
    use std::fmt::Write;
    let mut s=format!("# Minds 9 measured results\n\nCode `{}`; amendment `{}`; raw index `{}`. Protocol: `{AMENDMENT}`.\n\nNo Holds/Fails verdicts. Run seed is the sample unit. Endpoints use the last observed archived cohort, including dead founders; terminal endpoints are never extended to generation 59. All trajectories and parent-weight distributions are in analysis.json; complete founders, lineage, weights, initial sampling and draw order are in raw files.\n\n## Declared paired contrasts\n\nA minus B; no multiplicity-adjusted inference or sweep-cell selection. Intervals are descriptive Student t intervals. Unavailable means at least one undefined/unused metric; no unmatched seeds are dropped.\n\n| Contrast | Metric | Primary | n | Mean | 95% t interval | + / 0 / − | Missing seeds |\n|---|---|---|---:|---:|---|---|---|\n",index.code_revision,index.amendment_revision,path.display());
    for c in &a.contrasts {
        if let Some(p) = &c.summary {
            let ci = p
                .ci95
                .map_or("absent (n=1)".into(), |(l, h)| format!("[{l:.6}, {h:.6}]"));
            writeln!(
                s,
                "| {} | {} | {} | {} | {:.6} | {} | {} / {} / {} | — |",
                c.declaration.id,
                c.declaration.metric,
                c.declaration.primary,
                p.n,
                p.mean,
                ci,
                p.positive,
                p.zero,
                p.negative
            )
            .unwrap();
        } else {
            writeln!(
                s,
                "| {} | {} | {} | — | unavailable | absent | — | {:?} |",
                c.declaration.id, c.declaration.metric, c.declaration.primary, c.missing_seeds
            )
            .unwrap();
        }
    }
    s.push_str("\n## Terminal and endpoint records by condition\n\nMeans use all declared seeds. Completed generations vary after termination. Strategy-contest frequency changes are frozen-trait measures; interior shares alone do not establish stability. Mechanism-off watcher flags remain inherited, but watching advantage is unused. Scrounger enabled contests jointly change cheating and watching.\n\n| Condition | Runs | Extinct | Zero fitness | Completed generations min–max | Endpoint L mean | Endpoint D mean | Survival mean | Initial L / D realized mean |\n|---|---:|---:|---:|---|---:|---:|---:|---|\n");
    for c in &a.manifest.conditions {
        let rows: Vec<_> = a.runs.iter().filter(|r| r.condition_id == c.id).collect();
        let n = rows.len() as f64;
        let avg = |k: &str| {
            rows.iter()
                .map(|r| r.endpoint.metrics[k].unwrap_or(0.0))
                .sum::<f64>()
                / n
        };
        writeln!(
            s,
            "| {} | {} | {} | {} | {}–{} | {:.6} | {:.6} | {:.6} | {:.6} / {:.6} |",
            c.id,
            rows.len(),
            rows.iter()
                .filter(|r| r.terminal == Some(Terminal::Extinct))
                .count(),
            rows.iter()
                .filter(|r| r.terminal == Some(Terminal::ZeroFitness))
                .count(),
            rows.iter().map(|r| r.completed_generations).min().unwrap(),
            rows.iter().map(|r| r.completed_generations).max().unwrap(),
            avg("mean_larder"),
            avg("mean_defense"),
            avg("survival"),
            rows.iter().map(|r| r.initial_mean_larder).sum::<f64>() / n,
            rows.iter().map(|r| r.initial_mean_defense).sum::<f64>() / n
        )
        .unwrap();
    }

    s.push_str("\n## Fixed-strategy mechanism measurements\n\nPer-run means across all 40 seeds; exposure is pre-step and excludes closing stocks. Amounts are food units; returns and guards are paid agent-turns.\n\n| Condition | Watch span (0=off) | Scatter stock-ticks | Larder stock-ticks | Contacts | Discovery draws | Consumed / demanded | Delivered | Return turns | Guards executed | Blocked raids / discoveries |\n|---|---:|---:|---:|---:|---:|---|---:|---:|---:|---|\n");
    for c in a.manifest.conditions.iter().filter(|c| c.panel == "fixed") {
        let rows: Vec<_> = a.runs.iter().filter(|r| r.condition_id == c.id).collect();
        let mean = |k: &str| {
            rows.iter()
                .map(|r| r.endpoint.metrics[k].expect("defined counter"))
                .sum::<f64>()
                / rows.len() as f64
        };
        writeln!(s,"| {} | {} | {:.3} | {:.3} | {:.3} | {:.3} | {:.3} / {:.3} | {:.3} | {:.3} | {:.3} | {:.3} / {:.3} |",c.id,c.watch(),mean("scatter_stock_ticks"),mean("larder_stock_ticks"),mean("observation_contacts"),mean("observation_discovery_draws"),mean("metabolism_consumed"),mean("metabolism_demand"),mean("delivery_delivered"),mean("delivery_return_turns"),mean("guard_executed"),mean("guard_blocked_raids"),mean("guard_blocked_discoveries")).unwrap();
    }
    s.push_str("\n## Frozen strategy contest endpoints\n\nRealized initial and endpoint flags include all archived founders. Watching-off changes remain unused for spread interpretation; exact seed changes and intervals appear in the declared contrast table.\n\n| Condition | Initial cheater / watcher shares | Endpoint cheater / watcher shares | Whole-cohort survival |\n|---|---|---|---:|\n");
    for c in a
        .manifest
        .conditions
        .iter()
        .filter(|c| c.panel == "contests")
    {
        let rows: Vec<_> = a.runs.iter().filter(|r| r.condition_id == c.id).collect();
        let nf = rows.len() as f64;
        let mean = |k: &str| {
            rows.iter()
                .map(|r| r.endpoint.metrics[k].expect("defined share"))
                .sum::<f64>()
                / nf
        };
        writeln!(
            s,
            "| {} | {:.6} / {:.6} | {:.6} / {:.6} | {:.6} |",
            c.id,
            rows.iter().map(|r| r.initial_cheater_share).sum::<f64>() / nf,
            rows.iter().map(|r| r.initial_watcher_share).sum::<f64>() / nf,
            mean("cheater_share"),
            mean("watcher_share"),
            mean("survival")
        )
        .unwrap();
    }
    s.push_str("\n## Timing (separate seeds 1–5)\n\nWhole-tick time measures step calls on exact archived-cohort replays, excluding construction and file I/O. Runner time includes world construction, breeding and bookkeeping; excludes sampling, replay and file I/O. Death ticks count in live-at-tick-start agent-ticks; terminal records count as completed generations.\n\n| Condition | Seed | Generations | Ticks | Live agent-ticks | Terminal | Survival | Seconds/live agent-tick | Seconds/completed generation |\n|---|---:|---:|---:|---:|---|---:|---:|---:|\n");
    for r in t {
        writeln!(
            s,
            "| {} | {} | {} | {} | {} | {:?} | {:.6} | {:.9} | {:.6} |",
            r.condition_id,
            r.seed,
            r.timing.completed_generations,
            r.timing.completed_ticks,
            r.timing.live_agent_ticks,
            r.terminal,
            r.survival,
            r.seconds_per_live_agent_tick.unwrap_or(f64::NAN),
            r.seconds_per_completed_generation.unwrap_or(f64::NAN)
        )
        .unwrap();
    }
    s.push_str("\nDetailed capped fate ledgers were not collected; aggregate event flows and always-on pre-step cache/stock exposures remain authoritative. Zero exposure produces an absent ratio, not zero loss. Metabolic consumption excludes burial cost; guard probe harvest is explicit and separately labeled. Sensitivities cannot create a new main verdict. No V&J .219 calibration or Minds 7 endpoint classification is used.\n");
    s
}
pub fn cli(args: &[String]) -> Result<(), String> {
    match parse_command(args)? {
        Command::Help => print!("{HELP}"),
        Command::Manifest => {
            let m = manifest();
            validate_manifest(&m)?;
            println!(
                "{}",
                serde_json::to_string_pretty(&m).map_err(|e| e.to_string())?
            );
        }
        Command::Analyze { index, out } => {
            render_saved(std::path::Path::new(&index), std::path::Path::new(&out))?
        }
        Command::Run {
            amendment_revision,
            out,
        } => execute(std::path::Path::new(&out), &amendment_revision)?,
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use sugarscape_core::minds::spatial_hoarding::runner::{
        EpisodeEvents, ExposureTotals, FounderRecord, GenerationRecord, Lineage, Terminal,
    };

    #[test]
    fn minds9_manifest_materializes_every_declared_cell_without_worlds() {
        let m = manifest();
        assert_eq!(m.seeds, (1..=40).collect::<Vec<_>>());
        assert_eq!(m.timing_seeds, (1..=5).collect::<Vec<_>>());
        assert_eq!(m.conditions.len(), 81);
        assert_eq!(m.timing_conditions.len(), 21);
        for (panel, n) in [
            ("fixed", 18),
            ("discovery", 15),
            ("continuous", 9),
            ("contests", 18),
            ("sensitivity", 21),
        ] {
            assert_eq!(m.conditions.iter().filter(|c| c.panel == panel).count(), n);
        }
        for c in &m.conditions {
            for seed in &m.seeds {
                let a = c.materialize(*seed).unwrap();
                a.config.validate().unwrap();
                assert_eq!(a.initial.len(), 175);
                assert_eq!(
                    (a.config.episode_seed, a.config.breeding_seed),
                    (*seed, *seed)
                );
                assert_eq!(a, c.materialize(*seed).unwrap());
                if c.panel == "continuous" || c.panel == "discovery" || c.panel == "sensitivity" {
                    assert!(a
                        .initial
                        .iter()
                        .all(|t| !t.cheater && t.watches == a.config.episode.watching.on));
                }
            }
        }
    }

    #[test]
    fn minds9_panel_settings_include_exact_endpoints_rare_common_controls_and_probes() {
        let m = manifest();
        for l in [0.0, 0.5, 1.0] {
            for guard in [false, true] {
                for watch in [0, 2, 7] {
                    assert!(m.conditions.iter().any(|c| c.panel == "fixed"
                        && c.config.episode.spatial_hoarding.larder == l
                        && c.config.episode.spatial_hoarding.guard == guard
                        && c.watch() == watch));
                }
            }
        }
        for find in [0.0, 0.02, 0.05, 0.25, 1.0] {
            assert_eq!(
                m.conditions
                    .iter()
                    .filter(|c| c.panel == "discovery" && c.config.episode.theft.find == find)
                    .count(),
                3
            );
        }
        for family in ["scrounger", "watcher"] {
            for share in [0.1, 0.5, 0.9] {
                let cells: Vec<_> = m.conditions.iter().filter(|c| matches!(&c.cohort, CohortSpec::Contest { family: f, share: s } if f==family && *s==share)).collect();
                assert_eq!(cells.len(), 3);
                for cell in cells {
                    let a = cell.materialize(1).unwrap();
                    assert!(a.config.fixed_traits);
                    assert!(a
                        .initial
                        .iter()
                        .all(|t| t.larder == 0.0 && t.defense == 0.5));
                    let flagged = a
                        .initial
                        .iter()
                        .filter(|t| {
                            if family == "scrounger" {
                                t.cheater
                            } else {
                                t.watches
                            }
                        })
                        .count();
                    assert_eq!(flagged, (175.0_f64 * share).floor() as usize);
                }
            }
        }
        assert_eq!(
            m.conditions
                .iter()
                .filter(|c| c.config.probe.guard_harvest)
                .count(),
            3
        );
        assert_eq!(
            m.conditions
                .iter()
                .filter(|c| c.config.probe.scatter_first)
                .count(),
            3
        );
        assert_eq!(
            m.conditions
                .iter()
                .filter(|c| c.panel == "sensitivity"
                    && c.config.heritability == 1.0
                    && c.config.segregation_variance == 0.0)
                .count(),
            6
        );
    }

    fn fixture(terminal: Option<Terminal>) -> (Manifest, RunRow) {
        let mut m = manifest();
        m.conditions.truncate(1);
        m.seeds = vec![1];
        m.timing_conditions.clear();
        m.contrasts.clear();
        let c = &m.conditions[0];
        let a = c.materialize(1).unwrap();
        let alive = terminal != Some(Terminal::Extinct);
        let founders: Vec<_> = a
            .initial
            .iter()
            .enumerate()
            .map(|(slot, traits)| FounderRecord {
                slot,
                traits: *traits,
                alive,
                ticks_alive: 200,
                holdings: 0.0,
                scatter: 0.0,
                larder: 0.0,
                parent_weight: if alive { 1.0 } else { 0.0 },
                lineage: Lineage {
                    generation: 0,
                    slot,
                },
                parents: None,
            })
            .collect();
        let g = GenerationRecord {
            generation: 0,
            episode_seed: 1,
            breeding_seed: 1,
            founders,
            terminal,
            requested_ticks: 200,
            completed_ticks: 200,
            survivors: if alive { 175 } else { 0 },
            mean_larder: 0.0,
            mean_defense: 0.5,
            cheater_share: 0.0,
            watcher_share: 0.0,
            total_ticks_alive: 35000,
            closing_holdings: 0.0,
            closing_scatter: 0.0,
            closing_larder: 0.0,
            events: EpisodeEvents::default(),
            scatter_exposure: ExposureTotals::default(),
            larder_exposure: ExposureTotals::default(),
        };
        let row = RunRow {
            condition_id: c.id.clone(),
            seed: 1,
            envelope: RunEnvelope {
                config: a.config,
                initial: a.initial,
                initial_sampling: a.initial_sampling,
                draw_order: DRAW_ORDER.into(),
                generations: vec![g],
            },
        };
        (m, row)
    }

    #[test]
    fn minds9_terminal_rows_retained_and_undefined_exposure_absent() {
        let (m, row) = fixture(Some(Terminal::Extinct));
        let s = summarize_run(&m, &row).unwrap();
        assert_eq!(s.terminal, Some(Terminal::Extinct));
        assert_eq!(s.trajectory.len(), 1);
        assert_eq!(s.trajectory[0].metrics["scatter_loss_rate"], None);
        assert_eq!(s.trajectory[0].metrics["larder_pilferage_rate"], None);
        assert_eq!(s.endpoint.parent_weights.zero, 175);
    }

    #[test]
    fn minds9_rejects_duplicate_conditions_missing_seed_and_missing_terminal_rows() {
        let (m, row) = fixture(None);
        assert!(analyze_rows(&m, vec![row.clone(), row.clone()]).is_err());
        assert!(analyze_rows(&m, vec![]).is_err());
        let mut incomplete = row.clone();
        incomplete.envelope.generations.clear();
        assert!(summarize_run(&m, &incomplete).is_err());
        let mut longer = m.clone();
        longer.conditions[0].config.generations = 2;
        incomplete = row;
        incomplete.envelope.config.generations = 2;
        assert!(summarize_run(&longer, &incomplete).is_err());
    }

    #[test]
    fn minds9_rejects_nonfinite_measured_inputs_and_bad_summary_counts() {
        let (m, row) = fixture(None);
        let mut bad = row.clone();
        bad.envelope.generations[0].events.metabolism.consumed = f64::NAN;
        assert!(summarize_run(&m, &bad).is_err());
        bad = row;
        bad.envelope.generations[0].survivors = 174;
        assert!(summarize_run(&m, &bad).is_err());
    }

    #[test]
    fn minds9_cli_default_is_manifest_and_run_requires_revision_and_destination() {
        assert_eq!(parse_command(&[]).unwrap(), Command::Manifest);
        assert_eq!(parse_command(&["--help".into()]).unwrap(), Command::Help);
        assert!(parse_command(&["--run".into()]).is_err());
        assert!(parse_command(&["--seeds".into(), "3".into()]).is_err());
        assert!(parse_command(&[
            "--run".into(),
            "--amendment-revision".into(),
            "a".repeat(40),
            "--out".into(),
            "target".into()
        ])
        .is_ok());
    }
    #[test]
    fn minds9_timing_uses_completed_records_and_live_agent_ticks() {
        let t = Timing {
            tick_seconds: 2.0,
            runner_seconds: 9.0,
            completed_generations: 3,
            completed_ticks: 400,
            live_agent_ticks: 1000,
        };
        assert_eq!(t.per_agent_tick(), Some(0.002));
        assert_eq!(t.per_completed_generation(), Some(3.0));
    }
    #[test]
    fn minds9_validated_manifest_rejects_duplicate_ids_and_unknown_contrast_partner() {
        let mut m = manifest();
        validate_manifest(&m).unwrap();
        m.conditions[1].id = m.conditions[0].id.clone();
        assert!(validate_manifest(&m).is_err());
        m = manifest();
        m.contrasts[0].b = "missing".into();
        assert!(validate_manifest(&m).is_err());
    }
    #[test]
    fn minds9_zero_fitness_is_retained_distinct_from_extinction() {
        let (mut m, mut row) = fixture(None);
        m.conditions[0].config.selection = Selection::Stores;
        row.envelope.config.selection = Selection::Stores;
        row.envelope.generations[0].terminal = Some(Terminal::ZeroFitness);
        for f in &mut row.envelope.generations[0].founders {
            f.parent_weight = 0.0;
        }
        let s = summarize_run(&m, &row).unwrap();
        assert_eq!(s.terminal, Some(Terminal::ZeroFitness));
        assert_eq!(s.endpoint.metrics["survival"], Some(1.0));
    }
    #[test]
    fn minds9_exposure_ratios_use_pre_step_kind_denominators() {
        let (m, mut row) = fixture(None);
        let g = &mut row.envelope.generations[0];
        g.events.scatter.buried = 10.0;
        g.events.scatter.dug = 5.0;
        g.events.scatter.pilfered = 2.0;
        g.events.scatter.lost = 1.0;
        g.events.scatter.caches_pilfered = 2;
        g.scatter_exposure = ExposureTotals {
            cache_ticks: 4,
            stock_ticks: 30.0,
        };
        let s = summarize_run(&m, &row).unwrap();
        assert_eq!(s.endpoint.metrics["scatter_recovery"], Some(0.5));
        assert_eq!(s.endpoint.metrics["scatter_pilferage_rate"], Some(0.5));
        assert_eq!(s.endpoint.metrics["scatter_loss_rate"], Some(0.1));
        assert_eq!(s.endpoint.metrics["larder_loss_rate"], None);
    }

    #[test]
    fn minds9_analysis_declares_units_denominators_and_missingness() {
        let (m, row) = fixture(None);
        let a = analyze_rows(&m, vec![row]).unwrap();
        assert_eq!(
            a.metric_definitions["scatter_loss_rate"].denominator,
            "pre_step_stock_ticks"
        );
        assert_eq!(
            a.metric_definitions["metabolism_consumed"].unit,
            "food_units"
        );
        assert_eq!(
            a.metric_definitions["survival"].denominator,
            "all_archived_founders"
        );
        assert!(a.metric_definitions["watcher_change"]
            .missingness
            .contains("mechanism_off"));
    }
    #[test]
    fn minds9_full_envelope_json_roundtrip_keeps_supplied_cohort_and_terminal() {
        let (_, row) = fixture(Some(Terminal::Extinct));
        let encoded = serde_json::to_string(&row).unwrap();
        let decoded: RunRow = serde_json::from_str(&encoded).unwrap();
        assert_eq!(decoded, row);
    }

    #[test]
    fn minds9_sampled_extinct_endpoint_keeps_dead_genes_and_exact_json() {
        let (mut m, mut row) = fixture(Some(Terminal::Extinct));
        let c = manifest()
            .conditions
            .into_iter()
            .find(|c| c.id == "continuous-survival-w0")
            .unwrap();
        let a = c.materialize(1).unwrap();
        row.condition_id = c.id.clone();
        row.envelope.config = a.config;
        row.envelope.initial = a.initial.clone();
        row.envelope.initial_sampling = a.initial_sampling;
        let g = &mut row.envelope.generations[0];
        for (f, t) in g.founders.iter_mut().zip(a.initial) {
            f.traits = t;
        }
        g.mean_larder = g.founders.iter().map(|f| f.traits.larder).sum::<f64>() / 175.0;
        g.mean_defense = g.founders.iter().map(|f| f.traits.defense).sum::<f64>() / 175.0;
        m.conditions = vec![c];
        let s = summarize_run(&m, &row).unwrap();
        assert_eq!(
            s.endpoint.metrics["mean_larder"],
            Some(s.initial_mean_larder)
        );
        assert_eq!(s.endpoint.larder_traits.positive, 175);
        assert_eq!(s.completed_generations, 1);
        assert_eq!(
            serde_json::from_str::<RunRow>(&serde_json::to_string(&row).unwrap()).unwrap(),
            row
        );
    }
    #[test]
    fn minds9_markdown_reports_retained_terminal_and_mechanism_values() {
        let (m, row) = fixture(Some(Terminal::Extinct));
        let a = analyze_rows(&m, vec![row]).unwrap();
        let index = CampaignIndex {
            schema: SCHEMA.into(),
            code_revision: "a".repeat(40),
            amendment_revision: "b".repeat(40),
            amendment_path: AMENDMENT.into(),
            manifest: m,
            files: vec![],
        };
        let report = markdown(&a, &[], std::path::Path::new("index.json"), &index);
        assert!(report.contains("| fixed-l0-g0-w0 | 1 | 1 | 0 | 1–1 |"));
        assert!(report.contains("| fixed-l0-g0-w0 | 0 | 0.000 | 0.000 |"));
    }
    #[test]
    fn minds9_missing_ratio_makes_contrast_unavailable_without_dropping_seed() {
        let (mut m, row) = fixture(None);
        let mut b = m.conditions[0].clone();
        b.id = "second".into();
        m.conditions.push(b);
        m.contrasts.push(Contrast {
            id: "ratio".into(),
            a: m.conditions[0].id.clone(),
            b: "second".into(),
            metric: "scatter_loss_rate".into(),
            primary: false,
        });
        let mut second = row.clone();
        second.condition_id = "second".into();
        let a = analyze_rows(&m, vec![row, second]).unwrap();
        assert_eq!(a.contrasts[0].summary, None);
        assert_eq!(a.contrasts[0].missing_seeds, vec![1]);
    }
}
