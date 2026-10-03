//! Native GeoSim recorder: Rust owns configs; exact bytes bind scientific evidence.
use serde_json::Value;
use sugarscape_core::geosim::{GeosimConfig, GeosimWorld};

use serde::de::{self, MapAccess, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer};
use sha2::{Digest, Sha256};
use std::fmt;

struct Strict(Value);
impl<'de> Deserialize<'de> for Strict {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct JsonVisitor;
        impl<'de> Visitor<'de> for JsonVisitor {
            type Value = Strict;
            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("finite JSON with unique fields")
            }
            fn visit_bool<E: de::Error>(self, v: bool) -> Result<Strict, E> {
                Ok(Strict(Value::Bool(v)))
            }
            fn visit_i64<E: de::Error>(self, v: i64) -> Result<Strict, E> {
                Ok(Strict(v.into()))
            }
            fn visit_u64<E: de::Error>(self, v: u64) -> Result<Strict, E> {
                Ok(Strict(v.into()))
            }
            fn visit_f64<E: de::Error>(self, v: f64) -> Result<Strict, E> {
                serde_json::Number::from_f64(v)
                    .map(|n| Strict(Value::Number(n)))
                    .ok_or_else(|| E::custom("nonfinite JSON number"))
            }
            fn visit_str<E: de::Error>(self, v: &str) -> Result<Strict, E> {
                Ok(Strict(v.into()))
            }
            fn visit_string<E: de::Error>(self, v: String) -> Result<Strict, E> {
                Ok(Strict(v.into()))
            }
            fn visit_none<E: de::Error>(self) -> Result<Strict, E> {
                Ok(Strict(Value::Null))
            }
            fn visit_unit<E: de::Error>(self) -> Result<Strict, E> {
                Ok(Strict(Value::Null))
            }
            fn visit_seq<A: SeqAccess<'de>>(self, mut a: A) -> Result<Strict, A::Error> {
                let mut values = Vec::new();
                while let Some(v) = a.next_element::<Strict>()? {
                    values.push(v.0);
                }
                Ok(Strict(Value::Array(values)))
            }
            fn visit_map<A: MapAccess<'de>>(self, mut a: A) -> Result<Strict, A::Error> {
                let mut values = serde_json::Map::new();
                while let Some(k) = a.next_key::<String>()? {
                    if values.contains_key(&k) {
                        return Err(de::Error::custom(format!("duplicate JSON field: {k}")));
                    }
                    let v = a.next_value::<Strict>()?;
                    values.insert(k, v.0);
                }
                Ok(Strict(Value::Object(values)))
            }
        }
        d.deserialize_any(JsonVisitor)
    }
}
fn strict_json(bytes: &[u8]) -> Result<Value, String> {
    let mut d = serde_json::Deserializer::from_slice(bytes);
    let v = Strict::deserialize(&mut d).map_err(|e| e.to_string())?;
    d.end().map_err(|e| e.to_string())?;
    Ok(v.0)
}
fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn resolve_config(preset: &str, overrides: &Value) -> Result<GeosimConfig, String> {
    let id = match preset {
        "paper" => "geosim-paper",
        "artifact_2017" => "geosim-artifact-2017",
        _ => return Err(format!("unknown GeoSim preset {preset}")),
    };
    let p = sugarscape_core::geosim::presets()
        .into_iter()
        .find(|p| p.id == id)
        .ok_or("actual Rust preset unavailable")?;
    let sugarscape_core::model::ModelConfig::Geosim(c) = p.config else {
        return Err("preset is not GeoSim".into());
    };
    let mut value = serde_json::to_value(c).map_err(|e| e.to_string())?;
    for (key, v) in overrides
        .as_object()
        .ok_or("config_overrides must be an object")?
    {
        value[key] = v.clone();
    }
    let c: GeosimConfig =
        serde_json::from_value(value).map_err(|e| format!("invalid GeoSim config: {e}"))?;
    c.validate().map_err(|e| format!("{e:?}"))?;
    Ok(c)
}

use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
const TABLE: &str = "docs/superpowers/specs/2026-10-03-geosim-source-table.json";
const SPEC: &str = "docs/superpowers/specs/2026-10-03-geosim-design.md";
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct SourceEntry {
    path: String,
    sha256: String,
    bytes: u64,
}
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Arm {
    id: String,
    index: usize,
    family: String,
    preset: String,
    config_overrides: Value,
    sessions: u64,
    first_seed: u64,
}
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Job {
    index: usize,
    id: String,
    root_entropy: u64,
    spawn_key: Vec<u32>,
    state_u32: Vec<u32>,
}
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Manifest {
    schema_version: u32,
    model: String,
    execution_mode: String,
    provenance_status: String,
    spec: String,
    source_draws: u64,
    ks_draws: u64,
    analysis_seed: u64,
    source_table_sha256: Option<String>,
    method_contract: Value,
    method_contract_json: String,
    method_contract_sha256: String,
    source_inventory: Option<Vec<SourceEntry>>,
    source_inventory_sha256: Option<String>,
    arms: Vec<Arm>,
    analysis_jobs: Vec<Job>,
}
#[derive(Clone, Debug, Serialize)]
struct ResolvedArm {
    id: String,
    index: usize,
    family: String,
    preset: String,
    sessions: u64,
    first_seed: u64,
    config: GeosimConfig,
}
struct Prepared {
    manifest: Manifest,
    arms: Vec<ResolvedArm>,
    payload: String,
    payload_sha: String,
}
const REGISTERED_TEMPLATE: &str =
    include_str!("../../tests/fixtures/geosim-registered-manifest.json");
