//! The memory simplex (Axtell, Epstein & Young's Figs. 1–9): H at the top,
//! M at the lower left, L at the lower right; a memory plots at its mix of
//! the three, and the background shows the best reply there.

use super::config::Decision;
use super::stats::{View, H, L, M};

/// Cells across the triangle's base.
pub const SIDE: usize = 121;
/// Cells from the base to the top vertex (≈ SIDE·√3/2).
pub const TALL: usize = 105;
/// Cells between the two simplexes of the tag model.
pub const GAP: usize = 7;

/// Where a memory with counts `v` plots, in a simplex's own cells; the
/// center for an empty memory.
pub fn point(v: View) -> (usize, usize) {
    let total = f64::from(v[0] + v[1] + v[2]);
    let (pl, ph) = if total == 0.0 {
        (1.0 / 3.0, 1.0 / 3.0)
    } else {
        (f64::from(v[0]) / total, f64::from(v[2]) / total)
    };
    let x = pl * (SIDE - 1) as f64 + ph * ((SIDE - 1) / 2) as f64;
    let y = (1.0 - ph) * (TALL - 1) as f64;
    (x.round() as usize, y.round() as usize)
}

/// The mix (L, M, H) at a simplex cell, or `None` outside the triangle.
pub fn mix(x: usize, y: usize) -> Option<[f64; 3]> {
    if x >= SIDE || y >= TALL {
        return None;
    }
    let ph = 1.0 - y as f64 / (TALL - 1) as f64;
    let pl = (x as f64 - ph * ((SIDE - 1) / 2) as f64) / (SIDE - 1) as f64;
    let pm = 1.0 - ph - pl;
    // Half a cell across, plus the slack rounding y leaves: a memory's
    // rounded point stays inside.
    let eps = 0.5 / (SIDE - 1) as f64 + 0.5 / (TALL - 1) as f64;
    (pl >= -eps && pm >= -eps).then(|| [pl.max(0.0), pm.max(0.0), ph])
}

/// The best reply at a mix: under `Expected` the demand with the highest
/// expected payoff (ties to the most compromising: M, then L), under `Mode`
/// the reply to the commonest demand.
pub fn reply(p: [f64; 3], low: u32, decision: Decision) -> u8 {
    match decision {
        Decision::Expected => {
            let e = [
                f64::from(low),
                50.0 * (p[0] + p[1]),
                f64::from(100 - low) * p[0],
            ];
            if e[M as usize] >= e[L as usize] && e[M as usize] >= e[H as usize] {
                M
            } else if e[L as usize] >= e[H as usize] {
                L
            } else {
                H
            }
        }
        Decision::Mode => {
            if p[1] >= p[0] && p[1] >= p[2] {
                M
            } else if p[2] >= p[0] {
                L
            } else {
                H
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vertices_are_the_papers() {
        assert_eq!(point([0, 0, 5]), ((SIDE - 1) / 2, 0), "H at the top");
        assert_eq!(point([0, 5, 0]), (0, TALL - 1), "M at the lower left");
        assert_eq!(
            point([5, 0, 0]),
            (SIDE - 1, TALL - 1),
            "L at the lower right"
        );
        assert_eq!(mix(0, 0), None, "outside");
        let top = mix((SIDE - 1) / 2, 0).unwrap();
        assert!((top[2] - 1.0).abs() < 1e-9);
    }

    #[test]
    fn points_and_mixes_agree() {
        for v in [[3, 4, 3], [10, 0, 0], [1, 8, 1], [0, 2, 7]] {
            let (x, y) = point(v);
            let p = mix(x, y).unwrap();
            let t = f64::from(v[0] + v[1] + v[2]);
            for k in 0..3 {
                assert!((p[k] - f64::from(v[k]) / t).abs() < 0.02, "{v:?} {p:?}");
            }
        }
    }

    #[test]
    fn every_memory_plots_inside_the_triangle() {
        for total in 1..=100u32 {
            for l in 0..=total {
                for h in 0..=total - l {
                    let v = [l, total - l - h, h];
                    let (x, y) = point(v);
                    assert!(mix(x, y).is_some(), "{v:?} plots outside at ({x}, {y})");
                }
            }
        }
    }

    #[test]
    fn the_background_is_the_best_reply() {
        let at = |v: View| {
            let (x, y) = point(v);
            reply(mix(x, y).unwrap(), 30, Decision::Expected)
        };
        assert_eq!((at([0, 10, 0]), at([0, 0, 10]), at([10, 0, 0])), (M, L, H));
        assert_eq!(reply([0.2, 0.2, 0.6], 30, Decision::Mode), L);
    }
}
