use super::{Checked, Parameters};
use crate::config::FieldError;
use crate::foraging::{
    departure, publication, waypoint_strength, Departure, FindRecord, Waypoint, WaypointSelection,
    WaypointThreshold, WAYPOINT_THRESHOLD,
};

#[derive(Clone, Debug, PartialEq)]
struct Record {
    id: u64,
    site: u64,
    created_tick: u32,
}
#[derive(Clone, Debug, Default, PartialEq)]
pub(super) struct Server {
    records: Vec<Record>,
    next_id: u64,
    expired: u64,
}
#[derive(Clone, Debug, PartialEq)]
pub(super) struct Arrival {
    pub(super) departure: Departure,
    pub(super) published: bool,
}
/// Read-only advice, including weak records awaiting arrival-time expiry.
#[derive(Clone, Debug, PartialEq, serde::Serialize)]
pub struct ServerRecordView {
    pub id: u64,
    pub site: u64,
    pub created_tick: u32,
    pub strength: f64,
}
impl Server {
    pub(super) fn arrive(
        &mut self,
        parameters: &Parameters,
        tick: u32,
        capacity: u32,
        find: Option<FindRecord>,
        draws: [f64; 3],
    ) -> Checked<Arrival> {
        let views = self.views(parameters, tick)?;
        if capacity > 256 || self.records.len() > capacity as usize {
            return Err(vec![FieldError::new(
                "server.capacity",
                "retained records must fit the supplied food capacity in [0,256]",
            )]);
        }
        let p = parameters.information();
        let request = publication(&p, find, draws[0])?;
        // Validate both independent draws even when publication/fidelity makes them unused.
        departure(
            &p,
            find,
            &[],
            WaypointSelection::LaterArgosStrengthWeighted,
            WaypointThreshold::PaperBelow,
            draws[1],
            draws[2],
        )?;
        let mut pending = self.clone();
        if let Some(site) = request {
            let next_id = pending.next_id.checked_add(1).ok_or_else(|| {
                vec![FieldError::new(
                    "server.next_id",
                    "waypoint identity overflow",
                )]
            })?;
            pending.records.push(Record {
                id: pending.next_id,
                site,
                created_tick: tick,
            });
            pending.next_id = next_id;
        }
        // Original strengths correspond to original ordered records; a new record has strength one.
        let mut index = 0;
        pending.records.retain(|_| {
            let retain = views
                .get(index)
                .is_none_or(|view| retained_strength(view.strength));
            index += 1;
            retain
        });
        let removed = views
            .iter()
            .filter(|view| !retained_strength(view.strength))
            .count() as u64;
        pending.expired = pending.expired.checked_add(removed).ok_or_else(|| {
            vec![FieldError::new(
                "server.expired",
                "expiration counter overflow",
            )]
        })?;
        if pending.records.len() > capacity as usize {
            return Err(vec![FieldError::new(
                "server.records",
                "publication exceeds supplied food capacity",
            )]);
        }
        let waypoints = pending
            .views(parameters, tick)?
            .into_iter()
            .map(|view| Waypoint {
                id: view.id,
                site: view.site,
                strength: view.strength,
            })
            .collect::<Vec<_>>();
        let choice = departure(
            &p,
            find,
            &waypoints,
            WaypointSelection::LaterArgosStrengthWeighted,
            WaypointThreshold::PaperBelow,
            draws[1],
            draws[2],
        )?;
        *self = pending;
        Ok(Arrival {
            departure: choice,
            published: request.is_some(),
        })
    }
    pub(super) fn views(
        &self,
        parameters: &Parameters,
        tick: u32,
    ) -> Checked<Vec<ServerRecordView>> {
        parameters.validate()?;
        self.records
            .iter()
            .map(|record| {
                let age = tick.checked_sub(record.created_tick).ok_or_else(|| {
                    vec![FieldError::new(
                        "server.created_tick",
                        "message creation is after requested tick",
                    )]
                })?;
                Ok(ServerRecordView {
                    id: record.id,
                    site: record.site,
                    created_tick: record.created_tick,
                    strength: waypoint_strength(parameters.lambda_waypoint, f64::from(age))?,
                })
            })
            .collect()
    }
    pub(super) fn expired(&self) -> u64 {
        self.expired
    }
}

fn retained_strength(strength: f64) -> bool {
    strength >= WAYPOINT_THRESHOLD
}

#[cfg(test)]
mod tests {
    use super::super::tests::setup;
    use super::*;
    #[test]
    fn exact_strength_equality_survives_lazy_expiration() {
        assert!(retained_strength(0.001));
        assert!(retained_strength(f64::from_bits(0.001_f64.to_bits() + 1)));
        assert!(!retained_strength(f64::from_bits(0.001_f64.to_bits() - 1)));
    }
    #[test]
    fn maximal_identity_overflow_rolls_back_publication() {
        let parameters = setup().parameters;
        let mut server = Server {
            next_id: u64::MAX,
            ..Server::default()
        };
        let before = server.clone();
        assert!(server
            .arrive(
                &parameters,
                0,
                1,
                Some(FindRecord { site: 3, count: 1 }),
                [0.0; 3]
            )
            .is_err());
        assert_eq!(server, before);
    }
    #[test]
    fn expiration_overflow_rolls_back_record_removal() {
        let mut parameters = setup().parameters;
        parameters.lambda_waypoint = 10.0_f64.ln();
        let mut server = Server {
            records: vec![Record {
                id: 0,
                site: 3,
                created_tick: 0,
            }],
            next_id: 1,
            expired: u64::MAX,
        };
        let before = server.clone();
        assert!(server.arrive(&parameters, 4, 1, None, [0.0; 3]).is_err());
        assert_eq!(server, before);
    }
}
