//! Owner-local, paid retrieval, walking and reburial actions.
use crate::{agent::AgentId, world::World};
pub(crate) fn perceived_exposure(world: &World, owner: AgentId) -> bool {
    let a = world.agent(owner).expect("live owner");
    world
        .sight(a.pos, a.vision)
        .into_iter()
        .any(|(pos, _)| world.occupant(pos).is_some_and(|id| id != owner))
}

use super::state::{CancelReason, Fixture, Intent, Policy, SourceEvent, Stage};
use crate::{
    geometry::Pos,
    minds::caching,
    rules::{
        movement::{self, WalkOutcome},
        Harvest,
    },
};

pub(crate) fn cancel(world: &mut World, owner: AgentId, reason: CancelReason) {
    let intent = world
        .agent_mut(owner)
        .and_then(|a| a.protection.as_mut())
        .and_then(|s| s.intent.take());
    if let Some(intent) = intent {
        if let Some(a) = world
            .protection_actions
            .last_mut()
            .filter(|a| a.id == owner)
        {
            a.phase = "relocation".into();
            a.action = "cancel".into();
            a.source = Some(intent.source);
        }
        if let Some(e) = &mut world.relocation_events {
            *e.cancellations.entry(reason).or_default() += 1;
            e.source_events.push(SourceEvent {
                source: intent.source,
                cancellation: Some(reason),
                ..Default::default()
            });
        }
    }
}

pub(crate) fn clamp_after_metabolism(world: &mut World, owner: AgentId) {
    let carried = world
        .agent(owner)
        .and_then(|a| a.protection.as_ref())
        .and_then(|s| s.intent.as_ref())
        .and_then(|i| matches!(i.stage, Stage::ToDestination | Stage::Deposit).then_some(i.amount));
    let Some(carried) = carried else {
        return;
    };
    let amount = carried.min(caching::surplus(world, owner));
    if amount <= 0.0 {
        cancel(world, owner, CancelReason::SurplusExhausted);
    } else {
        world
            .agent_mut(owner)
            .unwrap()
            .protection
            .as_mut()
            .unwrap()
            .intent
            .as_mut()
            .unwrap()
            .amount = amount;
    }
}

