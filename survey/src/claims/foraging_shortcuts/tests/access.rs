use super::super::{
    access::validate_access,
    manifest::{candidate, condition},
    validate::validate_episode,
    wire::WireEpisode,
};
use super::support::*;
#[test]
fn sealed_baseline_is_null_after_food_remains_available() {
    let m = candidate().unwrap();
    let c = condition(&m, "access.straight.protected").unwrap();
    let e = decode_fixture(&c.id, 7).unwrap();
    validate_episode(c, &e.key, &e.episode).unwrap();
    assert!(validate_access(&e.episode.setup, &e.episode.snapshots)
        .unwrap()
        .iter()
        .all(|p| p.distance.is_none()));
}
#[test]
fn hidden_food_exposure_and_access_are_checked_on_opening_prefix() {
    let e: WireEpisode =
        serde_json::from_slice(&serde_json::to_vec(&component_episode()).unwrap()).unwrap();
    validate_access(&e.setup, &e.snapshots).unwrap();
    assert!(e.summary.access.records[0].first_exposure.is_some());
}

fn component() -> WireEpisode {
    serde_json::from_slice(&serde_json::to_vec(&component_episode()).unwrap()).unwrap()
}
#[test]
fn current_cache_and_frozen_opening_distance_are_checked_independently() {
    let valid = component();
    for index in 0..6 {
        let mut e = valid.clone();
        let f = &mut e.snapshots[1];
        match index {
            0 => f.summary.access.records[0].distance = Some(0),
            1 => {
                f.summary.access.records.pop();
            }
            2 => {
                f.summary.access.records[0]
                    .first_access
                    .as_mut()
                    .unwrap()
                    .nest_distance += 1
            }
            3 => f.summary.access.records[0].first_exposure = None,
            4 => f.summary.access.compute.calls += 1,
            _ => f.summary.access.compute.peak_queue += 1,
        }
        assert!(
            validate_access(&e.setup, &e.snapshots).is_err(),
            "mutation {index}"
        );
    }
}
#[test]
fn future_old_and_inconsistent_contexts_are_rejected() {
    let valid = component();
    for index in 0..5 {
        let mut e = valid.clone();
        let event = e.snapshots[1].summary.access.records[0]
            .first_access
            .as_mut()
            .unwrap();
        match index {
            0 => event.context.tick = 4,
            1 => event.context.opportunity = 0,
            2 => event.context.worker = 1,
            3 => event.context.food_delivered = 1,
            _ => event.context.spoil_disposed = 1,
        }
        assert!(
            validate_access(&e.setup, &e.snapshots).is_err(),
            "mutation {index}"
        );
    }
}
#[test]
fn delivered_food_retains_original_physical_route() {
    let e = component();
    let points = validate_access(&e.setup, &e.snapshots).unwrap();
    assert_eq!(points.last().unwrap().per_food, vec![(0, Some(1))]);
}
#[test]
fn corrected_review_disconnected_food_cannot_have_handling_milestones() {
    let e = disconnected_handling_fixture();
    assert!(validate_access(&e.setup, &e.snapshots).is_err());
}
#[test]
fn corrected_review_pickup_cannot_precede_any_food_access() {
    let mut e = component();
    validate_access(&e.setup, &e.snapshots).unwrap();
    let mut final_frame = e.snapshots.last().unwrap().clone();
    final_frame.spoil[0].born_tick = 10;
    final_frame.spoil[0].state = super::super::wire_state::WireSpoilState::Disposed { tick: 13 };
    let m = &mut final_frame.summary.milestones;
    for event in [
        &mut m.first_excavation,
        &mut m.first_exposure,
        &mut m.first_access,
    ] {
        let c = &mut event.as_mut().unwrap().context;
        c.tick = 10;
        c.opportunity = 11;
    }
    let c = &mut m.first_disposal.as_mut().unwrap().context;
    c.tick = 13;
    c.opportunity = 14;
    m.first_pickup_tick = Some(9);
    m.first_delivery_tick = Some(15);
    m.all_food_delivered_tick = Some(15);
    let r = &mut final_frame.summary.access.records[0];
    for event in [&mut r.first_exposure, &mut r.first_access] {
        let c = &mut event.as_mut().unwrap().context;
        c.tick = 10;
        c.opportunity = 11;
    }
    e.snapshots = vec![e.snapshots[0].clone(), final_frame];
    assert!(validate_access(&e.setup, &e.snapshots).is_err());
}
#[test]
fn access_points_serialize_for_saved_reporting() {
    let e = component();
    let points = validate_access(&e.setup, &e.snapshots).unwrap();
    assert_eq!(
        serde_json::to_value(points.last().unwrap()).unwrap(),
        serde_json::json!({"completed_ticks":16,"distance":1,"per_food":[[0,1]]})
    );
}
