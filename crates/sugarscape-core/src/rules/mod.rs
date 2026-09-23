//! The rules of Appendix B, one module each.

pub mod combat;
pub mod culture;
pub mod growback;
pub mod lifecycle;
pub mod movement;
pub mod pollution;
pub mod replacement;
pub mod sex;

use crate::agent::AgentId;
use crate::world::World;

/// Resources an agent collected from its site this turn.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Harvest {
    pub sugar: f64,
    pub spice: f64,
}

/// One agent's turn, in the book's order: move, metabolize, maybe die, then
/// (if still alive) mate with each neighbor and spread culture to them.
pub(crate) fn agent_turn(world: &mut World, id: AgentId) {
    let harvest = if world.config.combat.enabled {
        combat::act(world, id)
    } else {
        movement::act(world, id)
    };
    lifecycle::metabolize(world, id, harvest);
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
