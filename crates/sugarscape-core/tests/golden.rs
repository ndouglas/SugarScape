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
    ("iv-15-trade-sex", 0xda2f681086c8729b),
    ("iv-3-pollution", 0xa44cef03ce32f537),
    ("iv-18-foresight", 0x71bdcb5c44708373),
    ("iv-5-credit", 0xac5bbc30ed0fb306),
    // Chapter V (disease on). Re-recorded when `fingerprint()` started
    // hashing `immune_genome` too (previously only the trained `immune`
    // string was hashed).
    ("v-1-rid", 0x51a57db11c232edb),
    ("v-2-endemic", 0x899e70cd17f1484a),
    ("v-mcneill", 0x81def3a081577ad9),
    // Re-recorded when credit became per-good (N goods): vi-1-everything now
    // lends and borrows spice as well as sugar. No other entry changed.
    ("vi-1-everything", 0xd04f0968e3ac2c63),
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
    ("dock-mobility-15", 0x9d0a2ced876f00d2),
    ("dock-mobility-30", 0x10a0c00c27c1660d),
    // Minds 1: the ideal free distribution (two patches; the utility mind).
    ("ifd-even", 0xb5792be0a465638a),
    ("ifd-two-to-one", 0xbc6e9d14253d630d),
    ("ifd-four-to-one", 0x2eaf63d27955774d),
    ("ifd-far-sighted", 0x200e86235eb03f55),
    ("ifd-no-starving", 0x471d7b55040cfaa4),
    ("ifd-wander", 0x255ac0e4071cd48),
    // Re-recorded when ifd-crowding/ifd-travel moved to vision 10–20 (fix
    // round 1, task 6): at vision 1–6 no Flump sees both patches, so
    // crowding/travel could never move the patch split.
    ("ifd-crowding", 0x5a5e863436971c96),
    ("ifd-travel", 0x1a39e97e6ff051bb),
];

/// Other models (milestone 9): (preset id, fingerprint after 200 ticks from seed 1).
const MODEL_GOLDEN: &[(&str, u64)] = &[
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
    ("ax-dominance", 0x405accd101253f9d),
    ("ax-dominance-metanorms", 0x74717dff3c2ba3f1),
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
                || IMAGE_GOLDEN.iter().any(|&(id, _, _)| id == p.id),
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
