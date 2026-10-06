use super::{
    model::{add_mass, PolicyMasses},
    CalibrationView, Error,
};
use crate::deduction::strategic_reporting::{DecisionAction, DecisionObservation};
use serde::{Deserialize, Deserializer, Serialize};

/// An exact probability pair. Valid wire values have a positive denominator
/// and a numerator no greater than it; reduction is not required.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Ratio {
    pub numerator: u64,
    pub denominator: u64,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RatioWire {
    numerator: u64,
    denominator: u64,
}
impl<'de> Deserialize<'de> for Ratio {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let wire = RatioWire::deserialize(deserializer)?;
        if wire.denominator == 0 || wire.numerator > wire.denominator {
            return Err(serde::de::Error::custom(
                "ratio requires 0 <= numerator <= positive denominator",
            ));
        }
        Ok(Self {
            numerator: wire.numerator,
            denominator: wire.denominator,
        })
    }
}

/// An exact signed payoff or probability difference, preserving its pair.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct SignedRatio {
    pub numerator: i64,
    pub denominator: u64,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SignedRatioWire {
    numerator: i64,
    denominator: u64,
}
impl<'de> Deserialize<'de> for SignedRatio {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let wire = SignedRatioWire::deserialize(deserializer)?;
        if wire.denominator == 0 || wire.numerator.unsigned_abs() > wire.denominator {
            return Err(serde::de::Error::custom(
                "signed ratio requires absolute numerator <= positive denominator",
            ));
        }
        Ok(Self {
            numerator: wire.numerator,
            denominator: wire.denominator,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PolicyProbability {
    pub canonical_bits: u32,
    pub probability: Ratio,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LivePrediction {
    pub live_reports: [bool; 2],
    pub probability: Ratio,
    pub truth_and_reports_probability: Ratio,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CalibrationBelief {
    pub policies: Vec<PolicyProbability>,
    pub live: Vec<LivePrediction>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InferenceDecision {
    pub policies: Vec<PolicyProbability>,
    pub posterior_true: Ratio,
    pub action: DecisionAction,
}

fn probabilities(masses: &[(u32, u64)], total: u64) -> Result<Vec<PolicyProbability>, Error> {
    if total == 0 {
        return Err(Error::ZeroEvidence);
    }
    Ok(masses
        .iter()
        .map(|&(canonical_bits, numerator)| PolicyProbability {
            canonical_bits,
            probability: Ratio {
                numerator,
                denominator: total,
            },
        })
        .collect())
}

pub(super) fn calibration(
    policies: &[PolicyMasses],
    view: &CalibrationView,
) -> Result<CalibrationBelief, Error> {
    let mut total = 0;
    let mut masses = Vec::with_capacity(policies.len());
    let mut live_masses = [0; 4];
    let mut live_true_masses = [0; 4];
    for policy in policies {
        let mut mass = 0;
        for row in &policy.histories {
            if row.observation.calibration_truth == view.calibration_truth
                && row.observation.calibration_reports == view.calibration_reports
            {
                add_mass(&mut mass, row.total_mass)?;
                let [own, fixed] = row.observation.live_reports;
                let live_index = 2 * usize::from(own) + usize::from(fixed);
                add_mass(&mut live_masses[live_index], row.total_mass)?;
                add_mass(&mut live_true_masses[live_index], row.true_mass)?;
            }
        }
        add_mass(&mut total, mass)?;
        masses.push((policy.canonical_bits, mass));
    }
    let probabilities = probabilities(&masses, total)?;
    let live = (0..4)
        .map(|index| LivePrediction {
            live_reports: [index & 2 != 0, index & 1 != 0],
            probability: Ratio {
                numerator: live_masses[index],
                denominator: total,
            },
            truth_and_reports_probability: Ratio {
                numerator: live_true_masses[index],
                denominator: total,
            },
        })
        .collect();
    Ok(CalibrationBelief {
        policies: probabilities,
        live,
    })
}

pub(super) fn decide(
    policies: &[PolicyMasses],
    view: &DecisionObservation,
) -> Result<InferenceDecision, Error> {
    let mut total = 0;
    let mut true_mass = 0;
    let mut masses = Vec::with_capacity(policies.len());
    for policy in policies {
        let mut mass = 0;
        for row in &policy.histories {
            if row.observation == *view {
                add_mass(&mut mass, row.total_mass)?;
                add_mass(&mut true_mass, row.true_mass)?;
            }
        }
        add_mass(&mut total, mass)?;
        masses.push((policy.canonical_bits, mass));
    }
    let probabilities = probabilities(&masses, total)?;
    // i128 comparison leaves no unsigned subtraction or double-mass overflow.
    let false_mass = i128::from(total) - i128::from(true_mass);
    let action = if i128::from(true_mass) > false_mass {
        DecisionAction::Intervene
    } else {
        DecisionAction::Abstain
    };
    Ok(InferenceDecision {
        policies: probabilities,
        posterior_true: Ratio {
            numerator: true_mass,
            denominator: total,
        },
        action,
    })
}
