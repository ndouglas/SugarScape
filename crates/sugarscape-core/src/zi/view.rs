//! The frame: the market's schedules, the trade prices across periods (Gode
//! and Sunder's panels), and a strip of traders.

use crate::render::{lerp, Rgb};

/// The schedules panel's side, the prices panel's width, the gap, and the
/// traders strip's height.
pub const SCHED: usize = 200;
pub const PRICES_W: usize = 600;
pub const GAP: usize = 8;
pub const STRIP: usize = 60;
/// The frame.
pub const WIDE: usize = SCHED + GAP + PRICES_W;
pub const TALL: usize = SCHED + GAP + STRIP;
/// Periods the prices panel shows.
pub const SHOWN: usize = 10;

pub const BUYER: Rgb = [0x4a, 0x7c, 0xd8];
pub const SELLER: Rgb = [0xe0, 0x3c, 0x31];
pub const TRADE: Rgb = [0xf2, 0xc1, 0x4e];
pub const MARK: Rgb = [0xd8, 0xd4, 0xca];
pub const DIM: Rgb = [0x5a, 0x55, 0x4c];
pub const AHEAD: Rgb = [0x3c, 0xa8, 0x5a];
pub const BEHIND: Rgb = [0xf2, 0x9a, 0x3a];
pub const LOW: Rgb = [0x2a, 0x26, 0x20];
pub const HIGH: Rgb = [0x6a, 0xd8, 0xf6];

/// The row of `price` in a panel `tall` pixels high, `price_max` at the top.
pub fn row(price: f64, price_max: u32, tall: usize) -> usize {
    let t = (price / f64::from(price_max)).clamp(0.0, 1.0);
    ((1.0 - t) * (tall - 1) as f64).round() as usize
}

pub fn scale(t: f64, from: Rgb, to: Rgb) -> Rgb {
    lerp(from, to, t.clamp(0.0, 1.0))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn panels_line_up() {
        assert_eq!((WIDE, TALL), (808, 268));
        assert_eq!(
            (row(200.0, 200, SCHED), row(0.0, 200, SCHED)),
            (0, SCHED - 1)
        );
        assert_eq!(row(100.0, 200, 201), 100);
    }
}
