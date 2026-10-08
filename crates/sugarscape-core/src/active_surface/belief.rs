use super::*;

/// Original equal-weight catalog. Zero-weight records remain catalog metadata;
/// they never construct or transition a physical candidate.
#[derive(Clone)]
pub(super) struct Original {
    pub model: Mechanism,
    pub bits: EpisodeBits,
    pub positive: bool,
}
#[derive(Clone)]
pub(super) struct Candidate {
    pub id: usize,
    pub state: EpisodeState,
}
pub(super) struct Domain {
    pub originals: Vec<Original>,
    protocol: Protocol,
    prior: OwnPrior,
}
impl Domain {
    pub fn new(protocol: &Protocol, prior: OwnPrior) -> Result<Self, Error> {
        protocol.validate()?;
        if (protocol.policy == PolicyKind::Known) != matches!(prior, OwnPrior::PointMass(_)) {
            return Err(Error::InvalidProtocol(
                "experimenter prior disagrees with policy".into(),
            ));
        }
        let mut originals = Vec::with_capacity(1024);
        for model in Mechanism::ALL {
            for index in 0..256 {
                originals.push(Original {
                    model,
                    bits: EpisodeBits::from_index(index)?,
                    positive: prior == OwnPrior::Uniform || prior == OwnPrior::PointMass(model),
                });
            }
        }
        Ok(Self {
            originals,
            protocol: protocol.clone(),
            prior,
        })
    }
    pub fn candidates(&self) -> Result<Vec<Candidate>, Error> {
        self.originals
            .iter()
            .enumerate()
            .filter(|(_, o)| o.positive)
            .map(|(id, o)| {
                let mut priors = [OwnPrior::PointMass(o.model); 2];
                priors[self.protocol.experimenter.index()] = self.prior;
                Ok(Candidate {
                    id,
                    state: EpisodeState::new(
                        &self.protocol,
                        Environment::InFamily(o.model),
                        o.bits.clone(),
                        priors,
                    )?,
                })
            })
            .collect()
    }
    pub fn belief(
        &self,
        prefix: &Prefix,
        ids: impl IntoIterator<Item = usize>,
    ) -> Result<Belief, Error> {
        let trial = prefix.entries.iter().rev().find_map(|e| {
            if let Entry::Physical(LocalEntry::PrivateBit { trial, .. }) = e {
                Some(*trial)
            } else {
                None
            }
        });
        let mut mass = 0_u64;
        let mut models = [0_u64; 4];
        let mut target = 0_u64;
        for id in ids {
            let original = &self.originals[id];
            mass += 1;
            models[original.model.index()] += 1;
            if let Some(trial) = trial {
                let (a, b) = original.bits.0[usize::from(trial)];
                target += u64::from(if prefix.role == Role::A { b } else { a });
            }
        }
        if mass == 0 {
            return Err(Error::UnsupportedHistory {
                prefix: prefix.clone(),
            });
        }
        Ok(Belief {
            models: [
                Probability::new(models[0], mass)?,
                Probability::new(models[1], mass)?,
                Probability::new(models[2], mass)?,
                Probability::new(models[3], mass)?,
            ],
            target: trial.map(|_| Probability::new(target, mass)).transpose()?,
        })
    }
    /// Replay only the querying Agent's interventions. Candidate peer state is
    /// generated independently; no actual peer history is accepted or compared.
    pub fn support(&self, prefix: &Prefix) -> Result<Vec<usize>, Error> {
        let mut ids = Vec::new();
        for mut candidate in self.candidates()? {
            loop {
                let own = candidate.state.prefix(prefix.role);
                if own == prefix {
                    ids.push(candidate.id);
                    break;
                }
                if !prefix.entries.starts_with(&own.entries)
                    || own.checkpoint == Checkpoint::Finished
                {
                    break;
                }
                if let Checkpoint::Boundary(boundary) = own.checkpoint {
                    // An unselected boundary is the only place replay chooses.
                    let Some(Entry::OwnChoice {
                        boundary: recorded,
                        choice,
                    }) = prefix.entries.get(own.entries.len())
                    else {
                        break;
                    };
                    if *recorded != boundary {
                        break;
                    }
                    select_choice(&mut candidate.state, *choice)?;
                    if candidate.state.prefix(prefix.role) == prefix {
                        ids.push(candidate.id);
                        break;
                    }
                    if !prefix
                        .entries
                        .starts_with(&candidate.state.prefix(prefix.role).entries)
                    {
                        break;
                    }
                }
                advance_one(&mut candidate.state)?;
            }
        }
        Ok(ids)
    }
}
