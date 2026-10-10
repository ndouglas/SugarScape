//! W1's declared compilation inputs; independent of F5/BT selectors and Git.
use sha2::{Digest, Sha256};
use std::{collections::BTreeSet, fs, path::Path};

pub const DIRECTORIES: &[&str] = &["crates/sugarscape-core/src", "survey/src/war1"];
pub const FILES: &[&str] = &[
    "Cargo.toml",
    "Cargo.lock",
    "crates/sugarscape-core/Cargo.toml",
    "crates/sugarscape-core/assets/sugar-map.txt",
    "survey/Cargo.toml",
    "survey/Cargo.lock",
    "survey/build.rs",
    "survey/src/bin/war1.rs",
    "survey/build_support/war1_source_identity.rs",
    "docs/superpowers/specs/2026-10-09-war-1-engagement-design.md",
    "docs/superpowers/specs/2026-10-10-war-1-engineering-amendment.md",
    "docs/superpowers/plans/2026-10-09-war-1-engagement.md",
];
pub const REQUIRED_SOURCES: &[&str] = &[
    "crates/sugarscape-core/src/lib.rs",
    "survey/src/war1/mod.rs",
    "survey/src/war1/cli.rs",
    "survey/src/war1/io.rs",
    "survey/src/war1/wire.rs",
];
fn excluded(path: &str) -> bool {
    path.split('/').any(|p| {
        matches!(
            p,
            "target"
                | ".git"
                | ".superpowers"
                | "node_modules"
                | "__pycache__"
                | "generated"
                | "evidence"
                | "cache"
        )
    })
}
fn selected(path: &str) -> bool {
    !path.is_empty()
        && !path.contains('\\')
        && !path.starts_with('/')
        && path
            .split('/')
            .all(|p| !p.is_empty() && !matches!(p, "." | ".."))
        && !excluded(path)
        && (FILES.contains(&path)
            || DIRECTORIES
                .iter()
                .any(|d| path.strip_prefix(d).is_some_and(|s| s.starts_with('/'))))
}
pub fn fingerprint(
    entries: impl IntoIterator<Item = Result<(String, Vec<u8>), String>>,
) -> Result<String, String> {
    let mut entries = entries.into_iter().collect::<Result<Vec<_>, _>>()?;
    entries.sort_by(|a, b| a.0.cmp(&b.0));
    let mut seen = BTreeSet::new();
    let mut hash = Sha256::new();
    hash.update(b"war1-compiled-inputs-v1\0");
    for (path, bytes) in entries {
        if !selected(&path) || !seen.insert(path.clone()) {
            return Err(format!("invalid/duplicate W1 input {path}"));
        }
        hash.update((path.len() as u64).to_be_bytes());
        hash.update(path.as_bytes());
        hash.update((bytes.len() as u64).to_be_bytes());
        hash.update(bytes);
    }
    for path in FILES.iter().chain(REQUIRED_SOURCES).copied() {
        if !seen.contains(path) {
            return Err(format!("missing required W1 input {path}"));
        }
    }
    Ok(format!("{:x}", hash.finalize()))
}
pub fn read(root: &Path) -> Result<Vec<(String, Vec<u8>)>, String> {
    fn visit(root: &Path, relative: &str, out: &mut Vec<(String, Vec<u8>)>) -> Result<(), String> {
        if excluded(relative) {
            return Ok(());
        }
        let path = root.join(relative);
        // Check every ancestor too: an ordinary selected file below a symlink is not a regular source input.
        let mut prefix = root.to_path_buf();
        for part in Path::new(relative).components() {
            prefix.push(part);
            let metadata =
                fs::symlink_metadata(&prefix).map_err(|e| format!("{}: {e}", prefix.display()))?;
            if metadata.file_type().is_symlink() {
                return Err(format!("symlink W1 input {}", prefix.display()));
            }
        }
        let metadata =
            fs::symlink_metadata(&path).map_err(|e| format!("{}: {e}", path.display()))?;
        if metadata.is_dir() {
            for entry in fs::read_dir(&path).map_err(|e| e.to_string())? {
                let entry = entry.map_err(|e| e.to_string())?;
                let name = entry
                    .file_name()
                    .into_string()
                    .map_err(|_| "non UTF-8 W1 source path")?;
                visit(root, &format!("{relative}/{name}"), out)?;
            }
        } else if metadata.is_file() && selected(relative) {
            out.push((relative.into(), fs::read(&path).map_err(|e| e.to_string())?));
        } else {
            return Err(format!("nonregular/unselected W1 input {relative}"));
        }
        Ok(())
    }
    let mut entries = vec![];
    for path in DIRECTORIES.iter().chain(FILES) {
        visit(root, path, &mut entries)?;
    }
    entries.sort_by(|a, b| a.0.cmp(&b.0));
    fingerprint(entries.clone().into_iter().map(Ok))?;
    Ok(entries)
}

#[allow(dead_code)] // Runtime-only comparison; this module is shared with build.rs.
pub fn verify(root: &Path, compiled: &str) -> Result<(), String> {
    let current = fingerprint(read(root)?.into_iter().map(Ok))?;
    if current != compiled {
        return Err(format!("W1 source inputs changed since compilation: compiled {compiled}, current {current}; rebuild war1"));
    }
    Ok(())
}
