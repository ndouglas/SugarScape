//! `sugarscape`: run Sugarscape worlds and parameter sweeps from the command
//! line (milestone 5). Exit codes: 0 success, 1 I/O error, 2 usage or
//! validation error (printed as `field: message`, one per line).

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::{Args, Parser, Subcommand};
use sugarscape_core::config::FieldError;
use sugarscape_core::model::{ModelConfig, ModelKind, ModelWorld};
use sugarscape_core::presets;
use sugarscape_core::sweep::{self, Sweep};

#[derive(Debug, Parser)]
#[command(
    name = "sugarscape",
    version,
    about = "Run Sugarscape worlds and parameter sweeps"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// List the presets of every model (id, source, name).
    Presets,
    /// List the built-in sweeps (id, name).
    Sweeps,
    /// Run one world and write its statistics.
    Run(RunArgs),
    /// Run a parameter sweep.
    Sweep(SweepArgs),
}

#[derive(Debug, Args)]
#[group(required = true, multiple = false)]
struct ConfigSource {
    /// A preset id (see `sugarscape presets`).
    #[arg(long, value_name = "ID")]
    preset: Option<String>,
    /// A config JSON file of any model (a sugarscape config in the current or pre-N-goods shape).
    #[arg(long, value_name = "FILE")]
    config: Option<PathBuf>,
}

#[derive(Debug, Args)]
struct RunArgs {
    #[command(flatten)]
    source: ConfigSource,
    #[arg(long, default_value_t = 1)]
    seed: u64,
    #[arg(long, default_value_t = 1000)]
    ticks: u32,
    /// Write the statistics history (one row per tick).
    #[arg(long, value_name = "PATH")]
    series_csv: Option<PathBuf>,
    /// Write the agents alive at the end.
    #[arg(long, value_name = "PATH")]
    agents_csv: Option<PathBuf>,
    /// Write the config as loaded (normalized JSON).
    #[arg(long, value_name = "PATH")]
    config_out: Option<PathBuf>,
    /// Print the final world's fingerprint as 0x%016x.
    #[arg(long)]
    fingerprint: bool,
}

#[derive(Debug, Args)]
#[group(required = true, multiple = false)]
struct SweepSource {
    /// A sweep JSON file.
    #[arg(value_name = "FILE")]
    file: Option<PathBuf>,
    /// A built-in sweep id (see `sugarscape sweeps`).
    #[arg(long, value_name = "NAME")]
    builtin: Option<String>,
}

#[derive(Debug, Args)]
struct SweepArgs {
    #[command(flatten)]
    source: SweepSource,
    /// Worker threads (default: available parallelism).
    #[arg(long, value_parser = clap::value_parser!(u32).range(1..=1024))]
    jobs: Option<u32>,
    /// Override the sweep's seed count.
    #[arg(long, value_name = "N")]
    seeds: Option<u32>,
    /// Override the sweep's ticks (a window or block length must still fit).
    #[arg(long, value_name = "N")]
    ticks: Option<u32>,
    /// Write the result JSON here instead of stdout.
    #[arg(long, value_name = "PATH")]
    out: Option<PathBuf>,
    #[arg(long, value_name = "PATH")]
    runs_csv: Option<PathBuf>,
    #[arg(long, value_name = "PATH")]
    summary_csv: Option<PathBuf>,
    /// No progress on stderr.
    #[arg(long)]
    quiet: bool,
}

/// Why a command failed.
#[derive(Debug)]
enum Failure {
    /// Exit code 1.
    Io(String),
    /// Exit code 2.
    Invalid(Vec<FieldError>),
}

impl From<Vec<FieldError>> for Failure {
    fn from(errors: Vec<FieldError>) -> Self {
        Failure::Invalid(errors)
    }
}

fn read(path: &Path) -> Result<String, Failure> {
    std::fs::read_to_string(path)
        .map_err(|e| Failure::Io(format!("cannot read {}: {e}", path.display())))
}

fn write(path: &Path, text: &str) -> Result<(), Failure> {
    std::fs::write(path, text)
        .map_err(|e| Failure::Io(format!("cannot write {}: {e}", path.display())))
}

fn main() -> ExitCode {
    // Usage errors exit with 2 inside `parse`.
    match run(Cli::parse()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(Failure::Io(message)) => {
            eprintln!("error: {message}");
            ExitCode::from(1)
        }
        Err(Failure::Invalid(errors)) => {
            for e in errors {
                eprintln!("{}: {}", e.field, e.message);
            }
            ExitCode::from(2)
        }
    }
}

fn run(cli: Cli) -> Result<(), Failure> {
    match cli.command {
        Command::Presets => {
            for p in presets::catalog() {
                println!("{}\t{}\t{}", p.id, p.source, p.name);
            }
            Ok(())
        }
        Command::Sweeps => {
            for b in sweep::builtins() {
                println!("{}\t{}", b.id, Sweep::from_json(b.json)?.name);
            }
            Ok(())
        }
        Command::Run(args) => run_world(args),
        Command::Sweep(args) => run_sweep(args),
    }
}

