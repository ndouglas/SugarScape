//! Runs a config over seeds on a thread pool.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;

use sugarscape_core::config::Config;
use sugarscape_core::presets;
use sugarscape_core::world::World;

pub fn preset(id: &str) -> Config {
    presets::by_id(id)
        .unwrap_or_else(|| panic!("no preset {id}"))
        .config
}

/// Builds a world per seed and hands it to `f`, which steps and measures it.
/// Results come back in seed order.
pub fn each_seed<T: Send>(
    config: &Config,
    seeds: &[u64],
    f: impl Fn(World) -> T + Sync,
) -> Vec<T> {
    let next = AtomicUsize::new(0);
    let slots: Vec<Mutex<Option<T>>> = seeds.iter().map(|_| Mutex::new(None)).collect();
    let threads = std::thread::available_parallelism()
        .map_or(4, |n| n.get())
        .min(seeds.len().max(1));
    std::thread::scope(|s| {
        for _ in 0..threads {
            s.spawn(|| loop {
                let i = next.fetch_add(1, Ordering::Relaxed);
                if i >= seeds.len() {
                    break;
                }
                let w = World::new(config.clone(), seeds[i]).expect("a valid config");
                *slots[i].lock().unwrap() = Some(f(w));
            });
        }
    });
    slots
        .into_iter()
        .map(|m| m.into_inner().unwrap().expect("every seed ran"))
        .collect()
}

/// Runs `ticks` ticks per seed, then measures with `f`.
pub fn after<T: Send>(
    config: &Config,
    seeds: &[u64],
    ticks: u32,
    f: impl Fn(&World) -> T + Sync,
) -> Vec<T> {
    each_seed(config, seeds, |mut w| {
        w.run(ticks);
        f(&w)
    })
}

/// A statistics series; index t is tick t (index 0 is the initial state).
pub fn series(w: &World, name: &str) -> Vec<f64> {
    w.stats
        .series(name)
        .unwrap_or_else(|| panic!("no series {name}"))
}

/// Mean of `s[from..=to]`.
pub fn window_mean(s: &[f64], from: usize, to: usize) -> f64 {
    let w = &s[from..=to];
    w.iter().sum::<f64>() / w.len() as f64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seeds_run_in_order_and_series_index_is_the_tick() {
        let c = preset("ii-2-unit");
        let pops = after(&c, &[1, 2, 3], 5, |w| (w.tick, series(w, "population")));
        assert_eq!(pops.len(), 3);
        for (tick, s) in &pops {
            assert_eq!(*tick, 5);
            assert_eq!(s.len(), 6);
            assert_eq!(s[0], 400.0);
        }
        let again = after(&c, &[2], 5, |w| series(w, "population"));
        assert_eq!(again[0], pops[1].1, "seed 2 is deterministic and in slot 1");
        assert_eq!(window_mean(&[1.0, 2.0, 3.0, 4.0], 1, 2), 2.5);
    }
}
