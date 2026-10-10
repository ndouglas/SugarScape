use super::{
    config::{EngagementConfig, Geometry, Side},
    engine::Engagement,
    records::EndReason,
};
use crate::rng;
use rand::RngCore;

pub(super) fn config(geometry: Geometry) -> EngagementConfig {
    EngagementConfig {
        blue: 2,
        red: 2,
        blue_rate: 1.0,
        red_rate: 1.0,
        dt: 0.05,
        max_steps: 3,
        geometry,
    }
}

#[test]
fn both_actors_settle_before_removal() {
    let mut c = config(Geometry::DuelContact);
    c.blue = 1;
    c.red = 1;
    c.max_steps = 1;
    let mut engine = Engagement::new(c, 7).unwrap();
    let frame = engine.step_supplied(&[(0, 1)], &[0, 0]).unwrap().unwrap();
    assert_eq!(frame.survivors, [0, 0]);
    assert_eq!(
        frame
            .casualties
            .iter()
            .map(|c| (c.id, c.side))
            .collect::<Vec<_>>(),
        [(0, Side::Blue), (1, Side::Red)]
    );
    assert_eq!(frame.ending.unwrap().reason, EndReason::DoubleExtinction);
    assert!(engine.step().unwrap().is_none());
}

#[test]
fn reversed_matching_keeps_both_actions_and_canonical_draw_order() {
    let c = config(Geometry::DuelContact);
    let mut forward = Engagement::new(c.clone(), 7).unwrap();
    let mut reverse = Engagement::new(c, 7).unwrap();
    let a = forward
        .step_supplied(&[(0, 2), (1, 3)], &[0, u64::MAX, u64::MAX, 0])
        .unwrap();
    let b = reverse
        .step_supplied(&[(1, 3), (0, 2)], &[0, u64::MAX, u64::MAX, 0])
        .unwrap();
    assert_eq!(a, b);
    assert_eq!(
        a.unwrap()
            .casualties
            .iter()
            .map(|c| c.id)
            .collect::<Vec<_>>(),
        [0, 3]
    );
}

#[test]
fn invalid_numeric_attempt_keeps_both_streams() {
    let c = EngagementConfig {
        blue: 2,
        red: 2,
        blue_rate: 1e-18,
        red_rate: 1.0,
        dt: 0.1,
        max_steps: 2,
        geometry: Geometry::DuelContact,
    };
    let mut engine = Engagement::new(c, 7).unwrap();
    let before = serde_json::to_vec(&engine.checkpoint()).unwrap();
    let error = engine.step().unwrap_err();
    assert_eq!(error.attempted_step, 1);
    assert!(error.issue.detail.contains("target_id=2"));
    assert_eq!(serde_json::to_vec(&engine.checkpoint()).unwrap(), before);
}

#[test]
fn invalid_matching_or_word_count_preserves_state() {
    for pairs in [
        vec![(0, 2), (0, 3)],
        vec![(0, 2), (1, 2)],
        vec![(2, 0), (1, 3)],
        vec![(0, 0), (1, 3)],
        vec![(0, 2)],
        vec![(0, 2), (1, 4)],
    ] {
        let mut engine = Engagement::new(config(Geometry::DuelContact), 7).unwrap();
        let before = engine.checkpoint();
        assert!(engine.step_supplied(&pairs, &[0; 4]).is_err(), "{pairs:?}");
        assert_eq!(engine.checkpoint(), before);
    }
    for words in [vec![0; 3], vec![0; 5]] {
        let mut engine = Engagement::new(config(Geometry::DuelContact), 7).unwrap();
        let before = engine.checkpoint();
        assert!(engine.step_supplied(&[(0, 2), (1, 3)], &words).is_err());
        assert_eq!(engine.checkpoint(), before);
    }
}

#[test]
fn aimed_and_duel_report_independent_exposure_budgets() {
    for (geometry, hazards, exposed, rates, pairs, expected) in [
        (
            Geometry::AimedFire,
            [0.5, 2.0],
            [2, 1],
            [2.0, 1.0],
            2,
            [0.049380175943334666, 0.09516258196404043],
        ),
        (
            Geometry::DuelContact,
            [1.0, 1.0],
            [1, 1],
            [1.0, 1.0],
            1,
            [0.048770575499285984, 0.048770575499285984],
        ),
    ] {
        let mut c = config(geometry);
        c.red = 1;
        let mut engine = Engagement::new(c, 7).unwrap();
        let words = vec![u64::MAX; exposed.iter().sum::<u32>() as usize];
        let frame = engine.step_supplied(&[(0, 2)], &words).unwrap().unwrap();
        assert_eq!(frame.exposure.contact_pairs, pairs);
        assert_eq!(frame.exposure.exposed, exposed);
        assert_eq!(frame.exposure.contributed_rate, rates);
        assert_eq!(
            frame.exposure.integrated,
            [rates[0] * 0.05, rates[1] * 0.05]
        );
        for side in 0..2 {
            let p = frame.exposure.target_probability[side].as_ref().unwrap();
            assert_eq!(p.hazard, hazards[side]);
            assert!((f64::from(exposed[side]) * p.ideal - expected[side]).abs() < 1e-15);
            let expected_q = if geometry == Geometry::AimedFire {
                [0.04938017594333477, 0.09516258196404048][side]
            } else {
                0.048770575499286095
            };
            assert!((f64::from(exposed[side]) * p.realized - expected_q).abs() < 1e-16);
        }
    }
}

