//! Agent culture rule K: for each neighbor, pick a random tag position; if
//! the neighbor disagrees there, flip the neighbor's tag to match the agent's.
//! Group membership (Blue/Red) is derived from tags (`Tags::tribe`).

use rand::seq::SliceRandom;
use rand::Rng;

use crate::agent::AgentId;
use crate::world::World;

pub(crate) fn act(world: &mut World, id: AgentId) {
    let me = world.agent(id).expect("live agent");
    let (pos, tags) = (me.pos, me.tags);
    let mut neighbors = world.torus.neighbors(pos);
    neighbors.shuffle(&mut world.rng);
    for q in neighbors {
        let Some(n) = world.occupant(q) else { continue };
        let i = world.rng.gen_range(0..tags.len());
        world
            .agent_mut(n)
            .expect("occupant")
            .tags
            .set(i, tags.get(i));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::Tags;
    use crate::testkit::*;

    #[test]
    fn flips_exactly_one_tag_of_each_disagreeing_neighbor() {
        let mut w = blank_world(10, 10);
        let me = spawn(&mut w, 5, 5); // all zeros
        let n1 = spawn(&mut w, 5, 4);
        let n2 = spawn(&mut w, 6, 5);
        for n in [n1, n2] {
            w.agent_mut(n).unwrap().tags = Tags::new(u64::MAX, 11);
        }
        act(&mut w, me);
        assert_eq!(w.agent(n1).unwrap().tags.ones(), 10);
        assert_eq!(w.agent(n2).unwrap().tags.ones(), 10);
        assert_eq!(
            w.agent(me).unwrap().tags.ones(),
            0,
            "the agent itself is unchanged"
        );
    }

    #[test]
    fn agreeing_neighbors_are_unchanged() {
        let mut w = blank_world(10, 10);
        let me = spawn(&mut w, 5, 5);
        let n = spawn(&mut w, 5, 6);
        act(&mut w, me);
        assert_eq!(w.agent(n).unwrap().tags.ones(), 0);
    }
}
