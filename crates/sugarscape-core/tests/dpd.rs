//! The demographic Prisoner's Dilemma (milestone 19) against Epstein's
//! working paper (1997) and chapter (GSS 2006, ch. 9 and its appendix) and
//! Radax & Rengs' replication (RR, 2009). Every claim runs over seeds 1–30
//! (Epstein's and RR's 30 runs) in release: `cargo test -p sugarscape-core
//! --release --test dpd -- --ignored --nocapture`. A run's value is its
//! count at cycle 500 unless noted. Thresholds come from the measurements
//! recorded 2026-09-26 (docs/superpowers/plans/2026-09-26-demographic-pd.md,
//! the measurements decision); claims that do not hold are pinned as
//! measured.

use std::thread;

use sugarscape_core::dpd::{
    DeathTiming, DpdConfig, DpdWorld, EndowmentFrom, MetabolismPer, NewbornAge, NewbornsAct,
    Pairing, Play, Removal, Shuffle, Updating,
};
use sugarscape_core::model::Model;

/// Runs `c` for `ticks` from seeds 1–30 in parallel and measures each run.
fn each_seed<T: Send>(c: &DpdConfig, ticks: u32, f: impl Fn(&DpdWorld) -> T + Sync) -> Vec<T> {
    thread::scope(|s| {
        let handles: Vec<_> = (1..=30u64)
            .map(|seed| {
                let f = &f;
                s.spawn(move || {
                    let mut w = DpdWorld::new(c.clone(), seed).unwrap();
                    w.run(ticks);
                    f(&w)
                })
            })
            .collect();
        handles.into_iter().map(|h| h.join().unwrap()).collect()
    })
}

fn last(w: &DpdWorld, name: &str) -> f64 {
    *w.stats.series(name).unwrap().last().unwrap()
}

/// Mean and sample standard deviation.
fn mean_sd(v: &[f64]) -> (f64, f64) {
    let n = v.len() as f64;
    let m = v.iter().sum::<f64>() / n;
    let var = v.iter().map(|x| (x - m).powi(2)).sum::<f64>() / (n - 1.0);
    (m, var.sqrt())
}

/// (cooperators, defectors) at cycle `ticks`: each (mean, s.d.).
fn counts(c: &DpdConfig, ticks: u32) -> ((f64, f64), (f64, f64)) {
    let v = each_seed(c, ticks, |w| (last(w, "cooperators"), last(w, "defectors")));
    let cs: Vec<f64> = v.iter().map(|x| x.0).collect();
    let ds: Vec<f64> = v.iter().map(|x| x.1).collect();
    (mean_sd(&cs), mean_sd(&ds))
}

/// RR's t statistic for the equality of two means of 30 runs with the same
/// unknown variance, (source − ours) / s.e.; |t| < 2.0017 (df 58, α = 0.05)
/// is indistinguishable. With equal samples it is also Welch's statistic.
fn t_stat(source: (f64, f64), ours: (f64, f64)) -> f64 {
    let pooled = (source.1 * source.1 + ours.1 * ours.1) / 2.0;
    (source.0 - ours.0) / (pooled * 2.0 / 30.0).sqrt()
}

const T_CRIT: f64 = 2.0017;

fn near(ours: f64, target: f64, within: f64) -> bool {
    (ours - target).abs() <= within
}

fn run_1() -> DpdConfig {
    DpdConfig::default()
}

fn run_2() -> DpdConfig {
    DpdConfig {
        max_age: 100,
        ..Default::default()
    }
}

/// How each run ends: (coexistence, cooperators only, defectors only, nobody).
fn outcomes(c: &DpdConfig, ticks: u32) -> (usize, usize, usize, usize) {
    let v = each_seed(c, ticks, |w| (last(w, "cooperators"), last(w, "defectors")));
    let count = |f: fn(f64, f64) -> bool| v.iter().filter(|x| f(x.0, x.1)).count();
    (
        count(|c, d| c > 0.0 && d > 0.0),
        count(|c, d| c > 0.0 && d == 0.0),
        count(|c, d| c == 0.0 && d > 0.0),
        count(|c, d| c == 0.0 && d == 0.0),
    )
}