pub(crate) fn act(world: &mut World, owner: AgentId) -> Option<Harvest> {
    let lab = world.config.protection_lab.as_ref()?;
    let span = lab.exposure_span;
    let a = world.agent(owner)?;
    let state = a.protection.as_ref()?;
    if state.intent.is_none() {
        if matches!(lab.policy, Policy::Off)
            || a.holdings[0] < caching::reserve(world, owner)
            || perceived_exposure(world, owner)
        {
            return None;
        }
        let source = state
            .sources
            .iter()
            .filter(|(site, source)| {
                !source.attempted
                    && world.tick.saturating_sub(source.tick) <= span
                    && (matches!(lab.policy, Policy::Indiscriminate)
                        || state.exposure.entries.get(site).is_some_and(|e| {
                            e.exposed && world.tick.saturating_sub(e.tick) <= span
                        }))
            })
            .min_by_key(|(site, source)| (source.tick, **site))
            .map(|(&site, _)| site)?;
        let source_pos = world.torus.pos(source as usize);
        let destination = world
            .sight(a.pos, a.vision)
            .into_iter()
            .map(|(p, _)| p)
            .chain(std::iter::once(a.pos))
            .filter(|&p| {
                let site = world.torus.index(p) as u32;
                world.walls[site as usize] == 0
                    && (p == a.pos || !world.is_occupied(p))
                    && !a.caches.contains_key(&site)
                    && !state.sources.contains_key(&site)
                    && p != super::lab::transform(lab, Pos::new(3, 3))
                    && (!matches!(lab.fixture, Fixture::Mixed { .. })
                        || p != super::lab::transform(lab, Pos::new(5, 3)))
            })
            .min_by_key(|&p| {
                (
                    movement::lattice_distance(world.torus, source_pos, p),
                    world.torus.index(p),
                )
            })?;
        let at_source = a.pos == source_pos;
        let s = world.agent_mut(owner).unwrap().protection.as_mut().unwrap();
        s.sources.get_mut(&source).unwrap().attempted = true;
        s.intent = Some(Intent {
            source,
            destination,
            amount: 0.0,
            stage: if at_source {
                Stage::Retrieve
            } else {
                Stage::ToSource
            },
        });
        if let Some(e) = &mut world.relocation_events {
            e.starts += 1;
            e.source_events.push(SourceEvent {
                source,
                started: true,
                ..Default::default()
            });
        }
    }
    if let Some(e) = &mut world.relocation_events {
        e.action_ticks += 1;
    }
    let a = world.agent(owner).unwrap();
    let s = a.protection.as_ref().unwrap();
    let intent = s.intent.as_ref().unwrap().clone();
    if world.tick.saturating_sub(s.sources[&intent.source].tick) > span {
        cancel(world, owner, CancelReason::Expired);
        return Some(Harvest::default());
    }
    if let Some(a) = world
        .protection_actions
        .last_mut()
        .filter(|a| a.id == owner)
    {
        a.phase = "relocation".into();
        a.source = Some(intent.source);
        a.action = match intent.stage {
            Stage::ToSource | Stage::ToDestination => "walk",
            Stage::Retrieve => "retrieve",
            Stage::Deposit => "redeposit",
        }
        .into();
        a.target = Some(
            if matches!(intent.stage, Stage::ToSource | Stage::Retrieve) {
                world.torus.pos(intent.source as usize)
            } else {
                intent.destination
            },
        );
    }
    let a = world.agent(owner).unwrap();
    match intent.stage {
        Stage::ToSource | Stage::ToDestination => {
            let target = if matches!(intent.stage, Stage::ToSource) {
                world.torus.pos(intent.source as usize)
            } else {
                intent.destination
            };
            let before = a.pos;
            match movement::walk_without_gather(world, owner, target) {
                WalkOutcome::Unreachable => cancel(world, owner, CancelReason::Unreachable),
                WalkOutcome::Blocked => cancel(world, owner, CancelReason::Occupied),
                outcome => {
                    let after = world.agent(owner).unwrap().pos;
                    if let Some(e) = &mut world.relocation_events {
                        e.distance += movement::lattice_distance(world.torus, before, after);
                    }
                    if outcome == WalkOutcome::Arrived {
                        world
                            .agent_mut(owner)
                            .unwrap()
                            .protection
                            .as_mut()
                            .unwrap()
                            .intent
                            .as_mut()
                            .unwrap()
                            .stage = if matches!(intent.stage, Stage::ToSource) {
                            Stage::Retrieve
                        } else {
                            Stage::Deposit
                        };
                    }
                }
            }
        }
        Stage::Retrieve => {
            if a.pos != world.torus.pos(intent.source as usize) {
                cancel(world, owner, CancelReason::Unreachable);
            } else if !a.caches.get(&intent.source).is_some_and(|&q| q > 0.0) {
                cancel(world, owner, CancelReason::SourceMissing);
            } else {
                let cap = world.config.caching.capacity;
                let room = if cap == 0 {
                    f64::INFINITY
                } else {
                    (f64::from(cap) - a.holdings[0]).max(0.0)
                };
                if room <= 0.0 {
                    cancel(world, owner, CancelReason::NoRoom);
                } else {
                    let amount = caching::dig(world, owner, intent.source, room);
                    let a = world.agent_mut(owner).unwrap();
                    a.holdings[0] += amount;
                    let i = a.protection.as_mut().unwrap().intent.as_mut().unwrap();
                    i.amount = amount;
                    i.stage = Stage::ToDestination;
                    if let Some(e) = &mut world.relocation_events {
                        e.withdrawn += amount;
                        e.source_events.push(SourceEvent {
                            source: intent.source,
                            withdrawn: amount,
                            ..Default::default()
                        });
                    }
                }
            }
        }
        Stage::Deposit => {
            let site = world.torus.index(intent.destination);
            if a.pos != intent.destination
                || world.walls[site] != 0
                || world
                    .occupant(intent.destination)
                    .is_some_and(|id| id != owner)
                || a.caches.contains_key(&(site as u32))
            {
                cancel(world, owner, CancelReason::Occupied);
            } else if perceived_exposure(world, owner) {
                cancel(world, owner, CancelReason::WitnessVisible);
            } else {
                let amount = intent.amount.min(caching::surplus(world, owner));
                if amount <= 0.0 {
                    cancel(world, owner, CancelReason::SurplusExhausted);
                } else {
                    let before = world.events.bury_cost;
                    let deposited = caching::bury(world, owner, amount);
                    world
                        .agent_mut(owner)
                        .unwrap()
                        .protection
                        .as_mut()
                        .unwrap()
                        .intent = None;
                    if let Some(e) = &mut world.relocation_events {
                        e.completions += 1;
                        e.redeposited += deposited;
                        e.burial_cost += world.events.bury_cost - before;
                        e.source_events.push(SourceEvent {
                            source: intent.source,
                            redeposited: deposited,
                            ..Default::default()
                        });
                    }
                }
            }
        }
    }
    Some(Harvest::default())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        geometry::Pos,
        minds::{
            caching,
            protection::{
                state::Policy,
                tests_support::{rig, source_site},
            },
        },
    };
    #[test]
    fn protection_retrieval_walk_and_deposit_are_separate_actions() {
        let mut w = rig(Policy::Selective);
        assert_eq!(caching::bury(&mut w, 1, 12.0), 12.0);
        w.move_agent(2, Pos::new(7, 6));
        w.tick = 8;
        let source = source_site(&w);
        assert!(act(&mut w, 1).is_some());
        assert!(!w.agent(1).unwrap().caches.contains_key(&source));
        assert_eq!(w.agent(1).unwrap().holdings[0], 44.0);
        assert_eq!(w.agent(1).unwrap().pos, Pos::new(3, 3));
        assert!(act(&mut w, 1).is_some());
        assert_eq!(w.agent(1).unwrap().pos, Pos::new(3, 2));
        assert!(w.agent(1).unwrap().caches.is_empty());
        assert!(act(&mut w, 1).is_some());
        assert!(w
            .agent(1)
            .unwrap()
            .protection
            .as_ref()
            .unwrap()
            .intent
            .is_none());
        assert_eq!(w.agent(1).unwrap().holdings[0], 32.0);
    }
}

