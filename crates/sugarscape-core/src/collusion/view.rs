//! The frame: each firm's strategy as a map (its own last price across, the
//! next firm's down, the price it would charge as the color), the last
//! periods' prices with the Nash and monopoly prices marked, and — once the
//! session has finished — the response to a deviation.

use crate::render::{lerp, Rgb};

/// Periods the price panel shows.
pub const SHOWN: usize = 240;
/// The price and response panels' height.
pub const PANEL_H: usize = 121;
/// Cells between panels.
pub const GAP: usize = 8;
/// Pixels per period in the response panel.
pub const STEP: usize = 8;

pub const BACK: Rgb = [0x16, 0x18, 0x1e];
pub const COOL: Rgb = [0x3a, 0x6e, 0xd8];
pub const WARM: Rgb = [0xf0, 0x7a, 0x3a];
pub const NASH: Rgb = [0x5a, 0x9a, 0x6a];
pub const MONOPOLY: Rgb = [0xb0, 0x4a, 0x4a];
pub const MARK: Rgb = [0xf4, 0xf1, 0xea];
pub const AXIS: Rgb = [0x44, 0x48, 0x52];
/// Firms' colors in the price panels.
pub const FIRMS: [Rgb; 4] = [
    [0x8f, 0xc8, 0xff],
    [0xff, 0xc8, 0x6e],
    [0xb8, 0x8a, 0xe8],
    [0x7c, 0xe0, 0x8a],
];

/// Pixels per price in a strategy map: the map stays near 120 wide.
pub fn cell(prices: usize) -> usize {
    (120 / prices).max(2)
}

/// A strategy map's side in pixels.
pub fn side(prices: usize) -> usize {
    cell(prices) * prices
}

/// Price index `a` of `m` as a color: low prices cool, high prices warm.
pub fn price_color(a: u8, m: usize) -> Rgb {
    lerp(COOL, WARM, f64::from(a) / (m - 1).max(1) as f64)
}

/// How often a state was visited as a color (log scale against the most).
pub fn visit_color(visits: u32, most: u32) -> Rgb {
    if visits == 0 || most == 0 {
        return BACK;
    }
    let t = (f64::from(visits).ln_1p() / f64::from(most).ln_1p()).clamp(0.0, 1.0);
    lerp(AXIS, MARK, t)
}

/// The panel row of `price` between `lo` (bottom) and `hi` (top).
pub fn row(price: f64, lo: f64, hi: f64) -> usize {
    let t = if hi > lo {
        ((price - lo) / (hi - lo)).clamp(0.0, 1.0)
    } else {
        0.5
    };
    ((1.0 - t) * (PANEL_H - 1) as f64).round() as usize
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_and_rows_fit_their_panels() {
        assert_eq!((cell(15), side(15)), (8, 120));
        assert_eq!((cell(100), side(100)), (2, 200));
        assert_eq!(row(2.0, 1.0, 2.0), 0);
        assert_eq!(row(1.0, 1.0, 2.0), PANEL_H - 1);
        assert_eq!(price_color(0, 15), COOL);
        assert_eq!(price_color(14, 15), WARM);
        assert_eq!(visit_color(0, 10), BACK);
        assert_eq!(visit_color(10, 10), MARK);
    }
}
