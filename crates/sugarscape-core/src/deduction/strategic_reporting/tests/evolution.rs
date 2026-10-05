use super::*;
use crate::deduction::strategic_reporting::{
    enumerate, evaluate, Config, FrozenListener, Listener, Probability, UtilityTable,
};
use std::collections::VecDeque;

fn panel(listener: Listener, accuracy: u16) -> TrainingPanel {
    TrainingPanel::new(
        &Config::standard(
            Probability {
                numerator: accuracy,
                denominator: 5,
            },
            Probability {
                numerator: 3,
                denominator: 4,
            },
            UtilityTable::opposed(),
        ),
        vec![FrozenListener {
            algorithm: listener,
            assumed_copy_prior: Probability {
                numerator: 3,
                denominator: 4,
            },
        }],
    )
    .unwrap()
}
fn candidate(bits: u32, fitness: i64) -> Candidate {
    Candidate {
        policy: Policy::new(bits).unwrap(),
        fitness,
    }
}

#[test]
fn seed_derivation_is_literal_wrapping_and_separate_from_legacy() {
    for (seed, genetic, random) in [
        (0, 4328085122666890221, 5770780163555853628),
        (7, 11983550542175338024, 13426245583064301431),
        (u64::MAX, 16410692972529648832, 17853388013418612239),
    ] {
        assert_eq!(search_seed(seed, SearchMethod::Genetic), genetic);
        assert_eq!(search_seed(seed, SearchMethod::Random), random);
        for method in [SearchMethod::Genetic, SearchMethod::Random] {
            for old in [
                crate::deduction::testimony_game::SearchMethod::Genetic,
                crate::deduction::testimony_game::SearchMethod::Random,
            ] {
                assert_ne!(
                    search_seed(seed, method),
                    crate::deduction::testimony_game::search_seed(seed, old)
                );
            }
        }
    }
}

#[test]
fn exact_fitness_precedes_unsigned_encoding_and_full_ties_are_stable() {
    assert_eq!(
        rank(&candidate(262143, 1), &candidate(0, 0)),
        Ordering::Less
    );
    assert_eq!(rank(&candidate(3, 1), &candidate(4, 1)), Ordering::Less);
    assert_eq!(rank(&candidate(3, 1), &candidate(3, 1)), Ordering::Equal);
    let mut tagged = [
        (candidate(3, 1), 2),
        (candidate(3, 1), 1),
        (candidate(0, 0), 0),
    ];
    tagged.sort_by(|a, b| rank(&a.0, &b.0));
    assert_eq!(tagged.map(|v| v.1), [2, 1, 0]);
}

// Explicit raw streams pin draw width/order, rejection-free uniform endpoints,
// per-bit parent selection and the Bernoulli boundary. These literals were
// independently checked with Python integer arithmetic and rand 0.8.8's
// UniformInt, Standard<bool> and Bernoulli::from_ratio definitions.
struct Script(VecDeque<(u32, u64)>);
impl Script {
    fn new(draws: Vec<(u32, u64)>) -> Self {
        Self(draws.into())
    }
    fn draw(&mut self, width: u32) -> u64 {
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
        panic!("unexpected byte draw");
    }
    fn try_fill_bytes(&mut self, bytes: &mut [u8]) -> Result<(), rand::Error> {
        self.fill_bytes(bytes);
        Ok(())
    }
}

#[test]
fn uniform_initialization_uses_all_eighteen_bits() {
    let mut raw = Script::new(vec![(32, 0), (32, 0x8000_0000), (32, 0xffff_c000)]);
    assert_eq!(initialize(&mut raw).bits, 0);
    assert_eq!(initialize(&mut raw).bits, 131072);
    assert_eq!(initialize(&mut raw).bits, 262143);
    raw.exhausted();
    let mut rng = seeded(993);
    for _ in 0..128 {
        Policy::new(initialize(&mut rng).bits).unwrap();
    }
}

#[test]
fn crossover_uses_independent_draws_for_every_bit() {
    let mut raw = Script::new(
        (0..18)
            .map(|bit| {
                (
                    32,
                    if [1, 4, 17].contains(&bit) {
                        1 << 31
                    } else {
                        0
                    },
                )
            })
            .collect(),
    );
    let child = crossover(&mut raw, &Policy::positive(), &Policy::negative());
    assert_eq!(child.bits, 131090);
    raw.exhausted();
    for bit in 0..18 {
        let mut raw = Script::new(
            (0..18)
                .map(|index| (32, if index == bit { 1 << 31 } else { 0 }))
                .collect(),
        );
        assert_eq!(
            crossover(&mut raw, &Policy::positive(), &Policy::negative()).bits,
            1 << bit
        );
        raw.exhausted();
    }
}

