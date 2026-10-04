//! Supplied task identity and private, locally observed completion memory.

use super::{
    config::checked_cells,
    runner::{run_world, Recording},
    ActionEvent, Episode, Fixture, LabConfig, Pos, RunOptions, World,
};
use crate::config::FieldError;
use serde::{Deserialize, Deserializer, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AccessObjective {
    Explore,
    KnownGoal,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AccessTask {
    #[serde(deserialize_with = "deserialize_goal")]
    pub goal: Pos,
    pub objective: AccessObjective,
    pub goal_weight: u32,
}

fn deserialize_goal<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Pos, D::Error> {
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct GoalCoordinate {
        x: u32,
        y: u32,
    }
    let goal = GoalCoordinate::deserialize(deserializer)?;
    Ok(Pos {
        x: goal.x,
        y: goal.y,
    })
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AccessConfig {
    pub lab: LabConfig,
    pub task: AccessTask,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GoalObservation {
    pub tick: u64,
    pub opportunities_before: u64,
    pub worker: u32,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AccessDiagnostics {
    pub completion_observations: Vec<GoalObservation>,
    pub local_completion_checks: u64,
    pub goal_weight_evaluations: u64,
}

#[derive(Clone, Debug)]
pub(super) struct GoalState {
    pub task: AccessTask,
    pub seen_open: Vec<bool>,
    pub diagnostics: AccessDiagnostics,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct GoalGuidance {
    pub goal: Pos,
    pub weight: u32,
    pub seen_open: bool,
}

pub(super) fn validate_task(task: &AccessTask, world: &World) -> Result<(), Vec<FieldError>> {
    let mut errors = Vec::new();
    match world.index(task.goal) {
        None => errors.push(FieldError::new("task.goal", "must be in bounds")),
        Some(index) if world.open[index] => {
            errors.push(FieldError::new("task.goal", "must initially be solid"));
        }
        Some(index) if !world.diggable[index] => {
            errors.push(FieldError::new("task.goal", "must be diggable"));
        }
        Some(_) => {}
    }
    if task.goal_weight == 0 {
        errors.push(FieldError::new("task.goal_weight", "must be positive"));
    }
    match world.config.response_weight.checked_mul(task.goal_weight) {
        None => errors.push(FieldError::new(
            "task.goal_weight",
            "cue times goal weight overflows per-face weight",
        )),
        Some(weight) => {
            match checked_cells(world.setup.width, world.setup.height, "task.dimensions") {
                Err(error) => errors.push(error),
                Ok(cells) => {
                    if u64::from(weight).checked_mul(cells as u64).is_none() {
                        errors.push(FieldError::new(
                            "task.goal_weight",
                            "frontier ticket sum overflow",
                        ));
                    }
                }
            }
        }
    }
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

pub(super) fn install_goal_state(
    world: &mut World,
    task: &AccessTask,
) -> Result<(), Vec<FieldError>> {
    validate_task(task, world)?;
    world.goal_state = match task.objective {
        AccessObjective::Explore => None,
        AccessObjective::KnownGoal => Some(GoalState {
            task: task.clone(),
            seen_open: vec![false; world.workers.len()],
            diagnostics: AccessDiagnostics::default(),
        }),
    };
    Ok(())
}

/// Historical cost and shortest open-cell route at the first access action.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AccessMilestone {
    pub tick: u64,
    /// One-based committed action index, including blocked actions and waits.
    pub opportunity: u64,
    pub worker: u32,
    pub goal: Pos,
    pub digs: u64,
    pub disposed: u64,
    pub carried: u64,
    pub loose: u64,
    pub exit_distance: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AccessSummary {
    pub structurally_accessible: bool,
    pub first_access: Option<AccessMilestone>,
    pub final_exit_distance: Option<u32>,
    pub observed_opportunities: u64,
    pub deadline_censored: bool,
}

/// KnownGoal replay requires the outer task config as well as the nested Episode.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AccessEpisode {
    pub config: AccessConfig,
    pub episode: Episode,
    pub access: AccessSummary,
    pub task_assumptions: Vec<String>,
    /// Completion-record logical storage is completion_observations.len();
    /// it is separate from the unchanged base Episode.storage accounting.
    pub task_diagnostics: AccessDiagnostics,
}

/// Researcher observation only; never shared with worker completion memory.
#[derive(Clone, Debug)]
pub(super) struct AccessObserver {
    pub(super) goal: Pos,
    pub(super) first: Option<AccessMilestone>,
}

impl AccessObserver {
    pub(super) fn observe(&mut self, world: &World, recording: &Recording, event: &ActionEvent) {
        let index = world.index(self.goal).expect("validated access goal");
        if self.first.is_none() && world.open[index] {
            let inventory = world.inventory();
            self.first = Some(AccessMilestone {
                tick: event.tick,
                opportunity: recording.events.len() as u64,
                worker: event.worker,
                goal: self.goal,
                digs: world.excavated,
                disposed: inventory.disposed,
                carried: inventory.carried,
                loose: inventory.loose,
                exit_distance: recording.exit_field[index].expect("open goal connected to exit"),
            });
        }
    }

    pub(super) fn summary(&self, world: &World) -> AccessSummary {
        let index = world.index(self.goal).expect("validated access goal");
        let structurally_accessible = world.open[index];
        AccessSummary {
            structurally_accessible,
            first_access: self.first.clone(),
            final_exit_distance: world.recording.exit_field[index],
            observed_opportunities: world.recording.events.len() as u64,
            deadline_censored: !structurally_accessible,
        }
    }
}

/// Run the complete budget and observe structural access after each committed action.
pub fn run_access_episode(
    config: AccessConfig,
    seed: u64,
    options: RunOptions,
) -> Result<AccessEpisode, Vec<FieldError>> {
    if !matches!(config.lab.fixture, Fixture::Growing { .. }) {
        return Err(vec![FieldError::new(
            "lab.fixture",
            "resource access requires a growing fixture",
        )]);
    }
    let mut world = World::new(config.lab.clone(), seed)?;
    install_goal_state(&mut world, &config.task)?;
    let observer = AccessObserver {
        goal: config.task.goal,
        first: None,
    };
    let (episode, access, task_diagnostics) = run_world(world, seed, options, Some(observer))?;
    let mut task_assumptions = vec![
        "Access is structural for the present four-neighbor mover, ignoring transient occupancy; no realized consumer use or resource collection is measured.".into(),
        "First access is observed after a committed action; the full requested budget continues after access.".into(),
        "Task completion records consume one logical record per completion_observations entry, separately from base Episode storage; task checks and weight evaluations are separate from base BFS metrics.".into(),
    ];
    match config.task.objective {
        AccessObjective::Explore => task_assumptions.push("Explore is observer-only: workers receive no task coordinate or completion signal.".into()),
        AccessObjective::KnownGoal => task_assumptions.extend([
            "KnownGoal workers receive the supplied goal coordinate; this is benchmark knowledge rather than resource discovery.".into(),
            "Manhattan weighting is a supplied coordinate heuristic, not a physical signal or navigable route.".into(),
            "Completion memory is private and local: each worker latches only its own ordinary open-cell observation.".into(),
            "Reproducing KnownGoal requires the outer AccessConfig; the nested Episode lab config alone is insufficient.".into(),
        ]),
    }
    Ok(AccessEpisode {
        config,
        episode,
        access: access.expect("access observer produces summary"),
        task_assumptions,
        task_diagnostics,
    })
}
