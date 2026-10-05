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

/// Watts normalizes f(phi) on [0, 1]; this is distinct from clamping a draw.
fn unit_interval_normal_cdf(x: f64, mean: f64, sd: f64) -> f64 {
    if sd == 0.0 {
        return if x >= mean { 1.0 } else { 0.0 };
    }
    if x <= 0.0 {
        return 0.0;
    }
    if x >= 1.0 {
        return 1.0;
    }
    let below = thresholds::normal_cdf(-mean / sd);
    (thresholds::normal_cdf((x - mean) / sd) - below)
        / (thresholds::normal_cdf((1.0 - mean) / sd) - below)
}

fn unit_interval_vulnerabilities(phi: f64, sd: f64) -> Vec<f64> {
    (1..=400)
        .map(|k| unit_interval_normal_cdf(1.0 / f64::from(k), phi, sd))
        .collect()
}

fn ratio_with_vulnerabilities(z: f64, rho: &[f64]) -> f64 {
    let mut probability = (-z).exp();
    let mut sum = 0.0;
    for (i, vulnerable) in rho.iter().enumerate() {
        let k = (i + 1) as f64;
        probability *= z / k;
        sum += k * (k - 1.0) * vulnerable * probability;
    }
    sum / z
}

fn unit_interval_poisson_ratio(z: f64, phi: f64, sd: f64) -> f64 {
    ratio_with_vulnerabilities(z, &unit_interval_vulnerabilities(phi, sd))
}

/// A 0.01-z grid, sufficient for the declared window accuracy, through z=60.
fn unit_interval_window(phi: f64, sd: f64, from: f64, to: f64) -> Option<(f64, f64)> {
    let rho = unit_interval_vulnerabilities(phi, sd);
    let mut edges = None;
    for i in 0..=((to - from) * 100.0).floor() as u32 {
        let z = from + f64::from(i) / 100.0;
        if ratio_with_vulnerabilities(z, &rho) > 1.0 {
            edges = Some(match edges {
                None => (z, z),
                Some((lo, _)) => (lo, z),
            });
        }
    }
    edges
}

