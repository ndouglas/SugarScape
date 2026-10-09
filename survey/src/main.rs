//! One-off survey: does every preset and sweep show what the book and the
//! app say it shows? See docs/superpowers/specs/2026-09-24-model-survey-design.md.
//!
//! `cargo run --release -- [--only <id prefix>] [--seeds N]`
//!
//! `cargo run --release -- --calibration` runs only Minds 8b's claim 2
//! calibration (seeds 1–20 always) and writes its per-seed table to
//! `out/minds8b-calibration.md` (tracked).
//!
//! `cargo run --release -- --usage` runs only Minds 8b's usage check (seeds
//! 1–20 always; a check, not a claim) and writes `out/minds8b-usage.md`.
//!
//! `cargo run --release -- --presets` measures every watching preset
//! (seeds 1–20 always; reported, not judged) and writes
//! `out/minds8b-presets.md`.
//!
//! `cargo run --release -- --baseline` measures claim 3's variant forgo at
//! every share with watching off (seeds 1–60; reported, not judged, added
//! after the run) and prints the table appended to `out/minds8b-results.md`.

mod claim;
mod claims;
mod runner;
mod stats;

use std::panic::{catch_unwind, AssertUnwindSafe};
use std::time::Instant;

use claim::{Claim, Outcome, Verdict};
use serde::Serialize;

#[derive(Serialize)]
struct Row<'a> {
    id: &'a str,
    item: &'a str,
    source: claim::Source,
    citation: &'a str,
    text: &'a str,
    verdict: Verdict,
    measured: String,
    detail: String,
    seconds: f64,
}

fn select(claims: Vec<Claim>, only: Option<&str>) -> Result<Vec<Claim>, String> {
    let Some(prefix) = only else {
        return Ok(claims);
    };
    let mut items: Vec<&str> = claims
        .iter()
        .map(|c| c.id.split('.').next().unwrap())
        .collect();
    items.dedup();
    let known = items.join(", ");
    let chosen: Vec<Claim> = claims
        .into_iter()
        .filter(|c| c.id.starts_with(prefix))
        .collect();
    if chosen.is_empty() {
        Err(format!(
            "no claim id starts with {prefix:?}; known: {known}"
        ))
    } else {
        Ok(chosen)
    }
}

