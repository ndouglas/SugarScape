//! Minds 5: episodic memory and the cycle finder — what happened where, and
//! when it'll happen again.
//!
//! - [`period`] finds the smallest repeat length of a sequence, brute force
//!   (these sequences are day counts, never large).
//! - [`extrapolate`] reads a value `k` steps past the end, by that period.
//! - [`Episodes`] is a bounded day-by-day log (`place`, `food`), capped at
//!   [`MEMORY_CAP`] like Minds 3's site memory; [`Episodes::predict`]
//!   extrapolates the place and food sequences separately, since their
//!   periods differ (3 and 2 in Amodio's schedule).
//!
//! Nothing here draws.

use std::collections::VecDeque;

use super::super::memory::MEMORY_CAP;

/// The smallest p in 1..`seq.len()` with `seq[i] == seq[i + p]` for every i
/// in `0..seq.len() - p`; `None` if no such p exists within the span. A
/// sequence shorter than 2 has no period (p must be < len, so len 0 or 1
/// never qualifies); a constant sequence of length ≥ 2 has period 1.
pub fn period<T: PartialEq>(seq: &[T]) -> Option<usize> {
    let len = seq.len();
    if len < 2 {
        return None;
    }
    (1..len).find(|&p| (0..len - p).all(|i| seq[i] == seq[i + p]))
}

/// The value `k` steps past the end of `seq` (1 = the first new value),
/// read off its period; `None` when `seq` has no period. Indexes the last
/// period-length block of `seq` at `(k - 1) mod period`, computed as
/// `(k + period - 1) mod period` so it never underflows even at `k == 0`
/// (0 steps past the end is just `seq`'s last value).
pub fn extrapolate<T: PartialEq + Clone>(seq: &[T], k: usize) -> Option<T> {
    let p = period(seq)?;
    let len = seq.len();
    let offset = (k + p - 1) % p;
    Some(seq[len - p + offset].clone())
}

/// One day: where the agent was and whether it found food there. `place`
/// is a site index in the field, and a lab compartment id (0, 1, 2 = K1,
/// K2, K3) in the lab.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Episode {
    pub place: u32,
    pub food: bool,
}

/// A bounded day-by-day log of episodes (what-where-when memory), oldest
/// first. [`push`](Episodes::push) keeps it at [`MEMORY_CAP`] or fewer,
/// dropping the oldest day once full — the same bound Minds 3 puts on site
/// memory.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Episodes {
    pub days: VecDeque<Episode>,
}

impl Episodes {
    /// Records today's episode, dropping the oldest day once past
    /// [`MEMORY_CAP`].
    pub fn push(&mut self, episode: Episode) {
        self.days.push_back(episode);
        if self.days.len() > MEMORY_CAP {
            self.days.pop_front();
        }
    }

