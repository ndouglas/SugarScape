//! Variations on Schelling (milestone 32): Pancs & Vriend (2007), Gauvin,
//! Vannimenus & Nadal (2009), Singh, Vainchtein & Weiss (2009) and Zhang
//! (2004, JEBO), each under its authors' own rules and measured with its own
//! cluster count. Each rule takes its numbers from the paper. Pilot runs came
//! first for two measures, and the claims say how: Pancs & Vriend's clusters
//! (counted through uncontended blanks, as their §4.2.1 defines them) and
//! Zhang's cutoff of 600 (as mixed pairs, his ρ, it is the least possible on
//! his board, so it is read as his scaled potential, 0.075 ρ: 8,000 pairs).

use sugarscape_core::model::{ModelConfig, ModelWorld};
use sugarscape_core::schelling::{Neighborhood, SchellingConfig};

use crate::claim::{Claim, Outcome, Source, Verdict};
use crate::runner::{model_after, model_preset};

const PV: &str = "Pancs & Vriend 2007, J. Public Econ. 91";
const GVN: &str = "Gauvin, Vannimenus & Nadal 2009, Eur. Phys. J. B 70";
const SVW: &str = "Singh, Vainchtein & Weiss 2009, Demographic Research 21";
const ZHANG: &str = "Zhang 2004, J. Econ. Behav. Organ. 54";
/// 100,000 of Pancs & Vriend's periods: 5,000 steps of 20 turns.
const PV_TICKS: u32 = 5_000;
const GVN_TICKS: u32 = 1_000;
const SVW_TICKS: u32 = 300;

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

fn median(mut v: Vec<f64>) -> f64 {
    v.sort_by(f64::total_cmp);
    let n = v.len();
    if n % 2 == 1 {
        v[n / 2]
    } else {
        (v[n / 2 - 1] + v[n / 2]) / 2.0
    }
}

fn mean(v: &[f64]) -> f64 {
    v.iter().sum::<f64>() / v.len() as f64
}

fn board(id: &str, edit: impl Fn(&mut SchellingConfig)) -> ModelConfig {
    match model_preset(id) {
        ModelConfig::Schelling(mut c) => {
            edit(&mut c);
            ModelConfig::Schelling(c)
        }
        _ => panic!("{id} is not a Schelling preset"),
    }
}

/// `series`' last value after `ticks`, for each seed.
fn last(c: &ModelConfig, seeds: &[u64], ticks: u32, series: &str) -> Vec<f64> {
    model_after(c, seeds, ticks, |w: &ModelWorld| {
        w.model().latest_value(series).unwrap()
    })
}

/// The first step at which `series` is at most `x`, for each seed.
fn first_at_most(
    c: &ModelConfig,
    seeds: &[u64],
    ticks: u32,
    series: &str,
    x: f64,
) -> Vec<Option<usize>> {
    model_after(c, seeds, ticks, |w: &ModelWorld| {
        w.model()
            .series(series)
            .unwrap()
            .iter()
            .position(|&v| v <= x)
    })
}

fn tolerance(t: f64) -> impl Fn(&mut SchellingConfig) {
    move |c| {
        c.preference.min = 1.0 - t;
        c.preference.max = 1.0 - t;
    }
}

/// Gauvin et al. at tolerance `t` (5 % vacant): median s, and how many seeds
/// ended with nobody moving.
fn gvn(t: f64, seeds: &[u64]) -> (f64, usize) {
    let c = board("gvn-segregated", tolerance(t));
    let ends = model_after(&c, seeds, GVN_TICKS, |w: &ModelWorld| {
        let m = w.model();
        (
            m.latest_value("seg_s").unwrap(),
            m.latest_value("moves").unwrap(),
        )
    });
    let still = ends.iter().filter(|&&(_, moves)| moves == 0.0).count();
    (median(ends.into_iter().map(|(s, _)| s).collect()), still)
}

fn singh_vacancy(v: f64) -> impl Fn(&mut SchellingConfig) {
    move |c| c.population = (f64::from(c.width * c.height) * (1.0 - v)).round() as u32
}

fn zhang(beta: f64, neighborhood: Neighborhood, radius: u32) -> ModelConfig {
    board("zhang-random", move |c| {
        c.beta = beta;
        c.neighborhood = neighborhood;
        c.radius = radius;
    })
}

fn reached(v: &[Option<usize>]) -> (usize, f64) {
    let hit: Vec<f64> = v.iter().flatten().map(|&t| t as f64).collect();
    let n = hit.len();
    (n, if n == 0 { f64::NAN } else { median(hit) })
}

