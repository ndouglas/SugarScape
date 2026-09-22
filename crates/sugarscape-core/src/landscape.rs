//! Sites of the sugarscape and the built-in capacity maps.
//!
//! The two-peak map is a transcription of the book's Figure II-1, as
//! distributed with the NetLogo Sugarscape models.

use crate::config::LandscapeKind;

const TWO_PEAKS: &str = include_str!("../assets/sugar-map.txt");

#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct Site {
    pub sugar: f64,
    pub capacity: f64,
    pub pollution: f64,
}

impl Site {
    /// A site whose sugar starts at capacity.
    pub fn full(capacity: f64) -> Self {
        Self {
            sugar: capacity,
            capacity,
            pollution: 0.0,
        }
    }
}

/// Row-major capacities (row 0 = north). `TwoPeaks` is only valid at 50×50
/// (enforced by `Config::validate`).
pub fn capacities(kind: &LandscapeKind, width: u32, height: u32) -> Vec<f64> {
    match *kind {
        LandscapeKind::TwoPeaks => TWO_PEAKS
            .split_whitespace()
            .map(|t| t.parse::<f64>().expect("sugar map holds integers"))
            .collect(),
        LandscapeKind::Flat { capacity } => vec![capacity; (width * height) as usize],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn two_peak_map_has_peaks_northeast_and_southwest() {
        let caps = capacities(&LandscapeKind::TwoPeaks, 50, 50);
        assert_eq!(caps.len(), 2500);
        let at = |x: usize, y: usize| caps[y * 50 + x];
        assert_eq!(at(37, 5), 4.0, "northeast peak");
        assert_eq!(at(15, 40), 4.0, "southwest peak");
        assert_eq!(at(0, 0), 0.0, "northwest badlands");
        assert!(caps.iter().all(|&c| (0.0..=4.0).contains(&c)));
    }

    #[test]
    fn flat_map_is_uniform() {
        let caps = capacities(&LandscapeKind::Flat { capacity: 2.5 }, 10, 12);
        assert_eq!(caps.len(), 120);
        assert!(caps.iter().all(|&c| c == 2.5));
    }
}
