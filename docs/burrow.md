# Burrow 1 excavation lab

Burrow is a standalone checked replay lab for explicit excavation and spoil transport. Its
controllers use supplied preferences. Replay correctness does not establish biological validity,
learning, chamber counts, coordination or comparative success. A judged campaign requires a
separately reviewed protocol and manifest. See the [approved design](superpowers/specs/2026-10-03-burrow-1-excavation-design.md)
and [research programme](studies/2026-10-03-cultures-construction-and-underworlds.md). The next proposed artifacts are the
[measured protocol and candidate manifest](superpowers/specs/2026-10-04-burrow-1-measured-protocol.md)
and [resource-access design](superpowers/specs/2026-10-04-burrow-2-resource-access-design.md);
both await review and introduce no runtime changes.

Run an acceptance demonstration from the repository root, using an absent or empty directory:

```sh
cargo run --release -p sugarscape-cli -- burrow \
  --config docs/examples/burrow/relay-responsive.json \
  --seed 7 --ticks 512 --sample-every 32 --out /tmp/sugarscape-burrow-1-review
```

`--config` and `--out` are required. Seed, ticks and sampling interval default to `7`, `512`
and `32`. Seeds span the full unsigned 64-bit range. Zero ticks exports the initial state;
sampling must be positive. Validation errors exit with code 2 and contextual `field: message`
lines. I/O failures exit with code 1. The CLI validates the entire request before creating or
writing output, creates missing directories, accepts empty directories and refuses nonempty
directories or file paths. A later write failure reports the directory as potentially partial;
it does not report successful completion.

The four examples vary only `transport` (`direct`, `relay`) and `cue` (`blind`, `responsive`).
Their fixture uses the existing externally tagged JSON enum, with snake_case variant names:

```json
{
  "fixture": { "growing": { "width": 41, "height": 25, "workers": 8 } },
  "transport": "relay",
  "cue": "responsive",
  "freshness_window": 32,
  "relay_distance": 3,
  "response_weight": 3,
  "minimum_recent_units": 2
}
```

Omitted config fields receive core defaults; unknown fields are rejected. Other fixture forms
are `{"choice":{"side":"left","pile":"fresh_accumulation"}}` (also `right`,
`old_accumulation`, `single_fresh`) and `{"corridor":{"length":9,"workers":2}}`.
The examples are parity fixtures and demonstrations, not scientific sweeps.

## Outputs and replay

The CLI displays the final map and a concise integer summary. It writes four fixed filenames:

| File | Contents |
| --- | --- |
| `config.json` | Normalized shared configuration. |
| `episode.json` | Entire shared episode, including setup, actions, choices, sampled maps, snapshots, histories, rates, labels and storage. |
| `maps.txt` | Episode frames in order, each headed by `tick N fingerprint HEX`, followed by its ASCII map and a blank line. |
| `summary.json` | Exactly the episode's final integer snapshot. |

Seed is a decimal string, fingerprints are sixteen hexadecimal digits, and clocks use integer
ticks. Initial and terminal frames are retained, including terminal frames off the sampling
cadence. Choice fixtures stop after their first frontier selection, before committing an action;
they can have initial and terminal frames at the same tick. `completed_ticks` counts completed
worker permutation rounds, while `requested_ticks` retains the request. Sampling and diagnostics
do not consume random draws. Repeating a config, seed, ticks and sample interval reproduces
`episode.json` bytes on the same native build.

The WASM function `burrow_replay_json(config_json, seed, ticks, sample_every)` returns the same
serialized core episode. Its seed argument must contain only decimal digits and fit a `u64`;
negative, floating point and overflow strings are rejected. Ticks and sampling are checked as
finite integer JavaScript numbers before unsigned conversion: ticks must be in `0..=4294967295`,
and sampling in `1..=4294967295`. Fractional, negative, nonfinite and out-of-range values are
rejected with the corresponding field; core opportunity and ASCII limits still apply to valid
integers. Errors follow the existing boundary
contract: JSON strings of `[{"field":"...","message":"..."}]`. Full-record parity checks
cover all four examples at seeds `7` and `18446744073709551615` without adding web controls.

## Physical rules and observation limits

The world is a finite horizontal four-neighbor lattice with no wraparound, diagonals or gravity.
The growing fixture supplies exit `(0,12)`, open staging cells `x=0..2,y=10..14` and eight stable-ID
workers. Subsequent open cells arise from digs on accessible frontier faces. Each tick shuffles
workers once; each gets one sequential opportunity, observing earlier committed actions.

