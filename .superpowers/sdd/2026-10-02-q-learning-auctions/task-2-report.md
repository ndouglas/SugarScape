# Task 2 report — CLI, WASM, playground, and finite sweeps

## Patch sequence and red/green evidence

Applied `02-hosts-web-sweeps-tests.patch` before `02-hosts-web-sweeps-implementation.patch`. The required focused test failed before implementation with the expected boundary mismatch:

- Actual: `finished at tick 4 (its end year)`
- Expected: `finished at tick 4 (its last auction)`
- Exit code: 101
- Log: `/tmp/auctions-task2-red.log`

After applying the implementation patch, the focused case passed as part of the CLI suite. The complete CLI suite passed (4 unit tests and 27 integration tests; exit 0; `/tmp/auctions-task2-cli.log`).

## Automated checks

- `wasm-pack test --node crates/sugarscape-wasm`: 79 passed, 0 failed; exit 0; `/tmp/auctions-task2-wasm.log`. Includes the five auction fingerprints and exact 23-period/7-period batching with immutable completion.
- `cargo clippy --workspace --all-targets -- -D warnings`: exit 0; `/tmp/auctions-task2-clippy.log`.
- `cargo fmt --all --check`: exit 0; `/tmp/auctions-task2-fmt.log`.
- `npm ci`: exit 0, no vulnerabilities; `/tmp/auctions-task2-npm-ci.log`.
- `npm run build`: exit 0, including fresh release WASM generation and Vite production bundle; `/tmp/auctions-task2-build.log`.
- `npm test`: 56 files and 881 tests passed; exit 0; `/tmp/auctions-task2-web-test.log`.
- `npm run typecheck`: exit 0; `/tmp/auctions-task2-typecheck.log`.

The first web test/typecheck invocation happened concurrently with the first WASM build and failed because the generated `web/src/wasm-pkg/sugarscape.js` had not yet been emitted. After the build completed, both required checks passed. No code change was needed for that ordering issue.

## Browser smoke evidence

Used the local playground at `http://127.0.0.1:5173/` through the browser tool. The generated WASM app loaded without console errors.

- Default auction preset: the browser reached `t = 1000` and displayed `1000000 of 1000000 periods` and `1000000 auctions counted`; the whole-run bid view rendered both bidder axes.
- Q-value view rendered the caption `Rows per bidder: Q, chosen, updated`, with gold greedy and cyan played labels.
- Final-window bid view rendered `final 20%` in its caption.
- Three-bidder preset rendered the `first-two-bidder projection` label, and the series control included bidder 3 fields.
- Persistent preset rendered a 100,000,000-period horizon. One Step advanced to `t = 1`, `1000 of 100000000 periods`, and `1000 auctions counted`, with whole-run bid axes in the caption.
- One browser attempt to compare entered a `Copying A…` state and left the page controls disabled after a one-second wait. The page was reloaded; comparison interaction and transient-stability completion text were not verified in the browser. Automated comparison/form tests passed in the 881-test web suite.
- An attempted in-browser short-horizon edit changed the visible number fields but the live session still used its preset horizon. The smoke therefore used one bounded Step for the 100m preset, not a long run. This does not affect the automated exact-horizon tests.

## Host horizon and sweep validation

`AuctionsWorld::is_finished()` returns true when economic periods reach the configured horizon. Generic host stepping checks `finished()` during a batch, so the final 23-period session with 7 periods per tick stops at the exact horizon after four host ticks; it does not overshoot to 28 economic periods. The CLI’s focused red/green test confirms the new `last auction` stop reason at the partial final batch.

`ModelConfig::max_ticks()` remains `None` for auctions. In `sweep.rs`, this means the static `ticks > max_ticks` validation is skipped for auction configs. Runtime completion still honors the configured horizon through `finished()`, while each built-in sweep carries its own requested finite tick cap. This does not create horizon overshoot, but the static validator does not reject an auction sweep whose requested ticks exceed that auction config’s effective host-tick horizon.

## Current state

Stage 2 is marked In Progress in `IMPLEMENTATION_PLAN.md`. No commit was created, per the controller’s instruction to pause before committing. Browser comparison and transient-stability checks remain incomplete for controller-directed recovery.
