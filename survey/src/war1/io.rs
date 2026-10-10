//! Create-only bounded output and individually synchronized acknowledgment receipts.
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::{
    fs::{self, File, OpenOptions},
    io::{self, Read, Write},
    path::{Component, Path, PathBuf},
};
use sugarscape_core::war::{ObservedFrame, RunPayload, RunRecord};
pub const INPUT_LIMIT: usize = 1024 * 1024;
pub(crate) const RECORD_LIMIT: usize = 2 * 1024 * 1024;
pub(crate) const JOURNAL_LIMIT: u64 = 128 * 1024 * 1024;
const RNG_LIMIT: usize = 16 * 1024;

fn check_path(path: &Path) -> Result<PathBuf, String> {
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()
            .map_err(|e| e.to_string())?
            .join(path)
    };
    let mut prefix = PathBuf::new();
    for component in absolute.components() {
        if component == Component::ParentDir {
            return Err("paths must not contain '..'; use a canonical parent".into());
        }
        prefix.push(component);
        let m = fs::symlink_metadata(&prefix).map_err(|e| format!("{}: {e}", prefix.display()))?;
        if m.file_type().is_symlink() {
            return Err(format!("symlink path refused: {}", prefix.display()));
        }
    }
    Ok(absolute)
}
pub fn read_input(path: &Path) -> Result<Vec<u8>, String> {
    let path = check_path(path)?;
    // Reject FIFOs before open: opening one would wait indefinitely for a writer.
    if !fs::symlink_metadata(&path)
        .map_err(|e| e.to_string())?
        .is_file()
    {
        return Err("input must be a regular file".into());
    }
    let file = File::open(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    if !file.metadata().map_err(|e| e.to_string())?.is_file() {
        return Err("input must be a regular file".into());
    }
    let mut bytes = vec![];
    file.take(INPUT_LIMIT as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() > INPUT_LIMIT {
        return Err("input exceeds 1 MiB".into());
    }
    Ok(bytes)
}
pub fn create_output_directory(path: &Path) -> Result<(), String> {
    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let parent = check_path(parent)?;
    if !parent.is_dir() {
        return Err("output parent must be an existing directory".into());
    }
    let name = path.file_name().ok_or("output must name a new directory")?;
    fs::create_dir(parent.join(name))
        .map_err(|e| format!("create new output {}: {e}", path.display()))
}
pub(crate) fn sync_directory(path: &Path) -> Result<(), String> {
    File::open(path)
        .and_then(|file| file.sync_all())
        .map_err(|e| format!("sync directory {}: {e}", path.display()))
}
pub(crate) fn save_bytes(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|e| format!("create {}: {e}", path.display()))?;
    file.write_all(bytes)
        .and_then(|()| file.flush())
        .and_then(|()| file.sync_all())
        .map_err(|e| format!("save {}: {e}", path.display()))
}
pub(crate) fn save_json(path: &Path, value: &impl Serialize) -> Result<(), String> {
    let mut bytes = serde_json::to_vec_pretty(value).map_err(|e| e.to_string())?;
    bytes.push(b'\n');
    save_bytes(path, &bytes)
}
pub(crate) trait DurableWrite: Write {
    fn sync_all(&mut self) -> io::Result<()>;
}
impl DurableWrite for File {
    fn sync_all(&mut self) -> io::Result<()> {
        File::sync_all(self)
    }
}
pub(crate) struct Journal<W: DurableWrite = File> {
    pub(super) writer: W,
    digest: Sha256,
    pub(super) bytes: u64,
    persisted_steps: u64,
    failed: bool,
}
#[derive(Debug, PartialEq, Eq, Serialize)]
pub(crate) struct JournalReceipt {
    pub sha256: String,
    pub bytes: u64,
    pub acknowledged_steps: u64,
}
impl Journal<File> {
    pub(crate) fn new(path: &Path) -> Result<Self, String> {
        let file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path)
            .map_err(|e| format!("create journal {}: {e}", path.display()))?;
        Ok(Self::with_writer(file))
    }
}
// Serialize directly into a bounded writer instead of allocating an oversized JSON record first.
struct BoundedRecord(Vec<u8>);
impl Write for BoundedRecord {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if bytes.len() > RECORD_LIMIT.saturating_sub(self.0.len()) {
            return Err(io::Error::other("JSONL record exceeds 2 MiB"));
        }
        self.0.extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
impl<W: DurableWrite> Journal<W> {
    pub(super) fn with_writer(writer: W) -> Self {
        Self {
            writer,
            digest: Sha256::new(),
            bytes: 0,
            persisted_steps: 0,
            failed: false,
        }
    }
    pub(crate) fn append(&mut self, record: &RunRecord) -> Result<(), String> {
        if self.failed {
            return Err("journal already failed; retries are unavailable".into());
        }
        let result = self.append_record(record);
        if result.is_err() {
            self.failed = true;
        }
        result
    }
    fn append_record(&mut self, record: &RunRecord) -> Result<(), String> {
        if let RunPayload::Observed {
            frame: ObservedFrame::Book { frame },
        } = &record.payload
        {
            if frame.rng_state.len() > RNG_LIMIT {
                return Err("RNG text exceeds 16 KiB".into());
            }
        }
        let mut bounded = BoundedRecord(vec![]);
        serde_json::to_writer(&mut bounded, record).map_err(|e| e.to_string())?;
        bounded.write_all(b"\n").map_err(|e| e.to_string())?;
        let bytes = bounded.0;
        if bytes.len() as u64 > JOURNAL_LIMIT.saturating_sub(self.bytes) {
            return Err("engineering journal exceeds 128 MiB".into());
        }
        self.writer
            .write_all(&bytes)
            .and_then(|()| self.writer.flush())
            .and_then(|()| self.writer.sync_all())
            .map_err(|e| format!("journal write/flush/sync: {e}"))?;
        self.digest.update(&bytes);
        self.bytes += bytes.len() as u64;
        if matches!(
            &record.payload,
            RunPayload::Observed {
                frame: ObservedFrame::Graph { .. }
            }
        ) || matches!(&record.payload, RunPayload::Observed { frame: ObservedFrame::Book { frame } } if frame.tick > 0)
        {
            self.persisted_steps += 1;
        }
        Ok(())
    }
    pub(crate) fn acknowledged(&self) -> JournalReceipt {
        JournalReceipt {
            sha256: format!("{:x}", self.digest.clone().finalize()),
            bytes: self.bytes,
            acknowledged_steps: self.persisted_steps,
        }
    }
    pub(crate) fn finish(&mut self) -> Result<JournalReceipt, String> {
        if self.failed {
            return Err("journal contains an unacknowledged/failed append".into());
        }
        self.writer
            .flush()
            .and_then(|()| self.writer.sync_all())
            .map_err(|e| format!("journal finish flush/sync: {e}"))?;
        Ok(self.acknowledged())
    }
}
