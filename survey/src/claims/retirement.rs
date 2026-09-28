//! The timing of retirement (milestone 26): Axtell and Epstein (1999), with
//! the revised text in Epstein (2006, ch. 7). Transition time is the first
//! period with 95 % of eligible agents retired (the texts never define it);
//! a run that never reaches it reads NaN.

use sugarscape_core::model::{ModelConfig, ModelWorld};
use sugarscape_core::retirement::{Counts, Groups, Policy, Renewal, RetirementConfig};

use crate::claim::{all_of, equivalent, greater, Claim, Outcome, Source, Verdict};
use crate::runner::model_after;

const AE: &str = "Axtell & Epstein 1999, Brookings CSED WP 1";
const GSS: &str = "Epstein 2006, Generative Social Science, ch. 7";

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

fn config(edit: impl FnOnce(&mut RetirementConfig)) -> RetirementConfig {
    let mut c = RetirementConfig::default();
    edit(&mut c);
    c
}

fn seeds(n: u64) -> Vec<u64> {
    (1..=n).collect()
}

/// `series` at the last period of runs stopped at the norm (NaN if never).
fn at_norm(c: RetirementConfig, n: u64, periods: u32, series: &str) -> Vec<f64> {
    let c = RetirementConfig {
        stop_at_norm: true,
        ..c
    };
    let name = series.to_string();
    model_after(&ModelConfig::Retirement(c), &seeds(n), periods, move |w| {
        w.model().latest_value(&name).unwrap()
    })
}

fn transitions(c: RetirementConfig, n: u64, periods: u32) -> Vec<f64> {
    at_norm(c, n, periods, "transition")
}

/// The whole `retired` series of runs (not stopped).
fn series(c: RetirementConfig, n: u64, periods: u32) -> Vec<Vec<f64>> {
    model_after(
        &ModelConfig::Retirement(c),
        &seeds(n),
        periods,
        |w: &ModelWorld| w.model().series("retired").unwrap(),
    )
}

fn mean(v: &[f64]) -> f64 {
    let f: Vec<f64> = v.iter().copied().filter(|x| x.is_finite()).collect();
    if f.is_empty() {
        f64::NAN
    } else {
        f.iter().sum::<f64>() / f.len() as f64
    }
}

fn reached(v: &[f64]) -> usize {
    v.iter().filter(|x| x.is_finite()).count()
}

fn describe(v: &[f64]) -> String {
    format!("{:.1} ({} of {} reached)", mean(v), reached(v), v.len())
}

/// A falling mean along `x` (each at least as fast as the last, within `slack`).
fn falling(means: &[f64], slack: f64) -> bool {
    means.windows(2).all(|w| w[1] <= w[0] + slack)
}

