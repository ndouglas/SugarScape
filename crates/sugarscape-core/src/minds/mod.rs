//! The decision seam (Minds 1): which engine makes rule M's decision of
//! where an agent moves. `book` is rule M itself; each Minds step adds an
//! engine beside it, and every engine reduces to rule M when it sees and
//! values only what rule M does.

pub mod astar;
pub mod caching;
pub mod central;
pub mod goap;
pub mod grid;
pub mod memory;
pub mod mvt;
pub mod protection;
pub mod spatial_hoarding;
pub mod utility;

use crate::agent::AgentId;
use crate::config::DecisionRule;
use crate::rules::{movement, Harvest};
use crate::world::World;

/// Rule M's step under the configured decision rule: moves `id` and returns
/// its harvest.
pub(crate) fn decide(world: &mut World, id: AgentId) -> Harvest {
    if let Some(harvest) = protection::lab::scripted_action(world, id) {
        return harvest;
    }
    if let Some(harvest) = protection::controller::act(world, id) {
        return harvest;
    }
    if world.config.spatial_hoarding.enabled {
        if let Some(harvest) = spatial_hoarding::guard::guard_turn(world, id) {
            return harvest;
        }
        let a = world.agent(id).expect("live agent");
        if a.spatial.as_ref().is_some_and(|s| s.delivery.is_some()) {
            let target = spatial_hoarding::delivery::delivery_target(world, id).unwrap_or(a.pos);
            spatial_hoarding::stores::tick_events(world)
                .expect("enabled")
                .delivery
                .return_turns += 1;
            return movement::arrive(world, id, target);
        }
    }
    // Minds 5: central-place foraging (under `mvt` or `goap`, validated).
    if world.config.central.enabled {
        return central::act(world, id);
    }
    match world.config.decision.rule {
        DecisionRule::Book => movement::act(world, id),
        DecisionRule::Utility => utility::act(world, id),
        DecisionRule::Goap => goap::forage::act(world, id),
        DecisionRule::Mvt => mvt::act(world, id),
    }
}
