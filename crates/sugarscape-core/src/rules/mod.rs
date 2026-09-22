//! The rules of Appendix B, one module each.

pub mod growback;
pub mod lifecycle;
pub mod movement;

use crate::agent::AgentId;
use crate::world::World;

/// One agent's turn, in the book's order: move, metabolize, maybe die.
/// Later rules (sex, culture, combat) extend this sequence.
pub(crate) fn agent_turn(world: &mut World, id: AgentId) {
    let gathered = movement::act(world, id);
    lifecycle::metabolize(world, id, gathered);
    lifecycle::check_death(world, id);
}
