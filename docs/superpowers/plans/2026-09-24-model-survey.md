# Model Survey Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Check every book claim and every app-description claim about every preset and built-in sweep with 20 seeds and proper tests, and write a verdict report.

**Architecture:** A standalone Cargo package `survey/` (outside the workspace) depends on `sugarscape-core`. It has a parallel seed runner, a small statistics module, a claim type with four judges (range, comparison, equivalence, untestable), one claims module per chapter, and a `main` that runs claims and writes JSON plus a Markdown table. The controller writes the core and Chapter II. Four subagents write Chapters III–VI in parallel. The controller then triages and writes the report.

**Tech Stack:** Rust 2021 (rustc 1.96), `sugarscape-core` by path, `serde`/`serde_json`. No other dependencies.

**Spec:** `docs/superpowers/specs/2026-09-24-model-survey-design.md`

## Global Constraints

- Seeds 1–20 by default. A claim may use fewer only when runs are slow, and must say so in its `detail`.
- Range: Holds when ≥ 80% of seeds are in range, Weak at 50–80%, Fails below. "About" widens each bound by 10% of its magnitude.
- Comparison: one-sided Mann–Whitney U, Holds at p < 0.01; Weak when medians point the right way but p ≥ 0.01; Fails otherwise.
- Equivalence: TOST (Welch) at α = 0.05 against a stated margin (default 10% of the pooled mean); Fails when a two-sided Mann–Whitney gives p < 0.01; Weak otherwise.
- Thresholds are fixed in code before a claim is first run and are never changed to make a claim pass.
- Every claim records its source (`Book`, `App` or `Comment`) and a citation; book claims not quoted in a repo spec say "from memory" in the citation.
- The survey fixes nothing in `crates/` or `web/`.
- `survey/` is not a workspace member (`[workspace]` table in its own `Cargo.toml`).
- Commit messages end with `Claude-Session: https://claude.ai/code/session_01AnHsbmytycHYkPF3cQ6b9S`.

## Review Focus

1. A seed where a group is empty, such as a world that goes extinct or has no migrators, yields NaN. The judges drop NaN values, report how many seeds remain, and give Untestable below 5 (tested in Task 2).
2. A claim's check panics, for example indexing an empty population. `main` catches it, reports the verdict `Error` with the panic message, and runs the remaining claims (tested in Task 3).
3. Both samples are identical (all zeros, say). Mann–Whitney returns p = 1 for "greater" with no division by zero (tested in Task 1).
4. TOST gets zero variance in both groups. It is equivalent exactly when |difference| < margin (tested in Task 1).
5. `--only` matches no claim. It prints the known prefixes and exits with a nonzero status instead of writing an empty report (tested in Task 3).

The spec lists four verdicts; `Error` is added for item 2 so a crashed check is never mistaken for a model result.

---

### Task 1: Crate scaffold and statistics

**Files:**
- Create: `survey/Cargo.toml`, `survey/.gitignore`, `survey/src/main.rs` (stub), `survey/src/stats.rs`

**Interfaces:**
- Produces: `stats::{mean, median, quantile, frac_in, skewness, finite, normal_cdf, student_t_cdf, mw_greater, mw_two_sided, tost}`:
  - `fn mean(v: &[f64]) -> f64`, `fn median(v: &[f64]) -> f64`, `fn quantile(v: &[f64], q: f64) -> f64` (linear interpolation on sorted copy)
  - `fn frac_in(v: &[f64], lo: f64, hi: f64) -> f64` (inclusive)
  - `fn skewness(v: &[f64]) -> f64` (population moment ratio; 0 when sd = 0)
  - `fn finite(v: &[f64]) -> Vec<f64>` (drops NaN and infinities)
  - `fn normal_cdf(z: f64) -> f64`, `fn student_t_cdf(t: f64, df: f64) -> f64`
  - `fn mw_greater(a: &[f64], b: &[f64]) -> f64`: one-sided p that `a` tends to exceed `b`
  - `fn mw_two_sided(a: &[f64], b: &[f64]) -> f64`
  - `fn tost(a: &[f64], b: &[f64], margin: f64) -> f64`: p for "|mean(a) − mean(b)| < margin"

- [ ] **Step 1: Scaffold**

`survey/Cargo.toml`:
```toml
[package]
name = "survey"
version = "0.1.0"
edition = "2021"
publish = false

# Not a member of the repository's workspace.
[workspace]

[dependencies]
sugarscape-core = { path = "../crates/sugarscape-core" }
serde = { version = "1", features = ["derive"] }
serde_json = "1"

[profile.release]
opt-level = 3
```

`survey/.gitignore`:
```
/target
```

`survey/src/main.rs` (stub, replaced in Task 3):
```rust
mod stats;

fn main() {}
```

- [ ] **Step 2: Write the failing tests** (`survey/src/stats.rs`, tests only for now)

```rust
//! The statistics the survey's judges need: Mann–Whitney U, Welch TOST,
//! quantiles. Self-contained so the survey has no statistics dependency.

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64, tol: f64) -> bool {
        (a - b).abs() <= tol
    }

    #[test]
    fn summaries() {
        let v = [1.0, 2.0, 3.0, 4.0, 10.0];
        assert_eq!(mean(&v), 4.0);
        assert_eq!(median(&v), 3.0);
        assert_eq!(quantile(&v, 0.25), 2.0);
        assert_eq!(quantile(&v, 0.75), 4.0);
        assert_eq!(frac_in(&v, 2.0, 4.0), 0.6);
        assert!(skewness(&v) > 1.0);
        assert_eq!(skewness(&[3.0, 3.0]), 0.0);
        assert_eq!(finite(&[1.0, f64::NAN, f64::INFINITY, 2.0]), vec![1.0, 2.0]);
    }

    #[test]
    fn distributions_match_tables() {
        assert!(close(normal_cdf(1.96), 0.975, 1e-4));
        assert!(close(normal_cdf(0.0), 0.5, 1e-9));
        assert!(close(student_t_cdf(1.0, 1.0), 0.75, 1e-6));
        assert!(close(student_t_cdf(2.0, 10.0), 0.963306, 1e-5));
        assert!(close(student_t_cdf(-2.0, 10.0), 1.0 - 0.963306, 1e-5));
    }

    #[test]
    fn mann_whitney_exact_without_ties() {
        // Complete separation of 3 vs 3: p = 1 / C(6, 3) = 0.05.
        let (lo, hi) = ([1.0, 2.0, 3.0], [4.0, 5.0, 6.0]);
        assert!(close(mw_greater(&hi, &lo), 0.05, 1e-12));
        assert!(close(mw_greater(&lo, &hi), 1.0, 1e-12));
        assert!(close(mw_two_sided(&hi, &lo), 0.10, 1e-12));
    }

    #[test]
    fn mann_whitney_with_ties_and_identical_samples() {
        let z = [0.0; 20];
        assert_eq!(mw_greater(&z, &z), 1.0);
        assert_eq!(mw_two_sided(&z, &z), 1.0);
        let a: Vec<f64> = (0..20).map(|i| f64::from(i / 2) + 5.0).collect();
        let b: Vec<f64> = (0..20).map(|i| f64::from(i / 2)).collect();
        assert!(mw_greater(&a, &b) < 0.01);
    }

    #[test]
    fn tost_separates_equivalent_from_different() {
        let a: Vec<f64> = (0..20).map(|i| 100.0 + f64::from(i % 5)).collect();
        let b: Vec<f64> = (0..20).map(|i| 100.5 + f64::from(i % 5)).collect();
        assert!(tost(&a, &b, 5.0) < 0.05);
        let c: Vec<f64> = a.iter().map(|x| x + 20.0).collect();
        assert!(tost(&a, &c, 5.0) > 0.5);
    }

    #[test]
    fn tost_with_zero_variance() {
        assert_eq!(tost(&[3.0; 5], &[4.0; 5], 2.0), 0.0);
        assert_eq!(tost(&[3.0; 5], &[6.0; 5], 2.0), 1.0);
    }
}
```

