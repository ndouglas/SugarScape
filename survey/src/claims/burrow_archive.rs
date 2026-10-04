//! Immutable raw evidence, exact provenance and saved-only physical validation.
use super::burrow::{self, expected_keys, Manifest, Panel, RunKey};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    io::Write,
    path::{Component, Path},
    process::Command,
};
use sugarscape_core::burrow::{run_episode, validate_episode, Episode, RunOptions};

const SCHEMA: &str = "burrow-archive-v1";
const MANIFEST_PATH: &str = "docs/superpowers/specs/2026-10-04-burrow-1-draft-manifest.json";
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct RawRef {
    pub(super) key: RunKey,
    pub(super) path: String,
    pub(super) sha256: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Index {
    pub(super) schema: String,
    pub(super) code_revision: String,
    pub(super) protocol_revision: String,
    pub(super) manifest_sha256: String,
    pub(super) manifest: Manifest,
    pub(super) panel: Panel,
    pub(super) approval_context: String,
    pub(super) expected_keys: Vec<RunKey>,
    pub(super) completed: bool,
    pub(super) runs: Vec<RawRef>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Envelope {
    pub(super) schema: String,
    pub(super) key: RunKey,
    pub(super) code_revision: String,
    pub(super) protocol_revision: String,
    pub(super) manifest_sha256: String,
    #[serde(deserialize_with = "super::burrow::strict_options")]
    pub(super) options: RunOptions,
    pub(super) episode: Episode,
}
// Saved-only analysis consumes this API in the next task.
#[allow(dead_code)]
#[derive(Debug)]
pub(super) struct Archive {
    pub(super) index: Index,
    pub(super) records: BTreeMap<RunKey, Episode>,
}
fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn revision(value: &str) -> Result<(), String> {
    if value.len() == 40 && value.bytes().all(|b| b.is_ascii_hexdigit()) {
        Ok(())
    } else {
        Err("revision must be a full 40-hex commit".into())
    }
}
fn git(root: &Path, args: &[&str]) -> Result<Vec<u8>, String> {
    let result = Command::new("git")
        .current_dir(root)
        .args(args)
        .output()
        .map_err(|e| e.to_string())?;
    if !result.status.success() {
        return Err(String::from_utf8_lossy(&result.stderr).into());
    }
    Ok(result.stdout)
}
fn git_text(root: &Path, args: &[&str]) -> Result<String, String> {
    String::from_utf8(git(root, args)?)
        .map(|s| s.trim().into())
        .map_err(|e| e.to_string())
}
fn authorized(m: &Manifest, panel: Panel) -> Result<(), String> {
    if panel == Panel::Scientific && !(m.status == "registered" && m.execution_authorized) {
        return Err("scientific execution requires a separately reviewed committed executable registration; current candidate is unregistered".into());
    }
    Ok(())
}
fn preflight(
    m: &Manifest,
    revision_value: &str,
    approval: &str,
    out: &Path,
) -> Result<String, String> {
    revision(revision_value)?;
    if approval.trim().is_empty() {
        return Err("approval context must record an approval reference".into());
    }
    if out.symlink_metadata().is_ok() {
        return Err("output directory already exists".into());
    }
    let root = Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/.."));
    let code = git_text(root, &["rev-parse", "HEAD"])?;
    revision(&code)?;
    if git_text(
        root,
        &[
            "rev-parse",
            "--verify",
            &format!("{revision_value}^{{commit}}"),
        ],
    )? != revision_value
    {
        return Err("protocol revision must name the exact full commit".into());
    }
    if !git(root, &["status", "--porcelain", "--untracked-files=no"])?.is_empty() {
        return Err("archive execution requires a clean tracked tree".into());
    }
    let protocol = fs::read(root.join(&m.protocol)).map_err(|e| e.to_string())?;
    if git(root, &["show", &format!("{revision_value}:{}", m.protocol)])? != protocol {
        return Err("committed protocol differs from checkout bytes".into());
    }
    let checkout = fs::read(root.join(MANIFEST_PATH)).map_err(|e| e.to_string())?;
    if checkout != burrow::manifest_bytes()
        || git(root, &["show", &format!("{code}:{MANIFEST_PATH}")])? != checkout
    {
        return Err("committed manifest differs from checkout or executable bytes".into());
    }
    Ok(code)
}
fn write_new(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|e| format!("{}: {e}", path.display()))?;
    file.write_all(bytes)
        .and_then(|_| file.sync_all())
        .map_err(|e| format!("{}: {e}", path.display()))
}
fn json_new<T: Serialize>(path: &Path, value: &T) -> Result<(), String> {
    write_new(path, &serde_json::to_vec(value).map_err(|e| e.to_string())?)
}
fn initial_index(
    m: Manifest,
    panel: Panel,
    code: String,
    protocol: String,
    approval: String,
    hash: String,
) -> Index {
    let keys = expected_keys(&m, panel);
    Index {
        schema: SCHEMA.into(),
        code_revision: code,
        protocol_revision: protocol,
        manifest_sha256: hash,
        manifest: m,
        panel,
        approval_context: approval,
        expected_keys: keys,
        completed: false,
        runs: vec![],
    }
}
pub(super) fn run(
    panel: Panel,
    protocol_revision: &str,
    approval_context: &str,
    out: &Path,
) -> Result<(), String> {
    let m = burrow::manifest()?;
    authorized(&m, panel)?;
    let code = preflight(&m, protocol_revision, approval_context, out)?;
    let index = initial_index(
        m,
        panel,
        code,
        protocol_revision.into(),
        approval_context.into(),
        burrow::manifest_sha256(),
    );
    write_archive(index, out, |c, seed| {
        run_episode(c.config.clone(), seed, c.options.clone())
            .map_err(|e| format!("episode execution: {e:?}"))
    })
}
fn contextual(key: &RunKey, error: impl std::fmt::Display) -> String {
    format!("condition {} seed {}: {error}", key.condition, key.seed)
}
fn conditions(index: &Index) -> &[super::burrow::Condition] {
    match index.panel {
        Panel::Scientific => &index.manifest.conditions,
        Panel::Construction => &index.manifest.construction_conditions,
    }
}
fn write_archive(
    mut index: Index,
    out: &Path,
    mut execute: impl FnMut(&super::burrow::Condition, u64) -> Result<Episode, String>,
) -> Result<(), String> {
    fs::create_dir(out).map_err(|e| format!("new output directory {}: {e}", out.display()))?;
    // Save the full key set first, so even failure creating raw directories is identifiable.
    let mut failing_key: Option<RunKey> = None;
    let result = (|| {
        json_new(&out.join("index.incomplete.json"), &index)?;
        fs::create_dir(out.join("raw")).map_err(|e| e.to_string())?;
        fs::create_dir(out.join("progress")).map_err(|e| e.to_string())?;
        for (ordinal, key) in index.expected_keys.clone().into_iter().enumerate() {
            failing_key = Some(key.clone());
            let (condition_ordinal, c) = conditions(&index)
                .iter()
                .enumerate()
                .find(|(_, c)| c.id == key.condition)
                .ok_or("unknown condition")?;
            let episode = execute(c, key.seed.parse().map_err(|e| contextual(&key, e))?)
                .map_err(|e| contextual(&key, e))?;
            if episode.config != c.config || episode.seed != key.seed {
                return Err(contextual(&key, "episode config/seed mismatch"));
            }
            validate_episode(&episode, &c.options).map_err(|e| contextual(&key, e))?;
            let envelope = Envelope {
                schema: SCHEMA.into(),
                key: key.clone(),
                code_revision: index.code_revision.clone(),
                protocol_revision: index.protocol_revision.clone(),
                manifest_sha256: index.manifest_sha256.clone(),
                options: c.options.clone(),
                episode,
            };
            let bytes = serde_json::to_vec(&envelope).map_err(|e| contextual(&key, e))?;
            let path = format!("raw/{condition_ordinal:03}-{}.json", key.seed);
            write_new(&out.join(&path), &bytes).map_err(|e| contextual(&key, e))?;
            let raw = RawRef {
                key: key.clone(),
                path,
                sha256: digest(&bytes),
            };
            json_new(&out.join(format!("progress/{ordinal:04}.json")), &raw)
                .map_err(|e| contextual(&key, e))?;
            index.runs.push(raw);
        }
        index.completed = true;
        load_against(out, index.clone(), &index.manifest, &index.manifest_sha256)?;
        json_new(&out.join("index.json"), &index)
    })();
    result.map_err(|error| {
        // Final saved-byte validation may identify an earlier record than the last write.
        let key = index
            .expected_keys
            .iter()
            .find(|key| {
                error.starts_with(&format!("condition {} seed {}:", key.condition, key.seed))
            })
            .cloned()
            .or(failing_key);
        let failure = serde_json::json!({"schema":SCHEMA,"key":key,"error":error});
        let receipt = json_new(&out.join("failure.json"), &failure);
        match receipt {
            Ok(()) => format!("{error}; incomplete archive retained at {}", out.display()),
            Err(e) => format!(
                "{error}; incomplete archive retained at {}; failure receipt: {e}",
                out.display()
            ),
        }
    })
}
fn safe_path(path: &str) -> Result<(), String> {
    if path.is_empty()
        || Path::new(path)
            .components()
            .any(|c| !matches!(c, Component::Normal(_)))
    {
        Err("raw path must be relative with no parent components".into())
    } else {
        Ok(())
    }
}
#[allow(dead_code)] // Public saved-only consumer is installed in the next task.
pub(super) fn load(index_path: &Path) -> Result<Archive, String> {
    let index: Index = serde_json::from_slice(
        &fs::read(index_path).map_err(|e| format!("{}: {e}", index_path.display()))?,
    )
    .map_err(|e| format!("archive index: {e}"))?;
    burrow::validate_manifest(&index.manifest)?;
    authorized(&index.manifest, index.panel)?;
    load_against(
        index_path
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or(Path::new(".")),
        index,
        &burrow::manifest()?,
        &burrow::manifest_sha256(),
    )
}
fn load_against(
    root: &Path,
    index: Index,
    expected: &Manifest,
    hash: &str,
) -> Result<Archive, String> {
    revision(&index.code_revision)?;
    revision(&index.protocol_revision)?;
    if index.schema != SCHEMA
        || !index.completed
        || index.manifest != *expected
        || index.manifest_sha256 != hash
        || index.approval_context.trim().is_empty()
    {
        return Err("archive schema/completion/manifest/provenance mismatch".into());
    }
    let keys = expected_keys(expected, index.panel);
    if index.expected_keys != keys {
        return Err("archive expected key set/order differs from manifest".into());
    }
    let mut remaining: BTreeSet<_> = keys.iter().cloned().collect();
    let mut paths = BTreeSet::new();
    for raw in &index.runs {
        if !remaining.remove(&raw.key) {
            return Err(contextual(&raw.key, "duplicate or extra raw key"));
        }
        if !paths.insert(&raw.path) {
            return Err(contextual(&raw.key, "reused raw path"));
        }
    }
    if let Some(key) = remaining.first() {
        return Err(contextual(key, "missing raw key"));
    }
    if index.runs.iter().map(|r| &r.key).ne(keys.iter()) {
        return Err("archive raw keys are not in canonical order".into());
    }
    let root = root.canonicalize().map_err(|e| e.to_string())?;
    let mut records = BTreeMap::new();
    for raw in &index.runs {
        let record = (|| {
            safe_path(&raw.path)?;
            let path = root
                .join(&raw.path)
                .canonicalize()
                .map_err(|e| format!("raw {}: {e}", raw.path))?;
            if !path.starts_with(&root) {
                return Err("raw symlink escapes archive root".into());
            }
            let bytes = fs::read(&path).map_err(|e| e.to_string())?;
            if digest(&bytes) != raw.sha256 {
                return Err("raw SHA-256 mismatch".into());
            }
            let e: Envelope =
                serde_json::from_slice(&bytes).map_err(|e| format!("raw envelope: {e}"))?;
            let c = conditions(&index)
                .iter()
                .find(|c| c.id == raw.key.condition)
                .ok_or("unknown condition")?;
            if e.schema != SCHEMA
                || e.key != raw.key
                || e.code_revision != index.code_revision
                || e.protocol_revision != index.protocol_revision
                || e.manifest_sha256 != index.manifest_sha256
                || e.options != c.options
                || e.episode.config != c.config
                || e.episode.seed != raw.key.seed
            {
                return Err("raw envelope identity/provenance/options/config/seed mismatch".into());
            }
            validate_episode(&e.episode, &c.options)?;
            Ok(e.episode)
        })()
        .map_err(|e: String| contextual(&raw.key, e))?;
        records.insert(raw.key.clone(), record);
    }
    Ok(Archive { index, records })
}
#[cfg(test)]
#[path = "burrow_archive_tests.rs"]
mod tests;
