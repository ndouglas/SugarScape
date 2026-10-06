use super::{
    types::validate_probability, DecisionAction, DecisionObservation, Error, Listener, Probability,
};
use crate::deduction::testimony_game;
use serde::{Deserialize, Serialize};
/// Frozen legacy algorithm and its assumed copy/invert channel prior.
/// This assumption is serialized separately from the actual generative rules.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FrozenListener {
    pub algorithm: Listener,
    pub assumed_copy_prior: Probability,
}
impl FrozenListener {
    pub fn decide(&self, view: &DecisionObservation) -> Result<DecisionAction, Error> {
        view.rules.validate()?;
        validate_probability("assumed_copy_prior", &self.assumed_copy_prior)
            .map_err(|e| Error::InvalidConfig(vec![e]))?;
        let rules = testimony_game::Config {
            version: testimony_game::GAME_VERSION,
            reporters: [view.rules.strategic, view.rules.fixed],
            decider: view.rules.decider,
            permissions: view.rules.permissions.clone(),
            accuracy: view.rules.accuracy.clone(),
            copy_prior: self.assumed_copy_prior.clone(),
        };
        self.algorithm
            .decide(&testimony_game::DecisionObservation {
                rules,
                calibration_reports: view.calibration_reports,
                calibration_truth: view.calibration_truth,
                live_reports: view.live_reports,
            })
            .map(|decision| decision.action)
            .map_err(Error::LegacyListener)
    }
}
