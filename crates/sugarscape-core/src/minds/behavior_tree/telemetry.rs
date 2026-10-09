//! Optional observations of actual work; never a controller input.
use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkCounters {
    pub node_visits: u64,
    pub candidate_evaluations: u64,
    pub target_selections: u64,
    pub path_queries: u64,
    pub search_expansions: Option<u64>,
    pub search_unavailable_reason: Option<String>,
    pub fallback_short: u64,
    pub fallback_limit: u64,
}
impl Default for WorkCounters {
    fn default() -> Self {
        Self {
            node_visits: 0,
            candidate_evaluations: 0,
            target_selections: 0,
            path_queries: 0,
            search_expansions: Some(0),
            search_unavailable_reason: None,
            fallback_short: 0,
            fallback_limit: 0,
        }
    }
}
pub(crate) fn note_candidates(w: &mut crate::world::World, count: u64) {
    if let Some(c) = w.bt_work.as_mut() {
        c.candidate_evaluations += count;
    }
}
pub(crate) fn note_selection(w: &mut crate::world::World) {
    if let Some(c) = w.bt_work.as_mut() {
        c.target_selections += 1;
    }
}
pub(crate) fn note_path(w: &mut crate::world::World) {
    if let Some(c) = w.bt_work.as_mut() {
        c.path_queries += 1;
    }
}
pub(crate) fn note_search(
    w: &mut crate::world::World,
    expanded: Option<u64>,
    unavailable: Option<&str>,
) {
    if let Some(c) = w.bt_work.as_mut() {
        match expanded {
            Some(n) => {
                if let Some(total) = c.search_expansions.as_mut() {
                    *total += n;
                }
            }
            None => {
                c.search_expansions = None;
                c.search_unavailable_reason = unavailable.map(str::to_owned);
            }
        }
    }
}
pub(crate) fn reset(w: &mut crate::world::World) {
    if let Some(c) = w.bt_work.as_mut() {
        *c = WorkCounters::default();
    }
}