#[cfg(test)]
mod balance_tests {
    use super::*;
    use crate::minds::protection::{
        state::Policy,
        tests_support::{rig, source_site},
    };
    fn prepared() -> World {
        let mut w = rig(Policy::Selective);
        caching::bury(&mut w, 1, 12.0);
        w.move_agent(2, Pos::new(7, 6));
        w.tick = 8;
        w
    }
    fn reason(w: &World, r: CancelReason) {
        assert_eq!(
            w.relocation_events.as_ref().unwrap().cancellations.get(&r),
            Some(&1)
        );
        assert!(w
            .agent(1)
            .unwrap()
            .protection
            .as_ref()
            .unwrap()
            .intent
            .is_none());
    }
    #[test]
    fn protection_partial_capacity_withdraws_once_and_keeps_source_remainder() {
        let mut w = prepared();
        let site = source_site(&w);
        w.config.caching.capacity = 37;
        act(&mut w, 1);
        assert_eq!(w.agent(1).unwrap().holdings[0], 37.0);
        assert_eq!(w.agent(1).unwrap().caches[&site], 7.0);
        assert_eq!(
            w.agent(1)
                .unwrap()
                .protection
                .as_ref()
                .unwrap()
                .intent
                .as_ref()
                .unwrap()
                .amount,
            5.0
        );
    }
    #[test]
    fn protection_no_room_cancels_and_retains_stock() {
        let mut w = prepared();
        let source = source_site(&w);
        w.config.caching.capacity = 32;
        assert!(act(&mut w, 1).is_some());
        reason(&w, CancelReason::NoRoom);
        assert_eq!(w.agent(1).unwrap().holdings[0], 32.0);
        assert_eq!(w.agent(1).unwrap().caches[&source], 12.0);
    }
    #[test]
    fn protection_source_taken_before_arrival_cancels_at_retrieval() {
        let mut w = prepared();
        let source = source_site(&w);
        w.move_agent(1, Pos::new(3, 4));
        act(&mut w, 1);
        let taken = caching::dig(&mut w, 1, source, 12.0);
        assert_eq!(taken, 12.0);
        act(&mut w, 1);
        reason(&w, CancelReason::SourceMissing);
        assert_eq!(w.agent(1).unwrap().holdings[0], 32.0);
    }
    #[test]
    fn protection_metabolism_clamps_actual_carried_food_and_zero_surplus() {
        let mut w = prepared();
        act(&mut w, 1);
        let reserve = caching::reserve(&w, 1);
        w.agent_mut(1).unwrap().holdings[0] = reserve + 2.0;
        crate::rules::lifecycle::metabolize(&mut w, 1, Harvest::default());
        clamp_after_metabolism(&mut w, 1);
        assert_eq!(w.agent(1).unwrap().holdings[0], reserve + 1.0);
        assert_eq!(
            w.agent(1)
                .unwrap()
                .protection
                .as_ref()
                .unwrap()
                .intent
                .as_ref()
                .unwrap()
                .amount,
            1.0
        );
        crate::rules::lifecycle::metabolize(&mut w, 1, Harvest::default());
        clamp_after_metabolism(&mut w, 1);
        reason(&w, CancelReason::SurplusExhausted);
        assert_eq!(w.agent(1).unwrap().holdings[0], reserve);
    }
    #[test]
    fn protection_visible_witness_cancels_deposit_and_retains_holdings() {
        let mut w = prepared();
        act(&mut w, 1);
        act(&mut w, 1);
        w.move_agent(2, Pos::new(4, 2));
        act(&mut w, 1);
        reason(&w, CancelReason::WitnessVisible);
        assert_eq!(w.agent(1).unwrap().holdings[0], 44.0);
        assert!(w.agent(1).unwrap().caches.is_empty());
    }
    #[test]
    fn protection_deposit_pays_zero_positive_and_unaffordable_cost_without_duplicating_food() {
        for (cost, buried, left) in [(0.0, 12.0, 32.0), (0.25, 12.0, 29.0), (10.0, 4.0, 0.0)] {
            let mut w = prepared();
            w.config.caching.bury_cost = cost;
            act(&mut w, 1);
            act(&mut w, 1);
            act(&mut w, 1);
            assert_eq!(w.agent(1).unwrap().holdings[0], left);
            assert_eq!(w.agent(1).unwrap().caches.values().sum::<f64>(), buried);
            assert_eq!(left + buried + w.events.bury_cost, 44.0);
            assert!(act(&mut w, 1).is_none(), "one attempt, no chain");
        }
    }
    #[test]
    fn protection_expiry_during_intent_retains_food() {
        let mut w = prepared();
        act(&mut w, 1);
        w.tick = 65;
        act(&mut w, 1);
        reason(&w, CancelReason::Expired);
        assert_eq!(w.agent(1).unwrap().holdings[0], 44.0);
    }
    #[test]
    fn protection_owner_death_cancels_live_intent() {
        let mut w = prepared();
        act(&mut w, 1);
        w.kill(1, crate::world::DeathCause::OldAge);
        assert_eq!(
            w.relocation_events.as_ref().unwrap().cancellations[&CancelReason::OwnerDied],
            1
        );
        assert!(act(&mut w, 1).is_none());
    }
    #[test]
    fn protection_privacy_and_no_destination_do_not_start_or_exhaust_source() {
        let mut w = rig(Policy::Selective);
        caching::bury(&mut w, 1, 12.0);
        assert!(act(&mut w, 1).is_none());
        w.move_agent(2, Pos::new(7, 6));
        w.agent_mut(1).unwrap().vision = 0;
        assert!(act(&mut w, 1).is_none());
        assert!(
            !w.agent(1).unwrap().protection.as_ref().unwrap().sources[&source_site(&w)].attempted
        );
    }
    #[test]
    fn protection_blocked_destination_and_source_cancel() {
        let mut w = prepared();
        act(&mut w, 1);
        w.move_agent(2, Pos::new(3, 2));
        act(&mut w, 1);
        reason(&w, CancelReason::Occupied);
        assert_eq!(w.agent(1).unwrap().holdings[0], 44.0);
        let mut w = prepared();
        w.move_agent(1, Pos::new(3, 5));
        act(&mut w, 1);
        w.move_agent(2, Pos::new(3, 3));
        act(&mut w, 1);
        assert!(w
            .agent(1)
            .unwrap()
            .protection
            .as_ref()
            .unwrap()
            .intent
            .is_none());
    }
    #[test]
    fn protection_walks_leave_site_food_and_observer_memory_untouched() {
        let mut w = prepared();
        let source = source_site(&w);
        let old_seen = w.agent(2).unwrap().seen.clone();
        w.site_mut(Pos::new(3, 2)).resource[0] = 9.0;
        act(&mut w, 1);
        act(&mut w, 1);
        assert_eq!(w.site(Pos::new(3, 2)).resource[0], 9.0);
        assert_eq!(w.agent(1).unwrap().holdings[0], 44.0);
        assert_eq!(w.agent(2).unwrap().seen, old_seen);
        act(&mut w, 1);
        let e = w.relocation_events.as_ref().unwrap();
        assert_eq!(
            (e.starts, e.completions, e.action_ticks, e.distance),
            (1, 1, 3, 1)
        );
        assert_eq!((e.withdrawn, e.redeposited), (12.0, 12.0));
        assert_eq!(
            w.agent(1)
                .unwrap()
                .protection
                .as_ref()
                .unwrap()
                .sources
                .len(),
            1
        );
        assert!(w.agent(1).unwrap().protection.as_ref().unwrap().sources[&source].attempted);
    }
    #[test]
    fn protection_unreachable_source_consumes_action_and_does_not_take_stock() {
        let mut w = prepared();
        let source = source_site(&w);
        w.move_agent(1, Pos::new(3, 5));
        w.regions[source as usize] = u32::MAX;
        assert!(act(&mut w, 1).is_some());
        reason(&w, CancelReason::Unreachable);
        assert_eq!(w.agent(1).unwrap().caches[&source], 12.0);
        assert_eq!(w.agent(1).unwrap().holdings[0], 32.0);
    }
    #[test]
    fn protection_inclusive_freshness_and_policy_predicates() {
        for (policy, exposed, starts) in [
            (Policy::Off, true, false),
            (Policy::Selective, false, false),
            (Policy::Selective, true, true),
            (Policy::Erased, false, false),
            (Policy::Indiscriminate, false, true),
        ] {
            let mut w = prepared();
            w.config.protection_lab.as_mut().unwrap().policy = policy;
            let site = source_site(&w);
            w.agent_mut(1)
                .unwrap()
                .protection
                .as_mut()
                .unwrap()
                .exposure
                .entries
                .get_mut(&site)
                .unwrap()
                .exposed = exposed;
            w.tick = 64;
            assert_eq!(act(&mut w, 1).is_some(), starts);
        }
        let mut w = prepared();
        w.tick = 65;
        assert!(act(&mut w, 1).is_none());
    }
    #[test]
    fn protection_diagnostics_do_not_change_actions_or_hash() {
        let mut w = prepared();
        let mut unrecorded = w.clone();
        unrecorded.relocation_events = None;
        for _ in 0..3 {
            act(&mut w, 1);
            act(&mut unrecorded, 1);
            assert_eq!(w.fingerprint(), unrecorded.fingerprint());
            assert_eq!(
                w.agent(1).unwrap().holdings,
                unrecorded.agent(1).unwrap().holdings
            );
        }
    }
    #[test]
    fn protection_authoritative_state_and_live_settings_change_hash() {
        let w = prepared();
        let before = w.fingerprint();
        let mut changed = w.clone();
        changed.config.caching.bury_cost = 0.25;
        assert_ne!(before, changed.fingerprint());
        let mut changed = w.clone();
        changed.agent_mut(2).unwrap().watches = false;
        assert_ne!(before, changed.fingerprint());
        let mut changed = w.clone();
        changed.agent_mut(2).unwrap().seen.clear();
        assert_ne!(before, changed.fingerprint());
        let mut changed = w.clone();
        let source = source_site(&changed);
        changed
            .agent_mut(1)
            .unwrap()
            .protection
            .as_mut()
            .unwrap()
            .sources
            .get_mut(&source)
            .unwrap()
            .attempted = true;
        assert_ne!(before, changed.fingerprint());
        let mut changed = w.clone();
        act(&mut changed, 1);
        let with_intent = changed.fingerprint();
        changed
            .agent_mut(1)
            .unwrap()
            .protection
            .as_mut()
            .unwrap()
            .intent
            .as_mut()
            .unwrap()
            .amount -= 1.0;
        assert_ne!(with_intent, changed.fingerprint());
    }
    #[test]
    fn protection_checkpoint_keeps_mid_attempt_actions_and_hash() {
        let mut w = prepared();
        act(&mut w, 1);
        let mut model = crate::model::ModelWorld::Sugarscape(Box::new(w.clone()));
        let cp = model.checkpoint().unwrap();
        model.restore(&cp).unwrap();
        let mut restored = model.sugarscape().unwrap().clone();
        assert_eq!(w.fingerprint(), restored.fingerprint());
        for _ in 0..2 {
            act(&mut w, 1);
            act(&mut restored, 1);
            assert_eq!(w.fingerprint(), restored.fingerprint());
            assert_eq!(
                w.agent(1).unwrap().holdings,
                restored.agent(1).unwrap().holdings
            );
        }
    }
    #[test]
    fn protection_retrieve_never_withdraws_from_a_remote_position() {
        let mut w = prepared();
        let source = source_site(&w);
        w.move_agent(1, Pos::new(3, 4));
        act(&mut w, 1); // reaching the source only schedules retrieval
        w.move_agent(1, Pos::new(4, 3));
        act(&mut w, 1);
        reason(&w, CancelReason::Unreachable);
        assert_eq!(w.agent(1).unwrap().caches[&source], 12.0);
        assert_eq!(w.agent(1).unwrap().holdings[0], 32.0);
    }

