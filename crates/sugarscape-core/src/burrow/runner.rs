//! Sequential seeded opportunities and deterministic replay diagnostics.

use super::{
    controller::decide_measured,
    ledger::Ledger,
    observation::{exit_distances_measured, observe_measured, BfsStats},
    view, Action, ActionEvent, Delivery, ExitNeighbor, Fixture, Inventory, LabConfig, Outcome, Pos,
    Setup, WorkerView, World,
};
use crate::config::FieldError;
use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

const MAX_OPPORTUNITIES: u64 = 1_000_000;
const MAX_ASCII_BYTES: u64 = 64 * 1024 * 1024;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunOptions {
    pub ticks: u32,
    pub sample_every: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Snapshot {
    pub tick: u64,
    pub opportunities: u64,
    pub inventory: Inventory,
    pub successful_moves: u64,
    pub digs: u64,
    pub pickups: u64,
    pub drops: u64,
    pub disposals: u64,
    pub blocked: u64,
    pub waits: u64,
    pub exit_bfs_calls: u64,
    pub exit_bfs_visits: u64,
    pub exit_bfs_peak_queue: u64,
    pub observation_bfs_calls: u64,
    pub observation_bfs_visits: u64,
    pub observation_bfs_peak_queue: u64,
    pub controller_bfs_calls: u64,
    pub controller_bfs_visits: u64,
    pub controller_bfs_peak_queue: u64,
    pub connected_open: u64,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Frame {
    pub tick: u64,
    pub fingerprint: String,
    pub ascii: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChoiceEvent {
    pub tick: u64,
    pub worker: u32,
    pub target: Pos,
}
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkerWork {
    pub worker: u32,
    pub moves: u64,
    pub digs: u64,
    pub pickups: u64,
    pub drops: u64,
    pub disposed: u64,
    pub blocked: u64,
    pub waits: u64,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpatialWork {
    pub pos: Pos,
    pub digs: u64,
    pub workers: Vec<u32>,
}
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct LoadedTravel {
    pub loaded: u64,
    pub unloaded: u64,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DigDistance {
    pub tick: u64,
    pub worker: u32,
    pub pos: Pos,
    pub distance: u32,
}
/// Rates derive exclusively from authoritative integer opportunity/action totals.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Rates {
    pub excavation: Option<f64>,
    pub disposal: Option<f64>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Labels {
    pub sources: Vec<String>,
    pub assumptions: Vec<String>,
}
/// Logical records/cells, independent of allocator capacity and machine word size.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Storage {
    pub events: u64,
    pub choices: u64,
    pub frames: u64,
    pub snapshots: u64,
    pub deliveries: u64,
    pub carrier_ids: u64,
    pub worker_work: u64,
    pub spatial_work: u64,
    pub spatial_worker_ids: u64,
    pub dig_distances: u64,
    pub retained_records: u64,
    pub ascii_bytes: u64,
    pub peak_ascii_bytes: u64,
    pub peak_events: u64,
    pub peak_choices: u64,
    pub peak_material_records: u64,
    pub peak_exit_field_cells: u64,
    pub peak_observation_cells: u64,
    pub peak_observation_frontier: u64,
    pub peak_open_cells: u64,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Episode {
    pub config: LabConfig,
    pub setup: Setup,
    pub seed: String,
    pub requested_ticks: u32,
    pub completed_ticks: u32,
    pub stop_reason: String,
    pub events: Vec<ActionEvent>,
    pub choices: Vec<ChoiceEvent>,
    pub frames: Vec<Frame>,
    pub series: Vec<Snapshot>,
    pub deliveries: Vec<Delivery>,
    pub worker_work: Vec<WorkerWork>,
    pub final_summary: Snapshot,
    pub spatial_work: Vec<SpatialWork>,
    pub travel: LoadedTravel,
    pub dig_distances: Vec<DigDistance>,
    pub rates: Rates,
    pub labels: Labels,
    pub storage: Storage,
}

#[derive(Clone, Debug, Default)]
pub(super) struct Recording {
    pub(super) ledger: Ledger,
    pub(super) events: Vec<ActionEvent>,
    pub(super) choices: Vec<ChoiceEvent>,
    pub(super) worker_work: Vec<WorkerWork>,
    pub(super) spatial_work: BTreeMap<Pos, SpatialWork>,
    pub(super) travel: LoadedTravel,
    pub(super) dig_distances: Vec<DigDistance>,
    pub(super) exit_field: Vec<Option<u32>>,
    field_excavated: Option<u64>,
    exit_stats: BfsStats,
    pub(super) observation_stats: BfsStats,
    controller_stats: BfsStats,
    pub(super) peak_observation_cells: u64,
    pub(super) peak_observation_frontier: u64,
    pub(super) stopped: bool,
}
impl Recording {
    pub(super) fn new(world: &World) -> Self {
        Self {
            ledger: Ledger::new(world),
            worker_work: world
                .workers
                .iter()
                .map(|w| WorkerWork {
                    worker: w.id,
                    ..Default::default()
                })
                .collect(),
            ..Default::default()
        }
    }
    pub(super) fn ensure_exit_field(&mut self, world: &World) {
        if self.field_excavated != Some(world.excavated) {
            let (field, stats) = exit_distances_measured(world);
            self.exit_field = field;
            self.field_excavated = Some(world.excavated);
            self.exit_stats.include(stats);
        }
    }
    pub(super) fn record(&mut self, event: &ActionEvent, world: &World, distance: Option<u32>) {
        self.ledger.record(event, world);
        let work = &mut self.worker_work[event.worker as usize];
        if event.outcome != Outcome::Success {
            work.blocked += 1;
        } else {
            match event.action {
                Action::Move(_) => {
                    work.moves += 1;
                    if event.material.is_some() {
                        self.travel.loaded += 1;
                    } else {
                        self.travel.unloaded += 1;
                    }
                }
                Action::Dig(pos) => {
                    work.digs += 1;
                    let spatial = self.spatial_work.entry(pos).or_insert_with(|| SpatialWork {
                        pos,
                        digs: 0,
                        workers: Vec::new(),
                    });
                    spatial.digs += 1;
                    if let Err(index) = spatial.workers.binary_search(&event.worker) {
                        spatial.workers.insert(index, event.worker);
                    }
                    self.dig_distances.push(DigDistance {
                        tick: event.tick,
                        worker: event.worker,
                        pos,
                        distance: distance.expect("successful dig has an exit distance"),
                    });
                }
                Action::Pickup => work.pickups += 1,
                Action::Drop => work.drops += 1,
                Action::Dispose => work.disposed += 1,
                Action::Wait => work.waits += 1,
            }
        }
        self.events.push(event.clone());
    }
    pub(super) fn snapshot(&self, world: &World) -> Snapshot {
        let total = |f: fn(&WorkerWork) -> u64| self.worker_work.iter().map(f).sum();
        Snapshot {
            tick: world.tick,
            opportunities: self.events.len() as u64,
            inventory: world.inventory(),
            successful_moves: total(|w| w.moves),
            digs: total(|w| w.digs),
            pickups: total(|w| w.pickups),
            drops: total(|w| w.drops),
            disposals: total(|w| w.disposed),
            blocked: total(|w| w.blocked),
            waits: total(|w| w.waits),
            exit_bfs_calls: self.exit_stats.calls,
            exit_bfs_visits: self.exit_stats.visits,
            exit_bfs_peak_queue: self.exit_stats.peak_queue,
            observation_bfs_calls: self.observation_stats.calls,
            observation_bfs_visits: self.observation_stats.visits,
            observation_bfs_peak_queue: self.observation_stats.peak_queue,
            controller_bfs_calls: self.controller_stats.calls,
            controller_bfs_visits: self.controller_stats.visits,
            controller_bfs_peak_queue: self.controller_stats.peak_queue,
            connected_open: world.setup.open.len() as u64 + world.excavated,
        }
    }
}
impl World {
    pub fn step(&mut self) {
        if self.recording.stopped {
            return;
        }
        let next_tick = self.tick.checked_add(1).expect("burrow clock overflow");
        // Move diagnostics out while observing immutable authoritative state.
        let mut recording = std::mem::take(&mut self.recording);
        recording.ensure_exit_field(self);
        let mut order = self.worker_ids();
        order.shuffle(&mut self.rng);
        for id in order {
            self.worker_opportunity(id, &mut recording);
            if recording.stopped {
                break;
            }
        }
        if !recording.stopped {
            self.tick = next_tick;
        }
        self.recording = recording;
    }
    fn worker_ids(&self) -> Vec<u32> {
        self.workers.iter().map(|w| w.id).collect()
    }
    fn worker_opportunity(&mut self, id: u32, recording: &mut Recording) {
        recording.ensure_exit_field(self);
        let (observation, stats) = observe_measured(self, id);
        recording.observation_stats.include(stats);
        recording.peak_observation_cells = recording
            .peak_observation_cells
            .max(observation.open.len() as u64);
        recording.peak_observation_frontier = recording
            .peak_observation_frontier
            .max(observation.frontier.len() as u64);
        let worker = WorkerView::from(&self.workers[id as usize]);
        let distance = recording.exit_field[self.index(worker.pos).unwrap()].unwrap();
        let outward: Vec<_> = worker
            .pos
            .neighbors(self.setup.width, self.setup.height)
            .into_iter()
            .filter_map(|pos| {
                let next = recording.exit_field[self.index(pos).unwrap()]?;
                (next < distance).then(|| ExitNeighbor {
                    pos,
                    distance: next,
                    occupants: self.workers.iter().filter(|w| w.pos == pos).count() as u32,
                })
            })
            .collect();
        let (decision, stats) = decide_measured(
            &observation,
            &worker,
            &self.config,
            worker.pos == self.setup.exit,
            &outward,
            &mut self.rng,
        );
        recording.controller_stats.include(stats);
        self.workers[id as usize].target = decision.target;
        if let Some(target) = decision.selected {
            recording.choices.push(ChoiceEvent {
                tick: self.tick,
                worker: id,
                target,
            });
            if matches!(self.config.fixture, Fixture::Choice { .. }) {
                self.workers[id as usize].target = Some(target);
                recording.stopped = true;
                return;
            }
        }
        // The new face's shortest exit distance at excavation is one beyond
        // its closest open neighbor. Later shortcuts cannot rewrite this event.
        let dig_distance = if let Action::Dig(pos) = decision.action {
            pos.neighbors(self.setup.width, self.setup.height)
                .iter()
                .filter_map(|&p| recording.exit_field[self.index(p).unwrap()])
                .min()
                .map(|d| d + 1)
        } else {
            None
        };
        let event = self.apply(id, decision.action);
        recording.record(&event, self, dig_distance);
        recording.ensure_exit_field(self);
    }
    pub(super) fn snapshot(&self) -> Snapshot {
        self.recording.snapshot(self)
    }
}

/// Conservative requested frame bound: initial + cadence + final; zero ticks need only initial.
pub(super) fn requested_ascii_bytes(options: &RunOptions, frame_bytes: u64) -> Option<u64> {
    if options.sample_every == 0 {
        return None;
    }
    let frames = if options.ticks == 0 {
        1
    } else {
        2u64.checked_add(u64::from(options.ticks / options.sample_every))?
    };
    frames.checked_mul(frame_bytes)
}

pub(super) fn validate_options(world: &World, options: &RunOptions) -> Result<(), Vec<FieldError>> {
    let mut errors = Vec::new();
    if options.sample_every == 0 {
        errors.push(FieldError::new("sample_every", "must be positive"));
    }
    if world.tick.checked_add(u64::from(options.ticks)).is_none() {
        errors.push(FieldError::new("ticks", "requested clock advance overflow"));
    }
    if u64::from(options.ticks) * world.workers.len() as u64 > MAX_OPPORTUNITIES {
        errors.push(FieldError::new(
            "ticks",
            "replay supports at most 1000000 worker opportunities",
        ));
    }
    if options.sample_every > 0
        && requested_ascii_bytes(options, view::ascii_len(world))
            .is_none_or(|bytes| bytes > MAX_ASCII_BYTES)
    {
        errors.push(FieldError::new("sample_every", "conservative requested ASCII frames exceed 64 MiB; reduce ticks or increase sample_every"));
    }
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

pub fn run_episode(
    c: LabConfig,
    seed: u64,
    options: RunOptions,
) -> Result<Episode, Vec<FieldError>> {
    let mut world = World::new(c, seed)?;
    validate_options(&world, &options)?;
    // Initialization and all sampling are deterministic observers, never RNG consumers.
    let mut recording = std::mem::take(&mut world.recording);
    recording.ensure_exit_field(&world);
    world.recording = recording;
    let mut frames = vec![view::frame(&world)];
    let mut series = vec![world.snapshot()];
    let mut completed_ticks = 0;
    for _ in 0..options.ticks {
        world.step();
        if world.recording.stopped {
            break;
        }
        completed_ticks += 1;
        if completed_ticks % options.sample_every == 0 {
            frames.push(view::frame(&world));
            series.push(world.snapshot());
        }
    }
    let terminal = view::frame(&world);
    if frames.last() != Some(&terminal) {
        frames.push(terminal);
    }
    let final_summary = world.snapshot();
    if series.last() != Some(&final_summary) {
        series.push(final_summary.clone());
    }
    let deliveries = world.recording.ledger.finish(&world);
    let spatial_work: Vec<_> = world.recording.spatial_work.values().cloned().collect();
    let carrier_ids = deliveries.iter().map(|d| d.carriers.len() as u64).sum();
    let spatial_worker_ids = spatial_work.iter().map(|s| s.workers.len() as u64).sum();
    let mut storage = Storage {
        events: world.recording.events.len() as u64,
        choices: world.recording.choices.len() as u64,
        frames: frames.len() as u64,
        snapshots: series.len() as u64,
        deliveries: deliveries.len() as u64,
        carrier_ids,
        worker_work: world.workers.len() as u64,
        spatial_work: spatial_work.len() as u64,
        spatial_worker_ids,
        dig_distances: world.recording.dig_distances.len() as u64,
        retained_records: 0,
        ascii_bytes: frames.iter().map(|f| f.ascii.len() as u64).sum(),
        peak_ascii_bytes: frames.iter().map(|f| f.ascii.len() as u64).sum(),
        peak_events: world.recording.events.len() as u64,
        peak_choices: world.recording.choices.len() as u64,
        peak_material_records: world.units.len() as u64,
        peak_exit_field_cells: world.recording.exit_field.len() as u64,
        peak_observation_cells: world.recording.peak_observation_cells,
        peak_observation_frontier: world.recording.peak_observation_frontier,
        peak_open_cells: final_summary.connected_open,
    };
    storage.retained_records = storage.events
        + storage.choices
        + storage.frames
        + storage.snapshots
        + storage.deliveries
        + storage.carrier_ids
        + storage.worker_work
        + storage.spatial_work
        + storage.spatial_worker_ids
        + storage.dig_distances;
    let rates = Rates {
        excavation: (final_summary.opportunities > 0)
            .then(|| final_summary.digs as f64 / final_summary.opportunities as f64),
        disposal: (final_summary.opportunities > 0)
            .then(|| final_summary.disposals as f64 / final_summary.opportunities as f64),
    };
    let stop_reason = if world.recording.stopped {
        "choice_selected"
    } else {
        "tick_budget_exhausted"
    }
    .into();
    Ok(Episode {
        config: world.config,
        setup: world.setup,
        seed: seed.to_string(),
        requested_ticks: options.ticks,
        completed_ticks,
        stop_reason,
        events: world.recording.events,
        choices: world.recording.choices,
        frames,
        series,
        deliveries,
        worker_work: world.recording.worker_work,
        final_summary,
        spatial_work,
        travel: world.recording.travel,
        dig_distances: world.recording.dig_distances,
        rates,
        labels: view::labels(),
        storage,
    })
}
