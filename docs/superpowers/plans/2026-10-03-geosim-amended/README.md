# GeoSim amended premeasurement packet — 2026-10-03

This dated packet incorporates the reviewed Task 1 correction at `0b26aa867411526256b5bf8af9ee440eb9521efe` and Task 2 accessibility correction at `9d53d4ef8be6809ad6374bad9993a02458af596e`. [ACCESSIBILITY-AMENDMENT.md](ACCESSIBILITY-AMENDMENT.md) records the current refresh; [AMENDMENT.md](AMENDMENT.md) records historical Task 1 revision `cd1de5f`. The original approved packet remains unchanged in `geosim-plan` at `65bfa9eae59622a824d7c6f09db93b68c83c1dbb`.

Use the same six ordered execution gates from source-only base `4ab07d31fedc7e8f3cf35a36252da71218668fce`. Execution is verified at checkpoint 4; continue through checkpoint 5 only after the scoped Task 2 rereview and Task 3 gate begin.

```sh
python3 docs/superpowers/plans/2026-10-03-geosim-amended/replay.py --target . --through 4 --verify-only
python3 docs/superpowers/plans/2026-10-03-geosim-amended/replay.py --target . --through 5
# Continue through 6 only after its planned gate.
python3 docs/superpowers/plans/2026-10-03-geosim-amended/replay.py --target . --verify-only
python3 -m unittest discover -s docs/superpowers/plans/2026-10-03-geosim-amended -p test_replay.py -v
```

`replay.py` and `test_replay.py` are exact copies of the original tools. Stage 1 remains the original test-first discovery RED. Stage 2 retains the reviewed core/spec correction from `cd1de5f`; stages 3 and 4 now include the accessibility regression and implementation respectively. Stages 5 and 6 retain original patch bytes. `manifest.json` binds exact patch bytes, checkpoint inventories, preimages, postimages and executable modes. The prior explicit design-file classification and every other protected source/exemption remain unchanged. This exact packet directory remains preparation metadata.

`original-manifest.json` and `original-study-manifest.json` retain the original declarations. The scientific `study-manifest.json` remains byte-identical to `cd1de5f` after fresh regeneration. It is preparation evidence, not an execution binary receipt or registered result. Historical `declaration-differences.json`, `amendment-verification.json` and `evidence-preservation.json` record Task 1 evidence. Current `accessibility-verification.json` and `accessibility-evidence-preservation.json` record the fresh checks and retained logs/worktrees. No registered histories were run; combined Task 2 rereview remains pending.
