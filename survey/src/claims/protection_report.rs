//! Complete descriptive paired estimates; no overall classification.
use super::{
    protection::{Manifest, SCHEMA},
    protection_archive::{validate_archive, Index},
};
use crate::stats::{paired_summary, PairedSummary};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use sugarscape_core::{
    geometry::Pos,
    minds::protection::{
        runner::EpisodeRecord,
        state::{Fixture, LabConfig, Policy},
    },
};
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Estimate {
    pub id: String,
    pub metric: String,
    pub primary: bool,
    pub summary: Option<PairedSummary>,
    pub unavailable_seeds: Vec<u64>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SourceRow {
    pub source: u32,
    pub initial: f64,
    pub perceived: bool,
    pub actual_sightings: u64,
    pub withdrawn: f64,
    pub redeposited: f64,
    pub attempts: u64,
    pub cancellations: u64,
    pub distance: u64,
    pub first_transferred: Option<f64>,
    pub transferred_fraction: Option<f64>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PhaseRow {
    pub role: u64,
    pub phase: String,
    pub first_tick: u64,
    pub last_tick: u64,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OpportunityRow {
    pub tick: u64,
    pub source: u32,
    pub destination: Pos,
    pub actual_sighting: bool,
    pub owner_departed_at_release: bool,
    pub observer_reached: bool,
}
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Endpoint {
    pub condition: String,
    pub seed: u64,
    pub completed_ticks: u64,
    pub owner_ticks_alive: u64,
    pub owner_alive: bool,
    pub thief_transferred: Option<f64>,
    pub initial_food: f64,
    pub total_harvest: f64,
    pub owner_harvest: f64,
    pub observer_harvest: f64,
    pub closing_owner_holdings: f64,
    pub closing_observer_holdings: f64,
    pub closing_stock: f64,
    pub tagged_consumed: Option<f64>,
    pub tagged_cost: Option<f64>,
    pub tagged_terminal_loss: Option<f64>,
    pub burial_cost: f64,
    pub metabolic_demand: f64,
    pub metabolic_consumed: f64,
    pub attempts: u64,
    pub withdrawn: f64,
    pub redeposited: f64,
    pub protective_ticks: u64,
    pub distance: u64,
    pub cancellations: BTreeMap<String, u64>,
    pub restriction_ticks: BTreeMap<u64, u64>,
    pub old_site_arrivals: u64,
    pub new_site_arrivals: u64,
    pub old_site_raids: u64,
    pub new_site_raids: u64,
    pub wasted_raids: u64,
    pub discoveries: u64,
    pub cue_errors: u64,
    pub sightings: u64,
    pub mixed_selectivity: Option<f64>,
    pub sources: Vec<SourceRow>,
    pub phases: Vec<PhaseRow>,
    pub opportunities: Vec<OpportunityRow>,
    pub lineage_errors: Vec<String>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DuplicateRow {
    pub condition: String,
    pub runs: usize,
    pub unique_trajectories: usize,
    pub duplicate_runs: usize,
    pub largest_group: usize,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Analysis {
    pub schema: String,
    pub manifest: Manifest,
    pub estimates: Vec<Estimate>,
    pub endpoints: Vec<Endpoint>,
    pub duplicates: Vec<DuplicateRow>,
}
fn summarize(condition: &str, r: &EpisodeRecord) -> Endpoint {
    let actions: Vec<_> = r.frames.iter().flat_map(|f| &f.actions).collect();
    let prep = &r.frames[8]
        .roles
        .iter()
        .find(|r| r.id == 1)
        .unwrap()
        .protection
        .as_ref()
        .unwrap()
        .sources;
    let mut sources = Vec::new();
    for (&source, metadata) in prep {
        let perceived = r.frames[8]
            .roles
            .iter()
            .find(|r| r.id == 1)
            .unwrap()
            .protection
            .as_ref()
            .unwrap()
            .exposure
            .entries[&source]
            .exposed;
        let source_actions: Vec<_> = actions
            .iter()
            .filter(|a| a.id == 1 && a.source == Some(source))
            .collect();
        let events: Vec<_> = r
            .frames
            .iter()
            .flat_map(|f| &f.relocation.source_events)
            .filter(|e| e.source == source)
            .collect();
        let distance = r
            .frames
            .windows(2)
            .map(|w| {
                if !w[1]
                    .actions
                    .iter()
                    .any(|a| a.id == 1 && a.source == Some(source) && a.phase == "relocation")
                {
                    return 0;
                }
                let before = w[0].roles.iter().find(|r| r.id == 1).map(|r| r.pos);
                let after = w[1]
                    .roles
                    .iter()
                    .find(|r| r.id == 1)
                    .map(|r| r.pos)
                    .or_else(|| w[1].deaths.iter().find(|d| d.id == 1).map(|d| d.pos));
                before
                    .zip(after)
                    .map_or(0, |(a, b)| (a.x.abs_diff(b.x) + a.y.abs_diff(b.y)) as u64)
            })
            .sum();
        let transferred = r
            .cohorts
            .as_ref()
            .and_then(|l| l.cohorts.get(&source))
            .map(|c| c.transferred);
        sources.push(SourceRow {
            source,
            initial: metadata.initial_amount,
            perceived,
            actual_sightings: source_actions
                .iter()
                .filter(|a| a.action == "prepare_deposit" && a.actual_watchers.contains(&2))
                .count() as u64,
            withdrawn: events.iter().map(|e| e.withdrawn).sum(),
            redeposited: events.iter().map(|e| e.redeposited).sum(),
            attempts: events.iter().filter(|e| e.started).count() as u64,
            cancellations: events.iter().filter(|e| e.cancellation.is_some()).count() as u64,
            distance,
            first_transferred: transferred,
            transferred_fraction: transferred.map(|q| q / metadata.initial_amount),
        });
    }
    let mut phases = Vec::<PhaseRow>::new();
    for id in [1, 2] {
        for f in r.frames.iter().skip(1) {
            if let Some(a) = f.actions.iter().find(|a| a.id == id) {
                let tick = f.tick - 1;
                if let Some(last) = phases
                    .last_mut()
                    .filter(|p| p.role == id && p.phase == a.phase && p.last_tick + 1 == tick)
                {
                    last.last_tick = tick;
                } else {
                    phases.push(PhaseRow {
                        role: id,
                        phase: a.phase.clone(),
                        first_tick: tick,
                        last_tick: tick,
                    });
                }
            }
        }
    }
    let release = match r.lab.fixture {
        Fixture::Mixed { .. } => 20,
        Fixture::Stumble { .. } => 18,
        _ => 12,
    };
    let mut opportunities = Vec::new();
    for f in &r.frames {
        for a in &f.actions {
            if a.phase == "relocation" && a.gross_buried > 0.0 {
                let destination = a.target.unwrap();
                opportunities.push(OpportunityRow {
                    tick: f.tick - 1,
                    source: a.source.unwrap(),
                    destination,
                    actual_sighting: a.actual_watchers.contains(&2),
                    owner_departed_at_release: r
                        .frames
                        .get(release)
                        .is_some_and(|f| f.roles.iter().all(|r| r.pos != destination)),
                    observer_reached: r
                        .frames
                        .iter()
                        .skip(release + 1)
                        .any(|f| f.roles.iter().any(|r| r.id == 2 && r.pos == destination)),
                });
            }
        }
    }
    let old: BTreeSet<_> = prep.keys().copied().collect();
    let new: BTreeSet<_> = opportunities
        .iter()
        .map(|o| o.destination.y * 9 + o.destination.x)
        .collect();
    let arrivals = |sites: &BTreeSet<u32>| {
        r.frames
            .windows(2)
            .filter(|w| {
                let before = w[0].roles.iter().find(|r| r.id == 2);
                let after = w[1].roles.iter().find(|r| r.id == 2);
                before.zip(after).is_some_and(|(a, b)| {
                    a.pos != b.pos && sites.contains(&(b.pos.y * 9 + b.pos.x))
                })
            })
            .count() as u64
    };
    let raids = |sites: &BTreeSet<u32>| {
        actions
            .iter()
            .filter(|a| a.id == 2 && a.raid_site.is_some_and(|s| sites.contains(&s)))
            .count() as u64
    };
    let mut cancellations = BTreeMap::new();
    for f in &r.frames {
        for (reason, n) in &f.relocation.cancellations {
            *cancellations.entry(format!("{reason:?}")).or_default() += u64::from(*n);
        }
    }
    let last = r.frames.last().unwrap();
    let closing = |id| {
        last.roles
            .iter()
            .find(|r| r.id == id)
            .map_or(0.0, |r| r.holdings)
    };
    let tagged = |f: fn(&sugarscape_core::minds::protection::ledger::CohortBalance) -> f64| {
        r.cohorts.as_ref().map(|l| l.cohorts.values().map(f).sum())
    };
    let mixed_selectivity = if matches!(r.lab.fixture, Fixture::Mixed { .. }) {
        Some(
            sources
                .iter()
                .filter(|s| s.perceived)
                .map(|s| s.withdrawn / s.initial)
                .sum::<f64>()
                - sources
                    .iter()
                    .filter(|s| !s.perceived)
                    .map(|s| s.withdrawn / s.initial)
                    .sum::<f64>(),
        )
    } else {
        None
    };
    Endpoint {
        condition: condition.into(),
        seed: r.seed,
        completed_ticks: r.completed_ticks,
        owner_ticks_alive: r.owner_ticks_alive,
        owner_alive: r.owner_alive,
        thief_transferred: r.thief_transferred,
        initial_food: sources.iter().map(|s| s.initial).sum(),
        total_harvest: actions.iter().map(|a| a.harvest).sum(),
        owner_harvest: actions
            .iter()
            .filter(|a| a.id == 1)
            .map(|a| a.harvest)
            .sum(),
        observer_harvest: actions
            .iter()
            .filter(|a| a.id == 2)
            .map(|a| a.harvest)
            .sum(),
        closing_owner_holdings: closing(1),
        closing_observer_holdings: closing(2),
        closing_stock: last.roles.iter().flat_map(|r| r.caches.values()).sum(),
        tagged_consumed: tagged(|c| c.consumed),
        tagged_cost: tagged(|c| c.cost),
        tagged_terminal_loss: tagged(|c| c.lost_carried + c.lost_cached.values().sum::<f64>()),
        burial_cost: actions.iter().map(|a| a.burial_cost).sum(),
        metabolic_demand: actions.iter().map(|a| a.metabolic_demand).sum(),
        metabolic_consumed: actions.iter().map(|a| a.metabolic_consumed).sum(),
        attempts: r
            .frames
            .iter()
            .map(|f| u64::from(f.relocation.starts))
            .sum(),
        withdrawn: r.frames.iter().map(|f| f.relocation.withdrawn).sum(),
        redeposited: r.frames.iter().map(|f| f.relocation.redeposited).sum(),
        protective_ticks: r
            .frames
            .iter()
            .map(|f| u64::from(f.relocation.action_ticks))
            .sum(),
        distance: r
            .frames
            .iter()
            .map(|f| u64::from(f.relocation.distance))
            .sum(),
        cancellations,
        restriction_ticks: last.restriction_ticks.clone(),
        old_site_arrivals: arrivals(&old),
        new_site_arrivals: arrivals(&new),
        old_site_raids: raids(&old),
        new_site_raids: raids(&new),
        wasted_raids: actions.iter().filter(|a| a.raid_wasted).count() as u64,
        discoveries: actions
            .iter()
            .filter(|a| a.discovery_site.is_some())
            .count() as u64,
        cue_errors: sources
            .iter()
            .filter(|s| s.perceived != (s.actual_sightings > 0))
            .count() as u64,
        sightings: actions.iter().map(|a| a.actual_watchers.len() as u64).sum(),
        mixed_selectivity,
        sources,
        phases,
        opportunities,
        lineage_errors: r.ledger_errors.clone(),
    }
}
fn value(e: &Endpoint, metric: &str) -> Option<f64> {
    match metric {
        "original_food_transferred" => e.thief_transferred,
        "owner_ticks_alive" => Some(e.owner_ticks_alive as f64),
        "discovered_encounters" => Some(e.discoveries as f64),
        _ => None,
    }
}
fn estimate(
    id: String,
    metric: &str,
    primary: bool,
    seeds: &[u64],
    terms: &[(&str, f64)],
    rows: &BTreeMap<(&str, u64), &Endpoint>,
) -> Result<Estimate, String> {
    let mut unavailable_seeds = Vec::new();
    let mut a = BTreeMap::new();
    let b: BTreeMap<_, _> = seeds.iter().map(|&s| (s, 0.0)).collect();
    for &seed in seeds {
        let values: Option<Vec<_>> = terms
            .iter()
            .map(|(condition, weight)| {
                rows.get(&(*condition, seed))
                    .and_then(|e| value(e, metric))
                    .map(|q| q * weight)
            })
            .collect();
        if let Some(values) = values {
            a.insert(seed, values.iter().sum());
        } else {
            unavailable_seeds.push(seed);
        }
    }
    let summary = if unavailable_seeds.is_empty() {
        Some(paired_summary(&a, &b)?)
    } else {
        None
    };
    Ok(Estimate {
        id,
        metric: metric.into(),
        primary,
        summary,
        unavailable_seeds,
    })
}
fn condition<'a>(m: &'a Manifest, lab: &LabConfig) -> Result<&'a str, String> {
    m.conditions
        .iter()
        .find(|c| &c.lab == lab)
        .map(|c| c.id.as_str())
        .ok_or_else(|| "contrast cell absent from manifest".into())
}
fn analyze_endpoints(
    index: &Index,
    endpoints: Vec<Endpoint>,
    duplicates: Vec<DuplicateRow>,
) -> Result<Analysis, String> {
    let rows: BTreeMap<_, _> = endpoints
        .iter()
        .map(|e| ((e.condition.as_str(), e.seed), e))
        .collect();
    if rows.len() != index.manifest.conditions.len() * index.manifest.seeds.len()
        || rows.len() != endpoints.len()
    {
        return Err("endpoint identities do not form exact matrix".into());
    }
    for c in &index.manifest.conditions {
        for &seed in &index.manifest.seeds {
            if !rows.contains_key(&(c.id.as_str(), seed)) {
                return Err("endpoint missing registered seed".into());
            }
        }
    }
    let mut estimates = Vec::new();
    for c in &index.manifest.conditions {
        if matches!(c.lab.fixture, Fixture::Single { .. }) && c.lab.policy == Policy::Selective {
            for baseline in [Policy::Off, Policy::Indiscriminate] {
                let b = condition(
                    &index.manifest,
                    &LabConfig {
                        policy: baseline.clone(),
                        ..c.lab.clone()
                    },
                )?;
                for metric in ["original_food_transferred", "owner_ticks_alive"] {
                    estimates.push(estimate(
                        format!("{} minus {b}", c.id),
                        metric,
                        true,
                        &index.manifest.seeds,
                        &[(&c.id, 1.0), (b, -1.0)],
                        &rows,
                    )?);
                }
            }
            if let Fixture::Single {
                initial_observed,
                redeposit_observed: true,
            } = c.lab.fixture
            {
                let private = LabConfig {
                    fixture: Fixture::Single {
                        initial_observed,
                        redeposit_observed: false,
                    },
                    ..c.lab.clone()
                };
                let observed_off = condition(
                    &index.manifest,
                    &LabConfig {
                        policy: Policy::Off,
                        ..c.lab.clone()
                    },
                )?;
                let private_off = condition(
                    &index.manifest,
                    &LabConfig {
                        policy: Policy::Off,
                        ..private.clone()
                    },
                )?;
                let private = condition(&index.manifest, &private)?;
                for metric in ["original_food_transferred", "owner_ticks_alive"] {
                    estimates.push(estimate(
                        format!("opportunity interaction {}", c.id),
                        metric,
                        false,
                        &index.manifest.seeds,
                        &[
                            (&c.id, 1.0),
                            (observed_off, -1.0),
                            (private, -1.0),
                            (private_off, 1.0),
                        ],
                        &rows,
                    )?);
                }
            }
        }
        if matches!(c.lab.fixture, Fixture::Single { .. }) && c.lab.policy == Policy::Erased {
            let b = condition(
                &index.manifest,
                &LabConfig {
                    policy: Policy::Off,
                    ..c.lab.clone()
                },
            )?;
            for metric in ["original_food_transferred", "owner_ticks_alive"] {
                estimates.push(estimate(
                    format!("{} minus {b}", c.id),
                    metric,
                    false,
                    &index.manifest.seeds,
                    &[(&c.id, 1.0), (b, -1.0)],
                    &rows,
                )?);
            }
        }
        if matches!(c.lab.fixture, Fixture::Stumble { .. }) && c.lab.discovery == 0.25 {
            let b = condition(
                &index.manifest,
                &LabConfig {
                    discovery: 0.0,
                    ..c.lab.clone()
                },
            )?;
            for metric in [
                "original_food_transferred",
                "owner_ticks_alive",
                "discovered_encounters",
            ] {
                estimates.push(estimate(
                    format!("{} minus {b}", c.id),
                    metric,
                    false,
                    &index.manifest.seeds,
                    &[(&c.id, 1.0), (b, -1.0)],
                    &rows,
                )?);
            }
        }
    }
    estimates.sort_by(|a, b| (&a.id, &a.metric).cmp(&(&b.id, &b.metric)));
    Ok(Analysis {
        schema: SCHEMA.into(),
        manifest: index.manifest.clone(),
        estimates,
        endpoints,
        duplicates,
    })
}
pub fn analyze(index: &Index, records: &[EpisodeRecord]) -> Result<Analysis, String> {
    validate_archive(index, records)?;
    let mut endpoints = Vec::new();
    let mut trajectories = BTreeMap::<String, BTreeMap<Vec<u8>, usize>>::new();
    for (raw, r) in index.runs.iter().zip(records) {
        endpoints.push(summarize(&raw.condition, r));
        let mut frames = r.frames.clone();
        for f in &mut frames {
            f.fingerprint.clear();
        }
        let bytes = serde_json::to_vec(&frames).map_err(|e| e.to_string())?;
        *trajectories
            .entry(raw.condition.clone())
            .or_default()
            .entry(bytes)
            .or_default() += 1;
    }
    endpoints.sort_by(|a, b| (&a.condition, a.seed).cmp(&(&b.condition, b.seed)));
    let duplicates = trajectories
        .into_iter()
        .map(|(condition, groups)| {
            let runs = groups.values().sum();
            DuplicateRow {
                condition,
                runs,
                unique_trajectories: groups.len(),
                duplicate_runs: runs - groups.len(),
                largest_group: *groups.values().max().unwrap(),
            }
        })
        .collect();
    analyze_endpoints(index, endpoints, duplicates)
}
pub fn render_results(a: &Analysis) -> String {
    use std::fmt::Write;
    let mut out=String::from("# Registered protection campaign\n\nPaired descriptive Student-t 95% intervals use the full 40 registered seeds. No multiplicity-adjusted verdict or overall success classification is supplied. Programmed selectivity is a manipulation check. Vision, cue errors, contact routes, finite food, blocking, stale memories, paired RNG divergence and the finite horizon limit causal interpretation. Reflections and opportunity treatments remain separate.\n\nRaw run timing includes world construction and biological steps, excludes archive I/O and analysis, and is descriptive computational cost only. Timing and archive paths are excluded from this report and analysis.\n\n## Every registered comparison\n\n| comparison | metric | primary | n | mean | 95% interval | positive/zero/negative | unavailable seeds |\n|---|---|---|---|---|---|---|---|\n");
    for e in &a.estimates {
        if let Some(s) = &e.summary {
            writeln!(
                out,
                "| {} | {} | {} | {} | {:.9} | {:?} | {}/{}/{} | |",
                e.id, e.metric, e.primary, s.n, s.mean, s.ci95, s.positive, s.zero, s.negative
            )
            .unwrap();
        } else {
            writeln!(
                out,
                "| {} | {} | {} | unavailable | | | | {:?} |",
                e.id, e.metric, e.primary, e.unavailable_seeds
            )
            .unwrap();
        }
    }
    out.push_str("\n## All cells and secondary endpoints\n\nMeans retain deaths and all seeds. Food fractions use the original 12-unit deposit (6 per mixed source), never gross repeated burial. Missing lineage makes a food mean unavailable. Complete per-seed endpoints, source metrics, phase boundaries, cue/lineage diagnostics, opportunities, cancellations and restriction counts are retained in analysis.json; authoritative per-tick roles and death events remain in raw envelopes.\n\n| condition | food | owner alive ticks | alive horizon | harvest | owner/observer closing holdings | live stock | tagged consumed/cost/lost | burial cost | metabolic demand/consumed | attempts | withdrawn/redeposited | protective ticks/distance | old/new arrivals | old/new raids | wasted raids | discoveries | cue errors/sightings | selectivity | realized/departed/observed redeposits |\n|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|\n");
    for c in &a.manifest.conditions {
        let rows: Vec<_> = a.endpoints.iter().filter(|e| e.condition == c.id).collect();
        let n = rows.len() as f64;
        let mean = |f: fn(&Endpoint) -> f64| rows.iter().map(|e| f(e)).sum::<f64>() / n;
        let optional = |f: fn(&Endpoint) -> Option<f64>| {
            rows.iter()
                .map(|e| f(e))
                .collect::<Option<Vec<_>>>()
                .map(|q| format!("{:.6}", q.iter().sum::<f64>() / n))
                .unwrap_or_else(|| "unavailable".into())
        };
        writeln!(out,"| {} | {} | {:.6} | {:.6} | {:.6} | {:.6}/{:.6} | {:.6} | {}/{}/{} | {:.6} | {:.6}/{:.6} | {:.6} | {:.6}/{:.6} | {:.6}/{:.6} | {:.6}/{:.6} | {:.6}/{:.6} | {:.6} | {:.6} | {:.6}/{:.6} | {} | {:.6}/{:.6}/{:.6} |",c.id,optional(|e|e.thief_transferred),mean(|e|e.owner_ticks_alive as f64),mean(|e|u8::from(e.owner_alive)as f64),mean(|e|e.total_harvest),mean(|e|e.closing_owner_holdings),mean(|e|e.closing_observer_holdings),mean(|e|e.closing_stock),optional(|e|e.tagged_consumed),optional(|e|e.tagged_cost),optional(|e|e.tagged_terminal_loss),mean(|e|e.burial_cost),mean(|e|e.metabolic_demand),mean(|e|e.metabolic_consumed),mean(|e|e.attempts as f64),mean(|e|e.withdrawn),mean(|e|e.redeposited),mean(|e|e.protective_ticks as f64),mean(|e|e.distance as f64),mean(|e|e.old_site_arrivals as f64),mean(|e|e.new_site_arrivals as f64),mean(|e|e.old_site_raids as f64),mean(|e|e.new_site_raids as f64),mean(|e|e.wasted_raids as f64),mean(|e|e.discoveries as f64),mean(|e|e.cue_errors as f64),mean(|e|e.sightings as f64),optional(|e|e.mixed_selectivity),mean(|e|e.opportunities.iter().filter(|o|o.observer_reached).count()as f64),mean(|e|e.opportunities.iter().filter(|o|o.owner_departed_at_release).count()as f64),mean(|e|e.opportunities.iter().filter(|o|o.actual_sighting).count()as f64)).unwrap();
    }
    out.push_str("\n## Source denominators and travel\n\n| condition | source | initial | perceived cue | actual preparation sightings | withdrawn | redeposited | attempts | cancellations | distance | transferred fraction |\n|---|---|---|---|---|---|---|---|---|---|---|\n");
    for c in &a.manifest.conditions {
        let rows: Vec<_> = a.endpoints.iter().filter(|e| e.condition == c.id).collect();
        if let Some(first) = rows.first() {
            for source in &first.sources {
                let sources: Vec<_> = rows
                    .iter()
                    .flat_map(|e| &e.sources)
                    .filter(|s| s.source == source.source)
                    .collect();
                let n = sources.len() as f64;
                let fraction = sources
                    .iter()
                    .map(|s| s.transferred_fraction)
                    .collect::<Option<Vec<_>>>()
                    .map(|v| format!("{:.6}", v.iter().sum::<f64>() / n))
                    .unwrap_or_else(|| "unavailable".into());
                writeln!(
                    out,
                    "| {} | {} | {} | {} | {:.6} | {:.6} | {:.6} | {:.6} | {:.6} | {:.6} | {} |",
                    c.id,
                    source.source,
                    source.initial,
                    source.perceived,
                    sources
                        .iter()
                        .map(|s| s.actual_sightings as f64)
                        .sum::<f64>()
                        / n,
                    sources.iter().map(|s| s.withdrawn).sum::<f64>() / n,
                    sources.iter().map(|s| s.redeposited).sum::<f64>() / n,
                    sources.iter().map(|s| s.attempts as f64).sum::<f64>() / n,
                    sources.iter().map(|s| s.cancellations as f64).sum::<f64>() / n,
                    sources.iter().map(|s| s.distance as f64).sum::<f64>() / n,
                    fraction
                )
                .unwrap();
            }
        }
    }
    out.push_str("\n## Duplicate trajectories\n\nIdentity and fingerprints are excluded from duplicate grouping; complete biological frame/action/death records are retained. Equal endpoints alone are insufficient.\n\n| condition | runs | unique | duplicates beyond first | largest group |\n|---|---|---|---|---|\n");
    for d in &a.duplicates {
        writeln!(
            out,
            "| {} | {} | {} | {} | {} |",
            d.condition, d.runs, d.unique_trajectories, d.duplicate_runs, d.largest_group
        )
        .unwrap();
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn protection_analysis_hand_pairs_preserve_all_primary_estimates_and_full_seed_denominator() {
        let index = super::super::protection_archive::tests::complete_index();
        let mut rows = Vec::new();
        for c in &index.manifest.conditions {
            for (i, &seed) in index.manifest.seeds.iter().enumerate() {
                let difference = [-1.0, 0.0, 1.0, 2.0][i % 4];
                rows.push(Endpoint {
                    condition: c.id.clone(),
                    seed,
                    initial_food: 12.0,
                    thief_transferred: Some(if c.lab.policy == Policy::Selective {
                        4.0 + difference
                    } else {
                        4.0
                    }),
                    owner_ticks_alive: 32,
                    ..Default::default()
                });
            }
        }
        let a = analyze_endpoints(&index, rows.clone(), vec![]).unwrap();
        assert_eq!(a.estimates.iter().filter(|e| e.primary).count(), 64);
        let estimate = a
            .estimates
            .iter()
            .find(|e| e.primary && e.metric == "original_food_transferred")
            .unwrap();
        let s = estimate.summary.as_ref().unwrap();
        assert_eq!(
            (s.n, s.mean, s.positive, s.zero, s.negative),
            (40, 0.5, 20, 10, 10)
        );
        let input = index
            .manifest
            .seeds
            .iter()
            .enumerate()
            .map(|(i, &seed)| (seed, [-1.0, 0.0, 1.0, 2.0][i % 4]))
            .collect();
        let zero = index
            .manifest
            .seeds
            .iter()
            .map(|&seed| (seed, 0.0))
            .collect();
        assert_eq!(s, &paired_summary(&input, &zero).unwrap());
        let condition = rows
            .iter()
            .find(|e| e.condition.starts_with("single/p=selective/"))
            .unwrap()
            .condition
            .clone();
        rows.iter_mut()
            .find(|e| e.condition == condition && e.seed == 10001)
            .unwrap()
            .thief_transferred = None;
        let a = analyze_endpoints(&index, rows, vec![]).unwrap();
        let unavailable: Vec<_> = a
            .estimates
            .iter()
            .filter(|e| {
                e.primary && e.id.starts_with(&condition) && e.metric == "original_food_transferred"
            })
            .collect();
        assert_eq!(unavailable.len(), 2);
        for e in unavailable {
            assert_eq!(e.unavailable_seeds, [10001]);
            assert!(e.summary.is_none());
        }
        assert!(a
            .estimates
            .iter()
            .filter(|e| e.primary && e.metric == "owner_ticks_alive")
            .all(|e| e.summary.as_ref().unwrap().n == 40));
    }
    #[test]
    fn protection_source_fractions_use_original_deposit_and_keep_terminal_loss() {
        let r = super::super::protection_archive::tests::synthetic(
            LabConfig {
                fixture: Fixture::Mixed {
                    observed_first: true,
                },
                ..Default::default()
            },
            10001,
        );
        let e = summarize("mixed", &r);
        assert_eq!(e.initial_food, 12.0);
        assert_eq!(
            e.sources.iter().map(|s| s.initial).collect::<Vec<_>>(),
            [6.0, 6.0]
        );
        assert_eq!(e.tagged_terminal_loss, Some(12.0));
        assert_eq!(e.owner_ticks_alive, 32);
        assert!(!e.owner_alive);
        let mut r = r;
        let source = r
            .cohorts
            .as_mut()
            .unwrap()
            .cohorts
            .values_mut()
            .next()
            .unwrap();
        source.transferred = 3.0;
        source.lost_cached.values_mut().for_each(|q| *q = 3.0);
        // Gross handling is deliberately larger than the original deposit.
        r.frames[9].relocation.withdrawn = 24.0;
        r.frames[9].relocation.redeposited = 24.0;
        let e = summarize("mixed", &r);
        assert_eq!(e.sources[0].transferred_fraction, Some(0.5));
    }
    #[test]
    fn protection_saved_reanalysis_is_byte_identical_and_lists_lineage_failure() {
        use super::super::protection_archive::{
            load,
            tests::{complete_index, synthetic},
            write_new, Envelope,
        };
        let index = complete_index();
        let root =
            std::env::temp_dir().join(format!("protection-reanalysis-{}", std::process::id()));
        if root.exists() {
            std::fs::remove_dir_all(&root).unwrap();
        }
        std::fs::create_dir(&root).unwrap();
        std::fs::create_dir(root.join("raw")).unwrap();
        let affected = index
            .manifest
            .conditions
            .iter()
            .find(|c| c.panel == "single" && c.lab.policy == Policy::Off)
            .unwrap()
            .id
            .clone();
        for raw in &index.runs {
            let c = index
                .manifest
                .conditions
                .iter()
                .find(|c| c.id == raw.condition)
                .unwrap();
            let mut record = synthetic(c.lab.clone(), raw.seed);
            if raw.condition == affected && raw.seed == 10001 {
                record.cohorts = None;
                record.thief_transferred = None;
                record
                    .ledger_errors
                    .push("synthetic unavailable lineage".into());
            }
            let envelope = Envelope {
                schema: SCHEMA.into(),
                condition: raw.condition.clone(),
                seed: raw.seed,
                code_revision: index.code_revision.clone(),
                protocol_revision: index.protocol_revision.clone(),
                started_unix_ms: 123,
                elapsed_seconds: 0.1,
                timing_boundary: "world construction and completed biological steps; excludes file I/O and analysis".into(),
                record,
            };
            std::fs::write(root.join(&raw.path), serde_json::to_vec(&envelope).unwrap()).unwrap();
        }
        write_new(
            &root.join("index.json"),
            &serde_json::to_vec(&index).unwrap(),
        )
        .unwrap();
        let run = |out: &str| {
            super::super::protection::cli(&[
                "--analyze".into(),
                root.join("index.json").to_string_lossy().into_owned(),
                "--out".into(),
                root.join(out).to_string_lossy().into_owned(),
            ])
            .unwrap()
        };
        run("first");
        run("second");
        assert_eq!(
            std::fs::read(root.join("first/analysis.json")).unwrap(),
            std::fs::read(root.join("second/analysis.json")).unwrap()
        );
        assert_eq!(
            std::fs::read(root.join("first/results.md")).unwrap(),
            std::fs::read(root.join("second/results.md")).unwrap()
        );
        let report = std::fs::read_to_string(root.join("first/results.md")).unwrap();
        assert!(report.contains("unavailable"));
        assert!(report.contains("[10001]"));
        let (mut saved, records) = load(&root.join("index.json")).unwrap();
        saved.runs[0].seed = 10002;
        assert!(analyze(&saved, &records).is_err());
        std::fs::remove_dir_all(root).unwrap();
    }
}
