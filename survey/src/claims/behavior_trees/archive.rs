//! Single writer; a durable start and empty stream precede every constructor.
use super::{
    io,
    manifest::{self, Manifest},
    validate, wire,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    fs,
    io::Write,
    path::Path,
    process::Command,
    time::{Instant, SystemTime, UNIX_EPOCH},
};
use sugarscape_core::minds::behavior_tree::{
    records::Frame, runner::run_episode_to, EpisodeFailure, EpisodeRecord, RunOptions,
};
#[path = "../../../build_support/bt_source_identity.rs"]
pub(super) mod inputs;
include!(concat!(env!("OUT_DIR"), "/bt_compiled_inputs.rs"));
pub(super) fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
macro_rules! wire_struct{($name:ident{$($field:ident:$ty:ty),*$(,)?})=>{#[derive(Clone,Debug,PartialEq,Serialize,Deserialize)]#[serde(deny_unknown_fields)]pub struct $name{$(pub $field:$ty),*}}}
wire_struct!(FileIdentity {
    path: String,
    sha256: String,
    bytes: u64,
    mode: String
});
wire_struct!(Provenance{source_root:String,source_revision:String,protocol:FileIdentity,binary:FileIdentity,source_inventory:Vec<FileIdentity>,source_sha256:String,compiled_inputs_sha256:String,manifest_sha256:String,toolchain:String,build_flags:Vec<(String,String)>,compiled_input_paths:Vec<String>,selector_scope:String});
wire_struct!(AttemptIdentity {
    source_revision: String,
    protocol_sha256: String,
    binary_sha256: String,
    source_sha256: String,
    compiled_inputs_sha256: String,
    manifest_sha256: String
});
wire_struct!(AttemptStart {
    condition: String,
    seed: u64,
    identity: AttemptIdentity,
    started_unix_ms: u128
});
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum Outcome {
    Complete(EpisodeRecord),
    Failed(EpisodeFailure),
}
wire_struct!(OutcomeReceipt {
    condition: String,
    seed: u64,
    outcome: Outcome,
    elapsed_seconds: f64,
    timing_boundary: String
});
wire_struct!(AttemptRef {
    condition: String,
    seed: u64,
    start: String,
    frames: String,
    outcome: String
});
wire_struct!(Census {
    planned: usize,
    attempted: usize,
    complete: usize,
    failed: usize,
    invalid: usize,
    partial: usize,
    pending: usize,
    unstarted: usize
});
wire_struct!(Index{schema:String,provenance:Provenance,manifest:Manifest,completed:bool,attempts:Vec<AttemptRef>,census:Option<Census>});
#[derive(Clone, Debug)]
pub struct Attempt {
    pub start: AttemptStart,
    pub frames: Vec<Frame>,
    pub outcome: Option<OutcomeReceipt>,
    pub torn_tail: Option<String>,
}
#[derive(Clone, Debug)]
pub struct Archive {
    pub index: Index,
    pub attempts: Vec<Attempt>,
}
const SCHEMA: &str = "minds-behavior-tree-archive-v1";
const TIMING:&str="native wall seconds from immediately before episode constructor through final frame sink; excludes outcome encoding/sync; includes frame encoding/sync";
fn git(root: &Path, args: &[&str]) -> Result<Vec<u8>, String> {
    let out = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(args)
        .output()
        .map_err(|e| e.to_string())?;
    if !out.status.success() {
        return Err(String::from_utf8_lossy(&out.stderr).into());
    }
    Ok(out.stdout)
}
pub(super) fn identity(path: &Path, name: String) -> Result<FileIdentity, String> {
    use std::{io::Read, os::unix::fs::PermissionsExt};
    io::check_path(path)?;
    let metadata = fs::symlink_metadata(path).map_err(|e| e.to_string())?;
    if !metadata.is_file() {
        return Err(format!("identity needs regular file: {}", path.display()));
    }
    let mut file = fs::File::open(path).map_err(|e| e.to_string())?;
    let mut digest = Sha256::new();
    let mut buffer = [0u8; 65536];
    let mut bytes = 0;
    loop {
        let n = file.read(&mut buffer).map_err(|e| e.to_string())?;
        if n == 0 {
            break;
        }
        digest.update(&buffer[..n]);
        bytes += n as u64;
    }
    if bytes != metadata.len() {
        return Err("identity file length changed while reading".into());
    }
    Ok(FileIdentity {
        path: name,
        sha256: format!("{:x}", digest.finalize()),
        bytes,
        mode: format!("{:o}", metadata.permissions().mode()),
    })
}

fn attempt_identity(p: &Provenance) -> AttemptIdentity {
    AttemptIdentity {
        source_revision: p.source_revision.clone(),
        protocol_sha256: p.protocol.sha256.clone(),
        binary_sha256: p.binary.sha256.clone(),
        source_sha256: p.source_sha256.clone(),
        compiled_inputs_sha256: p.compiled_inputs_sha256.clone(),
        manifest_sha256: p.manifest_sha256.clone(),
    }
}
pub(super) fn preflight(
    root: &Path,
    exe: &Path,
    revision: &str,
    m: &Manifest,
) -> Result<Provenance, String> {
    if revision.len() != 40 || !revision.bytes().all(|c| c.is_ascii_hexdigit()) {
        return Err("full revision required".into());
    }
    if !COMPILED_BT_PROTOCOL_PRESENT {
        return Err("binary predates committed protocol".into());
    }
    let entries = inputs::read(root)?;
    let current = inputs::fingerprint(entries.clone().into_iter().map(Ok))?;
    if current != COMPILED_BT_INPUTS_SHA256
        || entries.iter().map(|e| e.0.as_str()).collect::<Vec<_>>() != COMPILED_BT_INPUT_PATHS
    {
        return Err("stale binary: actual BT inputs differ from compiled input stamp".into());
    }
    let head = String::from_utf8(git(root, &["rev-parse", "HEAD"])?).map_err(|e| e.to_string())?;
    if head.trim() != revision {
        return Err("revision must equal frozen HEAD".into());
    }
    if !git(root, &["status", "--porcelain", "--untracked-files=all"])?.is_empty() {
        return Err("source must be clean and frozen".into());
    }
    let committed = git(root, &["show", &format!("{revision}:{}", inputs::PROTOCOL)])?;
    if committed != io::read(&root.join(inputs::PROTOCOL))? {
        return Err("protocol bytes differ from commit".into());
    }
    let tracked = git(root, &["ls-files", "-z"])?;
    let mut inventory = vec![];
    for name in tracked.split(|b| *b == 0).filter(|b| !b.is_empty()) {
        let name = std::str::from_utf8(name).map_err(|e| e.to_string())?;
        inventory.push(identity(&root.join(name), name.into())?)
    }
    let source_sha256 = hash(&serde_json::to_vec(&inventory).map_err(|e| e.to_string())?);
    let build_flags = COMPILED_BT_FLAGS
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect();
    Ok(Provenance{source_root:root.canonicalize().map_err(|e|e.to_string())?.to_string_lossy().into(),source_revision:revision.into(),protocol:identity(&root.join(inputs::PROTOCOL),inputs::PROTOCOL.into())?,binary:identity(exe,exe.to_string_lossy().into())?,source_inventory:inventory,source_sha256,compiled_inputs_sha256:current,manifest_sha256:hash(&serde_json::to_vec(m).map_err(|e|e.to_string())?),toolchain:COMPILED_BT_TOOLCHAIN.into(),build_flags,compiled_input_paths:entries.into_iter().map(|e|e.0).collect(),selector_scope:"BT declared source directories and exact files; unrelated assets/dependencies are not claimed compiled; full Git inventory retained separately".into()})
}
pub fn run(revision: &str, out: &Path) -> Result<(), String> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let m = manifest::manifest();
    let p = preflight(root, &exe, revision, &m)?;
    let bound = p.clone();
    collect_with_frames(
        out,
        m,
        p,
        |lab, seed, sink| {
            run_episode_to(
                lab,
                seed,
                RunOptions {
                    diagnostics: true,
                    controller_timing: true,
                },
                sink,
            )
        },
        |path| {
            verify_bound_files(&bound)?;
            let head =
                String::from_utf8(git(root, &["rev-parse", "HEAD"])?).map_err(|e| e.to_string())?;
            if head.trim() != bound.source_revision {
                return Err("bound Git revision changed before constructor".into());
            }
            fs::OpenOptions::new()
                .append(true)
                .open(path)
                .map_err(|e| e.to_string())
        },
    )
}
/// Rehash the original bound executable, protocol, full tracked inventory and
/// compiled closure after the start/stream are durable and before construction.
pub(super) fn verify_bound_files(p: &Provenance) -> Result<(), String> {
    let root = Path::new(&p.source_root);
    for (path, expected) in [
        (Path::new(&p.binary.path).to_path_buf(), &p.binary),
        (root.join(&p.protocol.path), &p.protocol),
    ] {
        if identity(&path, expected.path.clone())? != *expected {
            return Err(format!(
                "bound file changed before constructor: {}",
                expected.path
            ));
        }
    }
    for expected in &p.source_inventory {
        if identity(&io::relative(root, &expected.path)?, expected.path.clone())? != *expected {
            return Err(format!(
                "bound source changed before constructor: {}",
                expected.path
            ));
        }
    }
    let entries = inputs::read(root)?;
    if inputs::fingerprint(entries.clone().into_iter().map(Ok))? != p.compiled_inputs_sha256
        || entries.iter().map(|e| e.0.clone()).collect::<Vec<_>>() != p.compiled_input_paths
    {
        return Err("bound compiled inputs changed before constructor".into());
    }
    Ok(())
}

type Sink<'a> = dyn FnMut(&Frame) -> Result<(), String> + 'a;
#[cfg(test)]
pub(super) fn collect(
    out: &Path,
    m: Manifest,
    p: Provenance,
    mut runner: impl FnMut(
        sugarscape_core::minds::behavior_tree::state::LabConfig,
        u64,
        &mut Sink<'_>,
    ) -> Result<EpisodeRecord, EpisodeFailure>,
) -> Result<(), String> {
    collect_with_frames(out, m, p, &mut runner, |path| {
        fs::OpenOptions::new()
            .append(true)
            .open(path)
            .map_err(|e| e.to_string())
    })
}
pub(super) fn collect_with_frames(
    out: &Path,
    m: Manifest,
    p: Provenance,
    mut runner: impl FnMut(
        sugarscape_core::minds::behavior_tree::state::LabConfig,
        u64,
        &mut Sink<'_>,
    ) -> Result<EpisodeRecord, EpisodeFailure>,
    mut open_frames: impl FnMut(&Path) -> Result<fs::File, String>,
) -> Result<(), String> {
    io::directory(out)?;
    let refs: Vec<_> = m
        .conditions
        .iter()
        .flat_map(|c| {
            m.seeds.iter().map(move |seed| {
                let stem = format!("{}-{seed}", c.id);
                AttemptRef {
                    condition: c.id.clone(),
                    seed: *seed,
                    start: format!("{stem}.start.json"),
                    frames: format!("{stem}.frames.jsonl"),
                    outcome: format!("{stem}.outcome.json"),
                }
            })
        })
        .collect();
    let mut index = Index {
        schema: SCHEMA.into(),
        provenance: p,
        manifest: m,
        completed: false,
        attempts: refs,
        census: None,
    };
    io::json(&out.join("index.json"), &index)?;
    let mut census = Census {
        planned: index.attempts.len(),
        attempted: 0,
        complete: 0,
        failed: 0,
        invalid: 0,
        partial: 0,
        pending: 0,
        unstarted: index.attempts.len(),
    };
    let result = (|| {
        for r in &index.attempts {
            let start = AttemptStart {
                condition: r.condition.clone(),
                seed: r.seed,
                identity: attempt_identity(&index.provenance),
                started_unix_ms: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .map_err(|e| e.to_string())?
                    .as_millis(),
            };
            io::json(&out.join(&r.start), &start)?;
            census.attempted += 1;
            census.unstarted -= 1;
            census.pending += 1;
            let path = out.join(&r.frames);
            io::create(&path, b"")?;
            let mut file = open_frames(&path)?;
            let mut io_error = None;
            let mut count = 0;
            let mut sink = |f: &Frame| {
                let result = io::append(&mut file, f);
                match &result {
                    Ok(()) => count += 1,
                    Err(e) => io_error = Some(e.clone()),
                }
                result
            };
            let lab = index
                .manifest
                .conditions
                .iter()
                .find(|c| c.id == r.condition)
                .unwrap()
                .lab
                .clone();
            let t = Instant::now();
            let result = runner(lab, r.seed, &mut sink);
            let elapsed = t.elapsed().as_secs_f64();
            file.flush().map_err(|e| e.to_string())?;
            if let Some(e) = io_error {
                census.partial += usize::from(count > 0);
                return Err(e);
            }
            let outcome = match result {
                Ok(record) => Outcome::Complete(record),
                Err(failure) => Outcome::Failed(failure),
            };
            census.partial += usize::from(count > 0);
            let receipt = OutcomeReceipt {
                condition: r.condition.clone(),
                seed: r.seed,
                outcome,
                elapsed_seconds: elapsed,
                timing_boundary: TIMING.into(),
            };
            io::json(&out.join(&r.outcome), &receipt)?;
            census.pending -= 1;
            census.partial -= usize::from(count > 0);
            match receipt.outcome {
                Outcome::Complete(_) => census.complete += 1,
                Outcome::Failed(_) => census.failed += 1,
            }
        }
        Ok(())
    })();
    if let Err(error) = result {
        let saved = io::json(&out.join("census.json"), &census);
        return Err(format!(
            "{error}; prefix census {census:?}; census persistence: {saved:?}"
        ));
    }
    index.completed = true;
    index.census = Some(census.clone());
    io::json(&out.join("census.json"), &census)?;
    io::json(&out.join("final-index.json"), &index)
}
pub fn load(path: &Path) -> Result<Archive, String> {
    let archive = load_declared(path, &manifest::manifest())?;
    let p = &archive.index.provenance;
    let root = Path::new(&p.source_root);
    if p.source_revision.len() != 40
        || !p.source_revision.bytes().all(|b| b.is_ascii_hexdigit())
        || p.protocol.path != inputs::PROTOCOL
    {
        return Err("invalid source/protocol identity".into());
    }
    let head = String::from_utf8(git(root, &["rev-parse", "HEAD"])?).map_err(|e| e.to_string())?;
    if head.trim() != p.source_revision
        || !git(root, &["status", "--porcelain", "--untracked-files=no"])?.is_empty()
    {
        return Err("saved source revision is no longer frozen".into());
    }
    let tracked = git(root, &["ls-files", "-z"])?;
    let names = tracked
        .split(|b| *b == 0)
        .filter(|b| !b.is_empty())
        .map(|b| std::str::from_utf8(b).map(str::to_owned))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    if p.source_inventory
        .iter()
        .map(|e| e.path.clone())
        .collect::<Vec<_>>()
        != names
    {
        return Err("saved source inventory is not complete tracked inventory".into());
    }
    if hash(&serde_json::to_vec(&p.source_inventory).map_err(|e| e.to_string())?) != p.source_sha256
    {
        return Err("source inventory digest mismatch".into());
    }
    for entry in &p.source_inventory {
        let path = io::relative(root, &entry.path)?;
        if identity(&path, entry.path.clone())? != *entry {
            return Err(format!("saved source identity changed: {}", entry.path));
        }
    }
    if identity(&root.join(&p.protocol.path), p.protocol.path.clone())? != p.protocol
        || identity(Path::new(&p.binary.path), p.binary.path.clone())? != p.binary
    {
        return Err("saved protocol/binary identity changed".into());
    }
    let entries = inputs::read(root)?;
    if inputs::fingerprint(entries.clone().into_iter().map(Ok))? != p.compiled_inputs_sha256
        || entries.iter().map(|e| e.0.clone()).collect::<Vec<_>>() != p.compiled_input_paths
    {
        return Err("saved compiled-input identity changed".into());
    }
    let committed = git(
        root,
        &[
            "show",
            &format!("{}:{}", p.source_revision, inputs::PROTOCOL),
        ],
    )?;
    if hash(&committed) != p.protocol.sha256 {
        return Err("saved protocol revision mismatch".into());
    }
    Ok(archive)
}
pub(super) fn load_declared(path: &Path, expected: &Manifest) -> Result<Archive, String> {
    let mut index: Index = wire::decode(&io::read(path)?)?;
    if index.schema != SCHEMA || &index.manifest != expected {
        return Err("noncanonical archive manifest/schema".into());
    }
    if hash(&serde_json::to_vec(&index.manifest).map_err(|e| e.to_string())?)
        != index.provenance.manifest_sha256
    {
        return Err("manifest provenance mismatch".into());
    }
    let expected_keys: Vec<_> = expected
        .conditions
        .iter()
        .flat_map(|c| expected.seeds.iter().map(move |s| (c.id.as_str(), *s)))
        .collect();
    if index
        .attempts
        .iter()
        .map(|a| (a.condition.as_str(), a.seed))
        .collect::<Vec<_>>()
        != expected_keys
    {
        return Err("attempt census/order differs from manifest".into());
    }
    let root = path.parent().ok_or("index has no parent")?;
    if index.completed {
        let initial: Index = wire::decode(&io::read(&root.join("index.json"))?)?;
        let mut expected_initial = index.clone();
        expected_initial.completed = false;
        expected_initial.census = None;
        if initial != expected_initial {
            return Err("final index conflicts with immutable initial index".into());
        }
    }

    let mut attempts = vec![];
    let mut census = Census {
        planned: expected_keys.len(),
        attempted: 0,
        complete: 0,
        failed: 0,
        invalid: 0,
        partial: 0,
        pending: 0,
        unstarted: 0,
    };
    let mut paths = std::collections::BTreeSet::new();
    for reference in &index.attempts {
        for p in [&reference.start, &reference.frames, &reference.outcome] {
            if !paths.insert(p) {
                return Err("duplicate archive path".into());
            }
            io::relative(root, p)?;
        }
        let start_path = io::relative(root, &reference.start)?;
        let frames_path = io::relative(root, &reference.frames)?;
        let outcome_path = io::relative(root, &reference.outcome)?;
        if !start_path.exists() {
            if frames_path.exists() || outcome_path.exists() {
                return Err("orphan stream/outcome without start".into());
            }
            census.unstarted += 1;
            continue;
        }
        let start: AttemptStart = wire::decode(&io::read(&start_path)?)?;
        if start.condition != reference.condition
            || start.seed != reference.seed
            || start.identity != attempt_identity(&index.provenance)
        {
            return Err("conflicting start identity".into());
        }
        census.attempted += 1;
        let mut frames = vec![];
        let mut torn_tail = None;
        if frames_path.exists() {
            let bytes = io::read(&frames_path)?;
            for line in bytes.split_inclusive(|b| *b == b'\n') {
                if !line.ends_with(b"\n") {
                    torn_tail = Some(String::from_utf8_lossy(line).into());
                    break;
                }
                let f: Frame = wire::decode(line)?;
                if f.tick != frames.len() as u64 {
                    return Err("conflicting frame order".into());
                }
                frames.push(f)
            }
        }
        let outcome = if outcome_path.exists() {
            Some(wire::decode::<OutcomeReceipt>(&io::read(&outcome_path)?)?)
        } else {
            None
        };
        if let Some(receipt) = &outcome {
            if receipt.condition != start.condition
                || receipt.seed != start.seed
                || !receipt.elapsed_seconds.is_finite()
                || receipt.elapsed_seconds < 0.0
                || receipt.elapsed_seconds > u64::MAX as f64
                || receipt.timing_boundary != TIMING
                || torn_tail.is_some()
            {
                return Err("conflicting outcome envelope/torn completed stream".into());
            }
            let lab = &expected
                .conditions
                .iter()
                .find(|c| c.id == start.condition)
                .unwrap()
                .lab;
            match &receipt.outcome {
                Outcome::Complete(record) => {
                    if record.frames != frames {
                        return Err("outcome differs from durable raw frames".into());
                    }
                    validate::validate_episode(record, lab, start.seed)?;
                    census.complete += 1
                }
                Outcome::Failed(failure) => {
                    if !frames.is_empty() {
                        validate::validate_frames(&frames[..1], lab, start.seed)?;
                    }

                    if failure.message.is_empty()
                        || failure.partial.is_none() && !frames.is_empty()
                        || failure.partial.as_ref().is_some_and(|p| {
                            p.frames != frames || p.lab != *lab || p.seed != start.seed
                        })
                    {
                        return Err("conflicting failure prefix".into());
                    }
                    census.failed += 1
                }
            }
        } else {
            if !frames.is_empty() {
                let lab = &expected
                    .conditions
                    .iter()
                    .find(|c| c.id == start.condition)
                    .unwrap()
                    .lab;
                validate::validate_frames(&frames, lab, start.seed)?;
            }
            census.pending += 1;
            census.partial += usize::from(!frames.is_empty())
        }
        attempts.push(Attempt {
            start,
            frames,
            outcome,
            torn_tail,
        });
    }
    if index.census.as_ref().is_some_and(|c| c != &census) {
        return Err("saved census differs from actual prefix".into());
    }
    if index.completed && (census.pending + census.unstarted > 0) {
        return Err("completed index has unfinished attempts".into());
    }
    index.census = Some(census);
    Ok(Archive { index, attempts })
}
