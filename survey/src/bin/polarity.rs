//! Native EPM sessions from a frozen manifest; no survey-specific model engine.
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::io::{BufRead, Write};
use std::path::Path;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Mutex,
};
use sugarscape_core::model::{Model, ModelConfig};
use sugarscape_core::polarity::{PolarityConfig, PolarityWorld};

fn prepare(manifest: Value, prefix: Option<&str>) -> Result<Vec<Value>, String> {
    if manifest["schema_version"] != 1 {
        return Err("manifest schema_version must be 1".into());
    }
    let arms = manifest["arms"]
        .as_array()
        .ok_or("manifest needs arms array")?;
    let mut ids = BTreeSet::new();
    let mut ranges = Vec::new();
    let mut prepared = Vec::new();
    for arm in arms {
        let id = arm["id"]
            .as_str()
            .filter(|s| !s.is_empty())
            .ok_or("arm needs nonempty id")?;
        if !ids.insert(id.to_string()) {
            return Err(format!("duplicate arm {id}"));
        }
        let seed = arm["first_seed"]
            .as_u64()
            .ok_or("arm needs first_seed u64")?;
        let count = arm["sessions"]
            .as_u64()
            .filter(|&n| n > 0 && n <= 1_000_000)
            .ok_or("sessions must be 1..1,000,000")?;
        let last = seed
            .checked_add(count - 1)
            .ok_or("seed range overflows u64")?;
        ranges.push((seed, last));
        let config = match ModelConfig::from_json(&arm["config"].to_string())
            .map_err(|e| format!("arm {id}: {e:?}"))?
        {
            ModelConfig::Polarity(c) => c,
            _ => return Err(format!("arm {id} must use model polarity")),
        };
        let mut resolved = serde_json::to_value(config).map_err(|e| e.to_string())?;
        resolved["model"] = json!("polarity");
        let mut row = arm.clone();
        row["resolved_config"] = resolved;
        if prefix.is_none_or(|p| id.starts_with(p)) {
            prepared.push(row);
        }
    }
    ranges.sort_unstable();
    if ranges.windows(2).any(|w| w[0].1 >= w[1].0) {
        return Err("manifest seed blocks overlap".into());
    }
    if prefix.is_some() && prepared.is_empty() {
        return Err("arm prefix matches nothing".into());
    }
    Ok(prepared)
}

fn config(value: &Value) -> Result<PolarityConfig, String> {
    match ModelConfig::from_json(&value.to_string()).map_err(|e| format!("{e:?}"))? {
        ModelConfig::Polarity(c) => Ok(c),
        _ => Err("expected polarity config".into()),
    }
}

fn execute(arm: &str, seed: u64, c: &PolarityConfig, hash: &str) -> Result<Value, String> {
    execute_with(arm, seed, c, hash, PolarityWorld::new, |world| {
        while !world.finished() {
            world.run(10_000);
        }
    })
}

fn panic_message(panic: &(dyn std::any::Any + Send)) -> String {
    panic
        .downcast_ref::<String>()
        .cloned()
        .or_else(|| panic.downcast_ref::<&str>().map(|s| s.to_string()))
        .unwrap_or_else(|| "unknown panic".into())
}

fn execute_with<C, R>(
    arm: &str,
    seed: u64,
    c: &PolarityConfig,
    hash: &str,
    construct: C,
    advance: R,
) -> Result<Value, String>
where
    C: FnOnce(
        PolarityConfig,
        u64,
    ) -> Result<PolarityWorld, Vec<sugarscape_core::config::FieldError>>,
    R: FnOnce(&mut PolarityWorld),
{
    let construction =
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| construct(c.clone(), seed)));
    let outcome = match construction {
        Ok(result) => {
            let mut world = result.map_err(|e| format!("{e:?}"))?;
            if let Err(panic) =
                std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| advance(&mut world)))
            {
                world.invalidate_after_panic(panic_message(&*panic));
            }
            serde_json::to_value(world.outcome().ok_or("finished model omitted outcome")?)
                .map_err(|e| e.to_string())?
        }
        Err(panic) => json!({"config":c,"seed":seed,"valid":false,
            "invalid_reason":format!("native construction panic: {}",panic_message(&*panic)),
            "finish_reason":"panic","state_available":false,
            "periods":null,"attempted_period":null,"sovereign_count":null,"terminal_category":null,
            "initial_predator_share":null,"predator_capital_share":null,"destruction":null,
            "signed_creation":null,"events":null,"episodes":null}),
    };
    let mut resolved = serde_json::to_value(c).map_err(|e| e.to_string())?;
    resolved["model"] = json!("polarity");
    Ok(json!({"arm":arm,"seed":seed,"config":resolved,"outcome":outcome,"manifest_sha256":hash}))
}

