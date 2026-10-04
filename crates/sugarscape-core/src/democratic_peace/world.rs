//! Atomic source periods with a private candidate state and RNG stream.
use super::*;
use crate::{
    config::FieldError,
    rng::{self, SimRng},
    stats::Stats,
};
use rand::{Rng, RngCore};
use std::collections::BTreeMap;
#[derive(Clone)]
pub(crate) struct Engine {
    pub width: u32,
    pub phase: &'static str,
    pub period: u64,
    pub cells: Vec<Cell>,
    pub states: BTreeMap<StateId, State>,
    pub fronts: BTreeMap<[StateId; 2], Front>,
    pub alliances: Vec<Alliance>,
    pub pariahs: BTreeMap<StateId, Vec<StateId>>,
    pub rng: SimRng,
    pub counters: Counters,
    pub first_extinction: Option<u64>,
    pub first_all_democratic: Option<u64>,
    pub events: Vec<Event>,
    pub events_dropped: u64,
    pub last_structural_event: Option<Event>,
    pub extraction: Vec<Extraction>,
    pub next_alliance: u64,
}
#[derive(Clone)]
pub struct DemocraticPeaceWorld {
    pub config: DemocraticPeaceConfig,
    pub tick: u64,
    pub stats: Stats<DemocraticPeaceSnapshot>,
    pub(crate) engine: Engine,
    pub(crate) setup: SetupCensus,
    pub(crate) seed: u64,
    pub(crate) result: Option<Outcome>,
    pub(crate) attempted: u64,
    pub(crate) last_tick_periods: u32,
}
pub(crate) fn draw(rng: &mut SimRng) -> f64 {
    rng.gen::<f64>()
}
pub(crate) fn chance(p: f64, rng: &mut SimRng) -> bool {
    draw(rng) < p
}
pub(crate) fn pick<T: Copy>(v: &[T], rng: &mut SimRng) -> Option<T> {
    match v.len() {
        0 => None,
        1 => {
            rng.next_u32();
            Some(v[0])
        }
        n => Some(v[rng.gen_range(0..n as u32) as usize]),
    }
}
pub(crate) fn shuffle<T>(v: &mut [T], rng: &mut SimRng) {
    for i in (1..v.len()).rev() {
        let j = rng.gen_range(0..=i as u32) as usize;
        v.swap(i, j);
    }
}
pub(crate) fn hash(bytes: &[u8]) -> u64 {
    bytes.iter().fold(14695981039346656037u64, |h, b| {
        (h ^ u64::from(*b)).wrapping_mul(1099511628211)
    })
}
impl DemocraticPeaceWorld {
    pub fn new(config: DemocraticPeaceConfig, seed: u64) -> Result<Self, Vec<FieldError>> {
        config.validate()?;
        let n = (config.width * config.height) as usize;
        let mut rng = rng::seeded(seed);
        let mut assign = |share: f64| -> Vec<bool> {
            if config.assignment == Assignment::IndependentBernoulli {
                (0..n).map(|_| chance(share, &mut rng)).collect()
            } else {
                let mut order: Vec<_> = (0..n).collect();
                shuffle(&mut order, &mut rng);
                let count = libm::floor(share * n as f64 + 0.5) as usize;
                let mut flags = vec![false; n];
                for &id in &order[..count] {
                    flags[id] = true;
                }
                flags
            }
        };
        let regimes = assign(config.initial_democratic_share);
        let resources = assign(config.initial_resourced_share);
        let setup = SetupCensus {
            initial_democratic_cells: regimes.iter().filter(|d| **d).count() as u32,
            initial_resourced_cells: resources.iter().filter(|d| **d).count() as u32,
            total_cells: n as u32,
        };
        let cells: Vec<_> = (0..n)
            .map(|id| {
                let owner = StateId {
                    capital_cell: id,
                    sovereignty_generation: 0,
                };
                let regime = if regimes[id] {
                    Regime::Democratic
                } else {
                    Regime::Predatory
                };
                Cell {
                    id,
                    owner,
                    initial_regime: regime,
                    latent_regime: regime,
                    next_generation: 0,
                }
            })
            .collect();
        let states = cells
            .iter()
            .map(|cell| {
                (
                    cell.owner,
                    State {
                        id: cell.owner,
                        regime: cell.latent_regime,
                        resources: if resources[cell.id] { 10. } else { 0. },
                        members: vec![cell.id],
                    },
                )
            })
            .collect();
        let mut engine = Engine {
            width: config.width,
            phase: "setup",
            period: 0,
            cells,
            states,
            fronts: BTreeMap::new(),
            alliances: vec![],
            pariahs: BTreeMap::new(),
            rng,
            counters: Counters::default(),
            first_extinction: None,
            first_all_democratic: None,
            events: vec![],
            events_dropped: 0,
            last_structural_event: None,
            extraction: vec![],
            next_alliance: 0,
        };
        engine
            .rebuild()
            .map_err(|e| vec![FieldError::new("setup", e)])?;
        engine
            .census(&config)
            .map_err(|e| vec![FieldError::new("setup", e)])?;
        let mut w = Self {
            config,
            tick: 0,
            stats: Stats::default(),
            engine,
            setup,
            seed,
            result: None,
            attempted: 0,
            last_tick_periods: 0,
        };
        w.stats.push(w.snapshot());
        Ok(w)
    }
    pub fn run(&mut self, ticks: u32) {
        for _ in 0..ticks {
            if self.result.is_some() {
                break;
            }
            self.last_tick_periods = 0;
            for _ in 0..self.config.periods_per_tick {
                if self.engine.period >= self.config.horizon() {
                    break;
                }
                self.attempted = self.engine.period + 1;
                let mut next = self.engine.clone();
                next.period = self.attempted;
                let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    next.period(&self.config)
                }));
                let r = match r {
                    Ok(result) => result,
                    Err(payload) => {
                        let message = payload
                            .downcast_ref::<String>()
                            .cloned()
                            .or_else(|| payload.downcast_ref::<&str>().map(|s| s.to_string()))
                            .unwrap_or_else(|| "non-string panic payload".into());
                        Err((
                            next.phase.to_string(),
                            format!("internal engine panic: {message}"),
                        ))
                    }
                };
                match r {
                    Ok(()) => {
                        self.engine = next;
                        self.last_tick_periods += 1;
                    }
                    Err((phase, reason)) => {
                        self.result = Some(Outcome {
                            valid: false,
                            finish_reason: "invalid".into(),
                            invalid_reason: Some(reason),
                            invalid_phase: Some(phase),
                            attempted_period: self.attempted,
                            completed_periods: self.engine.period,
                            final_metrics: None,
                            census: self.engine.counters.clone(),
                        });
                        break;
                    }
                }
            }
            self.tick += 1;
            if self.result.is_none() && self.engine.period == self.config.horizon() {
                self.result = Some(Outcome {
                    valid: true,
                    finish_reason: "complete".into(),
                    invalid_reason: None,
                    invalid_phase: None,
                    attempted_period: self.attempted,
                    completed_periods: self.engine.period,
                    final_metrics: Some(
                        self.engine.metrics(&self.config).expect("validated census"),
                    ),
                    census: self.engine.counters.clone(),
                });
            }
            self.stats.push(self.snapshot());
        }
    }
    pub fn completed_periods(&self) -> u64 {
        self.engine.period
    }
    pub fn outcome(&self) -> Option<&Outcome> {
        self.result.as_ref()
    }
    pub fn snapshot(&self) -> DemocraticPeaceSnapshot {
        let m = self
            .engine
            .metrics(&self.config)
            .expect("last committed valid census");
        DemocraticPeaceSnapshot {
            tick: self.tick,
            period: self.engine.period,
            periods: self.engine.period,
            completed_periods: self.engine.period,
            attempted_period: self.attempted,
            last_tick_periods: self.last_tick_periods,
            democratic_share: m.democratic_share,
            clustering_ratio: m.clustering_ratio,
            clustering_reason: m.clustering_reason.clone(),
            sovereign_count: m.sovereign_count,
            democratic_states: m.democratic_states,
            predatory_states: m.predatory_states,
            conflict_fronts: m.conflict_fronts,
            alliance_count: m.alliance_count,
            pariah_count: m.pariah_count,
            finish_reason: self.result.as_ref().map(|o| o.finish_reason.clone()),
            invalidity: self.result.as_ref().and_then(|o| o.invalid_reason.clone()),
            invalid_phase: self.result.as_ref().and_then(|o| o.invalid_phase.clone()),
            metrics: m,
        }
    }
    pub fn economic_fingerprint(&self) -> u64 {
        let mut c = self.config.clone();
        c.periods_per_tick = 1;
        c.event_recording = false;
        c.event_limit = 1000;
        let mut r = self.engine.rng.clone();
        let cursor = [r.next_u64(), r.next_u64(), r.next_u64(), r.next_u64()];
        hash(&serde_json::to_vec(&(c, self.seed, self.engine.canonical(), cursor)).unwrap())
    }
    pub fn science_json(&self) -> String {
        serde_json::json!({"model":"democratic_peace","config":self.config,"seed":self.seed,"period":self.engine.period,"periods":self.engine.period,"attempted_period":self.attempted,"completed_periods":self.engine.period,"tick":self.tick,"last_tick_periods":self.last_tick_periods,"setup":self.setup,"current_metrics":self.snapshot().metrics,"census":self.engine.counters,"outcome":self.result,"final_state_hash":format!("{:016x}",self.economic_fingerprint())}).to_string()
    }
    pub fn state_json(&self) -> String {
        let mut r = self.engine.rng.clone();
        serde_json::json!({"model":"democratic_peace","config":self.config,"seed":self.seed,"tick":self.tick,"attempted_period":self.attempted,"completed_periods":self.engine.period,"period":self.engine.period,"periods":self.engine.period,"last_tick_periods":self.last_tick_periods,"horizon_periods":self.config.horizon_periods,"current_metrics":self.snapshot().metrics,"census":self.engine.counters,"final_state_hash":format!("{:016x}",self.economic_fingerprint()),"state":self.engine.canonical(),"rng_cursor":[r.next_u64(),r.next_u64(),r.next_u64(),r.next_u64()],"setup":self.setup,"outcome":self.result,"events":self.engine.events,"events_dropped":self.engine.events_dropped}).to_string()
    }
    pub fn cells(&self) -> &[Cell] {
        &self.engine.cells
    }
    pub fn states(&self) -> &BTreeMap<StateId, State> {
        &self.engine.states
    }
    pub fn fronts(&self) -> &BTreeMap<[StateId; 2], Front> {
        &self.engine.fronts
    }
    pub fn alliances(&self) -> &[Alliance] {
        &self.engine.alliances
    }
    pub fn counters(&self) -> &Counters {
        &self.engine.counters
    }
}
impl Engine {
    fn period(&mut self, c: &DemocraticPeaceConfig) -> Result<(), (String, String)> {
        self.phase = "allocation";
        self.allocate(c).map_err(|e| ("allocation".into(), e))?;
        self.phase = "alignments";
        self.align(c).map_err(|e| ("alignments".into(), e))?;
        self.phase = "decisions";
        self.decide(c).map_err(|e| ("decisions".into(), e))?;
        self.phase = "interaction";
        let claims = self.combat(c).map_err(|e| ("interaction".into(), e))?;
        self.phase = "resource_update";
        self.extract(c).map_err(|e| ("resource_update".into(), e))?;
        self.phase = "structural_change";
        self.apply_claims(c, claims)
            .map_err(|e| ("structural_change".into(), e))?;
        for f in self.fronts.values_mut() {
            f.previous = f.actions;
            f.old_commitments = f.commitments;
            f.previous_initiations = f.initiations;
        }
        self.phase = "census";
        self.census(c).map_err(|e| ("census".into(), e))?;
        Ok(())
    }
    pub(crate) fn record(&mut self, c: &DemocraticPeaceConfig, e: Event) -> Result<(), String> {
        if c.event_recording {
            if self.events.len() < c.event_limit as usize {
                self.events.push(e);
            } else {
                types::bump(&mut self.events_dropped)?;
            }
        }
        Ok(())
    }
    pub(crate) fn canonical(&self) -> serde_json::Value {
        serde_json::json!({"period":self.period,"cells":self.cells,"states":self.states.values().collect::<Vec<_>>(),"fronts":self.fronts.values().collect::<Vec<_>>(),"alliances":self.alliances,"pariahs":self.pariahs.iter().map(|(id,sources)|serde_json::json!({"state":id,"sources":sources})).collect::<Vec<_>>(),"census":self.counters,"first_extinction_period":self.first_extinction,"first_all_democratic_period":self.first_all_democratic,"last_structural_event":self.last_structural_event,"extraction":self.extraction,"next_alliance":self.next_alliance})
    }
    pub(crate) fn extract(&mut self, c: &DemocraticPeaceConfig) -> Result<(), String> {
        let mut updates = Vec::new();
        for (&id, s) in &self.states {
            let mut province_terms = Vec::new();
            let mut resources = 10.;
            for &cell in &s.members {
                if cell == id.capital_cell {
                    continue;
                }
                let d = self.distance(c, id, cell)?;
                let term = 10. * c.tax_rate * libm::pow(c.distance_gradient, d);
                resources += term;
                province_terms.push((cell, d, term));
            }
            if !resources.is_finite() {
                return Err("nonfinite extracted resources".into());
            }
            updates.push(Extraction {
                state: id,
                resources_before: s.resources,
                resources_after: resources,
                province_terms,
            });
        }
        for r in &updates {
            self.states.get_mut(&r.state).unwrap().resources = r.resources_after;
        }
        self.extraction = updates;
        Ok(())
    }
    pub(crate) fn census(&mut self, c: &DemocraticPeaceConfig) -> Result<(), String> {
        self.validate()?;
        let m = self.metrics(c)?;
        if m.democratic_cells == 0 && self.first_extinction.is_none() {
            self.first_extinction = Some(self.period);
        }
        if m.democratic_cells == m.total_cells && self.first_all_democratic.is_none() {
            self.first_all_democratic = Some(self.period);
        }
        Ok(())
    }
    pub(crate) fn metrics(&self, c: &DemocraticPeaceConfig) -> Result<Metrics, String> {
        let d: Vec<_> = self
            .states
            .values()
            .filter(|s| s.regime == Regime::Democratic)
            .collect();
        let p: Vec<_> = self
            .states
            .values()
            .filter(|s| s.regime == Regime::Predatory)
            .collect();
        let dc = d.iter().map(|s| s.members.len()).sum::<usize>() as u32;
        let pc = p.iter().map(|s| s.members.len()).sum::<usize>() as u32;
        let total = self.cells.len() as u32;
        let mut exposure = 0.;
        let mut weights = 0.;
        for s in &d {
            let (neighbors, democratic) =
                if c.clustering_exposure == ClusteringExposure::UniqueStateNeighbors {
                    let n = self.neighbors(s.id);
                    let yes = n
                        .iter()
                        .filter(|id| self.states[id].regime == Regime::Democratic)
                        .count();
                    (n.len(), yes)
                } else {
                    let mut all = 0;
                    let mut yes = 0;
                    for &cell in &s.members {
                        for q in self.adjacent(cell) {
                            let owner = self.cells[q].owner;
                            if owner != s.id {
                                all += 1;
                                if self.states[&owner].regime == Regime::Democratic {
                                    yes += 1;
                                }
                            }
                        }
                    }
                    (all, yes)
                };
            let fraction = if neighbors == 0 {
                1.
            } else {
                democratic as f64 / neighbors as f64
            };
            let weight = if c.clustering_weights == ClusteringWeights::TerritoryWeighted {
                s.members.len() as f64
            } else {
                1.
            };
            exposure += weight * fraction;
            weights += weight;
        }
        let e = (!d.is_empty()).then(|| exposure / weights);
        let (reason, ratio) = if c.initial_democratic_share == 0. {
            (Some("undefined_initial_density".into()), None)
        } else if d.is_empty() {
            (Some("undefined_extinction".into()), None)
        } else {
            (None, Some(e.unwrap() / c.initial_democratic_share))
        };
        if [e, ratio].iter().flatten().any(|v| !v.is_finite()) {
            return Err("nonfinite clustering metric".into());
        }
        Ok(Metrics {
            democratic_cells: dc,
            total_cells: total,
            democratic_share: dc as f64 / total as f64,
            sovereign_count: self.states.len() as u32,
            democratic_states: d.len() as u32,
            predatory_states: p.len() as u32,
            democratic_mean_size: (!d.is_empty()).then(|| dc as f64 / d.len() as f64),
            predatory_mean_size: (!p.is_empty()).then(|| pc as f64 / p.len() as f64),
            democratic_max_size: d.iter().map(|s| s.members.len() as u32).max(),
            predatory_max_size: p.iter().map(|s| s.members.len() as u32).max(),
            democratic_size_reason: d.is_empty().then(|| "no_surviving_states".into()),
            predatory_size_reason: p.is_empty().then(|| "no_surviving_states".into()),
            democratic_exposure: e,
            clustering_ratio: ratio,
            clustering_reason: reason,
            conflict_fronts: self
                .fronts
                .values()
                .filter(|f| f.actions.iter().any(|a| *a))
                .count() as u32,
            alliance_count: self.alliances.len() as u32,
            pariah_count: self.pariahs.len() as u32,
            democratic_extinction: dc == 0,
            all_democratic: dc == total,
            first_extinction_period: self.first_extinction,
            first_all_democratic_period: self.first_all_democratic,
        })
    }
}
