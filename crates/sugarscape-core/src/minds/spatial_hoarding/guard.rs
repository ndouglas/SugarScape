//! Frozen, paid guard intentions precede the ordinary initiative shuffle.

use rand::Rng;

use super::{access, recover_own, stores};
use crate::agent::AgentId;
use crate::minds::caching;
use crate::portable::exp_neg;
use crate::rules::Harvest;
use crate::world::World;

fn probability(x: f64) -> f64 {
    if x >= 0.0 {
        1.0 / (1.0 + exp_neg(-x))
    } else {
        let e = exp_neg(x);
        e / (1.0 + e)
    }
}

pub(crate) fn prepare_guards(world: &mut World) {
    if !world.config.spatial_hoarding.enabled {
        return;
    }
    let ids = world.agent_ids(); // BTreeMap order: stable founder id before shuffle.
    for &id in &ids {
        if let Some(s) = world.agent_mut(id).expect("live agent").spatial.as_mut() {
            s.guarding = false;
        }
    }
    if !world.config.spatial_hoarding.guard {
        return;
    }
    for id in ids {
        let a = world.agent(id).expect("live agent");
        let Some(s) = a.spatial.as_ref() else {
            continue;
        };
        if a.cheater || s.larder <= 0.0 || !access::in_contact(world, a.pos, s.home) {
            continue;
        }
        let target = caching::reserve(world, id).max(1.0)
            + s.traits.defense * f64::from(world.config.caching.capacity);
        let p =
            probability(world.config.spatial_hoarding.defense_slope * (s.larder / target - 0.5));
        // Every eligible guard gets one intention draw, even a rounded endpoint.
        let guarding = world.rng.gen::<f64>() < p;
        world
            .agent_mut(id)
            .expect("live agent")
            .spatial
            .as_mut()
            .expect("spatial agent")
            .guarding = guarding;
        if guarding {
            stores::tick_events(world).expect("enabled").guard.intended += 1;
        }
    }
}

/// A guard action reports own recovery and, only under the probe, site harvest.
pub(crate) fn guard_turn(world: &mut World, id: AgentId) -> Option<Harvest> {
    let guarding = world
        .agent(id)
        .and_then(|a| a.spatial.as_ref())
        .is_some_and(|s| s.guarding);
    if !guarding {
        return None;
    }
    let recovered = recover_own(world, id);
    let e = &mut stores::tick_events(world).expect("enabled").guard;
    e.executed += 1;
    e.recovered += recovered;
    let mut harvest = Harvest {
        dug: recovered,
        ..Harvest::default()
    };
    if recovered == 0.0 && world.spatial_probe.guard_harvest {
        let a = world.agent(id).expect("live guard");
        let (at, used, remembers) = (a.pos, a.holdings[0], a.remembers);
        let gathered = crate::rules::movement::gather_site(world, id, at, used, remembers);
        stores::tick_events(world)
            .expect("enabled")
            .guard
            .probe_harvest += gathered.gathered[0];
        harvest.gathered = gathered.gathered;
    }
    Some(harvest)
}
