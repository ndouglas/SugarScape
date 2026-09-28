//! Threshold models (milestone 25): Granovetter's crowds (1978) and Watts's
//! global cascades (2002). Episodes are run in parallel, one world per seed,
//! each until it has finished its share of episodes; an episode's size is
//! read from the `last_size` series when `episodes` counts it.

use sugarscape_core::model::{ModelConfig, ModelWorld};
use sugarscape_core::thresholds::{
    self, Ceilings, Clusters, Crowd, Distribution, Friends, Network, Population, Rounding,
    ThresholdsConfig, Trigger, Update, Zero,
};

use crate::claim::{all_of, equivalent, greater, Claim, Outcome, Source, Verdict};
use crate::runner::model_after;

const GRANOVETTER: &str = "Granovetter 1978, AJS 83(6)";
const WATTS: &str = "Watts 2002, PNAS 99(9)";

fn outcome(holds: bool, measured: String) -> Outcome {
    Outcome {
        verdict: if holds {
            Verdict::Holds
        } else {
            Verdict::Fails
        },
        measured,
        detail: String::new(),
    }
}

fn config(edit: impl FnOnce(&mut ThresholdsConfig)) -> ThresholdsConfig {
    let mut c = ThresholdsConfig::default();
    edit(&mut c);
    c
}

/// Final shares of `per_seed` episodes on each of `seeds` worlds (repeat on).
fn episodes(c: &ThresholdsConfig, seeds: u64, per_seed: u32) -> Vec<f64> {
    let c = ThresholdsConfig {
        repeat: true,
        ..c.clone()
    };
    let out: Vec<Vec<f64>> = std::thread::scope(|s| {
        let hs: Vec<_> = (1..=seeds)
            .map(|seed| {
                let c = c.clone();
                s.spawn(move || {
                    let mut w = ModelWorld::new(ModelConfig::Thresholds(c), seed).unwrap();
                    let mut sizes = Vec::new();
                    let mut seen = 0.0;
                    while sizes.len() < per_seed as usize {
                        w.model_mut().run(1);
                        let m = w.model();
                        let e = m.latest_value("episodes").unwrap();
                        if e > seen {
                            seen = e;
                            sizes.push(m.latest_value("last_size").unwrap());
                        }
                    }
                    sizes
                })
            })
            .collect();
        hs.into_iter().map(|h| h.join().unwrap()).collect()
    });
    out.concat()
}

/// The first episode's final number acting (one world, run to rest).
fn settle(c: &ThresholdsConfig) -> f64 {
    let mut w = ModelWorld::new(ModelConfig::Thresholds(c.clone()), 1).unwrap();
    for _ in 0..100_000 {
        w.model_mut().run(1);
        if w.model().latest_value("episodes").unwrap() >= 1.0 {
            break;
        }
    }
    w.model().latest_value("acting").unwrap() * f64::from(c.actors)
}

fn share(v: &[f64], f: impl Fn(f64) -> bool) -> f64 {
    v.iter().filter(|&&x| f(x)).count() as f64 / v.len() as f64
}

/// The smallest σ (to 0.0001) at which a quantile crowd of 100 riots past half.
fn tipping(rounding: Rounding) -> f64 {
    let rioting = |sd: f64| {
        settle(&config(|c| {
            c.distribution = Distribution::Normal;
            c.sd = sd;
            c.rounding = rounding;
        })) > 50.0
    };
    let (mut lo, mut hi) = (0.10, 0.15);
    while hi - lo > 1e-4 {
        let mid = (lo + hi) / 2.0;
        if rioting(mid) {
            hi = mid;
        } else {
            lo = mid;
        }
    }
    hi
}

fn watts(n: u32, z: f64) -> ThresholdsConfig {
    config(|c| {
        c.actors = n;
        c.distribution = Distribution::Fixed;
        c.mean = 0.18;
        c.network = Network::Random;
        c.degree = z;
        c.trigger = Trigger::Random;
        c.update = Update::Asynchronous;
    })
}

fn friends(distribution: Distribution, a: f64, w: u32, symmetric: bool) -> ThresholdsConfig {
    config(|c| {
        c.distribution = distribution;
        c.friends = Friends {
            enabled: true,
            acquaintance: a,
            weight: w,
            symmetric,
        };
    })
}

