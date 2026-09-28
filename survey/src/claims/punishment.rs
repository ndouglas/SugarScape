//! Altruistic punishment (milestone 27): Boyd, Gintis, Bowles and Richerson
//! (2003), with Cooney's PDE model (2024) as the critique and Janssen's
//! NetLogo replication's readings. Every figure is the mean cooperation over
//! the last 1 000 of 2 000 periods (`long_run`), as in the paper. The
//! figures' values were read from the PDF at 300 dpi by marker (± 0.01).

use sugarscape_core::model::{Model, ModelConfig, ModelWorld};
use sugarscape_core::punishment::{
    Counted, Erring, Imitation, Pairing, Punishing, PunishmentConfig, PunishmentWorld, Start,
    Structure, Traits, Victory,
};

use crate::claim::{all_of, equivalent, greater, Claim, Outcome, Source, Verdict};
use crate::runner::model_after;

const BGBR: &str = "Boyd, Gintis, Bowles & Richerson 2003, PNAS 100: 3531";
const COONEY: &str = "Cooney 2024, arXiv:2405.18419 (Bull. Math. Biol. 2025)";
const JANSSEN: &str = "Janssen, CoMSES 2223 (a NetLogo replication)";

/// The group sizes of the figures.
const SIZES: [u32; 7] = [4, 8, 16, 32, 64, 128, 256];

/// Fig. 1b (and Figs. 3 and 4's base curve), ε 0.015, read from the figure.
const FIG1B: [f64; 7] = [0.87, 0.88, 0.86, 0.83, 0.79, 0.64, 0.55];
/// Fig. 1b's lowest and highest conflict rates (the legend's 0.0075 and 0.03).
const FIG1B_LOW: [f64; 7] = [0.81, 0.82, 0.77, 0.68, 0.47, 0.16, 0.07];
const FIG1B_HIGH: [f64; 7] = [0.92, 0.93, 0.92, 0.91, 0.89, 0.79, 0.66];
/// Fig. 1a at ε 0.015 (the lines merge at about 0.09 from n 32).
const FIG1A: [f64; 7] = [0.76, 0.57, 0.20, 0.09, 0.09, 0.09, 0.09];
const FIG1A_LOW: [f64; 7] = [0.63, 0.37, 0.14, 0.09, 0.09, 0.09, 0.09];
const FIG1A_HIGH: [f64; 7] = [0.87, 0.79, 0.49, 0.11, 0.09, 0.08, 0.08];

/// Figs. 2–4, read the same way; a marker hidden under another curve takes
/// that curve's value.
const FIG2A: [[f64; 7]; 3] = [
    [0.79, 0.66, 0.34, 0.10, 0.09, 0.09, 0.09],
    [0.75, 0.57, 0.20, 0.10, 0.09, 0.09, 0.09],
    [0.55, 0.21, 0.11, 0.10, 0.08, 0.08, 0.08],
];
const FIG2B: [[f64; 7]; 3] = [
    [0.89, 0.88, 0.87, 0.85, 0.82, 0.71, 0.73],
    [0.87, 0.88, 0.86, 0.83, 0.77, 0.61, 0.57],
    [0.86, 0.87, 0.85, 0.72, 0.12, 0.06, 0.06],
];
const FIG3_WEAK: [f64; 7] = [0.80, 0.72, 0.58, 0.21, 0.07, 0.06, 0.06];
const FIG4_FIXED: [f64; 7] = [0.84, 0.78, 0.50, 0.10, 0.07, 0.06, 0.06];

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

fn config(edit: impl FnOnce(&mut PunishmentConfig)) -> PunishmentConfig {
    let mut c = PunishmentConfig::default();
    edit(&mut c);
    c
}

fn none(c: &mut PunishmentConfig) {
    c.fine = 0.0;
    c.punish_cost = 0.0;
}

fn seeds(n: u64) -> Vec<u64> {
    (1..=n).collect()
}

