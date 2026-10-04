//! Deterministic saved-only reductions, paired estimates and censored histories.
use super::{
    burrow::{self, Condition, Manifest, Panel, RunKey},
    burrow_archive::{self, Archive},
};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    io::Write,
    path::Path,
};
use sugarscape_core::burrow::{
    Action, Cue, DigDistance, Episode, Fixture, LoadedTravel, Outcome, Pile, Side, Snapshot,
    SpatialWork, Storage, Transport, WorkerWork,
};
#[derive(Serialize, PartialEq, Debug)]
pub(super) struct Analysis {
    pub(super) schema: String,
    pub(super) code_revision: String,
    pub(super) protocol_revision: String,
    pub(super) manifest_sha256: String,
    pub(super) panel: Panel,
    pub(super) rows: Vec<SeedRow>,
    pub(super) means: Vec<ConditionMean>,
    pub(super) contrasts: Vec<ContrastRow>,
    pub(super) controls: Vec<ControlRow>,
    pub(super) choices: Vec<ChoiceRow>,
    pub(super) duplicates: Vec<DuplicateRow>,
    pub(super) limitations: Vec<String>,
}
#[derive(Serialize, PartialEq, Debug)]
pub(super) struct SeedRow {
    pub(super) key: RunKey,
    pub(super) completed_ticks: u32,
    pub(super) summary: Snapshot,
    pub(super) excavation_rate: Option<f64>,
    pub(super) disposal_rate: Option<f64>,
    pub(super) worker_work: Vec<WorkerWork>,
    pub(super) spatial_work: Vec<SpatialWork>,
    pub(super) dig_distances: Vec<DigDistance>,
    pub(super) travel: LoadedTravel,
    pub(super) storage: Storage,
    pub(super) materials: Vec<MaterialRow>,
    pub(super) delivered_only_mean_ticks: Option<f64>,
    pub(super) delivered_only_mean_opportunities: Option<f64>,
    pub(super) delivered: u64,
    pub(super) censored_carried: u64,
    pub(super) censored_loose: u64,
}
#[derive(Serialize, PartialEq, Debug)]
pub(super) struct MaterialRow {
    pub(super) material: u64,
    pub(super) born: u64,
    pub(super) disposed_at: Option<u64>,
    pub(super) fate: MaterialFate,
    pub(super) tick_age: u64,
    pub(super) opportunity_age: Option<u64>,
    pub(super) waiting_ticks: u64,
    pub(super) carriers: Vec<u32>,
    pub(super) carried_moves: u64,
}
#[derive(Serialize, PartialEq, Debug)]
#[serde(rename_all = "snake_case")]
pub(super) enum MaterialFate {
    Delivered,
    CensoredCarried,
    CensoredLoose,
}
#[derive(Serialize, PartialEq, Debug)]
pub(super) struct ConditionMean {
    pub(super) condition: String,
    pub(super) n: usize,
    pub(super) excavation: Option<f64>,
    pub(super) disposal: Option<f64>,
}
#[derive(Serialize, PartialEq, Debug)]
pub(super) struct ContrastRow {
    pub(super) id: String,
    pub(super) stratum: String,
    pub(super) outcome: String,
    pub(super) primary: bool,
    pub(super) n: usize,
    pub(super) mean: f64,
    pub(super) ci95: Option<(f64, f64)>,
    pub(super) positive: usize,
    pub(super) zero: usize,
    pub(super) negative: usize,
    pub(super) differences: Vec<(String, f64)>,
}
#[derive(Serialize, PartialEq, Debug)]
pub(super) struct ControlRow {
    pub(super) id: String,
    pub(super) seeds_checked: usize,
    pub(super) identical: bool,
}
#[derive(Serialize, PartialEq, Debug)]
pub(super) struct ChoiceRow {
    pub(super) condition: String,
    pub(super) left: u64,
    pub(super) right: u64,
    pub(super) pile_side: String,
    pub(super) expected_pile_probability: f64,
}
#[derive(Serialize, PartialEq, Debug)]
pub(super) struct DuplicateRow {
    pub(super) projection_sha256: String,
    pub(super) keys: Vec<RunKey>,
}

