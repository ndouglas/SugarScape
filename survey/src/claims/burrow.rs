//! Reviewed candidate identity; default routes never execute episodes.
use serde::{Deserialize, Serialize};
use sugarscape_core::burrow::{LabConfig, RunOptions, World};
const EXPECTED_SHA256: &str = "3ef020036fa96ebc5a440480dccfcab64fd8991dfc2ff2dc4129b05466acb890";
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Manifest {
    pub(super) schema: String,
    pub(super) status: String,
    pub(super) execution_authorized: bool,
    pub(super) protocol: String,
    pub(super) engine_baseline: String,
    pub(super) scientific_seeds: Vec<String>,
    pub(super) construction_seeds: Vec<String>,
    pub(super) expected_conditions: usize,
    pub(super) expected_scientific_episodes: usize,
    pub(super) expected_construction_conditions: usize,
    pub(super) expected_construction_episodes: usize,
    pub(super) primary_outcomes: Vec<String>,
    pub(super) primary_contrasts: Vec<Contrast>,
    pub(super) negative_control: NegativeControl,
    pub(super) conditions: Vec<Condition>,
    pub(super) construction_conditions: Vec<Condition>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Contrast {
    pub(super) id: String,
    pub(super) plus: String,
    pub(super) minus: String,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct NegativeControl {
    pub(super) plus: String,
    pub(super) minus: String,
    pub(super) comparison: String,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Condition {
    pub(super) id: String,
    pub(super) panel: String,
    #[serde(deserialize_with = "explicit_config")]
    pub(super) config: LabConfig,
    #[serde(deserialize_with = "strict_options")]
    pub(super) options: RunOptions,
}

fn explicit_config<'de, D: serde::Deserializer<'de>>(d: D) -> Result<LabConfig, D::Error> {
    let value = serde_json::Value::deserialize(d)?;
    for field in [
        "fixture",
        "transport",
        "cue",
        "freshness_window",
        "relay_distance",
        "response_weight",
        "minimum_recent_units",
    ] {
        if value.get(field).is_none() {
            return Err(serde::de::Error::custom(format!(
                "missing explicit lab field {field}"
            )));
        }
    }
    serde_json::from_value(value).map_err(serde::de::Error::custom)
}
fn strict_options<'de, D: serde::Deserializer<'de>>(d: D) -> Result<RunOptions, D::Error> {
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Options {
        ticks: u32,
        sample_every: u32,
    }
    let v = Options::deserialize(d)?;
    Ok(RunOptions {
        ticks: v.ticks,
        sample_every: v.sample_every,
    })
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum Panel {
    Scientific,
    Construction,
}
#[derive(Clone, Debug, PartialEq, Eq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct RunKey {
    pub(super) condition: String,
    pub(super) seed: String,
}
pub(super) fn manifest_bytes() -> &'static [u8] {
    include_bytes!("../../../docs/superpowers/specs/2026-10-04-burrow-1-draft-manifest.json")
}
pub(super) fn manifest_sha256() -> String {
    use sha2::{Digest, Sha256};
    format!("{:x}", Sha256::digest(manifest_bytes()))
}
fn candidate() -> Result<Manifest, String> {
    if manifest_sha256() != EXPECTED_SHA256 {
        return Err("burrow candidate byte identity changed; reviewed amendment required".into());
    }
    serde_json::from_slice(manifest_bytes()).map_err(|e| format!("burrow manifest: {e}"))
}
pub(super) fn manifest() -> Result<Manifest, String> {
    let m = candidate()?;
    validate_manifest(&m)?;
    Ok(m)
}
pub(super) fn validate_manifest(m: &Manifest) -> Result<(), String> {
    let expected = candidate()?;
    if m != &expected {
        return Err("burrow manifest differs from reviewed candidate identity (fields, conditions, seeds, counts or contrasts)".into());
    }
    for (panel, count) in [
        (Panel::Scientific, m.expected_scientific_episodes),
        (Panel::Construction, m.expected_construction_episodes),
    ] {
        if expected_keys(m, panel).len() != count {
            return Err(format!("burrow {panel:?} declared episode count mismatch"));
        }
    }
    for c in m.conditions.iter().chain(&m.construction_conditions) {
        World::new(c.config.clone(), 7)
            .map_err(|e| format!("burrow condition {} setup: {e:?}", c.id))?;
    }
    Ok(())
}
pub(super) fn expected_keys(m: &Manifest, panel: Panel) -> Vec<RunKey> {
    let (conditions, seeds) = match panel {
        Panel::Scientific => (&m.conditions, &m.scientific_seeds),
        Panel::Construction => (&m.construction_conditions, &m.construction_seeds),
    };
    let mut seeds = seeds.clone();
    seeds.sort_by_key(|s| s.parse::<u64>().ok());
    conditions
        .iter()
        .flat_map(|c| {
            seeds.iter().map(move |s| RunKey {
                condition: c.id.clone(),
                seed: s.clone(),
            })
        })
        .collect()
}
pub(crate) fn cli(args: &[String]) -> Result<(), String> {
    const UNAVAILABLE: &str = "burrow route unavailable: archive execution and saved analysis are not installed; scientific execution requires separate approval";
    match args {
        [] => {}
        [flag] if flag == "--manifest" => {}
        [flag] if flag == "--help" => {
            println!(concat!(
                "survey --burrow [--manifest|--help]\n",
                "Default prints the reviewed candidate only. Scientific execution and ",
                "executable registration require separate approval. --run archive execution ",
                "and --analyze saved analysis are unavailable at this task boundary."
            ));
            return Ok(());
        }
        [flag] if flag == "--run" || flag == "--analyze" => return Err(UNAVAILABLE.into()),
        [flag, index] if flag == "--analyze" && !index.starts_with("--") => {
            return Err(UNAVAILABLE.into())
        }
        _ => return Err("burrow: unknown, duplicate or conflicting flags; use --help".into()),
    }
    println!(
        "{}",
        serde_json::to_string_pretty(&manifest()?)
            .map_err(|e| format!("burrow manifest serialization: {e}"))?
    );
    Ok(())
}
