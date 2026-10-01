//! The frame: after A99's Animation 1 — each firm a row of cells, its
//! longest-serving member first, the largest firms first — and beside it
//! the firm-size distribution on log-log axes with its OLS line.

use crate::render::{lerp, Rgb};

/// Firm rows: cells of `CELL` pixels, up to `COLS` members and `ROWS` firms.
pub const CELL: usize = 3;
pub const COLS: usize = 200;
pub const ROWS: usize = 166;
pub const FIRMS_W: usize = COLS * CELL;
pub const FIRMS_H: usize = ROWS * CELL;
/// The size-distribution plot beside it.
pub const GAP: usize = 8;
pub const PLOT: usize = 200;
pub const PLOT_X: usize = FIRMS_W + GAP;
pub const WIDE: usize = FIRMS_W + GAP + PLOT;
pub const TALL: usize = FIRMS_H;
/// The plot's axes: ln size 0…ln 1000, ln frequency ln 10⁻⁷…0.
pub const LN_SIZE_MAX: f64 = 6.907_755_278_982_137;
pub const LN_FREQ_MIN: f64 = -16.118_095_650_958_32;

pub const FOUNDER: Rgb = [0xe0, 0x3c, 0x31];
pub const MEMBER: Rgb = [0x4a, 0x7c, 0xd8];
pub const LOW: Rgb = [0x2a, 0x26, 0x20];
pub const HIGH: Rgb = [0xf2, 0xc1, 0x4e];
pub const AXIS: Rgb = [0x5a, 0x56, 0x50];
pub const POINT: Rgb = [0xd8, 0xd4, 0xca];
pub const FIT: Rgb = [0xe0, 0x3c, 0x31];

pub fn scale(t: f64, from: Rgb, to: Rgb) -> Rgb {
    lerp(from, to, t.clamp(0.0, 1.0))
}

/// A plot point's pixel for ln size `x` and ln frequency `y` (None if off
/// the plot).
pub fn plot_at(x: f64, y: f64) -> Option<(usize, usize)> {
    let px = x / LN_SIZE_MAX;
    let py = 1.0 - (y - LN_FREQ_MIN) / -LN_FREQ_MIN;
    ((0.0..=1.0).contains(&px) && (0.0..=1.0).contains(&py)).then(|| {
        (
            PLOT_X + (px * (PLOT - 1) as f64) as usize,
            (py * (PLOT - 1) as f64) as usize,
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_frame_holds_the_rows_and_the_plot() {
        assert_eq!((WIDE, TALL), (600 + 8 + 200, 498));
        assert_eq!(plot_at(0.0, 0.0), Some((PLOT_X, 0)));
        assert_eq!(
            plot_at(LN_SIZE_MAX, LN_FREQ_MIN),
            Some((PLOT_X + PLOT - 1, PLOT - 1))
        );
        assert_eq!(plot_at(-0.1, 0.0), None);
    }
}
