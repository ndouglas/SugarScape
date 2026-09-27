//! Minds 1: the ideal free distribution under rule M and the utility mind
//! (docs/superpowers/specs/2026-09-27-minds-1-utility-design.md). The
//! matching exponent s is fitted per seed across the five input ratios
//! from means of per-sample log ratios (Earn & Johnstone 1997), never from
//! ratios of mean counts.

use sugarscape_core::config::{Config, DecisionRule, Idle, Map, URange};
use sugarscape_core::geometry::{Pos, Torus};
use sugarscape_core::landscape::patch_of;
use sugarscape_core::world::World;

use crate::claim::{all_of, greater, range, Claim, Source};
use crate::runner::{each_seed, preset, series};
use crate::stats::median;

const SPEC: &str = "docs/superpowers/specs/2026-09-27-minds-1-utility-design.md";
const RADII: [f64; 5] = [10.0, 8.5, 7.0, 6.0, 5.0];

fn with_radius(mut c: Config, r: f64) -> Config {
    if let Map::Peaks { peaks } = &mut c.goods[0].map {
        peaks[1].radius = r;
    }
    c
}

/// Row-major patch of every site (`patch_of`).
fn patch_map(c: &Config) -> Vec<Option<usize>> {
    let Map::Peaks { peaks } = &c.goods[0].map else {
        unreachable!("ifd-* use peaks")
    };
    (0..c.height)
        .flat_map(|y| (0..c.width).map(move |x| (x, y)))
        .map(|(x, y)| patch_of(peaks, x, y, c.width, c.height))
        .collect()
}

/// Sites of patch `k` (capacity ≥ 1): the nominal input divided by the rate.
fn sites(c: &Config, k: usize) -> f64 {
    f64::from(patch_map(c).iter().filter(|&&p| p == Some(k)).count() as u32)
}

/// ln(R₁/R₂) at each of `RADII`.
fn ln_inputs() -> Vec<f64> {
    RADII
        .iter()
        .map(|&r| {
            let c = with_radius(preset("ifd-even"), r);
            (sites(&c, 0) / sites(&c, 1)).ln()
        })
        .collect()
}

/// Mean over ticks 500, 510, …, 1000 of ln(first / other), skipping samples
/// with an empty patch; NaN when every sample has one.
fn log_ratio(w: &World) -> f64 {
    let (a, b) = (series(w, "on_first_patch"), series(w, "on_other_patches"));
    let v: Vec<f64> = (500..=1000)
        .step_by(10)
        .filter(|&t| a[t] > 0.0 && b[t] > 0.0)
        .map(|t| (a[t] / b[t]).ln())
        .collect();
    if v.is_empty() {
        f64::NAN
    } else {
        v.iter().sum::<f64>() / v.len() as f64
    }
}

/// Least-squares slope of y on x over the finite pairs; NaN with fewer than 3.
fn slope(x: &[f64], y: &[f64]) -> f64 {
    let pts: Vec<(f64, f64)> = x
        .iter()
        .zip(y)
        .filter(|(_, b)| b.is_finite())
        .map(|(a, b)| (*a, *b))
        .collect();
    if pts.len() < 3 {
        return f64::NAN;
    }
    let n = pts.len() as f64;
    let mx = pts.iter().map(|p| p.0).sum::<f64>() / n;
    let my = pts.iter().map(|p| p.1).sum::<f64>() / n;
    let sxy: f64 = pts.iter().map(|p| (p.0 - mx) * (p.1 - my)).sum();
    let sxx: f64 = pts.iter().map(|p| (p.0 - mx).powi(2)).sum();
    sxy / sxx
}

/// Per seed: s across the five input ratios, starting from `ifd-even`
/// edited by `edit`.
fn s_per_seed(seeds: &[u64], edit: impl Fn(&mut Config)) -> Vec<f64> {
    let per_radius: Vec<Vec<f64>> = RADII
        .iter()
        .map(|&r| {
            let mut c = with_radius(preset("ifd-even"), r);
            edit(&mut c);
            each_seed(&c, seeds, |mut w| {
                w.run(1000);
                log_ratio(&w)
            })
        })
        .collect();
    let ln_r = ln_inputs();
    (0..seeds.len())
        .map(|i| slope(&ln_r, &per_radius.iter().map(|v| v[i]).collect::<Vec<_>>()))
        .collect()
}

