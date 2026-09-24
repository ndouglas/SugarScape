//! End-to-end checks of the `sugarscape` binary.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use sugarscape_core::config::Config;
use sugarscape_core::presets;

fn sugarscape(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_sugarscape"))
        .args(args)
        .output()
        .expect("the binary runs")
}

fn stdout(out: &Output) -> String {
    String::from_utf8(out.stdout.clone()).unwrap()
}

fn stderr(out: &Output) -> String {
    String::from_utf8(out.stderr.clone()).unwrap()
}

/// A fresh directory for one test's files.
fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("sugarscape-cli-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn path(p: &Path) -> &str {
    p.to_str().unwrap()
}

fn read(p: &Path) -> String {
    std::fs::read_to_string(p).unwrap()
}

#[test]
fn presets_and_sweeps_are_listed() {
    let out = sugarscape(&["presets"]);
    assert!(out.status.success(), "{}", stderr(&out));
    assert!(stdout(&out)
        .lines()
        .any(|l| l.starts_with("ii-2-unit\tAnimation II-2\t")));
    let out = sugarscape(&["sweeps"]);
    assert!(out.status.success(), "{}", stderr(&out));
    let text = stdout(&out);
    for id in [
        "fig-ii-5",
        "fig-iv-6",
        "fig-iv-10-11",
        "n-goods-carrying-capacity",
        "bargaining-rules",
    ] {
        assert!(
            text.lines().any(|l| l.starts_with(&format!("{id}\t"))),
            "{text}"
        );
    }
}

#[test]
fn run_prints_the_golden_fingerprint_and_writes_its_files() {
    let dir = scratch("run");
    let (series, agents, config) = (
        dir.join("series.csv"),
        dir.join("agents.csv"),
        dir.join("config.json"),
    );
    let out = sugarscape(&[
        "run",
        "--preset",
        "ii-2-unit",
        "--ticks",
        "200",
        "--fingerprint",
        "--series-csv",
        path(&series),
        "--agents-csv",
        path(&agents),
        "--config-out",
        path(&config),
    ]);
    assert!(out.status.success(), "{}", stderr(&out));
    // tests/golden.rs: ii-2-unit after 200 ticks from seed 1.
    assert_eq!(stdout(&out), "0x75b93943813545e4\n");
    let series = read(&series);
    assert!(series.starts_with("tick,population,"));
    assert_eq!(series.lines().count(), 202, "header and ticks 0..=200");
    assert!(read(&agents).starts_with("id,x,y,"));
    let written = Config::from_json(&read(&config)).unwrap();
    assert_eq!(written, presets::by_id("ii-2-unit").unwrap().config);

    // The written config reproduces the run.
    let again = sugarscape(&[
        "run",
        "--config",
        path(&config),
        "--ticks",
        "200",
        "--fingerprint",
    ]);
    assert_eq!(stdout(&again), "0x75b93943813545e4\n");
}

#[test]
fn run_errors_have_exit_codes() {
    assert_eq!(sugarscape(&[]).status.code(), Some(2));
    assert_eq!(sugarscape(&["run"]).status.code(), Some(2));
    let out = sugarscape(&["run", "--preset", "no-such-preset"]);
    assert_eq!(out.status.code(), Some(2));
    assert!(
        stderr(&out).starts_with("preset: unknown preset \"no-such-preset\""),
        "{}",
        stderr(&out)
    );
    let out = sugarscape(&["run", "--config", "/nonexistent/sugarscape/config.json"]);
    assert_eq!(out.status.code(), Some(1));
    let dir = scratch("bad-config");
    let bad = dir.join("bad.json");
    std::fs::write(&bad, r#"{"population": 99999}"#).unwrap();
    let out = sugarscape(&["run", "--config", path(&bad)]);
    assert_eq!(out.status.code(), Some(2));
    assert!(
        stderr(&out).lines().any(|l| l.starts_with("population: ")),
        "{}",
        stderr(&out)
    );
}

/// The core's `tiny()` sweep: 2 series × 3 x values × 2 seeds, 20 ticks.
const TINY: &str = r#"{
  "name": "tiny",
  "base": { "preset": "ii-2-unit" },
  "set": { "population": 50 },
  "x": { "path": "vision.max", "values": [2, 4, 6] },
  "series": { "label": "Metabolism", "values": [
    { "at": 1, "set": { "goods.0.metabolism": { "min": 1, "max": 1 } } },
    { "at": 3, "name": "wide", "set": { "goods.0.metabolism": { "min": 1, "max": 5 } } }
  ] },
  "seeds": { "from": 5, "count": 2 },
  "ticks": 20,
  "metric": { "kind": "window_mean", "series": "population", "from": 10 }
}"#;

