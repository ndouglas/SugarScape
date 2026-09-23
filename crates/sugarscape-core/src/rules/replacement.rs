//! Agent replacement rule R_[a,b]: each agent that dies is replaced by a
//! random age-0 agent at a random empty site, with max age drawn from [a,b].
//! As in Chapter III's combat runs, the replacement joins the dead agent's
//! tribe.

use rand::seq::SliceRandom;

use crate::agent::{Agent, Tribe};
use crate::world::World;

pub(crate) fn apply(world: &mut World) {
    if !world.config.replacement.enabled {
        return;
    }
    let tribes: Vec<Tribe> = world.events.deaths.iter().map(|d| d.tribe).collect();
    for tribe in tribes {
        let empty = world.empty_sites();
        let Some(&pos) = empty.choose(&mut world.rng) else {
            return;
        };
        let mut agent = Agent::random(&world.config, pos, world.tick, &mut world.rng);
        agent.tags = agent.tags.forced_to(tribe);
        crate::rules::disease::endow(world, &mut agent);
        world.insert_agent(agent).expect("chosen site is empty");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testkit::*;

    fn replacing_world() -> World {
        let mut w = blank_world(5, 5);
        w.config.lifespan.enabled = true;
        w.config.replacement.enabled = true;
        w
    }

    #[test]
    fn each_death_is_replaced_by_a_fresh_agent_of_the_same_tribe() {
        let mut w = replacing_world();
        let id = spawn(&mut w, 2, 2);
        w.agent_mut(id).unwrap().metabolism = 50;
        w.step();
        assert!(w.agent(id).is_none());
        assert_eq!(w.population(), 1);
        let newcomer = w.agents().next().unwrap();
        assert_ne!(newcomer.id, id);
        assert_eq!(newcomer.tribe(), Tribe::Blue);
        assert_eq!(newcomer.age, 1, "born this tick, then aged with everyone");
        assert!((60..=100).contains(&newcomer.max_age));
    }

    #[test]
    fn no_replacement_when_disabled() {
        let mut w = blank_world(5, 5);
        let id = spawn(&mut w, 2, 2);
        w.agent_mut(id).unwrap().metabolism = 50;
        w.step();
        assert_eq!(w.population(), 0);
    }

    #[test]
    fn replacements_get_immune_systems_when_disease_is_on() {
        let mut w = replacing_world();
        w.config.disease.enabled = true;
        w.diseases = vec![crate::bits::Bits::parse("1111111111").unwrap()];
        let id = spawn(&mut w, 2, 2);
        w.agent_mut(id).unwrap().metabolism = 50;
        w.step();
        let newcomer = w.agents().next().unwrap();
        assert_eq!(newcomer.immune.len(), 50);
        assert_eq!(newcomer.immune, newcomer.immune_genome);
        assert!(newcomer.diseases.len() <= 1);
        if let Some(&d) = newcomer.diseases.first() {
            assert!(!newcomer.immune.contains(&w.diseases[d as usize]));
        }
    }
}
