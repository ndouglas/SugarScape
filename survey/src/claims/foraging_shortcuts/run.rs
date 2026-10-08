//! Explicit native collection with committed protocol, manifest, and executable identity.
use super::{
    archive::{limits, ArchiveWriter, Index},
    io::regular_file,
    manifest::{
        authorize, candidate, condition, expected_keys, manifest_bytes, Condition, Manifest,
    },
    sha256,
    wire::encode_envelope,
    CollectionMode, Provenance,
};
use sha2::{Digest, Sha256};
use std::{
    fs,
    io::Read,
    path::{Path, PathBuf},
    process::Command,
};
use sugarscape_core::foraging::construction as core;

pub(super) const MANIFEST_PATH: &str =
    "docs/superpowers/specs/2026-10-07-foraging-5-draft-manifest.json";

#[derive(Clone, Debug, PartialEq)]
pub(super) struct ExecutionContext {
    pub repo: PathBuf,
    pub executable: PathBuf,
}
#[derive(Clone, Debug, PartialEq)]
pub(super) struct RunRequest {
    pub mode: CollectionMode,
    pub protocol_revision: String,
    pub approval_context: String,
    pub out: PathBuf,
}
pub(super) fn revision(value: &str) -> Result<String, String> {
    if value.len() != 40 || !value.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err("protocol/code revision must be a full 40-hex Git commit".into());
    }
    Ok(value.to_ascii_lowercase())
}
fn git(root: &Path, args: &[&str]) -> Result<Vec<u8>, String> {
    let output = Command::new("git")
        .current_dir(root)
        .args(args)
        .output()
        .map_err(|e| format!("git {args:?} in {}: {e}", root.display()))?;
    if !output.status.success() {
        return Err(format!(
            "git {args:?} in {}: {}",
            root.display(),
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    Ok(output.stdout)
}
fn git_text(root: &Path, args: &[&str]) -> Result<String, String> {
    String::from_utf8(git(root, args)?)
        .map(|s| s.trim().to_owned())
        .map_err(|e| format!("git {args:?}: invalid UTF-8: {e}"))
}
fn executable_sha256(path: &Path) -> Result<String, String> {
    regular_file(path)?;
    let mut file = fs::File::open(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let mut hash = Sha256::new();
    let mut buf = [0; 64 * 1024];
    let mut total = 0u64;
    loop {
        let count = file
            .read(&mut buf)
            .map_err(|e| format!("{}: {e}", path.display()))?;
        if count == 0 {
            break;
        }
        total += count as u64;
        hash.update(&buf[..count]);
    }
    if total == 0 {
        return Err(format!("{}: collector executable is empty", path.display()));
    }
    Ok(format!("{:x}", hash.finalize()))
}

pub(super) fn preflight(
    ctx: &ExecutionContext,
    m: &Manifest,
    r: &RunRequest,
) -> Result<Provenance, String> {
    // Authorization precedes every repository, executable, and output operation.
    authorize(m, r.mode)?;
    if *m != candidate()? {
        return Err("requested manifest differs from the immutable executable factory".into());
    }
    let protocol_revision = revision(&r.protocol_revision)?;
    if r.approval_context.trim().is_empty() {
        return Err("approval context must record an approval reference".into());
    }
    if r.out.as_os_str().is_empty() {
        return Err("output directory must have a nonempty path".into());
    }
    match fs::symlink_metadata(&r.out) {
        Ok(_) => return Err(format!("output already exists: {}", r.out.display())),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => (),
        Err(e) => return Err(format!("inspect output {}: {e}", r.out.display())),
    }
    let repo = ctx
        .repo
        .canonicalize()
        .map_err(|e| format!("repository {}: {e}", ctx.repo.display()))?;
    let top = PathBuf::from(git_text(&repo, &["rev-parse", "--show-toplevel"])?);
    if top.canonicalize().map_err(|e| e.to_string())? != repo {
        return Err("collection context must name the Git repository root".into());
    }
    let code_revision = revision(&git_text(
        &repo,
        &["rev-parse", "--verify", "HEAD^{commit}"],
    )?)?;
    let resolved = git_text(
        &repo,
        &[
            "rev-parse",
            "--verify",
            &format!("{protocol_revision}^{{commit}}"),
        ],
    )?;
    if resolved != protocol_revision {
        return Err("protocol revision must name the exact full commit".into());
    }
    if !git(&repo, &["status", "--porcelain", "--untracked-files=no"])?.is_empty() {
        return Err("shortcut collection requires a clean tracked tree".into());
    }
    let protocol =
        fs::read(repo.join(&m.protocol)).map_err(|e| format!("protocol {}: {e}", m.protocol))?;
    if git(
        &repo,
        &["show", &format!("{protocol_revision}:{}", m.protocol)],
    )? != protocol
    {
        return Err("committed protocol differs from checkout bytes".into());
    }
    let bytes = manifest_bytes()?;
    let checkout = fs::read(repo.join(MANIFEST_PATH))
        .map_err(|e| format!("candidate manifest {MANIFEST_PATH}: {e}"))?;
    if checkout != bytes
        || git(
            &repo,
            &["show", &format!("{code_revision}:{MANIFEST_PATH}")],
        )? != bytes
    {
        return Err("candidate manifest differs between factory, checkout, or HEAD bytes".into());
    }
    Ok(Provenance {
        code_revision,
        protocol_revision,
        manifest_sha256: sha256(&bytes),
        collector_sha256: executable_sha256(&ctx.executable)?,
    })
}

pub(super) fn collect(ctx: &ExecutionContext, r: &RunRequest) -> Result<Index, String> {
    collect_with(ctx, r, |c, seed| {
        core::run(c.setup.clone(), seed, c.options.clone()).map_err(|e| format!("core run: {e:?}"))
    })
}

// Internal injection permits deterministic failure tests; CLI always uses the public F4 runner.
pub(super) fn collect_with(
    ctx: &ExecutionContext,
    r: &RunRequest,
    mut episode: impl FnMut(&Condition, u64) -> Result<core::Episode, String>,
) -> Result<Index, String> {
    let m = candidate()?;
    let p = preflight(ctx, &m, r)?;
    let mut writer =
        ArchiveWriter::create(&r.out, &m, r.mode, &p, &r.approval_context, limits(&m))?;
    for key in expected_keys(&m, r.mode) {
        let result = condition(&m, &key.condition)
            .and_then(|c| episode(c, key.seed))
            .and_then(|e| encode_envelope(&key, r.mode, &p, &e, m.raw_record_limit))
            .and_then(|bytes| writer.put(&key, &bytes));
        if let Err(error) = result {
            let error = format!("{} seed {}: {error}", key.condition, key.seed);
            if let Err(saved_error) = writer.fail(&error) {
                return Err(format!(
                    "{error}; also failed to preserve failure evidence: {saved_error}"
                ));
            }
            return Err(error);
        }
    }
    writer.finish()
}
