//! Authoritative protection state survives JSON persistence.
use sugarscape_core::geometry::Pos;
use sugarscape_core::minds::protection::state::*;

#[test]
fn state_round_trips_every_intent_stage_and_erased_memory() {
    for stage in [
        Stage::ToSource,
        Stage::Retrieve,
        Stage::ToDestination,
        Stage::Deposit,
    ] {
        for erased in [false, true] {
            let mut state = ProtectionState::default();
            state.sources.insert(
                30,
                Source {
                    tick: 0,
                    initial_amount: 12.0,
                    attempted: true,
                },
            );
            if !erased {
                state.exposure.remember(30, 0, true);
            }
            state.intent = Some(Intent {
                source: 30,
                destination: Pos::new(3, 2),
                amount: 12.0,
                stage: stage.clone(),
            });
            let restored: ProtectionState =
                serde_json::from_str(&serde_json::to_string(&state).unwrap()).unwrap();
            assert_eq!(restored, state);
        }
    }
}

#[test]
fn cancellation_and_death_records_round_trip() {
    use sugarscape_core::minds::protection::runner::DeathRecord;
    let event = SourceEvent {
        source: 30,
        cancellation: Some(CancelReason::OwnerDied),
        ..Default::default()
    };
    assert_eq!(
        serde_json::from_str::<SourceEvent>(&serde_json::to_string(&event).unwrap()).unwrap(),
        event
    );
    let death = DeathRecord {
        id: 1,
        pos: Pos::new(3, 3),
        cause: "starvation".into(),
    };
    assert_eq!(
        serde_json::from_str::<DeathRecord>(&serde_json::to_string(&death).unwrap()).unwrap(),
        death
    );
}

#[test]
fn enabled_fingerprint_covers_each_authoritative_protection_field() {
    use sugarscape_core::{minds::protection::lab::rig_config, world::World};
    let mut base = World::new(
        rig_config(LabConfig {
            policy: Policy::Selective,
            ..Default::default()
        }),
        7,
    )
    .unwrap();
    for _ in 0..9 {
        base.step();
    }
    type Mutation = (&'static str, fn(&mut World));
    let changes: &[Mutation] = &[
        ("policy", |w| {
            w.config.protection_lab.as_mut().unwrap().policy = Policy::Off
        }),
        ("source tick", |w| {
            w.agent_mut(1)
                .unwrap()
                .protection
                .as_mut()
                .unwrap()
                .sources
                .values_mut()
                .next()
                .unwrap()
                .tick += 1
        }),
        ("attempted", |w| {
            w.agent_mut(1)
                .unwrap()
                .protection
                .as_mut()
                .unwrap()
                .sources
                .values_mut()
                .next()
                .unwrap()
                .attempted = false
        }),
        ("exposure bit", |w| {
            w.agent_mut(1)
                .unwrap()
                .protection
                .as_mut()
                .unwrap()
                .exposure
                .entries
                .values_mut()
                .next()
                .unwrap()
                .exposed = false
        }),
        ("exposure tick", |w| {
            w.agent_mut(1)
                .unwrap()
                .protection
                .as_mut()
                .unwrap()
                .exposure
                .entries
                .values_mut()
                .next()
                .unwrap()
                .tick += 1
        }),
        ("destination", |w| {
            w.agent_mut(1)
                .unwrap()
                .protection
                .as_mut()
                .unwrap()
                .intent
                .as_mut()
                .unwrap()
                .destination
                .x += 1
        }),
        ("stage", |w| {
            w.agent_mut(1)
                .unwrap()
                .protection
                .as_mut()
                .unwrap()
                .intent
                .as_mut()
                .unwrap()
                .stage = Stage::Deposit
        }),
        ("pending amount", |w| {
            w.agent_mut(1)
                .unwrap()
                .protection
                .as_mut()
                .unwrap()
                .intent
                .as_mut()
                .unwrap()
                .amount += 1.0
        }),
        ("observer seen", |w| {
            w.agent_mut(2)
                .unwrap()
                .seen
                .values_mut()
                .next()
                .unwrap()
                .tick += 1
        }),
        ("span", |w| {
            w.config.protection_lab.as_mut().unwrap().exposure_span += 1
        }),
        ("schedule", |w| {
            w.config.protection_lab.as_mut().unwrap().fixture = Fixture::Mixed {
                observed_first: true,
            }
        }),
    ];
    for (name, mutate) in changes {
        let mut altered = base.clone();
        mutate(&mut altered);
        assert_ne!(base.fingerprint(), altered.fingerprint(), "{name}");
    }
}
