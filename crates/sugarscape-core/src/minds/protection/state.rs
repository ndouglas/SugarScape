//! Authoritative local event memory and relocation state.
use crate::geometry::Pos;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Policy {
    Off,
    Selective,
    Indiscriminate,
    Erased,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Fixture {
    Single {
        initial_observed: bool,
        redeposit_observed: bool,
    },
    Mixed {
        observed_first: bool,
    },
    CueVisibleNonwatcher,
    CueUnseenWatcher,
    Stumble {
        initial_observed: bool,
    },
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LabConfig {
    pub policy: Policy,
    pub fixture: Fixture,
    pub mirrored: bool,
    pub reburial_cost: f64,
    pub discovery: f64,
    pub exposure_span: u64,
    pub observer_span: u32,
}
impl Default for LabConfig {
    fn default() -> Self {
        Self {
            policy: Policy::Off,
            fixture: Fixture::Single {
                initial_observed: true,
                redeposit_observed: false,
            },
            mirrored: false,
            reburial_cost: 0.25,
            discovery: 0.0,
            exposure_span: 64,
            observer_span: 64,
        }
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Exposure {
    pub tick: u64,
    pub exposed: bool,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Source {
    pub tick: u64,
    pub initial_amount: f64,
    pub attempted: bool,
}
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ExposureMemory {
    pub entries: BTreeMap<u32, Exposure>,
}
impl ExposureMemory {
    pub fn remember(&mut self, site: u32, tick: u64, exposed: bool) {
        self.entries.insert(site, Exposure { tick, exposed });
        if self.entries.len() > crate::minds::memory::MEMORY_CAP {
            let drop = self
                .entries
                .iter()
                .min_by_key(|(site, e)| (e.tick, **site))
                .map(|(site, _)| *site)
                .unwrap();
            self.entries.remove(&drop);
        }
    }
    pub fn sweep(&mut self, now: u64, span: u64) {
        self.entries
            .retain(|_, e| now.saturating_sub(e.tick) <= span);
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Stage {
    ToSource,
    Retrieve,
    ToDestination,
    Deposit,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Intent {
    pub source: u32,
    pub destination: Pos,
    pub amount: f64,
    pub stage: Stage,
}
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ProtectionState {
    pub sources: BTreeMap<u32, Source>,
    pub exposure: ExposureMemory,
    pub intent: Option<Intent>,
}
