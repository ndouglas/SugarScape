use sugarscape_core::deduction::testimony_game::{
    enumerate, search_ga, search_random, search_seed, Config, Probability, SearchMethod,
    SearchSettings, SEARCH_SEED_DERIVATION,
};

#[test]
fn testimony_game_search_records_frozen_protocol_for_both_controls() {
    let distribution = enumerate(&Config::standard(
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
    let expected = SearchSettings {
        population: 64,
        generations: 50,
        elites: 2,
        tournament: 3,
        mutation_numerator: 1,
        mutation_denominator: 4,
        evaluations: 3164,
    };
    for (search, method) in [
        (search_ga as fn(_, _) -> _, SearchMethod::Genetic),
        (search_random, SearchMethod::Random),
    ] {
        let run = search(&distribution, 7).unwrap();
        assert_eq!(run.settings, expected);
        assert_eq!(run.seed, 7);
        assert_eq!(run.method, method);
        assert_eq!(run.derived_seed, search_seed(7, method));
        assert_eq!(run.seed_derivation, SEARCH_SEED_DERIVATION);
        assert_eq!(run.evaluations, expected.evaluations);
    }
}

#[test]
fn testimony_game_random_seed_golden_values_use_wrapping_arithmetic() {
    assert_eq!(search_seed(0, SearchMethod::Random), 2885390081777926814);
    assert_eq!(search_seed(7, SearchMethod::Random), 10540855501286374617);
    assert_eq!(
        search_seed(u64::MAX, SearchMethod::Random),
        14967997931640685425
    );
}

#[test]
fn testimony_game_search_settings_reject_unknown_fields() {
    assert!(serde_json::from_value::<SearchSettings>(serde_json::json!({
        "population":64,"generations":50,"elites":2,"tournament":3,
        "mutation_numerator":1,"mutation_denominator":4,"evaluations":3164,
        "extra":true
    }))
    .is_err());
}
