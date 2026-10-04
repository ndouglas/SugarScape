# GeoSim postmeasurement recorder and presentation amendment

Dated 2026-10-03. This amendment applies to recorder boundaries after the registered study and to the readable presentation of its retained results. It does not amend the frozen scientific design, estimators, model mechanics, RNG, jobs, arms, populations, tolerances or measured verdicts. No registered history or full analysis was rerun.

## Original measurement identity and preservation

The registered observations were generated from measured source commit `d64d3becd3682df4edeab90e62a1ee9f0526e1d5`, scientific inventory SHA256 `5d7f78abd30f8dee86abb63f11f110b36a7119f554c0ae0d663a9aab7f579181`, and recorder binary SHA256 `d2144adf6d4b24c12239d08d05b3aa7ea8596c08724009d53a9be2f8da9f24ff`. Their original frozen manifest, build receipt and exact resolved configuration remain authoritative for that measurement. The source design and source/protocol freeze occurred before registered periods; the postmeasurement preservation described here is a separate obligation and does not substitute for that freeze.

Before correction, `survey/out/geosim-execution-task6/pre-fix-source-seal/` retained and independently verified all 1,426 measured Git blobs and modes, all 325 inventory entries, the original executable, bindings, raw histories and complete analysis, and both generated and committed presentations. Its measured source tar SHA256 is `6691c005b4089867fd9716df2e8ea3ad15e2676ce7a0109d88992e269fd29836`. The seal records original and copied byte counts and hashes; the measured source remains reconstructible independently of the later checkout.

The original immutable bindings are:

| Artifact | SHA256 |
|---|---|
| Frozen manifest | `7e6baf8445230187f1d4169b114fecb9bd698b3569a3af79188230b41eb287d4` |
| Task 4 build receipt | `3dada7cf7217b57296dc78f258bd0b7f37912722a519283dcdb199ac2770deb8` |
| Task 4 exact resolved export file | `8c17439878df6d6b8ddac4a5c85a1937b7bc9cfff71134359b274b576c5a0c68` |
| Exact native resolved payload | `176583d2823ced2f898adcddbe486ab86fde68f6188a5c077f5769f0fde46f75` |
| Task 5 registered raw union | `6388068c85f2525d18b781d011a3c605686ca73456f3c9b6239ff92a3ecae4ad` |
| Task 5 full analysis | `5d6d3c26710d9d454f2d41386c62734de83ef34e133f554b840ca99b8e103ee4` |
| Committed compact matrix | `9aca8f622576cd6ca6c39bea3b3c424f5b4d25a6e16860b74c91b3adc871ca09` |

The observed study uses distinct Task 5 `sessions.jsonl` and Task 4 `resolved.json` destinations. All 1,490 retained attempts are 1,486 completed and four arithmetic-invalid histories; none has construction-error, construction-panic or incomplete status. The reviewed defects do not affect these observed paths or statuses. Every old row remains bound to the archived original source and original binary, never to the corrected executable. The frozen declaration and old source inventory were not regenerated.

## Recorder boundary correction

Code commits `d27844160c9a29e66c0a8d64f5eb6dae5fc5a4c3` and `6e601c922cfc69b6f78007b54f9b054c3ea40093` change only the recorder's destination protection and attempt-diagnostic validation, the corresponding Python record boundary, and their tests.

Before correction, lexically different `--out` and `--resolved-out` spellings could identify one destination, allowing validation or execution to replace accepted history bytes with a resolved export. The recorder now resolves existing ancestors and symlinks, normalizes relative/absolute and parent-component spellings, and compares destinations before any writes or resume acceptance. Missing distinct ancestors remain supported; equivalent missing destinations reject without creating ancestors. Dangling symlinks and non-directory ancestors fail before exports are written. Existing case aliases are resolved by canonicalization; initially missing ASCII-case-equivalent names and ancestors are conservatively rejected on macOS and Windows, including their optional case-sensitive volumes. A host probe reproduced mixed resolved/history output before this final guard; validation and execution now reject without creating either output.

