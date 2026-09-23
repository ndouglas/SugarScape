//! Sites of the sugarscape and the built-in capacity maps.
//!
//! The two-peak map is a transcription of the book's Figure II-1, as
//! distributed with the NetLogo Sugarscape models.

use crate::config::{LandscapeKind, MAX_GOODS, MAX_POLLUTANTS};

const TWO_PEAKS: &str = include_str!("../assets/sugar-map.txt");

#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct Site {
    /// Level of each good; slots ≥ n stay 0.
    pub resource: [f64; MAX_GOODS],
    /// Capacity of each good; slots ≥ n stay 0.
    pub capacity: [f64; MAX_GOODS],
    /// Level of each pollutant; slots ≥ m stay 0.
    pub pollution: [f64; MAX_POLLUTANTS],
}

impl Site {
    /// A site whose goods start at the given capacities (in good order).
    pub fn full(capacities: &[f64]) -> Self {
        let mut site = Self::default();
        for (i, &c) in capacities.iter().enumerate() {
            site.resource[i] = c;
            site.capacity[i] = c;
        }
        site
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

/// Spice capacities: the two-peak sugar map mirrored left↔right, putting spice
/// mountains in the northwest and southeast (the book's Figure IV-1).
pub fn spice_capacities(kind: &LandscapeKind, width: u32, height: u32) -> Vec<f64> {
    let sugar = capacities(kind, width, height);
    match kind {
        LandscapeKind::TwoPeaks => {
            let (w, h) = (width as usize, height as usize);
            (0..w * h)
                .map(|i| sugar[(i / w) * w + (w - 1 - i % w)])
                .collect()
        }
        LandscapeKind::Flat { .. } => sugar,
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

    #[test]
    fn spice_map_mirrors_sugar_map_into_northwest_and_southeast() {
        let sugar = capacities(&LandscapeKind::TwoPeaks, 50, 50);
        let spice = spice_capacities(&LandscapeKind::TwoPeaks, 50, 50);
        let at = |v: &[f64], x: usize, y: usize| v[y * 50 + x];
        assert_eq!(at(&spice, 12, 5), 4.0, "northwest peak");
        assert_eq!(at(&spice, 34, 40), 4.0, "southeast peak");
        for y in 0..50 {
            for x in 0..50 {
                assert_eq!(at(&spice, x, y), at(&sugar, 49 - x, y));
            }
        }
        assert_eq!(
            spice_capacities(&LandscapeKind::Flat { capacity: 3.0 }, 5, 5),
            vec![3.0; 25]
        );
    }
}
