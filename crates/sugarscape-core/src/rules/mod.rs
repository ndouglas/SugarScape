//! The rules of Appendix B, one module each.

pub mod combat;
pub mod credit;
pub mod culture;
pub mod disease;
pub mod growback;
pub mod lifecycle;
pub mod movement;
pub mod pollution;
pub mod replacement;
pub mod sex;
pub mod trade;
pub mod truffles;

use crate::agent::AgentId;
use crate::config::{DiseaseCure, MAX_GOODS};
use crate::world::World;

/// Resources an agent collected from its site this turn.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Harvest {
    /// Units of each good gathered (slots ≥ n are 0).
    pub gathered: [f64; MAX_GOODS],
    /// Minds 5: good 0 dug from the agent's own cache this turn (already in
    /// its holdings). Not newly gathered: it was counted when first
    /// gathered, so it forms no pollution, isn't income and doesn't feed
    /// the marginal-value rule's estimate of the habitat's intake rate.
    pub dug: f64,
}

impl Harvest {
    /// A harvest of `amounts`, in good order.
    pub fn of(amounts: &[f64]) -> Self {
        let mut gathered = [0.0; MAX_GOODS];
        gathered[..amounts.len()].copy_from_slice(amounts);
        Self { gathered, dug: 0.0 }
    }
}

/// One agent's turn, in the book's order: move, metabolize, maybe die, then
/// (if still alive) mate with each neighbor, spread culture to them, trade,
/// borrow, and (rule E) train its immune system and pass on disease.
pub(crate) fn agent_turn(world: &mut World, id: AgentId) {
    // The book's worked example: a disease learned last turn is gone now,
    // before it can cost another fee or be passed on again.
    if world.config.disease.enabled && world.config.disease.cure == DiseaseCure::NextTick {
        disease::cure_immune(world, id);
    }
    let harvest = if world.config.combat.enabled {
        combat::act(world, id)
    } else {
        crate::minds::decide(world, id)
    };
    if world.config.memory.span > 0 {
        crate::minds::memory::observe(world, id);
    }
    // Minds 5: burying, after the move and harvest and before eating.
    if world.config.caching.buries() {
        crate::minds::caching::rules::act(world, id, &harvest);
    }
    lifecycle::metabolize(world, id, harvest);
    if world.config.credit.enabled {
        credit::record_income(world, id, &harvest);
    }
    if lifecycle::check_death(world, id) {
        return;
    }
    if world.config.sex.enabled {
        sex::act(world, id);
    }
    if world.config.culture.enabled {
        culture::act(world, id);
    }
    if world.config.trade.enabled {
        trade::act(world, id);
    }
    if world.config.credit.enabled {
        credit::borrow(world, id);
    }
    if world.config.disease.enabled {
        disease::act(world, id);
    }
}
