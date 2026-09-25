//! A household (§1.1: five persons, one farm plot, one residence) and the
//! documented rules that concern only its own corn: storage aging and
//! consumption (ODD p.4–5, §3.5), the expected harvest (§3.7) and the
//! fission endowment (§3.10).

/// One household. Cells are frame indices (`valley::WIDTH`-wide rows from
/// the north).
#[derive(Clone, Debug, PartialEq)]
pub struct Household {
    pub id: u64,
    /// Its farm plot.
    pub farm: u32,
    /// Its residence (settlement) cell.
    pub home: u32,
    /// Age in years.
    pub age: u32,
    /// Corn by harvest age, newest first: `S0, S-1, …` (storage years + 1
    /// slots).
    pub corn: Vec<f64>,
    /// This (or the last) year's harvest `H0`.
    pub harvest: f64,
    /// The food it expects next year, `E[H]`.
    pub expected: f64,
}

impl Household {
    /// All the corn it holds.
    pub fn stock(&self) -> f64 {
        self.corn.iter().sum()
    }
}

/// Ages the store and adds this year's harvest (§3.5 step 1): the oldest
/// slot's leftovers spoil, every other slot moves one year older, and
/// `harvest` becomes `S0`.
pub fn store_harvest(corn: &mut [f64], harvest: f64) {
    corn.rotate_right(1);
    corn[0] = harvest;
}

/// Eats `need` kg, oldest corn first, exactly as ODD p.4 writes it (§3.5
/// step 2): a slot holding at least the need left is drawn down by it,
/// otherwise the need left falls by the slot and the slot empties. Returns
/// the need left (`NNR`), which removes the household when above 0. A
/// negative slot (a negative harvest, A-14) therefore raises the need left.
pub fn consume(corn: &mut [f64], need: f64) -> f64 {
    let mut left = need;
    for slot in corn.iter_mut().rev() {
        if *slot >= left {
            *slot -= left;
            left = 0.0;
        } else {
            left -= *slot;
            *slot = 0.0;
        }
    }
    left
}

/// `E[H]`: the corn left that will not spoil before next year's meals
/// (every slot but the oldest) plus this year's harvest, the forecast of
/// next year's (§3.7; ODD p.5 "stored corn left plus the expected harvest").
pub fn expected_harvest(corn: &[f64], harvest: f64) -> f64 {
    corn[..corn.len() - 1].iter().sum::<f64>() + harvest
}

/// The ODD's endowment (§3.10): the child takes `fraction` of each of the
/// parent's slots, which the parent loses, so corn is conserved.
pub fn split_endowment(parent: &mut [f64], fraction: f64) -> Vec<f64> {
    parent
        .iter_mut()
        .map(|slot| {
            let given = *slot * fraction;
            *slot -= given;
            given
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn storage_ages_and_the_oldest_leftovers_spoil() {
        let mut corn = vec![300.0, 200.0, 100.0];
        store_harvest(&mut corn, 900.0);
        assert_eq!(
            corn,
            [900.0, 300.0, 200.0],
            "the 100 kg left in S-2 spoiled"
        );
    }

    #[test]
    fn households_eat_the_oldest_corn_first() {
        let mut corn = vec![900.0, 300.0, 200.0];
        assert_eq!(consume(&mut corn, 800.0), 0.0);
        assert_eq!(corn, [600.0, 0.0, 0.0]);
        let mut short = vec![100.0, 50.0, 50.0];
        assert_eq!(
            consume(&mut short, 800.0),
            600.0,
            "NNR left: the household is removed"
        );
        assert_eq!(short, [0.0, 0.0, 0.0]);
    }

    #[test]
    fn a_negative_harvest_is_kept_and_adds_to_the_unmet_need() {
        // A-14 (adopted): harvests are not clamped at 0, and the ODD's
        // algorithm turns a negative S0 into unmet need even when older
        // corn covered the meal.
        let mut corn = vec![0.0, 900.0, 0.0];
        store_harvest(&mut corn, -50.0);
        assert_eq!(corn, [-50.0, 0.0, 900.0]);
        assert_eq!(consume(&mut corn, 800.0), 50.0);
        assert_eq!(corn, [0.0, 0.0, 100.0]);
        assert_eq!(expected_harvest(&[-50.0, 0.0, 100.0], -50.0), -100.0);
    }

    #[test]
    fn the_expected_harvest_counts_the_current_harvest_twice() {
        // §3.7: S0 already holds what is left of this year's harvest.
        let corn = vec![100.0, 50.0, 400.0];
        assert_eq!(
            expected_harvest(&corn, 900.0),
            1050.0,
            "S-2 spoils and is excluded"
        );
        assert_eq!(expected_harvest(&[70.0], 900.0), 900.0, "no storage");
    }

    #[test]
    fn the_documented_endowment_conserves_corn() {
        let mut parent = vec![900.0, 300.0, 30.0];
        let child = split_endowment(&mut parent, 0.25);
        assert_eq!(child, [225.0, 75.0, 7.5]);
        assert_eq!(parent, [675.0, 225.0, 22.5]);
        let total: f64 = parent.iter().chain(&child).sum();
        assert!((total - 1230.0).abs() < 1e-9);
    }
}
