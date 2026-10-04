//! Observed material histories, separate from controller inputs.

use super::{Action, ActionEvent, Outcome, World};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Delivery {
    pub material: u64,
    pub born: u64,
    pub disposed_at: Option<u64>,
    pub carriers: Vec<u32>,
    pub carried_moves: u64,
    pub waiting_ticks: u64,
}

#[derive(Clone, Debug)]
struct History {
    delivery: Delivery,
    loose_since: Option<u64>,
}

#[derive(Clone, Debug, Default)]
pub(super) struct Ledger {
    units: BTreeMap<u64, History>,
}

impl Ledger {
    pub(super) fn new(world: &World) -> Self {
        let units = world
            .units
            .values()
            .map(|unit| {
                (
                    unit.id,
                    History {
                        delivery: Delivery {
                            material: unit.id,
                            born: unit.born,
                            disposed_at: None,
                            carriers: Vec::new(),
                            carried_moves: 0,
                            waiting_ticks: 0,
                        },
                        // Birth before fixture start is supplied history, not observed waiting.
                        loose_since: Some(world.setup.start_tick),
                    },
                )
            })
            .collect();
        Self { units }
    }

    pub(super) fn record(&mut self, event: &ActionEvent, world: &World) {
        if event.outcome != Outcome::Success {
            return;
        }
        let Some(id) = event.material else {
            return;
        };
        let history = self.units.entry(id).or_insert_with(|| History {
            delivery: Delivery {
                material: id,
                born: world.units[&id].born,
                disposed_at: None,
                carriers: Vec::new(),
                carried_moves: 0,
                waiting_ticks: 0,
            },
            loose_since: None,
        });
        match event.action {
            Action::Dig(_) | Action::Pickup => {
                if let Some(start) = history.loose_since.take() {
                    history.delivery.waiting_ticks += event.tick - start;
                }
                if let Err(index) = history.delivery.carriers.binary_search(&event.worker) {
                    history.delivery.carriers.insert(index, event.worker);
                }
            }
            Action::Move(_) => history.delivery.carried_moves += 1,
            Action::Drop => history.loose_since = Some(event.tick),
            Action::Dispose => history.delivery.disposed_at = Some(event.tick),
            Action::Wait => {}
        }
    }

    pub(super) fn finish(&self, world: &World) -> Vec<Delivery> {
        self.units
            .values()
            .map(|history| {
                let mut delivery = history.delivery.clone();
                if let Some(start) = history.loose_since {
                    delivery.waiting_ticks += world.tick - start;
                }
                delivery
            })
            .collect()
    }
}
