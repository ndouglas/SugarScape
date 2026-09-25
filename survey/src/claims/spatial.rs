//! The spatial games (milestone 13): Nowak & May 1992, Huberman & Glance
//! 1993 and Nowak, Bonhoeffer & May 1994, each claim in its paper's words.

use sugarscape_core::model::{ModelConfig, ModelWorld};

use crate::claim::{range, Claim, Source};
use crate::runner::{model_after, model_preset};

const NM92: &str = "Nowak & May 1992, Nature 359";
const HG93: &str = "Huberman & Glance 1993, PNAS 90";
const NBM94: &str = "Nowak, Bonhoeffer & May 1994, PNAS 91";

fn series(w: &ModelWorld, name: &str) -> Vec<f64> {
    w.model()
        .series(name)
        .unwrap_or_else(|| panic!("no series {name}"))
}

fn last(w: &ModelWorld, name: &str) -> f64 {
    *series(w, name).last().expect("a recorded tick")
}

/// The mean of `fraction_c` from tick `from` on.
fn tail(w: &ModelWorld, from: usize) -> f64 {
    let s = series(w, "fraction_c");
    s[from..].iter().sum::<f64>() / (s.len() - from) as f64
}

/// Preset `id` with `edit` applied.
fn with(id: &str, edit: impl FnOnce(&mut sugarscape_core::spatial::SpatialConfig)) -> ModelConfig {
    let mut c = model_preset(id);
    if let ModelConfig::Spatial(s) = &mut c {
        edit(s);
    }
    c
}

