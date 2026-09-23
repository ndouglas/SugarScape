//! Rule E (Chapter V, Appendix B): immune response and disease transmission,
//! plus the disease list, new agents' diseases, genome inheritance and
//! outbreaks. Nothing here runs, or draws random numbers, while disease is off.

use crate::agent::{Agent, AgentId, DiseaseId};
use crate::bits::Bits;
use crate::config::{DiseaseRule, URange};
use crate::rng::SimRng;
use crate::world::{Infection, World};
use rand::seq::SliceRandom;
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

/// Rule E for one agent's turn: immune response, then transmission.
pub(crate) fn act(world: &mut World, id: AgentId) {
    respond(world, id);
    transmit(world, id);
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

/// Appendix B's transmission: each von Neumann neighbor, in random order, is
/// offered one of the agent's diseases chosen uniformly (mutated in one random
/// bit with probability `disease_mutation`) and catches it unless it already
/// carries it or is immune.
pub(crate) fn transmit(world: &mut World, id: AgentId) {
    let me = world.agent(id).expect("live agent");
    if me.diseases.is_empty() {
        return;
    }
    let (pos, carried) = (me.pos, me.diseases.clone());
    let mutation = world.config.disease.disease_mutation;
    let mut neighbors = world.torus.neighbors(pos);
    neighbors.shuffle(&mut world.rng);
    for q in neighbors {
        let Some(other) = world.occupant(q) else {
            continue;
        };
        let mut disease = *carried.choose(&mut world.rng).expect("carries a disease");
        if mutation > 0.0 && world.rng.gen_bool(mutation) {
            let mut variant = world.diseases[disease as usize];
            let bit = world.rng.gen_range(0..variant.len());
            variant.flip(bit);
            disease = find_or_add(world, variant);
        }
        if infect(world, other, disease) {
            world.agent_mut(other).expect("occupant").infected_by = Some(id);
            world.events.infections.push(Infection {
                infector: Some(id),
                infected: other,
                disease,
            });
        }
    }
}

/// Gives `id` the disease unless it already carries it or it is a substring of
/// its immune string. Returns whether the agent was infected.
pub(crate) fn infect(world: &mut World, id: AgentId, disease: DiseaseId) -> bool {
    let d = world.diseases[disease as usize];
    let a = world.agent_mut(id).expect("live agent");
    if a.diseases.contains(&disease) || a.immune.contains(&d) {
        return false;
    }
    a.diseases.push(disease);
    true
}

/// The id of `bits` in the disease list, appending it if it is new.
pub(crate) fn find_or_add(world: &mut World, bits: Bits) -> DiseaseId {
    match world.diseases.iter().position(|d| *d == bits) {
        Some(i) => i as DiseaseId,
        None => {
            world.diseases.push(bits);
            (world.diseases.len() - 1) as DiseaseId
        }
    }
}

/// A brand-new random disease of the given `length`: redrawn up to 100 times
/// until it differs from every listed disease; if none does, the last
/// draw's existing id is reused.
pub(crate) fn new_random_with_length(world: &mut World, length: URange) -> DiseaseId {
    let mut draw = random_disease(length, &mut world.rng);
    for _ in 0..100 {
        if !world.diseases.contains(&draw) {
            break;
        }
        draw = random_disease(length, &mut world.rng);
    }
    find_or_add(world, draw)
}

/// A brand-new random disease drawn from `disease.length`.
pub(crate) fn new_random(world: &mut World) -> DiseaseId {
    new_random_with_length(world, world.config.disease.length)
}

/// Applies the outbreaks due at the tick about to run: each creates a new
/// disease — its length drawn from the outbreak's own `length` override if
/// given, else from `disease.length` — and offers it to `min(agents,
/// population)` random living agents.
pub(crate) fn outbreaks(world: &mut World) {
    let due: Vec<(u32, Option<URange>)> = world
        .config
        .disease
        .outbreaks
        .iter()
        .filter(|o| o.tick == world.tick)
        .map(|o| (o.agents, o.length))
        .collect();
    for (agents, length) in due {
        let length = length.unwrap_or(world.config.disease.length);
        let disease = new_random_with_length(world, length);
        let ids = world.agent_ids();
        let k = (agents as usize).min(ids.len());
        for i in rand::seq::index::sample(&mut world.rng, ids.len(), k).into_vec() {
            if infect(world, ids[i], disease) {
                world.events.infections.push(Infection {
                    infector: None,
                    infected: ids[i],
                    disease,
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Outbreak;
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

    #[test]
    fn transmission_skips_immune_and_already_infected_neighbors() {
        let mut w = sick_world();
        w.diseases = vec![b("11")];
        let zeros = "0".repeat(50);
        let me = patient(&mut w, 5, 5, &zeros, vec![0]);
        let open = patient(&mut w, 5, 4, &zeros, vec![]);
        let immune = patient(&mut w, 6, 5, &format!("11{}", "0".repeat(48)), vec![]);
        let carrier = patient(&mut w, 4, 5, &zeros, vec![0]);
        w.agent_mut(carrier).unwrap().infected_by = Some(999);
        transmit(&mut w, me);
        assert_eq!(w.agent(open).unwrap().diseases, vec![0]);
        assert_eq!(w.agent(open).unwrap().infected_by, Some(me));
        assert!(w.agent(immune).unwrap().diseases.is_empty());
        assert_eq!(
            w.agent(carrier).unwrap().infected_by,
            Some(999),
            "not reinfected"
        );
        assert_eq!(
            w.events().infections,
            vec![Infection {
                infector: Some(me),
                infected: open,
                disease: 0
            }]
        );
    }

    #[test]
    fn each_neighbor_is_offered_one_disease() {
        let mut w = sick_world();
        w.diseases = vec![b("1"), b("11")];
        let zeros = "0".repeat(50);
        let me = patient(&mut w, 5, 5, &zeros, vec![0, 1]);
        let n = patient(&mut w, 5, 4, &zeros, vec![]);
        transmit(&mut w, me);
        assert_eq!(w.agent(n).unwrap().diseases.len(), 1);
    }

    #[test]
    fn mutated_transmissions_append_distinct_variants() {
        let mut w = sick_world();
        w.config.disease.disease_mutation = 1.0;
        w.diseases = vec![b("1111")];
        let zeros = "0".repeat(50);
        let me = patient(&mut w, 5, 5, &zeros, vec![0]);
        let n = patient(&mut w, 5, 4, &zeros, vec![]);
        transmit(&mut w, me);
        assert_eq!(w.diseases.len(), 2, "a one-bit variant was appended");
        assert_eq!(w.diseases[1].hamming(&w.diseases[0]), 1);
        assert_eq!(w.agent(n).unwrap().diseases, vec![1]);
        let variant = w.diseases[1];
        assert_eq!(find_or_add(&mut w, variant), 1, "listed strings are reused");
        assert_eq!(w.diseases.len(), 2);
    }

    #[test]
    fn disease_spreads_to_neighbors_during_a_tick() {
        let mut w = sick_world();
        w.diseases = vec![b("1111111111")];
        let zeros = "0".repeat(50);
        let a = patient(&mut w, 5, 5, &zeros, vec![0]);
        let n = patient(&mut w, 5, 6, &zeros, vec![]);
        w.step();
        // Whatever the turn order, one flip cannot cure a 10-bit disease.
        assert_eq!(w.agent(n).unwrap().diseases, vec![0]);
        assert_eq!(w.agent(n).unwrap().infected_by, Some(a));
        assert_eq!(w.agent(a).unwrap().diseases, vec![0]);
    }

    #[test]
    fn outbreaks_infect_random_agents_with_a_new_disease_at_their_tick() {
        let mut w = sick_world();
        w.config.disease.length = URange::new(8, 8);
        w.config.disease.outbreaks = vec![Outbreak {
            tick: 2,
            agents: 3,
            length: None,
        }];
        // Five agents two sites apart (no neighbors), with empty immune strings
        // so no random disease can be resisted.
        let ids: Vec<AgentId> = (0..5)
            .map(|i| {
                let id = spawn(&mut w, i * 2, 0);
                w.agent_mut(id).unwrap().immune = Bits::default();
                id
            })
            .collect();
        w.step(); // tick 0
        w.step(); // tick 1
        assert!(w.diseases.is_empty());
        w.step(); // starts at tick 2: the outbreak fires
        assert_eq!(w.diseases.len(), 1);
        let sick = ids
            .iter()
            .filter(|&&id| w.agent(id).unwrap().diseases == vec![0])
            .count();
        assert_eq!(sick, 3);
        assert_eq!(w.events().infections.len(), 3);
        assert!(w
            .events()
            .infections
            .iter()
            .all(|i| i.infector.is_none() && i.disease == 0));
        w.step();
        assert_eq!(w.diseases.len(), 1, "each outbreak fires once");
    }

    #[test]
    fn outbreak_size_is_capped_by_the_population() {
        let mut w = sick_world();
        w.config.disease.outbreaks = vec![Outbreak {
            tick: 0,
            agents: 50,
            length: None,
        }];
        let a = spawn(&mut w, 0, 0);
        let c = spawn(&mut w, 5, 5);
        for id in [a, c] {
            w.agent_mut(id).unwrap().immune = Bits::default();
        }
        w.step();
        assert_eq!(w.events().infections.len(), 2);
    }

    #[test]
    fn outbreak_length_override_sets_the_new_disease_s_length() {
        let mut w = sick_world();
        w.config.disease.length = URange::new(1, 1);
        w.config.disease.outbreaks = vec![Outbreak {
            tick: 0,
            agents: 1,
            length: Some(URange::new(10, 10)),
        }];
        let id = spawn(&mut w, 0, 0);
        w.agent_mut(id).unwrap().immune = Bits::default();
        w.step();
        assert_eq!(w.diseases.len(), 1);
        assert_eq!(
            w.diseases[0].len(),
            10,
            "the override's length wins over disease.length"
        );
    }

    #[test]
    fn new_random_diseases_differ_from_the_list_when_possible() {
        let mut w = sick_world();
        w.config.disease.length = URange::new(1, 1);
        w.diseases = vec![b("0")];
        assert_eq!(new_random(&mut w), 1);
        assert_eq!(w.diseases[1], b("1"));
        assert!(
            new_random(&mut w) <= 1,
            "no distinct 1-bit string is left: reuse"
        );
        assert_eq!(w.diseases.len(), 2);
    }
}