#[test]
#[ignore]
fn tables_1_and_2_are_not_reproduced() {
    // (run, config, ours C, ours D, Epstein C (s.d.), Epstein D (s.d.)).
    for (name, c, oc, od, ec, ed) in [
        ("Run 1", run_1(), 729.2, 170.5, (779.0, 15.0), (121.0, 15.0)),
        ("Run 2", run_2(), 694.8, 195.9, (784.0, 29.0), (99.0, 25.0)),
    ] {
        let (co, de) = counts(&c, 500);
        println!("{name}: C {co:.1?} D {de:.1?} (Epstein {ec:?} {ed:?})");
        assert!(near(co.0, oc, 0.06) && near(de.0, od, 0.06), "{name}");
        // Cooperation dominates, as Epstein's, but both t-tests reject.
        assert!(co.0 > 3.0 * de.0);
        assert!(t_stat(ec, co).abs() > T_CRIT && t_stat(ed, de).abs() > T_CRIT);
    }
}

#[test]
#[ignore]
fn run_1_is_about_4_to_1_by_t_50_not_5_to_1() {
    let (co, de) = counts(&run_1(), 50);
    let ratio = co.0 / de.0;
    println!("t = 50: {:.1} / {:.1} = {ratio:.2}", co.0, de.0);
    // Epstein: "a stable ratio of cooperators to defectors (approximately 5
    // to 1)" by t = 50. Measured 4.34.
    assert!(near(ratio, 4.34, 0.005), "{ratio}");
}

