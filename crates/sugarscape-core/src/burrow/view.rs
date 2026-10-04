//! Read-only maps, source labels and canonical state fingerprints.

use super::{state::UnitLocation, Cue, Fixture, Frame, Labels, Pile, Side, Transport, World};
use rand::RngCore;

const LEGEND: &str = "# solid; . open; E exit; o loose material; w worker; W loaded worker\nExit/worker overlays hide loose material; one glyph can represent two occupants. Trace and inventory retain hidden details.\n";

pub(super) fn ascii_len(world: &World) -> u64 {
    world.open.len() as u64 + u64::from(world.setup.height) + LEGEND.len() as u64
}

pub(super) fn ascii(world: &World) -> String {
    let mut cells: Vec<char> = world
        .open
        .iter()
        .map(|&open| if open { '.' } else { '#' })
        .collect();
    for unit in world.units.values() {
        if let UnitLocation::Loose(pos) = unit.location {
            cells[world.index(pos).unwrap()] = 'o';
        }
    }
    for worker in &world.workers {
        let index = world.index(worker.pos).unwrap();
        // A loaded occupant takes precedence over an unloaded co-occupant.
        if worker.carried.is_some() {
            cells[index] = 'W';
        } else if cells[index] != 'W' {
            cells[index] = 'w';
        }
    }
    cells[world.index(world.setup.exit).unwrap()] = 'E';
    let mut out = String::with_capacity(ascii_len(world) as usize);
    for row in cells.chunks(world.setup.width as usize) {
        out.extend(row);
        out.push('\n');
    }
    out.push_str(LEGEND);
    out
}
pub(super) fn frame(world: &World) -> Frame {
    Frame {
        tick: world.tick,
        fingerprint: format!("{:016x}", world.fingerprint()),
        ascii: ascii(world),
    }
}
pub(super) fn labels() -> Labels {
    Labels {
        sources: vec![
            "Initial open geometry, worker spawns and initial material are supplied fixture state.".into(),
            "All subsequent opened cells and material births are recorded dig events.".into(),
            "Local spoil response weights and transport rules are supplied controller preferences.".into(),
        ],
        assumptions: vec![
            "Finite bounded horizontal four-neighbor lattice; no wraparound, diagonals or gravity.".into(),
            "At most two workers per open cell and zero or one carried unit per worker.".into(),
            "The global exit-distance field is a supplied navigation scaffold; unloaded frontier decisions use local observations.".into(),
            "Initial material birth before fixture start is supplied history; observed loose waiting begins at fixture start.".into(),
            "Undisposed units are censored deliveries. Delivery age is disposal tick minus birth, or final tick minus birth when censored.".into(),
            "Spatial work and dig distances identify the excavated target cell; exit distance is recorded at opening, and ActionEvent.from preserves the worker standing position.".into(),
            "Connected open area follows the accessible-frontier invariant and is not evidence of coordination.".into(),
            "Storage sizes count logical retained records and peak cells; BFS work counts deterministic calls, visits and queue sizes. Wall-clock profiling is separate.".into(),
            "The one million worker-opportunity replay cap is an operational lab bound, not physical calibration.".into(),
            "Retained ASCII is capped at 64 MiB using a conservative requested frame count: one initial frame for zero ticks, otherwise initial + floor(ticks/sample_every) cadence frames + one final frame. Early-stop requests may be rejected before running; reduce ticks or increase sample_every.".into(),
            "Preference rules are supplied; replay verification does not establish biological validation, individual learning, chamber counts or comparative success.".into(),
        ],
    }
}
impl World {
    /// FNV-1a with explicit little-endian integer bytes, including RNG continuation.
    /// Diagnostics and search caches are excluded because they do not influence actions.
    pub fn fingerprint(&self) -> u64 {
        let mut h = 0xcbf2_9ce4_8422_2325u64;
        let mut eat = |value: u64| {
            for b in value.to_le_bytes() {
                h ^= u64::from(b);
                h = h.wrapping_mul(0x0100_0000_01b3);
            }
        };
        let pos = |p: super::Pos| (u64::from(p.x) << 32) | u64::from(p.y);
        eat(self.tick);
        eat(self.next_material);
        eat(self.excavated);
        eat(u64::from(self.recording.stopped));
        eat(match self.config.transport {
            Transport::Direct => 0,
            Transport::Relay => 1,
        });
        eat(match self.config.cue {
            Cue::Blind => 0,
            Cue::Responsive => 1,
        });
        eat(self.config.freshness_window);
        eat(u64::from(self.config.relay_distance));
        eat(u64::from(self.config.response_weight));
        eat(u64::from(self.config.minimum_recent_units));
        match self.config.fixture {
            Fixture::Growing {
                width,
                height,
                workers,
            } => {
                eat(0);
                eat(u64::from(width));
                eat(u64::from(height));
                eat(u64::from(workers));
            }
            Fixture::Choice { side, pile } => {
                eat(1);
                eat(match side {
                    Side::Left => 0,
                    Side::Right => 1,
                });
                eat(match pile {
                    Pile::FreshAccumulation => 0,
                    Pile::OldAccumulation => 1,
                    Pile::SingleFresh => 2,
                });
            }
            Fixture::Corridor { length, workers } => {
                eat(2);
                eat(u64::from(length));
                eat(u64::from(workers));
            }
        }
        if let Some(state) = &self.goal_state {
            eat(1); // Explicit KnownGoal discriminant; absent tasks add no bytes.
            eat(pos(state.task.goal));
            eat(u64::from(state.task.goal_weight));
            eat(state.seen_open.len() as u64);
            for &seen_open in &state.seen_open {
                eat(u64::from(seen_open));
            }
        }
        eat(u64::from(self.setup.width));
        eat(u64::from(self.setup.height));
        eat(pos(self.setup.exit));
        eat(self.setup.start_tick);
        for positions in [&self.setup.open, &self.setup.diggable, &self.setup.workers] {
            eat(positions.len() as u64);
            for &p in positions {
                eat(pos(p));
            }
        }
        eat(self.setup.spoil.len() as u64);
        for unit in &self.setup.spoil {
            eat(pos(unit.pos));
            eat(unit.born);
        }
        eat(self.open.len() as u64);
        for (&open, &diggable) in self.open.iter().zip(&self.diggable) {
            eat(u64::from(open));
            eat(u64::from(diggable));
        }
        eat(self.workers.len() as u64);
        for w in &self.workers {
            eat(u64::from(w.id));
            eat(pos(w.pos));
            eat(u64::from(w.carried.is_some()));
            if let Some(id) = w.carried {
                eat(id);
            }
            eat(u64::from(w.target.is_some()));
            if let Some(target) = w.target {
                eat(pos(target));
            }
            eat(u64::from(w.loaded_moves));
        }
        eat(self.units.len() as u64);
        for unit in self.units.values() {
            eat(unit.id);
            eat(unit.born);
            match unit.location {
                UnitLocation::Loose(p) => {
                    eat(0);
                    eat(pos(p));
                }
                UnitLocation::Carried(worker) => {
                    eat(1);
                    eat(u64::from(worker));
                }
                UnitLocation::Disposed => eat(2),
            }
        }
        let mut rng = self.rng.clone();
        for _ in 0..4 {
            eat(rng.next_u64());
        }
        h
    }
}
