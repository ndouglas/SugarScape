//! The simulation's single random stream.

use rand::SeedableRng;

pub type SimRng = rand_pcg::Pcg64Mcg;

pub fn seeded(seed: u64) -> SimRng {
    SimRng::seed_from_u64(seed)
}
