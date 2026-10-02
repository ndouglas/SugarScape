//! Deterministic home entrance geometry and bounded walking endpoints.

use crate::agent::AgentId;
use crate::geometry::Pos;
use crate::minds::{astar::astar, grid::TorusGrid};
use crate::rules::movement::WALK_LIMIT;
use crate::world::World;

/// Occupancy does not change entrance geometry.
pub(crate) fn in_contact(world: &World, at: Pos, home: Pos) -> bool {
    !world.is_wall(at)
        && !world.is_wall(home)
        && !world.walled_apart(at, home)
        && (at == home || world.torus.neighbors(home).contains(&at))
}

/// Tasks 3–4 route delivery and observed raids through this bounded search.
#[allow(dead_code)]
pub(crate) fn return_endpoint(world: &World, id: AgentId) -> Option<Pos> {
    let a = world.agent(id)?;
    endpoint(world, id, a.spatial.as_ref()?.home)
}

/// A free entrance, ranked by actual walking steps, then site index.
pub(crate) fn endpoint(world: &World, id: AgentId, home: Pos) -> Option<Pos> {
    let a = world.agent(id)?;
    let torus = world.torus;
    let grid = TorusGrid::new(torus, |q| q == a.pos || !world.is_occupied(q));
    std::iter::once(home)
        .chain(torus.neighbors(home))
        .filter(|&q| {
            in_contact(world, q, home)
                && !world.walled_apart(a.pos, q)
                && (q == a.pos || !world.is_occupied(q))
        })
        .filter_map(|q| {
            astar(&grid, torus.index(a.pos), torus.index(q), WALK_LIMIT)
                .map(|s| (s.path.len() - 1, torus.index(q), q))
        })
        .min_by_key(|&(length, site, _)| (length, site))
        .map(|(_, _, q)| q)
}

#[cfg(test)]
mod tests {
    use super::super::state::{FounderTraits, SpatialState};
    use super::*;
    use crate::geometry::Pos;
    use crate::testkit::*;

    fn owner(w: &mut crate::world::World, at: Pos, home: Pos) -> crate::agent::AgentId {
        let id = spawn(w, at.x, at.y);
        w.agent_mut(id).unwrap().spatial = Some(SpatialState::new(
            home,
            FounderTraits {
                larder: 1.0,
                defense: 0.5,
                cheater: false,
                watches: false,
            },
        ));
        id
    }

    #[test]
    fn home_contact_excludes_diagonals() {
        let w = blank_world(7, 7);
        assert!(in_contact(&w, Pos::new(3, 2), Pos::new(3, 3)));
        assert!(!in_contact(&w, Pos::new(2, 2), Pos::new(3, 3)));
        assert!(in_contact(&w, Pos::new(0, 3), Pos::new(6, 3)));
    }

    #[test]
    fn occupied_home_uses_shortest_free_endpoint_and_current_cell() {
        let mut w = blank_world(9, 9);
        let id = owner(&mut w, Pos::new(0, 0), Pos::new(3, 3));
        spawn(&mut w, 3, 3);
        assert_eq!(return_endpoint(&w, id), Some(Pos::new(3, 2)));
        w.move_agent(id, Pos::new(2, 3));
        assert_eq!(return_endpoint(&w, id), Some(Pos::new(2, 3)));
    }

    #[test]
    fn occupied_endpoints_cannot_teleport() {
        let mut w = blank_world(9, 9);
        let id = owner(&mut w, Pos::new(0, 0), Pos::new(3, 3));
        for p in [
            Pos::new(3, 3),
            Pos::new(3, 2),
            Pos::new(3, 4),
            Pos::new(2, 3),
            Pos::new(4, 3),
        ] {
            spawn(&mut w, p.x, p.y);
        }
        assert_eq!(return_endpoint(&w, id), None);
        assert_eq!(w.agent(id).unwrap().pos, Pos::new(0, 0));
    }

    #[test]
    fn walls_and_components_exclude_contact_and_endpoints() {
        let mut c = blank_config(9, 9);
        c.walls = [(2, 2, 3, 1), (2, 4, 3, 1), (2, 3, 1, 1), (4, 3, 1, 1)]
            .into_iter()
            .map(|(x, y, width, height)| crate::config::Wall {
                x,
                y,
                width,
                height,
                opaque: false,
            })
            .collect();
        let mut w = crate::world::World::new(c, 7).unwrap();
        let id = owner(&mut w, Pos::new(0, 0), Pos::new(3, 3));
        assert!(!in_contact(&w, Pos::new(3, 2), Pos::new(3, 3)));
        assert_eq!(return_endpoint(&w, id), None);
    }

    #[test]
    fn actual_path_length_beats_distance_and_ties_use_site_index() {
        let mut w = blank_world(11, 11);
        let id = owner(&mut w, Pos::new(1, 3), Pos::new(4, 3));
        // The nearest west entrance is reachable only via home; north wins.
        for (x, y) in [(2, 3), (3, 2), (3, 4)] {
            spawn(&mut w, x, y);
        }
        assert_eq!(return_endpoint(&w, id), Some(Pos::new(4, 2)));
    }
}
