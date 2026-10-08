//! Sampled worker state, physical categories and aggregate diagnostics.
use super::{
    access::{distances, neighbors},
    physical::{in_bounds, require, sum},
    wire::{WirePos, WireSetup},
    wire_state::{
        WireCargo, WireComputeCounts, WireFoodPhase, WireMode, WireSpoilState, WireWorkCounts,
    },
    wire_view::{WireAgentView, WireSnapshot},
};
use std::collections::{BTreeMap, BTreeSet};

fn work(w: &WireWorkCounts) -> Result<(), String> {
    require(
        sum(
            [w.moves, w.digs, w.pickups, w.deposits, w.disposals, w.waits],
            "work.opportunities",
        )? == w.opportunities,
        "work.opportunities",
    )?;
    require(
        sum(
            [
                w.departure_moves,
                w.search_moves,
                w.empty_return_moves,
                w.food_moves,
                w.spoil_moves,
            ],
            "work.moves",
        )? == w.moves,
        "work.moves",
    )?;
    require(
        sum(
            [
                w.transition_waits,
                w.empty_arrival_waits,
                w.no_neighbor_waits,
                w.empty_congestion_waits,
                w.food_congestion_waits,
                w.spoil_congestion_waits,
            ],
            "work.waits",
        )? == w.waits,
        "work.waits",
    )?;
    require(
        w.empty_returns == w.empty_arrival_waits
            && w.spoil_hauls == w.digs
            && w.publications <= w.deposits,
        "work.arrivals",
    )?;
    require(
        sum(
            [
                w.fidelity_departures,
                w.recruited_departures,
                w.uninformed_departures,
            ],
            "work.departures",
        )? == sum([1, w.deposits, w.empty_returns], "work.departures")?,
        "work.departures",
    )?;
    require(
        w.search_entries <= w.opportunities && w.abandoned_targets <= w.opportunities,
        "work.diagnostics",
    )
}
fn initial_agent(setup: &WireSetup, a: &WireAgentView) -> Result<(), String> {
    require(
        a.pos == setup.workers[a.id as usize]
            && a.phase == WireFoodPhase::Departing
            && a.mode == WireMode::Departing
            && a.cargo.is_none()
            && a.find.is_none()
            && a.site.is_none()
            && a.frontier.is_none()
            && a.face.is_none(),
        "initial.agent",
    )?;
    let expected = WireWorkCounts {
        uninformed_departures: 1,
        ..Default::default()
    };
    require(a.work == expected, "initial.work")?;
    let cells: Vec<_> = std::iter::once(a.pos)
        .chain(neighbors(setup, a.pos))
        .collect();
    let open = cells
        .iter()
        .filter(|p| setup.open.binary_search(p).is_ok())
        .count() as u32;
    let diggable = cells
        .iter()
        .filter(|p| setup.open.binary_search(p).is_err() && setup.diggable.binary_search(p).is_ok())
        .count() as u32;
    let expected = WireComputeCounts {
        observations: 1,
        cells_inspected: cells.len() as u64,
        cells_learned: cells.len() as u64,
        ..Default::default()
    };
    require(
        a.compute == expected
            && a.known_open == open
            && a.known_solid == cells.len() as u32 - open
            && a.known_diggable == diggable,
        "initial.observation",
    )
}
pub(super) fn validate_agents(
    setup: &WireSetup,
    f: &WireSnapshot,
    open: &BTreeSet<WirePos>,
) -> Result<(), String> {
    let s = &f.summary;
    let clock = u64::from(s.completed_ticks);
    let size = u64::from(setup.width) * u64::from(setup.height);
    require(
        f.agents.len() == setup.workers.len()
            && s.per_agent_work.len() == f.agents.len()
            && s.per_agent_compute.len() == f.agents.len(),
        "agents.count",
    )?;
    let (reachable, _) = distances(setup, open)?;
    let mut occupancy = BTreeMap::new();
    for (i, a) in f.agents.iter().enumerate() {
        let field = format!("agents[{i}]");
        require(
            a.id as usize == i && in_bounds(setup, a.pos) && reachable.contains_key(&a.pos),
            &format!("{field}.pos"),
        )?;
        let count = occupancy.entry(a.pos).or_insert(0);
        *count += 1;
        require(*count <= 2, "agents.capacity")?;
        require(
            a.work == s.per_agent_work[i] && a.compute == s.per_agent_compute[i],
            &format!("{field}.counters"),
        )?;
        work(&a.work).map_err(|e| format!("{field}.{e}"))?;
        require(
            a.work.opportunities == clock,
            &format!("{field}.work.opportunities"),
        )?;
        let c = &a.compute;
        require(
            c.observations == clock + 1
                && c.cells_inspected >= 3 * (clock + 1)
                && c.cells_inspected <= 5 * (clock + 1),
            &format!("{field}.compute.observations"),
        )?;
        require(
            sum(
                [u64::from(a.known_open), u64::from(a.known_solid)],
                "known counts",
            )? == c.cells_learned
                && c.cells_learned <= size
                && c.cells_learned <= c.cells_inspected
                && a.known_open > 0
                && u64::from(a.known_open) <= open.len() as u64
                && a.known_diggable <= a.known_solid
                && a.known_diggable as usize <= setup.diggable.len(),
            &format!("{field}.known"),
        )?;
        require(
            c.dig_confirmations == a.work.digs
                && sum(
                    [c.observed_revisions, c.dig_confirmations],
                    "compute.revisions",
                )? <= c.cells_learned
                && c.peak_queue <= size
                && c.route_visits
                    <= c.route_calls
                        .checked_mul(size)
                        .ok_or("compute.route_visits overflow")?
                && c.face_scans
                    <= c.observations
                        .checked_mul(size)
                        .ok_or("compute.face_scans overflow")?
                && c.frontier_scans
                    <= c.observations
                        .checked_mul(size)
                        .ok_or("compute.frontier_scans overflow")?,
            &format!("{field}.compute"),
        )?;
        for p in [a.site, a.frontier, a.face].into_iter().flatten() {
            require(in_bounds(setup, p), &format!("{field}.target"))?;
        }
        require(
            !(a.frontier.is_some() && a.face.is_some())
                && a.frontier.is_none_or(|p| open.contains(&p)),
            &format!("{field}.frontier"),
        )?;
        // A remembered dig face can be globally open; only immutable eligibility is observable.
        require(
            a.face
                .is_none_or(|p| setup.diggable.binary_search(&p).is_ok() && a.cargo.is_none()),
            &format!("{field}.face"),
        )?;
        require(
            a.phase != WireFoodPhase::Returning
                || (a.site.is_none() && a.frontier.is_none() && a.face.is_none()),
            &format!("{field}.returning"),
        )?;
        require(
            a.phase != WireFoodPhase::Searching || a.site.is_none(),
            &format!("{field}.searching"),
        )?;
        let mode = match a.cargo {
            Some(WireCargo::Food(id)) => {
                require(
                    a.phase == WireFoodPhase::Returning
                        && a.find.as_ref().is_some_and(|find| {
                            find.count > 0
                                && find.count <= 5
                                && f.food
                                    .iter()
                                    .any(|r| r.resource.id == id && r.resource.pos == find.site)
                        }),
                    &format!("{field}.find"),
                )?;
                WireMode::FoodReturning
            }
            Some(WireCargo::Spoil(_)) => {
                require(
                    a.find.is_none() && a.phase != WireFoodPhase::Returning,
                    &format!("{field}.spoil"),
                )?;
                WireMode::SpoilHauling
            }
            None => {
                require(a.find.is_none(), &format!("{field}.find"))?;
                match a.phase {
                    WireFoodPhase::Departing => WireMode::Departing,
                    WireFoodPhase::Searching => WireMode::Searching,
                    WireFoodPhase::Returning => WireMode::EmptyReturning,
                }
            }
        };
        require(a.mode == mode, &format!("{field}.mode"))?;
        require(
            a.work.pickups.checked_sub(a.work.deposits)
                == Some(u64::from(matches!(a.cargo, Some(WireCargo::Food(_)))))
                && a.work.digs.checked_sub(a.work.disposals)
                    == Some(u64::from(matches!(a.cargo, Some(WireCargo::Spoil(_))))),
            &format!("{field}.cargo.accounting"),
        )?;
        require(
            a.work.digs == f.spoil.iter().filter(|r| r.creator == a.id).count() as u64
                && a.work.disposals
                    == f.spoil
                        .iter()
                        .filter(|r| {
                            r.creator == a.id && matches!(r.state, WireSpoilState::Disposed { .. })
                        })
                        .count() as u64,
            &format!("{field}.spoil.accounting"),
        )?;
        if clock == 0 {
            initial_agent(setup, a)?;
        }
    }
    for (field, total) in s.work.fields() {
        require(
            sum(
                f.agents.iter().map(|a| {
                    a.work
                        .fields()
                        .into_iter()
                        .find(|(n, _)| *n == field)
                        .expect("same work fields")
                        .1
                }),
                &format!("work.{field}"),
            )? == total,
            &format!("work.{field}"),
        )?;
    }
    for (field, total) in s.compute.fields() {
        let values = f.agents.iter().map(|a| {
            a.compute
                .fields()
                .into_iter()
                .find(|(n, _)| *n == field)
                .expect("same compute fields")
                .1
        });
        let expected = if field == "peak_queue" {
            values.max().unwrap_or(0)
        } else {
            sum(values, &format!("compute.{field}"))?
        };
        require(expected == total, &format!("compute.{field}"))?;
    }
    require(
        s.work.opportunities
            == clock
                .checked_mul(f.agents.len() as u64)
                .ok_or("work.opportunities overflow")?,
        "work.opportunities",
    )?;
    Ok(())
}
