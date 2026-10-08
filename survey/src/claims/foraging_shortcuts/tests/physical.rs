use super::super::{physical::validate_frames, wire::WireEpisode};
use super::support::*;
#[test]
fn valid_component_frames_pass() {
    let e: WireEpisode =
        serde_json::from_slice(&serde_json::to_vec(&component_episode()).unwrap()).unwrap();
    validate_frames(&e.setup, &e.snapshots).unwrap();
}
#[test]
fn cargo_undercount_is_rejected() {
    let mut e = decode_fixture("route.straight.protected", 7).unwrap();
    e.episode.snapshots.last_mut().unwrap().summary.food.carried += 1;
    assert!(validate_frames(&e.episode.setup, &e.episode.snapshots).is_err());
}

fn component() -> WireEpisode {
    serde_json::from_slice(&serde_json::to_vec(&component_episode()).unwrap()).unwrap()
}
#[test]
fn malformed_material_records_and_cargo_are_rejected() {
    let valid = component();
    validate_frames(&valid.setup, &valid.snapshots).unwrap();
    for index in 0..12 {
        let mut e = valid.clone();
        let frame = &mut e.snapshots[1];
        match index {
            0 => frame.spoil[0].creator = 1,
            1 => frame.spoil[0].id = 1,
            2 => frame.spoil[0].origin = e.setup.waste,
            3 => frame.spoil[0].born_tick = frame.summary.completed_ticks,
            4 => frame.food[0].resource.id = 1,
            5 => {
                frame.food[0].state = super::super::wire_state::WireFoodState::Carried { agent: 1 }
            }
            6 => frame.agents[0].cargo = Some(super::super::wire_state::WireCargo::Food(0)),
            7 => frame.summary.spoil.carried += 1,
            8 => frame.open.pop().map(|_| ()).unwrap(),
            9 => frame.agents[0].pos = super::super::wire::WirePos { x: 2, y: 2 },
            10 => frame.summary.work.publications = u64::MAX,
            _ => frame.agents[0].compute.dig_confirmations = u64::MAX,
        }
        assert!(
            validate_frames(&e.setup, &e.snapshots).is_err(),
            "mutation {index}"
        );
    }
}
#[test]
fn carried_tag_bijection_does_not_merge_equal_numeric_namespaces() {
    let mut e = component();
    e.snapshots[1].agents[0].cargo = Some(super::super::wire_state::WireCargo::Food(0));
    assert!(validate_frames(&e.setup, &e.snapshots).is_err());
}
#[test]
fn material_states_and_provenance_cannot_reverse() {
    let valid = component();
    for index in 0..5 {
        let mut e = valid.clone();
        let f = e.snapshots.last_mut().unwrap();
        match index {
            0 => f.food[0].state = super::super::wire_state::WireFoodState::Available,
            1 => f.spoil[0].state = super::super::wire_state::WireSpoilState::Carried { agent: 0 },
            2 => f.spoil[0].born_tick += 1,
            3 => f.spoil[0].origin = e.setup.waste,
            _ => f.agents[0].work.moves = 0,
        }
        assert!(
            validate_frames(&e.setup, &e.snapshots).is_err(),
            "mutation {index}"
        );
    }
}
#[test]
fn computation_peak_aggregates_by_maximum_and_overflow_is_rejected() {
    let mut e = decode_fixture("route.straight.protected", 7).unwrap();
    e.episode.snapshots[1].summary.compute.peak_queue += 1;
    assert!(validate_frames(&e.episode.setup, &e.episode.snapshots).is_err());
    let mut e = component();
    e.snapshots[1].agents[0].work.moves = u64::MAX;
    e.snapshots[1].agents[0].work.digs = 1;
    assert!(validate_frames(&e.setup, &e.snapshots).is_err());
}
#[test]
fn remembered_face_on_opened_masked_cell_is_accepted() {
    let mut e = component();
    let last = e.snapshots.last_mut().unwrap();
    assert!(last.agents[0].cargo.is_none());
    last.agents[0].frontier = None;
    last.agents[0].face = Some(last.spoil[0].origin);
    validate_frames(&e.setup, &e.snapshots).unwrap();
}
#[test]
fn self_review_known_open_classifications_cannot_be_forgotten() {
    let mut e = component();
    let last = e.snapshots.last_mut().unwrap();
    last.agents[0].known_open -= 1;
    last.agents[0].known_solid += 1;
    assert!(validate_frames(&e.setup, &e.snapshots).is_err());
}
#[test]
fn self_review_retained_advice_cannot_expire_above_the_threshold() {
    let mut e = decode_fixture("route.straight.protected", 7).unwrap();
    let last = e.episode.snapshots.last_mut().unwrap();
    assert!(last.waypoints.pop().is_some());
    last.summary.expired_records += 1;
    assert!(validate_frames(&e.episode.setup, &e.episode.snapshots).is_err());
}
#[test]
fn self_review_observed_revisions_require_new_known_open_classifications() {
    let mut e = component();
    let last = e.snapshots.last_mut().unwrap();
    last.agents[0].compute.observed_revisions += 1;
    last.summary.compute = last.agents[0].compute.clone();
    last.summary.per_agent_compute[0] = last.agents[0].compute.clone();
    assert!(validate_frames(&e.setup, &e.snapshots).is_err());
}
// Component-only rate/horizon variant; never a candidate archive or estimate.
fn component_with_weak_advice() -> WireEpisode {
    let original = component_episode();
    let mut setup = original.setup;
    setup.parameters.lambda_waypoint = 100.0;
    let longer = sugarscape_core::foraging::construction::run(
        setup,
        7,
        sugarscape_core::foraging::construction::RunOptions {
            ticks: 32,
            sample_every: 4,
            snapshots: true,
        },
    )
    .unwrap();
    serde_json::from_slice(&serde_json::to_vec(&longer).unwrap()).unwrap()
}
#[test]
fn weak_advice_may_remain_until_a_worker_arrives() {
    let e = component_with_weak_advice();
    validate_frames(&e.setup, &e.snapshots).unwrap();
}
#[test]
fn self_review_expiry_requires_an_observed_arrival_opportunity() {
    let mut e = component_with_weak_advice();
    validate_frames(&e.setup, &e.snapshots).unwrap();
    let before = &e.snapshots[e.snapshots.len() - 2];
    let last = e.snapshots.last().unwrap();
    assert_eq!(before.summary.work.deposits, last.summary.work.deposits);
    assert_eq!(
        before.summary.work.empty_returns,
        last.summary.work.empty_returns
    );
    assert_eq!(last.waypoints.len(), 1);
    assert!(
        last.waypoints[0].strength.0.parse::<f64>().unwrap()
            < sugarscape_core::foraging::WAYPOINT_THRESHOLD
    );
    let last = e.snapshots.last_mut().unwrap();
    last.waypoints.clear();
    last.summary.expired_records = 1;
    assert!(validate_frames(&e.setup, &e.snapshots).is_err());
}
#[test]
fn advice_sites_can_be_depleted_and_duplicate() {
    let mut e = decode_fixture("route.straight.protected", 7).unwrap();
    let site = e.episode.setup.food[0].pos;
    for f in &mut e.episode.snapshots {
        for w in &mut f.waypoints {
            w.site = site;
        }
    }
    validate_frames(&e.episode.setup, &e.episode.snapshots).unwrap();
}
#[test]
fn corrected_review_handled_food_requires_nest_reachable_original_cell() {
    let baseline = component();
    validate_frames(&baseline.setup, &baseline.snapshots).unwrap();
    let bad = disconnected_handling_fixture();
    assert!(validate_frames(&bad.setup, &bad.snapshots).is_err());
}
#[test]
fn corrected_review_carried_spoil_must_have_time_to_reach_its_carrier_position() {
    let mut e = component();
    validate_frames(&e.setup, &e.snapshots).unwrap();
    e.snapshots.truncate(2);
    let f = e.snapshots.last_mut().unwrap();
    assert!(matches!(
        f.spoil[0].state,
        super::super::wire_state::WireSpoilState::Carried { .. }
    ));
    f.spoil[0].born_tick = f.summary.completed_ticks - 1;
    assert_ne!(f.agents[0].pos, e.setup.waste);
    f.agents[0].pos = e.setup.waste;
    assert!(validate_frames(&e.setup, &e.snapshots).is_err());
}

