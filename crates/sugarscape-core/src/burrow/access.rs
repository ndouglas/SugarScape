//! Supplied task identity and private, locally observed completion memory.

use super::{config::checked_cells, LabConfig, Pos, World};
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

// Task 2's public access constructor consumes these helpers and removes both allowances.
#[allow(dead_code)]
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

#[allow(dead_code)]
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
