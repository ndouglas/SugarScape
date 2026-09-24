//! Agent sex rule S (Chapter III). For each neighbor, in random order: if the
//! neighbor is fertile and of the opposite sex and either partner has an empty
//! neighboring site, a child is born there.
//!
//! Child: endowment = half of each parent's initial endowment (deducted from
//! the parents); vision, metabolism, max age and fertility onset each come from
//! a random parent (Table III-1); fertility end is drawn from the child's
//! sex-specific range (the ranges differ by sex, so it cannot be inherited
//! across sexes); each culture tag is the parents' shared value or, where they
//! differ, a random parent's.
//!
//! With disease (E) on, the child's immune genome crosses over like the tags
//! (then mutates per `disease.genome_mutation`); it starts untrained and
//! healthy.

use rand::seq::SliceRandom;
use rand::Rng;

use crate::agent::{in_slot_0, Agent, AgentId, Sex};
use crate::bits::Bits;
use crate::config::MAX_GOODS;
use crate::geometry::Pos;
use crate::social::Social;
use crate::world::World;

pub(crate) fn act(world: &mut World, id: AgentId) {
    let pos = world.agent(id).expect("live agent").pos;
    let mut neighbors = world.torus.neighbors(pos);
    neighbors.shuffle(&mut world.rng);
    for q in neighbors {
        let me = world.agent(id).expect("live agent");
        if !me.is_fertile() {
            return;
        }
        let Some(mate) = world.agent_at(q) else {
            continue;
        };
        if mate.sex == me.sex || !mate.is_fertile() {
            continue;
        }
        let mate_id = mate.id;
        let mut cradles: Vec<Pos> = world
            .torus
            .neighbors(pos)
            .into_iter()
            .chain(world.torus.neighbors(q))
            .filter(|&p| !world.is_occupied(p))
            .collect();
        cradles.sort();
        cradles.dedup();
        let Some(&cradle) = cradles.choose(&mut world.rng) else {
            continue;
        };
        birth(world, id, mate_id, cradle);
    }
}

