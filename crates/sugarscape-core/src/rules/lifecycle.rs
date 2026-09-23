//! Metabolism (with pollution formation) and death.

use crate::agent::AgentId;
use crate::config::MAX_GOODS;
use crate::rules::Harvest;
use crate::world::{DeathCause, World};

/// Burns each of the n goods at its effective metabolism (plus the disease fee per
/// carried disease). With rule P on, each pollutant k on the agent's site
/// gains Σᵢ Πₖᵢ·gatheredᵢ + Σᵢ Χₖᵢ·burnedᵢ (sums from 0.0 in good order).
pub(crate) fn metabolize(world: &mut World, id: AgentId, harvest: Harvest) {
    let n = world.config.goods.len();
    let fee = world.config.disease.active_fee();
    let agent = world.agent_mut(id).expect("live agent");
    let burned: [f64; MAX_GOODS] = std::array::from_fn(|i| {
        if i < n {
            agent.effective_metabolism(i, fee)
        } else {
            0.0
        }
    });
    for (have, burn) in agent.holdings.iter_mut().zip(&burned).take(n) {
        *have -= burn;
    }
    let pos = agent.pos;
    if world.config.pollution.enabled {
        let added: Vec<f64> = world
            .config
            .pollution
            .pollutants
            .iter()
            .map(|p| {
                let produced = p
                    .production
                    .iter()
                    .zip(&harvest.gathered)
                    .fold(0.0, |sum, (c, g)| sum + c * g);
                let consumed = p
                    .consumption
                    .iter()
                    .zip(&burned)
                    .fold(0.0, |sum, (c, b)| sum + c * b);
                produced + consumed
            })
            .collect();
        let site = world.site_mut(pos);
        for (level, amount) in site.pollution.iter_mut().zip(added) {
            *level += amount;
        }
    }
}