/// `series` at the end of 2 000 periods, from 10 seeds.
fn final_of(c: PunishmentConfig, series: &str) -> Vec<f64> {
    let name = series.to_string();
    model_after(&ModelConfig::Punishment(c), &seeds(10), 2000, move |w| {
        w.model().latest_value(&name).unwrap()
    })
}

/// The long-run cooperation (the last 1 000 of 2 000 periods), 10 seeds.
fn long_run(c: PunishmentConfig) -> Vec<f64> {
    final_of(c, "long_run")
}

/// The mean of `series` over the last 1 000 of 2 000 periods, 10 seeds.
fn window(c: PunishmentConfig, series: &str) -> Vec<f64> {
    let name = series.to_string();
    model_after(&ModelConfig::Punishment(c), &seeds(10), 2000, move |w| {
        let s = w.model().series(&name).unwrap();
        s[1001..].iter().sum::<f64>() / (s.len() - 1001) as f64
    })
}

fn mean(v: &[f64]) -> f64 {
    v.iter().sum::<f64>() / v.len() as f64
}

/// The mean long-run cooperation at each of `sizes`.
fn curve(edit: impl Fn(&mut PunishmentConfig), sizes: &[u32]) -> Vec<f64> {
    sizes
        .iter()
        .map(|&n| {
            mean(&long_run(config(|c| {
                edit(c);
                c.size = n;
            })))
        })
        .collect()
}

fn show(v: &[f64]) -> String {
    let parts: Vec<String> = v.iter().map(|x| format!("{x:.2}")).collect();
    format!("[{}]", parts.join(", "))
}

/// The largest gap between a measured curve and the figure's.
fn gap(measured: &[f64], figure: &[f64]) -> f64 {
    measured
        .iter()
        .zip(figure)
        .map(|(a, b)| (a - b).abs())
        .fold(0.0, f64::max)
}

/// The mean gap: the fit rule every "reproduces Fig. 1" claim uses (≤ 0.05).
fn mean_gap(measured: &[f64], figure: &[f64]) -> f64 {
    distance(measured, figure) / measured.len() as f64
}

/// The summed gap.
fn distance(measured: &[f64], figure: &[f64]) -> f64 {
    measured
        .iter()
        .zip(figure)
        .map(|(a, b)| (a - b).abs())
        .sum()
}

