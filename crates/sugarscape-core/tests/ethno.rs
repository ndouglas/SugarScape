//! The ethnocentrism model (milestone 14) against Hammond & Axelrod 2006
//! (HA06), its appendix and archived code, Hartshorn, Kaznatcheev & Shultz
//! 2013 (HKS13) and Jansson 2013 (J13). Every claim runs over seeds 1–10
//! (HKS13's Study 1 over 50) in release: `cargo test -p sugarscape-core
//! --release --test ethno -- --ignored --nocapture`. A run's summary is
//! the mean over its last 100 periods (1,901–2,000), averaged over seeds.
//! Thresholds come from the measurements recorded 2026-09-25
//! (docs/superpowers/plans/2026-09-25-ethnocentrism.md, Decision 12); claims
//! that do not hold are pinned as measured.

use std::thread;

use sugarscape_core::ethno::{
    Discrimination, EthnoConfig, EthnoWorld, KinBasis, Offspring, PairPlay, Start, Strategy,
};

/// Runs `c` for `ticks` from seeds 1..=`seeds` in parallel and measures each run.
fn each_seed<T: Send>(
    c: &EthnoConfig,
    seeds: u64,
    ticks: u32,
    f: impl Fn(&EthnoWorld) -> T + Sync,
) -> Vec<T> {
    let mut c = c.clone();
    c.end = c.end.max(ticks);
    let c = &c;
    thread::scope(|s| {
        let handles: Vec<_> = (1..=seeds)
            .map(|seed| {
                let f = &f;
                s.spawn(move || {
                    let mut w = EthnoWorld::new(c.clone(), seed).unwrap();
                    w.run(ticks);
                    f(&w)
                })
            })
            .collect();
        handles.into_iter().map(|h| h.join().unwrap()).collect()
    })
}

/// The mean of `name` over the run's last 100 periods, skipping undefined ones.
fn last_100(w: &EthnoWorld, name: &str) -> f64 {
    let s = w.stats.series(name).unwrap();
    let v: Vec<f64> = s[s.len() - 100..]
        .iter()
        .copied()
        .filter(|x| x.is_finite())
        .collect();
    v.iter().sum::<f64>() / v.len() as f64
}

fn mean(v: &[f64]) -> f64 {
    v.iter().sum::<f64>() / v.len() as f64
}

/// The seed mean of `name`'s last-100 mean, in percent.
fn pct(c: &EthnoConfig, ticks: u32, name: &str) -> f64 {
    100.0 * mean(&each_seed(c, 10, ticks, |w| last_100(w, name)))
}

/// (ethnocentric %, cooperation %) over seeds 1–10.
fn row(c: &EthnoConfig, ticks: u32) -> (f64, f64) {
    let v = each_seed(c, 10, ticks, |w| {
        (last_100(w, "ethnocentric"), last_100(w, "cooperation"))
    });
    let e = mean(&v.iter().map(|x| x.0).collect::<Vec<_>>());
    let k = mean(&v.iter().map(|x| x.1).collect::<Vec<_>>());
    (100.0 * e, 100.0 * k)
}

fn standard() -> EthnoConfig {
    EthnoConfig::default()
}

fn near(ours: f64, target: f64, within: f64) -> bool {
    (ours - target).abs() <= within
}

