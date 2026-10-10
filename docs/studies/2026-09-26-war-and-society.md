# War inside a society: a study program

**Date:** 2026-09-26
**Status:** first focused literature pass completed on 2026-10-09; study design remains
proposed. No new simulator or measurements are claimed. See the
[first literature pass](2026-10-09-war-literature-pass.md).
**Separate from:** the reproductions (`docs/papers.md`) and Flump Studio (`studio/`). Those
reproduce and explain published models. These are our own experiments, built on a model we have
reproduced.

## Principle

These studies measure the costs of war and ask what ends violence. Every study reports who died, how,
where and what stopped it, alongside who won.

The field has a pull toward strategy as a puzzle: arrows on maps, brilliant maneuvers, campaigns told
like chess. That pull turns deaths into moves. The questions that started this program came from the
ground instead: how many families were murdered, how many houses were burned, how long a pursuit
lasts, and when and where the violence stops. These are the program's priorities. The
[first literature pass](2026-10-09-war-literature-pass.md) finds existing civilian, demographic
and termination models; a comparative claim about how much modeling each topic receives
is not established.
Lewis Fry Richardson, who founded the quantitative study of war, was a Quaker ambulance driver who
counted deadly quarrels to help prevent them. That is the tradition this program belongs to.

## Method

- **Target attested regularities.** Each ingredient must reproduce a named regularity, or a
  substantially attested behavior, from the combat-studies literature, with its source cited and its
  numbers where there are numbers. A mechanic without a target is a knob, not a finding.
- **Ablate.** Switch each ingredient off in turn to find which ones are *necessary* for which
  regularities. Where the chain from individual to collective breaks is itself a finding.
- **Keep the book literal.** Sugarscape's rule C stays as it is and remains the baseline (rung 0),
  with its golden tests unchanged. The studies are a new model kind built on the sugarscape world, and
  every departure is a named switch.
- **Run both scales.** Each result runs dimensionless (ratios and shapes, as in Sugarscape) and
  calibrated to real units (cells as distances, ticks as hours). Questions about rates and counts
  (how fast, how many) need the calibrated runs. Comparing the two shows which conclusions depend on
  scale.
- **Stay honest in presentation.** Results are labeled as our experiments, never the book's.
  Historical parallels are questions a result raises, not claims it proves.

## Where this starts

From the War spike (`docs/superpowers/specs/2026-09-26-markets-and-war-spikes.md`), 20 seeds:

- **III-9 reproduces** the book's "stunning blitzkrieg" and its "increasing returns … the bigger you
  are the faster you grow". It stays quiet for hundreds of ticks, then one Flump does about 70 % of
  the killing, and one tribe ends ≥ 90 % in 14 of 20 runs by tick 2000. The winner ends about 20
  times richer.
- **Rule C is a deterrence rule.** A Flump may attack only a poorer enemy. The retaliation
  filter uses the attacker's sight range to check for richer enemies after movement. Equal
  individuals cannot attack each other under C; equal group totals can contain unequal pairs.
  The spike's rapid conquest is an observation under its conditions, not a general duration law.
- **III-11's "coherent battle fronts … a prolonged war of attrition" don't reproduce** under the
  book's stated random replacement. **III-14's conquest and conversion** become a civil war under the
  stated random tags.

The first open question follows: what would it take for **equals** to fight a **long** war?

## Step one: the literature pass

The [first focused pass](2026-10-09-war-literature-pass.md) is complete. It covers reciprocal
engagement, geometry, duration/termination and nearby prior work; it does not validate the
whole ladder. It recommends W1 reciprocal finite engagement before W2 morale/rout and W3
supply. The proposals below remain candidates unless that pass explicitly verifies them.

The original candidate regularities below were cited from memory and marked *(check)* where
a number or attribution needs verifying. Further focused passes must turn the remaining
candidates into a **regularities table** with these columns:

| Column | Content |
|---|---|
| Behavior | The attested regularity, stated as something a model could show |
| Numbers | Values and ranges, with their conditions (period, army, terrain) |
| Source | The citation, and how good the evidence is (data, case study, contested) |
| Ingredient | The rung expected to produce it |
| Measure | How the model would count it |

The same pass checks **prior work**, so we don't claim to be first at something already done:
- military operations research: Lanchester models and combat simulations such as JANUS;
- agent-based combat: Ilachinski's ISAAC/EINSTein at CNA, and New Zealand's MANA;
- Cederman's geopolitics models, and Epstein's civil violence (our milestone 11);
- agent-based models of civil-war violence after Kalyvas.

