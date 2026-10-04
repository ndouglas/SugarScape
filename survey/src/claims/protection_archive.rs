//! Immutable raw envelopes and strict saved-record validation.
use super::protection::{manifest, Manifest, SCHEMA};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    io::Write,
    path::{Component, Path},
    process::Command,
    time::{Instant, SystemTime, UNIX_EPOCH},
};
use sugarscape_core::{
    geometry::Pos,
    minds::protection::{
        ledger::{Ledger, Outflow},
        runner::{ActionRecord, EpisodeRecord, FrameRecord},
        state::{Fixture, LabConfig, Policy},
    },
};
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RawRef {
    pub condition: String,
    pub seed: u64,
    pub path: String,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Index {
    pub schema: String,
    pub code_revision: String,
    pub protocol_revision: String,
    pub manifest: Manifest,
    pub completed: bool,
    pub runs: Vec<RawRef>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Envelope {
    pub schema: String,
    pub condition: String,
    pub seed: u64,
    pub code_revision: String,
    pub protocol_revision: String,
    pub started_unix_ms: u128,
    pub elapsed_seconds: f64,
    pub timing_boundary: String,
    pub record: EpisodeRecord,
}
pub fn validate_revision(value: &str) -> Result<(), String> {
    if value.len() != 40 || !value.bytes().all(|b| b.is_ascii_hexdigit()) {
        Err("revision must be a full 40-hex commit".into())
    } else {
        Ok(())
    }
}
pub fn write_new(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|e| format!("{}: {e}", path.display()))?;
    file.write_all(bytes)
        .and_then(|_| file.sync_all())
        .map_err(|e| e.to_string())
}
pub fn new_directory(path: &Path) -> Result<(), String> {
    fs::create_dir(path).map_err(|e| format!("new output directory {}: {e}", path.display()))
}
fn git(root: &Path, args: &[&str]) -> Result<String, String> {
    let output = Command::new("git")
        .args(args)
        .current_dir(root)
        .output()
        .map_err(|e| e.to_string())?;
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).into());
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().into())
}
fn raw_path(ordinal: usize, seed: u64) -> String {
    format!("raw/{ordinal:03}-{seed}.json")
}
fn preflight(root: &Path, revision: &str, out: &Path) -> Result<String, String> {
    validate_revision(revision)?;
    let code = git(root, &["rev-parse", "HEAD"])?;
    if git(
        root,
        &["rev-parse", "--verify", &format!("{revision}^{{commit}}")],
    )? != revision
    {
        return Err("protocol revision must name the exact full commit".into());
    }
    let protocol = "docs/superpowers/specs/2026-10-02-minds-protection-protocol.md";
    if git(root, &["show", &format!("{revision}:{protocol}")])?
        != fs::read_to_string(root.join(protocol))
            .map_err(|e| e.to_string())?
            .trim()
    {
        return Err("committed protocol differs from current protocol".into());
    }
    if !git(root, &["status", "--porcelain", "--untracked-files=no"])?.is_empty() {
        return Err("scientific execution requires a clean tracked tree".into());
    }
    if out.exists() {
        return Err("output directory already exists".into());
    }
    Ok(code)
}
pub fn run(revision: &str, out: &Path) -> Result<(), String> {
    let code = preflight(
        Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/..")),
        revision,
        out,
    )?;
    new_directory(out)?;
    fs::create_dir(out.join("raw")).map_err(|e| e.to_string())?;
    let mut index = Index {
        schema: SCHEMA.into(),
        code_revision: code,
        protocol_revision: revision.into(),
        manifest: manifest(),
        completed: false,
        runs: vec![],
    };
    // This immutable initial index records interrupted execution, never resumes it.
    write_new(
        &out.join("index.incomplete.json"),
        &serde_json::to_vec_pretty(&index).map_err(|e| e.to_string())?,
    )?;
    for (ordinal, condition) in index.manifest.conditions.iter().enumerate() {
        for &seed in &index.manifest.seeds {
            let started_unix_ms = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_err(|e| e.to_string())?
                .as_millis();
            let start = Instant::now();
            let record = sugarscape_core::minds::protection::runner::run_episode(
                condition.lab.clone(),
                seed,
                true,
            )?;
            let envelope = Envelope {
                schema: SCHEMA.into(),
                condition: condition.id.clone(),
                seed,
                code_revision: index.code_revision.clone(),
                protocol_revision: index.protocol_revision.clone(),
                started_unix_ms,
                elapsed_seconds: start.elapsed().as_secs_f64(),
                timing_boundary: "world construction and completed biological steps; excludes file I/O and analysis".into(),
                record,
            };
            let path = raw_path(ordinal, seed);
            write_new(
                &out.join(&path),
                &serde_json::to_vec(&envelope).map_err(|e| e.to_string())?,
            )?;
            index.runs.push(RawRef {
                condition: condition.id.clone(),
                seed,
                path,
            });
        }
    }
    index.completed = true;
    let records = load_records(out, &index)?;
    validate_archive(&index, &records)?;
    write_new(
        &out.join("index.json"),
        &serde_json::to_vec_pretty(&index).map_err(|e| e.to_string())?,
    )?;
    let analysis = super::protection_report::analyze(&index, &records)?;
    write_new(
        &out.join("analysis.json"),
        &serde_json::to_vec_pretty(&analysis).map_err(|e| e.to_string())?,
    )?;
    write_new(
        &out.join("results.md"),
        super::protection_report::render_results(&analysis).as_bytes(),
    )
}
pub fn load(path: &Path) -> Result<(Index, Vec<EpisodeRecord>), String> {
    let index: Index = serde_json::from_slice(&fs::read(path).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())?;
    validate_index(&index)?;
    let records = load_records(path.parent().unwrap_or(Path::new(".")), &index)?;
    validate_archive(&index, &records)?;
    Ok((index, records))
}
fn load_records(root: &Path, index: &Index) -> Result<Vec<EpisodeRecord>, String> {
    let root = root.canonicalize().map_err(|e| e.to_string())?;
    index
        .runs
        .iter()
        .map(|raw| {
            safe_path(&raw.path)?;
            let path = root
                .join(&raw.path)
                .canonicalize()
                .map_err(|e| format!("missing raw {}: {e}", raw.path))?;
            if !path.starts_with(&root) {
                return Err("raw path escapes archive".into());
            }
            let e: Envelope = serde_json::from_slice(&fs::read(path).map_err(|e| e.to_string())?)
                .map_err(|e| e.to_string())?;
            if e.schema != SCHEMA
                || e.condition != raw.condition
                || e.seed != raw.seed
                || e.code_revision != index.code_revision
                || e.protocol_revision != index.protocol_revision
                || !e.elapsed_seconds.is_finite()
                || e.elapsed_seconds < 0.0
                || e.timing_boundary != "world construction and completed biological steps; excludes file I/O and analysis"
            {
                return Err("raw envelope identity/provenance/timing mismatch".into());
            }
            Ok(e.record)
        })
        .collect()
}
fn safe_path(path: &str) -> Result<(), String> {
    if path.is_empty()
        || Path::new(path)
            .components()
            .any(|c| !matches!(c, Component::Normal(_)))
    {
        Err("raw path must be a relative confined path".into())
    } else {
        Ok(())
    }
}
fn validate_index(index: &Index) -> Result<(), String> {
    validate_revision(&index.code_revision)?;
    validate_revision(&index.protocol_revision)?;
    if index.schema != SCHEMA || index.manifest != manifest() || !index.completed {
        return Err("archive needs complete registered manifest and schema".into());
    }
    let mut expected = BTreeMap::new();
    for (ordinal, c) in index.manifest.conditions.iter().enumerate() {
        for &seed in &index.manifest.seeds {
            expected.insert((c.id.clone(), seed), raw_path(ordinal, seed));
        }
    }
    if index.runs.len() != expected.len() {
        return Err("archive is missing registered runs".into());
    }
    for raw in &index.runs {
        safe_path(&raw.path)?;
        if expected
            .remove(&(raw.condition.clone(), raw.seed))
            .as_deref()
            != Some(raw.path.as_str())
        {
            return Err("duplicate/unregistered run or wrong raw path".into());
        }
    }
    Ok(())
}
pub fn validate_archive(index: &Index, records: &[EpisodeRecord]) -> Result<(), String> {
    validate_index(index)?;
    if records.len() != index.runs.len() {
        return Err("raw record count mismatch".into());
    }
    let configs: BTreeMap<_, _> = index
        .manifest
        .conditions
        .iter()
        .map(|c| (c.id.as_str(), &c.lab))
        .collect();
    for (raw, record) in index.runs.iter().zip(records) {
        if raw.seed != record.seed || configs[raw.condition.as_str()] != &record.lab {
            return Err("raw episode identity/configuration mismatch".into());
        }
        validate_episode(record)
            .map_err(|e| format!("{} seed {}: {e}", raw.condition, raw.seed))?;
    }
    Ok(())
}
fn near(a: f64, b: f64) -> bool {
    a.is_finite() && b.is_finite() && (a - b).abs() <= 1e-9 * a.abs().max(b.abs()).max(1.0)
}
fn pos(lab: &LabConfig, x: u32, y: u32) -> Pos {
    Pos::new(if lab.mirrored { 8 - x } else { x }, y)
}
fn site(p: Pos) -> u32 {
    p.y * 9 + p.x
}
fn role(
    frame: &FrameRecord,
    id: u64,
) -> Option<&sugarscape_core::minds::protection::runner::RoleRecord> {
    frame.roles.iter().find(|r| r.id == id)
}
/// Supplied positions are checked independently of the engine and of fixture_errors.
fn observer_positions(lab: &LabConfig) -> Vec<Pos> {
    let initial = match lab.fixture {
        Fixture::Single {
            initial_observed, ..
        }
        | Fixture::Stumble { initial_observed } => initial_observed,
        _ => true,
    };
    let later = matches!(
        lab.fixture,
        Fixture::Single {
            redeposit_observed: true,
            ..
        }
    );
    let mut route = match lab.fixture {
        Fixture::Mixed { .. } => vec![
            (3, 5),
            (3, 5),
            (3, 5),
            (3, 5),
            (4, 5),
            (5, 5),
            (6, 5),
            (7, 5),
            (7, 6),
        ],
        Fixture::CueUnseenWatcher => vec![(3, 6), (3, 6), (4, 6), (5, 6), (6, 6), (7, 6)],
        _ => {
            let mut p = vec![if initial { (3, 5) } else { (7, 5) }];
            p.push(p[0]);
            p.extend(match (initial, later) {
                (true, true) => vec![(3, 6)],
                (true, false) => vec![(4, 5), (5, 5), (6, 5), (7, 5), (7, 6)],
                (false, true) => vec![(6, 5), (5, 5), (4, 5), (3, 5), (3, 6)],
                (false, false) => vec![(7, 6)],
            });
            p
        }
    };
    let hold = match lab.fixture {
        Fixture::Mixed { .. } => 20,
        Fixture::Stumble { .. } => 18,
        _ => 12,
    };
    route.resize(hold + 1, *route.last().unwrap());
    if matches!(lab.fixture, Fixture::Stumble { .. }) {
        route.extend([
            (6, 6),
            (5, 6),
            (4, 6),
            (3, 6),
            (3, 5),
            (3, 4),
            (3, 3),
            (3, 2),
        ]);
    }
    route.into_iter().map(|(x, y)| pos(lab, x, y)).collect()
}
pub(super) fn validate_episode(r: &EpisodeRecord) -> Result<(), String> {
    let fail = |message: &str| Err(message.to_string());
    if r.schema != SCHEMA
        || r.requested_ticks != 64
        || r.completed_ticks > 64
        || r.completed_ticks < 8
        || r.frames.len() != r.completed_ticks as usize + 1
    {
        return fail("inconsistent schema/horizon/frame counts");
    }
    if !r.fixture_errors.is_empty() {
        return fail("fixture-invalid episode");
    }
    let lab = &r.lab;
    let mixed = matches!(lab.fixture, Fixture::Mixed { .. });
    let initial = &r.frames[0];
    let owner_start = pos(
        lab,
        if matches!(
            lab.fixture,
            Fixture::Mixed {
                observed_first: false
            }
        ) {
            5
        } else {
            3
        },
        3,
    );
    let route = observer_positions(lab);
    if initial.roles.len() != 2
        || role(initial, 1).is_none_or(|a| {
            a.pos != owner_start
                || a.holdings != 44.0
                || !a.caches.is_empty()
                || a.protection.as_ref().is_none_or(|s| {
                    !s.sources.is_empty() || !s.exposure.entries.is_empty() || s.intent.is_some()
                })
        })
        || role(initial, 2).is_none_or(|a| {
            a.pos != route[0]
                || a.holdings != 96.0
                || !a.caches.is_empty()
                || a.protection.is_some()
        })
        || initial.roles.iter().any(|a| !a.seen.is_empty())
        || !initial.actions.is_empty()
        || !initial.deaths.is_empty()
        || initial.relocation != Default::default()
        || !initial.restriction_ticks.is_empty()
    {
        return fail("wrong initial roles/endowments/memory");
    }
    let mut reconstructed = Ledger::new(1, 44.0);
    let mut restrictions = BTreeMap::<u64, u64>::new();
    let mut harvest_total = 0.0;
    for (i, f) in r.frames.iter().enumerate() {
        if f.tick != i as u64
            || f.fingerprint.len() != 16
            || !f.fingerprint.bytes().all(|b| b.is_ascii_hexdigit())
            || !near(
                f.current_bury_cost,
                if i <= 8 { 0.0 } else { lab.reburial_cost },
            )
        {
            return fail("invalid frame tick/fingerprint/cost schedule");
        }
        let live: BTreeSet<_> = f.roles.iter().map(|a| a.id).collect();
        if live.len() != f.roles.len() || live.iter().any(|id| ![1, 2].contains(id)) {
            return fail("invalid/duplicate living role");
        }
        for a in &f.roles {
            if !(1..=7).contains(&a.pos.x)
                || !(1..=7).contains(&a.pos.y)
                || !a.holdings.is_finite()
                || a.holdings <= 0.0
                || a.caches
                    .iter()
                    .any(|(&s, &q)| s >= 81 || !q.is_finite() || q < 0.0)
                || a.seen.iter().any(|e| {
                    e.site >= 81
                        || e.owner != 1
                        || e.tick >= f.tick
                        || !e.amount.is_finite()
                        || e.amount <= 0.0
                })
            {
                return fail("invalid role balance/position/sighting");
            }
            if a.id == 2 && (!a.caches.is_empty() || a.protection.is_some()) {
                return fail("observer acquired owner state");
            }
        }
        if [
            f.relocation.withdrawn,
            f.relocation.redeposited,
            f.relocation.burial_cost,
        ]
        .iter()
        .any(|q| !q.is_finite() || *q < 0.0)
            || f.relocation.source_events.iter().any(|e| {
                !e.withdrawn.is_finite()
                    || e.withdrawn < 0.0
                    || !e.redeposited.is_finite()
                    || e.redeposited < 0.0
            })
        {
            return fail("invalid relocation diagnostic");
        }
        if !near(
            f.relocation.withdrawn,
            f.relocation.source_events.iter().map(|e| e.withdrawn).sum(),
        ) || !near(
            f.relocation.redeposited,
            f.relocation
                .source_events
                .iter()
                .map(|e| e.redeposited)
                .sum(),
        ) || f.relocation.starts as usize
            != f.relocation
                .source_events
                .iter()
                .filter(|e| e.started)
                .count()
        {
            return fail("source relocation aggregate mismatch");
        }
        if f.roles.len() == 2 && f.roles[0].pos == f.roles[1].pos {
            return fail("roles occupy same site");
        }
        if i == 0 {
            continue;
        }
        let prev = &r.frames[i - 1];
        let prior: BTreeSet<_> = prev.roles.iter().map(|a| a.id).collect();
        let actors: BTreeSet<_> = f.actions.iter().map(|a| a.id).collect();
        let deaths: BTreeSet<_> = f.deaths.iter().map(|a| a.id).collect();
        if f.deaths
            .iter()
            .any(|d| !(1..=7).contains(&d.pos.x) || !(1..=7).contains(&d.pos.y))
        {
            return fail("invalid death position");
        }
        if actors != prior
            || actors.len() != f.actions.len()
            || deaths.len() != f.deaths.len()
            || deaths != prior.difference(&live).copied().collect()
            || !live.is_subset(&prior)
        {
            return fail("actions/deaths do not reconcile living roles");
        }
        let protective: Vec<_> = f
            .actions
            .iter()
            .filter(|a| a.phase == "relocation")
            .collect();
        if !near(
            f.relocation.withdrawn,
            protective.iter().map(|a| a.gross_dug).sum(),
        ) || !near(
            f.relocation.redeposited,
            protective.iter().map(|a| a.gross_buried).sum(),
        ) || !near(
            f.relocation.burial_cost,
            protective.iter().map(|a| a.burial_cost).sum(),
        ) {
            return fail("relocation diagnostics disagree with actions");
        }
        validate_effort(prev, f)?;
        for event in &f.relocation.source_events {
            if !f
                .actions
                .iter()
                .any(|a| a.id == 1 && a.phase == "relocation" && a.source == Some(event.source))
            {
                return fail("source event without matching protective action");
            }
        }
        let mut predicted = role(prev, 1).map(|a| a.caches.clone()).unwrap_or_default();
        let mut holdings: BTreeMap<_, _> = prev.roles.iter().map(|a| (a.id, a.holdings)).collect();
        for a in &f.actions {
            if [
                a.harvest,
                a.metabolic_demand,
                a.metabolic_consumed,
                a.gross_dug,
                a.gross_buried,
                a.burial_cost,
                a.raid_amount,
                a.discovery_amount,
            ]
            .iter()
            .any(|q| !q.is_finite() || *q < 0.0)
                || a.metabolic_demand != 1.0
                || a.metabolic_consumed > 1.0
                || a.harvest > 4.0
                || a.actual_watchers.iter().any(|&id| id != 2)
                || a.actual_watchers.len() > 1
            {
                return fail("invalid action outcomes/metabolism/watchers");
            }
            if a.source.is_some_and(|s| s >= 81)
                || a.raid_site.is_some_and(|s| s >= 81)
                || a.discovery_site.is_some_and(|s| s >= 81)
                || a.target
                    .is_some_and(|p| p.x == 0 || p.x >= 8 || p.y == 0 || p.y >= 8)
            {
                return fail("invalid action location");
            }
            let before = role(prev, a.id).unwrap();
            let after = role(f, a.id)
                .map(|r| r.pos)
                .or_else(|| f.deaths.iter().find(|d| d.id == a.id).map(|d| d.pos))
                .unwrap();
            validate_action_markers(lab, a, after)?;
            if before.pos.x.abs_diff(after.x) + before.pos.y.abs_diff(after.y) > 1 {
                return fail("speed-one movement violation");
            }
            let tick = i - 1;
            if a.id == 1 && tick < 8 {
                let deposit = if tick == 0 || (mixed && tick == 2) {
                    if mixed {
                        6.0
                    } else {
                        12.0
                    }
                } else {
                    0.0
                };
                let expected = if mixed {
                    match tick {
                        1 | 3 => pos(lab, 4, 3),
                        2 => pos(
                            lab,
                            if matches!(
                                lab.fixture,
                                Fixture::Mixed {
                                    observed_first: true
                                }
                            ) {
                                5
                            } else {
                                3
                            },
                            3,
                        ),
                        4.. => pos(lab, 4, 3),
                        _ => owner_start,
                    }
                } else {
                    owner_start
                };
                let walking = mixed && (1..=3).contains(&tick);
                if a.phase != "preparation"
                    || a.action
                        != if deposit > 0.0 {
                            "prepare_deposit"
                        } else if walking {
                            "walk"
                        } else {
                            "hold"
                        }
                    || after != expected
                    || a.gross_buried != deposit
                    || a.target != walking.then_some(expected)
                    || a.harvest != 0.0
                    || a.gross_dug != 0.0
                {
                    return fail("owner preparation schedule mismatch");
                }
                if deposit > 0.0 {
                    let observed = if mixed {
                        after == pos(lab, 3, 3)
                    } else {
                        !matches!(
                            lab.fixture,
                            Fixture::Single {
                                initial_observed: false,
                                ..
                            } | Fixture::Stumble {
                                initial_observed: false
                            } | Fixture::CueUnseenWatcher
                        )
                    };
                    let seen = if mixed {
                        after == pos(lab, 3, 3)
                    } else {
                        !matches!(
                            lab.fixture,
                            Fixture::Single {
                                initial_observed: false,
                                ..
                            } | Fixture::Stumble {
                                initial_observed: false
                            } | Fixture::CueVisibleNonwatcher
                        )
                    };
                    if a.source != Some(site(after))
                        || a.perceived_exposure != Some(observed)
                        || a.actual_watchers.contains(&2) != seen
                    {
                        return fail("invalid preparation cue/sighting/source");
                    }
                }
            }
            if a.id == 2 && i < route.len() {
                let encounter = matches!(lab.fixture, Fixture::Stumble { .. }) && tick >= 18;
                let walking = route[i] != route[i - 1];
                if after != route[i]
                    || a.target != walking.then_some(route[i])
                    || a.phase
                        != if encounter {
                            "encounter"
                        } else if tick < 8 {
                            "preparation"
                        } else {
                            "observer_hold"
                        }
                    || a.action != if walking { "walk" } else { "hold" }
                    || (!encounter
                        && (a.harvest != 0.0 || a.raid_amount != 0.0 || a.discovery_amount != 0.0))
                {
                    return fail("observer route/release/encounter schedule mismatch");
                }
            } else if a.id == 2 && (a.phase != "ordinary" || a.action != "foraging") {
                return fail("observer ordinary release mismatch");
            }
            if a.phase == "relocation" {
                if a.id != 1
                    || tick < 8
                    || matches!(lab.policy, Policy::Off | Policy::Erased)
                    || a.harvest != 0.0
                    || a.raid_amount != 0.0
                    || a.discovery_amount != 0.0
                    || !["walk", "retrieve", "redeposit", "cancel"].contains(&a.action.as_str())
                    || a.source.is_none()
                {
                    return fail("invalid protective action");
                }
                if a.action == "walk" && (a.gross_dug != 0.0 || a.gross_buried != 0.0) {
                    return fail("protective walk gathered food");
                }
                if a.gross_buried > 0.0 && a.target != Some(after) {
                    return fail(
                        "redeposit target missing or different from actual action-end position",
                    );
                }
                if a.gross_buried > 0.0 && !mixed {
                    let expected = matches!(
                        lab.fixture,
                        Fixture::Single {
                            redeposit_observed: true,
                            ..
                        }
                    );
                    if a.actual_watchers.contains(&2) != expected
                        || a.perceived_exposure != Some(false)
                        || after != pos(lab, 3, 2)
                    {
                        return fail("redeposit cue/sighting/destination mismatch");
                    }
                }
            } else if a.id == 1
                && tick >= 8
                && (a.phase != "ordinary" || a.action != "foraging" || a.gross_buried != 0.0)
            {
                return fail("invalid owner policy phase");
            }
            if a.phase == "preparation" || a.phase == "observer_hold" {
                *restrictions.entry(a.id).or_default() += 1;
            }
            harvest_total += a.harvest;
            if a.id == 1 {
                if a.raid_amount != 0.0 || a.discovery_amount != 0.0 {
                    return fail("owner pilfered");
                }
                reconstructed.harvest(a.harvest)?;
                if a.gross_dug > 0.0 {
                    reconstructed.withdraw(site(after), a.gross_dug)?;
                    let cache = predicted.entry(site(after)).or_default();
                    *cache -= a.gross_dug;
                }
                if a.gross_buried > 0.0 {
                    if a.action == "prepare_deposit" {
                        reconstructed.prepare(site(after), a.gross_buried)?;
                    } else {
                        reconstructed
                            .outflow(a.gross_buried, Outflow::Deposit { site: site(after) })?;
                    }
                    *predicted.entry(site(after)).or_default() += a.gross_buried;
                }
                reconstructed.outflow(a.burial_cost, Outflow::BurialCost)?;
                reconstructed.outflow(a.metabolic_consumed, Outflow::Consumption)?;
            } else {
                if a.gross_dug != 0.0 || a.gross_buried != 0.0 || a.burial_cost != 0.0 {
                    return fail("observer performed owner food operation");
                }
                for (location, q) in [
                    (a.raid_site, a.raid_amount),
                    (a.discovery_site, a.discovery_amount),
                ] {
                    if q > 0.0 {
                        let location = location.ok_or("transfer without site")?;
                        if location != site(after) {
                            return fail("remote food transfer");
                        }
                        reconstructed.pilfer(location, q)?;
                        *predicted.entry(location).or_default() -= q;
                    }
                }
            }
            if !near(a.burial_cost, a.gross_buried * f.current_bury_cost) {
                return fail("burial cost mismatch");
            }
            let pre_metabolism =
                holdings[&a.id] + a.harvest + a.gross_dug + a.raid_amount + a.discovery_amount
                    - a.gross_buried
                    - a.burial_cost;
            if !near(a.metabolic_consumed, pre_metabolism.clamp(0.0, 1.0)) {
                return fail("actual consumption mismatch");
            }
            holdings.insert(a.id, pre_metabolism - 1.0);
            if deaths.contains(&a.id) {
                let death = f.deaths.iter().find(|d| d.id == a.id).unwrap();
                if holdings[&a.id] > 0.0 || death.cause != "starvation" {
                    return fail("invalid death event");
                }
                if a.id == 1 {
                    reconstructed.lose_owner();
                    predicted.clear();
                }
            }
        }
        if harvest_total > if mixed { 0.0 } else { 8.0 } + 1e-9
            || f.restriction_ticks != restrictions
        {
            return fail("harvest/restriction balance mismatch");
        }
        for a in &f.roles {
            if !near(a.holdings, holdings[&a.id]) {
                return fail("holdings disagree with action balances");
            }
        }
        if let Some(owner) = role(f, 1) {
            for location in predicted.keys().chain(owner.caches.keys()) {
                if !near(
                    *predicted.get(location).unwrap_or(&0.0),
                    *owner.caches.get(location).unwrap_or(&0.0),
                ) {
                    return fail("cache disagrees with recorded transfers");
                }
            }
            physical(&reconstructed, owner.holdings, &owner.caches)?;
        }
        reconstructed.reconcile()?;
        if i == 8 {
            let owner = role(f, 1).ok_or("owner died during preparation")?;
            if !near(owner.holdings, 24.0) || !near(owner.caches.values().sum(), 12.0) {
                return fail("preparation endowment mismatch");
            }
        }
        if let Some(owner) = role(f, 1) {
            let state = owner.protection.as_ref().ok_or("missing owner state")?;
            if state.intent.as_ref().is_some_and(|intent| {
                !intent.amount.is_finite()
                    || intent.amount < 0.0
                    || !state.sources.contains_key(&intent.source)
                    || !(1..=7).contains(&intent.destination.x)
                    || !(1..=7).contains(&intent.destination.y)
                    || state.sources.contains_key(&site(intent.destination))
            }) {
                return fail("invalid protective intent");
            }
            if lab.policy == Policy::Selective
                && state.sources.iter().any(|(source, metadata)| {
                    metadata.attempted
                        && state
                            .exposure
                            .entries
                            .get(source)
                            .is_none_or(|e| !e.exposed)
                })
            {
                return fail("selective source contradicts exposure cue");
            }
            let expected: BTreeMap<_, _> = if mixed {
                BTreeMap::from([
                    (
                        site(pos(lab, 3, 3)),
                        (
                            if matches!(
                                lab.fixture,
                                Fixture::Mixed {
                                    observed_first: true
                                }
                            ) {
                                0
                            } else {
                                2
                            },
                            6.0,
                            true,
                        ),
                    ),
                    (
                        site(pos(lab, 5, 3)),
                        (
                            if matches!(
                                lab.fixture,
                                Fixture::Mixed {
                                    observed_first: true
                                }
                            ) {
                                2
                            } else {
                                0
                            },
                            6.0,
                            false,
                        ),
                    ),
                ])
            } else {
                BTreeMap::from([(
                    site(pos(lab, 3, 3)),
                    (
                        0,
                        12.0,
                        !matches!(
                            lab.fixture,
                            Fixture::Single {
                                initial_observed: false,
                                ..
                            } | Fixture::Stumble {
                                initial_observed: false
                            } | Fixture::CueUnseenWatcher
                        ),
                    ),
                )])
            };
            let expected: BTreeMap<_, _> = expected
                .into_iter()
                .filter(|(_, (tick, _, _))| *tick < i as u64)
                .collect();
            if state.sources.len() != expected.len()
                || state.sources.iter().any(|(s, source)| {
                    expected.get(s).is_none_or(|(tick, q, _)| {
                        source.tick != *tick || source.initial_amount != *q
                    })
                })
            {
                return fail("prepared source metadata mismatch");
            }
            if lab.policy == Policy::Erased && i >= 9 {
                if !state.exposure.entries.is_empty() {
                    return fail("erasure intervention mismatch");
                }
            } else if state.exposure.entries.len() != expected.len()
                || state.exposure.entries.iter().any(|(s, e)| {
                    expected
                        .get(s)
                        .is_none_or(|(tick, _, cue)| e.tick != *tick || e.exposed != *cue)
                })
            {
                return fail("owner exposure cue mismatch");
            }
            if matches!(lab.policy, Policy::Off | Policy::Erased)
                && (state.intent.is_some() || state.sources.values().any(|s| s.attempted))
            {
                return fail("control started protection");
            }
        }
    }
    if !mixed {
        let release = if matches!(lab.fixture, Fixture::Stumble { .. }) {
            18
        } else {
            12
        };
        for action in r
            .frames
            .iter()
            .flat_map(|f| &f.actions)
            .filter(|a| a.phase == "relocation" && a.gross_buried > 0.0)
        {
            let destination = action.target.ok_or("redeposit missing destination")?;
            if r.frames
                .get(release)
                .is_none_or(|f| f.roles.iter().any(|role| role.pos == destination))
            {
                return fail("redeposit blocked at supplied opportunity release");
            }
        }
    }
    let actual_alive = r.frames.last().unwrap().roles.iter().any(|a| a.id == 1);
    let actual_ticks = r
        .frames
        .iter()
        .take(r.completed_ticks as usize)
        .filter(|f| role(f, 1).is_some())
        .count() as u64;
    if r.owner_alive != actual_alive
        || r.owner_ticks_alive != actual_ticks
        || (r.completed_ticks < 64 && !r.frames.last().unwrap().roles.is_empty())
    {
        return fail("survival/horizon endpoint mismatch");
    }
    if let Some(ledger) = &r.cohorts {
        if !r.ledger_errors.is_empty() || ledger.owner != 1 {
            return fail("ledger available with diagnostic failure");
        }
        ledger.reconcile()?;
        if serde_json::to_value(ledger).map_err(|e| e.to_string())?
            != serde_json::to_value(&reconstructed).map_err(|e| e.to_string())?
        {
            // allow harmless floating-point serialization differences below
            compare_ledger(ledger, &reconstructed)?;
        }
        if r.thief_transferred
            .is_none_or(|q| !near(q, ledger.cohorts.values().map(|c| c.transferred).sum()))
        {
            return fail("food endpoint disagrees with lineage");
        }
    } else {
        validate_unavailable_lineage(r)?;
    }
    Ok(())
}
/// Stumble marks a hit before trying loot, so discovery need not transfer food.
/// Raid marks only a positive take or a wasted attempt; the latter may stumble.
fn validate_action_markers(lab: &LabConfig, a: &ActionRecord, after: Pos) -> Result<(), String> {
    let arrival = a.id == 2
        && (a.phase == "ordinary"
            || (a.phase == "encounter" && a.action == "walk" && a.target == Some(after)));
    if a.discovery_site
        .is_some_and(|s| !arrival || lab.discovery <= 0.0 || s != site(after))
        || (a.discovery_amount > 0.0 && a.discovery_site.is_none())
        || (a.discovery_site.is_some() && a.raid_amount > 0.0)
    {
        return Err("incoherent discovery marker".into());
    }
    if a.raid_site.is_some_and(|s| !arrival || s != site(after))
        || a.raid_wasted != (a.raid_site.is_some() && a.raid_amount == 0.0)
        || (a.raid_amount > 0.0 && a.raid_site.is_none())
    {
        return Err("incoherent raid marker".into());
    }
    Ok(())
}
fn validate_unavailable_lineage(r: &EpisodeRecord) -> Result<(), String> {
    if r.thief_transferred.is_some() {
        return Err("food endpoint without lineage".into());
    }
    if !r
        .ledger_errors
        .iter()
        .any(|reason| !reason.trim().is_empty())
    {
        return Err("unavailable lineage without a meaningful lineage diagnostic reason".into());
    }
    Ok(())
}
/// Cancellation can rename a completed walk after metabolism or owner death.
/// Effort follows the recorded phase and physical result, not that final name.
fn validate_effort(before: &FrameRecord, after: &FrameRecord) -> Result<(), String> {
    let protective: Vec<_> = after
        .actions
        .iter()
        .filter(|a| a.phase == "relocation")
        .collect();
    let mut distance = 0_u64;
    for action in &protective {
        let start = role(before, action.id)
            .ok_or("protective effort has no starting role")?
            .pos;
        let end = role(after, action.id)
            .map(|r| r.pos)
            .or_else(|| {
                after
                    .deaths
                    .iter()
                    .find(|d| d.id == action.id)
                    .map(|d| d.pos)
            })
            .ok_or("protective effort has no action-end/death position")?;
        distance += u64::from(start.x.abs_diff(end.x) + start.y.abs_diff(end.y));
    }
    let mut cancellations = BTreeMap::new();
    for event in &after.relocation.source_events {
        if let Some(reason) = event.cancellation {
            *cancellations.entry(reason).or_insert(0_u64) += 1;
        }
    }
    let declared: BTreeMap<_, _> = after
        .relocation
        .cancellations
        .iter()
        .map(|(&reason, &n)| (reason, u64::from(n)))
        .collect();
    if u64::from(after.relocation.action_ticks) != protective.len() as u64
        || u64::from(after.relocation.distance) != distance
        || declared != cancellations
        || (protective.is_empty() && !cancellations.is_empty())
    {
        return Err("protective effort/cancellation diagnostics disagree with recorded actions and positions".into());
    }
    Ok(())
}
fn physical(l: &Ledger, holdings: f64, caches: &BTreeMap<u32, f64>) -> Result<(), String> {
    if !near(
        l.unlabelled_carried + l.cohorts.values().map(|c| c.carried).sum::<f64>(),
        holdings.max(0.0),
    ) {
        return Err("labelled carried balance disagrees with engine".into());
    }
    let mut totals = l.unlabelled_cached.clone();
    for c in l.cohorts.values() {
        for (&s, &q) in &c.cached {
            *totals.entry(s).or_default() += q;
        }
    }
    for s in totals.keys().chain(caches.keys()) {
        if !near(
            *totals.get(s).unwrap_or(&0.0),
            *caches.get(s).unwrap_or(&0.0),
        ) {
            return Err("labelled cache balance disagrees with engine".into());
        }
    }
    Ok(())
}
fn compare_ledger(a: &Ledger, b: &Ledger) -> Result<(), String> {
    if a.cohorts.keys().ne(b.cohorts.keys()) || !near(a.unlabelled_carried, b.unlabelled_carried) {
        return Err("lineage identity/balance mismatch".into());
    }
    let compare_map = |a: &BTreeMap<u32, f64>, b: &BTreeMap<u32, f64>| {
        a.keys()
            .chain(b.keys())
            .all(|s| near(*a.get(s).unwrap_or(&0.0), *b.get(s).unwrap_or(&0.0)))
    };
    if !compare_map(&a.unlabelled_cached, &b.unlabelled_cached) {
        return Err("unlabelled cache mismatch".into());
    }
    for (id, c) in &a.cohorts {
        let d = &b.cohorts[id];
        if ![
            (c.initial, d.initial),
            (c.carried, d.carried),
            (c.consumed, d.consumed),
            (c.transferred, d.transferred),
            (c.cost, d.cost),
            (c.lost_carried, d.lost_carried),
        ]
        .iter()
        .all(|&(x, y)| near(x, y))
            || !compare_map(&c.cached, &d.cached)
            || !compare_map(&c.lost_cached, &d.lost_cached)
        {
            return Err("cohort disagrees with recorded operations".into());
        }
    }
    Ok(())
}