- [ ] **Step 3: Run to see it fail**

Run: `cd survey && cargo test --release stats`
Expected: compile errors (`mean`, `mw_greater`, … not found).

- [ ] **Step 4: Implement** (above the tests in `survey/src/stats.rs`)

```rust
pub fn finite(v: &[f64]) -> Vec<f64> {
    v.iter().copied().filter(|x| x.is_finite()).collect()
}

pub fn mean(v: &[f64]) -> f64 {
    v.iter().sum::<f64>() / v.len() as f64
}

fn sorted(v: &[f64]) -> Vec<f64> {
    let mut s = v.to_vec();
    s.sort_by(|a, b| a.partial_cmp(b).expect("finite values"));
    s
}

pub fn quantile(v: &[f64], q: f64) -> f64 {
    let s = sorted(v);
    let at = q * (s.len() - 1) as f64;
    let (i, frac) = (at.floor() as usize, at.fract());
    if i + 1 < s.len() {
        s[i] + frac * (s[i + 1] - s[i])
    } else {
        s[i]
    }
}

pub fn median(v: &[f64]) -> f64 {
    quantile(v, 0.5)
}

pub fn frac_in(v: &[f64], lo: f64, hi: f64) -> f64 {
    v.iter().filter(|x| (lo..=hi).contains(*x)).count() as f64 / v.len() as f64
}

fn variance(v: &[f64]) -> f64 {
    let m = mean(v);
    v.iter().map(|x| (x - m).powi(2)).sum::<f64>() / (v.len() as f64 - 1.0)
}

pub fn skewness(v: &[f64]) -> f64 {
    let m = mean(v);
    let n = v.len() as f64;
    let m2 = v.iter().map(|x| (x - m).powi(2)).sum::<f64>() / n;
    if m2 == 0.0 {
        return 0.0;
    }
    let m3 = v.iter().map(|x| (x - m).powi(3)).sum::<f64>() / n;
    m3 / m2.powf(1.5)
}

/// Abramowitz & Stegun 7.1.26 (absolute error < 1.5e-7).
fn erf(x: f64) -> f64 {
    let t = 1.0 / (1.0 + 0.3275911 * x.abs());
    let y = 1.0
        - t * (0.254829592
            + t * (-0.284496736 + t * (1.421413741 + t * (-1.453152027 + t * 1.061405429))))
            * (-x * x).exp();
    y.copysign(x)
}

pub fn normal_cdf(z: f64) -> f64 {
    0.5 * (1.0 + erf(z / std::f64::consts::SQRT_2))
}

/// Lanczos approximation (g = 7, n = 9).
fn ln_gamma(x: f64) -> f64 {
    const C: [f64; 9] = [
        0.999_999_999_999_809_9,
        676.520_368_121_885_1,
        -1_259.139_216_722_402_8,
        771.323_428_777_653_1,
        -176.615_029_162_140_6,
        12.507_343_278_686_905,
        -0.138_571_095_265_720_12,
        9.984_369_578_019_572e-6,
        1.505_632_735_149_311_6e-7,
    ];
    if x < 0.5 {
        let pi = std::f64::consts::PI;
        return (pi / (pi * x).sin()).ln() - ln_gamma(1.0 - x);
    }
    let x = x - 1.0;
    let t = x + 7.5;
    let sum = C[1..]
        .iter()
        .enumerate()
        .fold(C[0], |s, (i, c)| s + c / (x + i as f64 + 1.0));
    0.5 * (2.0 * std::f64::consts::PI).ln() + (x + 0.5) * t.ln() - t + sum.ln()
}

/// Continued fraction for the incomplete beta (Numerical Recipes `betacf`).
fn beta_cf(a: f64, b: f64, x: f64) -> f64 {
    let (tiny, eps) = (1e-300, 1e-14);
    let (qab, qap, qam) = (a + b, a + 1.0, a - 1.0);
    let mut c = 1.0;
    let mut d = 1.0 - qab * x / qap;
    if d.abs() < tiny {
        d = tiny;
    }
    d = 1.0 / d;
    let mut h = d;
    for m in 1..300 {
        let m = f64::from(m);
        let m2 = 2.0 * m;
        let aa = m * (b - m) * x / ((qam + m2) * (a + m2));
        d = 1.0 + aa * d;
        if d.abs() < tiny {
            d = tiny;
        }
        c = 1.0 + aa / c;
        if c.abs() < tiny {
            c = tiny;
        }
        d = 1.0 / d;
        h *= d * c;
        let aa = -(a + m) * (qab + m) * x / ((a + m2) * (qap + m2));
        d = 1.0 + aa * d;
        if d.abs() < tiny {
            d = tiny;
        }
        c = 1.0 + aa / c;
        if c.abs() < tiny {
            c = tiny;
        }
        d = 1.0 / d;
        let del = d * c;
        h *= del;
        if (del - 1.0).abs() < eps {
            break;
        }
    }
    h
}

/// The regularized incomplete beta I_x(a, b).
fn beta_inc(a: f64, b: f64, x: f64) -> f64 {
    if x <= 0.0 {
        return 0.0;
    }
    if x >= 1.0 {
        return 1.0;
    }
    let front = (ln_gamma(a + b) - ln_gamma(a) - ln_gamma(b) + a * x.ln() + b * (1.0 - x).ln()).exp();
    if x < (a + 1.0) / (a + b + 2.0) {
        front * beta_cf(a, b, x) / a
    } else {
        1.0 - front * beta_cf(b, a, 1.0 - x) / b
    }
}

pub fn student_t_cdf(t: f64, df: f64) -> f64 {
    let tail = 0.5 * beta_inc(df / 2.0, 0.5, df / (df + t * t));
    if t > 0.0 {
        1.0 - tail
    } else {
        tail
    }
}

/// U for `a`: pairs where a > b, plus half the ties.
fn u_stat(a: &[f64], b: &[f64]) -> f64 {
    a.iter()
        .map(|x| {
            b.iter()
                .map(|y| if x > y { 1.0 } else if x == y { 0.5 } else { 0.0 })
                .sum::<f64>()
        })
        .sum()
}

fn has_ties(a: &[f64], b: &[f64]) -> bool {
    let mut all = sorted(&[a, b].concat());
    let n = all.len();
    all.dedup();
    all.len() != n
}

/// Exact P(U ≥ u) under H0 with no ties, by the recurrence
/// f(u; m, n) = f(u − n; m − 1, n) + f(u; m, n − 1).
fn exact_upper(m: usize, n: usize, u: f64) -> f64 {
    let max = m * n;
    // counts[j][k] = number of arrangements of j a's and k b's with U = index.
    let mut counts = vec![vec![vec![0f64; max + 1]; n + 1]; m + 1];
    for j in 0..=m {
        for k in 0..=n {
            if j == 0 || k == 0 {
                counts[j][k][0] = 1.0;
                continue;
            }
            for w in 0..=j * k {
                let with_a_last = if w >= k { counts[j - 1][k][w - k] } else { 0.0 };
                counts[j][k][w] = with_a_last + counts[j][k - 1][w];
            }
        }
    }
    let total: f64 = counts[m][n].iter().sum();
    let from = u.ceil() as usize;
    counts[m][n][from.min(max + 1)..].iter().sum::<f64>() / total
}

/// Normal approximation with tie correction and continuity correction.
fn normal_upper(a: &[f64], b: &[f64], u: f64) -> f64 {
    let (m, n) = (a.len() as f64, b.len() as f64);
    let all = sorted(&[a, b].concat());
    let big_n = m + n;
    let mut ties = 0.0;
    let mut i = 0;
    while i < all.len() {
        let j = all[i..].iter().take_while(|x| **x == all[i]).count();
        let t = j as f64;
        ties += t * t * t - t;
        i += j;
    }
    let var = m * n / 12.0 * ((big_n + 1.0) - ties / (big_n * (big_n - 1.0)));
    if var <= 0.0 {
        return 1.0;
    }
    let z = (u - m * n / 2.0 - 0.5) / var.sqrt();
    1.0 - normal_cdf(z)
}

/// One-sided p-value that `a` tends to exceed `b`.
pub fn mw_greater(a: &[f64], b: &[f64]) -> f64 {
    let u = u_stat(a, b);
    let p = if !has_ties(a, b) && a.len() <= 20 && b.len() <= 20 {
        exact_upper(a.len(), b.len(), u)
    } else {
        normal_upper(a, b, u)
    };
    p.clamp(0.0, 1.0)
}

pub fn mw_two_sided(a: &[f64], b: &[f64]) -> f64 {
    (2.0 * mw_greater(a, b).min(mw_greater(b, a))).min(1.0)
}

/// Welch two one-sided tests: p for H1 "−margin < mean(a) − mean(b) < margin".
pub fn tost(a: &[f64], b: &[f64], margin: f64) -> f64 {
    let d = mean(a) - mean(b);
    let (va, vb) = (variance(a) / a.len() as f64, variance(b) / b.len() as f64);
    let se = (va + vb).sqrt();
    if se == 0.0 {
        return if d.abs() < margin { 0.0 } else { 1.0 };
    }
    let df = (va + vb).powi(2)
        / (va * va / (a.len() as f64 - 1.0) + vb * vb / (b.len() as f64 - 1.0));
    let p_low = 1.0 - student_t_cdf((d + margin) / se, df);
    let p_high = student_t_cdf((d - margin) / se, df);
    p_low.max(p_high)
}
```

