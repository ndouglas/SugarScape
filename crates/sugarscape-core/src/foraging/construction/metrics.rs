use super::Checked;
use crate::config::FieldError;
/// Computational diagnostics; queue peaks aggregate by maximum rather than sum.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize)]
pub(super) struct ComputeCounts {
    pub(super) observations: u64,
    pub(super) cells_inspected: u64,
    pub(super) cells_learned: u64,
    pub(super) observed_revisions: u64,
    pub(super) dig_confirmations: u64,
    pub(super) face_scans: u64,
    pub(super) route_calls: u64,
    pub(super) route_visits: u64,
    pub(super) peak_queue: u64,
    pub(super) frontier_scans: u64,
}
impl ComputeCounts {
    pub(super) fn checked_include(&mut self, other: &Self) -> Checked<()> {
        fn add(a: u64, b: u64, field: &str) -> Checked<u64> {
            a.checked_add(b).ok_or_else(|| {
                vec![FieldError::new(
                    format!("compute.{field}"),
                    "counter overflow",
                )]
            })
        }
        let candidate = Self {
            observations: add(self.observations, other.observations, "observations")?,
            cells_inspected: add(
                self.cells_inspected,
                other.cells_inspected,
                "cells_inspected",
            )?,
            cells_learned: add(self.cells_learned, other.cells_learned, "cells_learned")?,
            observed_revisions: add(
                self.observed_revisions,
                other.observed_revisions,
                "observed_revisions",
            )?,
            dig_confirmations: add(
                self.dig_confirmations,
                other.dig_confirmations,
                "dig_confirmations",
            )?,
            face_scans: add(self.face_scans, other.face_scans, "face_scans")?,
            route_calls: add(self.route_calls, other.route_calls, "route_calls")?,
            route_visits: add(self.route_visits, other.route_visits, "route_visits")?,
            peak_queue: self.peak_queue.max(other.peak_queue),
            frontier_scans: add(self.frontier_scans, other.frontier_scans, "frontier_scans")?,
        };
        *self = candidate;
        Ok(())
    }
}

/// Researcher-only connectivity work.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize)]
pub(super) struct AccessCompute {
    pub(super) calls: u64,
    pub(super) visits: u64,
    pub(super) peak_queue: u64,
}
impl AccessCompute {
    pub(super) fn checked_include(&mut self, other: &Self) -> Checked<()> {
        let calls = self
            .calls
            .checked_add(other.calls)
            .ok_or_else(|| vec![FieldError::new("access.calls", "counter overflow")])?;
        let visits = self
            .visits
            .checked_add(other.visits)
            .ok_or_else(|| vec![FieldError::new("access.visits", "counter overflow")])?;
        *self = Self {
            calls,
            visits,
            peak_queue: self.peak_queue.max(other.peak_queue),
        };
        Ok(())
    }
}
