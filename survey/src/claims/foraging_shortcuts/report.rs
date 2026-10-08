//! Deterministic saved-only analysis, without a core runner or World.
use super::{report_rows::EpisodeRow, CollectionMode, Geometry, Panel, Provenance};
use serde::Serialize;
use std::path::Path;
#[derive(Clone, Debug, PartialEq, Serialize)]
pub(super) struct PairedStats {
    pub(super) n: usize,
    pub(super) mean: f64,
    pub(super) ci95: Option<(f64, f64)>,
    pub(super) positive: usize,
    pub(super) zero: usize,
    pub(super) negative: usize,
}
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum ContrastRole {
    Primary,
    Secondary,
}
#[derive(Clone, Debug, PartialEq, Serialize)]
pub(super) struct Contrast {
    pub(super) id: String,
    pub(super) panel: Panel,
    pub(super) geometry: Geometry,
    pub(super) outcome: String,
    pub(super) role: ContrastRole,
    pub(super) stats: PairedStats,
}
#[derive(Clone, Debug, PartialEq, Serialize)]
pub(super) struct Analysis {
    pub(super) schema: String,
    pub(super) mode: CollectionMode,
    pub(super) provenance: Provenance,
    pub(super) rows: Vec<EpisodeRow>,
    pub(super) contrasts: Vec<Contrast>,
}

