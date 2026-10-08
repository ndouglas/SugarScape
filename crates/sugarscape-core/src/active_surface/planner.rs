use super::belief::{Candidate, Domain};
use super::*;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ChoiceValue {
    pub choice: Choice,
    pub value: ScoreFraction,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Decision {
    pub choice: Choice,
    pub alternatives: Vec<ChoiceValue>,
    pub continuation_policy: PolicyKind,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SearchStats {
    pub candidate_worlds: usize,
    pub positive_worlds: usize,
    pub decision_states: usize,
    pub branches: usize,
}
struct Solved {
    decision: Decision,
    belief: Belief,
    value: ScoreFraction,
}
/// Immutable local policy. Physical states are confined to original-candidate
/// inference and compilation; callers supply only their own Prefix or View.
pub struct CompiledPolicy {
    protocol: Protocol,
    prior: OwnPrior,
    domain: Domain,
    decisions: BTreeMap<Prefix, Solved>,
    stats: SearchStats,
}
impl CompiledPolicy {
    pub fn build(protocol: &Protocol, own_prior: OwnPrior) -> Result<Self, Error> {
        let domain = Domain::new(protocol, own_prior)?;
        let candidates = domain.candidates()?;
        let stats = SearchStats {
            candidate_worlds: domain.originals.len(),
            positive_worlds: candidates.len(),
            decision_states: 0,
            branches: 0,
        };
        let mut compiled = Self {
            protocol: protocol.clone(),
            prior: own_prior,
            domain,
            decisions: BTreeMap::new(),
            stats,
        };
        compiled.solve(candidates)?;
        compiled.stats.decision_states = compiled.decisions.len();
        Ok(compiled)
    }
    fn validate(&self, prefix: &Prefix) -> Result<(), Error> {
        self.protocol.validate_prefix(prefix)?;
        if prefix.role != self.protocol.experimenter || prefix.own_prior != self.prior {
            return Err(Error::InvalidHistory(
                "prefix does not belong to this compiled experimenter".into(),
            ));
        }
        Ok(())
    }
    pub fn infer(&self, prefix: &Prefix) -> Result<Belief, Error> {
        self.validate(prefix)?;
        if let Some(solved) = self.decisions.get(prefix) {
            return Ok(solved.belief.clone());
        }
        self.domain.belief(prefix, self.domain.support(prefix)?)
    }
    pub fn decide(&self, view: &View) -> Result<Decision, Error> {
        self.validate(&view.prefix)?;
        let reconstructed = View::from_prefix(view.prefix.clone(), self.infer(&view.prefix)?)?;
        if *view != reconstructed {
            return Err(Error::InvalidHistory(
                "view credits, issued bit, or belief differs from own history".into(),
            ));
        }
        if !matches!(view.prefix.checkpoint, Checkpoint::Boundary(_)) {
            return Err(Error::InvalidHistory(
                "decision requires a choice boundary".into(),
            ));
        }
        self.decisions
            .get(&view.prefix)
            .map(|s| s.decision.clone())
            .ok_or_else(|| Error::UnsupportedHistory {
                prefix: view.prefix.clone(),
            })
    }
    pub fn stats(&self) -> &SearchStats {
        &self.stats
    }
    fn solve(&mut self, candidates: Vec<Candidate>) -> Result<ScoreFraction, Error> {
        let prefix = candidates[0]
            .state
            .prefix(self.protocol.experimenter)
            .clone();
        if let Some(solved) = self.decisions.get(&prefix) {
            return Ok(solved.value);
        }
        let Checkpoint::Boundary(boundary) = prefix.checkpoint else {
            return Err(Error::InvalidHistory(
                "search expected choice boundary".into(),
            ));
        };
        let belief = self
            .domain
            .belief(&prefix, candidates.iter().map(|c| c.id))?;
        let choices = match boundary {
            Boundary::ProbeChoice { completed: 3 } => vec![Choice::StopProbing],
            Boundary::ProbeChoice { .. } => vec![Choice::StopProbing, Choice::ContinueProbe],
            Boundary::TrialChoice { .. } => vec![Choice::Inspect, Choice::AttemptCommunication],
        };
        let mut alternatives = Vec::new();
        for choice in choices {
            let mut next = candidates.clone();
            for candidate in &mut next {
                select_choice(&mut candidate.state, choice)?;
            }
            alternatives.push(ChoiceValue {
                choice,
                value: self.roll(next)?,
            });
        }
        let choice = match (self.protocol.policy, boundary) {
            (PolicyKind::FixedThree, Boundary::ProbeChoice { completed }) if completed < 3 => {
                Choice::ContinueProbe
            }
            (_, Boundary::ProbeChoice { completed: 3 })
            | (
                PolicyKind::FixedThree | PolicyKind::NoProbe | PolicyKind::InspectOnly,
                Boundary::ProbeChoice { .. },
            ) => Choice::StopProbing,
            (PolicyKind::InspectOnly, Boundary::TrialChoice { .. }) => Choice::Inspect,
            (PolicyKind::FixedThree, Boundary::TrialChoice { .. }) => {
                if belief.models[Mechanism::SharedPersistent.index()]
                    .checked_cmp(Probability::new(1, 2)?)?
                    == Ordering::Greater
                {
                    Choice::AttemptCommunication
                } else {
                    Choice::Inspect
                }
            }
            _ => maximizing_choice(&alternatives)?,
        };
        let value = alternatives
            .iter()
            .find(|v| v.choice == choice)
            .expect("legal selected choice")
            .value;
        self.decisions.insert(
            prefix,
            Solved {
                decision: Decision {
                    choice,
                    alternatives,
                    continuation_policy: self.protocol.policy,
                },
                belief,
                value,
            },
        );
        Ok(value)
    }
    /// Advance no farther than the next prediction, choice, or terminal point.
    /// Future private bits are never included in a previous prediction cohort.
    fn roll(&mut self, candidates: Vec<Candidate>) -> Result<ScoreFraction, Error> {
        let mass = u64::try_from(candidates.len()).map_err(|_| Error::ArithmeticOverflow)?;
        let credits = candidates[0].state.credits(self.protocol.experimenter);
        let mut branches: BTreeMap<Prefix, Vec<Candidate>> = BTreeMap::new();
        for mut candidate in candidates {
            loop {
                advance_one(&mut candidate.state)?;
                if matches!(
                    candidate
                        .state
                        .prefix(self.protocol.experimenter)
                        .checkpoint,
                    Checkpoint::Boundary(_) | Checkpoint::Prediction { .. } | Checkpoint::Finished
                ) {
                    break;
                }
            }
            branches
                .entry(candidate.state.prefix(self.protocol.experimenter).clone())
                .or_default()
                .push(candidate);
        }
        self.stats.branches += branches.len();
        let mut total = ScoreFraction::new(0, 1)?;
        for (prefix, branch) in branches {
            let weight = ScoreFraction::new(
                i64::try_from(branch.len()).map_err(|_| Error::ArithmeticOverflow)?,
                mass,
            )?;
            let spent = credits - branch[0].state.credits(self.protocol.experimenter);
            let continuation = match prefix.checkpoint {
                Checkpoint::Finished => ScoreFraction::new(0, 1)?,
                Checkpoint::Boundary(_) => self.solve(branch)?,
                Checkpoint::Prediction { .. } => {
                    let belief = self.domain.belief(&prefix, branch.iter().map(|c| c.id))?;
                    let p = belief.target.expect("prediction follows issued bit");
                    let correct = if predict_target(&belief)? {
                        p.numerator
                    } else {
                        p.denominator - p.numerator
                    };
                    let reward = ScoreFraction::new(
                        i64::try_from(correct).map_err(|_| Error::ArithmeticOverflow)?,
                        p.denominator,
                    )?
                    .checked_mul(ScoreFraction::new(12, 1)?)?;
                    reward.checked_add(self.roll(branch)?)?
                }
                _ => unreachable!("bounded transition endpoint"),
            };
            total = total.checked_add(weight.checked_mul(
                continuation.checked_add(ScoreFraction::new(-i64::from(spent), 1)?)?,
            )?)?;
        }
        Ok(total)
    }
}
/// Exact numeric comparison with the frozen Stop/Inspect tie priorities.
pub(super) fn maximizing_choice(alternatives: &[ChoiceValue]) -> Result<Choice, Error> {
    let mut best = alternatives
        .first()
        .ok_or_else(|| Error::InvalidHistory("no legal choices".into()))?;
    for value in alternatives.iter().skip(1) {
        let order = value.value.checked_cmp(best.value)?;
        let preferred = matches!(value.choice, Choice::StopProbing | Choice::Inspect);
        if order == Ordering::Greater || (order == Ordering::Equal && preferred) {
            best = value;
        }
    }
    Ok(best.choice)
}

/// Decode an issued target posterior by exact majority, predicting zero on a
/// tie. The caller must separately enforce the public prediction checkpoint.
pub fn predict_target(belief: &Belief) -> Result<bool, Error> {
    let target = belief.target.ok_or_else(|| {
        Error::InvalidHistory("target prediction requires an issued target posterior".into())
    })?;
    let target = Probability::new(target.numerator, target.denominator)?;
    Ok(target.checked_cmp(Probability::new(1, 2)?)? == Ordering::Greater)
}
