use super::*;
use crate::rng::{seeded, SimRng};
use rand::Rng;

/// Privileged host state. Only actor-specific requests cross the player boundary.
pub struct Session {
    config: Config,
    seed: u64,
    truths: [bool; 2],
    profiles: [Profile; 2],
    signals: [[bool; 2]; 2],
    responses: Vec<Response>,
    reports: [[bool; 2]; 2],
    outcome: Option<Outcome>,
}

fn rational_draw(rng: &mut SimRng, probability: &Probability) -> bool {
    rng.gen_range(0..probability.denominator) < probability.numerator
}

impl Session {
    pub fn new(config: Config, seed: u64) -> Result<Self, Error> {
        config.validate()?;
        let mut rng = seeded(seed);
        // GAME_VERSION 1: C, T, profiles 0/1, then C signals 0/1 and T signals 0/1.
        // Even deterministic rational endpoints consume their scheduled draw.
        let truths = [rng.gen_bool(0.5), rng.gen_bool(0.5)];
        let profiles = std::array::from_fn(|_| {
            if rational_draw(&mut rng, &config.copy_prior) {
                Profile::Copy
            } else {
                Profile::Invert
            }
        });
        let signals = std::array::from_fn(|exchange| {
            std::array::from_fn(|_| truths[exchange] == rational_draw(&mut rng, &config.accuracy))
        });
        Ok(Self {
            config,
            seed,
            truths,
            profiles,
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
            let exchange = count / 2;
            let slot = count % 2;
            (
                self.config.reporters[slot],
                if exchange == 0 {
                    Step::CalibrationReports
                } else {
                    Step::LiveReports
                },
                Observation::Reporter {
                    view: ReporterObservation {
                        rules: self.config.clone(),
                        signal: self.signals[exchange][slot],
                        profile: self.profiles[slot],
                        calibration_reports: (exchange == 1).then_some(self.reports[0]),
                        calibration_truth: (exchange == 1).then_some(self.truths[0]),
                    },
                },
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
        // Validate before mutating any report buffer, transcript, or outcome.
        match (&request.step, &response.action) {
            (Step::CalibrationReports | Step::LiveReports, Action::Report { positive }) => {
                self.reports[count / 2][count % 2] = *positive;
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
                    payoff: match action {
                        DecisionAction::Abstain => 0,
                        DecisionAction::Intervene if self.truths[1] => 1,
                        DecisionAction::Intervene => -1,
                    },
                });
            }
            _ => return Err(Error::InvalidResponse),
        }
        self.responses.push(response);
        Ok(())
    }

    pub fn outcome(&self) -> Option<&Outcome> {
        self.outcome.as_ref()
    }

    pub fn archive(&self) -> Archive {
        Archive {
            protocol_version: GAME_PROTOCOL_VERSION,
            config: self.config.clone(),
            seed: self.seed,
            responses: self.responses.clone(),
            checkpoint: Checkpoint {
                request: self.request(),
                outcome: self.outcome.clone(),
            },
        }
    }
}

pub fn replay(archive: &Archive) -> Result<Session, Error> {
    if archive.protocol_version != GAME_PROTOCOL_VERSION || archive.config.version != GAME_VERSION {
        return Err(Error::VersionMismatch);
    }
    let mut session = Session::new(archive.config.clone(), archive.seed)?;
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