## The ladder

Each rung adds ingredients on top of the ones below it, and each has its targets.

### 0. The book's rule C

The baseline: exactly Sugarscape's combat, with its blitzkrieg and its deterrence. Everything above
must reduce to it when every switch is off.

### 1. The engagement procedure

- **Ingredients:** an individual combat procedure borrowed from role-playing games (to-hit against
  defense, damage against hit points, initiative) and a battle rule for when perceived and true
  strength differ.
- **The battle rule:** deterministic (the truly stronger side wins) or probabilistic (the attacker
  wins with probability s_a / (s_a + s_d)). Either way, a losing attacker dies and the defender takes
  the loot. Lean deterministic first: it's the smaller departure.
- **Targets:**
  - Lanchester's linear law (one-on-one melee) against his square law (everyone can hit anyone), from
    the same procedure with different engagement geometry.
  - Wargames' combat results tables, which map force ratios to outcomes (attacker eliminated,
    exchange, defender retreats), as the bridge from individual fights to battle outcomes.
- **Hypothesis:** hit-point granularity controls how long a fight lasts and how random it is, but
  not who wins.

### 2. Morale, cohesion and contagion

- **Ingredients:** a breaking point for each Flump that depends on its neighbors; routs that spread;
  pursuit.
- **Targets:**
  - Battles are decided by one side breaking, not by attrition to the last soldier (Ardant du Picq,
    *Battle Studies*; Keegan, *The Face of Battle*).
  - The loser's losses concentrate in the rout and pursuit; get the ratios from casualty data:
    Clodfelter, Dupuy, and Bodart for the pre-1800 battles *(check)*.
  - Soldiers fight for their small group (Shils and Janowitz, 1948).
  - Panic spreads as a threshold cascade (Granovetter, queue item 5).

### 3. The body in combat

- **Ingredients:** an arousal level for each Flump, pushed up by events (an enemy in sight, an ally
  killed, being attacked, a win) and decaying with a fatigue debt.
- **Targets:**
  - Performance rises with arousal and then falls (Yerkes–Dodson; its generality is contested).
    Heart-rate "zones" (Grossman, Siddle) are thinly evidenced: use them as a knob, not a target.
  - Tunnel vision under high arousal, mapped onto the Flump's vision; freezing; pain suppression
    (stress-induced analgesia).
  - **Forward panic** (Randall Collins, *Violence: A Micro-sociological Theory*, 2008). Most people
    are poor at face-to-face violence. Built-up tension releases when the enemy suddenly looks weak,
    into overkill, the slaughter of the fleeing and atrocities. A small, competent minority does most
    of the violence.
  - The crash after the surge ends pursuits.
  - Nearly every soldier breaks down after a few months of continuous combat (Swank and Marchand,
    1946) *(check the figures)*.
  - The winner effect: winning raises aggression, a biological counterpart of rule C's increasing
    returns (well established in animals; mixed evidence in humans).

### 4. Logistics and communication

- **Ingredients:** supply trains of carrier Flumps with a metabolism and a carrying limit; foraging;
  raids on supply; messengers that move at a finite speed; commanders who know only what their
  messengers bring.
- **Targets:**
  - Carriers eat what they carry, which hard-limits how far an army can operate from supply (Engels,
    *Alexander the Great and the Logistics of the Macedonian Army*, 1978).
  - Before railways, an army lived off the land, so a region's forage capped the army it could hold,
    and armies had to keep moving; sieges starved the besiegers too (van Creveld, *Supplying War*).
  - The culminating point of the attack (Clausewitz).
  - The support share of armies (tooth-to-tail) rises with operating distance.
  - Fodder was the largest single commodity Britain shipped to the Western Front *(check)*.
  - Concentrating force pays under Lanchester's square law, but it needs coordination. The value of
    communication is measured in how much concentration it buys.
  - Mission command (*Auftragstaktik*) beats central orders once messages are slow enough: find the
    delay where it starts winning.
- **Worked example: Burma.** In 1942, lightly equipped Japanese troops outflanked road-bound British
  columns. In 1944 at Imphal, a plan that counted on captured supplies and cattle driven along as food
  ended in starvation and disease in the retreat *(check)*. Across the Pacific, starvation and disease
  may have killed more Japanese soldiers than combat did; Fujiwara Akira's estimate of about 60 % is
  the one usually cited *(check)*. Foraging fails where the land can't carry an army. Traveling light
  is an advantage until the army outruns what it can carry.