pub fn claims() -> Vec<Claim> {
    vec![
        Claim {
            id: "retirement.ae.rapid",
            item: "ae-rapid",
            source: Source::Book,
            citation: GSS,
            text: "15 % rational, 80 % imitators, 5 % random: 'Within the first 6 periods essentially all of the eligible population has retired … this trajectory is essentially monotone' (95 % retired by period 6 in most of 20 runs; never falling by more than 0.01)",
            check: |_| {
                let runs = series(config(|c| c.rational = 0.15), 20, 10);
                let by6 = runs.iter().filter(|s| s[6] >= 0.95).count();
                let monotone = runs.iter().filter(|s| s.windows(2).all(|w| w[1] >= w[0] - 0.01)).count();
                all_of(vec![
                    ("by period 6".into(), outcome(by6 >= 16, format!("{by6} of 20 at 95 % by period 6"))),
                    ("monotone".into(), outcome(monotone >= 16, format!("{monotone} of 20 never fall"))),
                ])
                .with("AE's text gives 15/75/5 (95 %) and its caption 20 %; GSS gives 15/80/5.")
            },
        },
        Claim {
            id: "retirement.ae.slow",
            item: "ae-slow",
            source: Source::Book,
            citation: AE,
            text: "5 % rational: 'It takes a long time for the absorbing state to be achieved … the trajectory is not monotone' (transition past 30 periods; the share falls by 0.02 or more somewhere in most of 20 runs)",
            check: |_| {
                let runs = series(config(|c| c.rational = 0.05), 20, 150);
                let dips = runs
                    .iter()
                    .filter(|s| {
                        let mut top = 0.0f64;
                        s.iter().any(|&x| {
                            top = top.max(x);
                            x < top - 0.02 && top < 0.9
                        })
                    })
                    .count();
                let t = transitions(config(|c| c.rational = 0.05), 20, 600);
                all_of(vec![
                    ("slow".into(), outcome(mean(&t) > 30.0, format!("transition {}", describe(&t)))),
                    ("not monotone".into(), outcome(dips >= 11, format!("{dips} of 20 fall back"))),
                ])
            },
        },
        Claim {
            id: "retirement.ae.footnote5",
            item: "ae-all-members",
            source: Source::Book,
            citation: AE,
            text: "Footnote 5: whether an agent counts all its network or only the eligible 'makes a difference to the numerical results … However, the qualitative character of the results … do not depend on this distinction' (the base case reaches the norm either way; 20 runs of 600 periods)",
            check: |_| {
                let e = transitions(RetirementConfig::default(), 20, 600);
                let a = transitions(config(|c| c.counts = Counts::All), 20, 600);
                outcome(reached(&a) >= 16, format!("eligible only: {}; all members: {}", describe(&e), describe(&a)))
            },
        },
        Claim {
            id: "retirement.ae.fig6-rationals",
            item: "ae-rational",
            source: Source::Book,
            citation: AE,
            text: "Figure 6-6: 'Reducing the proportion of rationals, while holding constant the proportion of randoms, increases transition time' (5 % random; 2 to 25 % rational; 20 runs)",
            check: |_| {
                let xs = [0.02, 0.05, 0.1, 0.15, 0.2, 0.25];
                let m: Vec<f64> = xs.iter().map(|&r| mean(&transitions(config(|c| c.rational = r), 20, 600))).collect();
                outcome(falling(&m, 1.0), format!("{:?} at {xs:?}", m.iter().map(|x| (x * 10.0).round() / 10.0).collect::<Vec<_>>()))
            },
        },
        Claim {
            id: "retirement.ae.fig6-minimum",
            item: "ae-rational",
            source: Source::Book,
            citation: AE,
            text: "Figure 6-6: 'When randoms comprise 0 percent or 5 percent of the population, certain minimum proportions of the population must be rational for a retirement age norm to arise' (with 5 % randoms, runs with 0 % and 2 % rational never reach the norm in 1 500 periods; the default renewal)",
            check: |_| {
                let zero = transitions(config(|c| c.rational = 0.0), 10, 1500);
                let two = transitions(config(|c| c.rational = 0.02), 10, 1500);
                let rz = transitions(
                    config(|c| {
                        c.rational = 0.0;
                        c.per_cohort = 50;
                        c.renewal = Renewal::Replace;
                    }),
                    5,
                    1500,
                );
                let r5 = transitions(
                    config(|c| {
                        c.rational = 0.05;
                        c.per_cohort = 50;
                        c.renewal = Renewal::Replace;
                    }),
                    5,
                    1500,
                );
                outcome(reached(&zero) == 0 && reached(&two) == 0, format!("0 %: {}; 2 %: {}", describe(&zero), describe(&two)))
                    .with(&format!("With friends who die replaced within the holder's age range (C 50): 0 %: {}; 5 %: {}.", describe(&rz), describe(&r5)))
            },
        },
        Claim {
            id: "retirement.ae.fig6-randoms",
            item: "ae-rational",
            source: Source::Book,
            citation: AE,
            text: "Figure 6-6: 'For a given fraction of rationals, the transition time decreases as the proportion of randoms increases' (5 % rational: 0, 5, 10 % random; 20 runs)",
            check: |_| {
                let t = |r: f64| transitions(config(|c| {
                    c.rational = 0.05;
                    c.random = r;
                }), 20, 600);
                let (a, b, c) = (t(0.0), t(0.05), t(0.1));
                all_of(vec![
                    ("0 → 5 %".into(), greater(&a, &b, "no randoms", "5 %")),
                    ("5 → 10 %".into(), greater(&b, &c, "5 %", "10 %")),
                ])
            },
        },
        Claim {
            id: "retirement.ae.cohort",
            item: "ae-base",
            source: Source::Book,
            citation: GSS,
            text: "'The first parameter, the number of agents per cohort (C), was found to have no effect on the average transition time for C > 100' (C 100 and 200 the same within 20 %; 20 runs)",
            check: |_| {
                let a = transitions(RetirementConfig::default(), 20, 600);
                let b = transitions(config(|c| c.per_cohort = 200), 20, 600);
                let s = transitions(config(|c| c.per_cohort = 25), 20, 600);
                equivalent(&a, &b, Some(0.2 * mean(&a)), "C 100", "C 200").with(&format!("C 25: {}.", describe(&s)))
            },
        },
        Claim {
            id: "retirement.ae.fig7",
            item: "ae-threshold",
            source: Source::Book,
            citation: AE,
            text: "Figure 6-7: 'Increasing the variance in the threshold decreases the average transition time' (spreads 0, 0.05, 0.1, 0.15, 0.2, 0.25, uniform around 0.5; 20 runs)",
            check: |_| {
                let xs = [0.0, 0.05, 0.1, 0.15, 0.2, 0.25];
                let m: Vec<f64> = xs.iter().map(|&s| mean(&transitions(config(|c| c.spread = s), 20, 600))).collect();
                outcome(falling(&m, 1.0), format!("{:?}", m.iter().map(|x| (x * 10.0).round() / 10.0).collect::<Vec<_>>()))
                    .with("Falling overall, but a little spread first doubles the time.")
            },
        },
        Claim {
            id: "retirement.ae.fig8",
            item: "ae-size",
            source: Source::Book,
            citation: AE,
            text: "Figure 6-8: transition time 'increases very rapidly with increasing social network size' (mean 10 to 40, ± 7); falls weakly with the size's spread (17 ± 0 to ± 14); increases with S̄ in U[10, S̄] (20 runs)",
            check: |_| {
                let size = |min: u32, max: u32| mean(&transitions(config(|c| c.size = sugarscape_core::retirement::Size { min, max }), 20, 600));
                let a: Vec<f64> = [10u32, 15, 20, 25, 30, 40].iter().map(|&m| size(m.saturating_sub(7), m + 7)).collect();
                let b: Vec<f64> = [0u32, 3, 7, 10, 14].iter().map(|&h| size(17 - h, 17 + h)).collect();
                let c: Vec<f64> = [10u32, 20, 30, 40, 60, 80].iter().map(|&m| size(10, m)).collect();
                let r = |v: &[f64]| format!("{:?}", v.iter().map(|x| x.round()).collect::<Vec<_>>());
                all_of(vec![
                    ("mean size".into(), outcome(a.windows(2).all(|w| w[1] >= w[0] - 1.0) && a[5] > 3.0 * a[0], r(&a))),
                    ("spread".into(), outcome(falling(&b, 1.0) && b[4] < b[0], r(&b))),
                    ("maximum".into(), outcome(c.windows(2).all(|w| w[1] >= w[0] - 1.0) && c[5] > 3.0 * c[0], r(&c))),
                ])
            },
        },
        Claim {
            id: "retirement.ae.fig9",
            item: "ae-extent",
            source: Source::Book,
            citation: AE,
            text: "Figure 6-9: 'the effect of increasing the extent (in the age dimension) of agent social networks is to decrease the transition times' (extent 1 against 10, at 10 % and 5 % rational; 20 runs)",
            check: |_| {
                let t = |r: f64, e: u32| transitions(config(|c| {
                    c.rational = r;
                    c.extent = e;
                }), 20, 600);
                all_of(vec![
                    ("10 % rational".into(), greater(&t(0.1, 1), &t(0.1, 10), "extent 1", "extent 10")),
                    ("5 % rational".into(), greater(&t(0.05, 1), &t(0.05, 10), "extent 1", "extent 10")),
                ])
                .with(&format!("At 5 %, extent 3: {}.", describe(&t(0.05, 3))))
            },
        },
        Claim {
            id: "retirement.ae.as-if",
            item: "ae-rational",
            source: Source::Book,
            citation: AE,
            text: "'The attainment per se of the age 65 retirement norm is compatible with any rationality fraction above a critical level' (every run from 2 % rational up reaches the norm; 5 % random; 10 runs of 600 periods)",
            check: |_| {
                let v: Vec<usize> = [0.02, 0.05, 0.1, 0.2].iter().map(|&r| reached(&transitions(config(|c| c.rational = r), 10, 600))).collect();
                outcome(v.iter().all(|&k| k == 10), format!("{v:?} of 10 at 2, 5, 10, 20 %"))
            },
        },
        Claim {
            id: "retirement.ae.mandatory",
            item: "ae-policy",
            source: Source::Book,
            citation: AE,
            text: "'Now we require that all agents retire at age 70. This increases the speed at which the age 65 retirement norm is established' (5 % rational; 20 runs)",
            check: |_| {
                let free = transitions(config(|c| c.rational = 0.05), 20, 600);
                let forced = transitions(config(|c| {
                    c.rational = 0.05;
                    c.mandatory = 70;
                }), 20, 600);
                greater(&free, &forced, "no mandatory age", "mandatory at 70")
                    .with("With 70+ forced out, they are most of the eligible: the 95 % measure then says little about retiring at 65.")
            },
        },
        Claim {
            id: "retirement.ae.policy",
            item: "ae-policy",
            source: Source::Book,
            citation: AE,
            text: "The policy switch (mandatory 70; eligibility 65 → 62 once the norm is established): 'a new norm indeed emerges after twenty to thirty periods'; 'in about 35 periods if between 1 and 4 percent of the population responds rationally' (periods from the switch to 95 % of those 62+ retired, at 1 %, 2 %, 4 % rational; within 20–40)",
            check: |_| {
                let parts = [0.01, 0.02, 0.04]
                    .into_iter()
                    .map(|r| {
                        let v = at_norm(
                            config(|c| {
                                c.rational = r;
                                c.mandatory = 70;
                                c.policy = Policy { enabled: true, to: 62 };
                            }),
                            20,
                            600,
                            "transition_new",
                        );
                        let m = mean(&v);
                        (format!("{} % rational", r * 100.0), outcome((20.0..=40.0).contains(&m), format!("{m:.1} periods")))
                    })
                    .collect();
                all_of(parts).with("Imitators just turned 62 count the retired 65-to-67-year-olds among their eligible friends and retire at once; the same under friends replaced.")
            },
        },
        Claim {
            id: "retirement.ae.groups-pull",
            item: "ae-coupling",
            source: Source::Book,
            citation: AE,
            text: "Figure 6-11: 'very little coupling is needed for the non-rational sub-population to be pulled into conformity' (the group without rationals reaches the norm sooner at coupling 0.1 than uncoupled; 20 runs of 300 periods)",
            check: |_| {
                let t = |k: f64, s: &str| {
                    let name = s.to_string();
                    model_after(
                        &ModelConfig::Retirement(config(|c| c.groups = Groups { enabled: true, coupling: k })),
                        &seeds(20),
                        300,
                        move |w| w.model().latest_value(&name).unwrap(),
                    )
                };
                greater(&t(0.0, "transition_a"), &t(0.1, "transition_a"), "uncoupled", "coupling 0.1")
            },
        },
        Claim {
            id: "retirement.ae.groups-rational",
            item: "ae-coupling-rational",
            source: Source::Book,
            citation: AE,
            text: "Figure 6-11: the sub-population with rational agents keeps its transition time as coupling rises (coupling 0 and 0.1 the same within 25 %; 20 runs of 300 periods)",
            check: |_| {
                let t = |k: f64| {
                    model_after(
                        &ModelConfig::Retirement(config(|c| c.groups = Groups { enabled: true, coupling: k })),
                        &seeds(20),
                        300,
                        |w| w.model().latest_value("transition_b").unwrap(),
                    )
                };
                let (a, b, c) = (t(0.0), t(0.1), t(0.25));
                equivalent(&a, &b, Some(0.25 * mean(&a)), "uncoupled", "coupling 0.1").with(&format!("At 0.25: {}.", describe(&c)))
            },
        },
    ]
}
