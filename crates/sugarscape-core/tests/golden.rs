//! Earlier runs are unchanged: with disease off, the milestone-1 and
//! Chapter IV presets evolve exactly as they did before Chapter V was added.

use sugarscape_core::model::{ModelConfig, ModelKind, ModelWorld};
use sugarscape_core::presets;
use sugarscape_core::world::World;

/// (preset id, fingerprint after 200 ticks from seed 1).
const GOLDEN: &[(&str, u64)] = &[
    ("ii-1-instant", 0x63d4454975b85fc),
    ("ii-2-unit", 0x75b93943813545e4),
    ("ii-5-wealth", 0x47bb9f4000e59377),
    // Moved to the book's full 20×20 block by the model survey's follow-up.
    ("ii-6-waves", 0x4420b1eab9f8692f),
    ("ii-7-seasons", 0x89e264519aec44ff),
    // Chapter IV: now scheduled (pollution at t=50, diffusion at t=100).
    ("ii-8-pollution", 0xfdd983512b708286),
    ("iii-2-sex", 0x99c33cc7e8ad705c),
    ("iii-4-inheritance", 0x1aa293d107d6b7fb),
    ("iii-6-culture", 0xf8973190b0435a81),
    ("iii-9-combat", 0xe71fa903dfae5e58),
    ("iii-11-combat-fixed", 0x4c4958c1a3e160e0),
    ("iii-12-collision", 0xfec4ea6a61dd15fc),
    ("iii-14-combat-culture", 0xf9e3a9dd87cda8e),
    // Chapter IV, recorded before any Chapter V change.
    ("iv-1-spice", 0xd937df7102a3a4),
    ("iv-3-trade", 0x6f14b0be4cd3b21e),
    // 2026-09-27: the book's Figure IV-14 fertility ages (35–45 / 45–55).
    ("iv-15-trade-sex", 0x3cb6f69f8627bb0d),
    ("iv-3-pollution", 0xa44cef03ce32f537),
    ("iv-18-foresight", 0x71bdcb5c44708373),
    ("iv-5-credit", 0xac5bbc30ed0fb306),
    // Chapter V (disease on). Re-recorded when `fingerprint()` started
    // hashing `immune_genome` too (previously only the trained `immune`
    // string was hashed).
    // 2026-09-27: rule E's defaults follow the book (one immune flip per agent, note 16;
    // a learned disease passed on through the tick it is learned), changing the disease presets.
    ("v-1-rid", 0x2b95091f2990df9c),
    ("v-2-endemic", 0xa6cc74ede42df04f),
    ("v-mcneill", 0x6e40133658894b2d),
    // Re-recorded when credit became per-good (N goods): vi-1-everything now
    // lends and borrows spice as well as sugar. No other entry changed.
    ("vi-1-everything", 0x248c95796772a102),
    // N goods.
    ("n-3-trade", 0x2c2589dca0955ed7),
    ("n-4-peaks", 0x6e7eb172a4a8b78d),
    ("n-2-pollutants", 0xfbeefd604de77824),
    // Model extensions: the culture preset with the book's three groups.
    // Groups change no rule while combat is off, so it equals iii-6-culture.
    ("iii-6-three-tribes", 0xf8973190b0435a81),
    // Chapter VI: indecomposability (Chapter IV's traits, trade off / on).
    // VI-2's book crash is not reproduced; this entry pins its run.
    ("vi-2-no-trade", 0xd35c40bead68ed38),
    ("vi-3-trade", 0x2a65351834fda082),
    // Milestone 14: Axelrod's culture rule in the Sugarscape (the docking).
    ("dock-mobility-15", 0x3361a01b1a7cd6a3),
    ("dock-mobility-30", 0x19cfa4ca0800089e),
    // Minds 1: the ideal free distribution (two patches; the utility mind).
    ("ifd-even", 0xb5792be0a465638a),
    ("ifd-two-to-one", 0xbc6e9d14253d630d),
    ("ifd-four-to-one", 0x2eaf63d27955774d),
    ("ifd-far-sighted", 0x200e86235eb03f55),
    ("ifd-no-starving", 0x471d7b55040cfaa4),
    ("ifd-wander", 0x255ac0e4071cd48),
    // Re-recorded when ifd-crowding/ifd-travel moved to vision 10–20 (fix
    // round 1, task 6): at vision 1–6 no agent sees both patches, so
    // crowding/travel could never move the patch split.
    ("ifd-crowding", 0x5a5e863436971c96),
    ("ifd-travel", 0x1a39e97e6ff051bb),
    // Minds 2
    ("walk-capacity", 0x221010b49063dacf),
    ("walk-wealth", 0xa83586c20c3a101b),
    ("walk-seasons", 0x6962baedceba3118),
    ("walk-waves", 0x521bce5f41d93cbd),
    ("walk-fast", 0xe4a6fece9ed8eb7c),
    ("ifd-fence", 0x6c30002185fdb871),
    ("ifd-fence-far", 0x2a0ceead5af4938c),
    ("ifd-wall", 0xd079a77c3ac80869),
    // Minds 3
    ("mem-open", 0x817e0bb1cb4065b),
    ("mem-catchment", 0xbae607062bd8a385),
    ("mem-walled", 0xe5a8f45dd79a6779),
    ("mem-seasons", 0x7445335112f02389),
    ("mem-truffles", 0xf511426e092ca85b),
    ("mem-trapline", 0xae81840c5bbbe91),
    ("mem-mvt", 0xa00361ad0017c367),
    // Minds 4
    ("goap-mvt", 0x8bc669f7b7ddb21),
    ("mvt-rule", 0xc7322d80421cf559),
    ("goap-open", 0xb392facfc7a398e6),
    ("goap-truffles", 0xc41e161ff2270496),
    ("goap-walled", 0xce63e7e69fa6b37b),
    // Minds 5
    ("cache-winter-none", 0xcfa8d29ca799daab),
    ("cache-winter-even", 0x1d316fffdb074b5e),
    ("cache-winter-compensate", 0x40a9f0b62d0894cc),
    ("cache-winter-plan", 0x79ec96490bcb0eb),
    ("cache-winter-mixed", 0x118657e436a33b14),
    ("central-near", 0x8d4aae8003a9c344),
    ("central-far", 0x7a9e5541f980a69f),
    ("central-linear", 0x175e71e582dd82c5),
    ("cache-raby", 0xc30d92bb27b9366b),
    ("cache-amodio", 0xbc5371ef776e55a5),
    // Minds 6
    ("theft-winter", 0x8812daf7a717c8d0),
    ("theft-winter-quarter", 0x4f48f55df13c19e7),
    ("theft-winter-half", 0xd0ee29237c28952f),
    // 2026-09-30: the arenas walled on all four sides (a (k + 2)-torus).
    ("theft-arena-2", 0x403d0fd47b215e1d),
    ("theft-arena-4", 0x5aab49c65af8928d),
    ("theft-arena-8", 0x24f7fb0d39bfc02),
    // Minds 8, re-recorded for Minds 8b's default `raid_if: better`. The
    // first round's watch-winter (0xd0ecb91d218578c7) is reproduced under
    // `raid_if: always` (minds/caching/watching.rs).
    ("watch-winter", 0xa57ae7d89c3fb6d),
    ("watch-winter-stumble", 0xcf858c2b12679f0c),
    ("watch-half", 0x6adc8be6bd1d0fde),
    ("watch-scroungers", 0x14a1f4f2cfe8b7af),
    ("watch-scroungers-only", 0x5d5bfa38a76c9368),
    ("watch-scroungers-forgo", 0xb1abef5e1f2bfac1),
    ("watch-ak", 0x7d3e4e1903e7d55f),
    // Enabled spatial episodes include authoritative spatial configuration,
    // stores, intentions, guard state and recorded observations.
    ("spatial-scatter", 0xc7e8bffd577af990),
    ("spatial-larder", 0x1cf19eed5ef4bae4),
    ("spatial-larder-guard", 0x1293b223b1add84e),
    ("watch-arena", 0x35bf42deadfb8424),
];

