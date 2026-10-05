//! Local JSON host. Session files are privileged; play output is one seat only.
use crate::{read, write, Failure};
use clap::{Args, Subcommand};
use serde_json::{json, Value};
use std::io::{BufRead, Write};
use std::path::PathBuf;
use sugarscape_core::config::FieldError;
use sugarscape_core::deduction::*;

const SEED_DERIVATION: &str = "policy_seed-v1";
#[derive(Debug, Args)]
pub struct DeductionArgs {
    #[command(subcommand)]
    command: Mode,
}
#[derive(Debug, Subcommand)]
enum Mode {
    Run(GameArgs),
    Testimony,
    TestimonyGame,
    StrategicReporting,
    Play(PlayArgs),
    Diagnose {
        #[arg(long)]
        out: Option<PathBuf>,
    },
    Replay {
        file: PathBuf,
    },
}
#[derive(Debug, Args)]
struct GameArgs {
    #[arg(long)]
    scenario: Option<String>,
    #[arg(long)]
    seed: Option<u64>,
    #[arg(long)]
    policy: Option<String>,
}
#[derive(Debug, Args)]
struct PlayArgs {
    #[command(flatten)]
    game: GameArgs,
    #[arg(long)]
    agent: Option<u16>,
    #[arg(long)]
    archive: Option<PathBuf>,
    #[arg(long)]
    resume: Option<PathBuf>,
}
fn invalid(message: &str) -> Failure {
    Failure::Invalid(vec![FieldError::new("deduction", message)])
}
fn io_error(error: std::io::Error) -> Failure {
    Failure::Io(format!("deduction I/O: {error}"))
}
fn policy(name: &str) -> Result<PolicyKind, Failure> {
    match name {
        "evidence" => Ok(PolicyKind::Evidence),
        "random" => Ok(PolicyKind::Random),
        "reckless" => Ok(PolicyKind::Reckless),
        "passive" => Ok(PolicyKind::Passive),
        _ => Err(invalid("unknown policy")),
    }
}
fn controllers(archive: &ReplayArchive, kind: PolicyKind) -> Vec<BuiltinController> {
    (0..archive.config.agents.len())
        .map(|id| BuiltinController::new(kind, policy_seed(archive.seed, kind, id as u16)))
        .collect()
}
struct Session {
    engine: Engine,
    controllers: Vec<BuiltinController>,
    selected: Option<u16>,
    kind: PolicyKind,
}
impl Session {
    fn new(game: &GameArgs, selected: Option<u16>) -> Result<Self, Failure> {
        if game.scenario.as_deref().unwrap_or("wink") != "wink" {
            return Err(invalid("unknown scenario"));
        }
        let kind = policy(game.policy.as_deref().unwrap_or("evidence"))?;
        let engine = Engine::new(wink_config(6), game.seed.unwrap_or(7))?;
        let controllers = controllers(&engine.archive(), kind);
        let session = Self {
            engine,
            controllers,
            selected,
            kind,
        };
        session.validate_seat()?;
        Ok(session)
    }
    fn validate_seat(&self) -> Result<(), Failure> {
        if self
            .selected
            .is_some_and(|id| usize::from(id) >= self.controllers.len())
        {
            Err(invalid("selected Agent is unavailable"))
        } else {
            Ok(())
        }
    }
    fn save(&self, path: &std::path::Path) -> Result<(), Failure> {
        let value = json!({"host_version":1,"scenario":"wink","selected_agent":self.selected,"policy":self.kind,"policy_seed_derivation":SEED_DERIVATION,"archive":self.engine.archive()});
        write(path, &(value.to_string() + "\n"))
    }
    fn restore(path: &std::path::Path, args: &PlayArgs) -> Result<Self, Failure> {
        let value: Value =
            serde_json::from_str(&read(path)?).map_err(|_| invalid("invalid session archive"))?;
        let fields = [
            "host_version",
            "scenario",
            "selected_agent",
            "policy",
            "policy_seed_derivation",
            "archive",
        ];
        let object = value
            .as_object()
            .ok_or_else(|| invalid("invalid session archive"))?;
        if object.len() != fields.len()
            || fields.iter().any(|key| !object.contains_key(*key))
            || value["host_version"] != 1
            || value["scenario"] != "wink"
            || value["policy_seed_derivation"] != SEED_DERIVATION
        {
            return Err(invalid("invalid session metadata"));
        }
        let selected = value["selected_agent"]
            .as_u64()
            .and_then(|id| u16::try_from(id).ok())
            .ok_or_else(|| invalid("invalid selected Agent"))?;
        let kind: PolicyKind = serde_json::from_value(value["policy"].clone())
            .map_err(|_| invalid("invalid session policy"))?;
        let archive: ReplayArchive = serde_json::from_value(value["archive"].clone())
            .map_err(|_| invalid("invalid session archive"))?;
        if archive.config != wink_config(6) {
            return Err(invalid("session scenario mismatch"));
        }
        if args.agent.is_some_and(|id| id != selected)
            || args.game.seed.is_some_and(|seed| seed != archive.seed)
            || args.game.scenario.as_deref().is_some_and(|s| s != "wink")
            || args
                .game
                .policy
                .as_deref()
                .map(policy)
                .transpose()?
                .is_some_and(|p| p != kind)
        {
            return Err(invalid("incompatible resume override"));
        }
        replay(&archive).map_err(|_| invalid("invalid session replay"))?;
        let mut session = Self {
            engine: Engine::new(archive.config.clone(), archive.seed)?,
            controllers: controllers(&archive, kind),
            selected: Some(selected),
            kind,
        };
        session.validate_seat()?;
        for response in &archive.responses {
            let request = session
                .engine
                .request()
                .ok_or_else(|| invalid("invalid session transcript"))?;
            if request.actor != selected
                && session.controllers[usize::from(request.actor)].respond(&request) != *response
            {
                return Err(invalid("session controller transcript mismatch"));
            }
            session
                .engine
                .submit(response.clone())
                .map_err(|_| invalid("invalid session transcript"))?;
        }
        Ok(session)
    }
}
fn emit(value: &Value, output: &mut impl Write) -> Result<(), Failure> {
    writeln!(output, "{value}").map_err(io_error)?;
    output.flush().map_err(io_error)
}
fn read_response(input: &mut impl BufRead) -> Result<Option<TurnResponse>, Failure> {
    let mut line = String::new();
    if input.read_line(&mut line).map_err(io_error)? == 0 {
        return Ok(None);
    }
    serde_json::from_str(&line)
        .map(Some)
        .map_err(|_| invalid("invalid response"))
}
fn drive(
    session: &mut Session,
    input: &mut impl BufRead,
    output: &mut impl Write,
    errors: &mut impl Write,
) -> Result<(), Failure> {
    while let Some(request) = session.engine.request() {
        if session.selected == Some(request.actor) {
            emit(
                &serde_json::to_value(&request).expect("request serializes"),
                output,
            )?;
            match read_response(input) {
                Ok(None) => return Ok(()),
                Ok(Some(response)) => {
                    if session.engine.submit(response).is_err() {
                        writeln!(errors, "invalid response").map_err(io_error)?;
                    }
                }
                Err(Failure::Invalid(_)) => {
                    writeln!(errors, "invalid response").map_err(io_error)?;
                }
                Err(error) => return Err(error),
            }
        } else {
            if session.selected.is_some_and(|id| {
                request.observation.roster[usize::from(id)].status == Status::Inactive
            }) {
                emit(
                    &json!({"status":"paused","reason":"selected_agent_inactive","actor":session.selected}),
                    output,
                )?;
                return Ok(());
            }
            let response = session.controllers[usize::from(request.actor)].respond(&request);
            session
                .engine
                .submit(response)
                .map_err(|_| invalid("built-in controller rejected"))?;
        }
    }
    if session.selected.is_some() {
        emit(
            &json!({"status":"finished","outcome":session.engine.outcome()}),
            output,
        )
    } else {
        emit(
            &json!({"outcome":session.engine.outcome(),"fingerprint":session.engine.fingerprint()}),
            output,
        )
    }
}
pub fn run(args: DeductionArgs) -> Result<(), Failure> {
    let stdin = std::io::stdin();
    let stdout = std::io::stdout();
    let stderr = std::io::stderr();
    let mut input = stdin.lock();
    let mut output = stdout.lock();
    let mut errors = stderr.lock();
    match args.command {
        Mode::Run(game) => drive(
            &mut Session::new(&game, None)?,
            &mut input,
            &mut output,
            &mut errors,
        ),
        Mode::Play(args) => {
            let mut session = match &args.resume {
                Some(path) => Session::restore(path, &args)?,
                None => Session::new(
                    &args.game,
                    Some(args.agent.ok_or_else(|| invalid("--agent is required"))?),
                )?,
            };
            drive(&mut session, &mut input, &mut output, &mut errors)?;
            if let Some(path) = args.archive {
                session.save(&path)?;
            }
            Ok(())
        }
        Mode::Testimony => testimony(&mut output),
        Mode::TestimonyGame => emit_testimony_game(
            testimony_game::diagnose_testimony_game().map_err(|e| invalid(&e.to_string()))?,
            &mut output,
        ),
        Mode::StrategicReporting => emit_strategic_reporting(
            strategic_reporting::diagnose().map_err(|e| invalid(&e.to_string()))?,
            &mut output,
        ),
        Mode::Diagnose { out } => {
            let value = serde_json::to_value(diagnose()).expect("report serializes");
            match out {
                Some(path) => write(&path, &(value.to_string() + "\n")),
                None => emit(&value, &mut output),
            }
        }
        Mode::Replay { file } => {
            let value: Value = serde_json::from_str(&read(&file)?)
                .map_err(|_| invalid("invalid replay archive"))?;
            // A saved play session uses the same strict validation and controller reconstruction.
            let engine = if value.get("host_version").is_some() {
                Session::restore(
                    &file,
                    &PlayArgs {
                        game: GameArgs {
                            scenario: None,
                            seed: None,
                            policy: None,
                        },
                        agent: None,
                        archive: None,
                        resume: None,
                    },
                )?
                .engine
            } else {
                let archive: ReplayArchive =
                    serde_json::from_value(value).map_err(|_| invalid("invalid replay archive"))?;
                replay(&archive).map_err(|_| invalid("invalid replay archive"))?
            };
            emit(
                &json!({"outcome":engine.outcome(),"fingerprint":engine.fingerprint(),"status":if engine.outcome().is_some(){"finished"}else{"paused"}}),
                &mut output,
            )
        }
    }
}