#[test]
fn sweep_files_are_the_same_for_any_jobs() {
    let dir = scratch("sweep");
    let spec = dir.join("tiny.json");
    std::fs::write(&spec, TINY).unwrap();
    let run = |jobs: &str| {
        let out = dir.join(format!("out-{jobs}.json"));
        let runs = dir.join(format!("runs-{jobs}.csv"));
        let summary = dir.join(format!("summary-{jobs}.csv"));
        let o = sugarscape(&[
            "sweep",
            path(&spec),
            "--jobs",
            jobs,
            "--out",
            path(&out),
            "--runs-csv",
            path(&runs),
            "--summary-csv",
            path(&summary),
        ]);
        assert!(o.status.success(), "{}", stderr(&o));
        assert_eq!(stdout(&o), "", "the result went to --out");
        let progress: Vec<String> = stderr(&o).lines().map(String::from).collect();
        assert_eq!(progress.len(), 12);
        assert!(progress[11].starts_with("[12/12] series="), "{progress:?}");
        (read(&out), read(&runs), read(&summary))
    };
    let one = run("1");
    let four = run("4");
    assert_eq!(one, four);
    assert!(one.0.contains("\"version\": 1"));
    assert_eq!(one.1.lines().next(), Some("series,x,seed,value"));
    assert_eq!(one.1.lines().count(), 13);
    assert_eq!(
        one.2.lines().next(),
        Some("series,series_name,x,n,mean,sd,min,max")
    );
    assert_eq!(one.2.lines().count(), 7);
}

#[test]
fn sweep_prints_json_and_quiet_silences_progress() {
    let dir = scratch("sweep-stdout");
    let spec = dir.join("tiny.json");
    std::fs::write(&spec, TINY).unwrap();
    let out = sugarscape(&[
        "sweep",
        path(&spec),
        "--quiet",
        "--seeds",
        "1",
        "--ticks",
        "15",
    ]);
    assert!(out.status.success(), "{}", stderr(&out));
    assert_eq!(stderr(&out), "");
    let result: sugarscape_core::sweep::SweepResult = serde_json::from_str(&stdout(&out)).unwrap();
    assert_eq!(
        (
            result.runs.len(),
            result.sweep.seeds.count,
            result.sweep.ticks
        ),
        (6, 1, 15)
    );
}

#[test]
fn sweep_errors_have_exit_codes() {
    let dir = scratch("sweep-errors");
    let spec = dir.join("tiny.json");
    std::fs::write(&spec, TINY).unwrap();
    // The window starts at tick 10: 5 ticks is too short.
    let out = sugarscape(&["sweep", path(&spec), "--ticks", "5", "--quiet"]);
    assert_eq!(out.status.code(), Some(2));
    assert!(
        stderr(&out).lines().any(|l| l.starts_with("metric.from: ")),
        "{}",
        stderr(&out)
    );
    let out = sugarscape(&["sweep", "--builtin", "nope"]);
    assert_eq!(out.status.code(), Some(2));
    assert!(stderr(&out).starts_with("builtin: unknown sweep \"nope\""));
    let broken = dir.join("broken.json");
    std::fs::write(&broken, "{").unwrap();
    let out = sugarscape(&["sweep", path(&broken)]);
    assert_eq!(out.status.code(), Some(2));
    assert!(stderr(&out).starts_with("sweep: "));
    assert_eq!(
        sugarscape(&["sweep", "/nonexistent/sweep.json"])
            .status
            .code(),
        Some(1)
    );
    assert_eq!(sugarscape(&["sweep"]).status.code(), Some(2));
    assert_eq!(
        sugarscape(&["sweep", path(&spec), "--jobs", "0"])
            .status
            .code(),
        Some(2)
    );
}
