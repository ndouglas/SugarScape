//! Metabolism (with pollution formation) and death.

use crate::agent::AgentId;
use crate::world::{DeathCause, World};

/// Burns `metabolism` sugar. With rule P on, the agent's site gains
/// production pollution α·gathered plus consumption pollution β·metabolism.
pub(crate) fn metabolize(world: &mut World, id: AgentId, gathered: f64) {
    let agent = world.agent_mut(id).expect("live agent");
    let burned = f64::from(agent.metabolism);
    agent.sugar -= burned;
    let pos = agent.pos;
    let p = world.config.pollution;
    if p.enabled {
        world.site_mut(pos).pollution += p.production * gathered + p.consumption * burned;
    }
}

/// Kills the agent if its sugar is at or below zero or, with lifespan on, it
/// has outlived its maximum age. Returns whether it died.
pub(crate) fn check_death(world: &mut World, id: AgentId) -> bool {
    let agent = world.agent(id).expect("live agent");
    let cause = if agent.sugar <= 0.0 {
        Some(DeathCause::Starvation)
    } else if world.config.lifespan.enabled && agent.age > agent.max_age {
        Some(DeathCause::OldAge)
    } else {
        None
    };
    match cause {
        Some(cause) => {
            world.kill(id, cause);
            true
        }
        None => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testkit::*;

    #[test]
    fn metabolism_burns_sugar() {
        let mut w = blank_world(5, 5);
        let id = spawn(&mut w, 2, 2);
        w.agent_mut(id).unwrap().metabolism = 3;
        metabolize(&mut w, id, 0.0);
        assert_eq!(w.agent(id).unwrap().sugar, 7.0);
    }

    #[test]
    fn starving_agents_die_and_free_their_site() {
        let mut w = blank_world(5, 5);
        let id = spawn(&mut w, 2, 2);
        w.agent_mut(id).unwrap().sugar = 0.0;
        assert!(check_death(&mut w, id));
        assert!(w.agent(id).is_none());
        assert_eq!(w.occupant(crate::geometry::Pos::new(2, 2)), None);
        assert_eq!(w.events().deaths[0].cause, DeathCause::Starvation);
    }

    #[test]
    fn healthy_agents_live() {
        let mut w = blank_world(5, 5);
        let id = spawn(&mut w, 2, 2);
        assert!(!check_death(&mut w, id));
        assert!(w.agent(id).is_some());
    }

    #[test]
    fn gathering_and_metabolism_pollute_the_site_when_enabled() {
        let mut w = blank_world(5, 5);
        let id = spawn(&mut w, 2, 2);
        w.agent_mut(id).unwrap().metabolism = 2;
        w.config.pollution.enabled = true;
        w.config.pollution.production = 0.5;
        w.config.pollution.consumption = 3.0;
        metabolize(&mut w, id, 4.0);
        assert_eq!(
            w.site(crate::geometry::Pos::new(2, 2)).pollution,
            0.5 * 4.0 + 3.0 * 2.0
        );
    }

    #[test]
    fn no_pollution_when_disabled() {
        let mut w = blank_world(5, 5);
        let id = spawn(&mut w, 2, 2);
        w.agent_mut(id).unwrap().metabolism = 2;
        metabolize(&mut w, id, 4.0);
        assert_eq!(w.site(crate::geometry::Pos::new(2, 2)).pollution, 0.0);
    }

    #[test]
    fn agents_older_than_max_age_die_only_with_lifespan_on() {
        let mut w = blank_world(5, 5);
        let id = spawn(&mut w, 2, 2);
        w.agent_mut(id).unwrap().age = 101;
        assert!(!check_death(&mut w, id), "immortal while lifespan is off");
        w.config.lifespan.enabled = true;
        assert!(check_death(&mut w, id));
        assert_eq!(w.events().deaths[0].cause, DeathCause::OldAge);
    }

    #[test]
    fn inheritance_splits_wealth_among_living_children() {
        let mut w = blank_world(5, 5);
        w.config.inheritance.enabled = true;
        let parent = spawn(&mut w, 0, 0);
        let c1 = spawn(&mut w, 1, 0);
        let c2 = spawn(&mut w, 2, 0);
        let c3 = spawn(&mut w, 3, 0);
        w.agent_mut(parent).unwrap().children = vec![c1, c2, c3];
        w.agent_mut(parent).unwrap().sugar = 9.0;
        w.kill(c3, DeathCause::Starvation);
        w.kill(parent, DeathCause::OldAge);
        assert_eq!(w.agent(c1).unwrap().sugar, 14.5);
        assert_eq!(w.agent(c2).unwrap().sugar, 14.5);
    }

    #[test]
    fn no_inheritance_when_disabled() {
        let mut w = blank_world(5, 5);
        let parent = spawn(&mut w, 0, 0);
        let child = spawn(&mut w, 1, 0);
        w.agent_mut(parent).unwrap().children = vec![child];
        w.kill(parent, DeathCause::OldAge);
        assert_eq!(w.agent(child).unwrap().sugar, 10.0);
    }

    #[test]
    fn inheritance_splits_spice_too() {
        let mut w = blank_world(5, 5);
        w.config.inheritance.enabled = true;
        let parent = spawn(&mut w, 0, 0);
        let child = spawn(&mut w, 1, 0);
        w.agent_mut(parent).unwrap().children = vec![child];
        w.agent_mut(parent).unwrap().spice = 6.0;
        w.kill(parent, DeathCause::OldAge);
        assert_eq!(w.agent(child).unwrap().spice, 16.0);
    }
}
