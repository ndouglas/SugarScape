//! Agent movement rule M (Chapter II): multicommodity M over n goods, with
//! the pollution-modified welfare s / (1 + p) when pollution is on.

use rand::seq::SliceRandom;

use crate::agent::AgentId;
use crate::config::{Config, MAX_GOODS};
use crate::geometry::Pos;
use crate::landscape::Site;
use crate::rng::SimRng;
use crate::rules::Harvest;
use crate::social::Seen;
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

/// Σ pₖ, in pollutant order from 0.0, over the pollutants that devalue
/// `good`; `None` when pollution is off or none does (the good then counts
/// undiscounted).
pub(crate) fn devaluation(config: &Config, site: &Site, good: usize) -> Option<f64> {
    if !config.pollution.enabled {
        return None;
    }
    let mut total = None;
    for (k, p) in config.pollution.pollutants.iter().enumerate() {
        if p.devalues[good] {
            total = Some(total.unwrap_or(0.0) + site.pollution[k]);
        }
    }
    total
}

/// Rule M: look along the four lattice directions as far as vision permits,
/// go to the nearest unoccupied site of maximum welfare and collect its sugar.
/// The agent's current site competes at distance 0, so it stays put when
/// nothing visible is better. Returns the harvest.
pub(crate) fn act(world: &mut World, id: AgentId) -> Harvest {
    if world.config.goods.len() >= 2 {
        return act_goods(world, id);
    }
    let agent = world.agent(id).expect("live agent");
    let (pos, vision) = (agent.pos, agent.vision);
    let (tags, mut social) = (agent.tags, agent.social);
    let welfare = |w: &World, p: Pos| {
        let s = w.site(p);
        match devaluation(&w.config, s, 0) {
            Some(d) => s.resource[0] / (1.0 + d),
            None => s.resource[0],
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
    social.moved(world, Seen::at(world, target), tags);
    let site = world.site_mut(target);
    let gathered = site.resource[0];
    site.resource[0] = 0.0;
    let a = world.agent_mut(id).expect("live agent");
    a.holdings[0] += gathered;
    a.social = social;
    Harvest::of(&[gathered])
}

/// Multicommodity M over n ≥ 2 goods: maximize (foresight) welfare after
/// gathering. Pollution discounts each good by 1/(1 + Σ pₖ) over the
/// pollutants that devalue it.
fn act_goods(world: &mut World, id: AgentId) -> Harvest {
    let n = world.config.goods.len();
    let fee = world.config.disease.active_fee();
    let a = world.agent(id).expect("live agent");
    let (pos, vision, phi, held) = (a.pos, a.vision, a.foresight, a.holdings);
    let (tags, mut social) = (a.tags, a.social);
    let mets = a.effective_metabolisms(n, fee);
    let value = |w: &World, p: Pos| {
        let s = w.site(p);
        let after: [f64; MAX_GOODS] = std::array::from_fn(|i| {
            if i >= n {
                return 0.0;
            }
            let counted = match devaluation(&w.config, s, i) {
                Some(d) => s.resource[i] * (1.0 / (1.0 + d)),
                None => s.resource[i],
            };
            held[i] + counted
        });
        crate::econ::foresight_welfare_n(&after[..n], &mets[..n], phi)
    };
    let mut candidates = vec![(pos, 0, value(world, pos))];
    for (q, d) in world.torus.sight(pos, vision) {
        if !world.is_occupied(q) {
            candidates.push((q, d, value(world, q)));
        }
    }
    let target = choose(&candidates, &mut world.rng);
    world.move_agent(id, target);
    social.moved(world, Seen::at(world, target), tags);
    let site = world.site_mut(target);
    let mut harvest = Harvest::default();
    for (got, level) in harvest
        .gathered
        .iter_mut()
        .zip(site.resource.iter_mut())
        .take(n)
    {
        *got = *level;
        *level = 0.0;
    }
    let a = world.agent_mut(id).expect("live agent");
    for (have, got) in a.holdings.iter_mut().zip(&harvest.gathered).take(n) {
        *have += got;
    }
    a.social = social;
    harvest
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{Pollutant, Pollution};
    use crate::testkit::*;

    fn mover(w: &mut World, vision: u32) -> AgentId {
        let id = spawn(w, 5, 5);
        w.agent_mut(id).unwrap().vision = vision;
        id
    }

    fn spicy(w: &mut World, vision: u32) -> AgentId {
        add_goods(&mut w.config, 2);
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

    #[test]
    fn with_three_goods_agents_seek_the_scarcest_good() {
        let mut w = blank_world(11, 11);
        add_goods(&mut w.config, 3);
        let id = mover(&mut w, 3);
        {
            let a = w.agent_mut(id).unwrap();
            a.metabolism[..3].copy_from_slice(&[1, 1, 1]);
            a.holdings[..3].copy_from_slice(&[30.0, 30.0, 2.0]);
        }
        set_resource(&mut w, 5, 7, 0, 4.0);
        set_resource(&mut w, 7, 5, 1, 4.0);
        set_resource(&mut w, 5, 3, 2, 2.0);
        let h = act(&mut w, id);
        assert_eq!(w.agent(id).unwrap().pos, Pos::new(5, 3));
        assert_eq!(h.gathered[..3], [0.0, 0.0, 2.0]);
        assert_eq!(w.agent(id).unwrap().holdings[2], 4.0);
        assert_eq!(w.site(Pos::new(5, 3)).resource[2], 0.0);
    }

    #[test]
    fn a_pollutant_discounts_only_the_goods_it_devalues() {
        let mut w = blank_world(11, 11);
        add_goods(&mut w.config, 3);
        let id = mover(&mut w, 3);
        {
            let a = w.agent_mut(id).unwrap();
            a.metabolism[..3].copy_from_slice(&[1, 1, 1]);
            a.holdings[..3].copy_from_slice(&[30.0, 30.0, 2.0]);
        }
        let pollutant = |name: &str, devalues: Vec<bool>| Pollutant {
            name: name.into(),
            production: vec![0.0; 3],
            consumption: vec![0.0; 3],
            devalues,
        };
        w.config.pollution = Pollution {
            enabled: true,
            pollutants: vec![
                pollutant("smoke", vec![true, false, false]),
                pollutant("runoff", vec![false, false, true]),
            ],
        };
        // 2 of good 2 under runoff 3 counts as 2 · 1/4 = 0.5; 1 of good 2
        // under smoke (which spares good 2) counts as 1.
        set_resource(&mut w, 5, 3, 2, 2.0);
        w.site_mut(Pos::new(5, 3)).pollution[1] = 3.0;
        set_resource(&mut w, 5, 7, 2, 1.0);
        w.site_mut(Pos::new(5, 7)).pollution[0] = 9.0;
        act(&mut w, id);
        assert_eq!(w.agent(id).unwrap().pos, Pos::new(5, 7));
    }
}