pub fn claims() -> Vec<Claim> {
    vec![
        Claim {
            id: "variations.pv.random",
            item: "pv-flat",
            source: Source::Book,
            citation: PV,
            text: "5 × 5, 10 of each: random allocations average 7.82 clusters (their Table 2). Holds if 1,000 random boards average within 5 % of 7.82 on their cluster count",
            check: |_| {
                let seeds: Vec<u64> = (1..=1_000).collect();
                let m = mean(&last(&model_preset("pv-flat"), &seeds, 0, "pv_clusters"));
                outcome((m - 7.82).abs() <= 0.05 * 7.82, format!("mean {m:.3} over 1,000 boards"))
            },
        },
        Claim {
            id: "variations.pv.flat",
            item: "pv-flat",
            source: Source::Book,
            citation: PV,
            text: "Flat (Schelling's) preferences, best responses, no inertia: after 100,000 periods 2.10 clusters on average, 91 % completely segregated. Holds if the mean is within 10 % of 2.10 and at least 80 % of seeds end in two clusters",
            check: |seeds| {
                let v = last(&model_preset("pv-flat"), seeds, PV_TICKS, "pv_clusters");
                let (m, two) = (mean(&v), v.iter().filter(|&&c| c == 2.0).count());
                outcome(
                    (m - 2.10).abs() <= 0.21 && two * 5 >= seeds.len() * 4,
                    format!("mean {m:.2}; {two} of {} in two clusters", seeds.len()),
                )
            },
        },
        Claim {
            id: "variations.pv.p50",
            item: "pv-p50",
            source: Source::Book,
            citation: PV,
            text: "Preferring a mix up to half (p50): 2.04 clusters, 98 % completely segregated. Holds if the mean is within 10 % of 2.04 and at least 85 % of seeds end in two clusters",
            check: |seeds| {
                let v = last(&model_preset("pv-p50"), seeds, PV_TICKS, "pv_clusters");
                let (m, two) = (mean(&v), v.iter().filter(|&&c| c == 2.0).count());
                outcome(
                    (m - 2.04).abs() <= 0.204 && two * 20 >= seeds.len() * 17,
                    format!("mean {m:.2}; {two} of {} in two clusters", seeds.len()),
                )
            },
        },
        Claim {
            id: "variations.pv.p100",
            item: "pv-p100",
            source: Source::Book,
            citation: PV,
            text: "Preferring half and half most (p100): 4.99 clusters after 100,000 periods (1,000 runs), and 854 of the 1,000 runs end in a strict equilibrium, where nobody wants to move. Holds if over 500 seeds (a spread this wide needs more than 20) the mean is within 10 % of 4.99 and between 75 % and 95 % of seeds end with nobody moving for their last 100 steps",
            check: |_| {
                let seeds: Vec<u64> = (1..=500).collect();
                let ends = model_after(&model_preset("pv-p100"), &seeds, PV_TICKS, |w: &ModelWorld| {
                    let m = w.model();
                    let moves = m.series("moves").unwrap();
                    let still = moves[moves.len() - 100..].iter().all(|&v| v == 0.0);
                    (m.latest_value("pv_clusters").unwrap(), still)
                });
                let m = mean(&ends.iter().map(|e| e.0).collect::<Vec<_>>());
                let still = ends.iter().filter(|e| e.1).count();
                outcome(
                    (m - 4.99).abs() <= 0.499 && (375..=475).contains(&still),
                    format!("mean {m:.2}; {still} of 500 still"),
                )
            },
        },
        Claim {
            id: "variations.pv.spiked",
            item: "pv-spiked",
            source: Source::Book,
            citation: PV,
            text: "Footnote 23: \"for the 2D setup the findings for the spiked utility function are very similar to the p100 function\" (not shown). Holds if over 500 seeds the mean clusters under spiked are within 10 % of those under p100",
            check: |_| {
                let seeds: Vec<u64> = (1..=500).collect();
                let at = |id| mean(&last(&model_preset(id), &seeds, PV_TICKS, "pv_clusters"));
                let (spiked, p100) = (at("pv-spiked"), at("pv-p100"));
                outcome(
                    (spiked - p100).abs() <= 0.1 * p100,
                    format!("mean {spiked:.2} spiked, {p100:.2} p100 (a random board: 7.8)"),
                )
            },
        },
        Claim {
            id: "variations.pv.ring",
            item: "pv-ring",
            source: Source::Book,
            citation: PV,
            text: "The ring, 10 + 10, four neighbors each side, even preferring half and half (p100): \"complete segregation\" in all 1,000 runs. Holds if every seed ends in two groups after 100,000 periods",
            check: |seeds| {
                let v = last(&model_preset("pv-ring"), seeds, PV_TICKS, "groups");
                let two = v.iter().filter(|&&g| g == 2.0).count();
                outcome(two == seeds.len(), format!("{two} of {} in two groups", seeds.len()))
            },
        },
        Claim {
            id: "variations.gvn.frozen",
            item: "gvn-frozen",
            source: Source::Book,
            citation: GVN,
            text: "Below T_f (1/2 at 2–4 % vacant, 2/5 to 1/2 at 6 %, their Table 1) the state is frozen: nobody can move. Holds if at 5 % vacant every seed ends with nobody moving at T = 0.35 and s under 0.1, and no seed at T = 0.5",
            check: |seeds| {
                let (s_low, still_low) = gvn(0.35, seeds);
                let (_, still_half) = gvn(0.5, seeds);
                outcome(
                    still_low == seeds.len() && s_low < 0.1 && still_half == 0,
                    format!("T 0.35: {still_low} of {} still, s {s_low:.3}; T 0.5: {still_half} still", seeds.len()),
                )
            },
        },
        Claim {
            id: "variations.gvn.segregated",
            item: "gvn-segregated",
            source: Source::Book,
            citation: GVN,
            text: "Between T_f and T_c, the segregated phase: s near 1 (two clusters). Holds if at 5 % vacant and T = 1/2 the median s is at least 0.9",
            check: |seeds| {
                let (s, _) = gvn(0.5, seeds);
                outcome(s >= 0.9, format!("median s {s:.3}"))
            },
        },
        Claim {
            id: "variations.gvn.three-quarters",
            item: "gvn-phase",
            source: Source::Book,
            citation: GVN,
            text: "\"the segregated phase occupies a large domain (up to a tolerance T as high as 3/4)\", mixed above, abruptly below 26 % vacant. Holds if at 5 % vacant the median s is at least 0.5 at T = 0.7 and under 0.2 at T = 0.8",
            check: |seeds| {
                let (s7, _) = gvn(0.7, seeds);
                let (s8, _) = gvn(0.8, seeds);
                outcome(s7 >= 0.5 && s8 < 0.2, format!("median s {s7:.3} at T 0.7, {s8:.3} at T 0.8"))
            },
        },
        Claim {
            id: "variations.svw.small-city",
            item: "svw-small",
            source: Source::Book,
            citation: SVW,
            text: "T = 3 on 8 × 8 (a third vacant): \"Most final states of the small city are segregated into two clusters\" (joined at sides or corners). Holds if more than half the seeds end in two clusters",
            check: |seeds| {
                let v = last(&model_preset("svw-small"), seeds, SVW_TICKS, "clusters8");
                let two = v.iter().filter(|&&c| c == 2.0).count();
                outcome(two * 2 > seeds.len(), format!("{two} of {} in two clusters", seeds.len()))
            },
        },
        Claim {
            id: "variations.svw.large-city",
            item: "svw-city",
            source: Source::Book,
            citation: SVW,
            text: "The same rule on 100 × 100: \"strictly a small city phenomenon\"; the number of clusters rises from 22 (24 % vacant) to 55 (33 %). Holds if the median clusters are within 20 % of 22 and of 55",
            check: |seeds| {
                let at = |v: f64| {
                    median(last(&board("svw-large", singh_vacancy(v)), seeds, SVW_TICKS, "clusters8"))
                };
                let (c24, c33) = (at(0.24), at(1.0 / 3.0));
                outcome(
                    (c24 - 22.0).abs() <= 4.4 && (c33 - 55.0).abs() <= 11.0,
                    format!("median {c24} clusters at 24 % vacant, {c33} at 33 %"),
                )
            },
        },
        Claim {
            id: "variations.zhang.checkerboard",
            item: "zhang-checkerboard",
            source: Source::Book,
            citation: ZHANG,
            text: "From a checkerboard (100 × 100, eight neighbors, β = 10), where everyone has the half-and-half mix they like best, segregation \"start[s] to emerge\" and \"once segregation emerges, it tends to persist\". Holds if the median mixed pairs fall below a quarter of the start's 20,000 by step 500 (5 million draws) and are no higher at step 1,000 than 10 % above step 500",
            check: |seeds| {
                let (half, full): (Vec<f64>, Vec<f64>) = model_after(
                    &model_preset("zhang-checkerboard"),
                    seeds,
                    1_000,
                    |w: &ModelWorld| {
                        let s = w.model().series("mixed_pairs").unwrap();
                        (s[500], s[1_000])
                    },
                )
                .into_iter()
                .unzip();
                let (h, f) = (median(half), median(full));
                outcome(h < 5_000.0 && f <= 1.1 * h, format!("median mixed pairs {h} at step 500, {f} at 1,000"))
            },
        },
        Claim {
            id: "variations.zhang.low-beta",
            item: "zhang-beta",
            source: Source::Book,
            citation: ZHANG,
            text: "\"for β < 2, our computer simulations never reach a state with a potential below 600\". Read literally, 600 mixed pairs is the least a 100 × 100 torus split in half can have (two straight bands, 300 pairs each), so \"below 600\" can never happen; read as his scaled potential, 0.075 ρ below 600, it is 8,000 mixed pairs. Holds if at β = 1.5 no seed falls to 8,000 in 1,000 steps (10 million draws)",
            check: |seeds| {
                let c = zhang(1.5, Neighborhood::Moore, 1);
                let (scaled, _) = reached(&first_at_most(&c, seeds, 1_000, "mixed_pairs", 8_000.0));
                outcome(scaled == 0, format!("{scaled} of {} reach 8,000", seeds.len()))
            },
        },
        Claim {
            id: "variations.zhang.waiting",
            item: "zhang-random",
            source: Source::Book,
            citation: ZHANG,
            text: "Fig. 8: with eight neighbors and β = 10 the expected wait for the potential to fall below 600 is about 40 million draws. Literally 600 mixed pairs is the least possible (see variations.zhang.low-beta), so read as his scaled potential (8,000 mixed pairs). Holds if the median seed first gets there between 20 and 80 million draws (2,000 to 8,000 steps)",
            check: |seeds| {
                let c = zhang(10.0, Neighborhood::Moore, 1);
                let runs = model_after(&c, seeds, 1_000, |w: &ModelWorld| {
                    let s = w.model().series("mixed_pairs").unwrap();
                    (s.iter().position(|&v| v <= 8_000.0), *s.last().unwrap())
                });
                let (n, t) = reached(&runs.iter().map(|r| r.0).collect::<Vec<_>>());
                let end = median(runs.iter().map(|r| r.1).collect());
                outcome(
                    n * 2 > seeds.len() && (2_000.0..=8_000.0).contains(&t),
                    format!(
                        "{n} of {} reach 8,000, median step {t} ({} draws); median {end} mixed pairs at step 1,000 (10 million draws), against the floor of 600",
                        seeds.len(),
                        t * 10_000.0
                    ),
                )
            },
        },
        Claim {
            id: "variations.zhang.neighborhood-order",
            item: "zhang-neighborhood",
            source: Source::Book,
            citation: ZHANG,
            text: "\"a bigger neighborhood actually decreases the waiting time for segregation\": twelve neighbors sort faster than eight, eight faster than four (β = 10, mixed pairs counted on the eight around). Holds if the median first step at or below 8,000 mixed pairs (his 600 as the scaled potential; 600 itself is the least possible, see variations.zhang.low-beta) falls strictly from four to eight to twelve",
            check: |seeds| {
                let wait = |n, r| reached(&first_at_most(&zhang(10.0, n, r), seeds, 60, "mixed_pairs", 8_000.0)).1;
                let (four, eight, twelve) = (
                    wait(Neighborhood::VonNeumann, 1),
                    wait(Neighborhood::Moore, 1),
                    wait(Neighborhood::VonNeumann, 2),
                );
                outcome(four > eight && eight > twelve, format!("median steps: {four} with four, {eight} with eight, {twelve} with twelve"))
            },
        },
        Claim {
            id: "variations.zhang.neighborhood-half",
            item: "zhang-neighborhood",
            source: Source::Book,
            citation: ZHANG,
            text: "\"Under the Moore neighborhood … the expected waiting time is less than half of that under the von Neumann neighborhood\". Holds if the median first step at or below 8,000 mixed pairs with eight neighbors is under half that with four",
            check: |seeds| {
                let wait = |n| reached(&first_at_most(&zhang(10.0, n, 1), seeds, 60, "mixed_pairs", 8_000.0)).1;
                let (four, eight) = (wait(Neighborhood::VonNeumann), wait(Neighborhood::Moore));
                outcome(eight < 0.5 * four, format!("median steps: {four} with four, {eight} with eight ({:.2})", eight / four))
            },
        },
    ]
}
