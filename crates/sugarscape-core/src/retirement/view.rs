//! The frame: Axtell and Epstein's picture of the population (one row per
//! age, 20 at the top), retirement by age (their Fig. 6-1) on the same rows,
//! and the share of eligible agents retired over time.

use crate::render::{lerp, Rgb};

/// Pixels per age row, and every panel's height (81 ages).
pub const ROW: usize = 2;
pub const TALL: usize = 81 * ROW;
/// Cells between panels.
pub const GAP: usize = 8;
/// The retirement-by-age panel's width.
pub const AGES_W: usize = 101;
/// Periods the time panel shows, and its width.
pub const SHOWN: usize = 300;
pub const TIME_W: usize = SHOWN + 1;

pub const RATIONAL: Rgb = [0xff, 0x9e, 0xc4];
pub const IMITATOR: Rgb = [0x4a, 0x7c, 0xd8];
pub const RANDOM: Rgb = [0xf2, 0xc1, 0x4e];
pub const RETIRED: Rgb = [0xe0, 0x3c, 0x31];
pub const EMPTY: Rgb = [0xee, 0xec, 0xe6];
pub const MARK: Rgb = [0x5a, 0x55, 0x4c];
pub const BAR: Rgb = [0x8f, 0xb8, 0xde];
pub const LINE: Rgb = [0xe8, 0xe4, 0xda];
pub const LOW: Rgb = [0xff, 0x5a, 0x3c];
pub const HIGH: Rgb = [0x3c, 0x6e, 0xff];
pub const GROUP_A: Rgb = [0x6c, 0xd0, 0x7a];
pub const GROUP_B: Rgb = [0xd0, 0x6c, 0xe0];

/// The population panel's columns (agents shown per age) and cell width:
/// twice C (newborn cohorts run above C), 2 pixels a cell up to C 100, 1 above.
pub fn population(per_cohort: u32) -> (usize, usize) {
    let cols = (2 * per_cohort as usize).min(600);
    (cols, if per_cohort <= 100 { 2 } else { 1 })
}

/// The time panel's row of share `x`: 1 at the top.
pub fn row(x: f64) -> usize {
    ((1.0 - x.clamp(0.0, 1.0)) * (TALL - 1) as f64).round() as usize
}

pub fn scale(t: f64, from: Rgb, to: Rgb) -> Rgb {
    lerp(from, to, t.clamp(0.0, 1.0))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn panels_line_up() {
        assert_eq!(TALL, 162);
        assert_eq!((row(1.0), row(0.0)), (0, 161));
        assert_eq!(population(100), (200, 2));
        assert_eq!(population(500), (600, 1));
    }
}