- **Question:** does a stable front form where the two sides' supply reaches meet? That would give
  III-11's fronts a mechanism the book didn't state.

### 5. Terrain and home ground

- **Targets:**
  - The defender's advantage and the effects of posture and terrain (Dupuy). The 3:1 rule is
    disputed.
  - Interior lines: a defender's shorter supply lines let it shift forces faster (Jomini).
  - Disease immunity on home ground: Leclerc's army in Haiti (1802), destroyed largely by yellow
    fever; West Africa as "the white man's grave". This is McNeill's *Plagues and Peoples* turned
    around, and Sugarscape's immune-system rule can already do it.
  - Diet: a tribe native to the spice hill lives on spice while invaders must carry sugar (the two
    goods of Chapter IV).
  - Climate: Finnish ski troops in the Winter War, and Russian winters (a seasons penalty that falls
    harder on outsiders).
  - Civilians give information to the side that controls them (Kalyvas).

### 6. Perception and information

- **Ingredients:**
  - Each tribe's estimate of the other's might: too high, about right or too low (a 3 × 3 grid), as
    bias. Overrating yourself equals underrating the other, so one ratio per tribe covers both.
  - Noise in each estimate, as the second knob of "reliability".
  - Actual military efficiency per tribe (true strength = efficiency × sugar), which with perception
    gives 81 cells × 20 seeds.
  - Stale information from slow messengers (rung 4), and deception.
- **Targets:**
  - Wars start when both sides are optimistic (Blainey, *The Causes of War*): the most war when both
    underrate each other, and peace or a short war when both see clearly.
  - Private information about capabilities causes war (Fearon, "Rationalist Explanations for War",
    1995): the noise version.
  - Mutual overrating gives a frozen, deterrent peace.
  - Lopsided misjudgment gives suicidal attacks.
  - What deception is worth: feints and decoys, as in Operation Fortitude.

### 7. Civilians

- **Ingredients:** noncombatant Flumps; families (the sex rule); homes (sites with stored sugar);
  burning (destroying stores and regrowth); flight.
- **Targets:**
  - Violence against civilians peaks where control is contested but leaning to one side, and falls
    where control is complete or absent (Kalyvas, *The Logic of Violence in Civil War*, 2006). This
    is a spatial prediction a grid can test.
  - In non-state warfare, raids and burned settlements kill more than battles do (Keeley, *War
    Before Civilization*).
  - Indirect deaths can arise from famine, displacement and disease. Their share is
    conflict-dependent; the [first pass](2026-10-09-war-literature-pass.md) endorses no universal
    majority. Sugarscape already models carrying capacity and migration.
  - Burning is also an attack on supply, which links this rung to rung 4.

### 8. Social threads

- **Disease in camps.** Before the twentieth century, disease killed more soldiers than battle (the
  US Civil War, roughly two to one *(check)*). Crowded armies plus Chapter V's disease rule.
- **Feuds and revenge.** A killing gives the victim's kin a grievance, and grievance passes down a
  family line as fortunes do. Blood feud (Boehm), and the conflict trap, in which civil wars recur
  (Collier).
- **Truces.** "Live and let live" in the WWI trenches (Axelrod, *The Evolution of Cooperation*):
  tacit cooperation growing out of stalemate. Our spatial games and tags do half the work.
- **Surrender.** Killing prisoners made enemies fight harder (Ferguson). When does surrender become a
  norm, and what does breaking it cost?
- **Who fights and who dies.**
  - The share of young men in a population predicts collective violence ("youth bulge": Mesquida and
    Wiener; Urdal).
  - New soldiers and pilots die at far higher rates, and survivors improve sharply.
  - In WWI, British junior officers died at higher rates than their men *(check)*, so the model
    should include leaders exposed at the front and command falling apart when they die.
- **Greed or grievance.** Lootable resources predict civil war better than grievance does (Collier
  and Hoeffler, 2004; contested, and the dispute is itself worth testing).
- **Cultures of honor.** Herding economies, where one raid can steal your wealth, breed
  reputation-based retaliation; farming economies don't (Nisbett and Cohen). Mobile, raidable sugar
  against fixed land.
- **Distance and killing.** Resistance to killing falls with distance (Grossman; thin evidence, so a
  switch, not a target).

