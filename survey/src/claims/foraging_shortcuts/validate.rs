//! Bind candidate inputs and clocks before accepting observed saved evidence.
use super::{
    access::validate_access,
    manifest::Condition,
    physical::{require, validate_frames},
    wire::WireEpisode,
    RunKey,
};
pub(super) fn validate_episode(c: &Condition, key: &RunKey, e: &WireEpisode) -> Result<(), String> {
    let validate = || -> Result<(), String> {
        require(c.id == key.condition, "key.condition")?;
        require(e.seed == key.seed, "episode.seed")?;
        let expected = c
            .setup
            .clone()
            .normalized()
            .map_err(|e| format!("expected setup: {e:?}"))?;
        require(
            serde_json::to_vec(&e.setup).map_err(|e| e.to_string())?
                == serde_json::to_vec(&expected).map_err(|e| e.to_string())?,
            "episode.setup.normalized",
        )?;
        require(
            serde_json::to_vec(&e.options).map_err(|e| e.to_string())?
                == serde_json::to_vec(&c.options).map_err(|e| e.to_string())?,
            "episode.options",
        )?;
        require(
            e.snapshots
                .iter()
                .map(|f| f.summary.completed_ticks)
                .collect::<Vec<_>>()
                == [0, 128, 256, 384, 512],
            "episode.frames",
        )?;
        require(
            e.snapshots.last().is_some_and(|f| f.summary == e.summary),
            "episode.summary",
        )?;
        validate_frames(&e.setup, &e.snapshots)?;
        validate_access(&e.setup, &e.snapshots)?;
        for f in &e.snapshots {
            let clock = u64::from(f.summary.completed_ticks) + 1;
            require(
                f.summary.compute.observations == 8 * clock
                    && f.summary.compute.cells_inspected == 40 * clock,
                "compute.candidate.observations.cells_inspected",
            )?;
            for a in &f.agents {
                require(
                    a.compute.cells_inspected == 5 * clock,
                    "compute.agent.cells_inspected",
                )?;
            }
        }
        let bytes = e.snapshots.iter().try_fold(0u64, |n, f| {
            let bytes = serde_json::to_vec(f).map_err(|e| e.to_string())?.len() as u64;
            n.checked_add(bytes)
                .ok_or_else(|| "snapshot_bytes overflow".to_string())
        })?;
        require(
            bytes == e.snapshot_bytes && bytes <= 64 * 1024 * 1024,
            "episode.snapshot_bytes",
        )?;
        Ok(())
    };
    validate().map_err(|e| format!("{}/{}: {e}", key.condition, key.seed))
}