pub fn claims() -> Vec<Claim> {
    use sugarscape_core::spatial::{Update, Winning};
    vec![
        Claim {
            id: "nm-2a.universal",
            item: "nm-2a-universal",
            source: Source::Book,
            citation: NM92,
            text: "in 2 > b > 1.8 f_C fluctuates around 0.318 (Fig. 2a: 12 log 2 − 8): the mean over t = 201–300 within 0.003 of 12 ln 2 − 8",
            check: |s| {
                let v = model_after(&model_preset("nm-2a-universal"), s, 300, |w| tail(w, 201));
                let t = 12.0 * std::f64::consts::LN_2 - 8.0;
                range(&v, t - 0.003, t + 0.003, false)
            },
        },
        Claim {
            id: "nm-1b.any-start",
            item: "nm-1b-chaos",
            source: Source::Book,
            citation: NM92,
            text: "f_C ≈ 0.318 'for almost all starting proportions': from 80% defectors the mean over t = 301–400 is 0.312–0.33",
            check: |s| {
                let c = with("nm-1b-chaos", |c| c.defectors = 0.8);
                range(&model_after(&c, s, 400, |w| tail(w, 301)), 0.312, 0.33, false)
            },
        },
        Claim {
            id: "nm-1a.static",
            item: "nm-1a-static",
            source: Source::Book,
            citation: NM92,
            text: "for 1.75 < b < 1.8 the equilibrium frequency of C is usually between 0.7 and 0.95",
            check: |s| {
                let v = model_after(&model_preset("nm-1a-static"), s, 200, |w| last(w, "fraction_c"));
                range(&v, 0.7, 0.95, false)
            },
        },
        Claim {
            id: "nm-no-self.fraction",
            item: "nm-no-self",
            source: Source::Book,
            citation: NM92,
            text: "without self-interaction the asymptotic C fraction is now ~0.299 (within 0.005)",
            check: |s| {
                let v = model_after(&model_preset("nm-no-self"), s, 1000, |w| tail(w, 501));
                range(&v, 0.294, 0.304, false)
            },
        },
        Claim {
            id: "nm-four-neighbors.fraction",
            item: "nm-four-neighbors",
            source: Source::Book,
            citation: NM92,
            text: "with only the four orthogonal neighbours f_C is around 0.374 (within 0.005)",
            check: |s| {
                let v = model_after(&model_preset("nm-four-neighbors"), s, 1000, |w| tail(w, 501));
                range(&v, 0.369, 0.379, false)
            },
        },
        Claim {
            id: "hg-async.defection",
            item: "hg-async-kaleidoscope",
            source: Source::Book,
            citation: HG93,
            text: "within a hundred generations or so the array evolves into a fixed state in which all players defect: all D by t = 200",
            check: |s| {
                let v = model_after(&model_preset("hg-async-kaleidoscope"), s, 200, |w| {
                    last(w, "fraction_c")
                });
                range(&v, 0.0, 0.0, false)
            },
        },
        Claim {
            id: "hg-async.always",
            item: "hg-async-kaleidoscope",
            source: Source::Book,
            citation: HG93,
            text: "as long as there is at least one defector in the initial state, the matrix always evolved rapidly into overall defection: all D at b = 1.7 by t = 300",
            check: |s| {
                let c = with("hg-async-kaleidoscope", |c| c.b = 1.7);
                range(&model_after(&c, s, 300, |w| last(w, "fraction_c")), 0.0, 0.0, false)
            },
        },
        Claim {
            id: "nbm-continuous.chaos-gone",
            item: "nbm-continuous",
            source: Source::Book,
            citation: NBM94,
            text: "for deterministic winning and b between 1.8 and 2 … all D for continuous time: all D at b = 1.9, t = 200",
            check: |s| {
                let c = with("nbm-continuous", |c| c.b = 1.9);
                range(&model_after(&c, s, 200, |w| last(w, "fraction_c")), 0.0, 0.0, false)
            },
        },
        Claim {
            id: "nbm-discrete.polymorphism",
            item: "nbm-discrete",
            source: Source::Book,
            citation: NBM94,
            text: "… we find polymorphism for discrete time: at b = 1.9 both C and D above 5% at t = 200",
            check: |s| {
                let c = with("nbm-discrete", |c| c.b = 1.9);
                range(&model_after(&c, s, 200, |w| last(w, "fraction_c")), 0.05, 0.95, false)
            },
        },
        Claim {
            id: "nbm-probabilistic.no-self",
            item: "nbm-probabilistic",
            source: Source::Book,
            citation: NBM94,
            text: "for m = 1 (proportional winning), C cannot persist in the absence of self-interaction: under 1% C at t = 200 for b = 1.05",
            check: |s| {
                let c = with("nbm-probabilistic", |c| {
                    c.self_weight = 0.0;
                    c.b = 1.05;
                    c.winning = Winning::Probabilistic;
                    c.m = 1.0;
                    c.update = Update::Synchronous;
                });
                range(&model_after(&c, s, 200, |w| last(w, "fraction_c")), 0.0, 0.01, false)
            },
        },
        Claim {
            id: "nbm-random-array.radius",
            item: "nbm-random-array",
            source: Source::Book,
            citation: NBM94,
            text: "if players interact with too many neighbors, the system became all D; for b = 1.6, r_c ~ 9: all D at r = 11 (from 50% defectors)",
            check: |s| {
                let c = with("nbm-random-array", |c| c.radius = 11.0);
                range(&model_after(&c, s, 300, |w| last(w, "fraction_c")), 0.0, 0.0, false)
            },
        },
        Claim {
            id: "nbm-random-array.ten-percent",
            item: "nbm-random-array",
            source: Source::App,
            citation: "spec 2026-09-25-spatial-games-design.md; plan Decision 12",
            text: "the radius threshold depends on the start: from NM92's 10% defectors, r = 11 keeps more than 30% C",
            check: |s| {
                let c = with("nbm-random-array", |c| {
                    c.radius = 11.0;
                    c.defectors = 0.1;
                });
                range(&model_after(&c, s, 300, |w| last(w, "fraction_c")), 0.3, 1.0, false)
            },
        },
        Claim {
            id: "nbm-cube.coexistence",
            item: "nbm-cube",
            source: Source::Book,
            citation: NBM94,
            text: "three-dimensional arrays … the results are similar to the two-dimensional ones: at b = 1.6 both C and D persist (f_C 0.1–0.9 over t = 101–200)",
            check: |s| {
                let v = model_after(&model_preset("nbm-cube"), s, 200, |w| tail(w, 101));
                range(&v, 0.1, 0.9, false)
            },
        },
    ]
}