/// Other models (milestone 9): (preset id, fingerprint after 200 ticks from seed 1).
const MODEL_GOLDEN: &[(&str, u64)] = &[
    ("s71-board", 0x8be271afb20afb6f),
    ("s71-center-out", 0x4d984d3877efd40e),
    ("s71-third", 0x1ad27c86391e5a56),
    ("s71-unequal-demands", 0x85e8185284286a17),
    ("s71-minority", 0xf38046ed144b9d70),
    ("s71-wide", 0xc055fcf62dfa79e3),
    ("s71-congregate", 0x87002c6991586729),
    ("s71-integrate", 0x585eb57399a48639),
    ("s71-line", 0xc24ebff812a7868a),
    ("s71-line-3", 0x40fcb924df508f4a),
    ("s71-line-minority", 0x26c48bba627b2b8c),
    ("s71-line-reach", 0x0db5a785349b5aab),
    ("tipping-fig18", 0x09f32cac3d61b2d9),
    ("tipping-fig19", 0x8183f2d74a5354c9),
    ("tipping-fig20", 0xea91f03009b2c727),
    ("tipping-fig21", 0x7720dc7b462269cd),
    ("tipping-fig22", 0xb58647a99df0e99b),
    ("tipping-intolerant", 0x6e204f0626e23a4d),
    ("tipping-minority", 0x3e9d9992d3a167c7),
    ("tipping-less-tolerant", 0x53b4b358243df7be),
    // Milestone 32: variations on Schelling (the small boards).
    ("pv-flat", 0x9ec9dad297a2f81d),
    ("pv-p50", 0x717f0ba10e75dfaf),
    ("pv-p100", 0xb903a558b48f4235),
    ("pv-spiked", 0xd9dd6c3a877d37c2),
    ("pv-ring", 0x14ac53042eb625d9),
    ("svw-small", 0x17c8cd55b1831276),
    ("vi-4-schelling-25", 0x7a7072c3433f5f6f),
    ("vi-5-schelling-25-residence", 0x9abe1c25e873debd),
    ("vi-6-schelling-50-residence", 0x637412f7af91f684),
    ("vi-7-schelling-mixed", 0x79346d2a338108cf),
    ("vi-8-ring-world", 0x1c341361c466db90),
    ("vi-9-ring-megagroup", 0x430d0c3b19b6e58e),
    ("lhv-published", 0x3b357e6f0cc5f74a),
    ("lhv-published-defaults", 0x7cdec8b85b1f909a),
    ("lhv-documented", 0x33ffb6476e824b0),
    ("cv-run-1-no-movement", 0x49637b8b34864721),
    ("cv-run-2-punctuated", 0x7888f03e0d511f6d),
    ("cv-run-3-salami", 0x51664e9ecc568140),
    ("cv-run-4-one-jump", 0xc8ad285446786559),
    ("cv-run-5-cop-reductions", 0x5713c0cafe4898dc),
    ("cv-run-6-coexistence", 0x1ce4fc6300e993ee),
    ("cv-run-7-cleansing", 0x11e4a982d405341),
    ("cv-run-8-nasty-regime", 0x5ce734d905ba0c5d),
    ("cv-safe-havens", 0x4259235e1ab53504),
    ("cv-netlogo", 0x87a92345c017b0ae),
    ("nm-1a-static", 0x5a72fde83bfb7283),
    ("nm-1b-chaos", 0x8360c871dcead14f),
    ("nm-2a-universal", 0x76ee3f7ba596e8ce),
    ("nm-3-kaleidoscope", 0xedd9930343121ef9),
    ("nm-no-self", 0x8feb7f382db6cbb5),
    ("nm-four-neighbors", 0xfd39a3d611f99c43),
    ("hg-async-kaleidoscope", 0xef13c172a34a980d),
    ("nbm-probabilistic", 0x79a048f606d28d74),
    ("nbm-discrete", 0x2befee9be89b26d9),
    ("nbm-continuous", 0x186717b420e5af31),
    ("nbm-random-array", 0xcf2c74041806d530),
    ("nbm-cube", 0x96baab61504e7923),
    ("rca-published", 0x1c83900b9b9f0b94),
    ("rca-literal", 0x8cc0a69cf4d14caa),
    ("rca-published-p2", 0x3f91b546c9734d3d),
    ("rca-literal-p2", 0x54380d2809b3fd62),
    ("rca-strict", 0x7b729c41a4436372),
    ("rs-no-forced-clones", 0x9dffef2174ca7415),
    ("eh-clones-only", 0x1bd933620c915c0e),
    ("eh-no-exact-clones", 0xafe94d6c0bb3b8ab),
    ("rca-adopt-p1", 0xc3f75d53d237a89),
    ("ac-sample-run", 0xeb302b62eb20f85d),
    ("ac-many-regions", 0xcfca4ee65fe3946c),
    ("ac-large-territory", 0xbe6a116733f7121),
    ("ac-torus", 0x8a21fc496bea71c3),
    ("ac-random-activation-20", 0xf0c8269aa3a3f7d2),
    ("ac-sweep-activation", 0x986f6c9a01898253),
    ("ac-neighbor-changes", 0xb12313a2dedfda7e),
    ("ac-soup", 0xe15b8cab349e25fa),
    ("ac-drift", 0xf254ab408f46810f),
    ("aey-equity", 0x6aa634df2b9c3944),
    ("aey-fractious", 0xd6b378e369c0066e),
    ("aey-transition", 0x38563aa17c61868d),
    ("aey-tags", 0x1455ea172db78c68),
    ("aey-classes", 0xe2a0783a5fc6734c),
    ("pvplh-small-tags", 0x3ca181d96a1c667),
    ("pvplh-mode", 0xb90f0d0cab7966b9),
    ("pvplh-progressive", 0xca60c420d61a7ffc),
    ("pvplh-lattice", 0x7975f5afc2d06c10),
    ("ha-standard", 0xf07433e56417f07c),
    ("ha-figure-1", 0x843632b62ddf7a6b),
    ("ha-appendix-mutation", 0xae8c7eda9113dae8),
    ("ha-appendix-double-play", 0xac2c2167fec9c326),
    ("ha-java-five-colors", 0xdc78c1e27b9ab453),
    ("ha-java-archive", 0xde2cff652c758fe7),
    ("ha-egoist-start", 0xabfdf5c9e1ccdb45),
    ("ha-cost-2", 0x41ba53998a8ee613),
    ("ha-cost-2-blind", 0x699aa05497139005),
    ("ha-misperception", 0x5567187174fd1c15),
    ("ha-each-color", 0x9ad570c3ea183419),
    ("jansson-offspring-anywhere", 0xcad22f8e7abafbfe),
    ("jansson-tag-mutation-30", 0xf9dbf338238a8f1b),
    ("jansson-kin", 0x265998639eacfbd0),
    ("jansson-kin-fixed", 0x3fac090571612879),
    ("hks-no-ethnocentrics", 0xbe867e7210bad2d2),
    ("hk-plurality", 0x587fb8e7c2480e66),
    ("hk-polarisation", 0x204ac33894adc63e),
    ("hk-consensus", 0x7d014897a7133c02),
    ("hk-regular-50", 0xbfd3a4ebe6714f04),
    ("hk-regular-plurality", 0x8336bae6d0f6a7e9),
    ("hk-regular-consensus", 0xa52300371b7c72af),
    ("hk-asym-a", 0x9cde60c378eb0e27),
    ("hk-asym-b", 0x5b3285bbf1750490),
    ("hk-asym-c", 0x67b217d709a3f7d3),
    ("hk-one-sided", 0x84fba5285f2ec536),
    ("hk-bias", 0x9a2ed969514cbda2),
    ("hk-serial", 0xb06db73333504889),
    ("hk-lattice", 0xe33359f120b204d9),
    ("cra-rwr", 0xc7f45f59d9b25490),
    ("cra-2dk", 0x3b8c19aab4aae805),
    ("cra-frne", 0xdf2fc96965742a81),
    ("cra-frn", 0x924d4b2fe686ae18),
    ("cra-ffr-01", 0x424eda2182150e01),
    ("cra-ffr-03", 0xbc09206184dc7003),
    ("cra-ffr-05", 0xf9573021f9848025),
    ("cra-random-start", 0x1871afd34df774a9),
    ("cra-copy-noise", 0xd8871da3505ee758),
    // Milestone 17: the demographic Prisoner's Dilemma.
    ("dpd-run-1", 0x3d64b053fbfee4f6),
    ("dpd-run-2", 0xe0198124ac5f4789),
    ("dpd-run-3", 0x1c819cc85b7bb351),
    ("dpd-run-4", 0xe6b5d66dee22ce07),
    ("dpd-run-5", 0x2f5ae2bdc6bd257a),
    ("dpd-working-paper", 0xaba834120f15810b),
    ("dpd-closest", 0xd1c7475297f9864c),
    ("dpd-soup", 0xdb64ad16a49146b),
    ("dpd-shifted", 0x39b59d546b2bb2e8),
    ("dpd-metabolism", 0x7b580a83419a3ea4),
    ("dpd-footnote-27", 0xd1adeadce6881068),
    ("dpd-rr-best", 0xe8fdc4ce027dd236),
    ("dpd-coordination", 0x47f8c68504a9157a),
    ("ax-norms", 0x52458d1993432cfe),
    ("ax-metanorms", 0x679a78d57f20640c),
    ("ax-dominance", 0x53ae18fb6b9c339a),
    ("ax-dominance-metanorms", 0x78d7e3e1840ba06d),
    ("gi-metanorms-long", 0xf80d7b08d057046f),
    ("gi-low-mutation", 0x2c37f146a39f0d51),
    ("gi-mild-metanorms", 0xb7435abcb67c192b),
    ("gi-temptation-10", 0xa191ff11a4f9ee68),
    ("gi-tournament", 0x95ea76458cee1a46),
    // Milestone 22: relative agreement (the two readings share their first 200 periods).
    ("dnaw-consensus", 0x1e241d02be7a49e5),
    ("dnaw-clusters", 0x247643148886db0b),
    ("dnaw-lattice", 0x60f9414e1157482d),
    ("dnaw-lattice-clusters", 0x459f1c2e1f67651b),
    ("ra-uniform", 0x9ed2107424379c37),
    ("ra-central", 0xcd795ff8a5bd9e44),
    ("ra-both", 0x6cc8db3561623a4c),
    ("ra-single", 0x769843f421a9f63e),
    ("ra-literal", 0x2a130dc7026094f7),
    ("ra-meadows-cliff", 0x4b9cf5817bc155d6),
    ("ra-deffuant-2013", 0x4b9cf5817bc155d6),
    ("ra-bc-extremists", 0xc304b87400a0bb46),
    ("ra-bc-printed", 0xef17ca06b9646fad),
    ("ad-moore", 0xd39d77107d873634),
    ("ad-small-world", 0xd29498f3ac055d5d),
    ("w-scale-free", 0xed9e58b01a7987d8),
    // Milestone 23: El Farol and the minority game.
    ("ef-arthur", 0x21d68cd385f4c107),
    ("ef-payoff", 0xfc1e2e95d6261a60),
    ("ef-random", 0xbf2299aeb9dfc6a),
    ("ef-shared", 0xaf9103c32858ba9c),
    ("mg-m6", 0x3aa1d7ced39f99c8),
    ("mg-m8", 0x9caf59baa2af5f1),
    ("mg-m10", 0xa1bf780e9b23275),
    ("mg-mixed", 0x6059c33de58936b),
    ("mg-inverse", 0xf9b094733c6a48aa),
    ("mg-evolution", 0xfdc5294338f8dc1d),
    ("mg-inbred", 0xc64cd71e7701ce15),
    ("mg-arms-race", 0x4c1e9852373241c9),
    ("mg-crowded", 0x5d20e99c75dfd073),
    ("mg-critical", 0x307ea6e032479849),
    ("mg-random-like", 0xbdb3ec814e99c525),
    ("cmo-binary", 0x2081105c24039d0c),
    // Milestone 24: ants and recruitment.
    ("ants-1a", 0x7b10d1c9e08f188e),
    ("ants-1b", 0x4a9423bcb26804a8),
    ("ants-1c", 0x8dcfeb59054ffc3d),
    ("ants-2a", 0x8dcfeb59054ffc3d),
    ("ants-2b", 0xf9258e5dd9d1d673),
    ("ants-crowd", 0x3332ad31a53efbe8),
    ("ants-becker", 0x4ebaae97020b8902),
    ("ants-lock", 0xbb3de1a046a835d4),
    ("ants-three", 0x4a2871368f4ab782),
    ("am-ring", 0x104a02f3c5f5011),
    ("am-random", 0x3e7b4009dc784788),
    ("am-scale-free", 0x9e1c3fd99aa8a637),
    ("am-independent", 0x23127da92f6506f),
    // Milestone 25: threshold models.
    ("gr-uniform", 0x85b7748fd9f64ab0),
    ("gr-perturbed", 0xbea93b91e6680639),
    ("gr-normal-12", 0xd933d5ae864ab70b),
    ("gr-normal-13", 0xcda701fbbdaa19ce),
    ("gr-normal-sampled", 0xa9bca3821a0f4774),
    ("gr-city", 0xa587dcb16521cf3c),
    ("gr-friends", 0x37ae8bba4d07be7c),
    ("gr-friends-perturbed", 0x6acc92415fc58d6e),
    ("gr-ceilings", 0x1cc528db96973a5a),
    ("gr-clusters", 0xd493a3251cde5fbe),
    ("watts-lower", 0xedb45df7d6e152de),
    ("watts-middle", 0x1ed3157ee5e9ac61),
    ("watts-upper", 0xea32457929c916d8),
    ("watts-hetero", 0x1ebbdf6d37320885),
    ("watts-hub", 0x994dd1ff0bedf61b),
    // Milestone 26: the timing of retirement.
    ("ae-rapid", 0x1d6b4cb389aa872f),
    ("ae-base", 0x2d5c384cbc8ebd2f),
    ("ae-slow", 0x63110dc295184e1b),
    ("ae-policy", 0x96c030d4a1280cd4),
    ("ae-groups", 0xadde267c611d5392),
    ("ae-all-members", 0x8f6694a3282613e3),
    ("ae-replace", 0x90d96b846be612f2),
    ("bg-base", 0x18a87a7bb2ab932d),
    ("bg-either", 0xa966f403050db40d),
    ("bg-none", 0x8082577a230b3670),
    ("bg-large", 0x7d2e95c561d9fdb0),
    ("bg-weak", 0x1ce6ad7a771157d0),
    ("bg-fixed", 0x58e1a7a64780e030),
    ("bg-mixing", 0x5d9e1aa7e6f5a06d),
    ("bg-benefit", 0xe5fec27869f67010),
    ("bg-continuous", 0xf763b0b75f9a2298),
    ("bg-ring", 0xc4e75af13ebdeaed),
    ("bg-janssen", 0xe95bb859afd7e9b0),
    ("gs-1", 0xe38d85243d149ae6),
    ("gs-2", 0xd40f4b475f30e6fe),
    ("gs-3", 0xd27a97984854b84e),
    ("gs-4", 0x3c72fdcb2726a9e9),
    ("gs-5", 0xdbd5cfd66c153ac1),
    ("gs-1-u", 0x298dee37f6ba4950),
    ("gs-4-u", 0xeaabdfdb9910b213),
    ("cliff-symmetric", 0xdad598c5704c7a9),
    ("cliff-flat", 0xce1e2689b16b59f1),
    ("cliff-excess-demand", 0xede597e74207bcda),
    ("cliff-excess-supply", 0xd4dbda6dac4d36bf),
    ("zip-symmetric", 0x1f0f2ad1aff93fe8),
    ("zip-flat", 0xbecfa69ef84f0057),
    ("zip-excess-demand", 0x21f769282f3c7ada),
    ("zip-excess-supply", 0x95340ce9f047cfd5),
    ("zip-demand-shift", 0x1f0f2ad1aff93fe8),
    ("zip-supply-shift", 0x1f0f2ad1aff93fe8),
    ("zip-retail", 0x843e23fc86d35493),
    // Milestone 29: Balinese Water Temples (200 months).
    ("lk-random", 0x7d84b477ca8fba95),
    ("lk-random-fixed", 0x9523e158c2e642a),
    ("lk-traditional", 0x8520f5f008de482f),
    ("lk-hyv", 0xcce2d26b2ee974a9),
    ("lk-perturbed", 0xcce2d26b2ee974a9),
    ("lk-stressed", 0x626e5d5529abfb7),
    ("lk-temples", 0xa92f00f760825795),
    ("janssen-code", 0xb423b71bbc6353d0),
    ("janssen-levels-14", 0xa1d94a968698c5ed),
    ("janssen-two-node", 0xf87c8032f908abcc),
    ("janssen-generalized", 0xa2ae0ad93a97f180),
    ("janssen-adaptive", 0xea4877048a644fad),
    ("janssen-fewer-links", 0x878f982a182d0672),
    // Minds 7: the evolution of larder hoarding (200 bouts: day 10 of generation 1).
    ("hoard-threshold", 0x83dbd8ba0dd130e0),
    ("hoard-scatter", 0xa077bede082fc87e),
    ("hoard-larder", 0x3da0a5fcbbe0803e),
    ("hoard-no-free-recovery", 0xaf76c42dbbd8c97c),
    ("hoard-cheaters", 0xd9fe57875f2c6602),
    // Milestone 33: The Emergence of Firms (200 periods).
    ("firms-base", 0xf289726485a9084c),
    ("firms-live", 0x8cb9364c43b4d2ad),
    ("firms-uniform", 0xa8f9007712e0714),
    ("firms-beta-17", 0x51a0c706a25d803a),
    ("firms-beta-21", 0x967e02c2d9f1f5b),
    ("firms-b-15", 0xb9b2b6650eb6c678),
    ("firms-b-random", 0xd82d8e6ac458f55b),
    ("firms-theta-075", 0xf13e84290fb10535),
    ("firms-friends-10", 0x61a9eadb71f6d389),
    ("firms-random-firms-10", 0xef5ffd52a915ef40),
    ("firms-loyal-10", 0x7f561840895caf83),
    ("firms-sticky", 0x4ce2e768347c9fb2),
    ("firms-groping", 0x317cb37372a9b66f),
    ("firms-seniority-5", 0x493639d792f2a77b),
    ("firms-base-pay-80", 0x6ed803ed3a45d863),
    ("firms-hiring-100", 0xd8fb44c3f2acfa13),
    ("firms-random-choices", 0xd5650a4bd4548db4),
    ("firms-2013", 0xc7acf4a34472ca50),
    // Algorithmic Collusion (200 ticks of 1 000 periods). The deviation presets learn as the
    // baseline does (they differ only in the analysis after convergence).
    ("collusion-calvano", 0xbfdc574972d2efcd),
    ("collusion-code", 0xa6ab4cce772a8a70),
    ("collusion-no-memory", 0x172d003af01b8b94),
    ("collusion-myopic", 0xf53b3963ec92f7f8),
    ("collusion-two-phase", 0xff7d1eb24dd21e2f),
    ("collusion-synchronous", 0xcbb6eed6a17dd495),
    ("collusion-explore-more", 0x90e7c74eb3f03d51),
    ("collusion-every-price", 0xbfdc574972d2efcd),
    ("collusion-invitation", 0xbfdc574972d2efcd),
    ("collusion-below-nash", 0x4b11e4afb4a24a8c),
];