- [ ] **Step 5: Run the tests**

Run: `cd survey && cargo test --release stats`
Expected: 6 passed.

- [ ] **Step 6: Commit**

```bash
git add survey/Cargo.toml survey/.gitignore survey/Cargo.lock survey/src/main.rs survey/src/stats.rs
git commit -m "Add the survey crate's statistics: Mann-Whitney U, Welch TOST, quantiles"
```

---

### Task 2: Runner and claims with judges

**Files:**
- Create: `survey/src/runner.rs`, `survey/src/claim.rs`
- Modify: `survey/src/main.rs` (add `mod runner; mod claim;`)

**Interfaces:**
- Consumes: `stats::*` from Task 1.
- Produces:
  - `runner::each_seed<T: Send>(config: &Config, seeds: &[u64], f: impl Fn(World) -> T + Sync) -> Vec<T>`: builds `World::new(config.clone(), seed)` for each seed on a thread pool; `f` owns and steps the world.
  - `runner::after<T: Send>(config: &Config, seeds: &[u64], ticks: u32, f: impl Fn(&World) -> T + Sync) -> Vec<T>`
  - `runner::preset(id: &str) -> Config` (panics on an unknown id)
  - `runner::series(w: &World, name: &str) -> Vec<f64>`: index t is tick t (index 0 is the initial state)
  - `runner::window_mean(s: &[f64], from: usize, to: usize) -> f64`: mean of `s[from..=to]`
  - `claim::{Source, Verdict, Outcome, Claim}`, and the judges `claim::{range, greater, equivalent, untestable}`:
    - `enum Source { Book, App, Comment }`
    - `enum Verdict { Holds, Weak, Fails, Untestable, Error }`
    - `struct Outcome { verdict: Verdict, measured: String, detail: String }`
    - `struct Claim { id: &'static str, item: &'static str, source: Source, citation: &'static str, text: &'static str, check: fn(&[u64]) -> Outcome }`
    - `fn range(values: &[f64], lo: f64, hi: f64, about: bool) -> Outcome`
    - `fn greater(a: &[f64], b: &[f64], a_name: &str, b_name: &str) -> Outcome`: claim "a > b"
    - `fn equivalent(a: &[f64], b: &[f64], margin: Option<f64>, a_name: &str, b_name: &str) -> Outcome`
    - `fn untestable(reason: &str) -> Outcome`
    - `Outcome::with(self, detail: &str) -> Outcome`: appends to `detail`

- [ ] **Step 1: Write the failing tests**

