//! Fixed, checked food-task rig and intervention clock.
use super::state::*;
use crate::{
    agent::{in_slot_0, Agent},
    config::*,
    geometry::Pos,
    world::World,
};

pub fn rig_config(lab: LabConfig) -> Config {
    let mut c = Config {
        width: 11,
        height: 11,
        population: 1,
        vision: URange::new(8, 8),
        walls: [(0, 0, 11, 1), (0, 10, 11, 1), (0, 1, 1, 9), (10, 1, 1, 9)]
            .into_iter()
            .map(|(x, y, width, height)| Wall {
                x,
                y,
                width,
                height,
                opaque: true,
            })
            .collect(),
        goods: vec![Good {
            map: Map::Flat { capacity: 0.0 },
            ..Good::sugar()
        }],
        movement: Movement {
            mode: MoveMode::Walk,
            speed: 1,
        },
        ..Config::default()
    };
    c.growback.rate = 0.0;
    c.memory.span = 128;
    c.memory.share = 1.0;
    c.memory.prior = MemoryPrior::Map;
    c.goap.horizon = lab.quota;
    c.decision.travel = 1.0;
    c.decision.crowding = 0.0;
    c.decision.idle = Idle::Stay;
    c.decision.rule = match lab.controller {
        Controller::ReactiveUtility => DecisionRule::Utility,
        Controller::TaskGoap | Controller::LegacyGoap => DecisionRule::Goap,
        _ => DecisionRule::BehaviorTree,
    };
    c.behavior_tree.profile = match lab.controller {
        Controller::GuardedTree => Profile::GuardedRate,
        Controller::UnguardedTree => Profile::UnguardedRate,
        _ => Profile::BookLeaf,
    };
    c.behavior_tree_lab = Some(lab);
    c
}
pub fn conditions() -> Vec<LabConfig> {
    let mut out = vec![];
    for controller in [
        Controller::ReactiveUtility,
        Controller::GuardedTree,
        Controller::MatchedFsm,
        Controller::UnguardedTree,
        Controller::TaskGoap,
        Controller::LegacyGoap,
    ] {
        for scenario in [
            Scenario::Stable,
            Scenario::BetterAlternative,
            Scenario::DepletedTarget,
            Scenario::TemporaryObstacle,
        ] {
            for quota in [20, 40] {
                for mirrored in [false, true] {
                    out.push(LabConfig {
                        controller,
                        scenario,
                        quota,
                        mirrored,
                    });
                }
            }
        }
    }
    out.sort_by_key(condition_id);
    out
}
pub fn condition_id(lab: &LabConfig) -> String {
    let controller = serde_json::to_value(lab.controller).unwrap();
    let scenario = serde_json::to_value(lab.scenario).unwrap();
    format!(
        "{}-{}-q{}-{}",
        controller.as_str().unwrap(),
        scenario.as_str().unwrap(),
        lab.quota,
        if lab.mirrored { "mirrored" } else { "original" }
    )
}
pub(crate) fn transform(lab: &LabConfig, p: Pos) -> Pos {
    if lab.mirrored {
        Pos::new(10 - p.x, p.y)
    } else {
        p
    }
}
pub(crate) fn initialize(w: &mut World) {
    let lab = w.config.behavior_tree_lab.clone().unwrap();
    // Build actual levels before random founder construction and map prior.
    for (pos, food, capacity) in [
        (Pos::new(3, 5), 4.0, 4.0),
        (Pos::new(7, 5), 24.0, 24.0),
        (Pos::new(4, 9), 24.0, 36.0),
    ] {
        let site = w.site_mut(transform(&lab, pos));
        site.resource[0] = food;
        site.capacity[0] = capacity;
    }
    let mut a = Agent::random(&w.config, transform(&lab, Pos::new(2, 5)), 0, &mut w.rng);
    a.holdings = in_slot_0(16.0);
    a.initial = in_slot_0(16.0);
    a.metabolism = in_slot_0(1);
    a.vision = 8;
    a.remembers = true;
    a.age = 0;
    a.max_age = 128;
    w.insert_agent(a).unwrap();
    w.behavior_tree_lab = Some(LabRuntime {
        task: TaskState::new(lab.quota),
        living_ticks: 0,
        external_added: 0.0,
        external_removed: 0.0,
        consumed: 0.0,
        death_loss: 0.0,
        diagnostics: true,
        controller_timing: false,
        fatal_error: None,
        errors: vec![],
        observation: None,
        receipt: None,
        controller_seconds: None,
    });
}
pub(crate) fn validation_errors(c: &Config) -> Vec<FieldError> {
    let mut errors = vec![];
    let Some(lab) = &c.behavior_tree_lab else {
        if matches!(
            c.behavior_tree.profile,
            Profile::GuardedRate | Profile::UnguardedRate
        ) {
            errors.push(FieldError::new(
                "behavior_tree.profile",
                "task profiles require the checked behavior-tree lab",
            ));
        }
        return errors;
    };
    if ![20, 40].contains(&lab.quota) {
        errors.push(FieldError::new(
            "behavior_tree_lab.quota",
            "supported quotas are 20 and 40",
        ));
    }
    // All knobs outside the four typed treatment axes are fixed, including
    // inactive extensions and future Config fields with serialized defaults.
    let expected = serde_json::to_value(rig_config(lab.clone())).expect("serializable rig");
    let actual = serde_json::to_value(c).expect("serializable config");
    let fields: std::collections::BTreeSet<_> = actual
        .as_object()
        .expect("configuration object")
        .keys()
        .chain(expected.as_object().expect("rig object").keys())
        .collect();
    for field in fields {
        if field != "behavior_tree_lab" && expected.get(field) != actual.get(field) {
            errors.push(FieldError::new(
                field,
                "unsupported behavior-tree lab configuration",
            ));
        }
    }
    errors
}

