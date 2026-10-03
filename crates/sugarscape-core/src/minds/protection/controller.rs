//! Owner-local cue: occupancy in ordinary sight, without observer traits.
use crate::{agent::AgentId, world::World};
pub(crate) fn perceived_exposure(world: &World, owner: AgentId) -> bool {
    let a = world.agent(owner).expect("live owner");
    world
        .sight(a.pos, a.vision)
        .into_iter()
        .any(|(pos, _)| world.occupant(pos).is_some_and(|id| id != owner))
}
