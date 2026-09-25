//! One-off survey: does every preset and sweep show what the book and the
//! app say it shows? See docs/superpowers/specs/2026-09-24-model-survey-design.md.
//!
//! `cargo run --release -- [--only <id prefix>] [--seeds N]`

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
    let Some(prefix) = only else { return Ok(claims) };
    let mut items: Vec<&str> = claims.iter().map(|c| c.id.split('.').next().unwrap()).collect();
    items.dedup();
    let known = items.join(", ");
    let chosen: Vec<Claim> = claims.into_iter().filter(|c| c.id.starts_with(prefix)).collect();
    if chosen.is_empty() {
        Err(format!("no claim id starts with {prefix:?}; known: {known}"))
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
    let flag = |name: &str| {
        args.iter()
            .position(|a| a == name)
            .and_then(|i| args.get(i + 1))
            .cloned()
    };
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
        Claim { id, item: "x", source: Source::App, citation: "", text: "", check }
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
        let make = || vec![fake("ii-2.a", |_| untestable("")), fake("iii-6.b", |_| untestable(""))];
        assert_eq!(select(make(), Some("ii-")).unwrap().len(), 1);
        assert_eq!(select(make(), None).unwrap().len(), 2);
        let err = select(make(), Some("vii")).err().unwrap();
        assert!(err.contains("ii-2") && err.contains("iii-6"), "{err}");
    }
}
