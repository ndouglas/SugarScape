//! Independent create-only, synchronized archive I/O.
use serde::Serialize;
use std::{
    fs::{self, File, OpenOptions},
    io::Write,
    path::{Component, Path},
};
pub(super) fn check_path(path: &Path) -> Result<(), String> {
    let mut at = std::path::PathBuf::new();
    for part in path.components() {
        if matches!(part, Component::ParentDir) {
            return Err("parent traversal rejected".into());
        }
        at.push(part.as_os_str());
        match fs::symlink_metadata(&at) {
            Ok(m) if m.file_type().is_symlink() => {
                return Err(format!("symlink rejected: {}", at.display()))
            }
            Ok(_) => (),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => (),
            Err(e) => return Err(e.to_string()),
        }
    }
    Ok(())
}
pub(super) fn sync_parent(path: &Path) -> Result<(), String> {
    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    check_path(parent)?;
    if !fs::metadata(parent).map_err(|e| e.to_string())?.is_dir() {
        return Err(format!(
            "parent sync requires directory: {}",
            parent.display()
        ));
    }
    File::open(parent)
        .and_then(|f| f.sync_all())
        .map_err(|e| format!("parent sync {}: {e}", path.display()))
}
pub(super) fn directory(path: &Path) -> Result<(), String> {
    check_path(path)?;
    fs::create_dir(path).map_err(|e| format!("create directory {}: {e}", path.display()))?;
    sync_parent(path)
}
pub(super) fn create(path: &Path, bytes: &[u8]) -> Result<(), String> {
    check_path(path)?;
    let mut f = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|e| format!("create {}: {e}", path.display()))?;
    f.write_all(bytes)
        .and_then(|()| f.flush())
        .and_then(|()| f.sync_all())
        .map_err(|e| format!("write/sync {}: {e}", path.display()))?;
    sync_parent(path)
}
pub(super) fn json(path: &Path, value: &impl Serialize) -> Result<(), String> {
    create(
        path,
        &serde_json::to_vec_pretty(value).map_err(|e| e.to_string())?,
    )
}
pub(super) fn read(path: &Path) -> Result<Vec<u8>, String> {
    check_path(path)?;
    let m = fs::symlink_metadata(path).map_err(|e| format!("{}: {e}", path.display()))?;
    if !m.is_file() || m.len() > 64 * 1024 * 1024 {
        return Err(format!("not bounded regular file: {}", path.display()));
    }
    fs::read(path).map_err(|e| format!("{}: {e}", path.display()))
}
pub(super) fn relative(root: &Path, name: &str) -> Result<std::path::PathBuf, String> {
    if name.is_empty()
        || name
            .split('/')
            .any(|p| p.is_empty() || p == "." || p == "..")
        || Path::new(name)
            .components()
            .any(|c| !matches!(c, Component::Normal(_)))
    {
        return Err(format!("unsafe relative path {name}"));
    }
    let path = root.join(name);
    check_path(&path)?;
    Ok(path)
}
pub(super) fn append(file: &mut File, frame: &impl Serialize) -> Result<(), String> {
    let mut bytes = serde_json::to_vec(frame).map_err(|e| e.to_string())?;
    bytes.push(b'\n');
    file.write_all(&bytes)
        .and_then(|()| file.flush())
        .and_then(|()| file.sync_all())
        .map_err(|e| format!("frame write/sync: {e}"))
}