/// Kills the agent if any good is at or below zero or, with lifespan on, it
/// has outlived its maximum age. Returns whether it died.
pub(crate) fn check_death(world: &mut World, id: AgentId) -> bool {
    let agent = world.agent(id).expect("live agent");
    let n = world.config.goods.len();
    let starving = agent.holdings[..n].iter().any(|&h| h <= 0.0);
    let cause = if starving {
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
        w.agent_mut(id).unwrap().metabolism[0] = 3;
        metabolize(&mut w, id, crate::rules::Harvest::of(&[0.0, 0.0]));
        assert_eq!(w.agent(id).unwrap().holdings[0], 7.0);
    }

    #[test]
    fn starving_agents_die_and_free_their_site() {
        let mut w = blank_world(5, 5);
        let id = spawn(&mut w, 2, 2);
        w.agent_mut(id).unwrap().holdings[0] = 0.0;
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
        w.agent_mut(id).unwrap().metabolism[0] = 2;
        w.config.pollution.enabled = true;
        w.config.pollution.pollutants[0].production[0] = 0.5;
        w.config.pollution.pollutants[0].consumption[0] = 3.0;
        metabolize(&mut w, id, crate::rules::Harvest::of(&[4.0, 0.0]));
        assert_eq!(
            w.site(crate::geometry::Pos::new(2, 2)).pollution[0],
            0.5 * 4.0 + 3.0 * 2.0
        );
    }

    #[test]
    fn no_pollution_when_disabled() {
        let mut w = blank_world(5, 5);
        let id = spawn(&mut w, 2, 2);
        w.agent_mut(id).unwrap().metabolism[0] = 2;
        metabolize(&mut w, id, crate::rules::Harvest::of(&[4.0, 0.0]));
        assert_eq!(w.site(crate::geometry::Pos::new(2, 2)).pollution[0], 0.0);
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
        w.agent_mut(parent).unwrap().holdings[0] = 9.0;
        w.kill(c3, DeathCause::Starvation);
        w.kill(parent, DeathCause::OldAge);
        assert_eq!(w.agent(c1).unwrap().holdings[0], 14.5);
        assert_eq!(w.agent(c2).unwrap().holdings[0], 14.5);
    }

    #[test]
    fn no_inheritance_when_disabled() {
        let mut w = blank_world(5, 5);
        let parent = spawn(&mut w, 0, 0);
        let child = spawn(&mut w, 1, 0);
        w.agent_mut(parent).unwrap().children = vec![child];
        w.kill(parent, DeathCause::OldAge);
        assert_eq!(w.agent(child).unwrap().holdings[0], 10.0);
    }

    #[test]
    fn inheritance_splits_spice_too() {
        let mut w = blank_world(5, 5);
        add_goods(&mut w.config, 2);
        w.config.inheritance.enabled = true;
        let parent = spawn(&mut w, 0, 0);
        let child = spawn(&mut w, 1, 0);
        w.agent_mut(parent).unwrap().children = vec![child];
        w.agent_mut(parent).unwrap().holdings[1] = 6.0;
        w.kill(parent, DeathCause::OldAge);
        assert_eq!(w.agent(child).unwrap().holdings[1], 16.0);
    }

    #[test]
    fn with_spice_agents_burn_both_and_die_of_either() {
        let mut w = blank_world(5, 5);
        add_goods(&mut w.config, 2);
        let id = spawn(&mut w, 2, 2);
        w.agent_mut(id).unwrap().metabolism[1] = 10;
        metabolize(&mut w, id, crate::rules::Harvest::default());
        assert_eq!(w.agent(id).unwrap().holdings[1], 0.0);
        assert!(check_death(&mut w, id), "spice starvation");
    }

    #[test]
    fn only_sugar_pollutes_unless_spice_pollutes() {
        let mut w = blank_world(5, 5);
        add_goods(&mut w.config, 2);
        w.config.pollution.enabled = true;
        let id = spawn(&mut w, 2, 2);
        metabolize(&mut w, id, crate::rules::Harvest::of(&[1.0, 4.0]));
        assert_eq!(w.site(crate::geometry::Pos::new(2, 2)).pollution[0], 1.0);
        {
            let p = &mut w.config.pollution.pollutants[0];
            p.production[1] = 1.0;
            p.consumption[1] = 1.0;
            p.devalues[1] = true;
        }
        metabolize(&mut w, id, crate::rules::Harvest::of(&[1.0, 4.0]));
        assert_eq!(
            w.site(crate::geometry::Pos::new(2, 2)).pollution[0],
            1.0 + 5.0
        );
    }

    #[test]
    fn each_carried_disease_adds_the_fee_to_both_metabolisms() {
        let mut w = blank_world(5, 5);
        add_goods(&mut w.config, 2);
        w.config.disease.enabled = true;
        w.config.disease.fee = 1.5;
        let id = spawn(&mut w, 2, 2);
        {
            let a = w.agent_mut(id).unwrap();
            a.metabolism[0] = 1;
            a.metabolism[1] = 2;
            a.diseases = vec![0, 3];
        }
        metabolize(&mut w, id, crate::rules::Harvest::default());
        let a = w.agent(id).unwrap();
        assert_eq!((a.holdings[0], a.holdings[1]), (10.0 - 4.0, 10.0 - 5.0));
        w.config.disease.enabled = false;
        metabolize(&mut w, id, crate::rules::Harvest::default());
        assert_eq!(
            w.agent(id).unwrap().holdings[0],
            6.0 - 1.0,
            "no fee while disease is off"
        );
    }

    #[test]
    fn with_three_goods_agents_burn_every_good_and_die_of_any() {
        let mut w = blank_world(5, 5);
        add_goods(&mut w.config, 3);
        let id = spawn(&mut w, 2, 2);
        {
            let a = w.agent_mut(id).unwrap();
            a.holdings[2] = 5.0;
            a.metabolism[2] = 2;
        }
        metabolize(&mut w, id, Harvest::default());
        assert_eq!(w.agent(id).unwrap().holdings[2], 3.0);
        assert!(!check_death(&mut w, id));
        w.agent_mut(id).unwrap().metabolism[2] = 3;
        metabolize(&mut w, id, Harvest::default());
        assert!(check_death(&mut w, id), "out of good 2");
    }

    #[test]
    fn consumption_pollution_counts_every_good_burned() {
        let mut w = blank_world(5, 5);
        add_goods(&mut w.config, 3);
        w.config.pollution.enabled = true;
        {
            let p = &mut w.config.pollution.pollutants[0];
            p.production = vec![0.0, 0.0, 2.0];
            p.consumption = vec![0.0, 0.0, 3.0];
        }
        let id = spawn(&mut w, 2, 2);
        {
            let a = w.agent_mut(id).unwrap();
            a.holdings[2] = 10.0;
            a.metabolism[2] = 1;
        }
        metabolize(&mut w, id, Harvest::of(&[0.0, 0.0, 4.0]));
        assert_eq!(
            w.site(crate::geometry::Pos::new(2, 2)).pollution[0],
            2.0 * 4.0 + 3.0 * 1.0
        );
    }

    #[test]
    fn inheritance_splits_every_good() {
        let mut w = blank_world(5, 5);
        add_goods(&mut w.config, 3);
        w.config.inheritance.enabled = true;
        let parent = spawn(&mut w, 0, 0);
        let c1 = spawn(&mut w, 1, 0);
        let c2 = spawn(&mut w, 2, 0);
        w.agent_mut(parent).unwrap().children = vec![c1, c2];
        w.agent_mut(parent).unwrap().holdings[2] = 6.0;
        w.kill(parent, DeathCause::OldAge);
        assert_eq!(w.agent(c1).unwrap().holdings[2], 3.0);
        assert_eq!(w.agent(c2).unwrap().holdings[2], 3.0);
    }
}
