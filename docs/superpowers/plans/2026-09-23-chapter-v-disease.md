# Chapter V — Disease Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add the book's Chapter V disease model — immune and disease bit strings, immune response and transmission (rule E), metabolic symptoms, genome inheritance, outbreaks, medicine/vaccination/mutation knobs, a disease network overlay and an "everything on" preset — to the SugarScape playground without changing any earlier run.

**Architecture:** A small `Bits` type (`src/bits.rs`) carries immune strings and diseases. Disease state lives on the agent (`immune_genome`, `immune`, `diseases`, `infected_by`); the numbered master list lives on `World::diseases`. Everything rule E does sits in `src/rules/disease.rs` and runs only while `config.disease.enabled` (including its RNG draws and fingerprint hashing). The metabolic fee enters through two `Agent` methods that replace the raw metabolisms wherever they are used. The WASM `Sim` gains `disease_list`, `infect`, `vaccinate` and a `"disease"` network; the web UI gains a Disease rules group, outbreak listing, color mode, overlay, two tools, a chart group and inspector rows.

**Tech Stack:** unchanged — Rust (`sugarscape-core`, `sugarscape-wasm`, wasm-bindgen, rand 0.8, proptest), Vite + TypeScript + uPlot + Vitest.

**Spec:** `docs/superpowers/specs/2026-09-23-chapter-v-disease-design.md` (binding), building on `docs/superpowers/specs/2026-09-22-sugarscape-wasm-playground-design.md` and `docs/superpowers/specs/2026-09-22-chapter-iv-sugar-and-spice-design.md`.

## Global Constraints

- **Earlier runs are unchanged:** with disease off, every existing preset (milestone 1 and Chapter IV) must produce the same `World::fingerprint()` after 200 ticks (seed 1) as before this milestone (`tests/golden.rs`, Task 1). New RNG draws (disease list, immune genomes, initial infections, mutation, transmission choices) happen only when `disease.enabled`; new per-turn steps are skipped when off; `fingerprint()` hashes disease state only when disease is on. The metabolic fee is `0.0` while disease is off (`DiseaseRule::active_fee`), and `m + 0.0 == m` exactly, so effective metabolism is byte-identical to the old metabolism.
- Determinism: all randomness through `World.rng`; iterate only `Vec`/`BTreeMap`/`BTreeSet`.
- Lattice: `y = 0` is north; torus; four directions only.
- Tick order: 1. apply scheduled changes, then **apply due outbreaks**. 2. For each agent in shuffled order: move (M or C) → metabolize (effective metabolism) → [credit income] → death check → S → K → T → L-borrow → **E (immune response, then transmission)**. 3. Settle due loans; growback; diffusion; replacement; ageing; stats.
- Bit strings print position 0 first (as culture tags do): `Bits::parse("10011").get(0) == true`.
- JS seeds are `u32`; errors crossing into JS are JSON `[{field, message}]`.
- Every commit message ends with a blank line and then:

  `Claude-Session: https://claude.ai/code/session_01Kq7NyxbMrkNsPfAhnsVcK3`

  (the commit commands below pass it as a second `-m`, which produces the blank line).
- Rust tasks end with `cargo fmt --all` and `cargo clippy --all-targets -- -D warnings` clean and `cargo test -p sugarscape-core` passing (including `tests/golden.rs`). Web tasks end with `cd web && npm run build && npm test` passing.
- Web tasks cannot be browser-verified by implementers; the controller runs a puppeteer check.

## Decisions (where the spec leaves room)

These are binding for this plan; each is also stated in the task that implements it.

1. `Bits` allows length 0 (`Bits::default()`), the placeholder immune string agents hold while disease is off. Every real immune string and disease has length 1–64.
2. After an agent's immune flips, **every** carried disease that is now a substring of `immune` is removed (not only the one just trained), so "no agent carries a disease it is immune to" holds between turns. Vaccination applies the same clean-up.
3. The initial master list is `count` independent random strings (duplicates allowed, as in the book). Outbreaks, mutations and the Infect tool's "New disease" add distinct strings; a mutated disease equal to a listed one reuses that id.
4. New agents draw `min(initial, list length)` distinct ids and keep those they are not immune to (so they may start with fewer).
5. Outbreak infections count in `new_infections` with no infector (no network edge; `infected_by` unchanged). Infect-tool infections are edits: not counted, `infected_by` unchanged. `infect(…, -1)` appends the new disease even when the agent resists it.
6. Effective metabolism also feeds `stats::supply_demand` (it is trade valuation); the `mean_metabolism` series stays the genetic trait.
7. Reset-only paths for schedules are `disease`, `disease.enabled`, `disease.count`, `disease.length` (and `.min`/`.max`), `disease.immune_length`; `disease.outbreaks` (and sub-paths) may not be scheduled.
8. Genome mutation draws are skipped when `genome_mutation == 0.0`; disease-mutation draws are skipped when `disease_mutation == 0.0`.
9. Vaccination writes the disease over the leftmost closest window of `immune` (the genome is untouched) and needs a listed disease (`-1` is an error). "Clear schedule" clears scheduled changes only; outbreaks are listed read-only.
10. `DiseaseRule` is `#[serde(default)]` so partial JSON (`{"disease":{"enabled":true}}`) loads.
11. Infect/Vaccinate buttons are hidden while disease is off; choosing either switches the color mode to Disease.
12. Chapter V presets also get golden fingerprints, and a test requires every preset to have one.

## File Structure

```
crates/sugarscape-core/
  tests/golden.rs            MOD  Chapter IV (Task 1) and Chapter V (Task 13) fingerprints; coverage test
  src/bits.rs                NEW  Bits: parse/get/set/flip, windows, Hamming, closest window, learn, imprint
  src/lib.rs                 MOD  pub mod bits
  src/config.rs              MOD  DiseaseRule, Outbreak, validation, reset-only paths, structural changes, active_fee
  src/agent.rs               MOD  DiseaseId, immune/genome/diseases/infected_by, genome draw, effective metabolism
  src/world.rs               MOD  World::diseases, Infection events, outbreaks in step, fingerprint, endowment on placement
  src/rules/mod.rs           MOD  pub mod disease; E at the end of agent_turn
  src/rules/disease.rs       NEW  list/endowment, genome inheritance, immune response, transmission, outbreaks
  src/rules/lifecycle.rs     MOD  effective metabolism when burning
  src/rules/movement.rs      MOD  effective metabolism in multicommodity welfare
  src/rules/trade.rs         MOD  effective metabolism in Holdings
  src/rules/credit.rs        MOD  effective metabolism in income
  src/rules/sex.rs           MOD  child genome
  src/rules/replacement.rs   MOD  endow replacements
  src/stats.rs               MOD  4 new series; effective metabolism in supply_demand
  src/network.rs             MOD  disease_edges
  src/render.rs              MOD  Disease color mode
  src/edit.rs                MOD  inspection, disease_list, infect, vaccinate, endow placed agents
  src/export.rs              MOD  immune,diseases columns; series header
  src/presets.rs             MOD  v-1-rid, v-2-endemic, v-mcneill, vi-1-everything
  src/testkit.rs             MOD  new Agent fields
  tests/invariants.rs        MOD  disease in the strategy + invariants
  tests/book.rs              MOD  Chapter V reproductions
crates/sugarscape-wasm/src/lib.rs, tests/web.rs   MOD  networks("disease"), disease_list, infect, vaccinate
web/src/types.ts, schema.ts                        MOD
web/src/schedule.ts (+ schedule.test.ts)           NEW  schedule + outbreak display lines
web/src/ui/rules-panel.ts                          MOD  outbreaks in the Schedule section
web/src/engine.ts                                  MOD  disease overlay, diseaseList/infect/vaccinate
web/src/ui/display.ts, grid-view.ts                MOD  Disease color mode, Disease network overlay
web/src/ui/disease-picker.ts (+ .test.ts)          NEW  picker options
web/src/ui/tools.ts                                MOD  Infect and Vaccinate tools
web/src/ui/charts-panel.ts, style.css              MOD  Disease chart group
web/src/ui/inspect-panel.ts                        MOD  immune strings, diseases, infected-by
README.md, docs/roadmap.md                         MOD
```

---

### Task 1: Golden fingerprints for Chapter IV presets

**Files:**
- Modify: `crates/sugarscape-core/tests/golden.rs`

**Interfaces:**
- Consumes: `presets::{all, by_id}`, `World::{new, run, fingerprint}`.
- Produces: `GOLDEN` covering all 19 current presets; `earlier_presets_are_unchanged` (renamed from `milestone_one_presets_are_unchanged`) and `every_preset_has_a_golden_entry`. Every later task must keep this file passing; Task 13 appends the Chapter V entries.

This task comes before any core change.

- [ ] **Step 1: Add the coverage test (it fails: six presets have no entry)**

Replace the whole of `crates/sugarscape-core/tests/golden.rs` with:
```rust
//! Earlier runs are unchanged: with disease off, the milestone-1 and
//! Chapter IV presets evolve exactly as they did before Chapter V was added.

use sugarscape_core::presets;
use sugarscape_core::world::World;

/// (preset id, fingerprint after 200 ticks from seed 1).
const GOLDEN: &[(&str, u64)] = &[
    ("ii-1-instant", 0x63d4454975b85fc),
    ("ii-2-unit", 0x75b93943813545e4),
    ("ii-5-wealth", 0x47bb9f4000e59377),
    ("ii-6-waves", 0xc16aa48d70070a46),
    ("ii-7-seasons", 0x89e264519aec44ff),
    // Chapter IV: now scheduled (pollution at t=50, diffusion at t=100).
    ("ii-8-pollution", 0xfdd983512b708286),
    ("iii-2-sex", 0x99c33cc7e8ad705c),
    ("iii-4-inheritance", 0x1aa293d107d6b7fb),
    ("iii-6-culture", 0xf8973190b0435a81),
    ("iii-9-combat", 0xe71fa903dfae5e58),
    ("iii-11-combat-fixed", 0x4c4958c1a3e160e0),
    ("iii-12-collision", 0xfec4ea6a61dd15fc),
    ("iii-14-combat-culture", 0xf9e3a9dd87cda8e),
];

fn fingerprint(id: &str) -> u64 {
    let preset = presets::by_id(id).unwrap_or_else(|| panic!("unknown preset {id}"));
    let mut world = World::new(preset.config, 1).unwrap();
    world.run(200);
    world.fingerprint()
}

#[test]
fn earlier_presets_are_unchanged() {
    for &(id, expected) in GOLDEN {
        assert_eq!(fingerprint(id), expected, "preset {id} changed");
    }
}

#[test]
fn every_preset_has_a_golden_entry() {
    for p in presets::all() {
        assert!(
            GOLDEN.iter().any(|&(id, _)| id == p.id),
            "record a golden fingerprint for {} (run print_golden)",
            p.id
        );
    }
}

/// Prints `GOLDEN` entries: `cargo test -p sugarscape-core --test golden -- --ignored --nocapture`.
#[test]
#[ignore]
fn print_golden() {
    for p in presets::all() {
        println!("    (\"{}\", {:#x}),", p.id, fingerprint(p.id));
    }
}
```

- [ ] **Step 2: Run to verify the failure**

Run: `cargo test -p sugarscape-core --test golden`
Expected: `every_preset_has_a_golden_entry` FAILS with "record a golden fingerprint for iv-1-spice"; `earlier_presets_are_unchanged` passes.

- [ ] **Step 3: Record the Chapter IV values**

Run: `cargo test -p sugarscape-core --test golden -- --ignored --nocapture print_golden`
Append the six `iv-*` lines to `GOLDEN` after the milestone-1 entries, under a comment:
```rust
    // Chapter IV, recorded before any Chapter V change.
    ("iv-1-spice", 0xd937df7102a3a4),
    ("iv-3-trade", 0x6f14b0be4cd3b21e),
    ("iv-15-trade-sex", 0xda2f681086c8729b),
    ("iv-3-pollution", 0xa44cef03ce32f537),
    ("iv-18-foresight", 0x71bdcb5c44708373),
    ("iv-5-credit", 0xac5bbc30ed0fb306),
```
These values were measured on commit `1da914d`. The printed values are authoritative: if any differs, paste the printed one (the code has not changed yet, so it is what the code does) and mention it in the commit body. The 13 milestone-1 values printed must equal the existing entries; if they don't, stop — the tree is not at the expected state.

- [ ] **Step 4: Verify**

Run: `cargo test -p sugarscape-core --test golden`
Expected: 2 passed, 1 ignored.

- [ ] **Step 5: Commit**

```bash
cargo fmt --all && cargo clippy --all-targets -- -D warnings
git add crates/sugarscape-core/tests/golden.rs
git commit -m "Record Chapter IV golden fingerprints" -m "Claude-Session: https://claude.ai/code/session_01Kq7NyxbMrkNsPfAhnsVcK3"
```

---

### Task 2: Bit strings

**Files:**
- Create: `crates/sugarscape-core/src/bits.rs`
- Modify: `crates/sugarscape-core/src/lib.rs`

**Interfaces:**
- Produces: `bits::Bits` (`Clone, Copy, Debug, Default, PartialEq, Eq`) with `new(bits: u64, len: u32) -> Bits` (len ≤ 64, masks), `random(len: u32, rng: &mut impl Rng) -> Bits`, `parse(&str) -> Option<Bits>`, `len() -> u32`, `is_empty() -> bool`, `bits() -> u64`, `get(u32) -> bool`, `set(u32, bool)`, `flip(u32)`, `window(start: u32, len: u32) -> Bits`, `hamming(&Bits) -> u32`, `closest_window(&Bits) -> Option<(u32, u32)>` (start, distance; leftmost on ties; `None` if the argument is empty or longer), `contains(&Bits) -> bool`, `learn(&mut self, &Bits) -> bool` (one Appendix B flip; `false` if nothing to do), `imprint(&mut self, &Bits) -> bool` (overwrite the closest window), `to_bit_string() -> String`.

- [ ] **Step 1: Write the module with its tests, implementation stubbed**

Create `crates/sugarscape-core/src/bits.rs` containing only the tests below plus `pub struct Bits;` so the file compiles as far as the tests (they will fail to compile against the stub). Add `pub mod bits;` to `lib.rs` (alphabetically, before `pub mod config;`).

```rust
#[cfg(test)]
mod tests {
    use super::*;

    fn b(s: &str) -> Bits {
        Bits::parse(s).unwrap()
    }

    #[test]
    fn parses_and_prints_position_zero_first() {
        let x = b("0110");
        assert_eq!((x.len(), x.get(0), x.get(1), x.get(3)), (4, false, true, false));
        assert_eq!(x.to_bit_string(), "0110");
        assert_eq!(Bits::parse("01a"), None);
        assert_eq!(Bits::parse(&"1".repeat(65)), None);
        assert!(Bits::default().is_empty());
        assert_eq!(Bits::new(u64::MAX, 3).bits(), 0b111, "bits beyond the length are masked");
    }

    #[test]
    fn set_and_flip() {
        let mut x = b("000");
        x.set(0, true);
        x.flip(2);
        assert_eq!(x.to_bit_string(), "101");
        x.flip(2);
        assert_eq!(x.to_bit_string(), "100");
    }

    #[test]
    fn windows_and_hamming_distance() {
        assert_eq!(b("1011101001").window(3, 4).to_bit_string(), "1101");
        assert_eq!(b("10111").hamming(&b("10011")), 1);
        let full = Bits::new(u64::MAX, 64);
        assert_eq!(full.window(60, 4).to_bit_string(), "1111");
        assert!(full.window(64, 0).is_empty());
    }

    #[test]
    fn book_example_learns_the_disease_in_one_flip() {
        // Appendix B: immune 1011101001, disease 10011. The closest window is
        // at position 0 (10111, distance 1); flipping its first differing bit
        // (position 2) gives 1001101001, which contains the disease.
        let mut immune = b("1011101001");
        let disease = b("10011");
        assert_eq!(immune.closest_window(&disease), Some((0, 1)));
        assert!(!immune.contains(&disease));
        assert!(immune.learn(&disease));
        assert_eq!(immune.to_bit_string(), "1001101001");
        assert!(immune.contains(&disease));
        assert!(!immune.learn(&disease), "nothing left to learn");
    }

    #[test]
    fn ties_go_to_the_leftmost_window() {
        // Windows 01, 10, 01 are all at distance 1 from 11.
        let mut immune = b("0101");
        assert_eq!(immune.closest_window(&b("11")), Some((0, 1)));
        assert!(immune.learn(&b("11")));
        assert_eq!(immune.to_bit_string(), "1101");
    }

    #[test]
    fn diseases_longer_than_the_immune_string_are_never_learned() {
        let mut immune = b("01");
        assert_eq!(immune.closest_window(&b("010")), None);
        assert!(!immune.contains(&b("010")));
        assert!(!immune.learn(&b("010")));
        assert!(!Bits::default().contains(&b("1")));
    }

    #[test]
    fn imprint_writes_the_disease_over_the_closest_window() {
        // Windows of 000110 against 111: 000 (3), 001 (2), 011 (1), 110 (1).
        let mut immune = b("000110");
        assert!(immune.imprint(&b("111")));
        assert_eq!(immune.to_bit_string(), "001110");
        assert!(immune.contains(&b("111")));
        let mut short = b("01");
        assert!(!short.imprint(&b("010")));
    }
}
```

