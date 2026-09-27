//! The Ants and Recruitment model's statistics: how the colony is split, how
//! widely the split swings against theory, and how often it flips.

use serde::Serialize;

use crate::stats::Series;

/// A source holding this share or more holds the colony (Kirman's 80 %).
pub const HOLD: f64 = 0.8;

/// The statistics series, in the order the CSV and the page list them.
pub const SERIES: [&str; 7] = [
    "share",
    "top_share",
    "variance",
    "theory_variance",
    "flips",
    "residence",
    "extreme",
];

/// One step's statistics.
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct AntsSnapshot {
    pub tick: u64,
    /// z: the share of ants at the first source.
    pub share: f64,
    /// The largest source's share.
    pub top_share: f64,
    /// Var[z] over the steps so far.
    pub variance: f64,
    /// The variance theory gives (Kirman's exact chain or Alfarano and
    /// Milaković's mean field); NaN (null) when neither applies.
    pub theory_variance: f64,
    /// Times the colony moved from one source holding 80 % to another.
    pub flips: u32,
    /// Mean steps between flips (0 before the first).
    pub residence: f64,
    /// The share of steps with some source holding 80 % or more.
    pub extreme: f64,
}

impl Series for AntsSnapshot {
    fn tick(&self) -> u64 {
        self.tick
    }

    fn value(&self, name: &str) -> Option<f64> {
        Some(match name {
            "tick" => self.tick as f64,
            "share" => self.share,
            "top_share" => self.top_share,
            "variance" => self.variance,
            "theory_variance" => self.theory_variance,
            "flips" => f64::from(self.flips),
            "residence" => self.residence,
            "extreme" => self.extreme,
            _ => return None,
        })
    }
}

/// Running mean and variance (Welford).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Running {
    pub count: u64,
    pub mean: f64,
    m2: f64,
}

impl Running {
    pub fn push(&mut self, x: f64) {
        self.count += 1;
        let d = x - self.mean;
        self.mean += d / self.count as f64;
        self.m2 += d * (x - self.mean);
    }

    /// The population variance of the values so far (0 before two).
    pub fn variance(&self) -> f64 {
        if self.count < 2 {
            0.0
        } else {
            self.m2 / self.count as f64
        }
    }

    pub fn bits(&self) -> [u64; 3] {
        [self.count, self.mean.to_bits(), self.m2.to_bits()]
    }
}

/// Which source holds the colony, and the flips between holders.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Regimes {
    /// The last source to hold 80 % or more.
    pub holder: Option<u32>,
    /// The step the current holder took over.
    pub since: u64,
    pub flips: u32,
    /// Steps from the first holder's arrival to the latest flip.
    pub span: u64,
}

impl Regimes {
    /// Sees step `t`'s largest source `top` with share `x`.
    pub fn see(&mut self, t: u64, top: u32, x: f64) {
        if x < HOLD {
            return;
        }
        match self.holder {
            None => {
                self.holder = Some(top);
                self.since = t;
            }
            Some(h) if h != top => {
                self.flips += 1;
                self.span += t - self.since;
                self.holder = Some(top);
                self.since = t;
            }
            Some(_) => {}
        }
    }

    pub fn residence(&self) -> f64 {
        if self.flips == 0 {
            0.0
        } else {
            self.span as f64 / f64::from(self.flips)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn running_variance_is_the_population_variance() {
        let mut r = Running::default();
        for x in [0.1, 0.9, 0.5, 0.5] {
            r.push(x);
        }
        assert!((r.mean - 0.5).abs() < 1e-12);
        assert!((r.variance() - 0.08).abs() < 1e-12);
    }

    #[test]
    fn flips_count_changes_of_holder() {
        let mut g = Regimes::default();
        for (t, top, x) in [
            (1, 0, 0.5),
            (2, 0, 0.85),
            (3, 0, 0.6),
            (4, 1, 0.55),
            (5, 0, 0.9),
            (7, 1, 0.8),
            (9, 1, 0.95),
            (12, 0, 0.81),
        ] {
            g.see(t, top, x);
        }
        // Held by 0 from step 2; the dip to 0.6 is no flip; 1 takes over at 7,
        // 0 again at 12.
        assert_eq!((g.flips, g.span), (2, 10));
        assert_eq!(g.residence(), 5.0);
        assert_eq!(Regimes::default().residence(), 0.0);
    }
}