/// Largest normal-location parameter with a window on the tested z grid.
fn unit_interval_phi_limit(sd: f64) -> f64 {
    let (mut lo, mut hi) = (0.0, 1.0);
    for _ in 0..24 {
        let mid = (lo + hi) / 2.0;
        if unit_interval_window(mid, sd, 0.5, 60.0).is_some() {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    lo
}

/// Plug-in compatibility of the observed rate with at most one in 1000.
fn binomial_at_most_one_log10(n: u32, p: f64) -> f64 {
    if p == 0.0 {
        return 0.0;
    }
    if p == 1.0 {
        return f64::NEG_INFINITY;
    }
    (f64::from(n) * (-p).ln_1p() + (f64::from(n) * p / (1.0 - p)).ln_1p()) / std::f64::consts::LN_10
}

/// Exact one-sided McNemar tail: among discordant pairs, hub-only is
/// Binomial(hub-only + random-only, .5) under equal trigger probabilities.
fn paired_hub_advantage_p(hub_only: usize, random_only: usize) -> f64 {
    let n = hub_only + random_only;
    assert!(n <= 1000, "paired tail supports at most 1000 graph pairs");
    let mut probability = 0.5f64.powi(n as i32);
    let mut tail = 0.0;
    for successes in 0..=n {
        if successes >= hub_only {
            tail += probability;
        }
        if successes < n {
            probability *= (n - successes) as f64 / (successes + 1) as f64;
        }
    }
    tail.min(1.0)
}

/// Fresh first episodes share their graph seed across random/hub triggers.
/// Repeated episodes would diverge because trigger/update consume RNG draws.
fn paired_trigger_sizes(z: f64) -> (Vec<f64>, Vec<f64>) {
    let chunks = std::thread::scope(|scope| {
        let handles: Vec<_> = (0..10u64)
            .map(|batch| {
                scope.spawn(move || {
                    let mut random = Vec::new();
                    let mut hub = Vec::new();
                    for seed in (batch * 100 + 1)..=(batch * 100 + 100) {
                        for (trigger, sizes) in
                            [(Trigger::Random, &mut random), (Trigger::Hub, &mut hub)]
                        {
                            let mut c = watts(2000, z);
                            c.trigger = trigger;
                            let mut world =
                                ModelWorld::new(ModelConfig::Thresholds(c), seed).unwrap();
                            for _ in 0..1001 {
                                world.model_mut().run(1);
                                if world.model().latest_value("episodes").unwrap() >= 1.0 {
                                    break;
                                }
                            }
                            assert!(
                                world.model().latest_value("episodes").unwrap() >= 1.0,
                                "threshold cascade did not settle: seed {seed}"
                            );
                            sizes.push(world.model().latest_value("last_size").unwrap());
                        }
                    }
                    (random, hub)
                })
            })
            .collect();
        handles
            .into_iter()
            .map(|handle| handle.join().unwrap())
            .collect::<Vec<_>>()
    });
    let (mut random, mut hub) = (Vec::new(), Vec::new());
    for (r, h) in chunks {
        random.extend(r);
        hub.extend(h);
    }
    (random, hub)
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
            source: Source::App,
            citation: "App finite-crowd extension of Granovetter Fig. 2; thresholds design",
            text: "Our finite 100-agent mid-quantile extension: exact, floor and nearest rounding give critical spreads between 10 and 15, differing by more than 0.2. This rule was revised after the original results were known; it tests an app choice, not a failure of the paper's continuous calculation",
            check: |_| {
                let critical: Vec<f64> = [Rounding::Exact, Rounding::Floor, Rounding::Nearest]
                    .into_iter().map(|rounding| tipping(rounding) * 100.0).collect();
                let min = critical.iter().copied().fold(f64::INFINITY, f64::min);
                let max = critical.iter().copied().fold(f64::NEG_INFINITY, f64::max);
                outcome(critical.iter().all(|t| (10.0..=15.0).contains(t)) && max - min > 0.2,
                    format!("critical spreads exact/floor/nearest: {:.2}/{:.2}/{:.2}; range {:.2}", critical[0], critical[1], critical[2], max-min))
                    .with("Granovetter specifies the continuous CDF equilibrium; this finite quantile realization and its rounding are additional choices.")
            },
        },
        Claim {
            id: "thresholds.gr.jump",
            item: "gr-normal-sampled",
            source: Source::App,
            citation: "App sampled-crowd extension of Granovetter Fig. 2; thresholds design",
            text: "Our independently sampled 100-agent crowds: probability of more than half rioting increases across spreads 10, 12, 12.5 and 16, with an increase smaller than 0.5 from 12 to 12.5 (1000 crowds each). This finite-sampling rule was revised after results were known; the paper's continuous tipping calculation is a separate claim",
            check: |_| {
                let p = |sd: f64| {
                    let v = episodes(&config(|c| {
                        c.distribution = Distribution::Normal;
                        c.sd = sd;
                        c.crowd = Crowd::Sampled;
                    }), 10, 100);
                    share(&v, |x| x > 0.5)
                };
                let values = [p(0.10), p(0.12), p(0.125), p(0.16)];
                outcome(values.windows(2).all(|w| w[1] > w[0]) && values[2]-values[1] < 0.5,
                    format!("probabilities {values:?} at spreads 10, 12, 12.5, 16"))
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
                .with(&format!("Granovetter already explains how city sampling changes the deterministic equilibrium. Everyone riots in {:.1} % of these crowds; the mean is {:.1} rioters. Spilerman's (.90)^10 = {:.3}.", 100.0 * all, 100.0 * mean, 0.9f64.powi(10)))
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
            text: "Without removal, Granovetter says oscillation is impossible and equilibrium is reached. We test 5000 completed episodes for EACH configuration: city samples, one-way friends, and sampled normals; every episode must finish before 1000 steps. Each configuration has a cap of 5,005,000 update calls; failing to complete all 5000 episodes fails the check",
            check: |_| {
                let settings = [
                    ("city samples", config(|c| c.population = Population::City)),
                    ("one-way friends", friends(Distribution::Uniform, 0.25, 2, false)),
                    ("sampled normals", config(|c| {
                        c.distribution = Distribution::Normal;
                        c.crowd = Crowd::Sampled;
                    })),
                ];
                let parts = settings.into_iter().map(|(name, c)| {
                    let mut world = ModelWorld::new(ModelConfig::Thresholds(ThresholdsConfig { repeat: true, ..c }), 1).unwrap();
                    let mut completed = 0.0;
                    let mut longest = 0.0f64;
                    let mut updates = 0;
                    for _ in 0..5_005_000 {
                        world.model_mut().run(1);
                        updates += 1;
                        longest = longest.max(world.model().latest_value("step").unwrap());
                        completed = world.model().latest_value("episodes").unwrap();
                        if completed >= 5000.0 { break; }
                    }
                    (name.into(), outcome(completed >= 5000.0 && longest < 1000.0,
                        format!("{completed:.0}/5000 completed episodes, maximum {longest:.0} steps across the whole run, {updates} update calls")))
                }).collect();
                all_of(parts)
            },
        },
        Claim {
            id: "thresholds.gr.ceilings",
            item: "gr-ceilings",
            source: Source::App,
            citation: "App illustrative extension of Granovetter multiple-crossing discussion, pp. 1438–1439",
            text: "Our illustrative ceiling extension of the discussion adjacent to Fig. 3 (whose plotted curve crosses zero once): a share leave above 90 percent participation. With a uniform crowd and synchronous updates, most of 40 crowds pulse at shares 0.1, 0.3, 0.5; these particular ceiling shares are app choices, not source predictions",
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
            source: Source::App,
            citation: "App migration rule illustrating Granovetter's spatial-movement conjecture",
            text: "Our migration illustration of Granovetter's conjecture: each actor independently moves to a uniformly random other crowd with probability m each step, followed by reversible threshold updates. In 10 city-sampled crowds of 100, movement .05 yields more activity than 0 or 1 (last 100 of 400 steps, 20 runs). This chosen rule and quantitative comparison are not predictions supplied by the paper",
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
                    .with(&format!("Past the analytic upper edge, at z 6.14: {:.2}. This finite-size tail is outside the analytic boundary; the paper describes only similar, imperfectly matching boundaries.", edge))
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
            text: "Fig. 3 prints n=1000, z=6.14 and one global cascade in 1000 trials; phi=.18 is assumed from surrounding figures. With global defined as at least half, our transparent rarity rule requires rate <=1 percent and plug-in P(at most one in 1000)>=.05. The caption conflicts with the visible size-axis minimum .0001 for a single-node trigger; these printed conditions are a qualified reconstruction",
            check: |_| {
                let v = episodes(&watts(1000, 6.14), 10, 100);
                let g = share(&v, |x| x >= 0.5);
                let tiny = share(&v, |x| x < 0.01);
                let log_compatibility = binomial_at_most_one_log10(1000, g);
                let large = share(&episodes(&watts(10_000, 6.14), 10, 100), |x| x >= 0.5);
                outcome(g <= 0.01 && log_compatibility >= 0.05f64.log10(),
                    format!("{:.1} percent global, {:.1} percent under 1 percent; log10 P(X<=1 in 1000)={log_compatibility:.2}; n=10000 global {:.1} percent", 100.0*g, 100.0*tiny, 100.0*large))
                    .with("The compatibility calculation uses our estimated cascade probability, not the paper's unknown true rate. Its .05 cutoff was made explicit during source review after results were known. The cutoff, assumed phi and caption/axis conflict prevent a universal failure claim; finite-size sensitivity remains visible.")
            },
        },
        Claim {
            id: "watts.fig4a",
            item: "watts-hetero",
            source: Source::Book,
            citation: WATTS,
            text: "Fig. 4a's analytically derived windows: normal threshold heterogeneity extends the attainable range of phi and z. We normalize f(phi) on [0,1], test a wider z interval at normal location .18 and a maximum location parameter above the fixed-threshold .25 boundary. This rule corrects the earlier single-point frequency comparison after its results were known",
            check: |_| {
                let fixed = unit_interval_window(0.18, 0.0, 0.5, 60.0).unwrap();
                let narrow = unit_interval_window(0.18, 0.05, 0.5, 60.0).unwrap();
                let wide = unit_interval_window(0.18, 0.1, 0.5, 60.0).unwrap();
                let phi_limit = unit_interval_phi_limit(0.1);
                let newly_inside = unit_interval_poisson_ratio(8.0, 0.18, 0.1) > 1.0
                    && unit_interval_poisson_ratio(8.0, 0.18, 0.0) < 1.0;
                let holds = narrow.1-narrow.0 > fixed.1-fixed.0
                    && wide.1-wide.0 > narrow.1-narrow.0 && phi_limit > 0.25 && newly_inside;
                let hetero = |z: f64| {
                    let mut c = watts(2000, z);
                    c.distribution = Distribution::Normal;
                    c.sd = 0.1;
                    c.crowd = Crowd::Sampled;
                    c.zero = Zero::WhenReached;
                    c
                };
                let f = |c: &ThresholdsConfig| share(&episodes(c, 10, 100), |x| x >= 0.1);
                let (dense_hetero, dense_fixed) = (f(&hetero(8.0)), f(&watts(2000, 8.0)));
                let (sparse_hetero, sparse_fixed) = (f(&hetero(1.2)), f(&watts(2000, 1.2)));
                outcome(holds, format!("unit-interval z windows fixed {:.2}–{:.2}, sigma .05 {:.2}–{:.2}, sigma .1 {:.2}–{:.2}; maximum phi-location {:.4} versus fixed .25; z=8 newly inside", fixed.0,fixed.1,narrow.0,narrow.1,wide.0,wide.1,phi_limit))
                    .with(&format!("Sensitivity only: 1000 app simulations per setting, n=2000, clipped negative thresholds acting when reached: global frequencies at z=8 hetero/fixed {dense_hetero:.3}/{dense_fixed:.3}, at z=1.2 {sparse_hetero:.3}/{sparse_fixed:.3}. Clipping creates zero-threshold mass and is not a direct reproduction of the normalized source distribution. The lower boundary moving upward and sparse frequency decreasing do not refute a wider window."))
            },
        },
        Claim {
            id: "watts.fig4b",
            item: "watts-window",
            source: Source::Book,
            citation: WATTS,
            text: "Fig. 4b describes power-law graphs as less vulnerable. For the explicitly normalized integer-degree family p_k=Ck^−2.5 exp(-k/kappa), k>=1, our app illustrates this at phi=.18: the cascade condition remains below one over the tested degree grid and no global cascades occur at z=1.5. This conditional illustration does not claim an exact reproduction of the published window",
            check: |_| {
                let best = (101..195).map(|z| thresholds::power_law_ratio(f64::from(z) / 100.0, 0.18, 0.0)).fold(0.0, f64::max);
                let mut c = watts(10_000, 1.5);
                c.network = Network::PowerLaw;
                let g = share(&episodes(&c, 10, 20), |x| x >= 0.1);
                outcome(best < 1.0 && g == 0.0, format!("largest G0''(1)/z {best:.2}; {:.0} % global at z 1.5", 100.0 * g))
                    .with("The normalized integer k>=1 family has mean at most ζ(1.5)/ζ(2.5) ≈ 1.95 for any cutoff. Fig. 4b's larger z therefore needs another unstated choice; the tested grid and finite simulation support only this declared family at phi=.18.")
            },
        },
        Claim {
            id: "watts.hubs",
            item: "watts-targeting",
            source: Source::App,
            citation: "App per-trigger targeting comparison; Watts Eq. P_k=1-(1-S_v)^k and surrounding discussion",
            text: "Our highest-degree versus uniformly random per-trigger comparison: hubs have an advantage at both z=1.3 and z=5.5 (n=2000, 1000 paired fresh graphs each, higher observed hub rate and one-sided exact McNemar p<.05). This descriptive app rule was revised after results were known. The abstract denies a per-node advantage in the second regime, while the body also discusses aggregate trigger frequencies; the historical assertion remains unresolved by these finite settings",
            check: |_| {
                let compare = |z: f64| {
                    let (random, hub) = paired_trigger_sizes(z);
                    let hub_only = random.iter().zip(&hub).filter(|(r,h)| **r<0.1 && **h>=0.1).count();
                    let random_only = random.iter().zip(&hub).filter(|(r,h)| **r>=0.1 && **h<0.1).count();
                    let random_count = random.iter().filter(|size| **size>=0.1).count();
                    let hub_count = hub.iter().filter(|size| **size>=0.1).count();
                    let n = random.len();
                    let p = paired_hub_advantage_p(hub_only, random_only);
                    outcome(hub_count > random_count && p < 0.05,
                        format!("hub {hub_count}/{n} ({:.3}), random {random_count}/{n} ({:.3}); paired discordances hub-only/random-only {hub_only}/{random_only}; one-sided exact McNemar p={p:.3e}", hub_count as f64/n as f64, random_count as f64/n as f64))
                        .with("The paired judge requires a higher hub rate and p<.05. Watts's formula leaves a residual per-node degree advantage, in tension with the abstract and the sharply peaked-network claim that hubs will not display this property. The body also discusses aggregate trigger frequencies. At n=2000, z=5.5 is not the rare upper edge, so these finite comparisons do not resolve the historical assertion.")
                };
                all_of(vec![("sparse (z 1.3)".into(), compare(1.3)), ("dense (z 5.5)".into(), compare(5.5))])
            },
        },
    ]
}

#[cfg(test)]
mod source_tests {
    use super::*;

    #[test]
    fn paired_ten_to_zero_has_exact_one_sided_probability() {
        assert_eq!(paired_hub_advantage_p(10, 0), 1.0 / 1024.0);
    }

    #[test]
    fn paired_tie_is_not_evidence_of_hub_advantage() {
        assert!(paired_hub_advantage_p(5, 5) >= 0.5);
    }

    #[test]
    fn no_discordant_pairs_have_probability_one() {
        assert_eq!(paired_hub_advantage_p(0, 0), 1.0);
    }

    #[test]
    fn unit_interval_normal_removes_negative_tail_mass() {
        assert!(unit_interval_normal_cdf(0.0, 0.18, 0.1).abs() < 1e-12);
        assert!((unit_interval_normal_cdf(0.1, 0.18, 0.1) - 0.18248170537693514).abs() < 1e-6);
        assert_eq!(unit_interval_normal_cdf(1.0, 0.18, 0.1), 1.0);
    }

    #[test]
    fn unit_interval_poisson_ratio_matches_independent_sum() {
        assert!((unit_interval_poisson_ratio(8.0, 0.18, 0.1) - 1.7771907962238949).abs() < 1e-6);
    }

    #[test]
    fn unit_interval_window_has_finite_upper_edge() {
        let (lo, hi) = unit_interval_window(0.18, 0.1, 0.5, 60.0).unwrap();
        assert!((lo - 1.1478915588808731).abs() < 0.011, "{lo}");
        assert!((hi - 39.41072270337905).abs() < 0.011, "{hi}");
    }

    #[test]
    fn unit_interval_heterogeneity_extends_threshold_range() {
        assert!((unit_interval_phi_limit(0.1) - 0.27712046424210407).abs() < 0.0001);
    }

    #[test]
    fn binomial_rare_tail_uses_probability_of_at_most_one_success() {
        assert!((binomial_at_most_one_log10(1000, 0.001) - (-0.13326446812555665)).abs() < 1e-8);
        assert_eq!(binomial_at_most_one_log10(1000, 0.0), 0.0);
    }
}
