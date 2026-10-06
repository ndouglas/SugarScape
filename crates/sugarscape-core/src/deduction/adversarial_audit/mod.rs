//! Exact adversarial reporting audit using public evidence only.
mod actions;
mod best_response;
mod diagnostics;
mod evaluation;
mod fixed_only;
mod scoring;
pub use super::strategic_reporting::Evaluation as AuditEvaluation;
use super::{strategic_reporting, strategy_inference};
pub use actions::{
    history_index, history_view, ActionRow, ActionSnapshot, ControllerKind, FrozenActions,
    FrozenDecision,
};
pub use best_response::{
    canonical_policies, exact_best_response, fitness_table, AttackBasis, BasisRow, BestResponse,
    FitnessRow,
};
pub use diagnostics::{
    diagnose, report_integrity, BoundCheck, ControlEvaluation, CrossTargetEvaluation,
    DiagnosticCheck, DiagnosticReport, EnvironmentRow, FitnessTable, FixedModelRow,
    FixedPosteriorRow, Metadata, NominalEvaluation, SnapshotRow, TargetedAudit,
};
pub use evaluation::{evaluate_fixed, evaluate_mixture, guarantee_shortfall, nominal_difference};
pub use fixed_only::{FixedMass, FixedModel, FixedView};
pub use scoring::{score, Score};

pub const ACTION_VERSION: u16 = 1;
pub const REPORT_VERSION: &str = "adversarial-audit-diagnostic-v1";

#[derive(Clone, Debug, PartialEq)]
pub enum Error {
    InvalidRules(&'static str),
    InvalidActions(&'static str),
    InvalidReport(&'static str),
    ZeroEvidence,
    ArithmeticOverflow,
    Reporting(strategic_reporting::Error),
    Inference(strategy_inference::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidRules(message) => write!(f, "invalid adversarial audit rules: {message}"),
            Self::InvalidActions(message) => {
                write!(f, "invalid adversarial audit actions: {message}")
            }
            Self::InvalidReport(message) => {
                write!(f, "invalid adversarial audit report: {message}")
            }
            Self::ZeroEvidence => f.write_str("adversarial audit history has zero evidence"),
            Self::ArithmeticOverflow => f.write_str("adversarial audit exact arithmetic overflow"),
            Self::Reporting(error) => write!(f, "adversarial audit reporting failed: {error}"),
            Self::Inference(error) => write!(f, "adversarial audit inference failed: {error}"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Reporting(error) => Some(error),
            Self::Inference(error) => Some(error),
            _ => None,
        }
    }
}

impl From<strategic_reporting::Error> for Error {
    fn from(error: strategic_reporting::Error) -> Self {
        Self::Reporting(error)
    }
}
impl From<strategy_inference::Error> for Error {
    fn from(error: strategy_inference::Error) -> Self {
        // Unsupported evidence retains the audit's explicit error, not an action.
        match error {
            strategy_inference::Error::ZeroEvidence => Self::ZeroEvidence,
            other => Self::Inference(other),
        }
    }
}

pub(super) fn validate_rules(config: &strategic_reporting::Config) -> Result<(), Error> {
    config.validate()?;
    if config.strategic_utility != strategic_reporting::UtilityTable::opposed() {
        return Err(Error::InvalidRules("requires opposed strategic utility"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    mod actions;
    mod best_response;
    mod diagnostics;
    mod evaluation;
}
