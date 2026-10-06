//! Strategic reporting with explicit utility and frozen legacy listener assumptions.
mod evolution;
pub use evolution::*;
mod enumeration;
pub use enumeration::*;
mod best_response;
pub use best_response::*;
mod diagnostics;
pub use diagnostics::*;
mod session;
pub use session::*;
mod types;
pub use types::*;
mod policy;
pub use policy::*;
mod listeners;
pub use super::testimony_game::{
    DecisionAction, Genome, Listener, Permissions, Probability, Profile,
};
pub use listeners::*;
pub const GAME_VERSION: u16 = 1;
pub const GAME_PROTOCOL_VERSION: u16 = 1;
#[derive(Clone, Debug, PartialEq)]
pub enum Error {
    InvalidConfig(Vec<super::FieldError>),
    InvalidDiagnostic(&'static str),
    InvalidResponse,
    VersionMismatch,
    InvalidArchive { index: Option<usize> },
    InvalidPolicy,
    ArithmeticOverflow,
    InvalidDistribution(&'static str),
    InvalidPanel(String),
    InvalidObservation(&'static str),
    LegacyListener(super::testimony_game::Error),
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidConfig(errors) => {
                f.write_str("invalid strategic reporting configuration")?;
                for error in errors {
                    write!(f, "; {}: {}", error.field, error.message)?;
                }
                Ok(())
            }
            Self::ArithmeticOverflow => {
                f.write_str("strategic reporting exact arithmetic overflow")
            }
            Self::InvalidDistribution(message) => {
                write!(f, "invalid strategic reporting distribution: {message}")
            }
            Self::InvalidPanel(message) => write!(f, "invalid frozen listener panel: {message}"),
            Self::InvalidDiagnostic(message) => {
                write!(f, "invalid strategic reporting diagnostic: {message}")
            }
            Self::InvalidResponse => f.write_str("invalid strategic reporting response"),
            Self::VersionMismatch => f.write_str("strategic reporting version mismatch"),
            Self::InvalidArchive { index: Some(index) } => write!(
                f,
                "invalid strategic reporting archived response at index {index}"
            ),
            Self::InvalidArchive { index: None } => {
                f.write_str("strategic reporting archive checkpoint mismatch")
            }
            Self::InvalidPolicy => f.write_str("policy requires an unsigned 18-bit encoding"),
            Self::InvalidObservation(message) => {
                write!(f, "invalid strategic reporting observation: {message}")
            }
            Self::LegacyListener(error) => {
                write!(f, "legacy assumed-channel listener failed: {error}")
            }
        }
    }
}
impl std::error::Error for Error {}
#[cfg(test)]
mod tests {
    mod atoms;
    mod best_response;
    mod enumeration;
    mod session;
}
