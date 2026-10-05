use super::Pos;
use crate::config::FieldError;
use crate::foraging::FindRecord;

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
pub enum Phase {
    Departing,
    Searching,
    Returning,
}
// Temporary staging allowance: world/controller consumers land in Task 4.
#[allow(dead_code)]
#[derive(Clone, Debug, PartialEq)]
pub(super) struct Agent {
    pub(super) id: u32,
    pub(super) pos: Pos,
    pub(super) heading: f64,
    pub(super) target: Pos,
    pub(super) phase: Phase,
    pub(super) informed: bool,
    pub(super) informed_turns: u32,
    pub(super) delay: u32,
    pub(super) cargo: Option<u64>,
    pub(super) find: Option<FindRecord>,
    pub(super) work: WorkCounts,
}
#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize)]
pub struct WorkCounts {
    pub opportunities: u64,
    pub waits: u64,
    pub directed_moves: u64,
    pub search_moves: u64,
    pub pickups: u64,
    pub deliveries: u64,
    pub empty_returns: u64,
    pub search_switches: u64,
    pub publications: u64,
    pub fidelity_departures: u64,
    pub recruited_departures: u64,
    pub uninformed_departures: u64,
}
// Temporary staging allowance: runner aggregation lands in Task 5.
#[allow(dead_code)]
impl WorkCounts {
    pub(super) fn checked_add_assign(&mut self, other: &Self) -> Result<(), Vec<FieldError>> {
        // Prepare all sums before mutation so overflow cannot leave a partial total.
        let sum = Self {
            opportunities: checked_sum("opportunities", self.opportunities, other.opportunities)?,
            waits: checked_sum("waits", self.waits, other.waits)?,
            directed_moves: checked_sum(
                "directed_moves",
                self.directed_moves,
                other.directed_moves,
            )?,
            search_moves: checked_sum("search_moves", self.search_moves, other.search_moves)?,
            pickups: checked_sum("pickups", self.pickups, other.pickups)?,
            deliveries: checked_sum("deliveries", self.deliveries, other.deliveries)?,
            empty_returns: checked_sum("empty_returns", self.empty_returns, other.empty_returns)?,
            search_switches: checked_sum(
                "search_switches",
                self.search_switches,
                other.search_switches,
            )?,
            publications: checked_sum("publications", self.publications, other.publications)?,
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
        };
        *self = sum;
        Ok(())
    }
}
fn checked_sum(field: &str, left: u64, right: u64) -> Result<u64, Vec<FieldError>> {
    left.checked_add(right)
        .ok_or_else(|| vec![FieldError::new(format!("work.{field}"), "counter overflow")])
}