#[test]
#[ignore]
fn table_1_as_measured() {
    // (row, config, periods, ours E, ours C, HA06 E, HA06 C). Ours are the
    // seed means (seeds 1–10), recorded 2026-09-25; HA06's s.e. is ~1–2.
    type Edit = fn(&mut EthnoConfig);
    let rows: [(&str, Edit, u32, f64, f64, f64, f64); 13] = [
        ("a standard", |_| {}, 2000, 75.9, 76.0, 76.3, 74.2),
        (
            "b cost 0.5%",
            |c| c.cost = 0.005,
            2000,
            76.0,
            78.5,
            76.0,
            77.8,
        ),
        ("c cost 2%", |c| c.cost = 0.02, 2000, 63.5, 64.7, 61.8, 56.1),
        (
            "d 2 colours",
            |c| c.colors = 2,
            2000,
            68.9,
            78.9,
            69.4,
            78.1,
        ),
        (
            "e 8 colours",
            |c| c.colors = 8,
            2000,
            77.9,
            74.4,
            79.1,
            71.7,
        ),
        (
            "f mutation 0.25%",
            |c| c.mutation = 0.0025,
            2000,
            83.4,
            80.2,
            82.8,
            79.8,
        ),
        (
            "g mutation 1%",
            |c| c.mutation = 0.01,
            2000,
            63.0,
            70.8,
            67.1,
            69.0,
        ),
        (
            "h immigration 0.5",
            |c| c.immigration = 0.5,
            2000,
            77.9,
            77.8,
            77.5,
            75.5,
        ),
        (
            "i immigration 2",
            |c| c.immigration = 2.0,
            2000,
            70.5,
            73.9,
            74.4,
            71.4,
        ),
        ("j 25 × 25", |c| c.width = 25, 2000, 64.4, 70.1, 70.5, 69.9),
        (
            "k 100 × 100",
            |c| c.width = 100,
            2000,
            76.5,
            79.1,
            78.2,
            76.0,
        ),
        ("l 500 periods", |_| {}, 500, 57.3, 78.6, 73.9, 73.4),
        ("m 4,000 periods", |_| {}, 4000, 77.1, 75.9, 77.3, 74.4),
    ];
    let mut within_3 = Vec::new();
    for (name, edit, ticks, our_e, our_c, ha_e, ha_c) in rows {
        let mut c = standard();
        edit(&mut c);
        let (e, k) = row(&c, ticks);
        println!("{name}: E {e:.1} (HA06 {ha_e}), C {k:.1} (HA06 {ha_c})");
        assert!(
            near(e, our_e, 0.06) && near(k, our_c, 0.06),
            "{name}: {e} {k}"
        );
        if near(e, ha_e, 3.0) && near(k, ha_c, 3.0) {
            within_3.push(&name[..1]);
        }
    }
    // Both columns within 3 points of HA06's: a, b, d, e, f, h and m. Not
    // reproduced: c (cooperation 64.7 against 56.1), g, i and j (too few
    // ethnocentrics: 63.0, 70.5, 64.4 against 67.1, 74.4, 70.5), k
    // (cooperation 79.1 against 76.0) and l (57.3% ethnocentric after 500
    // periods against 73.9%: our ethnocentrics take longer to dominate).
    assert_eq!(within_3, ["a", "b", "d", "e", "f", "h", "m"]);
}

#[test]
#[ignore]
fn table_1_cannot_tell_four_colours_from_five() {
    // The Java draws five tags for "four" (and nine for "eight"). Rows d/a/e
    // against 2/4/8 colours: 68.9/75.9/77.9; against 2/5/9: 68.9/75.4/81.8;
    // HA06: 69.4/76.3/79.1. Both within 2.7 points of every row.
    let e = |colors| {
        pct(
            &EthnoConfig {
                colors,
                ..standard()
            },
            2000,
            "ethnocentric",
        )
    };
    let (two, four, five, eight, nine) = (e(2), e(4), e(5), e(8), e(9));
    println!("2: {two:.1} 4: {four:.1} 5: {five:.1} 8: {eight:.1} 9: {nine:.1}");
    for (ours, ha) in [
        (two, 69.4),
        (four, 76.3),
        (five, 76.3),
        (eight, 79.1),
        (nine, 79.1),
    ] {
        assert!(near(ours, ha, 2.8), "{ours} vs {ha}");
    }
    assert!(near(five, 75.4, 0.06) && near(nine, 81.8, 0.06));
}

#[test]
#[ignore]
fn the_appendixs_mutation_rate_is_not_table_1s() {
    // HA06's appendix: MutationRate = 0.05. It gives 36% ethnocentric and
    // 56% cooperation, far from Table 1 a's 76.3/74.2; 0.005 gives 75.9/76.0.
    let (e, k) = row(
        &EthnoConfig {
            mutation: 0.05,
            ..standard()
        },
        2000,
    );
    println!("mutation 0.05: E {e:.1} C {k:.1}");
    assert!(near(e, 36.0, 0.06) && near(k, 56.4, 0.06));
}

