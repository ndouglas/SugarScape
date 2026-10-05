//! The timing of retirement (milestone 26): Axtell and Epstein (1999), with
//! the revised text in Epstein (2006, ch. 7). Transition time is the first
//! period with 95 % of eligible agents retired: an operational proxy, not
//! the texts' undefined retirement-age norm. Nonattainment is right-censored.

use sugarscape_core::model::{ModelConfig, ModelWorld};
use sugarscape_core::retirement::{Counts, Groups, Policy, RetirementConfig};

use crate::claim::{all_of, equivalent, greater, Claim, Outcome, Source, Verdict};
use crate::runner::{model_after, on_threads};

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
    let attained = reached(v);
    let timing = if attained == 0 {
        "no attainment".into()
    } else {
        format!("conditional mean {:.1}", mean(v))
    };
    format!(
        "{timing}; {attained}/{} attained, {} censored at configured horizon",
        v.len(),
        v.len() - attained
    )
}

/// A falling mean along `x` (each at least as fast as the last, within `slack`).
fn falling(means: &[f64], slack: f64) -> bool {
    means.windows(2).all(|w| w[1] <= w[0] + slack)
}

fn weak(measured: String, detail: &str) -> Outcome {
    Outcome {
        verdict: Verdict::Weak,
        measured,
        detail: detail.into(),
    }
}

fn grid(xs: &[f64], edit: impl Fn(&mut RetirementConfig, f64)) -> Vec<Vec<f64>> {
    xs.iter()
        .map(|&x| transitions(config(|c| edit(c, x)), 50, 600))
        .collect()
}

fn describe_grid(xs: &[f64], runs: &[Vec<f64>]) -> String {
    xs.iter()
        .zip(runs)
        .map(|(x, v)| format!("{x:.4}: {}", describe(v)))
        .collect::<Vec<_>>()
        .join("; ")
}

fn policy_transitions(c: RetirementConfig, n: u64, before: u32, after: u32) -> Vec<f64> {
    on_threads(&seeds(n), |seed| {
        let target = c.policy.to as f64;
        let mut w =
            ModelWorld::new(ModelConfig::Retirement(c.clone()), seed).expect("valid policy config");
        while w.model().tick() < u64::from(before)
            && w.model().latest_value("eligibility") != Some(target)
        {
            w.model_mut().run(1);
        }
        if w.model().latest_value("eligibility") != Some(target) {
            return f64::NAN;
        }
        w.model_mut().run(after);
        w.model().latest_value("transition_new").unwrap_or(f64::NAN)
    })
}

