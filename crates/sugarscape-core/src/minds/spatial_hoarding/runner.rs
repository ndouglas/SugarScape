//! Fresh seasonal worlds with archived founder slots and independent breeding randomness.
//! Episode seed is repeated each generation; initial maps, slots, bodies and endowments
//! therefore repeat. Only founder traits and their strategy flags cross the boundary.
//! Closing stocks remain closing stocks; resetting a season never records a loss.

use super::inheritance;
use super::state::{EpisodeProbe, FounderTraits};
use super::stores;
use crate::anasazi::random::normal;
use crate::config::Config;
use crate::hoard::{clamped_logit, inverse_logit};
use crate::rng;
use crate::world::World;
use serde::{Deserialize, Serialize};

/// Versioned, recorded random draw convention. Initial sampling has its own seed.
pub const DRAW_ORDER: &str = "v1: initial slot order: L normal, D normal; deterministic flags; breeding slot order: parent1 uniform, parent2 uniform, L normal, D normal; zero variance/fixed traits skip normals; independent initial, episode, breeding PCG64Mcg streams; episode seed repeated";

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Selection {
    Survival,
    Stores,
    Neutral,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Terminal {
    Extinct,
    ZeroFitness,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Lineage {
    pub generation: u32,
    pub slot: usize,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FounderRecord {
    pub slot: usize,
    pub traits: FounderTraits,
    pub alive: bool,
    /// Turns survived or attempted, including the turn on which starvation kills.
    pub ticks_alive: u64,
    /// Closing stocks: removed founders hold zero after their food leaves the world.
    pub holdings: f64,
    pub scatter: f64,
    pub larder: f64,
    pub parent_weight: f64,
    pub lineage: Lineage,
    pub parents: Option<[Lineage; 2]>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GenerationRecord {
    pub generation: u32,
    pub episode_seed: u64,
    /// Seed of the continuing breeding stream, not a reseed per generation.
    pub breeding_seed: u64,
    pub founders: Vec<FounderRecord>,
    pub terminal: Option<Terminal>,
    pub requested_ticks: u64,
    pub completed_ticks: u64,
    pub survivors: usize,
    /// Realized arithmetic means/shares across all archived founders, including the dead.
    pub mean_larder: f64,
    pub mean_defense: f64,
    pub cheater_share: f64,
    pub watcher_share: f64,
    pub total_ticks_alive: u64,
    pub closing_holdings: f64,
    pub closing_scatter: f64,
    pub closing_larder: f64,
    pub events: EpisodeEvents,
    pub scatter_exposure: ExposureTotals,
    pub larder_exposure: ExposureTotals,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RunnerConfig {
    pub episode: Config,
    pub founders: usize,
    pub ticks: u64,
    pub generations: u32,
    pub episode_seed: u64,
    pub breeding_seed: u64,
    pub selection: Selection,
    pub heritability: f64,
    pub segregation_variance: f64,
    pub fixed_traits: bool,
    pub probe: EpisodeProbe,
}

impl Default for RunnerConfig {
    fn default() -> Self {
        let mut episode = crate::presets::by_id("theft-winter")
            .expect("existing winter preset")
            .config;
        episode.spatial_hoarding.enabled = true;
        episode.watching.span = 2;
        Self {
            episode,
            founders: 175,
            ticks: 200,
            generations: 60,
            episode_seed: 1,
            breeding_seed: 1,
            selection: Selection::Survival,
            heritability: 0.8,
            segregation_variance: 0.5,
            fixed_traits: false,
            probe: EpisodeProbe::default(),
        }
    }
}

impl RunnerConfig {
    pub fn validate(&self) -> Result<(), String> {
        if self.founders == 0 || self.ticks == 0 || self.generations == 0 {
            return Err("founders, ticks and generations must be positive".into());
        }
        if self.founders != self.episode.population as usize {
            return Err("founders must equal episode.population".into());
        }
        if !self.episode.spatial_hoarding.enabled {
            return Err("seasonal runner needs spatial_hoarding.enabled".into());
        }
        if !self.heritability.is_finite() || !(0.0..=1.0).contains(&self.heritability) {
            return Err("heritability must be finite and between 0 and 1".into());
        }
        if !self.segregation_variance.is_finite() || self.segregation_variance < 0.0 {
            return Err("segregation_variance must be finite and nonnegative".into());
        }
        self.episode
            .validate()
            .map_err(|e| format!("invalid episode: {e:?}"))
    }
    /// The replay boundary: repeats the episode seed and applies supplied traits before tick zero.
    pub fn episode_world(&self, cohort: &[FounderTraits]) -> Result<World, String> {
        self.validate()?;
        World::new_with_spatial_probe(self.episode.clone(), self.episode_seed, cohort, self.probe)
            .map_err(|e| format!("invalid cohort: {e:?}"))
    }
}

/// Minds 7 centers and logit-scale spread. This distribution is not a claim about
/// realized arithmetic population means. Flags follow ordinary founder-id assignment.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct InitialCohortConfig {
    pub seed: u64,
    pub larder_center: f64,
    pub defense_center: f64,
    pub variance: f64,
}
impl Default for InitialCohortConfig {
    fn default() -> Self {
        Self {
            seed: 1,
            larder_center: 0.15,
            defense_center: 0.5,
            variance: 0.5,
        }
    }
}

pub fn sample_initial_cohort(
    episode: &Config,
    init: &InitialCohortConfig,
) -> Result<Vec<FounderTraits>, String> {
    episode
        .validate()
        .map_err(|e| format!("invalid episode: {e:?}"))?;
    if !episode.spatial_hoarding.enabled || episode.population == 0 {
        return Err("initial cohort needs spatial hoarding and positive population".into());
    }
    for center in [init.larder_center, init.defense_center] {
        if !center.is_finite() || !(0.0..=1.0).contains(&center) {
            return Err("initial centers must be finite probabilities".into());
        }
    }
    if !init.variance.is_finite() || init.variance < 0.0 {
        return Err("initial variance must be finite and nonnegative".into());
    }
    let mut rng = rng::seeded(init.seed);
    let sd = init.variance.sqrt();
    Ok((1..=u64::from(episode.population))
        .map(|id| {
            let larder = inverse_logit(
                clamped_logit(init.larder_center)
                    + if sd > 0.0 { sd * normal(&mut rng) } else { 0.0 },
            );
            let defense = inverse_logit(
                clamped_logit(init.defense_center)
                    + if sd > 0.0 { sd * normal(&mut rng) } else { 0.0 },
            );
            let cheater = episode.theft.founder_cheats(id);
            let watches = episode.watching.founder_watches(id, cheater);
            FounderTraits {
                larder,
                defense,
                cheater,
                watches,
            }
        })
        .collect())
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RunEnvelope {
    pub config: RunnerConfig,
    pub initial: Vec<FounderTraits>,
    /// None denotes an explicitly supplied cohort; the full cohort is always retained.
    pub initial_sampling: Option<InitialCohortConfig>,
    pub draw_order: String,
    pub generations: Vec<GenerationRecord>,
}

pub fn run(config: &RunnerConfig, initial: &[FounderTraits]) -> Result<RunEnvelope, String> {
    Ok(RunEnvelope {
        config: config.clone(),
        initial: initial.to_vec(),
        initial_sampling: None,
        draw_order: DRAW_ORDER.into(),
        generations: run_generations(config, initial)?,
    })
}
pub fn run_sampled(
    config: &RunnerConfig,
    init: &InitialCohortConfig,
) -> Result<RunEnvelope, String> {
    config.validate()?;
    let initial = sample_initial_cohort(&config.episode, init)?;
    let mut envelope = run(config, &initial)?;
    envelope.initial_sampling = Some(init.clone());
    Ok(envelope)
}

/// Exact ecological weights. Entire extinction takes precedence even for neutral control.
pub fn parent_weights(
    selection: Selection,
    founders: &[FounderRecord],
) -> Result<Vec<f64>, Terminal> {
    if !founders.iter().any(|f| f.alive) {
        return Err(Terminal::Extinct);
    }
    let weights: Vec<_> = founders
        .iter()
        .map(|f| match selection {
            Selection::Survival => f64::from(f.alive),
            Selection::Stores => {
                if f.alive {
                    f.scatter + f.larder
                } else {
                    0.0
                }
            }
            Selection::Neutral => 1.0,
        })
        .collect();
    if weights.iter().sum::<f64>() <= 0.0 {
        Err(Terminal::ZeroFitness)
    } else {
        Ok(weights)
    }
}

/// Reports every completed season including a terminal one. Breeding RNG persists
/// across generation boundaries and is never consulted by episode construction or steps.
pub fn run_generations(
    config: &RunnerConfig,
    initial: &[FounderTraits],
) -> Result<Vec<GenerationRecord>, String> {
    config.validate()?;
    let mut cohort = initial.to_vec();
    let mut parents = vec![None; config.founders];
    let mut breeding_rng = rng::seeded(config.breeding_seed);
    let mut records = Vec::new();
    for generation in 0..config.generations {
        let mut world = config.episode_world(&cohort)?;
        let ids: Vec<_> = world.agents().map(|a| a.id).collect();
        let mut ticks_alive = vec![0; config.founders];
        let mut events = EpisodeEvents::default();
        let mut scatter_exposure = ExposureTotals::default();
        let mut larder_exposure = ExposureTotals::default();
        for _ in 0..config.ticks {
            if world.population() == 0 {
                break;
            }
            for (slot, &id) in ids.iter().enumerate() {
                if world.agent(id).is_some() {
                    ticks_alive[slot] += 1;
                }
            }
            for a in world.agents() {
                for &amount in a.caches.values() {
                    scatter_exposure.add(amount);
                }
                larder_exposure.add(a.spatial.as_ref().expect("spatial founder").larder);
            }
            world.step();
            if let Some(tick) = world.events().spatial_stores {
                events.add(tick);
            }
        }
        let mut founders: Vec<_> = cohort
            .iter()
            .enumerate()
            .map(|(slot, &traits)| {
                let a = world.agent(ids[slot]);
                FounderRecord {
                    slot,
                    traits,
                    alive: a.is_some(),
                    ticks_alive: ticks_alive[slot],
                    holdings: a.map_or(0.0, |a| a.holdings[0]),
                    scatter: a.map_or(0.0, |a| a.caches.values().sum()),
                    larder: a.and_then(|a| a.spatial.as_ref()).map_or(0.0, |s| s.larder),
                    parent_weight: 0.0,
                    lineage: Lineage { generation, slot },
                    parents: parents[slot],
                }
            })
            .collect();
        let weights = parent_weights(config.selection, &founders);
        let terminal = weights.as_ref().err().copied();
        if let Ok(weights) = &weights {
            for (f, &weight) in founders.iter_mut().zip(weights) {
                f.parent_weight = weight;
            }
        }
        let n = config.founders as f64;
        let record = GenerationRecord {
            generation,
            episode_seed: config.episode_seed,
            breeding_seed: config.breeding_seed,
            terminal,
            requested_ticks: config.ticks,
            completed_ticks: world.tick,
            survivors: world.population(),
            mean_larder: cohort.iter().map(|t| t.larder).sum::<f64>() / n,
            mean_defense: cohort.iter().map(|t| t.defense).sum::<f64>() / n,
            cheater_share: cohort.iter().filter(|t| t.cheater).count() as f64 / n,
            watcher_share: cohort.iter().filter(|t| t.watches).count() as f64 / n,
            total_ticks_alive: ticks_alive.iter().sum(),
            closing_holdings: founders.iter().map(|f| f.holdings).sum(),
            closing_scatter: founders.iter().map(|f| f.scatter).sum(),
            closing_larder: founders.iter().map(|f| f.larder).sum(),
            founders,
            events,
            scatter_exposure,
            larder_exposure,
        };
        records.push(record);
        if terminal.is_some() || generation + 1 == config.generations {
            break;
        }
        let children = inheritance::breed(
            &records.last().expect("just recorded").founders,
            &weights.expect("nonterminal has weights"),
            config.founders,
            config.heritability,
            config.segregation_variance,
            config.fixed_traits,
            &mut breeding_rng,
        );
        cohort = children.iter().map(|c| c.traits).collect();
        parents = children.iter().map(|c| Some(c.parents)).collect();
    }
    Ok(records)
}

/// Tick-start nonempty caches and food units, summed before each executed tick.
/// Cache-ticks and food-unit-ticks are recorded even with discovery/watching off.
/// Closing stocks at the episode boundary do not contribute another tick.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ExposureTotals {
    pub cache_ticks: u64,
    pub stock_ticks: f64,
}
impl ExposureTotals {
    fn add(&mut self, amount: f64) {
        if amount > 0.0 {
            self.cache_ticks += 1;
            self.stock_ticks += amount;
        }
    }
}

/// All extension event fields summed over executed ticks; counts widen to u64.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct EpisodeEvents {
    pub scatter: FlowTotals,
    pub larder: FlowTotals,
    pub delivery: DeliveryTotals,
    pub guard: GuardTotals,
    pub observation: ObservationTotals,
    pub metabolism: MetabolismTotals,
}
impl EpisodeEvents {
    fn add(&mut self, e: stores::StoreEvents) {
        self.scatter.add(e.scatter);
        self.larder.add(e.larder);
        self.delivery.add(e.delivery);
        self.guard.add(e.guard);
        self.observation.add(e.observation);
        self.metabolism.add(e.metabolism);
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct FlowTotals {
    pub buried: f64,
    pub dug: f64,
    pub pilfered: f64,
    pub lost: f64,
    pub bury_cost: f64,
    pub loot_eaten: f64,
    pub digs: u64,
    pub pilfers: u64,
    pub pilfer_candidates: u64,
    pub caches_pilfered: u64,
}
impl FlowTotals {
    fn add(&mut self, e: stores::StoreFlow) {
        self.buried += e.buried;
        self.dug += e.dug;
        self.pilfered += e.pilfered;
        self.lost += e.lost;
        self.bury_cost += e.bury_cost;
        self.loot_eaten += e.loot_eaten;
        self.digs += u64::from(e.digs);
        self.pilfers += u64::from(e.pilfers);
        self.pilfer_candidates += u64::from(e.pilfer_candidates);
        self.caches_pilfered += u64::from(e.caches_pilfered);
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct DeliveryTotals {
    pub starts: u64,
    pub completions: u64,
    pub cancellations: u64,
    pub return_turns: u64,
    pub delivered: f64,
    pub bury_cost: f64,
}
impl DeliveryTotals {
    fn add(&mut self, e: stores::DeliveryEvents) {
        self.starts += u64::from(e.starts);
        self.completions += u64::from(e.completions);
        self.cancellations += u64::from(e.cancellations);
        self.return_turns += u64::from(e.return_turns);
        self.delivered += e.delivered;
        self.bury_cost += e.bury_cost;
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct GuardTotals {
    pub intended: u64,
    pub executed: u64,
    pub recovered: f64,
    pub probe_harvest: f64,
    pub blocked_raids: u64,
    pub blocked_discoveries: u64,
}
impl GuardTotals {
    fn add(&mut self, e: stores::GuardEvents) {
        self.intended += u64::from(e.intended);
        self.executed += u64::from(e.executed);
        self.recovered += e.recovered;
        self.probe_harvest += e.probe_harvest;
        self.blocked_raids += u64::from(e.blocked_raids);
        self.blocked_discoveries += u64::from(e.blocked_discoveries);
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ObservationTotals {
    pub burials_seen: u64,
    pub sightings: u64,
    pub seen_entries: u64,
    pub seen_arrivals: u64,
    pub contacts: u64,
    pub discovery_draws: u64,
    pub discovery_hits: u64,
    pub raid_attempts: u64,
    pub raids: u64,
    pub raided: f64,
    pub raids_empty: u64,
    pub raids_no_room: u64,
    pub raids_blocked: u64,
    pub discoveries_blocked: u64,
    pub scatter_draws_skipped: u64,
}
impl ObservationTotals {
    fn add(&mut self, e: stores::ObservationEvents) {
        self.burials_seen += u64::from(e.burials_seen);
        self.sightings += u64::from(e.sightings);
        self.seen_entries += u64::from(e.seen_entries);
        self.seen_arrivals += u64::from(e.seen_arrivals);
        self.contacts += u64::from(e.contacts);
        self.discovery_draws += u64::from(e.discovery_draws);
        self.discovery_hits += u64::from(e.discovery_hits);
        self.raid_attempts += u64::from(e.raid_attempts);
        self.raids += u64::from(e.raids);
        self.raided += e.raided;
        self.raids_empty += u64::from(e.raids_empty);
        self.raids_no_room += u64::from(e.raids_no_room);
        self.raids_blocked += u64::from(e.raids_blocked);
        self.discoveries_blocked += u64::from(e.discoveries_blocked);
        self.scatter_draws_skipped += u64::from(e.scatter_draws_skipped);
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct MetabolismTotals {
    pub demand: f64,
    pub consumed: f64,
}
impl MetabolismTotals {
    fn add(&mut self, e: stores::MetabolismEvents) {
        self.demand += e.demand;
        self.consumed += e.consumed;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{CachingRule, MoveMode, URange};

    fn traits(l: f64, d: f64) -> FounderTraits {
        FounderTraits {
            larder: l,
            defense: d,
            cheater: false,
            watches: false,
        }
    }
    fn founder(slot: usize, alive: bool, scatter: f64, larder: f64) -> FounderRecord {
        FounderRecord {
            slot,
            traits: traits(0.15, 0.5),
            alive,
            ticks_alive: 0,
            holdings: 99.0,
            scatter,
            larder,
            parent_weight: 0.0,
            lineage: Lineage {
                generation: 0,
                slot,
            },
            parents: None,
        }
    }
    fn config() -> RunnerConfig {
        let mut c = Config {
            population: 3,
            ..Config::default()
        };
        c.spatial_hoarding.enabled = true;
        c.caching.rule = CachingRule::Even;
        c.caching.capacity = 50;
        c.movement.mode = MoveMode::Walk;
        RunnerConfig {
            episode: c,
            founders: 3,
            ticks: 3,
            generations: 3,
            episode_seed: 7,
            breeding_seed: 11,
            selection: Selection::Survival,
            heritability: 0.8,
            segregation_variance: 0.5,
            fixed_traits: false,
            probe: EpisodeProbe::default(),
        }
    }
    #[test]
    fn exact_parent_weights_exclude_dead_except_neutral() {
        let f = [
            founder(0, true, 0.0, 0.0),
            founder(1, true, 3.0, 4.0),
            founder(2, false, 9.0, 9.0),
        ];
        assert_eq!(
            parent_weights(Selection::Survival, &f),
            Ok(vec![1.0, 1.0, 0.0])
        );
        assert_eq!(
            parent_weights(Selection::Stores, &f),
            Ok(vec![0.0, 7.0, 0.0])
        );
        assert_eq!(
            parent_weights(Selection::Neutral, &f),
            Ok(vec![1.0, 1.0, 1.0])
        );
    }
    #[test]
    fn zero_stores_and_extinction_are_distinct_without_fallback() {
        let live = [founder(0, true, 0.0, 0.0)];
        assert_eq!(
            parent_weights(Selection::Stores, &live),
            Err(Terminal::ZeroFitness)
        );
        assert_eq!(parent_weights(Selection::Survival, &live), Ok(vec![1.0]));
        for selection in [Selection::Survival, Selection::Stores, Selection::Neutral] {
            assert_eq!(
                parent_weights(selection, &[founder(0, false, 4.0, 5.0)]),
                Err(Terminal::Extinct)
            );
        }
    }
    #[test]
    fn serialized_envelope_replays_every_archived_cohort() {
        let c = config();
        let initial = vec![traits(0.0, 0.5); 3];
        let run = run(&c, &initial).unwrap();
        assert_eq!(run, super::run(&c, &initial).unwrap());
        let json = serde_json::to_string(&run).unwrap();
        let saved: RunEnvelope = serde_json::from_str(&json).unwrap();
        assert_eq!(
            run_generations(&saved.config, &saved.initial).unwrap(),
            saved.generations
        );
        for record in &saved.generations {
            let cohort: Vec<_> = record.founders.iter().map(|f| f.traits).collect();
            let mut w = saved.config.episode_world(&cohort).unwrap();
            for _ in 0..record.completed_ticks {
                w.step();
            }
            assert_eq!(w.population(), record.survivors);
            assert_eq!(
                w.agents().map(|a| a.holdings[0]).sum::<f64>(),
                record.closing_holdings
            );
        }
    }
    #[test]
    fn terminal_generation_is_retained_with_death_tick() {
        let mut c = config();
        c.episode.goods[0].endowment = URange::new(1, 1);
        c.episode.goods[0].metabolism = URange::new(100, 100);
        let r = run_generations(&c, &[traits(0.0, 0.5); 3]).unwrap();
        assert_eq!(r.len(), 1);
        assert_eq!(r[0].terminal, Some(Terminal::Extinct));
        assert_eq!(r[0].completed_ticks, 1);
        assert!(r[0]
            .founders
            .iter()
            .all(|f| !f.alive && f.ticks_alive == 1 && f.parent_weight == 0.0));
        assert_eq!(
            serde_json::to_string(&Terminal::ZeroFitness).unwrap(),
            "\"zero_fitness\""
        );
    }
    #[test]
    fn invalid_runner_configuration_and_cohorts_are_rejected() {
        let valid = config();
        for bad in 0..8 {
            let mut c = valid.clone();
            match bad {
                0 => c.founders = 0,
                1 => c.ticks = 0,
                2 => c.generations = 0,
                3 => c.heritability = f64::NAN,
                4 => c.heritability = 1.1,
                5 => c.segregation_variance = -1.0,
                6 => c.segregation_variance = f64::INFINITY,
                _ => c.founders = 2,
            }
            assert!(run_generations(&c, &[traits(0.0, 0.5); 3]).is_err());
        }
        assert!(run_generations(&valid, &[]).is_err());
        assert!(run_generations(&valid, &[traits(f64::NAN, 0.5); 3]).is_err());
    }
    #[test]
    fn initial_sampling_has_minds7_centers_spread_draw_order_and_flags() {
        let mut c = config();
        c.episode.theft.cheaters = 0.5;
        c.episode.watching.on = true;
        c.episode.watching.watchers = 0.5;
        let init = InitialCohortConfig {
            seed: 19,
            ..InitialCohortConfig::default()
        };
        let sample = sample_initial_cohort(&c.episode, &init).unwrap();
        let mut random = crate::rng::seeded(19);
        for (slot, f) in sample.iter().enumerate() {
            let l = crate::hoard::inverse_logit(
                crate::hoard::clamped_logit(0.15)
                    + 0.5_f64.sqrt() * crate::anasazi::random::normal(&mut random),
            );
            let d = crate::hoard::inverse_logit(
                0.5_f64.sqrt() * crate::anasazi::random::normal(&mut random),
            );
            assert_eq!((f.larder, f.defense), (l, d));
            let id = slot as u64 + 1;
            assert_eq!(f.cheater, c.episode.theft.founder_cheats(id));
            assert_eq!(f.watches, c.episode.watching.founder_watches(id, f.cheater));
        }
        let envelope = run_sampled(&c, &init).unwrap();
        assert_eq!(envelope.initial_sampling, Some(init));
        assert_eq!(envelope.initial, sample);
    }
    #[test]
    fn fresh_episode_resets_stores_intents_and_paths_with_same_bodies_and_map() {
        let c = config();
        let initial = vec![traits(0.8, 0.5); 3];
        let mut old = c.episode_world(&initial).unwrap();
        let bodies: Vec<_> = old
            .agents()
            .map(|a| (a.id, a.pos, a.holdings, a.metabolism, a.vision))
            .collect();
        let map = old.sites.clone();
        let id = old.agents().next().unwrap().id;
        let a = old.agent_mut(id).unwrap();
        a.holdings[0] = 1.0;
        a.fed = 3.0;
        a.caches.insert(0, 4.0);
        a.plan.path.push(a.pos);
        let s = a.spatial.as_mut().unwrap();
        s.larder = 5.0;
        s.guarding = true;
        s.delivery = Some(super::super::state::Delivery { amount: 1.0 });
        s.seen_larders.insert(
            99,
            super::super::state::SeenLarder {
                home: a.pos,
                amount: 3.0,
                tick: 0,
            },
        );
        let fresh = c.episode_world(&[traits(0.2, 0.9); 3]).unwrap();
        assert_eq!(
            fresh
                .agents()
                .map(|a| (a.id, a.pos, a.holdings, a.metabolism, a.vision))
                .collect::<Vec<_>>(),
            bodies
        );
        assert_eq!(fresh.sites, map);
        assert!(fresh.cache_log.is_empty() && fresh.cache_open.is_empty());
        for a in fresh.agents() {
            let s = a.spatial.as_ref().unwrap();
            assert!(a.caches.is_empty() && a.seen.is_empty() && a.plan.path.is_empty());
            assert_eq!(
                (
                    s.larder,
                    s.guarding,
                    s.delivery,
                    s.seen_larders.len(),
                    a.fed
                ),
                (0.0, false, None, 0, 0.0)
            );
        }
    }

    #[test]
    fn json_preserves_each_continuous_trait_bit_for_replay() {
        let run = run(&config(), &[traits(0.0, 0.5); 3]).unwrap();
        for f in run.generations.iter().flat_map(|g| &g.founders) {
            let encoded = serde_json::to_string(&f.traits).unwrap();
            let decoded: FounderTraits = serde_json::from_str(&encoded).unwrap();
            assert_eq!(
                decoded.larder.to_bits(),
                f.traits.larder.to_bits(),
                "{encoded}"
            );
            assert_eq!(
                decoded.defense.to_bits(),
                f.traits.defense.to_bits(),
                "{encoded}"
            );
        }
    }
    #[test]
    fn live_zero_fitness_season_finishes_and_keeps_terminal_summary() {
        let mut c = config();
        c.selection = Selection::Stores;
        c.episode.goods[0].endowment = URange::new(40, 40);
        c.episode.goods[0].metabolism = URange::new(1, 1);
        let initial = [FounderTraits {
            cheater: true,
            ..traits(0.0, 0.5)
        }; 3];
        let records = run_generations(&c, &initial).unwrap();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].terminal, Some(Terminal::ZeroFitness));
        assert_eq!(records[0].completed_ticks, c.ticks);
        assert_eq!(records[0].survivors, 3);
        assert!(records[0].closing_holdings > 0.0);
    }
    #[test]
    fn closing_stock_remains_food_and_exposure_counts_without_discovery() {
        let mut c = config();
        c.fixed_traits = true;
        c.episode.goods[0].endowment = URange::new(40, 40);
        c.episode.goods[0].metabolism = URange::new(1, 1);
        c.episode.theft.find = 0.0;
        c.episode.spatial_hoarding.find_larder = 0.0;
        let records = run_generations(&c, &[traits(0.0, 0.5); 3]).unwrap();
        let mut replay = c.episode_world(&[traits(0.0, 0.5); 3]).unwrap();
        let mut tick_start_stock = 0.0;
        let mut tick_start_caches = 0;
        for _ in 0..c.ticks {
            tick_start_stock += replay.agents().flat_map(|a| a.caches.values()).sum::<f64>();
            tick_start_caches += replay.agents().map(|a| a.caches.len() as u64).sum::<u64>();
            replay.step();
        }
        for r in records {
            assert!(r.closing_scatter > 0.0);
            assert_eq!(
                r.events.scatter.buried
                    - r.events.scatter.dug
                    - r.events.scatter.pilfered
                    - r.events.scatter.lost,
                r.closing_scatter
            );
            assert_eq!(r.events.scatter.lost, 0.0);
            assert!(r.scatter_exposure.cache_ticks > 0 && r.scatter_exposure.stock_ticks > 0.0);
            assert_eq!(r.scatter_exposure.cache_ticks, tick_start_caches);
            assert!((r.scatter_exposure.stock_ticks - tick_start_stock).abs() < 1e-9);
            assert_eq!(r.events.metabolism.demand, 9.0);
            assert_eq!(r.events.metabolism.consumed, 9.0);
        }
    }
}