### 9. From war to the state

- **Roving and stationary bandits** (Olson, 1993). A raider with a long horizon taxes instead of
  plundering, because victims' future output is worth more than today's loot. "War made the state,
  and the state made war" (Tilly). The engine already grows warlords, and taxing is one rule. This may
  be the strongest single idea here: we don't know of an agent-based model that grows a state out of
  banditry *(check)*.
- **Unpaid armies mutiny or pillage.** Parker's *The Army of Flanders and the Spanish Road*
  documents dozens of mutinies over pay arrears *(check the count)*.
- **Occupation.** Winning isn't holding. Quinlivan (RAND, 1995) gives about 20 troops per 1,000
  inhabitants for stabilization *(check)*. Below some ratio, occupation turns into insurgency, and
  milestone 11's civil violence is half of that model.
- **War weariness.** Public support falls with the logarithm of casualties (Mueller).
- **Adaptation.** Measure and countermeasure, and generals fighting the last war.
- **The size of wars.** Richardson's power-law claim for deadly quarrels and Cederman's
  GeoSim claim, measured separately in Milestone 37 below. The polarity
  reconstruction does not establish a war-size fit.

## The headline questions

1. What makes **equals** fight a **long** war, which the book's stated rules can't: misjudgment,
   logistics, or both?
2. Do the rout, the slaughter of the fleeing and atrocities against civilians emerge from arousal
   alone, with no evil agents and no orders?
3. Does a front form where supply reaches meet?
4. When does violence stop? Consider nightfall and terrain, exhaustion, looting, the pursuers'
   supply, and complete control, and find which of these matter and in what order.
5. Does a state grow out of a warlord?
6. From physiology to small units, battles, campaigns, wars and states: can one society carry the
   whole chain, each scale checked against an attested regularity? Where does it break?

The integration this program aims to investigate is a shared economic and demographic base
with explicit accounting of combat and its consequences. The
[first pass](2026-10-09-war-literature-pass.md) identifies prior models coupling battles,
resources, civilian decisions, displacement, demography, identity and authority. A claim to
be first, or an end-to-end gap claim, remains unestablished. The specific proposed
contribution is controlled ablation on a reproduced Sugarscape base.

## Measured polarity handoff (2026-10-03)

Milestone 36's [complete polarity findings](../superpowers/specs/2026-10-02-emergent-polarity-findings.md)
retain all 28,520 registered sessions across 572 arms: 28,281 valid and 239
invalid. These reconstruct EPM/provincial territorial conflicts, not GeoSim's
technology/war-size model or observed deadly quarrels. Source compatibility,
original/precision populations, invalidity and source uncertainty remain separate.

Across valid outcomes, 3,583,155 episodes include
81,753 domestic episodes and
16,116 censored episodes. Mean active-period
duration is 6.54904; uncensored median is
1. The
[descriptive summary](../superpowers/specs/2026-10-02-emergent-polarity-descriptive-summary.json)
retains family-specific counts, end causes, durations, resource accounting and
invalid partial-episode counts. Each episode carries start/end clocks, capitals,
initial sizes, path, positive losses, signed creation, winner and censoring. Its
duration includes the first conflict period; a cooperative ending timestamp does
not add an active conflict period. Open episodes at stopping are censored. Attacks,
DD encounters, conquests and episodes are distinct measures.

The separate original 640-session population records 1.18711×10^9 positive
destruction and 1.88860×10^9 signed creation units; the 6,400-session precision
population records 4.38912×10^12 and 3.50614×10^12, respectively. The Störmer
adaptation has a much larger finite tail (2.22173×10^191 signed creation units),
so pooling resource magnitudes would conceal protocol/parameter differences.
Positive resource destruction and signed resource creation are abstract units;
**resources are not casualties**. EPM stock observations count sovereign capitals,
provincial observations all primitive cells, with stock/session denominators;
terminal frequency is distinct from unmeasured period exposure. Invalid end-state
measurements describe the attempted clock and do not complete an aborted period.
The original 237 sequential implementation panics and all original verdicts remain
retained; a full 16×20 same-seed correction supplies final sequential data while
28,200 unaffected raw records remain byte-identical in that retained pre-domestic
dataset. The later [domestic source-fidelity amendment](../superpowers/specs/2026-10-03-emergent-polarity-domestic-amendment.md)
reruns all 14 provincial arms / 1,180 original seeds: 1,160 complete deterministic
records match, while all 20 overextension Outcomes change. All 27,340 nonprovincial
raw lines remain byte-identical. The actual source overextension illustration uses
a 10×10 grid (100 primitive units) with unknown seed; it supplies no trajectory
fit target. Updated overextension observations retain 92,623 voluntary revolt
actions across 81,739 domestic episodes, with positive domestic combat damage.
Actions, DD encounters and episodes remain distinct counts. No replacement seeds,
new resource cap or post-measurement judge changes were used.

