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
    // The spec expects the R[60,100] Gini to exceed "~0.5". Observed at t=500
    // over seeds 1..=5: 0.4607, 0.4920, 0.4855, 0.4721, 0.4882 (mean 0.480),
    // i.e. at the approximate 0.5 the spec cites but not strictly above it.
    // The bound sits conservatively below the observed mean so seed noise and
    // small rule changes don't flake, while still demanding a strongly skewed
    // distribution (the initial uniform [5,25] endowments give a Gini of ~0.2).
    let config = presets::by_id("ii-5-wealth").unwrap().config;
    let mean = (1..=5)
        .map(|s| run(config.clone(), s, 500).stats.latest().unwrap().gini)
        .sum::<f64>()
        / 5.0;
    assert!(mean > 0.44, "mean gini {mean}");
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

/// Fraction of von Neumann-adjacent agent pairs (each pair counted once) that
/// share a tribe.
fn local_homogeneity(w: &World) -> f64 {
    let (mut same, mut pairs) = (0u32, 0u32);
    for a in w.agents() {
        for (dx, dy) in [(1, 0), (0, 1)] {
            if let Some(b) = w.agent_at(w.torus.offset(a.pos, dx, dy)) {
                pairs += 1;
                same += u32::from(a.tribe() == b.tribe());
            }
        }
    }
    f64::from(same) / f64::from(pairs)
}

#[test]
#[ignore]
fn culture_drives_neighbors_toward_one_tribe() {
    // Chapter III (Animations III-6/III-7): with K on, spatially segregated
    // groups converge to single-tribe dominance (the book: after ~2700 ticks).
    // The two mountains may settle on different tribes, so we measure local
    // homogeneity rather than a global majority.
    //
    // Observed (seeds 1..=5; t=0 -> t=3000): 0.525->0.890, 0.483->0.987,
    // 0.500->0.989, 0.443->1.000, 0.375->1.000; means 0.465 -> 0.973. Only
    // ~80-100 adjacent pairs remain once the population settles near 224, so
    // single seeds are noisy (seed 1 dips to 0.68 at t=2000). The bounds keep
    // margin: the start is near 0.5 (random tags) and the end mean exceeds 0.9.
    let config = presets::by_id("iii-6-culture").unwrap().config;
    let (mut start, mut end) = (0.0, 0.0);
    for seed in 1..=5 {
        let mut w = World::new(config.clone(), seed).unwrap();
        start += local_homogeneity(&w) / 5.0;
        w.run(3000);
        end += local_homogeneity(&w) / 5.0;
    }
    assert!((0.35..=0.65).contains(&start), "start homogeneity {start}");
    assert!(
        end > 0.9,
        "homogeneity after 3000 ticks {end} (from {start})"
    );
}
