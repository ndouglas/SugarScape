# GeoSim premeasurement damage-incidence audit

Date2026-10-03. This exposes a source ambiguity already identified in the [mechanics notes](2026-10-03-geosim-mechanics-reading-notes.md), before the measurement freeze. It leaves the approved default mechanics and registered37-arm/1490-history workload unchanged. No registered result or fitted GeoSim output informed the choice.

## Conflicting source statements

Cederman2003 APSR TableA1, printed147/PDF14, describes `propDamage` as damage inflicted on the opponent. The resource recurrence on that page subtracts incoming `sum_j damage(j,i)`. Printed140 describes damage across parties and fronts without settling the incidence of unilateral D. Printed148/PDF15 instead says that an attacking party incurs the cost, and gives `.1*res(j,i)` for its own undiscounted locally allocated resources. These statements do not uniquely determine both the damage recipient and amount basis. The archived later GeoSim2 code damages the cooperating victim of unilateral D and both sides under mutual D; that supports a port reading but cannot erase the printed contradiction.

## Resolved named readings

`damage_incidence=attacked_party` retains the existing default: a side incurs damage when its opponent chooses D. `damage_incidence=acting_party` incurs damage when the side itself chooses D. Mutual D incurs damage on both sides in either reading; unilateral D distinguishes them.

`damage_basis=opponent_projected` retains the existing amount, fraction times the opposing projected commitment. `damage_basis=own_commitment` substitutes fraction times the damaged side's own undiscounted commitment. These fields are independent. Acting-party incidence combined with own-commitment basis implements the literal printed148 own-cost expression.

The artifact bundle remains attacked-party/opponent-projected. The registered own-commitment control still changes the basis alone. Acting-party incidence is exposed, serialized and validated but unmeasured in this bounded study. No extra registered arm, seed, Cartesian product or source judgment is added. Findings must identify it among unmeasured readings.

## Verification

Pin a unilateralD/C front with asymmetric raw and projected commitments: default damage goes to the cooperator, acting-party damage goes to the actor choosingD, and own-commitment basis uses the recipient's raw commitment. Pin mutualD/D separately. Configuration roundtrips and schema must retain inactive choices. Severity inclusion (`all_damaged_fronts` versus `mutual_only`) remains a separate field, so changing incidence cannot silently alter the selected census.
