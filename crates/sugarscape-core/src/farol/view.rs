//! The frame: attendance × time (Arthur's Fig. 1, Challet and Zhang's
//! Fig. 1), the attendance histogram on the same vertical scale (their
//! Figs. 3–4), and a grid of the agents.

use crate::render::{lerp, Rgb};

/// Rounds the time panel shows.
pub const SHOWN: usize = 240;
/// The time panel's width and every panel's height.
pub const TIME_W: usize = SHOWN + 1;
pub const TALL: usize = 201;
/// Cells between panels.
pub const GAP: usize = 8;
/// The histogram's position and width (bars grow right).
pub const HIST_X: usize = TIME_W + GAP;
pub const HIST_W: usize = 101;
/// Where the agent grid starts.
pub const GRID_X: usize = HIST_X + HIST_W + GAP;

pub const LINE: Rgb = [0xe8, 0xe4, 0xda];
pub const CAPACITY: Rgb = [0x7a, 0x74, 0x68];
pub const CROWDED: Rgb = [0x3a, 0x22, 0x22];
pub const BAR: Rgb = [0x8f, 0xb8, 0xde];
pub const WENT: Rgb = [0xf2, 0xc1, 0x4e];
pub const STAYED: Rgb = [0x3a, 0x44, 0x5c];
pub const POOR: Rgb = [0x2a, 0x2e, 0x3a];
pub const RICH: Rgb = [0x7c, 0xe0, 0x8a];
pub const SWITCHED: Rgb = [0xff, 0x7a, 0x59];
pub const STEADY: Rgb = [0x44, 0x4a, 0x55];
pub const SHORT: Rgb = [0x4a, 0x9c, 0xff];
pub const LONG: Rgb = [0xff, 0xc8, 0x3c];
/// Predictor families: same as, mirror, mean, trend.
pub const FAMILIES: [Rgb; 4] = [
    [0x4a, 0x9c, 0xff],
    [0xd0, 0x6c, 0xe0],
    [0x6c, 0xd0, 0x7a],
    [0xff, 0x9a, 0x3c],
];

/// The row of attendance `a` out of `n`: n at the top, 0 at the bottom.
pub fn row(a: u32, n: u32) -> usize {
    ((f64::from(n - a.min(n)) / f64::from(n)) * (TALL - 1) as f64).round() as usize
}

/// The attendance a row stands for (its nearest value).
pub fn attendance_at(y: usize, n: u32) -> u32 {
    (f64::from(n) * (1.0 - y as f64 / (TALL - 1) as f64)).round() as u32
}

/// The agent grid's side (agents per row) and cells per agent.
pub fn grid(n: u32) -> (usize, usize) {
    let side = (f64::from(n).sqrt().ceil() as usize).max(1);
    (side, (TALL / side).max(1))
}

pub fn gain_color(gain: f64, top: f64) -> Rgb {
    lerp(
        POOR,
        RICH,
        if top > 0.0 {
            (gain / top).clamp(0.0, 1.0)
        } else {
            0.0
        },
    )
}

pub fn memory_color(m: u32) -> Rgb {
    lerp(SHORT, LONG, f64::from(m.saturating_sub(1)) / 15.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rows_put_full_attendance_at_the_top() {
        assert_eq!((row(100, 100), row(0, 100), row(60, 100)), (0, 200, 80));
        assert_eq!(attendance_at(80, 100), 60);
        assert_eq!(grid(100), (10, 20));
        assert_eq!(grid(1001), (32, 6));
        assert_eq!((HIST_X, GRID_X), (249, 358));
    }
}
