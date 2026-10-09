//! The simulation's single random stream.

use rand::SeedableRng;

pub type SimRng = rand_pcg::Pcg64Mcg;

pub fn seeded(seed: u64) -> SimRng {
    SimRng::seed_from_u64(seed)
}

/// Exact generator state, kept as JSON text so its u128 never crosses a JSON number.
pub fn state_json(rng: &SimRng) -> String {
    serde_json::to_string(rng).expect("serializable RNG state")
}