/// Share of Flumps off both patches at tick 1000, per seed.
fn off_share(c: &Config, seeds: &[u64]) -> Vec<f64> {
    each_seed(c, seeds, |mut w| {
        w.run(1000);
        let off = *series(&w, "off_patch").last().unwrap();
        let pop = w.population() as f64;
        if pop == 0.0 {
            f64::NAN
        } else {
            off / pop
        }
    })
}

/// The catchment prediction of s at `vision`: the slope of ln(C₁/C₂) on
/// ln(R₁/R₂), C_k the expected number of sites from which a site of patch k
/// is in sight (or which is one), vision uniform on its range. A site that
/// sees both patches counts for both.
fn catchment_s(vision: URange) -> f64 {
    let mut y = Vec::new();
    for &r in &RADII {
        let c = with_radius(preset("ifd-even"), r);
        let patch = patch_map(&c);
        let at = |p: Pos| patch[(p.y * c.width + p.x) as usize];
        let torus = Torus::new(c.width, c.height);
        let catch = |k: usize| {
            let mut total = 0.0;
            for v in vision.min..=vision.max {
                let mut n = 0u32;
                for yy in 0..c.height {
                    for xx in 0..c.width {
                        let p = Pos::new(xx, yy);
                        let sees = at(p) == Some(k)
                            || torus.sight(p, v).iter().any(|&(q, _)| at(q) == Some(k));
                        n += u32::from(sees);
                    }
                }
                total += f64::from(n);
            }
            total / f64::from(vision.max - vision.min + 1)
        };
        y.push((catch(0) / catch(1)).ln());
    }
    slope(&ln_inputs(), &y)
}

fn vision(min: u32, max: u32) -> impl Fn(&mut Config) + Copy {
    move |c: &mut Config| c.vision = URange::new(min, max)
}

fn fed(c: &mut Config) {
    c.goods[0].endowment = URange::new(100_000, 100_000);
}

/// The utility mind at vision 10–20 (where a Flump can see both patches)
/// with crowding `m` and travel `k`.
fn utility_far(m: f64, k: f64) -> impl Fn(&mut Config) + Copy {
    move |c: &mut Config| {
        c.vision = URange::new(10, 20);
        c.decision.rule = DecisionRule::Utility;
        c.decision.crowding = m;
        c.decision.travel = k;
    }
}

