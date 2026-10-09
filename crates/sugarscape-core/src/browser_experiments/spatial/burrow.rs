//! Sampled display records around the unchanged Burrow runners.
use super::budget::{self, CaptureBudget};
use crate::{
    browser_experiments::{
        error, record, wire::lossless_value, Checkpoint, EpisodeKind, EpisodeRecord, FieldError,
        Input, Semantics, EPISODE_VERSION,
    },
    burrow as native,
};
use serde_json::{json, Value};
use std::collections::BTreeMap;

type Result<T> = std::result::Result<T, Vec<FieldError>>;

/// Run the original native API once, preserving its complete output.
pub fn run(input: &Input) -> Result<EpisodeRecord> {
    budget::preflight(input)?;
    match input {
        Input::BurrowExcavation {
            config,
            seed,
            ticks,
            sample_every,
        } => {
            let episode = native::run_episode(
                config.to_core()?,
                seed.value(),
                native::RunOptions {
                    ticks: *ticks,
                    sample_every: *sample_every,
                },
            )?;
            project(
                input,
                &episode,
                None,
                lossless_value(&episode)?,
                *ticks,
                *sample_every,
            )
        }
        Input::BurrowAccess {
            config,
            seed,
            ticks,
            sample_every,
        } => {
            let episode = native::run_access_episode(
                config.to_core()?,
                seed.value(),
                native::RunOptions {
                    ticks: *ticks,
                    sample_every: *sample_every,
                },
            )?;
            project(
                input,
                &episode.episode,
                Some(&episode),
                lossless_value(&episode)?,
                *ticks,
                *sample_every,
            )
        }
        _ => Err(error("study", "expected a Burrow spatial study")),
    }
}

/// Decode only the grid rows; trailing legend text is never interpreted as cells.
fn grid(episode: &native::Episode, frame: &native::Frame) -> Result<Value> {
    let mut lines = frame.ascii.split_inclusive('\n');
    let mut cells = Vec::new();
    for y in 0..episode.setup.height {
        let row = lines
            .next()
            .and_then(|line| line.strip_suffix('\n'))
            .ok_or_else(|| error("frame", "native ASCII grid row missing"))?;
        if row.len() != episode.setup.width as usize {
            return Err(error("frame", "native ASCII grid width mismatch"));
        }
        for (x, glyph) in row.bytes().enumerate() {
            if !b"#.EowW".contains(&glyph) {
                return Err(error("frame", "unknown native ASCII grid glyph"));
            }
            cells.push(json!({"x": x, "y": y, "glyph": char::from(glyph).to_string()}));
        }
    }
    Ok(
        json!({"width":episode.setup.width,"height":episode.setup.height,"cells":cells,"overlay_caveat":lines.collect::<String>()}),
    )
}

fn local(
    episode: &native::Episode,
    access: Option<&native::AccessEpisode>,
    frame: &native::Frame,
    opportunities: usize,
    terminal_choice: bool,
) -> Result<BTreeMap<String, Value>> {
    episode.worker_work.iter().map(|worker| {
        let events: Vec<_> = episode.events[..opportunities].iter().filter(|event| event.worker == worker.worker).collect();
        let choices: Vec<_> = episode.choices.iter().filter(|choice| {
            choice.worker == worker.worker && (choice.tick < frame.tick || (terminal_choice && choice.tick == frame.tick))
        }).collect();
        let mut value = json!({
            "events": events,
            "choices": choices,
            "holdings": null,
            "observations": null,
            "map": null,
            "availability": {"events":"committed_own_prefix","choices":"selected_own_prefix","holdings":"not_recorded_at_checkpoint","observations":"not_recorded","map":"not_recorded"}
        });
        if let Some(access) = access.filter(|a| a.config.task.objective == native::AccessObjective::KnownGoal) {
            // A marker at equality belongs to the next opportunity, after this
            // completed-round boundary; tick equality alone is insufficient.
            let completion: Vec<_> = access.task_diagnostics.completion_observations.iter()
                .filter(|marker| marker.worker == worker.worker && marker.opportunities_before < opportunities as u64).collect();
            value["goal"] = json!(access.config.task.goal);
            value["completion_observations"] = json!(completion);
            value["availability"]["goal"] = json!("supplied_known_goal");
            value["availability"]["completion_observations"] = json!("private_observed_prefix");
        }
        Ok((worker.worker.to_string(), lossless_value(&value)?))
    }).collect()
}