fn birth(world: &mut World, a_id: AgentId, b_id: AgentId, cradle: Pos) {
    let n = world.config.goods.len();
    let a = world.agent(a_id).expect("parent").clone();
    let b = world.agent(b_id).expect("parent").clone();
    let rng = &mut world.rng;
    let pick = |rng: &mut crate::rng::SimRng, x: u32, y: u32| if rng.gen_bool(0.5) { x } else { y };
    let sex = if rng.gen_bool(0.5) {
        Sex::Female
    } else {
        Sex::Male
    };
    let mut tags = a.tags;
    for i in 0..tags.len() {
        if a.tags.get(i) != b.tags.get(i) && rng.gen_bool(0.5) {
            tags.set(i, b.tags.get(i));
        }
    }
    let half = |x: &Agent| -> [f64; MAX_GOODS] {
        std::array::from_fn(|i| if i < n { x.initial[i] / 2.0 } else { 0.0 })
    };
    let (from_a, from_b) = (half(&a), half(&b));
    let endowment: [f64; MAX_GOODS] = std::array::from_fn(|i| from_a[i] + from_b[i]);
    let mut child = Agent {
        id: 0,
        pos: cradle,
        vision: pick(rng, a.vision, b.vision),
        metabolism: in_slot_0(pick(rng, a.metabolism[0], b.metabolism[0])),
        holdings: endowment,
        initial: endowment,
        age: 0,
        max_age: pick(rng, a.max_age, b.max_age),
        sex,
        fertility_onset: pick(rng, a.fertility_onset, b.fertility_onset),
        fertility_end: world.config.sex.end_for(sex).sample(rng),
        tags,
        parents: Some([a_id, b_id]),
        children: Vec::new(),
        born: world.tick,
        foresight: 0,
        income: [0.0; MAX_GOODS],
        immune_genome: Bits::default(),
        immune: Bits::default(),
        diseases: Vec::new(),
        infected_by: None,
        social: Social::default(),
    };
    // Goods 1..n pick where Chapter IV picked spice's metabolism.
    for (i, m) in child.metabolism.iter_mut().enumerate().take(n).skip(1) {
        *m = pick(rng, a.metabolism[i], b.metabolism[i]);
    }
    if world.config.foresight.enabled {
        child.foresight = pick(rng, a.foresight, b.foresight);
    }
    if world.config.disease.enabled {
        let genome = crate::rules::disease::inherit_genome(
            &a.immune_genome,
            &b.immune_genome,
            world.config.disease.genome_mutation,
            rng,
        );
        child.immune_genome = genome;
        child.immune = genome;
    }
    let pa = world.agent_mut(a_id).expect("parent");
    for (have, give) in pa.holdings.iter_mut().zip(&from_a).take(n) {
        *have -= give;
    }
    let pb = world.agent_mut(b_id).expect("parent");
    for (have, give) in pb.holdings.iter_mut().zip(&from_b).take(n) {
        *have -= give;
    }
    let child_id = world.insert_agent(child).expect("cradle was empty");
    world
        .agent_mut(a_id)
        .expect("parent")
        .children
        .push(child_id);
    world
        .agent_mut(b_id)
        .expect("parent")
        .children
        .push(child_id);
    world.events.births += 1;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::Tags;
    use crate::testkit::*;

    fn couple(w: &mut World) -> (AgentId, AgentId) {
        let mom = spawn(w, 2, 2);
        let dad = spawn(w, 3, 2);
        {
            let d = w.agent_mut(dad).unwrap();
            d.sex = Sex::Male;
            d.vision = 4;
            d.metabolism[0] = 3;
            d.initial[0] = 6.0;
            d.holdings[0] = 6.0;
            d.tags = Tags::new(u64::MAX, d.tags.len());
        }
        (mom, dad)
    }

    #[test]
    fn fertile_opposite_sex_neighbors_have_a_child() {
        let mut w = blank_world(10, 10);
        let (mom, dad) = couple(&mut w);
        act(&mut w, mom);
        assert_eq!(w.population(), 3);
        assert_eq!(w.events().births, 1);
        let child = w.agents().find(|a| a.parents.is_some()).unwrap().clone();
        assert_eq!(child.parents, Some([mom, dad]));
        assert_eq!(child.initial[0], 5.0 + 3.0);
        assert_eq!(child.holdings[0], 8.0);
        assert_eq!(child.age, 0);
        assert!([1, 4].contains(&child.vision));
        assert!([0, 3].contains(&child.metabolism[0]));
        assert_eq!(w.agent(mom).unwrap().holdings[0], 5.0);
        assert_eq!(w.agent(dad).unwrap().holdings[0], 3.0);
        assert_eq!(w.agent(mom).unwrap().children, vec![child.id]);
        assert_eq!(w.agent(dad).unwrap().children, vec![child.id]);
        let near = |p: Pos| {
            w.torus.neighbors(Pos::new(2, 2)).contains(&p)
                || w.torus.neighbors(Pos::new(3, 2)).contains(&p)
        };
        assert!(near(child.pos));
    }

    #[test]
    fn same_sex_neighbors_do_not_reproduce() {
        let mut w = blank_world(10, 10);
        let (mom, dad) = couple(&mut w);
        w.agent_mut(dad).unwrap().sex = Sex::Female;
        act(&mut w, mom);
        assert_eq!(w.population(), 2);
    }

    #[test]
    fn infertile_agents_do_not_reproduce() {
        let mut w = blank_world(10, 10);
        let (mom, dad) = couple(&mut w);
        w.agent_mut(dad).unwrap().age = 5; // too young
        act(&mut w, mom);
        assert_eq!(w.population(), 2);
        w.agent_mut(dad).unwrap().age = 20;
        w.agent_mut(dad).unwrap().holdings[0] = 5.0; // below its endowment of 6
        act(&mut w, mom);
        assert_eq!(w.population(), 2);
    }

    #[test]
    fn no_child_without_an_empty_neighboring_site() {
        // On a 2×1-ish crowded patch: fill every neighbor of both parents.
        let mut w = blank_world(10, 10);
        let (mom, dad) = couple(&mut w);
        for p in [(2, 1), (2, 3), (1, 2), (3, 1), (3, 3), (4, 2)] {
            spawn(&mut w, p.0, p.1);
        }
        let before = w.population();
        act(&mut w, mom);
        assert_eq!(w.population(), before);
        let _ = dad;
    }

    #[test]
    fn child_tags_agree_where_parents_agree() {
        let mut w = blank_world(10, 10);
        let (mom, dad) = couple(&mut w);
        w.agent_mut(mom).unwrap().tags = Tags::new(0b101, 11);
        w.agent_mut(dad).unwrap().tags = Tags::new(0b111, 11);
        act(&mut w, mom);
        let child = w.agents().find(|a| a.parents.is_some()).unwrap();
        assert!(child.tags.get(0) && child.tags.get(2));
        assert!((3..11).all(|i| !child.tags.get(i)));
    }

    #[test]
    fn with_spice_fertility_needs_both_goods_and_children_get_both() {
        let mut w = blank_world(10, 10);
        add_goods(&mut w.config, 2);
        let (mom, dad) = couple(&mut w);
        for id in [mom, dad] {
            let a = w.agent_mut(id).unwrap();
            a.holdings[1] = 8.0;
            a.initial[1] = 8.0;
            a.metabolism[1] = if id == mom { 2 } else { 3 };
        }
        w.agent_mut(dad).unwrap().holdings[1] = 7.0; // below its spice endowment
        act(&mut w, mom);
        assert_eq!(w.population(), 2, "infertile without enough spice");
        w.agent_mut(dad).unwrap().holdings[1] = 8.0;
        act(&mut w, mom);
        let child = w.agents().find(|a| a.parents.is_some()).unwrap().clone();
        assert_eq!((child.holdings[1], child.initial[1]), (8.0, 8.0));
        assert!([2, 3].contains(&child.metabolism[1]));
        assert_eq!(w.agent(mom).unwrap().holdings[1], 4.0);
    }

    #[test]
    fn children_inherit_an_immune_genome_and_start_healthy() {
        use crate::bits::Bits;
        let mut w = blank_world(10, 10);
        w.config.disease.enabled = true;
        let (mom, dad) = couple(&mut w);
        let genome = Bits::parse(&"10".repeat(25)).unwrap();
        for id in [mom, dad] {
            let a = w.agent_mut(id).unwrap();
            a.immune_genome = genome;
            a.immune = Bits::new(0, 50); // trained phenotypes are not inherited
            a.diseases = vec![0];
        }
        act(&mut w, mom);
        let child = w.agents().find(|a| a.parents.is_some()).unwrap();
        assert_eq!(
            child.immune_genome, genome,
            "parents agree everywhere; no mutation"
        );
        assert_eq!(child.immune, genome, "the phenotype starts untrained");
        assert!(child.diseases.is_empty() && child.infected_by.is_none());
    }

    #[test]
    fn with_three_goods_children_get_every_good() {
        let mut w = blank_world(10, 10);
        add_goods(&mut w.config, 3);
        let (mom, dad) = couple(&mut w);
        for (id, amount, m) in [(mom, 6.0, 2), (dad, 4.0, 5)] {
            let a = w.agent_mut(id).unwrap();
            a.holdings[2] = amount;
            a.initial[2] = amount;
            a.metabolism[2] = m;
        }
        act(&mut w, mom);
        let child = w.agents().find(|a| a.parents.is_some()).unwrap().clone();
        assert_eq!((child.holdings[2], child.initial[2]), (5.0, 5.0));
        assert!([2, 5].contains(&child.metabolism[2]));
        assert_eq!(w.agent(mom).unwrap().holdings[2], 3.0);
        assert_eq!(w.agent(dad).unwrap().holdings[2], 2.0);
    }
}