/// GSS Table 9.3 as printed: (T, R, cooperators' mean, 95% CI, range,
/// defectors' mean, 95% CI, range).
type Cell = (
    u32,
    u32,
    f64,
    (f64, f64),
    (f64, f64),
    f64,
    (f64, f64),
    (f64, f64),
);
const TABLE_9_3: [Cell; 45] = [
    (
        10,
        9,
        809.,
        (802., 816.),
        (772., 845.),
        77.,
        (71., 83.),
        (43., 109.),
    ),
    (
        10,
        8,
        748.,
        (738., 757.),
        (707., 802.),
        132.,
        (124., 140.),
        (82., 169.),
    ),
    (
        10,
        7,
        654.,
        (641., 668.),
        (568., 717.),
        198.,
        (188., 209.),
        (154., 280.),
    ),
    (
        10,
        6,
        469.,
        (447., 490.),
        (312., 604.),
        274.,
        (264., 285.),
        (191., 329.),
    ),
    (
        10,
        5,
        258.,
        (240., 277.),
        (150., 383.),
        270.,
        (261., 279.),
        (202., 319.),
    ),
    (
        10,
        4,
        231.,
        (177., 285.),
        (0., 598.),
        199.,
        (172., 225.),
        (0., 287.),
    ),
    (10, 3, 0., (0., 0.), (0., 1.), 0., (0., 0.), (0., 1.)),
    (10, 2, 0., (0., 0.), (0., 0.), 0., (0., 0.), (0., 0.)),
    (10, 1, 0., (0., 0.), (0., 0.), 0., (0., 0.), (0., 0.)),
    (
        9,
        8,
        806.,
        (799., 814.),
        (766., 850.),
        81.,
        (74., 88.),
        (39., 117.),
    ),
    (
        9,
        7,
        728.,
        (716., 740.),
        (669., 793.),
        146.,
        (136., 156.),
        (86., 190.),
    ),
    (
        9,
        6,
        604.,
        (583., 625.),
        (490., 768.),
        225.,
        (212., 239.),
        (102., 303.),
    ),
    (
        9,
        5,
        374.,
        (358., 390.),
        (268., 460.),
        290.,
        (283., 297.),
        (252., 323.),
    ),
    (
        9,
        4,
        203.,
        (175., 231.),
        (98., 442.),
        235.,
        (222., 248.),
        (159., 296.),
    ),
    (9, 3, 35., (3., 67.), (0., 382.), 38., (8., 68.), (0., 284.)),
    (9, 2, 0., (0., 0.), (0., 0.), 0., (0., 0.), (0., 0.)),
    (9, 1, 0., (0., 0.), (0., 0.), 0., (0., 0.), (0., 0.)),
    (
        8,
        7,
        807.,
        (796., 818.),
        (744., 879.),
        80.,
        (70., 89.),
        (14., 142.),
    ),
    (
        8,
        6,
        721.,
        (708., 734.),
        (626., 787.),
        153.,
        (142., 163.),
        (98., 231.),
    ),
    (
        8,
        5,
        530.,
        (516., 544.),
        (460., 604.),
        263.,
        (255., 271.),
        (209., 312.),
    ),
    (
        8,
        4,
        259.,
        (239., 279.),
        (128., 387.),
        271.,
        (263., 279.),
        (227., 313.),
    ),
    (
        8,
        3,
        93.,
        (53., 134.),
        (0., 513.),
        113.,
        (77., 148.),
        (0., 279.),
    ),
    (8, 2, 0., (0., 0.), (0., 0.), 0., (0., 0.), (0., 0.)),
    (8, 1, 0., (0., 0.), (0., 0.), 0., (0., 0.), (0., 0.)),
    (
        7,
        6,
        797.,
        (787., 807.),
        (739., 852.),
        88.,
        (79., 97.),
        (43., 133.),
    ),
    (
        7,
        5,
        668.,
        (652., 684.),
        (542., 782.),
        187.,
        (175., 199.),
        (101., 271.),
    ),
    (
        7,
        4,
        430.,
        (410., 449.),
        (323., 547.),
        286.,
        (277., 295.),
        (244., 345.),
    ),
    (
        7,
        3,
        126.,
        (90., 162.),
        (0., 370.),
        153.,
        (126., 180.),
        (0., 267.),
    ),
    (7, 2, 0., (0., 0.), (0., 0.), 0., (0., 0.), (0., 0.)),
    (7, 1, 0., (0., 0.), (0., 0.), 0., (0., 0.), (0., 0.)),
    (
        6,
        5,
        779.,
        (773., 784.),
        (752., 806.),
        121.,
        (115., 126.),
        (93., 148.),
    ),
    (
        6,
        4,
        587.,
        (576., 599.),
        (524., 658.),
        241.,
        (233., 248.),
        (193., 278.),
    ),
    (
        6,
        3,
        266.,
        (247., 285.),
        (120., 344.),
        280.,
        (270., 291.),
        (199., 320.),
    ),
    (6, 2, 24., (0., 57.), (0., 482.), 13., (0., 27.), (0., 166.)),
    (6, 1, 0., (0., 0.), (0., 0.), 0., (0., 0.), (0., 0.)),
    (
        5,
        4,
        741.,
        (729., 752.),
        (689., 810.),
        136.,
        (126., 146.),
        (79., 179.),
    ),
    (
        5,
        3,
        473.,
        (456., 489.),
        (379., 574.),
        283.,
        (274., 291.),
        (243., 329.),
    ),
    (
        5,
        2,
        125.,
        (70., 180.),
        (0., 589.),
        114.,
        (82., 145.),
        (0., 260.),
    ),
    (5, 1, 0., (0., 0.), (0., 0.), 0., (0., 0.), (0., 0.)),
    (
        4,
        3,
        710.,
        (693., 727.),
        (613., 814.),
        159.,
        (146., 172.),
        (76., 231.),
    ),
    // The printed defectors' CI "(254, 376)" contradicts mean 265, s.d. 32.
    (
        4,
        2,
        282.,
        (262., 302.),
        (171., 380.),
        265.,
        (254., 376.),
        (176., 322.),
    ),
    (4, 1, 0., (0., 0.), (0., 0.), 0., (0., 0.), (0., 0.)),
    (
        3,
        2,
        624.,
        (610., 637.),
        (516., 699.),
        221.,
        (210., 232.),
        (164., 284.),
    ),
    (3, 1, 4., (0., 12.), (0., 123.), 7., (0., 20.), (0., 197.)),
    (
        2,
        1,
        361.,
        (344., 377.),
        (256., 471.),
        280.,
        (273., 288.),
        (234., 337.),
    ),
];

