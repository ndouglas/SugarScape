//! Per-tick summary statistics (Chapter II's Gini coefficient and Lorenz
//! curve, plus population and trait means).

use serde::Serialize;

use crate::agent::Tribe;
use crate::world::World;

pub const SERIES: [&str; 8] = [
    "population",
    "gini",
    "mean_wealth",
    "mean_vision",
    "mean_metabolism",
    "blue_fraction",
    "births",
    "deaths",
];

#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct Snapshot {
    pub tick: u64,
    pub population: u32,
    pub gini: f64,
    pub mean_wealth: f64,
    pub mean_vision: f64,
    pub mean_metabolism: f64,
    pub blue_fraction: f64,
    /// Sexual births this tick; replacements (rule R) are not births.
    pub births: u32,
    pub deaths: u32,
}

impl Snapshot {
    pub fn of(world: &World) -> Self {
        let n = world.population();
        let mean = |f: &dyn Fn(&crate::agent::Agent) -> f64| {
            if n == 0 {
                0.0
            } else {
                world.agents().map(f).sum::<f64>() / n as f64
            }
        };
        let w = wealths(world);
        Self {
            tick: world.tick,
            population: n as u32,
            gini: gini(&w),
            mean_wealth: mean(&|a| a.sugar),
            mean_vision: mean(&|a| f64::from(a.vision)),
            mean_metabolism: mean(&|a| f64::from(a.metabolism)),
            blue_fraction: mean(&|a| if a.tribe() == Tribe::Blue { 1.0 } else { 0.0 }),
            births: world.events().births,
            deaths: world.events().deaths.len() as u32,
        }
    }

    pub fn value(&self, name: &str) -> Option<f64> {
        Some(match name {
            "tick" => self.tick as f64,
            "population" => f64::from(self.population),
            "gini" => self.gini,
            "mean_wealth" => self.mean_wealth,
            "mean_vision" => self.mean_vision,
            "mean_metabolism" => self.mean_metabolism,
            "blue_fraction" => self.blue_fraction,
            "births" => f64::from(self.births),
            "deaths" => f64::from(self.deaths),
            _ => return None,
        })
    }
}

#[derive(Clone, Debug, Default)]
pub struct Stats {
    history: Vec<Snapshot>,
}

impl Stats {
    pub fn push(&mut self, s: Snapshot) {
        self.history.push(s);
    }

    pub fn latest(&self) -> Option<&Snapshot> {
        self.history.last()
    }

    pub fn history(&self) -> &[Snapshot] {
        &self.history
    }

    /// The full history of one series (or `"tick"`), or `None` if unknown.
    pub fn series(&self, name: &str) -> Option<Vec<f64>> {
        Snapshot::default().value(name)?;
        Some(
            self.history
                .iter()
                .map(|s| s.value(name).expect("known series"))
                .collect(),
        )
    }
}

pub fn wealths(world: &World) -> Vec<f64> {
    world.agents().map(|a| a.sugar).collect()
}

/// G = 2·Σ i·x₍ᵢ₎ / (n·Σx) − (n+1)/n over ascending wealth, i from 1.
pub fn gini(values: &[f64]) -> f64 {
    let n = values.len();
    let total: f64 = values.iter().sum();
    if n == 0 || total <= 0.0 {
        return 0.0;
    }
    let mut v = values.to_vec();
    v.sort_by(f64::total_cmp);
    let weighted: f64 = v.iter().enumerate().map(|(i, x)| (i + 1) as f64 * x).sum();
    let n = n as f64;
    (2.0 * weighted / (n * total) - (n + 1.0) / n).max(0.0)
}

/// Share of total wealth held by the poorest k/(points−1) of agents, for
/// k = 0..points.
pub fn lorenz(values: &[f64], points: usize) -> Vec<f64> {
    assert!(points >= 2);
    let mut v = values.to_vec();
    v.sort_by(f64::total_cmp);
    let total: f64 = v.iter().sum();
    let mut cumulative = vec![0.0];
    for x in &v {
        cumulative.push(cumulative.last().unwrap() + x);
    }
    (0..points)
        .map(|k| {
            if total <= 0.0 {
                return k as f64 / (points - 1) as f64;
            }
            let m = (k as f64 / (points - 1) as f64 * v.len() as f64).round() as usize;
            cumulative[m] / total
        })
        .collect()
}

/// `bins` equal-width bins over [0, max]; returns (bin width, counts).
pub fn histogram(values: &[f64], bins: usize) -> (f64, Vec<f64>) {
    assert!(bins >= 1);
    let max = values.iter().copied().fold(0.0, f64::max);
    let width = if max > 0.0 { max / bins as f64 } else { 1.0 };
    let mut counts = vec![0.0; bins];
    for &x in values {
        let i = ((x.max(0.0) / width) as usize).min(bins - 1);
        counts[i] += 1.0;
    }
    (width, counts)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;

    #[test]
    fn gini_of_equal_wealth_is_zero_and_of_total_concentration_is_high() {
        assert_eq!(gini(&[5.0, 5.0, 5.0, 5.0]), 0.0);
        assert!((gini(&[0.0, 0.0, 0.0, 1.0]) - 0.75).abs() < 1e-12);
        assert_eq!(gini(&[]), 0.0);
        assert_eq!(
            gini(&[3.0, 1.0, 2.0]),
            gini(&[1.0, 2.0, 3.0]),
            "order-independent"
        );
    }

    #[test]
    fn lorenz_curve_of_equal_wealth_is_the_diagonal() {
        let l = lorenz(&[2.0, 2.0, 2.0, 2.0], 5);
        assert_eq!(l, vec![0.0, 0.25, 0.5, 0.75, 1.0]);
        assert_eq!(
            lorenz(&[0.0, 0.0, 0.0, 1.0], 5),
            vec![0.0, 0.0, 0.0, 0.0, 1.0]
        );
    }

    #[test]
    fn histogram_bins_from_zero_to_max() {
        let (width, counts) = histogram(&[0.5, 1.5, 3.9, 4.0], 4);
        assert_eq!(width, 1.0);
        assert_eq!(counts, vec![1.0, 1.0, 0.0, 2.0]);
    }

    #[test]
    fn world_records_a_snapshot_per_tick() {
        let mut w = World::new(Config::default(), 3).unwrap();
        assert_eq!(w.stats.history().len(), 1);
        assert_eq!(w.stats.latest().unwrap().population, 400);
        w.run(3);
        assert_eq!(w.stats.history().len(), 4);
        assert_eq!(w.stats.series("tick").unwrap(), vec![0.0, 1.0, 2.0, 3.0]);
        assert_eq!(w.stats.series("population").unwrap().len(), 4);
        assert!(w.stats.series("nonsense").is_none());
        let s = w.stats.latest().unwrap();
        assert_eq!(s.population as usize, w.population());
        assert!((1.0..=6.0).contains(&s.mean_vision));
        for name in SERIES {
            assert!(s.value(name).is_some(), "{name}");
        }
    }
}
