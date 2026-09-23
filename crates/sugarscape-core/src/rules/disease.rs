//! Rule E (Chapter V, Appendix B): immune response and disease transmission,
//! plus the disease list, new agents' diseases, genome inheritance and
//! outbreaks. Nothing here runs, or draws random numbers, while disease is off.

use crate::agent::{Agent, AgentId, DiseaseId};
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

/// Rule E for one agent's turn.
pub(crate) fn act(world: &mut World, id: AgentId) {
    respond(world, id);
}

/// Appendix B's immune response: for each carried disease, up to
/// `flips_per_tick` single-bit steps toward it (`Bits::learn`); then every
/// carried disease the trained string now contains is cured.
pub(crate) fn respond(world: &mut World, id: AgentId) {
    let flips = world.config.disease.flips_per_tick;
    let carried: Vec<Bits> = world
        .agent(id)
        .expect("live agent")
        .diseases
        .iter()
        .map(|&d| world.diseases[d as usize])
        .collect();
    let a = world.agent_mut(id).expect("live agent");
    for d in &carried {
        for _ in 0..flips {
            if !a.immune.learn(d) {
                break;
            }
        }
    }
    cure_immune(world, id);
}

/// Drops every carried disease that is a substring of the agent's immune string.
pub(crate) fn cure_immune(world: &mut World, id: AgentId) {
    let a = world.agent(id).expect("live agent");
    let kept: Vec<DiseaseId> = a
        .diseases
        .iter()
        .copied()
        .filter(|&d| !a.immune.contains(&world.diseases[d as usize]))
        .collect();
    world.agent_mut(id).expect("live agent").diseases = kept;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rng::seeded;
    use crate::testkit::*;

    fn b(s: &str) -> Bits {
        Bits::parse(s).unwrap()
    }

    fn sick_world() -> World {
        let mut w = blank_world(10, 10);
        w.config.disease.enabled = true;
        w
    }

    /// An agent at (x, y) with the given immune string, carrying `diseases`.
    fn patient(w: &mut World, x: u32, y: u32, immune: &str, diseases: Vec<DiseaseId>) -> AgentId {
        let id = spawn(w, x, y);
        let a = w.agent_mut(id).unwrap();
        a.immune = b(immune);
        a.diseases = diseases;
        id
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

    #[test]
    fn book_example_is_learned_in_one_tick() {
        // Appendix B's worked example, through a whole tick: the agent stays
        // put (nothing to gather), burns its fee, then its immune system flips
        // one bit and the disease is gone.
        let mut w = sick_world();
        w.diseases = vec![b("10011")];
        let id = patient(&mut w, 5, 5, "1011101001", vec![0]);
        w.step();
        let a = w.agent(id).unwrap();
        assert_eq!(a.immune.to_bit_string(), "1001101001");
        assert!(a.diseases.is_empty(), "cured");
        assert_eq!(a.sugar, 9.0, "metabolism 0 plus a fee of 1 for one disease");
    }

    #[test]
    fn a_disease_already_in_the_immune_string_is_cured_without_flips() {
        let mut w = sick_world();
        w.diseases = vec![b("11")];
        let id = patient(&mut w, 5, 5, "0000011000", vec![0]);
        respond(&mut w, id);
        let a = w.agent(id).unwrap();
        assert_eq!(a.immune.to_bit_string(), "0000011000");
        assert!(a.diseases.is_empty());
    }

    #[test]
    fn one_flip_per_tick_unless_medicine_adds_more() {
        let mut w = sick_world();
        w.diseases = vec![b("1111")];
        let id = patient(&mut w, 5, 5, "0000000000", vec![0]);
        respond(&mut w, id);
        assert_eq!(w.agent(id).unwrap().immune.to_bit_string(), "1000000000");
        respond(&mut w, id);
        assert_eq!(w.agent(id).unwrap().immune.to_bit_string(), "1100000000");
        assert_eq!(w.agent(id).unwrap().diseases, vec![0], "still sick");
        w.config.disease.flips_per_tick = 2;
        respond(&mut w, id);
        let a = w.agent(id).unwrap();
        assert_eq!(a.immune.to_bit_string(), "1111000000");
        assert!(a.diseases.is_empty(), "two flips finished the job");
    }

    #[test]
    fn every_disease_the_trained_string_now_contains_is_cured() {
        // Training on 11 flips position 0, which also makes 1 a substring.
        let mut w = sick_world();
        w.diseases = vec![b("11"), b("1")];
        let id = patient(&mut w, 5, 5, "0000000000", vec![0, 1]);
        respond(&mut w, id);
        let a = w.agent(id).unwrap();
        assert_eq!(a.immune.to_bit_string(), "1000000000");
        assert_eq!(a.diseases, vec![0]);
    }

    #[test]
    fn nothing_happens_while_disease_is_off() {
        let mut w = blank_world(10, 10);
        w.diseases = vec![b("10011")];
        let id = patient(&mut w, 5, 5, "1011101001", vec![0]);
        w.step();
        assert_eq!(w.agent(id).unwrap().immune.to_bit_string(), "1011101001");
    }
}
