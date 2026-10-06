use super::{knowledge::Knowledge, metrics::ComputeCounts, Checked, Pos};
use crate::config::FieldError;
use crate::foraging::FindRecord;

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
pub enum Phase {
    Departing,
    Searching,
    Returning,
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct Agent {
    pub(super) id: u32,
    pub(super) pos: Pos,
    pub(super) phase: Phase,
    pub(super) map: Knowledge,
    pub(super) cargo: Option<u64>,
    pub(super) find: Option<FindRecord>,
    pub(super) site: Option<Pos>,
    pub(super) frontier: Option<Pos>,
    pub(super) work: WorkCounts,
    pub(super) compute: ComputeCounts,
}

/// Authoritative physical work; computational work is counted separately.
#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize)]
pub struct WorkCounts {
    pub opportunities: u64,
    pub moves: u64,
    pub departure_moves: u64,
    pub search_moves: u64,
    pub empty_return_moves: u64,
    pub loaded_return_moves: u64,
    pub pickups: u64,
    pub deposits: u64,
    pub waits: u64,
    pub transition_waits: u64,
    pub congestion_waits: u64,
    pub no_neighbor_waits: u64,
    pub empty_arrival_waits: u64,
    pub search_entries: u64,
    pub empty_returns: u64,
    pub fidelity_departures: u64,
    pub recruited_departures: u64,
    pub uninformed_departures: u64,
    pub publications: u64,
    pub abandoned_targets: u64,
}
impl WorkCounts {
    pub(super) fn checked_include(&mut self, other: &Self) -> Checked<()> {
        let candidate = Self {
            opportunities: checked_sum("opportunities", self.opportunities, other.opportunities)?,
            moves: checked_sum("moves", self.moves, other.moves)?,
            departure_moves: checked_sum(
                "departure_moves",
                self.departure_moves,
                other.departure_moves,
            )?,
            search_moves: checked_sum("search_moves", self.search_moves, other.search_moves)?,
            empty_return_moves: checked_sum(
                "empty_return_moves",
                self.empty_return_moves,
                other.empty_return_moves,
            )?,
            loaded_return_moves: checked_sum(
                "loaded_return_moves",
                self.loaded_return_moves,
                other.loaded_return_moves,
            )?,
            pickups: checked_sum("pickups", self.pickups, other.pickups)?,
            deposits: checked_sum("deposits", self.deposits, other.deposits)?,
            waits: checked_sum("waits", self.waits, other.waits)?,
            transition_waits: checked_sum(
                "transition_waits",
                self.transition_waits,
                other.transition_waits,
            )?,
            congestion_waits: checked_sum(
                "congestion_waits",
                self.congestion_waits,
                other.congestion_waits,
            )?,
            no_neighbor_waits: checked_sum(
                "no_neighbor_waits",
                self.no_neighbor_waits,
                other.no_neighbor_waits,
            )?,
            empty_arrival_waits: checked_sum(
                "empty_arrival_waits",
                self.empty_arrival_waits,
                other.empty_arrival_waits,
            )?,
            search_entries: checked_sum(
                "search_entries",
                self.search_entries,
                other.search_entries,
            )?,
            empty_returns: checked_sum("empty_returns", self.empty_returns, other.empty_returns)?,
            fidelity_departures: checked_sum(
                "fidelity_departures",
                self.fidelity_departures,
                other.fidelity_departures,
            )?,
            recruited_departures: checked_sum(
                "recruited_departures",
                self.recruited_departures,
                other.recruited_departures,
            )?,
            uninformed_departures: checked_sum(
                "uninformed_departures",
                self.uninformed_departures,
                other.uninformed_departures,
            )?,
            publications: checked_sum("publications", self.publications, other.publications)?,
            abandoned_targets: checked_sum(
                "abandoned_targets",
                self.abandoned_targets,
                other.abandoned_targets,
            )?,
        };
        *self = candidate;
        Ok(())
    }

    pub(super) fn check(&self) -> Checked<()> {
        let equations = [
            (
                "opportunities",
                self.opportunities,
                &[self.moves, self.pickups, self.deposits, self.waits][..],
            ),
            (
                "moves",
                self.moves,
                &[
                    self.departure_moves,
                    self.search_moves,
                    self.empty_return_moves,
                    self.loaded_return_moves,
                ][..],
            ),
            (
                "waits",
                self.waits,
                &[
                    self.transition_waits,
                    self.congestion_waits,
                    self.no_neighbor_waits,
                    self.empty_arrival_waits,
                ][..],
            ),
        ];
        for (field, expected, parts) in equations {
            let sum = parts
                .iter()
                .try_fold(0, |total, part| checked_sum(field, total, *part))?;
            if sum != expected {
                return Err(vec![FieldError::new(
                    format!("work.{field}"),
                    "counter must equal the sum of its action categories",
                )]);
            }
        }
        Ok(())
    }
}
fn checked_sum(field: &str, left: u64, right: u64) -> Checked<u64> {
    left.checked_add(right)
        .ok_or_else(|| vec![FieldError::new(format!("work.{field}"), "counter overflow")])
}