- [ ] **Step 2: Run to verify failure**

Run: `cargo test -p sugarscape-core bits`
Expected: FAIL to compile — `parse`, `get`, … not found on the stub.

- [ ] **Step 3: Implement** (replace the stub, keep the tests at the bottom)

```rust
//! Fixed-length bit strings for Chapter V's immune systems and diseases.
//! Bit `i` is string position `i`; strings print position 0 first. Culture
//! tags keep their own `Tags` type.

use rand::Rng;

/// A string of `len` bits, 0 ≤ len ≤ 64. Length 0 is the empty string agents
/// hold while disease is off.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Bits {
    bits: u64,
    len: u32,
}

fn mask(len: u32) -> u64 {
    if len == 64 {
        u64::MAX
    } else {
        (1u64 << len) - 1
    }
}

impl Bits {
    pub fn new(bits: u64, len: u32) -> Self {
        assert!(len <= 64, "bit string length must be ≤ 64");
        Self {
            bits: bits & mask(len),
            len,
        }
    }

    pub fn random(len: u32, rng: &mut impl Rng) -> Self {
        Self::new(rng.gen(), len)
    }

    /// Parses `0`/`1` characters, position 0 first; `None` for any other
    /// character or more than 64 of them.
    pub fn parse(s: &str) -> Option<Self> {
        let mut bits = 0u64;
        let mut len = 0u32;
        for c in s.chars() {
            if len == 64 {
                return None;
            }
            match c {
                '0' => {}
                '1' => bits |= 1 << len,
                _ => return None,
            }
            len += 1;
        }
        Some(Self::new(bits, len))
    }

    pub fn len(&self) -> u32 {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn bits(&self) -> u64 {
        self.bits
    }

    fn check(&self, i: u32) {
        assert!(i < self.len, "bit index {i} out of range [0, {})", self.len);
    }

    pub fn get(&self, i: u32) -> bool {
        self.check(i);
        (self.bits >> i) & 1 == 1
    }

    pub fn set(&mut self, i: u32, value: bool) {
        self.check(i);
        if value {
            self.bits |= 1 << i;
        } else {
            self.bits &= !(1 << i);
        }
    }

    pub fn flip(&mut self, i: u32) {
        self.check(i);
        self.bits ^= 1 << i;
    }

    /// The `len` bits starting at position `start`.
    pub fn window(&self, start: u32, len: u32) -> Bits {
        assert!(
            start + len <= self.len,
            "window [{start}, {}) out of range [0, {})",
            start + len,
            self.len
        );
        Bits::new(self.bits.checked_shr(start).unwrap_or(0), len)
    }

    /// Number of positions at which two equal-length strings differ.
    pub fn hamming(&self, other: &Bits) -> u32 {
        assert_eq!(self.len, other.len, "Hamming distance needs equal lengths");
        (self.bits ^ other.bits).count_ones()
    }

    /// The window of this string closest to `d` in Hamming distance (leftmost
    /// on ties) as `(start, distance)`; `None` if `d` is empty or longer.
    pub fn closest_window(&self, d: &Bits) -> Option<(u32, u32)> {
        if d.is_empty() || d.len > self.len {
            return None;
        }
        let mut best: Option<(u32, u32)> = None;
        for start in 0..=self.len - d.len {
            let distance = self.window(start, d.len).hamming(d);
            if best.is_none_or(|(_, b)| distance < b) {
                best = Some((start, distance));
                if distance == 0 {
                    break;
                }
            }
        }
        best
    }

    /// Whether `d` occurs in this string (the book's "substring").
    pub fn contains(&self, d: &Bits) -> bool {
        matches!(self.closest_window(d), Some((_, 0)))
    }

    /// One step of Appendix B's immune response: unless `d` already occurs,
    /// flip the first bit (lowest position) where the closest window differs
    /// from `d`. Returns whether a bit was flipped.
    pub fn learn(&mut self, d: &Bits) -> bool {
        match self.closest_window(d) {
            Some((start, distance)) if distance > 0 => {
                let first = (self.window(start, d.len).bits ^ d.bits).trailing_zeros();
                self.flip(start + first);
                true
            }
            _ => false,
        }
    }

    /// Vaccination: overwrites the closest window with `d`, so `d` now occurs.
    /// Returns `false` (and changes nothing) if `d` does not fit.
    pub fn imprint(&mut self, d: &Bits) -> bool {
        let Some((start, _)) = self.closest_window(d) else {
            return false;
        };
        for i in 0..d.len {
            self.set(start + i, d.get(i));
        }
        true
    }

    /// Position 0 first.
    pub fn to_bit_string(&self) -> String {
        (0..self.len)
            .map(|i| if self.get(i) { '1' } else { '0' })
            .collect()
    }
}
```

- [ ] **Step 4: Verify**

Run: `cargo test -p sugarscape-core bits` — 7 passed. Then `cargo test -p sugarscape-core` — all pass.

- [ ] **Step 5: Commit**

```bash
cargo fmt --all && cargo clippy --all-targets -- -D warnings
git add crates/sugarscape-core/src/bits.rs crates/sugarscape-core/src/lib.rs
git commit -m "Add bit strings for immune systems and diseases" -m "Claude-Session: https://claude.ai/code/session_01Kq7NyxbMrkNsPfAhnsVcK3"
```

---

### Task 3: Disease configuration

**Files:**
- Modify: `crates/sugarscape-core/src/config.rs`

**Interfaces:**
- Produces: `config::Outbreak { tick: u64, agents: u32 }` (`Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize`); `config::DiseaseRule { enabled: bool, count: u32, length: URange, initial: u32, immune_length: u32, fee: f64, flips_per_tick: u32, genome_mutation: f64, disease_mutation: f64, outbreaks: Vec<Outbreak> }` (`Clone, Debug, PartialEq, Serialize, Deserialize`, `#[serde(default)]`, `Default` = Animation V-1); `Config.disease: DiseaseRule` (after `foresight`, before `schedule`); `RESET_ONLY_PATHS` now 9 entries; schedule rejects `disease.outbreaks`; `structural_changes` reports `disease.enabled`, `disease.count`, `disease.length`, `disease.immune_length`.

- [ ] **Step 1: Write failing tests** (append to `config.rs` tests)

```rust
    #[test]
    fn disease_defaults_are_animation_v1_and_off() {
        let d = Config::default().disease;
        assert!(!d.enabled);
        assert_eq!(
            (d.count, d.length, d.initial, d.immune_length),
            (10, URange::new(1, 10), 4, 50)
        );
        assert_eq!(
            (d.fee, d.flips_per_tick, d.genome_mutation, d.disease_mutation),
            (1.0, 1, 0.0, 0.0)
        );
        assert!(d.outbreaks.is_empty());
        let partial = Config::from_json(r#"{"disease":{"enabled":true}}"#).unwrap();
        assert!(partial.disease.enabled);
        assert_eq!(partial.disease.count, 10, "missing disease fields default");
    }

    #[test]
    fn disease_parameters_are_validated() {
        let with = |f: fn(&mut DiseaseRule)| {
            let mut c = Config::default();
            f(&mut c.disease);
            fields(c.validate())
        };
        let has = |errs: Vec<String>, field: &str| errs.contains(&field.to_string());
        assert!(has(with(|d| d.immune_length = 0), "disease.immune_length"));
        assert!(has(with(|d| d.immune_length = 65), "disease.immune_length"));
        assert!(has(with(|d| d.length = URange::new(0, 5)), "disease.length"));
        assert!(has(with(|d| d.length = URange::new(6, 5)), "disease.length"));
        assert!(
            has(with(|d| d.length = URange::new(1, 50)), "disease.length"),
            "diseases must be shorter than the 50-bit immune string"
        );
        assert!(has(with(|d| d.count = 0), "disease.count"));
        assert!(has(with(|d| d.count = 1001), "disease.count"));
        assert!(has(with(|d| d.initial = 11), "disease.initial"));
        assert!(has(with(|d| d.fee = -1.0), "disease.fee"));
        assert!(has(with(|d| d.flips_per_tick = 0), "disease.flips_per_tick"));
        assert!(has(with(|d| d.genome_mutation = 1.5), "disease.genome_mutation"));
        assert!(has(with(|d| d.disease_mutation = f64::NAN), "disease.disease_mutation"));
        assert!(has(
            with(|d| d.outbreaks = vec![Outbreak { tick: 0, agents: 5 }]),
            "disease.outbreaks"
        ));
        assert!(has(
            with(|d| d.outbreaks = vec![Outbreak { tick: 3, agents: 0 }]),
            "disease.outbreaks"
        ));
        assert!(with(|d| {
            d.enabled = true;
            d.genome_mutation = 0.01;
            d.disease_mutation = 1.0;
            d.outbreaks = vec![Outbreak { tick: 300, agents: 5 }];
        })
        .is_empty());
    }

    #[test]
    fn schedule_may_not_restructure_disease_or_set_outbreaks() {
        let rejected = |path: &str, value: serde_json::Value| {
            let c = Config {
                schedule: vec![change(5, path, value)],
                ..Default::default()
            };
            let errs = c.validate().unwrap_err();
            assert_eq!(errs[0].field, "schedule", "{path}");
            errs[0].message.clone()
        };
        for (path, value) in [
            ("disease.enabled", serde_json::json!(true)),
            ("disease.count", serde_json::json!(20)),
            ("disease.immune_length", serde_json::json!(40)),
            ("disease.length", serde_json::json!({"min": 1, "max": 5})),
            ("disease.length.max", serde_json::json!(5)),
        ] {
            let msg = rejected(path, value);
            assert!(msg.contains("only on reset"), "{path}: {msg}");
        }
        let disease = serde_json::to_value(Config::default().disease).unwrap();
        rejected("disease", disease);
        let msg = rejected("disease.outbreaks", serde_json::json!([]));
        assert!(msg.contains("their own schedule"), "{msg}");
        // Live knobs may be scheduled.
        let c = Config {
            schedule: vec![
                change(5, "disease.fee", serde_json::json!(2.0)),
                change(6, "disease.flips_per_tick", serde_json::json!(3)),
            ],
            ..Default::default()
        };
        c.validate().unwrap();
    }

    #[test]
    fn disease_structure_changes_only_on_reset() {
        let a = Config::default();
        let changed = |f: fn(&mut DiseaseRule)| {
            let mut b = a.clone();
            f(&mut b.disease);
            a.structural_changes(&b)
                .into_iter()
                .map(|e| e.field)
                .collect::<Vec<_>>()
        };
        assert_eq!(changed(|d| d.enabled = true), vec!["disease.enabled"]);
        assert_eq!(changed(|d| d.count = 20), vec!["disease.count"]);
        assert_eq!(changed(|d| d.length = URange::new(2, 8)), vec!["disease.length"]);
        assert_eq!(changed(|d| d.immune_length = 40), vec!["disease.immune_length"]);
        assert!(changed(|d| {
            d.initial = 2;
            d.fee = 2.0;
            d.flips_per_tick = 3;
            d.genome_mutation = 0.1;
            d.disease_mutation = 0.1;
            d.outbreaks = vec![Outbreak { tick: 9, agents: 1 }];
        })
        .is_empty());
    }
```

- [ ] **Step 2: Run to verify failure**

Run: `cargo test -p sugarscape-core config`
Expected: FAIL to compile — `DiseaseRule`, `Outbreak`, `Config.disease` not found.

- [ ] **Step 3: Implement**

Add after `Foresight`:
```rust
/// A novel disease appearing mid-run (Chapter V's McNeill scenario): at the
/// start of the tick when `World::tick == tick`, a brand-new random disease
/// infects `agents` random living agents (all of them if there are fewer).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Outbreak {
    pub tick: u64,
    pub agents: u32,
}

/// Rule E (Chapter V, Appendix B): immune response and disease transmission.
/// The defaults are Animation V-1's.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct DiseaseRule {
    pub enabled: bool,
    /// Size of the initial master list of diseases.
    pub count: u32,
    /// Disease string lengths.
    pub length: URange,
    /// Distinct random diseases given to each new (not newborn) agent.
    pub initial: u32,
    /// Immune string length (1–64).
    pub immune_length: u32,
    /// Extra metabolism of each good per carried disease.
    pub fee: f64,
    /// Immune bits flipped per carried disease per tick ("medicine").
    pub flips_per_tick: u32,
    /// Per-bit mutation probability of a child's immune genome.
    pub genome_mutation: f64,
    /// Probability that a transmitted disease mutates one random bit.
    pub disease_mutation: f64,
    pub outbreaks: Vec<Outbreak>,
}

impl Default for DiseaseRule {
    fn default() -> Self {
        Self {
            enabled: false,
            count: 10,
            length: URange::new(1, 10),
            initial: 4,
            immune_length: 50,
            fee: 1.0,
            flips_per_tick: 1,
            genome_mutation: 0.0,
            disease_mutation: 0.0,
            outbreaks: Vec::new(),
        }
    }
}
```

Replace `RESET_ONLY_PATHS` and its comment:
```rust
/// Paths a schedule may not set because they switch a rule that shapes every
/// agent's traits (spice, disease) or fix the disease list and immune strings;
/// changing them needs a reset.
pub const RESET_ONLY_PATHS: [&str; 9] = [
    "spice",
    "spice.enabled",
    "disease",
    "disease.enabled",
    "disease.count",
    "disease.length",
    "disease.length.min",
    "disease.length.max",
    "disease.immune_length",
];
```

In `Config` add `pub disease: DiseaseRule,` after `pub foresight: Foresight,`; in `Default for Config` add `disease: DiseaseRule::default(),` after `foresight`.

Add to `impl Errors`:
```rust
    fn probability(&mut self, v: f64, field: &str) {
        self.check(
            (0.0..=1.0).contains(&v),
            field,
            "must be a probability between 0 and 1",
        );
    }
```

In `validate_fields`, before `e.finish()`:
```rust
        let d = &self.disease;
        e.check(
            (1..=64).contains(&d.immune_length),
            "disease.immune_length",
            "must be between 1 and 64",
        );
        e.range(d.length, "disease.length");
        e.check(d.length.min >= 1, "disease.length", "diseases are at least 1 bit long");
        e.check(
            d.length.max < d.immune_length,
            "disease.length",
            "diseases must be shorter than the immune string",
        );
        e.check(
            (1..=1000).contains(&d.count),
            "disease.count",
            "must be between 1 and 1000",
        );
        e.check(
            d.initial <= d.count,
            "disease.initial",
            "cannot exceed the number of diseases",
        );
        e.non_negative(d.fee, "disease.fee");
        e.check(d.flips_per_tick >= 1, "disease.flips_per_tick", "must be ≥ 1");
        e.probability(d.genome_mutation, "disease.genome_mutation");
        e.probability(d.disease_mutation, "disease.disease_mutation");
        e.check(
            d.outbreaks.iter().all(|o| o.tick >= 1 && o.agents >= 1),
            "disease.outbreaks",
            "each outbreak needs tick ≥ 1 and at least 1 agent",
        );
```

In `apply_change`, after the `root == "schedule"` check, add:
```rust
            if path == "disease.outbreaks" || path.starts_with("disease.outbreaks.") {
                return Err(FieldError::new(
                    "schedule",
                    format!("{path}: outbreaks are their own schedule"),
                ));
            }
```

In `structural_changes`, before `out`:
```rust
        let (a, b) = (&self.disease, &next.disease);
        if a.enabled != b.enabled {
            out.push(FieldError::new("disease.enabled", msg));
        }
        if a.count != b.count {
            out.push(FieldError::new("disease.count", msg));
        }
        if a.length != b.length {
            out.push(FieldError::new("disease.length", msg));
        }
        if a.immune_length != b.immune_length {
            out.push(FieldError::new("disease.immune_length", msg));
        }
```
and update its doc comment to "…or — for spice and disease — every agent's traits and the disease list".

- [ ] **Step 4: Verify**

Run: `cargo test -p sugarscape-core`
Expected: all pass, including `tests/golden.rs` (configuration only).

- [ ] **Step 5: Commit**

```bash
cargo fmt --all && cargo clippy --all-targets -- -D warnings
git add crates/sugarscape-core/src/config.rs
git commit -m "Add disease configuration" -m "Claude-Session: https://claude.ai/code/session_01Kq7NyxbMrkNsPfAhnsVcK3"
```

---

### Task 4: Disease state on agents and the world

**Files:**
- Create: `crates/sugarscape-core/src/rules/disease.rs`
- Modify: `src/agent.rs`, `src/world.rs`, `src/testkit.rs`, `src/rules/mod.rs`, `src/rules/sex.rs`, `src/rules/replacement.rs`, `src/edit.rs`

