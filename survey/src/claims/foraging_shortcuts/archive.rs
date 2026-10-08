//! Immutable F5 archives; final publication requires every saved observation.
use super::{
    io::{checked_total, confined_path, directory, encode_metadata, read_bounded, regular_file},
    manifest::{authorize, candidate, condition, expected_keys, manifest_bytes, Manifest},
    sha256,
    validate::validate_episode,
    wire::{decode_envelope, WireEpisode, WireManifest},
    CollectionMode, Provenance, RunKey,
};
use crate::claims::protection_archive::{new_directory, validate_revision, write_new};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct RawRef {
    pub(super) key: RunKey,
    pub(super) path: String,
    pub(super) sha256: String,
    pub(super) bytes: u64,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Index {
    pub(super) schema: String,
    pub(super) manifest: WireManifest,
    pub(super) mode: CollectionMode,
    pub(super) provenance: Provenance,
    pub(super) approval_context: String,
    pub(super) expected_keys: Vec<RunKey>,
    pub(super) completed: bool,
    pub(super) runs: Vec<RawRef>,
    pub(super) raw_bytes: u64,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Limits {
    pub(super) record: u64,
    pub(super) total_raw: u64,
    pub(super) metadata: u64,
}
pub(super) fn limits(m: &Manifest) -> Limits {
    Limits {
        record: m.raw_record_limit,
        total_raw: m.raw_total_limit,
        metadata: m.metadata_limit,
    }
}
pub(super) fn raw_path(key: &RunKey) -> String {
    format!("raw/{}/{}.json", key.condition, key.seed)
}
fn wire_manifest(m: &Manifest) -> Result<WireManifest, String> {
    serde_json::from_slice(&serde_json::to_vec(m).map_err(|e| e.to_string())?)
        .map_err(|e| format!("manifest wire representation: {e}"))
}
fn validate_provenance(p: &Provenance) -> Result<(), String> {
    validate_revision(&p.code_revision).map_err(|e| format!("code_revision: {e}"))?;
    validate_revision(&p.protocol_revision).map_err(|e| format!("protocol_revision: {e}"))?;
    if p.manifest_sha256 != sha256(&manifest_bytes()?) {
        return Err("manifest_sha256 mismatch".into());
    }
    validate_digest(&p.collector_sha256).map_err(|e| format!("collector_sha256: {e}"))
}
fn validate_digest(digest: &str) -> Result<(), String> {
    if digest.len() != 64
        || !digest
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    {
        return Err("expected lowercase 64-hex SHA-256".into());
    }
    Ok(())
}
fn validate_index(index: &Index, m: &Manifest) -> Result<(), String> {
    if index.schema != "foraging-shortcut-archive-v1" || !index.completed {
        return Err("requires completed shortcut archive index".into());
    }
    if index.manifest != wire_manifest(m)? {
        return Err("complete manifest representation mismatch".into());
    }
    authorize(m, index.mode)?;
    validate_provenance(&index.provenance)?;
    if index.approval_context.trim().is_empty() {
        return Err("approval_context must be nonempty".into());
    }
    let keys = expected_keys(m, index.mode);
    if index.expected_keys != keys || index.runs.len() != keys.len() {
        return Err("expected key list or run count mismatch".into());
    }
    if index.raw_bytes > m.raw_total_limit {
        return Err("raw byte total exceeds limit".into());
    }
    let mut declared_total = 0;
    for (key, reference) in keys.iter().zip(&index.runs) {
        if reference.key != *key || reference.path != raw_path(key) {
            return Err(format!(
                "{}/{}: reference key order or canonical path mismatch",
                key.condition, key.seed
            ));
        }
        validate_digest(&reference.sha256).map_err(|e| format!("{}: {e}", reference.path))?;
        if reference.bytes > m.raw_record_limit {
            return Err(format!(
                "{}: raw record byte limit exceeded",
                reference.path
            ));
        }
        declared_total = checked_total(declared_total, reference.bytes, m.raw_total_limit)
            .map_err(|e| format!("{}: {e}", reference.path))?;
    }
    Ok(())
}

pub(super) struct ArchiveWriter {
    out: PathBuf,
    manifest: Manifest,
    mode: CollectionMode,
    provenance: Provenance,
    approval_context: String,
    limits: Limits,
    keys: Vec<RunKey>,
    next: usize,
    refs: Vec<RawRef>,
    raw_bytes: u64,
    failed: bool,
}
impl ArchiveWriter {
    pub(super) fn create(
        out: &Path,
        m: &Manifest,
        mode: CollectionMode,
        p: &Provenance,
        approval: &str,
        cap: Limits,
    ) -> Result<Self, String> {
        // Every identity/authorization/limit and initial metadata check precedes output creation.
        authorize(m, mode)?;
        if serde_json::to_vec(m).map_err(|e| e.to_string())?
            != serde_json::to_vec(&candidate()?).map_err(|e| e.to_string())?
        {
            return Err("archive manifest differs from frozen candidate".into());
        }
        if cap != limits(m) {
            return Err("archive limits differ from frozen candidate".into());
        }
        validate_provenance(p)?;
        if approval.trim().is_empty() {
            return Err("approval_context must be nonempty".into());
        }
        let mut writer = Self {
            out: out.to_owned(),
            manifest: m.clone(),
            mode,
            provenance: p.clone(),
            approval_context: approval.into(),
            limits: cap,
            keys: expected_keys(m, mode),
            next: 0,
            refs: vec![],
            raw_bytes: 0,
            failed: false,
        };
        let initial = encode_metadata(&writer.make_index(false)?, cap.metadata)?;
        new_directory(out)?;
        writer.out = directory(out)?;
        write_new(&writer.out.join("index.incomplete.json"), &initial)?;
        new_directory(&writer.out.join("raw"))?;
        new_directory(&writer.out.join("progress"))?;
        for c in &m.conditions {
            new_directory(&writer.out.join("raw").join(&c.id))?;
        }
        Ok(writer)
    }
    fn make_index(&self, completed: bool) -> Result<Index, String> {
        Ok(Index {
            schema: "foraging-shortcut-archive-v1".into(),
            manifest: wire_manifest(&self.manifest)?,
            mode: self.mode,
            provenance: self.provenance.clone(),
            approval_context: self.approval_context.clone(),
            expected_keys: self.keys.clone(),
            completed,
            runs: self.refs.clone(),
            raw_bytes: self.raw_bytes,
        })
    }
    pub(super) fn put(&mut self, key: &RunKey, bytes: &[u8]) -> Result<(), String> {
        let result = self
            .put_record(key, bytes)
            .map_err(|e| format!("{}/{} {}: {e}", key.condition, key.seed, raw_path(key)));
        if result.is_err() {
            self.failed = true;
        }
        result
    }
    fn put_record(&mut self, key: &RunKey, bytes: &[u8]) -> Result<(), String> {
        if self.failed {
            return Err("writer previously failed".into());
        }
        if self.keys.get(self.next) != Some(key) {
            return Err("unexpected, duplicate, or out-of-order key".into());
        }
        let envelope =
            decode_envelope(bytes, key, self.mode, &self.provenance, self.limits.record)?;
        validate_episode(
            condition(&self.manifest, &key.condition)?,
            key,
            &envelope.episode,
        )?;
        let next_total = checked_total(self.raw_bytes, bytes.len() as u64, self.limits.total_raw)?;
        let reference = RawRef {
            key: key.clone(),
            path: raw_path(key),
            sha256: sha256(bytes),
            bytes: bytes.len() as u64,
        };
        let receipt = encode_metadata(&reference, self.limits.metadata)?;
        let record_path = confined_path(&self.out, &reference.path, false)?;
        write_new(&record_path, bytes)?;
        let receipt_path =
            confined_path(&self.out, &format!("progress/{:03}.json", self.next), false)?;
        write_new(&receipt_path, &receipt)?;
        self.refs.push(reference);
        self.raw_bytes = next_total;
        self.next += 1;
        Ok(())
    }
    pub(super) fn fail(&mut self, message: &str) -> Result<(), String> {
        self.failed = true;
        #[derive(Serialize)]
        struct Failure<'a> {
            schema: &'static str,
            next_key: Option<&'a RunKey>,
            raw_bytes: u64,
            message: &'a str,
        }
        let bytes = encode_metadata(
            &Failure {
                schema: "foraging-shortcut-failure-v1",
                next_key: self.keys.get(self.next),
                raw_bytes: self.raw_bytes,
                message,
            },
            self.limits.metadata,
        )?;
        let path = confined_path(&self.out, "failure.json", false)?;
        write_new(&path, &bytes)
    }
    pub(super) fn finish(self) -> Result<Index, String> {
        if self.failed {
            return Err("cannot finish a failed writer".into());
        }
        if self.next != self.keys.len() {
            return Err("cannot finish incomplete key matrix".into());
        }
        let index = self.make_index(true)?;
        validate_index(&index, &self.manifest)?;
        let mut total = 0;
        for reference in &index.runs {
            validate_record(&self.out, &index, &self.manifest, reference)?;
            total = checked_total(total, reference.bytes, self.limits.total_raw)
                .map_err(|e| format!("{}: {e}", reference.path))?;
        }
        if total != index.raw_bytes {
            return Err("saved raw total mismatch".into());
        }
        let bytes = encode_metadata(&index, self.limits.metadata)?;
        // Keep this synced pending index as a metadata receipt. A no-overwrite same-directory
        // hard link publishes the complete bytes atomically; failed writes cannot expose index.json.
        let pending = confined_path(&self.out, "index.pending.json", false)?;
        write_new(&pending, &bytes)?;
        regular_file(&pending)?;
        let final_path = confined_path(&self.out, "index.json", false)?;
        fs::hard_link(&pending, &final_path)
            .map_err(|e| format!("publish {}: {e}", final_path.display()))?;
        Ok(index)
    }
}

pub(super) struct ArchiveReader {
    root: PathBuf,
    index: Index,
    manifest: Manifest,
    next: usize,
    bytes_seen: u64,
}
impl ArchiveReader {
    pub(super) fn open(path: &Path) -> Result<Self, String> {
        if path.file_name() != Some(std::ffi::OsStr::new("index.json")) {
            return Err("reader requires final index.json".into());
        }
        let parent = path
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or(Path::new("."));
        let root = directory(parent)?;
        let final_path = confined_path(&root, "index.json", true)?;
        let manifest = candidate()?;
        let bytes = read_bounded(&final_path, manifest.metadata_limit)?;
        let text =
            std::str::from_utf8(&bytes).map_err(|e| format!("{} UTF-8: {e}", path.display()))?;
        let index: Index =
            serde_json::from_str(text).map_err(|e| format!("{} JSON: {e}", path.display()))?;
        validate_index(&index, &manifest).map_err(|e| format!("{}: {e}", path.display()))?;
        // Inspect the entire exact path set before opening any raw record bytes.
        for reference in &index.runs {
            confined_path(&root, &reference.path, true)?;
        }
        Ok(Self {
            root,
            index,
            manifest,
            next: 0,
            bytes_seen: 0,
        })
    }
    pub(super) fn index(&self) -> &Index {
        &self.index
    }
    pub(super) fn next(&mut self) -> Result<Option<(RunKey, WireEpisode)>, String> {
        let Some(reference) = self.index.runs.get(self.next) else {
            if self.bytes_seen != self.index.raw_bytes {
                return Err("terminal raw byte total mismatch".into());
            }
            return Ok(None);
        };
        let episode = validate_record(&self.root, &self.index, &self.manifest, reference)?;
        self.bytes_seen = checked_total(
            self.bytes_seen,
            reference.bytes,
            self.manifest.raw_total_limit,
        )
        .map_err(|e| {
            format!(
                "{}/{} {}: {e}",
                reference.key.condition, reference.key.seed, reference.path
            )
        })?;
        self.next += 1;
        Ok(Some((reference.key.clone(), episode)))
    }
}
/// Shared saved-byte validation for pre-publication completion and streaming analysis.
pub(super) fn validate_record(
    root: &Path,
    index: &Index,
    m: &Manifest,
    reference: &RawRef,
) -> Result<WireEpisode, String> {
    let validate = || -> Result<WireEpisode, String> {
        if reference.path != raw_path(&reference.key)
            || !expected_keys(m, index.mode).contains(&reference.key)
        {
            return Err("noncanonical record path or unexpected key".into());
        }
        let path = confined_path(root, &reference.path, true)?;
        let bytes = read_bounded(&path, m.raw_record_limit)?;
        if bytes.len() as u64 != reference.bytes {
            return Err("raw byte count mismatch".into());
        }
        if sha256(&bytes) != reference.sha256 {
            return Err("raw SHA-256 mismatch".into());
        }
        let e = decode_envelope(
            &bytes,
            &reference.key,
            index.mode,
            &index.provenance,
            m.raw_record_limit,
        )?;
        validate_episode(
            condition(m, &reference.key.condition)?,
            &reference.key,
            &e.episode,
        )?;
        Ok(e.episode)
    };
    validate().map_err(|e| {
        format!(
            "{}/{} {}: {e}",
            reference.key.condition, reference.key.seed, reference.path
        )
    })
}

#[cfg(test)]
mod budget_tests {
    use super::super::tests::support::{encoded_fixture, owned_tempdir, test_provenance};
    use super::*;

    // Private state injection exercises boundary behavior without offering a production override.
    #[test]
    fn private_record_limit_accepts_exact_bytes_and_rejects_one_byte_short() {
        let m = candidate().unwrap();
        let key = &expected_keys(&m, CollectionMode::Construction)[0];
        let bytes = encoded_fixture(&key.condition, key.seed).unwrap();
        for short in [false, true] {
            let tmp = owned_tempdir();
            let out = tmp.path().join("archive");
            let mut w = ArchiveWriter::create(
                &out,
                &m,
                CollectionMode::Construction,
                &test_provenance(),
                "test",
                limits(&m),
            )
            .unwrap();
            w.limits.record = bytes.len() as u64 - u64::from(short);
            assert_eq!(w.put(key, &bytes).is_ok(), !short);
            if short {
                assert!(w.failed);
                assert!(!out.join(raw_path(key)).exists());
            }
        }
    }
    #[test]
    fn private_cumulative_limit_rejects_second_record_and_blocks_completion() {
        let m = candidate().unwrap();
        let keys = expected_keys(&m, CollectionMode::Construction);
        let a = encoded_fixture(&keys[0].condition, keys[0].seed).unwrap();
        let b = encoded_fixture(&keys[1].condition, keys[1].seed).unwrap();
        for short in [false, true] {
            let tmp = owned_tempdir();
            let out = tmp.path().join("archive");
            let mut w = ArchiveWriter::create(
                &out,
                &m,
                CollectionMode::Construction,
                &test_provenance(),
                "test",
                limits(&m),
            )
            .unwrap();
            w.limits.total_raw = (a.len() + b.len()) as u64 - u64::from(short);
            w.put(&keys[0], &a).unwrap();
            assert_eq!(w.put(&keys[1], &b).is_ok(), !short);
            if short {
                assert!(w.failed);
                assert!(!out.join(raw_path(&keys[1])).exists());
            }
            assert!(w.finish().is_err());
        }
    }
    #[test]
    fn private_metadata_limit_failure_precedes_record_write() {
        let m = candidate().unwrap();
        let key = &expected_keys(&m, CollectionMode::Construction)[0];
        let tmp = owned_tempdir();
        let out = tmp.path().join("archive");
        let mut w = ArchiveWriter::create(
            &out,
            &m,
            CollectionMode::Construction,
            &test_provenance(),
            "test",
            limits(&m),
        )
        .unwrap();
        w.limits.metadata = 1;
        assert!(w
            .put(key, &encoded_fixture(&key.condition, key.seed).unwrap())
            .is_err());
        assert!(!out.join(raw_path(key)).exists());
    }
    #[test]
    fn private_resigned_saved_body_is_validated_before_completion() {
        let m = candidate().unwrap();
        let keys = expected_keys(&m, CollectionMode::Construction);
        let tmp = owned_tempdir();
        let out = tmp.path().join("archive");
        let mut w = ArchiveWriter::create(
            &out,
            &m,
            CollectionMode::Construction,
            &test_provenance(),
            "test",
            limits(&m),
        )
        .unwrap();
        for key in &keys {
            w.put(key, &encoded_fixture(&key.condition, key.seed).unwrap())
                .unwrap();
        }
        let key = &keys[0];
        let mut e = super::super::tests::support::decode_fixture(&key.condition, key.seed).unwrap();
        e.episode.options.ticks = 511;
        let bytes = serde_json::to_vec(&e).unwrap();
        fs::write(out.join(raw_path(key)), &bytes).unwrap();
        w.raw_bytes = w.raw_bytes - w.refs[0].bytes + bytes.len() as u64;
        w.refs[0].bytes = bytes.len() as u64;
        w.refs[0].sha256 = sha256(&bytes);
        let result = w.finish();
        assert!(result.is_err(), "saved-body validation was bypassed");
        let error = result.err().unwrap();
        assert!(error.contains("episode.options"), "{error}");
        assert!(!out.join("index.json").exists());
    }
}
