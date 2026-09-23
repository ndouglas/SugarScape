//! Agent movement rule M (Chapter II), with the pollution-modified welfare
//! s / (1 + p) when pollution is on.

use rand::seq::SliceRandom;

use crate::agent::AgentId;
use crate::geometry::Pos;
use crate::rng::SimRng;
use crate::rules::Harvest;
use crate::world::World;

/// Picks among `(site, distance, value)` candidates: highest value, then
/// nearest, then uniformly at random.
pub(crate) fn choose(candidates: &[(Pos, u32, f64)], rng: &mut SimRng) -> Pos {
    let best_value = candidates
        .iter()
        .map(|c| c.2)
        .fold(f64::NEG_INFINITY, f64::max);
    let nearest = candidates
        .iter()
        .filter(|c| c.2 == best_value)
        .map(|c| c.1)
        .min()
        .expect("at least one candidate");
    let ties: Vec<Pos> = candidates
        .iter()
        .filter(|c| c.2 == best_value && c.1 == nearest)
        .map(|c| c.0)
        .collect();
    *ties.choose(rng).expect("non-empty ties")
}

/// Rule M: look along the four lattice directions as far as vision permits,
/// go to the nearest unoccupied site of maximum welfare and collect its sugar.
/// The agent's current site competes at distance 0, so it stays put when
/// nothing visible is better. Returns the harvest.
pub(crate) fn act(world: &mut World, id: AgentId) -> Harvest {
    if world.config.spice.enabled {
        return act_two_goods(world, id);
    }
    let agent = world.agent(id).expect("live agent");
    let (pos, vision) = (agent.pos, agent.vision);
    let polluted = world.config.pollution.enabled;
    let welfare = |w: &World, p: Pos| {
        let s = w.site(p);
        if polluted {
            s.resource[0] / (1.0 + s.pollution[0])
        } else {
            s.resource[0]
        }
    };
    let mut candidates = vec![(pos, 0, welfare(world, pos))];
    for (q, d) in world.torus.sight(pos, vision) {
        if !world.is_occupied(q) {
            candidates.push((q, d, welfare(world, q)));
        }
    }
    let target = choose(&candidates, &mut world.rng);
    world.move_agent(id, target);
    let site = world.site_mut(target);
    let gathered = site.resource[0];
    site.resource[0] = 0.0;
    world.agent_mut(id).expect("live agent").holdings[0] += gathered;
    Harvest::of(&[gathered])
}

