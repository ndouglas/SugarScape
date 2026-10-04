//! Thin checked file boundary for structural resource-access records.

use std::path::PathBuf;

use clap::Args;
use serde::Serialize;
use sugarscape_core::burrow::{
    run_access_episode, AccessConfig, AccessObjective, AccessSummary, RunOptions, Snapshot,
};
use sugarscape_core::config::FieldError;

use crate::burrow::{prepare_directory, write_export_file};
use crate::{read, Failure};

#[derive(Args, Debug)]
pub struct BurrowAccessArgs {
    #[arg(long)]
    pub config: PathBuf,
    #[arg(long, default_value_t = 7)]
    pub seed: u64,
    #[arg(long, default_value_t = 512)]
    pub ticks: u32,
    #[arg(long, default_value_t = 32)]
    pub sample_every: u32,
    #[arg(long)]
    pub out: PathBuf,
}

#[derive(Serialize)]
struct AccessExportSummary<'a> {
    base: &'a Snapshot,
    access: &'a AccessSummary,
}

pub fn run(args: BurrowAccessArgs) -> Result<(), Failure> {
    let config: AccessConfig = serde_json::from_str(&read(&args.config)?).map_err(|error| {
        Failure::Invalid(vec![FieldError::new(
            "burrow_access_config",
            error.to_string(),
        )])
    })?;
    let record = run_access_episode(
        config,
        args.seed,
        RunOptions {
            ticks: args.ticks,
            sample_every: args.sample_every,
        },
    )?;
    prepare_directory(&args.out)?;
    let task = &record.config.task;
    let objective = match task.objective {
        AccessObjective::Explore => "explore",
        AccessObjective::KnownGoal => "known_goal",
    };
    let heading = format!(
        "task goal=({},{}) objective={} goal_weight={} structural_access={} deadline_censored={} final_exit_distance={}\n",
        task.goal.x, task.goal.y, objective, task.goal_weight,
        record.access.structurally_accessible, record.access.deadline_censored,
        record.access.final_exit_distance.map_or("null".into(), |distance| distance.to_string())
    );
    let maps = heading.clone()
        + &record
            .episode
            .frames
            .iter()
            .map(|frame| {
                format!(
                    "tick {} fingerprint {}\n{}\n",
                    frame.tick, frame.fingerprint, frame.ascii
                )
            })
            .collect::<String>();
    let summary = AccessExportSummary {
        base: &record.episode.final_summary,
        access: &record.access,
    };
    let files = [
        (
            "config.json",
            serde_json::to_string_pretty(&record.config).expect("config serializes") + "\n",
        ),
        (
            "episode.json",
            serde_json::to_string_pretty(&record).expect("access episode serializes") + "\n",
        ),
        ("maps.txt", maps),
        (
            "summary.json",
            serde_json::to_string_pretty(&summary).expect("summary serializes") + "\n",
        ),
    ];
    for (name, contents) in files {
        write_export_file(&args.out, name, &contents)?;
    }
    print!(
        "{}{}",
        record
            .episode
            .frames
            .last()
            .expect("initial frame retained")
            .ascii,
        heading
    );
    let base = &record.episode.final_summary;
    println!(
        "tick={} completed_ticks={} opportunities={} digs={} disposed={} carried={} loose={} blocked={} waits={}",
        base.tick, record.episode.completed_ticks, base.opportunities, base.digs,
        base.inventory.disposed, base.inventory.carried, base.inventory.loose, base.blocked, base.waits
    );
    Ok(())
}
