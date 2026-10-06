//! Frozen finite-policy searches; only the immutable training panel enters selection.
use super::{Error, Policy, TrainingPanel};
use crate::rng::seeded;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

pub const SEARCH_VERSION: &str = "strategic-reporting-search-v1";
pub const SEARCH_SEED_DERIVATION: &str = "strategic-reporting-search-seed-v1";
const POLICY_BITS: u32 = 18;
const POPULATION: usize = 64;
const GENERATIONS: usize = 50;
const ELITES: usize = 2;
const TOURNAMENT: usize = 3;
const EVALUATIONS: usize = POPULATION + GENERATIONS * (POPULATION - ELITES);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SearchMethod {
    Genetic,
    Random,
}
/// Search streams are distinct from the frozen testimony-game search streams.
pub fn search_seed(seed: u64, method: SearchMethod) -> u64 {
    let identity = match method {
        SearchMethod::Genetic => 3u64,
        SearchMethod::Random => 4,
    };
    seed.wrapping_mul(6364136223846793005)
        .wrapping_add(identity.wrapping_mul(1442695040888963407))
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SearchSettings {
    pub version: u16,
    pub policy_bits: u32,
    pub population: u32,
    pub generations: u32,
    pub elites: u32,
    pub tournament: u32,
    pub crossover_numerator: u32,
    pub crossover_denominator: u32,
    pub mutation_numerator: u32,
    pub mutation_denominator: u32,
    pub evaluations: u32,
    pub ranking: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProgressPoint {
    pub evaluations: u32,
    pub best_fitness_numerator: i64,
    pub champion: Policy,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SearchRun {
    pub version: String,
    pub seed: u64,
    pub derived_seed: u64,
    pub method: SearchMethod,
    pub seed_derivation: String,
    pub settings: SearchSettings,
    pub champion: Policy,
    pub fitness_numerator: i64,
    pub denominator: u64,
    pub evaluations: u32,
    pub curve: Vec<ProgressPoint>,
}
fn settings() -> SearchSettings {
    SearchSettings {
        version: 1,
        policy_bits: POLICY_BITS,
        population: POPULATION as u32,
        generations: GENERATIONS as u32,
        elites: ELITES as u32,
        tournament: TOURNAMENT as u32,
        crossover_numerator: 1,
        crossover_denominator: 2,
        mutation_numerator: 1,
        mutation_denominator: POLICY_BITS,
        evaluations: EVALUATIONS as u32,
        ranking: "higher_exact_fitness_then_lower_unsigned_encoding_then_stable_order".into(),
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
struct Candidate {
    policy: Policy,
    fitness: i64,
}
fn rank(a: &Candidate, b: &Candidate) -> Ordering {
    b.fitness
        .cmp(&a.fitness)
        .then_with(|| a.policy.bits.cmp(&b.policy.bits))
}
fn initialize(rng: &mut impl Rng) -> Policy {
    Policy {
        bits: rng.gen_range(0..(1u32 << POLICY_BITS)),
    }
}
fn crossover(rng: &mut impl Rng, a: &Policy, b: &Policy) -> Policy {
    let mut bits = 0;
    for bit in 0..POLICY_BITS {
        let parent = if rng.gen::<bool>() { a } else { b };
        bits |= parent.bits & (1 << bit);
    }
    Policy { bits }
}
fn mutate(rng: &mut impl Rng, policy: &mut Policy) {
    for bit in 0..POLICY_BITS {
        if rng.gen_ratio(1, POLICY_BITS) {
            policy.bits ^= 1 << bit;
        }
    }
}
fn tournament<'a>(rng: &mut impl Rng, population: &'a [Candidate]) -> &'a Candidate {
    let mut winner = &population[rng.gen_range(0..population.len())];
    for _ in 1..TOURNAMENT {
        let candidate = &population[rng.gen_range(0..population.len())];
        if rank(candidate, winner) == Ordering::Less {
            winner = candidate;
        }
    }
    winner
}
fn record(
    panel: &TrainingPanel,
    policy: Policy,
    best: &mut Option<Candidate>,
    curve: &mut Vec<ProgressPoint>,
) -> Result<Candidate, Error> {
    // Repeated encodings are evaluated and recorded, never cached away.
    let candidate = Candidate {
        fitness: panel.fitness(&policy)?,
        policy,
    };
    if best
        .as_ref()
        .is_none_or(|previous| rank(&candidate, previous) == Ordering::Less)
    {
        *best = Some(candidate.clone());
    }
    let champion = best.as_ref().expect("one candidate has been evaluated");
    curve.push(ProgressPoint {
        evaluations: curve.len() as u32 + 1,
        best_fitness_numerator: champion.fitness,
        champion: champion.policy.clone(),
    });
    Ok(candidate)
}
fn run_search(
    panel: &TrainingPanel,
    rng: &mut impl Rng,
    method: SearchMethod,
) -> Result<Vec<ProgressPoint>, Error> {
    let mut curve = Vec::with_capacity(EVALUATIONS);
    let mut best = None;
    match method {
        SearchMethod::Random => {
            for _ in 0..EVALUATIONS {
                record(panel, initialize(rng), &mut best, &mut curve)?;
            }
        }
        SearchMethod::Genetic => {
            let mut population = Vec::with_capacity(POPULATION);
            for _ in 0..POPULATION {
                population.push(record(panel, initialize(rng), &mut best, &mut curve)?);
            }
            for _ in 0..GENERATIONS {
                // Stable ordering preserves the earlier entry on a complete tie.
                population.sort_by(rank);
                let mut next = population[..ELITES].to_vec();
                for _ in ELITES..POPULATION {
                    let a = tournament(rng, &population);
                    let b = tournament(rng, &population);
                    let mut child = crossover(rng, &a.policy, &b.policy);
                    mutate(rng, &mut child);
                    next.push(record(panel, child, &mut best, &mut curve)?);
                }
                population = next;
            }
        }
    }
    Ok(curve)
}
/// Run the fixed protocol; neither holdout evaluators nor callbacks enter selection.
/// The returned champion and curve are owned snapshots of training results.
pub fn search(panel: &TrainingPanel, seed: u64, method: SearchMethod) -> Result<SearchRun, Error> {
    let derived_seed = search_seed(seed, method);
    let curve = run_search(panel, &mut seeded(derived_seed), method)?;
    let last = curve
        .last()
        .expect("the fixed protocol evaluates 3164 candidates");
    Ok(SearchRun {
        version: SEARCH_VERSION.into(),
        seed,
        derived_seed,
        method,
        seed_derivation: SEARCH_SEED_DERIVATION.into(),
        settings: settings(),
        champion: last.champion.clone(),
        fitness_numerator: last.best_fitness_numerator,
        denominator: panel.denominator(),
        evaluations: curve.len() as u32,
        curve,
    })
}
#[cfg(test)]
#[path = "tests/evolution.rs"]
mod tests;
