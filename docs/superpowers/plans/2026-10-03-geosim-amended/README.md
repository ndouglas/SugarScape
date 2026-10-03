# GeoSim amended premeasurement packet — 2026-10-03

This current packet incorporates Task 1 correction `0b26aa8`, Task 2 accessibility correction `9d53d4e`, Task 3 protocol correction `49bcfdc` and representable generated-tail correction `9dd36f78ad8ce49650659408b44f4784d0d3c3ec`. [OVERFLOW-AMENDMENT.md](OVERFLOW-AMENDMENT.md) records the current round 2 refresh. [AMENDMENT.md](AMENDMENT.md), [ACCESSIBILITY-AMENDMENT.md](ACCESSIBILITY-AMENDMENT.md) and [PROTOCOL-AMENDMENT.md](PROTOCOL-AMENDMENT.md), together with their exact verification/evidence receipts, describe historical revisions `cd1de5f`, `bd2b7ad` and `ff8cbd39326f4b938fc715ab8ce3af421e25113a`. Earlier hashes/checkpoints and statements about unchanged declarations apply to those preserved revisions. The original approved packet remains unchanged at `65bfa9eae59622a824d7c6f09db93b68c83c1dbb` in `geosim-plan`.

Use the same six ordered execution gates from source-only base `4ab07d31fedc7e8f3cf35a36252da71218668fce`. Current execution is verified at checkpoint 6. Scoped overflow correction/packet rereview and actual binary/source freeze remain controller gates before registered measurement.

```sh
python3 docs/superpowers/plans/2026-10-03-geosim-amended/replay.py --target . --through 6 --verify-only
python3 -m unittest discover -s docs/superpowers/plans/2026-10-03-geosim-amended -p test_replay.py -v
# On a clean source-base checkout, apply all six ordered stages:
python3 /absolute/path/to/2026-10-03-geosim-amended/replay.py --target /absolute/path/to/clean-checkout
```

`replay.py` and `test_replay.py` remain exact original copies. Stages 1–4 remain byte-identical, including stage 2's normative note and all intermediate design preimages. Stage 5 retains all corrected recorder/Python tests, locked Cargo dependencies and scaffold, and now includes the exact overflow regression and regenerated compiled manifest template. Actual locked compile RED fails for missing production main/types/functions. Stage 6 completes the exact tested overflow implementation, numerical descriptor and dated scientific notes, preserving earlier corrections. `manifest.json` binds every patch, checkpoint, pre/postimage and Git mode. Product/protected rosters and security/exemption rules remain exact.

Current `study-manifest.json` is freshly generated from the actual final scientific bytes and method descriptor and independently byte-equal across expected and clean replay trees. It binds the unchanged 325-path roster; six files have new byte hashes for tail generation, regression, method/template and normative notes. Current method SHA256 `56ce49b6b77808c7cbfef0b9a182f6a573ab097925590cec78c8ed88567a10a0`, source inventory SHA256 `5d7f78abd30f8dee86abb63f11f110b36a7119f554c0ae0d663a9aab7f579181`, declaration SHA256 `7e6baf8445230187f1d4169b114fecb9bd698b3569a3af79188230b41eb287d4`. Only method revision/tail-generation evaluation changes; the 37 arms, 1,490 keys/seeds, 72 jobs, populations, rules, draws and tolerances remain exact. This is premeasurement preparation evidence, not an execution binary receipt or scientific result.

Original declarations and all historical packet/evidence assets remain preserved unchanged. Current checks/differences are in [overflow-verification.json](overflow-verification.json), with unique retained logs/scripts/receipts SHA-bound in [overflow-evidence-preservation.json](overflow-evidence-preservation.json). No registered periods, fixed registered analysis or benchmark ran during this refresh.