fn testimony(output: &mut impl Write) -> Result<(), Failure> {
    let report = diagnose_testimony().map_err(|e| invalid(&e.to_string()))?;
    emit_testimony(report, output)
}
fn emit_testimony(report: TestimonyReport, output: &mut impl Write) -> Result<(), Failure> {
    let passed = report.passed;
    let value = serde_json::to_value(report).expect("finite report serializes");
    emit(&value, output)?;
    if passed {
        Ok(())
    } else {
        Err(invalid("testimony reference checks failed"))
    }
}
#[cfg(test)]
mod testimony_tests {
    use super::*;
    struct BrokenWrite {
        flush: bool,
    }
    impl Write for BrokenWrite {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            if self.flush {
                Ok(bytes.len())
            } else {
                Err(std::io::Error::other("deliberate write failure"))
            }
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Err(std::io::Error::other("deliberate flush failure"))
        }
    }
    #[test]
    fn testimony_propagates_write_and_flush_failures() {
        for flush in [false, true] {
            assert!(matches!(
                testimony(&mut BrokenWrite { flush }),
                Err(Failure::Io(_))
            ));
        }
    }
    #[test]
    fn testimony_emits_failed_checks_before_invalid_result() {
        let mut report = diagnose_testimony().unwrap();
        report.fixtures[0].references[0].passed = false;
        report.fixtures[0].passed = false;
        report.passed = false;
        let mut output = Vec::new();
        assert!(matches!(
            emit_testimony(report, &mut output),
            Err(Failure::Invalid(_))
        ));
        let json: Value = serde_json::from_slice(&output).unwrap();
        assert_eq!(json["passed"], false);
        assert_eq!(json["fixtures"][0]["references"][0]["passed"], false);
    }
}

