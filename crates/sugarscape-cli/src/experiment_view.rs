//! Bounded display exports, with reconstruction delegated to the original adapters.
use crate::Failure;
use clap::{Args, Subcommand};
use std::{
    io::{Read, Write},
    path::{Path, PathBuf},
};
use sugarscape_core::browser_experiments as core;

#[derive(Debug, Args)]
pub struct ExperimentViewArgs {
    #[command(subcommand)]
    command: Mode,
}
#[derive(Debug, Subcommand)]
enum Mode {
    Catalog,
    Recorded,
    Run {
        #[arg(long, value_name = "FILE")]
        input: PathBuf,
    },
    Validate {
        #[arg(long, value_name = "FILE")]
        input: PathBuf,
    },
}
fn read_bounded(path: &Path, limit: usize, field: &str) -> Result<String, Failure> {
    let file = std::fs::File::open(path)
        .map_err(|error| Failure::Io(format!("cannot read {}: {error}", path.display())))?;
    let mut bytes = Vec::new();
    file.take((limit + 1) as u64)
        .read_to_end(&mut bytes)
        .map_err(|error| Failure::Io(format!("cannot read {}: {error}", path.display())))?;
    if bytes.len() > limit {
        return Err(vec![core::FieldError::new(
            field,
            format!("raw input exceeds {limit} bytes"),
        )]
        .into());
    }
    String::from_utf8(bytes)
        .map_err(|error| Failure::Io(format!("cannot read {} as UTF-8: {error}", path.display())))
}
fn write_json(writer: &mut impl Write, text: &str) -> Result<(), Failure> {
    writer
        .write_all(text.as_bytes())
        .and_then(|()| writer.write_all(b"\n"))
        .and_then(|()| writer.flush())
        .map_err(|error| Failure::Io(format!("experiment-view output: {error}")))
}
pub fn run(args: ExperimentViewArgs) -> Result<(), Failure> {
    let text = match args.command {
        Mode::Catalog => serde_json::to_string(&core::catalog()).map_err(|error| {
            Failure::Invalid(vec![core::FieldError::new("catalog", error.to_string())])
        })?,
        Mode::Recorded => core::recorded_results_json()?,
        Mode::Run { input } => {
            let input =
                core::normalize_input(&read_bounded(&input, core::MAX_INPUT_BYTES, "input")?)?;
            core::episode_json(&core::run(&input)?)?
        }
        Mode::Validate { input } => {
            let record =
                core::validate_episode(&read_bounded(&input, core::MAX_EPISODE_BYTES, "episode")?)?;
            core::episode_json(&record)?
        }
    };
    write_json(&mut std::io::stdout().lock(), &text)
}
#[cfg(test)]
mod tests {
    use super::*;
    struct FlushFailure;
    impl Write for FlushFailure {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            Ok(bytes.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Err(std::io::Error::other("flush failed"))
        }
    }
    #[test]
    fn output_flush_failure_is_operational() {
        assert!(
            matches!(write_json(&mut FlushFailure, "{}"), Err(Failure::Io(message)) if message.contains("flush failed"))
        );
    }
    #[test]
    fn output_adds_exactly_one_newline() {
        let mut bytes = Vec::new();
        write_json(&mut bytes, "{}").unwrap();
        assert_eq!(bytes, b"{}\n");
    }
}
