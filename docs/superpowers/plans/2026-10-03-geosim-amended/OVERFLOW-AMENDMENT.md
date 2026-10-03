# Representable generated-tail packet amendment — 2026-10-03 round 2

This sequential premeasurement refresh incorporates only correction `9dd36f78ad8ce49650659408b44f4784d0d3c3ec` relative to previous packet `ff8cbd39326f4b938fc715ab8ce3af421e25113a`. All original findings were addressed before this round; scoped rereview found a separate intermediate-exponential overflow. Earlier packet revisions, normative notes and evidence remain preserved unchanged in Git and the retained previous-packet snapshot. No registered periods or analysis preceded this correction. The full correction RED/GREEN and synthetic probe remain in the Task 3 fix-round-2 report; this refresh authors no product implementation.

Only stages 5 and 6 change. Stage 5 replaces `test_modern.py` and the compiled registered manifest template with their exact tested corrected bytes, retaining prior tests, support, Cargo dependencies/lockfile and missing-production scaffold. Its actual locked compile RED remains caused by missing production main/types/functions. Stage 6 completes corrected `modern.py`, `methods.py`, `NUMERICAL_METHODS.md` and normative design. The stage 2 note, exact intermediate design preimages, stages 1–4 patch bytes, replay tools/security, protected roster and metadata exemptions remain byte-identical.

Generation consumes the same exponential draws E after the same binomial/body-index draws. For E<=log(float64_max), retain xmin*exp(E), preserving near-cutoff arithmetic. Otherwise use exp(log(xmin)+E), avoiding intermediate exp(E) overflow when the final size is representable. Genuinely unrepresentable final values still fail the predetermined replicate without replacement. This is an arithmetic range branch with no scientific threshold/tolerance change and no RNG call in the helper. Only the method revision and tail-generation descriptor change; all remaining method fields, populations, source definitions, cutoffs, minimum support, alternatives, rules, p-families, draws, tolerances and verdicts remain exact.

| Binding | Previous SHA256 | Current SHA256 |
| --- | --- | --- |
| Replay manifest | `7e503ff22fab12159b178f2075e146b8eeb9ceac3189d55195a1ad391d6b24f3` | `31ff4a4dfb71e5a69f40023c71b77533b4cf3d81f4b83197168c84ea327fd17f` |
| `03-survey-tests.patch` | `916b7fb31715f83aaeac2e5dedb3599f6bb1042f5f57257378f704292e96c70e` | `a6aeeeda0ced1717604770e47f14b740063331bfaf23332af87bfd8dc3bac154` |
| `03-survey-implementation.patch` | `510c7f93c76277fd79d9c23b0b67e8609c86dbae81cd81b69411df4a0e287c39` | `5158aafe389cbab3d26d68bff762e303a4b749b0ad3f887dc012e89cb1d7e74f` |
| Scientific source inventory | `073e34974a496747eec5b6fd877011ea737bcd102571f0cf4163e1918adfcba1` | `5d7f78abd30f8dee86abb63f11f110b36a7119f554c0ae0d663a9aab7f579181` |
| Method contract | `9ff510fb666c3bdaf60c842dcd3db8f97e66232e54f40721c179bf4fc433f9dd` | `56ce49b6b77808c7cbfef0b9a182f6a573ab097925590cec78c8ed88567a10a0` |
| Premeasurement declaration | `1012a29246af4a50068a67cc19b3331bfc11de28bb5acadb74758a8573ca07fa` | `7e6baf8445230187f1d4169b114fecb9bd698b3569a3af79188230b41eb287d4` |

Final tree `553d48fc51b9ca11f92acbd2cc6a96c553909047`→`55868dde4ac2bc390dcafffe723112d82267be18`. Product counts remain 504/513/520/524/525/538/552 for checkpoints 0–6, plus 846 protected files, totaling 1,398 final files. The same 325 scientific paths have exactly six byte/hash changes:

| Scientific source | Previous SHA256 | Current SHA256 |
| --- | --- | --- |
| `docs/superpowers/specs/2026-10-03-geosim-design.md` | `97d69b8eb92e34342ad10565841a45af5ec9c7e33cb2f7adf671791caa54b0c6` | `49e3b73eaf6df870b238febccab3ddfb13021aab47a0c99326a8a303dd367ee5` |
| `survey/geosim/NUMERICAL_METHODS.md` | `4d46850335d2f262996a9756aed2dfb021e3acefc8bdbc7d636ca89fdb04ede5` | `247e51627f6a246ce870750640ab3128ddd433daaf8e7f19163de33805975a12` |
| `survey/geosim/methods.py` | `286f9beae3bb8153fd1594b1241e54cc0d65f38c1f657041654e085aaef12b06` | `2f35c276b5de8401fc9caa684812ec483c504d4cdf8dd69e726eb03d22bbb43b` |
| `survey/geosim/modern.py` | `421449b68d145a0e6450792196c8b5ab8564344226e6a50ac83caef672b4a775` | `2a14bca16c471af045470fad5ec22bcacf7117e14ca2f0d839f59319157eac8e` |
| `survey/geosim/test_modern.py` | `a1c79007c349dd5c6de90cb452c5b087ae424c5fcb9428ebbe56f7bac8d3d983` | `03291cfb4e242a5ec3790b053d346a9b28469ed196d6aaa1e7cd1bbcbdc1e072` |
| `survey/tests/fixtures/geosim-registered-manifest.json` | `b22fdc8b34f38957af1fae044b97bad3a69faf7057fec80288e9939770fe41f1` | `ea8a30237e06ef3f664a52e9b8f5a0c9d4cb71c66fb188a52e8173773cb24f3a` |

Only declaration fields method_contract, method_contract_json, method_contract_sha256, source_inventory and source_inventory_sha256 change. All 37 arms, 1,490 expected keys/seeds and 72 fixed jobs remain exact. Full byte counts and hashes are in [overflow-verification.json](overflow-verification.json).

Fresh verification passed 18 real-Git security tests, actual clean six-stage replay, meaningful stage 5 locked compile RED, independent complete Git-archive/expected/replay/execution byte-and-mode equality, execution checkpoint 6 verify-only and independent declaration generation from both final trees. Fresh inventories agree with all three roots; recursive method comparison permits only revision/tail-generation changes. Unique owned worktrees/logs/scripts/receipts remain retained and SHA-bound in [overflow-evidence-preservation.json](overflow-evidence-preservation.json). Original/prior packet and evidence manifests remain unchanged. No broad already-passed source suite, registered period, fixed analysis or benchmark ran; scoped overflow/packet rereview and actual binary freeze remain controller gates.
