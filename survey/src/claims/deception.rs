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
    Manifest {
        schema: SCHEMA.into(),
        conditions: sugarscape_core::minds::deception::conditions()
            .into_iter()
            .map(|lab| Condition {
                id: sugarscape_core::minds::deception::condition_id(&lab),
                lab,
            })
            .collect(),
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
    let mut flags = std::collections::BTreeMap::new();
    let mut i = 0;
    while i < args.len() {
        let key = args[i].as_str();
        if flags.contains_key(key) {
            return Err(format!("duplicate flag {key}"));
        }
        let value = match key {
            "--manifest" | "--help" | "--run" => None,
            "--protocol-revision" | "--out" | "--analyze" => {
                i += 1;
                Some(
                    args.get(i)
                        .filter(|v| !v.starts_with("--"))
                        .ok_or_else(|| format!("missing value for {key}"))?
                        .clone(),
                )
            }
            _ => {
                return Err(format!(
                    "unknown flag {key}; fixed seeds cannot be overridden"
                ))
            }
        };
        flags.insert(key, value);
        i += 1;
    }
    if flags.is_empty() {
        return Ok(Command::Manifest);
    }
    if flags.len() == 1 && flags.contains_key("--help") {
        return Ok(Command::Help);
    }
    if flags.len() == 1 && flags.contains_key("--manifest") {
        return Ok(Command::Manifest);
    }
    let value = |key| {
        flags
            .get(key)
            .and_then(Clone::clone)
            .ok_or_else(|| format!("required flag {key}"))
    };
    if flags.len() == 3
        && flags.contains_key("--run")
        && flags.contains_key("--protocol-revision")
        && flags.contains_key("--out")
    {
        let revision = value("--protocol-revision")?;
        super::deception_archive::validate_revision(&revision)?;
        return Ok(Command::Run {
            revision,
            out: value("--out")?,
        });
    }
    if flags.len() == 2 && flags.contains_key("--analyze") && flags.contains_key("--out") {
        return Ok(Command::Analyze {
            index: value("--analyze")?,
            out: value("--out")?,
        });
    }
    Err("use --manifest, --help, --run --protocol-revision COMMIT --out NEW_DIR, or --analyze INDEX --out NEW_DIR".into())
}
pub fn cli(args: &[String]) -> Result<(), String> {
    match parse(args)? {
        Command::Manifest => println!("{}", serde_json::to_string_pretty(&manifest()).map_err(|e| e.to_string())?),
        Command::Help => println!("survey --deception [--manifest|--help]\nsurvey --deception --run --protocol-revision FULL40HEX --out NEW_DIR\nsurvey --deception --analyze INDEX --out NEW_DIR\nFixed candidate protocol; provenance is not scientific acceptance."),
        Command::Run { revision, out } => super::deception_archive::run(&revision, std::path::Path::new(&out))?,
        Command::Analyze { index, out } => {
            let archive = super::deception_archive::load(std::path::Path::new(&index))?;
            let analysis = super::deception_report::analyze(&archive)?;
            super::deception_report::save(&analysis, std::path::Path::new(&out))?;
        }
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
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
