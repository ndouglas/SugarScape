use super::Checked;
use crate::config::FieldError;
/// Computational diagnostics; queue peaks aggregate by maximum rather than sum.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize)]
pub struct ComputeCounts {
    pub observations: u64,
    pub cells_inspected: u64,
    pub cells_learned: u64,
    pub route_calls: u64,
    pub route_visits: u64,
    pub peak_queue: u64,
    pub frontier_scans: u64,
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
            route_calls: add(self.route_calls, other.route_calls, "route_calls")?,
            route_visits: add(self.route_visits, other.route_visits, "route_visits")?,
            peak_queue: self.peak_queue.max(other.peak_queue),
            frontier_scans: add(self.frontier_scans, other.frontier_scans, "frontier_scans")?,
        };
        *self = candidate;
        Ok(())
    }
}
