//! Bounded sampled observations, not complete physical trajectories.
use super::{
    access::AccessPoint, manifest::Condition, wire::WireEpisode, wire_view::WireSummary, Geometry,
    Panel, Regime, RunKey,
};
use serde::Serialize;
#[derive(Clone, Debug, PartialEq, Serialize)]
pub(super) struct ShorteningWindow {
    pub(super) after_completed_tick: u32,
    pub(super) by_completed_tick: u32,
}
#[derive(Clone, Debug, PartialEq, Serialize)]
pub(super) struct EpisodeRow {
    pub(super) key: RunKey,
    pub(super) panel: Panel,
    pub(super) geometry: Geometry,
    pub(super) regime: Regime,
    pub(super) summary: WireSummary,
    pub(super) route: Vec<AccessPoint>,
    pub(super) initial_distance: Option<u32>,
    pub(super) final_distance: Option<u32>,
    pub(super) gain: Option<u32>,
    pub(super) first_shortening: Option<ShorteningWindow>,
    pub(super) sampled_physical_sha256: String,
}

/// Caller supplies an episode validated by the archive reader. Access replay also
/// validates sampled materials; it measures all original food sites after delivery.
pub(super) fn project_row(
    c: &Condition,
    key: &RunKey,
    e: &WireEpisode,
) -> Result<EpisodeRow, String> {
    if key.condition != c.id || key.seed != e.seed {
        return Err("projection condition/seed mismatch".into());
    }
    let route = super::access::validate_access(&e.setup, &e.snapshots)?;
    let initial_distance = route
        .first()
        .ok_or("projection needs initial frame")?
        .distance;
    if route
        .first()
        .ok_or("projection needs initial frame")?
        .per_food
        != super::scenario::patch_distances(&c.setup)?
    {
        return Err("projection initial resource distances differ from condition".into());
    }
    let final_distance = route.last().ok_or("projection needs final frame")?.distance;
    let gain = match (initial_distance, final_distance) {
        (Some(initial), Some(final_value)) => Some(
            initial
                .checked_sub(final_value)
                .ok_or("physical route increased from initial distance")?,
        ),
        (Some(_), None) => return Err("initially connected route became disconnected".into()),
        (None, _) => None,
    };
    let first_shortening = initial_distance.and_then(|initial| shortening_window(initial, &route));
    Ok(EpisodeRow {
        key: key.clone(),
        panel: c.panel,
        geometry: c.geometry,
        regime: c.regime,
        summary: e.summary.clone(),
        route,
        initial_distance,
        final_distance,
        gain,
        first_shortening,
        sampled_physical_sha256: sampled_physical_digest(e)?,
    })
}
fn shortening_window(initial: u32, route: &[AccessPoint]) -> Option<ShorteningWindow> {
    route
        .windows(2)
        .find(|pair| pair[1].distance.is_some_and(|d| d < initial))
        .map(|pair| ShorteningWindow {
            after_completed_tick: pair[0].completed_ticks,
            by_completed_tick: pair[1].completed_ticks,
        })
}
/// Only a sampled physical projection: private beliefs/advice, researcher/worker
/// compute, manifest input, condition and seed are deliberately absent.
fn sampled_physical_digest(e: &WireEpisode) -> Result<String, String> {
    #[derive(Serialize)]
    struct Worker<'a> {
        id: u32,
        pos: &'a super::wire::WirePos,
        phase: &'a super::wire_state::WireFoodPhase,
        mode: &'a super::wire_state::WireMode,
        cargo: &'a Option<super::wire_state::WireCargo>,
        work: &'a super::wire_state::WireWorkCounts,
    }
    #[derive(Serialize)]
    struct Frame<'a> {
        completed_ticks: u32,
        open: &'a [super::wire::WirePos],
        workers: Vec<Worker<'a>>,
        work: &'a super::wire_state::WireWorkCounts,
        food: &'a [super::wire_state::WireFoodView],
        spoil: &'a [super::wire_state::WireSpoilView],
    }
    let frames = e
        .snapshots
        .iter()
        .map(|f| Frame {
            completed_ticks: f.summary.completed_ticks,
            open: &f.open,
            workers: f
                .agents
                .iter()
                .map(|a| Worker {
                    id: a.id,
                    pos: &a.pos,
                    phase: &a.phase,
                    mode: &a.mode,
                    cargo: &a.cargo,
                    work: &a.work,
                })
                .collect(),
            work: &f.summary.work,
            food: &f.food,
            spoil: &f.spoil,
        })
        .collect::<Vec<_>>();
    serde_json::to_vec(&frames)
        .map(|bytes| super::sha256(&bytes))
        .map_err(|e| format!("sampled physical projection: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn first_shortening_uses_previous_completed_checkpoint() {
        let route = [48, 48, 30, 20, 15]
            .into_iter()
            .enumerate()
            .map(|(i, d)| AccessPoint {
                completed_ticks: i as u32 * 128,
                distance: Some(d),
                per_food: vec![],
            })
            .collect::<Vec<_>>();
        assert_eq!(
            shortening_window(48, &route),
            Some(ShorteningWindow {
                after_completed_tick: 128,
                by_completed_tick: 256,
            })
        );
        assert_eq!(shortening_window(15, &route), None);
    }
    #[test]
    fn equal_sampled_projections_exclude_advice_inputs_and_computation() {
        let e = super::super::tests::support::decode_fixture("route.straight.protected", 7)
            .unwrap()
            .episode;
        let mut alias = e.clone();
        alias.seed = 8;
        alias.setup.parameters.p_search = super::super::wire::WireFloat("0.5".into());
        for f in &mut alias.snapshots {
            f.summary.compute.observations += 1;
            f.summary.access.compute.calls += 1;
            f.waypoints.clear();
            for worker in &mut f.agents {
                worker.compute.observations += 1;
                worker.known_open += 1;
                worker.find = None;
                worker.site = None;
            }
        }
        assert_ne!(e, alias);
        assert_eq!(
            sampled_physical_digest(&e).unwrap(),
            sampled_physical_digest(&alias).unwrap()
        );
        alias.snapshots[0].agents[0].pos.x += 1;
        assert_ne!(
            sampled_physical_digest(&e).unwrap(),
            sampled_physical_digest(&alias).unwrap()
        );
    }
}
