//! Earlier runs are unchanged: with disease off, the milestone-1 and
//! Chapter IV presets evolve exactly as they did before Chapter V was added.

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
    // Chapter V (disease on).
    ("v-1-rid", 0xc78e4bf09c07a55f),
    ("v-2-endemic", 0x8c1f553686437f6b),
    ("v-mcneill", 0xe3f72305eb78fd69),
    ("vi-1-everything", 0x54a84166f47ed9f1),
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

/// Prints `GOLDEN` entries: `cargo test -p sugarscape-core --test golden -- --ignored --nocapture`.
#[test]
#[ignore]
fn print_golden() {
    for p in presets::all() {
        println!("    (\"{}\", {:#x}),", p.id, fingerprint(p.id));
    }
}
