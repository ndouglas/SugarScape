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
pub(super) fn test_provenance() -> Provenance {
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
pub(super) fn encoded_fixture(id: &str, seed: u64) -> Result<Vec<u8>, String> {
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
pub(super) fn decode_fixture(id: &str, seed: u64) -> Result<WireEnvelope, String> {
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