    /// Where the agent will be and whether it'll find food, `k` days ahead
    /// (1 = tomorrow). Extrapolates the place sequence and the food
    /// sequence separately — their periods differ (3 and 2 in Amodio's
    /// schedule) — and returns `None` if either has no period. Because the
    /// two are extrapolated on separate periods, it can predict a place and
    /// food pair never observed together (the spec permits this: each
    /// sequence is a regularity of its own).
    pub fn predict(&self, k: usize) -> Option<Episode> {
        let places: Vec<u32> = self.days.iter().map(|e| e.place).collect();
        let foods: Vec<bool> = self.days.iter().map(|e| e.food).collect();
        let place = extrapolate(&places, k)?;
        let food = extrapolate(&foods, k)?;
        Some(Episode { place, food })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::{Rng, SeedableRng};

    #[test]
    fn empty_and_singleton_sequences_have_no_period() {
        assert_eq!(period::<u32>(&[]), None);
        assert_eq!(period(&[7]), None);
        assert_eq!(extrapolate::<u32>(&[], 1), None);
        assert_eq!(extrapolate(&[7], 3), None);
    }

    #[test]
    fn a_constant_sequence_of_length_two_or_more_has_period_one() {
        assert_eq!(period(&[5, 5]), Some(1));
        assert_eq!(period(&[5, 5, 5, 5, 5]), Some(1));
        assert_eq!(extrapolate(&[5, 5, 5], 1), Some(5));
        assert_eq!(extrapolate(&[5, 5, 5], 100), Some(5));
    }

    #[test]
    fn an_aperiodic_sequence_has_no_period() {
        // No p < len makes every offset pair equal.
        assert_eq!(period(&[1, 2, 3, 4]), None);
        assert_eq!(period(&[1, 2, 4, 8, 16, 32]), None);
        assert_eq!(extrapolate(&[1, 2, 3, 4], 1), None);
    }

    #[test]
    fn period_is_minimal_not_just_any_repeat() {
        // Period 4 also satisfies p = 8, but 4 is the smallest.
        let seq = [1, 2, 3, 4, 1, 2, 3, 4];
        assert_eq!(period(&seq), Some(4));
        // A sequence that happens to repeat once at the very end (p = len -
        // 1) still reports that, not a smaller false match.
        assert_eq!(period(&[1, 2, 3, 1]), Some(3));
    }

    // Raby's protocol: two compartments alternating for 6 days.
    #[test]
    fn rabys_alternating_places_have_period_two() {
        let places = [0u32, 1, 0, 1, 0, 1];
        assert_eq!(period(&places), Some(2));
        assert_eq!(extrapolate(&places, 1), Some(0), "day 7 continues the swap");
        assert_eq!(extrapolate(&places, 2), Some(1), "day 8");
    }

    // Amodio's Experiment 2: K1, K2, K3, K1, K2, K3, K1, K2, K3 on days
    // 1-9; Food-First puts food on the odd days.
    #[test]
    fn amodios_schedule_predicts_days_ten_through_twelve() {
        let mut episodes = Episodes::default();
        for day in 1u32..=9 {
            let place = (day - 1) % 3; // K1, K2, K3, K1, ...
            let food = day % 2 == 1; // Food-First: food on the odd days
            episodes.push(Episode { place, food });
        }
        assert_eq!(
            episodes.predict(1),
            Some(Episode {
                place: 0,
                food: false
            }),
            "day 10: K1, food absent"
        );
        assert_eq!(
            episodes.predict(2),
            Some(Episode {
                place: 1,
                food: true
            }),
            "day 11: K2, food present"
        );
        assert_eq!(
            episodes.predict(3),
            Some(Episode {
                place: 2,
                food: false
            }),
            "day 12: K3, food absent"
        );
    }

    // Amodio's Experiment 2, Empty-First: food on the even days.
    #[test]
    fn amodios_empty_first_schedule_predicts_the_opposite_food() {
        let mut episodes = Episodes::default();
        for day in 1u32..=9 {
            let place = (day - 1) % 3;
            let food = day % 2 == 0; // Empty-First: food on the even days
            episodes.push(Episode { place, food });
        }
        assert_eq!(
            episodes.predict(1),
            Some(Episode {
                place: 0,
                food: true
            }),
            "day 10: K1, food present"
        );
        assert_eq!(
            episodes.predict(2),
            Some(Episode {
                place: 1,
                food: false
            }),
            "day 11: K2, food absent"
        );
        assert_eq!(
            episodes.predict(3),
            Some(Episode {
                place: 2,
                food: true
            }),
            "day 12: K3, food present"
        );
    }

    #[test]
    fn predict_is_none_when_either_sequence_has_no_period() {
        let mut episodes = Episodes::default();
        for (place, food) in [(0u32, true), (1, false), (2, true), (3, false)] {
            episodes.push(Episode { place, food });
        }
        // The places 0,1,2,3 never repeat within the span.
        assert_eq!(episodes.predict(1), None);
    }

    #[test]
    fn push_drops_the_oldest_day_once_past_memory_cap() {
        let mut episodes = Episodes::default();
        for i in 0..MEMORY_CAP + 5 {
            episodes.push(Episode {
                place: i as u32,
                food: i.is_multiple_of(2),
            });
        }
        assert_eq!(episodes.days.len(), MEMORY_CAP);
        assert_eq!(
            episodes.days.front().unwrap().place,
            5,
            "the first 5 dropped"
        );
        assert_eq!(episodes.days.back().unwrap().place, (MEMORY_CAP + 4) as u32);
    }

    /// A naive period finder, written independently of [`period`]: the
    /// smallest p such that every index reads back the value at `i mod p`
    /// (repeating the first p values out to `len`), rather than comparing
    /// adjacent offsets p apart. Mathematically the same definition, worked
    /// out a different way, so it's a fair brute-force check.
    fn naive_period<T: PartialEq>(seq: &[T]) -> Option<usize> {
        let len = seq.len();
        if len < 2 {
            return None;
        }
        (1..len).find(|&p| (0..len).all(|i| seq[i] == seq[i % p]))
    }

    #[test]
    fn brute_force_matches_a_naive_definition_over_a_thousand_sequences() {
        let mut rng = rand_pcg::Pcg64Mcg::seed_from_u64(0xE915_0DE5);
        for _ in 0..1000 {
            let alphabet = rng.gen_range(1u8..=3);
            let len = rng.gen_range(1usize..=12);
            let seq: Vec<u8> = (0..len).map(|_| rng.gen_range(0..alphabet)).collect();
            assert_eq!(
                period(&seq),
                naive_period(&seq),
                "seq = {seq:?} (alphabet {alphabet})"
            );
        }
    }
}
