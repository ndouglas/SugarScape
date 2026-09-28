//! The frame: participation over time, Granovetter's Figure 1 (the
//! thresholds' c.d.f. against the 45° line, with the episode's staircase) or
//! the histogram of episode sizes, and a grid of the actors.

use crate::render::{lerp, Rgb};

/// Steps the time panel shows.
pub const SHOWN: usize = 400;
/// The time panel's width and every panel's height.
pub const TIME_W: usize = SHOWN + 1;
pub const TALL: usize = 201;
/// Cells between panels.
pub const GAP: usize = 8;
/// The middle panel's position and width.
pub const MID_X: usize = TIME_W + GAP;
pub const MID_W: usize = 101;
/// Where the actor grid starts.
pub const GRID_X: usize = MID_X + MID_W + GAP;

pub const LINE: Rgb = [0xe8, 0xe4, 0xda];
pub const MARK: Rgb = [0x5a, 0x55, 0x4c];
pub const BAR: Rgb = [0x8f, 0xb8, 0xde];
pub const PATH: Rgb = [0xff, 0x8a, 0x5c];
pub const ACTING: Rgb = [0xff, 0x6b, 0x4a];
pub const IDLE: Rgb = [0x3a, 0x40, 0x4c];
pub const SEED: Rgb = [0xf2, 0xc1, 0x4e];
pub const LOW: Rgb = [0xff, 0x5a, 0x3c];
pub const HIGH: Rgb = [0x3c, 0x6e, 0xff];
pub const FEW: Rgb = [0x2a, 0x2e, 0x3a];
pub const MANY: Rgb = [0x7c, 0xe0, 0x8a];
/// Crowds' colors, cycled.
pub const CROWDS: [Rgb; 8] = [
    [0xf2, 0xc1, 0x4e],
    [0x4a, 0x9c, 0xff],
    [0x6c, 0xd0, 0x7a],
    [0xd0, 0x6c, 0xe0],
    [0xff, 0x7a, 0x59],
    [0x5c, 0xd6, 0xd6],
    [0xc8, 0xc8, 0x70],
    [0xa0, 0x88, 0xff],
];

/// The row of share `x`: 1 at the top, 0 at the bottom.
pub fn row(x: f64) -> usize {
    ((1.0 - x.clamp(0.0, 1.0)) * (TALL - 1) as f64).round() as usize
}

/// The share a row stands for.
pub fn share_at(y: usize) -> f64 {
    1.0 - y as f64 / (TALL - 1) as f64
}

/// The middle panel's column of share `x` (Figure 1's horizontal axis).
pub fn col(x: f64) -> usize {
    (x.clamp(0.0, 1.0) * (MID_W - 1) as f64).round() as usize
}

/// The actor grid's side (actors per row) and cells per actor.
pub fn grid(n: u32) -> (usize, usize) {
    let side = (f64::from(n).sqrt().ceil() as usize).max(1);
    (side, (TALL / side).max(1))
}

pub fn scale(t: f64, from: Rgb, to: Rgb) -> Rgb {
    lerp(from, to, t.clamp(0.0, 1.0))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn panels_line_up() {
        assert_eq!((row(1.0), row(0.0), row(0.5)), (0, 200, 100));
        assert!((share_at(100) - 0.5).abs() < 1e-12);
        assert_eq!((col(0.0), col(1.0), col(0.25)), (0, 100, 25));
        assert_eq!(grid(100), (10, 20));
        assert_eq!(grid(10_000), (100, 2));
        assert_eq!((MID_X, GRID_X), (409, 518));
    }
}
