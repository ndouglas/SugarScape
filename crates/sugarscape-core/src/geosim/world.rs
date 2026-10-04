//! Full source periods on the shared seeded portable engine.
use super::*;
use crate::{
    config::FieldError,
    rng::{self, SimRng},
    stats::Stats,
};
use rand::{Rng, RngCore};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Front {
    pub states: [StateId; 2],
    pub previous: [bool; 2],
    pub actions: [bool; 2],
    pub old_commitments: [f64; 2],
    pub commitments: [f64; 2],
    pub path: Option<[usize; 2]>,
    pub initiator: Option<usize>,
    pub last_damage: [f64; 2],
    pub last_victory_probabilities: [Option<f64>; 2],
}
impl Front {
    fn new(states: [StateId; 2]) -> Self {
        Self {
            states,
            previous: [false; 2],
            actions: [false; 2],
            old_commitments: [0.0; 2],
            commitments: [0.0; 2],
            path: None,
            initiator: None,
            last_damage: [0.0; 2],
            last_victory_probabilities: [None; 2],
        }
    }
}
#[derive(Clone)]
pub struct GeosimWorld {
    pub config: GeosimConfig,
    pub tick: u64,
    pub stats: Stats<GeosimSnapshot>,
    pub(crate) period: u64,
    pub(crate) completed: u64,
    pub(crate) cells: Vec<Cell>,
    pub(crate) states: BTreeMap<StateId, State>,
    pub(crate) members: BTreeMap<StateId, Vec<usize>>,
    pub(crate) fronts: BTreeMap<[StateId; 2], Front>,
    pub(crate) relations: BTreeMap<StateId, Vec<[StateId; 2]>>,
    pub(crate) seed: u64,
    pub(crate) rng: SimRng,
    pub(crate) tracker: WarTracker,
    pub(crate) retired: Vec<StateId>,
    pub(crate) ledger: Ledger,
    pub(crate) result: Option<Outcome>,
    pub(crate) events: Vec<Event>,
    pub(crate) events_dropped: u64,
    pub(crate) next_event: u64,
    pub(crate) last_structural_event: Option<Event>,
    pub(crate) last_tick_periods: u32,
    pub(crate) pending_fights: Vec<(StateId, StateId, f64)>,
    pub(crate) tracked_period: u64,
    pub(crate) prior_fighting: BTreeSet<StateId>,
    pub(crate) resource_updates: Vec<ResourceUpdate>,
}
pub(crate) fn hash(bytes: &[u8]) -> u64 {
    bytes.iter().fold(14695981039346656037u64, |h, &b| {
        (h ^ u64::from(b)).wrapping_mul(1099511628211)
    })
}
pub(crate) fn shuffle<T>(v: &mut [T], rng: &mut SimRng) {
    for i in (1..v.len()).rev() {
        let upper = u32::try_from(i).expect("validated GeoSim shuffle bound");
        let j = rng.gen_range(0_u32..=upper) as usize;
        v.swap(i, j);
    }
}
pub(crate) fn pick<T: Copy>(v: &[T], rng: &mut SimRng) -> Option<T> {
    match v.len() {
        0 => None,
        1 => Some(v[0]),
        n => {
            let upper = u32::try_from(n).expect("validated GeoSim selection bound");
            Some(v[rng.gen_range(0_u32..upper) as usize])
        }
    }
}
pub(crate) fn chance(p: f64, rng: &mut SimRng) -> bool {
    if p == 0.0 {
        false
    } else if p == 1.0 {
        true
    } else {
        rng.gen::<f64>() < p
    }
}
impl GeosimWorld {
    pub fn new(config: GeosimConfig, seed: u64) -> Result<Self, Vec<FieldError>> {
        config.validate()?;
        let n = (config.width * config.height) as usize;
        let mut rng = rng::seeded(seed);
        let mut order: Vec<_> = (0..n).collect();
        shuffle(&mut order, &mut rng);
        let founders = order[..config.initial_states as usize].to_vec();
        let founder_set: BTreeSet<_> = founders.iter().copied().collect();
        let mut owners: Vec<_> = (0..n)
            .map(|i| StateId {
                capital_cell: i,
                sovereignty_generation: 0,
            })
            .collect();
        let primitive_resources: Vec<f64> = (0..n)
            .map(|_| {
                if config.initial_capacity == InitialCapacity::ArtifactRandom1001 {
                    if chance(0.2, &mut rng) {
                        100.0
                    } else {
                        1.0
                    }
                } else {
                    0.0
                }
            })
            .collect();
        let mut unassigned: BTreeSet<_> = (0..n).filter(|i| !founder_set.contains(i)).collect();
        let mut growth_members: BTreeMap<_, Vec<usize>> =
            founders.iter().map(|&f| (f, vec![f])).collect();
        while !unassigned.is_empty() {
            let before = unassigned.len();
            let mut pass = founders.clone();
            if config.founder_growth == FounderGrowth::ShuffledRoundRobin {
                shuffle(&mut pass, &mut rng);
            }
            for f in pass {
                let eligible: BTreeSet<_> = growth_members[&f]
                    .iter()
                    .flat_map(|&id| territory::adjacent(&config, id))
                    .filter(|id| unassigned.contains(id))
                    .collect();
                let eligible: Vec<_> = eligible.into_iter().collect();
                if let Some(id) = pick(&eligible, &mut rng) {
                    owners[id] = owners[f];
                    unassigned.remove(&id);
                    growth_members.get_mut(&f).unwrap().push(id);
                }
            }
            if unassigned.len() == before {
                return Err(vec![FieldError::new(
                    "founder_growth",
                    "no growth progress with unassigned cells",
                )]);
            }
        }
        let cells = owners
            .into_iter()
            .enumerate()
            .map(|(id, owner)| Cell {
                id,
                owner,
                last_threshold: config.distance_threshold,
                next_generation: 0,
            })
            .collect();
        let mut w = Self {
            config,
            tick: 0,
            stats: Stats::default(),
            period: 0,
            completed: 0,
            cells,
            states: BTreeMap::new(),
            members: BTreeMap::new(),
            fronts: BTreeMap::new(),
            relations: BTreeMap::new(),
            seed,
            rng,
            tracker: WarTracker::default(),
            retired: Vec::new(),
            ledger: Ledger::default(),
            result: None,
            events: Vec::new(),
            events_dropped: 0,
            next_event: 0,
            last_structural_event: None,
            last_tick_periods: 0,
            pending_fights: Vec::new(),
            tracked_period: 0,
            prior_fighting: BTreeSet::new(),
            resource_updates: Vec::new(),
        };
        for &f in &founders {
            let id = w.cells[f].owner;
            w.states.insert(
                id,
                State {
                    id,
                    capacity: Some(0.0),
                    threshold: w.config.distance_threshold,
                    alert: false,
                    campaign: None,
                    previous_damage: 0.0,
                    newly_independent: w.config.initial_capacity
                        == InitialCapacity::ArtifactRandom1001,
                    extracted_yield: 0.0,
                    recurrence_residual: 0.0,
                },
            );
        }
        w.rebuild();
        for &f in &founders {
            let id = w.cells[f].owner;
            let yield_capacity = w.yield_capacity(id);
            let initial = if w.config.initial_capacity == InitialCapacity::ArtifactRandom1001 {
                w.members[&id].iter().map(|&i| primitive_resources[i]).sum()
            } else {
                yield_capacity
            };
            let s = w.states.get_mut(&id).unwrap();
            s.capacity = Some(initial);
            s.extracted_yield = yield_capacity;
        }
        w.record();
        Ok(w)
    }
    pub(crate) fn rebuild(&mut self) {
        self.members.clear();
        for c in &self.cells {
            self.members.entry(c.owner).or_default().push(c.id);
        }
        let mut keys = BTreeSet::new();
        for cell in &self.cells {
            for other in territory::adjacent(&self.config, cell.id) {
                let owner = self.cells[other].owner;
                if owner != cell.owner {
                    let key = if cell.owner < owner {
                        [cell.owner, owner]
                    } else {
                        [owner, cell.owner]
                    };
                    keys.insert(key);
                }
            }
        }
        self.relations.clear();
        for &key in &keys {
            self.relations.entry(key[0]).or_default().push(key);
            self.relations.entry(key[1]).or_default().push(key);
        }
        self.fronts.retain(|k, _| keys.contains(k));
        for key in keys {
            self.fronts.entry(key).or_insert_with(|| Front::new(key));
        }
        for f in self.fronts.values_mut() {
            if let Some([a, b]) = f.path {
                if self.cells[a].owner != f.states[0]
                    || self.cells[b].owner != f.states[1]
                    || !territory::adjacent(&self.config, a).contains(&b)
                {
                    f.path = None;
                }
            }
        }
    }
    pub(crate) fn projection(&self, state: StateId, cell: usize) -> f64 {
        let s = &self.states[&state];
        resources::distance_curve(
            territory::distance(&self.config, state.capital_cell, cell),
            self.config.distance_offset,
            s.threshold,
            self.config.distance_exponent,
            self.config.distance_formula == DistanceFormula::PrintedIncreasing,
        )
    }
    pub(crate) fn yield_capacity(&self, id: StateId) -> f64 {
        self.members[&id]
            .iter()
            .map(|&cell| self.projection(id, cell))
            .sum()
    }
    pub(crate) fn counting_start(&self) -> u64 {
        match self.config.count_boundary {
            CountBoundary::AfterInitialization => self.config.initialization_periods + 1,
            CountBoundary::AtInitialization => self.config.initialization_periods.max(1),
        }
    }
    pub fn run(&mut self, ticks: u32) {
        for _ in 0..ticks {
            if self.result.is_some() {
                break;
            }
            self.last_tick_periods = 0;
            for _ in 0..self.config.periods_per_tick {
                if self.result.is_some() {
                    break;
                }
                self.period += 1;
                let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| self.step()));
                match result {
                    Ok(Ok(())) => {
                        self.completed = self.period;
                        self.last_tick_periods += 1;
                        if self.completed == self.config.horizon() {
                            self.finish(true, "horizon", None);
                        }
                    }
                    Ok(Err(message)) => self.finish(false, "invalid", Some(message)),
                    Err(p) => {
                        let message = p
                            .downcast_ref::<String>()
                            .cloned()
                            .or_else(|| p.downcast_ref::<&str>().map(|s| s.to_string()))
                            .unwrap_or_else(|| "unknown period panic".into());
                        self.invalidate_after_panic(message);
                    }
                }
            }
            self.tick += 1;
            self.record();
        }
    }
    pub(crate) fn update_resources(&mut self) -> Result<(), String> {
        self.resource_updates.clear();
        let ids: Vec<_> = self.states.keys().copied().collect();
        for id in ids {
            let yield_capacity = self.yield_capacity(id);
            let s = self.states.get_mut(&id).unwrap();
            let old = s.capacity.ok_or("unavailable capacity")?;
            let loss = s.previous_damage;
            let reset = s.newly_independent;
            let target = yield_capacity
                + if self.config.damage_feedback == DamageFeedback::AddLosses {
                    loss
                } else {
                    -loss
                };
            let computed = if reset {
                yield_capacity
            } else {
                resources::capacity(
                    old,
                    yield_capacity,
                    loss,
                    self.config.resource_adjustment,
                    self.config.damage_feedback == DamageFeedback::AddLosses,
                )
            };
            let mut new = computed;
            let mut clipping = 0.0;
            if new.is_finite()
                && self.config.numerical_policy == NumericalPolicy::FloorZero
                && new < 0.0
            {
                clipping = -new;
                new = 0.0;
                self.ledger.clipping += clipping;
            }
            let finite = |v: f64| v.is_finite().then_some(v);
            let residual = new - computed - clipping;
            self.resource_updates.push(ResourceUpdate {
                state: id,
                period: self.period,
                old_capacity: finite(old),
                extracted_yield: finite(yield_capacity),
                applied_damage: finite(loss),
                target_capacity: finite(target),
                new_capacity: finite(new),
                clipping,
                residual: finite(residual),
                reset,
            });
            s.previous_damage = 0.0;
            s.newly_independent = false;
            s.extracted_yield = yield_capacity;
            if !new.is_finite() {
                s.capacity = None;
                return Err(format!(
                    "period {} state {:?}: nonfinite capacity recurrence",
                    self.period, id
                ));
            }
            s.capacity = Some(new);
            s.recurrence_residual = residual;
            self.ledger.recurrence_residual += residual;
            if new > old {
                self.ledger.capacity_increase += new - old;
            } else {
                self.ledger.capacity_decrease += old - new;
            }
        }
        Ok(())
    }
    pub(crate) fn log_event(&mut self, kind: &str, states: Vec<StateId>, cells: Vec<usize>) {
        let event = Event {
            id: self.next_event,
            period: self.period,
            kind: kind.into(),
            states,
            cells,
        };
        self.next_event += 1;
        if kind == "conquest" {
            self.last_structural_event = Some(event.clone());
        }
        if self.config.event_log {
            if self.events.len() < self.config.event_log_limit {
                self.events.push(event);
            } else {
                self.events_dropped += 1;
            }
        }
    }
    pub(crate) fn step(&mut self) -> Result<(), String> {
        self.pending_fights.clear();
        self.update_resources()?;
        self.allocate()?;
        self.decide()?;
        let (claims, edges) = self.fight()?;
        self.apply_claims(claims)?;
        if self.period >= self.counting_start() {
            let sov = self.states.keys().copied().collect();
            let adjacent: Vec<_> = self.fronts.keys().map(|key| (key[0], key[1])).collect();
            self.tracker
                .advance(self.period, &edges, &adjacent, &sov, &self.config);
            self.tracked_period = self.period;
        }
        self.prior_fighting = edges.iter().flat_map(|&(a, b, _)| [a, b]).collect();
        for f in self.fronts.values_mut() {
            f.previous = f.actions;
            f.old_commitments = f.commitments;
        }
        self.shocks()?;
        if !self.ledger.damage.is_finite()
            || !self.ledger.measured_damage.is_finite()
            || !self.ledger.clipping.is_finite()
            || self
                .tracker
                .active
                .values()
                .any(|w| !w.raw_severity.is_finite())
        {
            return Err("nonfinite cumulative science ledger".into());
        }
        Ok(())
    }
    pub(crate) fn shocks(&mut self) -> Result<(), String> {
        if self.period <= self.config.initialization_periods {
            return Ok(());
        }
        let threshold = self.config.distance_threshold
            + (self.period - self.config.initialization_periods) as f64 * self.config.shock_shift
                / self.config.observation_periods as f64;
        if !threshold.is_finite() || threshold <= 0.0 {
            return Err(format!(
                "period {}: nonfinite or nonpositive technology frontier",
                self.period
            ));
        }
        for s in self.states.values_mut() {
            if chance(self.config.shock_probability, &mut self.rng) {
                s.threshold = threshold;
                self.cells[s.id.capital_cell].last_threshold = threshold;
                self.ledger.shocks += 1;
            }
        }
        Ok(())
    }
    fn finish(&mut self, valid: bool, reason: &str, invalid_reason: Option<String>) {
        if !valid
            && self.period >= self.counting_start()
            && self.tracked_period < self.period
            && !self.pending_fights.is_empty()
        {
            let sov = self.states.keys().copied().collect();
            self.tracker
                .advance(self.period, &self.pending_fights, &[], &sov, &self.config);
            self.tracked_period = self.period;
        }
        self.result = Some(Outcome {
            config: self.config.clone(),
            seed: self.seed,
            rng_mode: "portable_pcg64_mcg".into(),
            periods: self.completed,
            attempted_period: self.period,
            counting_start: self.counting_start(),
            valid,
            state_available: true,
            finish_reason: reason.into(),
            invalid_reason,
            completed_wars: self.tracker.completed.clone(),
            censored_wars: self.tracker.censored(self.period),
            legacy_visible_wars: self.tracker.legacy_visible.clone(),
            exporter_backlog: self.tracker.backlog.iter().cloned().collect(),
            merges: self.tracker.merges.clone(),
            retired_states: self.retired.clone(),
            sovereign_count: self.states.len(),
            states: self.states.values().cloned().collect(),
            cells: self.cells.clone(),
            ledger: self.ledger.clone(),
            fronts: self.fronts.values().cloned().collect(),
            resource_updates: self.resource_updates.clone(),
            partial_period_fights: if valid {
                Vec::new()
            } else {
                self.pending_fights.clone()
            },
        });
    }
    pub fn trace_json(&self) -> String {
        serde_json::json!({"config":self.config,"seed":self.seed,"tick":self.tick,"periods":self.completed,"attempted_period":self.period,"cells":self.cells,"states":self.states.values().collect::<Vec<_>>(),"fronts":self.fronts.values().collect::<Vec<_>>(),"resource_updates":self.resource_updates,"wars":self.tracker,"prior_fighting":self.prior_fighting,"retired_states":self.retired,"ledger":self.ledger}).to_string()
    }
    pub fn completed_periods(&self) -> u64 {
        self.completed
    }
    pub fn outcome(&self) -> Option<&Outcome> {
        self.result.as_ref()
    }
    pub fn invalidate_after_panic(&mut self, message: String) {
        if self.result.is_none() {
            self.finish(false, "implementation_panic", Some(message));
        }
    }
    pub fn economic_fingerprint(&self) -> u64 {
        let mut r = self.rng.clone();
        let mut c = self.config.clone();
        c.periods_per_tick = 1;
        hash(
            &serde_json::to_vec(&(
                c,
                self.period,
                self.completed,
                &self.cells,
                self.states.values().collect::<Vec<_>>(),
                self.fronts.values().collect::<Vec<_>>(),
                &self.tracker,
                &self.prior_fighting,
                &self.retired,
                &self.ledger,
                r.next_u64(),
            ))
            .unwrap(),
        )
    }
    pub(crate) fn record(&mut self) {
        let total = self
            .states
            .values()
            .try_fold(0.0, |sum, s| s.capacity.map(|r| sum + r));
        self.stats.push(GeosimSnapshot {
            tick: self.tick,
            period: self.completed,
            periods: self.completed,
            attempted_period: self.period,
            last_tick_periods: self.last_tick_periods,
            sovereign_count: self.states.len(),
            total_capacity: total.filter(|x| x.is_finite()),
            largest_territory: self.members.values().map(Vec::len).max().unwrap_or(0),
            alerted_states: self.states.values().filter(|s| s.alert).count(),
            mean_threshold: (!self.states.is_empty()).then(|| {
                let sum = self
                    .states
                    .values()
                    .map(|state| state.threshold)
                    .sum::<f64>();
                if sum.is_finite() {
                    return sum / self.states.len() as f64;
                }
                // Positive finite thresholds have a finite incremental mean.
                // Use it only on overflow, retaining ordinary/subnormal rounding.
                self.states
                    .values()
                    .enumerate()
                    .fold(0.0, |mean, (i, state)| {
                        mean + (state.threshold - mean) / (i + 1) as f64
                    })
            }),
            completed_wars: self.tracker.completed.len(),
            active_wars: self.tracker.active.len(),
            collector_backlog: self.tracker.backlog.len(),
            damage: self.ledger.damage,
            conquests: self.ledger.conquests,
            shocks: self.ledger.shocks,
            finish_reason: self.result.as_ref().map(|o| o.finish_reason.clone()),
            invalidity: self.result.as_ref().and_then(|o| o.invalid_reason.clone()),
        });
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn tiny() -> GeosimConfig {
        GeosimConfig {
            width: 3,
            height: 3,
            initial_states: 1,
            initialization_periods: 0,
            observation_periods: 13,
            periods_per_tick: 5,
            ..Default::default()
        }
    }
    #[test]
    fn founder_growth_covers_grid_with_connected_governments() {
        let w = GeosimWorld::new(tiny(), 9).unwrap();
        assert_eq!(w.cells.len(), 9);
        assert_eq!(w.states.len(), 1);
        assert!(w.cells.iter().all(|c| c.owner == w.cells[0].owner));
    }
    #[test]
    fn source_horizon_partial_tick_and_no_front_are_immutable() {
        let mut w = GeosimWorld::new(tiny(), 9).unwrap();
        w.run(100);
        assert_eq!(w.completed_periods(), 13);
        assert_eq!(w.tick, 3);
        assert_eq!(w.last_tick_periods, 3);
        assert!(w.outcome().unwrap().valid);
        assert_eq!(w.outcome().unwrap().ledger.attacks, 0);
        let f = w.economic_fingerprint();
        w.run(2);
        assert_eq!(w.economic_fingerprint(), f);
        assert_eq!(w.tick, 3);
    }
    #[test]
    fn grouped_periods_match_full_single_periods() {
        let mut c = tiny();
        c.initial_states = 4;
        c.observation_periods = 20;
        let mut a = GeosimWorld::new(c.clone(), 42).unwrap();
        c.periods_per_tick = 1;
        let mut b = GeosimWorld::new(c, 42).unwrap();
        a.run(4);
        b.run(20);
        assert_eq!(a.completed_periods(), 20);
        assert_eq!(a.economic_fingerprint(), b.economic_fingerprint());
    }
}
#[cfg(test)]
mod boundary_tests {
    use super::super::tests::prescribed;
    use super::*;
    #[test]
    fn invalid_partial_period_keeps_completed_clock_and_null_capacity() {
        let mut w = prescribed(&[0, 0, 0, 0], 2);
        w.config.resource_adjustment = 1.0;
        w.config.damage_feedback = DamageFeedback::AddLosses;
        let id = w.cells[0].owner;
        let state = w.states.get_mut(&id).unwrap();
        state.previous_damage = f64::INFINITY;
        state.capacity = Some(f64::MAX);
        w.config.observation_periods = 1;
        w.run(1);
        assert!(!w.outcome().unwrap().valid);
        assert_eq!(w.completed_periods(), 0);
        assert_eq!(w.outcome().unwrap().attempted_period, 1);
        let json = serde_json::to_value(w.outcome().unwrap()).unwrap();
        assert_eq!(json["states"][0]["capacity"], serde_json::Value::Null);
        assert!(!json.to_string().contains("NaN"));
    }
    #[test]
    fn one_government_floor_zero_finishes_without_ratio_draws() {
        let c = GeosimConfig {
            width: 2,
            height: 2,
            initial_states: 1,
            initialization_periods: 0,
            observation_periods: 1,
            shock_probability: 0.0,
            numerical_policy: NumericalPolicy::FloorZero,
            ..Default::default()
        };
        let mut w = GeosimWorld::new(c, 42).unwrap();
        let mut expected = w.rng.clone();
        w.run(1);
        assert!(w.outcome().unwrap().valid);
        assert_eq!(w.ledger.fighting_front_periods, 0);
        assert_eq!(w.rng.next_u64(), expected.next_u64());
    }
    #[test]
    fn technology_catchup_occurs_after_burnin_for_next_period() {
        let mut w = prescribed(&[0, 0, 0, 0], 2);
        w.config.initialization_periods = 2;
        w.config.observation_periods = 2;
        w.config.shock_probability = 1.0;
        w.run(2);
        assert_eq!(w.states.values().next().unwrap().threshold, 2.0);
        w.run(1);
        assert_eq!(w.states.values().next().unwrap().threshold, 12.0);
        w.run(1);
        assert_eq!(w.states.values().next().unwrap().threshold, 22.0);
    }
    #[test]
    fn maximum_grid_all_founders_has_linear_geometry_and_finite_exports() {
        let c = GeosimConfig {
            width: 100,
            height: 100,
            initial_states: 10000,
            initialization_periods: 0,
            observation_periods: 1,
            attack_probability: 0.0,
            shock_probability: 0.0,
            ..Default::default()
        };
        let mut w = GeosimWorld::new(c, 1).unwrap();
        w.run(1);
        assert_eq!(w.states.len(), 10000);
        assert!(w.outcome().unwrap().valid);
        assert!(serde_json::to_string(w.outcome().unwrap()).is_ok());
    }
}