#[test]
#[ignore]
fn table_9_3_cell_by_cell() {
    // A mean counts as inside an interval Epstein printed rounded if it
    // rounds into it.
    let inside = |m: f64, (lo, hi): (f64, f64)| m >= lo - 0.5 && m < hi + 0.5;
    let (mut ci, mut range, mut both_ci, mut lone_defectors) = (0, 0, 0, 0);
    for (t, r, ec, eci, erange, ed, edci, edrange) in TABLE_9_3 {
        let c = DpdConfig {
            t: f64::from(t),
            r: f64::from(r),
            p: -f64::from(r),
            s: -f64::from(t),
            ..Default::default()
        };
        let (co, de) = counts(&c, 500);
        let (c_ci, d_ci) = (inside(co.0, eci), inside(de.0, edci));
        ci += usize::from(c_ci) + usize::from(d_ci);
        range += usize::from(inside(co.0, erange)) + usize::from(inside(de.0, edrange));
        both_ci += usize::from(c_ci && d_ci);
        // Where Epstein's whole population dies out, lone defectors remain.
        if ec == 0.0 && ed == 0.0 && erange.1 == 0.0 && de.0 > 0.0 {
            lone_defectors += 1;
        }
        println!(
            "({t}, {r}): C {:.1} [{ec}] D {:.1} [{ed}] {c_ci} {d_ci}",
            co.0, de.0
        );
    }
    println!("in CI {ci}/90, in range {range}/90, both in CI {both_ci}/45, lone defectors in {lone_defectors} cells");
    assert_eq!((ci, range, both_ci), (22, 51, 1));
    assert_eq!(
        lone_defectors, 11,
        "every all-zero cell keeps a few defectors"
    );
}

#[test]
#[ignore]
fn collapsed_populations_leave_lone_defectors_epstein_counts_none() {
    // Table 9.3 (10, 1): (0, 0) cooperators and defectors in every run. Here
    // the last defectors have nobody to play and no maximum age: they stay.
    let c = DpdConfig {
        t: 10.0,
        r: 1.0,
        p: -1.0,
        s: -10.0,
        ..Default::default()
    };
    let (co, de) = counts(&c, 500);
    assert_eq!(co.0, 0.0);
    assert!(near(de.0, 2.7, 0.06), "{de:?}");
    let (_, de) = counts(&c, 2000);
    assert!(de.0 >= 1.0, "{de:?}");
}

/// Swings of the cooperator count from above 400 to below 100.
fn swings(w: &DpdWorld) -> u32 {
    let (mut high, mut n) = (false, 0);
    for x in w.stats.series("cooperators").unwrap() {
        if x > 400.0 {
            high = true;
        } else if x < 100.0 && high {
            high = false;
            n += 1;
        }
    }
    n
}

#[test]
#[ignore]
fn run_4_dies_out_instead_of_cycling_and_the_paradox_does_not_appear() {
    let r1 = DpdConfig { r: 1.0, ..run_2() };
    // Epstein: predator–prey cycles; coexistence, cooperator monopoly or
    // extinction by seed; monopolies at R = 1 but not at R = 5.
    assert_eq!(outcomes(&r1, 500), (4, 0, 0, 26));
    assert_eq!(outcomes(&r1, 2000), (0, 0, 0, 30));
    let s: Vec<f64> = each_seed(&r1, 2000, |w| f64::from(swings(w)));
    let (m, _) = mean_sd(&s);
    assert!(near(m, 0.6, 0.02), "{m}");
    assert!(s.iter().all(|&x| x <= 2.0));
    assert_eq!(outcomes(&run_2(), 2000), (30, 0, 0, 0));
}

#[test]
#[ignore]
fn run_5_cooperation_persists_through_10000_cycles_in_27_of_30() {
    let c = DpdConfig {
        mutation: 0.5,
        ..run_2()
    };
    let v = each_seed(&c, 10_000, |w| {
        (last(w, "cooperators"), last(w, "population"))
    });
    let alive = v.iter().filter(|x| x.0 > 0.0).count();
    let empty = v.iter().filter(|x| x.1 == 0.0).count();
    assert_eq!((alive, empty), (27, 3), "the others die out entirely");
}

#[test]
#[ignore]
fn soup_runs_to_pure_defection() {
    let c = DpdConfig {
        pairing: Pairing::Soup,
        ..run_1()
    };
    let v = each_seed(&c, 500, |w| {
        let cs = w.stats.series("cooperators").unwrap();
        (cs.iter().position(|&x| x == 0.0), last(w, "population"))
    });
    let gone: Vec<usize> = v.iter().filter_map(|x| x.0).collect();
    assert_eq!(gone.len(), 29);
    assert!(gone.iter().all(|&t| t <= 14));
    assert!(v.iter().all(|x| x.1 <= 1.0), "a lone agent at most");
}

