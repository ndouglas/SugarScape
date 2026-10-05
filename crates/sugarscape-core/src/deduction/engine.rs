use super::*;
use rand::{Rng, RngCore};
use serde::Serialize;
#[derive(Clone, Debug, Serialize)]
pub(crate) struct AgentState {
    pub(crate) status: Status,
    pub(crate) grants: Vec<CapabilityId>,
    pub(crate) budget: u16,
    pub(crate) objective: ObjectiveTeam,
    pub(crate) attention: Vec<AgentId>,
    pub(crate) memory: Vec<VisibleEvent>,
    pub(crate) next_sequence: u64,
}
#[derive(Clone, Debug, Serialize)]
pub(crate) struct PendingEffect {
    pub(crate) due: Round,
    pub(crate) created: Round,
    pub(crate) source: AgentId,
    pub(crate) target: AgentId,
    pub(crate) effect: Effect,
}
#[derive(Clone)]
pub struct Engine {
    pub(crate) config: ScenarioConfig,
    pub(crate) seed: u64,
    pub(crate) rng: crate::rng::SimRng,
    pub(crate) agents: Vec<AgentState>,
    pub(crate) round: Round,
    pub(crate) phase: Phase,
    pub(crate) actors: Vec<AgentId>,
    pub(crate) cursor: usize,
    pub(crate) buffer: Vec<TurnResponse>,
    pub(crate) pending: Vec<PendingEffect>,
    pub(crate) responses: Vec<TurnResponse>,
    pub(crate) outcome: Option<Outcome>,
    pub(crate) assigned_holder: Option<AgentId>,
}
impl Engine {
    pub fn new(mut config: ScenarioConfig, seed: u64) -> Result<Self, Vec<FieldError>> {
        config.validate()?;
        config.normalize();
        let mut rng = crate::rng::seeded(seed);
        let mut agents: Vec<_> = config
            .agents
            .iter()
            .map(|a| AgentState {
                status: Status::Active,
                grants: a.grants.clone(),
                budget: if config.accusation.eligible.contains(&a.id) {
                    config.accusation.budget
                } else {
                    0
                },
                objective: a.objective,
                attention: vec![],
                memory: vec![],
                next_sequence: 0,
            })
            .collect();
        let mut assigned_holder = None;
        if let Some(a) = &config.assignment {
            let holder = a.eligible[rng.gen_range(0..a.eligible.len())];
            assigned_holder = Some(holder);
            for (id, state) in agents.iter_mut().enumerate() {
                state.grants.retain(|c| *c != a.capability);
                state.objective = if id == usize::from(holder) {
                    ObjectiveTeam::Threat
                } else {
                    ObjectiveTeam::Accuser
                };
            }
            agents[usize::from(holder)].grants.push(a.capability);
            agents[usize::from(holder)].grants.sort_unstable();
        }
        let actors = (0..agents.len() as AgentId).collect();
        Ok(Self {
            config,
            seed,
            rng,
            agents,
            round: 0,
            phase: Phase::Attention,
            actors,
            cursor: 0,
            buffer: vec![],
            pending: vec![],
            responses: vec![],
            outcome: None,
            assigned_holder,
        })
    }
    pub fn request(&self) -> Option<TurnRequest> {
        if self.outcome.is_some() {
            return None;
        }
        let actor = *self.actors.get(self.cursor)?;
        let a = &self.agents[usize::from(actor)];
        let targets = self
            .agents
            .iter()
            .enumerate()
            .filter(|(id, a)| *id != usize::from(actor) && a.status.is_active())
            .map(|(id, _)| id as AgentId)
            .collect();
        let can_speak = a.status == Status::Active;
        Some(TurnRequest {
            protocol_version: PROTOCOL_VERSION,
            request_id: self.responses.len() as u64,
            actor,
            round: self.round,
            phase: self.phase,
            observation: Observation {
                roster: self
                    .agents
                    .iter()
                    .enumerate()
                    .map(|(id, a)| RosterEntry {
                        id: id as AgentId,
                        status: a.status,
                    })
                    .collect(),
                grants: a.grants.clone(),
                accusation_budget: a.budget,
                events: a.memory.clone(),
                capabilities: self.config.capabilities.clone(),
                objectives: self.config.objectives.clone(),
                rules: PublicRules {
                    display_name: self.config.display_name.clone(),
                    max_rounds: self.config.max_rounds,
                    observation: self.config.observation.clone(),
                    memory: self.config.memory.clone(),
                    accusation: self.config.accusation.clone(),
                },
                objective: a.objective,
            },
            legal: LegalActions {
                targets,
                capabilities: if self.phase == Phase::Act {
                    a.grants.clone()
                } else {
                    vec![]
                },
                attention_capacity: self.config.observation.attention_capacity,
                accusation_budget: a.budget,
                can_watch: self.phase == Phase::Attention,
                can_use: self.phase == Phase::Act,
                can_say: self.phase == Phase::Discuss && can_speak,
                can_accuse: self.phase == Phase::Discuss
                    && can_speak
                    && self.config.accusation.enabled
                    && a.budget > 0,
            },
        })
    }
    pub fn submit(&mut self, response: TurnResponse) -> Result<(), ActionError> {
        self.validate_response(&response)?;
        self.accept_response(response);
        self.advance_until_request_or_outcome();
        Ok(())
    }
    fn validate_response(&self, response: &TurnResponse) -> Result<(), ActionError> {
        let invalid = ActionError::InvalidResponse;
        let r = self.request().ok_or(invalid)?;
        if response.actor != r.actor || response.request_id != r.request_id {
            return Err(invalid);
        }
        let known = |id: AgentId| usize::from(id) < self.agents.len();
        let ok = match &response.action {
            Action::Pass => true,
            Action::Watch { agents } => {
                r.legal.can_watch
                    && agents.len() <= usize::from(r.legal.attention_capacity)
                    && agents.iter().all(|id| r.legal.targets.contains(id))
                    && agents
                        .iter()
                        .collect::<std::collections::BTreeSet<_>>()
                        .len()
                        == agents.len()
            }
            Action::Use { capability, target } => {
                r.legal.can_use
                    && r.legal.capabilities.contains(capability)
                    && self
                        .config
                        .capabilities
                        .iter()
                        .find(|c| c.id == *capability)
                        .is_some_and(|c| match c.target {
                            TargetRule::SelfOnly => *target == r.actor,
                            TargetRule::OtherActive => r.legal.targets.contains(target),
                        })
            }
            Action::Accuse { target } => r.legal.can_accuse && r.legal.targets.contains(target),
            Action::Say { claim } => {
                r.legal.can_say
                    && match claim {
                        Claim::Suspect { agent } => known(*agent),
                        Claim::SawUse {
                            source,
                            target,
                            round,
                        } => known(*source) && known(*target) && *round <= r.round,
                        Claim::DenyUse { round } => *round <= r.round,
                    }
            }
        };
        if ok {
            Ok(())
        } else {
            Err(invalid)
        }
    }
    fn accept_response(&mut self, response: TurnResponse) {
        self.buffer.push(response.clone());
        self.responses.push(response);
        self.cursor += 1;
    }
    fn advance_until_request_or_outcome(&mut self) {
        while self.cursor >= self.actors.len() && self.outcome.is_none() {
            self.resolve_boundary();
            if self.outcome.is_some() {
                break;
            }
            self.phase = match self.phase {
                Phase::Attention => Phase::Act,
                Phase::Act => Phase::Discuss,
                Phase::Discuss => {
                    self.round += 1;
                    self.prune_memories();
                    for a in &mut self.agents {
                        a.attention.clear();
                    }
                    Phase::Attention
                }
            };
            self.actors = self
                .agents
                .iter()
                .enumerate()
                .filter(|(_, a)| a.status.is_active())
                .map(|(id, _)| id as AgentId)
                .collect();
            self.cursor = 0;
            self.buffer.clear();
        }
    }
    fn resolve_boundary(&mut self) {
        match self.phase {
            Phase::Attention => {
                for r in &self.buffer {
                    self.agents[usize::from(r.actor)].attention = match &r.action {
                        Action::Watch { agents } => agents.clone(),
                        _ => vec![],
                    };
                }
            }
            Phase::Act => {
                let mut uses = vec![];
                let mut evidence = vec![];
                for r in &self.buffer {
                    if let Action::Use { capability, target } = r.action {
                        let c = self
                            .config
                            .capabilities
                            .iter()
                            .find(|c| c.id == capability)
                            .expect("validated capability");
                        evidence.push((r.actor, target, capability, c.visibility));
                        uses.push(PendingEffect {
                            due: self.round + c.delay,
                            created: self.round,
                            source: r.actor,
                            target,
                            effect: c.effect.clone(),
                        });
                    }
                }
                for (source, target, capability, visibility) in evidence {
                    self.observe_use(source, target, capability, visibility);
                }
                self.pending.extend(uses);
                self.pending.sort_by_key(|e| (e.created, e.source));
                let pending = std::mem::take(&mut self.pending);
                for effect in pending {
                    if effect.due <= self.round {
                        self.apply_effect(effect);
                    } else {
                        self.pending.push(effect);
                    }
                }
            }
            Phase::Discuss => {
                let claims: Vec<_> = self
                    .buffer
                    .iter()
                    .filter_map(|r| {
                        if let Action::Say { claim } = &r.action {
                            Some(EventContent::Claim {
                                source: r.actor,
                                claim: claim.clone(),
                            })
                        } else {
                            None
                        }
                    })
                    .collect();
                for claim in claims {
                    self.publish(claim);
                }
                let holders: Vec<_> = self
                    .agents
                    .iter()
                    .map(|a| a.grants.contains(&self.config.objectives.capability))
                    .collect();
                let accusations: Vec<_> = self
                    .buffer
                    .iter()
                    .filter_map(|r| {
                        if let Action::Accuse { target } = r.action {
                            Some((r.actor, target))
                        } else {
                            None
                        }
                    })
                    .collect();
                for (source, target) in accusations {
                    self.agents[usize::from(source)].budget -= 1;
                    if holders[usize::from(target)] {
                        self.agents[usize::from(target)]
                            .grants
                            .retain(|c| *c != self.config.objectives.capability);
                    } else if self.config.accusation.wrong_cost == WrongCost::DeactivateAccuser {
                        self.agents[usize::from(source)].status = Status::Inactive;
                        self.publish(EventContent::StatusChange {
                            agent: source,
                            status: Status::Inactive,
                        });
                    }
                }
                self.evaluate_outcome();
            }
        }
    }
    fn apply_effect(&mut self, p: PendingEffect) {
        let a = &mut self.agents[usize::from(p.target)];
        if !a.status.is_active() {
            return;
        }
        match p.effect {
            Effect::SetStatus { status } => {
                if a.status != status {
                    a.status = status;
                    self.publish(EventContent::StatusChange {
                        agent: p.target,
                        status,
                    });
                }
            }
            Effect::Grant { capability } => {
                if !a.grants.contains(&capability) {
                    a.grants.push(capability);
                    a.grants.sort_unstable();
                }
            }
            Effect::Revoke { capability } => a.grants.retain(|c| *c != capability),
        }
    }
    fn evaluate_outcome(&mut self) {
        let active: Vec<_> = self
            .agents
            .iter()
            .filter(|a| a.status.is_active())
            .collect();
        let holders = active
            .iter()
            .filter(|a| a.grants.contains(&self.config.objectives.capability))
            .count();
        let completed = self.round + 1;
        let result = if holders == 0 {
            Some((ObjectiveTeam::Accuser, OutcomeReason::NoActiveThreatHolders))
        } else if active.len() - holders
            <= usize::from(self.config.objectives.non_holder_survivor_threshold)
        {
            Some((ObjectiveTeam::Threat, OutcomeReason::SurvivorThreshold))
        } else if completed >= self.config.max_rounds {
            Some((
                self.config.objectives.horizon_winner,
                OutcomeReason::Horizon,
            ))
        } else {
            None
        };
        self.outcome = result.map(|(winner, reason)| Outcome {
            winner,
            reason,
            completed_rounds: completed,
        });
    }
    pub fn outcome(&self) -> Option<&Outcome> {
        self.outcome.as_ref()
    }
    pub fn archive(&self) -> ReplayArchive {
        ReplayArchive {
            protocol_version: PROTOCOL_VERSION,
            rules_version: RULES_VERSION,
            config: self.config.clone(),
            seed: self.seed,
            responses: self.responses.clone(),
            fingerprint: self.fingerprint(),
        }
    }
    pub fn fingerprint(&self) -> u64 {
        let mut rng = self.rng.clone();
        let continuation = [
            rng.next_u64(),
            rng.next_u64(),
            rng.next_u64(),
            rng.next_u64(),
        ];
        let bytes = serde_json::to_vec(&(
            &self.config,
            self.seed,
            &self.responses,
            self.assigned_holder,
            self.round,
            self.phase,
            self.cursor,
            &self.actors,
            &self.buffer,
            &self.agents,
            &self.pending,
            &self.outcome,
            continuation,
        ))
        .expect("serializable engine state");
        bytes.iter().fold(0xcbf29ce484222325u64, |hash, b| {
            (hash ^ u64::from(*b)).wrapping_mul(0x100000001b3)
        })
    }
}