**Interfaces:**
- Consumes: `Bits` (Task 2), `DiseaseRule` (Task 3).
- Produces: `agent::DiseaseId = u32`; `Agent.{immune_genome: Bits, immune: Bits, diseases: Vec<DiseaseId>, infected_by: Option<AgentId>}` (after `income`); `Agent::random` draws `immune_genome` (and copies it to `immune`) only when `disease.enabled`, after all other draws; `World.diseases: Vec<Bits>` (pub; id = index), drawn at creation when disease is on, before agents are placed; `rules::disease::{random_disease(URange, &mut SimRng) -> Bits, initial_list(&DiseaseRule, &mut SimRng) -> Vec<Bits>, endow(&mut World, &mut Agent)}` (all `pub(crate)`); `endow` is applied to initial, replacement and hand-placed agents; `fingerprint()` hashes disease state when disease is on; `testkit::spawn` gives all-zero immune strings of `config.disease.immune_length` bits.

- [ ] **Step 1: Write failing tests**

`agent.rs` tests:
```rust
    #[test]
    fn immune_genomes_are_drawn_only_when_disease_is_on() {
        use crate::config::Config;
        use crate::rng::seeded;
        let off = Agent::random(&Config::default(), Pos::new(0, 0), 0, &mut seeded(4));
        assert!(off.immune.is_empty() && off.immune_genome.is_empty());
        assert!(off.diseases.is_empty() && off.infected_by.is_none());
        let mut on = Config::default();
        on.disease.enabled = true;
        let a = Agent::random(&on, Pos::new(0, 0), 0, &mut seeded(4));
        assert_eq!(a.immune.len(), 50);
        assert_eq!(a.immune, a.immune_genome, "the phenotype starts untrained");
        assert_eq!(
            (a.vision, a.metabolism, a.sugar, a.tags),
            (off.vision, off.metabolism, off.sugar, off.tags),
            "the genome is drawn after the existing traits"
        );
        assert!(a.diseases.is_empty(), "diseases come from the world's list");
    }
```

`world.rs` tests:
```rust
    #[test]
    fn disease_worlds_draw_a_list_and_endow_agents() {
        let mut c = Config::default();
        c.disease.enabled = true;
        let w = World::new(c, 1).unwrap();
        assert_eq!(w.diseases.len(), 10);
        assert!(w.diseases.iter().all(|d| (1..=10).contains(&d.len())));
        let mut carried = 0;
        for a in w.agents() {
            assert_eq!(a.immune.len(), 50);
            assert!(a.diseases.len() <= 4);
            let mut ids = a.diseases.clone();
            ids.sort_unstable();
            ids.dedup();
            assert_eq!(ids.len(), a.diseases.len(), "distinct diseases");
            for &d in &a.diseases {
                assert!(
                    !a.immune.contains(&w.diseases[d as usize]),
                    "never starts with a disease it is immune to"
                );
            }
            carried += a.diseases.len();
        }
        assert!(carried > 0, "some agents start sick");
        assert!(World::new(Config::default(), 1).unwrap().diseases.is_empty());
    }

    #[test]
    fn fingerprint_covers_disease_state_only_when_disease_is_on() {
        let mut w = crate::testkit::blank_world(5, 5);
        let id = crate::testkit::spawn(&mut w, 1, 1);
        let before = w.fingerprint();
        w.agent_mut(id).unwrap().diseases.push(0);
        w.agent_mut(id).unwrap().immune.flip(0);
        assert_eq!(w.fingerprint(), before, "ignored while disease is off");
        w.config.disease.enabled = true;
        let on = w.fingerprint();
        w.agent_mut(id).unwrap().immune.flip(1);
        assert_ne!(w.fingerprint(), on, "immune strings are hashed");
        let on = w.fingerprint();
        w.diseases.push(crate::bits::Bits::parse("101").unwrap());
        assert_ne!(w.fingerprint(), on, "the disease list is hashed");
    }
```

`replacement.rs` tests:
```rust
    #[test]
    fn replacements_get_immune_systems_when_disease_is_on() {
        let mut w = replacing_world();
        w.config.disease.enabled = true;
        w.diseases = vec![crate::bits::Bits::parse("1111111111").unwrap()];
        let id = spawn(&mut w, 2, 2);
        w.agent_mut(id).unwrap().metabolism = 50;
        w.step();
        let newcomer = w.agents().next().unwrap();
        assert_eq!(newcomer.immune.len(), 50);
        assert_eq!(newcomer.immune, newcomer.immune_genome);
        assert!(newcomer.diseases.len() <= 1);
        if let Some(&d) = newcomer.diseases.first() {
            assert!(!newcomer.immune.contains(&w.diseases[d as usize]));
        }
    }
```

`edit.rs` tests:
```rust
    #[test]
    fn placed_agents_get_immune_systems_when_disease_is_on() {
        let mut w = blank_world(10, 10);
        w.config.disease.enabled = true;
        w.diseases = vec![crate::bits::Bits::parse("1111111111").unwrap()];
        let id = w.place_agent(3, 3, &AgentOverrides::default()).unwrap();
        let a = w.agent(id).unwrap();
        assert_eq!(a.immune.len(), 50);
        assert_eq!(a.immune, a.immune_genome);
        assert!(a.diseases.len() <= 1, "initial 4 is capped by the list's length");
    }
```

- [ ] **Step 2: Run to verify failure**

Run: `cargo test -p sugarscape-core`
Expected: FAIL to compile — `immune`, `diseases`, `World::diseases` not found.

- [ ] **Step 3: Implement**

`agent.rs`: add `use crate::bits::Bits;`, and after `pub type AgentId = u64;`:
```rust
/// An index into `World::diseases`.
pub type DiseaseId = u32;
```
Append to `Agent` (after `income`):
```rust
    /// Chapter V: the inherited, untrained immune string (empty while disease
    /// is off).
    pub immune_genome: Bits,
    /// The trained immune string; starts as a copy of the genome.
    pub immune: Bits,
    /// Diseases currently carried: distinct indices into `World::diseases`.
    pub diseases: Vec<DiseaseId>,
    /// The agent that most recently infected this one.
    pub infected_by: Option<AgentId>,
```
In `Agent::random`, add to the struct literal `immune_genome: Bits::default(), immune: Bits::default(), diseases: Vec::new(), infected_by: None,` and, after the foresight block:
```rust
        if config.disease.enabled {
            let genome = Bits::random(config.disease.immune_length, rng);
            agent.immune_genome = genome;
            agent.immune = genome;
        }
```

`testkit.rs` `spawn`: add to the literal
```rust
        immune_genome: Bits::new(0, world.config.disease.immune_length),
        immune: Bits::new(0, world.config.disease.immune_length),
        diseases: Vec::new(),
        infected_by: None,
```
with `use crate::bits::Bits;`; update its doc comment to "…all-zero tags (Blue) and all-zero immune strings, no diseases."

`sex.rs` `birth`: add to the child literal (Task 5 fills the genome in)
```rust
        immune_genome: Bits::default(),
        immune: Bits::default(),
        diseases: Vec::new(),
        infected_by: None,
```
with `use crate::bits::Bits;`.

Create `crates/sugarscape-core/src/rules/disease.rs`:
```rust
//! Rule E (Chapter V, Appendix B): immune response and disease transmission,
//! plus the disease list, new agents' diseases, genome inheritance and
//! outbreaks. Nothing here runs, or draws random numbers, while disease is off.

use crate::agent::{Agent, DiseaseId};
use crate::bits::Bits;
use crate::config::{DiseaseRule, URange};
use crate::rng::SimRng;
use crate::world::World;

/// A random disease with its length drawn from `length`.
pub(crate) fn random_disease(length: URange, rng: &mut SimRng) -> Bits {
    let len = length.sample(rng);
    Bits::random(len, rng)
}

/// The initial master list: `count` independent random diseases.
pub(crate) fn initial_list(rule: &DiseaseRule, rng: &mut SimRng) -> Vec<Bits> {
    (0..rule.count)
        .map(|_| random_disease(rule.length, rng))
        .collect()
}

/// Gives a new (not newborn) agent `initial` distinct random diseases from the
/// list, skipping any it is already immune to.
pub(crate) fn endow(world: &mut World, agent: &mut Agent) {
    if !world.config.disease.enabled {
        return;
    }
    let n = world.diseases.len();
    let k = (world.config.disease.initial as usize).min(n);
    for i in rand::seq::index::sample(&mut world.rng, n, k).into_vec() {
        if !agent.immune.contains(&world.diseases[i]) {
            agent.diseases.push(i as DiseaseId);
        }
    }
}
```
Add `pub mod disease;` to `rules/mod.rs` (alphabetically after `pub mod culture;`).

`world.rs`:
- add `use crate::bits::Bits;`;
- add the field `pub diseases: Vec<Bits>,` to `World` after `landscape_edited`, documented `/// Chapter V's master list of diseases; a disease's id is its index.`;
- in `with_capacities` add `diseases: Vec::new(),` to the literal and, just before `world.populate();`:
```rust
        if world.config.disease.enabled {
            world.diseases = rules::disease::initial_list(&world.config.disease, &mut world.rng);
        }
```
- in `place`, between the tribe override and `insert_agent`:
```rust
            rules::disease::endow(self, &mut agent);
```
- in `fingerprint`, add `let disease = self.config.disease.enabled;` beside `spice`/`foresight`; in the agent loop after the foresight block:
```rust
            if disease {
                eat(u64::from(a.immune.len()));
                eat(a.immune.bits());
                eat(a.diseases.len() as u64);
                for &d in &a.diseases {
                    eat(u64::from(d));
                }
            }
```
and after the agent loop (before the loans loop):
```rust
        if disease {
            for d in &self.diseases {
                eat(u64::from(d.len()));
                eat(d.bits());
            }
        }
```

`replacement.rs` `apply`: after `agent.tags = agent.tags.forced_to(tribe);` add `crate::rules::disease::endow(world, &mut agent);`.

`edit.rs` `place_agent`: after the tribe override, before `self.insert_agent(agent)`, add `crate::rules::disease::endow(self, &mut agent);`.

- [ ] **Step 4: Verify**

Run: `cargo test -p sugarscape-core`
Expected: all pass, including `tests/golden.rs` (no draws or hashing while disease is off).

- [ ] **Step 5: Commit**

```bash
cargo fmt --all && cargo clippy --all-targets -- -D warnings
git add crates/sugarscape-core/src
git commit -m "Give agents immune strings and the world a disease list" -m "Claude-Session: https://claude.ai/code/session_01Kq7NyxbMrkNsPfAhnsVcK3"
```

---

### Task 5: Immune genome inheritance

**Files:**
- Modify: `crates/sugarscape-core/src/rules/disease.rs`, `crates/sugarscape-core/src/rules/sex.rs`

**Interfaces:**
- Consumes: `Agent.immune_genome` (Task 4).
- Produces: `rules::disease::inherit_genome(a: &Bits, b: &Bits, mutation: f64, rng: &mut SimRng) -> Bits` (`pub(crate)`): each bit is the parents' shared value or, where they differ, a random parent's; then each bit flips with probability `mutation` (no draws when `mutation == 0.0`). Newborns get it as `immune_genome` and `immune`, and start healthy — only when disease is on, after all existing birth draws.

- [ ] **Step 1: Write failing tests**

`disease.rs` — add a test module:
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::rng::seeded;

    fn b(s: &str) -> Bits {
        Bits::parse(s).unwrap()
    }

    #[test]
    fn child_genomes_keep_what_the_parents_share() {
        let mut rng = seeded(3);
        let (a, p) = (b("0000011111"), b("0101010101"));
        let mut saw_both = [false; 2];
        for _ in 0..40 {
            let g = inherit_genome(&a, &p, 0.0, &mut rng);
            for i in 0..10 {
                if a.get(i) == p.get(i) {
                    assert_eq!(g.get(i), a.get(i), "position {i}");
                }
            }
            saw_both[usize::from(g.get(1))] = true;
        }
        assert_eq!(saw_both, [true, true], "differing positions come from either parent");
    }

    #[test]
    fn genome_mutation_flips_bits() {
        let a = b("0011");
        assert_eq!(inherit_genome(&a, &a, 0.0, &mut seeded(1)), a);
        assert_eq!(inherit_genome(&a, &a, 1.0, &mut seeded(1)), b("1100"));
    }
}
```

`sex.rs` tests:
```rust
    #[test]
    fn children_inherit_an_immune_genome_and_start_healthy() {
        use crate::bits::Bits;
        let mut w = blank_world(10, 10);
        w.config.disease.enabled = true;
        let (mom, dad) = couple(&mut w);
        let genome = Bits::parse(&"10".repeat(25)).unwrap();
        for id in [mom, dad] {
            let a = w.agent_mut(id).unwrap();
            a.immune_genome = genome;
            a.immune = Bits::new(0, 50); // trained phenotypes are not inherited
            a.diseases = vec![0];
        }
        act(&mut w, mom);
        let child = w.agents().find(|a| a.parents.is_some()).unwrap();
        assert_eq!(child.immune_genome, genome, "parents agree everywhere; no mutation");
        assert_eq!(child.immune, genome, "the phenotype starts untrained");
        assert!(child.diseases.is_empty() && child.infected_by.is_none());
    }
```

- [ ] **Step 2: Run to verify failure**

Run: `cargo test -p sugarscape-core disease sex`
Expected: FAIL to compile — `inherit_genome` not found.

- [ ] **Step 3: Implement**

`disease.rs` (add `use rand::Rng;`):
```rust
/// A child's immune genome: the parents' shared bits, a random parent's where
/// they differ (as culture tags), then each bit flipped with probability
/// `mutation`.
pub(crate) fn inherit_genome(a: &Bits, b: &Bits, mutation: f64, rng: &mut SimRng) -> Bits {
    assert_eq!(a.len(), b.len(), "parents' genomes differ in length");
    let mut genome = *a;
    for i in 0..genome.len() {
        if a.get(i) != b.get(i) && rng.gen_bool(0.5) {
            genome.set(i, b.get(i));
        }
    }
    if mutation > 0.0 {
        for i in 0..genome.len() {
            if rng.gen_bool(mutation) {
                genome.flip(i);
            }
        }
    }
    genome
}
```

`sex.rs` `birth`, after the foresight block (`rng` is still `&mut world.rng`):
```rust
    if world.config.disease.enabled {
        let genome = crate::rules::disease::inherit_genome(
            &a.immune_genome,
            &b.immune_genome,
            world.config.disease.genome_mutation,
            rng,
        );
        child.immune_genome = genome;
        child.immune = genome;
    }
```
Extend the module doc comment: "With disease (E) on, the child's immune genome crosses over like the tags (then mutates per `disease.genome_mutation`); it starts untrained and healthy."

- [ ] **Step 4: Verify** — `cargo test -p sugarscape-core` (golden unchanged: the new draws are gated).

- [ ] **Step 5: Commit**

```bash
cargo fmt --all && cargo clippy --all-targets -- -D warnings
git add crates/sugarscape-core/src/rules/disease.rs crates/sugarscape-core/src/rules/sex.rs
git commit -m "Inherit immune genomes at birth" -m "Claude-Session: https://claude.ai/code/session_01Kq7NyxbMrkNsPfAhnsVcK3"
```

---

### Task 6: Effective metabolism

**Files:**
- Modify: `src/config.rs`, `src/agent.rs`, `src/rules/lifecycle.rs`, `src/rules/movement.rs`, `src/rules/trade.rs`, `src/rules/credit.rs`, `src/stats.rs`

**Interfaces:**
- Consumes: `Agent.diseases` (Task 4).
- Produces: `DiseaseRule::active_fee(&self) -> f64` (`fee` while enabled, else `0.0`); `Agent::effective_metabolism(&self, fee: f64) -> f64` and `Agent::effective_spice_metabolism(&self, fee: f64) -> f64` (base + `fee × diseases.len()`). They replace `f64::from(metabolism)` / `f64::from(spice_metabolism)` in `lifecycle::metabolize`, `movement::act_two_goods`, `trade::Holdings::of`, `credit::record_income` and `stats::supply_demand`. The `mean_metabolism` series keeps the genetic trait.

- [ ] **Step 1: Write failing tests**

`lifecycle.rs` tests:
```rust
    #[test]
    fn each_carried_disease_adds_the_fee_to_both_metabolisms() {
        let mut w = blank_world(5, 5);
        w.config.spice.enabled = true;
        w.config.disease.enabled = true;
        w.config.disease.fee = 1.5;
        let id = spawn(&mut w, 2, 2);
        {
            let a = w.agent_mut(id).unwrap();
            a.metabolism = 1;
            a.spice_metabolism = 2;
            a.diseases = vec![0, 3];
        }
        metabolize(&mut w, id, crate::rules::Harvest::default());
        let a = w.agent(id).unwrap();
        assert_eq!((a.sugar, a.spice), (10.0 - 4.0, 10.0 - 5.0));
        w.config.disease.enabled = false;
        metabolize(&mut w, id, crate::rules::Harvest::default());
        assert_eq!(w.agent(id).unwrap().sugar, 6.0 - 1.0, "no fee while disease is off");
    }
