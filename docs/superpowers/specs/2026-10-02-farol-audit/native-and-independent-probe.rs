use rand::{rngs::StdRng, Rng, SeedableRng};
use std::{fs::File, io::Write};
use sugarscape_core::{farol::*, model::Model};
fn independent(seed: u64, exact: bool, retain: bool) -> Vec<f64> {
    let mut rng = StdRng::seed_from_u64(seed);
    let n = 1001;
    let mut tab = vec![vec![vec![false; 16]; 5]; n];
    for a in &mut tab {
        for s in a {
            for b in s {
                *b = rng.gen_bool(0.5);
            }
        }
    }
    let mut pts = vec![vec![0.0; 5]; n];
    let mut active = vec![0; n];
    let mut h = rng.gen_range(0..16);
    let mut out = Vec::new();
    for t in 0..5000 {
        let mut action = vec![false; n];
        for i in 0..n {
            let best = pts[i].iter().copied().fold(f64::NEG_INFINITY, f64::max);
            let ties: Vec<usize> = (0..5).filter(|&k| pts[i][k] == best).collect();
            let k = if retain && t > 0 && pts[i][active[i]] == best {
                active[i]
            } else {
                ties[rng.gen_range(0..ties.len())]
            };
            active[i] = k;
            action[i] = tab[i][k][h];
        }
        let a = action.iter().filter(|&&x| x).count();
        let win = a <= 500;
        let x = if win { a } else { n - a };
        let p = if x == 0 {
            0.0
        } else {
            n as f64 / x as f64 - 2.0
        };
        let p = if exact { p } else { p.round() };
        for i in 0..n {
            for k in 0..5 {
                if tab[i][k][h] == win {
                    pts[i][k] += p;
                }
            }
        }
        h = ((h << 1) | usize::from(win)) & 15;
        out.push(a as f64);
    }
    out
}
fn record(label: &str, seed: u64, n: u32, a: &[f64], f: &mut File, hist: &mut File) {
    let c = if label.starts_with("arthur") {
        60.0
    } else {
        n as f64 / 2.0
    };
    let windows = [
        ("full", 0, a.len()),
        ("tail80", a.len() / 5, a.len()),
        ("early10", 0, a.len() / 10),
        ("late10", a.len() * 9 / 10, a.len()),
        (
            "paper1000_2000",
            1000usize.min(a.len()),
            2000usize.min(a.len()),
        ),
    ];
    for (w, l, r) in windows {
        if l == r {
            continue;
        }
        let b = &a[l..r];
        let mean = b.iter().sum::<f64>() / b.len() as f64;
        let fluct = b.iter().map(|x| (x - c).powi(2)).sum::<f64>() / b.len() as f64 / n as f64;
        let variance = b.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / b.len() as f64;
        let lag = b
            .windows(2)
            .map(|x| (x[0] - mean) * (x[1] - mean))
            .sum::<f64>()
            / (variance * b.len() as f64);
        let central = b
            .iter()
            .filter(|&&x| (x - c).abs() <= 0.05 * n as f64)
            .count() as f64
            / b.len() as f64;
        writeln!(
            f,
            "{label},{seed},{n},{},{w},{mean},{fluct},{central},{variance},{lag}",
            a.len()
        )
        .unwrap();
    }
    if label.contains("fig4") {
        let mut counts = vec![0; n as usize + 1];
        for &x in &a[a.len() / 5..] {
            counts[x as usize] += 1;
        }
        for (x, v) in counts.into_iter().enumerate() {
            writeln!(hist, "{label},{seed},{x},{v}").unwrap();
        }
    }
}
fn main() {
    let mode = std::env::args().nth(1).unwrap_or("inverse".into());
    let mut f = File::create(format!("{mode}-results.csv")).unwrap();
    writeln!(
        f,
        "config,seed,n,rounds,window,mean,fluctuation,central_share,variance,lag1"
    )
    .unwrap();
    let mut hist = File::create(format!("{mode}-hist.csv")).unwrap();
    writeln!(hist, "config,seed,attendance,count").unwrap();
    for seed in 1..=if mode == "smr" { 32 } else { 20 } {
        if mode == "inverse" {
            for exact in [false, true] {
                let label = if exact {
                    "fig4-native-exact"
                } else {
                    "fig4-native-rounded"
                };
                let mut c = FarolConfig::default();
                c.game = Game::Minority;
                c.agents = 1001;
                c.memory = 4;
                c.strategies = 5;
                c.payoff = Payoff::Inverse;
                c.rounding = if exact {
                    Rounding::Exact
                } else {
                    Rounding::Nearest
                };
                let mut w = FarolWorld::new(c, seed).unwrap();
                w.run(5000);
                record(
                    label,
                    seed,
                    1001,
                    &w.series("attendance").unwrap()[1..],
                    &mut f,
                    &mut hist,
                );
                for retain in [false, true] {
                    record(
                        &format!(
                            "fig4-independent-{}-{}",
                            if exact { "exact" } else { "rounded" },
                            if retain { "retain" } else { "redraw" }
                        ),
                        seed,
                        1001,
                        &independent(seed, exact, retain),
                        &mut f,
                        &mut hist,
                    );
                }
            }
        } else if mode == "baseline" {
            for shared in [false, true] {
                let mut c = FarolConfig::default();
                c.shared = shared;
                if !shared {
                    c.behavior = Behavior::Random;
                }
                let mut w = FarolWorld::new(c, seed).unwrap();
                w.run(2000);
                let a = w.series("attendance").unwrap();
                let label = if shared {
                    "arthur-shared"
                } else {
                    "arthur-random"
                };
                record(label, seed, 100, &a[1..], &mut f, &mut hist);
                let mut tr = File::create(format!("{label}-seed{seed}.csv")).unwrap();
                for (t, x) in a[1..].iter().enumerate() {
                    writeln!(tr, "{t},{x}").unwrap();
                }
            }
        } else if mode == "arthur" {
            for payoff in [false, true] {
                let mut c = FarolConfig::default();
                c.scoring = if payoff {
                    Scoring::Payoff
                } else {
                    Scoring::Error
                };
                let mut w = FarolWorld::new(c, seed).unwrap();
                w.run(2000);
                let a = w.series("attendance").unwrap();
                record(
                    &format!("arthur-{}", if payoff { "payoff" } else { "accuracy" }),
                    seed,
                    100,
                    &a[1..],
                    &mut f,
                    &mut hist,
                );
                if seed == 1 {
                    let mut trace = File::create(format!(
                        "arthur-native-{}.csv",
                        if payoff { "payoff" } else { "accuracy" }
                    ))
                    .unwrap();
                    for (t, x) in a[1..].iter().enumerate() {
                        writeln!(trace, "{t},{x}").unwrap();
                    }
                }
            }
        } else if mode == "smr" {
            for m in [2, 6, 12] {
                let mut c = FarolConfig::default();
                c.game = Game::Minority;
                c.agents = 101;
                c.strategies = 2;
                c.memory = m;
                let mut w = FarolWorld::new(c, seed).unwrap();
                w.run(10000);
                record(
                    &format!("smr-m{m}"),
                    seed,
                    101,
                    &w.series("attendance").unwrap()[1..],
                    &mut f,
                    &mut hist,
                );
            }
        } else {
            for n in [1001, 101] {
                for every in [10] {
                    for mutation in [0.0, 0.1] {
                        let mut c = FarolConfig::default();
                        c.game = Game::Minority;
                        c.agents = n;
                        c.memory = 6;
                        c.strategies = 5;
                        c.evolution = Evolution {
                            enabled: true,
                            every,
                            strategy_mutation: mutation,
                            memory_mutation: 0.0,
                        };
                        let mut w = FarolWorld::new(c, seed).unwrap();
                        w.run(40000);
                        record(
                            &format!("fig10-every{every}-mutation{mutation}"),
                            seed,
                            n,
                            &w.series("attendance").unwrap()[1..],
                            &mut f,
                            &mut hist,
                        );
                    }
                }
            }
        }
        eprintln!("{mode} seed {seed} done");
    }
}
