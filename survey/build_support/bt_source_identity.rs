//! Declared BT compiled-input closure, independent of Git and the F5 selector.
use sha2::{Digest, Sha256};
use std::{fs, path::Path};
pub const DIRECTORIES: &[&str] = &[
    "crates/sugarscape-core",
    "survey/src",
    "survey/build_support",
];
pub const FILES: &[&str] = &[
    "Cargo.toml",
    "Cargo.lock",
    "survey/Cargo.toml",
    "survey/Cargo.lock",
    "survey/build.rs",
    "docs/superpowers/specs/2026-10-08-minds-behavior-trees-design.md",
    "docs/superpowers/specs/2026-10-08-minds-behavior-trees-protocol.md",
];
pub const PROTOCOL: &str = "docs/superpowers/specs/2026-10-08-minds-behavior-trees-protocol.md";
fn safe(path: &str) -> bool {
    !path.is_empty()
        && !path.contains('\\')
        && path
            .split('/')
            .all(|p| !p.is_empty() && !matches!(p, "." | ".."))
        && !path.starts_with('/')
}
fn excluded(path: &str) -> bool {
    path.split('/').any(|p| {
        matches!(
            p,
            "target" | ".git" | ".superpowers" | "node_modules" | "__pycache__"
        )
    })
}
pub fn selected(path: &str) -> bool {
    safe(path)
        && !excluded(path)
        && (FILES.contains(&path)
            || DIRECTORIES
                .iter()
                .any(|d| path.strip_prefix(d).is_some_and(|t| t.starts_with('/'))))
}
pub fn fingerprint(
    entries: impl IntoIterator<Item = Result<(String, Vec<u8>), String>>,
) -> Result<String, String> {
    let mut entries = entries.into_iter().collect::<Result<Vec<_>, _>>()?;
    entries.sort_by(|a, b| a.0.cmp(&b.0));
    let mut h = Sha256::new();
    h.update(b"minds-behavior-tree-compiled-inputs-v1\0");
    let mut last = None;
    for (path, bytes) in entries {
        if !selected(&path) || last.as_ref() == Some(&path) {
            return Err(format!("invalid/duplicate BT input {path}"));
        }
        h.update((path.len() as u64).to_be_bytes());
        h.update(path.as_bytes());
        h.update((bytes.len() as u64).to_be_bytes());
        h.update(bytes);
        last = Some(path)
    }
    Ok(format!("{:x}", h.finalize()))
}
pub fn read(root: &Path) -> Result<Vec<(String, Vec<u8>)>, String> {
    fn visit(root: &Path, relative: &str, out: &mut Vec<(String, Vec<u8>)>) -> Result<(), String> {
        if excluded(relative) {
            return Ok(());
        }
        let path = root.join(relative);
        let m = fs::symlink_metadata(&path).map_err(|e| format!("{}: {e}", path.display()))?;
        if m.file_type().is_symlink() {
            return Err(format!("symlink BT input {relative}"));
        }
        if m.is_dir() {
            for child in fs::read_dir(&path).map_err(|e| e.to_string())? {
                let child = child.map_err(|e| e.to_string())?;
                let name = child
                    .file_name()
                    .into_string()
                    .map_err(|_| "non UTF-8 input")?;
                visit(root, &format!("{relative}/{name}"), out)?
            }
        } else if m.is_file() && selected(relative) {
            out.push((relative.into(), fs::read(&path).map_err(|e| e.to_string())?))
        } else {
            return Err(format!("nonregular/unselected BT input {relative}"));
        }
        Ok(())
    }
    let mut out = vec![];
    for path in DIRECTORIES.iter().chain(FILES) {
        if *path == PROTOCOL
            && matches!(fs::symlink_metadata(root.join(path)),Err(e) if e.kind()==std::io::ErrorKind::NotFound)
        {
            continue;
        }
        visit(root, path, &mut out)?
    }
    out.sort_by(|a, b| a.0.cmp(&b.0));
    fingerprint(out.clone().into_iter().map(Ok))?;
    Ok(out)
}
