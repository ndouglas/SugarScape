# GeoSim amended premeasurement packet — 2026-10-03

This dated packet incorporates the reviewed Task 1 correction at `0b26aa867411526256b5bf8af9ee440eb9521efe`. [AMENDMENT.md](AMENDMENT.md) records its scope and old/new bindings. The original approved packet remains unchanged in `geosim-plan` at `65bfa9eae59622a824d7c6f09db93b68c83c1dbb`.

Use the same six ordered execution gates from source-only base `4ab07d31fedc7e8f3cf35a36252da71218668fce`. The current execution tree is already verified at amended checkpoint 2; continue through checkpoint 3 only when the Task 2 test gate begins.

```sh
python3 docs/superpowers/plans/2026-10-03-geosim-amended/replay.py --target . --through 2 --verify-only
python3 docs/superpowers/plans/2026-10-03-geosim-amended/replay.py --target . --through 3
# Continue through 4, 5 and 6 only after their planned gates.
python3 docs/superpowers/plans/2026-10-03-geosim-amended/replay.py --target . --verify-only
python3 -m unittest discover -s docs/superpowers/plans/2026-10-03-geosim-amended -p test_replay.py -v
```

`replay.py` and `test_replay.py` are exact copies of the original tools. Stage 1 remains the original test-first discovery RED; stages 3–6 retain the original patch bytes. `manifest.json` binds exact patch bytes, checkpoint inventories, preimages, postimages and executable modes. Only the reviewed normative design is newly classified as an explicit product file; every other protected source remains protected. The exact new packet directory is preparation metadata. No other exclusion changes.

`original-manifest.json` and `original-study-manifest.json` retain the old declarations. `study-manifest.json` is regenerated from the complete amended temporary product tree; it is preparation evidence, not an execution binary receipt or registered result. `declaration-differences.json` and `amendment-verification.json` retain exact differences, actual verification counts, hashes and evidence/worktree locations. `evidence-preservation.json` binds the retained generation scripts, logs, inventories and temporary indexes before cleanup. No registered histories were run.
