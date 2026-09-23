//! Sites of the sugarscape and the built-in capacity maps.
//!
//! The two-peak map is a transcription of the book's Figure II-1, as
//! distributed with the NetLogo Sugarscape models.

use crate::config::{LandscapeKind, Map, Peak, MAX_GOODS, MAX_POLLUTANTS};

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

/// Row-major capacities of `map` on a width×height torus (row 0 = north).
/// `TwoPeaks` is only valid at 50×50 (enforced by `Config::validate`).
pub fn generate(map: &Map, width: u32, height: u32) -> Vec<f64> {
    let (w, h) = (width as usize, height as usize);
    match map {
        Map::TwoPeaks { transform } => {
            let base = capacities(&LandscapeKind::TwoPeaks, width, height);
            (0..w * h)
                .map(|i| {
                    let (x, y) = transform.source(i % w, i / w, w, h);
                    base[y * w + x]
                })
                .collect()
        }
        Map::Peaks { peaks } => (0..w * h)
            .map(|i| peak_capacity(peaks, i % w, i / w, w, h))
            .collect(),
        Map::Flat { capacity } => vec![*capacity; w * h],
    }
}

/// The highest ⌈height·(1 − d/radius)⌉ over `peaks` (0 where all are ≤ 0).
fn peak_capacity(peaks: &[Peak], x: usize, y: usize, w: usize, h: usize) -> f64 {
    let torus = |a: usize, b: u32, n: usize| {
        let d = (a as f64 - f64::from(b)).abs();
        d.min(n as f64 - d)
    };
    peaks
        .iter()
        .map(|p| {
            let (dx, dy) = (torus(x, p.x, w), torus(y, p.y, h));
            let v = (p.height * (1.0 - (dx * dx + dy * dy).sqrt() / p.radius)).ceil();
            if v > 0.0 {
                v
            } else {
                0.0
            }
        })
        .fold(0.0, f64::max)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{Map, Peak, Transform};

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

    fn two_peaks(transform: Transform) -> Vec<f64> {
        generate(&Map::TwoPeaks { transform }, 50, 50)
    }

    #[test]
    fn identity_is_the_books_map_and_mirror_x_is_todays_spice_map() {
        let base = two_peaks(Transform::Identity);
        assert_eq!(base, capacities(&LandscapeKind::TwoPeaks, 50, 50));
        let mirrored = two_peaks(Transform::MirrorX);
        assert_eq!(mirrored, spice_capacities(&LandscapeKind::TwoPeaks, 50, 50));
    }

    #[test]
    fn transforms_move_the_northeast_peak_as_documented() {
        let cases = [
            (Transform::Identity, (37, 5)),
            (Transform::Rotate90, (44, 37)),
            (Transform::Rotate180, (12, 44)),
            (Transform::Rotate270, (5, 12)),
            (Transform::MirrorX, (12, 5)),
            (Transform::MirrorY, (37, 44)),
            (Transform::Transpose, (5, 37)),
            (Transform::AntiTranspose, (44, 12)),
        ];
        let base = two_peaks(Transform::Identity);
        let total: f64 = base.iter().sum();
        for (t, (x, y)) in cases {
            let caps = two_peaks(t);
            assert_eq!(caps[y * 50 + x], 4.0, "{t:?}");
            assert_eq!(caps.iter().sum::<f64>(), total, "{t:?} is a permutation");
        }
        assert_eq!(Transform::ALL.len(), 8);
    }

    #[test]
    fn rotations_compose() {
        let r = |t: Transform, (x, y): (usize, usize)| t.source(x, y, 50, 50);
        for (x, y) in [(0, 0), (3, 17), (49, 0), (25, 49)] {
            let p = (x, y);
            let r90 = |p| r(Transform::Rotate90, p);
            assert_eq!(r90(r90(p)), r(Transform::Rotate180, p));
            assert_eq!(r90(r90(r90(p))), r(Transform::Rotate270, p));
            assert_eq!(r90(r90(r90(r90(p)))), p);
            assert_eq!(r(Transform::Transpose, r(Transform::Transpose, p)), p);
            assert_eq!(
                r(Transform::AntiTranspose, r(Transform::AntiTranspose, p)),
                p
            );
        }
    }

    #[test]
    fn peaks_fall_off_linearly_on_the_torus_and_take_the_highest() {
        let peak = |x, y, radius, height| Peak {
            x,
            y,
            radius,
            height,
        };
        let caps = generate(
            &Map::Peaks {
                peaks: vec![peak(10, 10, 5.0, 4.0)],
            },
            20,
            20,
        );
        let at = |x: usize, y: usize| caps[y * 20 + x];
        assert_eq!(at(10, 10), 4.0);
        assert_eq!(at(13, 10), 2.0, "⌈4 · (1 − 3/5)⌉ = ⌈1.6⌉");
        assert_eq!(at(10, 14), 1.0, "⌈4 · 0.2⌉");
        assert_eq!(at(15, 10), 0.0, "d = radius");
        assert_eq!(at(0, 0), 0.0);
        assert!(caps.iter().all(|&c| c.to_bits() != (-0.0f64).to_bits()));
        let wrap = generate(
            &Map::Peaks {
                peaks: vec![peak(0, 0, 5.0, 4.0)],
            },
            20,
            20,
        );
        assert_eq!(
            wrap[19], 4.0,
            "(19, 0) is 1 from (0, 0) across the edge: ⌈3.2⌉"
        );
        let two = generate(
            &Map::Peaks {
                peaks: vec![peak(2, 2, 3.0, 1.0), peak(3, 2, 2.0, 6.0)],
            },
            10,
            10,
        );
        assert_eq!(two[2 * 10 + 2], 3.0, "max(1, ⌈6 · (1 − 1/2)⌉)");
    }

    #[test]
    fn flat_maps_are_uniform_and_maps_round_trip_as_json() {
        assert_eq!(generate(&Map::Flat { capacity: 2.5 }, 4, 3), vec![2.5; 12]);
        let json = r#"{"kind":"two_peaks","transform":"rotate_90"}"#;
        let map: Map = serde_json::from_str(json).unwrap();
        assert_eq!(
            map,
            Map::TwoPeaks {
                transform: Transform::Rotate90
            }
        );
        assert_eq!(serde_json::to_string(&map).unwrap(), json);
        let peaks = r#"{"kind":"peaks","peaks":[{"x":1,"y":2,"radius":3.0,"height":4.0}]}"#;
        let map: Map = serde_json::from_str(peaks).unwrap();
        assert_eq!(serde_json::to_string(&map).unwrap(), peaks);
        for t in Transform::ALL {
            let s = serde_json::to_string(&t).unwrap();
            assert_eq!(serde_json::from_str::<Transform>(&s).unwrap(), t, "{s}");
        }
        assert_eq!(
            serde_json::to_string(&Transform::AntiTranspose).unwrap(),
            "\"anti_transpose\""
        );
    }
}
