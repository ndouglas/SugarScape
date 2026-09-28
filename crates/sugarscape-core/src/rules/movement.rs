//! Agent movement rule M (Chapter II): multicommodity M over n goods, with
//! the pollution-modified welfare s / (1 + p) when pollution is on.

use rand::seq::SliceRandom;

use crate::agent::{AgentId, Plan};
use crate::config::{Config, MoveMode, MAX_GOODS};
use crate::geometry::Pos;
use crate::landscape::Site;
use crate::minds::astar::astar;
use crate::minds::grid::TorusGrid;
use crate::rng::SimRng;
use crate::rules::Harvest;
use crate::social::Seen;
use crate::world::World;

/// Most sites A* may expand for a walking agent; past it the target counts
/// as unreachable (Minds 2).
pub const WALK_LIMIT: usize = 4096;

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
/// go to the nearest unoccupied site of maximum welfare and collect its sugar
/// (every good, with n ≥ 2). The agent's current site competes at distance 0,
/// so it stays put when nothing visible is better. Returns the harvest.
pub(crate) fn act(world: &mut World, id: AgentId) -> Harvest {
    let candidates = candidates(world, id);
    let target = choose(&candidates, &mut world.rng);
    arrive(world, id, target)
}

/// Rule M's candidates for `id`: its current site at distance 0, then the
/// unoccupied sites in sight in `torus.sight` order, each with rule M's
/// welfare. One good: sugar, discounted by 1/(1 + Σ pₖ) under pollution.
/// n ≥ 2 goods: foresight welfare after gathering, each good discounted by
/// the pollutants that devalue it.
pub(crate) fn candidates(world: &World, id: AgentId) -> Vec<(Pos, u32, f64)> {
    let a = world.agent(id).expect("live agent");
    let (pos, vision) = (a.pos, a.vision);
    let n = world.config.goods.len();
    let value: Box<dyn Fn(Pos) -> f64 + '_> = if n >= 2 {
        let fee = world.config.disease.active_fee();
        let (phi, held) = (a.foresight, a.holdings);
        let mets = a.effective_metabolisms(n, fee);
        Box::new(move |p: Pos| {
            let s = world.site(p);
            let after: [f64; MAX_GOODS] = std::array::from_fn(|i| {
                if i >= n {
                    return 0.0;
                }
                let counted = match devaluation(&world.config, s, i) {
                    Some(d) => s.resource[i] * (1.0 / (1.0 + d)),
                    None => s.resource[i],
                };
                held[i] + counted
            });
            crate::econ::foresight_welfare_n(&after[..n], &mets[..n], phi)
        })
    } else {
        Box::new(|p: Pos| {
            let s = world.site(p);
            match devaluation(&world.config, s, 0) {
                Some(d) => s.resource[0] / (1.0 + d),
                None => s.resource[0],
            }
        })
    };
    let mut out = vec![(pos, 0, value(pos))];
    for (q, d) in world.sight(pos, vision) {
        if !world.is_occupied(q) {
            out.push((q, d, value(q)));
        }
    }
    out
}

