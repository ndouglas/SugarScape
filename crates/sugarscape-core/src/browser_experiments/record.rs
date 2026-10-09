//! Versioned display records. Reproduction, not an imported success flag, is authority.
use super::{
    catalog, normalize_input, run, FieldError, StudyId, MAX_CHECKPOINTS, MAX_EPISODE_BYTES,
    MAX_INPUT_BYTES,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    collections::BTreeMap,
    io::{self, Write},
};

pub const EPISODE_VERSION: u16 = 1;
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EpisodeKind {
    ExperimentEpisode,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Semantics {
    Trajectory,
    ConditionalCase,
    EvidenceSequence,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Checkpoint {
    pub index: u32,
    pub clock: Value,
    pub kind: String,
    pub public: Value,
    pub local: BTreeMap<String, Value>,
    pub researcher: Option<Value>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EpisodeRecord {
    pub kind: EpisodeKind,
    pub version: u16,
    pub study: StudyId,
    pub rules_identity: String,
    pub input: Value,
    pub semantics: Semantics,
    pub checkpoints: Vec<Checkpoint>,
    pub payload: Value,
}

/// Count compact JSON without allocating an unbounded intermediate string.
struct CountingWriter {
    count: usize,
    limit: usize,
}
impl Write for CountingWriter {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if bytes.len() > self.limit - self.count {
            return Err(io::Error::other("serialized byte limit exceeded"));
        }
        self.count += bytes.len();
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
pub(crate) fn serialized_size<T: Serialize>(
    value: &T,
    limit: usize,
    field: &str,
) -> Result<usize, Vec<FieldError>> {
    let mut writer = CountingWriter { count: 0, limit };
    serde_json::to_writer(&mut writer, value).map_err(|e| super::error(field, e.to_string()))?;
    Ok(writer.count)
}
pub(crate) fn bounded_json<T: Serialize>(
    value: &T,
    limit: usize,
    field: &str,
) -> Result<String, Vec<FieldError>> {
    serialized_size(value, limit, field)?;
    serde_json::to_string(value).map_err(|e| super::error(field, e.to_string()))
}
/// Append a display checkpoint with a bounded, consecutive envelope index.
pub(crate) fn push_checkpoint(
    checkpoints: &mut Vec<Checkpoint>,
    mut checkpoint: Checkpoint,
) -> Result<(), Vec<FieldError>> {
    if checkpoints.len() >= MAX_CHECKPOINTS {
        return Err(super::error(
            "checkpoints",
            "episode exceeds 4096 checkpoints",
        ));
    }
    checkpoint.index = checkpoints.len() as u32;
    checkpoints.push(checkpoint);
    Ok(())
}
pub(crate) fn check_bounds(record: &EpisodeRecord) -> Result<(), Vec<FieldError>> {
    if record.checkpoints.len() > MAX_CHECKPOINTS {
        return Err(super::error(
            "checkpoints",
            "episode exceeds 4096 checkpoints",
        ));
    }
    serialized_size(&record.input, MAX_INPUT_BYTES, "input")?;
    serialized_size(record, MAX_EPISODE_BYTES, "episode")?;
    Ok(())
}
pub fn episode_json(record: &EpisodeRecord) -> Result<String, Vec<FieldError>> {
    check_bounds(record)?;
    serde_json::to_string(record).map_err(|e| super::error("episode", e.to_string()))
}
pub fn validate_episode(json: &str) -> Result<EpisodeRecord, Vec<FieldError>> {
    if json.len() > MAX_EPISODE_BYTES {
        return Err(super::error("episode", "episode exceeds 16 MiB"));
    }
    let saved: EpisodeRecord =
        serde_json::from_str(json).map_err(|e| super::error("episode", e.to_string()))?;
    if saved.version != EPISODE_VERSION {
        return Err(super::error(
            "version",
            "unsupported episode format version",
        ));
    }
    let expected_identity = catalog::rules_identity(saved.study)?;
    if saved.rules_identity != expected_identity {
        if matches!(
            saved.study,
            StudyId::ForagingFixed | StudyId::ForagingPassage | StudyId::ForagingConstruction
        ) {
            if let (Some((saved_source, saved_target)), Some((expected_source, expected_target))) = (
                saved.rules_identity.rsplit_once(":target:"),
                expected_identity.rsplit_once(":target:"),
            ) {
                if saved_source == expected_source && saved_target != expected_target {
                    return Err(super::error("rules_identity", format!("incompatible CPFA target {saved_target:?}; reconstruction requires {expected_target:?}")));
                }
            }
        }
        return Err(super::error(
            "rules_identity",
            "engine source or rules version mismatch",
        ));
    }
    check_bounds(&saved)?;
    let input = normalize_input(&bounded_json(&saved.input, MAX_INPUT_BYTES, "input")?)?;
    if input.study() != saved.study {
        return Err(super::error("study", "input study does not match record"));
    }
    let fresh = run(&input)?;
    if !super::equivalence::records_equivalent(&fresh, &saved) {
        return Err(super::error(
            "episode",
            "saved episode differs from reconstructed episode",
        ));
    }
    Ok(fresh)
}