fn run_claim(c: &Claim, seeds: &[u64]) -> Outcome {
    match catch_unwind(AssertUnwindSafe(|| (c.check)(seeds))) {
        Ok(o) => o,
        Err(e) => {
            let msg = e
                .downcast_ref::<String>()
                .cloned()
                .or_else(|| e.downcast_ref::<&str>().map(|s| s.to_string()))
                .unwrap_or_else(|| "unknown panic".into());
            Outcome {
                verdict: Verdict::Error,
                measured: String::new(),
                detail: format!("check panicked: {msg}"),
            }
        }
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if let Some(result) = claims::behavior_trees::route(&args[1..]) {
        if let Err(error) = result {
            eprintln!("behavior-trees: {error}");
            std::process::exit(2);
        }
        return;
    }
    if args[1..].iter().any(|a| a == "--foraging-shortcuts") {
        let mut route = args[1..].to_vec();
        let position = route
            .iter()
            .position(|a| a == "--foraging-shortcuts")
            .unwrap();
        route.remove(position);
        if let Err(e) = claims::foraging_shortcuts::cli(&route) {
            eprintln!("foraging-shortcuts: {e}");
            std::process::exit(2);
        }
        return;
    }
    if let Some(result) = claims::deception::route(&args[1..]) {
        if let Err(error) = result {
            eprintln!("{error}");
            std::process::exit(2);
        }
        return;
    }
    if args.iter().any(|a| a == "--burrow") {
        let mut route = args[1..].to_vec();
        let position = route.iter().position(|a| a == "--burrow").unwrap();
        route.remove(position);
        if let Err(e) = claims::burrow::cli(&route) {
            eprintln!("{e}");
            std::process::exit(2);
        }
        return;
    }
    if args.iter().any(|a| a == "--protection") {
        let mut route = args[1..].to_vec();
        let position = route.iter().position(|a| a == "--protection").unwrap();
        route.remove(position);
        if let Err(e) = claims::protection::cli(&route) {
            eprintln!("{e}");
            std::process::exit(2);
        }
        return;
    }
    if args.iter().any(|a| a == "--minds9") {
        let route: Vec<String> = args[1..]
            .iter()
            .filter(|a| a.as_str() != "--minds9")
            .cloned()
            .collect();
        if let Err(e) = claims::minds9::cli(&route) {
            eprintln!("{e}");
            std::process::exit(2);
        }
        return;
    }
    if args.iter().any(|a| a == "--help") {
        println!("survey [--only PREFIX] [--seeds N]\nsurvey --burrow --help (candidate manifest; no implicit execution)\nsurvey --minds9 --help (declared measured campaign)\nsurvey --protection --help (registered protection campaign)\nsurvey --foraging-shortcuts --help (draft shortcut manifest; explicit construction collection)");
        return;
    }
    let flag = |name: &str| {
        args.iter()
            .position(|a| a == name)
            .and_then(|i| args.get(i + 1))
            .cloned()
    };
    if args.iter().any(|a| a == "--calibration") {
        print!("{}", claims::minds8b::calibration_report());
        return;
    }
    if args.iter().any(|a| a == "--presets") {
        print!("{}", claims::minds8b::presets_report());
        return;
    }
    if args.iter().any(|a| a == "--baseline") {
        print!("{}", claims::minds8b::baseline_report());
        return;
    }
    if args.iter().any(|a| a == "--usage") {
        print!("{}", claims::minds8b::usage_report());
        return;
    }
    let only = flag("--only");
    let n: u64 = flag("--seeds").map_or(20, |s| s.parse().expect("--seeds takes a number"));
    let seeds: Vec<u64> = (1..=n).collect();
    let chosen = match select(claims::all(), only.as_deref()) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(2);
        }
    };
    println!("| id | source | verdict | measured |\n|---|---|---|---|");
    let mut rows = Vec::new();
    for c in &chosen {
        eprintln!("running {}", c.id);
        let start = Instant::now();
        let o = run_claim(c, &seeds);
        let seconds = start.elapsed().as_secs_f64();
        println!(
            "| {} | {:?} | {:?} | {} {} |",
            c.id, c.source, o.verdict, o.measured, o.detail
        );
        rows.push(Row {
            id: c.id,
            item: c.item,
            source: c.source,
            citation: c.citation,
            text: c.text,
            verdict: o.verdict,
            measured: o.measured,
            detail: o.detail,
            seconds,
        });
    }
    let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/out");
    std::fs::create_dir_all(dir).unwrap();
    let name = only.map_or("results.json".to_string(), |p| format!("results-{p}.json"));
    std::fs::write(
        format!("{dir}/{name}"),
        serde_json::to_string_pretty(&rows).unwrap(),
    )
    .unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::claim::{untestable, Source, Verdict};

    fn fake(id: &'static str, check: fn(&[u64]) -> Outcome) -> Claim {
        Claim {
            id,
            item: "x",
            source: Source::App,
            citation: "",
            text: "",
            check,
        }
    }

    #[test]
    fn the_registered_auction_baseline_can_be_selected_without_running_it() {
        let selected = select(claims::all(), Some("auctions.bs.baseline-direction")).unwrap();
        assert_eq!(selected[0].id, "auctions.bs.baseline-direction");
    }

    #[test]
    fn a_panicking_check_is_an_error_not_a_crash() {
        let c = fake("x.boom", |_| panic!("empty population"));
        let o = run_claim(&c, &[1]);
        assert_eq!(o.verdict, Verdict::Error);
        assert!(o.detail.contains("empty population"), "{}", o.detail);
    }

    #[test]
    fn only_filters_by_prefix_and_rejects_no_match() {
        let make = || {
            vec![
                fake("ii-2.a", |_| untestable("")),
                fake("iii-6.b", |_| untestable("")),
            ]
        };
        assert_eq!(select(make(), Some("ii-")).unwrap().len(), 1);
        assert_eq!(select(make(), None).unwrap().len(), 2);
        let err = select(make(), Some("vii")).err().unwrap();
        assert!(err.contains("ii-2") && err.contains("iii-6"), "{err}");
    }
}
