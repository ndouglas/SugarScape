use super::{
    manifest::{id, Manifest},
    report_types::*,
};
use std::collections::BTreeMap;
use sugarscape_core::minds::behavior_tree::state::Controller;
pub(super) const FAMILIES: [(Family, Controller); 4] = [
    (Family::Primary, Controller::ReactiveUtility),
    (Family::Secondary, Controller::TaskGoap),
    (Family::Ablation, Controller::UnguardedTree),
    (Family::LegacyReference, Controller::LegacyGoap),
];
pub(super) const METRICS: [Metric; 4] = [
    Metric::QuotaAttained,
    Metric::RestrictedCompletionTicks,
    Metric::GrossGathered,
    Metric::LivingTicks,
];
pub(super) fn label<T: serde::Serialize>(x: &T) -> String {
    serde_json::to_string(x)
        .unwrap()
        .trim_matches('"')
        .to_owned()
}
fn value(e: &Endpoint, m: Metric) -> f64 {
    match m {
        Metric::QuotaAttained => f64::from(e.quota_attained),
        Metric::RestrictedCompletionTicks => e.restricted_completion_ticks as f64,
        Metric::GrossGathered => e.gross_gathered,
        Metric::LivingTicks => e.living_ticks as f64,
    }
}
pub(super) fn rows<'a>(
    m: &Manifest,
    rows: &'a [Endpoint],
) -> Result<BTreeMap<(String, u64), &'a Endpoint>, String> {
    let mut indexed = BTreeMap::new();
    for e in rows {
        if !m.seeds.contains(&e.seed)
            || !m.conditions.iter().any(|c| c.id == e.condition)
            || indexed.insert((e.condition.clone(), e.seed), e).is_some()
        {
            return Err("unexpected/duplicate endpoint".into());
        }
        if !e.gross_gathered.is_finite()
            || e.gross_gathered < 0.0
            || e.living_ticks > 64
            || e.first_completion.is_some_and(|t| t == 0 || t > 64)
            || e.quota_attained != e.first_completion.is_some()
            || e.restricted_completion_ticks != e.first_completion.unwrap_or(64)
            || e.right_censored == e.quota_attained
            || e.unattained_reason.is_some() == e.quota_attained
        {
            return Err("invalid scalar endpoint".into());
        }
    }
    if indexed.len() != m.conditions.len() * m.seeds.len() {
        return Err("incomplete endpoint census".into());
    }
    Ok(indexed)
}
pub(super) fn summarize(m: &Manifest, entries: &[Endpoint]) -> Result<Vec<Estimate>, String> {
    let indexed = rows(m, entries)?;
    let mut out = vec![];
    for (family, other) in FAMILIES {
        for c in m
            .conditions
            .iter()
            .filter(|c| c.lab.controller == Controller::GuardedTree)
        {
            let mut b = c.lab.clone();
            b.controller = other;
            let bid = id(&b);
            for metric in METRICS {
                let a = m
                    .seeds
                    .iter()
                    .map(|s| {
                        Ok((
                            *s,
                            value(
                                indexed
                                    .get(&(c.id.clone(), *s))
                                    .ok_or("missing guarded partner")?,
                                metric,
                            ),
                        ))
                    })
                    .collect::<Result<BTreeMap<_, _>, String>>()?;
                let b = m
                    .seeds
                    .iter()
                    .map(|s| {
                        Ok((
                            *s,
                            value(
                                indexed
                                    .get(&(bid.clone(), *s))
                                    .ok_or("missing comparator partner")?,
                                metric,
                            ),
                        ))
                    })
                    .collect::<Result<BTreeMap<_, _>, String>>()?;
                out.push(Estimate {
                    id: format!("{}:{}:{}", label(&family), c.id, label(&metric)),
                    family,
                    metric,
                    denominator: m.seeds.len(),
                    summary: crate::stats::paired_summary(&a, &b)?,
                });
            }
        }
    }
    Ok(out)
}
use super::{
    archive::{hash, Archive, Outcome},
    io, validate,
};
use sugarscape_core::minds::behavior_tree::{records::Frame, EpisodeRecord, WorkCounters};
/// Physical task identity deliberately omits execution representation and RNG.
pub(super) fn physical(f: &Frame) -> Vec<u8> {
    serde_json::to_vec(&(
        f.tick,
        &f.actor,
        &f.cells,
        &f.receipt,
        f.task.quota,
        f.task.gross,
        f.task.first_completion,
        f.task.target,
        &f.task.failed_until,
        f.living_ticks,
        (
            f.external_added,
            f.external_removed,
            f.consumed,
            f.death_loss,
        ),
    ))
    .expect("typed physical frame")
}
pub(super) fn completion(
    m: &Manifest,
    entries: &[Endpoint],
) -> Result<Vec<CompletionSummary>, String> {
    let indexed = rows(m, entries)?;
    let mut out = vec![];
    for (family, other) in FAMILIES {
        for c in m
            .conditions
            .iter()
            .filter(|c| c.lab.controller == Controller::GuardedTree)
        {
            let mut b = c.lab.clone();
            b.controller = other;
            let bid = id(&b);
            let mut unavailable = vec![];
            let (mut aa, mut bb) = (BTreeMap::new(), BTreeMap::new());
            for seed in &m.seeds {
                let a = indexed[&(c.id.clone(), *seed)];
                let b = indexed[&(bid.clone(), *seed)];
                match (a.first_completion, b.first_completion) {
                    (Some(x), Some(y)) => {
                        aa.insert(*seed, x as f64);
                        bb.insert(*seed, y as f64);
                    }
                    _ => unavailable.push((
                        *seed,
                        format!(
                            "guarded={:?}; comparator={:?}",
                            a.unattained_reason, b.unattained_reason
                        ),
                    )),
                }
            }
            let summary = if unavailable.is_empty() {
                Some(crate::stats::paired_summary(&aa, &bb)?)
            } else {
                None
            };
            out.push(CompletionSummary {
                id: format!("{}:{}:unrestricted_completion_ticks", label(&family), c.id),
                denominator: m.seeds.len(),
                summary,
                unavailable,
            });
        }
    }
    Ok(out)
}
fn endpoint(condition: &str, r: &EpisodeRecord) -> Endpoint {
    Endpoint {
        condition: condition.into(),
        seed: r.seed,
        quota_attained: r.quota_attained,
        first_completion: r.first_completion,
        restricted_completion_ticks: r.restricted_completion_ticks,
        right_censored: r.right_censored,
        unattained_reason: r.unattained_reason.clone(),
        gross_gathered: r.gross_gathered,
        living_ticks: r.living_ticks,
        alive_at_horizon: r.alive_at_horizon,
    }
}
fn sum_work(frames: &[Frame]) -> Option<WorkCounters> {
    let mut total = WorkCounters::default();
    for f in frames.iter().skip(1).filter(|f| f.receipt.is_some()) {
        let w = f.work.as_ref()?;
        total.node_visits += w.node_visits;
        total.candidate_evaluations += w.candidate_evaluations;
        total.target_selections += w.target_selections;
        total.path_queries += w.path_queries;
        total.fallback_short += w.fallback_short;
        total.fallback_limit += w.fallback_limit;
        total.search_expansions = total
            .search_expansions
            .zip(w.search_expansions)
            .map(|(a, b)| a + b);
        if w.search_unavailable_reason.is_some() {
            total.search_unavailable_reason = w.search_unavailable_reason.clone()
        }
    }
    Some(total)
}
pub fn analyze(a: &Archive) -> Result<Analysis, String> {
    let m = super::manifest::manifest();
    if a.index.manifest != m || !a.index.completed {
        return Err("analysis requires complete canonical scientific archive".into());
    }
    let census = a.index.census.clone().ok_or("missing census")?;
    if census.complete != 3840
        || census.failed + census.invalid + census.partial + census.pending + census.unstarted > 0
        || a.attempts.len() != 3840
    {
        return Err("analysis requires every complete valid episode".into());
    }
    let mut records = BTreeMap::new();
    let mut endpoints = vec![];
    let mut duplicates: BTreeMap<String, Vec<(String, u64)>> = BTreeMap::new();
    let mut rng_identities = vec![];
    for attempt in &a.attempts {
        if attempt.torn_tail.is_some() {
            return Err("torn trajectory".into());
        }
        let receipt = attempt.outcome.as_ref().ok_or("pending attempt")?;
        let Outcome::Complete(r) = &receipt.outcome else {
            return Err("failed attempt".into());
        };
        let c = m
            .conditions
            .iter()
            .find(|c| c.id == attempt.start.condition)
            .ok_or("unknown condition")?;
        validate::validate_episode(r, &c.lab, attempt.start.seed)?;
        if r.frames != attempt.frames
            || records
                .insert((c.id.clone(), r.seed), (r, receipt.elapsed_seconds))
                .is_some()
        {
            return Err("duplicate or conflicting raw attempt".into());
        }
        endpoints.push(endpoint(&c.id, r));
        let signature = hash(&r.frames.iter().flat_map(physical).collect::<Vec<_>>());
        duplicates
            .entry(signature)
            .or_default()
            .push((c.id.clone(), r.seed));
        rng_identities.push((
            c.id.clone(),
            r.seed,
            r.frames.iter().map(|f| f.rng_state_json.clone()).collect(),
        ));
    }
    let estimates = summarize(&m, &endpoints)?;
    let completion = completion(&m, &endpoints)?;
    let mut cells = vec![];
    let mut matched = vec![];
    for c in &m.conditions {
        let mut cell = CellDiagnostics {
            condition: c.id.clone(),
            seeds: m.seeds.clone(),
            completed_ticks: vec![],
            survival_at_64: vec![],
            work: vec![],
            controller_seconds_total: vec![],
            task_active_calls: vec![],
            hold_calls: vec![],
            physical_signature: vec![],
            controller_seconds_per_invocation: vec![],
            episode_seconds: vec![],
        };
        for seed in &m.seeds {
            let (r, elapsed) = records[&(c.id.clone(), *seed)];
            cell.completed_ticks.push(r.completed_ticks);
            cell.survival_at_64.push(r.alive_at_horizon);
            cell.work.push(sum_work(&r.frames));
            let active: Vec<_> = r
                .frames
                .windows(2)
                .filter(|pair| pair[0].actor.is_some() && pair[0].task.first_completion.is_none())
                .map(|pair| pair[1].controller_seconds)
                .collect();
            cell.task_active_calls.push(active.len() as u64);
            cell.hold_calls.push(
                r.frames
                    .windows(2)
                    .filter(|pair| {
                        pair[0].actor.is_some() && pair[0].task.first_completion.is_some()
                    })
                    .count() as u64,
            );
            cell.controller_seconds_total.push(
                active
                    .iter()
                    .copied()
                    .collect::<Option<Vec<_>>>()
                    .map(|times| times.iter().sum()),
            );
            cell.controller_seconds_per_invocation.push(active);
            cell.episode_seconds.push(elapsed);
            cell.physical_signature.push(hash(
                &r.frames.iter().flat_map(physical).collect::<Vec<_>>(),
            ));
            if c.lab.controller == Controller::GuardedTree {
                let mut fsm = c.lab.clone();
                fsm.controller = Controller::MatchedFsm;
                let (s, _) = records[&(id(&fsm), *seed)];
                let equal_physical_task = r
                    .frames
                    .iter()
                    .zip(&s.frames)
                    .all(|(x, y)| physical(x) == physical(y));
                let equal_rng = r
                    .frames
                    .iter()
                    .zip(&s.frames)
                    .all(|(x, y)| x.rng_state_json == y.rng_state_json);
                let first_difference_tick = r
                    .frames
                    .iter()
                    .zip(&s.frames)
                    .find(|(x, y)| {
                        physical(x) != physical(y) || x.rng_state_json != y.rng_state_json
                    })
                    .map(|(x, _)| x.tick);
                matched.push(MatchedCheck {
                    stratum: c.id.clone(),
                    seed: *seed,
                    equal_physical_task,
                    equal_rng,
                    first_difference_tick,
                });
            }
        }
        cells.push(cell);
    }
    if matched.len() != 640
        || matched
            .iter()
            .any(|m| !m.equal_physical_task || !m.equal_rng)
    {
        return Err("guarded/FSM physical or RNG mismatch".into());
    }
    let mut alias: BTreeMap<Vec<String>, Vec<String>> = BTreeMap::new();
    for c in &cells {
        alias
            .entry(c.physical_signature.clone())
            .or_default()
            .push(c.condition.clone());
    }
    Ok(Analysis{schema:"minds-behavior-tree-measured-v1".into(),census,endpoints,estimates,matched_pairs:matched.len(),duplicate_groups:duplicates.into_values().filter(|v|v.len()>1).collect(),raw_frame_references:a.index.attempts.clone(),completion,cells,matched,aliases:alias.into_values().filter(|v|v.len()>1).map(|conditions|AliasGroup{conditions,reason:"identical complete physical/task trajectories across every paired seed; representation, RNG and timings excluded".into()}).collect(),timing_definition:"Per invocation: actual common controller wrapper including live expiry, excluding researcher observation, serialization and sink; unavailable on WASM, hold and dead ticks. Active denominator counts turns entered alive with quota pending, including a completing or dying action. Hold denominator counts turns entered alive with quota already complete. Per episode: constructor through last frame sink, including its encoding/sync, excluding outcome encoding/sync. Work totals include zero work on live holds; unavailable expansions remain null. Related food/lifetime endpoints and duplicate trajectories are not independent corroboration.".into(),rng_identities})
}
pub fn save(a: &Analysis, out: &std::path::Path) -> Result<(), String> {
    io::directory(out)?;
    io::json(&out.join("analysis.json"), a)?;
    let mut text=String::from("# Behavior-tree paired results\n\nAll declared pairs retained. Descriptive paired differences, not a claim of animal independence or an overall verdict.\n\n| Estimate | n | Mean | 95% interval | + / 0 / − |\n|---|---:|---:|---|---|\n");
    for e in &a.estimates {
        text.push_str(&format!(
            "| {} | {} | {} | {:?} | {} / {} / {} |\n",
            e.id,
            e.denominator,
            e.summary.mean,
            e.summary.ci95,
            e.summary.positive,
            e.summary.zero,
            e.summary.negative
        ));
    }
    text.push_str(&format!("\n{}\n\nUnrestricted completion, full per-cell distributions, alias groups, raw references and exact RNG identities are retained in analysis.json.\n",a.timing_definition));
    io::create(&out.join("results.md"), text.as_bytes())
}
