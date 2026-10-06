//! Exact listener inference over explicitly supplied reporting-policy priors.
mod catalog;
mod conditioning;
mod diagnostics;
mod evaluation;
mod model;
pub use catalog::{canonical_bits, Catalog, WeightedPolicy};
pub use conditioning::{
    CalibrationBelief, InferenceDecision, LivePrediction, PolicyProbability, Ratio, SignedRatio,
};
pub use diagnostics::{
    diagnose, report_integrity, CalibrationRow, CatalogRow, DiagnosticCheck, DiagnosticReport,
    EnvironmentRow, ExactValue, FixedRow, InferenceHistoryRow, InferenceModelRow, ListenerIdentity,
    Metadata, MixtureRow, PolicyProvenance,
};
pub use evaluation::{
    evaluate_fixed, evaluate_mixture, EvaluatedHistory, EvaluatedListener, Evaluation,
};
pub use model::{CalibrationView, Model};

use super::strategic_reporting;

pub const CATALOG_VERSION: u16 = 1;
pub const REPORT_VERSION: &str = "strategy-inference-diagnostic-v1";

#[derive(Clone, Debug, PartialEq)]
pub enum Error {
    InvalidCatalog(&'static str),
    InvalidObservation(&'static str),
    ZeroEvidence,
    ArithmeticOverflow,
    Existing(strategic_reporting::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidCatalog(message) => {
                write!(f, "invalid strategy inference catalog: {message}")
            }
            Self::InvalidObservation(message) => {
                write!(f, "invalid strategy inference observation: {message}")
            }
            Self::ZeroEvidence => f.write_str("strategy inference history has zero prior evidence"),
            Self::ArithmeticOverflow => f.write_str("strategy inference exact arithmetic overflow"),
            Self::Existing(error) => write!(f, "strategy inference failed: {error}"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Existing(error) => Some(error),
            _ => None,
        }
    }
}

impl From<strategic_reporting::Error> for Error {
    fn from(error: strategic_reporting::Error) -> Self {
        Self::Existing(error)
    }
}

#[cfg(test)]
mod tests {
    mod catalog;
    mod diagnostics;
    mod evaluation;
    mod inference;
}