#[test]
#[ignore]
fn the_appendixs_double_play_raises_ethnocentrism() {
    // Every decision twice: 80.9% ethnocentric (77.3% cooperation), against
    // 75.9 once — five points above Table 1 a, so the text's once fits better.
    let (e, k) = row(
        &EthnoConfig {
            pair_play: PairPlay::Twice,
            ..standard()
        },
        2000,
    );
    println!("twice: E {e:.1} C {k:.1}");
    assert!(near(e, 80.9, 0.06) && near(k, 77.3, 0.06));
}

#[test]
#[ignore]
fn the_archived_code_lands_where_the_paper_does() {
    // The archive as it runs — five colours, a full random start, no
    // immigration — 77.7% ethnocentric, 77.6% cooperation: the same regime.
    let c = EthnoConfig {
        colors: 5,
        start: Start::Random,
        immigration: 0.0,
        ..standard()
    };
    let (e, k) = row(&c, 2000);
    println!("archive: E {e:.1} C {k:.1}");
    assert!(near(e, 77.7, 0.06) && near(k, 77.6, 0.06));
}

#[test]
#[ignore]
fn a_lattice_of_egoists_becomes_just_as_ethnocentric() {
    // HA06: "just as dominant". Measured: 78.6% (7% at period 100, 26% at 200, 70% at 500).
    let c = EthnoConfig {
        start: Start::Selfish,
        immigration: 0.0,
        ..standard()
    };
    let e = pct(&c, 2000, "ethnocentric");
    println!("egoists: E {e:.1}");
    assert!(near(e, 78.6, 0.06) && e > 70.0);
}

#[test]
#[ignore]
fn each_colour_strategies_are_ethnocentric_only_in_a_loose_sense() {
    // HA06: "80 percent ethnocentric strategies". Helping one's own colour
    // only: 27.0%. Helping one's own colour and refusing at least one
    // other: 84.3%, which is HA06's number.
    let c = EthnoConfig {
        discrimination: Discrimination::EachColor,
        ..standard()
    };
    let v = each_seed(&c, 10, 1900, |w| {
        let mut w = w.clone();
        let (mut strict, mut loose) = (0.0, 0.0);
        for _ in 0..100 {
            w.step();
            let n = w.population() as f64;
            let all = (1u64 << w.config.colors) - 1;
            strict += w.agents().filter(|a| a.help == 1 << a.tag).count() as f64 / n;
            loose += w
                .agents()
                .filter(|a| a.help & (1 << a.tag) != 0 && a.help != all)
                .count() as f64
                / n;
        }
        (strict / 100.0, loose / 100.0)
    });
    let strict = 100.0 * mean(&v.iter().map(|x| x.0).collect::<Vec<_>>());
    let loose = 100.0 * mean(&v.iter().map(|x| x.1).collect::<Vec<_>>());
    println!("each colour: own only {strict:.1}, own and not all {loose:.1}");
    assert!(near(strict, 27.0, 0.06) && near(loose, 84.3, 0.06));
}

#[test]
#[ignore]
fn misperception_keeps_two_thirds_ethnocentric() {
    // HA06: "more than two-thirds". Measured 71.7%.
    let e = pct(
        &EthnoConfig {
            misperception: 0.1,
            ..standard()
        },
        2000,
        "ethnocentric",
    );
    println!("misperception: E {e:.1}");
    assert!(near(e, 71.7, 0.06) && e > 200.0 / 3.0);
}

#[test]
#[ignore]
fn blind_agents_at_doubled_cost_cooperate_far_more_than_14_percent() {
    // HA06: 56% cooperation discriminating, 14% colour-blind, at cost 2%.
    // Measured: 64.7% and 41.8%; blind cooperation reaches 14% only at cost 3% (12.7%).
    // Other readings at cost 2% (recorded, not switches): one colour with
    // two bits 40.2; a coin per decision (misperception 0.5) 44.9; blind
    // deciding twice 24.5; blind with benefit halved 11.6 (but seeing agents
    // then cooperate 17.7, not 56).
    let c = |discrimination, cost| EthnoConfig {
        discrimination,
        cost,
        ..standard()
    };
    let seeing = pct(&c(Discrimination::SameOther, 0.02), 2000, "cooperation");
    let blind = pct(&c(Discrimination::None, 0.02), 2000, "cooperation");
    let blind_3 = pct(&c(Discrimination::None, 0.03), 2000, "cooperation");
    println!("cost 2%: seeing {seeing:.1}, blind {blind:.1}; blind at 3%: {blind_3:.1}");
    assert!(near(seeing, 64.7, 0.06) && near(blind, 41.8, 0.06) && near(blind_3, 12.7, 0.06));
    assert!(blind < seeing && blind > 30.0);
}

