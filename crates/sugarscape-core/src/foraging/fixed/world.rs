use super::{
    draws::{checked_uniform, PcgDraws},
    ledger::Ledger,
    movement::edge_target,
    server::Server,
    state::Agent,
    Phase, Setup, WorkCounts,
};
use crate::config::FieldError;
#[derive(Clone, Debug, PartialEq)]
pub(super) struct State {
    pub(super) tick: u32,
    pub(super) agents: Vec<Agent>,
    pub(super) ledger: Ledger,
    pub(super) server: Server,
    pub(super) first_pickup_tick: Option<u32>,
    pub(super) first_delivery_tick: Option<u32>,
    pub(super) all_delivered_tick: Option<u32>,
}
/// Seeded fixed-world foraging with atomic, stable-ID ordered ticks.
#[derive(Clone)]
pub struct World {
    pub(super) setup: Setup,
    pub(super) state: State,
    pub(super) rng: crate::rng::SimRng,
}
impl World {
    /// Validates the explicit setup before drawing initial headings and targets.
    pub fn new(setup: Setup, seed: u64) -> Result<Self, Vec<FieldError>> {
        setup.validate()?;
        let mut rng = crate::rng::seeded(seed);
        let mut draws = PcgDraws(&mut rng);
        let mut agents = Vec::new();
        for id in 0..setup.agents {
            agents.push(Agent {
                id,
                pos: setup.nest,
                heading: checked_uniform(&mut draws)? * std::f64::consts::TAU,
                target: edge_target(&setup, &mut draws)?,
                phase: Phase::Departing,
                informed: false,
                informed_turns: 0,
                delay: 0,
                cargo: None,
                find: None,
                work: WorkCounts {
                    uninformed_departures: 1,
                    ..WorkCounts::default()
                },
            });
        }
        let ledger = Ledger::new(&setup);
        Ok(Self {
            setup,
            state: State {
                tick: 0,
                agents,
                ledger,
                server: Server::default(),
                first_pickup_tick: None,
                first_delivery_tick: None,
                all_delivered_tick: None,
            },
            rng,
        })
    }
    /// Commits a complete tick, or preserves both state and random stream.
    pub fn step(&mut self) -> Result<(), Vec<FieldError>> {
        let mut candidate = self.clone();
        super::controller::advance(
            &candidate.setup,
            &mut candidate.state,
            &mut PcgDraws(&mut candidate.rng),
        )?;
        *self = candidate;
        Ok(())
    }
}

impl World {
    /// Returns an observational view; does not draw or expire server messages.
    pub fn snapshot(&self) -> Result<super::Snapshot, Vec<FieldError>> {
        super::controller::check(&self.setup, &self.state)?;
        let mut work = WorkCounts::default();
        let agents = self
            .state
            .agents
            .iter()
            .map(|a| {
                work.checked_add_assign(&a.work)?;
                let find = a
                    .find
                    .map(|find| -> Result<super::FindView, Vec<FieldError>> {
                        Ok(super::FindView {
                            site: self.setup.position(find.site)?,
                            count: find.count,
                        })
                    })
                    .transpose()?;
                Ok(super::AgentView {
                    id: a.id,
                    pos: a.pos,
                    heading: a.heading,
                    target: a.target,
                    phase: a.phase,
                    informed: a.informed,
                    informed_turns: a.informed_turns,
                    delay: a.delay,
                    cargo: a.cargo,
                    find,
                    work: a.work.clone(),
                })
            })
            .collect::<Result<Vec<_>, Vec<FieldError>>>()?;
        Ok(super::Snapshot {
            completed_ticks: self.state.tick,
            inventory: self.state.ledger.inventory(),
            agents,
            resources: self.state.ledger.views().to_vec(),
            waypoints: self.state.server.views(&self.setup, self.state.tick)?,
            expired_records: self.state.server.expired(),
            work,
            first_pickup_tick: self.state.first_pickup_tick,
            first_delivery_tick: self.state.first_delivery_tick,
            all_delivered_tick: self.state.all_delivered_tick,
        })
    }
}