#[test]
#[ignore]
fn shifted_payoffs_do_not_converge_to_pure_defection() {
    let shifted = |c: DpdConfig| DpdConfig {
        t: 12.0,
        r: 11.0,
        p: 1.0,
        s: 0.0,
        ..c
    };
    // GSS: "maximum age of 100, zero mutation": all 30 coexist.
    assert_eq!(outcomes(&shifted(run_2()), 500), (30, 0, 0, 0));
    assert_eq!(outcomes(&shifted(run_2()), 2000), (30, 0, 0, 0));
    let (co, de) = counts(&shifted(run_2()), 500);
    assert!(near(co.0, 418.1, 0.06) && near(de.0, 478.1, 0.06));
    // Run 5's settings (the working paper's context) and Run 1's: the same.
    let run_5 = DpdConfig {
        mutation: 0.5,
        ..run_2()
    };
    assert_eq!(outcomes(&shifted(run_5), 500), (30, 0, 0, 0));
    assert_eq!(outcomes(&shifted(run_1()), 500), (30, 0, 0, 0));
}

#[test]
#[ignore]
fn metabolism_recovers_the_negative_payoffs_per_game_not_per_cycle() {
    let shifted = |per: MetabolismPer| DpdConfig {
        t: 12.0,
        r: 11.0,
        p: 1.0,
        s: 0.0,
        metabolism: 6.0,
        metabolism_per: per,
        ..run_2()
    };
    let per_game = each_seed(&shifted(MetabolismPer::Interaction), 500, |w| {
        w.fingerprint()
    });
    let negative = each_seed(&run_2(), 500, |w| w.fingerprint());
    assert_eq!(per_game, negative, "per game: Run 2 exactly");
    let (co, de) = counts(&shifted(MetabolismPer::Cycle), 500);
    assert!(near(co.0, 644.2, 0.06) && near(de.0, 252.6, 0.06));
    // Cooperative, but distinguishable from Run 2 (694.8 ± 29.4).
    assert!(t_stat((694.8, 29.4), co).abs() > T_CRIT);
}

#[test]
#[ignore]
fn footnote_27_does_not_reach_cooperative_monopoly() {
    for r in [11.0, 15.0] {
        let c = DpdConfig {
            t: 16.0,
            r,
            p: 5.0,
            s: 4.0,
            max_age: 10,
            ..Default::default()
        };
        assert_eq!(outcomes(&c, 500), (30, 0, 0, 0), "R = {r}");
        assert_eq!(outcomes(&c, 2000).1, 1, "R = {r}: one monopoly by 2,000");
    }
}

#[test]
#[ignore]
fn coordination_regions_persist_in_about_half_the_runs() {
    let c = DpdConfig {
        r: 1.0,
        s: -3.0,
        t: -3.0,
        p: 1.0,
        max_age: 1000,
        ..Default::default()
    };
    // Both conventions present, and the share of neighbouring pairs that
    // differ (per mille).
    let measure = |w: &DpdWorld| {
        let (mut unlike, mut pairs) = (0u32, 0u32);
        for s in 0..w.sites() {
            let Some(a) = w.agent_at(s) else { continue };
            for &j in w.geometry.neighbors(s) {
                if let Some(b) = w.agent_at(j as usize) {
                    pairs += 1;
                    unlike += u32::from(a.cooperator != b.cooperator);
                }
            }
        }
        let c = last(w, "cooperators");
        let mixed = c > 0.0 && c < last(w, "population");
        (mixed, 1000.0 * f64::from(unlike) / f64::from(pairs.max(1)))
    };
    for (ticks, persist) in [(500, 17), (2000, 16), (5000, 11)] {
        let v = each_seed(&c, ticks, measure);
        let mixed: Vec<f64> = v.iter().filter(|x| x.0).map(|x| x.1).collect();
        assert_eq!(mixed.len(), persist, "t = {ticks}");
        // Where both persist they are regions: few unlike neighbours.
        assert!(mean_sd(&mixed).0 < 50.0, "t = {ticks}");
    }
}

