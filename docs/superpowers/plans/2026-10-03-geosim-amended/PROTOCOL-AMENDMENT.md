# Protocol packet amendment — 2026-10-03

This sequential premeasurement refresh incorporates tested Task 3 correction `49bcfdc8ef6096ea8c18ad25383f7d0e0569e60c` relative to `2949fa0b44625b35feaf44189addb18daded3028`. The prior amended packet `bd2b7ad42d62c8f7d726bc4f82463b8bbb3b39a9` remains preserved in Git and in the retained evidence snapshot. Original packet `65bfa9eae59622a824d7c6f09db93b68c83c1dbb`, Task 1 revision `cd1de5fb0f9392609bf64e02dd3b3243d01893fe` and their historical receipts remain intact. No registered periods or fixed analysis have run. The Task 3 report retains the full correction test evidence; this refresh authors no new product implementation.

Only stages 5 and 6 are regenerated. Stage 5 takes the exact five corrected Python test files, native test and compiled manifest fixture from the tested correction; all previous Cargo.toml/Cargo.lock and missing-production scaffold bytes remain intact. Actual locked compile RED fails because production main/types/functions are absent, without a dependency/lockfile failure. Stage 6 completes all exact reviewed production/docs changes. Stages 1–4, replay security behavior, protected roster, exemptions and exact intermediate normative design preimages are unchanged. In particular, stage 2 retains its prior normative note; the dated Task 3 design correction applies only at stage 6.

The corrections reject missing final newlines and malformed nested authoritative evidence, center Pareto ratios consistently across candidate alpha/KS/likelihood/generated tails, emit inferred descriptive source fits and overlapping diagnostic strata, and implement the advertised optional resolved-output default. Method descriptors and normative docs freeze those changes before measurement. Primary source metrics, masks, source-table bytes, hypothesis families, arm populations, seeds, draws, tolerances and analysis-job roster remain unchanged; secondary checks stay inferred/descriptive and source equivalence remains Unresolved.

| Binding | Previous SHA256 | Current SHA256 |
| --- | --- | --- |
| Replay manifest | `2973290ae0954c07b14735840faf4b8490cde5be5af077e424b1755d556ba072` | `7e503ff22fab12159b178f2075e146b8eeb9ceac3189d55195a1ad391d6b24f3` |
| `03-survey-tests.patch` | `d75fb981fdbc983b3b9355a11481c12fc97693c242a118433492477751cb4a4d` | `916b7fb31715f83aaeac2e5dedb3599f6bb1042f5f57257378f704292e96c70e` |
| `03-survey-implementation.patch` | `2af986b9892e5a3e51df06abc1948dff7c94c70a6f5d4d51feed1a97d0c07e39` | `510c7f93c76277fd79d9c23b0b67e8609c86dbae81cd81b69411df4a0e287c39` |
| Scientific source inventory | `e84e46c4573c4e5ac217f1f57c45ce8f28174a902f24756bef494a27c252a04a` | `073e34974a496747eec5b6fd877011ea737bcd102571f0cf4163e1918adfcba1` |
| Method contract | `23dfcea82e39a0a869aa60df5e6d2b22244e70a837a05ac3b22185e94f7630e7` | `9ff510fb666c3bdaf60c842dcd3db8f97e66232e54f40721c179bf4fc433f9dd` |
| Premeasurement declaration | `8b5043e2c2a1be68d161231861e295f3dfb61fa98a8871ac6ad645917ff418b5` | `1012a29246af4a50068a67cc19b3331bfc11de28bb5acadb74758a8573ca07fa` |

Expected final Git tree changes from `8878653c459202ef11cfabc39b853d32608d7693` to `553d48fc51b9ca11f92acbd2cc6a96c553909047`. Product counts remain 504/513/520/524/525/538/552 at checkpoints 0–6, with 846 protected files and 1,398 final files. The unchanged 325-path scientific roster has exactly these 16 byte/hash changes:

