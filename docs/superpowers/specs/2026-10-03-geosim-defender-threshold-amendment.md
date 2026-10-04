# GeoSim premeasurement defender-threshold correction

Date2026-10-03. This corrects a source-reading mistake before scientific measurement, under the approved design's clause permitting a cited source-defined correction before the measurement freeze. The original approved design at d6aeb4e had SHA256 `43896d0c7535e6675959ae330b7c29b6173b0f2b460aeb3fe6eb663d3bbc2f89`. The correction is isolated in scratch and accompanies the verified implementation plan for review.

## Evidence

Cederman2003 APSR printed148/PDF page15, Interaction, defines the attacking state's logistic victory function and explicitly sets the defending state's threshold to `1/victThresh=1/3`. The accompanying strategic explanation gives the attacker a requirement of approximately three times the defending strength. The earlier [mechanics notes](2026-10-03-geosim-mechanics-reading-notes.md) already recorded the reciprocal threshold; the approved design's same-positive-threshold wording contradicted that source and those notes. The recovered later GeoSim2 `Geosim2Model.java:561–575` also applies the reciprocal threshold to the defender's own advantage ratio.

## Before and after

Before: both attacker and defender evaluated their own projected advantage ratios using the same positive `victory_threshold`. At equal commitments and threshold3/exponent20, each had probability `1/(1+3^20)`, about2.868×10^-10. This removed the source's defensive advantage and could materially prolong fighting.

After: attacker uses thresholdh; defender uses threshold1/h on its own reciprocal advantage ratio. At equal commitments, attacker probability is `1/(1+h^k)` and defender probability is `1/(1+h^-k)`. At attacker advantageh both probabilities are.5. Defender probability decreases as attacker advantage increases. Independent draws and defender priority still allow simultaneous successes; no claim that the draws are mutually exclusive follows from complementary marginal probabilities.

The resolved field `defender_threshold=reciprocal` is the paper and artifact-bundle default. `same_threshold` preserves the previous reconstruction as a named unmeasured alternative. It is not one of the fourteen registered reading controls; adding it to the exposed configuration does not expand the registered workload.

## Frozen scope and verification

All37arms/1490keys, seeds, horizons, source statistic definitions,88-target and six-contrast families, modern estimators and diagnostic draws remain as specified. Every original/precision/control arm inherits the corrected literal default. No registered study, source comparison, fitted GeoSim output or scientific judgment preceded this correction. Synthetic tests must pin equal-strength and attacker-advantageh values, monotonic defender probability, named alternative serialization, and unchanged independent defender-priority resolution. The previous interpretation remains recoverable in this amendment and the original Git revision; no old scientific verdict is overwritten.

This correction removes reciprocal defender threshold from the list of unrepresented later-port differences. Remaining RNG, phase ordering, structural, shadow and plotting differences still prevent full artifact identity certification.
