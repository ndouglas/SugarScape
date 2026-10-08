//! Request-only controllers and a bounded finite inference reference.
use super::*;
use crate::rng::{seeded, SimRng};
use rand::Rng;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PolicyKind {
    Evidence,
    Random,
    Reckless,
    Passive,
}
#[derive(Clone, Debug)]
pub struct BuiltinController {
    kind: PolicyKind,
    rng: SimRng,
    reported: Vec<(AgentId, AgentId, Round)>,
}
impl BuiltinController {
    pub fn new(kind: PolicyKind, seed: u64) -> Self {
        Self {
            kind,
            rng: seeded(seed),
            reported: Vec::new(),
        }
    }
    fn use_action(&mut self, r: &TurnRequest, random: bool) -> Option<Action> {
        let choices: Vec<_> = r
            .legal
            .capabilities
            .iter()
            .filter(|id| {
                self.kind != PolicyKind::Evidence || **id == r.observation.objectives.capability
            })
            .filter_map(|id| {
                let c = r.observation.capabilities.iter().find(|c| c.id == *id)?;
                let targets = match c.target {
                    TargetRule::SelfOnly => vec![r.actor],
                    TargetRule::OtherActive => r.legal.targets.clone(),
                };
                Some(
                    targets
                        .into_iter()
                        .map(|target| Action::Use {
                            capability: *id,
                            target,
                        })
                        .collect::<Vec<_>>(),
                )
            })
            .flatten()
            .collect();
        if !r.legal.can_use || choices.is_empty() {
            None
        } else {
            Some(
                choices[if random {
                    self.rng.gen_range(0..choices.len() as u64) as usize
                } else {
                    0
                }]
                .clone(),
            )
        }
    }
}
impl Controller for BuiltinController {
    fn respond(&mut self, r: &TurnRequest) -> TurnResponse {
        let mut action = Action::Pass;
        if self.kind != PolicyKind::Passive {
            match r.phase {
                Phase::Attention
                    if r.legal.can_watch
                        && r.legal.attention_capacity > 0
                        && !r.legal.targets.is_empty() =>
                {
                    let index = if self.kind == PolicyKind::Random {
                        self.rng.gen_range(0..r.legal.targets.len() as u64) as usize
                    } else {
                        (r.round as usize + usize::from(r.actor)) % r.legal.targets.len()
                    };
                    action = Action::Watch {
                        agents: vec![r.legal.targets[index]],
                    };
                }
                Phase::Act => {
                    action = self
                        .use_action(r, self.kind == PolicyKind::Random)
                        .unwrap_or(Action::Pass);
                }
                Phase::Discuss => {
                    let sighting =
                        r.observation
                            .events
                            .iter()
                            .rev()
                            .find_map(|e| match e.content {
                                EventContent::Use {
                                    source,
                                    target,
                                    capability,
                                } if capability == r.observation.objectives.capability
                                    && source != r.actor =>
                                {
                                    Some((source, target, e.round))
                                }
                                _ => None,
                            });
                    if r.legal.can_accuse
                        && r.legal.accusation_budget > 0
                        && !r.legal.targets.is_empty()
                    {
                        let target = match self.kind {
                            PolicyKind::Evidence => sighting
                                .map(|s| s.0)
                                .filter(|id| r.legal.targets.contains(id)),
                            PolicyKind::Reckless => r.legal.targets.iter().min().copied(),
                            PolicyKind::Random => {
                                if self.rng.gen_range(0..2) == 0 {
                                    Some(
                                        r.legal.targets[self
                                            .rng
                                            .gen_range(0..r.legal.targets.len() as u64)
                                            as usize],
                                    )
                                } else {
                                    None
                                }
                            }
                            PolicyKind::Passive => None,
                        };
                        if let Some(target) = target {
                            action = Action::Accuse { target };
                        }
                    }
                    if action == Action::Pass
                        && r.legal.can_say
                        && self.kind == PolicyKind::Evidence
                    {
                        if let Some(s) = sighting {
                            if !self.reported.contains(&s) {
                                self.reported.push(s);
                                action = Action::Say {
                                    claim: Claim::SawUse {
                                        source: s.0,
                                        target: s.1,
                                        round: s.2,
                                    },
                                };
                            }
                        }
                    }
                }
                _ => {}
            }
        }
        TurnResponse {
            request_id: r.request_id,
            actor: r.actor,
            action,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InferenceError {
    InvalidInput,
    ZeroEvidence,
}
impl std::fmt::Display for InferenceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for InferenceError {}
pub fn posterior(prior: &[f64], likelihood: &[f64]) -> Result<Vec<f64>, InferenceError> {
    if prior.is_empty()
        || prior.len() != likelihood.len()
        || prior
            .iter()
            .chain(likelihood)
            .any(|v| !v.is_finite() || *v < 0.0)
    {
        return Err(InferenceError::InvalidInput);
    }
    let weights: Vec<_> = prior.iter().zip(likelihood).map(|(p, l)| p * l).collect();
    let total: f64 = weights.iter().sum();
    if !total.is_finite() {
        return Err(InferenceError::InvalidInput);
    }
    if total <= 0.0 {
        return Err(InferenceError::ZeroEvidence);
    }
    Ok(weights.into_iter().map(|v| v / total).collect())
}
/// Invalid or unnormalized probabilities and abstention ties return None.
pub fn best_accusation(p: &[f64], correct: f64, wrong: f64, abstain: f64) -> Option<usize> {
    if p.is_empty()
        || p.iter().any(|v| !v.is_finite() || !(0.0..=1.0).contains(v))
        || (p.iter().sum::<f64>() - 1.0).abs() > 1e-12
        || [correct, wrong, abstain].iter().any(|v| !v.is_finite())
    {
        return None;
    }
    let mut best = abstain;
    let mut choice = None;
    for (i, mass) in p.iter().enumerate() {
        let utility = mass * correct + (1.0 - mass) * wrong;
        if utility > best {
            best = utility;
            choice = Some(i);
        }
    }
    choice
}
