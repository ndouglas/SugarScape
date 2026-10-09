# Browser visualization portability amendment

**Date:** 2026-10-08
**Status:** approved by the user (LGTM!), October 8, 2026; execution authorized.
**Parent:** [deduction and surface browser design](2026-10-08-deduction-surface-web-visualizations-design.md).

## Concrete conflict

The parent requires unchanged frozen study namespaces and complete native/WASM
parity. Actual target testing demonstrates that both cannot currently hold:

- Wink seed7 assigns holder3 on native64 and holder2 on WASM32. The original
  deduction engine samples an index with `rand` over `usize`; that sampler uses
  different integer widths on these targets. Runtime target selection in the
  built-in/fixed threat controllers has the same issue.
- Across all 5,760 retained testimony-game cases, WASM actions, null availability
  and exact conditional masses match the original native references. Twenty-two
  of 512 numeric posterior values differ, with maximum absolute error
  2.220446049250313e-16. The original model uses1e-12 tolerance. The other 5,248
  cases have null posteriors and match exactly. These are original floating
  ln/exp results, not new learned policies or altered channels.

The current exact parity failures remain recorded and are not skipped. Actual
browser checks ran all 8 defaults within 13.568 seconds, five slower active controls
within 60 seconds, and verified cancel/load failure settlement and worker release.
Performance success does not resolve the behavioral parity conflict.

## Proposed source exception

Permit only the five runtime `usize` index draws in these three files to sample
an explicitly bounded `u64` index, then convert the in-range result to `usize`:

- `crates/sugarscape-core/src/deduction/engine.rs`: hidden-holder selection.
- `crates/sugarscape-core/src/deduction/policy.rs`: three built-in/random index draws.
- `crates/sugarscape-core/src/deduction/diagnostics.rs`: fixed threat target selection.

The [concrete five-expression patch](../../../.superpowers/sdd/2026-10-08-deduction-surface-web-visualizations/portability-proposal.patch)
is prepared in retained evidence and has **not** been applied. This reproduces
the original64-bit native sampler on both targets. Leave the existing small
signed-integer draw and all training tournament sampling unchanged.

Require genuine cross-target RED/GREEN and independent review. Verify the complete
original native Wink diagnostic bytes and representative ordinary/diagnostic
trajectories at all four policies, including seeds0,7 and u64max. The original
native outcomes, controller random streams and fingerprints must remain exact.
If this narrow patch does not preserve them, stop and reassess rather than
rewrite original artifacts or broaden the exception silently.

## Floating record equivalence

Keep the existing floating inference engines and their returned values unchanged.
For full-record parity and cross-target episode import, compare every field and
shape, actions, availability, identity, exact integer/rational values and declared
model parameters exactly. Only computed legacy testimony probabilities and
computed floating conditional regret use the original absolute1e-12 tolerance.
Define those permitted numeric paths explicitly; do not apply tolerance to
arbitrary numeric fields, model assumptions, versions, input, clocks or indexes.

JSON number spellings such as0 and0.0 represent the same numeric value; imports
from a browser must not fail merely because JSON.stringify removed that spelling
distinction. This is numerical equivalence of existing floating outputs, not
authentication and not a license to accept a changed decision or null field.
Fresh Rust reconstruction and complete structural comparison remain mandatory.
Test forged values outside tolerance, changed actions, model parameters, shapes,
and availability. No rounding or rewritten scientific result is authorized.

## Identity and retained evidence

Preserve every original first/repeat report and original source/settings manifest.
Record this after-measurement portability revision explicitly. The viewer's
corrected source identity is separate from the original native measurement
identity; never claim all 99 protected source hashes remain unchanged after this
exception. Whitelist and review only the three sampler-file changes above; all
other frozen study files/settings remain byte-identical.

Retained results still describe the original native source. Their native
compatibility is established by exact before/after byte checks, not by silently
rewriting source receipts. Update the viewer guide with the target portability
and floating equivalence limits, while keeping development history in evidence.

## Acceptance and continuation

After approval, execute this narrow correction through an implementer with
independent review, update the parent plan/spec constraints to cite this exception,
and complete Task5 parity/browser gates on the revised source. All later UI,
preservation, final review, merge/push and exact-commit CI/Pages requirements remain.
No new experimental settings, retraining, policies or judged measurement series
are authorized by this amendment.
