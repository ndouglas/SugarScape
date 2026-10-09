use std::{collections::BTreeMap, path::PathBuf};
const HELP:&str="survey --behavior-trees [--manifest | --help | --run --protocol-revision FULL40HEX --out NEW_DIR | --analyze FINAL_INDEX --out NEW_DIR]\nDefault prints the fixed 96 by 40 manifest. Scientific execution requires separate prospective acceptance. No retry, resume, overrides, or arbitrary tree graphs.\n";
#[derive(Debug, PartialEq)]
pub(super) enum Command {
    Manifest,
    Help,
    Run { revision: String, out: PathBuf },
    Analyze { index: PathBuf, out: PathBuf },
}
pub(super) fn parse(args: &[String]) -> Result<Command, String> {
    if args.is_empty() {
        return Ok(Command::Manifest);
    }
    let mut f = BTreeMap::new();
    let mut i = 0;
    while i < args.len() {
        let key = args[i].as_str();
        let value = match key {
            "--manifest" | "--help" | "--run" => None,
            "--protocol-revision" | "--out" | "--analyze" => {
                i += 1;
                let v = args
                    .get(i)
                    .filter(|v| !v.is_empty() && !v.starts_with("--"))
                    .ok_or_else(|| format!("{key} needs a value"))?;
                Some(v.as_str())
            }
            _ => return Err(format!("unknown behavior-trees argument: {key}")),
        };
        if f.insert(key, value).is_some() {
            return Err(format!("duplicate behavior-trees argument: {key}"));
        }
        i += 1;
    }
    if f.len() == 1 && f.contains_key("--help") {
        return Ok(Command::Help);
    }
    if f.len() == 1 && f.contains_key("--manifest") {
        return Ok(Command::Manifest);
    }
    let value = |k| {
        f.get(k)
            .and_then(|v| *v)
            .ok_or_else(|| format!("missing {k}"))
    };
    if f.len() == 3 && f.contains_key("--run") {
        let revision = value("--protocol-revision")?;
        if revision.len() != 40 || !revision.bytes().all(|c| c.is_ascii_hexdigit()) {
            return Err("revision must be full 40 hexadecimal digits".into());
        }
        return Ok(Command::Run {
            revision: revision.to_ascii_lowercase(),
            out: value("--out")?.into(),
        });
    }
    if f.len() == 2 && f.contains_key("--analyze") {
        return Ok(Command::Analyze {
            index: value("--analyze")?.into(),
            out: value("--out")?.into(),
        });
    }
    Err("mixed/incomplete behavior-trees command".into())
}
pub fn cli(args: &[String]) -> Result<(), String> {
    use std::io::Write;
    match parse(args)? {
        Command::Manifest => {
            let b = serde_json::to_vec_pretty(&super::manifest::manifest())
                .map_err(|e| e.to_string())?;
            std::io::stdout().write_all(&b).map_err(|e| e.to_string())
        }
        Command::Help => std::io::stdout()
            .write_all(HELP.as_bytes())
            .map_err(|e| e.to_string()),
        Command::Run { revision, out } => super::archive::run(&revision, &out),
        Command::Analyze { index, out } => super::report::save(
            &super::report::analyze(&super::archive::load(&index)?)?,
            &out,
        ),
    }
}
