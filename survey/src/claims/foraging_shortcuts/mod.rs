//! Fixed F5 shortcut comparison inputs; manifest construction never runs workers.
mod manifest;
mod scenario;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[allow(
    dead_code,
    reason = "Panel is consumed by the Task 2 wire validator and Task 4 collector"
)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum Panel {
    Route,
    Access,
}

#[allow(
    dead_code,
    reason = "Geometry is consumed by the Task 2 wire validator and Task 4 collector"
)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum Geometry {
    Straight,
    Detour,
    Twisting,
}

#[allow(
    dead_code,
    reason = "Regime is consumed by the Task 2 wire validator and Task 4 collector"
)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum Regime {
    Paid,
    Protected,
    AlreadyOpen,
}

#[allow(
    dead_code,
    reason = "CollectionMode is consumed by the Task 3 archive and Task 4 collector"
)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum CollectionMode {
    Construction,
    Scientific,
}

#[allow(dead_code, reason = "RunKey is consumed by the Task 3 archive")]
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct RunKey {
    pub condition: String,
    pub seed: u64,
}

#[allow(
    dead_code,
    reason = "Provenance is consumed by the Task 3 archive and Task 4 collector"
)]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Provenance {
    pub code_revision: String,
    pub protocol_revision: String,
    pub manifest_sha256: String,
    pub collector_sha256: String,
}

#[allow(
    dead_code,
    reason = "sha256 is consumed by the Task 3 archive and Task 4 collector"
)]
pub(super) fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

#[cfg(test)]
mod tests;
