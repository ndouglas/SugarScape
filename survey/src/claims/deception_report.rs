//! Descriptive paired estimates; every registered seed remains in its denominator.
use super::deception::{Condition, SCHEMA};
use super::deception_archive::{self as archive, Archive, AttemptOutcome, Census};
use crate::stats::PairedSummary;
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, path::Path};
use sugarscape_core::minds::deception::{records::*, state::SenderPolicy};
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Unavailable {
    pub seed: u64,
    pub reason: String,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Estimate {
    pub id: String,
    pub metric: String,
    pub primary: bool,
    pub denominator: usize,
    pub summary: Option<PairedSummary>,
    pub unavailable: Vec<Unavailable>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Endpoint {
    pub condition: String,
    pub seed: u64,
    pub owner_ticks_alive: u64,
    pub owner_alive: bool,
    pub thief_transferred: Option<f64>,
    pub lineage_unavailable_reason: Option<String>,
    pub elapsed_seconds: f64,
    pub biological_group: String,
    pub fingerprints: Vec<String>,
    pub cohorts: Option<sugarscape_core::minds::protection::ledger::Ledger>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BiologicalGroup {
    pub id: String,
    pub members: Vec<String>,
    pub frames: Vec<FrameRecord>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DuplicateGroup {
    pub members: Vec<String>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Cell {
    pub condition: Condition,
    pub seeds: Vec<u64>,
    pub biological_groups: Vec<String>,
    pub completed_ticks: Vec<u64>,
    pub wall_seconds: Vec<f64>,
    pub unavailable_lineage: Vec<Unavailable>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Analysis {
    pub schema: String,
    pub census: Census,
    pub endpoints: Vec<Endpoint>,
    pub cells: Vec<Cell>,
    pub estimates: Vec<Estimate>,
    pub biological_groups: Vec<BiologicalGroup>,
    pub repeated_endpoints: Vec<DuplicateGroup>,
    pub effective_aliases: Vec<DuplicateGroup>,
}
pub fn analyze(a: &Archive) -> Result<Analysis, String> {
    archive::validate_archive(a)?;
    let records = a
        .attempts
        .iter()
        .map(|attempt| {
            let outcome = attempt
                .outcome
                .as_ref()
                .ok_or("missing outcome after validation")?;
            let AttemptOutcome::Complete(record) = &outcome.outcome else {
                return Err("incomplete after validation");
            };
            Ok((
                (attempt.start.condition.clone(), attempt.start.seed),
                (record, outcome.elapsed_seconds),
            ))
        })
        .collect::<Result<BTreeMap<_, _>, &str>>()?;
    let mut result = Analysis {
        schema: SCHEMA.into(),
        census: archive::census(a),
        endpoints: vec![],
        cells: vec![],
        estimates: vec![],
        biological_groups: vec![],
        repeated_endpoints: vec![],
        effective_aliases: vec![],
    };
    let mut groups: BTreeMap<String, BiologicalGroup> = BTreeMap::new();
    let mut endpoint_vectors: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut aliases: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for c in &a.index.manifest.conditions {
        let mut cell = Cell {
            condition: c.clone(),
            seeds: a.index.manifest.seeds.clone(),
            biological_groups: vec![],
            completed_ticks: vec![],
            wall_seconds: vec![],
            unavailable_lineage: vec![],
        };
        let mut endpoint_vector = vec![];
        let mut previous_group: Option<String> = None;
        for &seed in &a.index.manifest.seeds {
            let (r, seconds) = records
                .get(&(c.id.clone(), seed))
                .ok_or("missing registered record")?;
            let mut frames = r.frames.clone();
            for frame in &mut frames {
                frame.fingerprint.clear();
            }
            let id = if let Some(id) = previous_group
                .as_ref()
                .filter(|id| groups.get(*id).is_some_and(|g| g.frames == frames))
            {
                id.clone()
            } else {
                archive::hash(&serde_json::to_vec(&frames).map_err(|e| e.to_string())?)
            };
            previous_group = Some(id.clone());
            groups
                .entry(id.clone())
                .or_insert_with(|| BiologicalGroup {
                    id: id.clone(),
                    members: vec![],
                    frames,
                })
                .members
                .push(format!("{} seed {}", c.id, seed));
            result.endpoints.push(Endpoint {
                condition: c.id.clone(),
                seed,
                owner_ticks_alive: r.owner_ticks_alive,
                owner_alive: r.owner_alive,
                thief_transferred: r.thief_transferred,
                lineage_unavailable_reason: r.lineage_unavailable_reason.clone(),
                elapsed_seconds: *seconds,
                biological_group: id.clone(),
                fingerprints: r.frames.iter().map(|f| f.fingerprint.clone()).collect(),
                cohorts: r.cohorts.clone(),
            });
            endpoint_vector.push((r.owner_ticks_alive, r.thief_transferred));
            cell.biological_groups.push(id);
            cell.completed_ticks.push(r.completed_ticks);
            cell.wall_seconds.push(*seconds);
            if let Some(reason) = &r.lineage_unavailable_reason {
                cell.unavailable_lineage.push(Unavailable {
                    seed,
                    reason: reason.clone(),
                });
            }
        }
        endpoint_vectors
            .entry(serde_json::to_string(&endpoint_vector).map_err(|e| e.to_string())?)
            .or_default()
            .push(c.id.clone());
        aliases
            .entry(serde_json::to_string(&cell.biological_groups).map_err(|e| e.to_string())?)
            .or_default()
            .push(c.id.clone());
        result.cells.push(cell);
    }
    result.biological_groups = groups.into_values().collect();
    result.repeated_endpoints = endpoint_vectors
        .into_values()
        .filter(|v| v.len() > 1)
        .map(|members| DuplicateGroup { members })
        .collect();
    result.effective_aliases = aliases
        .into_values()
        .filter(|v| v.len() > 1)
        .map(|members| DuplicateGroup { members })
        .collect();
    for sham in a
        .index
        .manifest
        .conditions
        .iter()
        .filter(|c| c.lab.sender == SenderPolicy::Sham)
    {
        for (primary, policy) in [
            (true, SenderPolicy::MatchedNeutral),
            (false, SenderPolicy::Ordinary),
        ] {
            let mut config = sham.lab.clone();
            config.sender = policy;
            let baseline = a
                .index
                .manifest
                .conditions
                .iter()
                .find(|c| c.lab == config)
                .ok_or("missing config-matched baseline")?;
            for metric in ["owner_ticks_alive", "thief_transferred"] {
                let mut left = BTreeMap::new();
                let mut right = BTreeMap::new();
                let mut unavailable = vec![];
                for &seed in &a.index.manifest.seeds {
                    let (sham_record, _) = records
                        .get(&(sham.id.clone(), seed))
                        .ok_or("missing sham seed")?;
                    let (baseline_record, _) = records
                        .get(&(baseline.id.clone(), seed))
                        .ok_or("missing baseline seed")?;
                    if metric == "owner_ticks_alive" {
                        left.insert(seed, sham_record.owner_ticks_alive as f64);
                        right.insert(seed, baseline_record.owner_ticks_alive as f64);
                    } else {
                        match (
                            sham_record.thief_transferred,
                            baseline_record.thief_transferred,
                        ) {
                            (Some(x), Some(y)) => {
                                left.insert(seed, x);
                                right.insert(seed, y);
                            }
                            _ => {
                                let reasons = [(sham, sham_record), (baseline, baseline_record)]
                                    .iter()
                                    .filter_map(|(c, r)| {
                                        r.lineage_unavailable_reason
                                            .as_ref()
                                            .map(|reason| format!("{}: {}", c.id, reason))
                                    })
                                    .collect::<Vec<_>>()
                                    .join("; ");
                                unavailable.push(Unavailable {
                                    seed,
                                    reason: reasons,
                                });
                            }
                        }
                    }
                }
                let summary = if unavailable.is_empty() {
                    Some(crate::stats::paired_summary(&left, &right)?)
                } else {
                    None
                };
                result.estimates.push(Estimate {
                    id: format!("{} minus {}", sham.id, baseline.id),
                    metric: metric.into(),
                    primary,
                    denominator: a.index.manifest.seeds.len(),
                    summary,
                    unavailable,
                });
            }
        }
    }
    Ok(result)
}
pub fn render_results(a: &Analysis) -> String {
    use std::fmt::Write;
    let mut out=format!("# Deception laboratory descriptive results\n\nSchema: `{}`. Logical matrix: 96 cells × 40 paired seeds.\n\nExecution census: planned {}, attempted {}, complete {}, failed {}, invalid {}, partial {}, pending {}, unstarted {}, unavailable lineage {}.\n\nIntervals are paired Student-t 95% descriptive intervals. No significance verdict, multiplicity claim, or overall Holds/Fails is assigned. Duplicate trajectories and endpoint vectors do not establish sample independence. Provenance does not constitute prospective scientific acceptance.\n\n",a.schema,a.census.planned,a.census.attempted,a.census.complete,a.census.failed,a.census.invalid,a.census.partial,a.census.pending,a.census.unstarted,a.census.unavailable_lineage);
    for primary in [true, false] {
        let title = if primary {
            "64 primary: Sham minus Matched-neutral"
        } else {
            "64 secondary: Sham minus Ordinary"
        };
        writeln!(out,"## {title}\n\n| Estimate | Metric | Registered n | Mean difference | 95% interval | Positive / zero / negative | Unavailable seeds and reasons |\n|---|---|---:|---:|---|---|---|").unwrap();
        for e in a.estimates.iter().filter(|e| e.primary == primary) {
            let (mean, ci, signs) = if let Some(s) = &e.summary {
                (
                    format!("{:.9}", s.mean),
                    s.ci95.map_or("unavailable".into(), |(lo, hi)| {
                        format!("[{lo:.9}, {hi:.9}]")
                    }),
                    format!("{} / {} / {}", s.positive, s.zero, s.negative),
                )
            } else {
                (
                    "unavailable".into(),
                    "unavailable".into(),
                    "unavailable".into(),
                )
            };
            let reasons = e
                .unavailable
                .iter()
                .map(|u| {
                    format!(
                        "{}: {}",
                        u.seed,
                        u.reason.replace('|', "\\| ").replace('\n', " ")
                    )
                })
                .collect::<Vec<_>>()
                .join("; ");
            writeln!(
                out,
                "| {} | {} | {} | {} | {} | {} | {} |",
                e.id, e.metric, e.denominator, mean, ci, signs, reasons
            )
            .unwrap();
        }
        out.push('\n');
    }
    out.push_str("## Cell diagnostics\n\nEvery seed's evidence, beliefs, selected targets, inspection, occupancy, stocks, action costs, consumption, lifetime, restrictions, counts, timing, lineage and fingerprints is retained in `analysis.json`. Cells reference full biological frame groups; only fingerprints are cleared in grouped frames, with original fingerprints retained in endpoint rows. Grouping excludes seed, condition label and wall time.\n\n| Cell | n | Total seconds | Unavailable lineage | Biological groups |\n|---|---:|---:|---:|---:|\n");
    for c in &a.cells {
        let unique = c
            .biological_groups
            .iter()
            .collect::<std::collections::BTreeSet<_>>()
            .len();
        writeln!(
            out,
            "| {} | {} | {:.9} | {} | {} |",
            c.condition.id,
            c.seeds.len(),
            c.wall_seconds.iter().sum::<f64>(),
            c.unavailable_lineage.len(),
            unique
        )
        .unwrap();
    }
    for (title, groups) in [
        ("Repeated 40-seed endpoint vectors", &a.repeated_endpoints),
        (
            "Effective aliases from full 40-seed biological frames",
            &a.effective_aliases,
        ),
    ] {
        writeln!(out, "\n## {title}\n").unwrap();
        for group in groups {
            writeln!(out, "- {}", group.members.join("; ")).unwrap();
        }
    }
    out
}
pub fn save(a: &Analysis, out: &Path) -> Result<(), String> {
    archive::new_directory(out)?;
    save_files(a, out)
}
pub fn save_files(a: &Analysis, out: &Path) -> Result<(), String> {
    archive::write_new(
        &out.join("analysis.json"),
        &serde_json::to_vec_pretty(a).map_err(|e| e.to_string())?,
    )?;
    archive::write_new(&out.join("results.md"), render_results(a).as_bytes())
}
#[cfg(test)]
mod tests {
    use super::super::deception_archive::{tests::*, *};
    use super::*;
    fn full_archive() -> Archive {
        let mut i = index();
        i.completed = true;
        let mut attempts = vec![];
        for c in &i.manifest.conditions {
            let template = static_record(c.lab.clone(), 20001);
            for &seed in &i.manifest.seeds {
                let mut record = template.clone();
                record.seed = seed;
                let r = i
                    .attempts
                    .iter()
                    .find(|r| r.condition == c.id && r.seed == seed)
                    .unwrap();
                attempts.push(Attempt {
                    start: start(&i, r),
                    frames: record.frames.clone(),
                    outcome: Some(OutcomeReceipt {
                        condition: c.id.clone(),
                        seed,
                        elapsed_seconds: 0.5,
                        timing_boundary: TIMING.into(),
                        outcome: AttemptOutcome::Complete(record),
                    }),
                    interrupted_tail: None,
                });
            }
        }
        Archive { index: i, attempts }
    }
    fn save_static_wire(a: &Archive, root: &Path) {
        std::fs::create_dir(root).unwrap();
        std::fs::create_dir(root.join("attempts")).unwrap();
        std::fs::write(
            root.join("index.json"),
            serde_json::to_vec(&a.index).unwrap(),
        )
        .unwrap();
        let mut previous: Option<(String, std::path::PathBuf)> = None;
        for (r, attempt) in a.index.attempts.iter().zip(&a.attempts) {
            std::fs::create_dir(root.join(&r.start).parent().unwrap()).unwrap();
            std::fs::write(
                root.join(&r.start),
                serde_json::to_vec(&attempt.start).unwrap(),
            )
            .unwrap();
            if let Some((_, path)) = previous
                .as_ref()
                .filter(|(condition, _)| condition == &r.condition)
            {
                std::fs::hard_link(path, root.join(&r.frames)).unwrap();
            } else {
                let mut bytes = vec![];
                for frame in &attempt.frames {
                    serde_json::to_writer(&mut bytes, frame).unwrap();
                    bytes.push(b'\n');
                }
                std::fs::write(root.join(&r.frames), bytes).unwrap();
                previous = Some((r.condition.clone(), root.join(&r.frames)));
            }
            std::fs::write(
                root.join(&r.outcome),
                serde_json::to_vec(attempt.outcome.as_ref().unwrap()).unwrap(),
            )
            .unwrap();
        }
    }
    #[test]
    fn deception_report_requires_full_census() {
        let a = Archive {
            index: index(),
            attempts: vec![],
        };
        assert!(analyze(&a).unwrap_err().contains("execution census"));
    }
    #[test]
    fn deception_known_zero_difference_is_not_unavailable() {
        let v = BTreeMap::from([(20001, 4.0), (20002, 4.0)]);
        assert_eq!(
            crate::stats::paired_summary(&v, &v).unwrap().ci95,
            Some((0.0, 0.0))
        );
    }
    #[test]
    fn deception_report_retains_full_families_denominators_and_deterministic_outputs() {
        let a = full_archive();
        let analysis = analyze(&a).unwrap();
        assert_eq!(
            (
                analysis.endpoints.len(),
                analysis.cells.len(),
                analysis.estimates.len()
            ),
            (3840, 96, 128)
        );
        assert_eq!(analysis.estimates.iter().filter(|e| e.primary).count(), 64);
        assert!(analysis
            .estimates
            .iter()
            .all(|e| e.denominator == 40 && e.summary.as_ref().unwrap().n == 40));
        // Each condition repeats the same static trajectory across all40 labels.
        // Its paired differences are constant, but policy-valid conditions can
        // have different endpoints. The zero-difference control is separate.
        assert!(analysis.estimates.iter().all(|e| {
            let summary = e.summary.as_ref().unwrap();
            summary.ci95 == Some((summary.mean, summary.mean))
        }));
        assert!(
            !analysis.biological_groups.is_empty()
                && !analysis.repeated_endpoints.is_empty()
                && !analysis.effective_aliases.is_empty()
        );
        let dir = Temp::new();
        let first = dir.0.join("first");
        let second = dir.0.join("second");
        save(&analysis, &first).unwrap();
        let raw = dir.0.join("saved-wire");
        save_static_wire(&a, &raw);
        drop(a);
        let mut a = load(&raw.join("index.json")).unwrap();
        save(&analyze(&a).unwrap(), &second).unwrap();
        for file in ["analysis.json", "results.md"] {
            assert_eq!(
                std::fs::read(first.join(file)).unwrap(),
                std::fs::read(second.join(file)).unwrap()
            );
        }
        assert!(save(&analysis, &first).is_err());
        if let AttemptOutcome::Complete(r) = &mut a.attempts[0].outcome.as_mut().unwrap().outcome {
            r.cohorts = None;
            r.thief_transferred = None;
            r.diagnostics_enabled = false;
            r.lineage_unavailable_reason = Some("synthetic unavailable lineage".into());
        }
        let unavailable = analyze(&a).unwrap();
        assert!(unavailable
            .estimates
            .iter()
            .any(|e| e.metric == "thief_transferred"
                && e.summary.is_none()
                && e.denominator == 40
                && e.unavailable[0].seed == 20001));
        assert!(unavailable
            .estimates
            .iter()
            .filter(|e| e.metric == "owner_ticks_alive")
            .all(|e| e.summary.is_some()));
    }
}