/// HKS13's chi-square dominance at one cycle (p < .01): the index (E, H, S,
/// T) of a strategy that both beats uniform (3 df) and beats the runner-up
/// (1 df).
fn dominant(counts: [f64; 4]) -> Option<usize> {
    let n: f64 = counts.iter().sum();
    if n < 1.0 {
        return None;
    }
    let e = n / 4.0;
    let chi3: f64 = counts.iter().map(|o| (o - e).powi(2) / e).sum();
    let mut order = [0, 1, 2, 3];
    order.sort_by(|&a, &b| counts[b].total_cmp(&counts[a]));
    let (a, b) = (counts[order[0]], counts[order[1]]);
    let chi1 = if a + b > 0.0 {
        (a - b).powi(2) / (a + b)
    } else {
        0.0
    };
    (chi3 > 11.345 && chi1 > 6.635).then_some(order[0])
}

/// Each cycle's dominant strategy (index 0 = the start).
fn dominance(w: &EthnoWorld) -> Vec<Option<usize>> {
    let names = ["ethnocentric", "humanitarian", "selfish", "traitorous"];
    let pop = w.stats.series("population").unwrap();
    let shares: Vec<Vec<f64>> = names.iter().map(|n| w.stats.series(n).unwrap()).collect();
    (0..pop.len())
        .map(|t| {
            if pop[t] == 0.0 {
                return None;
            }
            dominant(std::array::from_fn(|k| (shares[k][t] * pop[t]).round()))
        })
        .collect()
}

#[test]
#[ignore]
fn hks13_ethnocentrics_dominate_from_about_period_300() {
    // HKS13 Study 1 (50 worlds, 1,000 cycles): final shares .08 selfish,
    // .02 traitorous, .73 ethnocentric, .17 humanitarian; ethnocentric
    // dominance "at around 300" as the population saturates just under
    // 1,600. Measured: 7.7/2.6/72.4/17.3; population 1,568; E dominant
    // (chi-square, p < .01) for 100 cycles running from a median of cycle 282.
    let v = each_seed(&standard(), 50, 1000, |w| {
        let dom = dominance(w);
        let onset = (1..=901).find(|&t| (t..t + 100).all(|u| dom[u] == Some(0)));
        let shares = [
            "selfish",
            "traitorous",
            "ethnocentric",
            "humanitarian",
            "population",
        ]
        .map(|n| last_100(w, n));
        (onset, shares)
    });
    let share = |k: usize| mean(&v.iter().map(|x| x.1[k]).collect::<Vec<_>>());
    let (s, t, e, h, pop) = (share(0), share(1), share(2), share(3), share(4));
    println!("final S {s:.3} T {t:.3} E {e:.3} H {h:.3}, population {pop:.0}");
    for (ours, hks) in [(s, 0.08), (t, 0.02), (e, 0.73), (h, 0.17)] {
        assert!(near(ours, hks, 0.01), "{ours} vs {hks}");
    }
    assert!(pop > 1500.0 && pop < 1600.0);
    let mut onsets: Vec<usize> = v.iter().map(|x| x.0.expect("every world")).collect();
    onsets.sort_unstable();
    println!("onsets {onsets:?}");
    assert_eq!(onsets[25], 282);
}

