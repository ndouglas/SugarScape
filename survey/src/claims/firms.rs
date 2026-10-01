//! The Emergence of Firms (milestone 33): Axtell (1999). The simulation
//! claims run 10 seeds × 5 000 periods (burn-in 500) of 1 000 agents unless
//! stated; µ is the paper's OLS on the log-log size pmf (size 1 and
//! frequencies below 10⁻⁵ dropped) unless the claim says maximum likelihood.
//! §4's tables are single estimates with no standard errors, and our base µ
//! is not the paper's, so each table is judged by direction (Mann–Whitney
//! between its extremes), with the paper's numbers beside ours.

use sugarscape_core::firms::effort::{eigenvalue, max_stable_size, nash, optimal_size};
use sugarscape_core::firms::fit::{
    gamma, growth_fit, growth_fit_nonzero, lifetimes, mu_mle, mu_ols, output_exponent,
    productivity, GrowthFit,
};
use sugarscape_core::firms::{
    Activation, AdjustScope, EffortSearch, FirmsConfig, FirmsWorld, Initial, Network, OthersEffort,
    Pay, Preferences, RandomBehavior, SeniorityOrder,
};
use sugarscape_core::model::{ModelConfig, ModelWorld};

use crate::claim::{all_of, greater, Claim, Outcome, Source, Verdict};
use crate::runner::model_after;

const A99: &str = "Axtell 1999, Brookings CSED WP 3";

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

fn seeds(n: u64) -> Vec<u64> {
    (1..=n).collect()
}

fn worlds(c: FirmsConfig, n: u64) -> Vec<FirmsWorld> {
    model_after(&ModelConfig::Firms(c), &seeds(n), 100_000, |w| match w {
        ModelWorld::Firms(f) => f.as_ref().clone(),
        _ => unreachable!(),
    })
}

fn base() -> FirmsConfig {
    FirmsConfig::default()
}

fn median(v: &[f64]) -> f64 {
    let mut s: Vec<f64> = v.iter().copied().filter(|x| x.is_finite()).collect();
    s.sort_by(f64::total_cmp);
    if s.is_empty() {
        return f64::NAN;
    }
    let m = s.len() / 2;
    if s.len().is_multiple_of(2) {
        0.5 * (s[m - 1] + s[m])
    } else {
        s[m]
    }
}

fn mean(v: &[f64]) -> f64 {
    let f: Vec<f64> = v.iter().copied().filter(|x| x.is_finite()).collect();
    f.iter().sum::<f64>() / f.len().max(1) as f64
}

fn show(v: &[f64]) -> String {
    format!(
        "[{}]",
        v.iter()
            .map(|x| format!("{x:.2}"))
            .collect::<Vec<_>>()
            .join(", ")
    )
}

/// A median, plus how many of its (seed) values were finite — `median` itself
/// drops non-finite ones silently, so this surfaces how many it kept.
fn median_with_n(v: &[f64]) -> String {
    let n = v.iter().filter(|x| x.is_finite()).count();
    if n == v.len() {
        format!("{:.2} ({n})", median(v))
    } else {
        format!("{:.2} ({n} of {})", median(v), v.len())
    }
}

/// Each seed's µ (the paper's OLS).
fn mus(ws: &[FirmsWorld]) -> Vec<f64> {
    ws.iter().map(|w| mu_ols(&w.records().sizes)).collect()
}

fn mus_of(c: FirmsConfig) -> Vec<f64> {
    mus(&worlds(c, 10))
}

/// A series' mean over the periods after the burn-in, per seed.
fn after_burn(ws: &[FirmsWorld], f: fn(&sugarscape_core::firms::FirmsSnapshot) -> f64) -> Vec<f64> {
    ws.iter()
        .map(|w| {
            let h = w.stats.history();
            let from = w.config.burn_in as usize + 1;
            mean(&h[from.min(h.len())..].iter().map(f).collect::<Vec<_>>())
        })
        .collect()
}

/// "µ falls from `lo_name` to `hi_name`": Mann–Whitney that `hi`'s µ are below `lo`'s.
fn falls(lo: Vec<f64>, hi: Vec<f64>, lo_name: &str, hi_name: &str, paper: &str) -> Outcome {
    greater(&lo, &hi, lo_name, hi_name).with(&format!(
        "medians {:.2} and {:.2}; the paper: {paper}",
        median(&lo),
        median(&hi)
    ))
}

