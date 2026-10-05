use super::*;
use crate::deduction::AgentId;
use crate::rng::{seeded, SimRng};
use rand::Rng;
use serde::{Deserialize, Serialize};
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Step {
    CalibrationReports,
    LiveReports,
    Decision,
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
    Strategic { view: StrategicObservation },
    Fixed { view: FixedObservation },
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
    pub reporter_utility: i8,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Checkpoint {
    pub request: Option<Request>,
    pub outcome: Option<Outcome>,
}
/// Privileged deterministic replay data, not authenticated evidence.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Archive {
    pub protocol_version: u16,
    pub config: Config,
    pub seed: u64,
    pub responses: Vec<Response>,
    pub checkpoint: Checkpoint,
}
/// Host-only state. Actor-specific requests are the controller boundary.
pub struct Session {
    config: Config,
    seed: u64,
    truths: [bool; 2],
    fixed_profile: Profile,
    signals: [[bool; 2]; 2],
    responses: Vec<Response>,
    reports: [[bool; 2]; 2],
    outcome: Option<Outcome>,
}
fn rational_draw(rng: &mut SimRng, p: &Probability) -> bool {
    rng.gen_range(0..p.denominator) < p.numerator
}
impl Session {
    pub fn new(config: Config, seed: u64) -> Result<Self, Error> {
        config.validate()?;
        let mut rng = seeded(seed);
        // Version 1 draws: C,T,fixed profile,C strategic/fixed signals,T strategic/fixed signals.
        // Rational endpoints deliberately consume their scheduled draws.
        let truths = [rng.gen_bool(0.5), rng.gen_bool(0.5)];
        let fixed_profile = if rational_draw(&mut rng, &config.fixed_copy_prior) {
            Profile::Copy
        } else {
            Profile::Invert
        };
        let signals = std::array::from_fn(|phase| {
            std::array::from_fn(|_| truths[phase] == rational_draw(&mut rng, &config.accuracy))
        });
        Ok(Self {
            config,
            seed,
            truths,
            fixed_profile,
            signals,
            responses: Vec::new(),
            reports: [[false; 2]; 2],
            outcome: None,
        })
    }
    pub fn request(&self) -> Option<Request> {
        let count = self.responses.len();
        if count >= 5 {
            return None;
        }
        let (actor, step, observation, legal) = if count < 4 {
            let phase = count / 2;
            let slot = count % 2;
            let calibration_reports = (phase == 1).then_some(self.reports[0]);
            let calibration_truth = (phase == 1).then_some(self.truths[0]);
            let observation = if slot == 0 {
                Observation::Strategic {
                    view: StrategicObservation {
                        rules: self.config.clone(),
                        signal: self.signals[phase][0],
                        calibration_signal: (phase == 1).then_some(self.signals[0][0]),
                        calibration_reports,
                        calibration_truth,
                    },
                }
            } else {
                Observation::Fixed {
                    view: FixedObservation {
                        rules: self.config.clone(),
                        signal: self.signals[phase][1],
                        profile: self.fixed_profile,
                        calibration_reports,
                        calibration_truth,
                    },
                }
            };
            (
                if slot == 0 {
                    self.config.strategic
                } else {
                    self.config.fixed
                },
                if phase == 0 {
                    Step::CalibrationReports
                } else {
                    Step::LiveReports
                },
                observation,
                vec![LegalAction::Report],
            )
        } else {
            (
                self.config.decider,
                Step::Decision,
                Observation::Decider {
                    view: DecisionObservation {
                        rules: self.config.clone(),
                        calibration_reports: self.reports[0],
                        calibration_truth: self.truths[0],
                        live_reports: self.reports[1],
                    },
                },
                vec![LegalAction::Intervene, LegalAction::Abstain],
            )
        };
        Some(Request {
            protocol_version: GAME_PROTOCOL_VERSION,
            request_id: count as u64 + 1,
            actor,
            step,
            observation,
            legal,
        })
    }
    pub fn submit(&mut self, response: Response) -> Result<(), Error> {
        let request = self.request().ok_or(Error::InvalidResponse)?;
        if response.actor != request.actor || response.request_id != request.request_id {
            return Err(Error::InvalidResponse);
        }
        let count = self.responses.len();
        // All rejection paths precede mutation of buffers, transcript and outcome.
        match (&request.step, &response.action) {
            (Step::CalibrationReports | Step::LiveReports, Action::Report { positive }) => {
                self.reports[count / 2][count % 2] = *positive
            }
            (Step::Decision, Action::Intervene | Action::Abstain) => {
                let action = if response.action == Action::Intervene {
                    DecisionAction::Intervene
                } else {
                    DecisionAction::Abstain
                };
                self.outcome = Some(Outcome {
                    live_truth: self.truths[1],
                    action,
                    payoff: decision_payoff(self.truths[1], action),
                    reporter_utility: self
                        .config
                        .strategic_utility
                        .utility(self.truths[1], action),
                });
            }
            _ => return Err(Error::InvalidResponse),
        }
        self.responses.push(response);
        Ok(())
    }
    pub fn outcome(&self) -> Option<Outcome> {
        self.outcome.clone()
    }
    pub fn archive(&self) -> Archive {
        Archive {
            protocol_version: GAME_PROTOCOL_VERSION,
            config: self.config.clone(),
            seed: self.seed,
            responses: self.responses.clone(),
            checkpoint: Checkpoint {
                request: self.request(),
                outcome: self.outcome(),
            },
        }
    }
    pub fn replay(archive: &Archive) -> Result<Self, Error> {
        if archive.protocol_version != GAME_PROTOCOL_VERSION
            || archive.config.version != GAME_VERSION
        {
            return Err(Error::VersionMismatch);
        }
        let mut session = Self::new(archive.config.clone(), archive.seed)?;
        for (index, response) in archive.responses.iter().enumerate() {
            session
                .submit(response.clone())
                .map_err(|_| Error::InvalidArchive { index: Some(index) })?;
        }
        if session.archive().checkpoint != archive.checkpoint {
            return Err(Error::InvalidArchive { index: None });
        }
        Ok(session)
    }
}
