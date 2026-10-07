use super::{
    state::{Agent, Cargo},
    terrain::Terrain,
    Checked, Pos,
};
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub enum SpoilState {
    Carried { agent: u32 },
    Disposed { tick: u32 },
}
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub struct SpoilView {
    pub id: u64,
    pub origin: Pos,
    pub creator: u32,
    pub born_tick: u32,
    pub state: SpoilState,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
pub struct SpoilInventory {
    pub excavated: u32,
    pub carried: u32,
    pub disposed: u32,
}
// Staged Task 4 world spoil transactions.
#[allow(dead_code)]
#[derive(Clone, Debug, PartialEq)]
pub(super) struct SpoilLedger {
    records: Vec<SpoilView>,
    next_id: u64,
    capacity: u32,
}
// Staged Task 4 world spoil transactions.
#[allow(dead_code)]
impl SpoilLedger {
    pub(super) fn new(capacity: u32) -> Checked<Self> {
        if capacity > 15_625 {
            return Err(error(
                "spoil.capacity",
                "must not exceed 15625 initially solid masked cells",
            ));
        }
        Ok(Self {
            records: vec![],
            next_id: 0,
            capacity,
        })
    }
    pub(super) fn spawn(&mut self, origin: Pos, agent: u32, tick: u32) -> Checked<u64> {
        if self.records.len() >= self.capacity as usize {
            return Err(error(
                "spoil.capacity",
                "excavation exceeds initial solid mask capacity",
            ));
        }
        if self.records.iter().any(|r| r.origin == origin) {
            return Err(error("spoil.origin", "one token per excavated cell"));
        }
        if self
            .records
            .iter()
            .any(|r| r.state == SpoilState::Carried { agent })
        {
            return Err(error(
                "spoil.owner",
                "cannot create spoil while already carrying spoil",
            ));
        }
        let next = self
            .next_id
            .checked_add(1)
            .ok_or_else(|| error("spoil.next_id", "identity overflow"))?;
        let id = self.next_id;
        self.records.push(SpoilView {
            id,
            origin,
            creator: agent,
            born_tick: tick,
            state: SpoilState::Carried { agent },
        });
        self.next_id = next;
        Ok(id)
    }
    pub(super) fn dispose(&mut self, id: u64, agent: u32, tick: u32) -> Checked<()> {
        let index = self
            .records
            .binary_search_by_key(&id, |r| r.id)
            .map_err(|_| error(format!("spoil[{id}]"), "unknown spoil identity"))?;
        let record = &self.records[index];
        if record.state != (SpoilState::Carried { agent }) || record.creator != agent {
            return Err(error(
                format!("spoil[{id}].state"),
                "disposal requires carriage by its creator",
            ));
        }
        if tick < record.born_tick {
            return Err(error(
                format!("spoil[{id}].tick"),
                "disposal precedes creation",
            ));
        }
        self.records[index].state = SpoilState::Disposed { tick };
        Ok(())
    }
    pub(super) fn inventory(&self) -> SpoilInventory {
        let carried = self
            .records
            .iter()
            .filter(|r| matches!(r.state, SpoilState::Carried { .. }))
            .count() as u32;
        SpoilInventory {
            excavated: self.records.len() as u32,
            carried,
            disposed: self.records.len() as u32 - carried,
        }
    }
    pub(super) fn views(&self) -> &[SpoilView] {
        &self.records
    }
    pub(super) fn check(&self, terrain: &Terrain, agents: &[Agent]) -> Checked<()> {
        terrain.check()?;
        if self.capacity != terrain.capacity()
            || self.records.len() > self.capacity as usize
            || self.records.len() != terrain.counts().excavated as usize
            || self.next_id != self.records.len() as u64
        {
            return Err(error(
                "spoil.inventory",
                "tokens must match excavation count and initial capacity",
            ));
        }
        let mut origins = std::collections::BTreeSet::new();
        let mut owners = std::collections::BTreeSet::new();
        for (index, r) in self.records.iter().enumerate() {
            if r.id != index as u64
                || !origins.insert(r.origin)
                || !terrain.was_excavated(r.origin)?
            {
                return Err(error(
                    format!("spoil[{}].origin", r.id),
                    "token requires a distinct identity and uniquely excavated origin",
                ));
            }
            match r.state {
                SpoilState::Carried { agent } => {
                    if agent != r.creator
                        || !owners.insert(agent)
                        || agents
                            .iter()
                            .filter(|a| a.id == agent && a.cargo == Some(Cargo::Spoil(r.id)))
                            .count()
                            != 1
                    {
                        return Err(error(
                            format!("spoil[{}].state", r.id),
                            "carried spoil must match exactly one creator's tagged cargo",
                        ));
                    }
                }
                SpoilState::Disposed { tick } => {
                    if tick < r.born_tick {
                        return Err(error(
                            format!("spoil[{}].tick", r.id),
                            "disposal precedes creation",
                        ));
                    }
                }
            }
        }
        let mut agent_ids = std::collections::BTreeSet::new();
        for a in agents {
            if !agent_ids.insert(a.id) {
                return Err(error("agents", "duplicate agent identity"));
            }
            if let Some(Cargo::Spoil(id)) = a.cargo {
                if !self
                    .records
                    .iter()
                    .any(|r| r.id == id && r.state == SpoilState::Carried { agent: a.id })
                {
                    return Err(error(
                        format!("agents[{}].cargo", a.id),
                        "spoil cargo must match this agent's ledger assignment",
                    ));
                }
            }
        }
        Ok(())
    }
}
fn error(field: impl Into<String>, message: impl Into<String>) -> Vec<crate::config::FieldError> {
    vec![crate::config::FieldError::new(field, message)]
}
#[cfg(test)]
mod material_access {
    use super::super::{
        state::Cargo,
        tests::{pos, setup},
    };
    use super::*;
    #[test]
    fn spoil_id_overflow_is_atomic() {
        let mut sp = SpoilLedger {
            records: vec![],
            next_id: u64::MAX,
            capacity: 1,
        };
        let before = sp.clone();
        assert!(sp.spawn(pos(3, 0), 0, 0).is_err());
        assert_eq!(sp, before);
    }
    #[test]
    fn ledger_check_rejects_corrupted_origin_creator_identity_and_time() {
        let s = setup();
        let mut t = Terrain::new(&s).unwrap();
        t.dig(pos(3, 0)).unwrap();
        let mut sp = SpoilLedger::new(1).unwrap();
        sp.spawn(pos(3, 0), 0, 10).unwrap();
        let worker = super::super::tests::material_access::agent(0, Some(Cargo::Spoil(0)));
        for mode in 0..4 {
            let mut bad = sp.clone();
            match mode {
                0 => bad.records[0].origin = pos(2, 0),
                1 => bad.records[0].creator = 1,
                2 => bad.records[0].id = 9,
                _ => bad.records[0].state = SpoilState::Disposed { tick: 9 },
            };
            assert!(bad.check(&t, std::slice::from_ref(&worker)).is_err());
        }
    }
}
