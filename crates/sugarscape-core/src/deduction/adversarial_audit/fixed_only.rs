use super::{validate_rules, Error, FrozenDecision};
use crate::deduction::strategic_reporting::{enumerate, histories, Config, DecisionAction, Policy};
use crate::deduction::strategy_inference::Ratio;
use serde::{Deserialize, Serialize};

/// Genuine C verification and the two fixed-channel reports, with public rules.
/// No strategic reports, private signals, live truth or actual policy enter this view.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FixedView {
    pub rules: Config,
    pub calibration_truth: bool,
    pub calibration_report: bool,
    pub live_report: bool,
}

#[derive(Clone, Debug)]
pub struct FixedMass {
    view: FixedView,
    total_mass: u64,
    true_mass: u64,
}

/// Immutable public-rule marginal; private mass rows cannot be deserialized.
#[derive(Clone, Debug)]
pub struct FixedModel {
    config: Config,
    denominator: u64,
    rows: [FixedMass; 8],
}

fn view_index(view: &FixedView) -> usize {
    (usize::from(view.calibration_truth) << 2)
        | (usize::from(view.calibration_report) << 1)
        | usize::from(view.live_report)
}

impl FixedModel {
    pub fn new(config: &Config) -> Result<Self, Error> {
        validate_rules(config)?;
        let distribution = enumerate(config)?;
        let mut rows = std::array::from_fn(|index| FixedMass {
            view: FixedView {
                rules: config.clone(),
                calibration_truth: index & 4 != 0,
                calibration_report: index & 2 != 0,
                live_report: index & 1 != 0,
            },
            total_mass: 0,
            true_mass: 0,
        });
        // Constant-positive reports are only a computational device for obtaining
        // public history masses. Summing away both strategic bits removes that
        // hypothetical policy; no realized policy is supplied to this listener.
        for history in histories(&distribution, &Policy::positive())? {
            let view = FixedView {
                rules: config.clone(),
                calibration_truth: history.observation.calibration_truth,
                calibration_report: history.observation.calibration_reports[1],
                live_report: history.observation.live_reports[1],
            };
            let row = &mut rows[view_index(&view)];
            row.total_mass = row
                .total_mass
                .checked_add(history.total_mass)
                .ok_or(Error::ArithmeticOverflow)?;
            row.true_mass = row
                .true_mass
                .checked_add(history.true_mass)
                .ok_or(Error::ArithmeticOverflow)?;
        }
        let total = rows.iter().try_fold(0u64, |sum, row| {
            sum.checked_add(row.total_mass)
                .ok_or(Error::ArithmeticOverflow)
        })?;
        if total != distribution.denominator() {
            return Err(Error::InvalidRules(
                "fixed marginal masses do not normalize",
            ));
        }
        Ok(Self {
            config: config.clone(),
            denominator: distribution.denominator(),
            rows,
        })
    }

    pub fn denominator(&self) -> u64 {
        self.denominator
    }

    pub fn decide(&self, view: &FixedView) -> Result<FrozenDecision, Error> {
        validate_rules(&view.rules)?;
        if view.rules != self.config {
            return Err(Error::InvalidRules(
                "public rules differ from fixed model rules",
            ));
        }
        let row = &self.rows[view_index(view)];
        if &row.view != view {
            return Err(Error::InvalidRules(
                "fixed evidence differs from indexed row",
            ));
        }
        if row.total_mass == 0 {
            return Err(Error::ZeroEvidence);
        }
        let false_mass = i128::from(row.total_mass)
            .checked_sub(i128::from(row.true_mass))
            .ok_or(Error::ArithmeticOverflow)?;
        let action = if i128::from(row.true_mass) > false_mass {
            DecisionAction::Intervene
        } else {
            DecisionAction::Abstain
        };
        Ok(FrozenDecision {
            posterior_true: Some(Ratio {
                numerator: row.true_mass,
                denominator: row.total_mass,
            }),
            action,
        })
    }
}
