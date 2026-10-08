//! Exclusive attempt receipts, durable frame prefixes, and saved-data validation.
use super::deception::{manifest, Condition, Manifest, SCHEMA};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    io::Write,
    path::{Component, Path, PathBuf},
    process::Command,
    time::{Instant, SystemTime, UNIX_EPOCH},
};
use sugarscape_core::{
    geometry::Pos,
    minds::{
        deception::{observation::Signal, records::*, state::*},
        protection::ledger::{Ledger, Outflow},
    },
};

pub const TIMING: &str = "construction + stepping + diagnostics + durable frame I/O; excludes final envelope I/O and analysis";
const PROTOCOL: &str = "docs/superpowers/specs/2026-10-07-minds-deception-protocol.md";
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FileIdentity {
    pub path: String,
    pub sha256: String,
    pub bytes: u64,
    pub mode: String,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Provenance {
    pub source_revision: String,
    pub protocol_revision: String,
    pub binary: FileIdentity,
    pub protocol: FileIdentity,
    pub protocol_bytes: String,
    pub source_inventory: Vec<FileIdentity>,
    pub source_sha256: String,
    pub manifest_sha256: String,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AttemptRef {
    pub condition: String,
    pub seed: u64,
    pub start: String,
    pub frames: String,
    pub outcome: String,
}
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Census {
    pub planned: usize,
    pub unstarted: usize,
    pub attempted: usize,
    pub complete: usize,
    pub failed: usize,
    /// Known invalid receipts or biologically invalid full-horizon records.
    pub invalid: usize,
    pub partial: usize,
    pub pending: usize,
    pub unavailable_lineage: usize,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Index {
    pub schema: String,
    pub provenance: Provenance,
    pub manifest: Manifest,
    pub completed: bool,
    pub attempts: Vec<AttemptRef>,
    pub census: Option<Census>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AttemptIdentity {
    pub source_revision: String,
    pub protocol_revision: String,
    pub binary_sha256: String,
    pub protocol_sha256: String,
    pub source_sha256: String,
    pub manifest_sha256: String,
}
impl Provenance {
    fn identity(&self) -> AttemptIdentity {
        AttemptIdentity {
            source_revision: self.source_revision.clone(),
            protocol_revision: self.protocol_revision.clone(),
            binary_sha256: self.binary.sha256.clone(),
            protocol_sha256: self.protocol.sha256.clone(),
            source_sha256: self.source_sha256.clone(),
            manifest_sha256: self.manifest_sha256.clone(),
        }
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AttemptStart {
    pub schema: String,
    pub condition: String,
    pub seed: u64,
    pub provenance: AttemptIdentity,
    pub started_unix_ms: u128,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AttemptOutcome {
    Complete(EpisodeRecord),
    Failed(EpisodeFailure),
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OutcomeReceipt {
    pub condition: String,
    pub seed: u64,
    pub elapsed_seconds: f64,
    pub timing_boundary: String,
    pub outcome: AttemptOutcome,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Attempt {
    pub start: AttemptStart,
    pub frames: Vec<FrameRecord>,
    pub outcome: Option<OutcomeReceipt>,
    pub interrupted_tail: Option<String>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Archive {
    pub index: Index,
    pub attempts: Vec<Attempt>,
}

pub fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn json<T: Serialize>(value: &T) -> Result<Vec<u8>, String> {
    serde_json::to_vec(value).map_err(|e| format!("serialize JSON: {e}"))
}
pub fn validate_revision(value: &str) -> Result<(), String> {
    if value.len() == 40 && value.bytes().all(|c| c.is_ascii_hexdigit()) {
        Ok(())
    } else {
        Err("revision must be full 40 hexadecimal characters".into())
    }
}
pub fn new_directory(path: &Path) -> Result<(), String> {
    reject_symlink_ancestors(path)?;
    fs::create_dir(path).map_err(|e| format!("create directory {}: {e}", path.display()))?;
    sync_parent(path)
}
fn sync_parent(path: &Path) -> Result<(), String> {
    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    fs::File::open(parent)
        .and_then(|f| f.sync_all())
        .map_err(|e| format!("sync directory {}: {e}", parent.display()))
}
pub fn write_new(path: &Path, bytes: &[u8]) -> Result<(), String> {
    reject_symlink_ancestors(path)?;
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|e| format!("create {}: {e}", path.display()))?;
    file.write_all(bytes)
        .and_then(|_| file.sync_all())
        .map_err(|e| format!("write/sync {}: {e}", path.display()))?;
    sync_parent(path)
}
fn reject_symlink_ancestors(path: &Path) -> Result<(), String> {
    for ancestor in path.ancestors().filter(|p| !p.as_os_str().is_empty()) {
        match fs::symlink_metadata(ancestor) {
            Ok(meta) if meta.file_type().is_symlink() => {
                return Err(format!("symlink forbidden: {}", ancestor.display()))
            }
            Ok(_) => {}
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => return Err(format!("inspect {}: {e}", ancestor.display())),
        }
    }
    Ok(())
}
fn confined(root: &Path, relative: &str) -> Result<PathBuf, String> {
    let relative = Path::new(relative);
    if relative.as_os_str().is_empty()
        || relative
            .components()
            .any(|c| !matches!(c, Component::Normal(_)))
    {
        return Err("reference must be a nonempty confined relative path".into());
    }
    let path = root.join(relative);
    reject_symlink_ancestors(&path)?;
    Ok(path)
}
fn read<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<T, String> {
    let bytes = fs::read(path).map_err(|e| format!("read {}: {e}", path.display()))?;
    serde_json::from_slice(&bytes).map_err(|e| format!("decode {}: {e}", path.display()))
}
fn exists(path: &Path) -> Result<bool, String> {
    match fs::symlink_metadata(path) {
        Ok(_) => Ok(true),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(e) => Err(format!("inspect {}: {e}", path.display())),
    }
}
fn refs(m: &Manifest) -> Vec<AttemptRef> {
    m.conditions
        .iter()
        .enumerate()
        .flat_map(|(ordinal, c)| {
            m.seeds.iter().map(move |&seed| {
                let dir = format!("attempts/{ordinal:03}-{seed}");
                AttemptRef {
                    condition: c.id.clone(),
                    seed,
                    start: format!("{dir}/start.json"),
                    frames: format!("{dir}/frames.jsonl"),
                    outcome: format!("{dir}/outcome.json"),
                }
            })
        })
        .collect()
}
fn git(root: &Path, args: &[&str]) -> Result<Vec<u8>, String> {
    let result = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(args)
        .output()
        .map_err(|e| format!("git {args:?}: {e}"))?;
    if !result.status.success() {
        return Err(format!(
            "git {args:?} exited {}: {}",
            result.status,
            String::from_utf8_lossy(&result.stderr)
        ));
    }
    Ok(result.stdout)
}
fn git_text(root: &Path, args: &[&str]) -> Result<String, String> {
    String::from_utf8(git(root, args)?)
        .map(|s| s.trim_end_matches(['\r', '\n']).to_string())
        .map_err(|e| format!("git text: {e}"))
}
fn file_identity(path: &Path, name: String, mode: String) -> Result<FileIdentity, String> {
    let bytes = fs::read(path).map_err(|e| format!("inventory {}: {e}", path.display()))?;
    Ok(FileIdentity {
        path: name,
        sha256: hash(&bytes),
        bytes: bytes.len() as u64,
        mode,
    })
}
fn preflight(root: &Path, revision: &str, out: &Path) -> Result<Provenance, String> {
    validate_revision(revision)?;
    let resolved = git_text(
        root,
        &["rev-parse", "--verify", &format!("{revision}^{{commit}}")],
    )?;
    if !resolved.eq_ignore_ascii_case(revision) {
        return Err("protocol revision does not resolve to the exact commit".into());
    }
    let committed = git(root, &["show", &format!("{revision}:{PROTOCOL}")])?;
    let current = fs::read(root.join(PROTOCOL)).map_err(|e| format!("current protocol: {e}"))?;
    if committed != current {
        return Err("raw committed/current protocol bytes differ".into());
    }
    if !git(root, &["status", "--porcelain", "--untracked-files=no"])?.is_empty() {
        return Err("tracked source tree must be clean".into());
    }
    reject_symlink_ancestors(out)?;
    if exists(out)? {
        return Err("output destination already exists".into());
    }
    let mut source_inventory = vec![];
    for entry in git(root, &["ls-files", "--stage", "-z"])?
        .split(|b| *b == 0)
        .filter(|e| !e.is_empty())
    {
        let entry = std::str::from_utf8(entry).map_err(|e| e.to_string())?;
        let (meta, name) = entry.split_once('\t').ok_or("malformed git inventory")?;
        let mode = meta
            .split_whitespace()
            .next()
            .ok_or("missing git file mode")?;
        let path = confined(root, name)?;
        source_inventory.push(file_identity(&path, name.into(), mode.into())?);
    }
    source_inventory.sort_by(|a, b| a.path.cmp(&b.path));
    let protocol = source_inventory
        .iter()
        .find(|i| i.path == PROTOCOL)
        .ok_or("protocol is not tracked")?
        .clone();
    let exe = std::env::current_exe()
        .map_err(|e| e.to_string())?
        .canonicalize()
        .map_err(|e| e.to_string())?;
    use std::os::unix::fs::PermissionsExt;
    let mode = format!(
        "{:06o}",
        0o100000
            | (fs::metadata(&exe)
                .map_err(|e| e.to_string())?
                .permissions()
                .mode()
                & 0o777)
    );
    Ok(Provenance {
        source_revision: git_text(root, &["rev-parse", "HEAD"])?,
        protocol_revision: resolved,
        binary: file_identity(&exe, exe.to_string_lossy().into_owned(), mode)?,
        protocol,
        protocol_bytes: String::from_utf8(current).map_err(|e| e.to_string())?,
        source_sha256: hash(&json(&source_inventory)?),
        source_inventory,
        manifest_sha256: hash(&json(&manifest())?),
    })
}
fn validate_provenance(p: &Provenance) -> Result<(), String> {
    validate_revision(&p.source_revision)?;
    validate_revision(&p.protocol_revision)?;
    let valid_hash = |h: &str| h.len() == 64 && h.bytes().all(|b| b.is_ascii_hexdigit());
    if p.binary.path.is_empty()
        || p.binary.bytes == 0
        || !valid_hash(&p.binary.sha256)
        || p.protocol.path != PROTOCOL
        || p.protocol.bytes != p.protocol_bytes.len() as u64
        || p.protocol.sha256 != hash(p.protocol_bytes.as_bytes())
        || p.manifest_sha256 != hash(&json(&manifest())?)
        || p.source_sha256 != hash(&json(&p.source_inventory)?)
        || !p.source_inventory.contains(&p.protocol)
    {
        return Err("provenance identity/hash mismatch".into());
    }
    let mut names = BTreeSet::new();
    for f in &p.source_inventory {
        if !names.insert(&f.path)
            || !valid_hash(&f.sha256)
            || !["100644", "100755"].contains(&f.mode.as_str())
            || Path::new(&f.path)
                .components()
                .any(|c| !matches!(c, Component::Normal(_)))
            || f.path.is_empty()
        {
            return Err("invalid source inventory".into());
        }
    }
    Ok(())
}
fn validate_index(index: &Index) -> Result<(), String> {
    if index.schema != SCHEMA
        || index.manifest != manifest()
        || index.attempts != refs(&index.manifest)
    {
        return Err("noncanonical schema/manifest/budget/attempt references".into());
    }
    validate_provenance(&index.provenance)
}
pub fn census(a: &Archive) -> Census {
    let mut counts = Census {
        planned: a.index.attempts.len(),
        attempted: a.attempts.len(),
        unstarted: a.index.attempts.len().saturating_sub(a.attempts.len()),
        ..Default::default()
    };
    for attempt in &a.attempts {
        count_outcome(&mut counts, attempt.outcome.as_ref());
        let structural = validate_attempt(&a.index, attempt).is_err();
        let biological = attempt.outcome.as_ref().is_some_and(|o| match &o.outcome {
            AttemptOutcome::Complete(r) => validate_episode(r).is_err(),
            _ => false,
        });
        counts.invalid += usize::from(structural || biological);
    }
    counts
}
pub fn load(path: &Path) -> Result<Archive, String> {
    load_inner(path).map_err(|error| match receipt_census(path) {
        Ok(counts) => format!("{error}; execution census: {counts:?}"),
        Err(census_error) => format!("{error}; execution census unavailable: {census_error}"),
    })
}
fn receipt_census(path: &Path) -> Result<Census, String> {
    reject_symlink_ancestors(path)?;
    let index: Index = read(path)?;
    let root = path.parent().unwrap_or(Path::new("."));
    let mut counts = Census {
        planned: index.attempts.len(),
        ..Default::default()
    };
    for reference in &index.attempts {
        let start_path = confined(root, &reference.start)?;
        let frame_path = confined(root, &reference.frames)?;
        let outcome_path = confined(root, &reference.outcome)?;
        if !exists(&start_path)? && !exists(&frame_path)? && !exists(&outcome_path)? {
            counts.unstarted += 1;
            continue;
        }
        counts.attempted += 1;
        let start = read::<AttemptStart>(&start_path);
        let outcome = if exists(&outcome_path)? {
            read::<OutcomeReceipt>(&outcome_path).map(Some)
        } else {
            Ok(None)
        };
        count_outcome(&mut counts, outcome.as_ref().ok().and_then(|o| o.as_ref()));
        let frames = read_frames(&frame_path);
        let invalid = match (start, outcome, frames) {
            (Ok(start), Ok(outcome), Ok((frames, interrupted_tail))) => {
                let attempt = Attempt {
                    start,
                    frames,
                    outcome,
                    interrupted_tail,
                };
                attempt.start.condition != reference.condition
                    || attempt.start.seed != reference.seed
                    || validate_attempt(&index, &attempt).is_err()
                    || attempt.outcome.as_ref().is_some_and(|o| match &o.outcome {
                        AttemptOutcome::Complete(r) => validate_episode(r).is_err(),
                        _ => false,
                    })
            }
            _ => true,
        };
        counts.invalid += usize::from(invalid);
    }
    Ok(counts)
}
fn load_inner(path: &Path) -> Result<Archive, String> {
    reject_symlink_ancestors(path)?;
    let index: Index = read(path)?;
    validate_index(&index)?;
    let root = path.parent().unwrap_or(Path::new("."));
    let mut attempts = vec![];
    for reference in &index.attempts {
        let start_path = confined(root, &reference.start)?;
        let frame_path = confined(root, &reference.frames)?;
        let outcome_path = confined(root, &reference.outcome)?;
        if !exists(&start_path)? {
            if exists(&frame_path)? || exists(&outcome_path)? {
                return Err("orphan frames/outcome without attempt start".into());
            }
            continue;
        }
        let start: AttemptStart = read(&start_path)?;
        if start.condition != reference.condition || start.seed != reference.seed {
            return Err("canonical slot identity mismatch".into());
        }
        let outcome = if exists(&outcome_path)? {
            Some(read(&outcome_path)?)
        } else {
            None
        };
        let (frames, interrupted_tail) = read_frames(&frame_path)?;
        attempts.push(Attempt {
            start,
            frames,
            outcome,
            interrupted_tail,
        });
    }
    let archive = Archive { index, attempts };
    validate_structure(&archive)?;
    Ok(archive)
}
fn validate_structure(a: &Archive) -> Result<(), String> {
    validate_index(&a.index)?;
    let mut keys = BTreeSet::new();
    for attempt in &a.attempts {
        if !keys.insert((&attempt.start.condition, attempt.start.seed)) {
            return Err("duplicate attempt identity".into());
        }
        validate_attempt(&a.index, attempt)?;
    }
    if a.index.completed
        && (a.attempts.len() != a.index.attempts.len()
            || a.attempts.iter().any(|a| a.outcome.is_none()))
    {
        return Err("completed index lacks finalized attempts".into());
    }
    if a.index
        .census
        .as_ref()
        .is_some_and(|saved| *saved != census(a))
    {
        return Err("saved census mismatch".into());
    }
    Ok(())
}
pub fn validate_archive(a: &Archive) -> Result<(), String> {
    let check = || -> Result<(), String> {
        validate_structure(a)?;
        if !a.index.completed || a.attempts.len() != 3840 {
            return Err("analysis requires the complete canonical matrix".into());
        }
        for attempt in &a.attempts {
            let Some(OutcomeReceipt {
                outcome: AttemptOutcome::Complete(record),
                ..
            }) = &attempt.outcome
            else {
                return Err("analysis requires every attempt complete".into());
            };
            validate_episode(record)?;
        }
        Ok(())
    };
    check().map_err(|e| format!("{e}; execution census: {:?}", census(a)))
}
// Private injection seam. The public run route always supplies the fixed manifest.
#[allow(clippy::type_complexity)]
fn execute_attempt<F>(
    root: &Path,
    r: &AttemptRef,
    p: &Provenance,
    c: &Condition,
    runner: F,
) -> Result<(), String>
where
    F: FnMut(
        LabConfig,
        u64,
        &mut dyn FnMut(&FrameRecord) -> Result<(), String>,
    ) -> Result<EpisodeRecord, EpisodeFailure>,
{
    execute_attempt_with_writer(root, r, p, c, runner, |file, bytes| {
        file.write_all(bytes)
            .and_then(|_| file.sync_all())
            .map_err(|e| format!("durable frame write: {e}"))
    })
}
fn execute_attempt_with_writer<F, W>(
    root: &Path,
    r: &AttemptRef,
    p: &Provenance,
    condition: &Condition,
    mut runner: F,
    mut writer: W,
) -> Result<(), String>
where
    F: FnMut(
        LabConfig,
        u64,
        &mut dyn FnMut(&FrameRecord) -> Result<(), String>,
    ) -> Result<EpisodeRecord, EpisodeFailure>,
    W: FnMut(&mut fs::File, &[u8]) -> Result<(), String>,
{
    let start_path = confined(root, &r.start)?;
    new_directory(start_path.parent().ok_or("attempt directory missing")?)?;
    let start = AttemptStart {
        schema: SCHEMA.into(),
        condition: r.condition.clone(),
        seed: r.seed,
        provenance: p.identity(),
        started_unix_ms: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| e.to_string())?
            .as_millis(),
    };
    write_new(&start_path, &json(&start)?)?;
    let frame_path = confined(root, &r.frames)?;
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&frame_path)
        .map_err(|e| e.to_string())?;
    file.sync_all().map_err(|e| e.to_string())?;
    sync_parent(&frame_path)?;
    let began = Instant::now();
    let mut io_error: Option<String> = None;
    let result = runner(condition.lab.clone(), r.seed, &mut |frame| {
        if let Some(error) = &io_error {
            return Err(error.clone());
        }
        let result = json(frame).and_then(|mut bytes| {
            bytes.push(b'\n');
            writer(&mut file, &bytes)
        });
        if let Err(error) = &result {
            io_error = Some(error.clone());
        }
        result
    });
    let elapsed_seconds = began.elapsed().as_secs_f64();
    let outcome = match result {
        Ok(record) if io_error.is_none() => AttemptOutcome::Complete(record),
        Ok(record) => AttemptOutcome::Failed(EpisodeFailure {
            message: io_error.clone().unwrap(),
            partial: Some(record),
        }),
        Err(failure) => AttemptOutcome::Failed(failure),
    };
    let receipt = OutcomeReceipt {
        condition: r.condition.clone(),
        seed: r.seed,
        elapsed_seconds,
        timing_boundary: TIMING.into(),
        outcome,
    };
    write_new(&confined(root, &r.outcome)?, &json(&receipt)?)?;
    if let Some(error) = io_error {
        return Err(error);
    }
    Ok(())
}
// The approved core failure DTO deliberately owns its known partial record.
#[allow(clippy::result_large_err)]
pub fn run(revision: &str, out: &Path) -> Result<(), String> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("missing source root")?;
    let provenance = preflight(root, revision, out)?;
    new_directory(out)?;
    let m = manifest();
    let index = Index {
        schema: SCHEMA.into(),
        provenance,
        attempts: refs(&m),
        manifest: m,
        completed: false,
        census: None,
    };
    execute_campaign(out, index, |lab, seed, sink| {
        sugarscape_core::minds::deception::run_episode_with_sink(lab, seed, true, sink)
    })?;
    let archive = load(&out.join("final-index.json"))?;
    let analysis = super::deception_report::analyze(&archive)?;
    super::deception_report::save_files(&analysis, out)
}

#[allow(clippy::result_large_err)]
fn execute_campaign<F>(root: &Path, mut index: Index, mut runner: F) -> Result<Index, String>
where
    F: FnMut(
        LabConfig,
        u64,
        &mut dyn FnMut(&FrameRecord) -> Result<(), String>,
    ) -> Result<EpisodeRecord, EpisodeFailure>,
{
    write_new(&root.join("index.json"), &json(&index)?)?;
    new_directory(&root.join("attempts"))?;
    for r in &index.attempts {
        let condition = index
            .manifest
            .conditions
            .iter()
            .find(|c| c.id == r.condition)
            .ok_or("unknown campaign condition")?;
        execute_attempt(root, r, &index.provenance, condition, &mut runner)?;
    }
    let mut attempts = vec![];
    for r in &index.attempts {
        let (frames, interrupted_tail) = read_frames(&confined(root, &r.frames)?)?;
        attempts.push(Attempt {
            start: read(&confined(root, &r.start)?)?,
            frames,
            interrupted_tail,
            outcome: Some(read(&confined(root, &r.outcome)?)?),
        });
    }
    index.completed = true;
    index.census = Some(census(&Archive {
        index: index.clone(),
        attempts,
    }));
    write_new(&root.join("final-index.json"), &json(&index)?)?;
    write_new(&root.join("census.json"), &json(&index.census)?)?;
    Ok(index)
}

fn count_outcome(counts: &mut Census, outcome: Option<&OutcomeReceipt>) {
    match outcome.map(|o| &o.outcome) {
        None => counts.pending += 1,
        Some(AttemptOutcome::Complete(r)) => {
            counts.complete += 1;
            counts.unavailable_lineage += usize::from(r.lineage_unavailable_reason.is_some());
        }
        Some(AttemptOutcome::Failed(f)) => {
            counts.failed += 1;
            counts.partial += usize::from(f.partial.is_some());
        }
    }
}
fn read_frames(path: &Path) -> Result<(Vec<FrameRecord>, Option<String>), String> {
    let bytes = fs::read(path).map_err(|e| format!("frame stream {}: {e}", path.display()))?;
    let split = bytes.iter().rposition(|b| *b == b'\n').map_or(0, |i| i + 1);
    let mut frames = vec![];
    for line in bytes[..split].split_inclusive(|b| *b == b'\n') {
        frames.push(
            serde_json::from_slice(line).map_err(|e| format!("malformed durable frame: {e}"))?,
        );
    }
    let tail = (split < bytes.len()).then(|| String::from_utf8_lossy(&bytes[split..]).into_owned());
    Ok((frames, tail))
}
fn validate_attempt(index: &Index, attempt: &Attempt) -> Result<(), String> {
    let start = &attempt.start;
    if start.schema != SCHEMA
        || start.provenance != index.provenance.identity()
        || !index
            .attempts
            .iter()
            .any(|r| r.condition == start.condition && r.seed == start.seed)
    {
        return Err("attempt identity/provenance mismatch".into());
    }
    if attempt
        .frames
        .iter()
        .enumerate()
        .any(|(i, f)| f.tick != i as u64)
    {
        return Err("invalid frame prefix ticks".into());
    }
    let Some(outcome) = &attempt.outcome else {
        return Ok(());
    };
    if outcome.condition != start.condition
        || outcome.seed != start.seed
        || !outcome.elapsed_seconds.is_finite()
        || outcome.elapsed_seconds < 0.0
        || outcome.timing_boundary != TIMING
    {
        return Err("outcome identity/timing mismatch".into());
    }
    let (record, complete) = match &outcome.outcome {
        AttemptOutcome::Complete(r) => (Some(r), true),
        AttemptOutcome::Failed(f) => (f.partial.as_ref(), false),
    };
    if let Some(record) = record {
        let condition = index
            .manifest
            .conditions
            .iter()
            .find(|c| c.id == start.condition)
            .ok_or("unknown condition")?;
        if record.schema != SCHEMA
            || record.seed != start.seed
            || record.lab != condition.lab
            || record.requested_ticks != 64
            || record.completed_ticks > 64
        {
            return Err("episode identity/budget mismatch".into());
        }
        if !record.frames.starts_with(&attempt.frames)
            || complete && (record.frames != attempt.frames || attempt.interrupted_tail.is_some())
        {
            return Err("durable frame stream conflicts with outcome".into());
        }
    }
    Ok(())
}

fn near(a: f64, b: f64) -> bool {
    a.is_finite() && b.is_finite() && (a - b).abs() <= 1e-9 * a.abs().max(b.abs()).max(1.0)
}
fn pos(l: &LabConfig, x: u32, y: u32) -> Pos {
    Pos::new(if l.mirrored { 8 - x } else { x }, y)
}
fn site(p: Pos) -> u32 {
    p.y * 9 + p.x
}
fn legal(p: Pos) -> bool {
    (1..=7).contains(&p.x) && (1..=7).contains(&p.y)
}
fn role(f: &FrameRecord, id: u64) -> Option<&RoleRecord> {
    f.roles.iter().find(|r| r.id == id)
}
fn scheduled(
    l: &LabConfig,
    t: u64,
    actor: u64,
    departure: bool,
) -> (&'static str, &'static str, Option<Pos>) {
    if actor == 2 {
        if t >= 32 {
            return ("ordinary", "ordinary", None);
        }
        if !l.display_seen {
            let route = if (8..13).contains(&t) {
                Some(([(4, 6), (5, 6), (6, 6), (7, 6), (7, 7)], t - 8))
            } else if (14..19).contains(&t) {
                Some(([(7, 6), (6, 6), (5, 6), (4, 6), (3, 6)], t - 14))
            } else {
                None
            };
            if let Some((points, i)) = route {
                let (x, y) = points[i as usize];
                return (
                    "scripted_observer",
                    "walk_without_gather",
                    Some(pos(l, x, y)),
                );
            }
        }
        return ("scripted_observer", "hold", None);
    }
    if departure {
        return ("departure", "departure", Some(pos(l, 3, 2)));
    }
    if t < 8 {
        return ("preparation", if t == 0 { "prepare" } else { "hold" }, None);
    }
    if l.sender == SenderPolicy::Ordinary || t >= 20 {
        return ("ordinary", "ordinary", None);
    }
    if t == 13 {
        return (
            "display",
            if l.sender == SenderPolicy::Sham {
                "sham"
            } else {
                "neutral"
            },
            None,
        );
    }
    let (phase, points, i): (&str, &[(u32, u32)], usize) = if t < 13 {
        (
            "todisplay",
            match l.layout {
                Layout::OnRoute => &[(3, 4), (3, 5)],
                Layout::OffRoute => &[(4, 3), (5, 3), (5, 4), (5, 5), (5, 6)],
            },
            (t - 8) as usize,
        )
    } else {
        (
            "return",
            match l.layout {
                Layout::OnRoute => &[(2, 5), (2, 4), (2, 3)],
                Layout::OffRoute => &[(5, 5), (4, 5), (3, 5), (2, 5), (2, 4), (2, 3)],
            },
            (t - 14) as usize,
        )
    };
    if let Some(&(x, y)) = points.get(i) {
        (phase, "walk_and_gather", Some(pos(l, x, y)))
    } else {
        (phase, "hold", None)
    }
}
fn compare_ledger(a: &Ledger, b: &Ledger) -> Result<(), String> {
    a.reconcile()?;
    b.reconcile()?;
    fn equal(a: &serde_json::Value, b: &serde_json::Value) -> bool {
        match (a, b) {
            (serde_json::Value::Number(x), serde_json::Value::Number(y)) => near(
                x.as_f64().unwrap_or(f64::NAN),
                y.as_f64().unwrap_or(f64::NAN),
            ),
            (serde_json::Value::Object(x), serde_json::Value::Object(y)) => {
                x.len() == y.len()
                    && x.iter()
                        .all(|(k, v)| y.get(k).is_some_and(|other| equal(v, other)))
            }
            _ => a == b,
        }
    }
    if !equal(
        &serde_json::to_value(a).map_err(|e| e.to_string())?,
        &serde_json::to_value(b).map_err(|e| e.to_string())?,
    ) {
        return Err("saved cohort ledger disagrees with reconstructed physical actions".into());
    }
    Ok(())
}
// P4-only saved-state reconstruction: no World, policy dispatch, or random draws.
fn ordinary_candidates(
    before: &RoleRecord,
    positions: &BTreeMap<u64, Pos>,
    stocks: &[StockRecord],
    caches: &BTreeMap<u32, f64>,
    memory: &[SeenRecord],
) -> Vec<(Pos, u32, f64)> {
    let occupied = |p: Pos| positions.iter().any(|(&id, &q)| id != before.id && p == q);
    let vision = if before.id == 1 { 2 } else { 6 };
    let mut values = BTreeMap::<u32, (u32, f64)>::new();
    for stock in stocks {
        let p = Pos::new(stock.site % 9, stock.site / 9);
        let distance = before.pos.x.abs_diff(p.x) + before.pos.y.abs_diff(p.y);
        if legal(p)
            && !occupied(p)
            && (p.x == before.pos.x || p.y == before.pos.y)
            && distance <= vision
        {
            values.insert(stock.site, (distance, stock.amount));
        }
    }
    let additions = if before.id == 1 && before.holdings < 4.0 {
        caches.clone()
    } else if before.id == 2 {
        let mut seen = BTreeMap::<u32, f64>::new();
        for e in memory {
            *seen.entry(e.site).or_default() += e.amount;
        }
        for value in seen.values_mut() {
            *value = value.min(128.0 - before.holdings);
        }
        seen
    } else {
        BTreeMap::new()
    };
    for (index, amount) in additions {
        let p = Pos::new(index % 9, index / 9);
        if legal(p) && !occupied(p) {
            let dx = before.pos.x.abs_diff(p.x);
            let dy = before.pos.y.abs_diff(p.y);
            values
                .entry(index)
                .and_modify(|(_, value)| *value = value.max(amount))
                .or_insert((dx.min(9 - dx) + dy.min(9 - dy), amount));
        }
    }
    values
        .into_iter()
        .map(|(index, (distance, value))| (Pos::new(index % 9, index / 9), distance, value))
        .collect()
}
fn ordinary_steps(before: &RoleRecord, positions: &BTreeMap<u64, Pos>, target: Pos) -> Vec<Pos> {
    if target == before.pos {
        return vec![before.pos];
    }
    let mut distance = [u32::MAX; 81];
    let mut queue = std::collections::VecDeque::from([target]);
    distance[site(target) as usize] = 0;
    while let Some(p) = queue.pop_front() {
        for (dx, dy) in [(0, -1), (-1, 0), (1, 0), (0, 1)] {
            let q = Pos::new((p.x as i32 + dx) as u32, (p.y as i32 + dy) as u32);
            if !legal(q)
                || positions
                    .iter()
                    .any(|(&id, &other)| id != before.id && other == q)
            {
                continue;
            }
            if distance[site(q) as usize] == u32::MAX {
                distance[site(q) as usize] = distance[site(p) as usize] + 1;
                queue.push_back(q);
            }
        }
    }
    let d = distance[site(before.pos) as usize];
    if d == u32::MAX {
        return vec![before.pos];
    }
    [(0, -1), (-1, 0), (1, 0), (0, 1)]
        .into_iter()
        .filter_map(|(dx, dy)| {
            let q = Pos::new(
                (before.pos.x as i32 + dx) as u32,
                (before.pos.y as i32 + dy) as u32,
            );
            (legal(q)
                && distance[site(q) as usize] == d - 1
                && !positions.iter().any(|(&id, &p)| id != before.id && p == q))
            .then_some(q)
        })
        .collect()
}
fn audit_ordinary(
    before: &RoleRecord,
    positions: &BTreeMap<u64, Pos>,
    stocks: &[StockRecord],
    caches: &BTreeMap<u32, f64>,
    memory: &[SeenRecord],
    target: Pos,
    after: Pos,
) -> Result<(), String> {
    let candidates = ordinary_candidates(before, positions, stocks, caches, memory);
    let chosen = candidates
        .iter()
        .find(|c| c.0 == target)
        .ok_or("ordinary target is not a candidate")?;
    if candidates
        .iter()
        .any(|c| c.2 > chosen.2 || c.2 == chosen.2 && c.1 < chosen.1)
    {
        return Err("ordinary target violates maximum value / nearest tie policy".into());
    }
    if !ordinary_steps(before, positions, target).contains(&after) {
        return Err("ordinary movement is not a legal shortest step toward selected target".into());
    }
    Ok(())
}

fn validate_episode(r: &EpisodeRecord) -> Result<(), String> {
    let fail = |s: &str| Err(s.to_string());
    if r.schema != SCHEMA
        || r.requested_ticks != 64
        || r.completed_ticks > 64
        || r.frames.len() != r.completed_ticks as usize + 1
        || r.frames.is_empty()
        || !sugarscape_core::minds::deception::conditions().contains(&r.lab)
    {
        return fail("schema/configuration/horizon/frame count mismatch");
    }
    if !r.fixture_errors.is_empty() {
        return fail("fixture-invalid episode");
    }
    let l = &r.lab;
    let initial = &r.frames[0];
    if initial.roles.len() != 2
        || role(initial, 1).is_none_or(|a| {
            a.pos != pos(l, 3, 3)
                || a.holdings != 44.0
                || a.sender != Some(SenderState::default())
                || !a.caches.is_empty()
                || !a.seen.is_empty()
        })
        || role(initial, 2).is_none_or(|a| {
            a.pos != pos(l, 3, 6)
                || a.holdings != 96.0
                || a.sender.is_some()
                || !a.caches.is_empty()
                || !a.seen.is_empty()
        })
        || !initial.actions.is_empty()
        || !initial.observations.is_empty()
        || !initial.choices.is_empty()
        || !initial.deaths.is_empty()
        || !initial.restrictions.is_empty()
    {
        return fail("invalid initial balances/roles/memory");
    }
    let mut ledger = Ledger::new(1, 44.0);
    let mut restrictions = BTreeMap::new();
    let mut alive_ticks = 0;
    let mut recovered = false;
    for (i, f) in r.frames.iter().enumerate() {
        if f.tick != i as u64
            || f.fingerprint.len() != 16
            || !f.fingerprint.bytes().all(|b| b.is_ascii_hexdigit())
        {
            return fail("invalid frame tick/fingerprint");
        }
        if f.stocks.len() != 81
            || f.stocks
                .iter()
                .enumerate()
                .any(|(j, s)| s.site != j as u32 || !s.amount.is_finite() || s.amount < 0.0)
        {
            return fail("invalid environmental stocks");
        }
        let live = f.roles.iter().map(|a| a.id).collect::<BTreeSet<_>>();
        if live.len() != f.roles.len()
            || live.iter().any(|id| ![1, 2].contains(id))
            || f.roles.len() == 2 && f.roles[0].pos == f.roles[1].pos
        {
            return fail("duplicate/unknown/overlapping roles");
        }
        for a in &f.roles {
            if !legal(a.pos)
                || !a.holdings.is_finite()
                || a.holdings <= 0.0
                || a.holdings > 128.0
                || a.caches
                    .iter()
                    .any(|(&s, &q)| s >= 81 || !q.is_finite() || q < 0.0)
                || a.seen.iter().any(|e| {
                    e.owner != 1
                        || e.site >= 81
                        || !e.amount.is_finite()
                        || e.amount <= 0.0
                        || e.tick >= f.tick
                })
                || a.seen
                    .iter()
                    .map(|e| (e.site, e.owner))
                    .collect::<BTreeSet<_>>()
                    .len()
                    != a.seen.len()
                || a.id == 2 && (!a.caches.is_empty() || a.sender.is_some())
                || a.id == 1 && (a.sender.is_none() || !a.seen.is_empty())
            {
                return fail("invalid role position/holdings/cache/evidence/state");
            }
        }
        if i == 0 {
            for s in &f.stocks {
                if s.amount
                    != if [site(pos(l, 2, 2)), site(pos(l, 2, 3))].contains(&s.site) {
                        4.0
                    } else {
                        0.0
                    }
                {
                    return fail("initial finite patch mismatch");
                }
            }
            continue;
        }
        if f.tick == 32
            && (role(f, 2).is_none_or(|a| a.pos != pos(l, 3, 6))
                || f.roles
                    .iter()
                    .any(|a| [pos(l, 3, 3), pos(l, 3, 5), pos(l, 5, 6)].contains(&a.pos)))
        {
            return fail("release requires source/display vacancy and observer at release site");
        }
        let prev = &r.frames[i - 1];
        let tick = (i - 1) as u64;
        alive_ticks += u64::from(role(prev, 1).is_some());
        let prior = prev.roles.iter().map(|a| a.id).collect::<BTreeSet<_>>();
        let actors = f.actions.iter().map(|a| a.actor).collect::<BTreeSet<_>>();
        let deaths = f.deaths.iter().map(|d| d.actor).collect::<BTreeSet<_>>();
        if actors != prior
            || actors.len() != f.actions.len()
            || deaths.len() != f.deaths.len()
            || !live.is_subset(&prior)
            || deaths != prior.difference(&live).copied().collect()
            || f.deaths
                .iter()
                .any(|d| d.cause != "starvation" || !legal(d.pos))
        {
            return fail("actions/deaths do not reconcile live roles");
        }
        let mut positions = prev
            .roles
            .iter()
            .map(|r| (r.id, r.pos))
            .collect::<BTreeMap<_, _>>();
        let mut caches = role(prev, 1).map(|r| r.caches.clone()).unwrap_or_default();
        let mut stocks = prev.stocks.clone();
        let mut memory = role(prev, 2).map(|r| r.seen.clone()).unwrap_or_default();
        let mut expected_observations = vec![];
        let mut choices = BTreeSet::new();
        let mut pending = role(prev, 1)
            .and_then(|r| r.sender.as_ref())
            .is_some_and(|s| s.pending_departure);
        for a in &f.actions {
            let before = role(prev, a.actor).ok_or("action without previous role")?;
            let after = role(f, a.actor)
                .map(|r| r.pos)
                .or_else(|| f.deaths.iter().find(|d| d.actor == a.actor).map(|d| d.pos))
                .ok_or("missing final role position")?;
            if a.pos != Some(after)
                || !legal(after)
                || before.pos.x.abs_diff(after.x) + before.pos.y.abs_diff(after.y) > 1
                || positions
                    .iter()
                    .any(|(&id, &p)| id != a.actor && p == after)
            {
                return fail("physical movement/occupancy mismatch");
            }
            if [
                a.harvest,
                a.dug,
                a.buried,
                a.effort,
                a.metabolic_demand,
                a.metabolic_consumed,
            ]
            .iter()
            .any(|q| !q.is_finite() || *q < 0.0)
                || a.metabolic_demand != 1.0
                || a.metabolic_consumed > 1.0
                || a.harvest > 4.0
            {
                return fail("nonfinite/invalid action food/metabolism");
            }
            let (phase, action, target) = scheduled(l, tick, a.actor, pending && a.actor == 1);
            if a.phase != phase
                || a.action != action
                || (action != "ordinary" && a.target != target)
            {
                return Err(format!("tick {tick} actor {} phase/action/route mismatch: {:?} {:?} {:?}; expected {phase} {action} {target:?}",a.actor,a.phase,a.action,a.target));
            }
            if a.actor == 1 {
                pending = false;
            }
            if let Some(target) = target {
                let occupant = positions
                    .iter()
                    .find(|(id, p)| **id != a.actor && **p == target)
                    .map(|(&id, _)| id);
                let outcome = if after == target {
                    "arrived"
                } else if after != before.pos {
                    "advanced"
                } else if occupant.is_some() {
                    "blocked"
                } else {
                    "unreachable"
                };
                if a.target_occupant != occupant
                    || a.walk_outcome.as_deref() != Some(outcome)
                    || a.source_recovered != recovered
                    || (action != "departure" && after != target)
                {
                    return fail("walk outcome/target occupancy/recovery mismatch");
                }
            } else if a.walk_outcome.is_some() || a.target_occupant.is_some() || a.source_recovered
            {
                return fail("unexpected walk diagnostics");
            }
            if matches!(
                action,
                "hold" | "prepare" | "neutral" | "sham" | "walk_without_gather"
            ) && (a.harvest != 0.0 || a.dug != 0.0)
            {
                return fail("restricted action gathered food");
            }
            if matches!(action, "hold" | "prepare" | "neutral" | "sham") && after != before.pos {
                return fail("stationary action moved role");
            }
            let expected_burial = if a.actor == 1 && tick == 0 { 12.0 } else { 0.0 };
            if a.buried != expected_burial {
                return fail("original burial or zero-food gesture mismatch");
            }
            let bout = matches!(action, "sham" | "neutral");
            let affordable = before.holdings >= l.effort_cost;
            if a.effort
                != if bout && affordable {
                    l.effort_cost
                } else {
                    0.0
                }
                || a.cancellation
                    != if bout && !affordable {
                        Some(
                            sugarscape_core::minds::deception::controller::BoutResult::Unaffordable,
                        )
                    } else {
                        None
                    }
            {
                return fail("gesture charge/cancellation mismatch");
            }
            let choice = f.choices.iter().find(|c| c.actor == a.actor);
            if action == "ordinary" {
                let c = choice.ok_or("ordinary action lacks choice/inspection")?;
                choices.insert(a.actor);
                audit_ordinary(
                    before, &positions, &stocks, &caches, &memory, c.target, after,
                )?;
                if a.target != Some(c.target)
                    || !legal(c.target)
                    || [c.remembered_value, c.actual_value, c.raid_amount]
                        .iter()
                        .any(|q| !q.is_finite() || *q < 0.0)
                    || c.inspection != Some(after)
                    || c.inspected_stock
                        .is_none_or(|q| !near(q, *caches.get(&site(after)).unwrap_or(&0.0)))
                    || c.arrived != (after == c.target)
                    || c.target_occupant
                        != positions
                            .iter()
                            .find(|(id, p)| **id != a.actor && **p == c.target)
                            .map(|(&id, _)| id)
                    || c.source_recovered != recovered
                {
                    return fail("choice target/actual inspection/occupancy mismatch");
                }
                let environment = stocks[site(c.target) as usize].amount;
                let own_cache = if a.actor == 1 && before.holdings < 4.0 {
                    *caches.get(&site(c.target)).unwrap_or(&0.0)
                } else {
                    0.0
                };
                let remembered = if a.actor == 2 {
                    memory
                        .iter()
                        .filter(|e| e.site == site(c.target))
                        .map(|e| e.amount)
                        .sum::<f64>()
                        .min(128.0 - before.holdings)
                } else {
                    0.0
                };
                let seen_actual = if remembered > 0.0 {
                    caches
                        .get(&site(c.target))
                        .copied()
                        .unwrap_or(0.0)
                        .min(128.0 - before.holdings)
                } else {
                    0.0
                };
                let vision = if a.actor == 1 { 2 } else { 6 };
                let visible = (c.target.x == before.pos.x || c.target.y == before.pos.y)
                    && c.target.x.abs_diff(before.pos.x) + c.target.y.abs_diff(before.pos.y)
                        <= vision
                    && !positions
                        .iter()
                        .any(|(&id, &p)| id != a.actor && p == c.target);
                if !near(c.actual_value, environment.max(own_cache).max(seen_actual))
                    || !near(
                        c.remembered_value,
                        (if visible { environment } else { 0.0 })
                            .max(own_cache)
                            .max(remembered),
                    )
                {
                    return fail("choice values disagree with physical stock/public memory");
                }
                if c.raid_amount > 0.0
                    && (a.actor != 2 || c.wasted || !memory.iter().any(|e| e.site == site(after)))
                {
                    return fail("raid without receiver evidence");
                }
                if a.actor == 2 {
                    let remembered = memory.iter().any(|e| e.site == site(after));
                    let actual = *caches.get(&site(after)).unwrap_or(&0.0);
                    if c.wasted != (remembered && actual == 0.0)
                        || !near(
                            c.raid_amount,
                            if remembered {
                                actual.min(128.0 - before.holdings)
                            } else {
                                0.0
                            },
                        )
                    {
                        return fail("raid/wasted inspection contradicts evidence/stock");
                    }
                    memory.retain(|e| e.site != site(after));
                    if c.raid_amount > 0.0 {
                        ledger.pilfer(site(after), c.raid_amount)?;
                        *caches.entry(site(after)).or_default() -= c.raid_amount;
                    }
                }
            } else if choice.is_some() {
                return fail("choice on scripted action");
            }
            if a.actor == 2 && (a.dug != 0.0 || a.effort != 0.0 || a.buried != 0.0) {
                return fail("observer performed sender operation");
            }
            let raid = choice.map_or(0.0, |c| c.raid_amount);
            validate_food_operation(
                a,
                before.holdings,
                *caches.get(&site(after)).unwrap_or(&0.0),
                stocks[site(after) as usize].amount,
                raid,
            )?;
            if a.harvest > 0.0 {
                let stock = &mut stocks[site(after) as usize];
                if a.harvest > stock.amount {
                    return fail("harvest exceeds finite environmental patch");
                }
                stock.amount -= a.harvest;
            }
            if a.actor == 1 {
                if a.dug > 0.0 {
                    if site(after) != site(pos(l, 3, 3))
                        || a.dug > *caches.get(&site(after)).unwrap_or(&0.0)
                    {
                        return fail("remote/excess source recovery");
                    }
                    ledger.withdraw(site(after), a.dug)?;
                    *caches.entry(site(after)).or_default() -= a.dug;
                    recovered = true;
                    pending = true;
                }
                ledger.harvest(a.harvest)?;
                if a.buried > 0.0 {
                    ledger.prepare(site(after), a.buried)?;
                    *caches.entry(site(after)).or_default() += a.buried;
                }
                ledger.outflow(a.effort, Outflow::ActionCost)?;
                if tick == 0 || action == "sham" && affordable && l.display_seen {
                    let transfer = if tick == 0 { 12.0 } else { 0.0 };
                    let signal = if tick == 0 || l.view == View::Clear {
                        Signal::VisibleTransfer { amount: transfer }
                    } else {
                        Signal::Cue {
                            nominal_amount: 12.0,
                        }
                    };
                    let obs = ObservedRecord {
                        receiver: 2,
                        public: sugarscape_core::minds::deception::observation::Observation {
                            actor: 1,
                            site: site(after),
                            tick,
                            signal,
                        },
                        actual_transfer: transfer,
                        actual_stock: *caches.get(&site(after)).unwrap_or(&0.0),
                    };
                    let amount = match obs.public.signal {
                        Signal::Cue { nominal_amount } => nominal_amount,
                        Signal::VisibleTransfer { amount } => amount,
                    };
                    if amount > 0.0 {
                        if let Some(e) = memory.iter_mut().find(|e| e.site == site(after)) {
                            e.amount += amount;
                            e.tick = tick;
                        } else {
                            memory.push(SeenRecord {
                                site: site(after),
                                owner: 1,
                                amount,
                                tick,
                            });
                        }
                        memory.sort_by_key(|e| (e.site, e.owner));
                    }
                    expected_observations.push(obs);
                }
            }
            let pre = before.holdings + a.harvest + a.dug + raid - a.buried - a.effort;
            if !near(a.metabolic_consumed, pre.clamp(0.0, 1.0)) {
                return fail("consumption differs from physical holdings");
            }
            if a.actor == 1 {
                ledger.outflow(a.metabolic_consumed, Outflow::Consumption)?;
            }
            if let Some(after_role) = role(f, a.actor) {
                if !near(after_role.holdings, pre - 1.0) {
                    return fail("holdings fail action reconciliation");
                }
            } else {
                if pre > 1.0 {
                    return fail("invalid starvation/removal");
                }
                if a.actor == 1 {
                    ledger.lose_owner();
                    caches.clear();
                }
                positions.remove(&a.actor);
            }
            if a.actor == 1
                && tick
                    < if l.sender == SenderPolicy::Ordinary {
                        8
                    } else {
                        20
                    }
                || a.actor == 2 && tick < 32
            {
                *restrictions.entry(a.actor).or_default() += 1;
            }
            if live.contains(&a.actor) {
                positions.insert(a.actor, after);
            }
        }
        if f.choices.len() != choices.len()
            || f.observations != expected_observations
            || f.restrictions != restrictions
        {
            return fail("observation/choice/restriction count mismatch");
        }
        for (s, predicted) in f.stocks.iter().zip(stocks) {
            if !near(s.amount, predicted.amount) {
                return fail("environmental stock reconciliation failed");
            }
        }
        if let Some(owner) = role(f, 1) {
            for s in owner.caches.keys().chain(caches.keys()) {
                if !near(
                    *owner.caches.get(s).unwrap_or(&0.0),
                    *caches.get(s).unwrap_or(&0.0),
                ) {
                    return fail("physical cache reconciliation failed");
                }
            }
            let carried =
                ledger.unlabelled_carried + ledger.cohorts.values().map(|c| c.carried).sum::<f64>();
            if !near(carried, owner.holdings) {
                return fail("cohort carried balance mismatch");
            }
            let s = owner.sender.as_ref().ok_or("owner missing state")?;
            let action = f
                .actions
                .iter()
                .find(|a| a.actor == 1)
                .ok_or("missing owner action")?;
            if format!("{:?}", s.stage).to_lowercase() != action.phase
                || s.pending_departure != pending
                || s.attempted != (l.sender != SenderPolicy::Ordinary && tick >= 13)
            {
                return fail("sender attempt/departure state mismatch");
            }
        }
        if role(f, 2).is_some_and(|r| r.seen != memory) {
            return fail("saved receiver belief disagrees with public evidence/inspection");
        }
        ledger.reconcile()?;
    }
    let last = r.frames.last().ok_or("missing terminal frame")?;
    if r.completed_ticks < 64 && !last.roles.is_empty()
        || r.owner_ticks_alive != alive_ticks
        || r.owner_alive != role(last, 1).is_some()
    {
        return fail("early horizon or owner lifetime mismatch");
    }
    match (&r.cohorts, r.thief_transferred) {
        (Some(saved), Some(food)) => {
            if !r.diagnostics_enabled
                || r.lineage_unavailable_reason.is_some()
                || !r.ledger_errors.is_empty()
            {
                return fail("available lineage has inconsistent status");
            }
            compare_ledger(saved, &ledger)?;
            if !near(food, ledger.cohorts.values().map(|c| c.transferred).sum()) {
                return fail("food endpoint differs from original cohort transfers");
            }
        }
        (None, None) => {
            if r.diagnostics_enabled
                || r.lineage_unavailable_reason
                    .as_ref()
                    .is_none_or(|s| s.trim().is_empty())
            {
                return fail("unavailable lineage requires nonblank reason");
            }
        }
        _ => return fail("partial lineage availability mismatch"),
    }
    Ok(())
}

fn validate_food_operation(
    a: &ActionRecord,
    held: f64,
    cache: f64,
    stock: f64,
    raid: f64,
) -> Result<(), String> {
    let gathers = matches!(
        a.action.as_str(),
        "ordinary" | "walk_and_gather" | "departure"
    );
    let digs = gathers && a.actor == 1 && held < 4.0 && cache > 0.0 && cache >= stock;
    let expected_dug = if digs { cache.min(128.0 - held) } else { 0.0 };
    let expected_harvest = if gathers && !digs && raid == 0.0 {
        stock.min(128.0 - held)
    } else {
        0.0
    };
    if !near(a.dug, expected_dug) || !near(a.harvest, expected_harvest) {
        return Err("food operation contradicts reserve/arrival stocks".into());
    }
    Ok(())
}

#[cfg(test)]
pub(super) mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};
    static NEXT: AtomicU64 = AtomicU64::new(0);
    pub struct Temp(pub PathBuf);
    impl Temp {
        pub fn new() -> Self {
            let p = std::env::temp_dir().canonicalize().unwrap().join(format!(
                "task4-test-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::SeqCst)
            ));
            fs::create_dir(&p).unwrap();
            Self(p)
        }
    }
    impl Drop for Temp {
        fn drop(&mut self) {
            fs::remove_dir_all(&self.0).unwrap();
        }
    }
    pub fn provenance() -> Provenance {
        let protocol_bytes = "explicit synthetic static fixture, not science".to_string();
        let protocol = FileIdentity {
            path: PROTOCOL.into(),
            sha256: hash(protocol_bytes.as_bytes()),
            bytes: protocol_bytes.len() as u64,
            mode: "100644".into(),
        };
        let source_inventory = vec![protocol.clone()];
        Provenance {
            source_revision: "a".repeat(40),
            protocol_revision: "b".repeat(40),
            binary: FileIdentity {
                path: "synthetic-binary".into(),
                sha256: "c".repeat(64),
                bytes: 1,
                mode: "100755".into(),
            },
            protocol,
            protocol_bytes,
            source_sha256: hash(&json(&source_inventory).unwrap()),
            source_inventory,
            manifest_sha256: hash(&json(&manifest()).unwrap()),
        }
    }
    pub fn index() -> Index {
        let m = manifest();
        Index {
            schema: SCHEMA.into(),
            provenance: provenance(),
            attempts: refs(&m),
            manifest: m,
            completed: false,
            census: None,
        }
    }
    pub fn start(i: &Index, r: &AttemptRef) -> AttemptStart {
        AttemptStart {
            schema: SCHEMA.into(),
            condition: r.condition.clone(),
            seed: r.seed,
            provenance: i.provenance.identity(),
            started_unix_ms: 1,
        }
    }
    #[test]
    fn deception_fix1_primitives_and_portable_temp() {
        let dir = Temp::new();
        assert!(dir
            .0
            .starts_with(std::env::temp_dir().canonicalize().unwrap()));
        assert_eq!(
            hash(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
        assert_eq!(json(&vec![1, 2]).unwrap(), b"[1,2]");
        let path = dir.0.join("value.json");
        assert!(!exists(&path).unwrap());
        write_new(&path, b"[1,2]").unwrap();
        assert_eq!(read::<Vec<u32>>(&path).unwrap(), vec![1, 2]);
        assert!(exists(&path).unwrap());
        let child = dir.0.join("child");
        new_directory(&child).unwrap();
        assert_eq!(
            confined(&dir.0, "child/value").unwrap(),
            child.join("value")
        );
    }
    #[test]
    fn deception_fix1_revision_format() {
        assert!(validate_revision(&"a".repeat(40)).is_ok());
        for invalid in ["HEAD".to_string(), "g".repeat(40), "a".repeat(39)] {
            assert!(validate_revision(&invalid).is_err());
        }
    }
    #[test]
    fn deception_fix1_run_preflight_cannot_start_on_abbreviation() {
        let dir = Temp::new();
        assert!(run("HEAD", &dir.0.join("denied")).is_err());
        assert!(!dir.0.join("denied").exists());
    }
    #[test]
    #[allow(clippy::result_large_err)]
    fn deception_fix1_attempt_start_precedes_runner_and_complete_matches_stream() {
        let dir = Temp::new();
        fs::create_dir(dir.0.join("attempts")).unwrap();
        let i = index();
        let r = &i.attempts[0];
        let c = &i.manifest.conditions[0];
        execute_attempt(&dir.0, r, &i.provenance, c, |_, seed, sink| {
            let saved: AttemptStart = read(&dir.0.join(&r.start)).unwrap();
            assert_eq!(saved.seed, seed);
            assert!(dir.0.join(&r.frames).exists());
            assert!(!dir.0.join(&r.outcome).exists());
            let record = static_record(c.lab.clone(), seed);
            for f in &record.frames {
                sink(f).unwrap();
            }
            Ok(record)
        })
        .unwrap();
        let receipt: OutcomeReceipt = read(&dir.0.join(&r.outcome)).unwrap();
        let AttemptOutcome::Complete(record) = receipt.outcome else {
            panic!("complete expected")
        };
        assert_eq!(
            fs::read_to_string(dir.0.join(&r.frames))
                .unwrap()
                .lines()
                .count(),
            record.frames.len()
        );
    }
    #[test]
    fn deception_fix1_complete_bad_stream_census_retains_complete_and_invalid() {
        for mode in ["malformed", "missing", "conflicting"] {
            let dir = Temp::new();
            let i = index();
            let r = &i.attempts[0];
            write_new(&dir.0.join("index.json"), &json(&i).unwrap()).unwrap();
            fs::create_dir_all(dir.0.join(&r.start).parent().unwrap()).unwrap();
            write_new(&dir.0.join(&r.start), &json(&start(&i, r)).unwrap()).unwrap();
            let record = static_record(i.manifest.conditions[0].lab.clone(), r.seed);
            let outcome = OutcomeReceipt {
                condition: r.condition.clone(),
                seed: r.seed,
                elapsed_seconds: 0.0,
                timing_boundary: TIMING.into(),
                outcome: AttemptOutcome::Complete(record),
            };
            write_new(&dir.0.join(&r.outcome), &json(&outcome).unwrap()).unwrap();
            if mode != "missing" {
                write_new(
                    &dir.0.join(&r.frames),
                    if mode == "malformed" { b"{bad}\n" } else { b"" },
                )
                .unwrap();
            }
            let error = load(&dir.0.join("index.json")).unwrap_err();
            assert!(
                error.contains("attempted: 1")
                    && error.contains("complete: 1")
                    && error.contains("invalid: 1"),
                "{mode}: {error}"
            );
        }
    }
    #[test]
    fn deception_fix1_balanced_but_impossible_old_ordinary_fixture_is_rejected() {
        let mut record = static_record(
            LabConfig {
                sender: SenderPolicy::Ordinary,
                ..Default::default()
            },
            20001,
        );
        let frame = &mut record.frames[9];
        let after = Pos::new(3, 2);
        frame.roles.iter_mut().find(|r| r.id == 1).unwrap().pos = after;
        let action = frame.actions.iter_mut().find(|a| a.actor == 1).unwrap();
        action.pos = Some(after);
        action.target = Some(after);
        let choice = frame.choices.iter_mut().find(|c| c.actor == 1).unwrap();
        choice.target = after;
        choice.inspection = Some(after);
        choice.remembered_value = 0.0;
        choice.actual_value = 0.0;
        assert!(
            validate_episode(&record)
                .unwrap_err()
                .contains("ordinary target"),
            "tick 8 ignores visible value-four patch"
        );
    }
    #[test]
    fn deception_fix1_ordinary_candidate_value_distance_and_direction_mutations() {
        let before = RoleRecord {
            id: 1,
            pos: Pos::new(3, 3),
            holdings: 20.0,
            caches: BTreeMap::new(),
            sender: Some(SenderState::default()),
            seen: vec![],
        };
        let positions = BTreeMap::from([(1, before.pos), (2, Pos::new(3, 6))]);
        let stocks = (0..81)
            .map(|site| StockRecord {
                site,
                amount: if site == 29 || site == 28 { 4.0 } else { 0.0 },
            })
            .collect::<Vec<_>>();
        let cache = BTreeMap::new();
        let mut rejected = vec![];
        for (target, after, why) in [
            (Pos::new(4, 4), Pos::new(4, 3), "noncandidate diagonal"),
            (Pos::new(3, 2), Pos::new(3, 2), "lower value"),
            (Pos::new(1, 3), Pos::new(2, 3), "farther equal value"),
            (
                Pos::new(2, 3),
                Pos::new(3, 2),
                "step away from chosen target",
            ),
            (Pos::new(2, 3), Pos::new(3, 3), "unjustified stay"),
        ] {
            rejected.push((
                why,
                audit_ordinary(&before, &positions, &stocks, &cache, &[], target, after).is_err(),
            ));
        }
        assert!(
            rejected.iter().all(|(_, rejected)| *rejected),
            "{rejected:?}"
        );
        audit_ordinary(
            &before,
            &positions,
            &stocks,
            &cache,
            &[],
            Pos::new(2, 3),
            Pos::new(2, 3),
        )
        .unwrap();
    }
    #[test]
    #[allow(clippy::result_large_err)]
    fn deception_fix1_campaign_records_failures_continues_once_and_finalizes_census() {
        let dir = Temp::new();
        let mut i = index();
        i.attempts.truncate(2);
        let mut calls = 0;
        let done = execute_campaign(&dir.0, i, |_, _, _| {
            calls += 1;
            Err(EpisodeFailure {
                message: "static biological failure".into(),
                partial: None,
            })
        })
        .unwrap();
        assert_eq!(calls, 2);
        assert!(done.completed);
        let counts = done.census.unwrap();
        assert_eq!((counts.attempted, counts.failed, counts.pending), (2, 2, 0));
        assert!(dir.0.join("index.json").exists());
        assert!(dir.0.join("final-index.json").exists());
    }
    #[test]
    fn deception_fix1_all_static_cells_follow_fixed_policy() {
        for condition in manifest().conditions {
            let record = static_record(condition.lab, 20001);
            validate_episode(&record).unwrap_or_else(|e| panic!("{}: {e}", condition.id));
        }
    }
    #[test]
    fn deception_fix1_blank_durable_line_is_invalid_pending() {
        let dir = Temp::new();
        let i = index();
        let r = &i.attempts[0];
        write_new(&dir.0.join("index.json"), &json(&i).unwrap()).unwrap();
        fs::create_dir_all(dir.0.join(&r.start).parent().unwrap()).unwrap();
        write_new(&dir.0.join(&r.start), &json(&start(&i, r)).unwrap()).unwrap();
        let mut bytes = json(&minimal_frame()).unwrap();
        bytes.extend_from_slice(b"\n\n");
        write_new(&dir.0.join(&r.frames), &bytes).unwrap();
        let error = load(&dir.0.join("index.json")).unwrap_err();
        assert!(
            error.contains("invalid: 1") && error.contains("pending: 1"),
            "{error}"
        );
    }
    #[test]
    fn deception_fix1_invalid_failed_frame_prefix_keeps_failed_partial_state() {
        let dir = Temp::new();
        let i = index();
        let r = &i.attempts[0];
        write_new(&dir.0.join("index.json"), &json(&i).unwrap()).unwrap();
        fs::create_dir_all(dir.0.join(&r.start).parent().unwrap()).unwrap();
        write_new(&dir.0.join(&r.start), &json(&start(&i, r)).unwrap()).unwrap();
        write_new(&dir.0.join(&r.frames), b"{malformed}\n").unwrap();
        let record = static_record(i.manifest.conditions[0].lab.clone(), r.seed);
        let receipt = OutcomeReceipt {
            condition: r.condition.clone(),
            seed: r.seed,
            elapsed_seconds: 0.0,
            timing_boundary: TIMING.into(),
            outcome: AttemptOutcome::Failed(EpisodeFailure {
                message: "known failed attempt".into(),
                partial: Some(record),
            }),
        };
        write_new(&dir.0.join(&r.outcome), &json(&receipt).unwrap()).unwrap();
        let error = load(&dir.0.join("index.json")).unwrap_err();
        assert!(
            error.contains("invalid: 1")
                && error.contains("failed: 1")
                && error.contains("partial: 1"),
            "{error}"
        );
    }
    #[test]
    fn deception_fix1_visible_distance_does_not_wrap_through_opaque_border() {
        let observer = RoleRecord {
            id: 2,
            pos: Pos::new(1, 3),
            holdings: 40.0,
            caches: BTreeMap::new(),
            sender: None,
            seen: vec![],
        };
        let positions = BTreeMap::from([(2, observer.pos), (1, Pos::new(4, 4))]);
        let stocks = (0..81)
            .map(|site| StockRecord {
                site,
                amount: if site == 34 || site == 31 { 4.0 } else { 0.0 },
            })
            .collect::<Vec<_>>();
        assert!(
            audit_ordinary(
                &observer,
                &positions,
                &stocks,
                &BTreeMap::new(),
                &[],
                Pos::new(7, 3),
                Pos::new(2, 3)
            )
            .is_err(),
            "six visible steps must lose to three; wall forbids wrapped sight"
        );
    }
    #[test]
    fn deception_fix1_census_rejects_symlink_index_before_reading_it() {
        let dir = Temp::new();
        write_new(&dir.0.join("actual.json"), &json(&index()).unwrap()).unwrap();
        std::os::unix::fs::symlink(dir.0.join("actual.json"), dir.0.join("linked.json")).unwrap();
        assert!(receipt_census(&dir.0.join("linked.json")).is_err());
    }
    fn minimal_frame() -> FrameRecord {
        FrameRecord {
            tick: 0,
            fingerprint: "0000000000000000".into(),
            roles: vec![],
            actions: vec![],
            observations: vec![],
            choices: vec![],
            deaths: vec![],
            restrictions: BTreeMap::new(),
            stocks: vec![],
        }
    }
    #[test]
    fn deception_archive_pending_prefix_retains_identity_and_unstarted_census() {
        let dir = Temp::new();
        let i = index();
        write_new(&dir.0.join("index.json"), &json(&i).unwrap()).unwrap();
        let r = &i.attempts[0];
        fs::create_dir_all(dir.0.join(&r.start).parent().unwrap()).unwrap();
        write_new(&dir.0.join(&r.start), &json(&start(&i, r)).unwrap()).unwrap();
        let f = minimal_frame();
        let mut bytes = serde_json::to_vec(&f).unwrap();
        bytes.extend_from_slice(b"\n{\"tick\":");
        write_new(&dir.0.join(&r.frames), &bytes).unwrap();
        let loaded = load(&dir.0.join("index.json")).unwrap();
        assert_eq!(loaded.attempts[0].frames, vec![f]);
        assert!(loaded.attempts[0].interrupted_tail.is_some());
        assert_eq!(
            (
                census(&loaded).attempted,
                census(&loaded).pending,
                census(&loaded).unstarted
            ),
            (1, 1, 3839)
        );
        assert!(validate_archive(&loaded)
            .unwrap_err()
            .contains("execution census"));
    }
    #[test]
    fn deception_archive_rejects_conflicting_destinations_and_path_attacks() {
        let dir = Temp::new();
        let file = dir.0.join("existing");
        write_new(&file, b"first").unwrap();
        assert!(write_new(&file, b"second").is_err());
        assert_eq!(fs::read(&file).unwrap(), b"first");
        for p in ["/etc/passwd", "../escape", "a/../../escape", ""] {
            assert!(confined(&dir.0, p).is_err());
        }
        std::os::unix::fs::symlink(&dir.0, dir.0.join("link")).unwrap();
        assert!(confined(&dir.0, "link/existing").is_err());
        assert!(new_directory(&dir.0).is_err());
    }
    #[test]
    fn deception_archive_identity_and_budget_tampering_is_rejected() {
        let i = index();
        for mutate in [0, 1, 2, 3] {
            let mut a = Archive {
                index: i.clone(),
                attempts: vec![],
            };
            match mutate {
                0 => a.index.manifest.seeds.pop().map(|_| ()).unwrap(),
                1 => a.index.provenance.source_revision = "wrong".into(),
                2 => a.index.provenance.protocol_bytes.push(' '),
                _ => a.index.attempts[0].frames = "../escape".into(),
            };
            assert!(validate_structure(&a).is_err());
        }
        let mut a = Archive {
            index: i.clone(),
            attempts: vec![Attempt {
                start: start(&i, &i.attempts[0]),
                frames: vec![],
                outcome: None,
                interrupted_tail: None,
            }],
        };
        a.attempts.push(a.attempts[0].clone());
        assert!(validate_structure(&a).is_err());
    }
    #[test]
    fn deception_real_seed7_producer_matches_saved_validator() {
        for lab in sugarscape_core::minds::deception::conditions()
            .into_iter()
            .filter(|lab| !lab.mirrored)
        {
            let r = sugarscape_core::minds::deception::run_episode(lab, 7, true).unwrap();
            validate_episode(&r).unwrap();
        }
    }
    #[test]
    #[allow(clippy::result_large_err)]
    fn deception_attempt_sink_failure_retains_start_partial_and_prefix_without_retry() {
        let dir = Temp::new();
        let i = index();
        fs::create_dir(dir.0.join("attempts")).unwrap();
        write_new(&dir.0.join("index.json"), &json(&i).unwrap()).unwrap();
        let r = &i.attempts[0];
        let c = &i.manifest.conditions[0];
        let mut writes = 0;
        let mut runs = 0;
        let result = execute_attempt_with_writer(
            &dir.0,
            r,
            &i.provenance,
            c,
            |_, _, sink| {
                runs += 1;
                let mut record = static_record(c.lab.clone(), r.seed);
                for j in 0..record.frames.len() {
                    if let Err(message) = sink(&record.frames[j]) {
                        record.frames.truncate(j + 1);
                        record.completed_ticks = j as u64;
                        return Err(EpisodeFailure {
                            message,
                            partial: Some(record),
                        });
                    }
                }
                Ok(record)
            },
            |file, bytes| {
                writes += 1;
                if writes == 2 {
                    file.write_all(b"{").unwrap();
                    file.sync_all().unwrap();
                    return Err("injected durable sink failure".into());
                }
                file.write_all(bytes)
                    .and_then(|_| file.sync_all())
                    .map_err(|e| e.to_string())
            },
        );
        assert!(result
            .unwrap_err()
            .contains("injected durable sink failure"));
        assert_eq!(runs, 1);
        let loaded = load(&dir.0.join("index.json")).unwrap();
        assert_eq!(loaded.attempts[0].frames.len(), 1);
        assert_eq!(loaded.attempts[0].interrupted_tail.as_deref(), Some("{"));
        assert_eq!(
            (
                census(&loaded).failed,
                census(&loaded).partial,
                census(&loaded).complete
            ),
            (1, 1, 0)
        );
        let AttemptOutcome::Failed(f) = &loaded.attempts[0].outcome.as_ref().unwrap().outcome
        else {
            panic!("expected failure receipt");
        };
        assert_eq!(f.partial.as_ref().unwrap().frames.len(), 2);
    }
    #[test]
    fn deception_malformed_saved_data_error_includes_execution_census() {
        let dir = Temp::new();
        let i = index();
        write_new(&dir.0.join("index.json"), &json(&i).unwrap()).unwrap();
        let r = &i.attempts[0];
        fs::create_dir_all(dir.0.join(&r.start).parent().unwrap()).unwrap();
        write_new(&dir.0.join(&r.start), &json(&start(&i, r)).unwrap()).unwrap();
        write_new(&dir.0.join(&r.frames), b"{broken}\n").unwrap();
        let e = load(&dir.0.join("index.json")).unwrap_err();
        assert!(
            e.contains("execution census")
                && e.contains("attempted: 1")
                && e.contains("pending: 1")
                && e.contains("invalid: 1"),
            "{e}"
        );
    }
    #[test]
    fn deception_loader_rejects_swapped_canonical_slot_identities() {
        let dir = Temp::new();
        let i = index();
        write_new(&dir.0.join("index.json"), &json(&i).unwrap()).unwrap();
        for (location, identity) in [(0, 1), (1, 0)] {
            let r = &i.attempts[location];
            fs::create_dir_all(dir.0.join(&r.start).parent().unwrap()).unwrap();
            write_new(
                &dir.0.join(&r.start),
                &json(&start(&i, &i.attempts[identity])).unwrap(),
            )
            .unwrap();
        }
        let result = load(&dir.0.join("index.json"));
        assert!(result.is_err(), "swapped slot was accepted");
        assert!(result.unwrap_err().contains("slot identity"));
    }
    #[test]
    fn deception_loader_malformed_outcome_preserves_known_attempt_count() {
        let dir = Temp::new();
        let i = index();
        write_new(&dir.0.join("index.json"), &json(&i).unwrap()).unwrap();
        let r = &i.attempts[0];
        fs::create_dir_all(dir.0.join(&r.start).parent().unwrap()).unwrap();
        write_new(&dir.0.join(&r.start), &json(&start(&i, r)).unwrap()).unwrap();
        write_new(&dir.0.join(&r.outcome), b"{broken}").unwrap();
        let e = load(&dir.0.join("index.json")).unwrap_err();
        assert!(
            e.contains("attempted: 1") && e.contains("pending: 1") && e.contains("invalid: 1"),
            "{e}"
        );
    }
    #[test]
    fn deception_action_stationary_display_cannot_move_to_return_waypoint() {
        let lab = LabConfig {
            sender: SenderPolicy::MatchedNeutral,
            layout: Layout::OnRoute,
            ..Default::default()
        };
        let mut r = static_record(lab, 20001);
        validate_episode(&r).unwrap();
        let frame = &mut r.frames[14];
        let moved = Pos::new(2, 5);
        frame.roles.iter_mut().find(|a| a.id == 1).unwrap().pos = moved;
        frame.actions.iter_mut().find(|a| a.actor == 1).unwrap().pos = Some(moved);
        assert!(validate_episode(&r).is_err(), "stationary display moved");
    }
    #[test]
    fn deception_action_release_requires_source_and_display_body_vacancy() {
        let lab = LabConfig {
            sender: SenderPolicy::MatchedNeutral,
            layout: Layout::OnRoute,
            ..Default::default()
        };
        let mut r = static_record(lab, 20001);
        validate_episode(&r).unwrap();
        let frame = &mut r.frames[32];
        let moved = Pos::new(3, 3);
        frame.roles.iter_mut().find(|a| a.id == 1).unwrap().pos = moved;
        let a = frame.actions.iter_mut().find(|a| a.actor == 1).unwrap();
        a.pos = Some(moved);
        a.target = Some(moved);
        let c = frame.choices.iter_mut().find(|a| a.actor == 1).unwrap();
        c.target = moved;
        c.inspection = Some(moved);
        c.inspected_stock = Some(12.0);
        assert!(validate_episode(&r).is_err(), "occupied source at release");
    }
    #[test]
    fn deception_action_food_operation_uses_reserve_and_actual_arrival_stock() {
        let mut a = ActionRecord {
            actor: 1,
            action: "ordinary".into(),
            dug: 12.0,
            ..Default::default()
        };
        assert!(
            validate_food_operation(&a, 20.0, 12.0, 0.0, 0.0).is_err(),
            "early dig above reserve"
        );
        a.dug = 0.0;
        assert!(
            validate_food_operation(&a, 1.0, 12.0, 0.0, 0.0).is_err(),
            "missing hungry source dig"
        );
        assert!(
            validate_food_operation(&a, 20.0, 0.0, 4.0, 0.0).is_err(),
            "missing arrival harvest"
        );
        a.harvest = 4.0;
        validate_food_operation(&a, 20.0, 0.0, 4.0, 0.0).unwrap();
    }
    #[test]
    fn deception_census_exposes_invalid_complete_receipts() {
        let i = index();
        let reference = &i.attempts[0];
        let mut record = static_record(i.manifest.conditions[0].lab.clone(), reference.seed);
        record.owner_ticks_alive += 1;
        let a = Archive {
            index: i.clone(),
            attempts: vec![Attempt {
                start: start(&i, reference),
                frames: record.frames.clone(),
                outcome: Some(OutcomeReceipt {
                    condition: reference.condition.clone(),
                    seed: reference.seed,
                    elapsed_seconds: 0.0,
                    timing_boundary: TIMING.into(),
                    outcome: AttemptOutcome::Complete(record),
                }),
                interrupted_tail: None,
            }],
        };
        assert!(format!("{:?}", census(&a)).contains("invalid: 1"));
    }
    #[test]
    fn deception_preflight_binds_raw_protocol_bytes_and_clean_source() {
        let dir = Temp::new();
        let root = &dir.0;
        git(root, &["init", "-q"]).unwrap();
        fs::create_dir_all(root.join(PROTOCOL).parent().unwrap()).unwrap();
        fs::write(root.join(PROTOCOL), b"candidate protocol\n").unwrap();
        fs::write(root.join("tracked.txt"), b"original").unwrap();
        git(root, &["add", "."]).unwrap();
        git(
            root,
            &[
                "-c",
                "user.name=Static Test",
                "-c",
                "user.email=static@example.invalid",
                "commit",
                "-qm",
                "synthetic protocol",
            ],
        )
        .unwrap();
        let revision = git_text(root, &["rev-parse", "HEAD"]).unwrap();
        let p = preflight(root, &revision, &root.join("fresh")).unwrap();
        assert_eq!(p.protocol.sha256, hash(b"candidate protocol\n"));
        fs::write(root.join("tracked.txt"), b"dirty").unwrap();
        assert!(preflight(root, &revision, &root.join("fresh"))
            .unwrap_err()
            .contains("tracked source tree must be clean"));
        fs::write(root.join("tracked.txt"), b"original").unwrap();
        assert!(preflight(root, "HEAD", &root.join("fresh")).is_err());
        fs::write(root.join(PROTOCOL), b"candidate protocol\n\n").unwrap();
        assert!(preflight(root, &revision, &root.join("fresh"))
            .unwrap_err()
            .contains("raw committed/current"));
        fs::write(root.join(PROTOCOL), b"candidate protocol\n").unwrap();
        fs::create_dir(root.join("exists")).unwrap();
        assert!(preflight(root, &revision, &root.join("exists")).is_err());
    }
    #[test]
    fn deception_saved_validation_rejects_biological_mutations() {
        let r = static_record(manifest().conditions[0].lab.clone(), 20001);
        validate_episode(&r).unwrap();
        for variant in 0..11 {
            let mut bad = r.clone();
            match variant {
                0 => bad.requested_ticks = 65,
                1 => bad.frames[0].roles[0].holdings = 45.0,
                2 => bad.frames[1].actions[0].metabolic_consumed = 0.0,
                3 => bad.frames[9].actions[0].phase = "wrong".into(),
                4 => bad.frames[1].observations.clear(),
                5 => bad.frames[0].stocks[0].amount = f64::NAN,
                6 => bad.cohorts.as_mut().unwrap().unlabelled_carried = f64::NAN,
                7 => bad.owner_ticks_alive += 1,
                8 => bad.frames[1].roles[0].pos.x = 0,
                9 => bad.frames[2].roles[0].sender.as_mut().unwrap().stage = Stage::Display,
                _ => {
                    bad.frames
                        .iter_mut()
                        .find(|f| !f.choices.is_empty())
                        .unwrap()
                        .choices[0]
                        .actual_value = 123.0
                }
            };
            assert!(validate_episode(&bad).is_err(), "variant {variant}");
        }
    }
    // Hand-written arithmetic and DTO construction only; never constructs World or RNG.
    pub fn static_record(lab: LabConfig, seed: u64) -> EpisodeRecord {
        let p = |x, y| Pos::new(if lab.mirrored { 8 - x } else { x }, y);
        let source = p(3, 3).y * 9 + p(3, 3).x;
        let mut roles = vec![
            RoleRecord {
                id: 1,
                pos: p(3, 3),
                holdings: 44.0,
                caches: BTreeMap::new(),
                sender: Some(SenderState::default()),
                seen: vec![],
            },
            RoleRecord {
                id: 2,
                pos: p(3, 6),
                holdings: 96.0,
                caches: BTreeMap::new(),
                sender: None,
                seen: vec![],
            },
        ];
        let mut stocks = (0..81)
            .map(|site| StockRecord {
                site,
                amount: if [p(2, 2), p(2, 3)].iter().any(|q| q.y * 9 + q.x == site) {
                    4.0
                } else {
                    0.0
                },
            })
            .collect::<Vec<_>>();
        let mut frames = vec![FrameRecord {
            tick: 0,
            fingerprint: "0000000000000000".into(),
            roles: roles.clone(),
            actions: vec![],
            observations: vec![],
            choices: vec![],
            deaths: vec![],
            restrictions: BTreeMap::new(),
            stocks: stocks.clone(),
        }];
        let mut ledger = Ledger::new(1, 44.0);
        let mut restrictions = BTreeMap::new();
        let mut alive = 0;
        let mut recovered = false;
        let mut cache = BTreeMap::<u32, f64>::new();
        for tick in 0..64 {
            let mut actions = vec![];
            let mut observations = vec![];
            let mut choices = vec![];
            let mut deaths = vec![];
            let mut positions = roles
                .iter()
                .map(|r| (r.id, r.pos))
                .collect::<BTreeMap<_, _>>();
            for role in &mut roles {
                let owner = role.id == 1;
                alive += u64::from(owner);
                let departure = role.sender.as_ref().is_some_and(|s| s.pending_departure);
                let (phase, action, target) = scheduled(&lab, tick, role.id, departure);
                if let Some(state) = &mut role.sender {
                    state.pending_departure = false;
                }
                let mut a = ActionRecord {
                    actor: role.id,
                    phase: phase.into(),
                    action: action.into(),
                    target,
                    metabolic_demand: 1.0,
                    metabolic_consumed: 1.0,
                    ..Default::default()
                };
                if let Some(target) = target {
                    role.pos = target;
                    a.walk_outcome = Some("arrived".into());
                    a.source_recovered = recovered;
                }
                if action == "ordinary" {
                    let candidates =
                        ordinary_candidates(role, &positions, &stocks, &cache, &role.seen);
                    let chosen = candidates
                        .iter()
                        .max_by(|a, b| {
                            a.2.total_cmp(&b.2)
                                .then_with(|| b.1.cmp(&a.1))
                                .then_with(|| site(b.0).cmp(&site(a.0)))
                        })
                        .unwrap();
                    let target = chosen.0;
                    let remembered_value = chosen.2;
                    let environment = stocks[site(target) as usize].amount;
                    let own = if owner && role.holdings < 4.0 {
                        *cache.get(&site(target)).unwrap_or(&0.0)
                    } else {
                        0.0
                    };
                    let seen = if !owner && role.seen.iter().any(|e| e.site == site(target)) {
                        cache
                            .get(&site(target))
                            .copied()
                            .unwrap_or(0.0)
                            .min(128.0 - role.holdings)
                    } else {
                        0.0
                    };
                    let after = ordinary_steps(role, &positions, target)[0];
                    let inspected = cache.get(&site(after)).copied().unwrap_or(0.0);
                    let saw = !owner && role.seen.iter().any(|e| e.site == site(after));
                    let raid = if saw {
                        inspected.min(128.0 - role.holdings)
                    } else {
                        0.0
                    };
                    choices.push(ChoiceRecord {
                        actor: role.id,
                        target,
                        remembered_value,
                        actual_value: environment.max(own).max(seen),
                        arrived: after == target,
                        raid_amount: raid,
                        wasted: saw && inspected == 0.0,
                        inspection: Some(after),
                        inspected_stock: Some(inspected),
                        target_occupant: positions
                            .iter()
                            .find(|(id, q)| **id != role.id && **q == target)
                            .map(|(&id, _)| id),
                        source_recovered: recovered,
                    });
                    role.pos = after;
                    a.target = Some(target);
                    if !owner {
                        role.seen.retain(|e| e.site != site(after));
                    }
                    if raid > 0.0 {
                        ledger.pilfer(site(after), raid).unwrap();
                        *cache.get_mut(&site(after)).unwrap() -= raid;
                        role.holdings += raid;
                    }
                }
                a.pos = Some(role.pos);
                if owner && tick == 0 {
                    a.buried = 12.0;
                    cache.insert(source, 12.0);
                    role.holdings -= 12.0;
                    ledger.prepare(source, 12.0).unwrap();
                    observations.push(ObservedRecord {
                        receiver: 2,
                        public: sugarscape_core::minds::deception::observation::Observation {
                            actor: 1,
                            site: source,
                            tick: 0,
                            signal: Signal::VisibleTransfer { amount: 12.0 },
                        },
                        actual_transfer: 12.0,
                        actual_stock: 12.0,
                    });
                }
                if owner && tick == 13 && lab.sender != SenderPolicy::Ordinary {
                    a.effort = lab.effort_cost;
                    role.holdings -= a.effort;
                    ledger.outflow(a.effort, Outflow::ActionCost).unwrap();
                    role.sender.as_mut().unwrap().attempted = true;
                    if lab.sender == SenderPolicy::Sham && lab.display_seen {
                        observations.push(ObservedRecord {
                            receiver: 2,
                            public: sugarscape_core::minds::deception::observation::Observation {
                                actor: 1,
                                site: role.pos.y * 9 + role.pos.x,
                                tick,
                                signal: match lab.view {
                                    View::Ambiguous => Signal::Cue {
                                        nominal_amount: 12.0,
                                    },
                                    View::Clear => Signal::VisibleTransfer { amount: 0.0 },
                                },
                            },
                            actual_transfer: 0.0,
                            actual_stock: 0.0,
                        });
                    }
                }
                if matches!(action, "walk_and_gather" | "ordinary" | "departure") {
                    let stock = &mut stocks[site(role.pos) as usize];
                    let cached = cache.get(&site(role.pos)).copied().unwrap_or(0.0);
                    let raid = choices
                        .last()
                        .filter(|c| c.actor == role.id)
                        .map_or(0.0, |c| c.raid_amount);
                    if owner && role.holdings < 4.0 && cached > 0.0 && cached >= stock.amount {
                        a.dug = cached.min(128.0 - role.holdings);
                        role.holdings += a.dug;
                        ledger.withdraw(site(role.pos), a.dug).unwrap();
                        *cache.get_mut(&site(role.pos)).unwrap() -= a.dug;
                        recovered = true;
                        role.sender.as_mut().unwrap().pending_departure = true;
                    } else if raid == 0.0 {
                        a.harvest = stock.amount.min(128.0 - role.holdings);
                        stock.amount -= a.harvest;
                        role.holdings += a.harvest;
                        if owner {
                            ledger.harvest(a.harvest).unwrap();
                        }
                    }
                }
                if owner {
                    role.sender.as_mut().unwrap().stage = match phase {
                        "preparation" => Stage::Preparation,
                        "todisplay" => Stage::ToDisplay,
                        "display" => Stage::Display,
                        "return" => Stage::Return,
                        "departure" => Stage::Departure,
                        _ => Stage::Ordinary,
                    };
                    ledger
                        .outflow(role.holdings.clamp(0.0, 1.0), Outflow::Consumption)
                        .unwrap();
                }
                a.metabolic_consumed = role.holdings.clamp(0.0, 1.0);
                role.holdings -= 1.0;
                if owner
                    && tick
                        < if lab.sender == SenderPolicy::Ordinary {
                            8
                        } else {
                            20
                        }
                    || !owner && tick < 32
                {
                    *restrictions.entry(role.id).or_default() += 1;
                }
                if role.holdings <= 0.0 {
                    deaths.push(DeathRecord {
                        actor: role.id,
                        pos: role.pos,
                        cause: "starvation".into(),
                    });
                    if owner {
                        ledger.lose_owner();
                        cache.clear();
                    }
                    positions.remove(&role.id);
                } else {
                    positions.insert(role.id, role.pos);
                }
                actions.push(a);
            }
            roles.retain(|r| r.holdings > 0.0);
            if let Some(owner) = roles.iter_mut().find(|r| r.id == 1) {
                owner.caches = cache.clone();
            }
            for obs in &observations {
                let amount = match obs.public.signal {
                    Signal::Cue { nominal_amount } => nominal_amount,
                    Signal::VisibleTransfer { amount } => amount,
                };
                if amount > 0.0 {
                    if let Some(receiver) = roles.iter_mut().find(|r| r.id == 2) {
                        receiver.seen.push(SeenRecord {
                            site: obs.public.site,
                            owner: 1,
                            amount,
                            tick,
                        });
                    }
                }
            }
            frames.push(FrameRecord {
                tick: tick + 1,
                fingerprint: "0000000000000000".into(),
                roles: roles.clone(),
                actions,
                observations,
                choices,
                deaths,
                restrictions: restrictions.clone(),
                stocks: stocks.clone(),
            });
        }
        EpisodeRecord {
            schema: SCHEMA.into(),
            lab,
            seed,
            requested_ticks: 64,
            completed_ticks: 64,
            owner_ticks_alive: alive,
            owner_alive: false,
            frames,
            fixture_errors: vec![],
            ledger_errors: vec![],
            thief_transferred: Some(ledger.cohorts.values().map(|c| c.transferred).sum()),
            cohorts: Some(ledger),
            diagnostics_enabled: true,
            lineage_unavailable_reason: None,
        }
    }
}
