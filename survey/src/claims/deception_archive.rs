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
    serde_json::to_vec_pretty(value).map_err(|e| e.to_string())
}
pub fn validate_revision(value: &str) -> Result<(), String> {
    if value.len() != 40
        || !value
            .bytes()
            .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
    {
        return Err("revision must be a full lowercase 40-hex commit".into());
    }
    Ok(())
}
pub fn new_directory(path: &Path) -> Result<(), String> {
    reject_symlink_ancestors(path)?;
    fs::create_dir(path).map_err(|e| format!("new directory {}: {e}", path.display()))?;
    sync_parent(path)
}
fn sync_parent(path: &Path) -> Result<(), String> {
    fs::File::open(
        path.parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or(Path::new(".")),
    )
    .and_then(|f| f.sync_all())
    .map_err(|e| format!("sync parent {}: {e}", path.display()))
}
pub fn write_new(path: &Path, bytes: &[u8]) -> Result<(), String> {
    reject_symlink_ancestors(path)?;
    let mut f = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|e| format!("create {}: {e}", path.display()))?;
    f.write_all(bytes)
        .and_then(|_| f.sync_all())
        .map_err(|e| format!("write {}: {e}", path.display()))?;
    sync_parent(path)
}
fn reject_symlink_ancestors(path: &Path) -> Result<(), String> {
    let mut current = PathBuf::new();
    for component in path.components() {
        current.push(component);
        match fs::symlink_metadata(&current) {
            Ok(m) if m.file_type().is_symlink() => {
                return Err(format!("symlink path {}", current.display()))
            }
            Ok(_) => {}
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => return Err(e.to_string()),
        }
    }
    Ok(())
}
fn confined(root: &Path, relative: &str) -> Result<PathBuf, String> {
    if relative.is_empty()
        || Path::new(relative)
            .components()
            .any(|c| !matches!(c, Component::Normal(_)))
    {
        return Err("archive reference must be relative without traversal".into());
    }
    let p = root.join(relative);
    reject_symlink_ancestors(&p)?;
    Ok(p)
}
fn read<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<T, String> {
    serde_json::from_slice(&fs::read(path).map_err(|e| format!("{}: {e}", path.display()))?)
        .map_err(|e| format!("{}: {e}", path.display()))
}
fn exists(path: &Path) -> Result<bool, String> {
    match fs::symlink_metadata(path) {
        Ok(m) => {
            if !m.is_file() {
                return Err(format!("expected regular file {}", path.display()));
            }
            Ok(true)
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(e) => Err(e.to_string()),
    }
}
fn refs(m: &Manifest) -> Vec<AttemptRef> {
    m.conditions
        .iter()
        .enumerate()
        .flat_map(|(ordinal, c)| {
            m.seeds.iter().map(move |&seed| {
                let p = format!("attempts/{ordinal:03}-{seed}");
                AttemptRef {
                    condition: c.id.clone(),
                    seed,
                    start: format!("{p}/start.json"),
                    frames: format!("{p}/frames.jsonl"),
                    outcome: format!("{p}/outcome.json"),
                }
            })
        })
        .collect()
}
fn git(root: &Path, args: &[&str]) -> Result<Vec<u8>, String> {
    let r = Command::new("git")
        .args(args)
        .current_dir(root)
        .output()
        .map_err(|e| e.to_string())?;
    if !r.status.success() {
        return Err(format!(
            "git {args:?}: {}: {}",
            r.status,
            String::from_utf8_lossy(&r.stderr)
        ));
    }
    Ok(r.stdout)
}
fn git_text(root: &Path, args: &[&str]) -> Result<String, String> {
    String::from_utf8(git(root, args)?)
        .map(|s| s.trim().into())
        .map_err(|e| e.to_string())
}
fn file_identity(path: &Path, name: String, mode: String) -> Result<FileIdentity, String> {
    let bytes = fs::read(path).map_err(|e| e.to_string())?;
    Ok(FileIdentity {
        path: name,
        sha256: hash(&bytes),
        bytes: bytes.len() as u64,
        mode,
    })
}
fn preflight(root: &Path, revision: &str, out: &Path) -> Result<Provenance, String> {
    validate_revision(revision)?;
    if git_text(
        root,
        &["rev-parse", "--verify", &format!("{revision}^{{commit}}")],
    )? != revision
    {
        return Err("protocol revision does not name exact commit".into());
    }
    let committed = git(root, &["show", &format!("{revision}:{PROTOCOL}")])?;
    let current = fs::read(root.join(PROTOCOL)).map_err(|e| e.to_string())?;
    if committed != current {
        return Err("raw committed/current protocol bytes differ".into());
    }
    if !git(root, &["status", "--porcelain", "--untracked-files=no"])?.is_empty() {
        return Err("execution requires clean tracked tree".into());
    }
    reject_symlink_ancestors(out)?;
    if fs::symlink_metadata(out).is_ok() {
        return Err("output destination exists".into());
    }
    let source_revision = git_text(root, &["rev-parse", "HEAD"])?;
    let tree = git(root, &["ls-tree", "-r", "-z", "HEAD"])?;
    let mut source_inventory = vec![];
    for entry in tree.split(|b| *b == 0).filter(|s| !s.is_empty()) {
        let e = std::str::from_utf8(entry).map_err(|e| e.to_string())?;
        let (meta, path) = e.split_once('\t').ok_or("invalid git tree")?;
        let mode = meta.split_whitespace().next().ok_or("missing tree mode")?;
        if mode != "100644" && mode != "100755" {
            return Err(format!("unsupported tracked entry {path} mode {mode}"));
        }
        let identity = file_identity(&root.join(path), path.into(), mode.into())?;
        if hash(&git(root, &["show", &format!("HEAD:{path}")])?) != identity.sha256 {
            return Err(format!("source bytes differ at {path}"));
        }
        source_inventory.push(identity);
    }
    let protocol = source_inventory
        .iter()
        .find(|f| f.path == PROTOCOL)
        .cloned()
        .ok_or("protocol missing from source inventory")?;
    let binary_path = std::env::current_exe().map_err(|e| e.to_string())?;
    #[cfg(unix)]
    let binary_mode = {
        use std::os::unix::fs::PermissionsExt;
        format!(
            "{:06o}",
            fs::metadata(&binary_path)
                .map_err(|e| e.to_string())?
                .permissions()
                .mode()
        )
    };
    #[cfg(not(unix))]
    let binary_mode = "100755".to_string();
    let binary = file_identity(&binary_path, binary_path.display().to_string(), binary_mode)?;
    Ok(Provenance {
        source_revision,
        protocol_revision: revision.into(),
        binary,
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
    let valid_hash = |s: &str| {
        s.len() == 64
            && s.bytes()
                .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
    };
    let mut names = BTreeSet::new();
    if p.source_inventory.is_empty()
        || p.source_inventory.iter().any(|f| {
            !names.insert(&f.path)
                || !valid_hash(&f.sha256)
                || !["100644", "100755"].contains(&f.mode.as_str())
        })
        || !valid_hash(&p.binary.sha256)
        || p.binary.bytes == 0
        || u32::from_str_radix(&p.binary.mode, 8).map_or(true, |mode| {
            mode & 0o170000 != 0o100000 || mode & 0o111 == 0
        })
        || p.source_sha256 != hash(&json(&p.source_inventory)?)
        || p.manifest_sha256 != hash(&json(&manifest())?)
        || p.protocol.path != PROTOCOL
        || p.protocol.sha256 != hash(p.protocol_bytes.as_bytes())
        || p.protocol.bytes != p.protocol_bytes.len() as u64
        || !p.source_inventory.contains(&p.protocol)
    {
        return Err("invalid source/protocol/binary/manifest identity".into());
    }
    Ok(())
}
fn validate_index(index: &Index) -> Result<(), String> {
    if index.schema != SCHEMA || index.manifest != manifest() || index.attempts != refs(&manifest())
    {
        return Err("index differs from canonical 96 × 40 matrix/config/budget".into());
    }
    validate_provenance(&index.provenance)
}
pub fn census(a: &Archive) -> Census {
    let mut c = Census {
        planned: a.index.attempts.len(),
        unstarted: a.index.attempts.len().saturating_sub(a.attempts.len()),
        attempted: a.attempts.len(),
        ..Default::default()
    };
    for attempt in &a.attempts {
        match attempt.outcome.as_ref().map(|r| &r.outcome) {
            None => c.pending += 1,
            Some(AttemptOutcome::Failed(f)) => {
                c.failed += 1;
                c.partial += usize::from(f.partial.is_some());
                c.invalid += usize::from(f.partial.as_ref().is_some_and(|r| {
                    r.completed_ticks == r.requested_ticks && validate_episode(r).is_err()
                }));
            }
            Some(AttemptOutcome::Complete(r)) => {
                c.complete += 1;
                c.invalid += usize::from(validate_episode(r).is_err());
                c.unavailable_lineage += usize::from(r.thief_transferred.is_none());
            }
        }
    }
    c
}
pub fn load(path: &Path) -> Result<Archive, String> {
    load_inner(path).map_err(|error| {
        let diagnostic = receipt_census(path)
            .map_or_else(|e| format!("unavailable ({e})"), |c| format!("{c:?}"));
        format!("{error}; execution census (receipt states): {diagnostic}")
    })
}
fn receipt_census(path: &Path) -> Result<Census, String> {
    reject_symlink_ancestors(path)?;
    let index: Index = read(path)?;
    if index.attempts != refs(&manifest()) {
        return Err("invalid canonical reference matrix".into());
    }
    let root = path.parent().unwrap_or(Path::new("."));
    let mut c = Census {
        planned: 3840,
        ..Default::default()
    };
    for r in &index.attempts {
        let start_path = confined(root, &r.start)?;
        if !exists(&start_path)? {
            c.unstarted += 1;
            continue;
        }
        c.attempted += 1;
        let mut invalid = read::<AttemptStart>(&start_path).map_or(true, |s| {
            s.schema != SCHEMA
                || s.condition != r.condition
                || s.seed != r.seed
                || s.provenance != index.provenance.identity()
        });
        let outcome = confined(root, &r.outcome)?;
        if !exists(&outcome)? {
            c.pending += 1;
        } else {
            match read::<OutcomeReceipt>(&outcome) {
                Err(_) => {
                    c.pending += 1;
                    invalid = true;
                }
                Ok(o) => {
                    invalid |= o.condition != r.condition
                        || o.seed != r.seed
                        || !o.elapsed_seconds.is_finite()
                        || o.elapsed_seconds < 0.0
                        || o.timing_boundary != TIMING;
                    match o.outcome {
                        AttemptOutcome::Complete(record) => {
                            c.complete += 1;
                            invalid |= validate_episode(&record).is_err();
                            c.unavailable_lineage +=
                                usize::from(record.thief_transferred.is_none());
                        }
                        AttemptOutcome::Failed(f) => {
                            c.failed += 1;
                            c.partial += usize::from(f.partial.is_some());
                            invalid |= f.partial.as_ref().is_some_and(|r| {
                                r.completed_ticks == r.requested_ticks
                                    && validate_episode(r).is_err()
                            });
                        }
                    }
                }
            }
        }
        c.invalid += usize::from(invalid);
    }
    Ok(c)
}
fn load_inner(path: &Path) -> Result<Archive, String> {
    reject_symlink_ancestors(path)?;
    let index: Index = read(path)?;
    validate_index(&index)?;
    let root = path.parent().unwrap_or(Path::new("."));
    let mut attempts = vec![];
    for r in &index.attempts {
        let start = confined(root, &r.start)?;
        let frames = confined(root, &r.frames)?;
        let outcome = confined(root, &r.outcome)?;
        if !exists(&start)? {
            if exists(&frames)? || exists(&outcome)? {
                return Err("attempt data without durable start".into());
            }
            continue;
        }
        let start: AttemptStart = read(&start)?;
        if start.condition != r.condition || start.seed != r.seed {
            return Err(format!("attempt slot identity mismatch at {}", r.start));
        }
        let outcome = if exists(&outcome)? {
            Some(read::<OutcomeReceipt>(&outcome)?)
        } else {
            None
        };
        let mut saved = vec![];
        let mut tail = None;
        if exists(&frames)? {
            let bytes = fs::read(&frames).map_err(|e| e.to_string())?;
            let mut lines = bytes.split_inclusive(|b| *b == b'\n').peekable();
            while let Some(line) = lines.next() {
                if !line.ends_with(b"\n") {
                    if lines.peek().is_some()
                        || outcome
                            .as_ref()
                            .is_some_and(|o| matches!(o.outcome, AttemptOutcome::Complete(_)))
                    {
                        return Err("truncated finalized frame stream".into());
                    }
                    tail = Some(String::from_utf8_lossy(line).into());
                    break;
                }
                saved.push(
                    serde_json::from_slice::<FrameRecord>(line)
                        .map_err(|e| format!("invalid durable frame: {e}"))?,
                );
            }
        } else if outcome.is_some() {
            return Err("outcome without frame stream".into());
        }
        attempts.push(Attempt {
            start,
            frames: saved,
            outcome,
            interrupted_tail: tail,
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
        let s = &attempt.start;
        if s.schema != SCHEMA
            || s.provenance != a.index.provenance.identity()
            || !keys.insert((&s.condition, s.seed))
            || !a
                .index
                .attempts
                .iter()
                .any(|r| r.condition == s.condition && r.seed == s.seed)
        {
            return Err("duplicate/mismatched attempt identity or provenance".into());
        }
        if attempt
            .frames
            .iter()
            .enumerate()
            .any(|(i, f)| f.tick != i as u64)
        {
            return Err("noncontiguous durable frames".into());
        }
        if let Some(o) = &attempt.outcome {
            if o.condition != s.condition
                || o.seed != s.seed
                || !o.elapsed_seconds.is_finite()
                || o.elapsed_seconds < 0.0
                || o.timing_boundary != TIMING
                || (attempt.interrupted_tail.is_some()
                    && matches!(o.outcome, AttemptOutcome::Complete(_)))
            {
                return Err("outcome identity/timing mismatch".into());
            }
            let record = match &o.outcome {
                AttemptOutcome::Complete(r) => Some(r),
                AttemptOutcome::Failed(f) => {
                    if f.message.trim().is_empty() {
                        return Err("empty failure reason".into());
                    }
                    f.partial.as_ref()
                }
            };
            if let Some(r) = record {
                if r.seed != s.seed
                    || sugarscape_core::minds::deception::condition_id(&r.lab) != s.condition
                    || r.schema != SCHEMA
                    || !r.frames.starts_with(&attempt.frames)
                {
                    return Err("outcome does not match saved identity/frame prefix".into());
                }
                if matches!(o.outcome, AttemptOutcome::Complete(_)) && r.frames != attempt.frames {
                    return Err("complete outcome lacks durable frames".into());
                }
            }
        }
    }
    let c = census(a);
    if a.index.census.as_ref().is_some_and(|saved| saved != &c)
        || a.index.completed && (c.unstarted > 0 || c.pending > 0)
    {
        return Err(format!("inaccurate final execution census: {c:?}"));
    }
    Ok(())
}
pub fn validate_archive(a: &Archive) -> Result<(), String> {
    let c = census(a);
    let result = (|| {
        validate_structure(a)?;
        if !a.index.completed
            || c.complete != 3840
            || c.attempted != 3840
            || c.failed != 0
            || c.pending != 0
        {
            return Err(
                "analysis requires all 3,840 complete attempts; no survivor pairing".into(),
            );
        }
        for attempt in &a.attempts {
            if let Some(OutcomeReceipt {
                outcome: AttemptOutcome::Complete(r),
                ..
            }) = &attempt.outcome
            {
                validate_episode(r).map_err(|e| {
                    format!(
                        "{} seed {}: {e}",
                        attempt.start.condition, attempt.start.seed
                    )
                })?;
            }
        }
        Ok(())
    })();
    result.map_err(|e: String| format!("{e}; execution census: {c:?}"))
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
            .map_err(|e| e.to_string())
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
    new_directory(start_path.parent().ok_or("attempt parent")?)?;
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
    let frames_path = confined(root, &r.frames)?;
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&frames_path)
        .map_err(|e| e.to_string())?;
    file.sync_all().map_err(|e| e.to_string())?;
    sync_parent(&frames_path)?;
    let begin = Instant::now();
    let mut io_error = None;
    let result = runner(condition.lab.clone(), r.seed, &mut |frame| {
        let result = (|| {
            let mut bytes = serde_json::to_vec(frame).map_err(|e| e.to_string())?;
            bytes.push(b'\n');
            writer(&mut file, &bytes)
                .map_err(|e| format!("durable frame I/O {}: {e}", frames_path.display()))
        })();
        if let Err(e) = &result {
            io_error = Some(e.clone());
        }
        result
    });
    let elapsed_seconds = begin.elapsed().as_secs_f64();
    let outcome = match result {
        Ok(record) => match validate_episode(&record) {
            Ok(()) => AttemptOutcome::Complete(record),
            Err(message) => AttemptOutcome::Failed(EpisodeFailure {
                message,
                partial: Some(record),
            }),
        },
        Err(f) => AttemptOutcome::Failed(f),
    };
    let receipt = OutcomeReceipt {
        condition: r.condition.clone(),
        seed: r.seed,
        elapsed_seconds,
        timing_boundary: TIMING.into(),
        outcome,
    };
    let saved = write_new(&confined(root, &r.outcome)?, &json(&receipt)?);
    match (io_error, saved) {
        (Some(e), Ok(())) => Err(e),
        (Some(e), Err(s)) => Err(format!("{e}; saving failure receipt also failed: {s}")),
        (None, s) => s,
    }
}
// The approved core failure DTO deliberately owns its known partial record.
#[allow(clippy::result_large_err)]
pub fn run(revision: &str, out: &Path) -> Result<(), String> {
    let p = preflight(
        Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/..")),
        revision,
        out,
    )?;
    new_directory(out)?;
    new_directory(&out.join("attempts"))?;
    let m = manifest();
    let mut index = Index {
        schema: SCHEMA.into(),
        provenance: p,
        attempts: refs(&m),
        manifest: m,
        completed: false,
        census: None,
    };
    write_new(&out.join("index.incomplete.json"), &json(&index)?)?;
    for r in &index.attempts {
        let c = index
            .manifest
            .conditions
            .iter()
            .find(|c| c.id == r.condition)
            .ok_or("missing canonical condition")?;
        if let Err(e) = execute_attempt(out, r, &index.provenance, c, |lab, seed, sink| {
            sugarscape_core::minds::deception::run_episode_with_sink(lab, seed, true, sink)
        }) {
            let recovery = (|| {
                let a = load(&out.join("index.incomplete.json"))?;
                index.census = Some(census(&a));
                write_new(&out.join("index.error.json"), &json(&index)?)?;
                write_new(&out.join("execution-error.txt"), e.as_bytes())
            })();
            return Err(match recovery {
                Ok(()) => e,
                Err(other) => format!("{e}; error census persistence failed: {other}"),
            });
        }
    }
    let mut archive = load(&out.join("index.incomplete.json"))?;
    index.completed = true;
    index.census = Some(census(&archive));
    archive.index = index.clone();
    write_new(&out.join("index.json"), &json(&index)?)?;
    let analysis = super::deception_report::analyze(&archive)?;
    super::deception_report::save_files(&analysis, out)
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
            let p =
                PathBuf::from("/private/tmp/sugarscape-minds-p4-20261007-vb9tv64o").join(format!(
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
                && e.contains("pending: 1"),
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
        for tick in 0..64 {
            let mut actions = vec![];
            let mut observations = vec![];
            let mut choices = vec![];
            let mut deaths = vec![];
            for role in &mut roles {
                let owner = role.id == 1;
                alive += u64::from(owner);
                let (phase, action, target) = scheduled(&lab, tick, role.id, false);
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
                }
                if owner && lab.sender == SenderPolicy::Ordinary && tick == 8 {
                    role.pos = p(3, 2);
                }
                a.pos = Some(role.pos);
                if owner && tick == 0 {
                    a.buried = 12.0;
                    role.caches.insert(source, 12.0);
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
                if a.action == "walk_and_gather" || a.action == "ordinary" {
                    let stock = &mut stocks[(role.pos.y * 9 + role.pos.x) as usize];
                    a.harvest = stock.amount;
                    stock.amount = 0.0;
                    role.holdings += a.harvest;
                    if owner {
                        ledger.harvest(a.harvest).unwrap();
                    }
                }
                if a.action == "ordinary" {
                    a.target = Some(role.pos);
                    let value = a.harvest.max(if owner && role.holdings - a.harvest < 4.0 {
                        *role
                            .caches
                            .get(&(role.pos.y * 9 + role.pos.x))
                            .unwrap_or(&0.0)
                    } else {
                        0.0
                    });
                    choices.push(ChoiceRecord {
                        actor: role.id,
                        target: role.pos,
                        remembered_value: value,
                        actual_value: value,
                        arrived: true,
                        raid_amount: 0.0,
                        wasted: false,
                        inspection: Some(role.pos),
                        inspected_stock: Some(
                            *role
                                .caches
                                .get(&(role.pos.y * 9 + role.pos.x))
                                .unwrap_or(&0.0),
                        ),
                        target_occupant: None,
                        source_recovered: false,
                    });
                }
                if owner {
                    role.sender.as_mut().unwrap().stage = match phase {
                        "preparation" => Stage::Preparation,
                        "todisplay" => Stage::ToDisplay,
                        "display" => Stage::Display,
                        "return" => Stage::Return,
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
                    }
                }
                actions.push(a);
            }
            roles.retain(|r| r.holdings > 0.0);
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
            cohorts: Some(ledger),
            thief_transferred: Some(0.0),
            diagnostics_enabled: true,
            lineage_unavailable_reason: None,
        }
    }
}
