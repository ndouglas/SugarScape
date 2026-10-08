use super::*;
use std::collections::BTreeMap;

#[derive(Default)]
struct Mass {
    last_id: Option<usize>,
    models: [u64; 4],
    total: u64,
    target: Option<u64>,
}
impl Mass {
    fn observe(&mut self, id: usize, model: Mechanism, bits: &EpisodeBits, prefix: &Prefix) {
        // The responder can have the identical prefix immediately before and
        // after a private choice. Each original world contributes only once.
        if self.last_id == Some(id) {
            return;
        }
        self.last_id = Some(id);
        self.total += 1;
        self.models[model.index()] += 1;
        if let Some(trial) = prefix.entries.iter().rev().find_map(|entry| match entry {
            Entry::Physical(LocalEntry::PrivateBit { trial, .. }) => Some(*trial),
            _ => None,
        }) {
            let (a, b) = bits.0[usize::from(trial)];
            *self.target.get_or_insert(0) += u64::from(if prefix.role == Role::A { b } else { a });
        }
    }
    fn belief(self) -> Result<Belief, Error> {
        Ok(Belief {
            models: [
                Probability::new(self.models[0], self.total)?,
                Probability::new(self.models[1], self.total)?,
                Probability::new(self.models[2], self.total)?,
                Probability::new(self.models[3], self.total)?,
            ],
            target: self
                .target
                .map(|n| Probability::new(n, self.total))
                .transpose()?,
        })
    }
}

/// Both Agents' independent local likelihoods under the solved active policy.
/// This executes original worlds; it never conditions on an actual peer trace.
pub struct Replay {
    protocol: Protocol,
    prior: OwnPrior,
    beliefs: BTreeMap<Prefix, Belief>,
}
impl Replay {
    pub fn build(protocol: &Protocol, policy: &CompiledPolicy) -> Result<Self, Error> {
        protocol.validate()?;
        if protocol != policy.protocol() {
            return Err(Error::InvalidProtocol(
                "replay protocol differs from compiled policy".into(),
            ));
        }
        let prior = policy.own_prior();
        let mut masses: BTreeMap<Prefix, Mass> = BTreeMap::new();
        for model in Mechanism::ALL {
            // IDs retain their original model*256+sequence identity. Known
            // zero-mass catalog entries never construct a physical state.
            if prior != OwnPrior::Uniform && prior != OwnPrior::PointMass(model) {
                continue;
            }
            for sequence in 0..256 {
                let id = model.index() * 256 + usize::from(sequence);
                let bits = EpisodeBits::from_index(sequence)?;
                let mut priors = [OwnPrior::PointMass(model); 2];
                priors[protocol.experimenter.index()] = prior;
                let mut state = EpisodeState::new(
                    protocol,
                    Environment::InFamily(model),
                    bits.clone(),
                    priors,
                )?;
                loop {
                    record(&mut masses, &state, id, model, &bits);
                    let prefix = state.prefix(protocol.experimenter);
                    if prefix.checkpoint == Checkpoint::Finished {
                        break;
                    }
                    if matches!(prefix.checkpoint, Checkpoint::Boundary(_)) {
                        let view = View::from_prefix(prefix.clone(), policy.infer(prefix)?)?;
                        select_choice(&mut state, policy.decide(&view)?.choice)?;
                        record(&mut masses, &state, id, model, &bits);
                    }
                    advance_one(&mut state)?;
                }
            }
        }
        let beliefs = masses
            .into_iter()
            .map(|(p, m)| Ok((p, m.belief()?)))
            .collect::<Result<_, Error>>()?;
        Ok(Self {
            protocol: protocol.clone(),
            prior,
            beliefs,
        })
    }
    pub fn infer(&self, prefix: &Prefix) -> Result<Belief, Error> {
        self.protocol.validate_prefix(prefix)?;
        if prefix.role == self.protocol.experimenter && prefix.own_prior != self.prior {
            return Err(Error::InvalidHistory(
                "prefix prior differs from solved policy".into(),
            ));
        }
        self.beliefs
            .get(prefix)
            .cloned()
            .ok_or_else(|| Error::UnsupportedHistory {
                prefix: prefix.clone(),
            })
    }
    pub(super) fn validate_policy(
        &self,
        protocol: &Protocol,
        policy: &CompiledPolicy,
    ) -> Result<(), Error> {
        if protocol != &self.protocol
            || protocol != policy.protocol()
            || self.prior != policy.own_prior()
        {
            return Err(Error::InvalidProtocol(
                "host, replay, and compiled policy must agree".into(),
            ));
        }
        Ok(())
    }
}
fn record(
    masses: &mut BTreeMap<Prefix, Mass>,
    state: &EpisodeState,
    id: usize,
    model: Mechanism,
    bits: &EpisodeBits,
) {
    for role in [Role::A, Role::B] {
        let prefix = state.prefix(role);
        masses
            .entry(prefix.clone())
            .or_default()
            .observe(id, model, bits, prefix);
    }
}
