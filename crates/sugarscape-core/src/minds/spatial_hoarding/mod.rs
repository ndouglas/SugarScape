//! Minds 9: spatial hoarding episode state and checked founder cohorts.

pub(crate) mod access;
pub mod state;
pub mod stores;

#[cfg(test)]
mod tests {
    use super::state::FounderTraits;
    use crate::config::{CachingRule, Config, MoveMode};
    use crate::stats::Snapshot;
    use crate::world::World;

    fn config() -> Config {
        let mut c = Config {
            population: 3,
            ..Config::default()
        };
        c.spatial_hoarding.enabled = true;
        c.caching.rule = CachingRule::Even;
        c.caching.capacity = 50;
        c.movement.mode = MoveMode::Walk;
        c.watching.on = true;
        c
    }

    fn cohort() -> [FounderTraits; 3] {
        [
            FounderTraits {
                larder: 0.0,
                defense: 0.2,
                cheater: true,
                watches: false,
            },
            FounderTraits {
                larder: 0.4,
                defense: 0.7,
                cheater: false,
                watches: true,
            },
            FounderTraits {
                larder: 1.0,
                defense: 1.0,
                cheater: true,
                watches: true,
            },
        ]
    }

    #[test]
    fn spatial_hoarding_cohort_initializes_slots_before_tick_zero_snapshot() {
        let founders = cohort();
        let world = World::new_with_spatial_cohort(config(), 7, &founders).unwrap();
        for (agent, expected) in world.agents().zip(founders) {
            let state = agent.spatial.as_ref().unwrap();
            assert_eq!(state.home, agent.pos);
            assert_eq!(state.traits, expected);
            assert_eq!(
                (agent.cheater, agent.watches),
                (expected.cheater, expected.watches)
            );
            assert_eq!(state.larder, 0.0);
            assert!(state.larder_since.is_none());
            assert!(state.delivery.is_none());
            assert!(!state.guarding);
            assert!(
                state.seen_larders.is_empty() && agent.seen.is_empty() && agent.caches.is_empty()
            );
            assert!(agent.home.is_none());
        }
        let initial = world.stats.history().first().unwrap();
        assert_eq!(initial.tick, 0);
        let cheaters = initial.cheaters.unwrap();
        assert_eq!((cheaters.hoarder_alive, cheaters.cheater_alive), (1, 2));
        let watchers = initial.watchers.unwrap();
        assert_eq!((watchers.watcher_alive, watchers.other_alive), (2, 1));
        assert_eq!(watchers.watcher_advantage, 0.0);
        let watcher_wealth: f64 = world
            .agents()
            .filter(|a| a.watches)
            .map(|a| a.holdings[0])
            .sum();
        assert_eq!(watchers.watcher_wealth, watcher_wealth / 2.0);
    }

    #[test]
    fn spatial_hoarding_cohort_snapshot_denominators_retain_dead_founders() {
        let mut world = World::new_with_spatial_cohort(config(), 7, &cohort()).unwrap();
        world.remove(2);
        let watchers = Snapshot::of(&world).watchers.unwrap();
        assert_eq!(watchers.watcher_advantage, -0.5);
    }

    #[test]
    fn spatial_hoarding_ordinary_constructor_uses_uniform_traits_and_existing_flag_deal() {
        let mut c = config();
        c.theft.cheaters = 0.5;
        c.watching.watchers = 0.5;
        let world = World::new(c, 7).unwrap();
        for (agent, flags) in world
            .agents()
            .zip([(false, false), (true, true), (false, false)])
        {
            let state = agent.spatial.as_ref().unwrap();
            assert_eq!((state.traits.larder, state.traits.defense), (0.15, 0.5));
            assert_eq!((state.traits.cheater, state.traits.watches), flags);
        }
    }

    #[test]
    fn spatial_hoarding_disabled_agents_have_no_extension_state() {
        let mut c = config();
        c.spatial_hoarding.enabled = false;
        assert!(World::new(c, 7)
            .unwrap()
            .agents()
            .all(|a| a.spatial.is_none()));
    }

    #[test]
    fn spatial_hoarding_cohort_rejects_disabled_extension_and_wrong_length() {
        let mut c = config();
        c.spatial_hoarding.enabled = false;
        let errors = World::new_with_spatial_cohort(c, 7, &cohort())
            .err()
            .unwrap();
        assert_eq!(errors[0].field, "spatial_hoarding.enabled");
        let errors = World::new_with_spatial_cohort(config(), 7, &cohort()[..2])
            .err()
            .unwrap();
        assert_eq!(errors[0].field, "spatial_hoarding.cohort");
    }

    #[test]
    fn spatial_hoarding_cohort_rejects_invalid_traits_with_slot_paths() {
        for value in [-0.1, 1.1, f64::NAN, f64::INFINITY] {
            let mut founders = cohort();
            founders[1].larder = value;
            founders[2].defense = value;
            let errors = World::new_with_spatial_cohort(config(), 7, &founders)
                .err()
                .unwrap();
            assert_eq!(
                errors.iter().map(|e| e.field.as_str()).collect::<Vec<_>>(),
                [
                    "spatial_hoarding.cohort.1.larder",
                    "spatial_hoarding.cohort.2.defense"
                ]
            );
        }
    }
}
