//! The papers' figures as presets.

use super::config::{Boundary, Lattice, Neighborhood, SpatialConfig, Start, Update, Winning};
use crate::model::ModelConfig;
use crate::presets::ModelPreset;

fn preset(
    id: &'static str,
    name: &'static str,
    source: &'static str,
    description: &'static str,
    edit: impl FnOnce(&mut SpatialConfig),
) -> ModelPreset {
    let mut c = SpatialConfig::default();
    edit(&mut c);
    ModelPreset {
        id,
        name,
        source,
        description,
        config: ModelConfig::Spatial(c),
    }
}

const NM92: &str = "Nowak & May 1992, Nature 359";
const HG93: &str = "Huberman & Glance 1993, PNAS 90";
const NBM94: &str = "Nowak, Bonhoeffer & May 1994, PNAS 91";

/// The kaleidoscope: one defector at the centre of a 99 × 99 lattice of
/// cooperators, fixed edges, b = 1.9 (NM92 Fig. 3).
fn kaleidoscope(c: &mut SpatialConfig) {
    c.width = 99;
    c.height = 99;
    c.start = Start::SingleDefector;
    c.b = 1.9;
}

/// NBM94's arena: 80 × 80, periodic, 50% defectors.
fn nbm_arena(c: &mut SpatialConfig) {
    c.width = 80;
    c.height = 80;
    c.boundary = Boundary::Periodic;
    c.defectors = 0.5;
}