#[test]
#[ignore]
fn radax_and_rengs_factorial_as_measured() {
    // RR's six model switches (their RNG library excluded), each TRUE/FALSE
    // in their order: remove dead immediately, die immediately, endowment
    // inherited, random birth age, asynchronous updating, Repast shuffle.
    // RR: 1 of 128 settings reproduces Run 1 (synchronous, which they set
    // aside), 7 reproduce Run 2, none both.
    let (mut run1, mut run2, mut both, mut fits2) = (0, 0, 0, Vec::new());
    for bits in 0..64u32 {
        let on = |k: u32| bits & (1 << (5 - k)) == 0;
        let edit = |mut c: DpdConfig| {
            c.removal = if on(0) {
                Removal::Immediate
            } else {
                Removal::EndOfCycle
            };
            c.death_timing = if on(1) {
                DeathTiming::Immediate
            } else {
                DeathTiming::OwnTurn
            };
            c.endowment_from = if on(2) {
                EndowmentFrom::Parent
            } else {
                EndowmentFrom::Granted
            };
            c.newborn_age = if on(3) {
                NewbornAge::Random
            } else {
                NewbornAge::Zero
            };
            c.updating = if on(4) {
                Updating::Asynchronous
            } else {
                Updating::Synchronous
            };
            c.shuffle = if on(5) { Shuffle::Full } else { Shuffle::Swaps };
            c
        };
        let (c1, d1) = counts(&edit(run_1()), 500);
        let (c2, d2) = counts(&edit(run_2()), 500);
        let r1 =
            t_stat((779.0, 15.0), c1).abs() < T_CRIT && t_stat((121.0, 15.0), d1).abs() < T_CRIT;
        let r2 =
            t_stat((784.0, 29.0), c2).abs() < T_CRIT && t_stat((99.0, 25.0), d2).abs() < T_CRIT;
        run1 += usize::from(r1);
        run2 += usize::from(r2);
        both += usize::from(r1 && r2);
        if r2 {
            let flags: String = (0..6).map(|k| if on(k) { 'T' } else { 'F' }).collect();
            fits2.push(flags);
        }
    }
    assert_eq!((run1, run2, both), (0, 1, 0));
    assert_eq!(fits2, ["TFTTTT"]);
    // RR's best Run 2 fit (F T F T T F: 780 ± 25 / 97 ± 22 in Repast) here.
    let best = DpdConfig {
        removal: Removal::EndOfCycle,
        endowment_from: EndowmentFrom::Granted,
        ..run_2()
    };
    let (co, de) = counts(&best, 500);
    assert!(
        near(co.0, 703.0, 0.06) && near(de.0, 163.5, 0.06),
        "{co:?} {de:?}"
    );
}

#[test]
#[ignore]
fn the_working_papers_rule_is_nearer_table_1_but_still_rejected() {
    let wp = |c: DpdConfig| DpdConfig {
        play: Play::RandomNeighbor,
        ..c
    };
    let (co, de) = counts(&wp(run_1()), 500);
    assert!(
        near(co.0, 758.9, 0.06) && near(de.0, 141.0, 0.06),
        "{co:?} {de:?}"
    );
    assert!(t_stat((779.0, 15.0), co).abs() > T_CRIT);
    let (co, de) = counts(&wp(run_2()), 500);
    assert!(
        near(co.0, 737.0, 0.06) && near(de.0, 156.2, 0.06),
        "{co:?} {de:?}"
    );
}

#[test]
#[ignore]
fn three_unsettled_choices_reproduce_both_tables_on_seeds_1_to_30() {
    // The working paper's rule, no initial wealth, newborns acting at once
    // (dpd-closest): both tables pass RR's test, where no timing setting of
    // the published rule does. Over seeds 31–60 Run 2's defectors fail.
    let closest = |c: DpdConfig| DpdConfig {
        play: Play::RandomNeighbor,
        initial_wealth: 0.0,
        newborns_act: NewbornsAct::ThisCycle,
        ..c
    };
    let (c1, d1) = counts(&closest(run_1()), 500);
    let (c2, d2) = counts(&closest(run_2()), 500);
    println!("Run 1 {c1:.1?} {d1:.1?}; Run 2 {c2:.1?} {d2:.1?}");
    assert!(near(c1.0, 786.1, 0.06) && near(d1.0, 113.7, 0.06));
    assert!(near(c2.0, 792.2, 0.06) && near(d2.0, 101.3, 0.06));
    for (source, ours) in [
        ((779.0, 15.0), c1),
        ((121.0, 15.0), d1),
        ((784.0, 29.0), c2),
        ((99.0, 25.0), d2),
    ] {
        assert!(t_stat(source, ours).abs() < T_CRIT, "{source:?} {ours:?}");
    }
}
