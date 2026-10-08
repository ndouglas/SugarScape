//! Fully resolved immutable candidate inputs and prospective execution gate.
use super::{scenario, CollectionMode, Geometry, Panel, Regime, RunKey};
use serde::Serialize;
use sugarscape_core::foraging::construction as core;

#[allow(
    dead_code,
    reason = "Condition is consumed by the Task 2 validator and Task 4 collector"
)]
#[derive(Clone, Debug, PartialEq, Serialize)]
pub(super) struct Condition {
    pub id: String,
    pub panel: Panel,
    pub geometry: Geometry,
    pub regime: Regime,
    pub setup: core::Setup,
    pub options: core::RunOptions,
}

#[allow(
    dead_code,
    reason = "Manifest is consumed by the Task 3 archive and Task 4 collector"
)]
#[derive(Clone, Debug, PartialEq, Serialize)]
pub(super) struct Manifest {
    pub schema: String,
    pub version: u32,
    pub status: String,
    pub execution_authorized: bool,
    pub protocol: String,
    pub conditions: Vec<Condition>,
    pub construction_seeds: Vec<u64>,
    pub scientific_seeds: Vec<u64>,
    pub raw_record_limit: u64,
    pub raw_total_limit: u64,
    pub metadata_limit: u64,
}

#[allow(
    dead_code,
    reason = "candidate is consumed by the Task 2 validator and Task 4 collector"
)]
pub(super) fn candidate() -> Result<Manifest, String> {
    let mut conditions = Vec::new();
    for (panel, panel_name) in [(Panel::Route, "route"), (Panel::Access, "access")] {
        for (geometry, geometry_name) in [
            (Geometry::Straight, "straight"),
            (Geometry::Detour, "detour"),
            (Geometry::Twisting, "twisting"),
        ] {
            for (regime, regime_name) in [
                (Regime::Paid, "paid"),
                (Regime::Protected, "protected"),
                (Regime::AlreadyOpen, "already_open"),
            ] {
                if panel == Panel::Access && regime == Regime::AlreadyOpen {
                    continue;
                }
                conditions.push(Condition {
                    id: format!("{panel_name}.{geometry_name}.{regime_name}"),
                    panel,
                    geometry,
                    regime,
                    setup: scenario::build(panel, geometry, regime)?,
                    options: core::RunOptions {
                        ticks: 512,
                        sample_every: 128,
                        snapshots: true,
                    },
                });
            }
        }
    }
    Ok(Manifest {
        schema: "foraging-shortcut-manifest-v1".into(),
        version: 1,
        status: "draft".into(),
        execution_authorized: false,
        protocol: "docs/superpowers/specs/2026-10-07-foraging-5-shortcut-comparison-design.md"
            .into(),
        conditions,
        construction_seeds: vec![7, 8],
        scientific_seeds: (10001..=10040).collect(),
        raw_record_limit: 4 * 1024 * 1024,
        raw_total_limit: 1024 * 1024 * 1024,
        metadata_limit: 4 * 1024 * 1024,
    })
}

#[allow(
    dead_code,
    reason = "manifest_bytes is consumed by the Task 4 manifest CLI"
)]
pub(super) fn manifest_bytes() -> Result<Vec<u8>, String> {
    let mut bytes = serde_json::to_vec_pretty(&candidate()?)
        .map_err(|e| format!("serialize shortcut manifest: {e}"))?;
    bytes.push(b'\n');
    Ok(bytes)
}

#[allow(
    dead_code,
    reason = "condition is consumed by the Task 2 validator and Task 3 archive"
)]
pub(super) fn condition<'a>(manifest: &'a Manifest, id: &str) -> Result<&'a Condition, String> {
    manifest
        .conditions
        .iter()
        .find(|c| c.id == id)
        .ok_or_else(|| format!("unknown shortcut condition: {id}"))
}

#[allow(
    dead_code,
    reason = "expected_keys is consumed by the Task 3 archive and Task 4 collector"
)]
pub(super) fn expected_keys(manifest: &Manifest, mode: CollectionMode) -> Vec<RunKey> {
    let seeds = match mode {
        CollectionMode::Construction => &manifest.construction_seeds,
        CollectionMode::Scientific => &manifest.scientific_seeds,
    };
    manifest
        .conditions
        .iter()
        .flat_map(|c| {
            seeds.iter().map(move |&seed| RunKey {
                condition: c.id.clone(),
                seed,
            })
        })
        .collect()
}

#[allow(dead_code, reason = "authorize is consumed by the Task 4 collector")]
pub(super) fn authorize(manifest: &Manifest, mode: CollectionMode) -> Result<(), String> {
    if mode == CollectionMode::Scientific
        && (!manifest.execution_authorized || manifest.status != "registered")
    {
        return Err("scientific shortcut execution requires an authorized registered manifest; candidate is draft".into());
    }
    Ok(())
}