fn emit_testimony_game(
    report: testimony_game::GameReport,
    output: &mut impl Write,
) -> Result<(), Failure> {
    let passed = report.passed;
    let value =
        serde_json::to_value(report).map_err(|_| invalid("invalid testimony-game report"))?;
    emit(&value, output)?;
    if passed {
        Ok(())
    } else {
        Err(invalid("testimony-game correctness checks failed"))
    }
}

#[cfg(test)]
mod testimony_game_tests {
    use super::*;
    #[test]
    fn output_helper_emits_failure_and_propagates_io() {
        let mut report = testimony_game::diagnose_testimony_game().unwrap();
        report.checks[0].actual = report.checks[0].expected + 1.0;
        report.checks[0].error = 1.0;
        report.checks[0].passed = false;
        report.passed = false;
        let mut output = Vec::new();
        assert!(matches!(
            emit_testimony_game(report.clone(), &mut output),
            Err(Failure::Invalid(_))
        ));
        let value: Value = serde_json::from_slice(&output).unwrap();
        assert_eq!(value["checks"][0]["passed"], false);
        struct Broken(bool);
        impl Write for Broken {
            fn write(&mut self, b: &[u8]) -> std::io::Result<usize> {
                if self.0 {
                    Ok(b.len())
                } else {
                    Err(std::io::Error::other("deliberate write failure"))
                }
            }
            fn flush(&mut self) -> std::io::Result<()> {
                Err(std::io::Error::other("deliberate flush failure"))
            }
        }
        for flush in [false, true] {
            assert!(matches!(
                emit_testimony_game(report.clone(), &mut Broken(flush)),
                Err(Failure::Io(_))
            ));
        }
    }
}

