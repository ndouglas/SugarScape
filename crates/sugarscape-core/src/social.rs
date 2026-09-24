//! Observational social bookkeeping: Chapter II's neighbor lists (the
//! neighbor connection network, Animation II-5) and Chapter III's friends
//! (Animation III-8). Nothing here reads or advances `World.rng`, changes a
//! rule's behavior, or is hashed, exported or shared (like trails).

use crate::agent::{AgentId, Tags};
use crate::geometry::Pos;
use crate::world::World;

/// Most friends an agent keeps (Chapter III: "the five agents it has
/// encountered who are nearest it culturally").
pub const MAX_FRIENDS: usize = 5;

/// An agent's neighbor list and friends. Fixed-size, so recording after
/// every move allocates nothing.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Social {
    neighbors: [AgentId; 4],
    neighbor_count: u8,
    /// (friend, Hamming distance between the two tag strings when they met),
    /// earliest-added first.
    friends: [(AgentId, u32); MAX_FRIENDS],
    friend_count: u8,
}

impl Social {
    /// The agents that were von Neumann neighbors after this agent's last
    /// move, in north, south, east, west order (some may have died since).
    pub fn neighbors(&self) -> &[AgentId] {
        &self.neighbors[..usize::from(self.neighbor_count)]
    }

    /// Friends with the Hamming distance recorded when they met, earliest-added
    /// first (some may have died since the agent last met a new neighbor).
    pub fn friends(&self) -> &[(AgentId, u32)] {
        &self.friends[..usize::from(self.friend_count)]
    }

    /// After a move (rule M, or C when combat replaces it), whether or not
    /// the agent changed site: `seen` replaces the neighbor list. While
    /// culture is on each neighbor on it, in order, is then met under the
    /// friend rule at the Hamming distance between `tags` (the mover's) and
    /// its tags now; with culture off the friends are cleared (they are not
    /// kept while off). Reads `world`; changes nothing there.
    pub(crate) fn moved(&mut self, world: &World, seen: Seen, tags: Tags) {
        let n = usize::from(seen.len);
        self.neighbors[..n].copy_from_slice(&seen.ids[..n]);
        self.neighbor_count = seen.len;
        if !world.config.culture.enabled {
            self.friend_count = 0;
            return;
        }
        let alive = |f: AgentId| world.agent(f).is_some();
        for &other in &seen.ids[..n] {
            // Most moves meet friends again: skip them before looking them up.
            if self.is_friend(other) {
                continue;
            }
            let them = world.agent(other).expect("a neighbor that was just seen");
            self.meet(other, (tags.bits() ^ them.tags.bits()).count_ones(), alive);
        }
    }

    fn is_friend(&self, other: AgentId) -> bool {
        self.friends().iter().any(|&(f, _)| f == other)
    }

    /// Drops friends for whom `alive` is false, keeping the order.
    fn drop_dead(&mut self, alive: impl Fn(AgentId) -> bool) {
        let mut kept = 0;
        for i in 0..usize::from(self.friend_count) {
            if alive(self.friends[i].0) {
                self.friends[kept] = self.friends[i];
                kept += 1;
            }
        }
        self.friend_count = kept as u8;
    }

    /// The friend rule for one neighbor met at `distance`: an existing friend
    /// is left as it is (distances are never rechecked, note 28); with fewer
    /// than five friends the neighbor is added; otherwise it replaces the
    /// friend farthest away if strictly closer (among equally far friends
    /// the earliest-added goes), and joins the end of the list. `alive` is
    /// consulted only when the list is full, to free dead friends' slots
    /// first (the same lists as dropping them when they die).
    fn meet(&mut self, other: AgentId, distance: u32, alive: impl Fn(AgentId) -> bool) {
        if self.is_friend(other) {
            return;
        }
        if usize::from(self.friend_count) == MAX_FRIENDS {
            self.drop_dead(alive);
        }
        let n = usize::from(self.friend_count);
        if n < MAX_FRIENDS {
            self.friends[n] = (other, distance);
            self.friend_count += 1;
            return;
        }
        let mut far = 0;
        for i in 1..n {
            if self.friends[i].1 > self.friends[far].1 {
                far = i;
            }
        }
        if distance < self.friends[far].1 {
            self.friends.copy_within(far + 1..n, far);
            self.friends[n - 1] = (other, distance);
        }
    }
}

