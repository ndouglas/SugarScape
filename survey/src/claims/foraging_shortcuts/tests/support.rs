use super::super::{
    manifest::{candidate, condition},
    wire::{decode_envelope, encode_envelope, WireEnvelope},
    CollectionMode, Provenance, RunKey,
};
use std::{
    collections::BTreeMap,
    sync::{Mutex, OnceLock},
};
use sugarscape_core::foraging::construction as core;
pub(in crate::claims::foraging_shortcuts) fn test_provenance() -> Provenance {
    Provenance {
        code_revision: "a".repeat(40),
        protocol_revision: "b".repeat(40),
        manifest_sha256: super::super::sha256(&super::super::manifest::manifest_bytes().unwrap()),
        collector_sha256: "c".repeat(64),
    }
}
pub(super) fn cached_candidate_episode(id: &str, seed: u64) -> Result<core::Episode, String> {
    if ![7, 8].contains(&seed) {
        return Err("construction fixture seed must be 7 or 8".into());
    }
    static CACHE: OnceLock<Mutex<BTreeMap<(String, u64), core::Episode>>> = OnceLock::new();
    let mut cache = CACHE
        .get_or_init(Default::default)
        .lock()
        .map_err(|e| e.to_string())?;
    let key = (id.to_owned(), seed);
    if let Some(e) = cache.get(&key) {
        return Ok(e.clone());
    }
    let m = candidate()?;
    let c = condition(&m, id)?;
    let e = core::run(c.setup.clone(), seed, c.options.clone()).map_err(|e| format!("{e:?}"))?;
    cache.insert(key, e.clone());
    Ok(e)
}
pub(in crate::claims::foraging_shortcuts) fn encoded_fixture(
    id: &str,
    seed: u64,
) -> Result<Vec<u8>, String> {
    encode_envelope(
        &RunKey {
            condition: id.into(),
            seed,
        },
        CollectionMode::Construction,
        &test_provenance(),
        &cached_candidate_episode(id, seed)?,
        4 * 1024 * 1024,
    )
}
pub(in crate::claims::foraging_shortcuts) fn decode_fixture(
    id: &str,
    seed: u64,
) -> Result<WireEnvelope, String> {
    decode_envelope(
        &encoded_fixture(id, seed)?,
        &RunKey {
            condition: id.into(),
            seed,
        },
        CollectionMode::Construction,
        &test_provenance(),
        4 * 1024 * 1024,
    )
}
pub(super) fn component_episode() -> core::Episode {
    let pos = |x, y| core::Pos { x, y };
    core::run(
        core::Setup {
            width: 3,
            height: 3,
            open: vec![pos(0, 0), pos(0, 1), pos(1, 1)],
            diggable: vec![pos(2, 1)],
            nest: vec![pos(0, 1), pos(1, 1)],
            waste: pos(0, 0),
            workers: vec![pos(1, 1)],
            food: vec![core::Resource {
                id: 0,
                pos: pos(2, 1),
            }],
            parameters: core::Parameters {
                p_search: 1.0,
                p_return: 0.0,
                lambda_fidelity: 0.0,
                lambda_publish: 0.0,
                lambda_waypoint: 0.0,
            },
        },
        7,
        core::RunOptions {
            ticks: 16,
            sample_every: 4,
            snapshots: true,
        },
    )
    .unwrap()
}
// A counter-consistent mutated body isolates impossible handling at a disconnected patch.
pub(super) fn disconnected_handling_fixture() -> super::super::wire::WireEpisode {
    use super::super::{wire::WirePos, wire_state::WireFoodState};
    let mut e: super::super::wire::WireEpisode =
        serde_json::from_slice(&serde_json::to_vec(&component_episode()).unwrap()).unwrap();
    let pos = WirePos { x: 4, y: 2 };
    e.setup.width = 5;
    e.setup.open.push(pos);
    e.setup.open.sort();
    e.setup.food[0].pos = pos;
    for f in &mut e.snapshots {
        f.open.push(pos);
        f.open.sort();
        f.summary.terrain.initial_open += 1;
        f.summary.terrain.open += 1;
        f.food[0].resource.pos = pos;
        if f.food[0].state == WireFoodState::Hidden {
            f.food[0].state = WireFoodState::Available;
            f.summary.food.hidden = 0;
            f.summary.food.available = 1;
        }
        for a in &mut f.agents {
            if let Some(find) = &mut a.find {
                find.site = pos;
            }
        }
        let r = &mut f.summary.access.records[0];
        r.initially_exposed = true;
        r.initially_accessible = false;
        r.first_exposure = None;
        r.first_access = None;
        r.accessible = false;
        r.distance = None;
        f.summary.access.initially_exposed = 1;
        f.summary.access.initially_accessible = 0;
        f.summary.access.accessible = 0;
        f.summary.milestones.first_exposure = None;
        f.summary.milestones.first_access = None;
    }
    e.summary = e.snapshots.last().unwrap().summary.clone();
    e
}