/// Multicommodity M: maximize (foresight) welfare after gathering. Pollution
/// discounts a site's sugar (and spice, if it pollutes) by 1/(1 + p).
fn act_two_goods(world: &mut World, id: AgentId) -> Harvest {
    let fee = world.config.disease.active_fee();
    let a = world.agent(id).expect("live agent");
    let (pos, vision, phi) = (a.pos, a.vision, a.foresight);
    let (w1, w2) = (a.holdings[0], a.holdings[1]);
    let (m1, m2) = (
        a.effective_metabolism(0, fee),
        a.effective_metabolism(1, fee),
    );
    let pollution = world.config.pollution;
    let value = |w: &World, p: Pos| {
        let s = w.site(p);
        let discount = if pollution.enabled {
            1.0 / (1.0 + s.pollution[0])
        } else {
            1.0
        };
        let x1 = s.resource[0] * discount;
        let x2 = if pollution.spice_pollutes {
            s.resource[1] * discount
        } else {
            s.resource[1]
        };
        crate::econ::foresight_welfare(w1 + x1, w2 + x2, m1, m2, phi)
    };
    let mut candidates = vec![(pos, 0, value(world, pos))];
    for (q, d) in world.torus.sight(pos, vision) {
        if !world.is_occupied(q) {
            candidates.push((q, d, value(world, q)));
        }
    }
    let target = choose(&candidates, &mut world.rng);
    world.move_agent(id, target);
    let site = world.site_mut(target);
    let harvest = Harvest::of(&[site.resource[0], site.resource[1]]);
    site.resource[0] = 0.0;
    site.resource[1] = 0.0;
    let a = world.agent_mut(id).expect("live agent");
    a.holdings[0] += harvest.gathered[0];
    a.holdings[1] += harvest.gathered[1];
    harvest
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testkit::*;

    fn mover(w: &mut World, vision: u32) -> AgentId {
        let id = spawn(w, 5, 5);
        w.agent_mut(id).unwrap().vision = vision;
        id
    }

    fn spicy(w: &mut World, vision: u32) -> AgentId {
        w.config.spice.enabled = true;
        let id = mover(w, vision);
        let a = w.agent_mut(id).unwrap();
        a.metabolism[0] = 1;
        a.metabolism[1] = 1;
        id
    }

    #[test]
    fn moves_to_the_richest_visible_site_and_gathers_it() {
        let mut w = blank_world(11, 11);
        let id = mover(&mut w, 3);
        set_sugar(&mut w, 5, 8, 3.0);
        set_sugar(&mut w, 7, 5, 2.0);
        let gathered = act(&mut w, id);
        assert_eq!(gathered.gathered[0], 3.0);
        assert_eq!(w.agent(id).unwrap().pos, Pos::new(5, 8));
        assert_eq!(w.agent(id).unwrap().holdings[0], 13.0);
        assert_eq!(w.site(Pos::new(5, 8)).resource[0], 0.0);
        assert_eq!(w.occupant(Pos::new(5, 5)), None);
    }

    #[test]
    fn prefers_the_nearest_of_equal_sites() {
        let mut w = blank_world(11, 11);
        let id = mover(&mut w, 3);
        set_sugar(&mut w, 5, 2, 2.0); // distance 3
        set_sugar(&mut w, 7, 5, 2.0); // distance 2
        act(&mut w, id);
        assert_eq!(w.agent(id).unwrap().pos, Pos::new(7, 5));
    }

    #[test]
    fn cannot_see_diagonally() {
        let mut w = blank_world(11, 11);
        let id = mover(&mut w, 3);
        set_sugar(&mut w, 6, 6, 4.0);
        act(&mut w, id);
        assert_eq!(w.agent(id).unwrap().pos, Pos::new(5, 5));
    }

    #[test]
    fn skips_occupied_sites() {
        let mut w = blank_world(11, 11);
        let id = mover(&mut w, 3);
        spawn(&mut w, 5, 7);
        set_sugar(&mut w, 5, 7, 4.0);
        set_sugar(&mut w, 3, 5, 1.0);
        act(&mut w, id);
        assert_eq!(w.agent(id).unwrap().pos, Pos::new(3, 5));
    }

    #[test]
    fn sees_across_the_wraparound_edge() {
        let mut w = blank_world(11, 11);
        let id = spawn(&mut w, 0, 5);
        set_sugar(&mut w, 10, 5, 1.0);
        act(&mut w, id);
        assert_eq!(w.agent(id).unwrap().pos, Pos::new(10, 5));
    }

    #[test]
    fn pollution_devalues_sites_when_enabled() {
        let mut w = blank_world(11, 11);
        let id = mover(&mut w, 3);
        set_sugar(&mut w, 6, 5, 4.0);
        w.site_mut(Pos::new(6, 5)).pollution[0] = 3.0; // welfare 1
        set_sugar(&mut w, 5, 7, 2.0); // welfare 2
        w.config.pollution.enabled = true;
        act(&mut w, id);
        assert_eq!(w.agent(id).unwrap().pos, Pos::new(5, 7));
    }

    #[test]
    fn choose_breaks_ties_by_distance_then_randomly() {
        let mut rng = crate::rng::seeded(1);
        let a = (Pos::new(0, 0), 2, 3.0);
        let b = (Pos::new(1, 0), 1, 3.0);
        let c = (Pos::new(2, 0), 1, 3.0);
        let mut picks = std::collections::BTreeSet::new();
        for _ in 0..50 {
            picks.insert(choose(&[a, b, c], &mut rng));
        }
        assert_eq!(picks.into_iter().collect::<Vec<_>>(), vec![b.0, c.0]);
    }

    #[test]
    fn with_spice_agents_seek_the_good_they_lack() {
        let mut w = blank_world(11, 11);
        let id = spicy(&mut w, 3);
        w.agent_mut(id).unwrap().holdings[0] = 30.0;
        w.agent_mut(id).unwrap().holdings[1] = 2.0;
        set_sugar(&mut w, 5, 7, 4.0);
        w.site_mut(Pos::new(7, 5)).resource[1] = 2.0;
        let h = act(&mut w, id);
        assert_eq!(
            w.agent(id).unwrap().pos,
            Pos::new(7, 5),
            "spice-poor agent picks spice"
        );
        assert_eq!(h, crate::rules::Harvest::of(&[0.0, 2.0]));
        assert_eq!(w.agent(id).unwrap().holdings[1], 4.0);
    }

    #[test]
    fn with_spice_both_goods_are_gathered() {
        let mut w = blank_world(11, 11);
        let id = spicy(&mut w, 1);
        set_sugar(&mut w, 5, 6, 2.0);
        w.site_mut(Pos::new(5, 6)).resource[1] = 3.0;
        let h = act(&mut w, id);
        assert_eq!((h.gathered[0], h.gathered[1]), (2.0, 3.0));
        assert_eq!(w.site(Pos::new(5, 6)).resource[1], 0.0);
    }

    #[test]
    fn disease_fees_shift_the_two_good_welfare_weights() {
        // Holding 50 sugar and 5 spice with metabolisms (9, 1), welfare weights
        // are 0.9/0.1 and 5 more sugar beats 5 more spice. Four diseases at a
        // fee of 2 make the metabolisms (17, 9): weights 17/26 and 9/26, and
        // the spice site wins.
        let target = |sick: bool| {
            let mut w = blank_world(11, 11);
            let id = spicy(&mut w, 1);
            {
                let a = w.agent_mut(id).unwrap();
                (
                    a.holdings[0],
                    a.holdings[1],
                    a.metabolism[0],
                    a.metabolism[1],
                ) = (50.0, 5.0, 9, 1);
                if sick {
                    a.diseases = vec![0, 1, 2, 3];
                }
            }
            w.config.disease.enabled = true;
            w.config.disease.fee = 2.0;
            set_sugar(&mut w, 5, 6, 5.0);
            w.site_mut(Pos::new(6, 5)).resource[1] = 5.0;
            act(&mut w, id);
            w.agent(id).unwrap().pos
        };
        assert_eq!(target(false), Pos::new(5, 6));
        assert_eq!(target(true), Pos::new(6, 5));
    }
}