fn run_world(args: RunArgs) -> Result<(), Failure> {
    let config = match (&args.source.preset, &args.source.config) {
        (Some(id), _) => presets::find(id).map(|p| p.config).ok_or_else(|| {
            Failure::Invalid(vec![FieldError::new(
                "preset",
                format!("unknown preset {id:?} (see `sugarscape presets`)"),
            )])
        })?,
        (None, Some(path)) => ModelConfig::from_json(&read(path)?)?,
        (None, None) => unreachable!("clap requires --preset or --config"),
    };
    let mut world = ModelWorld::new(config.clone(), args.seed)?;
    world.model_mut().run(args.ticks);
    let world = world.model();
    if world.finished() && world.tick() < u64::from(args.ticks) {
        // The anasazi stops at its end year; civil violence when a group is gone;
        // the tags model at its last generation; Axelrod's culture once stable,
        // and a sugarscape under his rule once its cultures settle.
        let why = match config.kind() {
            ModelKind::Civil => "a group has died out",
            ModelKind::Tags => "its last generation",
            ModelKind::Culture => "the lattice is stable",
            ModelKind::Sugarscape => "the cultures have settled",
            _ => "its end year",
        };
        eprintln!("finished at tick {} ({why})", world.tick());
    }
    if let Some(path) = &args.series_csv {
        write(path, &world.series_csv())?;
    }
    if let Some(path) = &args.agents_csv {
        write(path, &world.agents_csv())?;
    }
    if let Some(path) = &args.config_out {
        let json = serde_json::to_string_pretty(&config).expect("configs serialize");
        write(path, &(json + "\n"))?;
    }
    if args.fingerprint {
        println!("{:#018x}", world.fingerprint());
    }
    Ok(())
}

fn run_sweep(args: SweepArgs) -> Result<(), Failure> {
    let mut sweep = match (&args.source.file, &args.source.builtin) {
        (Some(path), _) => Sweep::from_json(&read(path)?)?,
        (None, Some(id)) => sweep::builtin(id).ok_or_else(|| {
            Failure::Invalid(vec![FieldError::new(
                "builtin",
                format!("unknown sweep {id:?} (see `sugarscape sweeps`)"),
            )])
        })?,
        (None, None) => unreachable!("clap requires a file or --builtin"),
    };
    if let Some(n) = args.seeds {
        sweep.seeds.count = n;
    }
    if let Some(t) = args.ticks {
        sweep.ticks = t;
    }
    let jobs = args.jobs.map_or_else(
        || std::thread::available_parallelism().map_or(1, |n| n.get()),
        |j| j as usize,
    );
    let total = sweep.point_count();
    let result = sweep::run_all(&sweep, jobs, |done, point| {
        if !args.quiet {
            eprintln!(
                "[{done}/{total}] series={} x={} seed={}",
                sweep.series_name(point.series),
                sweep.x.values[point.x].at,
                point.seed
            );
        }
    })?;
    let json = result.to_json();
    match &args.out {
        Some(path) => write(path, &json)?,
        None => print!("{json}"),
    }
    if let Some(path) = &args.runs_csv {
        write(path, &sweep::runs_csv(&result))?;
    }
    if let Some(path) = &args.summary_csv {
        write(path, &sweep::summary_csv(&result))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn the_command_line_is_well_formed() {
        Cli::command().debug_assert();
    }

    #[test]
    fn run_has_defaults_and_needs_exactly_one_source() {
        let cli = Cli::try_parse_from(["sugarscape", "run", "--preset", "ii-2-unit"]).unwrap();
        let Command::Run(args) = cli.command else {
            panic!("run expected");
        };
        assert_eq!((args.seed, args.ticks, args.fingerprint), (1, 1000, false));
        assert!(Cli::try_parse_from(["sugarscape", "run"]).is_err());
        assert!(
            Cli::try_parse_from(["sugarscape", "run", "--preset", "a", "--config", "b.json"])
                .is_err()
        );
    }

    #[test]
    fn sweep_takes_a_file_or_a_builtin() {
        assert!(Cli::try_parse_from(["sugarscape", "sweep", "s.json"]).is_ok());
        assert!(Cli::try_parse_from(["sugarscape", "sweep", "--builtin", "fig-ii-5"]).is_ok());
        assert!(Cli::try_parse_from(["sugarscape", "sweep"]).is_err());
        assert!(
            Cli::try_parse_from(["sugarscape", "sweep", "s.json", "--builtin", "fig-ii-5"])
                .is_err()
        );
        assert!(Cli::try_parse_from(["sugarscape", "sweep", "s.json", "--jobs", "0"]).is_err());
    }
}
