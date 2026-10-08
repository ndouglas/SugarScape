use serde::{Deserialize, Serialize};
use sugarscape_core::minds::deception::state::LabConfig;
pub const SCHEMA: &str = sugarscape_core::minds::deception::SCHEMA;
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Condition {
    pub id: String,
    pub lab: LabConfig,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    pub schema: String,
    pub conditions: Vec<Condition>,
    pub seeds: Vec<u64>,
}
pub fn manifest() -> Manifest {
    let mut conditions = sugarscape_core::minds::deception::conditions()
        .into_iter()
        .map(|lab| Condition {
            id: sugarscape_core::minds::deception::condition_id(&lab),
            lab,
        })
        .collect::<Vec<_>>();
    conditions.sort_by(|a, b| a.id.cmp(&b.id));
    Manifest {
        schema: SCHEMA.into(),
        conditions,
        seeds: (20001..=20040).collect(),
    }
}
#[derive(Debug, PartialEq)]
enum Command {
    Manifest,
    Help,
    Run { revision: String, out: String },
    Analyze { index: String, out: String },
}
fn parse(args: &[String]) -> Result<Command, String> {
    if args.is_empty() {
        return Ok(Command::Manifest);
    }
    if args == ["--help"] {
        return Ok(Command::Help);
    }
    if args == ["--manifest"] {
        return Ok(Command::Manifest);
    }
    let mut flags = std::collections::BTreeMap::new();
    let mut i = 0;
    while i < args.len() {
        let flag = args[i].as_str();
        let value = match flag {
            "--run" => None,
            "--analyze" | "--protocol-revision" | "--out" => {
                i += 1;
                Some(
                    args.get(i)
                        .filter(|s| !s.is_empty() && !s.starts_with("--"))
                        .ok_or_else(|| format!("missing value for {flag}"))?
                        .clone(),
                )
            }
            _ => return Err(format!("unknown or mixed option: {flag}")),
        };
        if flags.insert(flag, value).is_some() {
            return Err(format!("duplicate option: {flag}"));
        }
        i += 1;
    }
    let out = flags.remove("--out").flatten().ok_or("missing --out")?;
    if flags.contains_key("--run") && flags.len() == 2 {
        let revision = flags
            .remove("--protocol-revision")
            .flatten()
            .ok_or("missing --protocol-revision")?;
        super::deception_archive::validate_revision(&revision)?;
        return Ok(Command::Run { revision, out });
    }
    if flags.len() == 1 {
        if let Some(Some(index)) = flags.remove("--analyze") {
            return Ok(Command::Analyze { index, out });
        }
    }
    Err("expected exactly --run --protocol-revision FULL40HEX --out NEW_DIR or --analyze INDEX --out NEW_DIR".into())
}
pub fn cli(args: &[String]) -> Result<(), String> {
    match parse(args)? {
        Command::Help => println!("--deception [--manifest|--help] | --deception --run --protocol-revision FULL40HEX --out NEW_DIR | --deception --analyze INDEX --out NEW_DIR"),
        Command::Manifest => println!("{}", serde_json::to_string_pretty(&manifest()).map_err(|e| e.to_string())?),
        Command::Run { revision, out } => return super::deception_archive::run(&revision, std::path::Path::new(&out)),
        Command::Analyze { index, out } => {
            let archive = super::deception_archive::load(std::path::Path::new(&index))?;
            let analysis = super::deception_report::analyze(&archive)?;
            return super::deception_report::save(&analysis, std::path::Path::new(&out));
        }
    }
    Ok(())
}
pub fn route(args: &[String]) -> Option<Result<(), String>> {
    let position = args.iter().position(|s| s == "--deception")?;
    let mut args = args.to_vec();
    args.remove(position);
    Some(cli(&args))
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn deception_fix1_route_and_cli_print_without_execution() {
        assert!(route(&["--deception".into(), "--help".into()])
            .unwrap()
            .is_ok());
        assert!(route(&["--deception".into(), "--deception".into()])
            .unwrap()
            .is_err());
        assert!(route(&["--burrow".into()]).is_none());
        assert!(cli(&["--help".into()]).is_ok());
        assert!(cli(&["--manifest".into()]).is_ok());
    }
    #[test]
    fn deception_fix1_parser_exact_execution_forms() {
        let revision = "a".repeat(40);
        let args = ["--run", "--protocol-revision", &revision, "--out", "fresh"].map(String::from);
        assert_eq!(
            parse(&args).unwrap(),
            Command::Run {
                revision,
                out: "fresh".into()
            }
        );
        assert_eq!(
            parse(&["--out", "fresh", "--analyze", "saved/index.json"].map(String::from)).unwrap(),
            Command::Analyze {
                index: "saved/index.json".into(),
                out: "fresh".into()
            }
        );
    }
    #[test]
    fn deception_manifest_is_fixed_before_execution() {
        let m = manifest();
        assert_eq!(m.conditions.len(), 96);
        assert_eq!(m.seeds, (20001..=20040).collect::<Vec<_>>());
        assert_eq!(
            m.conditions
                .iter()
                .map(|c| &c.id)
                .collect::<std::collections::BTreeSet<_>>()
                .len(),
            96
        );
        assert!(m
            .conditions
            .iter()
            .any(|c| c.id == "matched-neutral-ambiguous-seen-on-route-cost3-m0"));
    }
    #[test]
    fn deception_parser_prints_without_execution_and_denies_overrides() {
        assert_eq!(parse(&[]).unwrap(), Command::Manifest);
        for (args, expected) in [
            (vec!["--help"], Command::Help),
            (vec!["--manifest"], Command::Manifest),
        ] {
            assert_eq!(
                parse(&args.into_iter().map(String::from).collect::<Vec<_>>()).unwrap(),
                expected
            );
        }
        for args in [
            vec!["--seeds", "7"],
            vec!["--help", "--help"],
            vec!["--out"],
            vec!["--analyze", "--out"],
            vec!["--run"],
            vec!["--manifest", "--run"],
            vec!["--burrow"],
            vec!["--layout", "on-route"],
            vec!["--deception"],
        ] {
            assert!(parse(&args.into_iter().map(String::from).collect::<Vec<_>>()).is_err());
        }
    }
}
