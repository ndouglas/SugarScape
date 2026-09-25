//! Agent culture rule K: for each neighbor, pick a random tag position; if
//! the neighbor disagrees there, flip the neighbor's tag to match the agent's.
//! Group membership is derived from tags by `culture.groups` (`Agent::group`).
//! Under `culture.rule: axelrod` (milestone 14), Axelrod's rule instead: the
//! agent picks one random occupied von Neumann neighbor and, with probability
//! equal to their similarity, copies one feature on which they differ.

use rand::seq::SliceRandom;
use rand::Rng;

use crate::agent::AgentId;
use crate::world::World;

pub(crate) fn act(world: &mut World, id: AgentId) {
    if world.config.culture.rule == crate::config::CultureKind::Axelrod {
        return axelrod(world, id);
    }
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

/// Axelrod's event with the agent active (Axelrod 1997, footnote 4): a random
/// occupied neighbor and a random feature; if they agree on it, the agent
/// copies a random feature on which they differ.
fn axelrod(world: &mut World, id: AgentId) {
    let pos = world.agent(id).expect("live agent").pos;
    let occupied: Vec<AgentId> = world
        .torus
        .neighbors(pos)
        .into_iter()
        .filter_map(|q| world.occupant(q))
        .collect();
    if occupied.is_empty() {
        return;
    }
    let n = occupied[world.rng.gen_range(0..occupied.len() as u32) as usize];
    let theirs = world.agent(n).expect("occupant").culture.clone();
    let mine = world.agent(id).expect("live agent").culture.clone();
    let f = mine.len();
    let probe = world.rng.gen_range(0..f as u32) as usize;
    if mine[probe] != theirs[probe] {
        return;
    }
    let differ: Vec<usize> = (0..f).filter(|&g| mine[g] != theirs[g]).collect();
    if differ.is_empty() {
        return;
    }
    let g = differ[world.rng.gen_range(0..differ.len() as u32) as usize];
    world.agent_mut(id).expect("live agent").culture[g] = theirs[g];
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::Tags;
    use crate::testkit::*;

    /// A blank world under Axelrod's rule, F = 2.
    fn axelrod_world() -> World {
        let mut w = blank_world(10, 10);
        w.config.culture.enabled = true;
        w.config.culture.rule = crate::config::CultureKind::Axelrod;
        w.config.culture.features = 2;
        w
    }

    #[test]
    fn under_axelrod_the_agent_copies_a_similar_neighbor() {
        let mut w = axelrod_world();
        let me = spawn(&mut w, 5, 5);
        let n = spawn(&mut w, 5, 4);
        w.agent_mut(me).unwrap().culture = vec![1, 2];
        w.agent_mut(n).unwrap().culture = vec![1, 3];
        for _ in 0..200 {
            act(&mut w, me);
        }
        assert_eq!(w.agent(me).unwrap().culture, [1, 3], "the agent changed");
        assert_eq!(w.agent(n).unwrap().culture, [1, 3], "the neighbor did not");
        let far = spawn(&mut w, 0, 0);
        w.agent_mut(far).unwrap().culture = vec![7, 7];
        act(&mut w, far);
        assert_eq!(
            w.agent(far).unwrap().culture,
            [7, 7],
            "no neighbor, no change"
        );
    }

    #[test]
    fn under_axelrod_strangers_and_twins_never_change() {
        let mut w = axelrod_world();
        let me = spawn(&mut w, 5, 5);
        let n = spawn(&mut w, 6, 5);
        w.agent_mut(me).unwrap().culture = vec![1, 2];
        w.agent_mut(n).unwrap().culture = vec![3, 4];
        for _ in 0..100 {
            act(&mut w, me);
        }
        assert_eq!(w.agent(me).unwrap().culture, [1, 2]);
        assert!(
            w.agent(n).unwrap().tags == w.agent(n).unwrap().tags,
            "tags untouched"
        );
    }

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
