//! Slow checks that the model reproduces the book's headline results.
//! Run with `cargo test -p sugarscape-core --release --test book -- --ignored`.

use sugarscape_core::config::Config;
use sugarscape_core::presets;
use sugarscape_core::world::World;

fn run(config: Config, seed: u64, ticks: u32) -> World {
    let mut w = World::new(config, seed).unwrap();
    w.run(ticks);
    w
}

#[test]
#[ignore]
fn carrying_capacity_is_near_224() {
    // Chapter II: "although 400 agents begin the simulation, a carrying
    // capacity of approximately 224 is eventually reached."
    let mean = (1..=5)
        .map(|s| run(Config::default(), s, 500).population() as f64)
        .sum::<f64>()
        / 5.0;
    assert!((190.0..=260.0).contains(&mean), "mean population {mean}");
}

#[test]
#[ignore]
fn replacement_produces_a_skewed_wealth_distribution() {
    let config = presets::by_id("ii-5-wealth").unwrap().config;
    let w = run(config, 1, 500);
    let gini = w.stats.latest().unwrap().gini;
    assert!(gini > 0.45, "gini {gini}");
}

#[test]
#[ignore]
fn sexual_reproduction_sustains_a_population() {
    let config = presets::by_id("iii-2-sex").unwrap().config;
    let w = run(config, 1, 600);
    let pops = w.stats.series("population").unwrap();
    let late = &pops[300..];
    let (min, max) = late
        .iter()
        .fold((f64::MAX, 0.0f64), |(lo, hi), &p| (lo.min(p), hi.max(p)));
    assert!(min > 100.0, "population collapsed to {min}");
    assert!(max / min < 1.6, "population swings from {min} to {max}");
}
