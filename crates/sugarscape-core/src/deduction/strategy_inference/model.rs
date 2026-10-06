use super::{canonical_bits, conditioning, CalibrationBelief, Catalog, Error, InferenceDecision};
use crate::deduction::strategic_reporting::{
    enumerate, histories, Config, DecisionObservation, HistoryMass,
};
use serde::{Deserialize, Serialize};

/// Verified calibration evidence, before either live report is observed.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CalibrationView {
    pub rules: Config,
    pub calibration_truth: bool,
    pub calibration_reports: [bool; 2],
}

pub(super) struct PolicyMasses {
    pub(super) canonical_bits: u32,
    pub(super) histories: Vec<HistoryMass>,
}

/// Immutable exact joint masses for a declared policy prior and public rules.
///
/// No realized policy, private world, or current evaluation enters this model.
/// Construction validates the rules before allocating the exact distribution.
pub struct Model {
    config: Config,
    catalog: Catalog,
    denominator: u64,
    policies: Vec<PolicyMasses>,
}

// 4 * 16^4 * 16 * 256: maximum distribution denominator times catalog weight.
pub(super) const MAX_MASS: u64 = 1_073_741_824;

pub(super) fn checked_mass(value: Option<u64>) -> Result<u64, Error> {
    value
        .filter(|mass| *mass <= MAX_MASS)
        .ok_or(Error::ArithmeticOverflow)
}

pub(super) fn add_mass(target: &mut u64, value: u64) -> Result<(), Error> {
    *target = checked_mass(target.checked_add(value))?;
    Ok(())
}

/// Shared bounded weighting for inference and actual-mixture evaluation.
pub(super) fn weighted_policy_masses(
    config: &Config,
    catalog: &Catalog,
) -> Result<(u64, Vec<PolicyMasses>), Error> {
    config.validate()?;
    let distribution = enumerate(config)?;
    let denominator = checked_mass(
        distribution
            .denominator()
            .checked_mul(u64::from(catalog.total_weight())),
    )?;
    let mut policies = Vec::with_capacity(catalog.entries().len());
    let mut total = 0;
    for entry in catalog.entries() {
        let mut rows = histories(&distribution, &entry.policy)?;
        for row in &mut rows {
            row.total_mass = checked_mass(row.total_mass.checked_mul(u64::from(entry.weight)))?;
            row.true_mass = checked_mass(row.true_mass.checked_mul(u64::from(entry.weight)))?;
            add_mass(&mut total, row.total_mass)?;
        }
        policies.push(PolicyMasses {
            canonical_bits: canonical_bits(&entry.policy)?,
            histories: rows,
        });
    }
    if total != denominator {
        return Err(Error::InvalidObservation("joint masses do not normalize"));
    }
    Ok((denominator, policies))
}

impl Model {
    pub fn new(config: &Config, catalog: &Catalog) -> Result<Self, Error> {
        let (denominator, policies) = weighted_policy_masses(config, catalog)?;
        Ok(Self {
            config: config.clone(),
            catalog: catalog.clone(),
            denominator,
            policies,
        })
    }

    pub fn config(&self) -> &Config {
        &self.config
    }
    pub fn catalog(&self) -> &Catalog {
        &self.catalog
    }
    pub fn denominator(&self) -> u64 {
        self.denominator
    }

    pub fn calibration(&self, view: &CalibrationView) -> Result<CalibrationBelief, Error> {
        self.validate_rules(&view.rules)?;
        conditioning::calibration(&self.policies, view)
    }

    pub fn decide(&self, view: &DecisionObservation) -> Result<InferenceDecision, Error> {
        self.validate_rules(&view.rules)?;
        conditioning::decide(&self.policies, view)
    }

    fn validate_rules(&self, rules: &Config) -> Result<(), Error> {
        rules.validate()?;
        if rules != &self.config {
            return Err(Error::InvalidObservation(
                "public rules differ from the model rules",
            ));
        }
        Ok(())
    }
}
