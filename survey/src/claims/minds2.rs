//! Minds 2: walking instead of rule M's jump, and fences between patches
//! (docs/superpowers/specs/2026-09-27-minds-2-walking-design.md). Which of
//! the book's results need the jump, and does requiring travel to switch
//! patches reduce undermatching (Baum & Kraft 1998)? s is fitted per seed
//! exactly as in Minds 1 (`minds1::s_per_seed`).
//!
//! Where both arms run the same seeds (jump against walk, speed 10 against
//! speed 1, the far gap against no fence, the wall against the fence), the
//! verdict comes from the per-seed differences (`paired_greater`, or
//! `range` on the differences); the unpaired Mann–Whitney is in the detail.

use sugarscape_core::config::{Config, MoveMode, Placement, URange};
use sugarscape_core::world::World;

use crate::claim::{greater, range, Claim, Outcome, Source};
use crate::claims::ch2::{migrant_share, wealths};
use crate::claims::minds1::s_per_seed;
use crate::runner::{after, each_seed, preset, series, window_mean};
use crate::stats::{self, median};

const SPEC: &str = "docs/superpowers/specs/2026-09-27-minds-2-walking-design.md";

fn walk(c: &mut Config) {
    c.movement.mode = MoveMode::Walk;
}

/// Mean population over ticks 300–500.
fn capacity(c: &Config, seeds: &[u64]) -> Vec<f64> {
    after(c, seeds, 500, |w| {
        window_mean(&series(w, "population"), 300, 500)
    })
}

/// Claim: `a` exceeds `b` seed by seed. Holds when a − b > 0 in at least 80 %
/// of seeds (`range` on the paired differences); the unpaired one-sided
/// Mann–Whitney is reported in the detail.
pub(crate) fn paired_greater(a: &[f64], b: &[f64], a_name: &str, b_name: &str) -> Outcome {
    let diffs: Vec<f64> = a.iter().zip(b).map(|(x, y)| x - y).collect();
    let unpaired = greater(a, b, a_name, b_name);
    let mut outcome = range(&diffs, f64::MIN_POSITIVE, f64::INFINITY, false);
    // `range` prints the bounds, [0.0000, inf] here; say what they mean.
    if let Some(i) = outcome.measured.rfind(" in [") {
        outcome.measured.truncate(i);
        outcome.measured.push_str(" > 0");
    }
    outcome.with(&format!(
        "Paired differences ({a_name} − {b_name}), same seeds. Unpaired: {:?}: {}.",
        unpaired.verdict, unpaired.measured
    ))
}

/// Share of agents whose torus distance from the center of the starting
/// block is above 25 at tick 100.
fn far_share(c: &Config, seeds: &[u64]) -> Vec<f64> {
    let Placement::Block {
        x,
        y,
        width,
        height,
    } = c.placement
    else {
        unreachable!("ii-6-waves places a block")
    };
    let cx = f64::from(x) + f64::from(width - 1) / 2.0;
    let cy = f64::from(y) + f64::from(height - 1) / 2.0;
    let (gw, gh) = (f64::from(c.width), f64::from(c.height));
    after(c, seeds, 100, move |w: &World| {
        let pop = w.population() as f64;
        if pop == 0.0 {
            return f64::NAN;
        }
        let d = |a: f64, b: f64, n: f64| {
            let d = (a - b).abs();
            d.min(n - d)
        };
        let far = w
            .agents()
            .filter(|a| {
                let dx = d(f64::from(a.pos.x), cx, gw);
                let dy = d(f64::from(a.pos.y), cy, gh);
                (dx * dx + dy * dy).sqrt() > 25.0
            })
            .count();
        far as f64 / pop
    })
}