#[test]
fn retained_spoil_lifetimes_cannot_overlap_within_a_checkpoint() {
    use super::super::{
        manifest::{candidate, condition},
        validate::validate_episode,
        wire_state::WireSpoilState,
    };
    let mut envelope = decode_fixture("route.straight.paid", 7).unwrap();
    let m = candidate().unwrap();
    let c = condition(&m, &envelope.key.condition).unwrap();
    validate_episode(c, &envelope.key, &envelope.episode).unwrap();
    let e = &mut envelope.episode;
    let frame = &e.snapshots[2];
    assert_eq!((frame.spoil[1].creator, frame.spoil[1].born_tick), (3, 151));
    assert_eq!(frame.spoil[1].state, WireSpoilState::Disposed { tick: 163 });
    assert_eq!(
        (frame.spoil[10].creator, frame.spoil[10].born_tick),
        (3, 195)
    );
    for frame in &mut e.snapshots[2..] {
        frame.spoil[1].state = WireSpoilState::Disposed { tick: 196 };
    }
    e.summary = e.snapshots.last().unwrap().summary.clone();
    e.snapshot_bytes = e
        .snapshots
        .iter()
        .map(|f| serde_json::to_vec(f).unwrap().len() as u64)
        .sum();
    let error = validate_episode(c, &envelope.key, e).unwrap_err();
    assert!(error.contains("spoil.lifetime.overlap"), "{error}");
}

#[test]
fn retained_spoil_lifetimes_include_terminal_carried_records() {
    use super::super::wire_state::WireSpoilState;
    // Component validation permits a coarser initial/final pair; no new simulation.
    let mut e = decode_fixture("route.straight.paid", 7).unwrap().episode;
    e.snapshots = vec![e.snapshots[0].clone(), e.snapshots.last().unwrap().clone()];
    validate_frames(&e.setup, &e.snapshots).unwrap();
    let last = e.snapshots.last_mut().unwrap();
    assert_eq!((last.spoil[19].creator, last.spoil[19].born_tick), (0, 295));
    assert_eq!((last.spoil[30].creator, last.spoil[30].born_tick), (0, 507));
    assert_eq!(last.spoil[30].state, WireSpoilState::Carried { agent: 0 });
    last.spoil[19].state = WireSpoilState::Disposed { tick: 508 };
    let error = validate_frames(&e.setup, &e.snapshots).unwrap_err();
    assert!(error.contains("spoil.lifetime.overlap"), "{error}");
}