At the bottom of `survey/src/claim.rs`:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn range_counts_seeds_and_widens_about() {
        let v: Vec<f64> = (0..20).map(f64::from).collect(); // 0..19
        assert_eq!(range(&v, 0.0, 15.0, false).verdict, Verdict::Holds); // 16/20
        assert_eq!(range(&v, 0.0, 11.0, false).verdict, Verdict::Weak); // 12/20
        assert_eq!(range(&v, 0.0, 5.0, false).verdict, Verdict::Fails); // 6/20
        // about: [10, 15] becomes [9, 16.5], 8/20.
        assert!(range(&v, 10.0, 15.0, true).measured.contains("8/20"));
    }

    #[test]
    fn judges_drop_nan_and_need_five_seeds() {
        let mut v = vec![f64::NAN; 16];
        v.extend([1.0, 1.0, 1.0, 1.0]);
        assert_eq!(range(&v, 0.0, 2.0, false).verdict, Verdict::Untestable);
        let a = [5.0, 6.0, 7.0, 8.0, 9.0, f64::NAN];
        let b = [1.0, 2.0, 3.0, 4.0, 0.0, 0.5];
        let o = greater(&a, &b, "a", "b");
        assert!(o.measured.contains("n = 5 vs 6"), "{}", o.measured);
    }

    #[test]
    fn greater_needs_significance_and_direction() {
        let hi: Vec<f64> = (0..20).map(|i| f64::from(i) + 30.0).collect();
        let lo: Vec<f64> = (0..20).map(f64::from).collect();
        assert_eq!(greater(&hi, &lo, "hi", "lo").verdict, Verdict::Holds);
        assert_eq!(greater(&lo, &hi, "lo", "hi").verdict, Verdict::Fails);
        let near: Vec<f64> = lo.iter().map(|x| x + 1.0).collect();
        assert_eq!(greater(&near, &lo, "near", "lo").verdict, Verdict::Weak);
    }

    #[test]
    fn equivalent_distinguishes_all_three() {
        let a: Vec<f64> = (0..20).map(|i| 100.0 + f64::from(i % 5)).collect();
        let close: Vec<f64> = a.iter().map(|x| x + 0.5).collect();
        let far: Vec<f64> = a.iter().map(|x| x + 40.0).collect();
        assert_eq!(equivalent(&a, &close, None, "a", "b").verdict, Verdict::Holds);
        assert_eq!(equivalent(&a, &far, None, "a", "b").verdict, Verdict::Fails);
        let noisy: Vec<f64> = (0..20).map(|i| 100.0 + f64::from(i * 7 % 40)).collect();
        let noisy2: Vec<f64> = noisy.iter().map(|x| x + 8.0).collect();
        assert_eq!(equivalent(&noisy, &noisy2, None, "a", "b").verdict, Verdict::Weak);
    }
}
```

At the bottom of `survey/src/runner.rs`:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seeds_run_in_order_and_series_index_is_the_tick() {
        let c = preset("ii-2-unit");
        let pops = after(&c, &[1, 2, 3], 5, |w| (w.tick, series(w, "population")));
        assert_eq!(pops.len(), 3);
        for (tick, s) in &pops {
            assert_eq!(*tick, 5);
            assert_eq!(s.len(), 6);
            assert_eq!(s[0], 400.0);
        }
        let again = after(&c, &[2], 5, |w| series(w, "population"));
        assert_eq!(again[0], pops[1].1, "seed 2 is deterministic and in slot 1");
        assert_eq!(window_mean(&[1.0, 2.0, 3.0, 4.0], 1, 2), 2.5);
    }
}
```

- [ ] **Step 2: Run to see it fail**

Run: `cd survey && cargo test --release`
Expected: compile errors for the missing items.

- [ ] **Step 3: Implement `survey/src/runner.rs`** (above its tests)

```rust
//! Runs a config over seeds on a thread pool.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;

use sugarscape_core::config::Config;
use sugarscape_core::presets;
use sugarscape_core::world::World;

pub fn preset(id: &str) -> Config {
    presets::by_id(id).unwrap_or_else(|| panic!("no preset {id}")).config
}

/// Builds a world per seed and hands it to `f`, which steps and measures it.
/// Results come back in seed order.
pub fn each_seed<T: Send>(config: &Config, seeds: &[u64], f: impl Fn(World) -> T + Sync) -> Vec<T> {
    let next = AtomicUsize::new(0);
    let slots: Vec<Mutex<Option<T>>> = seeds.iter().map(|_| Mutex::new(None)).collect();
    let threads = std::thread::available_parallelism()
        .map_or(4, |n| n.get())
        .min(seeds.len().max(1));
    std::thread::scope(|s| {
        for _ in 0..threads {
            s.spawn(|| loop {
                let i = next.fetch_add(1, Ordering::Relaxed);
                if i >= seeds.len() {
                    break;
                }
                let w = World::new(config.clone(), seeds[i]).expect("a valid config");
                *slots[i].lock().unwrap() = Some(f(w));
            });
        }
    });
    slots
        .into_iter()
        .map(|m| m.into_inner().unwrap().expect("every seed ran"))
        .collect()
}

/// Runs `ticks` ticks per seed, then measures with `f`.
pub fn after<T: Send>(config: &Config, seeds: &[u64], ticks: u32, f: impl Fn(&World) -> T + Sync) -> Vec<T> {
    each_seed(config, seeds, |mut w| {
        w.run(ticks);
        f(&w)
    })
}

/// A statistics series; index t is tick t (index 0 is the initial state).
pub fn series(w: &World, name: &str) -> Vec<f64> {
    w.stats.series(name).unwrap_or_else(|| panic!("no series {name}"))
}

/// Mean of `s[from..=to]`.
pub fn window_mean(s: &[f64], from: usize, to: usize) -> f64 {
    let w = &s[from..=to];
    w.iter().sum::<f64>() / w.len() as f64
}
```

- [ ] **Step 4: Implement `survey/src/claim.rs`** (above its tests)

