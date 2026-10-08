use super::super::{
    archive::{limits, raw_path, ArchiveReader, ArchiveWriter, Index},
    io::{checked_total, encode_metadata, read_bounded},
    manifest::{candidate, expected_keys},
    sha256, CollectionMode,
};
use super::support::{encoded_fixture, owned_tempdir, test_provenance, OwnedTempdir};
use std::{fs, path::Path};

fn writer(out: &Path) -> ArchiveWriter {
    let m = candidate().unwrap();
    ArchiveWriter::create(
        out,
        &m,
        CollectionMode::Construction,
        &test_provenance(),
        "engineering test",
        limits(&m),
    )
    .unwrap()
}
fn complete_archive() -> OwnedTempdir {
    let tmp = owned_tempdir();
    let mut w = writer(&tmp.path().join("archive"));
    for key in expected_keys(&candidate().unwrap(), CollectionMode::Construction) {
        w.put(&key, &encoded_fixture(&key.condition, key.seed).unwrap())
            .unwrap();
    }
    w.finish().unwrap();
    tmp
}
fn index_path(tmp: &OwnedTempdir) -> std::path::PathBuf {
    tmp.path().join("archive/index.json")
}
fn load_index(tmp: &OwnedTempdir) -> Index {
    serde_json::from_slice(&fs::read(index_path(tmp)).unwrap()).unwrap()
}
fn rewrite_index(tmp: &OwnedTempdir, index: &Index) {
    fs::write(index_path(tmp), serde_json::to_vec(index).unwrap()).unwrap();
}