| Scientific source | Previous SHA256 | Current SHA256 |
| --- | --- | --- |
| `docs/superpowers/specs/2026-10-03-geosim-design.md` | `048a8726bec0367ad486fb7f26e4f4baf414099f342d48e22670d9df0725c539` | `97d69b8eb92e34342ad10565841a45af5ec9c7e33cb2f7adf671791caa54b0c6` |
| `survey/geosim/NUMERICAL_METHODS.md` | `6298dd25490433cae4cc46be92ceaa14c3197c4cffe77c04e7d63dbbfbe2d5ce` | `4d46850335d2f262996a9756aed2dfb021e3acefc8bdbc7d636ca89fdb04ede5` |
| `survey/geosim/README.md` | `eb480eaf2e36395ed1bc40aca66d6d0d92b66f3577d19af2ab168cda6ea14875` | `f267c6e08159db3503138701331a4fdbe771bd4c1dcff4d47acb30d8c078f44b` |
| `survey/geosim/methods.py` | `67fa0934d2f7f259d868244a78d3f1e2d53010a29f7b23b808c41515a530e20a` | `286f9beae3bb8153fd1594b1241e54cc0d65f38c1f657041654e085aaef12b06` |
| `survey/geosim/modern.py` | `7b091ba654b8dfc9ca6c86899c22d93a583418fceb456564116f66bf3a2ca232` | `421449b68d145a0e6450792196c8b5ab8564344226e6a50ac83caef672b4a775` |
| `survey/geosim/numerics.py` | `119c48562b1d760392876fcb8abf55d8dca7cbac18edb8b0aabdd7b02ec5bcd0` | `736b80ec8cf2c49014b18f3de47939d31ce37bd1fdfa21d1b86a59df08ff4304` |
| `survey/geosim/records.py` | `00113e57b389361a75e26a9c466c14796bccc9e59376dd5fe85381ff4c1da54c` | `1f029432e3cf063bbea90a6db2952f65bb1379126e726f16a1ef2b54b35edbf6` |
| `survey/geosim/source.py` | `8b5d57b6b42c6dd7ebdc1396207a752976a5cfae9dbaafde30b77b54c572b95e` | `4b599ad80ee9f933f68eb5cba4ad023c7b8fc1eb768b0dabcf248baac1537078` |
| `survey/geosim/test_analysis.py` | `65de67cdc195f9a14f82042b4434f8ee0262f25480bf82b0ff58d6e67085970b` | `3fbeafb5293699786d265f668ed99dfe0be7e6ed962f472563c7adad9a350916` |
| `survey/geosim/test_modern.py` | `c1dfbd452cf3ad1f7a60c44ca406416fed79f84d01462cb32ff783e0370da25c` | `a1c79007c349dd5c6de90cb452c5b087ae424c5fcb9428ebbe56f7bac8d3d983` |
| `survey/geosim/test_numerics.py` | `bf353f6d9e5aa0f840901539224b393608132d1439415f7961af593a38686a52` | `ea9c04f95fb8a6e6d412d30c4fd3d2ba5c6591b1799ae3e6cc1e5fd925c603dc` |
| `survey/geosim/test_records.py` | `866df0843859d9296fb65f27513c578e3b3d8f6c2b23c10c4c17d20215d2fe2a` | `537a314de542d217b80dcc3ab7dc5696d962c934213fadd9cdb67a580504f1b2` |
| `survey/geosim/test_source.py` | `ece3296e4cfb82b54fecac6f4613fc62a1edb2fcb20fb19da909ea02e65f2dec` | `7eb5dfc05ee3000407083bd780abbc1f89cae49438baf8710129327fae611b1e` |
| `survey/src/bin/geosim.rs` | `a1d111403a0e559ac24a70cb609d374002a154a00560cf9e5ae3143d08b66cb7` | `23b7daad2b66dfa6442ca0913e8c4343db20a904113a166d686131aea636da78` |
| `survey/tests/fixtures/geosim-registered-manifest.json` | `476d6a849f7a660baae626b77530f3dfb537485fa05c5a22c6455750b45790d3` | `b22fdc8b34f38957af1fae044b97bad3a69faf7057fec80288e9939770fe41f1` |
| `survey/tests/geosim_native.rs` | `b8229b0a7ff0fb04a48b1d48ede70ca559d90d23e536b89e69a69e47807fa865` | `4c448bafcd4edee018d82e93c88a315a2820699fc1504cae05d73ca2f333475c` |

The only changed declaration fields are method_contract, method_contract_json, method_contract_sha256, source_inventory and source_inventory_sha256. All 37 arms, 1,490 keys/seeds and 72 jobs remain exact. Full old/new source entry byte counts and hashes are retained in [protocol-verification.json](protocol-verification.json).

Fresh verification passed 18 real-Git replay-security tests, actual clean six-stage replay, meaningful stage 5 locked compile RED, independent complete expected Git archive/worktree/replay/execution byte-and-mode equality, current execution checkpoint 6 verify-only, and independent declaration generation from both final trees. Fresh source inventories also agree with current execution. All unique logs/scripts/receipts and worktrees are retained in [protocol-evidence-preservation.json](protocol-evidence-preservation.json); prior evidence is unchanged and no cleanup occurred. Scoped correction/packet rereview, actual binary freeze, registered study and publication/deployment gates remain in force.
