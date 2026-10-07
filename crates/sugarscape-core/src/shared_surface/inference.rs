use std::collections::BTreeMap;

use super::{
    decide, protocol::remaining_credits, validate_action, Belief, Checkpoint, Decision,
    Environment, EpisodeBits, Error, Knowledge, LocalEntry, LocalPrefix, LocalView, Mechanism,
    Outcome, OwnPrior, Phase, Position, PriorMode, Probability, Protocol, Role, World,
    INITIAL_CREDITS, MAX_CANDIDATE_WORLDS,
};

/// Immutable full-prefix posteriors built by one synchronized chronological pass.
/// Keys contain role, opaque IDs, supplied own prior, public clock and all local entries.
/// No actual environment or another Agent's realized private history is an input.
pub struct Ensemble {
    protocol: Protocol,
    schedule: Vec<(Role, Position)>,
    beliefs: BTreeMap<LocalPrefix, Belief>,
    decisions: BTreeMap<LocalPrefix, Decision>,
    candidate_count: usize,
}

struct Candidate {
    model: Mechanism,
    latent_bits: EpisodeBits,
    world: World,
    prefixes: [LocalPrefix; 2],
    credits: [u8; 2],
    private_bits: [Option<bool>; 2],
    // Stale-mode excluded models retain domain records but never contribute or run.
    positive_mass: bool,
}

#[derive(Default)]
struct Masses {
    models: [u64; 4],
    target_true: u64,
    total: u64,
}

impl Ensemble {
    pub fn build(protocol: &Protocol) -> Result<Self, Error> {
        let schedule = protocol.schedule()?;
        let mut candidates = Vec::with_capacity(MAX_CANDIDATE_WORLDS);
        for model in Mechanism::ALL {
            for sequence in 0..256 {
                candidates.push(Candidate {
                    model, latent_bits: EpisodeBits::from_index(sequence)?,
                    world: World::new(Environment::InFamily(model), protocol.ids.clone()),
                    prefixes: [Role::A, Role::B].map(|role| LocalPrefix {
                        role, ids: protocol.ids.clone(), checkpoint: Checkpoint::EpisodeStart,
                        own_prior: candidate_prior(protocol, role, model), entries: Vec::new(),
                    }),
                    credits: [INITIAL_CREDITS; 2], private_bits: [None; 2],
                    positive_mass: !matches!(protocol.prior_mode, PriorMode::Stale(old) if old != model),
                });
            }
        }
        let mut ensemble = Self {
            protocol: protocol.clone(),
            schedule: schedule.clone(),
            beliefs: BTreeMap::new(),
            decisions: BTreeMap::new(),
            candidate_count: candidates.len(),
        };
        ensemble.cache_checkpoint(&mut candidates, Checkpoint::EpisodeStart)?;
        let mut phase = None;
        for (role, position) in schedule {
            if phase != Some(position.phase) {
                for candidate in candidates.iter_mut().filter(|c| c.positive_mass) {
                    candidate.start_phase(position.phase);
                }
                phase = Some(position.phase);
            }
            ensemble.cache_checkpoint(&mut candidates, Checkpoint::Action(position))?;
            // All beliefs were grouped before any candidate acts. Candidate peer priors
            // are delta at that candidate's model, never at an external actual model.
            for candidate in candidates.iter_mut().filter(|c| c.positive_mass) {
                let view = candidate.local_view(role, &ensemble.beliefs)?;
                let decision = match ensemble.decisions.entry(view.prefix.clone()) {
                    std::collections::btree_map::Entry::Occupied(entry) => entry.into_mut(),
                    std::collections::btree_map::Entry::Vacant(entry) => {
                        entry.insert(decide(&view, protocol.pair.roles[role.index()])?)
                    }
                };
                candidate.apply_local_decision(role, position, &decision.action)?;
            }
            // Cache after the paid outcome, before a round reset or next private bit.
            ensemble.cache_checkpoint(&mut candidates, Checkpoint::AfterSlot(position))?;
            if position.slot == 4 {
                for candidate in candidates.iter_mut().filter(|c| c.positive_mass) {
                    candidate.world.finish_round();
                }
            }
            if let Phase::Live { trial } = position.phase {
                if position.round == 3 && position.slot == 4 {
                    ensemble.cache_checkpoint(&mut candidates, Checkpoint::Prediction { trial })?;
                }
            }
        }
        Ok(ensemble)
    }