/// s per seed at vision 10–20 under walking, with `fences` (a preset's walls)
/// between the patches; the fences don't depend on the second patch's radius.
fn s_walk(seeds: &[u64], fences: &'static str) -> Vec<f64> {
    let walls = if fences.is_empty() {
        Vec::new()
    } else {
        preset(fences).walls
    };
    s_per_seed(seeds, move |c| {
        c.vision = URange::new(10, 20);
        walk(c);
        c.walls = walls.clone();
    })
}

/// Mean over ticks 500, 510, …, 1000 of N₁/N₂ (agents on the richer patch
/// over those on the other), skipping samples with an empty patch.
fn mean_ratio(w: &World) -> f64 {
    let (a, b) = (series(w, "on_first_patch"), series(w, "on_other_patches"));
    let v: Vec<f64> = (500..=1000)
        .step_by(10)
        .filter(|&t| a[t] > 0.0 && b[t] > 0.0)
        .map(|t| a[t] / b[t])
        .collect();
    if v.is_empty() {
        f64::NAN
    } else {
        stats::mean(&v)
    }
}

/// At the presets' own 2.10 : 1 (radius 7): the median over seeds of the
/// mean N₁/N₂ over ticks 500–1000, for each arm.
fn ratios_at_preset(seeds: &[u64]) -> String {
    let mut open = preset("ifd-far-sighted");
    walk(&mut open);
    let arms = [
        ("far fence", preset("ifd-fence-far")),
        ("near fence", preset("ifd-fence")),
        ("wall", preset("ifd-wall")),
        ("no fence walking", open),
        ("jump", preset("ifd-far-sighted")),
    ];
    let parts: Vec<String> = arms
        .iter()
        .map(|(name, c)| {
            let r = after(c, seeds, 1000, mean_ratio);
            format!("{name} {:.2}", median(&r))
        })
        .collect();
    format!(
        "At 2.10 : 1 (radius 7), median mean N₁/N₂ over ticks 500–1000: {}.",
        parts.join(", ")
    )
}

