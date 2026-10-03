# GeoSim premeasurement numerical portability amendment

Date: 2026-10-03. This amendment selects a common numerical backend after actual shared-engine native/WASM parity failed before registered measurement. Source equations, resolved scientific parameters, named readings, registered arms/seeds and statistical families remain unchanged.

## Evidence and scope

The 4x4/four-founder source fixture agreed at construction but diverged after period1. The retained author-host evidence `survey/out/geosim-host-pow-probe.txt` isolates the same input `(sqrt(5)/2)^3`: native standard power bits `3ff65c55827df1d3`, WASM `3ff65c55827df1d2`. A shared software implementation is therefore justified by a reproduced platform difference, not a hypothetical portability concern. Original traces and probes are preserved.

The independent compact standard primitive probe compares identical native/WASM inputs:44/501 powers,10/102 logarithms and14/203 exponentials differ in bits; all101 sampled square roots agree. An exponential input reconstructed from the source first-period front, `-2.6187923807842535`, yields native bits `3fb2a8f944831157` versus WASM `3fb2a8f944831158`. GeoSim hashes exact float values, including last victory probabilities and damage; low-bit differences can violate reproducibility without immediately changing a sampled discrete event.

GeoSim's power, natural logarithm and exponential calls use **libm0.2.16 software routines** (`pow`, `log`, `exp`). Its geometry continues to use standard `f64::sqrt`, which has no observed mismatch in the probe. Other model kinds retain their existing math calls and golden values. This is the named and pinned implementation backend, not a new scientific treatment or exposed reading axis. Source/config/executable hashes and Cargo.lock bind the implementation for measurement provenance.

## Dependency policy and license

Primary package references: [versioned libm documentation](https://docs.rs/libm/0.2.16/libm/), [versioned source manifest](https://docs.rs/crate/libm/0.2.16/source/Cargo.toml.orig), [versioned build policy](https://docs.rs/crate/libm/0.2.16/source/configure.rs), [versioned license](https://docs.rs/crate/libm/0.2.16/source/LICENSE.txt), and [upstream repository](https://github.com/rust-lang/compiler-builtins). Cached source receipt: Cargo.lock package checksum `b6d2cec3eae94f9f509c767b45932f1ada8350c4bdb85af2fcab4a3c14807981`; inspected package path `/Users/nathan/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/libm-0.2.16`.

The core direct dependency is `libm = { version = "=0.2.16", default-features = false, features = ["force-soft-floats"] }`. The exact version is already cached and present transitively in Cargo.lock. The package declares MIT; its LICENSE.txt states the whole crate is usable under MIT, with original notices preserved by the dependency distribution. No dependency source is copied into this repository.

Inspection of libm0.2.16 Cargo.toml, configure.rs and `src/math/support/macros.rs` confirms default `arch` enables optional architecture implementations, while `force-soft-floats` disables `arch_enabled` and `intrinsics_enabled` even if other dependencies unify those positive features. Its `pow` is a common Rust implementation; software `log` and `exp` use the same crate implementation on the tested native/WASM targets. The negative feature is deliberate here to prevent optional architecture dispatch. This policy is evidenced for the tested toolchain/targets, not a universal proof for arbitrary future platforms.

## Verification and measurement freeze

The original direct native/WASM power assertion is RED. Source-function exact-bit fixtures cover the proven distance projection and a nontrivial contest probability. A separate pinned-software native/WASM probe compares all907 primitive cases with zero differing outputs before the engine replacement. The core change retains the same source formulas and range/invalidity handling. Final actual engine parity must cover both source and artifact fixtures across forced shocks, conquest/collapse and cluster completion. Only new GeoSim goldens may change; all previous model goldens must pass unchanged.

No registered37-arm/1490-history ensemble, full-source benchmark, fitted GeoSim result or scientific judge preceded this amendment. Archive identity remains Unresolved; using a common math backend does not certify the original author implementation. Final commands, bit fixtures and any additional evidence are retained in the scratch portability report.
