//! Fixed founder traits and episode-local stores, intentions and observations.

use std::collections::BTreeMap;

use crate::agent::AgentId;
use crate::geometry::Pos;
use serde::{Deserialize, Serialize};

/// A deposit actually seen at a foreign home, without current-stock knowledge.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct SeenLarder {
    pub home: Pos,
    pub amount: f64,
    pub tick: u64,
}

/// Runner-only sensitivity switches; both ordinary defaults are false.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct EpisodeProbe {
    pub guard_harvest: bool,
    pub scatter_first: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FounderTraits {
    pub larder: f64,
    pub defense: f64,
    pub cheater: bool,
    pub watches: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum StoreKind {
    Scatter,
    Larder,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Delivery {
    pub amount: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SpatialState {
    pub home: Pos,
    pub traits: FounderTraits,
    pub larder: f64,
    pub larder_since: Option<u64>,
    pub delivery: Option<Delivery>,
    pub guarding: bool,
    pub seen_larders: BTreeMap<AgentId, SeenLarder>,
}

impl SpatialState {
    pub(crate) fn new(home: Pos, traits: FounderTraits) -> Self {
        Self {
            home,
            traits,
            larder: 0.0,
            larder_since: None,
            delivery: None,
            guarding: false,
            seen_larders: BTreeMap::new(),
        }
    }
}
