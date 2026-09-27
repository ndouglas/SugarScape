//! The frame: the share at the first source over time (Kirman's Figure II),
//! the time spent at each share against theory (his Figure I), and a grid of
//! the ants colored by source.

use crate::render::{lerp, Rgb};

/// Steps the time panel shows.
pub const SHOWN: usize = 400;
/// The time panel's width and every panel's height.
pub const TIME_W: usize = SHOWN + 1;
pub const TALL: usize = 201;
/// Cells between panels.
pub const GAP: usize = 8;
/// The histogram's position and width (bars grow right).
pub const HIST_X: usize = TIME_W + GAP;
pub const HIST_W: usize = 101;
/// Where the ant grid starts.
pub const GRID_X: usize = HIST_X + HIST_W + GAP;

pub const LINE: Rgb = [0xe8, 0xe4, 0xda];
pub const MARK: Rgb = [0x5a, 0x55, 0x4c];
pub const BAR: Rgb = [0x8f, 0xb8, 0xde];
pub const THEORY: Rgb = [0xff, 0x8a, 0x5c];
pub const HERDER: Rgb = [0x3a, 0x40, 0x4c];
pub const LONER: Rgb = [0xf2, 0xc1, 0x4e];
pub const FEW: Rgb = [0x2a, 0x2e, 0x3a];
pub const MANY: Rgb = [0x7c, 0xe0, 0x8a];
/// Kirman's black and white sources, then four more.
pub const SOURCES: [Rgb; 6] = [
    [0xf2, 0xc1, 0x4e],
    [0x4a, 0x9c, 0xff],
    [0x6c, 0xd0, 0x7a],
    [0xd0, 0x6c, 0xe0],
    [0xff, 0x7a, 0x59],
    [0x5c, 0xd6, 0xd6],
];

/// The row of share `x`: 1 at the top, 0 at the bottom.
pub fn row(x: f64) -> usize {
    ((1.0 - x.clamp(0.0, 1.0)) * (TALL - 1) as f64).round() as usize
}

/// The row of k ants out of n.
pub fn row_of(k: u32, n: u32) -> usize {
    row(f64::from(k) / f64::from(n))
}

/// The share a row stands for.
pub fn share_at(y: usize) -> f64 {
    1.0 - y as f64 / (TALL - 1) as f64
}

/// The ant grid's side (ants per row) and cells per ant.
pub fn grid(n: u32) -> (usize, usize) {
    let side = (f64::from(n).sqrt().ceil() as usize).max(1);
    (side, (TALL / side).max(1))
}

pub fn degree_color(d: usize, top: usize) -> Rgb {
    lerp(FEW, MANY, if top > 0 { d as f64 / top as f64 } else { 0.0 })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rows_put_the_whole_colony_at_the_top() {
        assert_eq!((row(1.0), row(0.0), row(0.8)), (0, 200, 40));
        assert_eq!(row_of(20, 100), 160);
        assert!((share_at(40) - 0.8).abs() < 1e-12);
        assert_eq!(grid(100), (10, 20));
        assert_eq!(grid(1000), (32, 6));
        assert_eq!((HIST_X, GRID_X), (409, 518));
    }
}