/// The most common number acting among episodes (sizes as shares of 100).
fn mode(v: &[f64]) -> usize {
    let mut counts = [0u32; 101];
    for x in v {
        counts[(x * 100.0).round() as usize] += 1;
    }
    (0..=100)
        .max_by_key(|&k| (counts[k], std::cmp::Reverse(k)))
        .unwrap()
}

pub fn claims() -> Vec<Claim> {
    vec![
        Claim {
            id: "thresholds.gr.uniform",
            item: "gr-uniform",
            source: Source::Book,
            citation: GRANOVETTER,
            text: "The uniform crowd (thresholds 0 to 99): 'The equilibrium is 100'; replacing the person at 1 by one at 2, 'the riot ends at that point, with one rioter'",
            check: |_| {
                let u = settle(&ThresholdsConfig::default());
                let p = settle(&config(|c| c.distribution = Distribution::Perturbed));
                outcome(u == 100.0 && p == 1.0, format!("{u:.0} and {p:.0}"))
            },
        },
        Claim {
            id: "thresholds.gr.fig2",
            item: "gr-normal-13",
            source: Source::Book,
            citation: GRANOVETTER,
            text: "Figure 2 (100 people, normal thresholds, mean 25): below σc ≈ 12.2 the equilibrium 'increases gradually to about six', then 'jumps to nearly 100, after which it declines' toward 50 (the continuous calculation)",
            check: |_| {
                let r = |sd: f64| thresholds::granovetter(100, 0.25, sd);
                let (lo, hi) = (r(0.122), r(0.123));
                all_of(vec![
                    ("σc between 12.2 and 12.3".into(), outcome(lo < 10.0 && hi > 90.0, format!("{lo:.2} at 12.2, {hi:.2} at 12.3"))),
                    ("about six below".into(), outcome((4.5..=7.0).contains(&lo), format!("{lo:.2}"))),
                    ("nearly 100 above".into(), outcome(r(0.15) > 99.0, format!("{:.2} at σ 15", r(0.15)))),
                    ("declines toward 50".into(), outcome(r(0.5) < r(0.3) && (r(10.0) - 50.0).abs() < 2.0, format!("{:.1}, {:.1}, {:.1}, {:.1} at σ 30, 50, 100, 1 000", r(0.3), r(0.5), r(1.0), r(10.0)))),
                ])
            },
        },
        Claim {
            id: "thresholds.gr.people",
            item: "gr-sd",
            source: Source::Book,
            citation: GRANOVETTER,
            text: "The same for a crowd of 100 people (the normal's quantiles as thresholds): it tips near σ 12.2 whether thresholds stay fractions or become whole people (rounded down, or to the nearest)",
            check: |_| {
                let parts = [("fractions", Rounding::Exact), ("rounded down", Rounding::Floor), ("rounded", Rounding::Nearest)]
                    .into_iter()
                    .map(|(name, r)| {
                        let t = tipping(r) * 100.0;
                        (name.to_string(), outcome((t - 12.2).abs() <= 0.1, format!("σc {t:.2}")))
                    })
                    .collect();
                all_of(parts)
            },
        },
        Claim {
            id: "thresholds.gr.jump",
            item: "gr-normal-sampled",
            source: Source::Book,
            citation: GRANOVETTER,
            text: "'a slight perturbation of the normal distribution around the critical standard deviation should have a wholly discontinuous, striking qualitative effect' (crowds of 100 drawn from the normal: the chance of a riot past half jumps by at least one half between σ 12 and 12.5; 1 000 crowds each)",
            check: |_| {
                let p = |sd: f64| {
                    let v = episodes(
                        &config(|c| {
                            c.distribution = Distribution::Normal;
                            c.sd = sd;
                            c.crowd = Crowd::Sampled;
                        }),
                        10,
                        100,
                    );
                    share(&v, |x| x > 0.5)
                };
                let (a, b, lo, hi) = (p(0.12), p(0.125), p(0.10), p(0.16));
                outcome(b - a >= 0.5, format!("{a:.2} at σ 12, {b:.2} at 12.5 ({lo:.2} at 10, {hi:.2} at 16)"))
            },
        },
        Claim {
            id: "thresholds.gr.city",
            item: "gr-city",
            source: Source::Book,
            citation: GRANOVETTER,
            text: "Crowds of 100 drawn from a uniform city: no instigator (.37), one instigator and no one at 1 % (.14), 'in over half the cases (.37 + .14 = .51) the equilibrium result is either no rioters or one rioter' (within .03; 5 000 crowds)",
            check: |_| {
                let v = episodes(&config(|c| c.population = Population::City), 10, 500);
                let (none, one) = (share(&v, |x| x == 0.0), share(&v, |x| x == 0.01));
                let all = share(&v, |x| x == 1.0);
                let mean = v.iter().sum::<f64>() / v.len() as f64;
                outcome(
                    (none - 0.37).abs() <= 0.03 && (one - 0.14).abs() <= 0.03 && (none + one - 0.51).abs() <= 0.03,
                    format!("{none:.3} + {one:.3} = {:.3}", none + one),
                )
                .with(&format!("But everyone riots in only {:.1} % of crowds; the mean is {:.1} rioters. Spilerman's (.90)^10 = {:.3}.", 100.0 * all, 100.0 * mean, 0.9f64.powi(10)))
            },
        },
        Claim {
            id: "thresholds.gr.friends-weight",
            item: "gr-friends-perturbed",
            source: Source::Book,
            citation: GRANOVETTER,
            text: "The perturbed crowd with friends: 'the null hypothesis [one rioter] becomes increasingly improbable as the weight attached to friends' behavior increases' (acquaintance ¼; weights 1, 2, 5, 10; 1 000 crowds each)",
            check: |_| {
                let p: Vec<f64> = [1, 2, 5, 10]
                    .into_iter()
                    .map(|w| share(&episodes(&friends(Distribution::Perturbed, 0.25, w, true), 10, 100), |x| x > 0.01))
                    .collect();
                outcome(p.windows(2).all(|w| w[1] >= w[0]) && p[3] > p[0], format!("P(more than one) {:.2}, {:.2}, {:.2}, {:.2}", p[0], p[1], p[2], p[3]))
            },
        },
        Claim {
            id: "thresholds.gr.friends-quarter",
            item: "gr-friends",
            source: Source::Book,
            citation: GRANOVETTER,
            text: "'the largest effects occur where people know, on the average, about one-quarter of the rest of the group' (the perturbed crowd, weight 5: the chance of more than one rioter peaks at acquaintance ¼ among 0.05, 0.1, 0.25, 0.5, 0.9)",
            check: |_| {
                let a = [0.05, 0.1, 0.25, 0.5, 0.9];
                let p: Vec<f64> = a
                    .iter()
                    .map(|&a| share(&episodes(&friends(Distribution::Perturbed, a, 5, true), 10, 100), |x| x > 0.01))
                    .collect();
                let top = (0..5).max_by(|&i, &j| p[i].total_cmp(&p[j])).unwrap();
                outcome(a[top] == 0.25, format!("{:?} at {:?}", p.iter().map(|x| (x * 100.0).round() / 100.0).collect::<Vec<_>>(), a))
            },
        },
        Claim {
            id: "thresholds.gr.friends-symmetry",
            item: "gr-friends",
            source: Source::Book,
            citation: GRANOVETTER,
            text: "'the symmetry of ties has little effect on outcomes' (the perturbed crowd, acquaintance ¼, weight 5: the chance of more than one rioter the same within 0.1, symmetric or one-way; 10 × 100 crowds)",
            check: |_| {
                let per_seed = |sym: bool| {
                    let v = episodes(&friends(Distribution::Perturbed, 0.25, 5, sym), 10, 100);
                    v.chunks(100).map(|c| share(c, |x| x > 0.01)).collect::<Vec<f64>>()
                };
                equivalent(&per_seed(true), &per_seed(false), Some(0.1), "mutual", "one-way")
            },
        },
        Claim {
            id: "thresholds.gr.friends-small",
            item: "gr-friends-perturbed",
            source: Source::Book,
            citation: GRANOVETTER,
            text: "The perturbed crowd with friends: 'the equilibrium changes very little and rarely exceeds five to 10 rioters' (the 95th percentile at most 10; acquaintance ¼, weight 5)",
            check: |_| {
                let mut v = episodes(&friends(Distribution::Perturbed, 0.25, 5, true), 10, 100);
                v.sort_by(f64::total_cmp);
                let p95 = v[(0.95 * v.len() as f64) as usize] * 100.0;
                outcome(p95 <= 10.0, format!("95th percentile {p95:.0} rioters; largest {:.0}", v.last().unwrap() * 100.0))
            },
        },
        Claim {
            id: "thresholds.gr.friends-uniform",
            item: "gr-friends",
            source: Source::Book,
            citation: GRANOVETTER,
            text: "The uniform crowd with friends: 'its equilibrium of 100 rioters is unstable against almost any kind of social structural influence. For most combinations of weights and acquaintance volume tested, the modal equilibrium result is one rioter' (weights 2, 5 × acquaintance 0.05, 0.1, 0.25, 0.5, 0.9)",
            check: |_| {
                let mut ones = 0;
                let mut modes = Vec::new();
                for w in [2, 5] {
                    for a in [0.05, 0.1, 0.25, 0.5, 0.9] {
                        let m = mode(&episodes(&friends(Distribution::Uniform, a, w, true), 10, 50));
                        modes.push(m);
                        if m == 1 {
                            ones += 1;
                        }
                    }
                }
                outcome(ones * 2 > modes.len(), format!("the mode is one rioter in {ones} of {} settings: {modes:?}", modes.len()))
            },
        },
        Claim {
            id: "thresholds.gr.no-oscillation",
            item: "gr-city",
            source: Source::Book,
            citation: GRANOVETTER,
            text: "'where no provision has been made for removal of participants, oscillatory behavior of r(t) is not possible, and an equilibrium will always be reached' (5 000 crowds from the city, with friends, and sampled normal crowds all settle before 1 000 steps)",
            check: |_| {
                let v: Vec<f64> = [
                    config(|c| c.population = Population::City),
                    friends(Distribution::Uniform, 0.25, 2, false),
                    config(|c| {
                        c.distribution = Distribution::Normal;
                        c.crowd = Crowd::Sampled;
                    }),
                ]
                .iter()
                .map(|c| {
                    let mut w = ModelWorld::new(ModelConfig::Thresholds(ThresholdsConfig { repeat: true, ..c.clone() }), 1).unwrap();
                    w.model_mut().run(20_000);
                    w.model().series("step").unwrap().into_iter().fold(0.0, f64::max)
                })
                .collect();
                outcome(v.iter().all(|&s| s < 1000.0), format!("longest episodes {:.0}, {:.0}, {:.0} steps", v[0], v[1], v[2]))
            },
        },
        Claim {
            id: "thresholds.gr.ceilings",
            item: "gr-ceilings",
            source: Source::Book,
            citation: GRANOVETTER,
            text: "Figure 3's curve crossing zero twice (a share leave once participation passes 90 %): 'equilibria cannot be guaranteed … aggregate behavior oscillated' (the uniform crowd, synchronous updating: most of 40 crowds never settle, at shares 0.1, 0.3, 0.5)",
            check: |_| {
                let parts = [0.1, 0.3, 0.5]
                    .into_iter()
                    .map(|q| {
                        let c = config(|c| c.ceilings = Ceilings { share: q, at: 0.9 });
                        let pulsing: Vec<(bool, f64)> = model_after(&ModelConfig::Thresholds(c), &(1..=40).collect::<Vec<u64>>(), 600, |w| {
                            let m = w.model();
                            (m.latest_value("episodes").unwrap() == 0.0, m.latest_value("swing").unwrap())
                        });
                        let n = pulsing.iter().filter(|p| p.0).count();
                        let swing = pulsing.iter().filter(|p| p.0).map(|p| p.1).sum::<f64>() / n.max(1) as f64;
                        (format!("share {q}"), outcome(n > 20, format!("{n} of 40 pulse, swinging by {swing:.2}")))
                    })
                    .collect();
                all_of(parts).with("Crowds that settle rest at 91; whether one pulses depends on who holds the ceilings. Asynchronous updating hovers near 90 instead.")
            },
        },
        Claim {
            id: "thresholds.gr.movement",
            item: "gr-movement",
            source: Source::Book,
            citation: GRANOVETTER,
            text: "'what level of movement among clusters would have the most incendiary effect … small movements among clusters may have greater effects than large ones' (10 crowds of 100 from the city: more rioters at movement 0.05 than at 0 and at 1; the mean over the last 100 of 400 steps, 20 runs)",
            check: |_| {
                let at = |m: f64| -> Vec<f64> {
                    let c = config(|c| {
                        c.population = Population::City;
                        c.clusters = Clusters { enabled: true, count: 10, movement: m };
                    });
                    model_after(&ModelConfig::Thresholds(c), &(1..=20).collect::<Vec<u64>>(), 400, |w| w.model().latest_value("recent_mean").unwrap())
                };
                let (none, some, all) = (at(0.0), at(0.05), at(1.0));
                all_of(vec![
                    ("against none".into(), greater(&some, &none, "m 0.05", "m 0")),
                    ("against full mixing".into(), greater(&some, &all, "m 0.05", "m 1")),
                ])
            },
        },
        Claim {
            id: "watts.window",
            item: "watts-window",
            source: Source::Book,
            citation: WATTS,
            text: "Fig. 1 (φ* 0.18, n 10 000, single seeds): global cascades (over 10 %) occur inside the analytic window zQ(K* − 1, z) = 1 and not outside it (100 seeds at each z)",
            check: |_| {
                let (lo, hi) = thresholds::poisson_window(0.18, 0.0, 0.5, 10.0).unwrap();
                let f = |z: f64| share(&episodes(&watts(10_000, z), 10, 10), |x| x >= 0.1);
                let inside: Vec<f64> = [1.5, 3.0, 5.0].into_iter().map(f).collect();
                let outside: Vec<f64> = [0.8, 7.0].into_iter().map(f).collect();
                let edge = f(6.14);
                outcome(inside.iter().all(|&x| x > 0.3) && outside.iter().all(|&x| x < 0.05), format!("analytic window z {lo:.2}–{hi:.2}; global in {inside:?} of seeds at z 1.5, 3, 5; {outside:?} at 0.8, 7"))
                    .with(&format!("Past the analytic upper edge, at z 6.14: {:.2}. The simulated window reaches further, as Watts says (\"do not agree perfectly\").", edge))
            },
        },
        Claim {
            id: "watts.size",
            item: "watts-middle",
            source: Source::Book,
            citation: WATTS,
            text: "Fig. 2: the average size of global cascades is governed by 'S, the connectivity of the network as a whole' (z 3: within 0.03 of S = 1 − e^−zS)",
            check: |_| {
                let v = episodes(&watts(10_000, 3.0), 10, 10);
                let g: Vec<f64> = v.into_iter().filter(|&x| x >= 0.1).collect();
                let mean = g.iter().sum::<f64>() / g.len() as f64;
                let mut s: f64 = 0.5;
                for _ in 0..200 {
                    s = 1.0 - (-3.0 * s).exp();
                }
                outcome((mean - s).abs() <= 0.03, format!("{mean:.3} against S {s:.3} ({} global of 100)", g.len()))
            },
        },
        Claim {
            id: "watts.fig3-lower",
            item: "watts-lower",
            source: Source::Book,
            citation: WATTS,
            text: "Fig. 3 (n 1 000, z 1.05): 'cascades at the lower critical point are power-law distributed, with slope 3/2 (the cumulative distribution has slope 1/2)' (slope of log P(size ≥ s) over s 2–64 within 0.1 of −½; 3 000 seeds)",
            check: |_| {
                let v = episodes(&watts(1000, 1.05), 10, 300);
                let xs = [2.0, 4.0, 8.0, 16.0, 32.0, 64.0];
                let pts: Vec<(f64, f64)> = xs.iter().map(|&x| (x, share(&v, |s| s * 1000.0 >= x - 1e-9))).filter(|p| p.1 > 0.0).map(|(x, p)| (f64::ln(x), p.ln())).collect();
                let n = pts.len() as f64;
                let (mx, my) = (pts.iter().map(|p| p.0).sum::<f64>() / n, pts.iter().map(|p| p.1).sum::<f64>() / n);
                let b = pts.iter().map(|p| (p.0 - mx) * (p.1 - my)).sum::<f64>() / pts.iter().map(|p| (p.0 - mx).powi(2)).sum::<f64>();
                outcome((b + 0.5).abs() <= 0.1, format!("cumulative slope {b:.2}"))
            },
        },
        Claim {
            id: "watts.fig3-upper",
            item: "watts-upper",
            source: Source::Book,
            citation: WATTS,
            text: "Fig. 3 (n 1 000, z 6.14): the distribution is bimodal, 'a single cascade occurring in 1,000 random trials' (at most 1 % of 1 000 seeds global)",
            check: |_| {
                let v = episodes(&watts(1000, 6.14), 10, 100);
                let g = share(&v, |x| x >= 0.5);
                let tiny = share(&v, |x| x < 0.01);
                outcome(g <= 0.01, format!("{:.1} % global (half the network or more), {:.1} % under 1 %", 100.0 * g, 100.0 * tiny))
                    .with(&format!("At n 10 000: {:.1} %. The upper edge moves with the network's size.", 100.0 * share(&episodes(&watts(10_000, 6.14), 10, 10), |x| x >= 0.5)))
            },
        },
        Claim {
            id: "watts.fig4a",
            item: "watts-hetero",
            source: Source::Book,
            citation: WATTS,
            text: "Fig. 4a: normally distributed thresholds (σ 0.1) 'cause the system to be less stable, yielding cascades over a greater range of both φ and z' (more global cascades than fixed thresholds at z 8 and at z 1.2; n 2 000, zero thresholds acting when reached)",
            check: |_| {
                let hetero = |z: f64| {
                    let mut c = watts(2000, z);
                    c.distribution = Distribution::Normal;
                    c.sd = 0.1;
                    c.crowd = Crowd::Sampled;
                    c.zero = Zero::WhenReached;
                    c
                };
                let f = |c: &ThresholdsConfig| {
                    let v = episodes(c, 10, 30);
                    v.chunks(30).map(|c| share(c, |x| x >= 0.1)).collect::<Vec<f64>>()
                };
                all_of(vec![
                    ("dense (z 8)".into(), greater(&f(&hetero(8.0)), &f(&watts(2000, 8.0)), "σ 0.1", "fixed")),
                    ("sparse (z 1.2)".into(), greater(&f(&hetero(1.2)), &f(&watts(2000, 1.2)), "σ 0.1", "fixed")),
                ])
            },
        },
        Claim {
            id: "watts.fig4b",
            item: "watts-window",
            source: Source::Book,
            citation: WATTS,
            text: "Fig. 4b: power-law degrees p_k = Ck^−τe^−k/κ, τ 2.5, with 'κ0 … adjusted to generate graphs with variable z', are 'much less vulnerable' (the cascade condition at φ* 0.18 fails for every buildable z, and no simulated global cascades at z 1.5)",
            check: |_| {
                let best = (101..195).map(|z| thresholds::power_law_ratio(f64::from(z) / 100.0, 0.18, 0.0)).fold(0.0, f64::max);
                let mut c = watts(10_000, 1.5);
                c.network = Network::PowerLaw;
                let g = share(&episodes(&c, 10, 20), |x| x >= 0.1);
                outcome(best < 1.0 && g == 0.0, format!("largest G0''(1)/z {best:.2}; {:.0} % global at z 1.5", 100.0 * g))
                    .with("But the family's mean degree cannot exceed ζ(1.5)/ζ(2.5) ≈ 1.95 for any κ: the figure's z beyond that cannot be built as stated.")
            },
        },
        Claim {
            id: "watts.hubs",
            item: "watts-targeting",
            source: Source::Book,
            citation: WATTS,
            text: "'the most connected nodes are far more likely than average nodes to trigger cascades, but not in the second regime' (n 2 000: the hub triggers more global cascades than a random seed at z 1.3, and not at z 5.5; 10 × 30 trials)",
            check: |_| {
                let f = |z: f64, t: Trigger| {
                    let mut c = watts(2000, z);
                    c.trigger = t;
                    episodes(&c, 10, 30).chunks(30).map(|c| share(c, |x| x >= 0.1)).collect::<Vec<f64>>()
                };
                let low = greater(&f(1.3, Trigger::Hub), &f(1.3, Trigger::Random), "hub", "random");
                let high = greater(&f(5.5, Trigger::Hub), &f(5.5, Trigger::Random), "hub", "random");
                let not_high = Outcome {
                    verdict: if high.verdict == Verdict::Holds { Verdict::Fails } else { Verdict::Holds },
                    measured: high.measured,
                    detail: String::new(),
                };
                all_of(vec![("sparse (z 1.3)".into(), low), ("dense (z 5.5): no advantage".into(), not_high)])
            },
        },
    ]
}
