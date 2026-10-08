//! Pure fixed-command parsing; default and help never call collection or construct worlds.
use super::{
    manifest::manifest_bytes,
    run::{collect, revision, ExecutionContext, RunRequest},
    CollectionMode,
};
use std::{collections::BTreeMap, io::Write, path::PathBuf};

const HELP: &str = "survey --foraging-shortcuts [--help | --run [--construction] --protocol-revision COMMIT --approval-context TEXT --out NEW_DIR | --analyze INDEX --out NEW_DIR]\nDefault prints the fully resolved draft manifest without execution. Construction uses fixed seeds 7,8; scientific execution requires an authorized registered manifest. Provenance flags record approval references and do not confer authorization.\n";

#[derive(Clone, Debug, PartialEq)]
pub(super) enum Command {
    Manifest,
    Help,
    Run(RunRequest),
    Analyze { index: PathBuf, out: PathBuf },
}
pub(super) fn parse(args: &[String]) -> Result<Command, String> {
    if args.is_empty() {
        return Ok(Command::Manifest);
    }
    let mut flags = BTreeMap::new();
    let mut i = 0;
    while i < args.len() {
        let flag = args[i].as_str();
        let value = match flag {
            "--help" | "--run" | "--construction" => None,
            "--protocol-revision" | "--approval-context" | "--out" | "--analyze" => {
                i += 1;
                let value = args
                    .get(i)
                    .ok_or_else(|| format!("{flag} requires a value"))?;
                if value.trim().is_empty() || value.starts_with("--") {
                    return Err(format!("{flag} requires a nonempty value"));
                }
                Some(value.as_str())
            }
            _ => return Err(format!("unknown shortcut argument: {flag}")),
        };
        if flags.insert(flag, value).is_some() {
            return Err(format!("repeated shortcut argument: {flag}"));
        }
        i += 1;
    }
    if flags.len() == 1 && flags.contains_key("--help") {
        return Ok(Command::Help);
    }
    let value = |flag| -> Result<&str, String> {
        flags
            .get(flag)
            .and_then(|v| *v)
            .ok_or_else(|| format!("shortcut command requires {flag}"))
    };
    if flags.contains_key("--run") {
        let expected = 4 + usize::from(flags.contains_key("--construction"));
        if flags.len() != expected
            || flags.contains_key("--help")
            || flags.contains_key("--analyze")
        {
            return Err("--run accepts only --construction, --protocol-revision, --approval-context, and --out".into());
        }
        let protocol_revision = revision(value("--protocol-revision")?)?;
        return Ok(Command::Run(RunRequest {
            mode: if flags.contains_key("--construction") {
                CollectionMode::Construction
            } else {
                CollectionMode::Scientific
            },
            protocol_revision,
            approval_context: value("--approval-context")?.to_owned(),
            out: PathBuf::from(value("--out")?),
        }));
    }
    if flags.len() == 2 && flags.contains_key("--analyze") && flags.contains_key("--out") {
        return Ok(Command::Analyze {
            index: value("--analyze")?.into(),
            out: value("--out")?.into(),
        });
    }
    Err("use shortcut default manifest, --help, --run with provenance and --out, or --analyze INDEX --out NEW_DIR".into())
}
fn stdout(bytes: &[u8]) -> Result<(), String> {
    let mut output = std::io::stdout().lock();
    write_output(&mut output, bytes)
}
pub(super) fn write_output(output: &mut impl Write, bytes: &[u8]) -> Result<(), String> {
    output
        .write_all(bytes)
        .and_then(|_| output.flush())
        .map_err(|e| format!("shortcut stdout: {e}"))
}
pub(crate) fn cli(args: &[String]) -> Result<(), String> {
    match parse(args)? {
        Command::Manifest => stdout(&manifest_bytes()?),
        Command::Help => stdout(HELP.as_bytes()),
        Command::Run(request) => {
            let context = ExecutionContext {
                repo: PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/..")),
                executable: std::env::current_exe()
                    .map_err(|e| format!("shortcut executable: {e}"))?,
            };
            collect(&context, &request).map(|_| ())
        }
        // Task 5 replaces only this saved-analysis dispatch arm.
        Command::Analyze { .. } => {
            Err("shortcut saved analysis is not yet supported (Task 5)".into())
        }
    }
}
