//! The Norms and Metanorms frame: Axelrod's boldness–vengefulness plane
//! (his Figs. 2 and 4; G&I's Figs. 3–10) on the left, with G&I's regions and
//! the population's recent trail, and a strip of the agents on the right.

use crate::render::{Rgb, BACKGROUND};

/// Cells per strategy level on the plane (8 levels a side).
pub const LEVEL: usize = 12;
/// The plane's side in cells.
pub const PLANE: usize = 8 * LEVEL;
/// Cells between the plane and the strip.
pub const GAP: usize = 6;
/// Where the strip starts, its width, and each agent's row height.
pub const STRIP_X: usize = PLANE + GAP;
pub const STRIP: usize = 64;
pub const ROW: usize = 4;
/// Generations of the mean (B, V) the plane keeps.
pub const TRAIL: usize = 200;

pub const PLANE_BG: Rgb = [0x22, 0x21, 0x1d];
pub const GRID: Rgb = [0x2c, 0x2b, 0x26];
pub const DENSE: Rgb = [0xff, 0xf6, 0xe0];
pub const SPARSE: Rgb = [0x6e, 0x6a, 0x60];
pub const ESTABLISHED: Rgb = [0x3d, 0xd6, 0x6b];
pub const COLLAPSED: Rgb = [0xd9, 0x48, 0x3b];
pub const TRAIL_OLD: Rgb = [0x5a, 0x4a, 0x1a];
pub const TRAIL_NEW: Rgb = [0xff, 0xd8, 0x4d];
pub const BOLD_BAR: Rgb = [0xd9, 0x48, 0x3b];
pub const VENGE_BAR: Rgb = [0x4f, 0x9d, 0xff];
pub const STRONG: Rgb = [0xff, 0x3d, 0x8b];
pub const WEAK: Rgb = [0x4f, 0x9d, 0xff];

/// The frame for `n` agents (a 4-cell gap between the groups under groups).
pub fn frame(n: usize, groups: bool) -> (usize, usize) {
    let rows = n * ROW + if groups { ROW } else { 0 };
    (STRIP_X + STRIP, rows.max(PLANE))
}

/// The top-left cell of the plane cell for boldness `b` and vengefulness `v`
/// (levels 0–7): boldness right, vengefulness up.
pub fn level_cell(b: u8, v: u8) -> (usize, usize) {
    (usize::from(b) * LEVEL, (7 - usize::from(v)) * LEVEL)
}

/// The levels at a plane pixel, or `None` off the plane.
pub fn level_at(x: usize, y: usize) -> Option<(u8, u8)> {
    (x < PLANE && y < PLANE).then(|| ((x / LEVEL) as u8, (7 - y / LEVEL) as u8))
}

/// The pixel for a mean (B, V) in [0, 1]: the centre of that position.
pub fn mean_pixel(b: f64, v: f64) -> (usize, usize) {
    let at = |t: f64| (t.clamp(0.0, 1.0) * 7.0 * LEVEL as f64 + LEVEL as f64 / 2.0) as usize;
    (
        at(b).min(PLANE - 1),
        (PLANE - 1).saturating_sub(at(v)).min(PLANE - 1),
    )
}

/// Agent `i`'s strip row top, with a gap after the strong group.
pub fn row_top(i: usize, strong: Option<usize>) -> usize {
    match strong {
        Some(s) if i >= s => (i + 1) * ROW,
        _ => i * ROW,
    }
}

/// The agent at a strip row pixel, if any.
pub fn agent_at_row(y: usize, n: usize, strong: Option<usize>) -> Option<usize> {
    let r = y / ROW;
    let i = match strong {
        Some(s) if r == s => return None,
        Some(s) if r > s => r - 1,
        _ => r,
    };
    (i < n).then_some(i)
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

    pub fn rect(&mut self, x: usize, y: usize, w: usize, h: usize, c: Rgb) {
        for dy in 0..h {
            for dx in 0..w {
                self.put(x + dx, y + dy, c);
            }
        }
    }

    /// A one-cell outline of a rectangle.
    pub fn outline(&mut self, x: usize, y: usize, w: usize, h: usize, c: Rgb) {
        for dx in 0..w {
            self.put(x + dx, y, c);
            self.put(x + dx, y + h - 1, c);
        }
        for dy in 0..h {
            self.put(x, y + dy, c);
            self.put(x + w - 1, y + dy, c);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_plane_puts_boldness_right_and_vengefulness_up() {
        assert_eq!(level_cell(0, 7), (0, 0));
        assert_eq!(level_cell(7, 0), (7 * LEVEL, 7 * LEVEL));
        assert_eq!(level_at(13, 1), Some((1, 7)));
        assert_eq!(level_at(PLANE, 0), None);
        assert_eq!(mean_pixel(0.0, 0.0), (LEVEL / 2, PLANE - 1 - LEVEL / 2));
    }

    #[test]
    fn strip_rows_skip_the_gap_between_groups() {
        assert_eq!((row_top(19, Some(20)), row_top(20, Some(20))), (76, 84));
        assert_eq!(agent_at_row(80, 30, Some(20)), None, "the gap");
        assert_eq!(agent_at_row(84, 30, Some(20)), Some(20));
        assert_eq!(agent_at_row(84, 20, None), None);
        assert_eq!(frame(20, false), (STRIP_X + STRIP, PLANE));
        assert_eq!(frame(30, true), (STRIP_X + STRIP, 124));
    }
}
