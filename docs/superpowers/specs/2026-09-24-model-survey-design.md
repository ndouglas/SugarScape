# Model survey: do the presets and sweeps show what they claim?

## Purpose

Several results look shaky: the N-goods sweep collapsed because of its layout
rather than its subject, and VI-2 vs VI-3 shows no difference between trade and
no trade. The existing book checks (`crates/sugarscape-core/tests/book.rs`) mostly
compare means over 3–5 seeds without asking whether a difference is real, and
nothing checks the descriptions the app shows users.

This is a **one-off audit**. It runs every checkable claim about every preset
and built-in sweep with enough seeds to decide it, and writes a verdict report.
It fixes nothing: fixes are triaged from the report as separate work.

## Claims

Every preset (29) and built-in sweep (5) is surveyed. Claims come from three
sources, each tagged in the report:

- **Book**: the headline result of the animation or figure the item
  reproduces (Epstein & Axtell, *Growing Artificial Societies*), cited from
  the repo's specs where they quote it, and marked "from memory" otherwise.
- **App**: every checkable statement in the item's description (preset
  descriptions in `crates/sugarscape-core/src/presets.rs`, sweep descriptions
  in `sweeps/*.json`), such as "dips to about 150–235 by t = 100–150".
- **Comment**: "Measured" figures recorded in code comments, checked lightly
  (they were recorded as measurements, not promises).

A statement too vague to test gets the verdict **Untestable**, with the reason.

## Statistics

Each condition runs **20 seeds** (1–20) by default. A claim may use fewer only
when a run is slow, and the report states the count. Each claim is one of:

- **Range** ("about 150–235"): holds when ≥ 80% of seeds fall in the range;
  "about" widens each bound by 10%. **Weak** at 50–80%.
- **Comparison** ("trade raises X"): one-sided Mann–Whitney U at p < 0.01,
  reported with the difference in medians. **Weak** when the direction is
  right but p ≥ 0.01.
- **Equivalence** ("similar carrying capacities"): TOST at α = 0.05 against a
  margin stated with the claim (10% of the pooled mean unless the text implies
  another). **Weak** when not shown equivalent but not shown different either.
- **Pattern** ("waves", "one tribe dominates"): an explicit metric and
  threshold defined per claim and written into the report with its
  justification, then judged as a range or comparison.

Verdicts: **Holds**, **Weak**, **Fails**, **Untestable**. Every Weak or Fails
verdict gets a suspected cause: **model bug**, **description wrong**, or
**book not reproduced** (the model follows the stated rules but not the
book's result), after the controller has read the numbers and rerun anything
surprising independently.

**Thresholds are fixed before a claim is run.** Loosening a threshold or
choosing a metric to make a claim pass is not allowed. A threshold that turns
out to be wrong is noted in the report with the original result kept.

## Harness

A standalone Cargo package at `survey/` (not a workspace member), depending on
`../crates/sugarscape-core`:

- `runner.rs`: runs a `Config` over seeds in parallel (`std::thread::scope`)
  for a given number of ticks, and hands each finished `World` to a
  claim-supplied closure that extracts what the claim measures, so worlds
  are not retained.
- `stats.rs`: Mann–Whitney U (exact for n ≤ 20 per group, normal
  approximation above), TOST on the difference in means, quantiles, fraction
  in range, bootstrap 95% intervals for means. Unit-tested against known
  values.
- `claim.rs`: `Claim { id, item, source, citation, text, kind, check }` where
  `check` returns an `Outcome { measured: String, verdict, detail }`. The
  cause is filled in by the controller in the report, not by code.
- `claims/ch2.rs`, `ch3.rs`, `ch4.rs`, `ch5.rs`, `ch6.rs` (Chapter VI, the
  N-goods presets and the built-in sweeps), each exporting
  `fn claims() -> Vec<Claim>`.
- `main.rs`: `cargo run --release -- [--only <prefix>] [--seeds N]` runs the
  selected claims and writes `survey/out/results.json` (raw numbers) and a
  Markdown verdict table to stdout.

## Execution

1. The controller writes the harness core and `ch2.rs`, and checks them end to
   end.
2. Four subagents, each in its own git worktree, write `ch3.rs`–`ch6.rs` from
   one shared brief: these sourcing, statistics and threshold rules; the
   harness API; and the rule that no threshold is tuned to the data.
3. The controller merges the modules, runs everything, reads every Weak and
   Fails verdict, reruns surprising results independently, and assigns
   causes.
4. The controller writes the report.

## Report

`docs/survey/2026-09-24-model-survey.md`:

- a summary table: item, claim, source, verdict, cause;
- one section per item: each claim quoted, what was measured, the numbers
  (median, interquartile range, test statistic, seeds), the verdict;
- a triage list ordered by severity: model bugs first, then false
  descriptions, then book results not reproduced.

## Out of scope

- Reviewing rule implementations line by line against Appendix B.
- The web UI and Compare wiring, except where a claim depends on them.
- Fixing anything the survey finds.
- Keeping the harness as a maintained part of the repo; it lives on this
  branch and may be dropped at merge.
