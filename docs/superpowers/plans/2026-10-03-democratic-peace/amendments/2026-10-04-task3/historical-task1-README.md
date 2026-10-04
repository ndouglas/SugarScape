# Premeasurement Task 1 replay amendment — 2026-10-04

This complete six-stage packet packages the reviewed Task 1 repairs before any registered measurement. The source-only base remains `4cf60ebf47e9e6585aa366ac1f2cca1d8b2456e9`. Original execution checkpoint 2 is `64636d717905000f72d2338d105f5e5cc72ff154`; repairs are `39d4164d8c71176fd76dd047b732b4c604b75218` (pool recomputation and census validation) and `4f6e791292a7d5c66bd7523acc4a3c2ad3340890` (exact live-capital generation). Corrected execution checkpoint 2 is `4f6e791292a7d5c66bd7523acc4a3c2ad3340890`. This amendment changes metadata only; the reviewed product repairs already exist in those commits.

The repairs recompute surviving alliance pools after structural pruning and reject invalid ownership, membership, IDs, capital generation and front topology before committing a candidate period. Four democratic-peace canonical-state fixture hashes changed in the first repair because corrected pools enter state hashing; no other-model golden values changed. The source table, approved spec, source defaults, probability readings, allocation/combat/release algorithms, scientific populations, seeds, source checks, analysis methods and judge remain fixed, apart from the reviewed invariant/pool repairs. No registered histories or runtime probes were run for this amendment. Bounded regression fixtures in the copied logs are prior Task 1 verification, not registered measurement.

## Packet identity

New `manifest.json` SHA256: `c533fc03519efa2cb2c2d91ca002ced49179eb161007162a1aebae3ee79954bf`.

| Patch | SHA256 |
|---|---|
| `01-core-tests.patch` | `9540a8bdfee9c82773b90580c7534987cb6332c474878fb383d536840f346869` |
| `02-core.patch` | `ab42711984262ed2fccb18cfb69faacfff209ba12f97d9975f33f9455def8ef7` |
| `03-host-tests.patch` | `7b496c6429af1d66e5c315865f146f70d93b21c7acdd28b7e358204c62f95b52` |
| `04-hosts.patch` | `758dd3dbe3753e2fc3e4d0a3c2867c7b493e1cb18dfbda8eba63ecd3ca6e62ea` |
| `05-protocol-tests.patch` | `a9e041d2996d123051ac29b7814a2907cbacf629407cb25cd8ac06f8ffc35dae` |
| `06-protocol.patch` | `26fbb856c5baf491716a4366e8095cbeff85305e79ca27278e88a1ee1f988c18` |

Patch 02 is one normal `git diff --cached --binary --full-index CHECKPOINT1_TREE`, produced in an isolated owned worktree/index after constructing original checkpoint 1. Its declared changes are derived from amended checkpoints 1/2, and its hash is computed from its bytes. Patches 01/03/04/05/06 are exact original copies. Checkpoints 0/1 and every mode remain unchanged. Checkpoints 2–6 substitute only the reviewed `claims.rs`, `territory.rs`, `tests.rs` and core `golden.rs` images. Patches 03–06 have no overlap with these four paths. All 897 protected images and the original source base remain unchanged. Local `.gitattributes` preserves the normal patch's empty context records and exact transcripts' final blank lines when checking whitespace; source-file formatting checks are unaffected.

`original-packet-inventory.json` records the preserved top-level original packet bytes/modes. The exact historical copies `original-preparation-verification.json`, `original-replay-verification.json`, `original-plan-self-review.json` and `original-README.md` document preparation of the original packet. Their product test totals, scheduling probes and scratch receipts are historical evidence for the original images; they do not certify amended bytes. Existing scratch/replay worktrees and their ignored receipts are preserved.

## Fresh amendment verification

[construction-verification.json](construction-verification.json) retains the actual construction commands, exit codes, checkpoint-1 tree and isolated worktree. [replay-verification.json](replay-verification.json) retains fresh commands, exit codes, complete final and execution-checkpoint-2 inventories, original hashes and retained regression evidence hashes.

Fresh replay worktree: `/Users/nathan/.config/superpowers/worktrees/SugarScape/democratic-peace-task1-amend-replay-d4e17d0bea30`. All six checkpoints passed in order from the original source base. Final 613 product + 897 protected = 1510 file paths/bytes/Git modes equal the prior original final replay at `/Users/nathan/.config/superpowers/worktrees/SugarScape/democratic-peace-replay-normalized` with only the four reviewed Task 1 images substituted. Execution verifies as amended checkpoint 2 (578 product + 897 protected files). All 18 replay positive/rejection tests passed. No product suites, registered worlds, runtime probes, inference or resampling were rerun.

```bash
amended_assets="$HOME/.config/superpowers/worktrees/SugarScape/democratic-peace/docs/superpowers/plans/2026-10-03-democratic-peace/amendments/2026-10-04-task1"
python3 "$amended_assets/replay.py" --target "$HOME/.config/superpowers/worktrees/SugarScape/democratic-peace" --through 2 --verify-only
python3 -m unittest discover -s "$amended_assets" -p test_replay.py
# Reproduce fresh replay/inventory verification in another uniquely owned worktree:
python3 "$amended_assets/verify_amendment.py" --execution "$HOME/.config/superpowers/worktrees/SugarScape/democratic-peace" --original-final "$HOME/.config/superpowers/worktrees/SugarScape/democratic-peace-replay-normalized" --worktree-root "$HOME/.config/superpowers/worktrees/SugarScape" --output /tmp/democratic-peace-task1-amendment-verification.json
```

The reproduction command requires execution to remain at checkpoint 2; continue tasks 2/3 through the same six-stage replay helper at their existing plan gates. Helper/test paths are local exact copies: `replay.py`, `test_replay.py`, `prepare_checks.py`, `register_study.py`, `publish_findings.py`. `verify_amendment.py` adds packaging verification only. `evidence/build-amendment.py` preserves the one-time construction script; it requires a new destination and must not overwrite this packet.

## Witnessed regression evidence and limitations

Exact original transcripts and scoped reviews are retained under `evidence/`. `dp-task1-fix-red.log` witnesses three behavioral failures (pool 3 instead of 2, invalid same-length membership accepted, corrupt candidate committed); `dp-task1-fix-focused-all.log` records 53 passing domain fixtures, three discovery fixtures and the democratic-peace reproducibility fixture. `dp-task1-fix2-red.log` witnesses counter-ahead/live-sovereign mismatch returning `Ok(())` (0 passed/1 failed); `dp-task1-fix2-census.log` records its focused GREEN (1 passed), and `dp-task1-fix2-domain.log` records all 53 domain fixtures passing. Both rounds retain golden and Clippy logs; final golden verification records 8 passed/3 existing ignored. The first scoped review closes the pool finding and identifies the remaining generation mismatch; the second scoped review closes that residual Important finding with no new Critical/Important breakage. These are witnessed historical fix executions, not reruns by packaging.

Copied formatting logs are empty and do not independently retain their command exit status; the implementer report records success. Scoped review does not prove original-2001 executable identity or scientific compatibility. The prior JSON-formatting Minor remains deferred. Later tasks must rebuild/bind their own sources and binary, run their stated gates, and retain new execution runtime evidence before registration; old scratch receipts are not substituted.
