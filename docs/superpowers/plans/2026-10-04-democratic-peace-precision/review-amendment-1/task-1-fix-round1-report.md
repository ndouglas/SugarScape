# Task 1 correction, round 1

Base: `8b02b9dd978137aa356db55093899aff50e71ce6`.

The native schema-2 validate-only path now rejects supplied follow-up activation, historical, literal-session, and checkpoint options immediately after manifest preparation, before receipt loading, alias checks, or writes. Ordinary validate-only options remain accepted, and this guard does not require activation to validate resolved configurations. The change is limited to `survey/src/bin/democratic_peace.rs` and `survey/tests/democratic_peace_native.rs`.

The focused behavioral regression covers all nine gate options. It checks exact-path, symlink, and hardlink `--historical-binary` aliases against temporary sentinels, plus a supplied historical study root with the resolved output inside it. All cases require the specific unsupported-option error and preserve sentinel bytes/link identity. No historical scientific artifact or source/binary/probe evidence was touched.

Red/green evidence:

- Focused test-only run failed as expected at the rejection assertion because the validate-only command returned success for the exact `--resolved`/`--historical-binary` alias. Log: `task-1-fix-round1-red.log`.
- Focused test after the guard passed (1 test). Log: `task-1-fix-round1-green.log`.
- Full native target passed: 9 unit and 8 integration tests. Log: `task-1-fix-round1-native.log`.
- `cargo fmt --manifest-path survey/Cargo.toml --all --check` passed. Log: `task-1-fix-round1-fmt.log`.
- `cargo fmt --all --check` passed with empty output. Log: `task-1-fix-round1-fmt-root.log`.
- `git diff --check` passed; recovery also checked `git diff --check 8b02b9d HEAD`.
- The unchanged Python protocol suite retains its original 55-pass evidence in `task-1-green.log`; neither Python code nor its tests changed in this correction.

The test-only patch and implementation-only patch are retained separately as `task-1-fix-round1-red.patch` and `task-1-fix-round1-implementation.patch` for the controller's additive replay overlay. The original replay packet was not changed. An initial test compilation attempt used an invalid Unix module import for `hard_link`; it was corrected to `std::fs::hard_link` before the meaningful red run. No other concerns found. Independent controller review remains pending.

Recovery verification found the correction already committed as `a58eaa9` (only the two native files above). Both retained additive patches are byte-identical to the corresponding committed diff against the stated base, including the test-only red patch. Recovery inspected the committed guard and complete regression and found no further scoped concerns.