pub fn claims() -> Vec<Claim> {
    vec![
        Claim {
            id: "ifd-matching.parker",
            item: "ifd-matching",
            source: Source::Book,
            citation: "Parker 1978 (via Collins, Houston & Lang 2002)",
            text: "Input matching: the ratio of animals at two sites matches the ratio of their input rates (s within 0.9–1.1), under rule M at vision 1–6 and at 10–20",
            check: |seeds| {
                all_of(
                    [(1, 6), (10, 20)]
                        .into_iter()
                        .map(|(a, b)| {
                            (format!("vision {a}–{b}"), range(&s_per_seed(seeds, vision(a, b)), 0.9, 1.1, false))
                        })
                        .collect(),
                )
            },
        },
        Claim {
            id: "ifd-matching.undermatching",
            item: "ifd-matching",
            source: Source::Book,
            citation: "Kennedy & Gray 1993",
            text: "Animals undermatch: fewer than input matching predicts are on the richer patch (s < 1), under rule M at vision 1–6, 5–10 and 10–20",
            check: |seeds| {
                all_of(
                    [(1, 6), (5, 10), (10, 20)]
                        .into_iter()
                        .map(|(a, b)| {
                            (format!("vision {a}–{b}"), range(&s_per_seed(seeds, vision(a, b)), 0.0, 0.999, false))
                        })
                        .collect(),
                )
            },
        },
        Claim {
            id: "ifd-matching.catchment",
            item: "ifd-matching",
            source: Source::Comment,
            citation: SPEC,
            text: "Catchment, not choice: under rule M at vision 1–6, s is within 0.1 of the catchment prediction in at least 80 % of seeds (the prediction is the slope of the log ratio of the patches' sight-catchments on the log input ratio)",
            check: |seeds| {
                let pred = catchment_s(URange::new(1, 6));
                let s = s_per_seed(seeds, |_| {});
                let m = median(&s);
                range(&s, pred - 0.1, pred + 0.1, false).with(&format!(
                    "Catchment prediction s = {pred:.4}; the seeds' median s = {m:.4} ({:+.4} from it).",
                    m - pred
                ))
            },
        },
        Claim {
            id: "ifd-no-starving.survival",
            item: "ifd-no-starving",
            source: Source::Comment,
            citation: SPEC,
            text: "Survival plays no part: under rule M at vision 1–6, s is the same seed by seed (within 0.05 in at least 80 % of seeds) with and without starvation",
            check: |seeds| {
                let starving = s_per_seed(seeds, |_| {});
                let no_starving = s_per_seed(seeds, fed);
                let diffs: Vec<f64> = no_starving.iter().zip(&starving).map(|(a, b)| a - b).collect();
                let same = diffs.iter().filter(|&&d| d == 0.0).count();
                range(&diffs, -0.05, 0.05, false).with(&format!(
                    "Per-seed differences in s (no starving − starving); identical in {same} of {} seeds. Medians: no starving {:.4}, starving {:.4}.",
                    diffs.len(),
                    median(&no_starving),
                    median(&starving)
                ))
            },
        },
        Claim {
            id: "ifd-no-starving.stuck",
            item: "ifd-no-starving",
            source: Source::Comment,
            citation: SPEC,
            text: "'Free' fails: under rule M a Flump who starts out of sight of sugar never moves; at R 2.10 and vision 1–6, over half of 100 are off both patches at tick 1000",
            check: |seeds| range(&off_share(&preset("ifd-no-starving"), seeds), 0.5, 1.0, false),
        },
        Claim {
            id: "ifd-wander.free",
            item: "ifd-wander",
            source: Source::Comment,
            citation: SPEC,
            text: "Wandering when nothing scores makes the Flumps free: under 5 % are off both patches at tick 1000",
            check: |seeds| range(&off_share(&preset("ifd-wander"), seeds), 0.0, 0.05, false),
        },
        Claim {
            id: "ifd-idle.toward-matching",
            item: "ifd-idle",
            source: Source::Comment,
            citation: SPEC,
            text: "Wandering moves s toward 1: with nobody starving at vision 1–6, s under the utility mind with idle wander exceeds s under rule M (idle stay)",
            check: |seeds| {
                let stay = s_per_seed(seeds, fed);
                let wander = s_per_seed(seeds, |c| {
                    fed(c);
                    c.decision.rule = DecisionRule::Utility;
                    c.decision.idle = Idle::Wander;
                });
                greater(&wander, &stay, "wander", "stay")
            },
        },
        Claim {
            id: "ifd-crowding.sutherland",
            item: "ifd-crowding",
            source: Source::Book,
            citation: "Sutherland 1983 (via Doncaster 1999)",
            text: "With interference m = 1, the distribution matches the inputs (s within 0.9–1.1); here under the utility mind at vision 10–20, with the interference local (Flumps on the site's neighbors), not patch-wide",
            check: |seeds| {
                let s0 = s_per_seed(seeds, utility_far(0.0, 0.0));
                let s1 = s_per_seed(seeds, utility_far(1.0, 0.0));
                let (a, b) = (median(&s0), median(&s1));
                let direction = if b < a { "falls" } else { "does not fall" };
                let rise = greater(&s1, &s0, "crowding 1", "crowding 0");
                range(&s1, 0.9, 1.1, false).with(&format!(
                    "s {direction} from crowding 0 (median {a:.4}) to crowding 1 (median {b:.4}). Does crowding raise s? {:?}: {}.",
                    rise.verdict, rise.measured
                ))
            },
        },
        Claim {
            id: "ifd-travel.baum-kraft",
            item: "ifd-travel",
            source: Source::Book,
            citation: "Baum & Kraft 1998",
            text: "'When travel was required to switch patches, undermatching decreased slightly': under the utility mind at vision 10–20, s with travel k = 0.5 exceeds s at k = 0 (here travel is a preference for nearby sugar under the one-tick jump, not a cost of switching)",
            check: |seeds| {
                greater(
                    &s_per_seed(seeds, utility_far(0.0, 0.5)),
                    &s_per_seed(seeds, utility_far(0.0, 0.0)),
                    "travel 0.5",
                    "travel 0",
                )
            },
        },
    ]
}
