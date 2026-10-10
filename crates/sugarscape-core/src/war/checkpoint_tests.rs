use super::{
    checkpoint::Checkpoint,
    config::{Geometry, Side},
    engine::Engagement,
    engine_tests::config,
    records::{EndReason, Ending},
};

#[test]
fn restores_initial_completed_and_terminal_states_exactly() {
    for completed in 0..=3 {
        let c = config(Geometry::DuelContact);
        let mut original = Engagement::new(c.clone(), 7).unwrap();
        for _ in 0..completed {
            original.step().unwrap();
        }
        let text = serde_json::to_string(&original.checkpoint()).unwrap();
        let saved: Checkpoint = serde_json::from_str(&text).unwrap();
        let mut restored = Engagement::new(c, 7).unwrap();
        restored.restore(&saved).unwrap();
        assert_eq!(restored.checkpoint(), saved);
        loop {
            let a = original.step().unwrap();
            let b = restored.step().unwrap();
            assert_eq!(a, b);
            assert_eq!(original.checkpoint(), restored.checkpoint());
            if a.is_none() {
                break;
            }
        }
    }
}

#[test]
fn restores_after_deaths_with_exact_global_ids() {
    let c = config(Geometry::DuelContact);
    let mut original = Engagement::new(c.clone(), 7).unwrap();
    original
        .step_supplied(&[(0, 2), (1, 3)], &[0, u64::MAX, u64::MAX, 0])
        .unwrap();
    let mut restored = Engagement::new(c, 7).unwrap();
    restored.restore(&original.checkpoint()).unwrap();
    assert_eq!(restored.step().unwrap(), original.step().unwrap());
    assert_eq!(restored.checkpoint(), original.checkpoint());
}

#[test]
fn field_mutations_are_rejected_without_partial_restore() {
    let mut engine = Engagement::new(config(Geometry::DuelContact), 7).unwrap();
    engine.step().unwrap();
    let before = engine.checkpoint();
    let mut cases = Vec::new();
    macro_rules! bad {
        ($field:ident,$value:expr) => {{
            let mut c = before.clone();
            c.$field = $value;
            cases.push(c);
        }};
    }
    bad!(schema, "unknown".into());
    bad!(seed, 8);
    bad!(step, 4);
    bad!(active_steps, before.step + 1);
    bad!(blue_alive, vec![0, 0]);
    bad!(blue_alive, vec![1, 0]);
    bad!(blue_alive, vec![2]);
    bad!(red_alive, vec![0]);
    bad!(red_alive, vec![4]);
    bad!(contact_rng, "invalid".into());
    bad!(casualty_rng, " ".repeat(2048));
    bad!(contact_rng, "{\"state\":1,\"extra\":2}".into());
    bad!(contact_rng, "{\"state\":1,\"state\":3}".into());
    bad!(contact_rng, "{\"state\":2}".into());
    bad!(
        ending,
        Some(Ending {
            reason: EndReason::DoubleExtinction,
            winner: None,
            censored: false
        })
    );
    let mut wrong = before.clone();
    wrong.config.dt = f64::from_bits(wrong.config.dt.to_bits() + 1);
    cases.push(wrong);
    let mut wrong = before.clone();
    wrong.config.geometry = Geometry::AimedFire;
    cases.push(wrong);
    for saved in cases {
        assert!(engine.restore(&saved).is_err(), "{saved:?}");
        assert_eq!(engine.checkpoint(), before);
    }
}

#[test]
fn signed_zero_config_change_is_rejected() {
    let mut c = config(Geometry::DuelContact);
    c.blue_rate = 0.0;
    let mut engine = Engagement::new(c, 7).unwrap();
    let before = engine.checkpoint();
    let mut saved = before.clone();
    saved.config.blue_rate = -0.0;
    assert!(engine.restore(&saved).is_err());
    assert_eq!(engine.checkpoint(), before);
}

#[test]
fn endings_and_completed_population_are_consistent() {
    let c = config(Geometry::DuelContact);
    let mut engine = Engagement::new(c, 7).unwrap();
    let before = engine.checkpoint();
    for (step, active, blue, red, end) in [
        (0, 0, vec![0], vec![2, 3], None),
        (1, 0, vec![0, 1], vec![2, 3], None),
        (3, 3, vec![0, 1], vec![2, 3], None),
        (1, 1, vec![], vec![2, 3], None),
        (
            1,
            1,
            vec![],
            vec![2, 3],
            Some(Ending {
                reason: EndReason::OneSideExtinction,
                winner: Some(Side::Blue),
                censored: false,
            }),
        ),
        (
            3,
            3,
            vec![0, 1],
            vec![2, 3],
            Some(Ending {
                reason: EndReason::Horizon,
                winner: None,
                censored: false,
            }),
        ),
    ] {
        let mut saved = before.clone();
        saved.step = step;
        saved.active_steps = active;
        saved.blue_alive = blue;
        saved.red_alive = red;
        saved.ending = end;
        assert!(engine.restore(&saved).is_err());
        assert_eq!(engine.checkpoint(), before);
    }
}

#[test]
fn records_reject_unknown_fields() {
    let engine = Engagement::new(config(Geometry::DuelContact), 7).unwrap();
    let text = serde_json::to_string(&engine.checkpoint()).unwrap();
    let altered = text.replacen('{', "{\"unexpected\":0,", 1);
    assert!(serde_json::from_str::<Checkpoint>(&altered).is_err());
    assert!(serde_json::from_str::<Ending>(
        "{\"reason\":\"horizon\",\"winner\":null,\"censored\":true,\"extra\":0}"
    )
    .is_err());
}
