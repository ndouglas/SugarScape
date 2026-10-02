//! Ordinary holdings remain authoritative throughout homeward delivery.

use rand::Rng;

use super::{access, state::Delivery, stores};
use crate::agent::AgentId;
use crate::geometry::Pos;
use crate::minds::caching;
use crate::world::World;

pub(crate) fn clamped_delivery(old: Option<Delivery>, surplus: f64) -> Option<Delivery> {
    old.and_then(|d| {
        let amount = d.amount.min(surplus.max(0.0));
        (amount > 0.0).then_some(Delivery { amount })
    })
}

/// None may mean no feasible entrance; callers still consume a pending return turn.
pub(crate) fn delivery_target(world: &World, id: AgentId) -> Option<Pos> {
    world.agent(id)?.spatial.as_ref()?.delivery?;
    access::return_endpoint(world, id)
}

/// Also called after metabolism, so next tick's intent cannot spend consumed food.
pub(crate) fn clamp(world: &mut World, id: AgentId) {
    let old = world
        .agent(id)
        .and_then(|a| a.spatial.as_ref())
        .and_then(|s| s.delivery);
    let Some(_) = old else {
        return;
    };
    let new = clamped_delivery(old, caching::surplus(world, id));
    world
        .agent_mut(id)
        .expect("live agent")
        .spatial
        .as_mut()
        .expect("spatial agent")
        .delivery = new;
    if new.is_none() {
        stores::tick_events(world)
            .expect("enabled")
            .delivery
            .cancellations += 1;
    }
}

/// Post-action, pre-metabolism: complete at most one batch, without reallocating.
pub(crate) fn finish_turn(world: &mut World, id: AgentId) {
    let s = world
        .agent(id)
        .expect("live agent")
        .spatial
        .as_ref()
        .expect("spatial agent");
    if s.guarding {
        return;
    }
    // Capture before clamping: even a cancelled return consumed this action.
    let completion_turn = s.delivery.is_some();
    finish_batch(world, id, completion_turn);
}

fn finish_batch(world: &mut World, id: AgentId, completion_turn: bool) {
    clamp(world, id);
    if completion_turn {
        deposit_pending(world, id);
        return;
    }
    let a = world.agent(id).expect("live agent");
    if a.cheater {
        return;
    }
    let probability = a.spatial.as_ref().expect("spatial agent").traits.larder;
    let amount = (world.config.caching.share * caching::surplus(world, id))
        .min(a.holdings[0] / (1.0 + world.config.caching.bury_cost));
    if amount <= 0.0 {
        return;
    }
    let larder = probability >= 1.0 || (probability > 0.0 && world.rng.gen::<f64>() < probability);
    if !larder {
        caching::bury(world, id, amount);
        return;
    }
    world
        .agent_mut(id)
        .expect("live agent")
        .spatial
        .as_mut()
        .expect("spatial agent")
        .delivery = Some(Delivery { amount });
    stores::tick_events(world).expect("enabled").delivery.starts += 1;
    deposit_pending(world, id);
}

fn deposit_pending(world: &mut World, id: AgentId) {
    let a = world.agent(id).expect("live agent");
    let s = a.spatial.as_ref().expect("spatial agent");
    let Some(delivery) = s.delivery else {
        return;
    };
    if !access::in_contact(world, a.pos, s.home) {
        return;
    }
    // Deposit enforces cost affordability against current holdings, exactly once.
    let cost_before = world.events.bury_cost;
    let delivered = stores::deposit(world, id, delivery.amount.min(caching::surplus(world, id)));
    if delivered <= 0.0 {
        return;
    }
    world
        .agent_mut(id)
        .expect("live agent")
        .spatial
        .as_mut()
        .expect("spatial agent")
        .delivery = None;
    let cost = world.events.bury_cost - cost_before;
    let e = &mut stores::tick_events(world).expect("enabled").delivery;
    e.completions += 1;
    e.delivered += delivered;
    e.bury_cost += cost;
}
