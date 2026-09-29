//! The decision seam (Minds 1): which engine makes rule M's decision of
//! where an agent moves. `book` is rule M itself; each Minds step adds an
//! engine beside it, and every engine reduces to rule M when it sees and
//! values only what rule M does.

pub mod astar;
pub mod caching;
pub mod goap;
pub mod grid;
pub mod memory;
pub mod mvt;
pub mod utility;

use crate::agent::AgentId;
use crate::config::DecisionRule;
use crate::rules::{movement, Harvest};
use crate::world::World;

/// Rule M's step under the configured decision rule: moves `id` and returns
/// its harvest.
pub(crate) fn decide(world: &mut World, id: AgentId) -> Harvest {
    match world.config.decision.rule {
        DecisionRule::Book => movement::act(world, id),
        DecisionRule::Utility => utility::act(world, id),
        DecisionRule::Goap => goap::forage::act(world, id),
        DecisionRule::Mvt => mvt::act(world, id),
    }
}
