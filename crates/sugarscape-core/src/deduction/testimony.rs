//! Bounded exact models of proposition truth and persistent reporting profiles.
use super::{AgentId, FieldError};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
pub type PropositionId = u16;
pub type ProfileId = u16;
pub type HypothesisId = u16;
pub type SignalGroupId = u16;
pub type EvidenceId = u64;
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Proposition {
    pub id: PropositionId,
    pub label: Option<String>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReportingProfile {
    pub id: ProfileId,
    pub positive_given_signal: [f64; 2],
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PropositionValue {
    pub proposition: PropositionId,
    pub value: bool,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SpeakerProfile {
    pub speaker: AgentId,
    pub profile: ProfileId,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TestimonyHypothesis {
    pub id: HypothesisId,
    pub prior: f64,
    pub propositions: Vec<PropositionValue>,
    pub profiles: Vec<SpeakerProfile>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SignalGroup {
    pub id: SignalGroupId,
    pub proposition: PropositionId,
    pub accuracy: f64,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TestimonyModel {
    pub propositions: Vec<Proposition>,
    pub speakers: Vec<AgentId>,
    pub profiles: Vec<ReportingProfile>,
    pub hypotheses: Vec<TestimonyHypothesis>,
    pub groups: Vec<SignalGroup>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EvidenceRecord {
    pub id: EvidenceId,
    pub content: TestimonyEvidence,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum TestimonyEvidence {
    Report {
        group: SignalGroupId,
        speaker: AgentId,
        positive: bool,
    },
    Verified {
        proposition: PropositionId,
        value: bool,
    },
}
#[derive(Clone, Debug, PartialEq)]
pub enum TestimonyError {
    InvalidModel(Vec<FieldError>),
    UnknownGroup(SignalGroupId),
    UnknownSpeaker(AgentId),
    UnknownProposition(PropositionId),
    ConflictingEvidence(EvidenceId),
    CapacityExceeded,
    ZeroEvidence,
}
impl std::fmt::Display for TestimonyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidModel(errors) => {
                write!(f, "invalid testimony model")?;
                for error in errors {
                    write!(f, "; {}: {}", error.field, error.message)?;
                }
                Ok(())
            }
            Self::UnknownGroup(id) => write!(f, "unknown signal group {id}"),
            Self::UnknownSpeaker(id) => write!(f, "unknown speaker {id}"),
            Self::UnknownProposition(id) => write!(f, "unknown proposition {id}"),
            Self::ConflictingEvidence(id) => write!(f, "conflicting evidence ID {id}"),
            Self::CapacityExceeded => write!(f, "testimony evidence capacity of 512 exceeded"),
            Self::ZeroEvidence => write!(
                f,
                "evidence has zero probability under every supported hypothesis"
            ),
        }
    }
}
impl std::error::Error for TestimonyError {}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HypothesisProbability {
    pub id: HypothesisId,
    pub probability: f64,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PropositionProbability {
    pub id: PropositionId,
    pub probability_true: f64,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProfileProbability {
    pub id: ProfileId,
    pub probability: f64,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SpeakerMarginal {
    pub speaker: AgentId,
    pub profiles: Vec<ProfileProbability>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BeliefSnapshot {
    pub hypotheses: Vec<HypothesisProbability>,
    pub propositions: Vec<PropositionProbability>,
    pub speakers: Vec<SpeakerMarginal>,
    pub evidence_count: usize,
}
#[derive(Clone, Debug, PartialEq)]
pub struct Belief {
    model: TestimonyModel,
    records: Vec<EvidenceRecord>,
    posterior: Vec<f64>,
}
impl Belief {
    pub fn new(model: TestimonyModel) -> Result<Self, TestimonyError> {
        validate_model(&model)?;
        let total: f64 = model.hypotheses.iter().map(|h| h.prior).sum();
        let posterior = model.hypotheses.iter().map(|h| h.prior / total).collect();
        Ok(Self {
            model,
            records: Vec::new(),
            posterior,
        })
    }
    /// Assimilate explicitly attributed evidence without mutating on failure.
    pub fn observe(&mut self, record: EvidenceRecord) -> Result<(), TestimonyError> {
        if let Some(existing) = self.records.iter().find(|r| r.id == record.id) {
            return if existing == &record {
                Ok(())
            } else {
                Err(TestimonyError::ConflictingEvidence(record.id))
            };
        }
        match record.content {
            TestimonyEvidence::Report { group, speaker, .. } => {
                if !self.model.groups.iter().any(|g| g.id == group) {
                    return Err(TestimonyError::UnknownGroup(group));
                }
                if !self.model.speakers.contains(&speaker) {
                    return Err(TestimonyError::UnknownSpeaker(speaker));
                }
            }
            TestimonyEvidence::Verified { proposition, .. } => {
                if !self.model.propositions.iter().any(|p| p.id == proposition) {
                    return Err(TestimonyError::UnknownProposition(proposition));
                }
            }
        }
        if self.records.len() == 512 {
            return Err(TestimonyError::CapacityExceeded);
        }
        let mut candidate = self.records.clone();
        candidate.push(record);
        let posterior = self.condition(&candidate)?;
        self.records = candidate;
        self.posterior = posterior;
        Ok(())
    }

    fn condition(&self, records: &[EvidenceRecord]) -> Result<Vec<f64>, TestimonyError> {
        let total_prior: f64 = self.model.hypotheses.iter().map(|h| h.prior).sum();
        let logs: Vec<_> = self
            .model
            .hypotheses
            .iter()
            .map(|h| {
                let verified = records.iter().all(|r| match r.content {
                    TestimonyEvidence::Verified { proposition, value } => h
                        .propositions
                        .iter()
                        .any(|p| p.proposition == proposition && p.value == value),
                    TestimonyEvidence::Report { .. } => true,
                });
                if !verified {
                    return f64::NEG_INFINITY;
                }
                let mut weight = (h.prior / total_prior).ln();
                for group in &self.model.groups {
                    weight += group_log_likelihood(&self.model, h, group, records);
                }
                weight
            })
            .collect();
        let maximum = logs.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        if maximum == f64::NEG_INFINITY {
            return Err(TestimonyError::ZeroEvidence);
        }
        let weights: Vec<_> = logs.iter().map(|&w| (w - maximum).exp()).collect();
        let total: f64 = weights.iter().sum();
        Ok(weights.into_iter().map(|w| w / total).collect())
    }

    pub fn snapshot(&self) -> BeliefSnapshot {
        BeliefSnapshot {
            hypotheses: self
                .model
                .hypotheses
                .iter()
                .zip(&self.posterior)
                .map(|(h, &probability)| HypothesisProbability {
                    id: h.id,
                    probability,
                })
                .collect(),
            propositions: self
                .model
                .propositions
                .iter()
                .map(|p| PropositionProbability {
                    id: p.id,
                    probability_true: self
                        .model
                        .hypotheses
                        .iter()
                        .zip(&self.posterior)
                        .filter(|(h, _)| {
                            h.propositions
                                .iter()
                                .any(|v| v.proposition == p.id && v.value)
                        })
                        .map(|(_, w)| w)
                        .sum(),
                })
                .collect(),
            speakers: self
                .model
                .speakers
                .iter()
                .map(|&speaker| SpeakerMarginal {
                    speaker,
                    profiles: self
                        .model
                        .profiles
                        .iter()
                        .map(|p| ProfileProbability {
                            id: p.id,
                            probability: self
                                .model
                                .hypotheses
                                .iter()
                                .zip(&self.posterior)
                                .filter(|(h, _)| {
                                    h.profiles
                                        .iter()
                                        .any(|v| v.speaker == speaker && v.profile == p.id)
                                })
                                .map(|(_, w)| w)
                                .sum(),
                        })
                        .collect(),
                })
                .collect(),
            evidence_count: self.records.len(),
        }
    }
}
fn probability_valid(p: f64) -> bool {
    p.is_finite() && (0.0..=1.0).contains(&p)
}
fn ids_valid(
    ids: impl IntoIterator<Item = u16>,
    min: usize,
    max: usize,
    field: &str,
    errors: &mut Vec<FieldError>,
) -> BTreeSet<u16> {
    let ids: Vec<_> = ids.into_iter().collect();
    if !(min..=max).contains(&ids.len()) {
        errors.push(FieldError::new(
            field,
            format!("count must be {min}..={max}"),
        ));
    }
    let set: BTreeSet<_> = ids.iter().copied().collect();
    if set.len() != ids.len() {
        errors.push(FieldError::new(field, "IDs must be unique"));
    }
    set
}
fn validate_model(m: &TestimonyModel) -> Result<(), TestimonyError> {
    let mut errors = Vec::new();
    let propositions = ids_valid(
        m.propositions.iter().map(|p| p.id),
        1,
        16,
        "propositions",
        &mut errors,
    );
    let speakers = ids_valid(m.speakers.iter().copied(), 1, 32, "speakers", &mut errors);
    let profiles = ids_valid(
        m.profiles.iter().map(|p| p.id),
        1,
        16,
        "profiles",
        &mut errors,
    );
    ids_valid(
        m.hypotheses.iter().map(|h| h.id),
        1,
        256,
        "hypotheses",
        &mut errors,
    );
    ids_valid(m.groups.iter().map(|g| g.id), 0, 256, "groups", &mut errors);
    for (i, p) in m.profiles.iter().enumerate() {
        for (signal, &value) in p.positive_given_signal.iter().enumerate() {
            if !probability_valid(value) {
                errors.push(FieldError::new(
                    format!("profiles[{i}].positive_given_signal[{signal}]"),
                    "must be finite and in [0,1]",
                ));
            }
        }
    }
    for (i, g) in m.groups.iter().enumerate() {
        if !propositions.contains(&g.proposition) {
            errors.push(FieldError::new(
                format!("groups[{i}].proposition"),
                "unknown proposition",
            ));
        }
        if !probability_valid(g.accuracy) {
            errors.push(FieldError::new(
                format!("groups[{i}].accuracy"),
                "must be finite and in [0,1]",
            ));
        }
    }
    for (i, h) in m.hypotheses.iter().enumerate() {
        if !probability_valid(h.prior) {
            errors.push(FieldError::new(
                format!("hypotheses[{i}].prior"),
                "must be finite and in [0,1]",
            ));
        }
        let assigned: BTreeSet<_> = h.propositions.iter().map(|v| v.proposition).collect();
        if assigned != propositions || assigned.len() != h.propositions.len() {
            errors.push(FieldError::new(
                format!("hypotheses[{i}].propositions"),
                "must assign every declared proposition exactly once",
            ));
        }
        let assigned: BTreeSet<_> = h.profiles.iter().map(|v| v.speaker).collect();
        if assigned != speakers || assigned.len() != h.profiles.len() {
            errors.push(FieldError::new(
                format!("hypotheses[{i}].profiles"),
                "must assign every declared speaker exactly once",
            ));
        }
        for (j, v) in h.profiles.iter().enumerate() {
            if !profiles.contains(&v.profile) {
                errors.push(FieldError::new(
                    format!("hypotheses[{i}].profiles[{j}].profile"),
                    "unknown reporting profile",
                ));
            }
        }
    }
    let total: f64 = m.hypotheses.iter().map(|h| h.prior).sum();
    if !total.is_finite() || total <= 0.0 || (total - 1.0).abs() > 1e-12 {
        errors.push(FieldError::new(
            "hypotheses.prior",
            "must have positive total mass and sum to one within 1e-12",
        ));
    }
    if errors.is_empty() {
        Ok(())
    } else {
        Err(TestimonyError::InvalidModel(errors))
    }
}
fn log_probability(p: f64, positive: bool) -> f64 {
    if positive {
        p.ln()
    } else {
        (-p).ln_1p()
    }
}
fn log_add(a: f64, b: f64) -> f64 {
    let m = a.max(b);
    if m == f64::NEG_INFINITY {
        m
    } else {
        m + ((a - m).exp() + (b - m).exp()).ln()
    }
}
/// Requires validated model assignments and report references.
fn group_log_likelihood(
    model: &TestimonyModel,
    hypothesis: &TestimonyHypothesis,
    group: &SignalGroup,
    records: &[EvidenceRecord],
) -> f64 {
    let reports: Vec<_> = records
        .iter()
        .filter_map(|r| match r.content {
            TestimonyEvidence::Report {
                group: id,
                speaker,
                positive,
            } if id == group.id => Some((speaker, positive)),
            _ => None,
        })
        .collect();
    if reports.is_empty() {
        return 0.0;
    }
    let truth = hypothesis
        .propositions
        .iter()
        .find(|v| v.proposition == group.proposition)
        .expect("validated proposition assignment")
        .value;
    let mut alternatives = [0.0; 2];
    for (signal, term) in alternatives.iter_mut().enumerate() {
        *term = log_probability(group.accuracy, (signal == 1) == truth);
        for &(speaker, positive) in &reports {
            let profile = hypothesis
                .profiles
                .iter()
                .find(|p| p.speaker == speaker)
                .expect("validated speaker assignment")
                .profile;
            let channel = &model
                .profiles
                .iter()
                .find(|p| p.id == profile)
                .expect("validated profile reference")
                .positive_given_signal;
            *term += log_probability(channel[signal], positive);
        }
    }
    log_add(alternatives[0], alternatives[1])
}
#[cfg(test)]
mod tests {
    use super::*;
    fn model(channel: [f64; 2], accuracy: f64) -> TestimonyModel {
        TestimonyModel {
            propositions: vec![Proposition { id: 7, label: None }],
            speakers: vec![19],
            profiles: vec![ReportingProfile {
                id: 3,
                positive_given_signal: channel,
            }],
            hypotheses: [false, true]
                .into_iter()
                .enumerate()
                .map(|(i, value)| TestimonyHypothesis {
                    id: i as u16,
                    prior: 0.5,
                    propositions: vec![PropositionValue {
                        proposition: 7,
                        value,
                    }],
                    profiles: vec![SpeakerProfile {
                        speaker: 19,
                        profile: 3,
                    }],
                })
                .collect(),
            groups: vec![SignalGroup {
                id: 11,
                proposition: 7,
                accuracy,
            }],
        }
    }
    #[test]
    fn testimony_channel_atoms_and_shared_signal() {
        for q in [0.0, 0.5, 0.8, 1.0] {
            for channel in [[0.0, 1.0], [1.0, 0.0], [1.0, 1.0]] {
                let m = model(channel, q);
                Belief::new(m.clone()).unwrap();
                for h in &m.hypotheses {
                    let truth = h.propositions[0].value;
                    for positive in [false, true] {
                        let r = EvidenceRecord {
                            id: 1,
                            content: TestimonyEvidence::Report {
                                group: 11,
                                speaker: 19,
                                positive,
                            },
                        };
                        let p_signal_true = if truth { q } else { 1.0 - q };
                        let expected_positive =
                            (1.0 - p_signal_true) * channel[0] + p_signal_true * channel[1];
                        let expected = if positive {
                            expected_positive
                        } else {
                            1.0 - expected_positive
                        };
                        let one =
                            group_log_likelihood(&m, h, &m.groups[0], std::slice::from_ref(&r));
                        assert!((one.exp() - expected).abs() < 1e-12);
                        let repeated = group_log_likelihood(&m, h, &m.groups[0], &[r.clone(), r]);
                        assert!((repeated.exp() - expected).abs() < 1e-12);
                    }
                    assert_eq!(group_log_likelihood(&m, h, &m.groups[0], &[]), 0.0);
                    let ignored = [
                        EvidenceRecord {
                            id: 2,
                            content: TestimonyEvidence::Verified {
                                proposition: 7,
                                value: truth,
                            },
                        },
                        EvidenceRecord {
                            id: 3,
                            content: TestimonyEvidence::Report {
                                group: 12,
                                speaker: 19,
                                positive: true,
                            },
                        },
                    ];
                    assert_eq!(group_log_likelihood(&m, h, &m.groups[0], &ignored), 0.0);
                }
            }
        }
    }
}
