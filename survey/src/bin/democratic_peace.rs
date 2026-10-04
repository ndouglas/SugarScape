//! Authoritative native democratic-peace recorder; registered runs require frozen identities.
use serde::de::{self, MapAccess, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::fmt;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{self, OpenOptions},
    io::{BufRead, BufReader, Write},
    path::{Path, PathBuf},
};
use sugarscape_core::democratic_peace::{DemocraticPeaceConfig, DemocraticPeaceWorld};

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

const SPEC: &str = "docs/superpowers/specs/2026-10-03-democratic-peace-design.md";
const TABLE: &str = "docs/superpowers/specs/2026-10-03-democratic-peace-source-table.json";
fn require(ok: bool, message: &str) -> Result<(), String> {
    if ok {
        Ok(())
    } else {
        Err(message.into())
    }
}
fn object(v: &Value) -> Result<&serde_json::Map<String, Value>, String> {
    v.as_object().ok_or_else(|| "expected JSON object".into())
}
fn text<'a>(v: &'a Value, k: &str) -> Result<&'a str, String> {
    v.get(k)
        .and_then(Value::as_str)
        .ok_or_else(|| format!("missing string {k}"))
}
fn uint(v: &Value, k: &str) -> Result<u64, String> {
    v.get(k)
        .and_then(Value::as_u64)
        .ok_or_else(|| format!("missing unsigned integer {k}"))
}
fn array<'a>(v: &'a Value, k: &str) -> Result<&'a Vec<Value>, String> {
    v.get(k)
        .and_then(Value::as_array)
        .ok_or_else(|| format!("missing array {k}"))
}
fn exact(v: &Value, keys: &[&str]) -> Result<(), String> {
    let got = object(v)?
        .keys()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    require(
        got == keys.iter().copied().collect(),
        "unknown or missing JSON fields",
    )
}
// Explicit CLI paths may use OS aliases such as /var -> /private/var.
// Source-inventory paths instead reject every symlink below the canonical repo.
fn safe_path(p: &Path) -> Result<(), String> {
    require(
        !p.components()
            .any(|c| matches!(c, std::path::Component::ParentDir)),
        "parent traversal forbidden",
    )?;
    if let Ok(m) = fs::symlink_metadata(p) {
        require(!m.file_type().is_symlink(), "symlink paths forbidden")?;
    }
    Ok(())
}
fn source_path(root: &Path, relative: &Path) -> Result<PathBuf, String> {
    let mut path = root.to_path_buf();
    for component in relative.components() {
        require(
            matches!(component, std::path::Component::Normal(_)),
            "unsafe source path",
        )?;
        path.push(component.as_os_str());
        safe_path(&path)?;
    }
    Ok(path)
}
fn read(p: &Path) -> Result<Vec<u8>, String> {
    safe_path(p)?;
    fs::read(p).map_err(|e| format!("{}: {e}", p.display()))
}
fn hash_json(v: &Value) -> Result<String, String> {
    Ok(digest(&serde_json::to_vec(v).map_err(|e| e.to_string())?))
}
fn is_hash(s: &str) -> bool {
    s.len() == 64
        && s.bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}