## Measured GeoSim handoff (2026-10-03)

Milestone 37's [findings](../superpowers/specs/2026-10-03-geosim-findings.md) retain
all 37 arms / 1,490 attempts: 1,486 complete and 4 invalid. Ten conditional source-arm
comparisons are Unresolved; the 75×75 grid is Incompatible. Only 8 of 88 source
targets have complete predictive inference; 80 retain unavailable slots. Exact
source equivalence remains Unresolved because severity scale, range/count
conventions and original executable identity are unverified.

All six fixed baseline-minus-no-shock and baseline-minus-context-off slope,
R² and range contrasts are Unresolved: their joint source-fit populations are
incomplete. The two separately declared modern descriptive contrasts are
available. Baseline-minus-no-shock mean alpha is−0.868 (95% interval−0.940 to−0.799)
and baseline-minus-context-off−0.980 (−1.029 to−0.935), consistent with a heavier
adaptive fitted baseline tail within this reconstruction. Cutoff changes differ:
mean xmin+1.322 (+0.967 to+1.724) against no shocks, but−1.521 (−2.003 to−1.026)
against no context. These whole-history descriptive comparisons add no mechanism
verdict or historical causal claim.

The original pooled baseline is not rejected by its iid KS diagnostic(p≈0.266);
the independent precision pool rejects(p≈0.002). Across 22 pools there are 15
rejections,5 nonrejections and 2 inconclusive results, with 1,000 successful
refitted draws per pool. There are 1,478 individual fits,8 insufficient tails and
4 invalid histories;36 of 37 whole-history parameter summaries are available.
Pooled iid fits/tests do not validate dependent histories. The declared nested
cutoff-Pareto half-chi-square calibration remains unvalidated. Keep original,
precision and alternative-reading populations distinct.

No regime-type or democratic-peace experiment ran. CoWv4 participant data was
recovered, but Thailand's war 170 death code −9 is unknown; the exact Clauset 2018
SupplementS1 and Cederman 1820–1997 input remain unavailable. Known-sum endpoints
cannot substitute for an empirical reproduction. GeoSim0 GPL-2.0-or-later differs from the
framework's LGPL-2.1-or-later and unresolved nested GeoSim2 model license; all recovered
code remains reference-only. The [author audit](../superpowers/specs/2026-10-03-geosim-author-code-reading-notes.md)
and [artifact audit](../superpowers/specs/2026-10-03-geosim-artifact-audit.md) retain
these limits.

The [execution provenance](../superpowers/specs/2026-10-03-geosim-provenance.json)
and [postmeasurement recorder amendment](../superpowers/specs/2026-10-03-geosim-postmeasurement-recorder-amendment.md)
keep the measured source/binary and original raw/full/compact results separate
from repaired and integrated builds. Carry the explicit front/territory/cluster
and completed/censored/queued observables into future studies. Abstract resource
damage does not measure casualties, AI intent or a real-world power law.

## Open questions

- **The battle rule:** deterministic or probabilistic (rung 1). Lean deterministic first.
- **Units:** what distances and durations a cell and a tick stand for in the calibrated runs, and
  what march and pursuit rates to calibrate against.
- **Time within a tick:** Sugarscape settles a fight inside one tick, but pursuits and battles need
  multi-tick engagements. The new model kind needs finer time without breaking rung 0.
- **Order:** which rungs first. Rungs 1–2 are the foundation. Rung 4 (logistics) and rung 9 (the
  bandit) look like the most distinctive results.
- **Presentation:** if any of this becomes video, it's labeled as our experiment, and violence
  against civilians is shown with restraint: as counts and consequences, never as spectacle.

## Current next step

The [first focused literature pass](2026-10-09-war-literature-pass.md) is complete. W1
engagement-benchmark design is next; no simulator implementation or measurement protocol
is approved by this note. Other existing campaigns retain their independent schedules.