```

`movement.rs` tests:
```rust
    #[test]
    fn disease_fees_shift_the_two_good_welfare_weights() {
        // Holding 50 sugar and 5 spice with metabolisms (9, 1), welfare weights
        // are 0.9/0.1 and 5 more sugar beats 5 more spice. Four diseases at a
        // fee of 2 make the metabolisms (17, 9): weights 17/26 and 9/26, and
        // the spice site wins.
        let target = |sick: bool| {
            let mut w = blank_world(11, 11);
            let id = spicy(&mut w, 1);
            {
                let a = w.agent_mut(id).unwrap();
                (a.sugar, a.spice, a.metabolism, a.spice_metabolism) = (50.0, 5.0, 9, 1);
                if sick {
                    a.diseases = vec![0, 1, 2, 3];
                }
            }
            w.config.disease.enabled = true;
            w.config.disease.fee = 2.0;
            set_sugar(&mut w, 5, 6, 5.0);
            w.site_mut(Pos::new(6, 5)).spice = 5.0;
            act(&mut w, id);
            w.agent(id).unwrap().pos
        };
        assert_eq!(target(false), Pos::new(5, 6));
        assert_eq!(target(true), Pos::new(6, 5));
    }
```

`trade.rs` tests:
```rust
    #[test]
    fn disease_fees_change_valuations() {
        // Both agents hold (100, 100) with metabolisms (1, 3): equal MRSs of
        // 1/3. Two diseases at fee 1 make A's metabolisms (3, 5), MRS 3/5, so
        // A now values sugar more and buys it.
        let mut w = blank_world(5, 5);
        w.config.spice.enabled = true;
        let a = trader(&mut w, 0, 100.0, 100.0);
        let b = trader(&mut w, 1, 100.0, 100.0);
        for id in [a, b] {
            w.agent_mut(id).unwrap().spice_metabolism = 3;
        }
        w.agent_mut(a).unwrap().diseases = vec![0, 1];
        trade_pair(&mut w, a, b);
        assert!(w.events().trades.is_empty(), "fees ignored while disease is off");
        w.config.disease.enabled = true;
        trade_pair(&mut w, a, b);
        assert!(!w.events().trades.is_empty());
        assert_eq!(w.events().trades[0].buyer, a);
    }
```

`credit.rs` tests:
```rust
    #[test]
    fn income_counts_the_disease_fee() {
        let mut w = credit_world();
        w.config.disease.enabled = true;
        w.config.disease.fee = 2.0;
        let b = borrower(&mut w);
        {
            let a = w.agent_mut(b).unwrap();
            a.metabolism = 1;
            a.diseases = vec![0];
        }
        record_income(&mut w, b, 6.0);
        assert!((w.agent(b).unwrap().income - 3.0).abs() < 1e-12, "6 − (1 + 2)");
    }
```

`stats.rs` tests:
```rust
    #[test]
    fn supply_and_demand_use_effective_metabolism() {
        use crate::testkit::*;
        let mut w = blank_world(5, 5);
        w.config.spice.enabled = true;
        w.config.disease.enabled = true;
        for (x, sugar, spice) in [(0, 30.0, 10.0), (1, 10.0, 30.0)] {
            let id = spawn(&mut w, x, 0);
            let a = w.agent_mut(id).unwrap();
            (a.sugar, a.spice, a.metabolism, a.spice_metabolism) = (sugar, spice, 1, 3);
        }
        let healthy = supply_demand(&w);
        for a in w.agent_ids() {
            w.agent_mut(a).unwrap().diseases = vec![0, 1];
        }
        let sick = supply_demand(&w);
        assert_ne!(healthy.demand, sick.demand, "weights (1, 3) became (3, 5)");
    }
```

- [ ] **Step 2: Run to verify failure**

Run: `cargo test -p sugarscape-core`
Expected: the five new tests FAIL (fees ignored).

- [ ] **Step 3: Implement**

`config.rs`:
```rust
impl DiseaseRule {
    /// The per-disease metabolic fee in force: `fee` while disease is on, else 0.
    pub fn active_fee(&self) -> f64 {
        if self.enabled {
            self.fee
        } else {
            0.0
        }
    }
}
```

`agent.rs` (`impl Agent`):
```rust
    /// Sugar burned per tick: metabolism plus `fee` per carried disease.
    pub fn effective_metabolism(&self, fee: f64) -> f64 {
        f64::from(self.metabolism) + fee * self.diseases.len() as f64
    }

    /// Spice burned per tick: spice metabolism plus `fee` per carried disease.
    pub fn effective_spice_metabolism(&self, fee: f64) -> f64 {
        f64::from(self.spice_metabolism) + fee * self.diseases.len() as f64
    }
```

`lifecycle.rs` `metabolize` — the start becomes:
```rust
pub(crate) fn metabolize(world: &mut World, id: AgentId, harvest: Harvest) {
    let spice_on = world.config.spice.enabled;
    let fee = world.config.disease.active_fee();
    let agent = world.agent_mut(id).expect("live agent");
    let burned = agent.effective_metabolism(fee);
    agent.sugar -= burned;
    let burned_spice = if spice_on {
        agent.effective_spice_metabolism(fee)
    } else {
        0.0
    };
```
(the rest unchanged). Update the doc comment: "Burns sugar (and spice when it's on) at the effective metabolism (plus the disease fee per carried disease)…"

`movement.rs` `act_two_goods` — replace the `(m1, m2)` line (compute `fee` before `a` is borrowed):
```rust
    let fee = world.config.disease.active_fee();
    let a = world.agent(id).expect("live agent");
    let (pos, vision, phi) = (a.pos, a.vision, a.foresight);
    let (w1, w2) = (a.sugar, a.spice);
    let (m1, m2) = (a.effective_metabolism(fee), a.effective_spice_metabolism(fee));
```

`trade.rs` `Holdings::of`:
```rust
    fn of(world: &World, id: AgentId) -> Self {
        let fee = world.config.disease.active_fee();
        let a = world.agent(id).expect("live agent");
        Self {
            sugar: a.sugar,
            spice: a.spice,
            m1: a.effective_metabolism(fee),
            m2: a.effective_spice_metabolism(fee),
        }
    }
```

`credit.rs` `record_income`:
```rust
/// Sets `income` = sugar gathered − effective sugar metabolism − per-tick
/// obligations.
pub(crate) fn record_income(world: &mut World, id: AgentId, sugar_gathered: f64) {
    let owed = obligations(world, id);
    let fee = world.config.disease.active_fee();
    let a = world.agent_mut(id).expect("live agent");
    a.income = sugar_gathered - a.effective_metabolism(fee) - owed;
}
```

`stats.rs` `supply_demand` — before the agent loop add `let fee = world.config.disease.active_fee();` and replace the `(m1, m2)` line with:
```rust
        let (m1, m2) = (a.effective_metabolism(fee), a.effective_spice_metabolism(fee));
```

- [ ] **Step 4: Verify** — `cargo test -p sugarscape-core` (golden unchanged: `m + 0.0 == m`).

- [ ] **Step 5: Commit**

```bash
cargo fmt --all && cargo clippy --all-targets -- -D warnings
git add crates/sugarscape-core/src
git commit -m "Charge a metabolic fee per carried disease" -m "Claude-Session: https://claude.ai/code/session_01Kq7NyxbMrkNsPfAhnsVcK3"
```

---

### Task 7: Immune response (E, part 1)

**Files:**
- Modify: `crates/sugarscape-core/src/rules/disease.rs`, `crates/sugarscape-core/src/rules/mod.rs`

**Interfaces:**
- Consumes: `Bits::{learn, contains}` (Task 2), `World::diseases`, `Agent.{immune, diseases}` (Task 4).
- Produces: `rules::disease::{respond(&mut World, AgentId), cure_immune(&mut World, AgentId), act(&mut World, AgentId)}` (`pub(crate)`); `agent_turn` runs `disease::act` last, when `disease.enabled`. `act` = `respond` in this task; Task 8 appends transmission.

- [ ] **Step 1: Write failing tests** (in `disease.rs` tests; add `use crate::testkit::*;` to the test module's imports — `AgentId`, `DiseaseId`, `Bits` and `World` come in through `use super::*` once Step 3 imports them)

```rust
    fn sick_world() -> World {
        let mut w = blank_world(10, 10);
        w.config.disease.enabled = true;
        w
    }

    /// An agent at (x, y) with the given immune string, carrying `diseases`.
    fn patient(w: &mut World, x: u32, y: u32, immune: &str, diseases: Vec<DiseaseId>) -> AgentId {
        let id = spawn(w, x, y);
        let a = w.agent_mut(id).unwrap();
        a.immune = b(immune);
        a.diseases = diseases;
        id
    }

    #[test]
    fn book_example_is_learned_in_one_tick() {
        // Appendix B's worked example, through a whole tick: the agent stays
        // put (nothing to gather), burns its fee, then its immune system flips
        // one bit and the disease is gone.
        let mut w = sick_world();
        w.diseases = vec![b("10011")];
        let id = patient(&mut w, 5, 5, "1011101001", vec![0]);
        w.step();
        let a = w.agent(id).unwrap();
        assert_eq!(a.immune.to_bit_string(), "1001101001");
        assert!(a.diseases.is_empty(), "cured");
        assert_eq!(a.sugar, 9.0, "metabolism 0 plus a fee of 1 for one disease");
    }

    #[test]
    fn a_disease_already_in_the_immune_string_is_cured_without_flips() {
        let mut w = sick_world();
        w.diseases = vec![b("11")];
        let id = patient(&mut w, 5, 5, "0000011000", vec![0]);
        respond(&mut w, id);
        let a = w.agent(id).unwrap();
        assert_eq!(a.immune.to_bit_string(), "0000011000");
        assert!(a.diseases.is_empty());
    }

    #[test]
    fn one_flip_per_tick_unless_medicine_adds_more() {
        let mut w = sick_world();
        w.diseases = vec![b("1111")];
        let id = patient(&mut w, 5, 5, "0000000000", vec![0]);
        respond(&mut w, id);
        assert_eq!(w.agent(id).unwrap().immune.to_bit_string(), "1000000000");
        respond(&mut w, id);
        assert_eq!(w.agent(id).unwrap().immune.to_bit_string(), "1100000000");
        assert_eq!(w.agent(id).unwrap().diseases, vec![0], "still sick");
        w.config.disease.flips_per_tick = 2;
        respond(&mut w, id);
        let a = w.agent(id).unwrap();
        assert_eq!(a.immune.to_bit_string(), "1111000000");
        assert!(a.diseases.is_empty(), "two flips finished the job");
    }

    #[test]
    fn every_disease_the_trained_string_now_contains_is_cured() {
        // Training on 11 flips position 0, which also makes 1 a substring.
        let mut w = sick_world();
        w.diseases = vec![b("11"), b("1")];
        let id = patient(&mut w, 5, 5, "0000000000", vec![0, 1]);
        respond(&mut w, id);
        let a = w.agent(id).unwrap();
        assert_eq!(a.immune.to_bit_string(), "1000000000");
        assert_eq!(a.diseases, vec![0]);
    }

    #[test]
    fn nothing_happens_while_disease_is_off() {
        let mut w = blank_world(10, 10);
        w.diseases = vec![b("10011")];
        let id = patient(&mut w, 5, 5, "1011101001", vec![0]);
        w.step();
        assert_eq!(w.agent(id).unwrap().immune.to_bit_string(), "1011101001");
    }
```

- [ ] **Step 2: Run to verify failure**

Run: `cargo test -p sugarscape-core rules::disease`
Expected: FAIL to compile — `respond` not found.

- [ ] **Step 3: Implement**

`disease.rs` (change the agent import to `use crate::agent::{Agent, AgentId, DiseaseId};`):
```rust
/// Rule E for one agent's turn.
pub(crate) fn act(world: &mut World, id: AgentId) {
    respond(world, id);
}

/// Appendix B's immune response: for each carried disease, up to
/// `flips_per_tick` single-bit steps toward it (`Bits::learn`); then every
/// carried disease the trained string now contains is cured.
pub(crate) fn respond(world: &mut World, id: AgentId) {
    let flips = world.config.disease.flips_per_tick;
    let carried: Vec<Bits> = world
        .agent(id)
        .expect("live agent")
        .diseases
        .iter()
        .map(|&d| world.diseases[d as usize])
        .collect();
    let a = world.agent_mut(id).expect("live agent");
    for d in &carried {
        for _ in 0..flips {
            if !a.immune.learn(d) {
                break;
            }
        }
    }
    cure_immune(world, id);
}

/// Drops every carried disease that is a substring of the agent's immune string.
pub(crate) fn cure_immune(world: &mut World, id: AgentId) {
    let a = world.agent(id).expect("live agent");
    let kept: Vec<DiseaseId> = a
        .diseases
        .iter()
        .copied()
        .filter(|&d| !a.immune.contains(&world.diseases[d as usize]))
        .collect();
    world.agent_mut(id).expect("live agent").diseases = kept;
}
```

`rules/mod.rs` — at the end of `agent_turn`:
```rust
    if world.config.disease.enabled {
        disease::act(world, id);
    }
```
and update its doc comment: "…mate with each neighbor, spread culture to them, trade, borrow, and (rule E) train its immune system and pass on disease."

- [ ] **Step 4: Verify** — `cargo test -p sugarscape-core` (golden unchanged).

- [ ] **Step 5: Commit**

```bash
cargo fmt --all && cargo clippy --all-targets -- -D warnings
git add crates/sugarscape-core/src/rules
git commit -m "Add the immune response" -m "Claude-Session: https://claude.ai/code/session_01Kq7NyxbMrkNsPfAhnsVcK3"
```

---

### Task 8: Transmission (E, part 2)

**Files:**
- Modify: `crates/sugarscape-core/src/world.rs`, `crates/sugarscape-core/src/rules/disease.rs`

**Interfaces:**
- Consumes: Task 7's `act`.
- Produces: `world::Infection { infector: Option<AgentId>, infected: AgentId, disease: DiseaseId }` (`Clone, Copy, Debug, PartialEq, Eq`); `TickEvents.infections: Vec<Infection>`; `rules::disease::{transmit(&mut World, AgentId), infect(&mut World, AgentId, DiseaseId) -> bool, find_or_add(&mut World, Bits) -> DiseaseId}` (`pub(crate)`); `act` = `respond` then `transmit`.

- [ ] **Step 1: Write failing tests** (in `disease.rs` tests; `Infection` comes in through `use super::*` once Step 3 imports it)

```rust
    #[test]
    fn transmission_skips_immune_and_already_infected_neighbors() {
        let mut w = sick_world();
        w.diseases = vec![b("11")];
        let zeros = "0".repeat(50);
        let me = patient(&mut w, 5, 5, &zeros, vec![0]);
        let open = patient(&mut w, 5, 4, &zeros, vec![]);
        let immune = patient(&mut w, 6, 5, &format!("11{}", "0".repeat(48)), vec![]);
        let carrier = patient(&mut w, 4, 5, &zeros, vec![0]);
        w.agent_mut(carrier).unwrap().infected_by = Some(999);
        transmit(&mut w, me);
        assert_eq!(w.agent(open).unwrap().diseases, vec![0]);
        assert_eq!(w.agent(open).unwrap().infected_by, Some(me));
        assert!(w.agent(immune).unwrap().diseases.is_empty());
        assert_eq!(w.agent(carrier).unwrap().infected_by, Some(999), "not reinfected");
        assert_eq!(
            w.events().infections,
            vec![Infection {
                infector: Some(me),
                infected: open,
                disease: 0
            }]
        );
    }

    #[test]
    fn each_neighbor_is_offered_one_disease() {
        let mut w = sick_world();
        w.diseases = vec![b("1"), b("11")];
        let zeros = "0".repeat(50);
        let me = patient(&mut w, 5, 5, &zeros, vec![0, 1]);
        let n = patient(&mut w, 5, 4, &zeros, vec![]);
        transmit(&mut w, me);
        assert_eq!(w.agent(n).unwrap().diseases.len(), 1);
    }

    #[test]
    fn mutated_transmissions_append_distinct_variants() {
        let mut w = sick_world();
        w.config.disease.disease_mutation = 1.0;
        w.diseases = vec![b("1111")];
        let zeros = "0".repeat(50);
        let me = patient(&mut w, 5, 5, &zeros, vec![0]);
        let n = patient(&mut w, 5, 4, &zeros, vec![]);
        transmit(&mut w, me);
        assert_eq!(w.diseases.len(), 2, "a one-bit variant was appended");
        assert_eq!(w.diseases[1].hamming(&w.diseases[0]), 1);
        assert_eq!(w.agent(n).unwrap().diseases, vec![1]);
        let variant = w.diseases[1];
        assert_eq!(find_or_add(&mut w, variant), 1, "listed strings are reused");
        assert_eq!(w.diseases.len(), 2);
    }

    #[test]
    fn disease_spreads_to_neighbors_during_a_tick() {
        let mut w = sick_world();
        w.diseases = vec![b("1111111111")];
        let zeros = "0".repeat(50);
        let a = patient(&mut w, 5, 5, &zeros, vec![0]);
        let n = patient(&mut w, 5, 6, &zeros, vec![]);
        w.step();
        // Whatever the turn order, one flip cannot cure a 10-bit disease.
        assert_eq!(w.agent(n).unwrap().diseases, vec![0]);
        assert_eq!(w.agent(n).unwrap().infected_by, Some(a));
        assert_eq!(w.agent(a).unwrap().diseases, vec![0]);
    }
```

- [ ] **Step 2: Run to verify failure**

Run: `cargo test -p sugarscape-core rules::disease`
Expected: FAIL to compile — `Infection`, `transmit` not found.

- [ ] **Step 3: Implement**

`world.rs` — `use crate::agent::{Agent, AgentId, DiseaseId, Tribe};` and after `Trade`:
```rust
/// One infection: `infector` gave `disease` to `infected` (`None` for an
/// outbreak).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Infection {
    pub infector: Option<AgentId>,
    pub infected: AgentId,
    pub disease: DiseaseId,
}
```
Add `pub infections: Vec<Infection>,` to `TickEvents` (after `defaults`).

`disease.rs` (add `use rand::seq::SliceRandom;` and change the world import to `use crate::world::{Infection, World};`; `use rand::Rng;` is already there from Task 5):
```rust
/// Rule E for one agent's turn: immune response, then transmission.
pub(crate) fn act(world: &mut World, id: AgentId) {
    respond(world, id);
    transmit(world, id);
}

