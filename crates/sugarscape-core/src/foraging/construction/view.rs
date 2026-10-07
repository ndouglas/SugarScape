use super::{
    AccessSummary, Cargo, ComputeCounts, FoodInventory, FoodPhase, FoodView, KnownCell, Milestones,
    Mode, Pos, SpoilInventory, SpoilView, TerrainInventory, WorkCounts, World,
};
use crate::config::FieldError;

/// One worker's sorted private beliefs about topology and diggability.
/// Remembered diggable walls may be stale until locally observed again.
/// No occupancy or food history is retained.
#[derive(Clone, Debug, PartialEq, serde::Serialize)]
pub struct KnowledgeView {
    pub agent: u32,
    pub cells: Vec<KnownCell>,
}
/// Frozen local food density at the successful pickup site.
#[derive(Clone, Debug, PartialEq, serde::Serialize)]
pub struct FindView {
    pub site: Pos,
    pub count: u32,
}
/// Retained advice evaluated at the current tick without lazy expiration.
#[derive(Clone, Debug, PartialEq, serde::Serialize)]
pub struct WaypointView {
    pub id: u64,
    pub site: Pos,
    pub created_tick: u32,
    pub strength: f64,
}
#[derive(Clone, Debug, PartialEq, serde::Serialize)]
pub struct AgentView {
    pub id: u32,
    pub pos: Pos,
    pub phase: FoodPhase,
    pub mode: Mode,
    pub cargo: Option<Cargo>,
    pub find: Option<FindView>,
    pub site: Option<Pos>,
    pub frontier: Option<Pos>,
    pub face: Option<Pos>,
    pub known_open: u32,
    pub known_solid: u32,
    pub known_diggable: u32,
    pub work: WorkCounts,
    pub compute: ComputeCounts,
}
/// Checked physical and computational totals. Events use processing ticks starting
/// at zero; completed ticks count commits. Missing milestones are censored.
#[derive(Clone, Debug, PartialEq, serde::Serialize)]
pub struct Summary {
    pub completed_ticks: u32,
    pub food: FoodInventory,
    pub spoil: SpoilInventory,
    pub terrain: TerrainInventory,
    pub work: WorkCounts,
    pub compute: ComputeCounts,
    pub per_agent_work: Vec<WorkCounts>,
    pub per_agent_compute: Vec<ComputeCounts>,
    pub expired_records: u64,
    pub access: AccessSummary,
    pub milestones: Milestones,
}
/// Researcher geometry, physical state and small per-worker knowledge counts.
/// Full private maps are available separately through [`World::knowledge`].
#[derive(Clone, Debug, PartialEq, serde::Serialize)]
pub struct Snapshot {
    pub summary: Summary,
    pub open: Vec<Pos>,
    pub nest: Vec<Pos>,
    pub waste: Pos,
    pub agents: Vec<AgentView>,
    pub food: Vec<FoodView>,
    pub spoil: Vec<SpoilView>,
    pub waypoints: Vec<WaypointView>,
}
impl World {
    /// Recomputes checked totals without draws, observations or server expiry.
    pub fn summary(&self) -> Result<Summary, Vec<FieldError>> {
        super::controller::check(&self.setup, &self.state)?;
        let mut work = WorkCounts::default();
        let mut compute = ComputeCounts::default();
        let mut per_agent_work = Vec::with_capacity(self.state.agents.len());
        let mut per_agent_compute = Vec::with_capacity(self.state.agents.len());
        for agent in &self.state.agents {
            work.checked_include(&agent.work)?;
            compute.checked_include(&agent.compute)?;
            per_agent_work.push(agent.work.clone());
            per_agent_compute.push(agent.compute);
        }
        Ok(Summary {
            completed_ticks: self.state.tick,
            food: self.state.food.inventory(),
            spoil: self.state.spoil.inventory(),
            terrain: self.state.terrain.counts(),
            work,
            compute,
            per_agent_work,
            per_agent_compute,
            expired_records: self.state.server.expired(),
            access: self.state.access.summary()?,
            milestones: self.state.milestones.clone(),
        })
    }
    /// Returns a bounded single-worker map, validating the worker identity.
    pub fn knowledge(&self, agent: u32) -> Result<KnowledgeView, Vec<FieldError>> {
        let worker = self.state.agents.get(agent as usize).ok_or_else(|| {
            vec![FieldError::new(
                format!("knowledge.agent[{agent}]"),
                "must identify an existing worker",
            )]
        })?;
        super::controller::check(&self.setup, &self.state)?;
        Ok(KnowledgeView {
            agent,
            cells: worker.map.known(),
        })
    }
    /// Returns observational geometry and state, preserving maps, counters and RNG.
    /// Weak waypoint records remain until a returning worker accesses the server.
    pub fn snapshot(&self) -> Result<Snapshot, Vec<FieldError>> {
        let summary = self.summary()?;
        let agents = self
            .state
            .agents
            .iter()
            .map(|agent| {
                let (known_open, known_solid, known_diggable) = agent.map.counts();
                Ok(AgentView {
                    id: agent.id,
                    pos: agent.pos,
                    phase: agent.phase,
                    mode: agent.mode(),
                    cargo: agent.cargo,
                    find: agent
                        .find
                        .map(|find| -> Result<FindView, Vec<FieldError>> {
                            Ok(FindView {
                                site: self.setup.position(find.site)?,
                                count: find.count,
                            })
                        })
                        .transpose()?,
                    site: agent.site,
                    frontier: agent.frontier,
                    face: agent.face,
                    known_open,
                    known_solid,
                    known_diggable,
                    work: agent.work.clone(),
                    compute: agent.compute,
                })
            })
            .collect::<Result<Vec<_>, Vec<FieldError>>>()?;
        let waypoints = self
            .state
            .server
            .views(&self.setup.parameters, self.state.tick)?
            .into_iter()
            .map(|record| {
                Ok(WaypointView {
                    id: record.id,
                    site: self.setup.position(record.site)?,
                    created_tick: record.created_tick,
                    strength: record.strength,
                })
            })
            .collect::<Result<Vec<_>, Vec<FieldError>>>()?;
        Ok(Snapshot {
            summary,
            open: self.state.terrain.open_positions(),
            nest: self.setup.nest.clone(),
            waste: self.setup.waste,
            agents,
            food: self.state.food.views().to_vec(),
            spoil: self.state.spoil.views().to_vec(),
            waypoints,
        })
    }
}
