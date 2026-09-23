//! Properties that must hold after every tick for any rule combination.

use proptest::prelude::*;
use sugarscape_core::config::{Config, URange};
use sugarscape_core::world::World;

// The brief's `prop_map` builds `Config` by assigning fields one at a time
// after `Config::default()` (clearer here than a ten-field struct literal).
#[allow(clippy::field_reassign_with_default)]
fn config_strategy() -> impl Strategy<Value = Config> {
    (
        50u32..=400,
        proptest::bool::ANY,
        proptest::bool::ANY,
        proptest::bool::ANY,
        proptest::bool::ANY,
        proptest::bool::ANY,
        proptest::bool::ANY,
        proptest::bool::ANY,
        proptest::bool::ANY,
        proptest::bool::ANY,
        (
            proptest::bool::ANY,
            proptest::bool::ANY,
            proptest::bool::ANY,
            proptest::bool::ANY,
        ),
    )
        .prop_map(
            |(
                pop,
                seasons,
                pollution,
                diffusion,
                lifespan,
                sex,
                inheritance,
                culture,
                combat,
                replacement,
                (spice, trade, credit, foresight),
            )| {
                let mut c = Config::default();
                c.population = pop;
                c.seasons.enabled = seasons;
                c.pollution.enabled = pollution;
                c.diffusion.enabled = diffusion;
                c.lifespan.enabled = lifespan;
                c.sex.enabled = sex;
                c.inheritance.enabled = inheritance;
                c.culture.enabled = culture;
                c.combat.enabled = combat;
                c.replacement.enabled = replacement && lifespan && !sex;
                if sex {
                    c.endowment.min = 50;
                    c.endowment.max = 100;
                }
                c.spice.enabled = spice && !combat;
                c.trade.enabled = trade && c.spice.enabled;
                c.foresight.enabled = foresight && c.spice.enabled;
                c.credit.enabled = credit && sex;
                if c.spice.enabled {
                    c.spice.endowment = URange::new(25, 50);
                }
                c
            },
        )
}

fn check(world: &World) -> Result<(), TestCaseError> {
    let mut occupied = 0;
    for i in 0..world.sites.len() {
        let pos = world.torus.pos(i);
        let site = world.site(pos);
        prop_assert!(
            site.sugar <= site.capacity + 1e-9,
            "sugar above capacity at {pos:?}"
        );
        prop_assert!(site.sugar >= 0.0 && site.pollution >= 0.0);
        prop_assert!(site.spice <= site.spice_capacity + 1e-9);
        if let Some(id) = world.occupant(pos) {
            occupied += 1;
            prop_assert_eq!(world.agent(id).map(|a| a.pos), Some(pos));
        }
    }
    prop_assert_eq!(
        occupied,
        world.population(),
        "one agent per site, all indexed"
    );
    for a in world.agents() {
        prop_assert!(a.sugar > 0.0, "living agent {} has sugar {}", a.id, a.sugar);
        if world.config.spice.enabled {
            prop_assert!(a.spice > 0.0, "living agent {} has spice {}", a.id, a.spice);
        }
    }
    for l in world.loans() {
        prop_assert!(world.agent(l.lender).is_some() && world.agent(l.borrower).is_some());
        prop_assert!(l.due > 0.0);
    }
    Ok(())
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(24))]

    #[test]
    fn invariants_hold_every_tick(config in config_strategy(), seed in any::<u64>()) {
        let mut world = World::new(config.clone(), seed).unwrap();
        check(&world)?;
        for _ in 0..40 {
            world.step();
            check(&world)?;
        }
        if config.replacement.enabled && !config.combat.enabled {
            prop_assert_eq!(world.population(), config.population as usize, "R keeps population constant");
        }
    }

    #[test]
    fn runs_are_deterministic(config in config_strategy(), seed in any::<u64>()) {
        let mut a = World::new(config.clone(), seed).unwrap();
        let mut b = World::new(config, seed).unwrap();
        a.run(25);
        b.run(25);
        prop_assert_eq!(a.fingerprint(), b.fingerprint());
    }
}