/// Appendix B's transmission: each von Neumann neighbor, in random order, is
/// offered one of the agent's diseases chosen uniformly (mutated in one random
/// bit with probability `disease_mutation`) and catches it unless it already
/// carries it or is immune.
pub(crate) fn transmit(world: &mut World, id: AgentId) {
    let me = world.agent(id).expect("live agent");
    if me.diseases.is_empty() {
        return;
    }
    let (pos, carried) = (me.pos, me.diseases.clone());
    let mutation = world.config.disease.disease_mutation;
    let mut neighbors = world.torus.neighbors(pos);
    neighbors.shuffle(&mut world.rng);
    for q in neighbors {
        let Some(other) = world.occupant(q) else {
            continue;
        };
        let mut disease = *carried.choose(&mut world.rng).expect("carries a disease");
        if mutation > 0.0 && world.rng.gen_bool(mutation) {
            let mut variant = world.diseases[disease as usize];
            let bit = world.rng.gen_range(0..variant.len());
            variant.flip(bit);
            disease = find_or_add(world, variant);
        }
        if infect(world, other, disease) {
            world.agent_mut(other).expect("occupant").infected_by = Some(id);
            world.events.infections.push(Infection {
                infector: Some(id),
                infected: other,
                disease,
            });
        }
    }
}

/// Gives `id` the disease unless it already carries it or it is a substring of
/// its immune string. Returns whether the agent was infected.
pub(crate) fn infect(world: &mut World, id: AgentId, disease: DiseaseId) -> bool {
    let d = world.diseases[disease as usize];
    let a = world.agent_mut(id).expect("live agent");
    if a.diseases.contains(&disease) || a.immune.contains(&d) {
        return false;
    }
    a.diseases.push(disease);
    true
}

/// The id of `bits` in the disease list, appending it if it is new.
pub(crate) fn find_or_add(world: &mut World, bits: Bits) -> DiseaseId {
    match world.diseases.iter().position(|d| *d == bits) {
        Some(i) => i as DiseaseId,
        None => {
            world.diseases.push(bits);
            (world.diseases.len() - 1) as DiseaseId
        }
    }
}
```
(Delete Task 7's one-line `act`.)

- [ ] **Step 4: Verify** — `cargo test -p sugarscape-core` (golden unchanged).

- [ ] **Step 5: Commit**

```bash
cargo fmt --all && cargo clippy --all-targets -- -D warnings
git add crates/sugarscape-core/src
git commit -m "Transmit diseases to neighbors" -m "Claude-Session: https://claude.ai/code/session_01Kq7NyxbMrkNsPfAhnsVcK3"
```

---

### Task 9: Outbreaks

**Files:**
- Modify: `crates/sugarscape-core/src/rules/disease.rs`, `crates/sugarscape-core/src/world.rs`

**Interfaces:**
- Consumes: `Outbreak` (Task 3), `infect`/`find_or_add` (Task 8).
- Produces: `rules::disease::{new_random(&mut World) -> DiseaseId, outbreaks(&mut World)}` (`pub(crate)`); `World::step` applies due outbreaks right after scheduled changes, when disease is on. Outbreak infections are recorded in `events.infections` with `infector: None`.

- [ ] **Step 1: Write failing tests** (in `disease.rs` tests; add `use crate::config::Outbreak;` — `URange` comes in through `use super::*`)

```rust
    #[test]
    fn outbreaks_infect_random_agents_with_a_new_disease_at_their_tick() {
        let mut w = sick_world();
        w.config.disease.length = URange::new(8, 8);
        w.config.disease.outbreaks = vec![Outbreak { tick: 2, agents: 3 }];
        // Five agents two sites apart (no neighbors), with empty immune strings
        // so no random disease can be resisted.
        let ids: Vec<AgentId> = (0..5)
            .map(|i| {
                let id = spawn(&mut w, i * 2, 0);
                w.agent_mut(id).unwrap().immune = Bits::default();
                id
            })
            .collect();
        w.step(); // tick 0
        w.step(); // tick 1
        assert!(w.diseases.is_empty());
        w.step(); // starts at tick 2: the outbreak fires
        assert_eq!(w.diseases.len(), 1);
        let sick = ids
            .iter()
            .filter(|&&id| w.agent(id).unwrap().diseases == vec![0])
            .count();
        assert_eq!(sick, 3);
        assert_eq!(w.events().infections.len(), 3);
        assert!(w
            .events()
            .infections
            .iter()
            .all(|i| i.infector.is_none() && i.disease == 0));
        w.step();
        assert_eq!(w.diseases.len(), 1, "each outbreak fires once");
    }

    #[test]
    fn outbreak_size_is_capped_by_the_population() {
        let mut w = sick_world();
        w.config.disease.outbreaks = vec![Outbreak { tick: 0, agents: 50 }];
        let a = spawn(&mut w, 0, 0);
        let c = spawn(&mut w, 5, 5);
        for id in [a, c] {
            w.agent_mut(id).unwrap().immune = Bits::default();
        }
        w.step();
        assert_eq!(w.events().infections.len(), 2);
    }

    #[test]
    fn new_random_diseases_differ_from_the_list_when_possible() {
        let mut w = sick_world();
        w.config.disease.length = URange::new(1, 1);
        w.diseases = vec![b("0")];
        assert_eq!(new_random(&mut w), 1);
        assert_eq!(w.diseases[1], b("1"));
        assert!(new_random(&mut w) <= 1, "no distinct 1-bit string is left: reuse");
        assert_eq!(w.diseases.len(), 2);
    }
```

- [ ] **Step 2: Run to verify failure**

Run: `cargo test -p sugarscape-core rules::disease`
Expected: FAIL to compile — `new_random` not found (and the outbreak test would fail: nothing fires).

- [ ] **Step 3: Implement**

`disease.rs`:
```rust
/// A brand-new random disease: redrawn up to 100 times until it differs from
/// every listed disease; if none does, the last draw's existing id is reused.
pub(crate) fn new_random(world: &mut World) -> DiseaseId {
    let length = world.config.disease.length;
    let mut draw = random_disease(length, &mut world.rng);
    for _ in 0..100 {
        if !world.diseases.contains(&draw) {
            break;
        }
        draw = random_disease(length, &mut world.rng);
    }
    find_or_add(world, draw)
}

/// Applies the outbreaks due at the tick about to run: each creates a new
/// disease and offers it to `min(agents, population)` random living agents.
pub(crate) fn outbreaks(world: &mut World) {
    let due: Vec<u32> = world
        .config
        .disease
        .outbreaks
        .iter()
        .filter(|o| o.tick == world.tick)
        .map(|o| o.agents)
        .collect();
    for agents in due {
        let disease = new_random(world);
        let ids = world.agent_ids();
        let k = (agents as usize).min(ids.len());
        for i in rand::seq::index::sample(&mut world.rng, ids.len(), k).into_vec() {
            if infect(world, ids[i], disease) {
                world.events.infections.push(Infection {
                    infector: None,
                    infected: ids[i],
                    disease,
                });
            }
        }
    }
}
```

`world.rs` `step` — after `self.apply_schedule();`:
```rust
        if self.config.disease.enabled {
            rules::disease::outbreaks(self);
        }
```

- [ ] **Step 4: Verify** — `cargo test -p sugarscape-core` (golden unchanged).

- [ ] **Step 5: Commit**

```bash
cargo fmt --all && cargo clippy --all-targets -- -D warnings
git add crates/sugarscape-core/src
git commit -m "Add disease outbreaks" -m "Claude-Session: https://claude.ai/code/session_01Kq7NyxbMrkNsPfAhnsVcK3"
```

---

### Task 10: Disease statistics and network

**Files:**
- Modify: `crates/sugarscape-core/src/stats.rs`, `crates/sugarscape-core/src/network.rs`, `crates/sugarscape-core/src/export.rs`

**Interfaces:**
- Consumes: `Infection`, `TickEvents.infections` (Task 8).
- Produces: `SERIES` (23) ends with `"infected_fraction", "mean_diseases", "diseases_in_circulation", "new_infections"`; `Snapshot.{infected_fraction: f64, mean_diseases: f64, diseases_in_circulation: u32, new_infections: u32}`; `network::disease_edges(&World) -> Vec<(Pos, Pos)>` (infector → infected among living agents this tick, deduplicated by ordered pair, outbreaks excluded).

- [ ] **Step 1: Write failing tests**

`stats.rs` tests:
```rust
    #[test]
    fn disease_series_count_carriers_and_new_infections() {
        use crate::testkit::*;
        use crate::world::Infection;
        let mut w = blank_world(5, 5);
        let a = spawn(&mut w, 0, 0);
        let b = spawn(&mut w, 1, 0);
        spawn(&mut w, 2, 0);
        spawn(&mut w, 3, 0);
        w.agent_mut(a).unwrap().diseases = vec![0, 2];
        w.agent_mut(b).unwrap().diseases = vec![2];
        w.events.infections = vec![Infection {
            infector: Some(a),
            infected: b,
            disease: 2,
        }];
        let s = Snapshot::of(&w);
        assert_eq!(s.infected_fraction, 0.5);
        assert_eq!(s.mean_diseases, 0.75);
        assert_eq!(s.diseases_in_circulation, 2);
        assert_eq!(s.new_infections, 1);
        for name in SERIES {
            assert!(s.value(name).is_some(), "{name}");
        }
    }
```

`network.rs` tests:
```rust
    #[test]
    fn disease_edges_point_from_infector_to_infected() {
        use crate::world::Infection;
        let mut w = blank_world(5, 5);
        let a = spawn(&mut w, 0, 0);
        let b = spawn(&mut w, 1, 0);
        let infection = |infector, infected| Infection {
            infector,
            infected,
            disease: 0,
        };
        w.events.infections = vec![
            infection(Some(a), b),
            infection(Some(a), b),
            infection(None, a),
            infection(Some(b), 999),
        ];
        assert_eq!(
            disease_edges(&w),
            vec![(Pos::new(0, 0), Pos::new(1, 0))],
            "deduplicated; outbreaks and dead agents dropped"
        );
    }
```

`export.rs` — change the expected header in `series_csv_has_a_header_and_a_row_per_tick` to:
```rust
        assert_eq!(lines[0], "tick,population,gini,mean_wealth,mean_vision,mean_metabolism,blue_fraction,births,deaths,mean_log_price,sd_log_price,trade_volume,sugar_traded,loans_made,amount_lent,defaults,debt_outstanding,mean_foresight,mean_spice,mean_spice_metabolism,infected_fraction,mean_diseases,diseases_in_circulation,new_infections");
```

- [ ] **Step 2: Run to verify failure**

Run: `cargo test -p sugarscape-core`
Expected: FAIL to compile — `infected_fraction`, `disease_edges` not found.

- [ ] **Step 3: Implement**

`stats.rs`:
- `pub const SERIES: [&str; 23] = [ …existing 19…, "infected_fraction", "mean_diseases", "diseases_in_circulation", "new_infections" ];`
- append to `Snapshot`:
```rust
    /// Share of living agents carrying at least one disease.
    pub infected_fraction: f64,
    pub mean_diseases: f64,
    /// Distinct diseases carried by anyone.
    pub diseases_in_circulation: u32,
    /// Infections this tick (transmissions and outbreaks).
    pub new_infections: u32,
```
- in `Snapshot::of`, append to the literal:
```rust
            infected_fraction: mean(&|a| if a.diseases.is_empty() { 0.0 } else { 1.0 }),
            mean_diseases: mean(&|a| a.diseases.len() as f64),
            diseases_in_circulation: world
                .agents()
                .flat_map(|a| a.diseases.iter().copied())
                .collect::<std::collections::BTreeSet<_>>()
                .len() as u32,
            new_infections: events.infections.len() as u32,
```
- in `value`:
```rust
            "infected_fraction" => self.infected_fraction,
            "mean_diseases" => self.mean_diseases,
            "diseases_in_circulation" => f64::from(self.diseases_in_circulation),
            "new_infections" => f64::from(self.new_infections),
```

`network.rs`:
```rust
/// Infector → infected pairs of living agents from this tick's transmissions
/// (Chapter V's disease network; outbreaks have no infector).
pub fn disease_edges(world: &World) -> Vec<(Pos, Pos)> {
    let pairs: BTreeSet<(AgentId, AgentId)> = world
        .events()
        .infections
        .iter()
        .filter_map(|i| Some((i.infector?, i.infected)))
        .collect();
    pairs
        .into_iter()
        .filter_map(|(a, b)| Some((world.agent(a)?.pos, world.agent(b)?.pos)))
        .collect()
}
```
Update the module doc: "Trade, credit and disease networks (Animations IV-4, IV-5 and Chapter V)."

- [ ] **Step 4: Verify** — `cargo test -p sugarscape-core`.

- [ ] **Step 5: Commit**

```bash
cargo fmt --all && cargo clippy --all-targets -- -D warnings
git add crates/sugarscape-core/src
git commit -m "Add disease statistics and the disease network" -m "Claude-Session: https://claude.ai/code/session_01Kq7NyxbMrkNsPfAhnsVcK3"
```

---

### Task 11: Disease color mode, inspection and agents CSV

**Files:**
- Modify: `crates/sugarscape-core/src/render.rs`, `crates/sugarscape-core/src/edit.rs`, `crates/sugarscape-core/src/export.rs`

**Interfaces:**
- Produces: `render::{SICK, HEALTHY}: Rgb`; `ColorMode::Disease` (parsed from `"disease"`); `edit::DiseaseView { id: DiseaseId, bits: String, distance: u32 }`; `AgentView.{immune: String, immune_genome: String, diseases: Vec<DiseaseView>, infected_by: Option<LinkView>}` (serialized as `null` when absent); agents CSV header ends `…,foresight,immune,diseases` (`diseases` = ids joined by `;`).

- [ ] **Step 1: Write failing tests**

`render.rs` tests:
```rust
    #[test]
    fn disease_mode_colors_sick_and_healthy_agents() {
        let mut w = blank_world(10, 10);
        let sick = spawn(&mut w, 1, 1);
        spawn(&mut w, 2, 2);
        w.agent_mut(sick).unwrap().diseases = vec![0];
        let mut buf = Vec::new();
        render(&w, ColorMode::Disease, Layer::Sugar, &mut buf);
        assert_eq!(pixel(&buf, &w, 1, 1)[..3], SICK);
        assert_eq!(pixel(&buf, &w, 2, 2)[..3], HEALTHY);
        assert_eq!("disease".parse::<ColorMode>().unwrap(), ColorMode::Disease);
    }
