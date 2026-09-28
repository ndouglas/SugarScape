//! The frame: every group as a block of its agents in a mosaic, and below it
//! cooperation and punishment over time.

use crate::render::{lerp, Rgb};

/// Pixels between groups (a defeated group's frame is drawn in them).
pub const GAP: usize = 2;
/// The time strip's height, and the pixels above it.
pub const STRIP: usize = 60;
pub const STRIP_GAP: usize = 6;
/// The narrowest frame (the time strip's periods).
pub const LEAST_WIDE: usize = 301;

pub const CONTRIBUTOR: Rgb = [0x4a, 0x7c, 0xd8];
pub const PUNISHER: Rgb = [0x3c, 0xa8, 0x5a];
pub const DEFECTOR: Rgb = [0xe0, 0x3c, 0x31];
/// A punisher who defected this period.
pub const ERRED: Rgb = [0xf2, 0x9a, 0x3a];
pub const LOW: Rgb = [0x2a, 0x26, 0x20];
pub const HIGH: Rgb = [0xf6, 0xd8, 0x6a];
pub const LOST: Rgb = [0x1a, 0x18, 0x14];
pub const MARK: Rgb = [0xd8, 0xd4, 0xca];

/// The mosaic's shape for `groups` groups of `size`: groups per row, rows,
/// cells per group row and column, and each cell's side in pixels.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Mosaic {
    pub cols: usize,
    pub rows: usize,
    pub wide: usize,
    pub tall: usize,
    pub cell: usize,
}

impl Mosaic {
    pub fn new(groups: u32, size: u32) -> Self {
        let (g, n) = (groups as usize, size as usize);
        let cols = ceil_sqrt(2 * g);
        let rows = g.div_ceil(cols);
        let wide = ceil_sqrt(n);
        let tall = n.div_ceil(wide);
        let cell = (480 / (cols * wide)).clamp(1, 8);
        Mosaic {
            cols,
            rows,
            wide,
            tall,
            cell,
        }
    }

    /// A group's block, in pixels.
    pub fn block(&self) -> (usize, usize) {
        (self.wide * self.cell, self.tall * self.cell)
    }

    /// The top-left pixel of group `g`.
    pub fn origin(&self, g: usize) -> (usize, usize) {
        let (bw, bh) = self.block();
        (
            GAP + (g % self.cols) * (bw + GAP),
            GAP + (g / self.cols) * (bh + GAP),
        )
    }

    /// The mosaic's width and height.
    pub fn extent(&self) -> (usize, usize) {
        let (bw, bh) = self.block();
        (GAP + self.cols * (bw + GAP), GAP + self.rows * (bh + GAP))
    }
}

fn ceil_sqrt(v: usize) -> usize {
    let mut s = 1;
    while s * s < v {
        s += 1;
    }
    s
}

/// The time strip's row of share `x`: 1 at the top.
pub fn row(x: f64) -> usize {
    ((1.0 - x.clamp(0.0, 1.0)) * (STRIP - 1) as f64).round() as usize
}

/// An agent's color by its traits: red for defectors, blue for contributors,
/// green for punishers, blended for continuous traits.
pub fn traits_color(cooperate: f64, punish: f64) -> Rgb {
    lerp(
        DEFECTOR,
        lerp(CONTRIBUTOR, PUNISHER, punish.clamp(0.0, 1.0)),
        cooperate.clamp(0.0, 1.0),
    )
}

pub fn scale(t: f64, from: Rgb, to: Rgb) -> Rgb {
    lerp(from, to, t.clamp(0.0, 1.0))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_base_mosaic_is_16_groups_wide() {
        let m = Mosaic::new(128, 32);
        assert_eq!((m.cols, m.rows, m.wide, m.tall, m.cell), (16, 8, 6, 6, 5));
        assert_eq!(m.block(), (30, 30));
        assert_eq!(m.origin(17), (2 + 32, 2 + 32));
        assert_eq!(m.extent(), (2 + 16 * 32, 2 + 8 * 32));
        let big = Mosaic::new(128, 256);
        assert_eq!((big.wide, big.tall, big.cell), (16, 16, 1));
        let tiny = Mosaic::new(2, 2);
        assert_eq!(
            (tiny.cols, tiny.rows, tiny.wide, tiny.tall, tiny.cell),
            (2, 1, 2, 1, 8)
        );
    }

    #[test]
    fn colors_are_the_types_at_the_corners() {
        assert_eq!(traits_color(0.0, 0.0), DEFECTOR);
        assert_eq!(traits_color(0.0, 1.0), DEFECTOR);
        assert_eq!(traits_color(1.0, 0.0), CONTRIBUTOR);
        assert_eq!(traits_color(1.0, 1.0), PUNISHER);
        assert_eq!((row(1.0), row(0.0)), (0, STRIP - 1));
    }
}
