//! Agent combat rule C_α (Chapter III, Appendix B). Replaces M when on.
//!
//! Candidate sites lie along the four lattice directions within vision.
//! Sites held by the agent's own tribe, or by other-tribe agents at least as
//! wealthy as the agent, are discarded. A site's reward is its sugar plus,
//! if occupied, min(α, occupant's sugar). Sites vulnerable to retaliation are
//! discarded: after taking the site, would some other-tribe agent visible
//! (with the attacker's vision) from the target site be wealthier than the
//! attacker's new wealth? The agent moves to the nearest maximum-reward site,
//! collects the reward, and the former occupant is killed. The victim's sugar
//! beyond the reward passes to its children if rule I is on.
//! The current site competes at distance 0 (the agent may stay put);
//! staying put is not subject to the retaliation filter.
//!
//! Equal-wealth targets are excluded: the book requires the predator to be
//! "bigger than" its prey. Only the site's sugar counts as gathered for
//! production pollution; loot taken from a victim does not.

use crate::agent::{AgentId, Tribe};
use crate::geometry::Pos;
use crate::rules::{movement::choose, Harvest};
use crate::world::{DeathCause, World};

pub(crate) fn act(world: &mut World, id: AgentId) -> Harvest {
    let me = world.agent(id).expect("live agent");
    let (pos, vision, tribe, wealth) = (me.pos, me.vision, me.tribe(), me.sugar);
    let cap = if world.config.combat.unlimited {
        f64::INFINITY
    } else {
        world.config.combat.reward
    };

    let mut candidates = vec![(pos, 0, world.site(pos).sugar)];
    for (q, d) in world.torus.sight(pos, vision) {
        let site_sugar = world.site(q).sugar;
        let reward = match world.agent_at(q) {
            Some(o) if o.tribe() == tribe || o.sugar >= wealth => continue,
            Some(o) => site_sugar + cap.min(o.sugar),
            None => site_sugar,
        };
        if vulnerable(world, id, q, vision, tribe, wealth + reward) {
            continue;
        }
        candidates.push((q, d, reward));
    }
    let target = choose(&candidates, &mut world.rng);

    let mut loot = 0.0;
    if let Some(victim_id) = world.occupant(target).filter(|&v| v != id) {
        let victim = world.agent_mut(victim_id).expect("occupant");
        loot = cap.min(victim.sugar);
        victim.sugar -= loot;
        world.kill(victim_id, DeathCause::Combat);
    }
    world.move_agent(id, target);
    let site = world.site_mut(target);
    let gathered = site.sugar;
    site.sugar = 0.0;
    world.agent_mut(id).expect("live agent").sugar += gathered + loot;
    Harvest {
        sugar: gathered,
        spice: 0.0,
    }
}

/// Whether some other-tribe agent visible from `target` would be wealthier
/// than the attacker after the attack.
fn vulnerable(
    world: &World,
    attacker: AgentId,
    target: Pos,
    vision: u32,
    tribe: Tribe,
    after: f64,
) -> bool {
    world.torus.sight(target, vision).into_iter().any(|(q, _)| {
        world
            .agent_at(q)
            .is_some_and(|o| o.id != attacker && o.tribe() != tribe && o.sugar > after)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::Tags;
    use crate::testkit::*;

    fn fighting_world() -> World {
        let mut w = blank_world(15, 15);
        w.config.combat.enabled = true;
        w
    }

    fn red(w: &mut World, x: u32, y: u32, sugar: f64) -> AgentId {
        let id = spawn(w, x, y);
        let a = w.agent_mut(id).unwrap();
        a.tags = Tags::new(u64::MAX, a.tags.len());
        a.sugar = sugar;
        id
    }

    fn blue(w: &mut World, x: u32, y: u32, sugar: f64, vision: u32) -> AgentId {
        let id = spawn(w, x, y);
        let a = w.agent_mut(id).unwrap();
        a.sugar = sugar;
        a.vision = vision;
        id
    }

    #[test]
    fn unlimited_combat_takes_the_victims_whole_wealth() {
        let mut w = fighting_world();
        let me = blue(&mut w, 5, 5, 10.0, 2);
        let victim = red(&mut w, 5, 7, 3.0);
        set_sugar(&mut w, 5, 7, 1.0);
        let gathered = act(&mut w, me);
        assert_eq!(gathered.sugar, 1.0);
        assert_eq!(w.agent(me).unwrap().pos, Pos::new(5, 7));
        assert_eq!(w.agent(me).unwrap().sugar, 14.0);
        assert!(w.agent(victim).is_none());
        assert_eq!(w.events().deaths[0].cause, DeathCause::Combat);
    }

    #[test]
    fn capped_reward_takes_at_most_alpha() {
        let mut w = fighting_world();
        w.config.combat.unlimited = false;
        w.config.combat.reward = 2.0;
        let me = blue(&mut w, 5, 5, 10.0, 2);
        red(&mut w, 5, 7, 3.0);
        act(&mut w, me);
        assert_eq!(w.agent(me).unwrap().sugar, 12.0);
    }

    #[test]
    fn capped_reward_leftovers_pass_to_children_with_inheritance() {
        let mut w = fighting_world();
        w.config.combat.unlimited = false;
        w.config.combat.reward = 2.0;
        w.config.inheritance.enabled = true;
        let me = blue(&mut w, 5, 5, 10.0, 2);
        let victim = red(&mut w, 5, 7, 3.0);
        let orphan = red(&mut w, 12, 12, 1.0);
        w.agent_mut(victim).unwrap().children = vec![orphan];
        act(&mut w, me);
        assert_eq!(w.agent(orphan).unwrap().sugar, 2.0);
    }

    #[test]
    fn never_attacks_own_tribe() {
        let mut w = fighting_world();
        let me = blue(&mut w, 5, 5, 10.0, 2);
        let friend = blue(&mut w, 5, 7, 3.0, 1);
        act(&mut w, me);
        assert!(w.agent(friend).is_some());
        assert_eq!(w.agent(me).unwrap().pos, Pos::new(5, 5));
    }

    #[test]
    fn never_attacks_an_equal_or_wealthier_agent() {
        let mut w = fighting_world();
        let me = blue(&mut w, 5, 5, 10.0, 2);
        let rich = red(&mut w, 5, 7, 10.0);
        act(&mut w, me);
        assert!(w.agent(rich).is_some());
    }

    #[test]
    fn avoids_sites_vulnerable_to_retaliation() {
        let mut w = fighting_world();
        let me = blue(&mut w, 5, 5, 10.0, 2);
        let victim = red(&mut w, 5, 7, 3.0);
        red(&mut w, 5, 9, 20.0); // visible from (5,7) with vision 2; 20 > 13
        act(&mut w, me);
        assert!(w.agent(victim).is_some());
    }

    #[test]
    fn moves_to_empty_sugar_like_rule_m() {
        let mut w = fighting_world();
        let me = blue(&mut w, 5, 5, 10.0, 2);
        set_sugar(&mut w, 7, 5, 2.0);
        act(&mut w, me);
        assert_eq!(w.agent(me).unwrap().pos, Pos::new(7, 5));
        assert_eq!(w.agent(me).unwrap().sugar, 12.0);
    }
}
