//! Saved physical validation. No World is built, stepped, or restored here.
use std::collections::BTreeMap;
use sugarscape_core::{
    geometry::{Pos, Torus},
    minds::{
        astar::astar,
        behavior_tree::{
            self as bt,
            records::*,
            runtime::{Status, TreeState},
            state::*,
        },
        grid::TorusGrid,
    },
    rng,
};
fn require(test: bool, why: &str) -> Result<(), String> {
    if test {
        Ok(())
    } else {
        Err(why.into())
    }
}
fn torus() -> Torus {
    Torus::new(11, 11)
}
fn index(p: Pos) -> usize {
    torus().index(p)
}
fn transform(lab: &LabConfig, x: u32, y: u32) -> Pos {
    Pos::new(if lab.mirrored { 10 - x } else { x }, y)
}
fn distance(a: Pos, b: Pos) -> u32 {
    let dx = a.x.abs_diff(b.x);
    let dy = a.y.abs_diff(b.y);
    dx.min(11 - dx) + dy.min(11 - dy)
}
fn route(cells: &[Cell], origin: Pos, target: Pos) -> Option<Vec<Pos>> {
    if cells[index(target)].wall != 0 {
        return None;
    }
    let grid = TorusGrid::new(torus(), |p| cells[index(p)].wall == 0);
    astar(&grid, index(origin), index(target), 4096)
        .map(|s| s.path.into_iter().map(|i| torus().pos(i)).collect())
}
fn observation(a: &Actor, cells: &[Cell], task: &TaskState, tick: u64) -> Observation {
    let sight = torus().sight_until(a.pos, 8, |p| cells[index(p)].wall == 2);
    let mut candidates = vec![Candidate {
        site: index(a.pos) as u32,
        pos: a.pos,
        distance: 0,
        value: cells[index(a.pos)].food,
        remembered: false,
    }];
    for &(pos, distance) in &sight {
        if cells[index(pos)].wall == 0 {
            candidates.push(Candidate {
                site: index(pos) as u32,
                pos,
                distance,
                value: cells[index(pos)].food,
                remembered: false,
            })
        }
    }
    for m in &a.memory {
        let pos = torus().pos(m.site as usize);
        if pos != a.pos && cells[m.site as usize].wall == 0 && !sight.iter().any(|(p, _)| *p == pos)
        {
            candidates.push(Candidate {
                site: m.site,
                pos,
                distance: distance(a.pos, pos),
                value: m.levels[0],
                remembered: true,
            })
        }
    }
    Observation {
        action_tick: tick,
        origin: a.pos,
        quota: task.quota,
        gross: task.gross,
        candidates,
    }
}
fn update_memory(a: &mut Actor, cells: &[Cell], tick: u64) {
    let mut memory: BTreeMap<_, _> = a.memory.iter().map(|m| (m.site, m.clone())).collect();
    let mut sight = torus().sight_until(a.pos, 8, |p| cells[index(p)].wall == 2);
    sight.push((a.pos, 0));
    for (pos, _) in sight {
        let site = index(pos) as u32;
        let value = cells[site as usize].food;
        let m = memory.entry(site).or_insert(MemoryRecord {
            site,
            levels: vec![0.0],
            most: vec![0.0],
            tick,
        });
        m.levels[0] = value;
        m.most[0] = m.most[0].max(value);
        m.tick = tick;
    }
    a.memory = memory.into_values().collect()
}
fn initial(lab: &LabConfig, seed: u64, f: &Frame) -> Result<(), String> {
    require(f.tick == 0 && f.cells.len() == 121, "initial frame/grid")?;
    let mut cells: Vec<_> = (0..121)
        .map(|site| {
            let p = torus().pos(site);
            Cell {
                site: site as u32,
                food: 0.0,
                capacity: 0.0,
                wall: if p.x == 0 || p.x == 10 || p.y == 0 || p.y == 10 {
                    2
                } else {
                    0
                },
            }
        })
        .collect();
    for (x, y, food, capacity) in [(3, 5, 4.0, 4.0), (7, 5, 24.0, 24.0), (4, 9, 24.0, 36.0)] {
        let cell = &mut cells[index(transform(lab, x, y))];
        cell.food = food;
        cell.capacity = capacity;
    }
    require(f.cells == cells, "initial food/capacities/walls")?;
    let expected = Actor {
        id: 1,
        pos: transform(lab, 2, 5),
        holdings: 16.0,
        metabolism: 1,
        vision: 8,
        remembers: true,
        age: 0,
        max_age: 128,
        memory: cells
            .iter()
            .filter(|c| c.wall == 0)
            .map(|c| MemoryRecord {
                site: c.site,
                levels: vec![c.food],
                most: vec![c.food],
                tick: 0,
            })
            .collect(),
        motion_plan: MotionPlan {
            target: None,
            path: vec![],
            walked: false,
        },
    };
    require(
        f.actor.as_ref() == Some(&expected),
        "initial actor traits/prior",
    )?;
    require(
        f.task == TaskState::new(lab.quota)
            && f.legacy_plan.is_none()
            && f.observation.is_none()
            && f.receipt.is_none()
            && f.work.is_none()
            && f.controller_seconds.is_none(),
        "initial controller/diagnostics",
    )?;
    require(
        f.external_added == 0.0
            && f.external_removed == 0.0
            && f.consumed == 0.0
            && f.death_loss == 0.0
            && f.living_ticks == 0,
        "initial balances",
    )?;
    let mut expected_rng = rng::seeded(seed);
    let _ = sugarscape_core::agent::Agent::random(
        &bt::lab::rig_config(lab.clone()),
        expected.pos,
        0,
        &mut expected_rng,
    );
    require(
        f.rng_state_json == rng::state_json(&expected_rng),
        "initial RNG/seed binding",
    )
}
fn check_frame(f: &Frame, lab: &LabConfig) -> Result<(), String> {
    require(
        f.cells.len() == 121
            && f.cells.iter().enumerate().all(|(i, c)| {
                c.site as usize == i
                    && c.food.is_finite()
                    && c.food >= 0.0
                    && c.food <= c.capacity
                    && c.capacity.is_finite()
                    && c.wall <= 2
            }),
        "cell bounds/order",
    )?;
    let parsed: rng::SimRng =
        serde_json::from_str(&f.rng_state_json).map_err(|e| format!("typed RNG: {e}"))?;
    require(
        rng::state_json(&parsed) == f.rng_state_json,
        "noncanonical RNG",
    )?;
    require(
        f.fingerprint.len() == 16 && f.fingerprint.bytes().all(|b| b.is_ascii_hexdigit()),
        "fingerprint syntax",
    )?;
    require(
        f.errors.is_empty()
            && f.task.quota == lab.quota
            && f.task.gross.is_finite()
            && f.task.gross >= 0.0,
        "error/task state",
    )?;
    require(
        [
            f.external_added,
            f.external_removed,
            f.consumed,
            f.death_loss,
        ]
        .iter()
        .all(|x| x.is_finite() && *x >= 0.0),
        "nonfinite/negative balance",
    )?;
    let held = f.actor.as_ref().map_or(0.0, |a| a.holdings);
    require(
        (68.0 + f.external_added
            - f.external_removed
            - f.cells.iter().map(|c| c.food).sum::<f64>()
            - held
            - f.consumed
            - f.death_loss)
            .abs()
            < 1e-9,
        "food conservation",
    )?;
    if let Some(a) = &f.actor {
        require(
            a.pos.x < 11 && a.pos.y < 11 && a.holdings.is_finite() && a.holdings > 0.0,
            "actor location/holdings",
        )?;
        require(
            a.memory.windows(2).all(|w| w[0].site < w[1].site)
                && a.memory.iter().all(|m| {
                    m.site < 121
                        && m.levels.len() == 1
                        && m.most.len() == 1
                        && m.levels[0].is_finite()
                        && m.levels[0] >= 0.0
                        && m.most[0].is_finite()
                        && m.most[0] >= m.levels[0]
                        && m.tick <= f.tick.saturating_sub(1)
                }),
            "memory shape/clock",
        )?;
    }
    if let Some(tree) = bt::tree_for_controller(lab.controller) {
        f.task
            .tree
            .validate_for_tree(&tree)
            .map_err(|e| e.message)?;
    } else {
        require(f.task.tree == TreeState::default(), "inactive tree state")?
    }
    if lab.controller != Controller::MatchedFsm {
        require(
            f.task.fsm.phase == FsmPhase::Selecting,
            "inactive FSM phase",
        )?
    }
    if lab.controller != Controller::TaskGoap {
        require(f.task.task_plan.is_none(), "inactive task plan")?
    }
    if lab.controller != Controller::LegacyGoap {
        require(f.legacy_plan.is_none(), "inactive legacy plan")?
    }
    if let Some(t) = f.controller_seconds {
        require(
            t.is_finite() && t >= 0.0 && t <= u64::MAX as f64,
            "controller timing",
        )?
    }
    Ok(())
}
/// Prefix validation is also used for incomplete durable streams.
pub(super) fn validate_frames(frames: &[Frame], lab: &LabConfig, seed: u64) -> Result<(), String> {
    require(
        [20, 40].contains(&lab.quota) && !frames.is_empty() && frames.len() <= 65,
        "invalid lab/prefix length",
    )?;
    initial(lab, seed, &frames[0])?;
    for (i, f) in frames.iter().enumerate() {
        require(f.tick == i as u64, "frame clock")?;
        check_frame(f, lab).map_err(|e| format!("tick {i}: {e}"))?;
        if i > 0 {
            transition(&frames[i - 1], f, lab).map_err(|e| format!("tick {i}: {e}"))?
        }
    }
    Ok(())
}
pub fn validate_episode(r: &EpisodeRecord, expected: &LabConfig, seed: u64) -> Result<(), String> {
    require(
        r.schema == bt::EPISODE_SCHEMA
            && r.lab == *expected
            && r.seed == seed
            && r.completed_ticks == 64
            && r.frames.len() == 65
            && r.errors.is_empty(),
        "episode schema/identity/completeness",
    )?;
    validate_frames(&r.frames, expected, seed)?;
    let f = r.frames.last().unwrap();
    let attained = f.task.first_completion.is_some();
    let reason = if attained {
        None
    } else {
        Some(if f.actor.is_some() {
            UnattainedReason::HorizonWithoutQuota
        } else {
            UnattainedReason::DiedBeforeQuota
        })
    };
    require(
        r.quota_attained == attained
            && r.first_completion == f.task.first_completion
            && r.restricted_completion_ticks == f.task.first_completion.unwrap_or(64)
            && r.right_censored != attained
            && r.unattained_reason == reason
            && r.gross_gathered == f.task.gross
            && r.living_ticks == f.living_ticks
            && r.alive_at_horizon == f.actor.is_some(),
        "endpoint differs from raw state",
    )
}
fn transition(before: &Frame, after: &Frame, lab: &LabConfig) -> Result<(), String> {
    let t = after.tick;
    let mut cells = before.cells.clone();
    let mut added = before.external_added;
    let mut removed = before.external_removed;
    if t == 3 {
        match lab.scenario {
            Scenario::Stable => (),
            Scenario::BetterAlternative => {
                cells[index(transform(lab, 4, 9))].food += 12.0;
                added += 12.0
            }
            Scenario::DepletedTarget => {
                let b = &mut cells[index(transform(lab, 7, 5))];
                removed += b.food;
                b.food = 0.0
            }
            Scenario::TemporaryObstacle => {
                let p = transform(lab, 5, 5);
                require(
                    before.actor.as_ref().is_none_or(|a| a.pos != p),
                    "occupied wall event",
                )?;
                cells[index(p)].wall = 2
            }
        }
    }
    if t == 7 && lab.scenario == Scenario::TemporaryObstacle {
        cells[index(transform(lab, 5, 5))].wall = 0
    }
    require(
        after.external_added == added && after.external_removed == removed,
        "intervention balances",
    )?;
    let mut task = before.task.clone();
    task.failed_until.retain(|_, until| t < *until);
    let Some(actor) = &before.actor else {
        require(
            after.actor.is_none()
                && after.receipt.is_none()
                && after.observation.is_none()
                && after.work.is_none()
                && after.controller_seconds.is_none()
                && after.living_ticks == before.living_ticks,
            "dead clock has actor work",
        )?;
        require(
            after.task == task
                && after.cells == cells
                && after.rng_state_json == before.rng_state_json
                && after.consumed == before.consumed
                && after.death_loss == before.death_loss
                && after.legacy_plan.is_none(),
            "dead clock state/expiry",
        )?;
        return Ok(());
    };
    require(
        after.living_ticks == before.living_ticks + 1,
        "living turn counter",
    )?;
    let receipt = after
        .receipt
        .as_ref()
        .ok_or("live turn missing physical receipt")?;
    require(
        receipt.actor == 1
            && receipt.action_tick == t
            && receipt.origin == actor.pos
            && receipt.destination.x < 11
            && receipt.destination.y < 11
            && receipt.target.x < 11
            && receipt.target.y < 11,
        "receipt identity/position",
    )?;
    let mut expected_actor = actor.clone();
    let mut expected_rng: rng::SimRng =
        serde_json::from_str(&before.rng_state_json).map_err(|e| e.to_string())?;
    let mut legacy = before.legacy_plan.clone();
    if before.task.first_completion.is_some() {
        require(
            after.observation.is_none()
                && after.controller_seconds.is_none()
                && after
                    .work
                    .as_ref()
                    .is_none_or(|w| *w == bt::WorkCounters::default()),
            "hold diagnostics",
        )?;
        require(
            receipt.target == actor.pos
                && receipt.destination == actor.pos
                && receipt.gathered == 0.0
                && !receipt.route_failed,
            "completed hold harvested/moved",
        )?;
    } else {
        let work = after
            .work
            .as_ref()
            .ok_or("active call missing work counters")?;
        require(
            work.node_visits <= 64
                && work.target_selections <= 1
                && work.path_queries <= 1
                // A generous fixed-rig ceiling: at most121 candidates, nine
                // shortlisted slots and4096 search expansions. It also makes
                // all64-turn diagnostic sums safe on hostile saved integers.
                && work.candidate_evaluations <= 1_000_000
                && work.fallback_short <= 1
                && work.fallback_limit <= 1
                && !(work.fallback_short == 1 && work.fallback_limit == 1)
                && work.search_expansions.is_none_or(|n| n <= 4096)
                && work.search_unavailable_reason.as_ref().is_none_or(|s| !s.trim().is_empty())
                && work.search_expansions.is_none() == work.search_unavailable_reason.is_some(),
            "work counter bounds/availability",
        )?;
        if !matches!(
            lab.controller,
            Controller::GuardedTree | Controller::UnguardedTree
        ) {
            require(work.node_visits == 0, "inactive tree work")?
        }
        if !matches!(
            lab.controller,
            Controller::TaskGoap | Controller::LegacyGoap
        ) {
            require(
                work.search_expansions == Some(0)
                    && work.fallback_short == 0
                    && work.fallback_limit == 0,
                "inactive planner work",
            )?
        }
        let o = observation(actor, &cells, &before.task, t);
        require(
            after.observation.as_ref() == Some(&o),
            "research observation differs from permitted pre-action projection",
        )?;
        let selected = select(
            &o,
            actor,
            &cells,
            &mut task,
            &mut legacy,
            lab.controller,
            &mut expected_rng,
        )?;
        let target = selected.unwrap_or(actor.pos);
        require(
            receipt.target == target,
            "target differs from retained/selected policy",
        )?;
        let path = route(&cells, actor.pos, target);
        let destination = path
            .as_ref()
            .map_or(actor.pos, |p| p.get(1).copied().unwrap_or(actor.pos));
        let rest = path
            .as_ref()
            .map_or(vec![], |p| p.iter().skip(2).copied().collect());
        require(
            receipt.destination == destination
                && receipt.route_failed == (destination != target && rest.is_empty()),
            "deterministic route/failed receipt",
        )?;
        expected_actor.pos = destination;
        expected_actor.motion_plan = MotionPlan {
            target: Some(target),
            path: rest,
            walked: true,
        };
        let gathered = cells[index(destination)].food;
        require(receipt.gathered == gathered, "actual destination harvest")?;
        cells[index(destination)].food = 0.0;
        task.gross += gathered;
        let failed = receipt.route_failed
            || (selected.is_some() && destination == target && gathered <= 0.0);
        if let Some(site) = task.target {
            if failed {
                task.failed_until.insert(site, t + 3);
                task.target = None
            } else if destination == target {
                task.target = None
            }
        }
        if destination == target {
            if let Some(plan) = task.task_plan.as_mut() {
                if plan.steps.first().is_some_and(|s| s.0 == target) {
                    plan.steps.remove(0);
                }
            }
            if let Some(plan) = legacy.as_mut() {
                if plan.steps.first().is_some_and(|s| s.0 == target) {
                    plan.steps.remove(0);
                }
            }
        }
        if task.gross >= f64::from(task.quota) {
            task.first_completion.get_or_insert(t);
            task.target = None;
            task.task_plan = None;
            legacy = None;
        }
        if lab.controller == Controller::MatchedFsm {
            task.fsm.phase = if task.first_completion.is_some() {
                FsmPhase::Finished
            } else if !failed && destination != target {
                FsmPhase::Moving
            } else if failed || task.target.is_none() && target == actor.pos {
                FsmPhase::Deferred
            } else {
                FsmPhase::Selecting
            };
        }
        if matches!(
            lab.controller,
            Controller::GuardedTree | Controller::UnguardedTree
        ) {
            task.tree = if task.first_completion.is_none() && !failed && destination != target {
                TreeState {
                    cursors: [(0, 2), (3, 1)].into(),
                    statuses: [
                        (0, Status::Running),
                        (1, Status::Success),
                        (2, Status::Success),
                        (3, Status::Running),
                        (4, Status::Success),
                        (5, Status::Running),
                    ]
                    .into(),
                    running_leaves: [5].into(),
                    deferred: None,
                }
            } else {
                TreeState::default()
            };
        }
    }
    require(
        after.task == task,
        "task/phase/continuation/cooldown settlement",
    )?;
    require(
        after.rng_state_json == rng::state_json(&expected_rng),
        "action RNG differs from exact policy draw",
    )?;
    expected_actor.holdings += receipt.gathered;
    let consumed = expected_actor.holdings.min(1.0);
    expected_actor.holdings -= 1.0;
    update_memory(&mut expected_actor, &cells, t - 1);
    expected_actor.age += 1;
    let alive = expected_actor.holdings > 0.0;
    let loss = if alive {
        0.0
    } else {
        expected_actor.holdings.max(0.0)
    };
    require(
        after.actor == alive.then_some(expected_actor),
        "post-action actor/traits/memory/motion/metabolism/death",
    )?;
    require(
        after.cells == cells
            && after.consumed == before.consumed + consumed
            && after.death_loss == before.death_loss + loss,
        "physical food accounting",
    )?;
    require(
        after.legacy_plan == if alive { legacy } else { None },
        "legacy plan continuation",
    )
}
fn select(
    o: &Observation,
    a: &Actor,
    cells: &[Cell],
    task: &mut TaskState,
    legacy: &mut Option<LegacyPlan>,
    controller: Controller,
    rng: &mut rng::SimRng,
) -> Result<Option<Pos>, String> {
    use bt::verification;
    match controller {
        Controller::GuardedTree | Controller::UnguardedTree | Controller::MatchedFsm => {
            let guarded = controller != Controller::UnguardedTree;
            if task
                .target
                .is_some_and(|site| !verification::allowed(o, task, site, guarded))
            {
                task.target = None
            }
            if task.target.is_none() {
                task.target = verification::select_target(o, task, rng)
            }
            Ok(task.target.map(|s| torus().pos(s as usize)))
        }
        Controller::ReactiveUtility => {
            let scored: Vec<_> = o
                .candidates
                .iter()
                .map(|c| (c.pos, c.distance, c.value / (1.0 + f64::from(c.distance))))
                .collect();
            Ok(Some(verification::choose(&scored, rng)?))
        }
        Controller::TaskGoap | Controller::LegacyGoap => {
            let failed = a
                .motion_plan
                .target
                .filter(|t| a.motion_plan.path.is_empty() && *t != a.pos);
            let candidates: Vec<_> = o
                .candidates
                .iter()
                .enumerate()
                .filter(|(i, c)| {
                    *i == 0 || (Some(c.pos) != failed && route(cells, a.pos, c.pos).is_some())
                })
                .map(|(_, c)| c)
                .collect();
            let steps = if controller == Controller::TaskGoap {
                task.task_plan.as_ref().map(|p| &p.steps)
            } else {
                legacy.as_ref().map(|p| &p.steps)
            };
            let retained = steps
                .and_then(|s| s.first())
                .filter(|(p, v)| {
                    candidates
                        .iter()
                        .any(|c| c.pos == *p && (c.remembered || c.value >= v / 2.0))
                })
                .map(|s| s.0);
            let target = if let Some(target) = retained {
                target
            } else {
                task.task_plan = None;
                *legacy = None;
                let mut others: Vec<_> = candidates.iter().skip(1).copied().collect();
                others.sort_by(|a, b| {
                    (b.value / (1.0 + f64::from(b.distance)))
                        .total_cmp(&(a.value / (1.0 + f64::from(a.distance))))
                        .then(a.distance.cmp(&b.distance))
                        .then(a.site.cmp(&b.site))
                });
                others.truncate(8);
                let sites: Vec<_> = std::iter::once(candidates[0])
                    .chain(others)
                    .map(|c| (c.pos, c.value))
                    .collect();
                let goal = if controller == Controller::TaskGoap {
                    (f64::from(task.quota) - task.gross).max(0.0)
                } else {
                    f64::from(task.quota)
                };
                let found = verification::plan(&sites, goal)?;
                if let Some(found) = found {
                    let steps = found.steps;
                    let target = steps.first().map_or(a.pos, |s| s.0);
                    if controller == Controller::TaskGoap {
                        task.task_plan = Some(TaskPlan { goal, steps })
                    } else {
                        *legacy = Some(LegacyPlan {
                            goal,
                            gathers: steps.iter().map(|s| s.1).sum(),
                            steps,
                        })
                    }
                    target
                } else {
                    let best = candidates
                        .iter()
                        .map(|c| c.value / (1.0 + f64::from(c.distance)))
                        .fold(f64::NEG_INFINITY, f64::max);
                    let top: Vec<_> = candidates
                        .iter()
                        .filter(|c| c.value / (1.0 + f64::from(c.distance)) == best)
                        .map(|c| (c.pos, c.distance, c.value))
                        .collect();
                    verification::choose(&top, rng)?
                }
            };
            if controller == Controller::TaskGoap {
                task.target = Some(index(target) as u32)
            }
            Ok(Some(target))
        }
    }
}
