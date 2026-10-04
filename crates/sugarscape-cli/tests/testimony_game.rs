use std::process::Command;
#[test]
fn testimony_game_command_rejects_tuning_flags() {
    let out = Command::new(env!("CARGO_BIN_EXE_sugarscape"))
        .args(["deduction", "testimony-game", "--seed", "7"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2));
}
#[test]
fn testimony_game_command_is_complete_and_repeatable() {
    let run = || {
        Command::new(env!("CARGO_BIN_EXE_sugarscape"))
            .args(["deduction", "testimony-game"])
            .output()
            .unwrap()
    };
    let a = run();
    let b = run();
    assert!(a.status.success(), "{}", String::from_utf8_lossy(&a.stderr));
    assert_eq!(a.stdout, b.stdout);
    let r: serde_json::Value = serde_json::from_slice(&a.stdout).unwrap();
    assert_eq!(r["passed"], true);
    assert_eq!(r["runs"].as_array().unwrap().len(), 40);
    assert_eq!(r["frozen_evaluations"].as_array().unwrap().len(), 160);
    assert_eq!(r["runs"][0]["settings"]["evaluations"], 3164);
    assert_eq!(r["search_version"], "testimony-search-v1");
    assert_eq!(r["metadata"]["search_seeds"]["start_inclusive"], 0);
    assert_eq!(r["metadata"]["search_seeds"]["end_exclusive"], 20);
    assert_eq!(
        r["metadata"]["seed_derivation"]["version"],
        "testimony-search-seed-v1"
    );
    assert_eq!(r["metadata"]["seed_derivation"]["genetic_identity"], 1);
    assert_eq!(r["metadata"]["seed_derivation"]["random_identity"], 2);
    for (method, identity) in [("genetic", 1u64), ("random", 2u64)] {
        let runs: Vec<_> = r["runs"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|run| run["method"] == method)
            .collect();
        assert_eq!(runs.len(), 20);
        let seeds: Vec<_> = runs
            .iter()
            .map(|run| run["seed"].as_u64().unwrap())
            .collect();
        assert_eq!(seeds, (0..20).collect::<Vec<_>>());
        for run in runs {
            assert_eq!(run["seed_derivation"], "testimony-search-seed-v1");
            let expected = run["seed"]
                .as_u64()
                .unwrap()
                .wrapping_mul(6364136223846793005)
                .wrapping_add(identity.wrapping_mul(1442695040888963407));
            assert_eq!(run["derived_seed"].as_u64(), Some(expected));
        }
    }
}
