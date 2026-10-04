//! Checked file boundary around the shared standalone excavation replay.

use std::path::{Path, PathBuf};

use clap::Args;
use sugarscape_core::burrow::{run_episode, LabConfig, RunOptions};
use sugarscape_core::config::FieldError;

use crate::{read, Failure};

#[derive(Args, Debug)]
pub struct BurrowArgs {
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

pub(crate) fn prepare_directory(path: &Path) -> Result<(), Failure> {
    if path.exists() {
        if !path.is_dir() {
            return Err(Failure::Io(format!(
                "output directory {} is not a directory",
                path.display()
            )));
        }
        let mut entries = std::fs::read_dir(path).map_err(|error| {
            Failure::Io(format!(
                "cannot inspect output directory {}: {error}",
                path.display()
            ))
        })?;
        if let Some(entry) = entries.next() {
            entry.map_err(|error| {
                Failure::Io(format!(
                    "cannot inspect output directory {}: {error}",
                    path.display()
                ))
            })?;
            return Err(Failure::Io(format!(
                "output directory {} is not empty",
                path.display()
            )));
        }
    } else {
        std::fs::create_dir_all(path).map_err(|error| {
            Failure::Io(format!(
                "cannot create output directory {}: {error}",
                path.display()
            ))
        })?;
    }
    Ok(())
}

pub(crate) fn write_export_file(out: &Path, name: &str, contents: &str) -> Result<(), Failure> {
    // Exclusive creation also prevents overwriting an entry created after the emptiness check.
    use std::io::Write;
    let path = out.join(name);
    let result = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
        .and_then(|mut file| file.write_all(contents.as_bytes()));
    result.map_err(|error| {
        Failure::Io(format!(
            "cannot write {}: {error}; output directory {} may contain partial output",
            path.display(),
            out.display()
        ))
    })
}

pub fn run(args: BurrowArgs) -> Result<(), Failure> {
    let config: LabConfig = serde_json::from_str(&read(&args.config)?).map_err(|error| {
        Failure::Invalid(vec![FieldError::new("burrow_config", error.to_string())])
    })?;
    let episode = run_episode(
        config,
        args.seed,
        RunOptions {
            ticks: args.ticks,
            sample_every: args.sample_every,
        },
    )?;
    prepare_directory(&args.out)?;
    let maps = episode
        .frames
        .iter()
        .map(|frame| {
            format!(
                "tick {} fingerprint {}\n{}\n",
                frame.tick, frame.fingerprint, frame.ascii
            )
        })
        .collect::<String>();
    let files = [
        (
            "config.json",
            serde_json::to_string_pretty(&episode.config).expect("config serializes") + "\n",
        ),
        (
            "episode.json",
            serde_json::to_string_pretty(&episode).expect("episode serializes") + "\n",
        ),
        ("maps.txt", maps),
        (
            "summary.json",
            serde_json::to_string_pretty(&episode.final_summary).expect("summary serializes")
                + "\n",
        ),
    ];
    for (name, contents) in files {
        write_export_file(&args.out, name, &contents)?;
    }
    print!(
        "{}",
        episode.frames.last().expect("initial frame retained").ascii
    );
    let summary = &episode.final_summary;
    let inventory = &summary.inventory;
    println!(
        "tick={} completed_ticks={} opportunities={} digs={} disposed={} carried={} loose={} blocked={} waits={}",
        summary.tick, episode.completed_ticks, summary.opportunities, summary.digs,
        inventory.disposed, inventory.carried, inventory.loose, summary.blocked, summary.waits
    );
    Ok(())
}

#[cfg(test)]
mod export_tests {
    use super::*;

    #[test]
    fn entry_created_after_directory_check_is_preserved_and_reports_partial_path() {
        let out = std::env::temp_dir().join(format!("burrow-exclusive-{}", std::process::id()));
        std::fs::create_dir_all(&out).unwrap();
        prepare_directory(&out).unwrap();
        let file = out.join("episode.json");
        std::fs::write(&file, "keep").unwrap();
        let error = write_export_file(&out, "episode.json", "replace").unwrap_err();
        let message = match error {
            Failure::Io(message) => message,
            _ => panic!("expected I/O error"),
        };
        assert!(message.contains(file.to_str().unwrap()));
        assert!(message.contains(&format!(
            "output directory {} may contain partial output",
            out.display()
        )));
        assert_eq!(std::fs::read_to_string(file).unwrap(), "keep");
        std::fs::remove_dir_all(out).unwrap();
    }
}