pub fn claims() -> Vec<Claim> {
    vec![
        Claim {
            id: "punishment.bg.fig1a",
            item: "bg-fig1a",
            source: Source::Book,
            citation: BGBR,
            text: "Fig. 1a, without punishment: 'Group selection is ineffective unless groups are quite small' (ε 0.015: long-run cooperation above 0.4 at n 4, below 0.15 from n 32)",
            check: |_| {
                let v = curve(none, &[4, 32, 64]);
                outcome(v[0] > 0.4 && v[1] < 0.15 && v[2] < 0.15, format!("{} at n 4, 32, 64 (figure 0.76, 0.09, 0.09)", show(&v)))
            },
        },
        Claim {
            id: "punishment.bg.fig1b",
            item: "bg-fig1b",
            source: Source::Book,
            citation: BGBR,
            text: "Fig. 1b: 'When there is punishment … group selection can maintain cooperation in substantially larger groups' (ε 0.015: the mean gap to the figure over n 4–256 at most 0.05 — the figure has 0.79 at n 64, 0.64 at 128, 0.55 at 256)",
            check: |_| {
                let v = curve(|_| {}, &SIZES);
                let g = mean_gap(&v, &FIG1B);
                outcome(g <= 0.05, format!("{} at n 4–256; figure {}; mean gap {g:.2}", show(&v), show(&FIG1B)))
                    .with("The shape holds (more cooperation with punishment at every size); the reach does not.")
            },
        },
        Claim {
            id: "punishment.bg.fig1b-helps",
            item: "bg-fig1b",
            source: Source::Book,
            citation: BGBR,
            text: "Punishment sustains more cooperation than its absence in groups too large for group selection alone (n 32, ε 0.015: long-run cooperation with punishment greater)",
            check: |_| {
                let with = long_run(config(|c| c.size = 32));
                let without = long_run(config(|c| {
                    none(c);
                    c.size = 32;
                }));
                greater(&with, &without, "punishment", "none")
            },
        },
        Claim {
            id: "punishment.bg.extinction",
            item: "bg-fig1b",
            source: Source::Book,
            citation: BGBR,
            text: "'increasing the rate of extinction increases the long run average amount of cooperation' (n 64 with punishment: ε 0.03 greater than 0.015, and 0.015 greater than 0.0075)",
            check: |_| {
                let at = |eps: f64| {
                    long_run(config(|c| {
                        c.size = 64;
                        c.conflict = eps;
                    }))
                };
                let (lo, mid, hi) = (at(0.0075), at(0.015), at(0.03));
                all_of(vec![
                    ("0.03 over 0.015".into(), greater(&hi, &mid, "ε 0.03", "ε 0.015")),
                    ("0.015 over 0.0075".into(), greater(&mid, &lo, "ε 0.015", "ε 0.0075")),
                ])
            },
        },
        Claim {
            id: "punishment.bg.caption",
            item: "bg-fig1-caption",
            source: Source::Book,
            citation: BGBR,
            text: "Fig. 1's caption: the conflict rates plotted are '0.075, 0.015, and 0.003' (the legend reads 0.0075, 0.015, 0.03). The caption's rates reproduce the figure's outer curves more closely than the legend's, and than twice the legend's (summed gap over both panels at n 4–256)",
            check: |_| {
                let at = |eps: f64, punish: bool| {
                    curve(
                        |c| {
                            c.conflict = eps;
                            if !punish {
                                none(c);
                            }
                        },
                        &SIZES,
                    )
                };
                let fig = [FIG1A_LOW, FIG1A_HIGH, FIG1B_LOW, FIG1B_HIGH];
                let total = |low: f64, high: f64| {
                    let m = [at(low, false), at(high, false), at(low, true), at(high, true)];
                    m.iter().zip(&fig).map(|(a, b)| distance(a, b)).sum::<f64>()
                };
                let legend = total(0.0075, 0.03);
                let caption = total(0.003, 0.075);
                let doubled = total(0.015, 0.06);
                outcome(
                    caption < legend.min(doubled),
                    format!("summed gap over 28 points: caption's rates {caption:.2}, legend's {legend:.2}, twice the legend's {doubled:.2}"),
                )
                .with("At the stated rates the model's cooperation collapses sooner than the figure's, so the caption's higher top rate looks closer than the legend's; twice the legend's fits best (see punishment.bg.either).")
            },
        },
        Claim {
            id: "punishment.bg.either",
            item: "bg-fig1-either",
            source: Source::Comment,
            citation: BGBR,
            text: "Ours: Fig. 1 is the stated model when either group of a pair can start the conflict, each with probability ε — a pair fights with probability 2ε − ε², about twice the text's, and a group dies at about ε a period, not the ε/2 = 0.0075 their Methods derive. At the legend's ε 0.0075, 0.015 and 0.03, all six curves (both panels) within a mean gap of 0.05 of the figure over n 4–256",
            check: |_| {
                let figures = [
                    (0.0075, false, FIG1A_LOW),
                    (0.015, false, FIG1A),
                    (0.03, false, FIG1A_HIGH),
                    (0.0075, true, FIG1B_LOW),
                    (0.015, true, FIG1B),
                    (0.03, true, FIG1B_HIGH),
                ];
                let parts = figures
                    .iter()
                    .map(|&(eps, punish, fig)| {
                        let v = curve(
                            |c| {
                                c.pairing = Pairing::Either;
                                c.conflict = eps;
                                if !punish {
                                    none(c);
                                }
                            },
                            &SIZES,
                        );
                        let g = mean_gap(&v, &fig);
                        let name = format!("{} ε {eps}", if punish { "1b" } else { "1a" });
                        (name, outcome(g <= 0.05, format!("{} against {} (mean gap {g:.3})", show(&v), show(&fig))))
                    })
                    .collect();
                all_of(parts)
            },
        },
        Claim {
            id: "punishment.bg.either-others",
            item: "bg-fig1-either",
            source: Source::Comment,
            citation: BGBR,
            text: "Ours: the reading found from Fig. 1 (either group starts the conflict) reproduces the figures it was not found from — Fig. 2a and 2b (m 0.002, 0.01, 0.05), Fig. 3 (p 0.4) and Fig. 4 (a fixed cost) at ε 0.015, each curve within a mean gap of 0.05 over n 4–256",
            check: |_| {
                let either = |c: &mut PunishmentConfig| c.pairing = Pairing::Either;
                let mut parts = Vec::new();
                for (k, &m) in [0.002, 0.01, 0.05].iter().enumerate() {
                    for punish in [false, true] {
                        let v = curve(
                            |c| {
                                either(c);
                                c.mixing = m;
                                if !punish {
                                    none(c);
                                }
                            },
                            &SIZES,
                        );
                        let fig = if punish { FIG2B[k] } else { FIG2A[k] };
                        let g = mean_gap(&v, &fig);
                        let name = format!("{} m {m}", if punish { "2b" } else { "2a" });
                        parts.push((name, outcome(g <= 0.05, format!("{} against {} ({g:.3})", show(&v), show(&fig)))));
                    }
                }
                for (name, fig, edit) in [
                    ("3 p 0.4", FIG3_WEAK, (|c: &mut PunishmentConfig| c.fine = 0.4) as fn(&mut PunishmentConfig)),
                    ("4 fixed", FIG4_FIXED, |c| c.punishing = Punishing::Fixed),
                ] {
                    let v = curve(
                        |c| {
                            either(c);
                            edit(c);
                        },
                        &SIZES,
                    );
                    let g = mean_gap(&v, &fig);
                    parts.push((name.into(), outcome(g <= 0.05, format!("{} against {} ({g:.3})", show(&v), show(&fig)))));
                }
                all_of(parts)
            },
        },
        Claim {
            id: "punishment.bg.fig2",
            item: "bg-fig2b",
            source: Source::Book,
            citation: BGBR,
            text: "Fig. 2: 'when the migration rate increases, levels of cooperation fall precipitously … at higher rates of mixing, cooperation does not persist in the largest groups' (with punishment at n 64: m 0.002 greater than 0.01 greater than 0.05; m 0.05 below 0.2 at n 128)",
            check: |_| {
                let at = |m: f64, n: u32| {
                    long_run(config(|c| {
                        c.mixing = m;
                        c.size = n;
                    }))
                };
                let (lo, mid, hi) = (at(0.002, 64), at(0.01, 64), at(0.05, 64));
                let big = mean(&at(0.05, 128));
                all_of(vec![
                    ("0.002 over 0.01".into(), greater(&lo, &mid, "m 0.002", "m 0.01")),
                    ("0.01 over 0.05".into(), greater(&mid, &hi, "m 0.01", "m 0.05")),
                    ("gone in large groups".into(), outcome(big < 0.2, format!("m 0.05, n 128: {big:.2}"))),
                ])
            },
        },
        Claim {
            id: "punishment.bg.fig2-reach",
            item: "bg-fig2b",
            source: Source::Book,
            citation: BGBR,
            text: "Fig. 2b: 'group selection can maintain cooperation in larger groups for all rates of mixing' — at m 0.002 about 0.71 at n 128 and 0.73 at 256 (within 0.1)",
            check: |_| {
                let v = curve(|c| c.mixing = 0.002, &[128, 256]);
                outcome(gap(&v, &[0.71, 0.73]) <= 0.1, format!("{} at n 128, 256", show(&v)))
            },
        },
        Claim {
            id: "punishment.bg.fig3",
            item: "bg-fig3",
            source: Source::Book,
            citation: BGBR,
            text: "Fig. 3: 'Lower values of p result in much lower levels of cooperation' (n 16: p 0.8 greater than 0.4; p 0.4 below 0.25 at n 32 — the figure's 0.21)",
            check: |_| {
                let hi = long_run(config(|c| c.size = 16));
                let lo = long_run(config(|c| {
                    c.size = 16;
                    c.fine = 0.4;
                }));
                let at32 = mean(&long_run(config(|c| {
                    c.size = 32;
                    c.fine = 0.4;
                })));
                all_of(vec![
                    ("lower".into(), greater(&hi, &lo, "p 0.8", "p 0.4")),
                    ("much lower".into(), outcome(at32 < 0.25, format!("p 0.4, n 32: {at32:.2}"))),
                ])
            },
        },
        Claim {
            id: "punishment.bg.fig4",
            item: "bg-fig4",
            source: Source::Book,
            citation: BGBR,
            text: "Fig. 4: 'Punishment does not aid in the evolution of cooperation when the costs born by punishers are fixed' (n 32: a fixed cost c is no better than no punishment — both below 0.2 — while the variable cost is above 0.5)",
            check: |_| {
                let fixed = mean(&long_run(config(|c| {
                    c.size = 32;
                    c.punishing = Punishing::Fixed;
                })));
                let without = mean(&long_run(config(|c| {
                    c.size = 32;
                    none(c);
                })));
                let variable = mean(&long_run(config(|c| c.size = 32)));
                outcome(
                    fixed < 0.2 && without < 0.2 && variable > 0.5,
                    format!("fixed {fixed:.2}, none {without:.2}, variable {variable:.2}"),
                )
            },
        },
        Claim {
            id: "punishment.bg.calibration-spread",
            item: "bg-base",
            source: Source::Book,
            citation: BGBR,
            text: "'we set the cost of cooperation, c, and punishing, k, so that traits with this cost advantage would spread in 50 time periods' (at baseline 1: defectors, with an advantage c, go from 10 % to 90 % of a group of 512 in 35–65 periods)",
            check: |_| {
                // Group 0 starts all punishers (contributors here, with no
                // fines); mutation seeds it with defectors until they pass
                // 10 %, then stops. Groups never mix, so group 0 stands alone.
                let c = config(|c| {
                    c.groups = 2;
                    c.size = 512;
                    none(c);
                    c.error = 0.0;
                    c.mixing = 0.0;
                    c.conflict = 0.0;
                    c.mutation = 0.1;
                });
                let v = model_after(&ModelConfig::Punishment(c.clone()), &seeds(10), 0, move |w| {
                    let ModelWorld::Punishment(start) = w else { unreachable!() };
                    let mut pw = start.as_ref().clone();
                    let share = |pw: &PunishmentWorld| (0..pw.size()).filter(|&a| pw.traits(a).0 < 0.5).count() as f64 / pw.size() as f64;
                    while share(&pw) < 0.1 {
                        pw.step();
                    }
                    let mut still = c.clone();
                    still.mutation = 0.0;
                    Model::set_config(&mut pw, ModelConfig::Punishment(still)).unwrap();
                    let from = pw.tick;
                    while share(&pw) < 0.9 && pw.tick < from + 1000 {
                        pw.step();
                    }
                    (pw.tick - from) as f64
                });
                let m = mean(&v);
                outcome((35.0..=65.0).contains(&m), format!("{m:.1} periods from 10 % to 90 % (10 runs)"))
            },
        },
        Claim {
            id: "punishment.bg.calibration-mixing",
            item: "bg-base",
            source: Source::Book,
            citation: BGBR,
            text: "'The migration rate, m, was set so that … passive diffusion will cause two neighboring groups that are initially as different as possible to achieve the same trait frequencies in ≈50 time periods (m = 0.01)' (two groups of 512, one all punishers, one all defectors, no costs, errors, conflict or mutation: within 0.1 of each other by period 50)",
            check: |_| {
                let c = config(|c| {
                    c.groups = 2;
                    c.size = 512;
                    c.cost = 0.0;
                    none(c);
                    c.error = 0.0;
                    c.conflict = 0.0;
                    c.mutation = 0.0;
                });
                // With two groups, their difference is twice `spread`.
                let d = model_after(&ModelConfig::Punishment(c), &seeds(10), 50, |w| {
                    2.0 * w.model().latest_value("spread").unwrap()
                });
                let m = mean(&d);
                outcome(m <= 0.1, format!("difference {m:.2} after 50 periods (from 1)"))
                    .with("Each period a member meets the other group with probability m and copies it half the time, so the difference shrinks by about m a period: e^(−0.5) ≈ 0.6 remains at 50.")
            },
        },
        Claim {
            id: "punishment.bg.mutation",
            item: "bg-mutation",
            source: Source::Book,
            citation: BGBR,
            text: "'Decreasing the mutation rate substantially increases the long run average levels of cooperation' (n 32: μ 0.001 greater than 0.01, by at least 0.1)",
            check: |_| {
                let lo = long_run(config(|c| c.mutation = 0.001));
                let base = long_run(PunishmentConfig::default());
                let gain = mean(&lo) - mean(&base);
                all_of(vec![
                    ("greater".into(), greater(&lo, &base, "μ 0.001", "μ 0.01")),
                    ("substantially".into(), outcome(gain >= 0.1, format!("gain {gain:.2}"))),
                ])
            },
        },
        Claim {
            id: "punishment.bg.error",
            item: "bg-error",
            source: Source::Book,
            citation: BGBR,
            text: "'Increasing e, the error rate, reduces the long run average amount of cooperation' (n 32: e 0.02 greater than 0.1)",
            check: |_| {
                let base = long_run(PunishmentConfig::default());
                let hi = long_run(config(|c| c.error = 0.1));
                greater(&base, &hi, "e 0.02", "e 0.1")
            },
        },
        Claim {
            id: "punishment.bg.groups",
            item: "bg-groups",
            source: Source::Book,
            citation: BGBR,
            text: "'Reducing the number of groups, N, adds random noise to the results' — noise, not a shift (n 32: the long-run cooperation of 16 groups equivalent to 128's within 0.05)",
            check: |_| {
                let few = long_run(config(|c| c.groups = 16));
                let base = long_run(PunishmentConfig::default());
                equivalent(&few, &base, Some(0.05), "N 16", "N 128")
            },
        },
        Claim {
            id: "punishment.bg.benefit",
            item: "bg-benefit",
            source: Source::Book,
            citation: BGBR,
            text: "A per-capita benefit b/n with conflict decided by payoffs: 'For reasonable values of b (2c, 4c, and 8c), the results of this model are qualitatively similar' (at n 32, punishment sustains more cooperation than its absence at each b)",
            check: |_| {
                let parts = [0.4, 0.8, 1.6]
                    .iter()
                    .map(|&b| {
                        let set = move |c: &mut PunishmentConfig| {
                            c.benefit = b;
                            c.victory = Victory::Payoff;
                        };
                        let with = long_run(config(set));
                        let without = long_run(config(|c| {
                            set(c);
                            none(c);
                        }));
                        (format!("b {b}"), greater(&with, &without, "punishment", "none"))
                    })
                    .collect();
                all_of(parts)
            },
        },
        Claim {
            id: "punishment.bg.continuous",
            item: "bg-continuous",
            source: Source::Book,
            citation: BGBR,
            text: "Continuous traits: 'The steady-state mean levels of cooperation in this model are similar to the base model' (within 0.15 of the base model at n 4, 32, 128, 256)",
            check: |_| {
                let sizes = [4, 32, 128, 256];
                let cont = curve(|c| c.traits = Traits::Continuous, &sizes);
                let base = curve(|_| {}, &sizes);
                let g = gap(&cont, &base);
                outcome(g <= 0.15, format!("continuous {}; base {} (largest gap {g:.2})", show(&cont), show(&base)))
                    .with("Uniform mutants keep the mean punishment trait near ½, and a defector then pays about p/2 = 0.4 > c: cooperation pays within groups.")
            },
        },
        Claim {
            id: "punishment.bg.ring",
            item: "bg-ring",
            source: Source::Book,
            citation: BGBR,
            text: "A ring of groups without extinction: 'We could find no reasonable parameter combination that led to significant long run average levels of cooperation in this last model' (b 2c and 8c, m 0.01, n 4, 8, 16: all below 0.3)",
            check: |_| {
                let mut worst = (0.0, 0.0, 0);
                for b in [0.4, 1.6] {
                    let v = curve(
                        |c| {
                            c.structure = Structure::Ring;
                            c.benefit = b;
                        },
                        &[4, 8, 16],
                    );
                    for (i, &x) in v.iter().enumerate() {
                        if x > worst.0 {
                            worst = (x, b, [4, 8, 16][i]);
                        }
                    }
                }
                outcome(worst.0 < 0.3, format!("highest {:.2} (b {}, n {})", worst.0, worst.1, worst.2))
            },
        },
        Claim {
            id: "punishment.bg.hundred",
            item: "bg-large",
            source: Source::Book,
            citation: BGBR,
            text: "'With parameter values chosen to represent cultural evolution in small-scale societies, cooperation is sustained in groups on the order of 100 individuals' (n 128: long-run cooperation at least 0.5)",
            check: |_| {
                let v = mean(&long_run(config(|c| c.size = 128)));
                outcome(v >= 0.5, format!("{v:.2} at n 128 (figure 0.64)"))
            },
        },
        Claim {
            id: "punishment.bg.baseline",
            item: "bg-baseline",
            source: Source::Comment,
            citation: BGBR,
            text: "Ours: the paper never states the payoff costs are subtracted from; at the stated conflict rate, some baseline reproduces Figs. 1a and 1b together (ε 0.015: a mean gap of at most 0.05 to both panels over n 4–256, for a baseline of 1, 2, 3 or 4)",
            check: |_| {
                let mut lines = Vec::new();
                let mut any = false;
                for base in [1.0, 2.0, 3.0, 4.0] {
                    let b = curve(|c| c.baseline = base, &SIZES);
                    let a = curve(
                        |c| {
                            c.baseline = base;
                            none(c);
                        },
                        &SIZES,
                    );
                    let (ga, gb) = (mean_gap(&a, &FIG1A), mean_gap(&b, &FIG1B));
                    any |= ga <= 0.05 && gb <= 0.05;
                    lines.push(format!("baseline {base}: 1a {} ({ga:.2}), 1b {} ({gb:.2})", show(&a), show(&b)));
                }
                outcome(any, lines.join("; "))
                    .with("A higher baseline weakens selection: punishment reaches larger groups, but without it the floor rises above the figure's 0.09. When either group can start a conflict, baseline 1 fits both (punishment.bg.either).")
            },
        },
        Claim {
            id: "punishment.janssen.readings",
            item: "bg-readings",
            source: Source::Comment,
            citation: JANSSEN,
            text: "Janssen's readings together — a benefit 0.5 × the share cooperating, every group challenging one (about twice the conflict), d counted by acts, imitation in turn, erring punishers punishing themselves — reproduce Fig. 1 at ε 0.015 (a mean gap of at most 0.05 to both panels over n 4–256)",
            check: |_| {
                let janssen = |c: &mut PunishmentConfig| {
                    c.benefit = 0.5;
                    c.pairing = Pairing::Challenge;
                    c.counted = Counted::Acts;
                    c.imitation = Imitation::InTurn;
                    c.erring = Erring::Itself;
                };
                let b = curve(janssen, &SIZES);
                let a = curve(
                    |c| {
                        janssen(c);
                        none(c);
                    },
                    &SIZES,
                );
                let (ga, gb) = (mean_gap(&a, &FIG1A), mean_gap(&b, &FIG1B));
                outcome(ga <= 0.05 && gb <= 0.05, format!("1a {} ({ga:.3}); 1b {} ({gb:.3})", show(&a), show(&b)))
                    .with("His challenge pairing roughly doubles the conflict; his benefit lifts cooperation in small groups without punishment.")
            },
        },
        Claim {
            id: "punishment.cooney.fine",
            item: "bg-cooney-fine",
            source: Source::Comment,
            citation: COONEY,
            text: "Under conflict decided by average payoffs normalized by the widest possible difference, 'a non-monotonic dependence of long-time average payoff on the strength of punishment' — and (Remark 6.1) not under the Fermi (tanh) rule (b 2c, n 32: the mean payoff over the last 1 000 periods dips below p = 0's and recovers above the dip, each by Mann–Whitney p < 0.01; under tanh, no such dip)",
            check: |_| {
                let fines = [0.0, 0.2, 0.3, 0.4, 0.5, 0.6, 0.8, 1.2, 1.6, 2.4];
                let dip = |victory: Victory| {
                    let runs: Vec<Vec<f64>> = fines
                        .iter()
                        .map(|&p| {
                            window(
                                config(|c| {
                                    c.benefit = 0.4;
                                    c.victory = victory;
                                    c.fine = p;
                                }),
                                "payoff",
                            )
                        })
                        .collect();
                    let means: Vec<f64> = runs.iter().map(|r| mean(r)).collect();
                    let low = (1..fines.len() - 1)
                        .min_by(|&a, &b| means[a].total_cmp(&means[b]))
                        .unwrap();
                    let down = greater(&runs[0], &runs[low], "p 0", "the dip");
                    let up = greater(&runs[fines.len() - 1], &runs[low], "p 2.4", "the dip");
                    let found = down.verdict == Verdict::Holds && up.verdict == Verdict::Holds;
                    (found, format!("{} at p {:?}; lowest at p {}", show(&means), fines, fines[low]))
                };
                let (normalized, a) = dip(Victory::Payoff);
                let (tanh, b) = dip(Victory::Tanh);
                all_of(vec![
                    ("a dip, normalized".into(), outcome(normalized, a)),
                    ("none under tanh".into(), outcome(!tanh, b)),
                ])
                .with("In this finite, mutating population, weak punishment costs its punishers without deterring anyone, whichever rule decides conflicts.")
            },
        },
        Claim {
            id: "punishment.cooney.cost",
            item: "bg-cooney-cost",
            source: Source::Comment,
            citation: COONEY,
            text: "Under normalized payoff conflict, 'increasing the cost of punishing defectors can increase the level of altruistic punishment at steady state' (b 2c, n 32: for some k₁ < k₂ in 0.05, 0.1, 0.2, 0.4, 0.8, the mean punishment over the last 1 000 periods is greater at k₂, Mann–Whitney p < 0.01)",
            check: |_| {
                let ks = [0.05, 0.1, 0.2, 0.4, 0.8];
                let runs: Vec<Vec<f64>> = ks
                    .iter()
                    .map(|&k| {
                        window(
                            config(|c| {
                                c.benefit = 0.4;
                                c.victory = Victory::Payoff;
                                c.punish_cost = k;
                            }),
                            "punishment",
                        )
                    })
                    .collect();
                let mut rises = Vec::new();
                for i in 0..ks.len() {
                    for j in i + 1..ks.len() {
                        if greater(&runs[j], &runs[i], "", "").verdict == Verdict::Holds {
                            rises.push(format!("{} → {}", ks[i], ks[j]));
                        }
                    }
                }
                let means: Vec<f64> = runs.iter().map(|r| mean(r)).collect();
                outcome(!rises.is_empty(), format!("{} at k {:?}; rises: {}", show(&means), ks, if rises.is_empty() { "none".into() } else { rises.join(", ") }))
            },
        },
        Claim {
            id: "punishment.bg.start",
            item: "bg-base",
            source: Source::Comment,
            citation: BGBR,
            text: "Ours: the paper's start — one group of punishers among 127 of defectors, 'Various random processes could cause such an initial shift' — does not decide the long run (n 32: from all defectors, equivalent within 0.05)",
            check: |_| {
                let base = long_run(PunishmentConfig::default());
                let from = long_run(config(|c| c.start = Start::AllDefectors));
                equivalent(&from, &base, Some(0.05), "all defectors", "one punisher group")
            },
        },
    ]
}