#[cfg(test)]
pub(super) mod tests {
    use super::*;
    #[test]
    fn protection_archive_accepts_construction_records_without_scientific_seeds() {
        for c in manifest().conditions.iter().filter(|c| !c.lab.mirrored) {
            let r = sugarscape_core::minds::protection::runner::run_episode(c.lab.clone(), 7, true)
                .unwrap();
            validate_episode(&r).unwrap_or_else(|e| panic!("{}: {e}", c.id));
        }
    }
    #[test]
    fn protection_archive_rejects_forged_relocation_outcomes() {
        let mut r =
            sugarscape_core::minds::protection::runner::run_episode(LabConfig::default(), 7, true)
                .unwrap();
        r.frames[12].relocation.withdrawn = f64::NAN;
        assert!(validate_episode(&r).is_err());
    }
    #[test]
    fn protection_archive_rejects_bad_cues_routes_deaths_horizon_and_lineage() {
        let r =
            sugarscape_core::minds::protection::runner::run_episode(LabConfig::default(), 7, true)
                .unwrap();
        let mut bad = r.clone();
        bad.frames[1]
            .actions
            .iter_mut()
            .find(|a| a.id == 1)
            .unwrap()
            .perceived_exposure = Some(false);
        assert!(validate_episode(&bad).is_err());
        let mut bad = r.clone();
        bad.frames[2]
            .roles
            .iter_mut()
            .find(|a| a.id == 2)
            .unwrap()
            .pos = Pos::new(7, 7);
        assert!(validate_episode(&bad).is_err());
        let mut bad = r.clone();
        bad.completed_ticks = 63;
        assert!(validate_episode(&bad).is_err());
        let mut bad = r.clone();
        bad.owner_ticks_alive -= 1;
        assert!(validate_episode(&bad).is_err());
        let mut bad = r.clone();
        bad.frames
            .iter_mut()
            .find(|f| !f.deaths.is_empty())
            .unwrap()
            .deaths
            .clear();
        assert!(validate_episode(&bad).is_err());
        let mut bad = r.clone();
        bad.thief_transferred = Some(f64::NAN);
        assert!(validate_episode(&bad).is_err());
        let mut bad = r.clone();
        bad.cohorts
            .as_mut()
            .unwrap()
            .cohorts
            .values_mut()
            .next()
            .unwrap()
            .consumed += 1.0;
        assert!(validate_episode(&bad).is_err());
        let mut diagnostic = r;
        diagnostic.cohorts = None;
        diagnostic.thief_transferred = None;
        diagnostic.ledger_errors.push("diagnostic failure".into());
        assert!(validate_episode(&diagnostic).is_ok());
    }
    fn assert_rejected_before_analysis(record: EpisodeRecord, expected: &str) {
        let mut index = complete_index();
        let condition = index
            .manifest
            .conditions
            .iter()
            .find(|c| c.lab == record.lab)
            .unwrap();
        let ordinal = index
            .runs
            .iter()
            .position(|raw| raw.condition == condition.id && raw.seed == record.seed)
            .unwrap();
        index.runs.swap(0, ordinal);
        assert!(validate_episode(&record).unwrap_err().contains(expected));
        let records = vec![record; index.runs.len()];
        let error = super::super::protection_report::analyze(&index, &records).unwrap_err();
        assert!(error.contains(expected), "{error}");
    }
    #[test]
    fn protection_archive_rejects_impossible_action_markers_before_analysis() {
        let lab = manifest()
            .conditions
            .into_iter()
            .find(|c| {
                matches!(c.lab.fixture, Fixture::Stumble { .. })
                    && c.lab.discovery > 0.0
                    && !c.lab.mirrored
            })
            .unwrap()
            .lab;
        let r = synthetic(lab, 10001);
        validate_episode(&r).unwrap();
        // All mutations are zero-transfer markers: physical balances remain intact.
        for (frame, actor, remote, zero_find) in [
            (1, 2, false, false),
            (9, 2, false, false),
            (19, 1, false, false),
            (25, 2, true, false),
            (25, 2, false, true),
        ] {
            let mut bad = r.clone();
            if zero_find {
                bad.lab.discovery = 0.0;
            }
            let location = site(role(&bad.frames[frame], actor).unwrap().pos);
            bad.frames[frame]
                .actions
                .iter_mut()
                .find(|a| a.id == actor)
                .unwrap()
                .discovery_site = Some(if remote { location + 1 } else { location });
            assert_rejected_before_analysis(bad, "discovery marker");
        }
        let mut bad = r.clone();
        let action = bad.frames[25]
            .actions
            .iter_mut()
            .find(|a| a.id == 2)
            .unwrap();
        action.action = "hold".into();
        action.target = None;
        action.discovery_site = Some(30);
        assert_rejected_before_analysis(bad, "discovery marker");
        for (frame, actor, location, wasted) in [
            (1, 2, 30, true),
            (9, 2, 30, true),
            (19, 1, 30, true),
            (25, 2, 31, true),
            (25, 2, 30, false),
        ] {
            let mut bad = r.clone();
            let action = bad.frames[frame]
                .actions
                .iter_mut()
                .find(|a| a.id == actor)
                .unwrap();
            action.raid_site = Some(location);
            action.raid_wasted = wasted;
            assert_rejected_before_analysis(bad, "raid marker");
        }
        let mut bad = r;
        bad.frames[25]
            .actions
            .iter_mut()
            .find(|a| a.id == 2)
            .unwrap()
            .raid_wasted = true;
        assert_rejected_before_analysis(bad, "raid marker");
    }
    #[test]
    fn protection_archive_preserves_zero_transfer_operation_markers() {
        let lab = manifest()
            .conditions
            .into_iter()
            .find(|c| c.lab.discovery > 0.0)
            .unwrap()
            .lab;
        let after = Pos::new(3, 3);
        let mut action = ActionRecord {
            id: 2,
            phase: "ordinary".into(),
            action: "foraging".into(),
            discovery_site: Some(site(after)),
            ..Default::default()
        };
        // A successful find with no carrying room still records discovery.
        validate_action_markers(&lab, &action, after).unwrap();
        // A wasted raid falls through to stumbling and can record both markers.
        action.raid_site = Some(site(after));
        action.raid_wasted = true;
        validate_action_markers(&lab, &action, after).unwrap();
        action.discovery_site = None;
        validate_action_markers(&lab, &action, after).unwrap();
        // A successful raid returns before discovery, and is never wasted.
        action.raid_amount = 1.0;
        assert!(validate_action_markers(&lab, &action, after).is_err());
        action.raid_wasted = false;
        validate_action_markers(&lab, &action, after).unwrap();
        action.discovery_site = Some(site(after));
        assert!(validate_action_markers(&lab, &action, after).is_err());
    }
    #[test]
    fn protection_archive_requires_nonblank_unavailable_lineage_reason() {
        let r = synthetic(manifest().conditions[0].lab.clone(), 10001);
        for reasons in [vec![], vec![String::new()], vec![" \t\n".into()]] {
            let mut bad = r.clone();
            bad.cohorts = None;
            bad.thief_transferred = None;
            bad.ledger_errors = reasons;
            assert_rejected_before_analysis(bad, "lineage diagnostic reason");
        }
        let mut diagnostic = r;
        diagnostic.cohorts = None;
        diagnostic.thief_transferred = None;
        diagnostic.ledger_errors = vec![" \t".into(), "ledger reconciliation failed".into()];
        validate_episode(&diagnostic).unwrap();
    }
    pub(crate) fn complete_index() -> Index {
        let mut index = Index {
            schema: SCHEMA.into(),
            code_revision: "a".repeat(40),
            protocol_revision: "b".repeat(40),
            manifest: manifest(),
            completed: true,
            runs: vec![],
        };
        for (i, c) in index.manifest.conditions.iter().enumerate() {
            for &seed in &index.manifest.seeds {
                index.runs.push(RawRef {
                    condition: c.id.clone(),
                    seed,
                    path: raw_path(i, seed),
                });
            }
        }
        index
    }
    /// Hand-authored wire fixture: no harvest or protection, owner dies at tick 32,
    /// all twelve original units remain cached until terminal loss. This fixture
    /// exercises archive accounting, not the biological controller's choices.
    pub(crate) fn synthetic(lab: LabConfig, seed: u64) -> EpisodeRecord {
        use sugarscape_core::minds::protection::{
            runner::{ActionRecord, DeathRecord, RoleRecord, SeenRecord},
            state::{ProtectionState, Source},
        };
        let mixed = matches!(lab.fixture, Fixture::Mixed { .. });
        let start = pos(
            &lab,
            if matches!(
                lab.fixture,
                Fixture::Mixed {
                    observed_first: false
                }
            ) {
                5
            } else {
                3
            },
            3,
        );
        let route = observer_positions(&lab);
        let mut frames = Vec::new();
        let mut state = ProtectionState::default();
        let mut caches = BTreeMap::new();
        let mut seen = Vec::new();
        let mut restrictions = BTreeMap::new();
        for i in 0..=64_u64 {
            let mut actions = Vec::new();
            let mut deaths = Vec::new();
            let mut owner_pos = if i == 0 {
                start
            } else if i <= 8 {
                if mixed {
                    match i {
                        1 => start,
                        2 | 4..=8 => pos(&lab, 4, 3),
                        _ => pos(
                            &lab,
                            if matches!(
                                lab.fixture,
                                Fixture::Mixed {
                                    observed_first: true
                                }
                            ) {
                                5
                            } else {
                                3
                            },
                            3,
                        ),
                    }
                } else {
                    start
                }
            } else {
                pos(&lab, 4, 3)
            };
            if i > 0 {
                let tick = i - 1;
                if tick < 32 {
                    let deposit = if tick == 0 || (mixed && tick == 2) {
                        if mixed {
                            6.0
                        } else {
                            12.0
                        }
                    } else {
                        0.0
                    };
                    let source = site(owner_pos);
                    let exposed = if mixed {
                        owner_pos == pos(&lab, 3, 3)
                    } else {
                        !matches!(
                            lab.fixture,
                            Fixture::Single {
                                initial_observed: false,
                                ..
                            } | Fixture::Stumble {
                                initial_observed: false
                            } | Fixture::CueUnseenWatcher
                        )
                    };
                    let watched = if mixed {
                        owner_pos == pos(&lab, 3, 3)
                    } else {
                        !matches!(
                            lab.fixture,
                            Fixture::Single {
                                initial_observed: false,
                                ..
                            } | Fixture::Stumble {
                                initial_observed: false
                            } | Fixture::CueVisibleNonwatcher
                        )
                    };
                    if deposit > 0.0 {
                        caches.insert(source, deposit);
                        state.sources.insert(
                            source,
                            Source {
                                tick,
                                initial_amount: deposit,
                                attempted: false,
                            },
                        );
                        state.exposure.remember(source, tick, exposed);
                        if watched {
                            seen.push(SeenRecord {
                                site: source,
                                owner: 1,
                                amount: deposit,
                                tick,
                            });
                        }
                    }
                    if tick == 8 && lab.policy == Policy::Erased {
                        state.exposure.entries.clear();
                    }
                    let walking = mixed && (1..=3).contains(&tick);
                    actions.push(ActionRecord {
                        id: 1,
                        phase: if tick < 8 { "preparation" } else { "ordinary" }.into(),
                        action: if deposit > 0.0 {
                            "prepare_deposit"
                        } else if walking {
                            "walk"
                        } else if tick < 8 {
                            "hold"
                        } else {
                            "foraging"
                        }
                        .into(),
                        gross_buried: deposit,
                        metabolic_demand: 1.0,
                        metabolic_consumed: 1.0,
                        source: (deposit > 0.0).then_some(source),
                        target: walking.then_some(owner_pos),
                        perceived_exposure: (deposit > 0.0).then_some(exposed),
                        actual_watchers: if deposit > 0.0 && watched {
                            vec![2]
                        } else {
                            vec![]
                        },
                        ..Default::default()
                    });
                    if tick < 8 {
                        *restrictions.entry(1).or_default() += 1;
                    }
                    if i == 32 {
                        deaths.push(DeathRecord {
                            id: 1,
                            pos: owner_pos,
                            cause: "starvation".into(),
                        });
                        caches.clear();
                    }
                }
                let scripted = (i as usize) < route.len();
                let observer_pos = route
                    .get(i as usize)
                    .copied()
                    .unwrap_or(*route.last().unwrap());
                let walking = scripted && observer_pos != route[i as usize - 1];
                let encounter =
                    scripted && matches!(lab.fixture, Fixture::Stumble { .. }) && tick >= 18;
                let phase = if !scripted {
                    "ordinary"
                } else if encounter {
                    "encounter"
                } else if tick < 8 {
                    "preparation"
                } else {
                    "observer_hold"
                };
                if phase == "preparation" || phase == "observer_hold" {
                    *restrictions.entry(2).or_default() += 1;
                }
                actions.push(ActionRecord {
                    id: 2,
                    phase: phase.into(),
                    action: if !scripted {
                        "foraging"
                    } else if walking {
                        "walk"
                    } else {
                        "hold"
                    }
                    .into(),
                    target: walking.then_some(observer_pos),
                    metabolic_demand: 1.0,
                    metabolic_consumed: 1.0,
                    ..Default::default()
                });
            }
            if i >= 32 {
                owner_pos = pos(&lab, 4, 3);
            }
            let mut roles = Vec::new();
            if i < 32 {
                roles.push(RoleRecord {
                    id: 1,
                    pos: owner_pos,
                    holdings: 44.0 - i as f64 - caches.values().sum::<f64>(),
                    caches: caches.clone(),
                    protection: Some(state.clone()),
                    seen: vec![],
                });
            }
            roles.push(RoleRecord {
                id: 2,
                pos: route
                    .get(i as usize)
                    .copied()
                    .unwrap_or(*route.last().unwrap()),
                holdings: 96.0 - i as f64,
                caches: BTreeMap::new(),
                protection: None,
                seen: seen.clone(),
            });
            frames.push(FrameRecord {
                tick: i,
                fingerprint: format!("{i:016x}"),
                roles,
                relocation: Default::default(),
                deaths,
                actions,
                current_bury_cost: if i <= 8 { 0.0 } else { lab.reburial_cost },
                restriction_ticks: restrictions.clone(),
            });
        }
        let cohorts = state
            .sources
            .iter()
            .map(|(&source, s)| {
                (
                    source,
                    sugarscape_core::minds::protection::ledger::CohortBalance {
                        initial: s.initial_amount,
                        lost_cached: BTreeMap::from([(source, s.initial_amount)]),
                        ..Default::default()
                    },
                )
            })
            .collect();
        EpisodeRecord {
            schema: SCHEMA.into(),
            lab,
            seed,
            requested_ticks: 64,
            completed_ticks: 64,
            owner_ticks_alive: 32,
            owner_alive: false,
            ledger_errors: vec![],
            thief_transferred: Some(0.0),
            cohorts: Some(Ledger {
                owner: 1,
                cohorts,
                unlabelled_carried: 0.0,
                unlabelled_cached: BTreeMap::new(),
            }),
            frames,
            fixture_errors: vec![],
        }
    }
    fn mixed_redeposit_wire() -> EpisodeRecord {
        use sugarscape_core::minds::protection::state::{Intent, SourceEvent, Stage};
        let mut r = synthetic(
            LabConfig {
                policy: Policy::Selective,
                fixture: Fixture::Mixed {
                    observed_first: true,
                },
                reburial_cost: 0.0,
                ..Default::default()
            },
            10001,
        );
        let source = site(Pos::new(3, 3));
        let destination = Pos::new(3, 2);
        for f in r.frames.iter_mut().skip(9) {
            if let Some(owner) = f.roles.iter_mut().find(|a| a.id == 1) {
                owner.pos = if f.tick <= 10 {
                    Pos::new(3, 3)
                } else {
                    destination
                };
                let state = owner.protection.as_mut().unwrap();
                state.sources.get_mut(&source).unwrap().attempted = true;
                state.intent = match f.tick {
                    9 => Some(Intent {
                        source,
                        destination,
                        amount: 0.0,
                        stage: Stage::Retrieve,
                    }),
                    10 => Some(Intent {
                        source,
                        destination,
                        amount: 6.0,
                        stage: Stage::ToDestination,
                    }),
                    11 => Some(Intent {
                        source,
                        destination,
                        amount: 6.0,
                        stage: Stage::Deposit,
                    }),
                    _ => None,
                };
                if f.tick == 10 || f.tick == 11 {
                    owner.holdings += 6.0;
                    owner.caches.remove(&source);
                }
                if f.tick >= 12 {
                    owner.caches.remove(&source);
                    owner.caches.insert(site(destination), 6.0);
                }
            }
            if let Some(death) = f.deaths.iter_mut().find(|d| d.id == 1) {
                death.pos = destination;
            }
            if (9..=12).contains(&f.tick) {
                let a = f.actions.iter_mut().find(|a| a.id == 1).unwrap();
                a.phase = "relocation".into();
                a.source = Some(source);
                a.target = Some(if f.tick <= 10 {
                    Pos::new(3, 3)
                } else {
                    destination
                });
                a.action = match f.tick {
                    9 | 11 => "walk",
                    10 => "retrieve",
                    _ => "redeposit",
                }
                .into();
                f.relocation.action_ticks = 1;
                if f.tick == 9 {
                    f.relocation.starts = 1;
                    f.relocation.source_events.push(SourceEvent {
                        source,
                        started: true,
                        ..Default::default()
                    });
                }
                if f.tick == 9 || f.tick == 11 {
                    f.relocation.distance = 1;
                }
                if f.tick == 10 {
                    a.gross_dug = 6.0;
                    f.relocation.withdrawn = 6.0;
                    f.relocation.source_events.push(SourceEvent {
                        source,
                        withdrawn: 6.0,
                        ..Default::default()
                    });
                }
                if f.tick == 12 {
                    a.gross_buried = 6.0;
                    a.perceived_exposure = Some(false);
                    f.relocation.redeposited = 6.0;
                    f.relocation.completions = 1;
                    f.relocation.source_events.push(SourceEvent {
                        source,
                        redeposited: 6.0,
                        ..Default::default()
                    });
                }
            }
        }
        // The six tagged units mix with 23 unlabelled units on retrieval.
        // Reburial moves 6*(6/29) tagged units; the rest are consumed by death.
        let cohort = r
            .cohorts
            .as_mut()
            .unwrap()
            .cohorts
            .get_mut(&source)
            .unwrap();
        cohort.consumed = 6.0 - 36.0 / 29.0;
        cohort.lost_cached = BTreeMap::from([(site(destination), 36.0 / 29.0)]);
        r
    }
    #[test]
    fn protection_archive_requires_actual_mixed_redeposit_target() {
        let r = mixed_redeposit_wire();
        validate_episode(&r).unwrap();
        for target in [None, Some(Pos::new(6, 2))] {
            let mut bad = r.clone();
            bad.frames[12]
                .actions
                .iter_mut()
                .find(|a| a.id == 1)
                .unwrap()
                .target = target;
            assert!(
                validate_episode(&bad).is_err(),
                "malformed target {target:?} accepted"
            );
        }
    }
    #[test]
    fn protection_archive_malformed_mixed_redeposit_analysis_returns_error() {
        let mut index = complete_index();
        let r = mixed_redeposit_wire();
        let condition = index
            .manifest
            .conditions
            .iter()
            .find(|c| c.lab == r.lab)
            .unwrap()
            .id
            .clone();
        let i = index
            .runs
            .iter()
            .position(|raw| raw.condition == condition && raw.seed == r.seed)
            .unwrap();
        index.runs.swap(0, i);
        for target in [None, Some(Pos::new(6, 2))] {
            let mut bad = r.clone();
            bad.frames[12]
                .actions
                .iter_mut()
                .find(|a| a.id == 1)
                .unwrap()
                .target = target;
            // Later records are deliberately unreadable: malformed target must be
            // rejected at the first registered record, before report rendering.
            let mut records = vec![bad.clone(); index.runs.len()];
            records[0] = bad;
            let error = super::super::protection_report::analyze(&index, &records).unwrap_err();
            assert!(error.contains("redeposit target"), "{error}");
        }
    }
    #[test]
    fn protection_archive_cancellation_effort_keeps_post_metabolism_and_death_movement() {
        use sugarscape_core::minds::protection::{
            runner::DeathRecord,
            state::{CancelReason, SourceEvent},
        };
        let r = mixed_redeposit_wire();
        for reason in [CancelReason::SurplusExhausted, CancelReason::OwnerDied] {
            // Mirror the engine's actual record order: protective walk, metabolism,
            // then cancel overwrites only the action name (and death removes role).
            let mut before = r.frames[10].clone();
            let mut after = r.frames[11].clone();
            let source = site(Pos::new(3, 3));
            let destination = Pos::new(3, 2);
            before
                .roles
                .iter_mut()
                .find(|a| a.id == 1)
                .unwrap()
                .holdings = if reason == CancelReason::OwnerDied {
                1.0
            } else {
                5.0
            };
            after.actions.iter_mut().find(|a| a.id == 1).unwrap().action = "cancel".into();
            after.relocation.cancellations.insert(reason, 1);
            after.relocation.source_events.push(SourceEvent {
                source,
                cancellation: Some(reason),
                ..Default::default()
            });
            if reason == CancelReason::OwnerDied {
                after.roles.retain(|a| a.id != 1);
                after.deaths.push(DeathRecord {
                    id: 1,
                    pos: destination,
                    cause: "starvation".into(),
                });
            } else {
                after.roles.iter_mut().find(|a| a.id == 1).unwrap().holdings = 4.0;
            }
            assert!(validate_effort(&before, &after).is_ok());
            let mut bad = after.clone();
            bad.relocation.distance = 0;
            assert!(validate_effort(&before, &bad).is_err());
            let mut bad = after.clone();
            bad.relocation.action_ticks = 0;
            assert!(validate_effort(&before, &bad).is_err());
            let mut bad = after.clone();
            bad.relocation.cancellations.insert(reason, 2);
            assert!(validate_effort(&before, &bad).is_err());
            let mut bad = after;
            bad.relocation.source_events.clear();
            assert!(validate_effort(&before, &bad).is_err());
        }
    }
    #[test]
    fn protection_archive_rejects_off_action_ticks_mutation() {
        let mut r = synthetic(LabConfig::default(), 10001);
        r.frames[9].relocation.action_ticks = 1;
        assert!(validate_episode(&r).is_err());
    }
    #[test]
    fn protection_archive_rejects_off_distance_mutation() {
        let mut r = synthetic(LabConfig::default(), 10001);
        r.frames[9].relocation.distance = 1;
        assert!(validate_episode(&r).is_err());
    }
    #[test]
    fn protection_archive_rejects_off_cancellation_map_mutation() {
        let mut r = synthetic(LabConfig::default(), 10001);
        r.frames[9].relocation.cancellations.insert(
            sugarscape_core::minds::protection::state::CancelReason::Expired,
            1,
        );
        assert!(validate_episode(&r).is_err());
    }
    #[test]
    fn protection_synthetic_wire_fixtures_validate_without_running_episodes() {
        for c in manifest().conditions {
            let r = synthetic(c.lab, 10001);
            validate_episode(&r).unwrap_or_else(|e| panic!("{}: {e}", c.id));
        }
    }
    #[test]
    fn protection_archive_requires_exact_index_and_confined_existing_raw() {
        let index = complete_index();
        assert!(validate_index(&index).is_ok());
        let mut bad = index.clone();
        bad.runs.pop();
        assert!(validate_index(&bad).is_err());
        let mut bad = index.clone();
        bad.runs[1] = bad.runs[0].clone();
        assert!(validate_index(&bad).is_err());
        let mut bad = index.clone();
        bad.runs[0].seed = 7;
        assert!(validate_index(&bad).is_err());
        let mut bad = index.clone();
        bad.runs[0].condition = "wrong".into();
        assert!(validate_index(&bad).is_err());
        let mut bad = index.clone();
        bad.runs[0].path = "../outside.json".into();
        assert!(validate_index(&bad).is_err());
        let mut bad = index.clone();
        bad.completed = false;
        assert!(validate_index(&bad).is_err());
        let mut bad = index.clone();
        bad.manifest.conditions[0].lab.discovery = 0.5;
        assert!(validate_index(&bad).is_err());
        assert!(load_records(Path::new(env!("CARGO_MANIFEST_DIR")), &index)
            .unwrap_err()
            .contains("missing raw"));
    }
    #[test]
    fn protection_execution_preflight_requires_committed_protocol_clean_tree_and_new_output() {
        let root =
            std::env::temp_dir().join(format!("protection-provenance-{}", std::process::id()));
        if root.exists() {
            fs::remove_dir_all(&root).unwrap();
        }
        fs::create_dir(&root).unwrap();
        git(&root, &["init", "--quiet"]).unwrap();
        let protocol = "docs/superpowers/specs/2026-10-02-minds-protection-protocol.md";
        fs::create_dir_all(root.join("docs/superpowers/specs")).unwrap();
        fs::write(root.join(protocol), "registered protocol\n").unwrap();
        fs::write(root.join("tracked"), "clean").unwrap();
        git(&root, &["add", "."]).unwrap();
        git(
            &root,
            &[
                "-c",
                "user.name=Construction Test",
                "-c",
                "user.email=construction@example.invalid",
                "commit",
                "--quiet",
                "-m",
                "synthetic committed protocol",
            ],
        )
        .unwrap();
        let revision = git(&root, &["rev-parse", "HEAD"]).unwrap();
        let out = root.join("new");
        assert_eq!(preflight(&root, &revision, &out).unwrap(), revision);
        assert!(!out.exists());
        assert!(preflight(&root, &revision[..7], &out).is_err());
        assert!(preflight(&root, &"a".repeat(40), &out).is_err());
        fs::create_dir(&out).unwrap();
        assert!(preflight(&root, &revision, &out)
            .unwrap_err()
            .contains("already exists"));
        fs::remove_dir(&out).unwrap();
        fs::write(root.join("tracked"), "dirty").unwrap();
        assert!(preflight(&root, &revision, &out)
            .unwrap_err()
            .contains("clean tracked tree"));
        fs::write(root.join("tracked"), "clean").unwrap();
        fs::write(root.join(protocol), "different protocol").unwrap();
        assert!(preflight(&root, &revision, &out)
            .unwrap_err()
            .contains("differs"));
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn protection_output_never_overwrites_existing_files_or_directory() {
        let path = std::env::temp_dir().join(format!("protection-archive-{}", std::process::id()));
        if path.exists() {
            fs::remove_dir_all(&path).unwrap();
        }
        new_directory(&path).unwrap();
        assert!(new_directory(&path).is_err());
        write_new(&path.join("run.json"), b"first").unwrap();
        assert!(write_new(&path.join("run.json"), b"second").is_err());
        assert_eq!(fs::read(path.join("run.json")).unwrap(), b"first");
        fs::remove_dir_all(path).unwrap();
    }
    #[test]
    fn protection_archive_rejects_arbitrary_frame_only_records() {
        let mut index = Index {
            schema: SCHEMA.into(),
            code_revision: "a".repeat(40),
            protocol_revision: "b".repeat(40),
            manifest: manifest(),
            completed: true,
            runs: vec![],
        };
        for (i, c) in index.manifest.conditions.iter().enumerate() {
            for &seed in &index.manifest.seeds {
                index.runs.push(RawRef {
                    condition: c.id.clone(),
                    seed,
                    path: raw_path(i, seed),
                });
            }
        }
        let records = index
            .runs
            .iter()
            .map(|raw| EpisodeRecord {
                schema: SCHEMA.into(),
                lab: index
                    .manifest
                    .conditions
                    .iter()
                    .find(|c| c.id == raw.condition)
                    .unwrap()
                    .lab
                    .clone(),
                seed: raw.seed,
                requested_ticks: 64,
                completed_ticks: 64,
                owner_ticks_alive: 64,
                owner_alive: true,
                ledger_errors: vec![],
                thief_transferred: None,
                cohorts: None,
                frames: vec![],
                fixture_errors: vec![],
            })
            .collect::<Vec<_>>();
        assert!(validate_archive(&index, &records).is_err());
    }
}
