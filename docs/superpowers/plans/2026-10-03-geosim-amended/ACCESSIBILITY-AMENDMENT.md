# Accessibility packet amendment — 2026-10-03

This sequential premeasurement refresh incorporates Task 2 correction `9d53d4ef8be6809ad6374bad9993a02458af596e` relative to `29b795321106378602a2808a9998189bd1ce5c48`. The correction associates shared schema controls with visible labels and gives paired controls distinct accessible names. Its existing regression and live-browser/web942/build evidence are retained in the Task 2 report; combined scoped rereview remains pending. No new UI, core or scientific implementation was authored here.

Only stage 3 (`02-hosts-web-sweeps-tests.patch`) and stage 4 (`02-hosts-web-sweeps-implementation.patch`) change. Stage 3 places the exact corrected `web/src/schema-form.test.ts` before the helper/renderer implementation. Stage 4 contains the exact corrected `web/src/schema-form.ts` and `web/src/ui/schema-panel.ts`. Their postimages persist through checkpoints 5 and 6. Stage 1, stage 2's reviewed core/spec correction, stages 5–6 patch bytes, replay tools, protected roster and every metadata exemption remain unchanged. Prior amended revision `cd1de5fb0f9392609bf64e02dd3b3243d01893fe` remains preserved in Git; original packet `65bfa9eae59622a824d7c6f09db93b68c83c1dbb` remains unchanged.

| Binding | Previous SHA256 | Current SHA256 |
| --- | --- | --- |
| Replay manifest | `ff98e449fa8b493604968f7887b8a2534ae16b1e613b676188d687d1dff53df2` | `2973290ae0954c07b14735840faf4b8490cde5be5af077e424b1755d556ba072` |
| Stage 3 test patch | `6f4ff3bcb26084d416a7b65058b44f7df4bde7bc19a458b8119ea5d3b7b41cc0` | `d3b9ded37e4a0bd0d1ef362e78c23a3f5b86ef1321f02422a66b397d745562a5` |
| Stage 4 implementation patch | `ed461d62d1684ce5076b131bddf98c50e74bae4fe7278ab69346ef954696681a` | `bc3c95e499da106103d45598c400b3634b454cf5877fd2a4c8cfffef3158b291` |

Expected final tree changes from `4fdd94aad9a2ab4db3aa85dc80e0672eafd35601` to `8878653c459202ef11cfabc39b853d32608d7693`. Inventory counts remain 504/513/520/524/525/538/552 product files at checkpoints 0–6 plus 846 protected files, totaling 1,398 final files. Execution stays at checkpoint 4 with unchanged product/spec/root-plan bytes and modes.

Fresh verification passed all 18 real-Git replay fixtures, actual clean six-stage replay, independent expected-versus-replayed final byte/mode equality and execution checkpoint 4 verification. Full declaration regeneration is byte-equal to the prior packet: declaration SHA256 `8b5043e2c2a1be68d161231861e295f3dfb61fa98a8871ac6ad645917ff418b5`, scientific inventory SHA256 `e84e46c4573c4e5ac217f1f57c45ce8f28174a902f24756bef494a27c252a04a`. All 325 scientific source entries, source roster, 37 arms, 1,490 keys/seeds, 72 jobs and methods remain unchanged; no scientific path includes the three web files. No engine/web/Rust suite or registered world was rerun for this refresh.

Current exact hashes, commands and unique owned replay-tree locations are recorded in [accessibility-verification.json](accessibility-verification.json) and [accessibility-evidence-preservation.json](accessibility-evidence-preservation.json). Prior evidence remains intact; no cleanup performed. Host rereview, recorder, execution freeze, study and final publication/deployment gates remain in force.
