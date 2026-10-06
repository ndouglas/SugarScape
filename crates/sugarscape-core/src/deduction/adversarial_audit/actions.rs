use super::{validate_rules, Error, FixedModel, FixedView, ACTION_VERSION};
use crate::deduction::strategic_reporting::{Config, DecisionAction, DecisionObservation};
use crate::deduction::strategy_inference::{Catalog, Model, Ratio};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ControllerKind {
    StrategyUniform,
    StrategyOptimizationInformed,
    FixedOnly,
    Passive,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FrozenDecision {
    /// Passive decisions have no posterior; other controllers supply exact evidence.
    pub posterior_true: Option<Ratio>,
    pub action: DecisionAction,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ActionRow {
    pub index: u8,
    pub observation: DecisionObservation,
    pub decision: FrozenDecision,
}

/// Public transport shape. Certification requires `FrozenActions::from_snapshot`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ActionSnapshot {
    pub version: u16,
    pub rules: Config,
    pub controller: ControllerKind,
    pub rows: Vec<ActionRow>,
}

/// Named decisions reconstructed from public rules; the private array is immutable.
#[derive(Clone, Debug)]
pub struct FrozenActions {
    config: Config,
    controller: ControllerKind,
    rows: [ActionRow; 32],
}

pub fn history_view(config: &Config, index: u8) -> Result<DecisionObservation, Error> {
    validate_rules(config)?;
    if index >= 32 {
        return Err(Error::InvalidActions("history index requires 0..32"));
    }
    Ok(DecisionObservation {
        rules: config.clone(),
        calibration_truth: index & 16 != 0,
        calibration_reports: [index & 8 != 0, index & 4 != 0],
        live_reports: [index & 2 != 0, index & 1 != 0],
    })
}

pub fn history_index(view: &DecisionObservation) -> usize {
    (usize::from(view.calibration_truth) << 4)
        | (usize::from(view.calibration_reports[0]) << 3)
        | (usize::from(view.calibration_reports[1]) << 2)
        | (usize::from(view.live_reports[0]) << 1)
        | usize::from(view.live_reports[1])
}

impl FrozenActions {
    pub fn freeze(config: &Config, controller: ControllerKind) -> Result<Self, Error> {
        validate_rules(config)?;
        let strategy = match controller {
            ControllerKind::StrategyUniform => Some(Model::new(config, &Catalog::uniform())?),
            ControllerKind::StrategyOptimizationInformed => {
                Some(Model::new(config, &Catalog::optimization_informed())?)
            }
            _ => None,
        };
        let fixed = if controller == ControllerKind::FixedOnly {
            Some(FixedModel::new(config)?)
        } else {
            None
        };
        let mut rows = Vec::with_capacity(32);
        for index in 0..32 {
            let observation = history_view(config, index)?;
            let decision = if let Some(model) = &strategy {
                let inference = model.decide(&observation)?;
                FrozenDecision {
                    posterior_true: Some(inference.posterior_true),
                    action: inference.action,
                }
            } else if let Some(model) = &fixed {
                model.decide(&FixedView {
                    rules: config.clone(),
                    calibration_truth: observation.calibration_truth,
                    calibration_report: observation.calibration_reports[1],
                    live_report: observation.live_reports[1],
                })?
            } else {
                FrozenDecision {
                    posterior_true: None,
                    action: DecisionAction::Abstain,
                }
            };
            rows.push(ActionRow {
                index,
                observation,
                decision,
            });
        }
        Ok(Self {
            config: config.clone(),
            controller,
            rows: rows
                .try_into()
                .map_err(|_| Error::InvalidActions("freeze requires 32 rows"))?,
        })
    }

    pub fn from_snapshot(snapshot: &ActionSnapshot) -> Result<Self, Error> {
        if snapshot.version != ACTION_VERSION {
            return Err(Error::InvalidActions("unsupported action version"));
        }
        validate_rules(&snapshot.rules)?;
        if snapshot.rows.len() != 32 {
            return Err(Error::InvalidActions("snapshot requires exactly 32 rows"));
        }
        for (index, row) in snapshot.rows.iter().enumerate() {
            if usize::from(row.index) != index || history_index(&row.observation) != index {
                return Err(Error::InvalidActions(
                    "snapshot history index or order differs",
                ));
            }
            if row.observation.rules != snapshot.rules {
                return Err(Error::InvalidActions(
                    "snapshot row rules differ from declared rules",
                ));
            }
        }
        let reconstructed = Self::freeze(&snapshot.rules, snapshot.controller)?;
        if reconstructed.snapshot() != *snapshot {
            return Err(Error::InvalidActions(
                "snapshot differs from reconstructed named controller",
            ));
        }
        Ok(reconstructed)
    }

    pub fn config(&self) -> &Config {
        &self.config
    }
    pub fn controller(&self) -> &ControllerKind {
        &self.controller
    }
    pub fn rows(&self) -> &[ActionRow; 32] {
        &self.rows
    }
    pub fn snapshot(&self) -> ActionSnapshot {
        ActionSnapshot {
            version: ACTION_VERSION,
            rules: self.config.clone(),
            controller: self.controller,
            rows: self.rows.to_vec(),
        }
    }
}