#[test]
fn cumulative_raw_budget_is_checked_without_overflow() {
    assert_eq!(checked_total(8, 2, 10).unwrap(), 10);
    assert!(checked_total(8, 3, 10).is_err());
    assert!(checked_total(u64::MAX, 1, u64::MAX).is_err());
}
#[test]
fn bounded_reads_accept_exact_limit_and_reject_one_byte_short_and_overflow() {
    let tmp = owned_tempdir();
    let p = tmp.path().join("bytes");
    fs::write(&p, b"abcd").unwrap();
    assert_eq!(read_bounded(&p, 4).unwrap(), b"abcd");
    assert!(read_bounded(&p, 3).is_err());
    assert!(read_bounded(&p, u64::MAX).is_err());
}
#[test]
fn draft_scientific_writer_creates_nothing() {
    let tmp = owned_tempdir();
    let out = tmp.path().join("new");
    let m = candidate().unwrap();
    assert!(ArchiveWriter::create(
        &out,
        &m,
        CollectionMode::Scientific,
        &test_provenance(),
        "engineering test",
        limits(&m)
    )
    .is_err());
    assert!(!out.exists());
}
#[test]
fn writer_preflight_rejects_noncanonical_manifest_limits_provenance_and_empty_approval() {
    let tmp = owned_tempdir();
    let m = candidate().unwrap();
    let out = tmp.path().join("new");
    for field in 0..4 {
        let mut p = test_provenance();
        match field {
            0 => p.code_revision = "fake".into(),
            1 => p.protocol_revision = "fake".into(),
            2 => p.manifest_sha256 = "d".repeat(64),
            _ => p.collector_sha256 = "fake".into(),
        }
        assert!(ArchiveWriter::create(
            &out,
            &m,
            CollectionMode::Construction,
            &p,
            "test",
            limits(&m)
        )
        .is_err());
        assert!(!out.exists());
    }
    let mut cap = limits(&m);
    cap.record -= 1;
    assert!(ArchiveWriter::create(
        &out,
        &m,
        CollectionMode::Construction,
        &test_provenance(),
        "test",
        cap
    )
    .is_err());
    let mut bad = m.clone();
    bad.conditions[0].options.ticks = 511;
    assert!(ArchiveWriter::create(
        &out,
        &bad,
        CollectionMode::Construction,
        &test_provenance(),
        "test",
        limits(&bad)
    )
    .is_err());
    assert!(ArchiveWriter::create(
        &out,
        &m,
        CollectionMode::Construction,
        &test_provenance(),
        " ",
        limits(&m)
    )
    .is_err());
    assert!(!out.exists());
}
#[test]
fn existing_output_is_never_overwritten() {
    let tmp = owned_tempdir();
    let out = tmp.path().join("old");
    fs::create_dir(&out).unwrap();
    fs::write(out.join("sentinel"), b"keep").unwrap();
    let m = candidate().unwrap();
    assert!(ArchiveWriter::create(
        &out,
        &m,
        CollectionMode::Construction,
        &test_provenance(),
        "test",
        limits(&m)
    )
    .is_err());
    assert_eq!(fs::read(out.join("sentinel")).unwrap(), b"keep");
}
#[test]
fn complete_archive_streams_exact_construction_matrix_and_keeps_pending_receipt() {
    let tmp = complete_archive();
    let mut r = ArchiveReader::open(&index_path(&tmp)).unwrap();
    let expected = expected_keys(&candidate().unwrap(), CollectionMode::Construction);
    let mut seen = vec![];
    while let Some((key, e)) = r.next().unwrap() {
        assert_eq!(e.seed, key.seed);
        seen.push(key);
    }
    assert_eq!(seen.len(), 30);
    assert_eq!(seen.first().unwrap().condition, "route.straight.paid");
    assert_eq!(seen.first().unwrap().seed, 7);
    assert_eq!(seen.last().unwrap().condition, "access.twisting.protected");
    assert_eq!(seen.last().unwrap().seed, 8);
    assert_eq!(seen, expected);
    assert!(r.index().completed);
    assert_eq!(
        fs::read(index_path(&tmp)).unwrap(),
        fs::read(tmp.path().join("archive/index.pending.json")).unwrap()
    );
    assert!(ArchiveReader::open(&tmp.path().join("archive/index.pending.json")).is_err());
    assert!(ArchiveReader::open(&tmp.path().join("archive/index.incomplete.json")).is_err());
}
#[test]
fn incomplete_and_rejected_writers_cannot_finish() {
    let tmp = owned_tempdir();
    let out = tmp.path().join("incomplete");
    let w = writer(&out);
    assert!(w.finish().is_err());
    assert!(!out.join("index.json").exists());
    let out = tmp.path().join("failed");
    let mut w = writer(&out);
    let keys = expected_keys(&candidate().unwrap(), CollectionMode::Construction);
    assert!(w
        .put(
            &keys[1],
            &encoded_fixture(&keys[1].condition, keys[1].seed).unwrap()
        )
        .is_err());
    assert!(w
        .put(
            &keys[0],
            &encoded_fixture(&keys[0].condition, keys[0].seed).unwrap()
        )
        .is_err());
    w.fail("wrong key order").unwrap();
    assert!(w.fail("second failure").is_err());
    assert!(w.finish().is_err());
    assert!(!out.join("index.json").exists());
    assert!(fs::read_to_string(out.join("failure.json"))
        .unwrap()
        .contains("wrong key order"));
}
#[test]
fn writer_rejects_duplicate_extra_out_of_range_and_invalid_body_keys() {
    let keys = expected_keys(&candidate().unwrap(), CollectionMode::Construction);
    for kind in 0..4 {
        let tmp = owned_tempdir();
        let out = tmp.path().join("archive");
        let mut w = writer(&out);
        let mut key = keys[0].clone();
        let mut bytes = encoded_fixture(&key.condition, key.seed).unwrap();
        match kind {
            0 => {
                w.put(&key, &bytes).unwrap();
            }
            1 => key.condition = "extra".into(),
            2 => key.seed = 10001,
            _ => {
                let mut e = super::support::decode_fixture(&key.condition, key.seed).unwrap();
                e.episode.options.ticks = 511;
                bytes = serde_json::to_vec(&e).unwrap();
            }
        }
        assert!(w.put(&key, &bytes).is_err());
        assert!(w.finish().is_err());
        assert!(!out.join("index.json").exists());
    }
}
#[test]
fn failed_raw_write_and_receipt_leave_no_completed_archive() {
    for receipt in [false, true] {
        let tmp = owned_tempdir();
        let out = tmp.path().join("archive");
        let mut w = writer(&out);
        let key = &expected_keys(&candidate().unwrap(), CollectionMode::Construction)[0];
        let path = if receipt {
            out.join("progress/000.json")
        } else {
            out.join(raw_path(key))
        };
        fs::write(&path, b"keep").unwrap();
        assert!(w
            .put(key, &encoded_fixture(&key.condition, key.seed).unwrap())
            .is_err());
        w.fail("injected exclusive write collision").unwrap();
        assert!(w.finish().is_err());
        assert_eq!(fs::read(path).unwrap(), b"keep");
        assert!(!out.join("index.json").exists());
    }
}
#[test]
fn finish_revalidates_saved_raw_before_publication_and_never_overwrites_pending_or_final() {
    for fault in 0..3 {
        let tmp = owned_tempdir();
        let out = tmp.path().join("archive");
        let mut w = writer(&out);
        let keys = expected_keys(&candidate().unwrap(), CollectionMode::Construction);
        for key in &keys {
            w.put(key, &encoded_fixture(&key.condition, key.seed).unwrap())
                .unwrap();
        }
        let path = match fault {
            0 => out.join(raw_path(&keys[0])),
            1 => out.join("index.pending.json"),
            _ => out.join("index.json"),
        };
        fs::write(&path, b"keep").unwrap();
        assert!(w.finish().is_err());
        assert_eq!(fs::read(&path).unwrap(), b"keep");
        if fault < 2 {
            assert!(!out.join("index.json").exists());
        }
    }
}
#[test]
fn reader_rejects_wrong_key_lists_ref_order_duplicates_missing_and_extra() {
    let tmp = complete_archive();
    let original = load_index(&tmp);
    for fault in 0..7 {
        let mut bad = original.clone();
        match fault {
            0 => bad.expected_keys.swap(0, 1),
            1 => bad.runs.swap(0, 1),
            2 => bad.runs[1] = bad.runs[0].clone(),
            3 => {
                bad.runs.pop();
            }
            4 => bad.runs.push(bad.runs[0].clone()),
            5 => bad.runs[0].key.seed = 10001,
            _ => bad.completed = false,
        }
        rewrite_index(&tmp, &bad);
        assert!(
            ArchiveReader::open(&index_path(&tmp)).is_err(),
            "fault {fault}"
        );
    }
}
#[test]
fn reader_rejects_manifest_and_provenance_and_envelope_mismatch() {
    let tmp = complete_archive();
    let original = load_index(&tmp);
    for fault in 0..7 {
        let mut bad = original.clone();
        match fault {
            0 => bad.manifest.conditions[0].options.ticks = 511,
            1 => bad.provenance.manifest_sha256 = "d".repeat(64),
            2 => bad.provenance.code_revision = "fake".into(),
            3 => bad.provenance.protocol_revision = "fake".into(),
            4 => bad.provenance.collector_sha256 = "fake".into(),
            5 => bad.schema = "fake".into(),
            _ => bad.mode = CollectionMode::Scientific,
        }
        rewrite_index(&tmp, &bad);
        assert!(
            ArchiveReader::open(&index_path(&tmp)).is_err(),
            "fault {fault}"
        );
    }
    for fault in 0..3 {
        let mut bad = original.clone();
        match fault {
            0 => bad.provenance.code_revision = "d".repeat(40),
            1 => bad.provenance.protocol_revision = "d".repeat(40),
            _ => bad.provenance.collector_sha256 = "d".repeat(64),
        }
        rewrite_index(&tmp, &bad);
        assert!(ArchiveReader::open(&index_path(&tmp))
            .unwrap()
            .next()
            .is_err());
    }
}
#[test]
fn reader_validates_raw_bytes_hash_and_resigned_observed_body() {
    let tmp = complete_archive();
    let original = load_index(&tmp);
    let raw = &original.runs[0];
    let path = tmp.path().join("archive").join(&raw.path);
    let bytes = fs::read(&path).unwrap();
    for fault in 0..4 {
        let mut bad = original.clone();
        let mut saved = bytes.clone();
        match fault {
            0 => bad.runs[0].bytes += 1,
            1 => bad.runs[0].sha256 = "d".repeat(64),
            2 => saved[0] = b' ',
            _ => {
                let mut e =
                    super::support::decode_fixture(&raw.key.condition, raw.key.seed).unwrap();
                e.episode.snapshots[0].summary.work.opportunities = 1;
                saved = serde_json::to_vec(&e).unwrap();
                bad.runs[0].bytes = saved.len() as u64;
                bad.runs[0].sha256 = sha256(&saved);
            }
        }
        fs::write(&path, saved).unwrap();
        rewrite_index(&tmp, &bad);
        assert!(
            ArchiveReader::open(&index_path(&tmp))
                .unwrap()
                .next()
                .is_err(),
            "fault {fault}"
        );
    }
}
#[test]
fn reader_rejects_unsafe_alternative_and_duplicate_record_paths() {
    let tmp = complete_archive();
    let original = load_index(&tmp);
    for path in [
        "/tmp/raw.json",
        "../raw.json",
        "raw/../raw.json",
        "./raw/route.straight.paid/7.json",
        "raw/route.straight.paid/07.json",
        "raw//route.straight.paid/7.json",
    ] {
        let mut bad = original.clone();
        bad.runs[0].path = path.into();
        rewrite_index(&tmp, &bad);
        assert!(ArchiveReader::open(&index_path(&tmp)).is_err(), "{path}");
    }
    let mut bad = original;
    bad.runs[1].path = bad.runs[0].path.clone();
    rewrite_index(&tmp, &bad);
    assert!(ArchiveReader::open(&index_path(&tmp)).is_err());
}
#[test]
fn reader_bounds_index_and_raw_and_checks_terminal_total() {
    let tmp = complete_archive();
    let original = load_index(&tmp);
    let p = index_path(&tmp);
    fs::write(
        &p,
        vec![b' '; candidate().unwrap().metadata_limit as usize + 1],
    )
    .unwrap();
    assert!(ArchiveReader::open(&p).is_err());
    rewrite_index(&tmp, &original);
    let raw = tmp.path().join("archive").join(&original.runs[0].path);
    let bytes = fs::read(&raw).unwrap();
    fs::write(
        &raw,
        vec![b' '; candidate().unwrap().raw_record_limit as usize + 1],
    )
    .unwrap();
    assert!(ArchiveReader::open(&p).unwrap().next().is_err());
    fs::write(raw, bytes).unwrap();
    let mut bad = original;
    bad.raw_bytes += 1;
    rewrite_index(&tmp, &bad);
    let mut r = ArchiveReader::open(&p).unwrap();
    for _ in &bad.runs {
        assert!(r.next().unwrap().is_some());
    }
    assert!(r.next().is_err());
}
#[cfg(unix)]
#[test]
fn reader_and_writer_refuse_symlink_files_directories_and_nonfiles() {
    use std::os::unix::fs::symlink;
    let tmp = complete_archive();
    let root = tmp.path().join("archive");
    let index = load_index(&tmp);
    let alias = tmp.path().join("alias");
    symlink(&root, &alias).unwrap();
    assert!(ArchiveReader::open(&alias.join("index.json")).is_err());
    let m = candidate().unwrap();
    assert!(ArchiveWriter::create(
        &alias,
        &m,
        CollectionMode::Construction,
        &test_provenance(),
        "test",
        limits(&m)
    )
    .is_err());
    let raw = root.join(&index.runs[0].path);
    let saved = tmp.path().join("saved.json");
    fs::rename(&raw, &saved).unwrap();
    symlink(&saved, &raw).unwrap();
    assert!(ArchiveReader::open(&index_path(&tmp)).is_err());
    fs::remove_file(&raw).unwrap();
    fs::create_dir(&raw).unwrap();
    assert!(ArchiveReader::open(&index_path(&tmp)).is_err());
    fs::remove_dir(&raw).unwrap();
    fs::rename(&saved, &raw).unwrap();
    let dir = root.join("raw/route.straight.paid");
    let saved_dir = tmp.path().join("saved-dir");
    fs::rename(&dir, &saved_dir).unwrap();
    symlink(&saved_dir, &dir).unwrap();
    assert!(ArchiveReader::open(&index_path(&tmp)).is_err());
    fs::remove_file(&dir).unwrap();
    fs::rename(&saved_dir, &dir).unwrap();
    fs::rename(index_path(&tmp), &saved).unwrap();
    symlink(&saved, index_path(&tmp)).unwrap();
    assert!(ArchiveReader::open(&index_path(&tmp)).is_err());
}

