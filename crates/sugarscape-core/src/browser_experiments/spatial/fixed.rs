//! Read-only sampled capture around the unchanged fixed-world engine.
use super::budget::{self, CaptureBudget};
use crate::{
    browser_experiments::{
        error, record, wire::lossless_value, Checkpoint, EpisodeKind, EpisodeRecord, FieldError,
        Input, Semantics, EPISODE_VERSION,
    },
    foraging::fixed as native,
};
use serde_json::{json, Value};
use std::collections::BTreeMap;

type Result<T> = std::result::Result<T, Vec<FieldError>>;
// Preserve the original runner's compact numeric Snapshot serialization limit.
const NATIVE_SNAPSHOT_LIMIT: usize = 64 * 1024 * 1024;

struct Capture {
    snapshots: Vec<native::Snapshot>,
    snapshot_bytes: usize,
    checkpoints: Vec<Checkpoint>,
    public: Value,
    budget: CaptureBudget,
}

/// Construct one original World, commit every requested tick, and capture only
/// original typed observational views at completed-tick boundaries.
pub fn run(input: &Input) -> Result<EpisodeRecord> {
    budget::preflight(input)?;
    let Input::ForagingFixed {
        setup,
        seed,
        ticks,
        sample_every,
    } = input
    else {
        return Err(error("study", "expected the fixed-world spatial study"));
    };
    let mut world = native::World::new(setup.to_core()?, seed.value())?;
    let public = lossless_value(&json!({
        "label":"Engineering demonstration",
        "arena":{"width":setup.width,"height":setup.height},
        "nest":setup.nest,
        "local_state_availability":"captured own Agent state; local food observations were not recorded"
    }))?;
    let mut budget = CaptureBudget::new();
    budget.charge(&public)?;
    let mut capture = Capture {
        snapshots: Vec::new(),
        snapshot_bytes: 0,
        checkpoints: Vec::new(),
        public,
        budget,
    };
    capture_fixed(&world, "spatial_initial", &mut capture)?;
    for completed in 1..=*ticks {
        world.step()?;
        if completed % sample_every == 0 || completed == *ticks {
            capture_fixed(
                &world,
                if completed == *ticks {
                    "spatial_terminal"
                } else {
                    "spatial_sample"
                },
                &mut capture,
            )?;
        }
    }
    let episode = native::Episode {
        seed: seed.value(),
        summary: world.summary()?,
        snapshots: capture.snapshots,
        snapshot_bytes: capture.snapshot_bytes as u64,
    };
    let native = lossless_value(&episode)?;
    // Deliberately conservative: typed snapshots were already charged before
    // retention; charge the complete final wire copy before retaining it too.
    capture.budget.charge(&native)?;
    let metadata = json!({
        "sampling":{"ticks":ticks,"sample_every":sample_every},
        "clock_semantics":"completed_tick_boundary",
        "target":super::target_tag()
    });
    capture.budget.charge(&metadata)?;
    let record = EpisodeRecord {
        kind: EpisodeKind::ExperimentEpisode,
        version: EPISODE_VERSION,
        study: input.study(),
        rules_identity: super::rules_identity(input.study())?,
        input: serde_json::to_value(input).map_err(|e| error("input", e.to_string()))?,
        semantics: Semantics::Trajectory,
        checkpoints: capture.checkpoints,
        payload: json!({"native":native,"capture":metadata}),
    };
    record::check_bounds(&record)?;
    Ok(record)
}

fn capture_fixed(world: &native::World, kind: &str, capture: &mut Capture) -> Result<()> {
    let snapshot = world.snapshot()?;
    let bytes = record::serialized_size(
        &snapshot,
        NATIVE_SNAPSHOT_LIMIT - capture.snapshot_bytes,
        "snapshot_bytes",
    )?;
    // Count original numeric serialization independently of quoted wire atoms.
    let snapshot_wire = lossless_value(&snapshot)?;
    capture.budget.charge(&snapshot_wire)?;
    let local: BTreeMap<_, _> = snapshot
        .agents
        .iter()
        .map(|agent| {
            Ok((
                agent.id.to_string(),
                lossless_value(&json!({"agent":agent}))?,
            ))
        })
        .collect::<Result<_>>()?;
    let checkpoint = Checkpoint {
        index: capture.checkpoints.len() as u32,
        clock: json!({"completed_ticks":snapshot.completed_ticks.to_string()}),
        kind: kind.into(),
        public: capture.public.clone(),
        local,
        researcher: Some(json!({"snapshot":snapshot_wire})),
    };
    capture.budget.charge(&checkpoint)?;
    record::push_checkpoint(&mut capture.checkpoints, checkpoint)?;
    capture.snapshot_bytes += bytes;
    capture.snapshots.push(snapshot);
    Ok(())
}
