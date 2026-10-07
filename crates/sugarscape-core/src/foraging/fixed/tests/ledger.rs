use super::super::ledger::{Inventory, Ledger, ResourceState};
use super::super::state::{Agent, Phase, WorkCounts};
use super::super::{Pos, Resource};
use super::setup;

fn resource(id: u64, x: u32, y: u32) -> Resource {
    Resource {
        id,
        pos: Pos { x, y },
    }
}
fn agent(id: u32, cargo: Option<u64>) -> Agent {
    Agent {
        id,
        pos: Pos { x: 2, y: 2 },
        heading: 0.0,
        target: Pos { x: 2, y: 2 },
        phase: Phase::Returning,
        informed: false,
        informed_turns: 0,
        delay: 0,
        cargo,
        find: None,
        work: WorkCounts::default(),
    }
}
#[test]
fn maximum_resource_id_is_a_real_resource() {
    let mut s = setup();
    s.resources = vec![resource(u64::MAX, 1, 1)];
    let mut ledger = Ledger::new(&s);
    let (id, find) = ledger.claim(&s, Pos { x: 1, y: 1 }, 0).unwrap().unwrap();
    assert_eq!(id, u64::MAX);
    assert_eq!(find, crate::foraging::FindRecord { site: 6, count: 1 });
    assert_eq!(
        ledger.inventory(),
        Inventory {
            initial: 1,
            available: 0,
            assigned: 1,
            delivered: 0
        }
    );
    ledger.check(&[agent(0, Some(id))]).unwrap();
    ledger.deposit(id, 0).unwrap();
    assert_eq!(
        ledger.inventory(),
        Inventory {
            initial: 1,
            available: 0,
            assigned: 0,
            delivered: 1
        }
    );
    assert!(ledger.deposit(id, 0).is_err());
    ledger.check(&[agent(0, None)]).unwrap();
}
#[test]
fn contention_claims_once_and_missing_cells_observe_nothing() {
    let mut s = setup();
    s.resources = vec![resource(4, 1, 1)];
    let mut ledger = Ledger::new(&s);
    assert!(ledger.claim(&s, Pos { x: 1, y: 1 }, 0).unwrap().is_some());
    assert_eq!(ledger.claim(&s, Pos { x: 1, y: 1 }, 1).unwrap(), None);
    assert_eq!(ledger.claim(&s, Pos { x: 4, y: 4 }, 1).unwrap(), None);
    assert_eq!(
        ledger.views()[0].state,
        ResourceState::Assigned { agent: 0 }
    );
}
#[test]
fn density_is_frozen_and_excludes_assigned_and_delivered_neighbors() {
    let mut s = setup();
    s.resources = vec![
        resource(9, 0, 0),
        resource(2, 1, 0),
        resource(7, 0, 1),
        resource(3, 1, 1),
        resource(6, 4, 4),
    ];
    let mut ledger = Ledger::new(&s);
    let (_, first) = ledger.claim(&s, Pos { x: 0, y: 0 }, 0).unwrap().unwrap();
    assert_eq!(first.count, 4);
    let (_, second) = ledger.claim(&s, Pos { x: 1, y: 0 }, 1).unwrap().unwrap();
    assert_eq!(second.count, 3);
    ledger.deposit(9, 0).unwrap();
    let (_, third) = ledger.claim(&s, Pos { x: 0, y: 1 }, 2).unwrap().unwrap();
    assert_eq!(third.count, 2);
    assert_eq!(first.count, 4);
    assert_eq!(
        ledger
            .views()
            .iter()
            .map(|v| v.resource.id)
            .collect::<Vec<_>>(),
        vec![2, 3, 6, 7, 9]
    );
}
#[test]
fn deposits_require_existing_resource_and_matching_assigned_owner() {
    let mut s = setup();
    s.resources = vec![resource(4, 1, 1)];
    let mut ledger = Ledger::new(&s);
    assert!(ledger.deposit(4, 0).is_err());
    ledger.claim(&s, Pos { x: 1, y: 1 }, 0).unwrap();
    assert!(ledger.deposit(4, 1).is_err());
    assert!(ledger.deposit(5, 0).is_err());
    assert_eq!(ledger.inventory().assigned, 1);
    ledger.deposit(4, 0).unwrap();
}
#[test]
fn one_agent_cannot_claim_multiple_resources() {
    let mut s = setup();
    s.resources = vec![resource(1, 0, 0), resource(2, 1, 0)];
    let mut ledger = Ledger::new(&s);
    ledger.claim(&s, Pos { x: 0, y: 0 }, u32::MAX).unwrap();
    assert!(ledger.claim(&s, Pos { x: 1, y: 0 }, u32::MAX).is_err());
    assert_eq!(
        ledger.inventory(),
        Inventory {
            initial: 2,
            available: 1,
            assigned: 1,
            delivered: 0
        }
    );
    ledger.check(&[agent(u32::MAX, Some(1))]).unwrap();
}
#[test]
fn check_rejects_missing_wrong_duplicate_and_unassigned_cargo() {
    let mut s = setup();
    s.resources = vec![resource(1, 0, 0), resource(2, 1, 0)];
    let mut ledger = Ledger::new(&s);
    ledger.claim(&s, Pos { x: 0, y: 0 }, 0).unwrap();
    for agents in [
        vec![],
        vec![agent(0, None)],
        vec![agent(1, Some(1))],
        vec![agent(0, Some(1)), agent(1, Some(1))],
        vec![agent(0, Some(2))],
        vec![agent(0, Some(99))],
        vec![agent(0, Some(1)), agent(0, None)],
    ] {
        assert!(
            ledger.check(&agents).is_err(),
            "accepted invalid cargo: {agents:?}"
        );
    }
    ledger.check(&[agent(0, Some(1)), agent(1, None)]).unwrap();
    ledger.deposit(1, 0).unwrap();
    assert!(ledger.check(&[agent(0, Some(1))]).is_err());
}
#[test]
fn empty_ledger_conserves_zero_resources() {
    let ledger = Ledger::new(&setup());
    assert_eq!(
        ledger.inventory(),
        Inventory {
            initial: 0,
            available: 0,
            assigned: 0,
            delivered: 0
        }
    );
    ledger.check(&[agent(0, None)]).unwrap();
}
#[test]
fn work_aggregation_adds_every_counter() {
    let other = WorkCounts {
        opportunities: 1,
        waits: 2,
        directed_moves: 3,
        search_moves: 4,
        pickups: 5,
        deliveries: 6,
        empty_returns: 7,
        search_switches: 8,
        publications: 9,
        fidelity_departures: 10,
        recruited_departures: 11,
        uninformed_departures: 12,
    };
    let mut total = other.clone();
    total.checked_add_assign(&other).unwrap();
    assert_eq!(
        total,
        WorkCounts {
            opportunities: 2,
            waits: 4,
            directed_moves: 6,
            search_moves: 8,
            pickups: 10,
            deliveries: 12,
            empty_returns: 14,
            search_switches: 16,
            publications: 18,
            fidelity_departures: 20,
            recruited_departures: 22,
            uninformed_departures: 24
        }
    );
}
#[test]
fn work_overflow_does_not_partially_change_total() {
    let mut total = WorkCounts {
        uninformed_departures: u64::MAX,
        ..WorkCounts::default()
    };
    let before = total.clone();
    let other = WorkCounts {
        opportunities: 1,
        uninformed_departures: 1,
        ..WorkCounts::default()
    };
    assert!(total.checked_add_assign(&other).is_err());
    assert_eq!(total, before);
}