fn valid_outcome(row: &Value, resolved: &Value) -> bool {
    let o = &row["outcome"];
    let required = [
        "config",
        "seed",
        "periods",
        "attempted_period",
        "finish_reason",
        "valid",
        "invalid_reason",
        "sovereign_count",
        "terminal_category",
        "initial_predator_share",
        "predator_capital_share",
        "destruction",
        "signed_creation",
        "events",
        "episodes",
    ];
    if required.iter().any(|k| o.get(k).is_none()) {
        return false;
    }
    let mut embedded = resolved.clone();
    embedded.as_object_mut().unwrap().remove("model");
    if o["seed"] != row["seed"] || o["config"] != embedded || !o["valid"].is_boolean() {
        return false;
    }
    let valid = o["valid"] == true;
    let reason = o["finish_reason"].as_str().unwrap_or("");
    if o["state_available"] == false {
        return !valid
            && reason == "panic"
            && o["invalid_reason"].as_str().is_some_and(|s| !s.is_empty())
            && required[2..]
                .iter()
                .filter(|k| !["valid", "invalid_reason", "finish_reason"].contains(k))
                .all(|k| o[k].is_null());
    }
    let (Some(periods), Some(attempted), Some(count)) = (
        o["periods"].as_u64(),
        o["attempted_period"].as_u64(),
        o["sovereign_count"].as_u64(),
    ) else {
        return false;
    };
    if count == 0
        || count > resolved["width"].as_u64().unwrap() * resolved["height"].as_u64().unwrap()
        || periods > attempted
        || attempted - periods > 1
        || attempted > resolved["horizon"].as_u64().unwrap()
        || !o["events"].is_object()
        || !o["episodes"].is_array()
    {
        return false;
    }
    let counters = [
        "attacks",
        "dd_encounters",
        "conquests",
        "capital_collapses",
        "disconnections",
        "revolts",
        "stale_claims",
        "locked_claims",
        "path_collisions",
        "double_successes",
    ];
    let signed = [
        "destruction",
        "signed_creation",
        "harvest",
        "taxes",
        "transfers",
        "clipping",
    ];
    if counters.iter().any(|k| o["events"][k].as_u64().is_none())
        || signed
            .iter()
            .any(|k| o["events"].get(k).is_none() || (valid && o["events"][k].as_f64().is_none()))
    {
        return false;
    }
    if valid
        && ([
            "initial_predator_share",
            "predator_capital_share",
            "destruction",
            "signed_creation",
        ]
        .iter()
        .any(|k| o[k].as_f64().is_none())
            || o["destruction"] != o["events"]["destruction"]
            || o["signed_creation"] != o["events"]["signed_creation"])
    {
        return false;
    }
    if valid {
        let category = if count == 1 {
            json!("one")
        } else if count == 2 {
            json!("two")
        } else if count <= 10 {
            json!("three_to_ten")
        } else if count <= 90 {
            json!("eleven_to_ninety")
        } else if count <= 100 {
            json!("ninety_one_to_hundred")
        } else {
            Value::Null
        };
        o["invalid_reason"].is_null()
            && periods == attempted
            && o["terminal_category"] == category
            && ((reason == "horizon" && periods == resolved["horizon"].as_u64().unwrap())
                || (reason == "hegemony" && count == 1 && resolved["stop_at_hegemony"] == true))
    } else {
        ["invalid", "panic"].contains(&reason)
            && o["terminal_category"].is_null()
            && o["invalid_reason"].as_str().is_some_and(|s| !s.is_empty())
    }
}

fn existing(rows: Vec<Value>, hash: &str) -> Result<BTreeSet<(String, u64)>, String> {
    let mut seen = BTreeSet::new();
    for row in rows {
        if row["manifest_sha256"] != hash {
            return Err("existing sessions use another manifest".into());
        }
        let arm = row["arm"]
            .as_str()
            .ok_or("existing row missing arm")?
            .to_string();
        let seed = row["seed"].as_u64().ok_or("existing row missing seed")?;
        if !seen.insert((arm, seed)) {
            return Err("existing sessions contain duplicate key".into());
        }
    }
    Ok(seen)
}

