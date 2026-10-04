//! Authenticate saved physical transactions without consuming controller randomness.
//!
//! Fingerprints and controller search counters remain provenance-bound diagnostics:
//! checked physical replay cannot authenticate policy decisions or RNG continuation.
use super::{
    observation::observe_measured,
    runner::{validate_options, Recording},
    view, *,
};

fn physical_snapshot(mut saved: Snapshot) -> Snapshot {
    saved.controller_bfs_calls = 0;
    saved.controller_bfs_visits = 0;
    saved.controller_bfs_peak_queue = 0;
    saved
}

/// Validate constructor setup, checked actions, material histories and sampled physical states.
pub fn validate_episode(record: &Episode, options: &RunOptions) -> Result<(), String> {
    let seed = record
        .seed
        .parse::<u64>()
        .map_err(|_| "seed: invalid decimal u64")?;
    if seed.to_string() != record.seed {
        return Err("seed: noncanonical decimal u64".into());
    }
    let mut world =
        World::new(record.config.clone(), seed).map_err(|e| format!("config: {e:?}"))?;
    validate_options(&world, options).map_err(|e| format!("options: {e:?}"))?;
    if world.setup != record.setup {
        return Err("setup: differs from constructor".into());
    }
    let choice = matches!(record.config.fixture, Fixture::Choice { .. });
    let start = world.tick;
    let workers = world.workers.len();
    let ticks = if choice { 0 } else { options.ticks };
    if record.requested_ticks != options.ticks
        || record.completed_ticks != ticks
        || record.stop_reason
            != if choice {
                "choice_selected"
            } else {
                "tick_budget_exhausted"
            }
        || record.events.len() as u64 != u64::from(ticks) * workers as u64
        || (choice && (options.ticks == 0 || record.choices.len() != 1))
    {
        return Err("horizon: invalid ticks, stop reason or event/choice count".into());
    }
    let cells = world.open.len() as u64;
    if record.choices.len() > record.events.len() + usize::from(choice) {
        return Err("choices: too many records".into());
    }
    let mut recording = Recording::new(&world);
    recording.ensure_exit_field(&world);
    let mut snapshots = vec![recording.snapshot(&world)];
    let mut ascii = vec![(start, view::ascii(&world))];
    let mut choice_index = 0;
    for round in 0..ticks {
        let mut seen = vec![false; workers];
        for offset in 0..workers {
            let index = round as usize * workers + offset;
            let saved = &record.events[index];
            world.tick = start + u64::from(round);
            if saved.tick != world.tick
                || saved.worker as usize >= workers
                || seen[saved.worker as usize]
            {
                return Err(format!("event {index}: invalid round clock or worker"));
            }
            seen[saved.worker as usize] = true;
            for pos in [
                Some(saved.from),
                Some(saved.to),
                match saved.action {
                    Action::Move(p) | Action::Dig(p) => Some(p),
                    _ => None,
                },
            ]
            .into_iter()
            .flatten()
            {
                if world.index(pos).is_none() {
                    return Err(format!("event {index}: coordinate out of bounds"));
                }
            }
            observe_opportunity(&world, saved.worker, &mut recording);
            if let Some(selected) = record.choices.get(choice_index) {
                if selected.tick < world.tick {
                    return Err(format!("choice {choice_index}: trace clock/order differs"));
                }
                if selected.tick == world.tick && selected.worker == saved.worker {
                    validate_choice(&world, selected, choice_index)?;
                    choice_index += 1;
                }
            }
            let distance = if let Action::Dig(pos) = saved.action {
                pos.neighbors(world.setup.width, world.setup.height)
                    .iter()
                    .filter_map(|&p| recording.exit_field[world.index(p).unwrap()])
                    .min()
                    .map(|d| d + 1)
            } else {
                None
            };
            let actual = world.apply(saved.worker, saved.action.clone());
            if actual != *saved {
                return Err(format!("event {index}: checked transaction differs"));
            }
            world
                .check_invariants()
                .map_err(|e| format!("event {index}: {e}"))?;
            recording.record(&actual, &world, distance);
            recording.ensure_exit_field(&world);
        }
        world.tick = start + u64::from(round) + 1;
        if (round + 1) % options.sample_every == 0 {
            snapshots.push(recording.snapshot(&world));
            ascii.push((world.tick, view::ascii(&world)));
        }
    }
    if choice {
        let selected = &record.choices[0];
        if selected.tick != start {
            return Err("choice 0: invalid start clock".into());
        }
        validate_choice(&world, selected, 0)?;
        observe_opportunity(&world, selected.worker, &mut recording);
        choice_index = 1;
        // Target assignment changes the fingerprint, but never the physical projection.
        ascii.push((start, view::ascii(&world)));
    }
    if choice_index != record.choices.len() {
        return Err(format!("choice {choice_index}: unmatched selection"));
    }
    let final_summary = recording.snapshot(&world);
    if snapshots.last() != Some(&final_summary) {
        snapshots.push(final_summary.clone());
    }
    if ascii.last().map(|(tick, _)| *tick) != Some(world.tick) {
        ascii.push((world.tick, view::ascii(&world)));
    }
    if physical_snapshot(record.final_summary.clone()) != final_summary {
        return Err("final_summary: physical totals differ".into());
    }
    if record.series.len() != snapshots.len() {
        return Err("series: sample count differs".into());
    }
    for (index, (saved, actual)) in record.series.iter().zip(&snapshots).enumerate() {
        if physical_snapshot(saved.clone()) != *actual {
            return Err(format!("series {index}: physical totals differ"));
        }
    }
    if record.frames.len() != ascii.len() {
        return Err("frames: sample count differs".into());
    }
    for (index, (saved, (tick, projection))) in record.frames.iter().zip(&ascii).enumerate() {
        if saved.tick != *tick || saved.ascii != *projection {
            return Err(format!("frame {index}: clock or ASCII differs"));
        }
        if saved.fingerprint.len() != 16
            || !saved
                .fingerprint
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
        {
            return Err(format!("frame {index}: malformed fingerprint"));
        }
    }
    if recording.ledger.finish(&world) != record.deliveries
        || recording.worker_work != record.worker_work
        || recording.spatial_work.values().cloned().collect::<Vec<_>>() != record.spatial_work
        || recording.travel != record.travel
        || recording.dig_distances != record.dig_distances
    {
        return Err("summaries: material, worker, spatial, travel or dig history differs".into());
    }
    let rates = Rates {
        excavation: (final_summary.opportunities > 0)
            .then(|| final_summary.digs as f64 / final_summary.opportunities as f64),
        disposal: (final_summary.opportunities > 0)
            .then(|| final_summary.disposals as f64 / final_summary.opportunities as f64),
    };
    if rates != record.rates {
        return Err("rates: differ from integer totals".into());
    }
    validate_diagnostics(record, cells)?;
    let s = &record.storage;
    let counts = [
        record.events.len() as u64,
        record.choices.len() as u64,
        record.frames.len() as u64,
        record.series.len() as u64,
        record.deliveries.len() as u64,
        record
            .deliveries
            .iter()
            .map(|d| d.carriers.len() as u64)
            .sum(),
        record.worker_work.len() as u64,
        record.spatial_work.len() as u64,
        record
            .spatial_work
            .iter()
            .map(|d| d.workers.len() as u64)
            .sum(),
        record.dig_distances.len() as u64,
    ];
    if counts
        != [
            s.events,
            s.choices,
            s.frames,
            s.snapshots,
            s.deliveries,
            s.carrier_ids,
            s.worker_work,
            s.spatial_work,
            s.spatial_worker_ids,
            s.dig_distances,
        ]
        || s.retained_records != counts.iter().sum::<u64>()
        || s.ascii_bytes
            != record
                .frames
                .iter()
                .map(|f| f.ascii.len() as u64)
                .sum::<u64>()
        || s.peak_ascii_bytes != s.ascii_bytes
        || s.peak_events != s.events
        || s.peak_choices != s.choices
        || s.peak_material_records != world.units.len() as u64
        || s.peak_exit_field_cells != cells
        || s.peak_observation_cells != recording.peak_observation_cells
        || s.peak_observation_frontier != recording.peak_observation_frontier
        || s.peak_open_cells != final_summary.connected_open
    {
        return Err("storage: collection sizes or peaks differ".into());
    }
    Ok(())
}