```rust
//! A claim about a preset or sweep, and the judges that decide it.

use serde::Serialize;

use crate::stats;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum Source {
    Book,
    App,
    Comment,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum Verdict {
    Holds,
    Weak,
    Fails,
    Untestable,
    Error,
}

#[derive(Clone, Debug, Serialize)]
pub struct Outcome {
    pub verdict: Verdict,
    pub measured: String,
    pub detail: String,
}

impl Outcome {
    pub fn with(mut self, detail: &str) -> Self {
        if !self.detail.is_empty() {
            self.detail.push_str(" ");
        }
        self.detail.push_str(detail);
        self
    }
}

pub struct Claim {
    /// `<item>.<short-name>`, e.g. `ii-2.capacity`.
    pub id: &'static str,
    /// A preset or sweep id.
    pub item: &'static str,
    pub source: Source,
    /// Where the claim is stated (a spec path, `presets.rs` description, or
    /// "book, from memory: …").
    pub citation: &'static str,
    /// The claim in the source's words.
    pub text: &'static str,
    pub check: fn(&[u64]) -> Outcome,
}

const MIN_SEEDS: usize = 5;

fn summary(v: &[f64]) -> String {
    format!(
        "median {:.4} (IQR {:.4}–{:.4})",
        stats::median(v),
        stats::quantile(v, 0.25),
        stats::quantile(v, 0.75)
    )
}

pub fn untestable(reason: &str) -> Outcome {
    Outcome { verdict: Verdict::Untestable, measured: String::new(), detail: reason.into() }
}

fn too_few(n: usize) -> Outcome {
    untestable(&format!("only {n} seeds gave a finite value (need {MIN_SEEDS})"))
}

/// Holds when ≥ 80% of seeds are in [lo, hi] (widened by 10% when `about`).
pub fn range(values: &[f64], lo: f64, hi: f64, about: bool) -> Outcome {
    let v = stats::finite(values);
    if v.len() < MIN_SEEDS {
        return too_few(v.len());
    }
    let (lo, hi) = if about { (lo - 0.1 * lo.abs(), hi + 0.1 * hi.abs()) } else { (lo, hi) };
    let frac = stats::frac_in(&v, lo, hi);
    let inside = (frac * v.len() as f64).round() as usize;
    let verdict = if frac >= 0.8 {
        Verdict::Holds
    } else if frac >= 0.5 {
        Verdict::Weak
    } else {
        Verdict::Fails
    };
    Outcome {
        verdict,
        measured: format!("{}; {inside}/{} in [{lo:.4}, {hi:.4}]", summary(&v), v.len()),
        detail: String::new(),
    }
}

/// Claim: `a` exceeds `b`. One-sided Mann–Whitney at p < 0.01.
pub fn greater(a: &[f64], b: &[f64], a_name: &str, b_name: &str) -> Outcome {
    let (a, b) = (stats::finite(a), stats::finite(b));
    if a.len().min(b.len()) < MIN_SEEDS {
        return too_few(a.len().min(b.len()));
    }
    let p = stats::mw_greater(&a, &b);
    let verdict = if p < 0.01 {
        Verdict::Holds
    } else if stats::median(&a) > stats::median(&b) {
        Verdict::Weak
    } else {
        Verdict::Fails
    };
    Outcome {
        verdict,
        measured: format!(
            "{a_name} {}; {b_name} {}; one-sided Mann–Whitney p = {p:.2e}; n = {} vs {}",
            summary(&a),
            summary(&b),
            a.len(),
            b.len()
        ),
        detail: String::new(),
    }
}

/// Claim: `a` and `b` are about the same. TOST at 0.05 against `margin`
/// (default 10% of the pooled mean).
pub fn equivalent(a: &[f64], b: &[f64], margin: Option<f64>, a_name: &str, b_name: &str) -> Outcome {
    let (a, b) = (stats::finite(a), stats::finite(b));
    if a.len().min(b.len()) < MIN_SEEDS {
        return too_few(a.len().min(b.len()));
    }
    let margin = margin.unwrap_or_else(|| 0.1 * stats::mean(&[a.as_slice(), b.as_slice()].concat()).abs());
    let p_eq = stats::tost(&a, &b, margin);
    let p_diff = stats::mw_two_sided(&a, &b);
    let verdict = if p_eq < 0.05 {
        Verdict::Holds
    } else if p_diff < 0.01 {
        Verdict::Fails
    } else {
        Verdict::Weak
    };
    Outcome {
        verdict,
        measured: format!(
            "{a_name} {}; {b_name} {}; TOST p = {p_eq:.2e} (margin {margin:.4}); two-sided Mann–Whitney p = {p_diff:.2e}; n = {} vs {}",
            summary(&a),
            summary(&b),
            a.len(),
            b.len()
        ),
        detail: String::new(),
    }
}
```

Modify `survey/src/main.rs`:
```rust
mod claim;
mod runner;
mod stats;

fn main() {}
```

- [ ] **Step 5: Run the tests**

Run: `cd survey && cargo test --release`
Expected: 11 passed (6 stats, 4 claim, 1 runner). Unused-code warnings are fine until Task 3.

- [ ] **Step 6: Commit**

```bash
git add survey/src
git commit -m "Add the survey's seed runner and its range, comparison and equivalence judges"
```

---

### Task 3: `main`, and Chapter II claims end to end

**Files:**
- Create: `survey/src/claims/mod.rs`, `survey/src/claims/ch2.rs`
- Modify: `survey/src/main.rs`

**Interfaces:**
- Consumes: `runner::*`, `claim::*`.
- Produces:
  - `claims::all() -> Vec<Claim>`, concatenating `ch2::claims()` (and, after Task 4, `ch3`–`ch6`)
  - `fn select(claims: Vec<Claim>, only: Option<&str>) -> Result<Vec<Claim>, String>` in `main.rs`: filters by id prefix; `Err` lists item prefixes when nothing matches
  - `fn run_claim(c: &Claim, seeds: &[u64]) -> Outcome`: catches a panic in `check` and returns `Verdict::Error` with the message
  - Output: `survey/out/results.json`, an array of `{ id, item, source, citation, text, verdict, measured, detail, seconds }`, plus a Markdown table on stdout

- [ ] **Step 1: Write the failing tests** (in `survey/src/main.rs`)

```rust
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
```

- [ ] **Step 2: Run to see it fail**

Run: `cd survey && cargo test --release main`
Expected: compile errors (`run_claim`, `select` missing).

- [ ] **Step 3: Implement `survey/src/main.rs`**

```rust
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
            Outcome { verdict: Verdict::Error, measured: String::new(), detail: format!("check panicked: {msg}") }
        }
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let flag = |name: &str| args.iter().position(|a| a == name).and_then(|i| args.get(i + 1)).cloned();
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
        println!("| {} | {:?} | {:?} | {} {} |", c.id, c.source, o.verdict, o.measured, o.detail);
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
    std::fs::write(format!("{dir}/{name}"), serde_json::to_string_pretty(&rows).unwrap()).unwrap();
}
```

(Keep the tests from Step 1 at the bottom of the file.)

- [ ] **Step 4: Implement `survey/src/claims/mod.rs`**

```rust
mod ch2;

use crate::claim::Claim;

pub fn all() -> Vec<Claim> {
    [ch2::claims()].into_iter().flatten().collect()
}
```

- [ ] **Step 5: Implement `survey/src/claims/ch2.rs`**

Every threshold below is fixed now, before the first run.