```

`edit.rs` tests:
```rust
    #[test]
    fn inspection_shows_immune_strings_diseases_and_infector() {
        use crate::bits::Bits;
        let mut w = blank_world(10, 10);
        w.config.disease.enabled = true;
        w.diseases = vec![Bits::parse("111").unwrap()];
        let a = spawn(&mut w, 1, 1);
        let b = spawn(&mut w, 2, 1);
        {
            let x = w.agent_mut(a).unwrap();
            x.immune = Bits::parse(&format!("0110{}", "0".repeat(46))).unwrap();
            x.diseases = vec![0];
            x.infected_by = Some(b);
        }
        let v = w.inspect(1, 1).unwrap().agent.unwrap();
        assert_eq!(v.immune, format!("0110{}", "0".repeat(46)));
        assert_eq!(v.immune_genome, "0".repeat(50));
        assert_eq!(v.diseases.len(), 1);
        let d = &v.diseases[0];
        assert_eq!((d.id, d.bits.as_str(), d.distance), (0, "111", 1));
        assert_eq!(v.infected_by.unwrap().id, b);
        assert!(w.inspect(2, 1).unwrap().agent.unwrap().infected_by.is_none());
    }
```

`export.rs` — in `agents_csv_has_a_row_per_agent`, change the expected header to:
```rust
            "id,x,y,sex,age,max_age,vision,metabolism,sugar,initial_sugar,tribe,tags,spice,initial_spice,spice_metabolism,foresight,immune,diseases\n"
```
and add:
```rust
    #[test]
    fn agents_csv_lists_immune_strings_and_disease_ids() {
        let mut w = crate::testkit::blank_world(5, 5);
        let id = crate::testkit::spawn(&mut w, 0, 0);
        w.agent_mut(id).unwrap().diseases = vec![3, 7];
        let csv = agents_csv(&w);
        let row = csv.lines().nth(1).unwrap();
        assert!(row.ends_with(&format!(",{},3;7", "0".repeat(50))), "{row}");
    }
```

- [ ] **Step 2: Run to verify failure**

Run: `cargo test -p sugarscape-core`
Expected: FAIL to compile — `ColorMode::Disease`, `AgentView.immune` not found.

- [ ] **Step 3: Implement**

`render.rs`:
```rust
pub const SICK: Rgb = [0xff, 0x4d, 0x4d];
pub const HEALTHY: Rgb = [0x3d, 0x7e, 0xff];
```
Add `Disease` to `ColorMode`, `"disease" => Self::Disease,` to `from_str`, and to `agent_color`:
```rust
        ColorMode::Disease => {
            if a.diseases.is_empty() {
                HEALTHY
            } else {
                SICK
            }
        }
```

`edit.rs` (`use crate::agent::{Agent, AgentId, DiseaseId, Sex, Tribe};`):
```rust
#[derive(Clone, Debug, Serialize)]
pub struct DiseaseView {
    pub id: DiseaseId,
    pub bits: String,
    /// Smallest Hamming distance between the disease and a window of the
    /// agent's immune string.
    pub distance: u32,
}
```
Append to `AgentView` (after `loans`):
```rust
    pub immune: String,
    pub immune_genome: String,
    pub diseases: Vec<DiseaseView>,
    pub infected_by: Option<LinkView>,
```
and to the `AgentView` literal in `inspect` (after `loans: …`):
```rust
            immune: a.immune.to_bit_string(),
            immune_genome: a.immune_genome.to_bit_string(),
            diseases: a
                .diseases
                .iter()
                .map(|&id| {
                    let d = self.diseases[id as usize];
                    DiseaseView {
                        id,
                        bits: d.to_bit_string(),
                        distance: a
                            .immune
                            .closest_window(&d)
                            .map_or(d.len(), |(_, distance)| distance),
                    }
                })
                .collect(),
            infected_by: a.infected_by.map(link),
```

`export.rs` `agents_csv`: header gains `,immune,diseases`; the format string gains `,{},{}` and the arguments gain:
```rust
            a.immune.to_bit_string(),
            a.diseases
                .iter()
                .map(|d| d.to_string())
                .collect::<Vec<_>>()
                .join(";")
```

- [ ] **Step 4: Verify** — `cargo test -p sugarscape-core`.

- [ ] **Step 5: Commit**

```bash
cargo fmt --all && cargo clippy --all-targets -- -D warnings
git add crates/sugarscape-core/src
git commit -m "Color, inspect and export disease state" -m "Claude-Session: https://claude.ai/code/session_01Kq7NyxbMrkNsPfAhnsVcK3"
```

---

### Task 12: Infect, vaccinate and the disease list

**Files:**
- Modify: `crates/sugarscape-core/src/edit.rs`

**Interfaces:**
- Consumes: `rules::disease::{infect, new_random, cure_immune}` (Tasks 7–9), `Bits::imprint` (Task 2).
- Produces: `edit::DiseaseEntry { id: DiseaseId, bits: String, carriers: u32 }` (`Serialize`); `World::disease_list(&self) -> Vec<DiseaseEntry>`; `World::infect(&mut self, x: u32, y: u32, disease: i64) -> Result<bool, String>` (negative = new random disease, appended even if resisted; returns whether infected); `World::vaccinate(&mut self, x: u32, y: u32, radius: u32, disease: DiseaseId) -> Result<u32, String>` (agents within Euclidean `radius`, wrapping; returns how many were vaccinated). All three error with `"disease is off"` when disease is off (`disease_list` just returns the empty list).

- [ ] **Step 1: Write failing tests** (`edit.rs` tests)

```rust
    #[test]
    fn infect_tool_gives_a_listed_or_new_disease() {
        use crate::bits::Bits;
        let mut w = blank_world(10, 10);
        w.config.disease.enabled = true;
        w.diseases = vec![Bits::parse("101").unwrap()];
        let a = spawn(&mut w, 1, 1);
        assert!(w.infect(1, 1, 0).unwrap());
        assert!(!w.infect(1, 1, 0).unwrap(), "already carries it");
        assert_eq!(w.agent(a).unwrap().diseases, vec![0]);
        assert!(w.agent(a).unwrap().infected_by.is_none(), "a tool is not an infector");
        w.infect(1, 1, -1).unwrap();
        assert_eq!(w.diseases.len(), 2, "a new disease is appended");
        assert_ne!(w.diseases[1], w.diseases[0]);
        assert!(w.infect(1, 1, 7).is_err(), "unknown disease");
        assert!(w.infect(5, 5, 0).is_err(), "no agent");
        assert!(w.events().infections.is_empty(), "edits are not counted as infections");
        w.config.disease.enabled = false;
        assert_eq!(w.infect(1, 1, 0).unwrap_err(), "disease is off");
    }

    #[test]
    fn vaccination_writes_the_disease_into_immune_strings_in_the_brush() {
        use crate::bits::Bits;
        let mut w = blank_world(10, 10);
        w.config.disease.enabled = true;
        w.diseases = vec![Bits::parse("111").unwrap()];
        let near = spawn(&mut w, 5, 5);
        let edge = spawn(&mut w, 5, 6);
        let far = spawn(&mut w, 8, 8);
        for id in [near, edge, far] {
            w.agent_mut(id).unwrap().diseases = vec![0];
        }
        assert_eq!(w.vaccinate(5, 5, 1, 0).unwrap(), 2);
        for id in [near, edge] {
            let a = w.agent(id).unwrap();
            assert!(a.diseases.is_empty(), "cured");
            assert!(a.immune.to_bit_string().starts_with("111"), "leftmost closest window");
            assert_eq!(a.immune_genome.to_bit_string(), "0".repeat(50), "genome untouched");
        }
        assert_eq!(w.agent(far).unwrap().diseases, vec![0]);
        assert!(w.vaccinate(5, 5, 1, 3).is_err(), "unknown disease");
        let list = w.disease_list();
        assert_eq!(list.len(), 1);
        assert_eq!((list[0].id, list[0].bits.as_str(), list[0].carriers), (0, "111", 1));
    }
```

- [ ] **Step 2: Run to verify failure**

Run: `cargo test -p sugarscape-core edit`
Expected: FAIL to compile — `infect`, `vaccinate`, `disease_list` not found.

- [ ] **Step 3: Implement** (in `edit.rs`; add `use std::collections::BTreeSet;` and `use crate::rules;`)

```rust
#[derive(Clone, Debug, Serialize)]
pub struct DiseaseEntry {
    pub id: DiseaseId,
    pub bits: String,
    /// Living agents carrying it.
    pub carriers: u32,
}
```
Inside `impl World`:
```rust
    fn disease_on(&self) -> Result<(), String> {
        if self.config.disease.enabled {
            Ok(())
        } else {
            Err("disease is off".into())
        }
    }

    /// The master disease list with each disease's carriers.
    pub fn disease_list(&self) -> Vec<DiseaseEntry> {
        let mut carriers = vec![0u32; self.diseases.len()];
        for a in self.agents() {
            for &d in &a.diseases {
                carriers[d as usize] += 1;
            }
        }
        self.diseases
            .iter()
            .zip(carriers)
            .enumerate()
            .map(|(i, (d, carriers))| DiseaseEntry {
                id: i as DiseaseId,
                bits: d.to_bit_string(),
                carriers,
            })
            .collect()
    }

    /// Infects the agent at (x, y) with listed `disease`, or — when `disease`
    /// is negative — with a brand-new random disease (appended to the list even
    /// if the agent resists it). No effect if the agent is immune or already
    /// carries it. Returns whether it was infected.
    pub fn infect(&mut self, x: u32, y: u32, disease: i64) -> Result<bool, String> {
        self.disease_on()?;
        let pos = self.checked_pos(x, y)?;
        let id = self
            .occupant(pos)
            .ok_or_else(|| format!("no agent at ({x}, {y})"))?;
        let d = if disease < 0 {
            rules::disease::new_random(self)
        } else {
            DiseaseId::try_from(disease)
                .ok()
                .filter(|&d| (d as usize) < self.diseases.len())
                .ok_or_else(|| format!("unknown disease {disease}"))?
        };
        Ok(rules::disease::infect(self, id, d))
    }

    /// Writes `disease` into the immune string of every agent within Euclidean
    /// `radius` of (x, y) (wrapping), over its closest window, then cures any
    /// carried disease the string now contains. The genome is untouched.
    /// Returns how many agents were vaccinated.
    pub fn vaccinate(
        &mut self,
        x: u32,
        y: u32,
        radius: u32,
        disease: DiseaseId,
    ) -> Result<u32, String> {
        self.disease_on()?;
        let center = self.checked_pos(x, y)?;
        let d = *self
            .diseases
            .get(disease as usize)
            .ok_or_else(|| format!("unknown disease {disease}"))?;
        let r = radius as i32;
        let mut ids = BTreeSet::new();
        for dy in -r..=r {
            for dx in -r..=r {
                if dx * dx + dy * dy <= r * r {
                    if let Some(id) = self.occupant(self.torus.offset(center, dx, dy)) {
                        ids.insert(id);
                    }
                }
            }
        }
        let mut vaccinated = 0;
        for id in ids {
            if self.agent_mut(id).expect("occupant").immune.imprint(&d) {
                rules::disease::cure_immune(self, id);
                vaccinated += 1;
            }
        }
        Ok(vaccinated)
    }
```

- [ ] **Step 4: Verify** — `cargo test -p sugarscape-core`.

- [ ] **Step 5: Commit**

```bash
cargo fmt --all && cargo clippy --all-targets -- -D warnings
git add crates/sugarscape-core/src/edit.rs
git commit -m "Add infect and vaccinate edits and the disease list" -m "Claude-Session: https://claude.ai/code/session_01Kq7NyxbMrkNsPfAhnsVcK3"
```

---

### Task 13: Chapter V presets

**Files:**
- Modify: `crates/sugarscape-core/src/presets.rs`, `crates/sugarscape-core/tests/golden.rs`, `crates/sugarscape-core/tests/book.rs` (temporary measurement only; not committed)

**Interfaces:**
- Consumes: `DiseaseRule`, `Outbreak`.
- Produces: preset ids `v-1-rid`, `v-2-endemic`, `v-mcneill`, `vi-1-everything` (23 presets total); golden entries for them.

- [ ] **Step 1: Write failing tests** (`presets.rs` tests; change `assert_eq!(presets.len(), 19)` to `23`)

```rust
    #[test]
    fn chapter_v_presets_use_the_books_disease_setups() {
        let v1 = by_id("v-1-rid").unwrap().config;
        let d = &v1.disease;
        assert!(d.enabled);
        assert_eq!(
            (d.count, d.length, d.initial, d.immune_length),
            (10, URange::new(1, 10), 4, 50)
        );
        assert!(!v1.sex.enabled, "Chapter II agents");
        let v2 = by_id("v-2-endemic").unwrap().config.disease;
        assert_eq!((v2.count, v2.initial), (25, 10));
        let m = by_id("v-mcneill").unwrap().config;
        assert!(m.sex.enabled && m.lifespan.enabled && m.disease.enabled);
        assert_eq!(m.disease.outbreaks, vec![Outbreak { tick: 300, agents: 5 }]);
        let e = by_id("vi-1-everything").unwrap().config;
        assert!(
            e.spice.enabled
                && e.sex.enabled
                && e.lifespan.enabled
                && e.inheritance.enabled
                && e.culture.enabled
                && e.trade.enabled
                && e.credit.enabled
                && e.disease.enabled
        );
        assert!(!e.combat.enabled && !e.replacement.enabled);
    }
```

- [ ] **Step 2: Run to verify failure**

Run: `cargo test -p sugarscape-core presets`
Expected: FAIL — unknown preset ids, count 19.

- [ ] **Step 3: Add the presets**

In `presets.rs`, import `Outbreak` (`use crate::config::{Config, Outbreak, Placement, ScheduledChange, URange};`) and add:
```rust
/// Animation V-1's disease setup: 10 diseases of length 1–10, 4 per agent,
/// 50-bit immune strings (the `DiseaseRule` defaults).
fn disease(c: &mut Config) {
    c.disease.enabled = true;
}
```
Append to `all()`:
```rust
        preset(
            "v-1-rid",
            "({G₁}, {M, E})",
            "Animation V-1",
            "Immune systems learn the diseases their agents carry: the society rids itself of disease.",
            disease,
        ),
        preset(
            "v-2-endemic",
            "({G₁}, {M, E}) with 25 diseases",
            "Animation V-2",
            "Too many diseases for one immune string: learning one immunity can overwrite another, and disease stays endemic.",
            |c| {
                disease(c);
                c.disease.count = 25;
                c.disease.initial = 10;
            },
        ),
        preset(
            "v-mcneill",
            "({G₁}, {M, S, E}) + outbreak",
            "Chapter V (after McNeill)",
            "A reproducing society carrying its familiar diseases meets a novel one at t = 300, brought in by 5 agents.",
            |c| {
                demography(c);
                disease(c);
                c.disease.outbreaks = vec![Outbreak {
                    tick: 300,
                    agents: 5,
                }];
            },
        ),
        preset(
            "vi-1-everything",
            "({G₁}, {M, S, I, K, T, L, E})",
            "Chapter VI",
            "Every rule at once: spice, sex, finite lives, inheritance, culture, trade, credit and disease.",
            |c| {
                demography(c);
                c.inheritance.enabled = true;
                c.culture.enabled = true;
                c.spice.enabled = true;
                c.trade.enabled = true;
                c.credit.enabled = true;
                disease(c);
                // Endowments: measured in Step 4; replace this comment with
                // the measurements.
                c.endowment = URange::new(25, 50);
                c.spice.endowment = URange::new(25, 50);
            },
        ),
```

- [ ] **Step 4: Measure `vi-1-everything`'s endowments**

The spec requires endowments measured so the population survives. `iv-18-foresight` found demography's 50–100 unreachable once fertility needs both goods; trade and credit may change that, so measure rather than assume. Append this temporary test to `tests/book.rs` (add `use sugarscape_core::config::URange;`):
```rust
#[test]
#[ignore]
fn measure_everything_on() {
    for (lo, hi) in [(25, 50), (50, 100), (15, 40)] {
        let mut c = presets::by_id("vi-1-everything").unwrap().config;
        c.endowment = URange::new(lo, hi);
        c.spice.endowment = URange::new(lo, hi);
        for seed in 1..=5 {
            let w = run(c.clone(), seed, 1000);
            let p = w.stats.series("population").unwrap();
            let s = w.stats.latest().unwrap();
            println!(
                "{lo}-{hi} seed {seed}: pop t=250 {} t=500 {} t=1000 {} infected {:.3} births {}",
                p[250], p[500], p[1000], s.infected_fraction,
                w.stats.series("births").unwrap().iter().sum::<f64>()
            );
        }
    }
}
```
Run: `cargo test -p sugarscape-core --release --test book -- --ignored --nocapture measure_everything_on`

Choose the **first** range in the order 25–50, 50–100, 15–40 for which every seed 1–5 still has a population ≥ 50 at t = 1000. Set both `c.endowment` and `c.spice.endowment` to it and replace the placeholder comment with a comment in the style of `iv-18-foresight`'s: why the endowment differs from `demography()`, and the per-seed t = 1000 populations for **every** range tried. If no range qualifies, stop and report BLOCKED with the printed table rather than guessing (the controller will decide, e.g. a lower disease fee). Delete `measure_everything_on` (and the `URange` import if unused) before committing.

- [ ] **Step 5: Record golden fingerprints for the new presets**

Run: `cargo test -p sugarscape-core --test golden -- --ignored --nocapture print_golden`
Check the 19 earlier values are unchanged, then append the four new lines to `GOLDEN` under `// Chapter V (disease on).`

