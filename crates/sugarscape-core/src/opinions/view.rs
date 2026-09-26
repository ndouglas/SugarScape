//! The opinion × time diagram (HK's Figs. 2, 4, 7, 8, 13 and 18): opinion
//! up, periods left to right, each agent a line colored red → magenta by its
//! starting opinion, gray where sorted neighbors are within each other's
//! reach; with a lattice, the torus to its right.

use crate::render::{Rgb, BACKGROUND};

/// Opinion profiles kept for the diagram: the current one and 60 before it.
pub const HISTORY: usize = 61;
/// Cells per period.
pub const STEP: usize = 4;
/// The diagram's width and height in cells.
pub const WIDE: usize = STEP * (HISTORY - 1) + 1;
pub const TALL: usize = 201;
/// Cells between the diagram and the lattice.
pub const GAP: usize = 8;
/// Where the lattice panel starts.
pub const LATTICE_X: usize = WIDE + GAP;

/// The gray between neighbors within each other's reach.
pub const BAND: Rgb = [0x3a, 0x38, 0x33];

/// The diagram's row for an opinion: 1 at the top, 0 at the bottom.
pub fn row(x: f64) -> usize {
    ((1.0 - x.clamp(0.0, 1.0)) * (TALL - 1) as f64).round() as usize
}

/// The opinion at a diagram row.
pub fn opinion_at(y: usize) -> f64 {
    1.0 - y as f64 / (TALL - 1) as f64
}

/// HK's coloring: red at 0 through the spectrum to magenta at 1.
pub fn hue(x: f64) -> Rgb {
    let h = x.clamp(0.0, 1.0) * 5.0; // six sextants of 60°; magenta ends the fifth
    let k = h.floor().min(4.0);
    let f = h - k;
    let (hi, lo) = (255.0, 40.0);
    let up = lo + (hi - lo) * f;
    let down = hi - (hi - lo) * f;
    let [r, g, b] = match k as u8 {
        0 => [hi, up, lo],
        1 => [down, hi, lo],
        2 => [lo, hi, up],
        3 => [lo, down, hi],
        _ => [up, lo, hi],
    };
    [r as u8, g as u8, b as u8]
}

/// Cells per lattice site, so the lattice fits the diagram's height.
pub fn site_cells(width: u32, height: u32) -> usize {
    (TALL / width.max(height) as usize).max(1)
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

    /// A vertical run of cells from row `a` to row `b`, either order.
    pub fn column(&mut self, x: usize, a: usize, b: usize, c: Rgb) {
        for y in a.min(b)..=a.max(b) {
            self.put(x, y, c);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rows_run_from_one_at_the_top_to_zero_at_the_bottom() {
        assert_eq!((row(1.0), row(0.0), row(0.5)), (0, TALL - 1, 100));
        assert!((opinion_at(row(0.37)) - 0.37).abs() <= 0.5 / (TALL - 1) as f64);
    }

    #[test]
    fn hues_go_from_red_to_magenta() {
        assert_eq!(hue(0.0), [255, 40, 40]);
        assert_eq!(hue(1.0), [255, 40, 255]);
        assert_eq!(hue(0.4), [40, 255, 40]);
    }

    #[test]
    fn the_lattice_fits_the_diagrams_height() {
        assert_eq!(site_cells(25, 25), 8);
        assert!(site_cells(44, 3) * 44 <= TALL);
        assert_eq!(WIDE, 241);
    }
}
