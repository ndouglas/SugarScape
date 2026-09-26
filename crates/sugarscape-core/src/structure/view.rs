//! The Social Structure frame: the agents as a block of cells on the left
//! (the torus itself under 2DK, index order otherwise) and CRA's p–q plane
//! on the right (Figs. 2–4), with the population's recent trail.

use crate::render::{lerp, Rgb, BACKGROUND};

/// The p–q plane's side in cells (p = 0 … 1 in steps of 0.01).
pub const PLANE: usize = 101;
/// Cells between the block and the plane.
pub const GAP: usize = 6;
/// Periods of the population's mean (p, q) the plane keeps.
pub const TRAIL: usize = 200;

pub const LOW: Rgb = [0xd9, 0x48, 0x3b];
pub const HIGH: Rgb = [0x3d, 0xd6, 0x6b];
pub const TFT: Rgb = [0x3d, 0xd6, 0x6b];
pub const ALLD: Rgb = [0xd9, 0x48, 0x3b];
pub const ALLC: Rgb = [0x4f, 0x9d, 0xff];
pub const OTHER: Rgb = [0x8a, 0x86, 0x7a];
pub const PLANE_BG: Rgb = [0x22, 0x21, 0x1d];
pub const TRAIL_OLD: Rgb = [0x5a, 0x4a, 0x1a];
pub const TRAIL_NEW: Rgb = [0xff, 0xd8, 0x4d];
pub const DOT: Rgb = [0xb8, 0xb4, 0xa8];
pub const DOT_MANY: Rgb = [0xff, 0xff, 0xff];

/// Cells a side for n agents (⌈√n⌉).
pub fn block_side(n: usize) -> usize {
    let mut s = (n as f64).sqrt() as usize;
    while s * s < n {
        s += 1;
    }
    s
}

/// Cells per agent, so the block is about as tall as the plane.
pub fn cell(n: usize) -> usize {
    (PLANE / block_side(n)).max(1)
}

/// The frame: the block, a gap, the plane.
pub fn frame(n: usize) -> (usize, usize) {
    let block = block_side(n) * cell(n);
    (block + GAP + PLANE, block.max(PLANE))
}

/// Where the plane starts.
pub fn plane_x(n: usize) -> usize {
    block_side(n) * cell(n) + GAP
}

/// The plane cell of (p, q): p to the right, q up.
pub fn plane_cell(p: f64, q: f64) -> (usize, usize) {
    let at = |v: f64| (v.clamp(0.0, 1.0) * (PLANE - 1) as f64).round() as usize;
    (at(p), PLANE - 1 - at(q))
}

/// A strategy's class: the nearest of TFT (1,1,0), ALLD (0,0,0), ALLC
/// (1,1,1) if within 0.5, else other.
pub fn class(y: f64, p: f64, q: f64) -> &'static str {
    let d = |a: [f64; 3]| ((y - a[0]).powi(2) + (p - a[1]).powi(2) + (q - a[2]).powi(2)).sqrt();
    let near = [
        ("tft", d([1.0, 1.0, 0.0])),
        ("alld", d([0.0, 0.0, 0.0])),
        ("allc", d([1.0, 1.0, 1.0])),
    ]
    .into_iter()
    .min_by(|a, b| a.1.total_cmp(&b.1))
    .unwrap();
    if near.1 <= 0.5 {
        near.0
    } else {
        "other"
    }
}

pub fn class_color(name: &str) -> Rgb {
    match name {
        "tft" => TFT,
        "alld" => ALLD,
        "allc" => ALLC,
        _ => OTHER,
    }
}

/// A 0–1 value on the low–high scale.
pub fn scale(v: f64) -> Rgb {
    lerp(LOW, HIGH, v)
}

/// A frame being drawn.
pub struct Canvas<'a> {
    pub buf: &'a mut Vec<u8>,
    pub wide: usize,
}

impl Canvas<'_> {
    pub fn clear(&mut self, wide: usize, tall: usize) {
        self.wide = wide;
        self.buf.clear();
        self.buf.resize(wide * tall * 4, 0);
        for k in 0..wide * tall {
            self.buf[k * 4..k * 4 + 4].copy_from_slice(&[
                BACKGROUND[0],
                BACKGROUND[1],
                BACKGROUND[2],
                255,
            ]);
        }
    }

    pub fn put(&mut self, x: usize, y: usize, c: Rgb) {
        let k = (y * self.wide + x) * 4;
        self.buf[k..k + 4].copy_from_slice(&[c[0], c[1], c[2], 255]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_frame_fits_block_and_plane() {
        assert_eq!((block_side(256), cell(256)), (16, 6));
        assert_eq!(frame(256), (96 + GAP + PLANE, PLANE));
        assert_eq!((block_side(4096), cell(4096)), (64, 1));
        assert_eq!(block_side(250), 16);
    }

    #[test]
    fn the_plane_puts_q_up_and_p_right() {
        assert_eq!(plane_cell(0.0, 0.0), (0, PLANE - 1));
        assert_eq!(plane_cell(1.0, 1.0), (PLANE - 1, 0));
        assert_eq!(plane_cell(0.5, 0.25), (50, 75));
    }

    #[test]
    fn strategies_fall_into_classes() {
        assert_eq!(class(1.0, 0.95, 0.05), "tft");
        assert_eq!(class(0.1, 0.0, 0.1), "alld");
        assert_eq!(class(0.9, 1.0, 0.9), "allc");
        assert_eq!(class(0.5, 0.5, 0.5), "other");
    }
}