fn validate_choice(world: &World, selected: &ChoiceEvent, index: usize) -> Result<(), String> {
    if selected.worker as usize >= world.workers.len()
        || !observe_measured(world, selected.worker)
            .0
            .frontier
            .contains(&selected.target)
    {
        return Err(format!(
            "choice {index}: invalid worker or locally observed target"
        ));
    }
    Ok(())
}
fn observe_opportunity(world: &World, worker: u32, recording: &mut Recording) {
    let (observation, stats) = observe_measured(world, worker);
    recording.observation_stats.include(stats);
    recording.peak_observation_cells = recording
        .peak_observation_cells
        .max(observation.open.len() as u64);
    recording.peak_observation_frontier = recording
        .peak_observation_frontier
        .max(observation.frontier.len() as u64);
}
fn validate_diagnostics(record: &Episode, cells: u64) -> Result<(), String> {
    let mut prior = (0, 0, 0);
    for (index, s) in record
        .series
        .iter()
        .chain(std::iter::once(&record.final_summary))
        .enumerate()
    {
        let current = (
            s.controller_bfs_calls,
            s.controller_bfs_visits,
            s.controller_bfs_peak_queue,
        );
        if (index == 0 && current != (0, 0, 0))
            || current.1 < current.0
            || current.2 > current.1
            || current.0 < prior.0
            || current.1 < prior.1
            || current.2 < prior.2
            || current.2 > cells
            || (current.0 == 0 && (current.1 != 0 || current.2 != 0))
            || u128::from(current.1) > u128::from(current.0) * u128::from(cells)
        {
            return Err(format!(
                "series {index}: invalid controller BFS diagnostics"
            ));
        }
        prior = current;
    }
    if record.series.last() != Some(&record.final_summary) {
        return Err("series: terminal diagnostics differ".into());
    }
    Ok(())
}
