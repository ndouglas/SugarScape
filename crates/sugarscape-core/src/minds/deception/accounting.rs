//! Parallel P4 lineage callbacks. Diagnostics cannot control physical actions.
use super::state::NOMINAL_AMOUNT;
use crate::{
    minds::protection::ledger::{Ledger, Outflow},
    world::World,
};

pub(crate) fn fail(w: &mut World, error: String) {
    if w.config.deception_lab.is_none() {
        return;
    }
    if let Some(r) = w.deception.as_mut().filter(|r| r.diagnostics) {
        r.ledger_errors.push(format!("tick {}: {error}", w.tick));
        r.ledger = None;
        r.diagnostics = false;
    }
}
pub(crate) fn update(w: &mut World, actor: u64, f: impl FnOnce(&mut Ledger) -> Result<(), String>) {
    if w.config.deception_lab.is_none() {
        return;
    }
    let Some(l) = w
        .deception
        .as_mut()
        .filter(|r| r.diagnostics)
        .and_then(|r| r.ledger.as_mut())
        .filter(|l| l.owner == actor)
    else {
        return;
    };
    if let Err(error) = f(l) {
        fail(w, format!("agent {actor}: {error}"));
    }
}
/// Labels only the first tick-zero original cache at the registered source.
pub(crate) fn is_original_deposit(w: &World, actor: u64, site: u32, q: f64) -> bool {
    w.config.deception_lab.is_some()
        && actor == 1
        && w.tick == 0
        && q == NOMINAL_AMOUNT
        && w.deception
            .as_ref()
            .is_some_and(|r| !r.prepared && r.source == site)
}
pub(crate) fn on_deposit(w: &mut World, actor: u64, site: u32, q: f64) {
    let original = is_original_deposit(w, actor, site, q);
    if original {
        w.deception.as_mut().expect("enabled runtime").prepared = true;
    }
    update(w, actor, |l| {
        if original {
            l.prepare(site, q)
        } else {
            l.outflow(q, Outflow::Deposit { site })
        }
    });
}
pub(crate) fn reconcile_world(w: &mut World) {
    if w.config.deception_lab.is_none() {
        return;
    }
    let Some(l) = w
        .deception
        .as_ref()
        .filter(|r| r.diagnostics)
        .and_then(|r| r.ledger.as_ref())
    else {
        return;
    };
    let result = if let Some(a) = w.agent(l.owner) {
        l.reconcile_physical(a.holdings[0], &a.caches)
    } else {
        l.reconcile()
    };
    if let Err(error) = result {
        fail(w, error);
    }
}
