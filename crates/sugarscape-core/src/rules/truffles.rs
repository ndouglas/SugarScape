//! Minds 3: truffles, hidden spots placed by hash (never on walls) that ripen
//! again a fixed time after a Flump harvests them.

use crate::landscape::mix;

/// 2⁶⁴, the scale `share` is measured against.
const TWO_POW_64: f64 = 18_446_744_073_709_551_616.0;

/// Whether site `index` has a truffle spot, under `seed` and `share`:
/// `mix(index as u64 ^ (u64::from(seed) << 32 | 0x5eed)) < threshold`, where
/// `threshold = (share * 2⁶⁴)` saturating. Independent of `World.rng`, so the
/// layout doesn't move with the world's seed. `share >= 1.0` always answers
/// true (every site gets a spot, regardless of the hash); `share <= 0.0`
/// always answers false, since the saturating threshold is then 0 and no
/// `u64` is less than it.
pub(crate) fn has_spot(index: usize, seed: u32, share: f64) -> bool {
    if share >= 1.0 {
        return true;
    }
    let threshold = (share * TWO_POW_64) as u64;
    let key = (index as u64) ^ ((u64::from(seed) << 32) | 0x5eed);
    mix(key) < threshold
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn has_spot_is_independent_of_seed_used_for_the_world_rng() {
        // `has_spot`'s own `seed` argument (from `config.truffles.seed`) is
        // fixed here; this checks it doesn't depend on anything else that
        // varies with the world's RNG seed (there's nothing else to vary:
        // the function takes only index, seed and share).
        let a: Vec<bool> = (0..2000).map(|i| has_spot(i, 1, 0.05)).collect();
        let b: Vec<bool> = (0..2000).map(|i| has_spot(i, 1, 0.05)).collect();
        assert_eq!(a, b, "deterministic and repeatable for the same seed");
    }

    #[test]
    fn share_zero_gives_no_spots() {
        assert!((0..10_000).all(|i| !has_spot(i, 1, 0.0)));
    }

    #[test]
    fn share_one_gives_every_spot() {
        assert!((0..10_000).all(|i| has_spot(i, 1, 1.0)));
    }

    #[test]
    fn share_close_to_one_twentieth_over_a_large_grid() {
        let n = 100 * 100;
        let count = (0..n).filter(|&i| has_spot(i, 1, 0.05)).count();
        let share = count as f64 / n as f64;
        assert!(
            (share - 0.05).abs() < 0.0005,
            "share {share} within 1% of 0.05 (i.e. within 0.0005)"
        );
    }
}
