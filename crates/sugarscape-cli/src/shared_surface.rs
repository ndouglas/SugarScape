use crate::Failure;
use clap::{Args, Subcommand};
use std::io::Write;
use sugarscape_core::config::FieldError;
use sugarscape_core::shared_surface::{diagnose, report_integrity, DiagnosticReport};
#[derive(Debug, Args)]
pub struct SharedSurfaceArgs {
    #[command(subcommand)]
    pub mode: SharedSurfaceMode,
}
#[derive(Debug, Subcommand)]
pub enum SharedSurfaceMode {
    Diagnose,
}
pub fn run(args: SharedSurfaceArgs) -> Result<(), Failure> {
    match args.mode {
        SharedSurfaceMode::Diagnose => {
            let mut report = diagnose().map_err(|e| {
                Failure::Invalid(vec![FieldError::new("shared_surface", e.to_string())])
            })?;
            let stdout = std::io::stdout();
            emit(&mut report, &mut std::io::BufWriter::new(stdout.lock()))
        }
    }
}
fn emit<W: Write>(report: &mut DiagnosticReport, writer: &mut W) -> Result<(), Failure> {
    let integrity = report_integrity(report)
        .map_err(|e| Failure::Invalid(vec![FieldError::new("shared_surface", e.to_string())]))?;
    report.passed = integrity;
    if !integrity {
        for check in &mut report.checks {
            check.passed = false;
        }
    }
    serde_json::to_writer(&mut *writer, report).map_err(|e| Failure::Io(e.to_string()))?;
    writer
        .write_all(b"\n")
        .map_err(|e| Failure::Io(e.to_string()))?;
    writer.flush().map_err(|e| Failure::Io(e.to_string()))?;
    if integrity {
        Ok(())
    } else {
        Err(Failure::Invalid(vec![FieldError::new(
            "shared_surface",
            "diagnostic report failed current-payload validation",
        )]))
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn incomplete() -> DiagnosticReport {
        DiagnosticReport {
            version: "shared-surface-diagnostic-v1".into(),
            protocol_version: 1,
            supplied_structure: vec![],
            traces: vec![],
            primary: vec![],
            secondary: vec![],
            checks: vec![],
            passed: true,
        }
    }
    #[test]
    fn emission_recomputes_integrity_and_writes_failed_report_before_invalid() {
        let mut report = incomplete();
        let mut bytes = Vec::new();
        assert!(matches!(
            emit(&mut report, &mut bytes),
            Err(Failure::Invalid(_))
        ));
        let emitted: DiagnosticReport = serde_json::from_slice(&bytes).unwrap();
        assert!(!emitted.passed);
    }
    struct Broken {
        flush: bool,
    }
    impl Write for Broken {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            if self.flush {
                Ok(bytes.len())
            } else {
                Err(std::io::Error::other("write failed"))
            }
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Err(std::io::Error::other("flush failed"))
        }
    }
    #[test]
    fn emission_propagates_write_and_flush_errors() {
        for flush in [false, true] {
            assert!(matches!(
                emit(&mut incomplete(), &mut Broken { flush }),
                Err(Failure::Io(_))
            ));
        }
    }
}
