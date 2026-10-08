use crate::Failure;
use clap::{Args, Subcommand};
use std::io::Write;
use sugarscape_core::active_surface::{diagnose, report_integrity, DiagnosticReport, Error};
use sugarscape_core::config::FieldError;

#[derive(Debug, Args)]
pub struct ActiveSurfaceArgs {
    #[command(subcommand)]
    pub mode: ActiveSurfaceMode,
}
#[derive(Debug, Subcommand)]
pub enum ActiveSurfaceMode {
    Diagnose,
}

pub fn run(args: ActiveSurfaceArgs) -> Result<(), Failure> {
    match args.mode {
        ActiveSurfaceMode::Diagnose => {
            let mut report = diagnose().map_err(invalid)?;
            let stdout = std::io::stdout();
            emit(&mut report, &mut std::io::BufWriter::new(stdout.lock()))
        }
    }
}
fn invalid(error: Error) -> Failure {
    Failure::Invalid(vec![FieldError::new("active_surface", error.to_string())])
}
fn emit<W: Write>(report: &mut DiagnosticReport, writer: &mut W) -> Result<(), Failure> {
    // Production always uses the full current-payload validator. The private
    // callback seam exercises bounded writer behavior only in unit tests.
    emit_checked(report, writer, report_integrity)
}
fn emit_checked<W: Write>(
    report: &mut DiagnosticReport,
    writer: &mut W,
    validate: impl FnOnce(&DiagnosticReport) -> Result<bool, Error>,
) -> Result<(), Failure> {
    report.passed = validate(report).map_err(invalid)?;
    if !report.passed {
        for check in &mut report.checks {
            check.passed = false;
        }
    }
    serde_json::to_writer(&mut *writer, report).map_err(|e| Failure::Io(e.to_string()))?;
    writer
        .write_all(b"\n")
        .map_err(|e| Failure::Io(e.to_string()))?;
    writer.flush().map_err(|e| Failure::Io(e.to_string()))?;
    if report.passed {
        Ok(())
    } else {
        Err(Failure::Invalid(vec![FieldError::new(
            "active_surface",
            "diagnostic report failed current-payload validation",
        )]))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn incomplete() -> DiagnosticReport {
        DiagnosticReport {
            version: "active-surface-diagnostic-v1".into(),
            protocol_version: 1,
            supplied_structure: vec![],
            histories: vec![],
            panels: vec![],
            checks: vec![],
            passed: true,
        }
    }
    #[test]
    fn emission_recomputes_payload_integrity_and_retains_failed_json() {
        let mut report = incomplete();
        let mut bytes = Vec::new();
        assert!(matches!(
            emit(&mut report, &mut bytes),
            Err(crate::Failure::Invalid(_))
        ));
        let emitted: DiagnosticReport = serde_json::from_slice(&bytes).unwrap();
        assert!(!emitted.passed);
        assert_eq!(bytes.last(), Some(&b'\n'));
    }
    struct Broken {
        late_flush: bool,
    }
    impl std::io::Write for Broken {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            if self.late_flush {
                Ok(bytes.len())
            } else {
                Err(std::io::Error::other("write failed"))
            }
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Err(std::io::Error::other("late flush failed"))
        }
    }
    #[test]
    fn emission_propagates_write_and_late_flush_failure_as_io() {
        for late_flush in [false, true] {
            assert!(matches!(
                emit(&mut incomplete(), &mut Broken { late_flush }),
                Err(crate::Failure::Io(_))
            ));
        }
    }
    #[test]
    fn emission_calls_distinct_validator_outcomes_and_flushes_success() {
        for accepted in [false, true] {
            let mut called = 0;
            let mut flushed = false;
            struct Writer<'a>(&'a mut bool, Vec<u8>);
            impl std::io::Write for Writer<'_> {
                fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
                    self.1.extend_from_slice(bytes);
                    Ok(bytes.len())
                }
                fn flush(&mut self) -> std::io::Result<()> {
                    *self.0 = true;
                    Ok(())
                }
            }
            let mut writer = Writer(&mut flushed, Vec::new());
            let result = emit_checked(&mut incomplete(), &mut writer, |report| {
                called += 1;
                assert!(report.passed);
                Ok(accepted)
            });
            let value: DiagnosticReport = serde_json::from_slice(&writer.1).unwrap();
            assert_eq!(value.passed, accepted);
            assert_eq!(result.is_ok(), accepted);
            assert_eq!(called, 1);
            assert!(flushed);
        }
    }
    #[test]
    fn emission_propagates_operational_validator_error_without_writing_success() {
        let mut bytes = Vec::new();
        let result = emit_checked(&mut incomplete(), &mut bytes, |_| {
            Err(Error::ArithmeticOverflow)
        });
        assert!(matches!(result, Err(crate::Failure::Invalid(_))));
        assert!(bytes.is_empty());
    }
}