#[test]
fn bounded_metadata_accepts_exact_bytes_and_preflight_rejects_oversized_approval() {
    let value = "receipt";
    let expected = b"\"receipt\"";
    assert_eq!(
        encode_metadata(&value, expected.len() as u64).unwrap(),
        expected
    );
    assert!(encode_metadata(&value, expected.len() as u64 - 1).is_err());
    let tmp = owned_tempdir();
    let out = tmp.path().join("archive");
    let m = candidate().unwrap();
    let approval = "x".repeat(m.metadata_limit as usize);
    assert!(ArchiveWriter::create(
        &out,
        &m,
        CollectionMode::Construction,
        &test_provenance(),
        &approval,
        limits(&m)
    )
    .is_err());
    assert!(!out.exists());
}
#[test]
fn reader_rejects_missing_records_and_invalid_utf8_json_or_unknown_index_fields() {
    let tmp = complete_archive();
    let original = fs::read(index_path(&tmp)).unwrap();
    for bytes in [
        vec![0xff],
        b"{".to_vec(),
        original.clone().into_iter().take(30).collect(),
    ] {
        fs::write(index_path(&tmp), bytes).unwrap();
        assert!(ArchiveReader::open(&index_path(&tmp)).is_err());
    }
    let mut unknown = original.clone();
    unknown.splice(1..1, b"\"extra\":0,".iter().copied());
    fs::write(index_path(&tmp), unknown).unwrap();
    assert!(ArchiveReader::open(&index_path(&tmp)).is_err());
    fs::write(index_path(&tmp), original).unwrap();
    let index = load_index(&tmp);
    fs::remove_file(tmp.path().join("archive").join(&index.runs[0].path)).unwrap();
    assert!(ArchiveReader::open(&index_path(&tmp)).is_err());
}
#[test]
fn writer_rejects_oversized_raw_and_failure_receipt_collisions() {
    let tmp = owned_tempdir();
    let out = tmp.path().join("archive");
    let mut w = writer(&out);
    let m = candidate().unwrap();
    let key = &expected_keys(&m, CollectionMode::Construction)[0];
    assert!(w
        .put(key, &vec![b' '; m.raw_record_limit as usize + 1])
        .is_err());
    assert!(!out.join(raw_path(key)).exists());
    fs::write(out.join("failure.json"), b"keep").unwrap();
    assert!(w.fail("oversized record").is_err());
    assert!(w.finish().is_err());
    assert_eq!(fs::read(out.join("failure.json")).unwrap(), b"keep");
}
#[cfg(unix)]
#[test]
fn writer_refuses_symlink_raw_and_progress_directories_before_receipting() {
    use std::os::unix::fs::symlink;
    let m = candidate().unwrap();
    let key = &expected_keys(&m, CollectionMode::Construction)[0];
    for progress in [false, true] {
        let tmp = owned_tempdir();
        let out = tmp.path().join("archive");
        let mut w = writer(&out);
        let path = if progress {
            out.join("progress")
        } else {
            out.join("raw/route.straight.paid")
        };
        let outside = tmp.path().join("outside");
        fs::create_dir(&outside).unwrap();
        fs::remove_dir(&path).unwrap();
        symlink(&outside, &path).unwrap();
        assert!(w
            .put(key, &encoded_fixture(&key.condition, key.seed).unwrap())
            .is_err());
        assert!(w.finish().is_err());
        assert!(fs::read_dir(outside).unwrap().next().is_none());
    }
}
