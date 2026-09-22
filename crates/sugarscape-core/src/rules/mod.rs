//! The rules of Appendix B, one module each.

pub mod culture;
pub mod growback;
pub mod lifecycle;
pub mod movement;
pub mod pollution;
pub mod replacement;
pub mod sex;

use crate::agent::AgentId;
use crate::world::World;

/// One agent's turn, in the book's order: move, metabolize, maybe die, then
/// (if still alive) mate with each neighbor and spread culture to them.
pub(crate) fn agent_turn(world: &mut World, id: AgentId) {
    let gathered = movement::act(world, id);
    lifecycle::metabolize(world, id, gathered);
    if lifecycle::check_death(world, id) {
        return;
    }
    if world.config.sex.enabled {
        sex::act(world, id);
    }
    if world.config.culture.enabled {
        culture::act(world, id);
    }
}
