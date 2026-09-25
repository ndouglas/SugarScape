//! Earlier runs are unchanged: with disease off, the milestone-1 and
//! Chapter IV presets evolve exactly as they did before Chapter V was added.

use sugarscape_core::model::ModelWorld;
use sugarscape_core::presets;
use sugarscape_core::world::World;

/// (preset id, fingerprint after 200 ticks from seed 1).
const GOLDEN: &[(&str, u64)] = &[
    ("ii-1-instant", 0x63d4454975b85fc),
    ("ii-2-unit", 0x75b93943813545e4),
    ("ii-5-wealth", 0x47bb9f4000e59377),
    ("ii-6-waves", 0xc16aa48d70070a46),
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
            GOLDEN.iter().chain(MODEL_GOLDEN).any(|&(id, _)| id == p.id),
            "record a golden fingerprint for {} (run print_golden)",
            p.id
        );
    }
}

/// Prints `GOLDEN` entries, then `MODEL_GOLDEN`'s:
/// `cargo test -p sugarscape-core --test golden -- --ignored --nocapture`.
#[test]
#[ignore]
fn print_golden() {
    for p in presets::all() {
        println!("    (\"{}\", {:#x}),", p.id, fingerprint(p.id));
    }
    println!("MODEL_GOLDEN:");
    for p in presets::catalog()
        .iter()
        .filter(|p| p.config.sugarscape().is_none())
    {
        println!("    (\"{}\", {:#x}),", p.id, model_fingerprint(p.id));
    }
}
