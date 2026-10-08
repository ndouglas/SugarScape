# P4 durable archive preparation — initial verified preservation

Prepared 2026-10-08T07:14:00.046591+00:00 by `/root/p4_durable_archive_preparer`.
Status: **initial scientific/historical/environment preservation and compact public archive preparations verified; final empirical/integration/publication supplement and full closure pending**.

Durable root: `/Users/nathan/.local/share/sugarscape/evidence/minds-deception-2026-10-07`. Source root: `/private/tmp/sugarscape-minds-p4-20261007-vb9tv64o`. Exclusively created new archive; no prior archive replaced. All source copies are under `preserved/`; exact external Cargo sources and Rust infrastructure are under `linked-environment/`. Regular file count across preserved source and exact external environment inventories: 157,924; logical regular-file bytes: 54,416,607,144. Logical totals include build/sparse/hardlink-path multiplicity and are not extra physical disk consumption. Native APFS clones preserve bytes without materializing another full target. Files and source histories were not deleted.

## Frozen scientific identity

Accepted scientific revision remains `e10f98d99586d94f5adc8eab1f95b80e5c5b1865`, separate from later authorized integration. `metadata/frozen-scientific-identity-verification.json` verifies every registered 2,385 source identity entry, every 411 packet identity entry and every 11,525 raw campaign identity entry against the durable copies. Zero differences in SHA256, byte count, and full mode.

- Canonical source inventory SHA256 `29fcabbd65973632740b86ed2a9496f336c5ab489dee87536ede5d960cedaf42`.
- Packet inventory SHA256 `0d20a539df5f11947671b82fce57388ca1be216649a6cf5adac795650f2489c8`.
- Accepted native executable SHA256 `38f7137b7b92521b8e5d82c43b883c1f7dc92f9151112c73665913299c54caa4`, 19,118,640 bytes, mode `100755`.
- Protocol SHA256 `7e0b9a7fb6973a3ba95fad70ddf911105f753a73b7fe45a8ab5040e3d6901d4d`; manifest SHA256 `cd4996a57ccc216dac634994817796c16a0cc6dc45f48b4e903073b3700ee2be`.
- Full campaign: 11,525 files; 2,556,882,562 bytes. Saved reanalysis: 2 files; 1,052,134,482 bytes. Both source/durable hashes and modes match for every file; campaign inventory SHA256 `cadf7f7f0768e7b50ad6d640a754a808017407bbad674833fe2d1aef390452ff` remains unchanged.

`metadata/preserved-*-inventory.jsonl` records original-current and archive bytes/modes/SHA256 for every regular file, plus directory modes and every symlink text. Historical source, original/fix packets, construction outputs, all retained red/green/full-gate receipts, failed historical execution attempts and retained scripts are copied, not replaced by newer evidence. Exact full Git histories of both frozen source directories are included.

## Infrastructure and reconstructed environment

The pre-integration target snapshot was completed around 2026-10-08 03:07 America/New_York. Integration agent held all target writes until full original-versus-clone verification finished, then was explicitly released. `infrastructure-target-inventory.jsonl` covers 72,818 regular files, 1,761 directories, 47,627,899,472 logical regular-file bytes, and zero hash/mode differences. Its SHA256 is `57746a97daa715f0f47978ddc222b1ba7bc5a1d0b7e29bb660bf3d14120bf65b`. This is a separate infrastructure identity; it does not replace the frozen scientific source/packet identities. Later builds may legitimately change the live target; the archived pre-integration copy stays unchanged.

`linked-environment-map.json` preserves original Cargo links and absolute targets. Read-only linked sources `/Users/nathan/.cargo/bin`, `/Users/nathan/.cargo/git`, `/Users/nathan/.cargo/registry` were materialized with APFS clones only into this new archive, with matching full inventories. No shared cache was changed. Rust receipt identifies rustc/Cargo 1.98.1, host aarch64-apple-darwin, LLVM22.1.8; only `/Users/nathan/.rustup/toolchains/1.98.1-aarch64-apple-darwin` was preserved, including its actual `wasm32-unknown-unknown` standard library target. Its 214 regular files total 570,654,712 bytes, all verified. The Cargo rustup proxy's resolved binary is separately preserved and verified; no unrelated toolchain, user setting or credential was copied.

`reconstruction/cargo-home` is a **new owned** Cargo environment with relative symlinks to the materialized archive. `preserved/cargo-home` retains the original exact bytes and link texts. `reconstruction-cargo-map.json`, `rust-toolchain-map.json`, and `all-preserved-symlinks.json` document mappings. Direct archived Rust toolchain `bin` can be selected ahead of proxies when restoring the environment. The frozen native binary embeds the original absolute scientific source root; reconstruction must restore that path or provide an explicit mapping in a new recovery environment. Original absolute symlinks are intentionally not silently rewritten. This preparation did not run a restored executable or claim that relocation alone proves runtime reproducibility. macOS system libraries/SDK, Homebrew Node/npm/Git and other host tools remain external platform requirements, with original version/path receipts retained; their complete operating system installations were not copied.