fn run() -> Result<(), String> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let value = |name: &str| {
        args.iter()
            .position(|s| s == name)
            .and_then(|i| args.get(i + 1))
            .cloned()
            .ok_or_else(|| format!("required {name} VALUE"))
    };
    let allowed = [
        "--manifest",
        "--manifest-sha256",
        "--out",
        "--arm",
        "--validate",
        "--resolved-out",
    ];
    let mut i = 0;
    while i < args.len() {
        if !allowed.contains(&args[i].as_str()) {
            return Err(format!("unknown argument {}", args[i]));
        }
        if args[i] == "--validate" {
            i += 1;
        } else {
            if args.get(i + 1).is_none_or(|v| v.starts_with("--")) {
                return Err(format!("{} needs value", args[i]));
            }
            i += 2;
        }
    }
    let manifest_path = value("--manifest")?;
    let hash = value("--manifest-sha256")?;
    if hash.len() != 64 || !hash.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err(
            "manifest SHA256 must have 64 hexadecimal digits; use survey/polarity/run.py".into(),
        );
    }
    let text = std::fs::read_to_string(&manifest_path).map_err(|e| e.to_string())?;
    let all = prepare(
        serde_json::from_str(&text).map_err(|e| e.to_string())?,
        None,
    )?;
    let prefix = args
        .iter()
        .position(|s| s == "--arm")
        .map(|i| args[i + 1].as_str());
    let selected: Vec<_> = all
        .iter()
        .filter(|a| prefix.is_none_or(|p| a["id"].as_str().unwrap().starts_with(p)))
        .collect();
    if selected.is_empty() {
        return Err("arm prefix matches nothing".into());
    }
    if args.iter().any(|s| s == "--resolved-out") {
        let destination = value("--resolved-out")?;
        if let Some(parent) = Path::new(&destination)
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
        {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let resolution = json!({"schema_version":1,"manifest_sha256":hash,
            "arms":all.iter().map(|a| json!({"id":a["id"],"config":a["resolved_config"]})).collect::<Vec<_>>()});
        std::fs::write(
            destination,
            serde_json::to_vec_pretty(&resolution).map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())?;
    }
    if args.iter().any(|s| s == "--validate") {
        println!(
            "Validated {} arms, {} sessions; no periods executed",
            selected.len(),
            selected
                .iter()
                .map(|a| a["sessions"].as_u64().unwrap())
                .sum::<u64>()
        );
        return Ok(());
    }
    let out = value("--out")?;
    let rows = if Path::new(&out).exists() {
        std::io::BufReader::new(std::fs::File::open(&out).map_err(|e| e.to_string())?)
            .lines()
            .enumerate()
            .map(|(n, line)| {
                serde_json::from_str::<Value>(&line.map_err(|e| e.to_string())?)
                    .map_err(|e| format!("raw line {}: {e}", n + 1))
            })
            .collect::<Result<Vec<_>, _>>()?
    } else {
        Vec::new()
    };
    let seen = existing(rows.clone(), &hash)?;
    let indexed: BTreeMap<_, _> = all.iter().map(|a| (a["id"].as_str().unwrap(), a)).collect();
    for row in &rows {
        let arm = indexed
            .get(row["arm"].as_str().unwrap())
            .ok_or("existing raw arm absent from manifest")?;
        let first = arm["first_seed"].as_u64().unwrap();
        let n = arm["sessions"].as_u64().unwrap();
        let seed = row["seed"].as_u64().unwrap();
        if seed < first
            || seed - first >= n
            || row["config"] != arm["resolved_config"]
            || !valid_outcome(row, &arm["resolved_config"])
        {
            return Err("existing raw session violates resolved manifest contract".into());
        }
    }
    let mut cases = Vec::new();
    for arm in selected {
        let id = arm["id"].as_str().unwrap();
        let c = config(&arm["resolved_config"])?;
        let first = arm["first_seed"].as_u64().unwrap();
        let n = arm["sessions"].as_u64().unwrap();
        for seed in (0..n).map(|offset| first + offset) {
            if !seen.contains(&(id.to_string(), seed)) {
                cases.push((id.to_string(), seed, c.clone()));
            }
        }
    }
    if let Some(parent) = Path::new(&out)
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
    {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let file = Mutex::new(
        std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&out)
            .map_err(|e| e.to_string())?,
    );
    let next = AtomicUsize::new(0);
    let completed = AtomicUsize::new(0);
    let error = Mutex::new(None::<String>);
    let threads = std::thread::available_parallelism()
        .map_or(1, |n| n.get())
        .min(cases.len().max(1));
    std::thread::scope(|scope| {
        for _ in 0..threads {
            scope.spawn(|| loop {
                if error.lock().unwrap().is_some() {
                    break;
                }
                let i = next.fetch_add(1, Ordering::Relaxed);
                let Some((arm, seed, c)) = cases.get(i) else {
                    break;
                };
                let result = execute(arm, *seed, c, &hash).and_then(|row| {
                    let mut output = file.lock().unwrap();
                    serde_json::to_writer(&mut *output, &row).map_err(|e| e.to_string())?;
                    writeln!(output).map_err(|e| e.to_string())?;
                    output.flush().map_err(|e| e.to_string())
                });
                if let Err(message) = result {
                    *error.lock().unwrap() = Some(message);
                    break;
                }
                let n = completed.fetch_add(1, Ordering::Relaxed) + 1;
                if n.is_multiple_of(100) {
                    eprintln!("polarity: {n}/{} new sessions retained", cases.len());
                }
            });
        }
    });
    if let Some(message) = error.into_inner().unwrap() {
        return Err(message);
    }
    println!(
        "Retained {} new sessions; {} existing sessions preserved",
        completed.load(Ordering::Relaxed),
        seen.len()
    );
    Ok(())
}