```rust
//! Chapter II: ii-1-instant, ii-2-unit, ii-5-wealth, ii-6-waves,
//! ii-7-seasons, ii-8-pollution. (Figure II-5's sweep is in ch6.)

use std::collections::{BTreeMap, BTreeSet};

use sugarscape_core::world::World;

use crate::claim::{greater, range, untestable, Claim, Source};
use crate::runner::{after, each_seed, preset, series, window_mean};
use crate::stats;

fn positions(w: &World) -> BTreeMap<u64, (u32, u32)> {
    w.agents().map(|a| (a.id, (a.pos.x, a.pos.y))).collect()
}

/// Share of agents alive at both times that did not move between them.
fn stationary_share(before: &BTreeMap<u64, (u32, u32)>, after: &BTreeMap<u64, (u32, u32)>) -> f64 {
    let both: Vec<_> = after.iter().filter(|(id, _)| before.contains_key(id)).collect();
    if both.is_empty() {
        return f64::NAN;
    }
    both.iter().filter(|(id, p)| before[id] == **p).count() as f64 / both.len() as f64
}

fn stationary_between(id: &str, from: u32, to: u32, seeds: &[u64]) -> Vec<f64> {
    each_seed(&preset(id), seeds, |mut w| {
        w.run(from);
        let before = positions(&w);
        w.run(to - from);
        stationary_share(&before, &positions(&w))
    })
}

/// (mean metabolism, mean vision) of agents dead by `ticks`, and of survivors.
fn dead_vs_alive(seeds: &[u64], ticks: u32) -> Vec<[f64; 4]> {
    each_seed(&preset("ii-1-instant"), seeds, |mut w| {
        let start: Vec<(u64, f64, f64)> = w
            .agents()
            .map(|a| (a.id, f64::from(a.metabolism[0]), f64::from(a.vision)))
            .collect();
        w.run(ticks);
        let alive: BTreeSet<u64> = w.agents().map(|a| a.id).collect();
        let pick = |dead: bool, f: fn(&(u64, f64, f64)) -> f64| {
            let v: Vec<f64> = start.iter().filter(|a| alive.contains(&a.0) != dead).map(f).collect();
            if v.is_empty() { f64::NAN } else { stats::mean(&v) }
        };
        [pick(true, |a| a.1), pick(false, |a| a.1), pick(true, |a| a.2), pick(false, |a| a.2)]
    })
}

fn col(rows: &[[f64; 4]], i: usize) -> Vec<f64> {
    rows.iter().map(|r| r[i]).collect()
}

/// Per seed: (migrator mean vision, hibernator mean vision, migrator mean
/// metabolism, hibernator mean metabolism) over agents alive for all of
/// t = 100..=300. Migrators change hemisphere (north is y < 25) at least
/// twice; hibernators never do.
fn seasonal_groups(seeds: &[u64]) -> Vec<[f64; 4]> {
    each_seed(&preset("ii-7-seasons"), seeds, |mut w| {
        w.run(100);
        let mut last: BTreeMap<u64, bool> = w.agents().map(|a| (a.id, a.pos.y < 25)).collect();
        let mut switches: BTreeMap<u64, u32> = last.keys().map(|id| (*id, 0)).collect();
        for _ in 0..200 {
            w.step();
            let now: BTreeMap<u64, bool> = w.agents().map(|a| (a.id, a.pos.y < 25)).collect();
            switches.retain(|id, _| now.contains_key(id));
            for (id, n) in switches.iter_mut() {
                if now[id] != last[id] {
                    *n += 1;
                }
            }
            last = now;
        }
        let traits: BTreeMap<u64, (f64, f64)> = w
            .agents()
            .map(|a| (a.id, (f64::from(a.vision), f64::from(a.metabolism[0]))))
            .collect();
        let group = |migrant: bool, f: fn(&(f64, f64)) -> f64| {
            let v: Vec<f64> = switches
                .iter()
                .filter(|(_, n)| if migrant { **n >= 2 } else { **n == 0 })
                .map(|(id, _)| f(&traits[id]))
                .collect();
            if v.is_empty() { f64::NAN } else { stats::mean(&v) }
        };
        [group(true, |t| t.0), group(false, |t| t.0), group(true, |t| t.1), group(false, |t| t.1)]
    })
}

fn wealths(w: &World) -> Vec<f64> {
    w.agents().map(|a| a.holdings[0]).collect()
}

/// Coefficient of variation of site pollution.
fn pollution_cv(w: &World) -> f64 {
    let p: Vec<f64> = w.sites.iter().map(|s| s.pollution[0]).collect();
    let m = stats::mean(&p);
    if m == 0.0 {
        return f64::NAN;
    }
    let sd = (p.iter().map(|x| (x - m).powi(2)).sum::<f64>() / p.len() as f64).sqrt();
    sd / m
}

pub fn claims() -> Vec<Claim> {
    vec![
        Claim {
            id: "ii-1.settle",
            item: "ii-1-instant",
            source: Source::App,
            citation: "presets.rs ii-1-instant description",
            text: "agents climb to the best ridge they can see and settle",
            // Metric: share of agents that do not move between t = 90 and t = 100.
            check: |s| range(&stationary_between("ii-1-instant", 90, 100, s), 0.9, 1.0, false),
        },
        Claim {
            id: "ii-1.starve-metabolism",
            item: "ii-1-instant",
            source: Source::App,
            citation: "presets.rs ii-1-instant description",
            text: "the poorly endowed starve (higher metabolism among the dead)",
            check: |s| {
                let r = dead_vs_alive(s, 100);
                greater(&col(&r, 0), &col(&r, 1), "dead metabolism", "survivor metabolism")
            },
        },
        Claim {
            id: "ii-1.starve-vision",
            item: "ii-1-instant",
            source: Source::App,
            citation: "presets.rs ii-1-instant description",
            text: "the poorly endowed starve (lower vision among the dead)",
            check: |s| {
                let r = dead_vs_alive(s, 100);
                greater(&col(&r, 3), &col(&r, 2), "survivor vision", "dead vision")
            },
        },
        Claim {
            id: "ii-1.static",
            item: "ii-1-instant",
            source: Source::Book,
            citation: "book, from memory: Animation II-1 reaches a static configuration",
            text: "once settled, nobody else dies (deaths over t = 101..=200 are 0)",
            check: |s| {
                let d = after(&preset("ii-1-instant"), s, 200, |w| series(w, "deaths")[101..=200].iter().sum::<f64>());
                range(&d, 0.0, 0.0, false)
            },
        },
        Claim {
            id: "ii-2.capacity",
            item: "ii-2-unit",
            source: Source::Book,
            citation: "tests/book.rs quoting Chapter II: \"a carrying capacity of approximately 224 is eventually reached\"; presets.rs: \"near 224\"",
            text: "population falls to a carrying capacity near 224 (mean over t = 400..=500)",
            check: |s| {
                let p = after(&preset("ii-2-unit"), s, 500, |w| window_mean(&series(w, "population"), 400, 500));
                range(&p, 224.0, 224.0, true)
            },
        },
        Claim {
            id: "ii-2.on-mountains",
            item: "ii-2-unit",
            source: Source::App,
            citation: "presets.rs ii-2-unit description",
            text: "hiving on the two sugar mountains (≥ 80% of agents on sites of capacity ≥ 2 at t = 500)",
            check: |s| {
                let f = after(&preset("ii-2-unit"), s, 500, |w| {
                    let on = w.agents().filter(|a| w.site(a.pos).capacity[0] >= 2.0).count();
                    on as f64 / w.population() as f64
                });
                range(&f, 0.8, 1.0, false)
            },
        },
        Claim {
            id: "ii-2.continuous",
            item: "ii-2-unit",
            source: Source::App,
            citation: "presets.rs ii-2-unit description",
            text: "continuous hiving (at most half the agents stay put over t = 490..500)",
            check: |s| range(&stationary_between("ii-2-unit", 490, 500, s), 0.0, 0.5, false),
        },
        Claim {
            id: "ii-5.gini",
            item: "ii-5-wealth",
            source: Source::Book,
            citation: "docs/superpowers/specs/2026-09-22-sugarscape-wasm-playground-design.md: \"R[60,100] Gini exceeds ~0.5\"",
            text: "the Gini coefficient exceeds about 0.5 (t = 500)",
            check: |s| {
                let g = after(&preset("ii-5-wealth"), s, 500, |w| w.stats.latest().unwrap().gini);
                range(&g, 0.5, 1.0, true)
            },
        },
        Claim {
            id: "ii-5.skewed",
            item: "ii-5-wealth",
            source: Source::App,
            citation: "presets.rs ii-5-wealth description",
            text: "a skewed wealth distribution emerges (sample skewness ≥ 1 at t = 500)",
            check: |s| {
                let k = after(&preset("ii-5-wealth"), s, 500, |w| stats::skewness(&wealths(w)));
                range(&k, 1.0, f64::INFINITY, false)
            },
        },
        Claim {
            id: "ii-5.emerges",
            item: "ii-5-wealth",
            source: Source::App,
            citation: "presets.rs ii-5-wealth description",
            text: "the skew emerges (Gini at t = 500 exceeds Gini at t = 0)",
            check: |s| {
                let g = after(&preset("ii-5-wealth"), s, 500, |w| {
                    let g = series(w, "gini");
                    (g[0], g[500])
                });
                let (start, end): (Vec<f64>, Vec<f64>) = g.into_iter().unzip();
                greater(&end, &start, "Gini t=500", "Gini t=0")
            },
        },
        Claim {
            id: "ii-6.propagates",
            item: "ii-6-waves",
            source: Source::App,
            citation: "presets.rs ii-6-waves description",
            text: "a block in the southwest propagates northeast (≥ 20% of agents in the NE quadrant x ≥ 25, y < 25 at t = 200)",
            check: |s| {
                let f = after(&preset("ii-6-waves"), s, 200, |w| {
                    let ne = w.agents().filter(|a| a.pos.x >= 25 && a.pos.y < 25).count();
                    ne as f64 / w.population() as f64
                });
                range(&f, 0.2, 1.0, false)
            },
        },
        Claim {
            id: "ii-6.waves",
            item: "ii-6-waves",
            source: Source::App,
            citation: "presets.rs ii-6-waves description",
            text: "collective waves no individual can move in",
            check: |_| untestable("a visual claim about wave fronts; no agreed metric distinguishes a wave from a drift, so only propagation (ii-6.propagates) is checked"),
        },
        Claim {
            id: "ii-7.migrants-see-further",
            item: "ii-7-seasons",
            source: Source::App,
            citation: "presets.rs ii-7-seasons description",
            text: "high-vision agents migrate (migrators have higher mean vision than hibernators)",
            check: |s| {
                let r = seasonal_groups(s);
                greater(&col(&r, 0), &col(&r, 1), "migrator vision", "hibernator vision")
            },
        },
        Claim {
            id: "ii-7.hibernators-burn-less",
            item: "ii-7-seasons",
            source: Source::App,
            citation: "presets.rs ii-7-seasons description",
            text: "low-metabolism agents hibernate (hibernators have lower mean metabolism than migrators)",
            check: |s| {
                let r = seasonal_groups(s);
                greater(&col(&r, 2), &col(&r, 3), "migrator metabolism", "hibernator metabolism")
            },
        },
        Claim {
            id: "ii-8.onset",
            item: "ii-8-pollution",
            source: Source::App,
            citation: "presets.rs ii-8-pollution description",
            text: "pollution starts at t = 50 (zero through t = 49, positive at t = 60)",
            check: |s| {
                let ok = after(&preset("ii-8-pollution"), s, 60, |w| {
                    let p = series(w, "mean_pollution_0");
                    f64::from(u8::from(p[..=49].iter().all(|x| *x == 0.0) && p[60] > 0.0))
                });
                range(&ok, 1.0, 1.0, false)
            },
        },
        Claim {
            id: "ii-8.diffusion",
            item: "ii-8-pollution",
            source: Source::App,
            citation: "presets.rs ii-8-pollution description",
            text: "diffusion spreads pollution from t = 100 (site pollution less uneven at t = 110 than t = 99)",
            check: |s| {
                let r = each_seed(&preset("ii-8-pollution"), s, |mut w| {
                    w.run(99);
                    let before = pollution_cv(&w);
                    w.run(11);
                    (before, pollution_cv(&w))
                });
                let (before, after): (Vec<f64>, Vec<f64>) = r.into_iter().unzip();
                greater(&before, &after, "CV t=99", "CV t=110")
            },
        },
        Claim {
            id: "ii-8.lowers-capacity",
            item: "ii-8-pollution",
            source: Source::Book,
            citation: "book, from memory: Animation II-8, pollution makes the landscape less habitable",
            text: "carrying capacity is lower with pollution than ii-2-unit (mean population t = 400..=500)",
            check: |s| {
                let pop = |id| after(&preset(id), s, 500, |w| window_mean(&series(w, "population"), 400, 500));
                greater(&pop("ii-2-unit"), &pop("ii-8-pollution"), "ii-2-unit", "ii-8-pollution")
            },
        },
    ]
}

```

