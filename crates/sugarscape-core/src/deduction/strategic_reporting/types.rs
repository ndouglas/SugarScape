use super::{DecisionAction, Error, Permissions, Probability, Profile, GAME_VERSION};
use crate::deduction::{AgentId, FieldError};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UtilityTable {
    pub intervene_true: i8,
    pub intervene_false: i8,
    pub abstain_true: i8,
    pub abstain_false: i8,
}
impl UtilityTable {
    pub fn aligned() -> Self {
        Self {
            intervene_true: 1,
            intervene_false: -1,
            abstain_true: 0,
            abstain_false: 0,
        }
    }
    pub fn opposed() -> Self {
        Self {
            intervene_true: -1,
            intervene_false: 1,
            abstain_true: 0,
            abstain_false: 0,
        }
    }
    pub fn utility(&self, truth: bool, action: DecisionAction) -> i8 {
        match (truth, action) {
            (true, DecisionAction::Intervene) => self.intervene_true,
            (false, DecisionAction::Intervene) => self.intervene_false,
            (true, DecisionAction::Abstain) => self.abstain_true,
            (false, DecisionAction::Abstain) => self.abstain_false,
        }
    }
    pub fn validate(&self) -> Result<(), Error> {
        let mut errors = Vec::new();
        for (field, value) in [
            ("intervene_true", self.intervene_true),
            ("intervene_false", self.intervene_false),
            ("abstain_true", self.abstain_true),
            ("abstain_false", self.abstain_false),
        ] {
            if !(-1..=1).contains(&value) {
                errors.push(FieldError::new(
                    format!("strategic_utility.{field}"),
                    "requires an integer in -1..1",
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
pub fn decision_payoff(truth: bool, action: DecisionAction) -> i8 {
    UtilityTable::aligned().utility(truth, action)
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub version: u16,
    pub strategic: AgentId,
    pub fixed: AgentId,
    pub decider: AgentId,
    pub permissions: Vec<Permissions>,
    pub accuracy: Probability,
    pub fixed_copy_prior: Probability,
    pub strategic_utility: UtilityTable,
}
impl Config {
    pub fn standard(
        accuracy: Probability,
        fixed_copy_prior: Probability,
        strategic_utility: UtilityTable,
    ) -> Self {
        let old = super::super::testimony_game::Config::standard(
            accuracy.clone(),
            fixed_copy_prior.clone(),
        );
        Self {
            version: GAME_VERSION,
            strategic: 0,
            fixed: 1,
            decider: 2,
            permissions: old.permissions,
            accuracy,
            fixed_copy_prior,
            strategic_utility,
        }
    }
    pub fn validate(&self) -> Result<(), Error> {
        let mut errors = Vec::new();
        if self.version != GAME_VERSION {
            errors.push(FieldError::new(
                "version",
                "unsupported strategic reporting version",
            ));
        }
        for (field, p) in [
            ("accuracy", &self.accuracy),
            ("fixed_copy_prior", &self.fixed_copy_prior),
        ] {
            if let Err(e) = validate_probability(field, p) {
                errors.push(e);
            }
        }
        let participants = BTreeSet::from([self.strategic, self.fixed, self.decider]);
        if participants.len() != 3 {
            errors.push(FieldError::new(
                "participants",
                "strategic, fixed and decider must be distinct",
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
            let reporter = p.agent == self.strategic || p.agent == self.fixed;
            if p.receive_signal != reporter
                || p.report != reporter
                || !p.observe_verification
                || p.decide != (p.agent == self.decider)
            {
                errors.push(FieldError::new(
                    format!("permissions[{index}]"),
                    "permissions must match the declared reporting or deciding participant",
                ));
            }
        }
        if let Err(Error::InvalidConfig(utility_errors)) = self.strategic_utility.validate() {
            errors.extend(utility_errors);
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(Error::InvalidConfig(errors))
        }
    }
}
pub(crate) fn validate_probability(field: &str, p: &Probability) -> Result<(), FieldError> {
    if !(1..=16).contains(&p.denominator) || p.numerator > p.denominator {
        Err(FieldError::new(
            field,
            "requires denominator 1..16 and numerator 0..denominator",
        ))
    } else {
        Ok(())
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StrategicObservation {
    pub rules: Config,
    pub signal: bool,
    pub calibration_signal: Option<bool>,
    pub calibration_reports: Option<[bool; 2]>,
    pub calibration_truth: Option<bool>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FixedObservation {
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
