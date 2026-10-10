# W1 engineering amendment: source inputs and observation records

Date: 2026-10-10. Supplemental correction authorized in the final W1 review.

This amendment supplements the approved [design](2026-10-09-war-1-engagement-design.md)
and [implementation plan](../plans/2026-10-09-war-1-engagement.md). Their original
bytes remain unchanged. This is an engineering clarification, not a scientific
registration, calibration, collection protocol, or new browser behavior.

## Required compiled source inputs

Add `crates/sugarscape-core/assets/sugar-map.txt` and this amendment to W1's
explicit mandatory file selection. The ordinary landscape implementation embeds
that map, and each prescribed literal combat control uses it. The resolved
Config does not expand its bytes, so binding only core Rust sources omitted a
compiled input. The existing versioned path-and-byte digest and runtime source
comparison must cover both added files; changed bytes, a missing file, or a
symlink must fail verification. Rebuild the binary and compiled stamp after
applying this amendment. The executable SHA-256 remains a separate identity.

## Historical caching identity and live source authentication

The frozen caching identities describe the pre-W1 source, including the original
`crates/sugarscape-core/src/lib.rs` bytes at commit
`83727abd328a363bcb3c9a9215b65081f8a8a0bc`. Those bytes have SHA-256
`6ac710e9fe3521e6591dc8edc02b884e2aaae55a36298ab7313d9a9fb4c85f4f`.
Retain them only as the test fixture
`web/src/episodes/test-fixtures/pre-war-core-lib.rs.txt`.

The source-integrity test authenticates that retained file against its frozen
hash and uses those bytes for the historical identity's lib.rs entry. Every
other declared file (479 per identity) continues to use its live bytes. The
canonical historical digest and every existing identity, scientific fixture,
expected result, and public artifact remain frozen.

Separately compare the entire live lib.rs byte sequence against the retained
original plus exactly this appended native feature gate (including its leading
blank line and trailing newline):

```rust
#[cfg(all(feature = "war-benchmarks", not(target_arch = "wasm32")))]
pub mod war;
```

Reject every other delta, including changes to either gate condition, unrelated
source edits, and extra bytes. Do not remove arbitrary sections, omit a hash,
or replace the historical expected digest. Historical source authentication
and this strict live-source compatibility check are distinct assertions; the
current full source tree does not have the historical identity. W1's own stamp
continues to hash the actual live lib.rs. Production browser identity generation,
imports, exports, and model behavior are unchanged.

## Graph casualty cause and Book observation boundary

The graph observation envelope includes the serialized cause
`benchmark_exposure`. It applies to the frame's casualty receipts and identifies
opposing aggregate exposure; it does not identify a unique lethal attacker.
This fulfills the approved plan's prose promise, whose displayed DTO omitted
the field. Book observations continue to carry actual World death causes.

Before emitting a completed Book observation, require all serialized Snapshot
numeric observations to be finite, including nested and vector statistics.
Absent optional statistics remain absent. An unrepresentable observation fails
with contextual `invalid_observation` and preserves separate completed and
emitted clocks. World execution and statistics mathematics remain literal and
unchanged; the runner does not replace invalid numbers with unlabeled nulls.