- [ ] **Step 6: Run tests, then the Chapter II survey**

Run: `cd survey && cargo test --release && cargo run --release -- --only ii-`
Expected: 13 tests pass; a Markdown table with 17 rows; `survey/out/results-ii-.json` exists. Any verdict is acceptable here. The step checks that the pipeline runs, not that the claims hold. `Error` rows mean a bug in a check: fix the check (not its threshold) and rerun.

- [ ] **Step 7: Commit**

```bash
git add survey/src
git commit -m "Survey Chapter II's presets end to end"
```

---

### Task 4: Chapters III–VI, sweeps (four parallel subagents)

**Files (one per subagent, each in its own worktree branched from this branch after Task 3):**
- Create: `survey/src/claims/ch3.rs`, `ch4.rs`, `ch5.rs` or `ch6.rs`
- Modify: `survey/src/claims/mod.rs` (add its `mod` line and its `claims()` to `all()`)

**Interfaces:**
- Consumes: exactly the Task 2 and 3 APIs listed above. `ch2.rs` is the worked example.
- Produces: `pub fn claims() -> Vec<Claim>` per module.

The claims cannot be written into this plan ahead of time: finding them in the sources is the task. Each subagent gets this brief, with its own list of items:

> You are adding survey claims for **<ITEMS>** in `survey/src/claims/<MODULE>.rs`. Read `docs/superpowers/specs/2026-09-24-model-survey-design.md`, then `docs/superpowers/plans/2026-09-24-model-survey.md` (Global Constraints and Tasks 2–3), then `survey/src/claims/ch2.rs` as the worked example.
>
> For each item:
> 1. Read its description (`crates/sugarscape-core/src/presets.rs`, or `sweeps/<id>.json`) and any comments near its definition. Every checkable statement is an `App` claim; recorded "Measured" figures are `Comment` claims.
> 2. Find the book's headline result for it in `docs/superpowers/specs/*.md` and `crates/sugarscape-core/tests/book.rs` (cite the file). If no repo file states it, you may add at most two `Book` claims from your knowledge of *Growing Artificial Societies*, citing "book, from memory: <figure/animation>".
> 3. Write one `Claim` per statement. Pick the judge by the statement's form (range, greater, equivalent). For a pattern, define a metric in a comment in the check and in `text`. Fix thresholds from the source's words before running anything. Never change a threshold, metric, tick or seed count after seeing a result. If a statement can't be measured with the public `sugarscape_core` API, use `untestable` with the reason.
> 4. Use 20 seeds (`s`). If one claim's runs exceed about 2 minutes, use `&s[..10]` and say so in `text`.
>
> Then run `cd survey && cargo test --release && cargo run --release -- --only <PREFIX>` and fix every `Error` verdict (a bug in your check, never a threshold). Commit `ch<N>.rs` and the `mod.rs` change with a message ending `Claude-Session: https://claude.ai/code/session_01AnHsbmytycHYkPF3cQ6b9S`. Report: the claims you wrote (id, source, threshold and why), the verdict table from the run, anything you could not source, and anything that surprised you. Do **not** try to explain or fix failing claims, and do not edit anything outside `survey/src/claims/`.