#[test]
fn one_zero_rate_skips_actual_rng_draws_for_immune_targets() {
    for geometry in [Geometry::AimedFire, Geometry::DuelContact] {
        let mut c = config(geometry);
        c.blue_rate = 0.0;
        let mut engine = Engagement::new(c, 7).unwrap();
        let before = engine.checkpoint();
        let mut expected: rng::SimRng = serde_json::from_str(&before.casualty_rng).unwrap();
        expected.next_u64();
        expected.next_u64();
        let frame = engine.step().unwrap().unwrap();
        assert_eq!(frame.exposure.exposed, [2, 0]);
        assert_eq!(engine.checkpoint().casualty_rng, rng::state_json(&expected));
        if geometry == Geometry::AimedFire {
            assert_eq!(engine.checkpoint().contact_rng, before.contact_rng);
        }
    }
}

#[test]
fn initialization_end_precedence_consumes_no_rng_or_steps() {
    for (counts, rates, reason, winner) in [
        ([0, 0], [0.0, 0.0], EndReason::DoubleExtinction, None),
        (
            [0, 2],
            [0.0, 0.0],
            EndReason::OneSideExtinction,
            Some(Side::Red),
        ),
        (
            [2, 0],
            [1.0, 1.0],
            EndReason::OneSideExtinction,
            Some(Side::Blue),
        ),
        ([2, 2], [0.0, 0.0], EndReason::RateZero, None),
    ] {
        let mut c = config(Geometry::DuelContact);
        c.blue = counts[0];
        c.red = counts[1];
        c.blue_rate = rates[0];
        c.red_rate = rates[1];
        let mut engine = Engagement::new(c, 7).unwrap();
        assert_eq!(engine.ending().unwrap().reason, reason);
        assert_eq!(engine.ending().unwrap().winner, winner);
        let before = engine.checkpoint();
        assert!(engine.step().unwrap().is_none());
        assert_eq!(engine.checkpoint(), before);
    }
}

#[test]
fn clocks_horizon_and_final_extinction_are_distinct() {
    for (words, reason, censored) in [
        ([u64::MAX, u64::MAX], EndReason::Horizon, true),
        ([0, u64::MAX], EndReason::OneSideExtinction, false),
    ] {
        let mut c = config(Geometry::DuelContact);
        c.blue = 1;
        c.red = 1;
        c.max_steps = 1;
        let mut engine = Engagement::new(c, 7).unwrap();
        let frame = engine.step_supplied(&[(0, 1)], &words).unwrap().unwrap();
        assert_eq!(
            (
                frame.step,
                frame.active_steps,
                frame.calendar_time,
                frame.active_time
            ),
            (1, 1, 0.05, 0.05)
        );
        let end = frame.ending.unwrap();
        assert_eq!((end.reason, end.censored), (reason, censored));
    }
}

#[test]
fn casualty_ids_never_repeat_and_living_ids_remain_sorted() {
    let mut engine = Engagement::new(config(Geometry::DuelContact), 7).unwrap();
    let first = engine
        .step_supplied(&[(0, 2), (1, 3)], &[0, u64::MAX, u64::MAX, 0])
        .unwrap()
        .unwrap();
    assert_eq!(first.survivors, [1, 1]);
    let saved = engine.checkpoint();
    assert_eq!((saved.blue_alive, saved.red_alive), (vec![1], vec![2]));
    let next = engine.step_supplied(&[(1, 2)], &[0, 0]).unwrap().unwrap();
    assert_eq!(
        next.casualties.iter().map(|c| c.id).collect::<Vec<_>>(),
        [1, 2]
    );
}

#[test]
fn observation_and_seed_order_do_not_change_trajectories() {
    fn run(
        seed: u64,
        observe: bool,
    ) -> (Vec<super::records::Frame>, super::checkpoint::Checkpoint) {
        let mut engine = Engagement::new(config(Geometry::DuelContact), seed).unwrap();
        let mut frames = Vec::new();
        while let Some(frame) = engine.step().unwrap() {
            if observe {
                let _ = serde_json::to_string(&frame).unwrap();
                let _ = engine.checkpoint();
            }
            frames.push(frame);
        }
        (frames, engine.checkpoint())
    }
    let seven = run(7, false);
    let eight = run(8, true);
    assert_eq!(run(8, false), eight);
    assert_eq!(run(7, true), seven);
}
