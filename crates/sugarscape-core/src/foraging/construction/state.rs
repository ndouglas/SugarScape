use super::{knowledge::Knowledge, metrics::ComputeCounts, Checked, Pos};
/// One carrying slot; food and spoil identities use separate tagged namespaces.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
pub enum Cargo {
    Food(u64),
    Spoil(u64),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
pub enum FoodPhase {
    Departing,
    Searching,
    Returning,
}
// Staged Task 4 coupled worker state.
#[allow(dead_code)]
#[derive(Clone, Debug, PartialEq)]
pub(super) struct Agent {
    pub(super) id: u32,
    pub(super) pos: Pos,
    pub(super) phase: FoodPhase,
    pub(super) map: Knowledge,
    pub(super) cargo: Option<Cargo>,
    pub(super) find: Option<crate::foraging::FindRecord>,
    pub(super) site: Option<Pos>,
    pub(super) frontier: Option<Pos>,
    pub(super) face: Option<Pos>,
    pub(super) work: WorkCounts,
    pub(super) compute: ComputeCounts,
}
/// Paid physical actions; computational diagnostics are counted separately.
#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize)]
pub struct WorkCounts {
    pub opportunities: u64,
    pub moves: u64,
    pub digs: u64,
    pub pickups: u64,
    pub deposits: u64,
    pub disposals: u64,
    pub waits: u64,
    pub departure_moves: u64,
    pub search_moves: u64,
    pub empty_return_moves: u64,
    pub food_moves: u64,
    pub spoil_moves: u64,
    pub transition_waits: u64,
    pub empty_arrival_waits: u64,
    pub no_neighbor_waits: u64,
    pub empty_congestion_waits: u64,
    pub food_congestion_waits: u64,
    pub spoil_congestion_waits: u64,
    pub search_entries: u64,
    pub empty_returns: u64,
    pub fidelity_departures: u64,
    pub recruited_departures: u64,
    pub uninformed_departures: u64,
    pub publications: u64,
    pub abandoned_targets: u64,
    pub spoil_hauls: u64,
}
// Staged Task 4 worker/controller state and physical-accounting consumers.
#[allow(dead_code)]
impl WorkCounts {
    pub(super) fn checked_include(&mut self, other: &Self) -> Checked<()> {
        let candidate = Self {
            opportunities: checked_sum("opportunities", self.opportunities, other.opportunities)?,
            moves: checked_sum("moves", self.moves, other.moves)?,
            digs: checked_sum("digs", self.digs, other.digs)?,
            pickups: checked_sum("pickups", self.pickups, other.pickups)?,
            deposits: checked_sum("deposits", self.deposits, other.deposits)?,
            disposals: checked_sum("disposals", self.disposals, other.disposals)?,
            waits: checked_sum("waits", self.waits, other.waits)?,
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
            food_moves: checked_sum("food_moves", self.food_moves, other.food_moves)?,
            spoil_moves: checked_sum("spoil_moves", self.spoil_moves, other.spoil_moves)?,
            transition_waits: checked_sum(
                "transition_waits",
                self.transition_waits,
                other.transition_waits,
            )?,
            empty_arrival_waits: checked_sum(
                "empty_arrival_waits",
                self.empty_arrival_waits,
                other.empty_arrival_waits,
            )?,
            no_neighbor_waits: checked_sum(
                "no_neighbor_waits",
                self.no_neighbor_waits,
                other.no_neighbor_waits,
            )?,
            empty_congestion_waits: checked_sum(
                "empty_congestion_waits",
                self.empty_congestion_waits,
                other.empty_congestion_waits,
            )?,
            food_congestion_waits: checked_sum(
                "food_congestion_waits",
                self.food_congestion_waits,
                other.food_congestion_waits,
            )?,
            spoil_congestion_waits: checked_sum(
                "spoil_congestion_waits",
                self.spoil_congestion_waits,
                other.spoil_congestion_waits,
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
            spoil_hauls: checked_sum("spoil_hauls", self.spoil_hauls, other.spoil_hauls)?,
        };
        *self = candidate;
        Ok(())
    }
    pub(super) fn check(&self) -> Checked<()> {
        for (field, expected, parts) in [
            (
                "opportunities",
                self.opportunities,
                &[
                    self.moves,
                    self.digs,
                    self.pickups,
                    self.deposits,
                    self.disposals,
                    self.waits,
                ][..],
            ),
            (
                "moves",
                self.moves,
                &[
                    self.departure_moves,
                    self.search_moves,
                    self.empty_return_moves,
                    self.food_moves,
                    self.spoil_moves,
                ][..],
            ),
            (
                "waits",
                self.waits,
                &[
                    self.transition_waits,
                    self.empty_arrival_waits,
                    self.no_neighbor_waits,
                    self.empty_congestion_waits,
                    self.food_congestion_waits,
                    self.spoil_congestion_waits,
                ][..],
            ),
        ] {
            let sum = parts
                .iter()
                .try_fold(0, |sum, part| checked_sum(field, sum, *part))?;
            if sum != expected {
                return Err(vec![crate::config::FieldError::new(
                    format!("work.{field}"),
                    "counter must equal the sum of its action categories",
                )]);
            }
        }
        Ok(())
    }
}
fn checked_sum(field: &str, left: u64, right: u64) -> Checked<u64> {
    left.checked_add(right).ok_or_else(|| {
        vec![crate::config::FieldError::new(
            format!("work.{field}"),
            "counter overflow",
        )]
    })
}
