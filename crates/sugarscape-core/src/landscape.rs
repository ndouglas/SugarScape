//! Sites of the sugarscape and the built-in capacity maps.
//!
//! The two-peak map is a transcription of the book's Figure II-1, as
//! distributed with the NetLogo Sugarscape models.

use crate::config::{Map, Peak, MAX_GOODS, MAX_POLLUTANTS};

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

/// The book's two-peak map, row-major (row 0 = north).
fn two_peaks() -> Vec<f64> {
    TWO_PEAKS
        .split_whitespace()
        .map(|t| t.parse::<f64>().expect("sugar map holds integers"))
        .collect()
}

/// Row-major capacities of `map` on a width×height torus (row 0 = north).
/// `TwoPeaks` is only valid at 50×50 (enforced by `Config::validate`);
/// `Noise` depends only on its own parameters.
pub fn generate(map: &Map, width: u32, height: u32) -> Vec<f64> {
    let (w, h) = (width as usize, height as usize);
    match map {
        Map::TwoPeaks { transform } => {
            let base = two_peaks();
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
        Map::Noise {
            seed,
            scale,
            octaves,
            height,
        } => (0..w * h)
            .map(|i| {
                let at = ((i % w) as f64, (i / w) as f64);
                let v = noise_at(*seed, *scale, *octaves, (w, h), at);
                (*height * v).round().clamp(0.0, *height)
            })
            .collect(),
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

/// SplitMix64's output function: a fixed integer hash (no RNG state).
fn mix(z: u64) -> u64 {
    let mut z = z.wrapping_add(0x9E37_79B9_7F4A_7C15);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

/// The value in [0, 1) at lattice corner (i, j) of octave `octave`.
fn corner(seed: u32, octave: u32, i: u64, j: u64) -> f64 {
    let h = mix(mix(mix((u64::from(seed) << 32) | u64::from(octave)) ^ i) ^ j);
    (h >> 11) as f64 / (1u64 << 53) as f64
}

/// Lattice cells of octave `octave` along a side of `cells` grid cells:
/// max(1, round(cells · 2^octave / scale)) (Decision 10).
fn period(cells: usize, scale: f64, octave: u32) -> u64 {
    ((cells as f64 * f64::from(1u32 << octave) / scale).round() as u64).max(1)
}

fn smoothstep(t: f64) -> f64 {
    t * t * (3.0 - 2.0 * t)
}

/// Fractal value noise in [0, 1] at (x, y) on a w×h torus. Cell x samples
/// lattice coordinate x · period / w, and lattice indices wrap modulo the
/// period, so x = w is exactly x = 0 and the map tiles seamlessly. Only
/// integer hashing and + − × ÷ are used: identical on every platform.
fn noise_at(
    seed: u32,
    scale: f64,
    octaves: u32,
    (w, h): (usize, usize),
    (x, y): (f64, f64),
) -> f64 {
    let (mut sum, mut total, mut amplitude) = (0.0, 0.0, 1.0);
    for o in 0..octaves {
        let (px, py) = (period(w, scale, o), period(h, scale, o));
        let (u, v) = (x * px as f64 / w as f64, y * py as f64 / h as f64);
        let (fu, fv) = (u.floor(), v.floor());
        let (tx, ty) = (smoothstep(u - fu), smoothstep(v - fv));
        let (i0, j0) = (fu as u64 % px, fv as u64 % py);
        let (i1, j1) = ((i0 + 1) % px, (j0 + 1) % py);
        let c = |i, j| corner(seed, o, i, j);
        let top = c(i0, j0) + (c(i1, j0) - c(i0, j0)) * tx;
        let bottom = c(i0, j1) + (c(i1, j1) - c(i0, j1)) * tx;
        sum += amplitude * (top + (bottom - top) * ty);
        total += amplitude;
        amplitude *= 0.5;
    }
    sum / total
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Transform;

    fn two_peaks(transform: Transform) -> Vec<f64> {
        generate(&Map::TwoPeaks { transform }, 50, 50)
    }

    #[test]
    fn identity_is_the_books_map_and_mirror_x_is_todays_spice_map() {
        let base = two_peaks(Transform::Identity);
        // `super::` reaches the module's zero-argument `two_peaks`, which the
        // `two_peaks(transform)` helper above shadows.
        assert_eq!(base, super::two_peaks());
        let mirrored = two_peaks(Transform::MirrorX);
        for y in 0..50 {
            for x in 0..50 {
                assert_eq!(mirrored[y * 50 + x], base[y * 50 + 49 - x]);
            }
        }
        assert_eq!(base[5 * 50 + 37], 4.0, "northeast peak");
        assert_eq!(base[40 * 50 + 15], 4.0, "southwest peak");
        assert_eq!(base[0], 0.0, "northwest badlands");
        assert!(base.iter().all(|&c| (0.0..=4.0).contains(&c)));
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

    fn noise(seed: u32, scale: f64, octaves: u32, height: f64) -> Map {
        Map::Noise {
            seed,
            scale,
            octaves,
            height,
        }
    }

    #[test]
    fn noise_maps_are_in_range_deterministic_and_seeded() {
        let caps = generate(&noise(7, 8.0, 3, 4.0), 50, 40);
        assert_eq!(caps.len(), 2000);
        assert!(caps
            .iter()
            .all(|&c| (0.0..=4.0).contains(&c) && c.fract() == 0.0));
        assert!(caps.iter().any(|&c| c != caps[0]), "not flat");
        assert_eq!(
            caps,
            generate(&noise(7, 8.0, 3, 4.0), 50, 40),
            "no RNG state"
        );
        assert_ne!(
            caps,
            generate(&noise(8, 8.0, 3, 4.0), 50, 40),
            "the seed matters"
        );
        assert!(generate(&noise(7, 8.0, 3, 0.0), 50, 40)
            .iter()
            .all(|&c| c == 0.0));
        assert!(generate(&noise(7, 8.0, 3, 2.5), 50, 40)
            .iter()
            .all(|&c| (0.0..=2.5).contains(&c)));
        for (w, h) in [(5, 5), (50, 50), (37, 11)] {
            for octaves in 1..=6 {
                for scale in [1.0, 3.5, 100.0] {
                    for i in 0..w * h {
                        let at = ((i % w) as f64, (i / w) as f64);
                        let v = noise_at(3, scale, octaves, (w, h), at);
                        assert!((0.0..=1.0).contains(&v), "{v} at {at:?}");
                    }
                }
            }
        }
    }

    #[test]
    fn noise_tiles_the_torus_seamlessly() {
        let (w, h) = (50usize, 40usize);
        for (seed, scale, octaves) in [(1, 10.0, 1), (2, 8.0, 2), (3, 20.0, 3)] {
            let at =
                |x: usize, y: usize| noise_at(seed, scale, octaves, (w, h), (x as f64, y as f64));
            // The largest change between neighboring cells: smoothstep's slope
            // is at most 1.5 and corner values differ by less than 1, so octave
            // o changes by at most 1.5 · period / cells per cell.
            let bound = |cells: usize| {
                let (mut sum, mut total, mut amplitude) = (0.0, 0.0, 1.0);
                for o in 0..octaves {
                    sum += amplitude * 1.5 * period(cells, scale, o) as f64 / cells as f64;
                    total += amplitude;
                    amplitude *= 0.5;
                }
                sum / total + 1e-12
            };
            let (bx, by) = (bound(w), bound(h));
            for y in 0..h {
                assert_eq!(at(w, y), at(0, y), "x = W is x = 0");
                for x in 0..w {
                    let step = (at((x + 1) % w, y) - at(x, y)).abs();
                    assert!(
                        step <= bx,
                        "x {x} → {} at y {y}: {step} > {bx}",
                        (x + 1) % w
                    );
                }
            }
            for x in 0..w {
                assert_eq!(at(x, h), at(x, 0), "y = H is y = 0");
                for y in 0..h {
                    let step = (at(x, (y + 1) % h) - at(x, y)).abs();
                    assert!(
                        step <= by,
                        "y {y} → {} at x {x}: {step} > {by}",
                        (y + 1) % h
                    );
                }
            }
        }
    }

    #[test]
    fn noise_maps_ignore_the_world_seed_and_round_trip_as_json() {
        let mut c = crate::config::Config::default();
        c.goods[0].map = noise(5, 6.0, 2, 4.0);
        let a = crate::world::World::new(c.clone(), 1).unwrap();
        let b = crate::world::World::new(c, 2).unwrap();
        assert_eq!(a.capacities(0), b.capacities(0));
        assert!(!a.landscape_edited(0));
        let json = r#"{"kind":"noise","seed":7,"scale":8.0,"octaves":3,"height":4.0}"#;
        let map: Map = serde_json::from_str(json).unwrap();
        assert_eq!(map, noise(7, 8.0, 3, 4.0));
        assert_eq!(serde_json::to_string(&map).unwrap(), json);
    }
}