fn require(condition: bool, reason: &str) -> Result<(), String> {
    if condition {
        Ok(())
    } else {
        Err(reason.into())
    }
}
fn keys(value: &Value, expected: &[&str]) -> Result<(), String> {
    let got = value
        .as_object()
        .ok_or("expected JSON object")?
        .keys()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    require(
        got == expected.iter().copied().collect(),
        "unknown or missing JSON fields",
    )
}
fn read(path: &Path) -> Result<Vec<u8>, String> {
    std::fs::read(path).map_err(|e| format!("{}: {e}", path.display()))
}
const REQUIRED_FILES: &[&str] = &[
    "Cargo.toml",
    "Cargo.lock",
    "crates/sugarscape-core/Cargo.toml",
    "crates/sugarscape-core/assets/sugar-map.txt",
    "crates/sugarscape-core/tests/geosim_discovery.rs",
    "crates/sugarscape-core/tests/geosim_host_sweep.rs",
    "crates/sugarscape-core/tests/geosim_invalid_sweep.rs",
    "survey/Cargo.toml",
    "survey/Cargo.lock",
    "survey/src/bin/geosim.rs",
    "survey/tests/geosim_native.rs",
    "survey/tests/fixtures/geosim-registered-manifest.json",
    "survey/geosim/.python-version",
    "survey/geosim/requirements.txt",
    "survey/geosim/README.md",
    "survey/geosim/NUMERICAL_METHODS.md",
    "survey/geosim/reference/extract.py",
    "survey/geosim/reference/test_extract.py",
    "survey/geosim/reference/test_launch.py",
    "survey/geosim/reference/GeoSimReferenceProbe.java",
    "docs/superpowers/specs/2026-10-03-geosim-source-table.json",
    "docs/superpowers/specs/2026-10-03-geosim-design.md",
    "docs/superpowers/specs/2026-10-03-geosim-source-extraction.md",
    "docs/superpowers/specs/2026-10-03-geosim-reading-notes.md",
    "docs/superpowers/specs/2026-10-03-geosim-author-code-reading-notes.md",
    "docs/superpowers/specs/2026-10-03-geosim-mechanics-reading-notes.md",
    "docs/superpowers/specs/2026-10-03-geosim-defender-threshold-amendment.md",
    "docs/superpowers/specs/2026-10-03-geosim-damage-incidence-audit.md",
    "docs/superpowers/specs/2026-10-03-geosim-artifact-audit.md",
    "docs/superpowers/specs/2026-10-03-geosim-finite-technology-amendment.md",
    "docs/superpowers/specs/2026-10-03-geosim-portability-amendment.md",
];
fn safe_path(root: &Path, name: &str) -> Result<PathBuf, String> {
    require(
        !name.is_empty()
            && Path::new(name)
                .components()
                .all(|c| matches!(c, std::path::Component::Normal(_)))
            && Path::new(name)
                .components()
                .map(|c| c.as_os_str().to_str().unwrap())
                .collect::<Vec<_>>()
                .join("/")
                == name,
        "unsafe source path",
    )?;
    let root = root.canonicalize().map_err(|e| e.to_string())?;
    let mut current = root;
    for component in Path::new(name).components() {
        current.push(component);
        require(
            !std::fs::symlink_metadata(&current)
                .map_err(|e| format!("{name}: {e}"))?
                .file_type()
                .is_symlink(),
            "source symlink is forbidden",
        )?;
    }
    require(current.is_file(), "inventory source is not a file")?;
    Ok(current)
}
fn collect_sources(
    root: &Path,
    directory: &str,
    suffixes: &[&str],
    out: &mut BTreeSet<String>,
) -> Result<(), String> {
    for entry in std::fs::read_dir(root.join(directory)).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        let relative = path
            .strip_prefix(root)
            .map_err(|e| e.to_string())?
            .to_str()
            .ok_or("non-UTF8 source path")?
            .to_owned();
        if relative.starts_with("survey/geosim/reference/raw/")
            || path.components().any(|c| c.as_os_str() == "__pycache__")
        {
            continue;
        }
        let kind = entry.file_type().map_err(|e| e.to_string())?;
        require(!kind.is_symlink(), "source symlink is forbidden")?;
        if kind.is_dir() {
            if relative != "survey/geosim/reference/raw" {
                collect_sources(root, &relative, suffixes, out)?;
            }
        } else if suffixes.iter().any(|suffix| relative.ends_with(suffix)) {
            out.insert(relative);
        }
    }
    Ok(())
}
fn required_inventory_paths(root: &Path) -> Result<BTreeSet<String>, String> {
    let root = root.canonicalize().map_err(|e| e.to_string())?;
    let mut required = REQUIRED_FILES
        .iter()
        .map(|s| s.to_string())
        .collect::<BTreeSet<_>>();
    for (directory, suffixes) in [
        ("crates/sugarscape-core/src", vec![".rs"]),
        ("survey/geosim", vec![".py", ".java"]),
        ("crates/sugarscape-core/tests", vec![".rs"]),
        ("survey/tests", vec![".rs"]),
    ] {
        collect_sources(&root, directory, &suffixes, &mut required)?;
    }
    for name in [
        ".cargo/config",
        ".cargo/config.toml",
        "rust-toolchain",
        "rust-toolchain.toml",
        "build.rs",
        "crates/sugarscape-core/build.rs",
        "survey/build.rs",
    ] {
        if std::fs::symlink_metadata(root.join(name)).is_ok() {
            required.insert(name.into());
        }
    }
    for name in &required {
        safe_path(&root, name)?;
    }
    Ok(required)
}
fn inventory_digest(root: &Path, entries: &[SourceEntry]) -> Result<String, String> {
    let required = required_inventory_paths(root)?;
    let mut seen = BTreeSet::new();
    let mut previous = None;
    for e in entries {
        require(
            previous.is_none_or(|p: &str| p < e.path.as_str()),
            "source inventory must be unique and path sorted",
        )?;
        previous = Some(&e.path);
        seen.insert(e.path.clone());
        let b = read(&safe_path(root, &e.path)?)?;
        require(
            b.len() as u64 == e.bytes && digest(&b) == e.sha256,
            "source inventory byte/hash mismatch",
        )?;
    }
    require(
        required.is_subset(&seen),
        "source inventory omits required relevant engine/protocol/contract sources",
    )?;
    // Value's sorted map keys exactly match Python's canonical JSON for this integer/string inventory.
    let canonical = serde_json::to_vec(&serde_json::to_value(entries).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())?;
    Ok(digest(&canonical))
}
fn prepare(bytes: &[u8], root: &Path, expected_sha: &str) -> Result<Prepared, String> {
    require(
        digest(bytes) == expected_sha,
        "actual manifest-byte SHA256 mismatch",
    )?;
    let v = strict_json(bytes)?;
    let template = strict_json(REGISTERED_TEMPLATE.as_bytes())?;
    require(
        v.as_object().map(|o| o.keys().collect::<BTreeSet<_>>())
            == template
                .as_object()
                .map(|o| o.keys().collect::<BTreeSet<_>>()),
        "unknown or missing manifest fields",
    )?;
    let manifest: Manifest = serde_json::from_value(v.clone()).map_err(|e| e.to_string())?;
    require(
        manifest.schema_version == 1 && manifest.model == "geosim" && manifest.spec == SPEC,
        "invalid manifest version/model/spec",
    )?;
    require(
        manifest.source_draws == 100000
            && manifest.ks_draws == 1000
            && manifest.analysis_seed == 2026100301,
        "analysis workload changed",
    )?;
    let table_bytes = read(&root.join(TABLE))?;
    require(
        template["source_table_sha256"] == digest(&table_bytes),
        "source table differs from compiled approved contract",
    )?;
    require(
        manifest.source_table_sha256.as_deref() == Some(digest(&table_bytes).as_str()),
        "source table exact byte hash mismatch",
    )?;
    require(
        digest(manifest.method_contract_json.as_bytes()) == manifest.method_contract_sha256
            && strict_json(manifest.method_contract_json.as_bytes())? == manifest.method_contract,
        "method payload hash/equality mismatch",
    )?;
    require(
        manifest.method_contract == template["method_contract"],
        "unapproved method contract",
    )?;
    require(
        v["analysis_jobs"] == template["analysis_jobs"],
        "analysis job identities or child seed states changed",
    )?;
    // Typed children also reject unknown fields; values above bind the pinned SeedSequence outputs.
    require(
        manifest.analysis_jobs.iter().enumerate().all(|(i, j)| {
            j.index == i
                && j.root_entropy == manifest.analysis_seed
                && j.spawn_key == [i as u32]
                && j.state_u32.len() == 4
                && !j.id.is_empty()
        }),
        "invalid job contract",
    )?;
    match manifest.execution_mode.as_str() {
        "registered" => require(
            v["arms"] == template["arms"],
            "registered37-arm/1490-key contract changed",
        )?,
        "synthetic_unregistered" => {
            require(!manifest.arms.is_empty(), "synthetic manifest has no arms")?;
            require(
                manifest.arms.iter().all(|a| {
                    a.sessions > 0
                        && a.first_seed
                            .checked_add(a.sessions - 1)
                            .is_some_and(|end| end < 370000001 || a.first_seed > 370369999)
                }),
                "synthetic seeds overlap registered ranges or overflow",
            )?;
        }
        _ => return Err("unknown execution mode".into()),
    }
    match manifest.provenance_status.as_str() {
        "provisional_unfrozen" => require(
            manifest.source_inventory.is_none() && manifest.source_inventory_sha256.is_none(),
            "provisional manifest must explicitly lack frozen inventory",
        )?,
        "frozen" => {
            let entries = manifest
                .source_inventory
                .as_ref()
                .ok_or("frozen manifest lacks inventory")?;
            require(
                manifest.source_inventory_sha256.as_deref()
                    == Some(inventory_digest(root, entries)?.as_str()),
                "source inventory digest mismatch",
            )?;
        }
        _ => return Err("unknown provenance status".into()),
    }
    let mut ids = BTreeSet::new();
    let mut seeds = BTreeSet::new();
    let mut arms = Vec::new();
    for (i, a) in manifest.arms.iter().enumerate() {
        require(
            a.index == i && !a.id.is_empty() && ids.insert(a.id.clone()),
            "duplicate or invalid arm index/id",
        )?;
        for r in 0..a.sessions {
            require(
                seeds.insert((a.id.clone(), a.first_seed + r)),
                "duplicate arm/seed key",
            )?;
        }
        arms.push(ResolvedArm {
            id: a.id.clone(),
            index: a.index,
            family: a.family.clone(),
            preset: a.preset.clone(),
            sessions: a.sessions,
            first_seed: a.first_seed,
            config: resolve_config(&a.preset, &a.config_overrides)?,
        });
    }
    let payload = serde_json::to_string(
        &arms
            .iter()
            .map(|a| serde_json::json!({"id":a.id,"config":a.config}))
            .collect::<Vec<_>>(),
    )
    .map_err(|e| e.to_string())?;
    let payload_sha = digest(payload.as_bytes());
    Ok(Prepared {
        manifest,
        arms,
        payload,
        payload_sha,
    })
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Bindings {
    manifest_sha256: String,
    binary_sha256: String,
    source_inventory_sha256: Option<String>,
    build_receipt_sha256: Option<String>,
    resolved_configs_sha256: String,
}
fn panic_text(p: &(dyn std::any::Any + Send)) -> String {
    p.downcast_ref::<String>()
        .cloned()
        .or_else(|| p.downcast_ref::<&str>().map(|s| s.to_string()))
        .unwrap_or_else(|| "non-string implementation panic".into())
}
fn execute_with<C, A>(
    arm: &ResolvedArm,
    repeat: u64,
    binding: &Bindings,
    construct: C,
    advance: A,
) -> Result<Value, String>
where
    C: FnOnce(GeosimConfig, u64) -> Result<GeosimWorld, Vec<sugarscape_core::config::FieldError>>,
    A: FnOnce(&mut GeosimWorld),
{
    let seed = arm.first_seed.checked_add(repeat).ok_or("seed overflow")?;
    let mut attempt = serde_json::json!({"status":"incomplete","construction_errors":[],"panic_context":null,"recorder_error":null});
    let mut outcome = Value::Null;
    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        construct(arm.config.clone(), seed)
    })) {
        Err(p) => {
            attempt["status"] = "construction_panic".into();
            attempt["panic_context"] = panic_text(&*p).into();
        }
        Ok(Err(errors)) => {
            attempt["status"] = "construction_error".into();
            attempt["construction_errors"] =
                serde_json::to_value(errors).map_err(|e| e.to_string())?;
        }
        Ok(Ok(mut world)) => {
            if let Err(p) =
                std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| advance(&mut world)))
            {
                let message = panic_text(&*p);
                attempt["panic_context"] = message.clone().into();
                world.invalidate_after_panic(message);
            }
            if let Some(result) = world.outcome() {
                attempt["status"] = if !attempt["panic_context"].is_null()
                    || result.finish_reason == "implementation_panic"
                {
                    "implementation_panic"
                } else if result.valid {
                    "completed"
                } else {
                    "invalid"
                }
                .into();
                if result.finish_reason == "implementation_panic"
                    && attempt["panic_context"].is_null()
                {
                    attempt["panic_context"] = result
                        .invalid_reason
                        .clone()
                        .unwrap_or_else(|| "inner implementation panic".into())
                        .into();
                }
                outcome = serde_json::to_value(result)
                    .map_err(|e| format!("serialize actual Outcome: {e}"))?;
            } else {
                attempt["recorder_error"] =
                    "world advance returned without a terminal Outcome".into();
            }
        }
    }
    let mut row = serde_json::json!({"schema_version":1,"model":"geosim","arm":arm.id,"seed":seed,"arm_index":arm.index,"repeat_index":repeat,"config":arm.config,"attempt":attempt,"outcome":outcome});
    for (k, v) in serde_json::to_value(binding)
        .map_err(|e| e.to_string())?
        .as_object()
        .unwrap()
    {
        row[k] = v.clone();
    }
    Ok(row)
}
fn array(v: &Value) -> Result<&Vec<Value>, String> {
    v.as_array().ok_or_else(|| "expected array".into())
}
fn state_identity(v: &Value, cells: usize) -> Result<sugarscape_core::geosim::StateId, String> {
    keys(v, &["capital_cell", "sovereignty_generation"])?;
    let id: sugarscape_core::geosim::StateId =
        serde_json::from_value(v.clone()).map_err(|e| e.to_string())?;
    require(id.capital_cell < cells, "generation identity outside grid")?;
    Ok(id)
}
fn uint(v: &Value) -> Result<u64, String> {
    v.as_u64()
        .ok_or_else(|| "expected nonnegative integer".into())
}
fn validate_war(
    v: &Value,
    c: &GeosimConfig,
    boundary: u64,
    attempted: u64,
    censored: bool,
    valid: bool,
) -> Result<(), String> {
    keys(
        v,
        &[
            "id",
            "parents",
            "start_period",
            "end_period",
            "last_active_period",
            "active_periods",
            "elapsed_periods",
            "raw_severity",
            "exported_severity",
            "participants",
            "end_cause",
            "java_saturated",
            "java_subunit_zero",
            "fighting_periods",
        ],
    )?;
    let start = uint(&v["start_period"])?;
    let last = uint(&v["last_active_period"])?;
    let end = if censored {
        require(
            v["end_period"].is_null() && v["end_cause"] == "horizon_censored",
            "censored war has completion",
        )?;
        attempted
    } else {
        require(
            v["end_cause"].as_str().is_some_and(|s| !s.is_empty()),
            "completed war has no cause",
        )?;
        uint(&v["end_period"])?
    };
    require(
        boundary <= start && start <= last && last <= end && end <= attempted,
        "war clocks outside attempted history",
    )?;
    let fights = array(&v["fighting_periods"])?;
    let mut prior = None;
    for p in fights {
        let p = uint(p)?;
        require(
            start <= p && p <= end && prior.is_none_or(|old| old < p),
            "invalid fighting period order/range",
        )?;
        prior = Some(p);
    }
    require(
        uint(&v["active_periods"])? == fights.len() as u64
            && prior.is_none_or(|p| p == last)
            && uint(&v["elapsed_periods"])? == end - start + 1,
        "war active/elapsed clock mismatch",
    )?;
    let mut identities = BTreeSet::new();
    for p in array(&v["participants"])? {
        keys(p, &["state", "last_fighting_period"])?;
        require(
            identities.insert(state_identity(&p["state"], (c.width * c.height) as usize)?),
            "duplicate participant generation",
        )?;
        let period = uint(&p["last_fighting_period"])?;
        require(
            start <= period && period <= end,
            "participant clock outside war",
        )?;
    }
    for name in ["raw_severity", "exported_severity"] {
        require(
            v[name].as_f64().is_some_and(|x| x >= 0.0) || (!valid && v[name].is_null()),
            "invalid severity",
        )?;
    }
    if let Some(raw) = v["raw_severity"].as_f64() {
        let expected = if c.severity_export == sugarscape_core::geosim::SeverityExport::RawDamage {
            raw
        } else {
            f64::from((raw * 100.0) as i32)
        };
        require(
            v["exported_severity"].as_f64() == Some(expected),
            "resolved severity export mismatch",
        )?;
    }
    Ok(())
}
fn normalize_invalid_floats(v: &mut Value) {
    // Only required f64 fields may have null from a genuine nonfinite partial diagnostic.
    // This validation clone never replaces the raw serialized Outcome.
    const FLOATS: &[&str] = &[
        "raw_severity",
        "exported_severity",
        "previous_damage",
        "extracted_yield",
        "recurrence_residual",
        "damage",
        "measured_damage",
        "capacity_increase",
        "capacity_decrease",
        "clipping",
        "retirement_capacity",
        "reemergence_capacity",
    ];
    match v {
        Value::Object(map) => {
            for (k, v) in map {
                if FLOATS.contains(&k.as_str()) && v.is_null() {
                    *v = serde_json::json!(0.0);
                } else if ["old_commitments", "commitments", "last_damage"].contains(&k.as_str()) {
                    if let Some(a) = v.as_array_mut() {
                        for x in a {
                            if x.is_null() {
                                *x = serde_json::json!(0.0);
                            }
                        }
                    }
                }
                normalize_invalid_floats(v);
            }
        }
        Value::Array(a) => {
            for x in a {
                normalize_invalid_floats(x);
            }
        }
        _ => {}
    }
}
fn validate_outcome(v: &Value, c: &GeosimConfig, seed: u64, status: &str) -> Result<(), String> {
    keys(
        v,
        &[
            "config",
            "seed",
            "rng_mode",
            "periods",
            "attempted_period",
            "counting_start",
            "valid",
            "state_available",
            "finish_reason",
            "invalid_reason",
            "completed_wars",
            "censored_wars",
            "legacy_visible_wars",
            "exporter_backlog",
            "merges",
            "retired_states",
            "sovereign_count",
            "states",
            "cells",
            "ledger",
            "fronts",
            "resource_updates",
            "partial_period_fights",
        ],
    )?;
    require(
        v["config"] == serde_json::to_value(c).unwrap() && uint(&v["seed"])? == seed,
        "Outcome config/seed mismatch",
    )?;
    let valid = v["valid"].as_bool().ok_or("invalid availability flag")?;
    let periods = uint(&v["periods"])?;
    let attempted = uint(&v["attempted_period"])?;
    let boundary = uint(&v["counting_start"])?;
    require(
        periods <= attempted && attempted <= c.horizon().min(periods.saturating_add(1)),
        "invalid completed/attempted clocks",
    )?;
    let expected_boundary =
        if c.count_boundary == sugarscape_core::geosim::CountBoundary::AtInitialization {
            c.initialization_periods.max(1)
        } else {
            c.initialization_periods + 1
        };
    require(boundary == expected_boundary, "counting boundary mismatch")?;
    require(
        v["state_available"].is_boolean() && v["rng_mode"] == "portable_pcg64_mcg",
        "invalid state/RNG availability",
    )?;
    if valid {
        require(
            ["completed", "implementation_panic"].contains(&status)
                && periods == c.horizon()
                && attempted == periods
                && v["state_available"] == true
                && v["finish_reason"] == "horizon"
                && v["invalid_reason"].is_null(),
            "valid Outcome is not successful horizon",
        )?;
    } else {
        require(
            ["invalid", "implementation_panic"].contains(&status)
                && v["invalid_reason"].as_str().is_some_and(|s| !s.is_empty()),
            "invalid Outcome/attempt mismatch",
        )?;
        require(
            v["finish_reason"] != "implementation_panic" || status == "implementation_panic",
            "inner panic finish/status mismatch",
        )?;
    }
    let mut completed = BTreeMap::new();
    for w in array(&v["completed_wars"])? {
        validate_war(w, c, boundary, attempted, false, valid)?;
        let id = uint(&w["id"])?;
        require(completed.insert(id, w).is_none(), "duplicate completed war")?;
        if valid {
            require(
                uint(&w["end_period"])? <= periods,
                "valid history contains partial completion",
            )?;
        }
    }
    let mut censored = BTreeSet::new();
    for w in array(&v["censored_wars"])? {
        validate_war(w, c, boundary, attempted, true, valid)?;
        let id = uint(&w["id"])?;
        require(
            !completed.contains_key(&id) && censored.insert(id),
            "completed/censored identity overlap or duplicate",
        )?;
    }
    let mut queued = BTreeSet::new();
    for label in ["legacy_visible_wars", "exporter_backlog"] {
        for w in array(&v[label])? {
            let id = uint(&w["id"])?;
            require(
                completed.get(&id).is_some_and(|x| *x == w) && queued.insert(id),
                "collector mismatch/duplicate/overlap",
            )?;
        }
    }
    let positive = completed
        .iter()
        .filter(|(_, w)| w["raw_severity"].as_f64().is_some_and(|x| x > 0.0))
        .map(|(id, _)| *id)
        .collect::<BTreeSet<_>>();
    let known = queued
        .iter()
        .filter(|id| !completed[id]["raw_severity"].is_null())
        .copied()
        .collect::<BTreeSet<_>>();
    require(
        positive == known,
        "positive raw collector partition not conserved",
    )?;
    if c.completed_export == sugarscape_core::geosim::CompletedExport::AllCompleted {
        require(
            array(&v["exporter_backlog"])?.is_empty(),
            "all_completed has backlog",
        )?;
    }
    let cells = (c.width * c.height) as usize;
    let state_ids = array(&v["states"])?
        .iter()
        .map(|s| state_identity(&s["id"], cells))
        .collect::<Result<BTreeSet<_>, _>>()?;
    require(
        state_ids.len() == array(&v["states"])?.len()
            && uint(&v["sovereign_count"])? == state_ids.len() as u64,
        "duplicate sovereign identities/count mismatch",
    )?;
    let cell_ids = array(&v["cells"])?
        .iter()
        .map(|x| uint(&x["id"]))
        .collect::<Result<BTreeSet<_>, _>>()?;
    require(
        cell_ids.len() == cells && array(&v["cells"])?.len() == cells,
        "duplicate/missing grid cells",
    )?;
    for fight in array(&v["partial_period_fights"])? {
        let f = array(fight)?;
        require(f.len() == 3, "invalid partial fight tuple")?;
        state_identity(&f[0], cells)?;
        state_identity(&f[1], cells)?;
        require(
            f[2].as_f64().is_some_and(|x| x >= 0.0) || (!valid && f[2].is_null()),
            "invalid partial fight damage",
        )?;
    }

    for x in array(&v["states"])? {
        keys(
            x,
            &[
                "id",
                "capacity",
                "threshold",
                "alert",
                "campaign",
                "previous_damage",
                "newly_independent",
                "extracted_yield",
                "recurrence_residual",
            ],
        )?;
        state_identity(&x["id"], cells)?;
        if !x["campaign"].is_null() {
            state_identity(&x["campaign"], cells)?;
        }
    }
    for x in array(&v["cells"])? {
        keys(x, &["id", "owner", "last_threshold", "next_generation"])?;
        state_identity(&x["owner"], cells)?;
        require(uint(&x["id"])? < cells as u64, "cell id outside grid")?;
    }
    for x in array(&v["retired_states"])? {
        state_identity(x, cells)?;
    }
    for x in array(&v["merges"])? {
        keys(x, &["period", "survivor", "absorbed"])?;
        require(
            uint(&x["period"])? <= attempted,
            "merge outside attempted clock",
        )?;
    }
    for x in array(&v["fronts"])? {
        keys(
            x,
            &[
                "states",
                "previous",
                "actions",
                "old_commitments",
                "commitments",
                "path",
                "initiator",
                "last_damage",
                "last_victory_probabilities",
            ],
        )?;
        for id in array(&x["states"])? {
            state_identity(id, cells)?;
        }
    }
    for x in array(&v["resource_updates"])? {
        keys(
            x,
            &[
                "state",
                "period",
                "old_capacity",
                "extracted_yield",
                "applied_damage",
                "target_capacity",
                "new_capacity",
                "clipping",
                "residual",
                "reset",
            ],
        )?;
        state_identity(&x["state"], cells)?;
        require(
            uint(&x["period"])? <= attempted,
            "resource update outside attempted clock",
        )?;
    }
    keys(
        &v["ledger"],
        &[
            "attacks",
            "fighting_front_periods",
            "mutual_front_periods",
            "conquests",
            "collapses",
            "disconnections",
            "stale_claims",
            "locked_claims",
            "double_successes",
            "path_collisions",
            "shocks",
            "damage",
            "measured_damage",
            "capacity_increase",
            "capacity_decrease",
            "clipping",
            "retirement_capacity",
            "reemergence_capacity",
            "recurrence_residual",
        ],
    )?;
    let mut typed = v.clone();
    if !valid {
        normalize_invalid_floats(&mut typed);
        if let Some(fights) = typed["partial_period_fights"].as_array_mut() {
            for f in fights {
                if f[2].is_null() {
                    f[2] = serde_json::json!(0.0);
                }
            }
        }
    }
    let _: sugarscape_core::geosim::Outcome =
        serde_json::from_value(typed).map_err(|e| format!("invalid Outcome fields/types: {e}"))?;
    Ok(())
}
fn validate_record(
    v: &Value,
    arm: &ResolvedArm,
    binding: &Bindings,
) -> Result<(String, u64), String> {
    keys(
        v,
        &[
            "schema_version",
            "model",
            "arm",
            "seed",
            "arm_index",
            "repeat_index",
            "config",
            "attempt",
            "outcome",
            "manifest_sha256",
            "binary_sha256",
            "source_inventory_sha256",
            "build_receipt_sha256",
            "resolved_configs_sha256",
        ],
    )?;
    require(
        v["schema_version"] == 1 && v["model"] == "geosim",
        "invalid record schema/model",
    )?;
    for (k, want) in serde_json::to_value(binding).unwrap().as_object().unwrap() {
        require(&v[k] == want, "record provenance binding mismatch")?;
    }
    let repeat = uint(&v["repeat_index"])?;
    let seed = uint(&v["seed"])?;
    require(
        v["arm"] == arm.id
            && uint(&v["arm_index"])? == arm.index as u64
            && repeat < arm.sessions
            && arm.first_seed.checked_add(repeat) == Some(seed),
        "record exact arm/seed key mismatch",
    )?;
    require(
        v["config"] == serde_json::to_value(&arm.config).unwrap(),
        "record config differs from Rust authority",
    )?;
    let a = &v["attempt"];
    keys(
        a,
        &[
            "status",
            "construction_errors",
            "panic_context",
            "recorder_error",
        ],
    )?;
    let status = a["status"].as_str().ok_or("missing attempt status")?;
    require(
        [
            "completed",
            "invalid",
            "construction_error",
            "construction_panic",
            "implementation_panic",
            "incomplete",
        ]
        .contains(&status),
        "unknown attempt status",
    )?;
    let errors = array(&a["construction_errors"])?;
    for error in errors {
        keys(error, &["field", "message"])?;
        require(
            error["field"].is_string() && error["message"].is_string(),
            "construction errors require string field and message",
        )?;
    }
    if status == "construction_error" {
        require(!errors.is_empty(), "construction_error lacks errors")?;
    }
    if ["construction_panic", "implementation_panic"].contains(&status) {
        require(
            meaningful_context(&a["panic_context"]),
            "panic attempt requires nonempty panic_context",
        )?;
    }
    if status == "completed" {
        require(
            a["panic_context"].is_null(),
            "completed attempt contains a caught panic",
        )?;
    }

    for name in ["panic_context", "recorder_error"] {
        require(
            a[name].is_null() || a[name].is_string(),
            "invalid attempt diagnostic",
        )?;
    }
    if v["outcome"].is_null() {
        require(
            ["construction_error", "construction_panic", "incomplete"].contains(&status),
            "missing Outcome without unavailable status",
        )?;
        if status == "incomplete" {
            require(
                meaningful_context(&a["recorder_error"]),
                "incomplete requires nonempty recorder_error",
            )?;
        }
    } else {
        validate_outcome(&v["outcome"], &arm.config, seed, status)?;
    }
    Ok((arm.id.clone(), seed))
}
fn meaningful_context(value: &Value) -> bool {
    value.as_str().is_some_and(|s| {
        // Python str.strip also treats these four ASCII separators as whitespace.
        s.chars()
            .any(|c| !c.is_whitespace() && !('\u{001c}'..='\u{001f}').contains(&c))
    })
}
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct BuildReceipt {
    schema_version: u32,
    model: String,
    manifest_sha256: String,
    source_inventory_sha256: String,
    prebuild_inventory_sha256: String,
    postbuild_inventory_sha256: String,
    binary_sha256: String,
    build_command: Vec<String>,
    cwd: String,
    target: String,
    rustc_version: String,
    cargo_version: String,
    lockfile_hashes: BTreeMap<String, String>,
    features: Vec<String>,
    build_flags: Vec<String>,
}
fn tool_version(name: &str) -> Result<String, String> {
    let output = std::process::Command::new(name)
        .arg("--version")
        .output()
        .map_err(|e| e.to_string())?;
    require(output.status.success(), "tool version command failed")?;
    String::from_utf8(output.stdout)
        .map(|s| s.trim().to_owned())
        .map_err(|e| e.to_string())
}
fn receipt_binding(
    bytes: &[u8],
    root: &Path,
    manifest_sha: &str,
    binary_sha: &str,
    inventory_sha: &str,
) -> Result<String, String> {
    let receipt: BuildReceipt =
        serde_json::from_value(strict_json(bytes)?).map_err(|e| e.to_string())?;
    require(
        receipt.schema_version == 1 && receipt.model == "geosim",
        "receipt schema/model mismatch",
    )?;
    require(
        receipt.manifest_sha256 == manifest_sha
            && receipt.binary_sha256 == binary_sha
            && receipt.source_inventory_sha256 == inventory_sha
            && receipt.prebuild_inventory_sha256 == inventory_sha
            && receipt.postbuild_inventory_sha256 == inventory_sha,
        "receipt actual manifest/binary/pre-post/current source binding mismatch",
    )?;
    require(
        Path::new(&receipt.cwd)
            .canonicalize()
            .map_err(|e| e.to_string())?
            == root.canonicalize().map_err(|e| e.to_string())?,
        "receipt build cwd differs from source root",
    )?;
    require(
        receipt
            .build_command
            .iter()
            .any(|s| Path::new(s).file_name().is_some_and(|n| n == "cargo"))
            && receipt.build_command.iter().any(|s| s == "geosim")
            && receipt
                .build_command
                .iter()
                .any(|s| s.ends_with("survey/Cargo.toml")),
        "receipt lacks explicit native recorder build command",
    )?;
    require(
        !receipt.target.is_empty()
            && receipt.rustc_version == tool_version("rustc")?
            && receipt.cargo_version == tool_version("cargo")?,
        "receipt target/toolchain version mismatch",
    )?;
    require(
        receipt
            .features
            .iter()
            .chain(&receipt.build_flags)
            .all(|s| !s.is_empty()),
        "invalid receipt feature/build flag",
    )?;
    require(
        ["Cargo.lock", "survey/Cargo.lock"]
            .iter()
            .all(|name| receipt.lockfile_hashes.contains_key(*name)),
        "receipt omits required Cargo locks",
    )?;
    for (name, want) in &receipt.lockfile_hashes {
        require(
            digest(&read(&safe_path(root, name)?)?) == *want,
            "receipt lockfile actual-byte mismatch",
        )?;
    }
    Ok(digest(bytes))
}
#[derive(Default)]
struct Args {
    values: BTreeMap<String, String>,
    validate: bool,
    allow_fixture: bool,
}
fn args() -> Result<Args, String> {
    let mut result = Args::default();
    let mut input = std::env::args().skip(1);
    while let Some(flag) = input.next() {
        match flag.as_str() {
            "--validate" => {
                require(!result.validate, "duplicate --validate")?;
                result.validate = true;
            }
            "--allow-unfrozen-fixture" => {
                require(!result.allow_fixture, "duplicate fixture opt-in")?;
                result.allow_fixture = true;
            }
            "--manifest" | "--manifest-sha256" | "--source-root" | "--build-receipt"
            | "--resolved-out" | "--out" | "--arm" => {
                let value = input
                    .next()
                    .ok_or_else(|| format!("missing {flag} value"))?;
                require(
                    !value.starts_with("--") && result.values.insert(flag.clone(), value).is_none(),
                    "missing or duplicate option",
                )?;
            }
            _ => return Err(format!("unknown option {flag}")),
        }
    }
    Ok(result)
}
fn option<'a>(args: &'a Args, name: &str) -> Result<&'a str, String> {
    args.values
        .get(name)
        .map(String::as_str)
        .ok_or_else(|| format!("required option {name}"))
}
fn resume(
    path: &Path,
    prepared: &Prepared,
    binding: &Bindings,
) -> Result<BTreeSet<(String, u64)>, String> {
    use std::io::BufRead;
    let mut keys = BTreeSet::new();
    let file = match std::fs::File::open(path) {
        Ok(f) => f,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(keys),
        Err(e) => return Err(e.to_string()),
    };
    let arms = prepared
        .arms
        .iter()
        .map(|a| (a.id.as_str(), a))
        .collect::<BTreeMap<_, _>>();
    let mut reader = std::io::BufReader::new(file);
    let mut line = Vec::new();
    let mut number = 0;
    loop {
        line.clear();
        if reader
            .read_until(b'\n', &mut line)
            .map_err(|e| e.to_string())?
            == 0
        {
            break;
        }
        number += 1;
        require(
            line.last() == Some(&b'\n'),
            "resume has incomplete trailing JSON line",
        )?;
        let row = strict_json(&line).map_err(|e| format!("raw line {number}: {e}"))?;
        let arm = arms
            .get(row["arm"].as_str().ok_or("record arm missing")?)
            .ok_or("unregistered resumed arm")?;
        let key =
            validate_record(&row, arm, binding).map_err(|e| format!("raw line {number}: {e}"))?;
        require(keys.insert(key), "duplicate resumed exact arm/seed key")?;
    }
    Ok(keys)
}
fn atomic_json(path: &Path, value: &Value) -> Result<(), String> {
    use std::io::Write;
    if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let temporary = path.with_extension(format!("tmp-{}", std::process::id()));
    let bytes = serde_json::to_vec_pretty(value).map_err(|e| e.to_string())?;
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temporary)
        .map_err(|e| format!("cannot create atomic output {}: {e}", temporary.display()))?;
    file.write_all(&bytes).map_err(|e| e.to_string())?;
    drop(file);
    std::fs::rename(&temporary, path).map_err(|e| e.to_string())
}
fn output_destination(path: &Path) -> Result<PathBuf, String> {
    use std::path::Component;
    require(
        !path.as_os_str().is_empty(),
        "output path must not be empty",
    )?;
    let absolute = if path.is_absolute() {
        path.to_owned()
    } else {
        std::env::current_dir()
            .map_err(|e| e.to_string())?
            .join(path)
    };
    let mut destination = PathBuf::new();
    let mut components = absolute.components().peekable();
    while let Some(component) = components.next() {
        match component {
            Component::Prefix(_) | Component::RootDir => destination.push(component.as_os_str()),
            Component::CurDir => {}
            Component::ParentDir => {
                destination.pop();
            }
            Component::Normal(name) => {
                destination.push(name);
                match std::fs::symlink_metadata(&destination) {
                    Ok(_) => {
                        destination = std::fs::canonicalize(&destination).map_err(|e| {
                            format!("cannot resolve output {}: {e}", destination.display())
                        })?;
                        let metadata =
                            std::fs::metadata(&destination).map_err(|e| e.to_string())?;
                        require(
                            if components.peek().is_some() { metadata.is_dir() } else { metadata.is_file() },
                            "output ancestor must be a directory and existing destination a regular file",
                        )?;
                    }
                    Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
                    Err(e) => {
                        return Err(format!(
                            "cannot inspect output {}: {e}",
                            destination.display()
                        ))
                    }
                }
            }
        }
    }
    Ok(destination)
}
fn run_cli(args: Args) -> Result<(), String> {
    use std::io::Write;
    let root = Path::new(option(&args, "--source-root")?);
    let bytes = read(Path::new(option(&args, "--manifest")?))?;
    let manifest_sha = option(&args, "--manifest-sha256")?;
    let prepared = prepare(&bytes, root, manifest_sha)?;
    let frozen = prepared.manifest.provenance_status == "frozen";
    let registered = prepared.manifest.execution_mode == "registered";
    if registered {
        require(
            !args.allow_fixture,
            "fixture bypass is forbidden for registered manifests",
        )?;
        require(
            args.validate || frozen,
            "registered execution requires frozen source/build bindings",
        )?;
    } else if !frozen {
        require(
            args.allow_fixture,
            "synthetic unfrozen execution/validation requires explicit --allow-unfrozen-fixture",
        )?;
    }
    let selected = args.values.get("--arm");
    if let Some(id) = selected {
        require(
            prepared.arms.iter().any(|a| &a.id == id),
            "--arm must match an exact registered id",
        )?;
    }
    let binary_sha = digest(&read(&std::env::current_exe().map_err(|e| e.to_string())?)?);
    let receipt_sha = if frozen {
        let receipt = read(Path::new(option(&args, "--build-receipt")?))?;
        Some(receipt_binding(
            &receipt,
            root,
            manifest_sha,
            &binary_sha,
            prepared
                .manifest
                .source_inventory_sha256
                .as_deref()
                .ok_or("frozen inventory missing")?,
        )?)
    } else {
        require(
            !args.values.contains_key("--build-receipt"),
            "unfrozen manifest cannot claim a frozen build receipt",
        )?;
        None
    };
    let binding = Bindings {
        manifest_sha256: manifest_sha.into(),
        binary_sha256: binary_sha,
        source_inventory_sha256: prepared.manifest.source_inventory_sha256.clone(),
        build_receipt_sha256: receipt_sha,
        resolved_configs_sha256: prepared.payload_sha.clone(),
    };
    let mut export = serde_json::json!({"schema_version":1,"model":"geosim","resolved_configs_json":prepared.payload,"arms":prepared.arms});
    for (k, v) in serde_json::to_value(&binding).unwrap().as_object().unwrap() {
        export[k] = v.clone();
    }
    let out = Path::new(option(&args, "--out")?);
    let resolved_out = args.values.get("--resolved-out").map_or_else(
        || {
            let mut name = out.as_os_str().to_owned();
            name.push(".resolved.json");
            std::path::PathBuf::from(name)
        },
        std::path::PathBuf::from,
    );
    let history_destination = output_destination(out)?;
    let resolved_destination = output_destination(&resolved_out)?;
    // Missing paths cannot be canonicalized. These platforms commonly alias ASCII case,
    // so conservatively reject such spellings even on their case-sensitive volumes.
    let same_destination = history_destination == resolved_destination
        || (cfg!(any(target_os = "macos", target_os = "windows"))
            && history_destination
                .as_os_str()
                .as_encoded_bytes()
                .eq_ignore_ascii_case(resolved_destination.as_os_str().as_encoded_bytes()));
    require(
        !same_destination,
        "resolved output must differ from history output",
    )?;
    let seen = resume(out, &prepared, &binding)?;
    atomic_json(&resolved_out, &export)?;
    if args.validate {
        println!(
            "validated {} arms/{} keys without world construction; provenance={}",
            prepared.arms.len(),
            prepared.arms.iter().map(|a| a.sessions).sum::<u64>(),
            prepared.manifest.provenance_status
        );
        return Ok(());
    }
    if let Some(parent) = out.parent().filter(|p| !p.as_os_str().is_empty()) {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(out)
        .map_err(|e| e.to_string())?;
    for arm in &prepared.arms {
        if selected.is_some_and(|id| id != &arm.id) {
            continue;
        }
        for repeat in 0..arm.sessions {
            let seed = arm.first_seed + repeat;
            if seen.contains(&(arm.id.clone(), seed)) {
                continue;
            }
            let row = execute_with(arm, repeat, &binding, GeosimWorld::new, |w| {
                w.run(
                    w.config
                        .horizon()
                        .div_ceil(u64::from(w.config.periods_per_tick)) as u32,
                )
            })?;
            validate_record(&row, arm, &binding)?;
            let encoded = serde_json::to_vec(&row).map_err(|e| e.to_string())?;
            file.write_all(&encoded)
                .and_then(|()| file.write_all(b"\n"))
                .and_then(|()| file.flush())
                .map_err(|e| e.to_string())?;
            if row["attempt"]["status"] == "incomplete" {
                return Err("incomplete attempt preserved; no terminal Outcome available".into());
            }
        }
    }
    Ok(())
}
fn main() {
    if let Err(error) = args().and_then(run_cli) {
        eprintln!("GeoSim recorder: {error}");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn atomic_export_never_clobbers_an_existing_temporary_destination() {
        let directory = std::env::temp_dir().join(format!(
            "geosim-atomic-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir(&directory).unwrap();
        let resolved = directory.join("resolved.json");
        let temporary = resolved.with_extension(format!("tmp-{}", std::process::id()));
        std::fs::write(&resolved, b"existing resolved receipt").unwrap();
        std::fs::write(&temporary, b"existing history bytes").unwrap();
        let result = atomic_json(&resolved, &json!({"resolved_configs_json":"new export"}));
        assert!(
            result.is_err(),
            "existing atomic temporary destination accepted"
        );
        assert_eq!(
            std::fs::read(&temporary).unwrap(),
            b"existing history bytes"
        );
        assert_eq!(
            std::fs::read(&resolved).unwrap(),
            b"existing resolved receipt"
        );
        #[cfg(unix)]
        {
            std::fs::remove_file(&temporary).unwrap();
            let history = directory.join("history.jsonl");
            std::fs::write(&history, b"existing history through symlink").unwrap();
            std::os::unix::fs::symlink(&history, &temporary).unwrap();
            assert!(atomic_json(&resolved, &json!({"x":1})).is_err());
            assert_eq!(
                std::fs::read(&history).unwrap(),
                b"existing history through symlink"
            );
            assert!(std::fs::symlink_metadata(&temporary)
                .unwrap()
                .file_type()
                .is_symlink());
        }
        std::fs::remove_dir_all(directory).unwrap();
    }
    #[test]
    fn hash_is_actual_sha256_not_a_caller_label() {
        assert_eq!(
            digest(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }
    #[test]
    fn recursive_duplicate_fields_and_nonfinite_tokens_are_rejected() {
        for bytes in [
            b"{\"nested\":{\"seed\":1,\"seed\":2}}".as_slice(),
            b"{\"x\":NaN}",
            b"{\"x\":1e999}",
        ] {
            assert!(strict_json(bytes).is_err());
        }
    }
    #[test]
    fn rust_paper_defaults_and_overrides_are_authoritative() {
        let c=resolve_config("paper", &json!({"width":2,"height":2,"initial_states":1,"observation_periods":13,"initialization_periods":0,"periods_per_tick":5})).unwrap();
        assert_eq!(
            (c.width, c.height, c.horizon(), c.periods_per_tick),
            (2, 2, 13, 5)
        );
        assert_eq!(
            c.defender_threshold,
            sugarscape_core::geosim::DefenderThreshold::Reciprocal
        );
    }
    #[test]
    fn artifact_resolution_uses_actual_named_preset() {
        let c = resolve_config("artifact_2017", &json!({})).unwrap();
        assert_eq!(c, GeosimConfig::artifact_2017());
    }
    #[test]
    fn unknown_config_keys_readings_and_presets_fail_before_construction() {
        for (preset, overrides) in [
            ("paper", json!({"unknown":1})),
            ("paper", json!({"damage_incidence":"unknown"})),
            ("absent", json!({})),
        ] {
            assert!(resolve_config(preset, &overrides).is_err());
        }
    }
}

#[cfg(test)]
mod manifest_tests {
    use super::*;
    const REGISTERED: &str = include_str!("../../tests/fixtures/geosim-registered-manifest.json");
    fn root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .to_owned()
    }
    #[test]
    fn all_registered_configs_are_resolved_without_world_construction() {
        let p = prepare(
            REGISTERED.as_bytes(),
            &root(),
            &digest(REGISTERED.as_bytes()),
        )
        .unwrap();
        assert_eq!(p.arms.len(), 37);
        assert_eq!(p.arms.iter().map(|a| a.sessions).sum::<u64>(), 1490);
        assert_eq!(p.arms[6].config.superiority_threshold, 2.5);
        assert_eq!(p.arms[6].config.victory_threshold, 2.5);
        assert_eq!(p.arms[6].config.shock_shift, 10.0);
        assert_eq!(
            p.arms[36].config.numerical_policy,
            sugarscape_core::geosim::NumericalPolicy::FloorZero
        );
        let values: Value = strict_json(p.payload.as_bytes()).unwrap();
        assert_eq!(values.as_array().unwrap().len(), 37);
        assert_eq!(p.payload_sha, digest(p.payload.as_bytes()));
    }
    #[test]
    fn exact_manifest_hash_and_registration_changes_are_rejected() {
        assert!(prepare(REGISTERED.as_bytes(), &root(), &"a".repeat(64)).is_err());
        for field in ["first_seed", "sessions", "index"] {
            let mut v = strict_json(REGISTERED.as_bytes()).unwrap();
            v["arms"][0][field] = serde_json::json!(999);
            let b = serde_json::to_vec(&v).unwrap();
            assert!(prepare(&b, &root(), &digest(&b)).is_err());
        }
    }
    #[test]
    fn unknown_manifest_fields_and_changed_method_payload_fail() {
        let mut v = strict_json(REGISTERED.as_bytes()).unwrap();
        v["unknown"] = serde_json::json!(true);
        let b = serde_json::to_vec(&v).unwrap();
        assert!(prepare(&b, &root(), &digest(&b)).is_err());
        let mut v = strict_json(REGISTERED.as_bytes()).unwrap();
        v["method_contract_json"] = serde_json::json!("{}");
        let b = serde_json::to_vec(&v).unwrap();
        assert!(prepare(&b, &root(), &digest(&b)).is_err());
    }
}

#[cfg(test)]
mod record_tests {
    use super::*;
    use serde_json::json;
    fn arm() -> ResolvedArm {
        ResolvedArm{id:"fixture.tiny".into(),index:0,family:"fixture".into(),preset:"paper".into(),sessions:1,first_seed:41,config:resolve_config("paper",&json!({"width":2,"height":2,"initial_states":4,"initialization_periods":0,"observation_periods":4,"periods_per_tick":4,"attack_probability":1,"superiority_threshold":0.1,"resource_adjustment":1,"damage_fraction":1})).unwrap()}
    }
    fn binding() -> Bindings {
        Bindings {
            manifest_sha256: digest(b"manifest"),
            binary_sha256: digest(b"self"),
            source_inventory_sha256: None,
            build_receipt_sha256: None,
            resolved_configs_sha256: digest(b"configs"),
        }
    }
    #[test]
    fn actual_invalid_world_retains_attempted_clock_and_exact_resume_integrity() {
        let a = arm();
        let b = binding();
        let row = execute_with(&a, 0, &b, GeosimWorld::new, |w| w.run(1)).unwrap();
        assert_eq!(row["attempt"]["status"], "invalid");
        validate_record(&row, &a, &b).unwrap();
        for name in ["binary_sha256", "resolved_configs_sha256"] {
            let mut changed = row.clone();
            changed[name] = json!("a".repeat(64));
            assert!(validate_record(&changed, &a, &b).is_err());
        }
        let mut changed = row.clone();
        changed["config"]["war_shadow"] = json!(999);
        assert!(validate_record(&changed, &a, &b).is_err());
        let mut changed = row.clone();
        changed["outcome"]["attempted_period"] = json!(999);
        assert!(validate_record(&changed, &a, &b).is_err());
    }
    #[test]
    fn outer_panic_after_terminal_outcome_is_never_reported_completed() {
        let mut a = arm();
        a.config.damage_fraction = 0.0;
        let b = binding();
        let row = execute_with(&a, 0, &b, GeosimWorld::new, |w| {
            w.run(1);
            panic!("postfinish motif");
        })
        .unwrap();
        assert_eq!(row["outcome"]["valid"], true);
        assert_eq!(row["attempt"]["status"], "implementation_panic");
        validate_record(&row, &a, &b).unwrap();
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("out/geosim-protocol-implementation/native-postfinish-panic-fixture.json");
        atomic_json(&path, &row).unwrap();
    }
    #[test]
    fn construction_failure_outer_panic_inner_finish_and_incomplete_have_distinct_status() {
        let a = arm();
        let b = binding();
        let row = execute_with(
            &a,
            0,
            &b,
            |_, _| {
                Err(vec![sugarscape_core::config::FieldError::new(
                    "width",
                    "fixture rejected",
                )])
            },
            |_| {},
        )
        .unwrap();
        assert_eq!(row["attempt"]["status"], "construction_error");
        assert!(row["outcome"].is_null());
        validate_record(&row, &a, &b).unwrap();
        let row = execute_with(&a, 0, &b, |_, _| panic!("constructor motif"), |_| {}).unwrap();
        assert_eq!(row["attempt"]["status"], "construction_panic");
        assert!(row["outcome"].is_null());
        let row = execute_with(&a, 0, &b, GeosimWorld::new, |_| panic!("advance motif")).unwrap();
        assert_eq!(row["attempt"]["status"], "implementation_panic");
        assert!(row["outcome"].is_object());
        validate_record(&row, &a, &b).unwrap();
        let row = execute_with(&a, 0, &b, GeosimWorld::new, |w| {
            w.invalidate_after_panic("inner caught motif".into())
        })
        .unwrap();
        assert_eq!(row["attempt"]["status"], "implementation_panic");
        assert!(row["attempt"]["panic_context"]
            .as_str()
            .unwrap()
            .contains("inner caught motif"));
        let row = execute_with(&a, 0, &b, GeosimWorld::new, |_| {}).unwrap();
        assert_eq!(row["attempt"]["status"], "incomplete");
        assert!(row["outcome"].is_null());
        validate_record(&row, &a, &b).unwrap();
    }
}

#[cfg(test)]
mod provenance_tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn inventory_paths_are_canonical_and_source_symlinks_are_forbidden() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .to_owned();
        assert!(safe_path(&root, "survey//Cargo.toml").is_err());
        assert!(safe_path(&root, "../Cargo.toml").is_err());
        #[cfg(unix)]
        {
            let path =
                std::env::temp_dir().join(format!("geosim-inventory-path-{}", std::process::id()));
            std::fs::create_dir_all(&path).unwrap();
            std::fs::write(path.join("real"), b"source").unwrap();
            std::os::unix::fs::symlink(path.join("real"), path.join("alias")).unwrap();
            assert!(safe_path(&path, "alias").is_err());
            std::fs::remove_dir_all(path).unwrap();
        }
    }
    #[test]
    fn receipt_must_bind_actual_manifest_binary_sources_pre_post_and_locks() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .to_owned();
        let manifest = digest(b"manifest");
        let binary = digest(b"binary");
        let inventory = digest(b"inventory");
        let v = json!({"schema_version":1,"model":"geosim","manifest_sha256":manifest,"binary_sha256":binary,"source_inventory_sha256":inventory,"prebuild_inventory_sha256":inventory,"postbuild_inventory_sha256":inventory,
   "build_command":["cargo","build","--manifest-path","survey/Cargo.toml","--bin","geosim"],"cwd":root,"target":"aarch64-apple-darwin","rustc_version":tool_version("rustc").unwrap(),"cargo_version":tool_version("cargo").unwrap(),"lockfile_hashes":{"Cargo.lock":digest(&read(&root.join("Cargo.lock")).unwrap()),"survey/Cargo.lock":digest(&read(&root.join("survey/Cargo.lock")).unwrap())},"features":[],"build_flags":[]});
        let bytes = serde_json::to_vec(&v).unwrap();
        assert_eq!(
            receipt_binding(&bytes, &root, &manifest, &binary, &inventory).unwrap(),
            digest(&bytes)
        );
        for name in [
            "manifest_sha256",
            "binary_sha256",
            "source_inventory_sha256",
            "prebuild_inventory_sha256",
            "postbuild_inventory_sha256",
        ] {
            let mut changed = v.clone();
            changed[name] = json!("a".repeat(64));
            assert!(receipt_binding(
                &serde_json::to_vec(&changed).unwrap(),
                &root,
                &manifest,
                &binary,
                &inventory
            )
            .is_err());
        }
        let mut changed = v.clone();
        changed["lockfile_hashes"]["survey/Cargo.lock"] = json!("a".repeat(64));
        assert!(receipt_binding(
            &serde_json::to_vec(&changed).unwrap(),
            &root,
            &manifest,
            &binary,
            &inventory
        )
        .is_err());
        let mut changed = v;
        changed["unknown"] = json!(true);
        assert!(receipt_binding(
            &serde_json::to_vec(&changed).unwrap(),
            &root,
            &manifest,
            &binary,
            &inventory
        )
        .is_err());
    }
}

#[cfg(test)]
mod census_tests {
    use super::*;
    use serde_json::json;
    fn motif() -> (ResolvedArm, Bindings, Value) {
        let config=resolve_config("paper",&json!({"width":2,"height":2,"initial_states":1,"initialization_periods":0,"observation_periods":4,"periods_per_tick":4,"attack_probability":0,"shock_probability":0,"severity_export":"java_int100"})).unwrap();
        let arm = ResolvedArm {
            id: "fixture.census".into(),
            index: 0,
            family: "fixture".into(),
            preset: "paper".into(),
            sessions: 1,
            first_seed: 51,
            config,
        };
        let binding = Bindings {
            manifest_sha256: digest(b"m"),
            binary_sha256: digest(b"b"),
            source_inventory_sha256: None,
            build_receipt_sha256: None,
            resolved_configs_sha256: digest(b"r"),
        };
        let mut row = execute_with(&arm, 0, &binding, GeosimWorld::new, |w| w.run(1)).unwrap();
        let war = json!({"id":9,"parents":[],"start_period":1,"end_period":2,"last_active_period":1,"active_periods":1,"elapsed_periods":2,"raw_severity":0.001,"exported_severity":0.0,"participants":[{"state":{"capital_cell":0,"sovereignty_generation":7},"last_fighting_period":1}],"end_cause":"shadow_expired","java_saturated":false,"java_subunit_zero":true,"fighting_periods":[1]});
        row["outcome"]["completed_wars"] = json!([war.clone()]);
        row["outcome"]["legacy_visible_wars"] = json!([war]);
        (arm, binding, row)
    }
    #[test]
    fn positive_subunit_zero_requires_collector_membership_and_zero_raw_forbids_it() {
        let (a, b, row) = motif();
        validate_record(&row, &a, &b).unwrap();
        let mut changed = row.clone();
        changed["outcome"]["legacy_visible_wars"] = json!([]);
        assert!(validate_record(&changed, &a, &b).is_err());
        let mut changed = row.clone();
        changed["outcome"]["exporter_backlog"] = changed["outcome"]["legacy_visible_wars"].clone();
        assert!(validate_record(&changed, &a, &b).is_err());
        let mut changed = row.clone();
        for list in ["completed_wars", "legacy_visible_wars"] {
            changed["outcome"][list][0]["raw_severity"] = json!(0.0);
        }
        assert!(validate_record(&changed, &a, &b).is_err());
        changed["outcome"]["legacy_visible_wars"] = json!([]);
        validate_record(&changed, &a, &b).unwrap();
    }
    #[test]
    fn legacy_backlog_keeps_positive_raw_even_when_integer_export_is_zero() {
        let (mut a, b, mut row) = motif();
        a.config.completed_export = sugarscape_core::geosim::CompletedExport::OnePerPeriod;
        row["config"] = serde_json::to_value(&a.config).unwrap();
        row["outcome"]["config"] = row["config"].clone();
        row["outcome"]["exporter_backlog"] = row["outcome"]["legacy_visible_wars"].clone();
        row["outcome"]["legacy_visible_wars"] = serde_json::json!([]);
        validate_record(&row, &a, &b).unwrap();
    }
    #[test]
    fn completed_and_censored_identity_are_disjoint_and_generations_are_exact() {
        let (a, b, row) = motif();
        let mut changed = row.clone();
        let mut censored = changed["outcome"]["completed_wars"][0].clone();
        censored["end_period"] = Value::Null;
        censored["end_cause"] = json!("horizon_censored");
        censored["elapsed_periods"] = json!(4);
        changed["outcome"]["censored_wars"] = json!([censored]);
        assert!(validate_record(&changed, &a, &b).is_err());
        let mut changed = row;
        changed["outcome"]["completed_wars"][0]["participants"][0]["state"]["capital_cell"] =
            json!(4);
        assert!(validate_record(&changed, &a, &b).is_err());
    }
    #[test]
    fn partial_completion_and_null_ledger_remain_raw_but_valid_horizon_rejects_them() {
        let (a, b, mut row) = motif();
        row["attempt"]["status"] = json!("invalid");
        row["outcome"]["valid"] = json!(false);
        row["outcome"]["periods"] = json!(1);
        row["outcome"]["attempted_period"] = json!(2);
        row["outcome"]["finish_reason"] = json!("invalid");
        row["outcome"]["invalid_reason"] = json!("partial motif");
        row["outcome"]["ledger"]["damage"] = Value::Null;
        row["outcome"]["resource_updates"] = json!([]);
        let original = row.clone();
        validate_record(&row, &a, &b).unwrap();
        assert_eq!(row, original);
        row["attempt"]["status"] = json!("completed");
        row["outcome"]["valid"] = json!(true);
        assert!(validate_record(&row, &a, &b).is_err());
    }
    #[test]
    fn duplicate_cells_states_and_unknown_partial_identity_fields_are_rejected() {
        let (a, b, row) = motif();
        let mut changed = row.clone();
        changed["outcome"]["cells"][1] = changed["outcome"]["cells"][0].clone();
        assert!(validate_record(&changed, &a, &b).is_err());
        let mut changed = row.clone();
        let state = changed["outcome"]["states"][0].clone();
        changed["outcome"]["states"]
            .as_array_mut()
            .unwrap()
            .push(state);
        assert!(validate_record(&changed, &a, &b).is_err());
        let mut changed = row;
        changed["outcome"]["partial_period_fights"] = json!([[{"capital_cell":0,"sovereignty_generation":7,"unknown":true},{"capital_cell":1,"sovereignty_generation":0},0]]);
        assert!(validate_record(&changed, &a, &b).is_err());
    }
}
