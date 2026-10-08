//! Strict saved observations; these types cannot restore a core World.
use super::wire_view::{WireSnapshot, WireSummary};
use super::{CollectionMode, Geometry, Panel, Provenance, Regime, RunKey};
use serde::{Deserialize, Serialize};
use std::io::{self, Write};
use sugarscape_core::foraging::construction as core;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Copy, Eq, PartialOrd, Ord)]
#[serde(deny_unknown_fields)]
pub(super) struct WirePos {
    pub(super) x: u32,
    pub(super) y: u32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct WireResource {
    pub(super) id: u64,
    pub(super) pos: WirePos,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct WireParameters {
    pub(super) p_search: WireFloat,
    pub(super) p_return: WireFloat,
    pub(super) lambda_fidelity: WireFloat,
    pub(super) lambda_publish: WireFloat,
    pub(super) lambda_waypoint: WireFloat,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct WireSetup {
    pub(super) width: u32,
    pub(super) height: u32,
    pub(super) open: Vec<WirePos>,
    pub(super) diggable: Vec<WirePos>,
    pub(super) waste: WirePos,
    pub(super) nest: Vec<WirePos>,
    pub(super) workers: Vec<WirePos>,
    pub(super) food: Vec<WireResource>,
    pub(super) parameters: WireParameters,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct WireRunOptions {
    pub(super) ticks: u32,
    pub(super) sample_every: u32,
    pub(super) snapshots: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct WireEpisode {
    pub(super) setup: WireSetup,
    pub(super) seed: u64,
    pub(super) options: WireRunOptions,
    pub(super) summary: WireSummary,
    pub(super) snapshots: Vec<WireSnapshot>,
    pub(super) snapshot_bytes: u64,
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct WireFloat(pub(super) String);
fn finite_token(token: &str) -> Result<(), String> {
    if token.len() > 64 {
        return Err("numeric token exceeds 64 bytes".into());
    }
    let value = serde_json::from_str::<f64>(token).map_err(|e| e.to_string())?;
    if !value.is_finite() {
        return Err("numeric token must be finite".into());
    }
    Ok(())
}
impl Serialize for WireFloat {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        finite_token(&self.0).map_err(serde::ser::Error::custom)?;
        let raw = serde_json::value::RawValue::from_string(self.0.clone())
            .map_err(serde::ser::Error::custom)?;
        raw.serialize(s)
    }
}
impl<'de> Deserialize<'de> for WireFloat {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let raw = Box::<serde_json::value::RawValue>::deserialize(d)?;
        finite_token(raw.get()).map_err(serde::de::Error::custom)?;
        Ok(Self(raw.get().to_owned()))
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct WireEnvelope {
    pub(super) schema: String,
    pub(super) key: RunKey,
    pub(super) mode: CollectionMode,
    pub(super) provenance: Provenance,
    pub(super) episode: WireEpisode,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct WireCondition {
    pub(super) id: String,
    pub(super) panel: Panel,
    pub(super) geometry: Geometry,
    pub(super) regime: Regime,
    pub(super) setup: WireSetup,
    pub(super) options: WireRunOptions,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct WireManifest {
    pub(super) schema: String,
    pub(super) version: u32,
    pub(super) status: String,
    pub(super) execution_authorized: bool,
    pub(super) protocol: String,
    pub(super) conditions: Vec<WireCondition>,
    pub(super) construction_seeds: Vec<u64>,
    pub(super) scientific_seeds: Vec<u64>,
    pub(super) raw_record_limit: u64,
    pub(super) raw_total_limit: u64,
    pub(super) metadata_limit: u64,
}
#[derive(Serialize)]
struct OriginalEnvelope<'a> {
    schema: &'static str,
    key: &'a RunKey,
    mode: CollectionMode,
    provenance: &'a Provenance,
    episode: &'a core::Episode,
}
struct BoundedWriter {
    bytes: Vec<u8>,
    limit: u64,
}
impl Write for BoundedWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let next = (self.bytes.len() as u64)
            .checked_add(buf.len() as u64)
            .filter(|n| *n <= self.limit)
            .ok_or_else(|| io::Error::other("raw record byte limit exceeded"))?;
        self.bytes
            .try_reserve(buf.len())
            .map_err(io::Error::other)?;
        self.bytes.extend_from_slice(buf);
        debug_assert_eq!(next, self.bytes.len() as u64);
        Ok(buf.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
pub(super) fn encode_envelope(
    key: &RunKey,
    mode: CollectionMode,
    p: &Provenance,
    e: &core::Episode,
    limit: u64,
) -> Result<Vec<u8>, String> {
    let mut writer = BoundedWriter {
        bytes: vec![],
        limit,
    };
    serde_json::to_writer(
        &mut writer,
        &OriginalEnvelope {
            schema: "foraging-shortcut-episode-v1",
            key,
            mode,
            provenance: p,
            episode: e,
        },
    )
    .map_err(|e| format!("{}/{} raw record: {e}", key.condition, key.seed))?;
    Ok(writer.bytes)
}
pub(super) fn decode_envelope(
    bytes: &[u8],
    key: &RunKey,
    mode: CollectionMode,
    p: &Provenance,
    limit: u64,
) -> Result<WireEnvelope, String> {
    let bind = || -> Result<WireEnvelope, String> {
        if bytes.len() as u64 > limit {
            return Err("raw record byte limit exceeded".into());
        }
        let e: WireEnvelope = serde_json::from_slice(bytes).map_err(|e| format!("wire: {e}"))?;
        if e.schema != "foraging-shortcut-episode-v1" {
            return Err("schema mismatch".into());
        }
        if e.key != *key {
            return Err("key mismatch".into());
        }
        if e.mode != mode {
            return Err("mode mismatch".into());
        }
        if e.provenance != *p {
            return Err("provenance mismatch".into());
        }
        if e.episode.seed != key.seed {
            return Err("episode.seed mismatch".into());
        }
        Ok(e)
    };
    bind().map_err(|e| format!("{}/{}: {e}", key.condition, key.seed))
}

// Core always emits nullable fields; absence is a schema error rather than null.
pub(super) fn required_option<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: Deserialize<'de>,
{
    Option::<T>::deserialize(deserializer)
}
