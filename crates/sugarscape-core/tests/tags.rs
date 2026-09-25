//! The tags model against Riolo, Cohen & Axelrod 2001 and Edmonds & Hales
//! 2003, shortened to run with the other tests (the survey runs the paper's
//! 30 000 generations over 20 seeds).

use sugarscape_core::tags::{TagsConfig, TagsWorld, TieRule};

/// The mean donation rate over `generations` generations, over seeds 1–3.
fn donation(tie_rule: TieRule, pairings: u32, generations: u32) -> f64 {
    let mut sum = 0.0;
    for seed in 1..=3 {
        let mut w = TagsWorld::new(
            TagsConfig {
                tie_rule,
                pairings,
                ..TagsConfig::default()
            },
            seed,
        )
        .unwrap();
        w.run(generations);
        let d = w.stats.series("donation_rate").unwrap();
        sum += d.iter().sum::<f64>() / d.len() as f64;
    }
    sum / 3.0
}

#[test]
fn only_ties_to_the_current_agent_give_the_papers_two_pairings() {
    // RCA's Table 1: 4.3 %; E&H's Table 7: 42.6 % with coin-flip ties.
    let current = donation(TieRule::Current, 2, 3000);
    let random = donation(TieRule::Random, 2, 3000);
    assert!(current < 0.1, "{current}");
    assert!(random > 0.3, "{random}");
}

#[test]
fn three_pairings_cooperate_whatever_the_tie_rule() {
    // Table 1: 73.6 %.
    for tie in [TieRule::Random, TieRule::Current, TieRule::Other] {
        let d = donation(tie, 3, 3000);
        assert!((0.68..0.78).contains(&d), "{tie:?}: {d}");
    }
}