pub fn presets() -> Vec<ModelPreset> {
    vec![
        preset(
            "nm-1a-static",
            "Fig. 1a — a static network",
            NM92,
            "NM92's Fig. 1a: a 200 × 200 lattice with fixed edges, 10% defectors at random, each player playing its eight neighbors and itself, b = 1.77 (the paper's 1.75 < b < 1.8). The paper: an irregular, mainly static network of D lines in a sea of C, with f_C \"usually between 0.7 and 0.95\". Measured (seeds 1–20): f_C 0.737–0.748 at t = 200, with 3% of sites still blinking each generation.",
            |c| c.b = 1.77,
        ),
        preset(
            "nm-1b-chaos",
            "Fig. 1b — spatial chaos",
            NM92,
            "NM92's Fig. 1b: as Fig. 1a with b = 1.9 (2 > b > 1.8): spatial chaos, C and D both persisting in shifting patterns, lots of yellow and green in the Change colors. Measured (seeds 1–20): f_C settles at 0.318–0.324 by t = 200–300 on this 200 × 200 lattice.",
            |_| {},
        ),
        preset(
            "nm-2a-universal",
            "Fig. 2a — 0.318 from any start",
            NM92,
            "NM92's Fig. 2a: 400 × 400, fixed edges, f_C(0) = 0.6, b = 1.9. The paper: f_C fluctuates around 12 log 2 − 8 ≈ 0.318 \"for almost all starting proportions and configurations\". Measured (seeds 1–20): 0.3179 ± 0.0005 over t = 201–300 — the constant reproduces to three decimals. The sweep nm-universal varies the start.",
            |c| {
                c.width = 400;
                c.height = 400;
                c.defectors = 0.4;
            },
        ),
        preset(
            "nm-3-kaleidoscope",
            "Fig. 3 — the evolutionary kaleidoscope",
            NM92,
            "NM92's Fig. 3, the evolutionary kaleidoscope: one defector at the center of a 99 × 99 lattice of cooperators, fixed edges, b = 1.9. Reproduced exactly: the pattern keeps its four-fold symmetry every generation and reaches the edges at t = 49, as the paper says; f_C then averages 0.315 over t = 100–221. Compare it with its asynchronous twin (Huberman and Glance) from the presets menu.",
            kaleidoscope,
        ),
        preset("nm-no-self", "No self-interaction", NM92, "NM92's variant without self-interaction (a = 0): the interesting region becomes 5/3 > b > 8/5; here b = 1.62, 200 × 200, 10% defectors. The paper: f_C ~0.299. Measured (seeds 1–20): 0.301–0.305 over t = 501–1000.", |c| {
            c.self_weight = 0.0;
            c.b = 1.62;
        }),
        preset("nm-four-neighbors", "Four neighbors", NM92, "NM92's variant with only the four orthogonal neighbors (and self): the interesting region is 2 > b > 5/3; here b = 1.8, 200 × 200, 10% defectors. The paper: f_C around 0.374. Measured (seeds 1–20): 0.379–0.382 over t = 501–1000, the same at every b in the window — close to, but not within 0.005 of, the paper's value.", |c| {
            c.neighborhood = Neighborhood::VonNeumann;
            c.b = 1.8;
        }),
        preset(
            "hg-async-kaleidoscope",
            "The kaleidoscope, asynchronous",
            HG93,
            "HG93's Fig. 1: NM92's kaleidoscope with asynchronous updating — each microstep one random player is rescored and replaced by its best-scoring neighbor, N microsteps a generation. The paper: \"within a hundred generations or so\" all players defect, and \"as long as there is at least one defector … always\". Measured (seeds 1–20): all D at t = 56–149 (mean 101) — reproduced; but HG93 never say which b they used, and below 1.8 the claim fails: at b = 1.7 the lone defector dies out (f_C 0.974–0.999). The sweep hg-async runs every b.",
            |c| {
                kaleidoscope(c);
                c.update = Update::Asynchronous;
            },
        ),
        preset(
            "nbm-probabilistic",
            "Proportional winning (m = 1)",
            NBM94,
            "NBM94's probabilistic winning (Eq. 1) at m = 1, \"proportional winning\": each site goes to C with probability Σ A (C) / Σ A over itself and its neighbors. 80 × 80, periodic edges, b = 1.35, 50% defectors (NBM94 do not state their start; an even start is what their m = 0 row shows). Measured (seeds 1–20): f_C 0.28–0.33 at t = 200. The sweeps nbm-grid-discrete and nbm-grid-continuous run the paper's whole b × m grid.",
            |c| {
                nbm_arena(c);
                c.winning = Winning::Probabilistic;
                c.m = 1.0;
                c.b = 1.35;
            },
        ),
        preset(
            "nbm-discrete",
            "Discrete time, b = 1.71",
            NBM94,
            "NBM94's arena in discrete time: 80 × 80, periodic, deterministic winning, b = 1.71, 50% defectors. Measured (seeds 1–20): f_C 0.86–0.92 at t = 200, mostly static. Its continuous-time twin is nbm-continuous (both open side by side from the presets menu).",
            |c| {
                nbm_arena(c);
                c.b = 1.71;
            },
        ),
        preset(
            "nbm-continuous",
            "Continuous time, b = 1.71",
            NBM94,
            "NBM94's continuous time (as HG93's asynchronous updating) in the same arena, b = 1.71. NBM94: coexistence survives continuous time across a broad band of b; only the 1.8 < b < 2 chaos disappears. Measured (seeds 1–20): f_C 0.71–0.73 at t = 200 — C and D coexist; at b = 1.9 the same setup is all D in 20 of 20 seeds while discrete time stays chaotic in 16 of 20.",
            |c| {
                nbm_arena(c);
                c.update = Update::Asynchronous;
                c.b = 1.71;
            },
        ),
        preset(
            "nbm-random-array",
            "A random array, r = 5",
            NBM94,
            "NBM94's irregular arrays: 5% of a 200 × 200 grid's cells hold players, each playing everyone within radius r = 5, b = 1.6, 50% defectors. NBM94: coexistence for intermediate b \"provided r was not too big\"; \"for b = 1.6, r_c ~ 9\". Measured (seeds 1–20, 50% defectors): all D in 0, 10 and 20 of 20 seeds at r = 5, 9 and 11 — reproduced; but from NM92's 10% start no radius up to 11 ends all D (f_C above 0.3), so r_c depends on the start the paper does not report. The sweep nbm-radius shows both.",
            |c| {
                c.lattice = Lattice::Random;
                c.occupancy = 0.05;
                c.radius = 5.0;
                c.b = 1.6;
                c.defectors = 0.5;
            },
        ),
        preset("nbm-cube", "A cube, 30 × 30 × 30", NBM94, "A 30 × 30 × 30 cube with periodic edges, each player playing its 26 neighbors and itself; the view shows one z-slice (the Slice menu). NBM94 say only that 3D results \"are similar to the two-dimensional ones\"; b = 1.6 and 10% defectors were chosen by measurement: the cube coexists for b from 1.1 to 1.8 and is all but all D at 1.9. Measured (seeds 1–20): f_C 0.331–0.342 over t = 101–200 with a quarter of the cube changing every generation — a 3D spatial chaos.", |c| {
            c.lattice = Lattice::Cube;
            c.width = 30;
            c.boundary = Boundary::Periodic;
            c.b = 1.6;
        }),
    ]
}
