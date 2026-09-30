//! The frame: one column per agent (20 at the default), in index order. From
//! the top: a status strip (dead, cheater, defending, raiding, fed, hungry),
//! two thin strips for its larder probability L and defense propensity D,
//! then its larder stock growing up from a midline and its scattered caches
//! growing down from it, one cell an item.

use crate::render::{lerp, Rgb};

/// The frame's size in cells; each agent's column is `WIDE / n` wide.
pub const WIDE: usize = 200;
pub const TALL: usize = 100;
/// Row bands: status, L, D, then the stores around the midline.
pub const STATUS_ROWS: usize = 8;
pub const L_ROW: usize = 9;
pub const D_ROW: usize = 11;
pub const MID: usize = 56;
/// Items drawn either side of the midline, at most.
pub const SHOWN: usize = 40;

pub const DEAD: Rgb = [0x3a, 0x36, 0x30];
pub const CHEATER: Rgb = [0xb0, 0x6a, 0xd8];
pub const DEFENDING: Rgb = [0x4a, 0x7c, 0xd8];
pub const RAIDING: Rgb = [0xe0, 0x3c, 0x31];
pub const FED: Rgb = [0x3c, 0xa8, 0x5a];
pub const HUNGRY: Rgb = [0xf2, 0x9a, 0x3a];
pub const LARDER: Rgb = [0xf2, 0xc1, 0x4e];
pub const SCATTER: Rgb = [0x7a, 0xb0, 0x5a];
pub const MIDLINE: Rgb = [0x5a, 0x55, 0x4c];
pub const LOW: Rgb = [0x2a, 0x26, 0x20];
pub const HIGH: Rgb = [0x6a, 0xd8, 0xf6];

/// The agent a frame column belongs to, of `n`.
pub fn agent_at(x: usize, n: usize) -> usize {
    (x * n / WIDE).min(n.saturating_sub(1))
}

/// The columns `[from, to)` of agent `i` of `n`, less a one-cell gutter.
pub fn columns(i: usize, n: usize) -> (usize, usize) {
    let from = i * WIDE / n;
    let to = ((i + 1) * WIDE / n).saturating_sub(1).max(from + 1);
    (from, to.min(WIDE))
}

pub fn scale(t: f64, from: Rgb, to: Rgb) -> Rgb {
    lerp(from, to, t.clamp(0.0, 1.0))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn columns_tile_the_frame() {
        assert_eq!((WIDE, TALL), (200, 100));
        assert_eq!(columns(0, 20), (0, 9));
        assert_eq!(columns(19, 20), (190, 199));
        for x in 0..WIDE {
            let i = agent_at(x, 20);
            assert_eq!(i, x / 10);
        }
        assert_eq!(agent_at(199, 7), 6);
    }
}
