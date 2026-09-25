//! Runs a config over seeds on a thread pool.

use std::panic::AssertUnwindSafe;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;

use sugarscape_core::config::Config;
use sugarscape_core::model::{ModelConfig, ModelWorld};
use sugarscape_core::presets;
use sugarscape_core::world::World;

pub fn preset(id: &str) -> Config {
    presets::by_id(id)
        .unwrap_or_else(|| panic!("no preset {id}"))
        .config
}

/// A preset of any model (`presets::find`).
pub fn model_preset(id: &str) -> ModelConfig {
    presets::find(id)
        .unwrap_or_else(|| panic!("no preset {id}"))
        .config
}

/// Builds a world per seed and hands it to `f`, which steps and measures it.
/// Results come back in seed order. A panic in `f` is re-raised on the
/// calling thread with its own message (a scoped thread's panic would
/// otherwise surface only as "a scoped thread panicked").
pub fn each_seed<T: Send>(config: &Config, seeds: &[u64], f: impl Fn(World) -> T + Sync) -> Vec<T> {
    on_threads(seeds, |seed| {
        f(World::new(config.clone(), seed).expect("a valid config"))
    })
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

/// Like `after`, for a world of any model: builds it from `config` per seed,
/// runs `ticks` ticks (fewer if it finishes) and measures it with `f`.
pub fn model_after<T: Send>(
    config: &ModelConfig,
    seeds: &[u64],
    ticks: u32,
    f: impl Fn(&ModelWorld) -> T + Sync,
) -> Vec<T> {
    on_threads(seeds, |seed| {
        let mut w = ModelWorld::new(config.clone(), seed).expect("a valid config");
        w.model_mut().run(ticks);
        f(&w)
    })
}

/// `f(seed)` for every seed on a thread pool, in seed order (see
/// `each_seed` for panics).
fn on_threads<T: Send>(seeds: &[u64], f: impl Fn(u64) -> T + Sync) -> Vec<T> {
    let next = AtomicUsize::new(0);
    type Slot<T> = Mutex<Option<std::thread::Result<T>>>;
    let slots: Vec<Slot<T>> = seeds.iter().map(|_| Mutex::new(None)).collect();
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
                let result = std::panic::catch_unwind(AssertUnwindSafe(|| f(seeds[i])));
                *slots[i].lock().unwrap() = Some(result);
            });
        }
    });
    slots
        .into_iter()
        .map(|m| match m.into_inner().unwrap().expect("every seed ran") {
            Ok(v) => v,
            Err(payload) => std::panic::resume_unwind(payload),
        })
        .collect()
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

    #[test]
    fn worlds_of_any_model_run_per_seed_and_stop_when_finished() {
        let c = model_preset("cv-run-7-cleansing");
        let ends = model_after(&c, &[1, 2], 5000, |w| {
            (w.model().tick(), w.model().finished())
        });
        assert!(
            ends.iter().all(|&(tick, done)| done && tick < 5000),
            "{ends:?}"
        );
        assert_ne!(ends[0], ends[1], "each seed its own world");
    }

    #[test]
    fn a_panic_in_a_seed_keeps_its_message() {
        let c = preset("ii-2-unit");
        let err = std::panic::catch_unwind(|| {
            each_seed(&c, &[1, 2, 3], |w| {
                if w.population() > 0 {
                    panic!("empty population on seed")
                }
            })
        })
        .unwrap_err();
        let msg = err
            .downcast_ref::<String>()
            .cloned()
            .or_else(|| err.downcast_ref::<&str>().map(|s| s.to_string()))
            .unwrap_or_default();
        assert!(msg.contains("empty population on seed"), "{msg:?}");
    }
}