pub fn claims() -> Vec<Claim> {
    [
        Claim {
            id: "retirement.ae.rapid",
            item: "ae-rapid",
            source: Source::Book,
            citation: GSS,
            text: "Operational rule revised after the earlier result was known. The source describes one essentially monotone six-period realization (AE prose 15 %, caption 20 %; GSS 15 %). Test compatibility: at least one 15 %-rational run reaches 95 % by six and remains within 0.01 of monotone through ten, among 50 runs; this is not a source probability claim.",
            check: |_| {
                let runs = series(config(|c| c.rational = 0.15), 50, 10);
                let compatible = runs
                    .iter()
                    .filter(|s| s[6] >= 0.95 && s.windows(2).all(|w| w[1] >= w[0] - 0.01))
                    .count();
                outcome(compatible > 0, format!("{compatible}/50 compatible six-period trajectories")).with("95 % and the monotonicity tolerance are reconstruction criteria; no exact figure reproduction asserted.")
            },
        },
        Claim {
            id: "retirement.ae.slow",
            item: "ae-slow",
            source: Source::Book,
            citation: AE,
            text: "Operational rule revised after the earlier result was known. At 5 % rational, test the qualitative slow, wavering trajectory (mean first 95 % crossing past 30; pre-cascade dips of at least .02 in a majority of 50). The source trajectory reaches its displayed plateau around 375; this test does not reproduce that timing or perfect absorption.",
            check: |_| {
                let runs = series(config(|c| c.rational = 0.05), 50, 150);
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
                let t = transitions(config(|c| c.rational = 0.05), 50, 600);
                outcome(
                    mean(&t) > 30.0 && dips > 25,
                    format!("{}; {dips}/50 pre-cascade dips", describe(&t)),
                )
            },
        },
        Claim {
            id: "retirement.ae.footnote5",
            item: "ae-all-members",
            source: Source::Book,
            citation: AE,
            text: "Operational rule revised after the earlier result was known. Footnote 5 states qualitative invariance between all-member and eligible-member counting. Compare fixed parameters over 600 rounds, 50 runs; differences in first 95 % crossing are diagnostic because the source does not define its norm.",
            check: |_| {
                let e = transitions(RetirementConfig::default(), 50, 600);
                let a = transitions(config(|c| c.counts = Counts::All), 50, 600);
                weak(format!("eligible: {}; all: {}",describe(&e),describe(&a)), "Fixed-denominator proxy behavior differs under Slot reconstruction. All-member imitators can and do retire; event mode 65 alone can reflect a small minority. This cannot categorically adjudicate the source's qualitative norm claim.")
            },
        },
        Claim {
            id: "retirement.ae.fig6-rationals",
            item: "ae-rational",
            source: Source::Book,
            citation: AE,
            text: "Operational rule revised after the earlier result was known. Figure 6-6: reducing rational share increases transition time. Test the declining first 95 % crossing trend at 5, 10, 15, 20, 25 % rational, 5 % random, 50 runs each, horizon 600; source stopping rule is undefined.",
            check: |_| {
                let xs = [0.05, 0.1, 0.15, 0.2, 0.25];
                let v = grid(&xs, |c, x| c.rational = x);
                let m: Vec<_> = v.iter().map(|v| mean(v)).collect();
                outcome(falling(&m, 1.0), describe_grid(&xs, &v))
            },
        },
        Claim {
            id: "retirement.ae.fig6-minimum",
            item: "ae-rational",
            source: Source::Book,
            citation: AE,
            text: "Operational rule revised after the earlier result was known. The source asserts critical rational shares. Observe 0 and 2 % rational with 5 % random over 2000 rounds, 50 Slot runs each. Finite-horizon first 95 % crossing cannot establish or refute an infinite-time critical retirement-age norm.",
            check: |_| {
                let z = transitions(config(|c| c.rational = 0.0), 50, 2000);
                let t = transitions(config(|c| c.rational = 0.02), 50, 2000);
                weak(format!("0 %: {}; 2 %: {}",describe(&z),describe(&t)),"Slot and oldest-cohort-first traversal are reconstruction choices, not uniquely specified by source pseudocode. No omitted renewal rule is identified.")
            },
        },
        Claim {
            id: "retirement.ae.fig6-randoms",
            item: "ae-rational",
            source: Source::Book,
            citation: AE,
            text: "Operational rule revised after the earlier result was known. Figure 6-6: more random agents shorten transition time. Compare 0, 5, 10 % random with 5 % rational, 50 runs each, horizon 600; first 95 % crossing is a reconstruction proxy.",
            check: |_| {
                let v = grid(&[0.0, 0.05, 0.1], |c, x| {
                    c.rational = 0.05;
                    c.random = x;
                });
                all_of(vec![
                    ("0 to 5 %".into(), greater(&v[0], &v[1], "0 %", "5 %")),
                    ("5 to 10 %".into(), greater(&v[1], &v[2], "5 %", "10 %")),
                ])
                .with(&describe_grid(&[0.0, 0.05, 0.1], &v))
            },
        },
        Claim {
            id: "retirement.ae.cohort",
            item: "ae-base",
            source: Source::Book,
            citation: GSS,
            text: "Operational rule revised after the earlier result was known. GSS limits cohort-size insensitivity to C > 100. Compare C 200 and C 300, 50 runs each, first 95 % crossing within 600 and a declared 20 % equivalence margin; not all source norm definitions.",
            check: |_| {
                let a = transitions(config(|c| c.per_cohort = 200), 50, 600);
                let b = transitions(config(|c| c.per_cohort = 300), 50, 600);
                equivalent(&a, &b, Some(0.2 * mean(&a)), "C200", "C300").with(&format!(
                    "{}; {}",
                    describe(&a),
                    describe(&b)
                ))
            },
        },
        Claim {
            id: "retirement.ae.fig7",
            item: "ae-threshold",
            source: Source::Book,
            citation: AE,
            text: "Operational rule revised after the earlier result was known. Figure 6-7 shows an overall decline with a final uptick, not strict monotonicity. Use positive uniform half-widths .05,.10,.20,.30,.40,.50 divided by sqrt(3), base 10 % rational, 50 runs each over 600; compare endpoints. Zero spread is a separate extension.",
            check: |_| {
                let xs = [0.05, 0.1, 0.2, 0.3, 0.4, 0.5].map(|x| x / 3.0f64.sqrt());
                let v = grid(&xs, |c, x| c.spread = x);
                greater(&v[0], &v[5], "smallest positive spread", "largest spread")
                    .with(&describe_grid(&xs, &v))
            },
        },
        Claim {
            id: "retirement.ae.fig8",
            item: "ae-size",
            source: Source::Book,
            citation: AE,
            text: "Operational rule revised after the earlier result was known. Figure 6-8 network-size trend: larger mean and maximum slow first 95 % crossing; wider size spread weakly speeds it. These 50-run grids over 600 check shape only, not literal source quantitative reproduction.",
            check: |_| {
                let size = |min, max| {
                    transitions(
                        config(|c| c.size = sugarscape_core::retirement::Size { min, max }),
                        50,
                        600,
                    )
                };
                let a = size(3, 17);
                let b = size(33, 47);
                let c = size(17, 17);
                let d = size(3, 31);
                let e = size(10, 10);
                let f = size(10, 80);
                all_of(vec![
                    ("mean".into(), greater(&b, &a, "large mean", "small mean")),
                    (
                        "spread".into(),
                        greater(&c, &d, "zero spread", "wide spread"),
                    ),
                    ("maximum".into(), greater(&f, &e, "maximum80", "maximum10")),
                ])
                .with(&format!(
                    "mean: {} / {}; spread: {} / {}; maximum: {} / {}",
                    describe(&a),
                    describe(&b),
                    describe(&c),
                    describe(&d),
                    describe(&e),
                    describe(&f)
                ))
            },
        },
        Claim {
            id: "retirement.ae.fig9",
            item: "ae-extent",
            source: Source::Book,
            citation: AE,
            text: "Operational rule revised after the earlier result was known. Figure 6-9 extent domains: compare 2 versus 10 at 10 % rational and 6 versus 10 at 5 %, 50 runs each over 600. Endpoint decline tests the plotted qualitative pattern with first 95 % crossing, not every adjacent point.",
            check: |_| {
                let t = |r, e| {
                    transitions(
                        config(|c| {
                            c.rational = r;
                            c.extent = e;
                        }),
                        50,
                        600,
                    )
                };
                let a = t(0.1, 2);
                let b = t(0.1, 10);
                let c = t(0.05, 6);
                let d = t(0.05, 10);
                all_of(vec![
                    ("10 %".into(), greater(&a, &b, "extent2", "extent10")),
                    ("5 %".into(), greater(&c, &d, "extent6", "extent10")),
                ])
                .with(&format!(
                    "{}; {}; {}; {}",
                    describe(&a),
                    describe(&b),
                    describe(&c),
                    describe(&d)
                ))
            },
        },
        Claim {
            id: "retirement.ae.as-if",
            item: "ae-rational",
            source: Source::Book,
            citation: AE,
            text: "Operational rule revised after the earlier result was known. The source says age 65 norm attainment is compatible with any rationality fraction above a critical level. Observe first 95 % crossing at 2,5,10,20 % rational, 50 runs each over 600; these cannot locate that critical level or establish a persistent age norm.",
            check: |_| {
                let xs = [0.02, 0.05, 0.1, 0.2];
                let v = grid(&xs, |c, x| c.rational = x);
                weak(describe_grid(&xs,&v),"Widespread aggregate retirement is compatible with several rational shares under the chosen rules; the source norm and infinite-horizon criticality remain unadjudicated.")
            },
        },
        Claim {
            id: "retirement.ae.mandatory",
            item: "ae-policy",
            source: Source::Book,
            citation: AE,
            text: "Operational rule revised after the earlier result was known. Mandatory 70 is said to speed establishment of age 65 norm. Compare first 95 % crossing with and without mandatory 70 at 5 % rational, 50 runs over 600; forced retirement confounds this diagnostic.",
            check: |_| {
                let a = transitions(config(|c| c.rational = 0.05), 50, 600);
                let b = transitions(
                    config(|c| {
                        c.rational = 0.05;
                        c.mandatory = 70;
                    }),
                    50,
                    600,
                );
                weak(
                    format!("free: {}; mandatory70: {}", describe(&a), describe(&b)),
                    "The aggregate proxy does not establish a modal or persistent age 65 norm.",
                )
            },
        },
        Claim {
            id: "retirement.ae.policy",
            item: "ae-policy",
            source: Source::Book,
            citation: GSS,
            text: "Operational rule revised after the earlier result was known. AE policy uses threshold .5; revised GSS uses U[.5,1] (mean .75, SD .14433756729740646), random 5 %. Compare 1,2,4,5 % rational, 50 runs, automatic first 95 % crossing switch and at most 100 periods afterward. AE figure means near 70/40/30/20 at 1/2/3/4 % are approximate readings; GSS describes twenty and about 35. These diagnostics do not reproduce an author-defined norm.",
            check: |_| {
                let mut parts = Vec::new();
                for revised in [false, true] {
                    for r in [0.01, 0.02, 0.04, 0.05] {
                        let c = config(|c| {
                            c.rational = r;
                            c.mandatory = 70;
                            c.policy = Policy {
                                enabled: true,
                                to: 62,
                            };
                            if revised {
                                c.threshold = 0.75;
                                c.spread = 0.14433756729740646;
                            }
                        });
                        let v = policy_transitions(c, 50, 1000, 100);
                        parts.push(format!(
                            "{} R{r}: {}",
                            if revised { "GSS" } else { "AE" },
                            describe(&v)
                        ));
                    }
                }
                weak(parts.join("; "),"Automatic first 95 % crossing initialization need not establish age 65 norm. Conditional means exclude right-censored runs. Neither a first crossing nor a short-lived mode establishes a persistent new norm; source quantitative failure is not inferred.")
            },
        },
        Claim {
            id: "retirement.ae.groups-pull",
            item: "ae-coupling",
            source: Source::Book,
            citation: AE,
            text: "Operational rule revised after the earlier result was known. Figure 6-11: slight coupling pulls the group without rationals toward conformity. Compare source coupling .05 and .10, 50 runs over 600; config rational.10 means 10 % within B,0 % in A, expected 5 % globally. First95 is an operational proxy.",
            check: |_| {
                let t = |k| {
                    model_after(
                        &ModelConfig::Retirement(config(|c| {
                            c.groups = Groups {
                                enabled: true,
                                coupling: k,
                            }
                        })),
                        &seeds(50),
                        600,
                        |w| w.model().latest_value("transition_a").unwrap(),
                    )
                };
                let a = t(0.05);
                let b = t(0.1);
                greater(&a, &b, "coupling.05", "coupling.10").with(&format!(
                    "{}; {}",
                    describe(&a),
                    describe(&b)
                ))
            },
        },
        Claim {
            id: "retirement.ae.groups-rational",
            item: "ae-coupling-rational",
            source: Source::Book,
            citation: AE,
            text: "Operational rule revised after the earlier result was known. Figure 6-11 shows the rational group slowing and both groups converging as coupling rises. Compare rational-group first 95 % crossing at source.05 and.20, 50 runs over 600,10 % rational within B (expected 5 % global). No uncoupled point appears in the figure.",
            check: |_| {
                let t = |k, s: &str| {
                    let name = s.to_string();
                    model_after(
                        &ModelConfig::Retirement(config(|c| {
                            c.groups = Groups {
                                enabled: true,
                                coupling: k,
                            }
                        })),
                        &seeds(50),
                        600,
                        move |w| w.model().latest_value(&name).unwrap(),
                    )
                };
                let a = t(0.05, "transition_b");
                let b = t(0.2, "transition_b");
                greater(&b,&a,"coupling.20","coupling.05").with(&format!("B: {}; {}; A at.20: {}. Source and reconstruction both slow B; numerical equivalence is not asserted.",describe(&a),describe(&b),describe(&t(0.2,"transition_a"))))
            },
        },
    ].into_iter().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn policy_followup_respects_zero_post_switch_horizon() {
        let c = config(|c| {
            c.rational = 1.0;
            c.random = 0.0;
            c.policy = Policy {
                enabled: true,
                to: 62,
            };
        });
        assert_eq!(reached(&policy_transitions(c, 1, 10, 0)), 0);
    }

    #[test]
    fn policy_followup_does_not_run_without_a_switch() {
        let c = config(|c| {
            c.policy = Policy {
                enabled: true,
                to: 62,
            }
        });
        assert_eq!(reached(&policy_transitions(c, 1, 0, 100)), 0);
    }

    // Losing censor counts would turn conditional response times into apparent complete ensembles.
    #[test]
    fn summary_preserves_censored_runs() {
        let summary = describe(&[10.0, 20.0, f64::NAN]);
        assert!(summary.contains("1 censored"), "{summary}");
    }

    #[test]
    fn summary_reports_no_attainment_without_nan_mean() {
        let summary = describe(&[f64::NAN; 3]);
        assert!(summary.contains("no attainment"), "{summary}");
        assert!(!summary.contains("NaN"), "{summary}");
    }
}
