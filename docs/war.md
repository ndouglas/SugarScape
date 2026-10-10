# Native W1 engagement cases

W1 is implemented behind the default-off native `war-benchmarks` feature. It
supports fixed engineering cases. No measurement campaign, historical fit,
calibration, findings publication or browser feature is registered by this
delivery. A prospective measurement protocol and approval are still required
for ensembles, timestep refinement and scientific collection.

From the repository root:

```sh
cargo run --manifest-path survey/Cargo.toml --locked --features war-benchmarks --bin war1 -- --help
cargo run --manifest-path survey/Cargo.toml --locked --features war-benchmarks --bin war1 -- validate --input case.json
cargo run --manifest-path survey/Cargo.toml --locked --features war-benchmarks --bin war1 -- run --input case.json --out case-output
```

The argument order is exact. `validate` prints the resolved input and does not
create output. `run` requires a new output directory under an existing directory;
it refuses reused destinations, symlink inputs or ancestors, nonregular inputs
and `..` path components. Use a canonical parent when preparing temporary paths.
Files are created exclusively; existing files are never overwritten. Exit codes
are 0 for success, 2 for usage/input errors and 1 for execution/provenance/output
errors. There is no batch, retry, overwrite, collection or resume command.

An engineering graph example is:

```json
{"mode":"reciprocal_graph","config":{"blue":2,"red":2,"blue_rate":1.0,"red_rate":1.0,"dt":0.05,"max_steps":8,"geometry":"aimed_fire"},"seed":7,"capture_steps":[0,1,8]}
```

A literal book control is:

```json
{"mode":"book_c","preset":"iii-9-combat","seed":8,"max_steps":4,"capture_steps":[4]}
```

`book_c` resolves an existing preset to its complete ordinary Config and ticks
the existing World unchanged. Headers save that Config and observations record
the actual combat flag. World death causes, site-capture kills/loot, agent stores
and site stores retain their existing meanings. Complete harvest flows, removed
wealth, death sites and active battle time are unavailable. Stock changes do not
establish those flows. A halted World tick or nonfinite observed stock sum is a
failed case with its actual completed prefix.

`reciprocal_graph` owns finite stationary individuals and separate contact and
casualty RNG streams. `aimed_fire` applies the aggregate opposing exposure to
every eligible individual; `duel_contact` matches disjoint opposing pairs with
capacity `min(B,R)`. Contact pairs are exposure relations; they do not count
shots or identify individual killers. Both sides contribute before simultaneous
settlement. Casualty IDs appear on their death frames once. The graph has no
holdings, geography or economy, so resource observations are unavailable.

Rates and `dt` describe dimensionless model time. The kernel computes ideal
`p = -expm1(-hazard * dt)`, while its 53-bit sampler realizes
`q = ceil(p * 2^53) / 2^53`. This quantization has an absolute bound below
`2^-53` and no relative accuracy guarantee. Positive probabilities below the
sampler resolution, nonfinite arithmetic and positive-product underflow fail
explicitly. Independent mathematical references describe continuous force,
stop at their extinction boundary and are not empirical combat laws.

Completed steps/calendar time and positive-exposure active steps/time are
separate clocks. Extinction precedes rate-zero and horizon endings. Horizon
termination is administrative censoring, not demonstrated stalemate or peace.
Invalid attempts retain the last completed state and semantic RNG streams.
Emitted steps count successful sink callbacks; acknowledged steps count
completed records whose write, flush and file synchronization all succeeded.
The literal book control has World ticks and no engagement-active clock.

Inputs deny unknown and recursively duplicate fields, trailing JSON, negative
or floating seeds and nonzero numeric literals that overflow or round to zero
in binary64. Seeds remain exact unsigned 64-bit integers. Inputs are limited to
1 MiB, requested diagnostic frames to 1024, JSONL records to 2 MiB, journals to
128 MiB and RNG strings to 16 KiB per stream. Core count, work and horizon bounds
apply too. Requested frames that follow an early ending are explicitly
unavailable. Frames stream to disk; diagnostic retention is bounded. The
literal World still retains its pre-existing per-tick Snapshot history, so
total book-control memory is not constant in the tick horizon.

Output contains exact `input.json`, resolved/provenance `metadata.json` and
streamed `frames.jsonl`. Each complete line is flushed and synchronized before
acknowledgment. `success.json` is written only after semantic completion and
final journal synchronization, then synchronized along with the output and its
parent directory. A failed final marker synchronization revokes the marker this
invocation created. Core terminal records alone do not declare durable artifact
completion. On failure, partial frames remain and `failure.json` is attempted
with semantic/emitted clocks, reason and the acknowledged-prefix SHA-256, byte
count and step count. Its digest may cover less than the physical file when a
write was partial or unacknowledged. Failure-receipt errors are reported too;
outputs are never silently truncated or retried.

The separately gated W1 stamp hashes normalized, sorted, length-prefixed paths
and bytes under domain `war1-compiled-inputs-v1\0`: core source and manifest,
repository manifests/lock, survey manifests/lock/build script, W1 local sources,
binary entrypoint and stamp helper, plus the approved design and implementation
plan. Generated/cache/evidence directories are excluded. Missing required files,
duplicate paths and selected symlinks fail. Invocation recomputes this declared
source closure from the build's repository root and refuses changed sources
until rebuilt. Metadata also saves compiler `rustc -vV`, inherited flags,
feature selection, paths, raw-input SHA-256 and a separate SHA-256 of the actual
executable. The resolved-input FNV checksum is a reproducibility identifier;
neither it nor the declared-source digest supplies authentication or replaces
binary identity or a prospective measurement manifest. Keep the compiled source
tree available to invoke the executable. Existing F5/BT selectors are preserved.

Graph checkpoints save config, seed, clocks, living IDs, ending and both RNG
streams; core restoration validates these and reproduces continuation.
Checkpoints are reproducibility data, not proof of historical authenticity.
The CLI records available failure checkpoints but does not expose restoration
or resume; book controls have no graph checkpoint.

The `war` CI job explicitly tests native core semantics, unchanged goldens,
real-binary CLI behavior and feature-specific clippy. Default survey builds omit
the binary, and wasm32 excludes the core module even when its feature is selected.
See the [approved design](superpowers/specs/2026-10-09-war-1-engagement-design.md)
and [living study status](studies/2026-09-26-war-and-society.md).
