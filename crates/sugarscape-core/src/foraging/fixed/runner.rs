use super::{Inventory, Setup, Snapshot, WorkCounts, World};
use crate::config::FieldError;
use std::io::{self, Write};

const SNAPSHOT_LIMIT: u64 = 64 * 1024 * 1024;

/// Fixed horizon and observational sampling. The interval must be positive even
/// when snapshots are disabled; execution never stops at resource exhaustion.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RunOptions {
    pub ticks: u32,
    pub sample_every: u32,
    pub snapshots: bool,
}
/// Integer diagnostics. Missing event times are censored at the fixed horizon.
/// Events use processing ticks (starting at zero); completed ticks count commits.
#[derive(Clone, Debug, PartialEq, serde::Serialize)]
pub struct Summary {
    pub completed_ticks: u32,
    pub inventory: Inventory,
    pub work: WorkCounts,
    pub per_agent: Vec<WorkCounts>,
    pub expired_records: u64,
    pub first_pickup_tick: Option<u32>,
    pub first_delivery_tick: Option<u32>,
    pub all_delivered_tick: Option<u32>,
}
/// Replay identity requires the same setup, seed, version and supported platform.
/// Bytes count compact JSON for each snapshot, excluding enclosing fields and
/// inter-frame delimiters. Sampling and serialization consume no random draws.
#[derive(Clone, Debug, PartialEq, serde::Serialize)]
pub struct Episode {
    pub seed: u64,
    pub summary: Summary,
    pub snapshots: Vec<Snapshot>,
    pub snapshot_bytes: u64,
}
impl World {
    /// Validates inventory and recomputes checked totals from authoritative
    /// per-agent counters without drawing or expiring waypoint records.
    pub fn summary(&self) -> Result<Summary, Vec<FieldError>> {
        super::controller::check(&self.setup, &self.state)?;
        let mut work = WorkCounts::default();
        let per_agent = self
            .state
            .agents
            .iter()
            .map(|a| {
                work.checked_add_assign(&a.work)?;
                Ok(a.work.clone())
            })
            .collect::<Result<Vec<_>, Vec<FieldError>>>()?;
        Ok(Summary {
            completed_ticks: self.state.tick,
            inventory: self.state.ledger.inventory(),
            work,
            per_agent,
            expired_records: self.state.server.expired(),
            first_pickup_tick: self.state.first_pickup_tick,
            first_delivery_tick: self.state.first_delivery_tick,
            all_delivered_tick: self.state.all_delivered_tick,
        })
    }
}
/// Runs every requested tick, with ascending agent IDs and their associated
/// competitive-access bias. Controllers use local observations and nest advice,
/// never the global researcher view. Invalid setup and options are aggregated
/// before construction; failures yield no successful partial episode.
pub fn run(setup: Setup, seed: u64, options: RunOptions) -> Result<Episode, Vec<FieldError>> {
    let mut errors = setup.validate().err().unwrap_or_default();
    if !(1..=7200).contains(&options.ticks) {
        errors.push(FieldError::new("ticks", "must be in 1..=7200"));
    }
    if options.sample_every == 0 {
        errors.push(FieldError::new("sample_every", "must be positive"));
    }
    if u64::from(setup.agents) * u64::from(options.ticks) > 1_000_000 {
        errors.push(FieldError::new(
            "opportunities",
            "agents * ticks must be at most 1000000",
        ));
    }
    if !errors.is_empty() {
        return Err(errors);
    }
    let mut world = World::new(setup, seed)?;
    let mut snapshots = Vec::new();
    let mut snapshot_bytes = 0;
    if options.snapshots {
        append_snapshot(
            &mut snapshots,
            &mut snapshot_bytes,
            world.snapshot()?,
            SNAPSHOT_LIMIT,
        )?;
    }
    for completed in 1..=options.ticks {
        world.step()?;
        if options.snapshots
            && (completed % options.sample_every == 0 || completed == options.ticks)
        {
            append_snapshot(
                &mut snapshots,
                &mut snapshot_bytes,
                world.snapshot()?,
                SNAPSHOT_LIMIT,
            )?;
        }
    }
    Ok(Episode {
        seed,
        summary: world.summary()?,
        snapshots,
        snapshot_bytes,
    })
}
struct CountingWriter {
    bytes: u64,
    limit: u64,
}
impl Write for CountingWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let size = u64::try_from(buf.len()).map_err(io::Error::other)?;
        let next = self
            .bytes
            .checked_add(size)
            .filter(|next| *next <= self.limit)
            .ok_or_else(|| io::Error::other("snapshot byte budget exceeded"))?;
        self.bytes = next;
        Ok(buf.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
pub(super) fn append_snapshot(
    frames: &mut Vec<Snapshot>,
    bytes: &mut u64,
    snapshot: Snapshot,
    limit: u64,
) -> Result<(), Vec<FieldError>> {
    let mut writer = CountingWriter {
        bytes: *bytes,
        limit,
    };
    serde_json::to_writer(&mut writer, &snapshot).map_err(|error| {
        vec![FieldError::new(
            "snapshot_bytes",
            format!(
                "completed tick {}: {error}; limit {limit} bytes",
                snapshot.completed_ticks
            ),
        )]
    })?;
    *bytes = writer.bytes;
    frames.push(snapshot);
    Ok(())
}
