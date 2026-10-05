use super::{Pos, Setup};
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
// Temporary staging allowance: world/controller consumers land in Task 4.
#[allow(dead_code)]
#[derive(Clone, Debug, Default, PartialEq)]
pub(super) struct Server {
    records: Vec<Record>,
    next_id: u64,
    expired: u64,
}
/// Read-only retained message, including weak records awaiting arrival-time expiration.
#[derive(Clone, Debug, PartialEq, serde::Serialize)]
pub struct WaypointView {
    pub id: u64,
    pub site: Pos,
    pub created_tick: u32,
    pub strength: f64,
}
// Temporary staging allowance: arrival and snapshot consumers land in Tasks 4–5.
#[allow(dead_code)]
impl Server {
    pub(super) fn arrive(
        &mut self,
        setup: &Setup,
        tick: u32,
        find: Option<FindRecord>,
        draws: [f64; 3],
    ) -> Result<(Departure, bool), Vec<FieldError>> {
        // Validate original inputs before preparing any publication or expiration.
        let views = self.views(setup, tick)?;
        if let Some(find) = find {
            setup.position(find.site)?;
        }
        let publish = publication(&setup.parameters, find, draws[0])?;
        departure(
            &setup.parameters,
            find,
            &[],
            WaypointSelection::LaterArgosStrengthWeighted,
            WaypointThreshold::PaperBelow,
            draws[1],
            draws[2],
        )?;
        let mut pending = self.clone();
        if let Some(site) = publish {
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
        // Preserve order and retain threshold equality, as the F1 paper policy does.
        let mut index = 0;
        pending.records.retain(|_| {
            let retain = views
                .get(index)
                .is_none_or(|v| v.strength >= WAYPOINT_THRESHOLD);
            index += 1;
            retain
        });
        let removed = views
            .iter()
            .filter(|v| v.strength < WAYPOINT_THRESHOLD)
            .count() as u64;
        pending.expired = pending.expired.checked_add(removed).ok_or_else(|| {
            vec![FieldError::new(
                "server.expired",
                "expiration counter overflow",
            )]
        })?;
        let snapshot = pending
            .views(setup, tick)?
            .into_iter()
            .map(|v| Waypoint {
                id: v.id,
                site: setup.site(v.site),
                strength: v.strength,
            })
            .collect::<Vec<_>>();
        let choice = departure(
            &setup.parameters,
            find,
            &snapshot,
            WaypointSelection::LaterArgosStrengthWeighted,
            WaypointThreshold::PaperBelow,
            draws[1],
            draws[2],
        )?;
        *self = pending;
        Ok((choice, publish.is_some()))
    }
    pub(super) fn views(
        &self,
        setup: &Setup,
        tick: u32,
    ) -> Result<Vec<WaypointView>, Vec<FieldError>> {
        setup.validate()?;
        if self.records.len() > setup.resources.len() {
            return Err(vec![FieldError::new(
                "server.records",
                "retained messages exceed resource capacity",
            )]);
        }
        self.records
            .iter()
            .map(|r| {
                let age = tick.checked_sub(r.created_tick).ok_or_else(|| {
                    vec![FieldError::new(
                        "server.created_tick",
                        "message creation is after requested tick",
                    )]
                })?;
                Ok(WaypointView {
                    id: r.id,
                    site: setup.position(r.site)?,
                    created_tick: r.created_tick,
                    strength: waypoint_strength(setup.parameters.lambda_waypoint, f64::from(age))?,
                })
            })
            .collect()
    }
    pub(super) fn expired(&self) -> u64 {
        self.expired
    }
}

#[cfg(test)]
mod tests {
    use super::super::tests::setup;
    use super::*;

    #[test]
    fn identity_overflow_does_not_publish() {
        let mut setup = setup();
        setup.resources.push(super::super::Resource {
            id: 9,
            pos: Pos { x: 1, y: 1 },
        });
        let mut server = Server {
            next_id: u64::MAX,
            ..Server::default()
        };
        let before = server.clone();
        assert!(server
            .arrive(&setup, 0, Some(FindRecord { site: 6, count: 1 }), [0.0; 3])
            .is_err());
        assert_eq!(server, before);
    }
    #[test]
    fn expiration_overflow_does_not_remove_messages() {
        let mut setup = setup();
        setup.resources.push(super::super::Resource {
            id: 9,
            pos: Pos { x: 1, y: 1 },
        });
        setup.parameters.lambda_waypoint = 10.0_f64.ln();
        let mut server = Server {
            records: vec![Record {
                id: 0,
                site: 6,
                created_tick: 0,
            }],
            next_id: 1,
            expired: u64::MAX,
        };
        let before = server.clone();
        assert!(server.arrive(&setup, 4, None, [0.0; 3]).is_err());
        assert_eq!(server, before);
    }
}
