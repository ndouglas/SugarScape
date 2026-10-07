use super::{
    controller::{check, occupancy},
    decision::Policy,
    draws::PcgDraws,
    knowledge::Knowledge,
    ledger::Ledger,
    metrics::ComputeCounts,
    observation::observe,
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
/// Seeded passage world. A step commits all workers and the RNG together.
#[derive(Clone)]
pub struct World {
    pub(super) setup: Setup,
    pub(super) state: State,
    pub(super) rng: crate::rng::SimRng,
}
impl From<&Setup> for Policy {
    fn from(setup: &Setup) -> Self {
        Self {
            width: setup.width,
            height: setup.height,
            nest: setup.nest.clone(),
            parameters: setup.parameters.clone(),
        }
    }
}
impl World {
    /// Validates and normalizes inputs, then learns ordinary spawn observations without draws.
    pub fn new(setup: Setup, seed: u64) -> Result<Self, Vec<FieldError>> {
        let setup = setup.normalized()?;
        let agents = setup
            .workers
            .iter()
            .enumerate()
            .map(|(id, &pos)| {
                Ok(Agent {
                    id: id as u32,
                    pos,
                    phase: Phase::Departing,
                    map: Knowledge::new(setup.width, setup.height)?,
                    cargo: None,
                    find: None,
                    site: None,
                    frontier: None,
                    work: WorkCounts {
                        uninformed_departures: 1,
                        ..Default::default()
                    },
                    compute: ComputeCounts::default(),
                })
            })
            .collect::<Result<Vec<_>, Vec<FieldError>>>()?;
        let mut state = State {
            tick: 0,
            agents,
            ledger: Ledger::new(&setup),
            server: Server::default(),
            first_pickup_tick: None,
            first_delivery_tick: None,
            all_delivered_tick: None,
        };
        let occupants = occupancy(&state);
        let food = state.ledger.available();
        for agent in &mut state.agents {
            let observation = observe(&setup, agent.pos, &occupants, &food)?;
            let learned = agent.map.learn(&observation)?;
            agent.compute.checked_include(&ComputeCounts {
                observations: 1,
                cells_inspected: observation.cells.len() as u64,
                cells_learned: learned,
                ..Default::default()
            })?;
        }
        check(&setup, &state)?;
        Ok(Self {
            setup,
            state,
            rng: crate::rng::seeded(seed),
        })
    }
    /// Advances one complete ordered tick, preserving state and RNG on every error.
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
