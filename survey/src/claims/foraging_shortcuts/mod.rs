//! Fixed F5 shortcut comparison inputs; manifest construction never runs workers.
mod access;
mod agents;
mod archive;
mod cli;
pub(crate) use cli::cli;
mod io;
mod manifest;
mod physical;
mod report;
mod report_rows;
mod run;
mod scenario;
mod validate;
mod wire;
mod wire_state;
mod wire_view;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum Panel {
    Route,
    Access,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum Geometry {
    Straight,
    Detour,
    Twisting,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum Regime {
    Paid,
    Protected,
    AlreadyOpen,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum CollectionMode {
    Construction,
    Scientific,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct RunKey {
    pub condition: String,
    pub seed: u64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Provenance {
    pub code_revision: String,
    pub protocol_revision: String,
    pub manifest_sha256: String,
    pub collector_sha256: String,
}

pub(super) fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

#[cfg(test)]
mod tests;