fn ordered_rows(rows: &[EpisodeRow], mode: CollectionMode) -> Result<Vec<&EpisodeRow>, String> {
    let manifest = super::manifest::candidate()?;
    let keys = super::manifest::expected_keys(&manifest, mode);
    if rows.len() != keys.len() {
        return Err("analysis requires the complete exact seed/condition set".into());
    }
    let mut keyed = std::collections::BTreeMap::new();
    for row in rows {
        let c = super::manifest::condition(&manifest, &row.key.condition)?;
        if (row.panel, row.geometry, row.regime) != (c.panel, c.geometry, c.regime) {
            return Err(format!(
                "{}/{}: ambiguous row roles",
                row.key.condition, row.key.seed
            ));
        }
        if row.summary.food.delivered > 16 || row.summary.access.accessible > 16 {
            return Err("contrast count outside fixed food budget".into());
        }
        if keyed.insert(row.key.clone(), row).is_some() {
            return Err("duplicate analysis seed/condition".into());
        }
    }
    keys.iter()
        .map(|key| {
            keyed
                .get(key)
                .copied()
                .ok_or_else(|| format!("missing analysis pair {}/{}", key.condition, key.seed))
        })
        .collect()
}
pub(super) fn build_contrasts(rows: &[EpisodeRow]) -> Result<Vec<Contrast>, String> {
    let rows = ordered_rows(rows, CollectionMode::Scientific)?;
    let mut contrasts = Vec::with_capacity(15);
    let geometries = [
        (Geometry::Straight, "straight"),
        (Geometry::Detour, "detour"),
        (Geometry::Twisting, "twisting"),
    ];
    for (geometry, name) in geometries {
        contrasts.push(contrast(
            &rows,
            Panel::Route,
            geometry,
            &format!("route.{name}"),
            ("paid", "protected"),
            "delivered",
            ContrastRole::Primary,
        )?);
    }
    for (geometry, name) in geometries {
        for pair in [("already_open", "protected"), ("paid", "already_open")] {
            contrasts.push(contrast(
                &rows,
                Panel::Route,
                geometry,
                &format!("route.{name}"),
                pair,
                "delivered",
                ContrastRole::Secondary,
            )?);
        }
    }
    for (geometry, name) in geometries {
        for outcome in ["delivered", "accessible"] {
            contrasts.push(contrast(
                &rows,
                Panel::Access,
                geometry,
                &format!("access.{name}"),
                ("paid", "protected"),
                outcome,
                ContrastRole::Secondary,
            )?);
        }
    }
    Ok(contrasts)
}
fn contrast(
    rows: &[&EpisodeRow],
    panel: Panel,
    geometry: Geometry,
    prefix: &str,
    pair: (&str, &str),
    outcome: &str,
    role: ContrastRole,
) -> Result<Contrast, String> {
    let values = |regime: &str| {
        rows.iter()
            .filter(|r| r.key.condition == format!("{prefix}.{regime}"))
            .map(|r| {
                (
                    r.key.seed,
                    f64::from(if outcome == "delivered" {
                        r.summary.food.delivered
                    } else {
                        r.summary.access.accessible
                    }),
                )
            })
            .collect::<std::collections::BTreeMap<_, _>>()
    };
    let s = crate::stats::paired_summary(&values(pair.0), &values(pair.1))?;
    Ok(Contrast {
        id: format!("{prefix}.{}_minus_{}.{outcome}", pair.0, pair.1),
        panel,
        geometry,
        outcome: outcome.into(),
        role,
        stats: PairedStats {
            n: s.n,
            mean: s.mean,
            ci95: s.ci95,
            positive: s.positive,
            zero: s.zero,
            negative: s.negative,
        },
    })
}
pub(super) fn analyze(index: &Path, out: &Path) -> Result<Analysis, String> {
    let mut reader = super::archive::ArchiveReader::open(index)?;
    let manifest = super::manifest::candidate()?;
    let mut analysis = Analysis {
        schema: "foraging-shortcut-analysis-v1".into(),
        mode: reader.index().mode,
        provenance: reader.index().provenance.clone(),
        rows: Vec::with_capacity(reader.index().expected_keys.len()),
        contrasts: vec![],
    };
    // next() must reach terminal None: it checks cumulative saved bytes as well.
    // Each raw episode is dropped before the next read; only bounded rows remain.
    while let Some((key, episode)) = reader.next()? {
        let row = super::report_rows::project_row(
            super::manifest::condition(&manifest, &key.condition)?,
            &key,
            &episode,
        )
        .map_err(|e| format!("{}/{} projection: {e}", key.condition, key.seed))?;
        analysis.rows.push(row);
    }
    ordered_rows(&analysis.rows, analysis.mode)?;
    if analysis.mode == CollectionMode::Scientific {
        analysis.contrasts = build_contrasts(&analysis.rows)?;
    }
    let mut json =
        serde_json::to_vec_pretty(&analysis).map_err(|e| format!("analysis JSON: {e}"))?;
    json.push(b'\n');
    let text = markdown(&analysis)?;
    // All validation/serialization completes before any output path is created.
    super::super::protection_archive::new_directory(out)?;
    super::super::protection_archive::write_new(&out.join("analysis.json"), &json)
        .and_then(|_| {
            super::super::protection_archive::write_new(&out.join("results.md"), text.as_bytes())
        })
        .map_err(|e| format!("report output incomplete: {e}"))?;
    Ok(analysis)
}
fn option_number(value: Option<u32>) -> String {
    value.map_or_else(|| "null".into(), |v| v.to_string())
}
pub(super) fn markdown(a: &Analysis) -> Result<String, String> {
    let rows = ordered_rows(&a.rows, a.mode)?;
    let mut text = String::from("# F5 shortcut comparison saved evidence\n\n");
    match a.mode {
        CollectionMode::Construction => text.push_str("Construction engineering checks. All 30 seed rows, including programmed zeros and censored outcomes, are retained. No condition means or scientific contrasts are estimated.\n\n"),
        CollectionMode::Scientific => text.push_str("Descriptive computational seed variation: condition means and paired Student-t 95% intervals describe whole episodes, not animal-population uncertainty. No pooled efficacy score or threshold verdict.\n\n"),
    }
    text.push_str("Candidate inputs are supplied mechanism choices, not biological calibration. The source workbook/README and figure remain uninspected.\n\n");
    text.push_str("Rows preserve full final summaries, all work categories, worker computation and separate researcher access computation. Checkpoints count completed ticks; milestones retain zero-based processing ticks and one-based opportunity indexes. Null milestones are censored at the fixed cutoff; null route distances mean disconnected. All original food positions remain in the route curve after delivery.\n\n");
    text.push_str("The digest is a sampled physical projection of saved clock/open/worker position-phase-mode-cargo-work/food/spoil states. Equal digests indicate equal sampled projections; coarse frames cannot prove complete trajectory, controller or RNG identity. First shortening is observed in (previous checkpoint, current checkpoint], not an excavation timestamp. Sealed initial-to-final gain and first shortening remain null.\n\n");
    text.push_str(&format!("Provenance: code `{}`, protocol `{}`, manifest SHA-256 `{}`, collector SHA-256 `{}`. Raw references below are relative to the archive index, independent of this report's output directory.\n\n",
        a.provenance.code_revision, a.provenance.protocol_revision, a.provenance.manifest_sha256, a.provenance.collector_sha256));
    for (panel, title) in [
        (Panel::Route, "Route panel"),
        (Panel::Access, "Sealed access panel (secondary)"),
    ] {
        text.push_str(&format!("## {title}\n\n"));
        text.push_str("| Condition | Seed | Delivered | Carried food | Accessible | Initial distance | Final distance | Gain | Shortening window (completed ticks) | First access processing tick | First pickup processing tick | First delivery processing tick | All food processing tick |\n| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- | ---: | ---: | ---: | ---: |\n");
        for r in rows.iter().filter(|r| r.panel == panel) {
            let window = r.first_shortening.as_ref().map_or_else(
                || "null".into(),
                |w| format!("({}, {}]", w.after_completed_tick, w.by_completed_tick),
            );
            text.push_str(&format!(
                "| {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} |\n",
                r.key.condition,
                r.key.seed,
                r.summary.food.delivered,
                r.summary.food.carried,
                r.summary.access.accessible,
                option_number(r.initial_distance),
                option_number(r.final_distance),
                option_number(r.gain),
                window,
                option_number(
                    r.summary
                        .milestones
                        .first_access
                        .as_ref()
                        .map(|e| e.context.tick)
                ),
                option_number(r.summary.milestones.first_pickup_tick),
                option_number(r.summary.milestones.first_delivery_tick),
                option_number(r.summary.milestones.all_food_delivered_tick)
            ));
        }
        text.push('\n');
    }
    if a.mode == CollectionMode::Scientific {
        text.push_str("## Condition means (descriptive)\n\n| Condition | Episodes | Delivered mean | Accessible mean |\n| --- | ---: | ---: | ---: |\n");
        let manifest = super::manifest::candidate()?;
        for c in &manifest.conditions {
            let condition_rows = rows
                .iter()
                .filter(|r| r.key.condition == c.id)
                .collect::<Vec<_>>();
            let n = condition_rows.len();
            let delivered = condition_rows
                .iter()
                .map(|r| f64::from(r.summary.food.delivered))
                .sum::<f64>()
                / n as f64;
            let accessible = condition_rows
                .iter()
                .map(|r| f64::from(r.summary.access.accessible))
                .sum::<f64>()
                / n as f64;
            text.push_str(&format!(
                "| {} | {n} | {delivered:.6} | {accessible:.6} |\n",
                c.id
            ));
        }
        text.push('\n');
        let contrasts = build_contrasts(&a.rows)?;
        if contrasts != a.contrasts {
            return Err("analysis contrasts differ from complete paired rows".into());
        }
        for (role, title) in [
            (ContrastRole::Primary, "Primary route delivery differences"),
            (
                ContrastRole::Secondary,
                "Secondary route references and sealed access differences",
            ),
        ] {
            text.push_str(&format!("## {title}\n\n| Contrast (A minus B) | n | Mean difference | Student-t 95% interval | Positive | Zero | Negative |\n| --- | ---: | ---: | --- | ---: | ---: | ---: |\n"));
            for c in contrasts.iter().filter(|c| c.role == role) {
                let s = &c.stats;
                let interval = s
                    .ci95
                    .map_or_else(|| "null".into(), |(lo, hi)| format!("[{lo:.6}, {hi:.6}]"));
                text.push_str(&format!(
                    "| {} | {} | {:.6} | {} | {} | {} | {} |\n",
                    c.id, s.n, s.mean, interval, s.positive, s.zero, s.negative
                ));
            }
            text.push('\n');
        }
    } else if !a.contrasts.is_empty() {
        return Err("construction analysis cannot contain scientific contrasts".into());
    }
    text.push_str("## Complete seed rows and censoring\n\nThe following records retain every final metric, milestone, per-resource access record, route checkpoint and sampled physical digest without selecting only successful delivery.\n\n");
    for row in rows {
        text.push_str(&format!(
            "### {} / {}\n\nRaw reference: `{}`.\n\n```json\n{}\n```\n\n",
            row.key.condition,
            row.key.seed,
            super::archive::raw_path(&row.key),
            serde_json::to_string_pretty(row).map_err(|e| e.to_string())?
        ));
    }
    Ok(text)
}
