# Emergent polarity: sequential obsolete-front correction

**Date:** 2026-10-03 UTC. Implementation correction authorized after the complete
registered native workload exposed a broken sequential control. The source,
manifest, 572 resolved configurations, registered seeds, invalid-stock rules and
scientific judges remain fixed. This is not a change made to improve source fit.

The retained original complete telemetry dataset has 28,520 records, including
237 sequential panic outcomes. Its dataset SHA256 is
`cbca333bfbad29d1220e5d0456da1698a7de27e0c9b5fb0da03e02ebace2f3cd`.
The original executable and all original panic records remain in
`survey/out/polarity-telemetry-*`; original report/verdict hashes are bound in the
execution provenance and before/after receipt.

An exact reproduction used registered arm
`ambiguity.p005.r2.update.sequential`, seed `128000002`, with a full Rust
backtrace. It completed 15 economic periods and failed in attempted period 16.
`combat.rs` indexed a missing entry in `self.fronts` while processing a cached
list of actor-adjacent keys. An earlier conquest in that list calls `structural`,
which rebuilds adjacency and removes obsolete fronts. The subsequent cached key
can therefore describe a front that no longer exists. Snapshot execution applies
structural changes after evaluating fronts and does not share this stale-key path.

The correction checks that each cached front still exists before recording
visitation or resolution. It does not resolve an obsolete dyad, advance its
episode, apply damage, or consume its random draws. Existing live fronts keep
the same order and logic. The targeted four-cell conquest motif and exact
registered-seed period regression first fail on the retained original code and
then pass with the minimal correction. Core/golden and actual native/release
WASM checks cover the changed engine.

The correction reruns all 16 registered sequential arms and all their 20 original
seeds, including the 83 previously valid outcomes and the 237 failures. It does
not rerun only failed seeds. Separate `survey/out/polarity-sequential-*` files
bind the corrected committed head, executable, complete zero-period native
resolution, unchanged source/manifest, commands and measured runtime. All 83
previously valid complete records must remain equal.

The authoritative final dataset is assembled by retaining the literal raw lines
of all 28,200 non-sequential records from the original complete telemetry run and
substituting all 320 corresponding sequential records from the corrected run.
The separate [strict export correction](2026-10-03-emergent-polarity-export-amendment.md)
retains original scientific computation functions; both original and final reports
use that tested serializer. A strict receipt validates complete registered keys/configurations, original
seed ranges, every unaffected line, and all previously valid sequential records.
The unchanged analysis then recomputes all registered 100,000-draw judges using
seed 2026100202; every changed verdict/result is shown before and after. Earlier
reports remain retained evidence and do not supply the final scientific judges.

Registered reject-nonpositive invalid outcomes and nonfinite-ratio invalid
outcomes remain unchanged. Direct terminal nonpositive-stock frequencies are
still distinct from period-exposure frequencies, which were not measured.
