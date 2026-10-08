//! Constraints on observations, never reconstruction of private maps or policy.
use super::{
    access::{distances, neighbors},
    wire::{WirePos, WireSetup},
    wire_state::{WireCargo, WireFoodState, WireSpoilState},
    wire_view::WireSnapshot,
};
use std::collections::BTreeSet;

pub(super) fn require(ok: bool, field: &str) -> Result<(), String> {
    if ok {
        Ok(())
    } else {
        Err(format!("{field}: observed constraints disagree"))
    }
}
pub(super) fn sum(values: impl IntoIterator<Item = u64>, field: &str) -> Result<u64, String> {
    values.into_iter().try_fold(0u64, |a, b| {
        a.checked_add(b)
            .ok_or_else(|| format!("{field}: counter overflow"))
    })
}
pub(super) fn in_bounds(setup: &WireSetup, p: WirePos) -> bool {
    p.x < setup.width && p.y < setup.height
}
fn sorted<T: Ord>(values: &[T]) -> bool {
    values.windows(2).all(|w| w[0] < w[1])
}
pub(super) fn validate_setup(setup: &WireSetup) -> Result<(), String> {
    require(
        (3..=125).contains(&setup.width) && (3..=125).contains(&setup.height),
        "setup.dimensions",
    )?;
    require(
        (1..=256).contains(&setup.workers.len()) && setup.food.len() <= 256,
        "setup.population",
    )?;
    for (field, positions) in [
        ("setup.open", &setup.open),
        ("setup.nest", &setup.nest),
        ("setup.diggable", &setup.diggable),
    ] {
        require(
            sorted(positions) && positions.iter().all(|p| in_bounds(setup, *p)),
            field,
        )?;
    }
    require(
        !setup.open.is_empty()
            && setup.nest.len() >= 2
            && setup
                .nest
                .iter()
                .all(|p| setup.open.binary_search(p).is_ok()),
        "setup.nest",
    )?;
    require(
        in_bounds(setup, setup.waste)
            && setup.open.binary_search(&setup.waste).is_ok()
            && setup.nest.binary_search(&setup.waste).is_err(),
        "setup.waste",
    )?;
    require(
        setup
            .workers
            .iter()
            .all(|p| setup.nest.binary_search(p).is_ok()),
        "setup.workers",
    )?;
    let nest: BTreeSet<_> = setup.nest.iter().copied().collect();
    let (d, _) = distances(setup, &nest)?;
    require(
        setup.nest.iter().all(|p| d.contains_key(p)),
        "setup.nest.connected",
    )?;
    // Multi-source distance alone cannot prove that nest sources connect to each other.
    let mut connected = BTreeSet::from([setup.nest[0]]);
    let mut queue = vec![setup.nest[0]];
    while let Some(p) = queue.pop() {
        for n in neighbors(setup, p) {
            if nest.contains(&n) && connected.insert(n) {
                queue.push(n);
            }
        }
    }
    require(connected == nest, "setup.nest.connected")?;
    require(
        sorted(&setup.food.iter().map(|f| f.id).collect::<Vec<_>>()),
        "setup.food.ids",
    )?;
    let mut food_cells = BTreeSet::new();
    for f in &setup.food {
        require(
            in_bounds(setup, f.pos)
                && !nest.contains(&f.pos)
                && f.pos != setup.waste
                && food_cells.insert(f.pos),
            "setup.food.pos",
        )?;
    }
    let initial: BTreeSet<_> = setup.open.iter().copied().collect();
    let (d, _) = distances(setup, &initial)?;
    require(d.contains_key(&setup.waste), "setup.waste.connected")?;
    for f in [
        &setup.parameters.p_search,
        &setup.parameters.p_return,
        &setup.parameters.lambda_fidelity,
        &setup.parameters.lambda_publish,
        &setup.parameters.lambda_waypoint,
    ] {
        let n = serde_json::from_str::<f64>(&f.0).map_err(|e| e.to_string())?;
        require(n.is_finite() && n >= 0.0, "setup.parameters")?;
    }
    for f in [&setup.parameters.p_search, &setup.parameters.p_return] {
        require(
            serde_json::from_str::<f64>(&f.0).map_err(|e| e.to_string())? <= 1.0,
            "setup.parameters.probability",
        )?;
    }
    Ok(())
}
pub(super) fn birth_opportunity(setup: &WireSetup, tick: u32, worker: u32) -> Result<u64, String> {
    require((worker as usize) < setup.workers.len(), "event.worker")?;
    u64::from(tick)
        .checked_mul(setup.workers.len() as u64)
        .and_then(|n| n.checked_add(u64::from(worker) + 1))
        .ok_or_else(|| "event.opportunity overflow".into())
}
fn materials(setup: &WireSetup, f: &WireSnapshot, open: &BTreeSet<WirePos>) -> Result<(), String> {
    let s = &f.summary;
    require(
        f.food.len() == setup.food.len()
            && f.spoil.len() <= setup.width as usize * setup.height as usize,
        "material.counts",
    )?;
    let (reachable, _) = distances(setup, open)?;
    let mut food = [0u32; 4];
    let mut food_cargo = BTreeSet::new();
    for (r, expected) in f.food.iter().zip(&setup.food) {
        require(r.resource == *expected, "food.resource")?;
        require(
            !matches!(
                r.state,
                WireFoodState::Carried { .. } | WireFoodState::Delivered
            ) || reachable.contains_key(&r.resource.pos),
            "food.handled.reachable",
        )?;
        match r.state {
            WireFoodState::Hidden => {
                food[0] += 1;
                require(!open.contains(&r.resource.pos), "food.hidden")?;
            }
            WireFoodState::Available => {
                food[1] += 1;
                require(open.contains(&r.resource.pos), "food.available")?;
            }
            WireFoodState::Carried { agent } => {
                food[2] += 1;
                require(
                    open.contains(&r.resource.pos)
                        && f.agents
                            .get(agent as usize)
                            .is_some_and(|a| a.cargo == Some(WireCargo::Food(r.resource.id)))
                        && food_cargo.insert(agent),
                    "food.carried",
                )?;
            }
            WireFoodState::Delivered => {
                food[3] += 1;
                require(open.contains(&r.resource.pos), "food.delivered")?;
            }
        }
    }
    require(
        s.food.initial == setup.food.len() as u32
            && [
                s.food.hidden,
                s.food.available,
                s.food.carried,
                s.food.delivered,
            ] == food,
        "food.inventory",
    )?;
    let initial: BTreeSet<_> = setup.open.iter().copied().collect();
    let mut origins = BTreeSet::new();
    let mut spoil_cargo = BTreeSet::new();
    let mut slots = BTreeSet::new();
    let mut disposed = 0;
    for (i, r) in f.spoil.iter().enumerate() {
        require(
            r.id == i as u64
                && in_bounds(setup, r.origin)
                && setup.diggable.binary_search(&r.origin).is_ok()
                && !initial.contains(&r.origin)
                && origins.insert(r.origin)
                && r.born_tick < s.completed_ticks,
            "spoil.origin.birth.id",
        )?;
        let born = birth_opportunity(setup, r.born_tick, r.creator)?;
        require(slots.insert(born), "spoil.birth.slot")?;
        match r.state {
            WireSpoilState::Carried { agent } => {
                require(
                    agent == r.creator
                        && f.agents
                            .get(agent as usize)
                            .is_some_and(|a| a.cargo == Some(WireCargo::Spoil(r.id)))
                        && spoil_cargo.insert(agent),
                    "spoil.carried",
                )?;
                let carrier = &f.agents[agent as usize];
                let distance = sum(
                    [
                        u64::from(r.origin.x.abs_diff(carrier.pos.x)),
                        u64::from(r.origin.y.abs_diff(carrier.pos.y)),
                    ],
                    "spoil.carried.distance",
                )?;
                // Dig commits from an adjacent cell without moving its creator.
                let minimum_moves = distance.checked_sub(1).unwrap_or(1);
                let available = s
                    .completed_ticks
                    .checked_sub(r.born_tick)
                    .and_then(|ticks| ticks.checked_sub(1))
                    .ok_or("spoil.carried.birth clock underflow")?;
                require(
                    minimum_moves <= u64::from(available),
                    "spoil.carried.birth.travel_bound",
                )?;
            }
            WireSpoilState::Disposed { tick } => {
                disposed += 1;
                let disposal = birth_opportunity(setup, tick, r.creator)?;
                require(
                    tick < s.completed_ticks
                        && tick > r.born_tick
                        && tick - r.born_tick
                            >= r.origin.x.abs_diff(setup.waste.x)
                                + r.origin.y.abs_diff(setup.waste.y)
                        && slots.insert(disposal),
                    "spoil.disposal.tick.slot",
                )?;
            }
        }
    }
    for a in &f.agents {
        match a.cargo {
            Some(WireCargo::Food(_)) => {
                require(food_cargo.contains(&a.id), "cargo.food.bijection")?
            }
            Some(WireCargo::Spoil(_)) => {
                require(spoil_cargo.contains(&a.id), "cargo.spoil.bijection")?
            }
            None => {}
        }
    }
    let excavated = f.spoil.len() as u32;
    require(
        s.spoil.excavated == excavated
            && s.spoil.disposed == disposed
            && s.spoil.carried == excavated - disposed,
        "spoil.inventory",
    )?;
    require(
        s.terrain.initial_open == setup.open.len() as u32
            && s.terrain.excavated == excavated
            && s.terrain.open == open.len() as u32
            && open == &initial.union(&origins).copied().collect(),
        "terrain.inventory.open",
    )?;
    require(
        s.work.digs == u64::from(excavated)
            && s.work.disposals == u64::from(disposed)
            && s.work.pickups == sum([u64::from(food[2]), u64::from(food[3])], "work.pickups")?
            && s.work.deposits == u64::from(food[3])
            && s.work.publications <= s.work.deposits,
        "work.material",
    )?;
    require(
        sum(
            [f.waypoints.len() as u64, s.expired_records],
            "work.publications",
        )? == s.work.publications,
        "work.publications",
    )?;
    let rate = setup
        .parameters
        .lambda_waypoint
        .0
        .parse::<f64>()
        .map_err(|e| e.to_string())?;
    require(
        sorted(&f.waypoints.iter().map(|w| w.id).collect::<Vec<_>>()),
        "waypoints.ids",
    )?;
    for w in &f.waypoints {
        require(
            w.id < s.work.publications
                && in_bounds(setup, w.site)
                && open.contains(&w.site)
                && w.created_tick < s.completed_ticks,
            "waypoints.site.tick",
        )?;
        let strength = sugarscape_core::foraging::waypoint_strength(
            rate,
            f64::from(s.completed_ticks - w.created_tick),
        )
        .map_err(|e| format!("waypoints.strength: {e:?}"))?;
        require(
            w.strength.0 == serde_json::to_string(&strength).map_err(|e| e.to_string())?,
            "waypoints.strength",
        )?;
    }
    if s.completed_ticks == 0 {
        require(
            f.spoil.is_empty()
                && f.waypoints.is_empty()
                && s.expired_records == 0
                && f.food.iter().all(|r| {
                    r.state
                        == if initial.contains(&r.resource.pos) {
                            WireFoodState::Available
                        } else {
                            WireFoodState::Hidden
                        }
                }),
            "initial.material",
        )?;
    }
    Ok(())
}
fn cumulative(setup: &WireSetup, old: &WireSnapshot, new: &WireSnapshot) -> Result<(), String> {
    let start = old.summary.completed_ticks;
    require(start < new.summary.completed_ticks, "frames.clock")?;
    for (before, after) in old.agents.iter().zip(&new.agents) {
        require(
            before.known_open <= after.known_open,
            "agents.known_open.monotone",
        )?;
        for ((field, a), (_, b)) in before.work.fields().into_iter().zip(after.work.fields()) {
            require(
                a <= b,
                &format!("agents[{}].work.{field}.monotone", before.id),
            )?;
        }
        for ((field, a), (_, b)) in before
            .compute
            .fields()
            .into_iter()
            .zip(after.compute.fields())
        {
            require(
                a <= b,
                &format!("agents[{}].compute.{field}.monotone", before.id),
            )?;
        }
        let revised = sum(
            [
                after.compute.observed_revisions - before.compute.observed_revisions,
                after.compute.dig_confirmations - before.compute.dig_confirmations,
            ],
            "agents.known_open.revisions",
        )?;
        require(
            u64::from(after.known_open - before.known_open) >= revised,
            "agents.known_open.revisions",
        )?;
        let moves = after.work.moves - before.work.moves;
        let displacement = u64::from(before.pos.x.abs_diff(after.pos.x))
            + u64::from(before.pos.y.abs_diff(after.pos.y));
        require(
            moves >= displacement && (moves - displacement) % 2 == 0,
            "agents.moves.displacement.parity",
        )?;
        if before.cargo.is_some() && before.cargo == after.cargo {
            require(
                before.find == after.find && before.phase == after.phase,
                "agents.persistent_cargo",
            )?;
            require(
                before.work.digs == after.work.digs
                    && before.work.pickups == after.work.pickups
                    && before.work.deposits == after.work.deposits
                    && before.work.disposals == after.work.disposals,
                "agents.persistent_cargo.handling",
            )?;
            let carried_moves = match before.cargo {
                Some(WireCargo::Food(_)) => after.work.food_moves - before.work.food_moves,
                Some(WireCargo::Spoil(_)) => after.work.spoil_moves - before.work.spoil_moves,
                None => 0,
            };
            require(moves == carried_moves, "agents.persistent_cargo.moves")?;
        }
    }
    for (before, after) in old.food.iter().zip(&new.food) {
        let rank = |s: &WireFoodState| match s {
            WireFoodState::Hidden => 0,
            WireFoodState::Available => 1,
            WireFoodState::Carried { .. } => 2,
            WireFoodState::Delivered => 3,
        };
        require(
            rank(&before.state) <= rank(&after.state)
                && (!matches!(before.state, WireFoodState::Carried { .. })
                    || !matches!(after.state, WireFoodState::Carried { .. })
                    || before.state == after.state),
            "food.state.monotone",
        )?;
    }
    require(new.spoil.len() >= old.spoil.len(), "spoil.retained")?;
    for (i, r) in new.spoil.iter().enumerate() {
        if let Some(prior) = old.spoil.get(i) {
            require(
                r.id == prior.id
                    && r.origin == prior.origin
                    && r.creator == prior.creator
                    && r.born_tick == prior.born_tick,
                "spoil.provenance.retained",
            )?;
            match (&prior.state, &r.state) {
                (WireSpoilState::Disposed { .. }, _) => {
                    require(prior.state == r.state, "spoil.disposal.monotone")?
                }
                (WireSpoilState::Carried { .. }, WireSpoilState::Disposed { tick }) => {
                    require(*tick >= start, "spoil.disposal.old_tick")?
                }
                _ => require(prior.state == r.state, "spoil.carrier.retained")?,
            }
        } else {
            require(r.born_tick >= start, "spoil.birth.old_tick")?;
        }
    }
    require(
        old.summary.expired_records <= new.summary.expired_records,
        "waypoints.expired.monotone",
    )?;
    let arrivals = sum(
        [
            new.summary.work.deposits - old.summary.work.deposits,
            new.summary.work.empty_returns - old.summary.work.empty_returns,
        ],
        "waypoints.arrivals",
    )?;
    require(
        new.summary.expired_records == old.summary.expired_records || arrivals > 0,
        "waypoints.expired.arrival",
    )?;
    require(
        new.summary.work.publications - old.summary.work.publications
            <= new.summary.work.deposits - old.summary.work.deposits,
        "waypoints.publications.deposits",
    )?;
    let rate = setup
        .parameters
        .lambda_waypoint
        .0
        .parse::<f64>()
        .map_err(|e| e.to_string())?;
    for prior in &old.waypoints {
        if !new.waypoints.iter().any(|w| w.id == prior.id) {
            let age = new.summary.completed_ticks - prior.created_tick;
            let strength = sugarscape_core::foraging::waypoint_strength(rate, f64::from(age))
                .map_err(|e| format!("waypoints.expired: {e:?}"))?;
            require(
                strength < sugarscape_core::foraging::WAYPOINT_THRESHOLD,
                "waypoints.expired.strength_bound",
            )?;
        }
    }
    for w in &new.waypoints {
        if let Some(prior) = old.waypoints.iter().find(|p| p.id == w.id) {
            require(
                prior.site == w.site && prior.created_tick == w.created_tick,
                "waypoints.retained",
            )?;
        } else {
            require(
                w.id >= old.summary.work.publications && w.created_tick >= start,
                "waypoints.old_record",
            )?;
        }
    }
    Ok(())
}
pub(super) fn validate_frames(setup: &WireSetup, frames: &[WireSnapshot]) -> Result<(), String> {
    validate_setup(setup)?;
    require(
        frames
            .first()
            .is_some_and(|f| f.summary.completed_ticks == 0),
        "frames.initial",
    )?;
    let mut previous = None;
    for f in frames {
        let validate = || -> Result<(), String> {
            require(
                f.summary.completed_ticks <= 7200
                    && (f.summary.completed_ticks as u64) * setup.workers.len() as u64 <= 1_000_000,
                "frames.budget",
            )?;
            require(
                sorted(&f.open) && f.open.iter().all(|p| in_bounds(setup, *p)),
                "open.sorted.bounds",
            )?;
            require(f.nest == setup.nest && f.waste == setup.waste, "nest.waste")?;
            let open = f.open.iter().copied().collect();
            materials(setup, f, &open)?;
            super::agents::validate_agents(setup, f, &open)?;
            if let Some(old) = previous {
                cumulative(setup, old, f)?;
            }
            Ok(())
        };
        validate().map_err(|e| format!("frame[{}].{e}", f.summary.completed_ticks))?;
        previous = Some(f);
    }
    Ok(())
}
