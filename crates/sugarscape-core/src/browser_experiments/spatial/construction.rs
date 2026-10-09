//! Bounded read-only physical and private-memory capture around the unchanged construction engine.
use super::{
    budget::{self, CaptureBudget},
    capture,
};
use crate::{
    browser_experiments::{
        error, record, wire::lossless_value, Checkpoint, EpisodeKind, EpisodeRecord, FieldError,
        Input, Semantics, EPISODE_VERSION,
    },
    foraging::construction as native,
};
use serde_json::{json, Value};

type Result<T> = std::result::Result<T, Vec<FieldError>>;
const NATIVE_SNAPSHOT_LIMIT: usize = 64 * 1024 * 1024;
struct Capture {
    snapshots: Vec<native::Snapshot>,
    snapshot_bytes: usize,
    checkpoints: Vec<Checkpoint>,
    public: Value,
    budget: CaptureBudget,
}

/// Commit the native ordered ticks in exactly one original World. Sampling and
/// knowledge capture use only its original observational methods.
pub fn run(input: &Input) -> Result<EpisodeRecord> {
    budget::preflight(input)?;
    let Input::ForagingConstruction {
        setup,
        seed,
        ticks,
        sample_every,
    } = input
    else {
        return Err(error("study", "expected the construction spatial study"));
    };
    let setup = setup.to_core()?.normalized()?;
    let mut budget = CaptureBudget::new();
    // Charge the adapter's retained normalized setup as well as its final wire copy.
    budget.charge(&lossless_value(&setup)?)?;
    let mut world = native::World::new(setup.clone(), seed.value())?;
    let public = lossless_value(&json!({
        "label":"Engineering demonstration",
        "arena":{"width":setup.width,"height":setup.height},
        "nest":setup.nest,
        "waste":setup.waste,
        "local_state_availability":"captured own Agent state and private topology beliefs; per-cell observation times, food history and occupancy were not recorded"
    }))?;
    budget.charge(&public)?;
    let mut capture = Capture {
        snapshots: Vec::new(),
        snapshot_bytes: 0,
        checkpoints: Vec::new(),
        public,
        budget,
    };
    capture_world(&world, "spatial_initial", &mut capture)?;
    for completed in 1..=*ticks {
        world.step()?;
        if completed % sample_every == 0 || completed == *ticks {
            capture_world(
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
        setup,
        seed: seed.value(),
        options: native::RunOptions {
            ticks: *ticks,
            sample_every: *sample_every,
            snapshots: true,
        },
        summary: world.summary()?,
        snapshots: capture.snapshots,
        snapshot_bytes: capture.snapshot_bytes as u64,
    };
    let native = lossless_value(&episode)?;
    // Typed Snapshot staging, researcher snapshots and this complete native wire
    // copy are three deliberately separate charges under the original profile.
    capture.budget.charge(&native)?;
    let metadata = json!({"sampling":{"ticks":ticks,"sample_every":sample_every},"clock_semantics":"completed_tick_boundary","target":super::target_tag()});
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
fn capture_world(world: &native::World, kind: &str, capture: &mut Capture) -> Result<()> {
    let snapshot = world.snapshot()?;
    let bytes = record::serialized_size(
        &snapshot,
        NATIVE_SNAPSHOT_LIMIT - capture.snapshot_bytes,
        "snapshot_bytes",
    )?;
    capture.budget.charge(&lossless_value(&snapshot)?)?;
    let checkpoint = checkpoint_from_snapshot(
        world,
        &snapshot,
        kind,
        capture.checkpoints.len() as u32,
        &capture.public,
        &mut capture.budget,
    )?;
    record::push_checkpoint(&mut capture.checkpoints, checkpoint)?;
    capture.snapshot_bytes += bytes;
    capture.snapshots.push(snapshot);
    Ok(())
}
pub(crate) fn checkpoint_from_snapshot(
    world: &native::World,
    snapshot: &native::Snapshot,
    kind: &str,
    index: u32,
    public: &Value,
    budget: &mut CaptureBudget,
) -> Result<Checkpoint> {
    let mut checkpoint = capture::begin_checkpoint(
        snapshot,
        snapshot.summary.completed_ticks,
        kind,
        index,
        public,
        budget,
    )?;
    for agent in &snapshot.agents {
        // Original typed World access stays family-local, one Agent at a time.
        // No global inventories, access caches or origins enter this pair.
        let knowledge = world.knowledge(agent.id)?;
        capture::insert_local(&mut checkpoint, agent.id, agent, &knowledge, budget)?;
    }
    Ok(checkpoint)
}

/// Exercise the production checkpoint capture using a real World without steps.
#[cfg(test)]
pub(crate) fn adapter_checkpoint(world: &native::World) -> Result<Checkpoint> {
    checkpoint_from_snapshot(
        world,
        &world.snapshot()?,
        "spatial_sample",
        0,
        &Value::Null,
        &mut CaptureBudget::new(),
    )
}