fn main() {
    if let Err(error) = run() {
        eprintln!("polarity survey: {error}");
        std::process::exit(2);
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn injected_construction_and_execution_panics_retain_available_state() {
        let c = config(
            &json!({"model":"polarity","predator_share":0,"horizon":3,"periods_per_tick":1}),
        )
        .unwrap();
        let construction = execute_with(
            "a",
            7,
            &c,
            "hash",
            |_, _| panic!("injected construction"),
            |_| {},
        )
        .unwrap();
        assert_eq!(construction["outcome"]["valid"], false);
        assert_eq!(construction["outcome"]["state_available"], false);
        assert_eq!(construction["outcome"]["periods"], Value::Null);
        assert_eq!(construction["outcome"]["seed"], 7);
        assert!(valid_outcome(&construction, &construction["config"]));
        let running = execute_with("a", 7, &c, "hash", PolarityWorld::new, |world| {
            world.run(1);
            panic!("injected execution");
        })
        .unwrap();
        assert_eq!(running["outcome"]["valid"], false);
        assert_eq!(running["outcome"]["periods"], 1);
        assert_eq!(running["outcome"]["attempted_period"], 1);
        assert!(running["outcome"]["events"]["harvest"].is_number());
        assert!(running["outcome"]["episodes"].is_array());
        assert_eq!(running["outcome"]["seed"], 7);
        assert!(valid_outcome(&running, &running["config"]));
    }
    #[test]
    fn duplicate_arm_ids_are_rejected_before_execution() {
        let arm = json!({"id":"a","family":"original","config":{"model":"polarity"},"first_seed":1,"sessions":20});
        let m = json!({"schema_version":1,"arms":[arm.clone(),arm]});
        assert!(prepare(m, None).unwrap_err().contains("duplicate"));
    }
    #[test]
    fn overflowing_seed_range_is_rejected_before_execution() {
        let m = json!({"schema_version":1,"arms":[{"id":"a","family":"original","config":{"model":"polarity"},"first_seed":u64::MAX,"sessions":20}]});
        assert!(prepare(m, None).unwrap_err().contains("seed"));
    }
    #[test]
    fn short_peaceful_session_reaches_configured_period_endpoint() {
        let config = PolarityConfig::default();
        let mut value = serde_json::to_value(config).unwrap();
        value["predator_share"] = json!(0.);
        value["horizon"] = json!(3);
        value["periods_per_tick"] = json!(2);
        let config: PolarityConfig = serde_json::from_value(value).unwrap();
        let record = execute("a", 7, &config, "hash").unwrap();
        assert_eq!(record["outcome"]["periods"], 3);
        assert_eq!(record["outcome"]["sovereign_count"], 100);
        assert_eq!(record["outcome"]["valid"], true);
    }
    #[test]
    fn resume_rejects_a_different_manifest_instead_of_mixing_sessions() {
        let rows = vec![json!({"arm":"a","seed":1,"manifest_sha256":"wrong"})];
        assert!(existing(rows, "wanted").unwrap_err().contains("manifest"));
    }
    #[test]
    fn invalid_partial_period_is_retained_with_completed_and_attempted_clocks() {
        let c = config(&json!({"model":"polarity","width":2,"height":2,
            "predator_share":0,"initial_mean":1,"initial_sd":0,
            "harvest_mean":-2,"harvest_sd":0,"resource_policy":"reject_nonpositive"}))
        .unwrap();
        let record = execute("invalid", 7, &c, "manifest").unwrap();
        assert_eq!(record["outcome"]["valid"], false);
        assert_eq!(record["outcome"]["periods"], 0);
        assert_eq!(record["outcome"]["attempted_period"], 1);
        assert_eq!(record["outcome"]["terminal_category"], Value::Null);
        assert!(record["outcome"]["invalid_reason"]
            .as_str()
            .unwrap()
            .contains("stock"));
        assert!(existing(vec![record], "manifest")
            .unwrap()
            .contains(&("invalid".into(), 7)));
    }
    #[test]
    fn finite_initial_stocks_with_nonfinite_aggregate_are_retained_without_periods() {
        let c = config(&json!({"model":"polarity","initial_mean":1e308,"initial_sd":0})).unwrap();
        let record = execute("overflow", 1, &c, "manifest").unwrap();
        assert_eq!(record["outcome"]["valid"], false);
        assert_eq!(record["outcome"]["periods"], 0);
        assert_eq!(record["outcome"]["attempted_period"], 0);
        assert!(record["outcome"]["invalid_reason"]
            .as_str()
            .unwrap()
            .contains("aggregate"));
    }
}
