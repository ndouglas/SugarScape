//! Researcher-only replay of monotone geometry, without worker decisions or RNG.
use super::{
    physical::{birth_opportunity, in_bounds, require, sum, validate_frames},
    wire::{WirePos, WireSetup},
    wire_state::{WireAccessCompute, WireSpoilState},
    wire_view::{WireEventContext, WireEventMilestone, WireFoodAccessRecord, WireSnapshot},
};
use std::collections::{BTreeMap, BTreeSet, VecDeque};
#[derive(Clone, Debug, PartialEq, serde::Serialize)]
pub(super) struct AccessPoint {
    pub(super) completed_ticks: u32,
    pub(super) distance: Option<u32>,
    pub(super) per_food: Vec<(u64, Option<u32>)>,
}
pub(super) fn neighbors(setup: &WireSetup, p: WirePos) -> Vec<WirePos> {
    [(0, -1), (0, 1), (1, 0), (-1, 0)]
        .into_iter()
        .filter_map(|(dx, dy)| {
            let x = i64::from(p.x) + dx;
            let y = i64::from(p.y) + dy;
            (x >= 0 && y >= 0 && x < i64::from(setup.width) && y < i64::from(setup.height))
                .then_some(WirePos {
                    x: x as u32,
                    y: y as u32,
                })
        })
        .collect()
}
pub(super) fn distances(
    setup: &WireSetup,
    open: &BTreeSet<WirePos>,
) -> Result<(BTreeMap<WirePos, u32>, WireAccessCompute), String> {
    let mut distances = BTreeMap::new();
    let mut queue = VecDeque::new();
    for &p in &setup.nest {
        require(in_bounds(setup, p) && open.contains(&p), "access.nest")?;
        if distances.insert(p, 0).is_none() {
            queue.push_back(p);
        }
    }
    let mut compute = WireAccessCompute {
        calls: 1,
        visits: 0,
        peak_queue: queue.len() as u64,
    };
    while let Some(p) = queue.pop_front() {
        compute.visits += 1;
        let next = distances[&p] + 1;
        for n in neighbors(setup, p) {
            if open.contains(&n) && !distances.contains_key(&n) {
                distances.insert(n, next);
                queue.push_back(n);
                compute.peak_queue = compute.peak_queue.max(queue.len() as u64);
            }
        }
    }
    Ok((distances, compute))
}
fn context(
    setup: &WireSetup,
    frames: &[WireSnapshot],
    c: &WireEventContext,
    op: u64,
    excavated: u32,
    disposed: u32,
    accessible_before: u32,
) -> Result<(), String> {
    require(
        c.opportunity == op
            && birth_opportunity(setup, c.tick, c.worker)? == op
            && c.excavated == excavated
            && c.spoil_disposed == disposed,
        "access.event.context",
    )?;
    let mut lower = 0;
    let mut upper = setup.food.len() as u32;
    for f in frames {
        let clock = u64::from(f.summary.completed_ticks) * setup.workers.len() as u64;
        if clock < op {
            lower = f.summary.food.delivered;
        } else {
            upper = f.summary.food.delivered;
            break;
        }
    }
    require(
        c.food_delivered >= lower
            && c.food_delivered <= upper
            && c.food_delivered <= accessible_before,
        "access.event.food_delivered.bounds",
    )
}
struct Prefix<'a> {
    setup: &'a WireSetup,
    frames: &'a [WireSnapshot],
    distances: &'a BTreeMap<WirePos, u32>,
    op: u64,
    excavated: u32,
    disposed: u32,
    accessible_before: u32,
}
impl Prefix<'_> {
    fn event(&self, event: &WireEventMilestone, pos: WirePos) -> Result<(), String> {
        context(
            self.setup,
            self.frames,
            &event.context,
            self.op,
            self.excavated,
            self.disposed,
            self.accessible_before,
        )?;
        require(
            event.pos == pos && self.distances.get(&pos) == Some(&event.nest_distance),
            "access.event.position.distance",
        )
    }
}
fn handling_milestones(setup: &WireSetup, frames: &[WireSnapshot]) -> Result<(), String> {
    let mut prior = None;
    for f in frames {
        let s = &f.summary;
        let m = &s.milestones;
        for (field, event, count) in [
            ("first_pickup_tick", m.first_pickup_tick, s.work.pickups),
            (
                "first_delivery_tick",
                m.first_delivery_tick,
                s.work.deposits,
            ),
        ] {
            require(event.is_some() == (count > 0), field)?;
            if let Some(tick) = event {
                require(tick < s.completed_ticks, field)?;
                for before in frames {
                    if before.summary.completed_ticks <= tick {
                        let old = if field == "first_pickup_tick" {
                            before.summary.work.pickups
                        } else {
                            before.summary.work.deposits
                        };
                        require(old == 0, field)?;
                    } else {
                        let after = if field == "first_pickup_tick" {
                            before.summary.work.pickups
                        } else {
                            before.summary.work.deposits
                        };
                        require(after > 0, field)?;
                        break;
                    }
                }
            }
        }
        require(
            m.all_food_delivered_tick.is_some()
                == (!setup.food.is_empty() && s.food.delivered == setup.food.len() as u32),
            "all_food_delivered_tick",
        )?;
        if let Some(t) = m.all_food_delivered_tick {
            require(
                t < s.completed_ticks && m.first_delivery_tick.is_some_and(|first| first <= t),
                "all_food_delivered_tick",
            )?;
            for before in frames {
                if before.summary.completed_ticks <= t {
                    require(
                        before.summary.food.delivered < setup.food.len() as u32,
                        "all_food_delivered_tick.old",
                    )?;
                } else {
                    require(
                        before.summary.food.delivered == setup.food.len() as u32,
                        "all_food_delivered_tick.future",
                    )?;
                    break;
                }
            }
        }
        if let (Some(p), Some(d)) = (m.first_pickup_tick, m.first_delivery_tick) {
            require(p < d, "first_delivery_tick.after_pickup")?;
        }
        if let Some(old) = prior {
            let old: &super::wire_view::WireMilestones = old;
            for (a, b) in [
                (old.first_pickup_tick, m.first_pickup_tick),
                (old.first_delivery_tick, m.first_delivery_tick),
                (old.all_food_delivered_tick, m.all_food_delivered_tick),
            ] {
                require(a.is_none() || a == b, "handling.milestones.frozen")?;
            }
        }
        prior = Some(m);
    }
    Ok(())
}
pub(super) fn validate_access(
    setup: &WireSetup,
    frames: &[WireSnapshot],
) -> Result<Vec<AccessPoint>, String> {
    // Standalone component callers receive the same ledger safety as full episodes.
    validate_frames(setup, frames)?;
    handling_milestones(setup, frames)?;
    let last = frames.last().ok_or("access.frames empty")?;
    require(
        last.summary.access.records.len() == setup.food.len(),
        "access.records.count",
    )?;
    let mut open: BTreeSet<_> = setup.open.iter().copied().collect();
    let (mut d, mut compute) = distances(setup, &open)?;
    let mut expected: Vec<_> = setup
        .food
        .iter()
        .map(|r| {
            let distance = d.get(&r.pos).copied();
            WireFoodAccessRecord {
                id: r.id,
                initially_exposed: open.contains(&r.pos),
                initially_accessible: distance.is_some(),
                first_exposure: None,
                first_access: None,
                accessible: distance.is_some(),
                distance,
            }
        })
        .collect();
    let mut events = Vec::new();
    for r in &last.spoil {
        events.push((birth_opportunity(setup, r.born_tick, r.creator)?, false, r));
        if let WireSpoilState::Disposed { tick } = r.state {
            events.push((birth_opportunity(setup, tick, r.creator)?, true, r));
        }
    }
    events.sort_by_key(|e| e.0);
    require(
        events.windows(2).all(|w| w[0].0 < w[1].0),
        "access.event.slots",
    )?;
    let mut next = 0;
    let mut digs = 0u32;
    let mut disposals = 0u32;
    let mut first_excavation = None;
    let mut first_exposure = None;
    let mut first_access = None;
    let mut first_disposal = None;
    let mut result = Vec::with_capacity(frames.len());
    for f in frames {
        let clock = u64::from(f.summary.completed_ticks) * setup.workers.len() as u64;
        while let Some(&(op, is_disposal, r)) = events.get(next).filter(|e| e.0 <= clock) {
            let accessible_before = expected.iter().filter(|r| r.accessible).count() as u32;
            if is_disposal {
                disposals += 1;
                if first_disposal.is_none() {
                    let event = last
                        .summary
                        .milestones
                        .first_disposal
                        .as_ref()
                        .ok_or("milestones.first_disposal missing")?;
                    Prefix {
                        setup,
                        frames,
                        distances: &d,
                        op,
                        excavated: digs,
                        disposed: disposals,
                        accessible_before,
                    }
                    .event(event, setup.waste)?;
                    require(
                        event.context.worker == r.creator,
                        "milestones.first_disposal.worker",
                    )?;
                    first_disposal = Some(event.clone());
                }
            } else {
                require(
                    r.id == u64::from(digs)
                        && neighbors(setup, r.origin).iter().any(|p| d.contains_key(p))
                        && open.insert(r.origin),
                    "access.opening.connected.order",
                )?;
                digs += 1;
                let (new_d, new_compute) = distances(setup, &open)?;
                d = new_d;
                compute.calls = sum([compute.calls, new_compute.calls], "access.compute.calls")?;
                compute.visits = sum(
                    [compute.visits, new_compute.visits],
                    "access.compute.visits",
                )?;
                compute.peak_queue = compute.peak_queue.max(new_compute.peak_queue);
                let prefix = Prefix {
                    setup,
                    frames,
                    distances: &d,
                    op,
                    excavated: digs,
                    disposed: disposals,
                    accessible_before,
                };
                let mut opening_context = None;
                if first_excavation.is_none() {
                    let event = last
                        .summary
                        .milestones
                        .first_excavation
                        .as_ref()
                        .ok_or("milestones.first_excavation missing")?;
                    prefix.event(event, r.origin)?;
                    first_excavation = Some(event.clone());
                    opening_context = Some(event.context.clone());
                }
                for (index, resource) in setup.food.iter().enumerate() {
                    let record = &mut expected[index];
                    let frozen = &last.summary.access.records[index];
                    let expose = !record.initially_exposed
                        && record.first_exposure.is_none()
                        && open.contains(&resource.pos);
                    let access = !record.initially_accessible
                        && record.first_access.is_none()
                        && d.contains_key(&resource.pos);
                    for (needed, event) in [
                        (expose, &frozen.first_exposure),
                        (access, &frozen.first_access),
                    ] {
                        if needed {
                            let event = event.as_ref().ok_or("access.first event missing")?;
                            prefix.event(event, resource.pos)?;
                            if let Some(prior) = &opening_context {
                                require(*prior == event.context, "access.same_opening.context")?;
                            } else {
                                opening_context = Some(event.context.clone());
                            }
                        }
                    }
                    if expose {
                        record.first_exposure = frozen.first_exposure.clone();
                        if first_exposure.is_none() {
                            first_exposure = record.first_exposure.clone();
                        }
                    }
                    if access {
                        record.first_access = frozen.first_access.clone();
                        if first_access.is_none() {
                            first_access = record.first_access.clone();
                        }
                    }
                    record.distance = d.get(&resource.pos).copied();
                    record.accessible = record.distance.is_some();
                }
            }
            next += 1;
        }
        let validate = || -> Result<(), String> {
            require(
                open == f.open.iter().copied().collect()
                    && digs as usize == f.spoil.len()
                    && disposals == f.summary.spoil.disposed,
                "access.frame.prefix",
            )?;
            let a = &f.summary.access;
            require(
                a.records == expected,
                "access.records.flags.distance.first_events",
            )?;
            require(
                a.compute == compute,
                "access.compute.calls.visits.peak_queue",
            )?;
            require(
                a.initially_exposed
                    == expected.iter().filter(|r| r.initially_exposed).count() as u32
                    && a.initially_accessible
                        == expected.iter().filter(|r| r.initially_accessible).count() as u32
                    && a.accessible == expected.iter().filter(|r| r.accessible).count() as u32,
                "access.inventory",
            )?;
            let m = &f.summary.milestones;
            require(
                m.first_excavation == first_excavation
                    && m.first_exposure == first_exposure
                    && m.first_access == first_access
                    && m.first_disposal == first_disposal,
                "milestones.geometry.first_events",
            )?;
            Ok(())
        };
        validate().map_err(|e| format!("frame[{}].{e}", f.summary.completed_ticks))?;
        let per_food = expected
            .iter()
            .map(|r| (r.id, r.distance))
            .collect::<Vec<_>>();
        let distance = per_food.iter().filter_map(|(_, d)| *d).min();
        result.push(AccessPoint {
            completed_ticks: f.summary.completed_ticks,
            distance,
            per_food,
        });
    }
    require(next == events.len(), "access.events.future")?;
    // The worker identity within a handling tick is absent. Including every opening
    // in that tick gives a conservative geometry bound over any reachable food.
    for (field, tick, delivery) in [
        (
            "first_pickup_tick",
            last.summary.milestones.first_pickup_tick,
            false,
        ),
        (
            "first_delivery_tick",
            last.summary.milestones.first_delivery_tick,
            true,
        ),
    ] {
        if let Some(t) = tick {
            let mut prefix: BTreeSet<_> = setup.open.iter().copied().collect();
            prefix.extend(
                last.spoil
                    .iter()
                    .filter(|r| r.born_tick <= t)
                    .map(|r| r.origin),
            );
            let (at_tick, _) = distances(setup, &prefix)?;
            let distance = setup
                .food
                .iter()
                .filter_map(|r| at_tick.get(&r.pos).copied())
                .min()
                .ok_or_else(|| format!("{field}.access: no food reachable by handling tick"))?;
            let minimum = if delivery {
                u64::from(distance)
                    .checked_mul(2)
                    .and_then(|d| d.checked_add(2))
                    .ok_or("handling distance overflow")?
            } else {
                u64::from(distance)
                    .checked_add(1)
                    .ok_or("handling distance overflow")?
            };
            require(u64::from(t) >= minimum, &format!("{field}.geometry_bound"))?;
        }
    }
    Ok(result)
}