    /// Return the posterior from original world counts, never a likelihood multiplier.
    pub fn infer(&self, prefix: &LocalPrefix) -> Result<Belief, Error> {
        self.validate_prefix(prefix)?;
        self.beliefs
            .get(prefix)
            .cloned()
            .ok_or_else(|| Error::UnsupportedHistory {
                prefix: prefix.clone(),
            })
    }

    /// Retained family domain (4 mechanisms × 256 latent sequences). Stale priors give
    /// only 256 records positive mass; zero-mass records are never replayed or grouped.
    pub fn candidate_count(&self) -> usize {
        self.candidate_count
    }

    fn cache_checkpoint(
        &mut self,
        candidates: &mut [Candidate],
        checkpoint: Checkpoint,
    ) -> Result<(), Error> {
        for candidate in candidates.iter_mut().filter(|c| c.positive_mass) {
            for prefix in &mut candidate.prefixes {
                prefix.checkpoint = checkpoint;
            }
        }
        for role in [Role::A, Role::B] {
            let groups = group_local_prefixes(candidates, role)?;
            for (prefix, masses) in groups {
                let target = match checkpoint {
                    Checkpoint::Action(Position {
                        phase: Phase::Live { .. },
                        ..
                    })
                    | Checkpoint::AfterSlot(Position {
                        phase: Phase::Live { .. },
                        ..
                    })
                    | Checkpoint::Prediction { .. } => {
                        Some(Probability::new(masses.target_true, masses.total)?)
                    }
                    _ => None,
                };
                let models = masses
                    .models
                    .map(|count| Probability::new(count, masses.total));
                let [a, b, c, d] = models;
                self.beliefs.insert(
                    prefix,
                    Belief {
                        models: [a?, b?, c?, d?],
                        target,
                    },
                );
            }
        }
        Ok(())
    }

    fn validate_prefix(&self, prefix: &LocalPrefix) -> Result<(), Error> {
        if prefix.ids != self.protocol.ids {
            return invalid("prefix identifiers differ from the protocol");
        }
        let prior_valid = match self.protocol.prior_mode {
            PriorMode::Stale(old) => prefix.own_prior == OwnPrior::PointMass(old),
            _ => match self.protocol.pair.roles[prefix.role.index()] {
                Knowledge::Known => matches!(prefix.own_prior, OwnPrior::PointMass(_)),
                Knowledge::Unknown | Knowledge::NoCommunication => {
                    prefix.own_prior == OwnPrior::Uniform
                }
            },
        };
        if !prior_valid {
            return invalid("supplied own prior disagrees with the declared treatment");
        }
        let (endpoint, include_action) = match prefix.checkpoint {
            Checkpoint::EpisodeStart => {
                return if prefix.entries.is_empty() {
                    Ok(())
                } else {
                    invalid("episode start must have an empty local transcript")
                };
            }
            Checkpoint::Action(position) | Checkpoint::AfterSlot(position) => {
                position.validate()?;
                let index = self
                    .schedule
                    .iter()
                    .position(|(_, p)| *p == position)
                    .ok_or_else(|| {
                        Error::InvalidProtocol("checkpoint outside declared schedule".into())
                    })?;
                (index, matches!(prefix.checkpoint, Checkpoint::AfterSlot(_)))
            }
            Checkpoint::Prediction { trial } => {
                let index = self
                    .schedule
                    .iter()
                    .position(|(_, p)| {
                        *p == Position {
                            phase: Phase::Live { trial },
                            round: 3,
                            slot: 4,
                        }
                    })
                    .ok_or_else(|| {
                        Error::InvalidProtocol("prediction outside declared trials".into())
                    })?;
                (index, true)
            }
        };
        let mut entries = prefix.entries.iter();
        let mut phase = None;
        let mut credits = INITIAL_CREDITS;
        for (index, (role, position)) in self.schedule.iter().enumerate().take(endpoint + 1) {
            if phase != Some(position.phase) {
                if entries.next()
                    != Some(&LocalEntry::Reset {
                        phase: position.phase,
                    })
                {
                    return invalid("missing or mistimed public phase reset");
                }
                if let Phase::Live { trial } = position.phase {
                    if !matches!(entries.next(), Some(LocalEntry::PrivateBit { trial: issued, .. }) if *issued == trial)
                    {
                        return invalid("missing or mistimed own private bit");
                    }
                }
                phase = Some(position.phase);
            }
            if *role == prefix.role && (index < endpoint || include_action) {
                let Some(LocalEntry::Action(event)) = entries.next() else {
                    return invalid("missing local action at a completed opportunity");
                };
                if event.position != *position {
                    return invalid("local action position differs from public chronology");
                }
                validate_action(prefix.role, event.position, &event.action)?;
                credits = remaining_credits(credits, &event.action)?;
                if credits != event.credits_after {
                    return invalid("local action credits do not match its cost");
                }
                let outcome_valid = matches!(
                    (event.action, event.outcome),
                    (super::Action::Read, Outcome::Read(_))
                        | (super::Action::Write(_), Outcome::Accepted)
                        | (super::Action::Wait, Outcome::Waited)
                        | (super::Action::InspectOwnTarget, Outcome::Inspected(_))
                );
                if !outcome_valid {
                    return invalid("outcome type does not match local action");
                }
            }
        }
        if entries.next().is_some() {
            return invalid("extra local entries or future observations");
        }
        Ok(())
    }
}

