# GeoSim verified preparation packet

Apply this packet only in a worktree based on source-only commit `4ab07d31fedc7e8f3cf35a36252da71218668fce`. The six ordered patches recreate frozen scratch product commit `1d5d1cc` exactly; they do not contain raw archives, downloaded author code, generated binaries, or scientific outcomes. The source-only base retains the corrected source audits and authored reference tools.

```sh
python3 docs/superpowers/plans/2026-10-03-geosim/replay.py --target . --through 1
# Continue through 2, 3, 4, 5 and 6 after each planned gate.
python3 docs/superpowers/plans/2026-10-03-geosim/replay.py --target . --verify-only
python3 -m unittest discover -s docs/superpowers/plans/2026-10-03-geosim -p test_replay.py
```

`manifest.json` binds each patch SHA256, complete checkpoint inventories, preimages, postimages and Git executable modes. Replay rejects changed protected sources, unexpected product files, unsafe paths, symlink ancestors and unsupported modes. Applying patches requires source-base ancestry. Read-only verification can certify independently committed identical trees without shared ancestry. Explicit metadata exclusions cover this packet, its plan, intended generated findings/declaration paths, and local `papers`/`target` references; they do not exempt normative specs or authored reference tools. Ignored raw receipts and process ledgers are outside product scope.

Test patches include exact collocated test modules before their production definitions. Scientific tests also include the recorder dependency manifest and lockfile so `cargo test --locked` reaches missing recorder definitions without changing a tracked preimage. The final implementation restores exact frozen bytes.

`replay-verification.json` records retained actual RED/GREEN receipts, independent clean application, final whole-tree equality and byte-equal regenerated scientific declaration. `study-manifest.json` is the premeasurement declaration; `preparation-verification.json` records the controller's declaration validation and unregistered runtime benchmark. These assets are preparation evidence, not registered findings. Archived artifact identity remains Unresolved.

The separate actual plan governs execution and integration. Main advanced independently to the reviewed Minds implementation; resolve aggregate integration conflicts without overwriting that work. Do not use a scratch runtime receipt in place of the actual execution tree's rebuilt source/binary bindings.