#[test]
fn mutation_flips_selected_bits_with_one_in_eighteen_boundary() {
    // floor(f64(1/18) * 2^64), as used by rand's specified gen_ratio.
    const THRESHOLD: u64 = 1024819115206086144;
    let mut raw = Script::new(
        (0..18)
            .map(|bit| {
                (
                    64,
                    if [0, 7, 17].contains(&bit) {
                        THRESHOLD - 1
                    } else {
                        THRESHOLD
                    },
                )
            })
            .collect(),
    );
    let mut p = Policy::copy();
    mutate(&mut raw, &mut p);
    assert_eq!(p.bits, 43563);
    raw.exhausted();
    for bit in 0..18 {
        let mut raw = Script::new(
            (0..18)
                .map(|index| (64, if index == bit { 0 } else { u64::MAX }))
                .collect(),
        );
        let mut p = Policy::negative();
        mutate(&mut raw, &mut p);
        assert_eq!(p.bits, 1 << bit);
        raw.exhausted();
    }
}

#[test]
fn tournament_keeps_first_equal_winner_and_samples_with_replacement() {
    let population = [
        candidate(8, 5),
        candidate(8, 5),
        candidate(9, 5),
        candidate(0, 4),
    ];
    let scale = 1u64 << (usize::BITS - 2);
    let mut raw = Script::new(vec![
        (usize::BITS, 2 * scale),
        (usize::BITS, scale),
        (usize::BITS, 0),
    ]);
    assert!(std::ptr::eq(
        tournament(&mut raw, &population),
        &population[1]
    ));
    raw.exhausted();
    let mut raw = Script::new(vec![(usize::BITS, 0); 3]);
    assert!(std::ptr::eq(
        tournament(&mut raw, &population),
        &population[0]
    ));
    raw.exhausted();
}

#[test]
fn both_methods_count_repeated_evaluations_and_disclose_frozen_settings() {
    let training = panel(Listener::Passive, 4);
    for method in [SearchMethod::Genetic, SearchMethod::Random] {
        let curve =
            run_search(&training, &mut rand::rngs::mock::StepRng::new(0, 0), method).unwrap();
        assert_eq!(curve.len(), 3164);
        assert!(curve
            .iter()
            .all(|p| p.champion.bits == 0 && p.best_fitness_numerator == 0));
        for (index, point) in curve.iter().enumerate() {
            assert_eq!(point.evaluations, index as u32 + 1);
        }
    }
    let s = settings();
    assert_eq!(s.version, 1);
    assert_eq!(
        (
            s.policy_bits,
            s.population,
            s.generations,
            s.elites,
            s.tournament
        ),
        (18, 64, 50, 2, 3)
    );
    assert_eq!((s.crossover_numerator, s.crossover_denominator), (1, 2));
    assert_eq!(
        (s.mutation_numerator, s.mutation_denominator, s.evaluations),
        (1, 18, 3164)
    );
    assert_eq!(
        s.ranking,
        "higher_exact_fitness_then_lower_unsigned_encoding_then_stable_order"
    );
}

#[test]
fn frozen_search_is_reproducible_and_holdout_cannot_change_training() {
    let training = panel(Listener::Bayesian, 4);
    let holdout = panel(Listener::Skeptical, 3);
    // Correctness probes deliberately use a seed outside collection seeds 0..19.
    for method in [SearchMethod::Genetic, SearchMethod::Random] {
        let run = search(&training, 991, method).unwrap();
        let before = serde_json::to_vec(&run).unwrap();
        assert_eq!(run.version, "strategic-reporting-search-v1");
        assert_eq!(run.seed_derivation, "strategic-reporting-search-seed-v1");
        assert_eq!(run.evaluations, 3164);
        assert_eq!(run.curve.len(), 3164);
        assert_eq!(run.denominator, training.denominator());
        assert_eq!(
            run.fitness_numerator,
            training.fitness(&run.champion).unwrap()
        );
        for point in &run.curve {
            assert_eq!(
                point.best_fitness_numerator,
                training.fitness(&point.champion).unwrap()
            );
        }
        for pair in run.curve.windows(2) {
            assert_ne!(
                rank(
                    &candidate(pair[1].champion.bits, pair[1].best_fitness_numerator),
                    &candidate(pair[0].champion.bits, pair[0].best_fitness_numerator)
                ),
                Ordering::Greater
            );
        }
        assert_eq!(run.curve.last().unwrap().champion, run.champion);
        assert_eq!(
            run.curve.last().unwrap().best_fitness_numerator,
            run.fitness_numerator
        );
        holdout.fitness(&run.champion).unwrap();
        evaluate(
            &enumerate(holdout.config()).unwrap(),
            &run.champion,
            &holdout.listeners()[0],
        )
        .unwrap();
        assert_eq!(serde_json::to_vec(&run).unwrap(), before);
        assert_eq!(
            serde_json::to_vec(&search(&training, 991, method).unwrap()).unwrap(),
            before
        );
    }
}

#[test]
fn search_contracts_reject_unknown_fields() {
    let mut json = serde_json::to_value(settings()).unwrap();
    json["unknown"] = true.into();
    assert!(serde_json::from_value::<SearchSettings>(json).is_err());
    let run = search(&panel(Listener::Passive, 4), u64::MAX, SearchMethod::Random).unwrap();
    let mut json = serde_json::to_value(run).unwrap();
    json["unknown"] = true.into();
    assert!(serde_json::from_value::<SearchRun>(json).is_err());
    assert!(serde_json::from_str::<SearchMethod>("\"not_a_method\"").is_err());
}
