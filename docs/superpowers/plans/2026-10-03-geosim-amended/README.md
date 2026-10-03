# GeoSim amended premeasurement packet — 2026-10-03

This packet incorporates Task 1 correction `0b26aa867411526256b5bf8af9ee440eb9521efe`, Task 2 accessibility correction `9d53d4ef8be6809ad6374bad9993a02458af596e`, and Task 3 protocol correction `49bcfdc8ef6096ea8c18ad25383f7d0e0569e60c`. [PROTOCOL-AMENDMENT.md](PROTOCOL-AMENDMENT.md) records the current refresh. [AMENDMENT.md](AMENDMENT.md) and [ACCESSIBILITY-AMENDMENT.md](ACCESSIBILITY-AMENDMENT.md), together with their verification receipts, describe preserved historical revisions `cd1de5f` and `bd2b7ad`. Their prior hashes/checkpoints and claims of unchanged scientific declarations apply to those revisions. The original approved packet remains unchanged in `geosim-plan` at `65bfa9eae59622a824d7c6f09db93b68c83c1dbb`.

Use the same six ordered execution gates from source-only base `4ab07d31fedc7e8f3cf35a36252da71218668fce`. Current execution is verified at checkpoint 6. This refresh precedes registered measurement; scoped Task 3 correction/packet rereview and the actual binary/source freeze remain controller gates.

```sh
python3 docs/superpowers/plans/2026-10-03-geosim-amended/replay.py --target . --through 6 --verify-only
python3 -m unittest discover -s docs/superpowers/plans/2026-10-03-geosim-amended -p test_replay.py -v
# On a clean source-base checkout, apply all six ordered stages:
python3 /absolute/path/to/2026-10-03-geosim-amended/replay.py --target /absolute/path/to/clean-checkout
```

`replay.py` and `test_replay.py` remain exact original copies. Stages 1–4 remain byte-identical to `bd2b7ad`; stage 2's normative design note and every exact intermediate design preimage remain intact. Stage 5 places the corrected recorder/Python tests and compiled manifest fixture before the implementation, retaining all locked dependencies and scaffold support. Its actual `cargo test --locked ... --no-run` fails for missing production types/functions/main. Stage 6 completes the tested implementation and dated normative design/method corrections. `manifest.json` binds patches, checkpoint inventories, pre/postimages and Git modes. The design remains an explicit product file; all protected sources and security/exemption rules remain unchanged.

`original-manifest.json` and `original-study-manifest.json` retain the original declarations. Current `study-manifest.json` is freshly generated from the corrected scientific bytes and method descriptor, and independently byte-equal across expected and clean replay trees. It binds 325 unchanged required paths, of which 16 changed bytes: strict recorder evidence schemas, stable Pareto evaluation, descriptive diagnostics, optional resolved output, tests/fixture and dated scientific documentation. Method SHA256 is `9ff510fb666c3bdaf60c842dcd3db8f97e66232e54f40721c179bf4fc433f9dd`; source inventory SHA256 is `073e34974a496747eec5b6fd877011ea737bcd102571f0cf4163e1918adfcba1`. The 37 arms, 1,490 keys/seeds and 72 analysis jobs remain identical. This declaration is premeasurement preparation evidence, not a binary receipt or scientific result.

Current checks and source differences are in [protocol-verification.json](protocol-verification.json); all unique logs/scripts/receipts are retained and SHA-bound in [protocol-evidence-preservation.json](protocol-evidence-preservation.json). Prior packet/assets/evidence remain preserved. No registered histories, fixed analysis or benchmark ran during this refresh.