fn invalid<T>(message: &str) -> Result<T, Error> {
    Err(Error::InvalidProtocol(message.into()))
}

fn candidate_prior(protocol: &Protocol, role: Role, model: Mechanism) -> OwnPrior {
    match protocol.prior_mode {
        PriorMode::Stale(old) => OwnPrior::PointMass(old),
        _ if protocol.pair.roles[role.index()] == Knowledge::Known => OwnPrior::PointMass(model),
        _ => OwnPrior::Uniform,
    }
}

/// Count each original latent world exactly once in its role-local equivalence class.
/// OwnPrior in the key restricts known groups to their candidate mechanism; unknown
/// groups span mechanisms. Hidden peer histories and future bits never enter a key.
fn group_local_prefixes(
    candidates: &[Candidate],
    role: Role,
) -> Result<BTreeMap<LocalPrefix, Masses>, Error> {
    let mut groups: BTreeMap<LocalPrefix, Masses> = BTreeMap::new();
    for candidate in candidates.iter().filter(|c| c.positive_mass) {
        let prefix = &candidate.prefixes[role.index()];
        let masses = groups.entry(prefix.clone()).or_default();
        masses.models[candidate.model.index()] = masses.models[candidate.model.index()]
            .checked_add(1)
            .ok_or(Error::ArithmeticOverflow)?;
        masses.total = masses
            .total
            .checked_add(1)
            .ok_or(Error::ArithmeticOverflow)?;
        if candidate.current_target(role) == Some(true) {
            masses.target_true = masses
                .target_true
                .checked_add(1)
                .ok_or(Error::ArithmeticOverflow)?;
        }
    }
    Ok(groups)
}

impl Candidate {
    fn start_phase(&mut self, phase: Phase) {
        self.world.reset(phase);
        for role in [Role::A, Role::B] {
            let index = role.index();
            self.prefixes[index]
                .entries
                .push(LocalEntry::Reset { phase });
            self.private_bits[index] = match phase {
                Phase::Calibration => None,
                Phase::Live { trial } => {
                    let (x, y) = self.latent_bits.0[usize::from(trial)];
                    let bit = if role == Role::A { x } else { y };
                    self.prefixes[index]
                        .entries
                        .push(LocalEntry::PrivateBit { trial, bit });
                    Some(bit)
                }
            };
        }
    }

    fn current_target(&self, role: Role) -> Option<bool> {
        let trial = match self.prefixes[role.index()].checkpoint {
            Checkpoint::Action(Position {
                phase: Phase::Live { trial },
                ..
            })
            | Checkpoint::AfterSlot(Position {
                phase: Phase::Live { trial },
                ..
            })
            | Checkpoint::Prediction { trial } => trial,
            _ => return None,
        };
        let (x, y) = self.latent_bits.0[usize::from(trial)];
        Some(if role == Role::A { y } else { x })
    }

    fn local_view(
        &self,
        role: Role,
        beliefs: &BTreeMap<LocalPrefix, Belief>,
    ) -> Result<LocalView, Error> {
        let prefix = self.prefixes[role.index()].clone();
        let belief = beliefs
            .get(&prefix)
            .cloned()
            .ok_or_else(|| Error::UnsupportedHistory {
                prefix: prefix.clone(),
            })?;
        Ok(LocalView {
            prefix,
            credits: self.credits[role.index()],
            private_bit: self.private_bits[role.index()],
            belief,
        })
    }

    fn apply_local_decision(
        &mut self,
        role: Role,
        position: Position,
        action: &super::Action,
    ) -> Result<(), Error> {
        let event = self.world.apply(
            role,
            position,
            action,
            self.current_target(role),
            self.credits[role.index()],
        )?;
        self.credits[role.index()] = event.credits_after;
        self.prefixes[role.index()]
            .entries
            .push(LocalEntry::Action(event));
        Ok(())
    }
}