/// Owns one exclusively created test directory; never cleans up a pre-existing path.
pub(in crate::claims::foraging_shortcuts) struct OwnedTempdir(std::path::PathBuf);
impl OwnedTempdir {
    pub(in crate::claims::foraging_shortcuts) fn path(&self) -> &std::path::Path {
        &self.0
    }
}
impl Drop for OwnedTempdir {
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.0).expect("remove owned test directory");
    }
}
pub(in crate::claims::foraging_shortcuts) fn owned_tempdir() -> OwnedTempdir {
    use std::sync::atomic::{AtomicU64, Ordering};
    static NEXT: AtomicU64 = AtomicU64::new(0);
    loop {
        let path = std::env::temp_dir().join(format!(
            "sugarscape-f5-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        match std::fs::create_dir(&path) {
            Ok(()) => return OwnedTempdir(path),
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(e) => panic!("create owned test directory {}: {e}", path.display()),
        }
    }
}

pub(super) fn fixture_git(root: &std::path::Path, args: &[&str]) -> String {
    let output = std::process::Command::new("git")
        .current_dir(root)
        .args(args)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap().trim().to_owned()
}
pub(super) fn fixture_commit(root: &std::path::Path) -> String {
    fixture_git(root, &["add", "."]);
    fixture_git(
        root,
        &[
            "-c",
            "user.name=F5 Test",
            "-c",
            "user.email=f5-test@example.invalid",
            "commit",
            "--quiet",
            "-m",
            "owned construction fixture",
        ],
    );
    fixture_git(root, &["rev-parse", "HEAD"])
}
pub(super) fn owned_git_context() -> (
    OwnedTempdir,
    super::super::run::ExecutionContext,
    super::super::run::RunRequest,
) {
    use super::super::{
        manifest::manifest_bytes,
        run::{ExecutionContext, RunRequest, MANIFEST_PATH},
    };
    let tmp = owned_tempdir();
    let m = candidate().unwrap();
    fixture_git(tmp.path(), &["init", "--quiet"]);
    let protocol = tmp.path().join(&m.protocol);
    std::fs::create_dir_all(protocol.parent().unwrap()).unwrap();
    std::fs::write(
        &protocol,
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../docs/superpowers/specs/2026-10-07-foraging-5-shortcut-comparison-design.md"
        )),
    )
    .unwrap();
    std::fs::write(tmp.path().join(MANIFEST_PATH), manifest_bytes().unwrap()).unwrap();
    let revision = fixture_commit(tmp.path());
    let executable = tmp.path().join("fixture-collector");
    std::fs::write(&executable, b"owned executable identity fixture").unwrap();
    let context = ExecutionContext {
        repo: tmp.path().to_owned(),
        executable,
    };
    let request = RunRequest {
        mode: CollectionMode::Construction,
        protocol_revision: revision,
        approval_context: "approved engineering construction fixture".into(),
        out: tmp.path().join("new-archive"),
    };
    (tmp, context, request)
}

pub(super) struct TestArchiveFixture {
    pub(super) temp: OwnedTempdir,
    pub(super) index: std::path::PathBuf,
}
/// Pure byte-writing tests reuse core episode cache, but use real owned provenance.
fn construction_archive_bytes() -> Vec<(std::path::PathBuf, Vec<u8>)> {
    use super::super::{
        archive::{limits, ArchiveWriter},
        manifest::expected_keys,
        run::preflight,
    };
    let (temp, context, request) = owned_git_context();
    let manifest = candidate().unwrap();
    let provenance = preflight(&context, &manifest, &request).unwrap();
    let mut writer = ArchiveWriter::create(
        &request.out,
        &manifest,
        request.mode,
        &provenance,
        &request.approval_context,
        limits(&manifest),
    )
    .unwrap();
    for key in expected_keys(&manifest, request.mode) {
        let episode = cached_candidate_episode(&key.condition, key.seed).unwrap();
        let bytes = encode_envelope(
            &key,
            request.mode,
            &provenance,
            &episode,
            manifest.raw_record_limit,
        )
        .unwrap();
        writer.put(&key, &bytes).unwrap();
    }
    let index = writer.finish().unwrap();
    let mut files = vec![(
        std::path::PathBuf::from("index.json"),
        std::fs::read(request.out.join("index.json")).unwrap(),
    )];
    for reference in index.runs {
        files.push((
            std::path::PathBuf::from(&reference.path),
            std::fs::read(request.out.join(reference.path)).unwrap(),
        ));
    }
    drop(temp);
    files
}

/// Each mutation owns a fresh copy of immutable evidence validated once at creation.
pub(super) fn complete_construction_archive() -> TestArchiveFixture {
    type SavedFiles = Vec<(std::path::PathBuf, Vec<u8>)>;
    static CACHE: OnceLock<SavedFiles> = OnceLock::new();
    let files = CACHE.get_or_init(construction_archive_bytes);
    let temp = owned_tempdir();
    let root = temp.path().join("saved");
    for (relative, bytes) in files {
        let path = root.join(relative);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, bytes).unwrap();
    }
    TestArchiveFixture {
        index: root.join("index.json"),
        temp,
    }
}