- [ ] **Step 6: Verify**

```bash
cargo test -p sugarscape-core
```
Expected: all pass, including `every_preset_has_a_golden_entry`.

- [ ] **Step 7: Commit**

```bash
cargo fmt --all && cargo clippy --all-targets -- -D warnings
git add crates/sugarscape-core/src/presets.rs crates/sugarscape-core/tests/golden.rs
git commit -m "Add Chapter V presets" -m "Claude-Session: https://claude.ai/code/session_01Kq7NyxbMrkNsPfAhnsVcK3"
```

---

### Task 14: Disease invariants and book reproductions

**Files:**
- Modify: `crates/sugarscape-core/tests/invariants.rs`, `crates/sugarscape-core/tests/book.rs`

**Interfaces:**
- Consumes: presets from Task 13; `World::diseases`; `Agent.{immune, immune_genome, diseases}`; `Bits::contains`; the `infected_fraction`, `new_infections` and `population` series.

- [ ] **Step 1: Invariants**

In `tests/invariants.rs`, import `Outbreak` (`use sugarscape_core::config::{Config, Outbreak, URange};`), add a twelfth strategy element `(proptest::bool::ANY, proptest::bool::ANY)` destructured as `(disease, mutate)`, and after the credit/spice constraints:
```rust
                c.disease.enabled = disease;
                if disease && mutate {
                    c.disease.genome_mutation = 0.02;
                    c.disease.disease_mutation = 0.1;
                    c.disease.flips_per_tick = 2;
                    c.disease.outbreaks = vec![Outbreak { tick: 10, agents: 5 }];
                }
```
In `check`, before `Ok(())`:
```rust
    let d = &world.config.disease;
    if !d.enabled {
        prop_assert!(world.diseases.is_empty());
    }
    for a in world.agents() {
        if !d.enabled {
            prop_assert!(a.diseases.is_empty());
            continue;
        }
        prop_assert_eq!(a.immune.len(), d.immune_length);
        prop_assert_eq!(a.immune_genome.len(), d.immune_length);
        let mut ids = a.diseases.clone();
        ids.sort_unstable();
        ids.dedup();
        prop_assert_eq!(ids.len(), a.diseases.len(), "agent {} carries a duplicate", a.id);
        for &id in &a.diseases {
            let Some(disease) = world.diseases.get(id as usize) else {
                return Err(TestCaseError::fail(format!("invalid disease id {id}")));
            };
            prop_assert!(
                !a.immune.contains(disease),
                "agent {} carries disease {} it is immune to",
                a.id,
                id
            );
        }
    }
```
Run: `cargo test -p sugarscape-core --release --test invariants` — pass. If an invariant fails, fix the rule (not the invariant) — Decision 2 is what keeps "no carried disease is a substring" true.

- [ ] **Step 2: Book reproductions** (append to `tests/book.rs`; **measure first**: run each test once with the assertions replaced by `println!` of the per-seed values, record the observed values in the comment, then set bounds with margin, as the existing tests do)

```rust
#[test]
#[ignore]
fn immune_learning_rids_the_society_of_disease() {
    // Animation V-1: with 10 short diseases and 50-bit immune strings every
    // agent eventually learns all the diseases it meets.
    // Observed (seeds 1..=3): first tick with nobody infected = …; infected
    // share at t=1000 = … (record the measured values here).
    let config = presets::by_id("v-1-rid").unwrap().config;
    for seed in 1..=3 {
        let w = run(config.clone(), seed, 1000);
        let f = w.stats.series("infected_fraction").unwrap();
        assert!(
            f.contains(&0.0),
            "seed {seed}: never disease-free (t=1000 share {})",
            f[1000]
        );
        assert_eq!(f[1000], 0.0, "seed {seed}: disease returned");
    }
}

#[test]
#[ignore]
fn many_diseases_stay_endemic() {
    // Animation V-2: 25 diseases, 10 per agent — learning one immunity
    // disturbs others, so the society cannot rid itself of disease.
    // Observed infected share at t=1000 (seeds 1..=3): … (record here).
    let config = presets::by_id("v-2-endemic").unwrap().config;
    for seed in 1..=3 {
        let w = run(config.clone(), seed, 1000);
        let share = w.stats.latest().unwrap().infected_fraction;
        assert!(share > 0.0, "seed {seed}: disease died out");
    }
}

#[test]
#[ignore]
fn a_novel_disease_spreads_after_the_mcneill_outbreak() {
    // The t=300 outbreak seeds 5 agents; the new disease spreads beyond them.
    // Series index = tick; the outbreak's step is recorded at index 301.
    // Observed (seeds 1..=3): infected agents at t=300 = …, peak over
    // t=301..=400 = … (record here).
    let config = presets::by_id("v-mcneill").unwrap().config;
    for seed in 1..=3 {
        let w = run(config.clone(), seed, 400);
        let share = w.stats.series("infected_fraction").unwrap();
        let pop = w.stats.series("population").unwrap();
        let infected = |t: usize| share[t] * pop[t];
        let peak = (301..=400).map(infected).fold(0.0, f64::max);
        assert!(
            peak > infected(300) && peak > 5.0,
            "seed {seed}: {} infected at t=300, peak {peak} after the outbreak",
            infected(300)
        );
    }
}

#[test]
#[ignore]
fn everything_on_society_survives() {
    // Chapter VI's everything-on run; endowments chosen in presets.rs from
    // the measurements recorded there. Observed t=1000 populations
    // (seeds 1..=3): … (copy from the preset comment).
    let config = presets::by_id("vi-1-everything").unwrap().config;
    for seed in 1..=3 {
        let w = run(config.clone(), seed, 1000);
        assert!(w.population() >= 50, "seed {seed}: population {}", w.population());
    }
}
```
If a book result does not reproduce, debug against the rule text (Appendix B, Tasks 7–9) before adjusting anything; any adjusted bound needs a comment with the observed values and why the book still supports it. Replace every "…" with the measured numbers — no "…" may remain.

- [ ] **Step 3: Verify**

```bash
cargo test -p sugarscape-core
cargo test -p sugarscape-core --release --test invariants
cargo test -p sugarscape-core --release --test book -- --ignored
```

- [ ] **Step 4: Commit**

```bash
cargo fmt --all && cargo clippy --all-targets -- -D warnings
git add crates/sugarscape-core/tests
git commit -m "Check disease invariants and reproduce Chapter V results" -m "Claude-Session: https://claude.ai/code/session_01Kq7NyxbMrkNsPfAhnsVcK3"
```

---

### Task 15: WASM bindings

**Files:**
- Modify: `crates/sugarscape-wasm/src/lib.rs`, `crates/sugarscape-wasm/tests/web.rs`

**Interfaces:**
- Consumes: `network::disease_edges` (Task 10), `World::{disease_list, infect, vaccinate}` (Task 12).
- Produces JS: `Sim.networks("disease")` (infector → infected this tick, `[x1, y1, x2, y2, …]`); `Sim.disease_list(): string` (JSON `[{ id, bits, carriers }]`); `Sim.infect(x, y, disease: number): boolean` (−1 = new random disease; throws when disease is off, no agent or unknown id); `Sim.vaccinate(x, y, radius, disease: number): number` (count vaccinated). `render()` accepts `"disease"` automatically.

- [ ] **Step 1: Failing test** (`tests/web.rs`)

```rust
#[wasm_bindgen_test]
fn disease_api_lists_infects_and_vaccinates() {
    let mut off = Sim::new("{}", 1, None).unwrap();
    assert!(off.infect(0, 0, -1).is_err(), "disease is off");

    let mut sim = Sim::new(r#"{"disease":{"enabled":true}}"#, 1, None).unwrap();
    let list: serde_json::Value = serde_json::from_str(&sim.disease_list()).unwrap();
    assert_eq!(list.as_array().unwrap().len(), 10);
    assert!(list[0]["bits"].is_string() && list[0]["carriers"].is_number());
    let p = sim.locate(1.0).unwrap();
    sim.infect(p[0], p[1], -1).unwrap();
    let list: serde_json::Value = serde_json::from_str(&sim.disease_list()).unwrap();
    assert_eq!(list.as_array().unwrap().len(), 11, "a new disease was appended");
    assert_eq!(sim.vaccinate(p[0], p[1], 0, 10).unwrap(), 1);
    let view: serde_json::Value = serde_json::from_str(&sim.inspect(p[0], p[1]).unwrap()).unwrap();
    let carried = view["agent"]["diseases"].as_array().unwrap();
    assert!(carried.iter().all(|d| d["id"] != 10), "vaccinated against #10");
    assert!(sim.vaccinate(p[0], p[1], 0, 99).is_err());
    sim.step(3);
    assert_eq!(sim.networks("disease").unwrap().len() % 4, 0);
    sim.render("disease", "sugar").unwrap();
    assert_eq!(sim.series("new_infections").unwrap().len(), 4);
}
```

- [ ] **Step 2: Run** — `wasm-pack test --node crates/sugarscape-wasm` fails to compile (`disease_list`, `infect`, `vaccinate` missing).

- [ ] **Step 3: Implement** (in `impl Sim`)

In `networks`, add the arm `"disease" => network::disease_edges(&self.world),` and change its doc comment to: "Edges as `[x1, y1, x2, y2, …]` for `"trade"` (this tick), `"credit"` (outstanding) or `"disease"` (infector → infected, this tick)."

```rust
    /// JSON `[{ id, bits, carriers }]`.
    pub fn disease_list(&self) -> String {
        serde_json::to_string(&self.world.disease_list()).expect("list serializes")
    }

    /// Infects the agent at (x, y) with `disease` (−1 = a brand-new random
    /// disease). Returns whether it was infected.
    pub fn infect(&mut self, x: u32, y: u32, disease: i32) -> Result<bool, JsValue> {
        self.world
            .infect(x, y, i64::from(disease))
            .map_err(edit_error)
    }

    /// Vaccinates every agent within `radius` of (x, y) against `disease`.
    /// Returns how many agents were vaccinated.
    pub fn vaccinate(&mut self, x: u32, y: u32, radius: u32, disease: u32) -> Result<u32, JsValue> {
        self.world
            .vaccinate(x, y, radius, disease)
            .map_err(edit_error)
    }
```

- [ ] **Step 4: Verify** — `wasm-pack test --node crates/sugarscape-wasm` (7 passed); `cargo clippy --all-targets -- -D warnings`.

- [ ] **Step 5: Commit**

```bash
cargo fmt --all
git add crates/sugarscape-wasm
git commit -m "Expose disease list, infection, vaccination and the disease network to JavaScript" -m "Claude-Session: https://claude.ai/code/session_01Kq7NyxbMrkNsPfAhnsVcK3"
```

---

### Task 16: Web types, Disease rules and outbreaks in the schedule

**Files:**
- Create: `web/src/schedule.ts`, `web/src/schedule.test.ts`
- Modify: `web/src/types.ts`, `web/src/schema.ts`, `web/src/ui/rules-panel.ts`

**Interfaces:**
- Consumes: the Rust JSON shapes from Tasks 3, 10, 11 and 12.
- Produces TS: `Outbreak { tick: number; agents: number }`; `DiseaseRule { enabled; count; length: URange; initial; immune_length; fee; flips_per_tick; genome_mutation; disease_mutation; outbreaks: Outbreak[] }`; `Config.disease: DiseaseRule`; `Snapshot.{infected_fraction, mean_diseases, diseases_in_circulation, new_infections}`; `DiseaseView { id; bits; distance }`; `AgentView.{immune: string; immune_genome: string; diseases: DiseaseView[]; infected_by: LinkView | null}`; `DiseaseEntry { id; bits; carriers }`; `ColorMode` + `'disease'`; `scheduleLines(config: Config): string[]`; schema group "Disease (E)".

- [ ] **Step 1: Failing vitest** (`web/src/schedule.test.ts`)

```ts
import { describe, expect, it } from 'vitest';
import { scheduleLines } from './schedule';
import type { Config } from './types';

const config = (schedule: Config['schedule'], outbreaks: Config['disease']['outbreaks']) =>
  ({ schedule, disease: { outbreaks } }) as unknown as Config;

describe('scheduleLines', () => {
  it('lists scheduled changes and outbreaks in tick order', () => {
    const lines = scheduleLines(
      config(
        [
          { tick: 100, set: { 'diffusion.enabled': true } },
          { tick: 50, set: { 'pollution.enabled': true } },
        ],
        [
          { tick: 300, agents: 5 },
          { tick: 75, agents: 1 },
        ],
      ),
    );
    expect(lines).toEqual([
      't = 50 · pollution.enabled = true',
      't = 75 · new disease → 1 agent',
      't = 100 · diffusion.enabled = true',
      't = 300 · new disease → 5 agents',
    ]);
  });

  it('is empty without entries', () => {
    expect(scheduleLines(config([], []))).toEqual([]);
  });
});
```

- [ ] **Step 2: Run** — `cd web && npx vitest run src/schedule.test.ts` fails (module missing).

- [ ] **Step 3: Implement**

`web/src/types.ts` — add:
```ts
export interface Outbreak { tick: number; agents: number }

export interface DiseaseRule {
  enabled: boolean;
  count: number;
  length: URange;
  initial: number;
  immune_length: number;
  fee: number;
  flips_per_tick: number;
  genome_mutation: number;
  disease_mutation: number;
  outbreaks: Outbreak[];
}
```
In `Config`, add `disease: DiseaseRule;` after `foresight`. Append to `Snapshot`:
```ts
  infected_fraction: number;
  mean_diseases: number;
  diseases_in_circulation: number;
  new_infections: number;
```
Add:
```ts
export interface DiseaseView { id: number; bits: string; distance: number }
export interface DiseaseEntry { id: number; bits: string; carriers: number }
```
Append to `AgentView`:
```ts
  immune: string;
  immune_genome: string;
  diseases: DiseaseView[];
  infected_by: LinkView | null;
```
Change `ColorMode` to `'tribe' | 'wealth' | 'sex' | 'age' | 'vision' | 'credit' | 'disease'`.

`web/src/schedule.ts`:
```ts
import type { Config } from './types';

/** Scheduled changes and outbreaks as display lines, in tick order (stable on ties). */
export function scheduleLines(config: Config): string[] {
  const lines: { tick: number; text: string }[] = [];
  for (const e of config.schedule) {
    for (const [path, value] of Object.entries(e.set)) {
      lines.push({ tick: e.tick, text: `t = ${e.tick} · ${path} = ${JSON.stringify(value)}` });
    }
  }
  for (const o of config.disease.outbreaks) {
    lines.push({ tick: o.tick, text: `t = ${o.tick} · new disease → ${o.agents} agent${o.agents === 1 ? '' : 's'}` });
  }
  return lines.sort((a, b) => a.tick - b.tick).map((l) => l.text);
}
```

`web/src/schema.ts` — append after the Foresight group:
```ts
  {
    title: 'Disease (E)', enable: 'disease.enabled', enableResets: true,
    note: 'Immune strings learn the diseases agents carry; each disease raises metabolism. Turning disease on or off, the disease list and the immune length rebuild the world; the rest applies to the running world.',
    controls: [
      { kind: 'number', path: 'disease.count', label: 'Diseases', min: 1, max: 100, step: 1, reset: true },
      { kind: 'range', path: 'disease.length', label: 'Disease length', min: 1, max: 63, reset: true },
      { kind: 'number', path: 'disease.immune_length', label: 'Immune length', min: 2, max: 64, step: 1, reset: true },
      { kind: 'number', path: 'disease.initial', label: 'Diseases per new agent', min: 0, max: 100, step: 1 },
      { kind: 'number', path: 'disease.fee', label: 'Metabolism per disease', min: 0, max: 5, step: 0.5 },
      { kind: 'number', path: 'disease.flips_per_tick', label: 'Immune flips per tick (medicine)', min: 1, max: 10, step: 1 },
      { kind: 'number', path: 'disease.genome_mutation', label: 'Genome mutation rate', min: 0, max: 0.1, step: 0.001 },
      { kind: 'number', path: 'disease.disease_mutation', label: 'Disease mutation rate', min: 0, max: 1, step: 0.01 },
    ],
  },
```

`web/src/ui/rules-panel.ts` — import `scheduleLines` from `'../schedule'` and replace `scheduleSection`:
```ts
  private scheduleSection(): HTMLElement {
    const list = h('ul', { class: 'schedule' });
    const clear = h('button', { onclick: () => this.commit((c) => (c.schedule = []), false) }, 'Clear schedule');
    const section = h('section', { class: 'group' }, h('h3', {}, 'Schedule'), list, clear, this.errorSlot('schedule'));
    this.syncers.push(() => {
      const lines = scheduleLines(this.engine.config);
      section.hidden = lines.length === 0;
      // Outbreaks are listed read-only; the button clears scheduled changes.
      clear.hidden = this.engine.config.schedule.length === 0;
      list.replaceChildren(...lines.map((line) => h('li', {}, line)));
    });
    return section;
  }
```