#[cfg(test)]
mod strategic_reporting_tests {
    use super::*;
    fn test_report(
        checks: Vec<strategic_reporting::DiagnosticCheck>,
    ) -> strategic_reporting::DiagnosticReport {
        strategic_reporting::DiagnosticReport {
            version: strategic_reporting::DIAGNOSTIC_VERSION.into(),
            game_version: 1,
            protocol_version: 1,
            search_version: strategic_reporting::SEARCH_VERSION.into(),
            metadata: json!({}),
            environments: vec![],
            checks,
            fixed_evaluations: vec![],
            runs: vec![],
            frozen_evaluations: vec![],
            summaries: vec![],
            paired_differences: vec![],
            passed: false,
        }
    }
    #[test]
    fn false_integrity_report_is_emitted_before_usage_error() {
        let mut report = test_report(vec![strategic_reporting::DiagnosticCheck {
            quantity: "deliberate failure".into(),
            expected_numerator: 1,
            actual_numerator: 0,
            denominator: 1,
            passed: false,
        }]);
        report.passed = true; // Output gate must recompute integrity rather than trust this flag.
        let mut output = Vec::new();
        assert!(matches!(
            emit_strategic_reporting(report, &mut output),
            Err(Failure::Invalid(_))
        ));
        let value: Value = serde_json::from_slice(&output).unwrap();
        assert_eq!(value["passed"], false);
        assert_eq!(value["checks"][0]["passed"], false);
    }
    #[test]
    fn report_write_and_flush_failures_propagate() {
        struct Broken(bool);
        impl Write for Broken {
            fn write(&mut self, b: &[u8]) -> std::io::Result<usize> {
                if self.0 {
                    Ok(b.len())
                } else {
                    Err(std::io::Error::other("deliberate write failure"))
                }
            }
            fn flush(&mut self) -> std::io::Result<()> {
                Err(std::io::Error::other("deliberate flush failure"))
            }
        }
        for flush in [false, true] {
            let report = test_report(vec![]);
            assert!(matches!(
                emit_strategic_reporting(report, &mut Broken(flush)),
                Err(Failure::Io(_))
            ));
        }
    }
}

fn emit_strategic_reporting(
    mut report: strategic_reporting::DiagnosticReport,
    output: &mut impl Write,
) -> Result<(), Failure> {
    report.passed = strategic_reporting::report_integrity(&report);
    let passed = report.passed;
    let value =
        serde_json::to_value(report).map_err(|_| invalid("invalid strategic-reporting report"))?;
    emit(&value, output)?;
    if passed {
        Ok(())
    } else {
        Err(invalid(
            "strategic-reporting correctness/integrity checks failed",
        ))
    }
}