pub fn claims() -> Vec<Claim> {
    vec![
        Claim {
            id: "walk-capacity.book",
            item: "walk-capacity",
            source: Source::Book,
            citation:
                "Animation II-2: \"a carrying capacity of approximately 224 is eventually reached\"",
            text: "The population stabilizes at about 224 (Animation II-2), when agents walk",
            check: |seeds| {
                let jump = capacity(&preset("ii-2-unit"), seeds);
                range(
                    &capacity(&preset("walk-capacity"), seeds),
                    214.0,
                    234.0,
                    false,
                )
                .with(&format!(
                    "Mean population over ticks 300–500. Jump (ii-2-unit): median {:.1}.",
                    median(&jump)
                ))
            },
        },
        Claim {
            id: "walk-capacity.lower",
            item: "walk-capacity",
            source: Source::Comment,
            citation: SPEC,
            text: "Walking lowers the carrying capacity below the jump's",
            check: |seeds| {
                paired_greater(
                    &capacity(&preset("ii-2-unit"), seeds),
                    &capacity(&preset("walk-capacity"), seeds),
                    "jump",
                    "walk",
                )
            },
        },
        Claim {
            id: "walk-wealth.skewed",
            item: "walk-wealth",
            source: Source::Book,
            citation: "Animation II-5 (ii-5-wealth)",
            text: "Wealth is right-skewed (Animation II-5), when agents walk",
            check: |seeds| {
                let at = |id| {
                    after(&preset(id), seeds, 500, |w| {
                        (stats::skewness(&wealths(w)), w.stats.latest().unwrap().gini)
                    })
                };
                let (k, g): (Vec<f64>, Vec<f64>) = at("walk-wealth").into_iter().unzip();
                let (jk, jg): (Vec<f64>, Vec<f64>) = at("ii-5-wealth").into_iter().unzip();
                range(&k, 0.0, f64::INFINITY, false).with(&format!(
                    "Skewness of wealth at tick 500. Walk: Gini median {:.3}. Jump (ii-5-wealth): skewness median {:.3}, Gini median {:.3}.",
                    median(&g),
                    median(&jk),
                    median(&jg)
                ))
            },
        },
        Claim {
            id: "walk-seasons.migrate",
            item: "walk-seasons",
            source: Source::Book,
            citation: "Animation II-7 (ii-7-seasons)",
            text: "Some agents migrate with the seasons (Animation II-7), when agents walk",
            check: |seeds| {
                let share = |id| each_seed(&preset(id), seeds, |mut w| migrant_share(&mut w));
                let jump = share("ii-7-seasons");
                range(&share("walk-seasons"), 0.05, 1.0, false).with(&format!(
                    "Share of agents alive over ticks 100–300 that change hemisphere at least twice. Jump (ii-7-seasons): median {:.3}.",
                    median(&jump)
                ))
            },
        },
        Claim {
            id: "walk-waves.reach",
            item: "walk-waves",
            source: Source::Book,
            citation: "Animation II-6 (ii-6-waves)",
            text: "A wave reaches the far mountain (Animation II-6), when agents walk",
            check: |seeds| {
                let jump = far_share(&preset("ii-6-waves"), seeds);
                range(&far_share(&preset("walk-waves"), seeds), 0.25, 1.0, false).with(&format!(
                    "Share of agents farther than 25 (torus distance) from the starting block's center at tick 100. Jump (ii-6-waves): median {:.3}.",
                    median(&jump)
                ))
            },
        },
        Claim {
            id: "walk-speed.recovers",
            item: "walk-speed",
            source: Source::Comment,
            citation: SPEC,
            text: "The capacity under walk rises toward the jump's as speed rises",
            check: |seeds| {
                let at = |speed| {
                    let mut c = preset("walk-capacity");
                    c.movement.speed = speed;
                    capacity(&c, seeds)
                };
                let jump = capacity(&preset("ii-2-unit"), seeds);
                let ten = at(10);
                paired_greater(&ten, &at(1), "speed 10", "speed 1").with(&format!(
                    "Mean population over ticks 300–500. Speed 3 (walk-fast): median {:.1}. Speed 10: median {:.1}, against jump (ii-2-unit) median {:.1}.",
                    median(&at(3)),
                    median(&ten),
                    median(&jump)
                ))
            },
        },
        Claim {
            id: "ifd-fence-far.baum-kraft",
            item: "ifd-fence-far",
            source: Source::Book,
            citation: "Baum & Kraft 1998",
            text: "Requiring travel to switch patches reduces undermatching (Baum & Kraft 1998)",
            check: |seeds| {
                let far = s_walk(seeds, "ifd-fence-far");
                let open = s_walk(seeds, "");
                let jump = s_per_seed(seeds, |c| c.vision = URange::new(10, 20));
                paired_greater(&far, &open, "s far gap", "s no fence").with(&format!(
                    "s at vision 10–20 across the five input ratios; no fence is ifd-far-sighted switched to walk. Jump with no fence (ifd-far-sighted): median s {:.4}. {}",
                    median(&jump),
                    ratios_at_preset(seeds)
                ))
            },
        },
        Claim {
            id: "ifd-wall.visual",
            item: "ifd-wall",
            source: Source::Book,
            citation: "Baum & Kraft 1998",
            text: "A visual barrier has no effect (Baum & Kraft 1998), seed by seed",
            check: |seeds| {
                let wall = s_walk(seeds, "ifd-wall");
                let fence = s_walk(seeds, "ifd-fence");
                let diffs: Vec<f64> = wall.iter().zip(&fence).map(|(a, b)| a - b).collect();
                range(&diffs, -0.05, 0.05, false).with(&format!(
                    "Per-seed s (wall) − s (fence), gap at rows 20–21. Medians: wall {:.4}, fence {:.4}. {}",
                    median(&wall),
                    median(&fence),
                    ratios_at_preset(seeds)
                ))
            },
        },
    ]
}