pub fn claims() -> Vec<Claim> {
    vec![
        Claim {
            id: "firms.a99.table1",
            item: "firms-base",
            source: Source::Book,
            citation: A99,
            text: "Table 1 (θ = 0.7, a = b = 1): the Nash effort, utility and dominant eigenvalue (N − 1)k for N = 1–7 — each to 3 decimal places; stable to 6, optimal at 5",
            check: |_| {
                let rows = [(1, 0.770, 0.799, 0.0), (2, 0.646, 0.964, -0.188), (3, 0.558, 1.036, -0.368), (4, 0.492, 1.065, -0.547), (5, 0.441, 1.069, -0.726), (6, 0.399, 1.061, -0.904), (7, 0.364, 1.045, -1.082)];
                let ok = rows.iter().all(|&(n, e, u, l)| {
                    let (ne, nu) = nash(0.7, n, 1.0, 1.0);
                    (ne - e).abs() < 5e-4 && (nu - u).abs() < 5e-4 && (n == 1 || (eigenvalue(0.7, n, 1.0, 1.0) - l).abs() < 5e-4)
                });
                let sizes = (max_stable_size(0.7, 1.0, 1.0, 50), optimal_size(0.7, 1.0, 1.0, 50));
                outcome(ok && sizes == (6, 5), format!("every row within 0.0005; stable to {}, optimal {}", sizes.0, sizes.1))
                    .with("The text's eigenvalue for N = 3, −0.552, is not the table's −0.368 (the table is right).")
            },
        },
        Claim {
            id: "firms.a99.sizes",
            item: "firms-base",
            source: Source::Book,
            citation: A99,
            text: "Figs. 6 and 9: 'Optimal group sizes are relatively small—less than 10—for agents having θ < 0.85, then rise quickly', and 'the optimal size of a homogeneous group is very nearly at the stability boundary' — optimal size under 10 at θ = 0.84, at least 30 at θ = 0.95, and the maximum stable size within 1 of the optimum for θ from 0.3 to 0.95",
            check: |_| {
                let grid: Vec<f64> = (0..14).map(|k| 0.3 + 0.05 * f64::from(k)).collect();
                let pairs: Vec<(u32, u32)> = grid.iter().map(|&t| (optimal_size(t, 1.0, 1.0, 400), max_stable_size(t, 1.0, 1.0, 400))).collect();
                let near = pairs.iter().all(|&(o, s)| s >= o && s - o <= 1);
                let (o84, o95) = (optimal_size(0.84, 1.0, 1.0, 400), optimal_size(0.95, 1.0, 1.0, 400));
                outcome(near && o84 < 10 && o95 >= 30, format!("optimal 0.84 → {o84}, 0.95 → {o95}; (optimal, stable) θ 0.3–0.95: {pairs:?}"))
            },
        },
        Claim {
            id: "firms.a99.mu",
            item: "firms-base",
            source: Source::Book,
            citation: A99,
            text: "§3.4: firm sizes follow a power law with 'µ = 1.28' (OLS, adjusted R² 0.99) — the median over 10 seeds within 0.15 of 1.28 (0.15: about the spread of Table 11's four identical rows)",
            check: |_| {
                let ws = worlds(base(), 10);
                let (m, mle) = (mus(&ws), ws.iter().map(|w| mu_mle(&w.records().sizes, 2)).collect::<Vec<_>>());
                outcome((median(&m) - 1.28).abs() <= 0.15, format!("OLS µ by seed {} (median {:.2}); maximum likelihood {} (median {:.2})", show(&m), median(&m), show(&mle), median(&mle)))
            },
        },
        Claim {
            id: "firms.a99.output",
            item: "firms-base",
            source: Source::Book,
            citation: A99,
            text: "Fig. 18: firm output follows a power law with exponent 0.88; fig. 19: productivity 0.58·s^1.15, 'the hypothesis of constant returns cannot be rejected' — the output exponent within 0.15 of 0.88, and the productivity exponent between 0.9 and 1.25 (medians, 10 seeds)",
            check: |_| {
                let ws = worlds(base(), 10);
                let out: Vec<f64> = ws.iter().map(|w| output_exponent(&w.records().outputs)).collect();
                let prod: Vec<f64> = ws.iter().map(|w| productivity(&w.records().size_output).1).collect();
                all_of(vec![
                    ("output exponent".into(), outcome((median(&out) - 0.88).abs() <= 0.15, format!("{} (median {:.2})", show(&out), median(&out)))),
                    ("productivity".into(), outcome((0.9..=1.25).contains(&median(&prod)), format!("{} (median {:.2})", show(&prod), median(&prod)))),
                ])
            },
        },
        Claim {
            id: "firms.a99.growth",
            item: "firms-base",
            source: Source::Book,
            citation: A99,
            text: "Fig. 20: growth rates are better fit by a Laplace than a Gaussian; fig. 21: σ_r ∝ s^−γ with 'γ = 0.174 ± 0.004' — the Laplace's log-likelihood higher in at least 8 of 10 seeds, and γ (sizes 3–300, at least 30 observations each) within 0.008 (twice the reported error) of 0.174",
            check: |_| {
                let ws = worlds(base(), 10);
                let fits: Vec<_> = ws.iter().map(|w| growth_fit(&w.records().growth)).collect();
                let laplace = fits.iter().filter(|f| f.laplace_ll > f.gauss_ll).count();
                let g: Vec<f64> = ws.iter().map(|w| gamma(&w.records().growth_by_size, 3, 300, 30)).collect();
                // Not part of the rule: how much of the Laplace's win is the
                // spike of firms that kept their size (r = 0).
                let zero: Vec<f64> = ws.iter().zip(&fits).map(|(w, f)| w.records().zero_growth as f64 / f.n as f64).collect();
                let nonzero: Vec<_> = ws.iter().map(|w| growth_fit_nonzero(w.records())).collect();
                let nz_laplace = nonzero.iter().filter(|f| f.laplace_ll > f.gauss_ll).count();
                let nz_ll = |pick: fn(&GrowthFit) -> f64| mean(&nonzero.iter().map(pick).collect::<Vec<_>>());
                let range = |v: &[f64]| (v.iter().copied().fold(f64::INFINITY, f64::min), v.iter().copied().fold(f64::NEG_INFINITY, f64::max));
                let ((z_lo, z_hi), (g_lo, g_hi)) = (range(&zero), range(&g));
                all_of(vec![
                    ("Laplace".into(), outcome(laplace >= 8, format!("{laplace} of 10 seeds; mean sd of r {:.3}; {:.0}–{:.0} % of rates are zero; without them the Laplace wins {nz_laplace} of 10 (mean log-likelihood {:.2} Laplace, {:.2} Gaussian)", mean(&fits.iter().map(|f| f.sd).collect::<Vec<_>>()), 100.0 * z_lo, 100.0 * z_hi, nz_ll(|f| f.laplace_ll), nz_ll(|f| f.gauss_ll)))),
                    ("γ".into(), outcome((median(&g) - 0.174).abs() <= 0.008, format!("{} (median {:.3}, seeds {g_lo:.2}–{g_hi:.2})", show(&g), median(&g)))),
                ])
                .with(&format!("The Laplace's Holds rests on the zero growth rates: {:.0}–{:.0} % of observations are firms that kept their size, singletons included, and on the nonzero rates alone the Laplace wins {nz_laplace} of 10 seeds. Per seed, γ runs from {g_lo:.2} to {g_hi:.2}, against the paper's ± 0.004.", 100.0 * z_lo, 100.0 * z_hi))
            },
        },
        Claim {
            id: "firms.a99.lifetimes",
            item: "firms-base",
            source: Source::Book,
            citation: A99,
            text: "p. 46–47: firm lifetimes average 23.4 periods (sd 27.1), and lifetime is linear in log rank with slope about −70 — the mean within 25 % of 23.4 and the slope between −105 and −35 (medians, 10 seeds)",
            check: |_| {
                let ws = worlds(base(), 10);
                let l: Vec<(u64, f64, f64, f64)> = ws.iter().map(|w| lifetimes(&w.records().lifetimes)).collect();
                let team: Vec<f64> = ws.iter().map(|w| lifetimes(&w.records().lifetimes_team).1).collect();
                let (m, slope) = (median(&l.iter().map(|x| x.1).collect::<Vec<_>>()), median(&l.iter().map(|x| x.3).collect::<Vec<_>>()));
                outcome((m - 23.4).abs() <= 0.25 * 23.4 && (-105.0..=-35.0).contains(&slope), format!("mean {m:.1} (sd {:.1}); firms that ever had two members {:.1}; rank slope {slope:.1}", median(&l.iter().map(|x| x.2).collect::<Vec<_>>()), median(&team)))
                    .with("The paper's own counts disagree: about 45 births a period × 23.4 periods would need about 1 050 firms of 1 000 agents.")
            },
        },
        Claim {
            id: "firms.a99.levels",
            item: "firms-base",
            source: Source::Book,
            citation: A99,
            text: "§3.3–3.4: 'The average firm size is about 4', the largest firm reaches about 200, and figures 14–16 show total output of about 450–600 — mean size between 3 and 5, the largest firm at least 150 in at least 5 of 10 seeds, and output between 450 and 600 (means after the burn-in)",
            check: |_| {
                let ws = worlds(base(), 10);
                let size = after_burn(&ws, |s| s.mean_size);
                let largest: Vec<f64> = ws.iter().map(|w| w.stats.history().iter().skip(w.config.burn_in as usize).map(|s| f64::from(s.largest)).fold(0.0, f64::max)).collect();
                let out = after_burn(&ws, |s| s.output);
                all_of(vec![
                    ("mean size".into(), outcome((3.0..=5.0).contains(&median(&size)), format!("{} (median {:.2})", show(&size), median(&size)))),
                    ("largest".into(), outcome(largest.iter().filter(|&&x| x >= 150.0).count() >= 5, show(&largest))),
                    ("output".into(), outcome((450.0..=600.0).contains(&median(&out)), format!("{} (median {:.0})", show(&out), median(&out)))),
                ])
                .with("With everyone alone at the start, output is already about 934 (mean singleton output 0.934 × 1 000).")
            },
        },
        Claim {
            id: "firms.a99.beta",
            item: "firms-beta",
            source: Source::Book,
            citation: A99,
            text: "Table 3: µ falls as increasing returns β rise (2.06 at β 1.7 to 0.95 at 2.1) — µ at β 1.7 above µ at β 2.1",
            check: |_| falls(mus_of(FirmsConfig { beta: 1.7, ..base() }), mus_of(FirmsConfig { beta: 2.1, ..base() }), "β 1.7", "β 2.1", "2.06 and 0.95"),
        },
        Claim {
            id: "firms.a99.b",
            item: "firms-b",
            source: Source::Book,
            citation: A99,
            text: "Table 4: µ falls as b rises (2.09 at b 0.5 to 0.53 at 1.5), and b drawn per firm from [0.5, 1.5] 'behaves much more like b = 1.25, not b = 1.0' — µ at b 0.5 above µ at 1.5, and the drawn b's median µ nearer b 1.25's than b 1.0's",
            check: |_| {
                let at = |b: f64| mus_of(FirmsConfig { b, ..base() });
                let (lo, hi, one, one25) = (at(0.5), at(1.5), at(1.0), at(1.25));
                let drawn = mus_of(FirmsConfig { b: 0.5, b_max: 1.5, ..base() });
                let nearer = (median(&drawn) - median(&one25)).abs() < (median(&drawn) - median(&one)).abs();
                all_of(vec![
                    ("falls".into(), falls(lo, hi, "b 0.5", "b 1.5", "2.09 and 0.53")),
                    ("drawn b".into(), outcome(nearer, format!("drawn {:.2}; b 1.0 {:.2}; b 1.25 {:.2} (the paper: 0.89, 1.28, 0.91)", median(&drawn), median(&one), median(&one25)))),
                ])
            },
        },
        Claim {
            id: "firms.a99.preferences",
            item: "firms-preferences",
            source: Source::Book,
            citation: A99,
            text: "Table 5: 'the general power-law character … remain[s]' across preference distributions, and the homogeneous θ = 0.75 gives more large firms (0.91 against 1.28) — every row's median µ (OLS) between 0.5 and 3, and µ at θ = 0.75 below θ ~ U[0, 1]'s",
            check: |_| {
                let rows: [(&str, FirmsConfig); 9] = [
                    ("uniform", base()),
                    ("middle", FirmsConfig { preferences: Preferences::Middle, ..base() }),
                    ("triangular", FirmsConfig { preferences: Preferences::Triangular, ..base() }),
                    ("triangular 0.75", FirmsConfig { preferences: Preferences::TriangularHigh, ..base() }),
                    ("normal", FirmsConfig { preferences: Preferences::Normal, ..base() }),
                    ("beta", FirmsConfig { preferences: Preferences::Beta, ..base() }),
                    ("θ 0.75", FirmsConfig { preferences: Preferences::Fixed, theta: 0.75, ..base() }),
                    ("CES ρ −1–0", FirmsConfig { preferences: Preferences::Ces, rho: -1.0, rho_max: 0.0, ..base() }),
                    ("CES ρ 0–10", FirmsConfig { preferences: Preferences::Ces, rho: 0.0, rho_max: 10.0, ..base() }),
                ];
                let results: Vec<(&str, Vec<f64>)> = rows.into_iter().map(|(n, c)| (n, mus_of(c))).collect();
                let medians: Vec<f64> = results.iter().map(|(_, m)| median(m)).collect();
                let character = medians.iter().all(|m| (0.5..=3.0).contains(m));
                let with_n: Vec<String> = results.iter().map(|(_, m)| median_with_n(m)).collect();
                all_of(vec![
                    ("power-law character".into(), outcome(character, format!("medians [{}] (the paper: 1.28, 1.21, 1.31, 1.01, 1.30, 0.99, 0.91, 1.56, 1.26)", with_n.join(", ")))),
                    ("homogeneous".into(), falls(results[0].1.clone(), results[6].1.clone(), "uniform", "θ 0.75", "1.28 and 0.91")),
                ])
            },
        },
        Claim {
            id: "firms.a99.networks",
            item: "firms-friends",
            source: Source::Book,
            citation: A99,
            text: "Tables 6–7: larger networks 'stabilize large firms' — µ at ν 2 above µ at ν 10, for fixed friends (1.28 to 0.99) and for random firms (1.28 to 1.03)",
            check: |_| {
                let f = |network, neighbors| mus_of(FirmsConfig { network, neighbors, ..base() });
                all_of(vec![
                    ("friends".into(), falls(f(Network::Friends, 2), f(Network::Friends, 10), "ν 2", "ν 10", "1.28 and 0.99")),
                    ("random firms".into(), falls(f(Network::RandomFirms, 2), f(Network::RandomFirms, 10), "ν 2", "ν 10", "1.28 and 1.03")),
                ])
            },
        },
        Claim {
            id: "firms.a99.loyalty",
            item: "firms-loyalty",
            source: Source::Book,
            citation: A99,
            text: "Table 8: 'loyalty is a stabilizing factor for large firms … relatively strong effect' — µ at λ 0 above µ at λ 10 (1.28 and 0.77)",
            check: |_| falls(mus_of(base()), mus_of(FirmsConfig { loyalty: 10, ..base() }), "λ 0", "λ 10", "1.28 and 0.77"),
        },
        Claim {
            id: "firms.a99.sticky-groping",
            item: "firms-sticky",
            source: Source::Book,
            citation: A99,
            text: "Tables 9–10 (β 2): sticky effort (±0.05) and groping both lower µ (1.28 to 0.92 and 1.19), groping's effect 'more pronounced' (the text; the tables say less) — µ free above µ sticky, µ free above µ groping, and µ groping below µ sticky (the text's claim)",
            check: |_| {
                let (free, sticky, groping) = (mus_of(base()), mus_of(FirmsConfig { effort_window: 0.1, ..base() }), mus_of(FirmsConfig { groping: true, ..base() }));
                all_of(vec![
                    ("sticky".into(), falls(free.clone(), sticky.clone(), "free", "sticky", "1.28 and 0.92")),
                    ("groping".into(), falls(free, groping.clone(), "free", "groping", "1.28 and 1.19")),
                    ("more pronounced".into(), falls(sticky, groping, "sticky", "groping", "0.92 and 1.19 — the table contradicts the text")),
                ])
                .with("The sub-checks hold, but our sticky and groping µ (−0.11, −0.23) are negative: not a power law, but a one-giant-firm regime — under the literal adjust_scope = everywhere, the largest firm holds all 1 000 agents in every seed (see firms.ours.adjust-scope), while the paper's 0.92 and 1.19 are power laws.")
                .with("This claim tests the text's 'groping … more pronounced'; the spec and the paper's own tables give the opposite direction (sticky 0.92 against groping 1.19), and under that direction the third sub-check would fail.")
            },
        },
        Claim {
            id: "firms.a99.seniority",
            item: "firms-seniority",
            source: Source::Book,
            citation: A99,
            text: "Table 11: seniority shares 'make large firms somewhat more stable' (5^−rank 1.11 against equal shares 1.28) — µ under equal shares above µ under 5^−rank",
            check: |_| falls(mus_of(base()), mus_of(FirmsConfig { pay: Pay::Seniority, seniority_base: 5.0, ..base() }), "equal", "5^−rank", "1.28 and 1.11"),
        },
        Claim {
            id: "firms.a99.base-pay",
            item: "firms-base-pay",
            source: Source::Book,
            citation: A99,
            text: "Table 12: base pay 'stabilize[s] large firms somewhat' (80 % of singleton income: 0.85 against 1.28) — µ under equal shares above µ with base pay at 80 % of each agent's singleton income",
            check: |_| falls(mus_of(base()), mus_of(FirmsConfig { pay: Pay::Base, base_share: 0.8, ..base() }), "equal", "base 80 %", "1.28 and 0.85"),
        },
        Claim {
            id: "firms.a99.hiring",
            item: "firms-hiring",
            source: Source::Book,
            citation: A99,
            text: "Table 13 (captioned 'target output', a rule the paper never gives; the rows are hiring standards): standards 'first stabilize large firms to some extent … and then … destabilize them', and at φ = 100 % the power law 'breaks down' — µ at φ 60 % below µ at 0 % and the largest firm at φ 100 % (median over seeds) under 20",
            check: |_| {
                let at = |h: f64| worlds(FirmsConfig { hiring: h, ..base() }, 10);
                let (zero, sixty, full) = (at(0.0), at(0.6), at(1.0));
                let largest: Vec<f64> = full.iter().map(|w| w.stats.history().iter().map(|s| f64::from(s.largest)).fold(0.0, f64::max)).collect();
                all_of(vec![
                    ("stabilize".into(), falls(mus(&zero), mus(&sixty), "φ 0", "φ 60 %", "1.28 and 1.03")),
                    ("breaks down".into(), outcome(median(&largest) < 20.0, format!("largest firm at φ 100 %: {}", show(&largest)))),
                ])
            },
        },
        Claim {
            id: "firms.a99.random",
            item: "firms-random-choices",
            source: Source::Book,
            citation: A99,
            text: "§4.1: random choices 'fail to yield a power law … firms greater than 9 or 10 are rarely observed', and random effort gives 'nothing like power law size distributions' — under 1 % of sampled firms larger than 10, for each (median over 10 seeds)",
            check: |_| {
                let share = |c: FirmsConfig| {
                    worlds(c, 10)
                        .iter()
                        .map(|w| {
                            let s = &w.records().sizes;
                            let total: u64 = s.iter().sum();
                            let largest = w.stats.history().iter().map(|x| x.largest).max().unwrap_or(0);
                            (s.iter().skip(11).sum::<u64>() as f64 / total.max(1) as f64, f64::from(largest))
                        })
                        .collect::<Vec<(f64, f64)>>()
                };
                let (cl, el) = (share(FirmsConfig { random_behavior: RandomBehavior::Choices, ..base() }), share(FirmsConfig { random_behavior: RandomBehavior::Effort, ..base() }));
                let (choices, effort): (Vec<f64>, Vec<f64>) = (cl.iter().map(|x| x.0).collect(), el.iter().map(|x| x.0).collect());
                let largest: Vec<f64> = el.iter().map(|x| x.1).collect();
                all_of(vec![
                    ("random choices".into(), outcome(median(&choices) < 0.01, format!("share above 10: {}", show(&choices.iter().map(|x| 100.0 * x).collect::<Vec<_>>())) + " %")),
                    ("random effort".into(), outcome(median(&effort) < 0.01, format!("share above 10: {}", show(&effort.iter().map(|x| 100.0 * x).collect::<Vec<_>>())) + " %")
                        .with("The paper says only that random effort gives 'nothing like power law size distributions' (§4.1, §5); our run, where one firm holds everyone, satisfies that literal statement. The Fails here comes from reusing random choices' 'rarely greater than 9 or 10' threshold, which the paper states only for random choices.")),
                ])
                .with(&format!("Under random effort the largest firm reaches {} of 1 000 agents: not a power law, but of the opposite kind — agents gather in giant firms.", show(&largest)))
            },
        },
        Claim {
            id: "firms.a99.invariances",
            item: "firms-readings",
            source: Source::Book,
            citation: A99,
            text: "§4 introduction: population size ('invariant'), uniform against random activation ('essentially no difference') and the initial condition ('did not seem to matter much') leave µ alone — each within 0.15 of the base case's median µ (10 000 agents: 3 seeds, 2 000 periods)",
            check: |_| {
                let b = median(&mus_of(base()));
                let near = |name: &str, m: f64| (name.to_string(), outcome((m - b).abs() <= 0.15, format!("{m:.2} against {b:.2}")));
                let big = median(&mus(&worlds(FirmsConfig { agents: 10_000, stop_at: 2000, ..base() }, 3)));
                let small = median(&mus(&worlds(FirmsConfig { stop_at: 2000, ..base() }, 3)));
                all_of(vec![
                    ("10 000 agents".into(), outcome((big - small).abs() <= 0.15, format!("{big:.2} against {small:.2} at 1 000 (2 000 periods)"))),
                    near("uniform activation", median(&mus_of(FirmsConfig { activation: Activation::Uniform, ..base() }))),
                    near("random groups", median(&mus_of(FirmsConfig { initial: Initial::RandomGroups, ..base() }))),
                    near("one firm", median(&mus_of(FirmsConfig { initial: Initial::OneFirm, ..base() }))),
                ])
            },
        },
        Claim {
            id: "firms.ours.readings",
            item: "firms-readings",
            source: Source::Comment,
            citation: A99,
            text: "Ours: no reading of the unstated rules gives µ = 1.28 — the median µ over 10 seeds more than 0.15 from 1.28 under the literal reading, live efforts, uniform activation, both, and a coarse line search",
            check: |_| {
                let cases: [(&str, FirmsConfig); 5] = [
                    ("literal", base()),
                    ("live", FirmsConfig { others_effort: OthersEffort::Live, ..base() }),
                    ("uniform", FirmsConfig { activation: Activation::Uniform, ..base() }),
                    ("live and uniform", FirmsConfig { others_effort: OthersEffort::Live, activation: Activation::Uniform, ..base() }),
                    ("grid of 10", FirmsConfig { effort_search: EffortSearch::Grid, grid_steps: 10, ..base() }),
                ];
                all_of(cases.into_iter().map(|(n, c)| {
                    let m = median(&mus_of(c));
                    (n.to_string(), outcome((m - 1.28).abs() > 0.15, format!("{m:.2}")))
                }).collect())
            },
        },
        Claim {
            id: "firms.ours.estimators",
            item: "firms-base",
            source: Source::Comment,
            citation: A99,
            text: "Ours: the paper's OLS and a maximum-likelihood fit disagree by more than 0.5 on the same sizes (medians, 10 seeds) — the distribution is not one clean power law",
            check: |_| {
                let ws = worlds(base(), 10);
                let (o, m) = (median(&mus(&ws)), median(&ws.iter().map(|w| mu_mle(&w.records().sizes, 2)).collect::<Vec<_>>()));
                outcome((o - m).abs() > 0.5, format!("OLS {o:.2}, maximum likelihood {m:.2}"))
            },
        },
        Claim {
            id: "firms.ours.noise",
            item: "firms-seniority",
            source: Source::Comment,
            citation: A99,
            text: "Ours: Table 11's rows 2^−rank, 2^−(rank+1), 2^−(rank+2) and 2^−(rank+3) are one model (normalized shares are identical), so their spread (0.89–1.07) is the paper's run-to-run noise; ours, seed to seed at the base case, is at least as large (the range of 10 seeds at least 0.18)",
            check: |_| {
                let m = mus_of(base());
                let range = m.iter().copied().fold(f64::MIN, f64::max) - m.iter().copied().fold(f64::MAX, f64::min);
                outcome(range >= 0.18, format!("range {range:.2} over {}", show(&m)))
            },
        },
        Claim {
            id: "firms.ours.myopic",
            item: "firms-base",
            source: Source::Comment,
            citation: A99,
            text: "Ours: fn 19 says that under constant returns working together is never individually rational — at equilibrium; myopic agents who take others' effort as given still join (with b = 0 and every θ = 0.75, firms of two or more form in every seed)",
            check: |_| {
                let ws = worlds(FirmsConfig { b: 0.0, preferences: Preferences::Fixed, theta: 0.75, stop_at: 200, ..base() }, 10);
                let formed = ws.iter().filter(|w| w.stats.history().iter().any(|s| s.largest >= 2)).count();
                outcome(formed == 10, format!("{formed} of 10 seeds"))
            },
        },
        Claim {
            id: "firms.a13.zipf",
            item: "firms-2013",
            source: Source::Book,
            citation: "Axtell 2013, Endogenous Dynamics of Firms and Labor",
            text: "Axtell's 2013 parameterization (per-firm a, b and β; 2–6 friends; 4 % activated a period) gives Zipf-distributed firm sizes, α ≈ 1.06 — at 10 000 agents, the median µ (the paper's OLS) within 0.15 of 1.06 (3 seeds × 2 000 periods)",
            check: |_| {
                let c = FirmsConfig {
                    agents: 10_000,
                    a: 0.0,
                    a_max: 0.5,
                    b: 0.75,
                    b_max: 1.25,
                    beta: 1.5,
                    beta_max: 2.0,
                    neighbors: 2,
                    neighbors_max: 6,
                    activation_rate: 0.04,
                    stop_at: 2000,
                    ..base()
                };
                let ws = worlds(c, 3);
                let m = mus(&ws);
                let largest: Vec<f64> = ws.iter().map(|w| w.stats.history().iter().map(|s| f64::from(s.largest)).fold(0.0, f64::max)).collect();
                outcome((median(&m) - 1.06).abs() <= 0.15, format!("µ {} (median {:.2}); the largest firm {}", show(&m), median(&m), show(&largest)))
                    .with("The rule was written after the planning runs had measured µ 0.98–1.05.")
                    .with("Lifetimes are in periods, and a period is not the same unit: the firms-2013 preset's mean of 77 periods, at 4 % activated a period, is about 3.1 activations per agent, against the 1999 base case's 3.9 periods at one activation per agent a period on average (about 3.9).")
            },
        },
        Claim {
            id: "firms.ours.adjust-scope",
            item: "firms-sticky",
            source: Source::Comment,
            citation: A99,
            text: "Ours: whether sticky effort and groping apply outside the agent's own firm (A99 does not say) decides the outcome — applied in any firm, the whole population is in one firm at some point in at least 8 of 10 seeds; applied only at home, the largest firm stays below half the population in every seed",
            check: |_| {
                let reached = |c: FirmsConfig| worlds(c, 10).iter().map(|w| w.stats.history().iter().map(|s| s.largest).max().unwrap_or(0)).collect::<Vec<u32>>();
                let parts: Vec<(String, Outcome)> = [("sticky", FirmsConfig { effort_window: 0.1, ..base() }), ("groping", FirmsConfig { groping: true, ..base() })]
                    .into_iter()
                    .flat_map(|(name, c)| {
                        let any = reached(c.clone());
                        let home = reached(FirmsConfig { adjust_scope: AdjustScope::OwnFirm, ..c });
                        [
                            (format!("{name}, any firm"), outcome(any.iter().filter(|&&x| x == 1000).count() >= 8, format!("largest {any:?}"))),
                            (format!("{name}, own firm"), outcome(home.iter().all(|&x| x < 500), format!("largest {home:?}"))),
                        ]
                    })
                    .collect();
                all_of(parts).with("The rule was written after the planning runs.")
            },
        },
        Claim {
            id: "firms.ours.seniority-order",
            item: "firms-seniority",
            source: Source::Comment,
            citation: A99,
            text: "Ours: Table 11's modest effect (µ 1.11 at 5^−rank) comes from neither ordering of seniority shares — paying the longest-serving most (the text), no firm grows past 10 in any seed; paying the newest most, the median µ is below 0.5",
            check: |_| {
                let senior = worlds(FirmsConfig { pay: Pay::Seniority, seniority_base: 5.0, ..base() }, 10);
                let junior = mus_of(FirmsConfig { pay: Pay::Seniority, seniority_base: 5.0, seniority_order: SeniorityOrder::JuniorFirst, ..base() });
                let largest: Vec<u32> = senior.iter().map(|w| w.stats.history().iter().map(|s| s.largest).max().unwrap_or(0)).collect();
                all_of(vec![
                    ("senior first".into(), outcome(largest.iter().all(|&x| x <= 10), format!("largest {largest:?}"))),
                    ("newest first".into(), outcome(median(&junior) < 0.5, format!("µ {} (median {:.2})", show(&junior), median(&junior)))),
                ])
                .with("The rule was written after the planning runs.")
            },
        },
    ]
}