#[test]
#[ignore]
fn hks13_early_humanitarian_dominance_in_a_third_of_worlds() {
    // HKS13 Study 3: of 50 worlds, 16 early humanitarian dominance, 16 early
    // ethnocentric, 18 strong competition. Our rule (their chi-square
    // tests, cycles 1–300): humanitarian if H dominates for 50 cycles
    // running and in more cycles than E; ethnocentric if E dominates in at
    // least 150 cycles and more than H; else competition. Measured 17/18/15.
    let v = each_seed(&standard(), 50, 300, |w| {
        let dom = dominance(w);
        let (mut run, mut best) = (0, 0);
        for d in &dom[1..=300] {
            run = if *d == Some(1) { run + 1 } else { 0 };
            best = best.max(run);
        }
        let h = dom[1..=300].iter().filter(|d| **d == Some(1)).count();
        let e = dom[1..=300].iter().filter(|d| **d == Some(0)).count();
        if best >= 50 && h > e {
            0
        } else if e >= 150 && e > h {
            1
        } else {
            2
        }
    });
    let count = |k| v.iter().filter(|&&c| c == k).count();
    println!("H {} E {} competition {}", count(0), count(1), count(2));
    assert_eq!((count(0), count(1), count(2)), (17, 18, 15));
}

#[test]
#[ignore]
fn hks13_study_2_orders_hold_in_every_subset() {
    // Table 3 (10 worlds, last 100 of 2,000): E > H > S > T in every subset
    // but HST, where T beats S. Measured: every order holds; the full set's
    // counts 1184/211/127/38 against 1183/229/123/47 (traitors 20% short).
    let four = Strategy::FOUR;
    let names = ["ethnocentric", "humanitarian", "selfish", "traitorous"];
    for mask in 1u32..16 {
        let allowed: Vec<Strategy> = (0..4)
            .filter(|k| mask & (1 << k) != 0)
            .map(|k| four[k])
            .collect();
        let c = EthnoConfig {
            allowed: allowed.clone(),
            ..standard()
        };
        let v = each_seed(&c, 10, 2000, |w| {
            let pop = last_100(w, "population");
            names.map(|n| last_100(w, n) * pop)
        });
        let counts: Vec<f64> = (0..4)
            .map(|k| mean(&v.iter().map(|x| x[k]).collect::<Vec<_>>()))
            .collect();
        let mut present: Vec<usize> = (0..4).filter(|k| mask & (1 << k) != 0).collect();
        present.sort_by(|&a, &b| counts[b].total_cmp(&counts[a]));
        let order: String = present.iter().map(|&k| four[k].letter()).collect();
        let expected: String = if mask == 0b1110 {
            "HTS".into()
        } else {
            (0..4)
                .filter(|k| mask & (1 << k) != 0)
                .map(|k| four[k].letter())
                .collect()
        };
        println!("{expected}: {order} {counts:.0?}");
        assert_eq!(order, expected);
        if mask == 0b1111 {
            for (ours, table) in counts.iter().zip([1183.0, 229.0, 123.0, 47.0]) {
                assert!((ours - table).abs() <= 0.25 * table, "{ours} vs {table}");
            }
        }
    }
}

#[test]
#[ignore]
fn j13_offspring_anywhere_is_the_null_model() {
    // J13 §3.6: with offspring on a random site the results are "similar to
    // the null model" (12% cooperators with one partner, 3.4% with four).
    // Measured: 4.5% cooperation; 88.7% selfish, 8.3% ethnocentric.
    let c = EthnoConfig {
        offspring: Offspring::Anywhere,
        ..standard()
    };
    let (k, s) = (pct(&c, 2000, "cooperation"), pct(&c, 2000, "selfish"));
    println!("anywhere: cooperation {k:.1}, selfish {s:.1}");
    assert!(near(k, 4.5, 0.06) && near(s, 88.7, 0.06));
}

#[test]
#[ignore]
fn j13_humanitarians_pass_ethnocentrics_near_30_percent_tag_mutation() {
    // J13 §4.4: "At a marker mutation rate of 30%, altruists surpass
    // ethnocentrics"; at 60% traitors pass ethnocentrics. Measured E/H:
    // 54.5/32.2 at 20%, 39.9/44.9 at 30%; E/T 21.8/20.8 at 60%, 14.7/29.8 at 75%.
    let at = |t| {
        let c = EthnoConfig {
            tag_mutation: Some(t),
            ..standard()
        };
        (
            pct(&c, 2000, "ethnocentric"),
            pct(&c, 2000, "humanitarian"),
            pct(&c, 2000, "traitorous"),
        )
    };
    let (e2, h2, _) = at(0.2);
    let (e3, h3, _) = at(0.3);
    let (e75, _, t75) = at(0.75);
    println!("20%: {e2:.1}/{h2:.1}; 30%: {e3:.1}/{h3:.1}; 75%: E {e75:.1} T {t75:.1}");
    assert!(e2 > h2 && h3 > e3 && t75 > e75);
    assert!(near(e3, 39.9, 0.06) && near(h3, 44.9, 0.06));
}