fn fingerprint(id: &str) -> u64 {
    let preset = presets::by_id(id).unwrap_or_else(|| panic!("unknown preset {id}"));
    let mut world = World::new(preset.config, 1).unwrap();
    world.run(200);
    world.fingerprint()
}

#[test]
fn earlier_presets_are_unchanged() {
    for &(id, expected) in GOLDEN {
        assert_eq!(fingerprint(id), expected, "preset {id} changed");
    }
}

#[test]
fn every_preset_has_a_golden_entry() {
    for p in presets::all() {
        assert!(
            GOLDEN.iter().any(|&(id, _)| id == p.id),
            "record a golden fingerprint for {} (run print_golden)",
            p.id
        );
    }
}

/// Any model's preset `id` after 200 ticks from seed 1.
fn model_fingerprint(id: &str) -> u64 {
    let preset = presets::find(id).unwrap_or_else(|| panic!("unknown preset {id}"));
    let mut world = ModelWorld::new(preset.config, 1).unwrap();
    world.model_mut().run(200);
    world.model().fingerprint()
}

/// Big boards (50 × 50 and 100 × 100, milestone 32): (id, ticks, fingerprint
/// from seed 1), 20 ticks so a debug test run stays quick.
const BIG_GOLDEN: &[(&str, u32, u64)] = &[
    ("gvn-frozen", 20, 0xc12bbc1f827dec02),
    ("gvn-segregated", 20, 0xdee4748f1bfb310b),
    ("gvn-mixed", 20, 0x367617ebe919e13f),
    ("svw-large", 20, 0xab7f0a875a6994bb),
    ("svw-t4", 20, 0xbbfbe12164dd29c4),
    ("zhang-checkerboard", 20, 0xe8aee08d720ff908),
    ("zhang-random", 20, 0x61b6343a85676408),
];

