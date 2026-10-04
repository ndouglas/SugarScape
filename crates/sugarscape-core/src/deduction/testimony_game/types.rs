use super::super::{AgentId, FieldError, TestimonyError};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
pub const GAME_VERSION: u16 = 1;
pub const GAME_PROTOCOL_VERSION: u16 = 1;
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Probability {
    pub numerator: u16,
    pub denominator: u16,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Permissions {
    pub agent: AgentId,
    pub receive_signal: bool,
    pub report: bool,
    pub observe_verification: bool,
    pub decide: bool,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub version: u16,
    pub reporters: [AgentId; 2],
    pub decider: AgentId,
    pub permissions: Vec<Permissions>,
    pub accuracy: Probability,
    pub copy_prior: Probability,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReporterObservation {
    pub rules: Config,
    pub signal: bool,
    pub profile: Profile,
    pub calibration_reports: Option<[bool; 2]>,
    pub calibration_truth: Option<bool>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DecisionObservation {
    pub rules: Config,
    pub calibration_reports: [bool; 2],
    pub calibration_truth: bool,
    pub live_reports: [bool; 2],
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Request {
    pub protocol_version: u16,
    pub request_id: u64,
    pub actor: AgentId,
    pub step: Step,
    pub observation: Observation,
    pub legal: Vec<LegalAction>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Response {
    pub request_id: u64,
    pub actor: AgentId,
    pub action: Action,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Outcome {
    pub live_truth: bool,
    pub action: DecisionAction,
    pub payoff: i8,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Checkpoint {
    pub request: Option<Request>,
    pub outcome: Option<Outcome>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Archive {
    pub protocol_version: u16,
    pub config: Config,
    pub seed: u64,
    pub responses: Vec<Response>,
    pub checkpoint: Checkpoint,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Profile {
    Copy,
    Invert,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Step {
    CalibrationReports,
    LiveReports,
    Decision,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DecisionAction {
    Intervene,
    Abstain,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LegalAction {
    Report,
    Intervene,
    Abstain,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Observation {
    Reporter { view: ReporterObservation },
    Decider { view: DecisionObservation },
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Action {
    Report { positive: bool },
    Intervene,
    Abstain,
}
impl<'de> Deserialize<'de> for Action {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
        enum WireAction {
            Report { positive: bool },
            Intervene {},
            Abstain {},
        }
        Ok(match WireAction::deserialize(deserializer)? {
            WireAction::Report { positive } => Self::Report { positive },
            WireAction::Intervene {} => Self::Intervene,
            WireAction::Abstain {} => Self::Abstain,
        })
    }
}
#[derive(Clone, Debug, PartialEq)]
pub enum Error {
    InvalidConfig(Vec<FieldError>),
    InvalidResponse,
    VersionMismatch,
    InvalidArchive { index: Option<usize> },
    InvalidGenome,
    Inference(TestimonyError),
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidConfig(errors) => {
                f.write_str("invalid testimony game configuration")?;
                for e in errors {
                    write!(f, "; {}: {}", e.field, e.message)?;
                }
                Ok(())
            }
            Self::InvalidResponse => f.write_str("invalid response"),
            Self::VersionMismatch => f.write_str("testimony game version mismatch"),
            Self::InvalidArchive { index: Some(index) } => {
                write!(f, "invalid archived response at index {index}")
            }
            Self::InvalidArchive { index: None } => f.write_str("archive checkpoint mismatch"),
            Self::InvalidGenome => f.write_str("invalid testimony game genome"),
            Self::Inference(error) => write!(f, "testimony inference failed: {error}"),
        }
    }
}
impl std::error::Error for Error {}
pub fn report(profile: Profile, signal: bool) -> bool {
    match profile {
        Profile::Copy => signal,
        Profile::Invert => !signal,
    }
}
impl Config {
    pub fn standard(accuracy: Probability, copy_prior: Probability) -> Self {
        Self {
            version: GAME_VERSION,
            reporters: [0, 1],
            decider: 2,
            accuracy,
            copy_prior,
            permissions: vec![
                Permissions {
                    agent: 0,
                    receive_signal: true,
                    report: true,
                    observe_verification: true,
                    decide: false,
                },
                Permissions {
                    agent: 1,
                    receive_signal: true,
                    report: true,
                    observe_verification: true,
                    decide: false,
                },
                Permissions {
                    agent: 2,
                    receive_signal: false,
                    report: false,
                    observe_verification: true,
                    decide: true,
                },
            ],
        }
    }
    pub fn validate(&self) -> Result<(), Error> {
        let mut errors = Vec::new();
        if self.version != GAME_VERSION {
            errors.push(FieldError::new(
                "version",
                "unsupported testimony game version",
            ));
        }
        for (field, p) in [
            ("accuracy", &self.accuracy),
            ("copy_prior", &self.copy_prior),
        ] {
            if !(1..=16).contains(&p.denominator) || p.numerator > p.denominator {
                errors.push(FieldError::new(
                    field,
                    "requires denominator 1..16 and numerator 0..denominator",
                ));
            }
        }
        let participants = BTreeSet::from([self.reporters[0], self.reporters[1], self.decider]);
        if participants.len() != 3 {
            errors.push(FieldError::new(
                "reporters",
                "reporters and decider must be distinct",
            ));
        }
        let agents: BTreeSet<_> = self.permissions.iter().map(|p| p.agent).collect();
        if self.permissions.len() != 3 || agents.len() != 3 || agents != participants {
            errors.push(FieldError::new(
                "permissions",
                "requires exactly one entry per participant",
            ));
        }
        for (index, p) in self.permissions.iter().enumerate() {
            let reporter = self.reporters.contains(&p.agent);
            if p.receive_signal != reporter
                || p.report != reporter
                || !p.observe_verification
                || p.decide != (p.agent == self.decider)
            {
                errors.push(FieldError::new(
                    format!("permissions[{index}]"),
                    "permissions must match the fixed reporting or deciding participant",
                ));
            }
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(Error::InvalidConfig(errors))
        }
    }
}
