use super::{
    access::{AccessObserver, Milestones},
    food::FoodLedger,
    server::Server,
    spoil::SpoilLedger,
    state::Agent,
    terrain::Terrain,
    Setup,
};
#[derive(Clone, Debug, PartialEq)]
pub(super) struct State {
    pub(super) tick: u32,
    pub(super) terrain: Terrain,
    pub(super) agents: Vec<Agent>,
    pub(super) food: FoodLedger,
    pub(super) spoil: SpoilLedger,
    pub(super) server: Server,
    pub(super) access: AccessObserver,
    pub(super) milestones: Milestones,
}
#[derive(Clone)]
pub struct World {
    pub(super) setup: Setup,
    pub(super) state: State,
    pub(super) rng: crate::rng::SimRng,
}
impl World {
    /// Validates/normalizes inputs and observes spawn cells without consuming RNG.
    pub fn new(setup: Setup, seed: u64) -> Result<Self, Vec<crate::config::FieldError>> {
        let setup = setup.normalized()?;
        let terrain = Terrain::new(&setup)?;
        let food = FoodLedger::new(&setup, &terrain)?;
        let spoil = SpoilLedger::new(terrain.capacity())?;
        let access = AccessObserver::new(&setup, &terrain, &food)?;
        let agents = setup
            .workers
            .iter()
            .enumerate()
            .map(|(id, &pos)| {
                Ok(Agent {
                    id: id as u32,
                    pos,
                    phase: super::FoodPhase::Departing,
                    map: super::knowledge::Knowledge::new(setup.width, setup.height)?,
                    cargo: None,
                    find: None,
                    site: None,
                    frontier: None,
                    face: None,
                    work: super::WorkCounts {
                        uninformed_departures: 1,
                        ..Default::default()
                    },
                    compute: super::metrics::ComputeCounts::default(),
                })
            })
            .collect::<Result<Vec<_>, Vec<crate::config::FieldError>>>()?;
        let mut state = State {
            tick: 0,
            terrain,
            agents,
            food,
            spoil,
            server: Server::default(),
            access,
            milestones: Milestones::default(),
        };
        let occupants = super::controller::occupancy(&state);
        let food = state.food.available();
        for agent in &mut state.agents {
            let observation =
                super::observation::observe(&state.terrain, agent.pos, &occupants, &food)?;
            let delta = agent.map.learn(&observation)?;
            agent
                .compute
                .checked_include(&super::metrics::ComputeCounts {
                    observations: 1,
                    cells_inspected: observation.cells.len() as u64,
                    cells_learned: delta.first,
                    observed_revisions: delta.observed_revisions,
                    dig_confirmations: delta.dig_confirmations,
                    ..Default::default()
                })?;
        }
        super::controller::check(&setup, &state)?;
        Ok(Self {
            setup,
            state,
            rng: crate::rng::seeded(seed),
        })
    }
    /// Commits one complete ascending-ID tick together with its seeded RNG.
    pub fn step(&mut self) -> Result<(), Vec<crate::config::FieldError>> {
        let mut candidate = self.clone();
        super::controller::advance(
            &candidate.setup,
            &mut candidate.state,
            &mut super::draws::PcgDraws(&mut candidate.rng),
        )?;
        *self = candidate;
        Ok(())
    }
}
