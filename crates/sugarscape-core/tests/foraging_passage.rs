use sugarscape_core::foraging::passage::{
    run, CellKnowledge, Parameters, Pos, Resource, RunOptions, Setup, World,
};
fn pos(x: u32, y: u32) -> Pos {
    Pos { x, y }
}
fn setup() -> Setup {
    Setup {
        width: 5,
        height: 5,
        open: vec![pos(0, 0), pos(1, 0), pos(2, 0), pos(3, 0), pos(3, 1)],
        nest: vec![pos(0, 0), pos(1, 0)],
        workers: vec![pos(0, 0)],
        resources: vec![Resource {
            id: u64::MAX,
            pos: pos(3, 0),
        }],
        parameters: Parameters {
            p_search: 1.0,
            p_return: 0.0,
            lambda_fidelity: 0.0,
            lambda_publish: 0.0,
            lambda_waypoint: 0.0,
        },
    }
}
#[test]
fn public_passage_run_replays_and_reports_conserved_work_without_assuming_delivery() {
    let s = setup();
    let e = run(
        s.clone(),
        12,
        RunOptions {
            ticks: 40,
            sample_every: 7,
            snapshots: true,
        },
    )
    .unwrap();
    let mut w = World::new(s, 12).unwrap();
    for _ in 0..40 {
        w.step().unwrap();
    }
    assert_eq!(e.summary, w.summary().unwrap());
    assert_eq!(
        e,
        run(
            e.setup.clone(),
            e.seed,
            RunOptions {
                ticks: 40,
                sample_every: 7,
                snapshots: true
            }
        )
        .unwrap()
    );
    assert_eq!(e.summary.work.opportunities, 40);
    assert_eq!(
        e.summary.work.opportunities,
        e.summary.work.moves
            + e.summary.work.pickups
            + e.summary.work.deposits
            + e.summary.work.waits
    );
    let inventory = e.summary.inventory;
    assert_eq!(
        inventory.initial,
        inventory.available + inventory.carried + inventory.delivered
    );
    assert_eq!(e.snapshots.first().unwrap().summary.completed_ticks, 0);
    assert_eq!(e.snapshots.last().unwrap().summary.completed_ticks, 40);
    assert_eq!(e.summary.compute.observations, 41);
    let knowledge = w.knowledge(0).unwrap();
    assert!(knowledge
        .cells
        .iter()
        .all(|c| c.kind != CellKnowledge::Unknown));
    assert!(knowledge
        .cells
        .windows(2)
        .all(|pair| pair[0].pos < pair[1].pos));
    let _: Result<_, Vec<sugarscape_core::config::FieldError>> = w.knowledge(u32::MAX);
}