pub(crate) fn begin_step(w: &mut World) -> Result<(), String> {
    let Some(lab) = w.config.behavior_tree_lab.clone() else {
        return Ok(());
    };
    let actor_alive = w.agent(1).is_some();
    let r = w
        .behavior_tree_lab
        .as_mut()
        .expect("constructed lab runtime");
    if let Some(error) = &r.fatal_error {
        return Err(error.clone());
    }
    if !actor_alive {
        // No actor turn will expire these live deadlines; prior frames retain
        // their history without a controller call or a random draw.
        super::policy::expire_failed(&mut r.task, w.tick + 1);
    }
    r.observation = None;
    r.receipt = None;
    r.controller_seconds = None;
    w.bt_work = None;
    let action_tick = w.tick + 1;
    let blocked = transform(&lab, Pos::new(5, 5));
    let result = if action_tick == 3 {
        match lab.scenario {
            Scenario::Stable => Ok(()),
            Scenario::BetterAlternative => {
                w.site_mut(transform(&lab, Pos::new(4, 9))).resource[0] += 12.0;
                w.behavior_tree_lab.as_mut().unwrap().external_added += 12.0;
                Ok(())
            }
            Scenario::DepletedTarget => {
                let site = w.site_mut(transform(&lab, Pos::new(7, 5)));
                let removed = site.resource[0];
                site.resource[0] = 0.0;
                w.behavior_tree_lab.as_mut().unwrap().external_removed += removed;
                Ok(())
            }
            Scenario::TemporaryObstacle => w.close_behavior_tree_wall(blocked),
        }
    } else {
        if action_tick == 7 && lab.scenario == Scenario::TemporaryObstacle {
            w.open_wall(blocked);
        }
        Ok(())
    };
    if let Err(error) = &result {
        w.behavior_tree_lab.as_mut().unwrap().fatal_error = Some(error.clone());
    }
    result
}