/// The agents in a site's von Neumann neighborhood, in north, south, east,
/// west order: what an agent records right after its move.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct Seen {
    ids: [AgentId; 4],
    len: u8,
}

impl Seen {
    /// Reads only the occupancy grid (no agent lookups). Grids are at least
    /// 5 × 5, so the four sites are distinct and never `pos` itself.
    pub(crate) fn at(world: &World, pos: Pos) -> Self {
        let mut seen = Self::default();
        for q in world.torus.neighbors(pos) {
            if let Some(other) = world.occupant(q) {
                seen.ids[usize::from(seen.len)] = other;
                seen.len += 1;
            }
        }
        seen
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testkit::*;

    fn tags(w: &mut World, id: AgentId, bits: u64) {
        let len = w.config.tag_length;
        w.agent_mut(id).unwrap().tags = Tags::new(bits, len);
    }

    fn social(w: &World, id: AgentId) -> Social {
        w.agent(id).unwrap().social
    }

    /// What a move of `id` onto its current site records (as M does).
    fn record(w: &mut World, id: AgentId) {
        let a = w.agent(id).unwrap();
        let (mut social, seen) = (a.social, Seen::at(w, a.pos));
        social.moved(w, seen, a.tags);
        w.agent_mut(id).unwrap().social = social;
    }

    #[test]
    fn a_move_records_the_neighbors_in_north_south_east_west_order() {
        let mut w = blank_world(10, 10);
        let me = spawn(&mut w, 5, 5);
        let east = spawn(&mut w, 6, 5);
        let north = spawn(&mut w, 5, 4);
        spawn(&mut w, 6, 6); // diagonal: not a von Neumann neighbor
        record(&mut w, me);
        assert_eq!(social(&w, me).neighbors(), &[north, east]);
        assert!(
            social(&w, north).neighbors().is_empty(),
            "only the mover records"
        );
    }

    #[test]
    fn a_stationary_move_still_replaces_the_list() {
        let mut w = blank_world(10, 10);
        let me = spawn(&mut w, 5, 5);
        let a = spawn(&mut w, 5, 6);
        record(&mut w, me);
        assert_eq!(social(&w, me).neighbors(), &[a]);
        let pos = w.agent(a).unwrap().pos;
        w.remove_agent(pos.x, pos.y).unwrap();
        let b = spawn(&mut w, 4, 5);
        record(&mut w, me);
        assert_eq!(social(&w, me).neighbors(), &[b]);
    }

    #[test]
    fn lists_can_be_asymmetric_and_outlast_the_neighborhood() {
        let mut w = blank_world(10, 10);
        let i = spawn(&mut w, 2, 2);
        let k = spawn(&mut w, 2, 3);
        record(&mut w, i); // i moved next to k: k is on i's list
        assert_eq!(social(&w, i).neighbors(), &[k]);
        assert!(social(&w, k).neighbors().is_empty(), "i is not on k's list");
        // k moves away; i keeps k on its list until i's own next move.
        w.move_agent(k, crate::geometry::Pos::new(7, 7));
        record(&mut w, k);
        assert_eq!(social(&w, i).neighbors(), &[k]);
        assert!(social(&w, k).neighbors().is_empty());
    }

    #[test]
    fn friends_fill_to_five_in_meeting_order_with_their_distances() {
        let mut w = blank_world(10, 10);
        w.config.culture.enabled = true;
        let me = spawn(&mut w, 5, 5);
        let mut met = Vec::new();
        for (x, y, bits) in [(5, 4, 0b1), (5, 6, 0b11), (6, 5, 0b111), (4, 5, 0b1111)] {
            let id = spawn(&mut w, x, y);
            tags(&mut w, id, bits);
            met.push(id);
        }
        record(&mut w, me);
        let expected: Vec<(AgentId, u32)> = met.iter().copied().zip([1, 2, 3, 4]).collect();
        assert_eq!(social(&w, me).friends(), &expected[..]);
        // Meeting the same neighbors again adds nobody.
        record(&mut w, me);
        assert_eq!(social(&w, me).friends().len(), 4);
        // A fifth friend fills the list.
        let pos = w.agent(met[0]).unwrap().pos;
        w.remove_agent(pos.x, pos.y).unwrap();
        let fifth = spawn(&mut w, 5, 4);
        tags(&mut w, fifth, 0b1_1111);
        record(&mut w, me);
        let f = social(&w, me);
        assert_eq!(f.friends().len(), MAX_FRIENDS);
        assert_eq!(f.friends()[4], (fifth, 5));
    }

    /// `me` at (5, 5) with five friends at distances `ds` (earliest first);
    /// every friend is taken off to column 0 again, so the next move meets
    /// only the agents a test places.
    fn five_friends(ds: [u32; 5]) -> (World, AgentId, Vec<AgentId>) {
        let mut w = blank_world(10, 10);
        w.config.culture.enabled = true;
        let me = spawn(&mut w, 5, 5);
        let mut ids = Vec::new();
        for d in ds {
            let id = spawn(&mut w, 5, 4);
            tags(&mut w, id, (1u64 << d) - 1);
            record(&mut w, me);
            w.move_agent(id, crate::geometry::Pos::new(0, ids.len() as u32));
            ids.push(id);
        }
        let got: Vec<u32> = social(&w, me).friends().iter().map(|f| f.1).collect();
        assert_eq!(got, ds);
        (w, me, ids)
    }

    #[test]
    fn a_strictly_closer_neighbor_replaces_the_farthest_friend() {
        let (mut w, me, ids) = five_friends([3, 7, 2, 7, 5]);
        let meet = |w: &mut World, ones: u32| {
            let id = spawn(w, 6, 5);
            tags(w, id, (1u64 << ones) - 1);
            record(w, me);
            w.remove_agent(6, 5).unwrap();
            id
        };
        let far = meet(&mut w, 8);
        assert!(
            !social(&w, me).friends().iter().any(|f| f.0 == far),
            "farther than every friend"
        );
        let tie = meet(&mut w, 7);
        assert!(
            !social(&w, me).friends().iter().any(|f| f.0 == tie),
            "ties keep existing friends"
        );
        let closer = meet(&mut w, 1);
        assert_eq!(
            social(&w, me).friends(),
            &[
                (ids[0], 3),
                (ids[2], 2),
                (ids[3], 7),
                (ids[4], 5),
                (closer, 1)
            ],
            "the earlier of the two farthest (7) is replaced; the newcomer joins the end"
        );
    }

    #[test]
    fn friends_are_never_rechecked_after_tags_change() {
        let (mut w, me, ids) = five_friends([1, 1, 1, 1, 1]);
        tags(&mut w, ids[0], u64::MAX); // now far away culturally
        let newcomer = spawn(&mut w, 6, 5);
        tags(&mut w, newcomer, 0b1);
        record(&mut w, me);
        assert_eq!(
            social(&w, me).friends()[0],
            (ids[0], 1),
            "the stored distance stands"
        );
        assert!(!social(&w, me).friends().iter().any(|f| f.0 == newcomer));
    }

    #[test]
    fn a_dead_friend_frees_its_slot() {
        let (mut w, me, ids) = five_friends([1, 2, 3, 4, 5]);
        let pos = w.agent(ids[1]).unwrap().pos;
        w.remove_agent(pos.x, pos.y).unwrap();
        let newcomer = spawn(&mut w, 6, 5);
        tags(&mut w, newcomer, 0b111_1111); // 7: farther than every friend
        record(&mut w, me);
        let got: Vec<AgentId> = social(&w, me).friends().iter().map(|f| f.0).collect();
        assert_eq!(got, vec![ids[0], ids[2], ids[3], ids[4], newcomer]);
    }

    #[test]
    fn with_culture_off_friends_are_empty_and_not_kept() {
        let mut w = blank_world(10, 10);
        let me = spawn(&mut w, 5, 5);
        let n = spawn(&mut w, 5, 6);
        record(&mut w, me);
        assert_eq!(
            social(&w, me).neighbors(),
            &[n],
            "neighbors are always recorded"
        );
        assert!(social(&w, me).friends().is_empty());
        w.config.culture.enabled = true;
        record(&mut w, me);
        assert_eq!(social(&w, me).friends().len(), 1);
        w.config.culture.enabled = false;
        record(&mut w, me);
        assert!(
            social(&w, me).friends().is_empty(),
            "turning culture off clears at the next move"
        );
    }

    #[test]
    fn every_move_records_during_a_tick() {
        let mut w = blank_world(10, 10);
        let a = spawn(&mut w, 5, 5);
        let b = spawn(&mut w, 5, 6);
        w.step();
        // Vision 1 on a zero landscape: both stay (distance 0 wins), and each
        // records the other after its own move.
        assert_eq!(social(&w, a).neighbors(), &[b]);
        assert_eq!(social(&w, b).neighbors(), &[a]);
    }
}
