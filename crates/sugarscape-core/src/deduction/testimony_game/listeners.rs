use super::{Config, DecisionAction, DecisionObservation, Error};
use crate::deduction::{
    best_accusation, Belief, EvidenceRecord, Proposition, PropositionValue, ReportingProfile,
    SignalGroup, SpeakerProfile, TestimonyError, TestimonyEvidence, TestimonyHypothesis,
    TestimonyModel,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Genome {
    pub b: i16,
    pub u: i16,
    pub d: i16,
    pub k: i16,
}
impl Genome {
    pub fn validate(&self) -> Result<(), Error> {
        if (-16..=16).contains(&self.b)
            && (0..=16).contains(&self.u)
            && (0..=16).contains(&self.d)
            && (-16..=16).contains(&self.k)
        {
            Ok(())
        } else {
            Err(Error::InvalidGenome)
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Listener {
    Bayesian,
    Credulous,
    Skeptical,
    DirectEvidence,
    Passive,
    Evolved(Genome),
}
impl<'de> Deserialize<'de> for Listener {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
        enum WireListener {
            Bayesian {},
            Credulous {},
            Skeptical {},
            DirectEvidence {},
            Passive {},
            Evolved { b: i16, u: i16, d: i16, k: i16 },
        }
        Ok(match WireListener::deserialize(deserializer)? {
            WireListener::Bayesian {} => Self::Bayesian,
            WireListener::Credulous {} => Self::Credulous,
            WireListener::Skeptical {} => Self::Skeptical,
            WireListener::DirectEvidence {} => Self::DirectEvidence,
            WireListener::Passive {} => Self::Passive,
            WireListener::Evolved { b, u, d, k } => Self::Evolved(Genome { b, u, d, k }),
        })
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ListenerDecision {
    pub action: DecisionAction,
    pub posterior_true: Option<f64>,
}
impl Listener {
    pub fn decide(&self, view: &DecisionObservation) -> Result<ListenerDecision, Error> {
        view.rules.validate()?;
        let q = value(&view.rules.accuracy);
        let rho = value(&view.rules.copy_prior);
        let posterior = match self {
            Self::Bayesian => Some(bayesian(view)?),
            Self::Credulous => Some(channel_posterior(q, view.live_reports)?),
            Self::Skeptical => Some(channel_posterior(
                rho * q + (1.0 - rho) * (1.0 - q),
                view.live_reports,
            )?),
            Self::DirectEvidence => Some(0.5),
            Self::Passive => None,
            Self::Evolved(g) => {
                g.validate()?;
                let score: f64 = (0..2)
                    .map(|slot| {
                        let trust = if rho == 0.0 || rho == 1.0 {
                            rho
                        } else {
                            let adjustment =
                                if view.calibration_reports[slot] == view.calibration_truth {
                                    f64::from(g.u) / 4.0
                                } else {
                                    -f64::from(g.d) / 4.0
                                };
                            sigmoid(rho.ln() - (-rho).ln_1p() + f64::from(g.b) / 4.0 + adjustment)
                        };
                        (2.0 * q - 1.0)
                            * (2.0 * trust - 1.0)
                            * if view.live_reports[slot] { 1.0 } else { -1.0 }
                    })
                    .sum();
                return Ok(ListenerDecision {
                    action: if score > f64::from(g.k) / 8.0 {
                        DecisionAction::Intervene
                    } else {
                        DecisionAction::Abstain
                    },
                    posterior_true: None,
                });
            }
        };
        Ok(ListenerDecision {
            action: posterior
                .map(probability_action)
                .unwrap_or(DecisionAction::Abstain),
            posterior_true: posterior,
        })
    }
}
fn value(p: &super::Probability) -> f64 {
    f64::from(p.numerator) / f64::from(p.denominator)
}
fn sigmoid(x: f64) -> f64 {
    if x >= 0.0 {
        1.0 / (1.0 + (-x).exp())
    } else {
        let e = x.exp();
        e / (1.0 + e)
    }
}
fn probability_action(p: f64) -> DecisionAction {
    // The helper may choose the negative proposition, which is not a legal intervention.
    if best_accusation(&[p, 1.0 - p], 1.0, -1.0, 0.0) == Some(0) {
        DecisionAction::Intervene
    } else {
        DecisionAction::Abstain
    }
}
fn channel_posterior(accuracy: f64, reports: [bool; 2]) -> Result<f64, Error> {
    let yes: f64 = reports
        .iter()
        .map(|&r| if r { accuracy } else { 1.0 - accuracy })
        .product();
    let no: f64 = reports
        .iter()
        .map(|&r| if r { 1.0 - accuracy } else { accuracy })
        .product();
    if yes + no == 0.0 {
        Err(Error::Inference(TestimonyError::ZeroEvidence))
    } else {
        Ok(yes / (yes + no))
    }
}
fn bayesian(view: &DecisionObservation) -> Result<f64, Error> {
    let Config { reporters, .. } = &view.rules;
    let q = value(&view.rules.accuracy);
    let rho = value(&view.rules.copy_prior);
    let mut hypotheses = Vec::with_capacity(16);
    for bits in 0..16u16 {
        let c = bits & 8 != 0;
        let t = bits & 4 != 0;
        let p0 = bits & 2 != 0;
        let p1 = bits & 1 != 0;
        hypotheses.push(TestimonyHypothesis {
            id: bits,
            prior: 0.25 * if p0 { rho } else { 1.0 - rho } * if p1 { rho } else { 1.0 - rho },
            propositions: vec![
                PropositionValue {
                    proposition: 0,
                    value: c,
                },
                PropositionValue {
                    proposition: 1,
                    value: t,
                },
            ],
            profiles: vec![
                SpeakerProfile {
                    speaker: reporters[0],
                    profile: u16::from(p0),
                },
                SpeakerProfile {
                    speaker: reporters[1],
                    profile: u16::from(p1),
                },
            ],
        });
    }
    let mut belief = Belief::new(TestimonyModel {
        propositions: vec![
            Proposition { id: 0, label: None },
            Proposition { id: 1, label: None },
        ],
        speakers: reporters.to_vec(),
        profiles: vec![
            ReportingProfile {
                id: 0,
                positive_given_signal: [1.0, 0.0],
            },
            ReportingProfile {
                id: 1,
                positive_given_signal: [0.0, 1.0],
            },
        ],
        hypotheses,
        groups: (0..4)
            .map(|id| SignalGroup {
                id,
                proposition: id / 2,
                accuracy: q,
            })
            .collect(),
    })
    .map_err(Error::Inference)?;
    for (slot, &speaker) in reporters.iter().enumerate() {
        belief
            .observe(EvidenceRecord {
                id: slot as u64,
                content: TestimonyEvidence::Report {
                    group: slot as u16,
                    speaker,
                    positive: view.calibration_reports[slot],
                },
            })
            .map_err(Error::Inference)?;
    }
    belief
        .observe(EvidenceRecord {
            id: 2,
            content: TestimonyEvidence::Verified {
                proposition: 0,
                value: view.calibration_truth,
            },
        })
        .map_err(Error::Inference)?;
    for (slot, &speaker) in reporters.iter().enumerate() {
        belief
            .observe(EvidenceRecord {
                id: 3 + slot as u64,
                content: TestimonyEvidence::Report {
                    group: 2 + slot as u16,
                    speaker,
                    positive: view.live_reports[slot],
                },
            })
            .map_err(Error::Inference)?;
    }
    // Even an uninformative channel must condition and validate delivered evidence.
    if q == 0.5 {
        return Ok(0.5);
    }
    let posterior = belief.snapshot().propositions[1].probability_true;
    // Normalize inference roundoff on mathematical ties. Valid rational rules bound
    // every genuine non-tie's distance from 1/2 by at least 1/(2 * 67_108_864),
    // so this tolerance cannot turn supported non-tie evidence into abstention.
    Ok(if (posterior - 0.5).abs() <= 1e-12 {
        0.5
    } else {
        posterior
    })
}