pub(super) fn physical_projection(record: &Episode) -> String {
    serde_json::to_string(&(&record.events, &record.choices)).expect("integer physical projection")
}
fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn average(values: impl Iterator<Item = f64>) -> Option<f64> {
    let values: Vec<_> = values.collect();
    (!values.is_empty()).then(|| values.iter().sum::<f64>() / values.len() as f64)
}
fn material_rows(record: &Episode) -> Result<Vec<MaterialRow>, String> {
    let mut fates: BTreeMap<u64, MaterialFate> = (0..record.setup.spoil.len() as u64)
        .map(|id| (id, MaterialFate::CensoredLoose))
        .collect();
    let mut births = BTreeMap::new();
    let mut disposals = BTreeMap::new();
    for (i, event) in record.events.iter().enumerate() {
        if event.outcome != Outcome::Success {
            continue;
        }
        let Some(id) = event.material else { continue };
        match event.action {
            Action::Dig(_) => {
                births.insert(id, i as u64 + 1);
                fates.insert(id, MaterialFate::CensoredCarried);
            }
            Action::Pickup => {
                fates.insert(id, MaterialFate::CensoredCarried);
            }
            Action::Drop => {
                fates.insert(id, MaterialFate::CensoredLoose);
            }
            Action::Dispose => {
                disposals.insert(id, i as u64 + 1);
                fates.insert(id, MaterialFate::Delivered);
            }
            _ => {}
        }
    }
    record
        .deliveries
        .iter()
        .map(|d| {
            let fate = fates
                .remove(&d.material)
                .ok_or_else(|| format!("missing material fate {}", d.material))?;
            let end_tick = d.disposed_at.unwrap_or(record.final_summary.tick);
            let tick_age = end_tick
                .checked_sub(d.born)
                .ok_or("material clock precedes birth")?;
            let opportunity_age = if matches!(record.config.fixture, Fixture::Growing { .. }) {
                let birth = *births
                    .get(&d.material)
                    .ok_or("growing material lacks dig index")?;
                let end = disposals
                    .get(&d.material)
                    .copied()
                    .unwrap_or(record.events.len() as u64);
                Some(
                    end.checked_sub(birth)
                        .ok_or("material disposal precedes dig")?,
                )
            } else {
                None
            };
            Ok(MaterialRow {
                material: d.material,
                born: d.born,
                disposed_at: d.disposed_at,
                fate,
                tick_age,
                opportunity_age,
                waiting_ticks: d.waiting_ticks,
                carriers: d.carriers.clone(),
                carried_moves: d.carried_moves,
            })
        })
        .collect()
}
fn seed_row(key: RunKey, record: &Episode) -> Result<SeedRow, String> {
    let materials = material_rows(record)?;
    let count = |fate: MaterialFate| materials.iter().filter(|r| r.fate == fate).count() as u64;
    let delivered = count(MaterialFate::Delivered);
    let censored_carried = count(MaterialFate::CensoredCarried);
    let censored_loose = count(MaterialFate::CensoredLoose);
    let delivered_only_mean_ticks = average(
        materials
            .iter()
            .filter(|r| r.fate == MaterialFate::Delivered)
            .map(|r| r.tick_age as f64),
    );
    let delivered_only_mean_opportunities = average(
        materials
            .iter()
            .filter(|r| r.fate == MaterialFate::Delivered)
            .filter_map(|r| r.opportunity_age.map(|v| v as f64)),
    );
    let s = &record.final_summary;
    Ok(SeedRow {
        key,
        completed_ticks: record.completed_ticks,
        summary: s.clone(),
        excavation_rate: (s.opportunities > 0).then(|| s.digs as f64 / s.opportunities as f64),
        disposal_rate: (s.opportunities > 0).then(|| s.disposals as f64 / s.opportunities as f64),
        worker_work: record.worker_work.clone(),
        spatial_work: record.spatial_work.clone(),
        dig_distances: record.dig_distances.clone(),
        travel: record.travel.clone(),
        storage: record.storage.clone(),
        materials,
        delivered_only_mean_ticks,
        delivered_only_mean_opportunities,
        delivered,
        censored_carried,
        censored_loose,
    })
}
fn check_controls(
    conditions: &[Condition],
    seeds: &[String],
    records: &BTreeMap<RunKey, Episode>,
) -> Result<Vec<ControlRow>, String> {
    let mut controls = vec![];
    for blind in conditions.iter().filter(|c| {
        matches!(c.config.fixture, Fixture::Growing { .. }) && c.config.cue == Cue::Blind
    }) {
        let direct = blind.config.transport == Transport::Direct;
        let reduction = blind.config.response_weight == 1;
        if !direct && !reduction {
            continue;
        }
        let mut responsive_config = blind.config.clone();
        responsive_config.cue = Cue::Responsive;
        let responsive = conditions
            .iter()
            .find(|c| c.config == responsive_config)
            .ok_or_else(|| format!("missing responsive control for {}", blind.id))?;
        for seed in seeds {
            let left = records
                .get(&RunKey {
                    condition: blind.id.clone(),
                    seed: seed.clone(),
                })
                .ok_or("missing blind control record")?;
            let right = records
                .get(&RunKey {
                    condition: responsive.id.clone(),
                    seed: seed.clone(),
                })
                .ok_or("missing responsive control record")?;
            if physical_projection(left) != physical_projection(right) {
                return Err(format!(
                    "condition {} seed {}: physical control differs from {}",
                    responsive.id, seed, blind.id
                ));
            }
        }
        if direct {
            controls.push(ControlRow {
                id: format!("direct_cue.{}", blind.id),
                seeds_checked: seeds.len(),
                identical: true,
            });
        }
        if reduction {
            controls.push(ControlRow {
                id: format!("weight_one.{}", blind.id),
                seeds_checked: seeds.len(),
                identical: true,
            });
        }
    }
    Ok(controls)
}
type RateMaps = BTreeMap<String, BTreeMap<u64, f64>>;
fn contrast_row(
    id: &str,
    stratum: &str,
    outcome: &str,
    primary: bool,
    plus: &BTreeMap<u64, f64>,
    minus: &BTreeMap<u64, f64>,
) -> Result<ContrastRow, String> {
    let contrast = crate::stats::paired_summary(plus, minus)?;
    Ok(ContrastRow {
        id: id.into(),
        stratum: stratum.into(),
        outcome: outcome.into(),
        primary,
        n: contrast.n,
        mean: contrast.mean,
        ci95: contrast.ci95,
        positive: contrast.positive,
        zero: contrast.zero,
        negative: contrast.negative,
        differences: plus
            .iter()
            .map(|(s, v)| (s.to_string(), v - minus[s]))
            .collect(),
    })
}
fn estimate_contrasts(
    manifest: &Manifest,
    maps: &RateMaps,
    outcome: &str,
) -> Result<Vec<ContrastRow>, String> {
    let expected: BTreeSet<u64> = manifest
        .scientific_seeds
        .iter()
        .map(|s| s.parse::<u64>().map_err(|e| e.to_string()))
        .collect::<Result<_, _>>()?;
    for (condition, map) in maps {
        if map.keys().copied().collect::<BTreeSet<_>>() != expected {
            return Err(format!(
                "condition {condition}: paired seed set differs from manifest"
            ));
        }
    }
    let mut rows = vec![];
    for stratum in ["primary", "workforce.2", "workforce.4", "workforce.16"] {
        let lookup = |suffix: &str| {
            maps.get(&format!("{stratum}.{suffix}"))
                .ok_or_else(|| format!("missing rate map {stratum}.{suffix}"))
        };
        let db = lookup("direct.blind")?;
        let dr = lookup("direct.responsive")?;
        let rb = lookup("relay.blind")?;
        let rr = lookup("relay.responsive")?;
        for ((id, plus, minus), declared) in [
            ("relay_cue", rr, rb),
            ("blind_transport", rb, db),
            ("responsive_transport", rr, dr),
        ]
        .into_iter()
        .zip(&manifest.primary_contrasts)
        {
            rows.push(
                contrast_row(
                    &declared.id,
                    stratum,
                    outcome,
                    stratum == "primary",
                    plus,
                    minus,
                )
                .map_err(|e| format!("{id}: {e}"))?,
            );
        }
        let relay = rr.iter().map(|(s, v)| (*s, v - rb[s])).collect();
        let direct = dr.iter().map(|(s, v)| (*s, v - db[s])).collect();
        rows.push(contrast_row(
            "interaction_redundant_with_relay_cue",
            stratum,
            outcome,
            false,
            &relay,
            &direct,
        )?);
    }
    Ok(rows)
}
fn choice_row(
    c: &Condition,
    seeds: &[String],
    records: &BTreeMap<RunKey, Episode>,
) -> Result<ChoiceRow, String> {
    let Fixture::Choice { side, pile } = c.config.fixture else {
        return Err("choice report requires choice fixture".into());
    };
    let mut left = 0;
    let mut right = 0;
    for seed in seeds {
        let key = RunKey {
            condition: c.id.clone(),
            seed: seed.clone(),
        };
        let record = records
            .get(&key)
            .ok_or_else(|| format!("condition {} seed {}: missing choice record", c.id, seed))?;
        match record.choices.first().map(|v| v.target.x) {
            Some(1) => left += 1,
            Some(7) => right += 1,
            _ => {
                return Err(format!(
                    "condition {} seed {}: invalid choice target",
                    c.id, seed
                ))
            }
        }
    }
    Ok(ChoiceRow {
        condition: c.id.clone(),
        left,
        right,
        pile_side: match side {
            Side::Left => "left",
            Side::Right => "right",
        }
        .into(),
        expected_pile_probability: if c.config.cue == Cue::Responsive
            && pile == Pile::FreshAccumulation
        {
            0.75
        } else {
            0.5
        },
    })
}
pub(super) fn analyze(archive: &Archive) -> Result<Analysis, String> {
    let index = &archive.index;
    let (conditions, seeds) = match index.panel {
        Panel::Scientific => (&index.manifest.conditions, &index.manifest.scientific_seeds),
        Panel::Construction => (
            &index.manifest.construction_conditions,
            &index.manifest.construction_seeds,
        ),
    };
    let keys = burrow::expected_keys(&index.manifest, index.panel);
    if archive.records.len() != keys.len() || keys.iter().any(|k| !archive.records.contains_key(k))
    {
        return Err("analysis requires exact complete archive key set".into());
    }
    let controls = check_controls(conditions, seeds, &archive.records)?;
    let mut rows = vec![];
    let mut projections: BTreeMap<String, Vec<RunKey>> = BTreeMap::new();
    for key in keys {
        let record = &archive.records[&key];
        if index.panel == Panel::Scientific
            && matches!(record.config.fixture, Fixture::Growing { .. })
            && record.final_summary.opportunities != 4096
        {
            return Err(format!(
                "condition {} seed {}: growing denominator must be 4096",
                key.condition, key.seed
            ));
        }
        projections
            .entry(hash(physical_projection(record).as_bytes()))
            .or_default()
            .push(key.clone());
        rows.push(
            seed_row(key.clone(), record)
                .map_err(|e| format!("condition {} seed {}: {e}", key.condition, key.seed))?,
        );
    }
    let mut means = vec![];
    let mut contrasts = vec![];
    let mut choices = vec![];
    if index.panel == Panel::Scientific {
        for c in conditions {
            let condition_rows: Vec<_> = rows.iter().filter(|r| r.key.condition == c.id).collect();
            means.push(ConditionMean {
                condition: c.id.clone(),
                n: condition_rows.len(),
                excavation: average(condition_rows.iter().filter_map(|r| r.excavation_rate)),
                disposal: average(condition_rows.iter().filter_map(|r| r.disposal_rate)),
            });
            if matches!(c.config.fixture, Fixture::Choice { .. }) {
                choices.push(choice_row(c, seeds, &archive.records)?);
            }
        }
        for outcome in ["excavation", "disposal"] {
            let mut maps: RateMaps = BTreeMap::new();
            for row in &rows {
                let rate = if outcome == "excavation" {
                    row.excavation_rate
                } else {
                    row.disposal_rate
                };
                if let Some(value) = rate {
                    maps.entry(row.key.condition.clone()).or_default().insert(
                        row.key.seed.parse::<u64>().map_err(|e| e.to_string())?,
                        value,
                    );
                }
            }
            contrasts.extend(estimate_contrasts(&index.manifest, &maps, outcome)?);
        }
    }
    Ok(Analysis{schema:"burrow-analysis-v1".into(),code_revision:index.code_revision.clone(),protocol_revision:index.protocol_revision.clone(),manifest_sha256:index.manifest_sha256.clone(),panel:index.panel,rows,means,contrasts,controls,choices,duplicates:projections.into_iter().filter(|(_,keys)|keys.len()>1).map(|(projection_sha256,keys)|DuplicateRow{projection_sha256,keys}).collect(),limitations:vec![
        "Loaded navigation uses a supplied global exit-distance scaffold; BFS diagnostics are separate from paid worker actions.".into(),
        "Workforce strata retain tick-based freshness, changing opportunities within the 32-tick cue window; strata are never pooled.".into(),
        "Geometry, worker spawn order, boundaries and choice piles are supplied. Mirrored choice sides do not imply growing-map rotation controls.".into(),
        "Configuration-independent duplicate action/choice trajectories are reported with multiplicities; matched seeds can diverge after treatment draws.".into(),
        "Student-t 95% intervals describe computational stochastic variation, not calibration or animal-population uncertainty.".into(),
        "Interaction equals the relay cue effect when direct physical controls hold and is a redundant secondary quantity.".into(),
        "Delivered-only means and carrier distributions are conditional; terminal carried and loose units remain censored, with observed ages and waiting.".into(),
        "Construction corridors supply one dig cell: legality, conservation and consistency are engineering evidence, not repeated-production transport throughput claims.".into(),
        "Physical validation reconstructs recorded transitions without controller replay; search diagnostics and fingerprints do not independently prove policy fidelity.".into(),
    ]})
}
fn scalar(value: Option<f64>) -> String {
    value.map_or_else(|| "null".into(), |v| v.to_string())
}
fn table<T: Serialize>(output: &mut String, records: &[T]) {
    let values: Vec<_> = records
        .iter()
        .map(|r| serde_json::to_value(r).expect("finite report values"))
        .collect();
    let Some(serde_json::Value::Object(first)) = values.first() else {
        output.push_str("No rows.\n\n");
        return;
    };
    let fields: Vec<_> = first.keys().collect();
    output.push_str(&format!(
        "| {} |\n| {} |\n",
        fields
            .iter()
            .map(|s| s.as_str())
            .collect::<Vec<_>>()
            .join(" | "),
        vec!["---"; fields.len()].join(" | ")
    ));
    for value in &values {
        output.push_str(&format!(
            "| {} |\n",
            fields
                .iter()
                .map(|f| value[*f].to_string().replace('|', "\\|"))
                .collect::<Vec<_>>()
                .join(" | ")
        ));
    }
    output.push('\n');
}
pub(super) fn render_results(analysis: &Analysis) -> String {
    let mut out=format!("# Burrow saved-record analysis\n\nSchema: `{}`. Panel: `{:?}`.\n\nCode revision: `{}`. Protocol revision: `{}`. Manifest SHA-256: `{}`.\n\n",analysis.schema,analysis.panel,analysis.code_revision,analysis.protocol_revision,analysis.manifest_sha256);
    out.push_str("Saved records passed physical legality, conservation and record-consistency validation before reporting. No stochastic episodes are generated by this report.\n\n");
    if analysis.panel == Panel::Construction {
        out.push_str("Construction evidence retains all 36 corridor endpoints, including censored units. Growing scientific contrasts and condition means are absent. These one-cell excavation fixtures do not measure repeated-production transport throughput.\n\n");
    }
    out.push_str("## Condition means\n\n");
    table(&mut out, &analysis.means);
    out.push_str("## Paired signed contrasts\n\nSix primary estimates are three contrasts × two outcomes within the eight-worker stratum when scientific records exist. Workforce estimates stay in separate secondary strata. Interaction is redundant with relay cue when direct control holds. Intervals are descriptive.\n\n");
    table(&mut out, &analysis.contrasts);
    out.push_str("## Physical controls\n\n");
    table(&mut out, &analysis.controls);
    out.push_str("## Supplied choice checks\n\nCounts retain mirrored pile sides; supplied probabilities are 3/4 or 1/2, with no sample-fit verdict.\n\n");
    table(&mut out, &analysis.choices);
    out.push_str("## Duplicate physical trajectories\n\nEach key list supplies the multiplicity of identical events and choices, excluding config, frames and fingerprints.\n\n");
    table(&mut out, &analysis.duplicates);
    out.push_str("## Seed-level diagnostics\n\nDelivered-only means are conditional on completed delivery; carried/loose censor counts remain alongside them. Tick ages and opportunity ages use separate clocks. Observed loose waiting starts at fixture start for supplied material.\n\n");
    for row in &analysis.rows {
        out.push_str(&format!("### {} / seed {}\n\nCompleted ticks: {}. Excavation rate: {}. Disposal rate: {}. Delivered: {}. Censored carried: {}. Censored loose: {}. Delivered-only mean ticks: {}. Delivered-only mean opportunities: {}.\n\n",row.key.condition,row.key.seed,row.completed_ticks,scalar(row.excavation_rate),scalar(row.disposal_rate),row.delivered,row.censored_carried,row.censored_loose,scalar(row.delivered_only_mean_ticks),scalar(row.delivered_only_mean_opportunities)));
        out.push_str("Action totals, inventory, disjoint exit/observation/controller searches and connected open cells (initial staging plus new digs; no coordination inference):\n\n");
        table(&mut out, std::slice::from_ref(&row.summary));
        out.push_str("Worker work:\n\n");
        table(&mut out, &row.worker_work);
        out.push_str("Spatial work and unique excavated cells:\n\n");
        table(&mut out, &row.spatial_work);
        out.push_str("Event-time dig distances:\n\n");
        table(&mut out, &row.dig_distances);
        out.push_str("Loaded/unloaded travel:\n\n");
        table(&mut out, std::slice::from_ref(&row.travel));
        out.push_str("Logical storage counts (no wall-clock timing):\n\n");
        table(&mut out, std::slice::from_ref(&row.storage));
        out.push_str(
            "All material histories and carrier distributions (delivered and censored):\n\n",
        );
        table(&mut out, &row.materials);
    }
    out.push_str("## Limitations and supplied assumptions\n\n");
    for limitation in &analysis.limitations {
        out.push_str(&format!("- {limitation}\n"));
    }
    out
}
fn write_new(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let mut file = fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(path)
        .map_err(|e| format!("{}: {e}", path.display()))?;
    file.write_all(bytes)
        .and_then(|_| file.sync_all())
        .map_err(|e| format!("{}: {e}", path.display()))
}
pub(super) fn analyze_saved(index_path: &Path, out: &Path) -> Result<(), String> {
    let archive = burrow_archive::load(index_path)?;
    let analysis = analyze(&archive)?;
    let json = serde_json::to_vec_pretty(&analysis).map_err(|e| e.to_string())?;
    let markdown = render_results(&analysis);
    fs::create_dir(out).map_err(|e| format!("new analysis directory {}: {e}", out.display()))?;
    write_new(&out.join("analysis.json"), &json)?;
    write_new(&out.join("results.md"), markdown.as_bytes())
}
#[cfg(test)]
#[path = "burrow_report_tests.rs"]
mod tests;
