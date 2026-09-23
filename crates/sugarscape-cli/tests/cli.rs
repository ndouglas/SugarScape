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