fn atomic_write(p: &Path, bytes: &[u8]) -> Result<(), String> {
    safe_path(p)?;
    let temporary = p.with_extension(format!("tmp-{}", std::process::id()));
    safe_path(&temporary)?;
    let mut f = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temporary)
        .map_err(|e| e.to_string())?;
    f.write_all(bytes).map_err(|e| e.to_string())?;
    f.sync_all().map_err(|e| e.to_string())?;
    fs::rename(&temporary, p).map_err(|e| e.to_string())
}
fn walk_files(root: &Path, path: &Path, out: &mut BTreeSet<String>) -> Result<(), String> {
    safe_path(path)?;
    for entry in fs::read_dir(path).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let p = entry.path();
        safe_path(&p)?;
        if p.is_dir() {
            walk_files(root, &p, out)?
        } else if p
            .extension()
            .is_some_and(|x| x == "rs" || x == "py" || x == "json")
        {
            out.insert(
                p.strip_prefix(root)
                    .map_err(|e| e.to_string())?
                    .to_string_lossy()
                    .into(),
            );
        }
    }
    Ok(())
}
fn validate_inventory(m: &Value, root: &Path, required: bool) -> Result<Option<String>, String> {
    let Some(entries) = m.get("source_inventory").and_then(Value::as_array) else {
        require(!required, "source inventory required")?;
        require(
            m["source_inventory_sha256"].is_null(),
            "partial inventory identity",
        )?;
        return Ok(None);
    };
    let declared = text(m, "source_inventory_sha256")?;
    require(is_hash(declared), "invalid inventory digest")?;
    require(
        hash_json(&m["source_inventory"])? == declared,
        "inventory digest mismatch",
    )?;
    let mut names = BTreeSet::new();
    let mut prior = String::new();
    for entry in entries {
        exact(entry, &["path", "bytes", "sha256"])?;
        let name = text(entry, "path")?;
        let rel = Path::new(name);
        require(
            !rel.is_absolute()
                && !name.contains('\\')
                && !name.contains('\0')
                && !name.contains('\n')
                && !name.contains('\r'),
            "unsafe inventory path",
        )?;
        require(
            name.split('/')
                .all(|x| !x.is_empty() && x != ".." && x != "."),
            "unsafe inventory path",
        )?;
        require(
            prior.as_str() < name,
            "inventory paths must be unique and sorted",
        )?;
        prior = name.into();
        names.insert(name.into());
        let data = read(&source_path(root, rel)?)?;
        require(
            uint(entry, "bytes")? == data.len() as u64,
            "inventory byte count mismatch",
        )?;
        require(
            text(entry, "sha256")? == digest(&data),
            "changed source inventory bytes",
        )?;
    }
    let mut needed = BTreeSet::from([
        "Cargo.toml".into(),
        "Cargo.lock".into(),
        "crates/sugarscape-core/Cargo.toml".into(),
        "survey/Cargo.toml".into(),
        "survey/Cargo.lock".into(),
        "survey/src/bin/democratic_peace.rs".into(),
        SPEC.into(),
        TABLE.into(),
        "docs/superpowers/specs/2026-10-03-democratic-peace-source-review.json".into(),
        "docs/superpowers/specs/2026-10-03-democratic-peace-source-extraction.md".into(),
    ]);
    walk_files(root, &root.join("crates/sugarscape-core/src"), &mut needed)?;
    walk_files(root, &root.join("survey/democratic_peace"), &mut needed)?;
    require(
        needed.is_subset(&names),
        "source inventory omits production, methods or source evidence",
    )?;
    Ok(Some(declared.into()))
}
struct Arm {
    id: String,
    index: u64,
    canonical: u64,
    family: String,
    sessions: u64,
    first_seed: u64,
    config: DemocraticPeaceConfig,
}
struct Prepared {
    manifest: Value,
    arms: Vec<Arm>,
    payload: Vec<u8>,
    payload_sha: String,
    manifest_sha: String,
    binary_sha: String,
    inventory_sha: Option<String>,
}
fn prepare(path: &Path, root: &Path, will_run: bool) -> Result<Prepared, String> {
    let bytes = read(path)?;
    let m = strict_json(&bytes)?;
    exact(
        &m,
        &[
            "schema_version",
            "model",
            "execution_mode",
            "provenance_status",
            "spec",
            "precision_registered",
            "runtime_gate",
            "analysis_seed",
            "source_draws",
            "permutation_draws",
            "contrast_draws",
            "source_table_sha256",
            "method_contract",
            "method_contract_json",
            "method_contract_sha256",
            "source_inventory",
            "source_inventory_sha256",
            "arms",
            "analysis_jobs",
        ],
    )?;
    require(
        uint(&m, "schema_version")? == 1 && text(&m, "model")? == "democratic_peace",
        "unsupported manifest schema/model",
    )?;
    require(text(&m, "spec")? == SPEC, "wrong approved specification")?;
    let mode = text(&m, "execution_mode")?;
    require(
        ["registered", "fixture", "runtime_probe"].contains(&mode),
        "unknown execution mode",
    )?;
    let registered = mode == "registered";
    let provenance = text(&m, "provenance_status")?;
    require(
        if registered {
            ["provisional_unfrozen", "frozen"].contains(&provenance)
        } else if mode == "fixture" {
            provenance == "fixture_unregistered"
        } else {
            provenance == "runtime_probe_unregistered"
        },
        "execution/provenance disagreement",
    )?;
    if will_run && registered {
        require(
            provenance == "frozen",
            "registered execution requires frozen provenance",
        )?;
    }
    require(
        uint(&m, "analysis_seed")? == 2026100302,
        "wrong analysis root",
    )?;
    for k in ["source_draws", "permutation_draws", "contrast_draws"] {
        require(
            uint(&m, k)? == 100000,
            "registered draw contract cannot change",
        )?;
    }
    let contract = text(&m, "method_contract_json")?;
    require(
        strict_json(contract.as_bytes())? == m["method_contract"],
        "method contract JSON mismatch",
    )?;
    require(
        digest(contract.as_bytes()) == text(&m, "method_contract_sha256")?,
        "method contract digest mismatch",
    )?;
    let template = strict_json(include_bytes!("../../democratic_peace/methods.json"))?;
    require(
        m["method_contract"] == template,
        "method contract differs from production template",
    )?;
    let table = read(&root.join(TABLE))?;
    require(
        text(&m, "source_table_sha256")? == digest(&table),
        "source table changed",
    )?;
    let table_value = strict_json(&table)?;
    require(
        text(&table_value, "paper_sha256")?
            == "069ad22da938727b2020c5d2c3a52aa383687c74085f5bc0af8cea74817b5215",
        "wrong primary source PDF identity",
    )?;
    require(
        array(&table_value, "slots")?.len() == 105
            && array(&table_value, "structural_exclusions")?.len() == 3,
        "source target family changed",
    )?;
    if registered && (will_run || provenance == "frozen") {
        require(
            table_value["extraction"]["classification"] != json!("synthetic_fixture"),
            "registered source cannot be synthetic",
        )?;
        let figures = array(&table_value, "figures")?;
        require(
            figures.len() == 3
                && figures.iter().all(|f| {
                    f["independent_visual_check"]["second_reader_status"] == json!("reviewed")
                }),
            "registered source lacks independent visual review",
        )?;
    }
    let inventory_sha = validate_inventory(&m, root, will_run || provenance == "frozen")?;
    let precision = m["precision_registered"]
        .as_bool()
        .ok_or("precision_registered must be boolean")?;
    require(
        text(&m, "runtime_gate")?
            == if precision {
                "full_precision"
            } else {
                "original_only"
            },
        "runtime gate disagreement",
    )?;
    let entries = array(&m, "arms")?;
    if registered {
        require(
            entries.len() == if precision { 216 } else { 108 },
            "registered population cannot be incomplete",
        )?;
    } else {
        require(
            !entries.is_empty() && entries.len() <= if mode == "fixture" { 4 } else { 8 },
            "unregistered population exceeds bounded gate",
        )?;
    }
    let mobiles = [0.15, 0.5, 0.85];
    let densities = [
        0.0, 0.05, 0.1, 0.15, 0.2, 0.25, 0.3, 0.4, 0.5, 0.6, 0.7, 1.0,
    ];
    let mechanisms = ["tagging", "alliances", "collective_security"];
    let mut arms = Vec::new();
    let mut ids = BTreeSet::new();
    let mut seeds = BTreeSet::new();
    for (i, v) in entries.iter().enumerate() {
        exact(
            v,
            &[
                "id",
                "family",
                "index",
                "canonical_index",
                "preset",
                "mobile_index",
                "mechanism_index",
                "density_index",
                "config_overrides",
                "sessions",
                "first_seed",
            ],
        )?;
        require(uint(v, "index")? == i as u64, "arm index disagreement")?;
        let canonical = uint(v, "canonical_index")?;
        let id = text(v, "id")?.to_string();
        require(
            !id.is_empty() && ids.insert(id.clone()),
            "empty/duplicate arm ID",
        )?;
        let family = text(v, "family")?.to_string();
        require(
            text(v, "preset")? == "printed_2001"
                || (!registered && text(v, "preset")? == "prose_probability"),
            "unregistered reading in primary population",
        )?;
        let mut config =
            serde_json::to_value(DemocraticPeaceConfig::default()).map_err(|e| e.to_string())?;
        if text(v, "preset")? == "prose_probability" {
            config["probability_direction"] = json!("prose_increasing");
        }
        for (k, value) in object(&v["config_overrides"])? {
            require(object(&config)?.contains_key(k), "unknown config override")?;
            config[k] = value.clone();
        }
        let c: DemocraticPeaceConfig =
            serde_json::from_value(config.clone()).map_err(|e| format!("invalid config: {e}"))?;
        c.validate().map_err(|e| format!("{e:?}"))?;
        let sessions = uint(v, "sessions")?;
        let first = uint(v, "first_seed")?;
        if registered {
            let expected_canonical = (i % 108) as u64;
            require(canonical == expected_canonical, "canonical index drift")?;
            let mi = (canonical / 36) as usize;
            let mech = ((canonical / 12) % 3) as usize;
            let di = (canonical % 12) as usize;
            require(
                uint(v, "mobile_index")? == mi as u64
                    && uint(v, "mechanism_index")? == mech as u64
                    && uint(v, "density_index")? == di as u64,
                "factor index disagreement",
            )?;
            let expected_family = if i < 108 { "original" } else { "precision" };
            require(family == expected_family, "family order drift")?;
            require(
                sessions == if i < 108 { 30 } else { 100 },
                "registered sample size drift",
            )?;
            require(
                first
                    == if i < 108 {
                        380000001 + canonical * 10000
                    } else {
                        390000001 + canonical * 10000
                    },
                "registered seeds drift",
            )?;
            exact(
                &v["config_overrides"],
                &[
                    "mobile_share",
                    "mechanism",
                    "initial_democratic_share",
                    "periods_per_tick",
                ],
            )?;
            require(
                config["mobile_share"] == json!(mobiles[mi])
                    && config["mechanism"] == json!(mechanisms[mech])
                    && config["initial_democratic_share"] == json!(densities[di])
                    && config["periods_per_tick"] == json!(100),
                "registered factor values drift",
            )?;
        } else {
            require(
                sessions > 0 && sessions <= if mode == "fixture" { 2 } else { 1 },
                "unregistered sessions exceed bounded gate",
            )?;
            require(
                first >= 400000000,
                "unregistered seed collides with registered families",
            )?;
            let width = config["width"].as_u64().ok_or("missing width")?;
            let height = config["height"].as_u64().ok_or("missing height")?;
            let horizon = config["horizon_periods"]
                .as_u64()
                .ok_or("missing horizon")?;
            require(
                if mode == "fixture" {
                    width <= 4 && height <= 4 && horizon <= 20
                } else {
                    width == 15 && height == 15 && horizon == 1000
                },
                "unregistered dimensions/horizon exceed bounded gate",
            )?;
        }
        for r in 0..sessions {
            let seed = first.checked_add(r).ok_or("seed overflow")?;
            require(seeds.insert(seed), "seed families overlap")?;
        }
        arms.push(Arm {
            id,
            index: i as u64,
            canonical,
            family,
            sessions,
            first_seed: first,
            config: c,
        });
    }
    let jobs = array(&m, "analysis_jobs")?;
    require(
        jobs.len() == 129,
        "analysis job family must reserve129 jobs",
    )?;
    let mut names = BTreeSet::new();
    for (i, j) in jobs.iter().enumerate() {
        exact(
            j,
            &["index", "id", "root_entropy", "spawn_key", "state_u32"],
        )?;
        require(
            uint(j, "index")? == i as u64
                && uint(j, "root_entropy")? == 2026100302
                && j["spawn_key"] == json!([i]),
            "analysis child stream drift",
        )?;
        require(
            names.insert(text(j, "id")?.to_string()),
            "duplicate analysis job",
        )?;
        require(
            array(j, "state_u32")?.len() == 4
                && array(j, "state_u32")?
                    .iter()
                    .all(|v| v.as_u64().is_some_and(|x| x <= u32::MAX as u64)),
            "invalid analysis state",
        )?;
    }
    let resolved = Value::Array(
        arms.iter()
            .map(|a| json!({"id":a.id,"config":a.config}))
            .collect(),
    );
    let payload = serde_json::to_vec(&resolved).map_err(|e| e.to_string())?;
    let payload_sha = digest(&payload);
    let binary_sha = digest(&read(&std::env::current_exe().map_err(|e| e.to_string())?)?);
    Ok(Prepared {
        manifest: m,
        arms,
        payload,
        payload_sha,
        manifest_sha: digest(&bytes),
        binary_sha,
        inventory_sha,
    })
}
fn receipt(
    path: Option<&Path>,
    p: &Prepared,
    required: bool,
    root: &Path,
) -> Result<Option<String>, String> {
    let Some(path) = path else {
        require(!required, "build receipt required before execution")?;
        return Ok(None);
    };
    let bytes = read(path)?;
    let r = strict_json(&bytes)?;
    exact(
        &r,
        &[
            "schema_version",
            "model",
            "manifest_sha256",
            "source_inventory_sha256",
            "prebuild_inventory_sha256",
            "postbuild_inventory_sha256",
            "binary_sha256",
            "build_command",
            "cwd",
            "target",
            "rustc_version",
            "cargo_version",
            "lockfile_hashes",
            "features",
            "build_flags",
        ],
    )?;
    require(
        uint(&r, "schema_version")? == 1 && text(&r, "model")? == "democratic_peace",
        "invalid build receipt schema",
    )?;
    require(
        text(&r, "manifest_sha256")? == p.manifest_sha
            && text(&r, "binary_sha256")? == p.binary_sha,
        "receipt manifest/binary mismatch",
    )?;
    for k in [
        "source_inventory_sha256",
        "prebuild_inventory_sha256",
        "postbuild_inventory_sha256",
    ] {
        require(
            r[k] == json!(p.inventory_sha),
            "receipt source inventory mismatch",
        )?;
    }
    for k in ["build_command", "features", "build_flags"] {
        require(
            array(&r, k)?.iter().all(Value::is_string),
            "invalid build invocation",
        )?;
    }
    for k in ["cwd", "target", "rustc_version", "cargo_version"] {
        require(!text(&r, k)?.is_empty(), "incomplete build receipt")?;
    }
    validate_build_invocation(&r, root)?;
    Ok(Some(digest(&bytes)))
}
fn validate_attempt_context(a: &Value, science: &Value) -> Result<(), String> {
    let status = text(a, "status")?;
    let errors = array(a, "construction_errors")?;
    for e in errors {
        exact(e, &["field", "message"])?;
        require(
            !text(e, "field")?.is_empty() && !text(e, "message")?.is_empty(),
            "empty construction error",
        )?;
    }
    for key in ["panic_context", "recorder_error"] {
        require(
            a[key].is_null() || a[key].as_str().is_some_and(|s| !s.is_empty()),
            "invalid failure context",
        )?;
    }
    require(
        status != "construction_error" || !errors.is_empty(),
        "construction error lacks context",
    )?;
    require(
        !status.ends_with("_panic") || a["panic_context"].is_string(),
        "panic lacks context",
    )?;
    require(
        status != "incomplete" || a["recorder_error"].is_string(),
        "incomplete lacks context",
    )?;
    if matches!(status, "completed" | "invalid") {
        require(
            errors.is_empty() && a["panic_context"].is_null() && a["recorder_error"].is_null(),
            "terminal engine attempt contains recorder errors",
        )?;
        require(science.is_object(), "terminal engine attempt lacks science")?;
    }
    if matches!(status, "construction_error" | "construction_panic") {
        require(science.is_null(), "construction failure contains science")?;
    }
    Ok(())
}
fn validate_build_invocation(r: &Value, root: &Path) -> Result<(), String> {
    require(
        !array(r, "build_command")?.is_empty(),
        "empty build command",
    )?;
    let locks = object(&r["lockfile_hashes"])?;
    require(
        locks.contains_key("Cargo.lock") && locks.contains_key("survey/Cargo.lock"),
        "receipt omits required Cargo locks",
    )?;
    for (name, value) in locks {
        let expected = value.as_str().ok_or("invalid lockfile digest")?;
        require(is_hash(expected), "invalid lockfile digest")?;
        require(
            digest(&read(&source_path(root, Path::new(name))?)?) == expected,
            "receipt lockfile SHA256 mismatch",
        )?;
    }
    Ok(())
}
fn canonical_destination(path: &Path) -> Result<PathBuf, String> {
    safe_path(path)?;
    if path.exists() {
        return fs::canonicalize(path).map_err(|e| e.to_string());
    }
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()
            .map_err(|e| e.to_string())?
            .join(path)
    };
    let mut ancestor = absolute.as_path();
    let mut suffix = Vec::new();
    while !ancestor.exists() {
        suffix.push(ancestor.file_name().ok_or("unresolvable output ancestor")?);
        ancestor = ancestor.parent().ok_or("unresolvable output ancestor")?;
    }
    require(ancestor.is_dir(), "output ancestor is not a directory")?;
    let mut result = fs::canonicalize(ancestor).map_err(|e| e.to_string())?;
    for part in suffix.into_iter().rev() {
        result.push(part);
    }
    Ok(result)
}
fn same_file(left: &Path, right: &Path) -> bool {
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        if let (Ok(a), Ok(b)) = (fs::metadata(left), fs::metadata(right)) {
            return a.dev() == b.dev() && a.ino() == b.ino();
        }
    }
    false
}
fn validate_destinations(inputs: &[PathBuf], outputs: &[PathBuf]) -> Result<(), String> {
    let protected = inputs
        .iter()
        .map(|p| canonical_destination(p))
        .collect::<Result<Vec<_>, _>>()?;
    let mut seen: Vec<PathBuf> = Vec::new();
    for path in outputs {
        require(!path.is_dir(), "output is a directory")?;
        let path = canonical_destination(path)?;
        require(
            !protected
                .iter()
                .chain(seen.iter())
                .any(|p| p == &path || same_file(p, &path)),
            "output aliases input or another output",
        )?;
        seen.push(path);
    }
    Ok(())
}
fn identity(p: &Prepared, receipt: &Option<String>) -> Value {
    json!({"manifest_sha256":p.manifest_sha,"binary_sha256":p.binary_sha,"source_inventory_sha256":p.inventory_sha,"build_receipt_sha256":receipt,"resolved_configs_sha256":p.payload_sha})
}
fn attempt(status: &str, errors: Value, panic: Option<String>, error: Option<String>) -> Value {
    json!({"status":status,"construction_errors":errors,"panic_context":panic,"recorder_error":error})
}
fn panic_text(p: Box<dyn std::any::Any + Send>) -> String {
    p.downcast_ref::<String>()
        .cloned()
        .or_else(|| p.downcast_ref::<&str>().map(|v| v.to_string()))
        .unwrap_or_else(|| "non-string panic".into())
}
fn record(p: &Prepared, a: &Arm, r: u64, receipt: &Option<String>) -> Result<Value, String> {
    let seed = a.first_seed + r;
    let constructed = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        DemocraticPeaceWorld::new(a.config.clone(), seed)
    }));
    let (attempt, outcome) = match constructed {
        Err(error) => (
            attempt(
                "construction_panic",
                json!([]),
                Some(panic_text(error)),
                None,
            ),
            Value::Null,
        ),
        Ok(Err(errors)) => (
            attempt(
                "construction_error",
                serde_json::to_value(errors).map_err(|e| e.to_string())?,
                None,
                None,
            ),
            Value::Null,
        ),
        Ok(Ok(mut world)) => {
            let ticks = a
                .config
                .horizon()
                .div_ceil(a.config.periods_per_tick as u64);
            let result =
                std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| world.run(ticks as u32)));
            let science =
                std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| world.science_json()));
            let (value, error) = match science {
                Ok(s) => (strict_json(s.as_bytes())?, None),
                Err(e) => (Value::Null, Some(panic_text(e))),
            };
            let attempt = match result {
                Err(e) => attempt(
                    "implementation_panic",
                    json!([]),
                    Some(panic_text(e)),
                    error,
                ),
                Ok(()) => {
                    let status = if error.is_some() {
                        "implementation_panic"
                    } else if value["outcome"]["valid"] == json!(true)
                        && value["completed_periods"] == json!(a.config.horizon())
                    {
                        "completed"
                    } else if value["outcome"]["valid"] == json!(false) {
                        "invalid"
                    } else {
                        "incomplete"
                    };
                    attempt(
                        status,
                        json!([]),
                        error.clone(),
                        if status == "incomplete" {
                            Some(
                                "engine returned before source horizon without invalid outcome"
                                    .into(),
                            )
                        } else {
                            None
                        },
                    )
                }
            };
            (attempt, value)
        }
    };
    let mut row = identity(p, receipt);
    let o = row.as_object_mut().ok_or("record identity invalid")?;
    for (k, v) in [
        ("schema_version", json!(1)),
        ("model", json!("democratic_peace")),
        ("arm", json!(a.id)),
        ("seed", json!(seed)),
        ("arm_index", json!(a.index)),
        ("canonical_index", json!(a.canonical)),
        ("repeat_index", json!(r)),
        ("family", json!(a.family)),
        ("execution_mode", p.manifest["execution_mode"].clone()),
        (
            "config",
            serde_json::to_value(&a.config).map_err(|e| e.to_string())?,
        ),
        ("attempt", attempt),
        ("outcome", outcome),
    ] {
        o.insert(k.into(), v);
    }
    Ok(row)
}
fn validate_existing(
    row: &Value,
    p: &Prepared,
    receipt: &Option<String>,
) -> Result<(String, u64), String> {
    exact(
        row,
        &[
            "schema_version",
            "model",
            "arm",
            "seed",
            "arm_index",
            "canonical_index",
            "repeat_index",
            "family",
            "execution_mode",
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
        uint(row, "schema_version")? == 1 && text(row, "model")? == "democratic_peace",
        "resume schema/model mismatch",
    )?;
    let binding = identity(p, receipt);
    for (k, v) in object(&binding)? {
        require(row[k] == *v, "resume evidence identity mismatch")?;
    }
    let id = text(row, "arm")?;
    let a = p
        .arms
        .iter()
        .find(|a| a.id == id)
        .ok_or("unexpected resume arm")?;
    let r = uint(row, "repeat_index")?;
    require(
        r < a.sessions
            && uint(row, "seed")? == a.first_seed + r
            && uint(row, "arm_index")? == a.index
            && uint(row, "canonical_index")? == a.canonical,
        "resume attempted key mismatch",
    )?;
    require(
        row["config"] == serde_json::to_value(&a.config).map_err(|e| e.to_string())?
            && row["family"] == json!(a.family)
            && row["execution_mode"] == p.manifest["execution_mode"],
        "resume config/family mismatch",
    )?;
    exact(
        &row["attempt"],
        &[
            "status",
            "construction_errors",
            "panic_context",
            "recorder_error",
        ],
    )?;
    let status = text(&row["attempt"], "status")?;
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
        "unknown attempted outcome",
    )?;
    validate_attempt_context(&row["attempt"], &row["outcome"])?;
    if status == "completed" || status == "invalid" {
        let science = &row["outcome"];
        require(
            science["model"] == json!("democratic_peace")
                && science["config"] == row["config"]
                && science["seed"] == row["seed"],
            "resume engine evidence mismatch",
        )?;
        let completed = uint(science, "completed_periods")?;
        let attempted = uint(science, "attempted_period")?;
        require(
            completed <= a.config.horizon()
                && attempted >= completed
                && attempted <= a.config.horizon(),
            "resume source clock mismatch",
        )?;
        require(
            if status == "completed" {
                science["outcome"]["valid"] == json!(true)
                    && completed == a.config.horizon()
                    && science["outcome"]["final_metrics"].is_object()
            } else {
                science["outcome"]["valid"] == json!(false)
                    && science["outcome"]["final_metrics"].is_null()
            },
            "resume completion/invalidity mismatch",
        )?;
    }
    Ok((id.into(), uint(row, "seed")?))
}
fn run() -> Result<(), String> {
    let mut opts = BTreeMap::new();
    let mut validate = false;
    let mut args = std::env::args().skip(1);
    while let Some(k) = args.next() {
        if k == "--validate-only" {
            require(!validate, "duplicate validate flag")?;
            validate = true;
        } else {
            require(
                ["--manifest", "--resolved", "--out", "--receipt", "--repo"].contains(&k.as_str()),
                "unknown command option",
            )?;
            let v = args
                .next()
                .ok_or_else(|| format!("missing value for {k}"))?;
            require(opts.insert(k, v).is_none(), "duplicate command option")?;
        }
    }
    let manifest = PathBuf::from(opts.get("--manifest").ok_or("--manifest required")?);
    let resolved = PathBuf::from(opts.get("--resolved").ok_or("--resolved required")?);
    let root = opts
        .get("--repo")
        .map(PathBuf::from)
        .unwrap_or(std::env::current_dir().map_err(|e| e.to_string())?);
    safe_path(&root)?;
    let root = fs::canonicalize(root).map_err(|e| e.to_string())?;
    let p = prepare(&manifest, &root, !validate)?;
    let receipt_path = opts.get("--receipt").map(PathBuf::from);
    let receipt = receipt(receipt_path.as_deref(), &p, !validate, &root)?;
    let mut inputs = vec![
        manifest.clone(),
        std::env::current_exe().map_err(|e| e.to_string())?,
    ];
    if let Some(path) = &receipt_path {
        inputs.push(path.clone());
    }
    if let Some(entries) = p.manifest["source_inventory"].as_array() {
        for entry in entries {
            inputs.push(source_path(&root, Path::new(text(entry, "path")?))?);
        }
    } else {
        for name in [
            SPEC,
            TABLE,
            "Cargo.lock",
            "survey/Cargo.lock",
            "survey/src/bin/democratic_peace.rs",
        ] {
            inputs.push(root.join(name));
        }
        let mut sources = BTreeSet::new();
        walk_files(
            &root,
            &root.join("crates/sugarscape-core/src"),
            &mut sources,
        )?;
        walk_files(&root, &root.join("survey/democratic_peace"), &mut sources)?;
        inputs.extend(sources.into_iter().map(|name| root.join(name)));
    }
    let mut outputs = vec![resolved.clone()];
    if let Some(path) = opts.get("--out") {
        outputs.push(PathBuf::from(path));
    }
    validate_destinations(&inputs, &outputs)?;
    let mut export = identity(&p, &receipt);
    let o = export.as_object_mut().ok_or("export identity invalid")?;
    o.insert("schema_version".into(), json!(1));
    o.insert("model".into(), json!("democratic_peace"));
    o.insert("arms".into(), strict_json(&p.payload)?);
    o.insert(
        "resolved_configs_json".into(),
        json!(String::from_utf8(p.payload.clone()).map_err(|e| e.to_string())?),
    );
    atomic_write(
        &resolved,
        &serde_json::to_vec(&export).map_err(|e| e.to_string())?,
    )?;
    if validate {
        println!(
            "validated {} arms/{} keys without world construction; provenance={}",
            p.arms.len(),
            p.arms.iter().map(|a| a.sessions).sum::<u64>(),
            text(&p.manifest, "provenance_status")?
        );
        return Ok(());
    }
    let out = PathBuf::from(opts.get("--out").ok_or("--out required for execution")?);
    safe_path(&out)?;
    let mut seen = BTreeSet::new();
    if out.exists() {
        let bytes = read(&out)?;
        require(
            bytes.is_empty() || bytes.last() == Some(&b'\n'),
            "truncated final JSONL record",
        )?;
        for line in BufReader::new(bytes.as_slice()).lines() {
            let line = line.map_err(|e| e.to_string())?;
            require(!line.trim().is_empty(), "blank JSONL record")?;
            let key = validate_existing(&strict_json(line.as_bytes())?, &p, &receipt)?;
            require(seen.insert(key), "duplicate attempted key in resume")?;
        }
    }
    let mut writer = OpenOptions::new()
        .create(true)
        .append(true)
        .open(out)
        .map_err(|e| e.to_string())?;
    for a in &p.arms {
        for r in 0..a.sessions {
            let key = (a.id.clone(), a.first_seed + r);
            if seen.contains(&key) {
                continue;
            }
            let row = record(&p, a, r, &receipt)?;
            validate_existing(&row, &p, &receipt)?;
            serde_json::to_writer(&mut writer, &row).map_err(|e| e.to_string())?;
            writer.write_all(b"\n").map_err(|e| e.to_string())?;
            writer.flush().map_err(|e| e.to_string())?;
            writer.sync_data().map_err(|e| e.to_string())?;
            seen.insert(key);
        }
    }
    require(
        seen.len() as u64 == p.arms.iter().map(|a| a.sessions).sum::<u64>(),
        "attempted census incomplete",
    )?;
    println!("recorded {} attempted histories", seen.len());
    Ok(())
}
fn main() {
    if let Err(error) = run() {
        eprintln!("democratic-peace recorder: {error}");
        std::process::exit(1)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn missing_panic_context_and_completed_errors_are_rejected() {
        assert!(validate_attempt_context(
            &attempt("construction_panic", json!([]), None, None),
            &Value::Null
        )
        .is_err());
        assert!(validate_attempt_context(
            &attempt("completed", json!([]), Some("forged panic".into()), None),
            &json!({"outcome":{"valid":true}})
        )
        .is_err());
        assert!(validate_attempt_context(
            &attempt("construction_error", json!([]), None, None),
            &Value::Null
        )
        .is_err());
    }
    #[test]
    fn explicit_construction_failure_and_partial_panic_are_retained() {
        assert!(validate_attempt_context(
            &attempt(
                "construction_error",
                json!([{"field":"setup","message":"failure"}]),
                None,
                None
            ),
            &Value::Null
        )
        .is_ok());
        assert!(validate_attempt_context(
            &attempt(
                "implementation_panic",
                json!([]),
                Some("run failed".into()),
                None
            ),
            &json!({"outcome":null})
        )
        .is_ok());
    }
    #[test]
    fn forged_required_build_locks_and_empty_command_are_rejected() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
        let locks = json!({"Cargo.lock":digest(&fs::read(root.join("Cargo.lock")).unwrap()), "survey/Cargo.lock":digest(&fs::read(root.join("survey/Cargo.lock")).unwrap())});
        assert!(validate_build_invocation(
            &json!({"build_command":[],"lockfile_hashes":locks}),
            root
        )
        .is_err());
        assert!(validate_build_invocation(&json!({"build_command":["cargo","build"],"lockfile_hashes":{"Cargo.lock":"wrong","survey/Cargo.lock":"wrong"}}), root).is_err());
        assert!(validate_build_invocation(
            &json!({"build_command":["cargo","build"],"lockfile_hashes":locks}),
            root
        )
        .is_ok());
    }
    #[test]
    fn output_aliases_reject_without_creating_files() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
        let input = root.join("survey/Cargo.toml");
        let missing = std::env::temp_dir().join(format!("dp-missing-{}", std::process::id()));
        assert!(
            validate_destinations(std::slice::from_ref(&input), std::slice::from_ref(&input))
                .is_err()
        );
        assert!(validate_destinations(&[], &[missing.clone(), missing]).is_err());
    }
    #[test]
    fn duplicate_nested_json_fields_are_rejected() {
        assert!(strict_json(br#"{"outer":{"value":1,"value":2}}"#)
            .unwrap_err()
            .contains("duplicate JSON field"));
    }
    #[test]
    fn valid_zero_and_null_json_remain_distinct() {
        let v = strict_json(br#"{"zero":0,"undefined":null}"#).unwrap();
        assert_eq!(v["zero"], json!(0));
        assert!(v["undefined"].is_null());
    }
    #[test]
    fn nonfinite_json_and_trailing_garbage_are_rejected() {
        assert!(strict_json(b"{\"n\":1e999}").is_err());
        assert!(strict_json(b"{} {}").is_err());
    }
    #[test]
    fn source_inventory_paths_cannot_escape_root() {
        let d = std::env::temp_dir();
        assert!(safe_path(&d.join("../escape")).is_err());
    }
}