#[test]
#[ignore]
fn j13_markers_track_kinship() {
    // J13 Table 4: relatives are 74.7% of neighbouring pairs; P(same marker
    // | relative) 95.3%, P(relative | same marker) 89.2%; 89% of an
    // ethnocentric's help goes to kin. Measured 75.4, 95.1, 90.1 and 86.8
    // (all helps).
    let c = standard();
    let m = |n| pct(&c, 2000, n);
    let (r, tr, rt, kh) = (
        m("relatives"),
        m("tag_given_relative"),
        m("relative_given_tag"),
        m("kin_help"),
    );
    println!("relatives {r:.1} p(i|r) {tr:.1} p(r|i) {rt:.1} kin help {kh:.1}");
    assert!(
        near(r, 74.7, 1.0) && near(tr, 95.3, 1.0) && near(rt, 89.2, 1.0) && near(kh, 89.0, 2.5)
    );
}

#[test]
#[ignore]
fn j13_kin_discriminators_win_by_less_than_table_5_unless_the_basis_is_fixed() {
    // J13 Table 5: kin 76.2%, ingroup (E) 16.4%. J13 does not say how the
    // basis is inherited. Mutating like the other bits: kin 52.1%, E 26.7%.
    // Fixed at immigration: kin 65.5%, E 12.6% — nearer Table 5.
    let at = |kin_basis| {
        let c = EthnoConfig {
            kin_strategies: true,
            kin_basis,
            ..standard()
        };
        (pct(&c, 2000, "kin"), pct(&c, 2000, "ethnocentric"))
    };
    let (kin, e) = at(KinBasis::Mutates);
    let (kin_f, e_f) = at(KinBasis::Fixed);
    println!("mutates: kin {kin:.1} E {e:.1}; fixed: kin {kin_f:.1} E {e_f:.1}");
    assert!(near(kin, 52.1, 0.06) && near(e, 26.7, 0.06));
    assert!(near(kin_f, 65.5, 0.06) && near(e_f, 12.6, 0.06));
    assert!((kin_f - 76.2).abs() < (kin - 76.2).abs());
}

#[test]
#[ignore]
fn j13_more_markers_close_the_gap_sooner_than_36() {
    // J13 §5.3: the kin–ingroup gap falls below ten points at 36 markers.
    // Measured (kin − E, seeds 1–10), mutating basis: 25.4 at 4, 11.6 at 8,
    // 7.0 at 12, 14.4 at 16, then −1.3 to 6.7 from 20 to 40 — below ten from
    // 12 or 20, not 36. Fixed basis: 52.9 at 4, 25.5 at 8, then between −1.2
    // and 21.2 (s.e. 8–11) from 12 to 36, and −15.3 at 40; over 20 seeds the
    // gap stays 10–16 at 16–36 but for 1.5 at 24, and is −3.7 at 40.
    let gap = |colors, kin_basis| {
        let c = EthnoConfig {
            kin_strategies: true,
            kin_basis,
            colors,
            ..standard()
        };
        pct(&c, 2000, "kin") - pct(&c, 2000, "ethnocentric")
    };
    let (g4, g20, g36) = (
        gap(4, KinBasis::Mutates),
        gap(20, KinBasis::Mutates),
        gap(36, KinBasis::Mutates),
    );
    let (f4, f40) = (gap(4, KinBasis::Fixed), gap(40, KinBasis::Fixed));
    println!("mutates: 4 {g4:.1} 20 {g20:.1} 36 {g36:.1}; fixed: 4 {f4:.1} 40 {f40:.1}");
    assert!(g4 > 20.0 && g20 < 10.0 && g36 < 10.0);
    assert!(near(f4, 52.9, 0.1) && f40 < 0.0);
}