/// Moves `id` to `target` (its own site to stay), records its neighbors, and
/// gathers every good there.
pub(crate) fn go_and_gather(world: &mut World, id: AgentId, target: Pos) -> Harvest {
    let n = world.config.goods.len();
    let a = world.agent(id).expect("live agent");
    let (tags, mut social) = (a.tags, a.social);
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

/// Reaches `target` by the configured movement, then gathers where the
/// agent stops. `jump` (rule M) goes there in one tick. `walk` takes `speed`
/// steps along an A* path on the 4-way torus (walls and occupied sites
/// impassable, except the target) and stays when there is none within
/// `WALK_LIMIT`. Records the agent's plan; draws nothing.
pub(crate) fn arrive(world: &mut World, id: AgentId, target: Pos) -> Harvest {
    let pos = world.agent(id).expect("live agent").pos;
    let m = world.config.movement;
    if m.mode == MoveMode::Jump || target == pos {
        world.agent_mut(id).expect("live agent").plan = Plan {
            target: Some(target),
            path: Vec::new(),
            walked: m.mode == MoveMode::Walk,
        };
        return go_and_gather(world, id, target);
    }
    let torus = world.torus;
    // Walls split the non-wall sites into components labeled at build. A
    // target in another component can't be reached, so skip the search:
    // this only short-circuits searches A* would fail anyway, and the
    // walker stays exactly as on a `None` from A*.
    let found = if world.walled_apart(pos, target) {
        None
    } else {
        let grid = TorusGrid::new(torus, |q| q == target || !world.is_occupied(q));
        astar(&grid, torus.index(pos), torus.index(target), WALK_LIMIT)
    };
    let (stop, rest) = match found {
        Some(s) => {
            let steps = (m.speed as usize).min(s.path.len() - 1);
            let rest = s.path[steps + 1..].iter().map(|&i| torus.pos(i)).collect();
            (torus.pos(s.path[steps]), rest)
        }
        None => (pos, Vec::new()),
    };
    world.agent_mut(id).expect("live agent").plan = Plan {
        target: Some(target),
        path: rest,
        walked: true,
    };
    go_and_gather(world, id, stop)
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

    fn walker(w: &mut World, vision: u32, speed: u32) -> AgentId {
        w.config.movement = crate::config::Movement {
            mode: crate::config::MoveMode::Walk,
            speed,
        };
        mover(w, vision)
    }

    #[test]
    fn a_walker_takes_one_step_toward_the_target_and_gathers_only_there() {
        let mut w = blank_world(11, 11);
        let id = walker(&mut w, 3, 1);
        set_sugar(&mut w, 5, 8, 3.0);
        set_sugar(&mut w, 5, 6, 1.0); // on the way: stepped onto, so gathered
        let h = act(&mut w, id);
        // Rule M picks (5, 8) (the most sugar); the walker steps to (5, 6).
        assert_eq!(w.agent(id).unwrap().pos, Pos::new(5, 6));
        assert_eq!(h.gathered[0], 1.0);
        assert_eq!(w.site(Pos::new(5, 8)).resource[0], 3.0, "not reached yet");
        let plan = &w.agent(id).unwrap().plan;
        assert_eq!(plan.target, Some(Pos::new(5, 8)));
        assert_eq!(plan.path, vec![Pos::new(5, 7), Pos::new(5, 8)]);
    }

    #[test]
    fn a_walker_goes_around_an_occupant() {
        let mut w = blank_world(11, 11);
        let id = walker(&mut w, 3, 1);
        spawn(&mut w, 5, 6);
        set_sugar(&mut w, 5, 7, 3.0);
        act(&mut w, id);
        let p = w.agent(id).unwrap().pos;
        assert!(
            p == Pos::new(4, 5) || p == Pos::new(6, 5),
            "a sidestep, got {p:?}"
        );
    }

    #[test]
    fn speed_three_stops_at_the_target() {
        let mut w = blank_world(11, 11);
        let id = walker(&mut w, 3, 3);
        set_sugar(&mut w, 5, 7, 3.0);
        let h = act(&mut w, id);
        assert_eq!(
            (w.agent(id).unwrap().pos, h.gathered[0]),
            (Pos::new(5, 7), 3.0)
        );
        assert!(w.agent(id).unwrap().plan.path.is_empty());
    }

    #[test]
    fn a_walker_with_no_path_stays_and_gathers_where_it_is() {
        let mut w = blank_world(11, 11);
        let id = walker(&mut w, 3, 1);
        set_sugar(&mut w, 5, 5, 0.5);
        set_sugar(&mut w, 5, 8, 3.0);
        for (x, y) in [(5, 4), (5, 6), (4, 5), (6, 5)] {
            spawn(&mut w, x, y); // boxed in
        }
        let before = w.rng.clone();
        let candidates_before = candidates(&w, id);
        let h = act(&mut w, id);
        assert_eq!(
            (w.agent(id).unwrap().pos, h.gathered[0]),
            (Pos::new(5, 5), 0.5)
        );
        // No draws beyond `choose`: `w.rng` after `act` matches a clone
        // advanced by exactly one `choose` over the pre-`act` candidates.
        let mut expected = before;
        let _ = choose(&candidates_before, &mut expected);
        assert_eq!(w.rng, expected);
    }

    #[test]
    fn plan_is_not_hashed() {
        let mut w = blank_world(5, 5);
        let id = spawn(&mut w, 1, 1);
        let before = w.fingerprint();
        w.agent_mut(id).unwrap().plan = crate::agent::Plan {
            target: Some(Pos::new(2, 2)),
            path: vec![Pos::new(1, 2), Pos::new(2, 2)],
            walked: true,
        };
        assert_eq!(w.fingerprint(), before, "plan is observational, not hashed");
    }

    fn walled_world(walls: Vec<crate::config::Wall>) -> World {
        let mut c = blank_config(11, 11);
        c.walls = walls;
        World::new(c, 7).unwrap()
    }

    fn wall(x: u32, y: u32, width: u32, height: u32, opaque: bool) -> crate::config::Wall {
        crate::config::Wall {
            x,
            y,
            width,
            height,
            opaque,
        }
    }

    #[test]
    fn a_walker_boxed_in_by_walls_stays_and_gathers_where_it_is_without_panicking() {
        // Fences (not opaque), not occupants, on all four neighbors: they
        // block movement but not sight, so sugar beyond one is visible and
        // chosen as the target, yet the walker can't reach it.
        let mut w = walled_world(vec![
            wall(5, 4, 1, 1, false),
            wall(5, 6, 1, 1, false),
            wall(4, 5, 1, 1, false),
            wall(6, 5, 1, 1, false),
        ]);
        w.config.movement = crate::config::Movement {
            mode: crate::config::MoveMode::Walk,
            speed: 1,
        };
        let id = spawn(&mut w, 5, 5);
        w.agent_mut(id).unwrap().vision = 3;
        set_sugar(&mut w, 5, 5, 0.5);
        set_sugar(&mut w, 5, 2, 3.0); // visible past the fence at (5, 4)
        let h = act(&mut w, id);
        assert_eq!(
            (w.agent(id).unwrap().pos, h.gathered[0]),
            (Pos::new(5, 5), 0.5)
        );
    }

    #[test]
    fn a_walker_stays_put_when_the_target_is_a_pocket_sealed_by_fences() {
        // A ring of fences around (5, 5) leaves it visible (fences don't
        // stop sight) but unreachable (they do block movement) from any
        // side.
        let mut w = walled_world(vec![
            wall(4, 4, 3, 1, false),
            wall(4, 6, 3, 1, false),
            wall(4, 5, 1, 1, false),
            wall(6, 5, 1, 1, false),
        ]);
        w.config.movement = crate::config::Movement {
            mode: crate::config::MoveMode::Walk,
            speed: 1,
        };
        let id = spawn(&mut w, 5, 1);
        w.agent_mut(id).unwrap().vision = 4;
        set_sugar(&mut w, 5, 1, 0.2);
        set_sugar(&mut w, 5, 5, 5.0); // in the sealed pocket
        let h = act(&mut w, id);
        assert_eq!(
            (w.agent(id).unwrap().pos, h.gathered[0]),
            (Pos::new(5, 1), 0.2)
        );
    }

    #[test]
    fn the_wall_component_precheck_changes_nothing_but_the_search() {
        // The sealed pocket again: the precheck skips A*, and the walker
        // stays and gathers exactly as when A* runs and finds no path.
        let mut w = walled_world(vec![
            wall(4, 4, 3, 1, false),
            wall(4, 6, 3, 1, false),
            wall(4, 5, 1, 1, false),
            wall(6, 5, 1, 1, false),
        ]);
        w.config.movement = crate::config::Movement {
            mode: crate::config::MoveMode::Walk,
            speed: 1,
        };
        let id = spawn(&mut w, 5, 1);
        w.agent_mut(id).unwrap().vision = 4;
        set_sugar(&mut w, 5, 1, 0.2);
        set_sugar(&mut w, 5, 5, 5.0);
        assert!(w.walled_apart(Pos::new(5, 1), Pos::new(5, 5)));
        assert!(!w.walled_apart(Pos::new(5, 1), Pos::new(9, 9)));
        let mut searched = w.clone();
        searched.regions.clear(); // no precheck: A* runs and fails
        let (h, h_searched) = (act(&mut w, id), act(&mut searched, id));
        assert_eq!(
            (w.agent(id).unwrap().pos, h.gathered[0]),
            (Pos::new(5, 1), 0.2)
        );
        assert_eq!(h.gathered, h_searched.gathered);
        assert_eq!(w.agent(id).unwrap().plan, searched.agent(id).unwrap().plan);
        assert_eq!(w.rng, searched.rng);
        assert_eq!(w.fingerprint(), searched.fingerprint());
    }

    #[test]
    fn without_walls_no_components_are_labeled() {
        let w = blank_world(11, 11);
        assert!(w.regions.is_empty());
        assert!(!w.walled_apart(Pos::new(0, 0), Pos::new(5, 5)));
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
    fn candidates_list_the_current_site_first_then_sight_order_skipping_occupied() {
        let mut w = blank_world(11, 11);
        let id = mover(&mut w, 2);
        spawn(&mut w, 5, 6); // occupied: skipped
        set_sugar(&mut w, 5, 5, 1.0);
        set_sugar(&mut w, 7, 5, 3.0);
        let c = candidates(&w, id);
        assert_eq!(c[0], (Pos::new(5, 5), 0, 1.0));
        assert_eq!(c.len(), 1 + 8 - 1);
        assert!(c.iter().all(|x| x.0 != Pos::new(5, 6)));
        assert!(c.windows(2).all(|p| p[0].1 <= p[1].1), "distance order");
        assert!(c.contains(&(Pos::new(7, 5), 2, 3.0)));
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
