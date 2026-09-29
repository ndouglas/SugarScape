//! The frame: a map of the subaks and dams, and below it a strip of each
//! dam's water over the year's months.

use crate::render::{lerp, Rgb};

/// Pixels per map unit, the map's margin (units), and its size.
pub const UNIT: f64 = 8.0;
pub const MARGIN: f64 = 3.0;
/// The data's extent: x −23…27, y −30…28.
pub const X0: f64 = -23.0;
pub const X1: f64 = 27.0;
pub const Y0: f64 = -30.0;
pub const Y1: f64 = 28.0;
pub const MAP_W: usize = ((X1 - X0 + 2.0 * MARGIN) * UNIT) as usize;
pub const MAP_H: usize = ((Y1 - Y0 + 2.0 * MARGIN) * UNIT) as usize;
/// The strip: 12 months × 12 dams.
pub const GAP: usize = 8;
pub const CELL_W: usize = 20;
pub const CELL_H: usize = 6;
pub const STRIP_H: usize = 12 * CELL_H;
pub const WIDE: usize = MAP_W;
pub const TALL: usize = MAP_H + GAP + STRIP_H;

pub const FALLOW: Rgb = [0x8a, 0x6a, 0x3a];
pub const SIX: Rgb = [0xb8, 0xd8, 0x5a];
pub const FOUR: Rgb = [0x7a, 0xc8, 0x4a];
pub const HYV: Rgb = [0x2a, 0x9a, 0x3a];
pub const VEG: Rgb = [0xf2, 0xc1, 0x4e];
pub const LOW: Rgb = [0x2a, 0x26, 0x20];
pub const HIGH: Rgb = [0x6a, 0xd8, 0xf6];
pub const DRY: Rgb = [0xe0, 0x3c, 0x31];
pub const WET: Rgb = [0x4a, 0x7c, 0xd8];
pub const RIVER: Rgb = [0x3a, 0x5a, 0x9a];
pub const LINK: Rgb = [0x4a, 0x46, 0x40];
pub const DAM: Rgb = [0xd8, 0xd4, 0xca];
pub const TEMPLES: [Rgb; 14] = [
    [0xe6, 0x19, 0x4b],
    [0x3c, 0xb4, 0x4b],
    [0xff, 0xe1, 0x19],
    [0x43, 0x63, 0xd8],
    [0xf5, 0x82, 0x31],
    [0x91, 0x1e, 0xb4],
    [0x46, 0xf0, 0xf0],
    [0xf0, 0x32, 0xe6],
    [0xbc, 0xf6, 0x0c],
    [0xfa, 0xbe, 0xbe],
    [0x00, 0x80, 0x80],
    [0xe6, 0xbe, 0xff],
    [0x9a, 0x63, 0x24],
    [0xff, 0xfa, 0xc8],
];

/// A map point's pixel.
pub fn at(x: f64, y: f64) -> (f64, f64) {
    ((x - X0 + MARGIN) * UNIT, (Y1 - y + MARGIN) * UNIT)
}

/// A subak's disc radius, pixels.
pub fn radius(area: f64) -> f64 {
    1.5 + area.sqrt() / 2.0
}

pub fn scale(t: f64, from: Rgb, to: Rgb) -> Rgb {
    lerp(from, to, t.clamp(0.0, 1.0))
}

/// A distinct color for each of the 252 plans and starts (display only).
pub fn option_color(plan: u8, start: u8) -> Rgb {
    let k = f64::from(plan) * 12.0 + f64::from(start);
    let h = (k * 0.618_033_988_75).fract() * 6.0;
    let (s, v) = (0.65, 0.92);
    let c = v * s;
    let x = c * (1.0 - ((h % 2.0) - 1.0).abs());
    let (r, g, b) = match h as u32 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };
    let m = v - c;
    [
        ((r + m) * 255.0) as u8,
        ((g + m) * 255.0) as u8,
        ((b + m) * 255.0) as u8,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_map_holds_the_data() {
        assert_eq!((WIDE, TALL), (448, 512 + 8 + 72));
        let (x, y) = at(X0, Y1);
        assert_eq!((x, y), (MARGIN * UNIT, MARGIN * UNIT));
        assert_ne!(option_color(0, 0), option_color(0, 1));
    }
}