- [ ] **Step 4: Verify** — `cd web && npm run build && npm test`.

- [ ] **Step 5: Commit**

```bash
git add web/src/types.ts web/src/schema.ts web/src/schedule.ts web/src/schedule.test.ts web/src/ui/rules-panel.ts
git commit -m "Add the Disease rules group and list outbreaks in the schedule" -m "Claude-Session: https://claude.ai/code/session_01Kq7NyxbMrkNsPfAhnsVcK3"
```

---

### Task 17: Disease colors, network overlay and the Infect/Vaccinate tools

**Files:**
- Create: `web/src/ui/disease-picker.ts`, `web/src/ui/disease-picker.test.ts`
- Modify: `web/src/engine.ts`, `web/src/ui/display.ts`, `web/src/ui/grid-view.ts`, `web/src/ui/tools.ts`

**Interfaces:**
- Consumes: `Sim.{networks, disease_list, infect, vaccinate}` (Task 15), `DiseaseEntry` (Task 16).
- Produces: `engine.ts`: `export type Overlay = 'trade' | 'credit' | 'disease'`; `Engine.overlays: Record<Overlay, boolean>`; `Engine.setDisplay({ overlays?: Partial<Record<Overlay, boolean>> })`; `Engine.diseaseList(): DiseaseEntry[]`; `Engine.infect(x, y, disease): FieldError[] | null`; `Engine.vaccinate(x, y, radius, disease): FieldError[] | null`; `disease-picker.ts`: `diseaseOptions(list: DiseaseEntry[], allowNew: boolean): [string, string][]`.

- [ ] **Step 1: Failing vitest** (`web/src/ui/disease-picker.test.ts`)

```ts
import { describe, expect, it } from 'vitest';
import { diseaseOptions } from './disease-picker';

const list = [
  { id: 0, bits: '101', carriers: 4 },
  { id: 1, bits: '0011', carriers: 0 },
];

describe('diseaseOptions', () => {
  it('lists diseases by id and bits, with New disease first when allowed', () => {
    expect(diseaseOptions(list, true)).toEqual([
      ['-1', 'New disease'],
      ['0', '#0 · 101'],
      ['1', '#1 · 0011'],
    ]);
  });
  it('omits New disease for vaccination', () => {
    expect(diseaseOptions(list, false)).toEqual([
      ['0', '#0 · 101'],
      ['1', '#1 · 0011'],
    ]);
  });
});
```

- [ ] **Step 2: Run** — `cd web && npx vitest run src/ui/disease-picker.test.ts` fails (module missing).

- [ ] **Step 3: Implement**

`web/src/ui/disease-picker.ts`:
```ts
import type { DiseaseEntry } from '../types';

/** `[value, label]` options for a disease picker; value `-1` is "New disease". */
export function diseaseOptions(list: DiseaseEntry[], allowNew: boolean): [string, string][] {
  const options: [string, string][] = list.map((d) => [String(d.id), `#${d.id} · ${d.bits}`]);
  return allowNew ? [['-1', 'New disease'], ...options] : options;
}
```

`web/src/engine.ts`:
- import `DiseaseEntry` in the type import;
- add `export type Overlay = 'trade' | 'credit' | 'disease';`;
- `overlays: Record<Overlay, boolean> = { trade: false, credit: false, disease: false };`
- `setDisplay(d: { colorMode?: ColorMode; layer?: Layer; overlays?: Partial<Record<Overlay, boolean>> }): void` (body unchanged);
- add after `erase`:
```ts
  diseaseList(): DiseaseEntry[] {
    return JSON.parse(this.sim.disease_list()) as DiseaseEntry[];
  }

  /** `disease` −1 infects with a brand-new random disease. */
  infect(x: number, y: number, disease: number): FieldError[] | null {
    return this.edit(() => void this.sim.infect(x, y, disease));
  }

  vaccinate(x: number, y: number, radius: number, disease: number): FieldError[] | null {
    return this.edit(() => void this.sim.vaccinate(x, y, radius, disease));
  }
```

`web/src/ui/display.ts`:
- `import type { Engine, Overlay } from '../engine';`
- append `['disease', 'Disease']` to `MODES`;
- change the helper's signature to `const overlay = (kind: Overlay, label: string) => {` (body unchanged);
- append `overlay('disease', 'Disease network'),` after the credit overlay.

`web/src/ui/grid-view.ts` — the overlay loop becomes:
```ts
    for (const [kind, color] of [['trade', '--c3'], ['credit', '--c2'], ['disease', '--red']] as const) {
```

`web/src/ui/tools.ts` — replace the file:
```ts
import type { Engine, PlaceOverrides } from '../engine';
import { diseaseOptions } from './disease-picker';
import { h } from './dom';
import type { GridView } from './grid-view';

type Tool = 'inspect' | 'paint' | 'place' | 'erase' | 'infect' | 'vaccinate';

const TOOLS: [Tool, string][] = [
  ['inspect', 'Inspect'],
  ['paint', 'Paint capacity'],
  ['place', 'Place agent'],
  ['erase', 'Erase agent'],
  ['infect', 'Infect'],
  ['vaccinate', 'Vaccinate'],
];

/** Tools that exist only while disease is on. */
const DISEASE_TOOLS: Tool[] = ['infect', 'vaccinate'];

/** Tool picker; routes grid clicks/drags to the active tool. Edit errors (e.g. occupied site) are ignored. */
export function buildTools(engine: Engine, grid: GridView, onInspect: () => void): HTMLElement {
  let tool: Tool = 'inspect';
  let radius = 1;
  let value = 4;
  let sex: '' | 'female' | 'male' = '';
  let tribe: '' | 'blue' | 'red' = '';
  /** Selected disease id; −1 is a new random disease (Infect only). */
  let disease = -1;
  /** Length of the disease list the picker was last filled from. */
  let known = -1;
  const brushed = () => tool === 'paint' || tool === 'vaccinate';

  const buttons = TOOLS.map(([t, label]) => h('button', { onclick: () => choose(t) }, label));
  const options = h('div', { class: 'tool-options' });
  const picker = h('select', { onchange: () => (disease = Number(picker.value)) });

  const number = (label: string, min: number, max: number, get: () => number, set: (v: number) => void) => {
    const input = h('input', { type: 'number', min, max, value: get(), class: 'num' });
    input.addEventListener('change', () => {
      set(Math.min(max, Math.max(min, Number(input.value))));
      input.value = String(get());
      if (brushed()) grid.brushRadius = radius;
    });
    return h('label', {}, `${label} `, input);
  };
  const select = <T extends string>(label: string, values: [T, string][], set: (v: T) => void) => {
    const s = h('select', {}, ...values.map(([v, l]) => h('option', { value: v }, l)));
    s.addEventListener('change', () => set(s.value as T));
    return h('label', {}, `${label} `, s);
  };

  /** Refills the picker when the disease list has grown (outbreaks, mutations, Infect), or when forced. */
  function refreshPicker(force = false): void {
    if (!DISEASE_TOOLS.includes(tool) || !engine.config.disease.enabled) return;
    const list = engine.diseaseList();
    if (!force && list.length === known) return;
    known = list.length;
    picker.replaceChildren(...diseaseOptions(list, tool === 'infect').map(([v, l]) => h('option', { value: v }, l)));
    const values = Array.from(picker.options, (o) => Number(o.value));
    if (!values.includes(disease)) disease = values[0] ?? -1;
    picker.value = String(disease);
  }

  function choose(next: Tool): void {
    tool = next;
    buttons.forEach((b, i) => b.setAttribute('aria-pressed', String(TOOLS[i][0] === tool)));
    grid.brushRadius = brushed() ? radius : null;
    if (tool === 'paint') engine.setDisplay({ layer: 'capacity' });
    if (DISEASE_TOOLS.includes(tool)) engine.setDisplay({ colorMode: 'disease' });
    const pick = h('label', {}, 'Disease ', picker);
    options.replaceChildren(
      ...(tool === 'paint'
        ? [number('Radius', 0, 10, () => radius, (v) => (radius = v)), number('Capacity', 0, 4, () => value, (v) => (value = v))]
        : tool === 'place'
          ? [
              select('Sex', [['', 'Random'], ['female', 'Female'], ['male', 'Male']], (v) => (sex = v)),
              select('Tribe', [['', 'Random'], ['blue', 'Blue'], ['red', 'Red']], (v) => (tribe = v)),
            ]
          : tool === 'infect'
            ? [pick, h('span', { class: 'hint' }, 'Click an agent to infect it.')]
            : tool === 'vaccinate'
              ? [number('Radius', 0, 10, () => radius, (v) => (radius = v)), pick]
              : tool === 'inspect'
                ? [h('span', { class: 'hint' }, 'Click an agent or site.')]
                : [h('span', { class: 'hint' }, 'Click or drag over agents to remove them.')]),
    );
    refreshPicker(true);
    grid.draw();
  }

  /** Hides the disease tools while disease is off (leaving them if one was active). */
  function syncAvailability(): void {
    const on = engine.config.disease.enabled;
    buttons.forEach((b, i) => {
      if (DISEASE_TOOLS.includes(TOOLS[i][0])) b.hidden = !on;
    });
    if (!on && DISEASE_TOOLS.includes(tool)) choose('inspect');
    else refreshPicker(true);
  }

  grid.onCell = (x, y, kind) => {
    switch (tool) {
      case 'inspect':
        if (kind === 'down') {
          engine.select(x, y);
          onInspect();
        }
        break;
      case 'paint':
        engine.paint(x, y, radius, value);
        break;
      case 'place':
        if (kind === 'down') {
          const o: PlaceOverrides = {};
          if (sex) o.sex = sex;
          if (tribe) o.tribe = tribe;
          engine.place(x, y, o);
        }
        break;
      case 'erase':
        engine.erase(x, y);
        break;
      case 'infect':
        if (kind === 'down') engine.infect(x, y, disease);
        break;
      case 'vaccinate':
        engine.vaccinate(x, y, radius, disease);
        break;
    }
  };

  engine.on('reset', syncAvailability);
  engine.on('config', syncAvailability);
  engine.on('tick', () => refreshPicker());
  engine.on('edit', () => refreshPicker());
  choose('inspect');
  syncAvailability();
  return h('div', { class: 'tools' }, h('div', { class: 'tool-buttons' }, ...buttons), options);
}
```

- [ ] **Step 4: Verify** — `cd web && npm run build && npm test`.

- [ ] **Step 5: Commit**

```bash
git add web/src/engine.ts web/src/ui/display.ts web/src/ui/grid-view.ts web/src/ui/tools.ts web/src/ui/disease-picker.ts web/src/ui/disease-picker.test.ts
git commit -m "Add disease colors, the disease network and Infect/Vaccinate tools" -m "Claude-Session: https://claude.ai/code/session_01Kq7NyxbMrkNsPfAhnsVcK3"
```

---

### Task 18: Disease charts, inspector, README and roadmap

**Files:**
- Modify: `web/src/ui/charts-panel.ts`, `web/src/style.css`, `web/src/ui/inspect-panel.ts`, `README.md`, `docs/roadmap.md`

**Interfaces:**
- Consumes: the four disease series (Task 10), `AgentView` disease fields (Task 16).
- Produces: a **Disease** chart section (`<section class="disease">`, shown when `config.disease.enabled`, re-evaluated on `reset`/`config`) with "Infected" (share, y in [0, 1]), "Diseases per agent", "Diseases in circulation", "New infections"; inspector rows (only while disease is on) "Immune string" (with the number of bits learned vs the genome), "Immune genome", "Diseases" (`#id bits (distance)`), "Infected by" (link).

- [ ] **Step 1: Charts**

In `charts-panel.ts` `build()`, replace the economy section setup (from the `// Market charts need spice…` comment through the `syncSection` definition) with:
```ts
    // Market charts need spice; loan charts need credit; the section needs
    // either. The disease section needs disease.
    const economy = h('section', { class: 'economy' }, h('h3', {}, 'Economy'));
    const disease = h('section', { class: 'disease' }, h('h3', {}, 'Disease'));
    this.el.append(economy, disease);
    const spiceOn = () => this.engine.config.spice.enabled;
    const creditOn = () => this.engine.config.credit.enabled;
    const diseaseOn = () => this.engine.config.disease.enabled;
    const syncSection = () => {
      economy.hidden = !(spiceOn() || creditOn());
      disease.hidden = !diseaseOn();
      for (const p of this.plots) p.figure.hidden = !p.visible();
      this.drawnTick = null;
    };
```
and, after the "Spice & foresight" chart and before the final `syncSection();`:
```ts
    addTimeChart(
      { title: 'Infected', lines: [{ key: 'infected_fraction', label: 'Infected share', color: '--red' }], range: [0, 1] },
      disease,
      diseaseOn,
    );
    addTimeChart({ title: 'Diseases per agent', lines: [{ key: 'mean_diseases', label: 'Mean', color: '--c2' }] }, disease, diseaseOn);
    addTimeChart(
      { title: 'Diseases in circulation', lines: [{ key: 'diseases_in_circulation', label: 'Distinct diseases', color: '--c4' }] },
      disease,
      diseaseOn,
    );
    addTimeChart({ title: 'New infections', lines: [{ key: 'new_infections', label: 'Infections', color: '--c1' }] }, disease, diseaseOn);
```
In `style.css`, change the two economy rules to cover both sections:
```css
.charts .economy, .charts .disease { display: grid; gap: 12px; }
.charts .economy h3, .charts .disease h3 { font-size: 13px; text-transform: uppercase; letter-spacing: 0.05em; color: var(--muted); margin: 8px 0 0; }
```

- [ ] **Step 2: Inspector**

In `inspect-panel.ts`, add a method:
```ts
  private diseaseRows(a: AgentView): HTMLElement[] {
    const row = (k: string, v: HTMLElement | string) => h('tr', {}, h('th', {}, k), h('td', {}, v));
    const learned = [...a.immune].filter((bit, i) => bit !== a.immune_genome[i]).length;
    return [
      row('Immune string', h('span', {}, h('code', {}, a.immune), h('span', { class: 'hint' }, ` ${learned} bits learned`))),
      row('Immune genome', h('code', {}, a.immune_genome)),
      row(
        'Diseases',
        a.diseases.length === 0
          ? h('span', { class: 'hint' }, 'none')
          : h(
              'span',
              { class: 'links' },
              ...a.diseases.map((d) =>
                h('span', { title: 'Hamming distance to the closest window of the immune string' }, `#${d.id} `, h('code', {}, d.bits), ` (${d.distance})`),
              ),
            ),
      ),
      row('Infected by', a.infected_by ? this.links([a.infected_by]) : h('span', { class: 'hint' }, 'nobody')),
    ];
  }
```
and in `agentRows`, after the `Culture tags` row:
```ts
      ...(this.engine.config.disease.enabled ? this.diseaseRows(a) : []),
```

- [ ] **Step 3: README and roadmap**

`README.md` — in "Rules implemented", after the Chapter IV paragraph, add:
```markdown
Chapter V: immune and disease bit strings, immune response and transmission (E), a
metabolic fee per carried disease, immune-genome inheritance with optional mutation,
disease mutation, outbreaks of novel diseases (the McNeill scenario), Infect and Vaccinate
tools, and a disease-network overlay. `vi-1-everything` runs every rule from Chapters II–V
together.
```
and under "Notes" add:
```markdown
- Switching disease on or off, and changing the number of diseases, their lengths or the
  immune-string length, rebuilds the world; the fee, flips per tick ("medicine") and
  mutation rates apply to the running world. Outbreaks are listed in the Schedule section.
```

`docs/roadmap.md` — replace the "Milestone 3" section with:
```markdown
## Milestone 3: Chapter V — disease (done)

Immune and disease bit strings, immune response and transmission (E), metabolic symptoms,
immune-genome inheritance, disease mutation, outbreaks, Infect/Vaccinate tools, a disease
network overlay and the book's presets (V-1 rids itself of disease, V-2 endemic, the
McNeill outbreak), plus `vi-1-everything`. See
`docs/superpowers/specs/2026-09-23-chapter-v-disease-design.md`.
```

- [ ] **Step 4: Full verification**

```bash
cargo fmt --all --check && cargo clippy --all-targets -- -D warnings
cargo test -p sugarscape-core
cargo test -p sugarscape-core --release --test invariants
cargo test -p sugarscape-core --release --test book -- --ignored
wasm-pack test --node crates/sugarscape-wasm
(cd web && npm run build && npm test)
```

- [ ] **Step 5: Commit**

```bash
git add web/src/ui/charts-panel.ts web/src/style.css web/src/ui/inspect-panel.ts README.md docs/roadmap.md
git commit -m "Add Disease charts and inspector rows; document Chapter V" -m "Claude-Session: https://claude.ai/code/session_01Kq7NyxbMrkNsPfAhnsVcK3"
```