Assignments:

| Subagent | Module | Items | `--only` prefix |
|---|---|---|---|
| A | `ch3.rs` | iii-2-sex, iii-4-inheritance, iii-6-culture, iii-6-three-tribes, iii-9-combat, iii-11-combat-fixed, iii-12-collision, iii-14-combat-culture | `iii-` |
| B | `ch4.rs` | iv-1-spice, iv-3-trade, iv-15-trade-sex, iv-3-pollution, iv-18-foresight, iv-5-credit | `iv-` |
| C | `ch5.rs` | v-1-rid, v-2-endemic, v-mcneill | `v-` |
| D | `ch6.rs` | vi-1-everything, vi-2-no-trade, vi-3-trade, n-3-trade, n-4-peaks, n-2-pollutants, and the sweeps fig-ii-5, fig-iv-6, fig-iv-10-11, n-goods-carrying-capacity, bargaining-rules | `vi-`, `n-`, `fig-`, `sweep-` (sweep claim ids start with the sweep id) |

For the sweeps, subagent D uses `sugarscape_core::sweep::{builtin, run_all}` with the file's own seeds and ticks. Sweep claims compare per-seed values from `SweepResult::runs`, grouped by `(series, x)`.

- [ ] **Step 1: Dispatch A–D in parallel** (Agent tool, `isolation: "worktree"`), each with the brief above filled in.
- [ ] **Step 2: Merge** each returned branch into this one (`git merge --no-ff <branch>`). Conflicts can only occur in `claims/mod.rs`; resolve them by keeping every `mod` line and every `claims()` call.
- [ ] **Step 3: Verify the merge**

Run: `cd survey && cargo test --release && cargo run --release --quiet -- --only nothing`
Expected: tests pass; the second command exits with status 2 and lists the prefixes ii through vi, n and fig.

- [ ] **Step 4: Commit the merge** (if the merges were fast-forwards there is nothing to commit).

---

### Task 5: Full run and triage

**Files:**
- Create: `survey/out/results.json` (committed as the raw data), `survey/out/table.md`

- [ ] **Step 1: Run everything**

Run: `cd survey && cargo run --release > out/table.md`
Expected: every claim has a verdict other than `Error`, and `out/results.json` is written.

- [ ] **Step 2: Triage.** For every `Weak` and `Fails` row, the controller:
  1. rereads the claim's source and checks that the check measures what the text says. If the check is wrong, fix it and rerun that claim, and keep the first result in the report's notes.
  2. independently confirms a surprising result with a second measurement, such as the CLI (`./target/release/sugarscape run --preset <id> --series-csv …`) or a different metric.
  3. assigns a cause: **model bug** (the code contradicts a stated rule; cite the rule and the code), **description wrong** (the model follows the rules and the text overstates or misstates), or **book not reproduced** (the rules match but the book's result doesn't appear).
  Record each finding in a notes file, `survey/out/triage.md`, as `id | cause | evidence`.

- [ ] **Step 3: Commit**

```bash
git add survey/out/results.json survey/out/table.md survey/out/triage.md survey/src
git commit -m "Run the full model survey and triage its failures"
```

---

### Task 6: Report

**Files:**
- Create: `docs/survey/2026-09-24-model-survey.md`

- [ ] **Step 1: Write the report** with:
  1. **Summary:** counts by verdict and by source, and a one-paragraph reading.
  2. **Triage list:** model bugs first, then wrong descriptions, then book results not reproduced. Each entry gives the claim, the evidence, and the suggested fix.
  3. **Summary table:** item | claim id | source | verdict | cause.
  4. **One section per item:** each claim quoted with its citation, what was measured, the numbers (median, IQR, test, p, n), the verdict, and triage notes.
  5. **Method:** seeds, judges and thresholds (from Global Constraints), and how to rerun (`cd survey && cargo run --release -- --only <prefix>`).
- [ ] **Step 2: Check** that every row in `survey/out/results.json` appears in the summary table, and that every Weak and Fails row has a cause.
- [ ] **Step 3: Commit**

```bash
git add docs/survey/2026-09-24-model-survey.md
git commit -m "Report the model survey's verdicts and triage"
```