#[test]
fn big_boards_are_unchanged() {
    for &(id, ticks, expected) in BIG_GOLDEN {
        let preset = presets::find(id).unwrap_or_else(|| panic!("unknown preset {id}"));
        let mut world = ModelWorld::new(preset.config, 1).unwrap();
        world.model_mut().run(ticks);
        assert_eq!(world.model().fingerprint(), expected, "preset {id} changed");
    }
}

/// Image scoring's presets: (id, generations, fingerprint from seed 1). The
/// island presets (100 groups of 100, 50,000 rounds a generation, and with
/// perception errors a record per pair of members) run 20 generations, the
/// one-group presets 200.
const IMAGE_GOLDEN: &[(&str, u32, u64)] = &[
    // Milestone 21: image scoring.
    ("ns-fig-1", 200, 0x98875bd71738cf05),
    ("ns-fig-2", 200, 0x91995405bae5fd9),
    ("ns-fig-3-n20", 200, 0xb4bd11bc229673c4),
    ("ns-fig-3-n50", 200, 0x5907ce5cb47602cf),
    ("ns-fig-3-n100", 200, 0xaf771beff4611d0d),
    ("ns-fig-4a", 200, 0xe7d6cefc18bd8d6a),
    ("ns-fig-4b", 200, 0xa4c19c5fa5fcfe3),
    ("ns-fig-4c", 200, 0x64755eafb526a816),
    ("ns-fig-4d", 200, 0x72ca0b1d80947f44),
    ("ns-own-only", 200, 0x20d6768b1fbd7081),
    ("ns-no-offset", 200, 0xc595349de0c0bf65),
    ("lh-fig-1a", 20, 0x46669de796924497),
    ("lh-fig-1b", 20, 0x2c3d0dea6af0c894),
    ("lh-fig-2a", 200, 0x56250416634c6ed9),
    ("lh-fig-2b", 20, 0x566a697764e9cdd9),
    ("lh-fig-2c", 20, 0x963ec9f09d2fadbe),
    ("lh-fig-3a", 20, 0x46059fbe6bab2056),
    ("lh-fig-3b", 20, 0x4d6575c59b0da89a),
    ("lh-fig-4a", 20, 0xd5654fa4b600eb57),
    ("lh-fig-4b", 20, 0x4bfe8ea9f3e8598b),
    ("lh-fig-4c", 20, 0x6371653b544f6387),
];