## Mutable snapshots and retained first audit failure

The initial copy intentionally excluded **only** the actively written `checkout/.superpowers/sdd/2026-10-07-minds-deception/scientific-execution/actual-artifacts` directory; all other initial top-level content was retained. Active scientific-execution progress and figure-prep files, integration logs and later source changes need supplemental final snapshots. No active output is called final.

The first read-only inventory traversed the live mutable checkout after authorized integration created `.git/ORIG_HEAD`, absent from the earlier snapshot. That verification aborted with FileNotFoundError; its exact log and partial inventory are retained. No biological file, source or cache was repaired or changed. `coordination-event.json` records the correction: classify mutable checkout as a dated snapshot, retain its exact hashes, and independently verify frozen scientific directories. The revised checkout summary records 10 live-current differences (Git index/log/branch ref, current figure-prep files/progress, three upstream-integrated source files); these are not frozen scientific mismatches. Additional live entries created after the snapshot were not part of the initial point-in-time copy and require a supplement. There was one initial read-only inventory abort and one successful corrected approach, not three failed attempts or any scientific rerun.

## Compact lossless public preparation

Local `public/` preparations are unpublished. GNU tar uses sorted names, GNU format, mtime0, owner/group0 and numeric owners; gzip `-n -6` suppresses timestamps. Every regular extracted member is streaming-hashed against its preserved original, and every mode and symlink text checked. Tar time/ownership normalization does not change file bytes or permission modes. Extraction verification has zero differences; exact extraction inventories and compressed hashes are bound by `PUBLIC-ARCHIVE-MANIFEST.json` and `SHA256SUMS`.

| Recommended filename | Exact bytes | Content |
|---|---:|---|
| `p4-e10f98d-registered-campaign.tar.gz` | 49,295,835 | Full raw campaign, all starts/frames/outcomes/indexes/census/full analysis; no filtering |
| `p4-e10f98d-saved-reanalysis.tar.gz` | 17,680,810 | Both exact saved-data outputs |
| `p4-e10f98d-accepted-evidence.tar.gz` | 48,777,544 | Full frozen source/Git history, accepted packet/native+WASM, prospective and measurement receipts, dated initial execution snapshot |

All three fit the requested GitHub100MB per-file ceiling. Split fallback limit was95,000,000bytes; each fits one part, renamed to normal `.tar.gz`. Total compressed payload115,754,189bytes. Build targets, Cargo registry and other infrastructure caches stay in the full durable archive and are omitted only from compact public bundles. The public route needs one copy of the complete raw campaign; avoid committing duplicate full campaign copies elsewhere. Empirical/final report artifacts and review/publication receipts belong in an additive fourth supplement after their own gates.

## Exact command and verification records

Reproducible exact argv and script bytes are preserved in `metadata/archive_preparation.py`, `archive_verification.py`, `public_archive_preparation.py`, `clone-commands.json`, per-bundle `*-command.json`, `public-canonical-filename-renames.json`, `rust-toolchain-map.json`, and `reconstruction-cargo-map.json`; first and corrected logs are retained. Clone command is `/bin/cp -cRp SOURCE NEW_DESTINATION`; individual proxy/config copies use `/bin/cp -cp`. No fallback ordinary full target copy was used. Inventory reads use Python SHA256 over all original/copied bytes; public extraction validation uses `/bin/cat`, gzip stream and tar stream without additional temporary full raw copies. Source checkout HEAD/status were checked with Git read-only commands; no Git commit/merge/push was done by this preparer.

This preparer invoked no World/RNG, simulation, saved-data native reanalysis, construction driver or scientific validator; created no helper agent; changed no scientific source, data or executable; and deleted nothing. Report and archive-owned metadata are the only writes outside new archive storage. Original immutable science was verified by its source/packet/campaign inventories after copying, and source HEAD remained e10f98d with empty full status. Authorized concurrent integration writes are separately owned, with snapshot/current differences disclosed.

## Pending final phase

Append-only supplemental snapshot of completed actual-artifacts; completed empirical review and report; integration gates/CI; public source/report integration; merge/push; Pages deployment and external publication receipts; exact final report URL; final durable archive manifest. None is claimed complete here. Future supplements must retain earlier copies and recorded failures, bind all final file hashes/modes, and leave accepted scientific revision e10f98d unchanged. This report establishes preparation, not final publication or experiment closure.
