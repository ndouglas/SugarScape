use super::io::*;
use sha2::{Digest, Sha256};
use std::io::{self, Write};
use sugarscape_core::war::{ObservedFrame, RunPayload, RunRecord};

#[derive(Default)]
struct Writer {
    bytes: Vec<u8>,
    limit: Option<usize>,
    fail_flush: bool,
    fail_sync: bool,
}
impl Write for Writer {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        let remaining = self
            .limit
            .unwrap_or(usize::MAX)
            .saturating_sub(self.bytes.len());
        if remaining == 0 {
            return Err(io::Error::other("write failed"));
        }
        let count = bytes.len().min(remaining).min(3);
        self.bytes.extend_from_slice(&bytes[..count]);
        Ok(count)
    }
    fn flush(&mut self) -> io::Result<()> {
        if self.fail_flush {
            Err(io::Error::other("flush failed"))
        } else {
            Ok(())
        }
    }
}
impl DurableWrite for Writer {
    fn sync_all(&mut self) -> io::Result<()> {
        if self.fail_sync {
            Err(io::Error::other("sync failed"))
        } else {
            Ok(())
        }
    }
}
fn record() -> RunRecord {
    RunRecord {
        schema: "test".into(),
        input_identity: "test".into(),
        payload: RunPayload::Observed {
            frame: ObservedFrame::Book {
                frame: sugarscape_core::war::BookFrame {
                    tick: 1,
                    fingerprint: "test".into(),
                    rng_state: "{}".into(),
                    snapshot: sugarscape_core::stats::Snapshot::default(),
                    combat_enabled: true,
                    deaths: vec![],
                    kills: vec![],
                    agent_stores: vec![],
                    site_stores: vec![],
                    unavailable: vec![],
                },
            },
        },
    }
}

#[test]
fn short_writes_are_completed_before_acknowledgment() {
    let mut journal = Journal::with_writer(Writer::default());
    journal.append(&record()).unwrap();
    let receipt = journal.finish().unwrap();
    assert_eq!(receipt.acknowledged_steps, 1);
    assert_eq!(
        receipt.sha256,
        format!("{:x}", Sha256::digest(&journal.writer.bytes))
    );
    assert_eq!(receipt.bytes, journal.writer.bytes.len() as u64);
}
#[test]
fn failed_partial_write_keeps_only_acknowledged_prefix() {
    let mut journal = Journal::with_writer(Writer::default());
    journal.append(&record()).unwrap();
    let prefix = journal.acknowledged();
    journal.writer.limit = Some(journal.writer.bytes.len() + 5);
    assert!(journal.append(&record()).is_err());
    assert_eq!(journal.acknowledged(), prefix);
    assert!(journal.finish().is_err());
}
#[test]
fn flush_and_sync_failures_never_acknowledge_a_record() {
    for writer in [
        Writer {
            fail_flush: true,
            ..Writer::default()
        },
        Writer {
            fail_sync: true,
            ..Writer::default()
        },
    ] {
        let mut journal = Journal::with_writer(writer);
        assert!(journal.append(&record()).is_err());
        assert_eq!(journal.acknowledged().bytes, 0);
        assert_eq!(journal.acknowledged().acknowledged_steps, 0);
        assert!(journal.finish().is_err());
    }
}
#[test]
fn finish_failure_does_not_declare_completion() {
    let mut journal = Journal::with_writer(Writer::default());
    journal.append(&record()).unwrap();
    journal.writer.fail_sync = true;
    assert!(journal.finish().is_err());
    assert_eq!(journal.acknowledged().acknowledged_steps, 1);
}
#[test]
fn journal_limit_rejects_before_writing_and_poison_finish() {
    let mut journal = Journal::with_writer(Writer::default());
    journal.bytes = JOURNAL_LIMIT;
    assert!(journal.append(&record()).is_err());
    assert!(journal.writer.bytes.is_empty());
    assert!(journal.finish().is_err());
}
#[test]
fn record_limit_rejects_before_writing() {
    let mut large = record();
    large.input_identity = "x".repeat(RECORD_LIMIT);
    let mut journal = Journal::with_writer(Writer::default());
    assert!(journal.append(&large).is_err());
    assert!(journal.writer.bytes.is_empty());
}
