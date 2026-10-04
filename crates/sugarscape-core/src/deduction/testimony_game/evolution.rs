use super::{fitness, Distribution, Error, Genome};
use crate::rng::seeded;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SearchMethod {
    Genetic,
    Random,
}
pub fn search_seed(seed: u64, method: SearchMethod) -> u64 {
    let identity = match method {
        SearchMethod::Genetic => 1u64,
        SearchMethod::Random => 2,
    };
    seed.wrapping_mul(6364136223846793005)
        .wrapping_add(identity.wrapping_mul(1442695040888963407))
}
#[derive(Clone, Debug, PartialEq, Eq)]
struct Candidate {
    genome: Genome,
    fitness: i64,
}
fn rank(a: &Candidate, b: &Candidate) -> Ordering {
    fn norm(g: &Genome) -> i16 {
        g.b.abs() + g.u + g.d + g.k.abs()
    }
    fn tuple(g: &Genome) -> (i16, i16, i16, i16) {
        (g.b, g.u, g.d, g.k)
    }
    b.fitness
        .cmp(&a.fitness)
        .then_with(|| norm(&a.genome).cmp(&norm(&b.genome)))
        .then_with(|| tuple(&a.genome).cmp(&tuple(&b.genome)))
}
fn initialize(rng: &mut impl Rng) -> Genome {
    Genome {
        b: rng.gen_range(-16..=16),
        u: rng.gen_range(0..=16),
        d: rng.gen_range(0..=16),
        k: rng.gen_range(-16..=16),
    }
}
fn crossover(rng: &mut impl Rng, a: &Genome, b: &Genome) -> Genome {
    Genome {
        b: if rng.gen::<bool>() { a.b } else { b.b },
        u: if rng.gen::<bool>() { a.u } else { b.u },
        d: if rng.gen::<bool>() { a.d } else { b.d },
        k: if rng.gen::<bool>() { a.k } else { b.k },
    }
}
fn mutate(rng: &mut impl Rng, g: &mut Genome) {
    for (gene, low, high) in [
        (&mut g.b, -16, 16),
        (&mut g.u, 0, 16),
        (&mut g.d, 0, 16),
        (&mut g.k, -16, 16),
    ] {
        if rng.gen_ratio(1, 4) {
            let direction = if rng.gen::<bool>() { 1 } else { -1 };
            *gene = (*gene + direction).clamp(low, high);
        }
    }
}
fn tournament<'a>(rng: &mut impl Rng, population: &'a [Candidate]) -> &'a Candidate {
    let mut winner = &population[rng.gen_range(0..population.len())];
    for _ in 1..3 {
        let candidate = &population[rng.gen_range(0..population.len())];
        if rank(candidate, winner) == Ordering::Less {
            winner = candidate;
        }
    }
    winner
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SearchSettings {
    pub population: u32,
    pub generations: u32,
    pub elites: u32,
    pub tournament: u32,
    pub mutation_numerator: u32,
    pub mutation_denominator: u32,
    pub evaluations: u32,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProgressPoint {
    pub evaluations: u32,
    pub best_fitness_numerator: i64,
    pub champion: Genome,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SearchRun {
    pub seed: u64,
    pub derived_seed: u64,
    pub method: SearchMethod,
    pub seed_derivation: String,
    pub settings: SearchSettings,
    pub champion: Genome,
    pub fitness_numerator: i64,
    pub denominator: u64,
    pub evaluations: u32,
    pub curve: Vec<ProgressPoint>,
}
/// The frozen search protocol; seed streams are independent of game worlds.
pub const SEARCH_SEED_DERIVATION: &str = "testimony-search-seed-v1";
fn settings() -> SearchSettings {
    SearchSettings {
        population: 64,
        generations: 50,
        elites: 2,
        tournament: 3,
        mutation_numerator: 1,
        mutation_denominator: 4,
        evaluations: 3164,
    }
}
pub fn search_ga(distribution: &Distribution, seed: u64) -> Result<SearchRun, Error> {
    search(distribution, seed, SearchMethod::Genetic)
}
pub fn search_random(distribution: &Distribution, seed: u64) -> Result<SearchRun, Error> {
    search(distribution, seed, SearchMethod::Random)
}
fn search(
    distribution: &Distribution,
    seed: u64,
    method: SearchMethod,
) -> Result<SearchRun, Error> {
    let settings = settings();
    let derived_seed = search_seed(seed, method);
    let mut rng = seeded(derived_seed);
    let curve = run_search(
        &mut rng,
        method,
        settings.population as usize,
        settings.generations as usize,
        settings.elites as usize,
        |g| fitness(distribution, g),
    )?;
    let last = curve
        .last()
        .expect("frozen search evaluates a nonempty population");
    Ok(SearchRun {
        seed,
        derived_seed,
        method,
        seed_derivation: SEARCH_SEED_DERIVATION.into(),
        settings,
        champion: last.champion.clone(),
        fitness_numerator: last.best_fitness_numerator,
        denominator: distribution.denominator,
        evaluations: curve.len() as u32,
        curve,
    })
}
// Private sizes allow small counting fixtures; public protocol is never configurable.
fn run_search(
    rng: &mut impl Rng,
    method: SearchMethod,
    population_size: usize,
    generations: usize,
    elites: usize,
    mut evaluate: impl FnMut(&Genome) -> Result<i64, Error>,
) -> Result<Vec<ProgressPoint>, Error> {
    let budget = population_size + generations * (population_size - elites);
    let mut curve = Vec::with_capacity(budget);
    let mut best: Option<Candidate> = None;
    let mut record = |genome: Genome| -> Result<Candidate, Error> {
        let candidate = Candidate {
            fitness: evaluate(&genome)?,
            genome,
        };
        if best
            .as_ref()
            .is_none_or(|old| rank(&candidate, old) == Ordering::Less)
        {
            best = Some(candidate.clone());
        }
        let champion = best.as_ref().expect("evaluated candidate");
        curve.push(ProgressPoint {
            evaluations: curve.len() as u32 + 1,
            best_fitness_numerator: champion.fitness,
            champion: champion.genome.clone(),
        });
        Ok(candidate)
    };
    match method {
        SearchMethod::Random => {
            for _ in 0..budget {
                record(initialize(rng))?;
            }
        }
        SearchMethod::Genetic => {
            let mut population = Vec::with_capacity(population_size);
            for _ in 0..population_size {
                population.push(record(initialize(rng))?);
            }
            for _ in 0..generations {
                // Stable sorting preserves input order for fully equal candidates.
                population.sort_by(rank);
                let mut next = population[..elites].to_vec();
                for _ in elites..population_size {
                    let a = tournament(rng, &population);
                    let b = tournament(rng, &population);
                    let mut child = crossover(rng, &a.genome, &b.genome);
                    mutate(rng, &mut child);
                    next.push(record(child)?);
                }
                population = next;
            }
        }
    }
    Ok(curve)
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::rng::seeded;
    use rand::rngs::mock::StepRng;
    fn g(b: i16, u: i16, d: i16, k: i16) -> Genome {
        Genome { b, u, d, k }
    }
    fn c(genome: Genome, fitness: i64) -> Candidate {
        Candidate { genome, fitness }
    }
    #[test]
    fn seed_formula_golden_and_overflow() {
        assert_eq!(search_seed(0, SearchMethod::Genetic), 1442695040888963407);
        assert_eq!(search_seed(7, SearchMethod::Genetic), 9098160460397411210);
        assert_eq!(
            search_seed(u64::MAX, SearchMethod::Genetic),
            13525302890751722018
        );
        assert_ne!(
            search_seed(7, SearchMethod::Genetic),
            search_seed(7, SearchMethod::Random)
        );
    }
    #[test]
    fn exact_payoff_precedes_norm_and_lexicographic_order() {
        assert_eq!(
            rank(&c(g(16, 16, 16, 16), 1), &c(g(0, 0, 0, 0), 0)),
            Ordering::Less
        );
        assert_eq!(
            rank(&c(g(0, 0, 0, 0), 1), &c(g(-1, 0, 0, 0), 1)),
            Ordering::Less
        );
        assert_eq!(
            rank(&c(g(-1, 0, 0, 0), 1), &c(g(0, 0, 0, 1), 1)),
            Ordering::Less
        );
        let candidate = c(g(1, 2, 3, 4), 2);
        assert_eq!(rank(&candidate, &candidate), Ordering::Equal);
    }
    #[test]
    fn initialization_bounds_and_uniform_range_endpoints() {
        let mut rng = seeded(9);
        let mut seen = [
            std::collections::BTreeSet::new(),
            std::collections::BTreeSet::new(),
            std::collections::BTreeSet::new(),
            std::collections::BTreeSet::new(),
        ];
        for _ in 0..4000 {
            let x = initialize(&mut rng);
            x.validate().unwrap();
            for (s, v) in seen.iter_mut().zip([x.b, x.u, x.d, x.k]) {
                s.insert(v);
            }
        }
        assert_eq!(seen.map(|s| s.len()), [33, 17, 17, 33]);
    }
    #[test]
    fn crossover_selects_parent_genes() {
        let a = g(-16, 0, 0, -16);
        let b = g(16, 16, 16, 16);
        let mut rng = seeded(2);
        for _ in 0..100 {
            let x = crossover(&mut rng, &a, &b);
            for ((v, l), r) in [x.b, x.u, x.d, x.k]
                .into_iter()
                .zip([a.b, a.u, a.d, a.k])
                .zip([b.b, b.u, b.d, b.k])
            {
                assert!(v == l || v == r);
            }
        }
    }
    #[test]
    fn deterministic_mutation_occurrence_direction_and_clamp() {
        let mut x = g(-16, 0, 16, 16);
        mutate(&mut StepRng::new(0, 0), &mut x);
        assert_eq!(x, g(-16, 0, 15, 15));
        let mut x = g(0, 8, 8, 0);
        mutate(&mut StepRng::new(u64::MAX, 0), &mut x);
        assert_eq!(x, g(0, 8, 8, 0));
        let mut x = g(0, 8, 8, 0);
        mutate(&mut StepRng::new(1u64 << 62, 0), &mut x);
        assert_eq!(x, g(0, 8, 8, 0));
    }
    #[test]
    fn tournament_samples_with_replacement() {
        let population = [c(g(0, 0, 0, 0), 0), c(g(1, 0, 0, 0), 1)];
        assert_eq!(
            tournament(&mut StepRng::new(0, 0), &population),
            &population[0]
        );
    }
    #[test]
    fn counted_repeated_candidates_and_elites_preserve_best() {
        let mut count = 0;
        let curve = run_search(
            &mut StepRng::new(0, 0),
            SearchMethod::Genetic,
            4,
            3,
            2,
            |_| {
                count += 1;
                Ok(if count == 1 { 10 } else { 0 })
            },
        )
        .unwrap();
        assert_eq!(count, 10);
        assert_eq!(curve.len(), 10);
        assert!(curve.iter().all(|p| p.best_fitness_numerator == 10));
        for (i, p) in curve.iter().enumerate() {
            assert_eq!(p.evaluations, i as u32 + 1);
        }
    }
    #[test]
    fn public_frozen_search_budget_curve_fitness_and_repeatability() {
        use super::super::{enumerate, Config, Probability};
        let dist = enumerate(&Config::standard(
            Probability {
                numerator: 4,
                denominator: 5,
            },
            Probability {
                numerator: 3,
                denominator: 4,
            },
        ))
        .unwrap();
        for search in [search_ga, search_random] {
            let run = search(&dist, 0).unwrap();
            let repeat = search(&dist, 0).unwrap();
            assert_eq!(
                serde_json::to_vec(&run).unwrap(),
                serde_json::to_vec(&repeat).unwrap()
            );
            assert_eq!(run.evaluations, 3164);
            assert_eq!(run.curve.len(), 3164);
            assert_eq!(run.denominator, dist.denominator);
            for (i, p) in run.curve.iter().enumerate() {
                assert_eq!(p.evaluations, i as u32 + 1);
                assert_eq!(
                    p.best_fitness_numerator,
                    fitness(&dist, &p.champion).unwrap()
                );
            }
            for pair in run.curve.windows(2) {
                assert_ne!(
                    rank(
                        &c(pair[1].champion.clone(), pair[1].best_fitness_numerator),
                        &c(pair[0].champion.clone(), pair[0].best_fitness_numerator)
                    ),
                    Ordering::Greater
                );
            }
            let last = run.curve.last().unwrap();
            assert_eq!(last.champion, run.champion);
            assert_eq!(last.best_fitness_numerator, run.fitness_numerator);
        }
    }

    // A raw deterministic stream verifies occurrence thresholds and consumed draws
    // without probabilistic frequency assertions or another RNG dependency.
    struct Script(std::collections::VecDeque<(u8, u64)>);
    impl Script {
        fn new(values: &[(u8, u64)]) -> Self {
            Self(values.iter().copied().collect())
        }
        fn draw(&mut self, width: u8) -> u64 {
            let (expected, value) = self.0.pop_front().expect("unexpected RNG draw");
            assert_eq!(width, expected);
            value
        }
        fn exhausted(&self) {
            assert!(self.0.is_empty());
        }
    }
    impl rand::RngCore for Script {
        fn next_u32(&mut self) -> u32 {
            self.draw(32) as u32
        }
        fn next_u64(&mut self) -> u64 {
            self.draw(64)
        }
        fn fill_bytes(&mut self, _: &mut [u8]) {
            panic!("unexpected byte draw")
        }
        fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), rand::Error> {
            self.fill_bytes(dest);
            Ok(())
        }
    }
    #[test]
    fn operator_raw_draw_order_and_mutation_probability_boundary() {
        let mut rng = Script::new(&[(32, 1 << 31), (32, 0), (32, 1 << 31), (32, 0)]);
        assert_eq!(initialize(&mut rng), g(0, 0, 8, -16));
        rng.exhausted();
        let mut rng = Script::new(&[(32, 1 << 31), (32, 0), (32, 1 << 31), (32, 0)]);
        assert_eq!(
            crossover(&mut rng, &g(1, 2, 3, 4), &g(5, 6, 7, 8)),
            g(1, 6, 3, 8)
        );
        rng.exhausted();
        let mut rng = Script::new(&[
            (64, (1 << 62) - 1),
            (32, 1 << 31),
            (64, 1 << 62),
            (64, 0),
            (32, 0),
            (64, u64::MAX),
        ]);
        let mut genome = g(0, 8, 8, 0);
        mutate(&mut rng, &mut genome);
        assert_eq!(genome, g(1, 8, 7, 0));
        rng.exhausted();
        let mut rng = Script::new(&[
            (64, 0),
            (32, 1 << 31),
            (64, u64::MAX),
            (64, u64::MAX),
            (64, u64::MAX),
        ]);
        let mut genome = g(16, 8, 8, 0);
        mutate(&mut rng, &mut genome);
        assert_eq!(genome, g(16, 8, 8, 0));
        rng.exhausted();
        let mut rng = Script::new(&[(64, 1 << 63), (64, 0), (64, 0)]);
        let population = [c(g(0, 0, 0, 0), 0), c(g(1, 0, 0, 0), 1)];
        assert_eq!(tournament(&mut rng, &population), &population[1]);
        rng.exhausted();
    }
    #[test]
    fn malformed_distribution_errors_propagate_for_both_searches() {
        use super::super::{enumerate, Config, Probability};
        let mut dist = enumerate(&Config::standard(
            Probability {
                numerator: 4,
                denominator: 5,
            },
            Probability {
                numerator: 3,
                denominator: 4,
            },
        ))
        .unwrap();
        dist.denominator = 0;
        assert!(search_ga(&dist, 0).is_err());
        assert!(search_random(&dist, 0).is_err());
    }
}