    #[test]
    fn protection_off_lab_suppresses_generic_surplus_burial() {
        let mut w = rig(Policy::Off);
        w.tick = 8;
        w.config.caching.rule = crate::config::CachingRule::Even;
        w.agent_mut(1).unwrap().caching_rule = crate::config::CachingRule::Even;
        crate::rules::agent_turn(&mut w, 1);
        assert!(w.agent(1).unwrap().caches.is_empty());
        assert_eq!(w.events.buried, 0.0);
    }
    #[test]
    fn protection_destinations_exclude_own_caches_and_prepared_sources() {
        let mut w = prepared();
        for p in [Pos::new(3, 2), Pos::new(2, 3)] {
            let site = w.torus.index(p) as u32;
            w.agent_mut(1).unwrap().caches.insert(site, 1.0);
        }
        let source = w.torus.index(Pos::new(4, 3)) as u32;
        w.agent_mut(1)
            .unwrap()
            .protection
            .as_mut()
            .unwrap()
            .sources
            .insert(
                source,
                super::super::state::Source {
                    tick: 2,
                    initial_amount: 1.0,
                    attempted: true,
                },
            );
        act(&mut w, 1);
        assert_eq!(
            w.agent(1)
                .unwrap()
                .protection
                .as_ref()
                .unwrap()
                .intent
                .as_ref()
                .unwrap()
                .destination,
            Pos::new(3, 4)
        );
    }
    #[test]
    fn protection_deposit_rechecks_actual_position_and_retains_food() {
        let mut w = prepared();
        act(&mut w, 1);
        act(&mut w, 1);
        w.move_agent(1, Pos::new(4, 2));
        act(&mut w, 1);
        reason(&w, CancelReason::Occupied);
        assert!(w.agent(1).unwrap().caches.is_empty());
        assert_eq!(w.agent(1).unwrap().holdings[0], 44.0);
    }
    #[test]
    fn protection_source_order_is_oldest_tick_then_site() {
        for older in [false, true] {
            let mut w = prepared();
            let first = source_site(&w);
            let second = w.torus.index(Pos::new(5, 3)) as u32;
            let s = w.agent_mut(1).unwrap().protection.as_mut().unwrap();
            s.sources.get_mut(&first).unwrap().tick = u64::from(older);
            s.sources.insert(
                second,
                super::super::state::Source {
                    tick: 0,
                    initial_amount: 12.0,
                    attempted: false,
                },
            );
            s.exposure.remember(second, 0, true);
            act(&mut w, 1);
            assert_eq!(
                w.agent(1)
                    .unwrap()
                    .protection
                    .as_ref()
                    .unwrap()
                    .intent
                    .as_ref()
                    .unwrap()
                    .source,
                if older { second } else { first }
            );
        }
    }
    #[test]
    fn protection_below_reserve_does_not_start_but_equality_is_eligible() {
        let mut w = prepared();
        let source = source_site(&w);
        let reserve = caching::reserve(&w, 1);
        w.agent_mut(1).unwrap().holdings[0] = reserve - 1.0;
        assert!(act(&mut w, 1).is_none());
        let state = w.agent(1).unwrap().protection.as_ref().unwrap();
        assert!(!state.sources[&source].attempted);
        assert!(state.intent.is_none());
        let events = w.relocation_events.as_ref().unwrap();
        assert_eq!((events.starts, events.action_ticks), (0, 0));
        assert!(events.source_events.is_empty());
        assert_eq!(w.agent(1).unwrap().holdings[0], reserve - 1.0);
        assert_eq!(w.agent(1).unwrap().caches[&source], 12.0);

        w.agent_mut(1).unwrap().holdings[0] = reserve;
        assert!(act(&mut w, 1).is_some());
        let state = w.agent(1).unwrap().protection.as_ref().unwrap();
        assert!(state.sources[&source].attempted);
        assert!(state.intent.is_some());
        let events = w.relocation_events.as_ref().unwrap();
        assert_eq!((events.starts, events.action_ticks), (1, 1));
        assert_eq!(w.agent(1).unwrap().holdings[0], reserve + 12.0);
    }
}
