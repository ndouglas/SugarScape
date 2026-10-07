use super::{Setup, Snapshot, Summary, World};
use crate::config::FieldError;
use std::io::{self, Write};

const SNAPSHOT_LIMIT: u64 = 64 * 1024 * 1024;

/// Fixed horizon and observational sampling. The interval must be positive even
/// when snapshots are disabled; execution never stops at resource exhaustion.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub struct RunOptions {
    pub ticks: u32,
    pub sample_every: u32,
    pub snapshots: bool,
}
/// Normalized replay inputs preserve spawn identity and original resource coordinates.
/// Replay identity requires the same setup, seed, version and supported platform.
/// Bytes count compact JSON for each snapshot, excluding enclosing fields and
/// inter-frame delimiters. Sampling and serialization consume no random draws.
#[derive(Clone, Debug, PartialEq, serde::Serialize)]
pub struct Episode {
    pub setup: Setup,
    pub seed: u64,
    pub summary: Summary,
    pub snapshots: Vec<Snapshot>,
    pub snapshot_bytes: u64,
}
/// Runs every requested tick, with ascending agent IDs and their associated
/// competitive-access bias. Controllers use local observations and nest advice,
/// never the global researcher view. Invalid setup and options are aggregated
/// before construction; failures yield no successful partial episode.
pub fn run(setup: Setup, seed: u64, options: RunOptions) -> Result<Episode, Vec<FieldError>> {
    validate_options(&setup, &options)?;
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
        setup: world.setup.clone(),
        seed,
        summary: world.summary()?,
        snapshots,
        snapshot_bytes,
    })
}
pub(super) fn validate_options(setup: &Setup, options: &RunOptions) -> Result<(), Vec<FieldError>> {
    let mut errors = setup.validate().err().unwrap_or_default();
    if !(1..=7200).contains(&options.ticks) {
        errors.push(FieldError::new("ticks", "must be in 1..=7200"));
    }
    if options.sample_every == 0 {
        errors.push(FieldError::new("sample_every", "must be positive"));
    }
    if setup.workers.len() as u64 * u64::from(options.ticks) > 1_000_000 {
        errors.push(FieldError::new(
            "opportunities",
            "workers * ticks must be at most 1000000",
        ));
    }
    if !errors.is_empty() {
        return Err(errors);
    }
    Ok(())
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
                snapshot.summary.completed_ticks
            ),
        )]
    })?;
    *bytes = writer.bytes;
    frames.push(snapshot);
    Ok(())
}
