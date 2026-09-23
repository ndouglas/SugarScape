//! Rule E (Chapter V, Appendix B): immune response and disease transmission,
//! plus the disease list, new agents' diseases, genome inheritance and
//! outbreaks. Nothing here runs, or draws random numbers, while disease is off.

use crate::agent::{Agent, DiseaseId};
use crate::bits::Bits;
use crate::config::{DiseaseRule, URange};
use crate::rng::SimRng;
use crate::world::World;
use rand::Rng;

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

/// A child's immune genome: the parents' shared bits, a random parent's where
/// they differ (as culture tags), then each bit flipped with probability
/// `mutation`.
pub(crate) fn inherit_genome(a: &Bits, b: &Bits, mutation: f64, rng: &mut SimRng) -> Bits {
    assert_eq!(a.len(), b.len(), "parents' genomes differ in length");
    let mut genome = *a;
    for i in 0..genome.len() {
        if a.get(i) != b.get(i) && rng.gen_bool(0.5) {
            genome.set(i, b.get(i));
        }
    }
    if mutation > 0.0 {
        for i in 0..genome.len() {
            if rng.gen_bool(mutation) {
                genome.flip(i);
            }
        }
    }
    genome
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rng::seeded;

    fn b(s: &str) -> Bits {
        Bits::parse(s).unwrap()
    }

    #[test]
    fn child_genomes_keep_what_the_parents_share() {
        let mut rng = seeded(3);
        let (a, p) = (b("0000011111"), b("0101010101"));
        let mut saw_both = [false; 2];
        for _ in 0..40 {
            let g = inherit_genome(&a, &p, 0.0, &mut rng);
            for i in 0..10 {
                if a.get(i) == p.get(i) {
                    assert_eq!(g.get(i), a.get(i), "position {i}");
                }
            }
            saw_both[usize::from(g.get(1))] = true;
        }
        assert_eq!(
            saw_both,
            [true, true],
            "differing positions come from either parent"
        );
    }

    #[test]
    fn genome_mutation_flips_bits() {
        let a = b("0011");
        assert_eq!(inherit_genome(&a, &a, 0.0, &mut seeded(1)), a);
        assert_eq!(inherit_genome(&a, &a, 1.0, &mut seeded(1)), b("1100"));
    }
}