A worker carries zero or one unit; full hands cannot dig. No open cell holds more than two workers.
Digging opens one cell and creates one material unit; pickup/drop transfers an existing unit;
disposal at the exit removes a unit from active inventory. Birth times survive drops and handoffs.
Moves, digs, pickups, drops, disposal, blocked attempts and waits each cost one opportunity.
Blocked attempts and waits have separate totals. One unit is a bookkeeping quantity, not a
calibrated mass, and ticks/opportunities are not seconds or energy units.

Workers observe their current cell and open cells within two open-cell hops, occupants, loose
spoil and diggable faces adjoining those observed cells. They retain local frontier targets and
wait when congestion prevents routing to a retained target. They receive no global frontier
targets. The shortest exit-distance field and descending-neighbor information are explicitly
supplied global navigation scaffolds for transport, rebuilt after geometry changes. Neither
controller has a room count, desired nest shape or global construction plan.

Direct transport carries toward disposal; relay transport can drop after three successful loaded
moves when still away from the exit. Responsive selection weights locally observed frontier
approaches with at least two recent units by three. Recent means age strictly less than 32 ticks
in these examples. Blindness removes only this selection influence; workers still see and
transport loose spoil. Preferences and staging are supplied, not learned.

ASCII glyphs are `#` solid, `.` open, `E` exit, `o` loose material, `w` unloaded worker and `W`
loaded worker. Exit and worker overlays hide spoil, and one glyph can represent two occupants;
a loaded co-occupant takes precedence over an unloaded worker. Maps are projections. Consult
trace and inventory for hidden holdings, piles and multiplicity.

## Accounting and operational limits

The inventory obeys `initial + excavated = carried + loose + disposed`. Authoritative integer
snapshots record actions, opportunities, inventory and connected open area. Connected area follows
accessible-frontier construction and does not classify success. Rates divide digs/disposals by
actual opportunities; with zero opportunities both are JSON `null` (unavailable).

Delivery histories retain material IDs, birth/disposal times, deduplicated carrier IDs, successful
loaded moves and observed loose waiting. `disposed_at: null` means a censored delivery. Delivered
age is disposal tick minus birth; unfinished age is final tick minus birth and must retain its
censoring label. Old fixture units have supplied prior birth history; their observed waiting
starts at fixture initialization, not at birth. Per-worker work, loaded/unloaded successful travel,
spatial work and event-time dig distances are exported. Spatial coordinates identify the excavated
target cell; `ActionEvent.from` records where the worker stood. A dig's distance is minimum prior
open-neighbor exit distance plus one, frozen at opening even if a later shortcut appears.

BFS diagnostics separately measure exit-field, observation and controller-route calls, visited
cells and maximum queue lengths. These disjoint measured searches count actual algorithm work;
they are separate from action costs. Exit fields initialize even with zero ticks and rebuild only
after geometry changes. `storage` reports logical collection records, retained ASCII bytes and
peak record/cell counts, independent of allocator capacity and machine word size. `retained_records`
sums events, choices, frames, snapshots, deliveries, carrier IDs, worker work, spatial work,
spatial worker IDs and dig distances; it excludes singleton config/setup/summary objects.
These measurements do not replace separate wall-clock profiling.

Operational caps are 262144 cells, 4096 workers (also constrained by fixture spawn capacity),
one million requested worker opportunities and checked clock/weight/dimension arithmetic. Growing
staging has capacity for 28 workers because the exit is excluded from spawning. Retained ASCII
is conservatively capped at 64 MiB before stepping or reserving replay storage. Frame bytes equal
`width * height + height + legend UTF-8 bytes`; requested frame count is one for zero ticks,
otherwise `2 + floor(ticks / sample_every)`. The checked product must fit the cap. This can reject
a request that would stop early or retain fewer frames; reduce requested ticks or increase the
sample interval. These are lab resource limits, not physical calibration.

## Approved designs and implementation sequence

The measured-study protocol and resource-access design were approved on 2026-10-04. Their separate implementation plans await review:

1. [Measured archive and analysis harness](superpowers/plans/2026-10-04-burrow-1-measured-harness.md): physical transaction validation, complete manifest, immutable raw archives and saved-only descriptive analysis. Engineering acceptance uses construction seeds; scientific registration and execution remain separate.
2. [Resource access](superpowers/plans/2026-10-04-burrow-2-resource-access.md): private local goal guidance, event-time structural access and checked CLI/WASM exports, preserving ordinary excavation replay bytes.

Implement and review the harness first, then resource access in a separate branch. Neither implementation depends on inspecting scientific treatment outcomes.