The atomic export previously truncated its predictable `tmp-PID` pathname if an existing history or symlink occupied it. It now creates the temporary file exclusively (`create_new`), rejecting a collision without truncating or following the existing object. The focused actual-process fixture uses Bash `exec` to retain the recorder PID and demonstrates unchanged history bytes at that temporary pathname.

Native resume now requires each construction-error member to be exactly `{field: string, message: string}` and rejects unknown, missing, wrongly typed and non-object members. Required construction/implementation-panic context and incomplete recorder context must contain non-whitespace text. Python already stripped panic context, but previously checked incomplete context by truthiness; it now also strips required incomplete context. Native whitespace handling includes Python's four additional ASCII separators U+001C–U+001F. Valid construction-error, panic and incomplete rows remain retainable; failed keys are never replaced.

Focused checks passed: 19 native unit tests, 11 actual-process integration tests, all 35 Python record tests, Clippy with warnings denied, and formatting. An independently built new release agreed with Python on 28 attempt cases across 56 validation/execution probes, including empty, wrong-type, ordinary whitespace and U+0085/U+001C context, and valid failures. These are temporary synthetic fixtures only.

## New build identity

The corrected recorder was built in the separate `survey/out/geosim-postmeasurement-fix-build` target directory. All inventoried scientific sources match code commit `6e601c922cfc69b6f78007b54f9b054c3ea40093`; pending presentation documents are outside that roster. That code produced a new 325-entry source inventory SHA256 `5a4c8c65cfc69c9a9f59860a751ccb617a4f8cbe90f55e5e134e93c146349a66`; independently recomputed prebuild and postbuild inventories match. The actual new release binary SHA256 is `794d9fca4fb89337b3d77e6fca82d3b34e42aa04a9e6ea4e73c8238409787685`. The new receipt is `survey/out/geosim-postmeasurement-fix/postmeasurement-build-receipt.json`, explicitly classified as a postmeasurement recorder-safety build, with code identity, actual command, toolchain, locks, binary and pre/post source hashes. It certifies no registered observations. The new source identity and eventual integrated main identity differ from the original measured identity by design.

## Presentation correction and provenance

Only the committed readable Markdown was manually corrected. Eight available `source.grid75` targets now display `Available` instead of `Unavailable` in the reason column. The execution appendix now distinguishes all 72 reserved child-job slots from eligible jobs that performed the prescribed 100,000 resamples; unavailable jobs generated no resamples. All 22 pooled KS jobs performed 1,000 successful generated-and-refitted replicates each. This clarifies existing execution evidence without recomputing a judge or altering the reporting module.

| Presentation | Location | SHA256 |
|---|---|---|
| Original generated | `survey/out/geosim-execution-task5/findings.md` and its sealed copy | `fde9a064dbcf8f6adcdb98ce5e7d747397d167e8d1911d7683f2ccee07184a1b` |
| Original committed | `docs/superpowers/specs/2026-10-03-geosim-findings.md` at `6a1b36f03fb603f7d231d9895692f928e7325640`, and its sealed copy | `3f4f986254692c4c38eeb1aa32a8cc17d2d0009b122f9c5a4a91abcc942276bd` |
| Corrected presentation | [Current findings](2026-10-03-geosim-findings.md) | `a8854a283793a15a6fdf6825bc46deb6b3ec4691ee481316a4f7d2f991660686` |

The [current provenance](2026-10-03-geosim-provenance.json) updates the current Markdown entry's actual bytes and SHA, retains the original generated entry, and separately identifies the original committed presentation, original provenance and this correction. All matrix fields and original scientific bindings are unchanged. Original provenance SHA256 `768695266d0561dac10154c459eb218fb0c602ada0458fd764ee49a8e2a91138` remains retained at the old findings head and in the pre-fix seal.

## Deferred maintenance ruling

The final-review ledger retains three nonblocking minors: direct injected inner-step panic coverage, ordinary parity subprocess output, and existing WASM packaging metadata notices. Existing invalidity/outer-panic and actual parity evidence support current behavior; none demonstrates a changed measured result. Those maintenance items remain deferred. Independent scoped rereview and the controller's final Task 6 gates still follow this amendment.
