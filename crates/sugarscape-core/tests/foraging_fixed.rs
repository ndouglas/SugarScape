use sugarscape_core::foraging::{
    fixed::{run, Pos, Resource, RunOptions, Setup, World},
    CpfaParameters,
};
fn setup() -> Setup {
    Setup {
        width: 5,
        height: 5,
        nest: Pos { x: 2, y: 2 },
        agents: 1,
        resources: vec![
            Resource {
                id: u64::MAX,
                pos: Pos { x: 3, y: 2 },
            },
            Resource {
                id: 17,
                pos: Pos { x: 1, y: 1 },
            },
        ],
        parameters: CpfaParameters {
            p_search: 1.0,
            p_return: 0.0,
            omega: 0.0,
            lambda_informed: 0.0,
            lambda_fidelity: 0.0,
            lambda_publish: 0.0,
            lambda_waypoint: 0.0,
        },
    }
}
fn options(ticks: u32) -> RunOptions {
    RunOptions {
        ticks,
        sample_every: 7,
        snapshots: true,
    }
}
// Break caught: nonrepeatable output, manual/run divergence, identity loss or nonconservation.
#[test]
fn public_replay_matches_manual_steps_and_preserves_inventory() {
    let s = setup();
    let episode = run(s.clone(), 12, options(20)).unwrap();
    assert_eq!(episode, run(s.clone(), 12, options(20)).unwrap());
    let mut world = World::new(s, 12).unwrap();
    for _ in 0..20 {
        world.step().unwrap();
    }
    assert_eq!(episode.summary, world.summary().unwrap());
    assert_eq!(
        episode.snapshots.last().unwrap(),
        &world.snapshot().unwrap()
    );
    assert_eq!(
        episode
            .snapshots
            .iter()
            .map(|s| s.completed_ticks)
            .collect::<Vec<_>>(),
        vec![0, 7, 14, 20]
    );
    for frame in &episode.snapshots {
        let i = &frame.inventory;
        assert_eq!(i.initial, i.available + i.assigned + i.delivered);
        assert_eq!(
            frame
                .resources
                .iter()
                .map(|r| r.resource.id)
                .collect::<Vec<_>>(),
            vec![17, u64::MAX]
        );
    }
}
// Break caught: converting assigned cargo into delivery at the cutoff.
#[test]
fn public_partial_return_preserves_assigned_token() {
    let mut s = setup();
    s.width = 7;
    s.height = 7;
    s.nest = Pos { x: 3, y: 3 };
    // Eight explicit cells two steps from the nest cover every zero-turn heading.
    s.resources = [
        Pos { x: 1, y: 1 },
        Pos { x: 1, y: 3 },
        Pos { x: 1, y: 5 },
        Pos { x: 3, y: 1 },
        Pos { x: 3, y: 5 },
        Pos { x: 5, y: 1 },
        Pos { x: 5, y: 3 },
        Pos { x: 5, y: 5 },
    ]
    .into_iter()
    .zip(0_u64..)
    .map(|(pos, id)| Resource { id, pos })
    .collect();
    let episode = run(
        s,
        12,
        RunOptions {
            ticks: 8,
            sample_every: 1,
            snapshots: true,
        },
    )
    .unwrap();
    let assigned = episode
        .snapshots
        .iter()
        .find(|s| s.inventory.assigned > 0)
        .expect("fixture includes pickup");
    assert_eq!(assigned.inventory.delivered, 0);
    assert_eq!(assigned.first_delivery_tick, None);
    assert_eq!(episode.summary.inventory.assigned, 1);
    assert_eq!(episode.summary.inventory.delivered, 0);
    assert_eq!(episode.summary.first_delivery_tick, None);
}
