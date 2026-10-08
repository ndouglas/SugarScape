//! Bounded bytes and confined, nonsymlink archive filesystem access.
use serde::Serialize;
use std::{
    fs,
    io::{Read, Write},
    path::{Component, Path, PathBuf},
};

pub(super) fn checked_total(previous: u64, next: u64, limit: u64) -> Result<u64, String> {
    previous
        .checked_add(next)
        .filter(|n| *n <= limit)
        .ok_or_else(|| "raw byte budget exceeded or overflowed".into())
}
pub(super) fn read_bounded(path: &Path, limit: u64) -> Result<Vec<u8>, String> {
    let read = || -> Result<Vec<u8>, String> {
        let ceiling = limit
            .checked_add(1)
            .ok_or_else(|| "read bound overflow".to_owned())?;
        let mut bytes = Vec::new();
        fs::File::open(path)
            .map_err(|e| e.to_string())?
            .take(ceiling)
            .read_to_end(&mut bytes)
            .map_err(|e| e.to_string())?;
        if bytes.len() as u64 > limit {
            return Err("input byte limit exceeded".into());
        }
        Ok(bytes)
    };
    read().map_err(|e| format!("{}: {e}", path.display()))
}
struct MetadataWriter {
    bytes: Vec<u8>,
    limit: u64,
}
impl Write for MetadataWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        (self.bytes.len() as u64)
            .checked_add(buf.len() as u64)
            .filter(|n| *n <= self.limit)
            .ok_or_else(|| std::io::Error::other("metadata byte limit exceeded"))?;
        self.bytes
            .try_reserve(buf.len())
            .map_err(std::io::Error::other)?;
        self.bytes.extend_from_slice(buf);
        Ok(buf.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
pub(super) fn encode_metadata<T: Serialize>(value: &T, limit: u64) -> Result<Vec<u8>, String> {
    let mut writer = MetadataWriter {
        bytes: vec![],
        limit,
    };
    serde_json::to_writer_pretty(&mut writer, value).map_err(|e| format!("metadata: {e}"))?;
    Ok(writer.bytes)
}
pub(super) fn directory(path: &Path) -> Result<PathBuf, String> {
    let metadata = fs::symlink_metadata(path).map_err(|e| format!("{}: {e}", path.display()))?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(format!("{}: expected nonsymlink directory", path.display()));
    }
    path.canonicalize()
        .map_err(|e| format!("{}: {e}", path.display()))
}
pub(super) fn regular_file(path: &Path) -> Result<(), String> {
    let metadata = fs::symlink_metadata(path).map_err(|e| format!("{}: {e}", path.display()))?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(format!(
            "{}: expected nonsymlink regular file",
            path.display()
        ));
    }
    Ok(())
}
/// Require literal relative components and inspect each descendant before any read/write.
pub(super) fn confined_path(
    root: &Path,
    relative: &str,
    existing: bool,
) -> Result<PathBuf, String> {
    let canonical_root = directory(root)?;
    let parts = Path::new(relative).components().collect::<Vec<_>>();
    if parts.is_empty() || parts.iter().any(|p| !matches!(p, Component::Normal(_))) {
        return Err(format!("unsafe archive path: {relative}"));
    }
    let mut path = canonical_root.clone();
    for (i, part) in parts.iter().enumerate() {
        path.push(part.as_os_str());
        if i + 1 == parts.len() {
            if existing {
                regular_file(&path)?;
            }
        } else {
            directory(&path)?;
        }
    }
    if existing {
        let canonical = path.canonicalize().map_err(|e| e.to_string())?;
        if canonical != path || !canonical.starts_with(&canonical_root) {
            return Err(format!(
                "raw path escapes canonical archive root: {relative}"
            ));
        }
    }
    Ok(path)
}