fn image_fingerprint(id: &str, ticks: u32) -> u64 {
    let preset = presets::find(id).unwrap_or_else(|| panic!("unknown preset {id}"));
    let mut world = ModelWorld::new(preset.config, 1).unwrap();
    world.model_mut().run(ticks);
    world.model().fingerprint()
}

#[test]
fn image_presets_are_unchanged() {
    for &(id, ticks, expected) in IMAGE_GOLDEN {
        assert_eq!(
            image_fingerprint(id, ticks),
            expected,
            "preset {id} changed"
        );
    }
}

#[test]
fn other_models_are_unchanged() {
    for &(id, expected) in MODEL_GOLDEN {
        assert_eq!(model_fingerprint(id), expected, "preset {id} changed");
    }
}

#[test]
fn every_model_preset_has_a_golden_entry() {
    for p in presets::catalog() {
        assert!(
            GOLDEN.iter().chain(MODEL_GOLDEN).any(|&(id, _)| id == p.id)
                || IMAGE_GOLDEN
                    .iter()
                    .chain(BIG_GOLDEN)
                    .any(|&(id, _, _)| id == p.id),
            "record a golden fingerprint for {} (run print_golden)",
            p.id
        );
    }
}

/// Prints `GOLDEN` entries, then `MODEL_GOLDEN`'s, then `IMAGE_GOLDEN`'s:
/// `cargo test -p sugarscape-core --test golden -- --ignored --nocapture`.
#[test]
#[ignore]
fn print_golden() {
    for p in presets::all() {
        println!("    (\"{}\", {:#x}),", p.id, fingerprint(p.id));
    }
    println!("MODEL_GOLDEN:");
    let image = |p: &presets::ModelPreset| p.config.kind() == ModelKind::Image;
    for p in presets::catalog()
        .iter()
        .filter(|p| p.config.sugarscape().is_none() && !image(p))
    {
        println!("    (\"{}\", {:#x}),", p.id, model_fingerprint(p.id));
    }
    println!("IMAGE_GOLDEN:");
    for p in presets::catalog().iter().filter(|p| image(p)) {
        let ModelConfig::Image(c) = &p.config else {
            unreachable!()
        };
        let ticks = if c.groups > 1 { 20 } else { 200 };
        println!(
            "    (\"{}\", {ticks}, {:#x}),",
            p.id,
            image_fingerprint(p.id, ticks)
        );
    }
}
