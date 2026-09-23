//! Rule E (Chapter V, Appendix B): immune response and disease transmission,
//! plus the disease list, new agents' diseases, genome inheritance and
//! outbreaks. Nothing here runs, or draws random numbers, while disease is off.

use crate::agent::{Agent, DiseaseId};
use crate::bits::Bits;
use crate::config::{DiseaseRule, URange};
use crate::rng::SimRng;
use crate::world::World;

/// A random disease with its length drawn from `length`.
pub(crate) fn random_disease(length: URange, rng: &mut SimRng) -> Bits {
    let len = length.sample(rng);
    Bits::random(len, rng)
}

/// The initial master list: `count` independent random diseases.
pub(crate) fn initial_list(rule: &DiseaseRule, rng: &mut SimRng) -> Vec<Bits> {
    (0..rule.count)
        .map(|_| random_disease(rule.length, rng))
        .collect()
}

/// Gives a new (not newborn) agent `initial` distinct random diseases from the
/// list, skipping any it is already immune to.
pub(crate) fn endow(world: &mut World, agent: &mut Agent) {
    if !world.config.disease.enabled {
        return;
    }
    let n = world.diseases.len();
    let k = (world.config.disease.initial as usize).min(n);
    for i in rand::seq::index::sample(&mut world.rng, n, k).into_vec() {
        if !agent.immune.contains(&world.diseases[i]) {
            agent.diseases.push(i as DiseaseId);
        }
    }
}
