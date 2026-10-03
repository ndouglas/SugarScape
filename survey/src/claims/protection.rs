//! Registered measured protection campaign; printing never executes episodes.
use serde::{Deserialize, Serialize};
use sugarscape_core::minds::protection::state::{Fixture, LabConfig, Policy};

pub const SCHEMA: &str = "minds-protection-measured-v1";
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Condition {
    pub id: String,
    pub panel: String,
    pub lab: LabConfig,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Manifest {
    pub schema: String,
    pub conditions: Vec<Condition>,
    pub seeds: Vec<u64>,
}
pub fn manifest() -> Manifest {
    let mut conditions = Vec::new();
    for (policy, p) in [
        (Policy::Off, "off"),
        (Policy::Selective, "selective"),
        (Policy::Indiscriminate, "indiscriminate"),
        (Policy::Erased, "erased"),
    ] {
        for cost in [0.0, 0.25] {
            for mirrored in [false, true] {
                let base = LabConfig {
                    policy: policy.clone(),
                    reburial_cost: cost,
                    mirrored,
                    ..Default::default()
                };
                let mut add = |panel: &str, factors: String, fixture: Fixture, discovery: f64| {
                    conditions.push(Condition {
                        id: format!("{panel}/p={p}/{factors}/c={cost}/m={}", u8::from(mirrored)),
                        panel: panel.into(),
                        lab: LabConfig {
                            fixture,
                            discovery,
                            ..base.clone()
                        },
                    });
                };
                for initial in [false, true] {
                    for later in [false, true] {
                        add(
                            "single",
                            format!("i={}/r={}", visibility(initial), visibility(later)),
                            Fixture::Single {
                                initial_observed: initial,
                                redeposit_observed: later,
                            },
                            0.0,
                        );
                    }
                }
                for first in [false, true] {
                    add(
                        "mixed",
                        format!("o={}", if first { "a-first" } else { "b-first" }),
                        Fixture::Mixed {
                            observed_first: first,
                        },
                        0.0,
                    );
                }
                add(
                    "cue",
                    "e=visible-nonwatcher".into(),
                    Fixture::CueVisibleNonwatcher,
                    0.0,
                );
                add(
                    "cue",
                    "e=unseen-watcher".into(),
                    Fixture::CueUnseenWatcher,
                    0.0,
                );
                for initial in [false, true] {
                    for find in [0.0, 0.25] {
                        add(
                            "stumble",
                            format!("i={}/f={find}", visibility(initial)),
                            Fixture::Stumble {
                                initial_observed: initial,
                            },
                            find,
                        );
                    }
                }
            }
        }
    }
    conditions.sort_by(|a, b| a.id.cmp(&b.id));
    Manifest {
        schema: SCHEMA.into(),
        conditions,
        seeds: (10001..=10040).collect(),
    }
}
fn visibility(value: bool) -> &'static str {
    if value {
        "observed"
    } else {
        "private"
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
        super::protection_archive::validate_revision(&revision)?;
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
        Command::Manifest => println!("{}",serde_json::to_string_pretty(&manifest()).map_err(|e|e.to_string())?),
        Command::Help => println!("survey --protection [--manifest | --help | --run --protocol-revision COMMIT --out NEW_DIR | --analyze INDEX --out NEW_DIR]\nFixed matrix: 192 conditions, seeds 10001–10040. Scientific execution requires prior review; provenance flags do not confer approval."),
        Command::Run { revision,out } => super::protection_archive::run(&revision,std::path::Path::new(&out))?,
        Command::Analyze { index,out } => {
            let (index,records)=super::protection_archive::load(std::path::Path::new(&index))?;
            let analysis=super::protection_report::analyze(&index,&records)?;
            super::protection_archive::new_directory(std::path::Path::new(&out))?;
            super::protection_archive::write_new(&std::path::Path::new(&out).join("analysis.json"),&serde_json::to_vec_pretty(&analysis).map_err(|e|e.to_string())?)?;
            super::protection_archive::write_new(&std::path::Path::new(&out).join("results.md"),super::protection_report::render_results(&analysis).as_bytes())?;
        }
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::{BTreeMap, BTreeSet};
    #[test]
    fn protection_manifest_contains_the_complete_registered_matrix() {
        let m = manifest();
        assert_eq!(m.conditions.len(), 192);
        assert_eq!(m.seeds, (10001..=10040).collect::<Vec<_>>());
        let ids: BTreeSet<_> = m.conditions.iter().map(|c| &c.id).collect();
        assert_eq!(ids.len(), m.conditions.len());
        let mut counts = BTreeMap::new();
        for c in &m.conditions {
            *counts.entry(c.panel.as_str()).or_insert(0) += 1;
        }
        assert_eq!(
            counts,
            BTreeMap::from([("single", 64), ("mixed", 32), ("cue", 32), ("stumble", 64)])
        );
    }
    #[test]
    fn protection_cli_rejects_unknown_duplicate_and_seed_override() {
        for args in [
            vec!["--seeds", "3"],
            vec!["--manifest", "--manifest"],
            vec!["--wat"],
            vec!["--run", "--protocol-revision", "abc", "--out", "new"],
        ] {
            assert!(cli(&args.into_iter().map(String::from).collect::<Vec<_>>()).is_err());
        }
    }
    #[test]
    fn protection_help_and_default_never_need_provenance_or_output() {
        assert!(cli(&[]).is_ok());
        assert!(cli(&["--help".into()]).is_ok());
        assert!(cli(&["--manifest".into()]).is_ok());
    }
}