fn project(
    input: &Input,
    episode: &native::Episode,
    access: Option<&native::AccessEpisode>,
    native_value: Value,
    ticks: u32,
    sample_every: u32,
) -> Result<EpisodeRecord> {
    let mut budget = CaptureBudget::new();
    budget.charge(&native_value)?;
    let mut checkpoints = Vec::new();
    for (index, frame) in episode.frames.iter().enumerate() {
        let initial = index == 0;
        let terminal = !initial && index + 1 == episode.frames.len();
        let terminal_choice = terminal && episode.stop_reason == "choice_selected";
        let opportunities = episode
            .events
            .partition_point(|event| event.tick < frame.tick);
        // Initial/terminal Choice frames share a clock. Match physical prefix,
        // then retain the stage-appropriate diagnostics, never zip by index.
        let mut matching = episode.series.iter().filter(|snapshot| {
            snapshot.tick == frame.tick && snapshot.opportunities == opportunities as u64
        });
        let snapshot = if initial {
            matching.next()
        } else {
            matching.next_back()
        }
        .ok_or_else(|| error("frame", "native frame has no matching diagnostic boundary"))?;
        let completed = frame
            .tick
            .checked_sub(episode.setup.start_tick)
            .ok_or_else(|| error("frame", "native frame predates fixture start"))?;
        let clock =
            json!({"completed_ticks":completed.to_string(),"native_tick":frame.tick.to_string()});
        let mut researcher =
            json!({"frame":frame,"snapshot":snapshot,"grid":grid(episode, frame)?});
        if let Some(access) = access {
            let first = access
                .access
                .first_access
                .as_ref()
                .filter(|milestone| milestone.opportunity <= opportunities as u64);
            researcher["access"] = json!({"goal":access.config.task.goal,"objective":access.config.task.objective,"structurally_accessible":first.is_some(),"first_access":first});
        }
        let checkpoint = Checkpoint {
            index: index as u32,
            clock,
            kind: if initial {
                "spatial_initial"
            } else if terminal {
                "spatial_terminal"
            } else {
                "spatial_sample"
            }
            .into(),
            public: json!({"label":"Engineering demonstration","local_state_availability":"own traces only; observations, map and current holdings were not recorded"}),
            local: local(episode, access, frame, opportunities, terminal_choice)?,
            researcher: Some(lossless_value(&researcher)?),
        };
        // Charge the exact wire representation before each retained occurrence.
        budget.charge(
            &serde_json::to_value(&checkpoint).map_err(|e| error("checkpoint", e.to_string()))?,
        )?;
        record::push_checkpoint(&mut checkpoints, checkpoint)?;
    }
    let capture = json!({"sampling":{"ticks":ticks,"sample_every":sample_every},"clock_semantics":"native_sampled_tick"});
    budget.charge(&capture)?;
    let record = EpisodeRecord {
        kind: EpisodeKind::ExperimentEpisode,
        version: EPISODE_VERSION,
        study: input.study(),
        rules_identity: super::rules_identity(input.study())?,
        input: serde_json::to_value(input).map_err(|e| error("input", e.to_string()))?,
        semantics: Semantics::Trajectory,
        checkpoints,
        payload: json!({"native":native_value,"capture":capture}),
    };
    record::check_bounds(&record)?;
    Ok(record)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn local_projection_keeps_original_blocked_action_distinct_from_wait() {
        // The stock controller requests legal actions. Exercise the display
        // boundary with real original apply outcomes, not a fabricated outcome
        // tag or a claim that this is an authenticated controller episode.
        let config = native::LabConfig::default();
        let mut world = native::World::new(config.clone(), 7).unwrap();
        let blocked = world.apply(0, native::Action::Move(native::Pos { x: u32::MAX, y: 0 }));
        let wait = world.apply(0, native::Action::Wait);
        let mut episode = native::run_episode(
            config,
            7,
            native::RunOptions {
                ticks: 0,
                sample_every: 1,
            },
        )
        .unwrap();
        episode.events = vec![blocked.clone(), wait.clone()];
        let mut boundary = episode.frames[0].clone();
        boundary.tick = 1;
        let projected = local(&episode, None, &boundary, 2, false).unwrap();
        assert!(matches!(blocked.outcome, native::Outcome::Blocked { .. }));
        assert_eq!(wait.outcome, native::Outcome::Success);
        assert_eq!(
            projected["0"]["events"],
            lossless_value(&vec![blocked, wait]).unwrap()
        );
        assert_ne!(
            projected["0"]["events"][0]["outcome"],
            projected["0"]["events"][1]["outcome"]
        );
        assert_eq!(projected["0"]["events"][1]["action"], "wait");
    }
}