#[cfg(test)]
mod counting_boundary_tests {
    use super::super::tests::prescribed;
    use super::*;
    #[test]
    fn burnin_fighting_refreshes_actions_but_not_census_until_named_boundary() {
        for (boundary, expected_start) in [
            (CountBoundary::AfterInitialization, 3),
            (CountBoundary::AtInitialization, 2),
        ] {
            let mut w = prescribed(&[0, 0, 2, 2], 2);
            w.config.initialization_periods = 2;
            w.config.observation_periods = 3;
            w.config.count_boundary = boundary;
            w.config.attack_probability = 1.0;
            w.config.superiority_threshold = 1e-30;
            w.config.victory_threshold = 1e-30;
            w.config.defender_threshold = DefenderThreshold::SameThreshold;
            w.run(1);
            assert!(w.tracker.active.is_empty());
            assert!(w.ledger.damage > 0.0);
            w.run(2);
            assert_eq!(
                w.tracker.active.values().next().unwrap().start_period,
                expected_start
            );
        }
    }
}

#[cfg(test)]
mod finite_technology_tests {
    use super::*;
    use crate::model::Model;
    use rand::RngCore;
    fn huge(threshold: f64, shift: f64, observation: u64, ppt: u32) -> GeosimConfig {
        GeosimConfig {
            width: 2,
            height: 2,
            initial_states: 1,
            initialization_periods: 0,
            observation_periods: observation,
            periods_per_tick: ppt,
            distance_threshold: threshold,
            shock_shift: shift,
            shock_probability: 1.0,
            attack_probability: 0.0,
            ..Default::default()
        }
    }
    #[test]
    fn nonfinite_frontier_rejects_endpoint_before_shock_draws_or_assignment() {
        let mut w = GeosimWorld::new(huge(1e308, 1e308, 1, 1), 2).unwrap();
        let expected_rng = w.rng.clone().next_u64();
        w.run(1);
        let out = w.outcome().unwrap();
        assert!(!out.valid);
        assert_eq!((out.periods, out.attempted_period), (0, 1));
        assert!(out
            .invalid_reason
            .as_ref()
            .unwrap()
            .contains("technology frontier"));
        assert_eq!(out.ledger.shocks, 0);
        assert!(out.states.iter().all(|s| s.threshold == 1e308));
        assert_eq!(w.rng.clone().next_u64(), expected_rng);
        let json: serde_json::Value = serde_json::from_str(&w.latest_json()).unwrap();
        assert_eq!(json["mean_threshold"], serde_json::json!(1e308));
    }
    #[test]
    fn intermediate_frontier_product_overflow_is_invalid_and_grouping_is_immutable() {
        let mut worlds = Vec::new();
        for ppt in [1, 5] {
            let mut w = GeosimWorld::new(huge(1.0, 1e308, 2, ppt), 2).unwrap();
            w.run(5);
            let out = w.outcome().unwrap();
            assert!(!out.valid);
            assert_eq!((out.periods, out.attempted_period), (1, 2));
            assert_eq!(out.ledger.shocks, 1);
            assert!(out.states.iter().all(|s| s.threshold.is_finite()));
            let before = w.economic_fingerprint();
            let tick = w.tick;
            w.run(5);
            assert_eq!(w.economic_fingerprint(), before);
            assert_eq!(w.tick, tick);
            worlds.push(w);
        }
        assert_eq!(
            worlds[0].economic_fingerprint(),
            worlds[1].economic_fingerprint()
        );
    }
    #[test]
    fn finite_large_multi_state_threshold_mean_does_not_overflow() {
        let mut c = huge(1e308, 0.0, 2, 1);
        c.initial_states = 4;
        c.shock_probability = 0.0;
        let mut w = GeosimWorld::new(c, 2).unwrap();
        let initial: serde_json::Value = serde_json::from_str(&w.latest_json()).unwrap();
        assert_eq!(initial["mean_threshold"], serde_json::json!(1e308));
        w.run(2);
        assert!(w.outcome().unwrap().valid);
        let terminal: serde_json::Value = serde_json::from_str(&w.latest_json()).unwrap();
        assert_eq!(terminal["mean_threshold"], serde_json::json!(1e308));
    }
    #[test]
    fn equal_subnormal_thresholds_retain_positive_mean() {
        let tiny = f64::from_bits(1);
        let mut c = huge(tiny, 0.0, 1, 1);
        c.initial_states = 4;
        c.shock_probability = 0.0;
        let w = GeosimWorld::new(c, 2).unwrap();
        let initial: serde_json::Value = serde_json::from_str(&w.latest_json()).unwrap();
        assert_eq!(
            initial["mean_threshold"].as_f64().unwrap().to_bits(),
            tiny.to_bits()
        );
    }
}

#[cfg(test)]
mod subnormal_mean_tests {
    use super::*;
    use crate::model::Model;
    #[test]
    fn mixed_subnormal_mean_keeps_finite_sum_rounding() {
        let mut w = GeosimWorld::new(
            GeosimConfig {
                width: 2,
                height: 2,
                initial_states: 2,
                distance_threshold: f64::from_bits(1),
                ..Default::default()
            },
            2,
        )
        .unwrap();
        for (i, state) in w.states.values_mut().enumerate() {
            state.threshold = f64::from_bits(i as u64 + 1);
        }
        w.record();
        let latest: serde_json::Value = serde_json::from_str(&w.latest_json()).unwrap();
        assert_eq!(latest["mean_threshold"].as_f64().unwrap().to_bits(), 2);
    }
}
