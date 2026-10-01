# The geopolitics lineage: Cederman's emergent actors and what grew from them — campaign plan

**Date:** 2026-10-01
**Replaces:** queue entry "Emergent actors in world politics" (the whole 1997 book as one very large milestone).
**Reading notes:** `2026-10-01-cederman-reading-notes.md` (the book chapter by chapter, and the literature after it).
**Builds on:** the milestone specs in this directory; each phase below gets its own design spec before it is built.

## The decision

The book was read in full on 2026-10-01 and judged as a source of insight, not only as a source of
models. Its lasting contribution is the stance that states and nations should emerge, form and
dissolve inside the model instead of being assumed; that stance is what GeoSim, the war-size and
democratic-peace models, Turchin's and Gavrilets's models inherit. Its own models are weaker:

- **Emergent polarity (ch. 4–5)** gives existence proofs ("hegemonic takeoff can come from positive
  feedback") on a 10 × 10 grid, 20 seeds a setting, outcomes in coarse bands, with no data. Two of
  its three propositions are contradicted by its own figures; as printed, wars cannot persist; the
  only replication (Störmer 2018) found about three-quarters of random parameter settings inert.
- **Nationalist mobilization (ch. 7)** is a four-state deterministic Markov chain, not an
  agent-based model.
- **Nationalist coordination (ch. 8)** is a 24-province toy with no empirical anchor, and the book
  contradicts itself on its key parameter.

The value lies downstream: GeoSim's war sizes have an empirical target (Richardson's law, the
Correlates of War data) and a refit that can change the conclusion; the democratic-peace models have
clean multi-run tables; and a conquest engine is the substrate Turchin et al. 2013, the strongest
empirical test in the family, needs. So the campaign follows the lineage, with the book's polarity
model as the originator at medium size. The war-and-society study (`docs/studies/2026-09-26-war-and-society.md`)
carries the questions about war itself; this campaign need not model war well, only faithfully.

**Priority:** after algorithmic collusion and Q-learning auctions.

## Phases

### 1. Emergent polarity (medium; new kind `polarity`)

- **Sources, originator first:** Cederman 1994, *ISQ* 38(4) ("Emergent Polarity"), the model's first
  publication, read before the book — it may settle some of the book's ambiguities; then the book's
  ch. 4 and the two ch. 5 variants GeoSim builds on.
- **Base:** 10 × 10 grid of primitive units, predators and status quo states, capitals and provinces,
  N(50, 10) resources and N(2, 5) harvests, the Fig. 4.7 decisions, Fig. 4.8 combat, conquest and
  collapse, and the trust-based alliances; the Fig. 4.10, 4.11 and 4.13 designs (8 predator shares ×
  offense/defense × 20 seeds, to t = 1000 or hegemony), digitized by marker.
- **Named switches for the readings that decide results:** how a war persists (the literal Fig. 4.7
  rule alternates attack and restraint forever; the text says every attack is war until victory);
  whether one-sided attacks destroy resources (ch. 4 yes, ch. 5 fn 1 no); update order (Duffy 1992:
  serial against simultaneous); bounded grid against torus; victory checked before or after losses.
- **Variants from ch. 5:** two-level action (tax and provincial revolt) and overextension
  (probabilistic combat, distance-decaying tax), with the book's inverted rules (pp. 114, 120) and
  wrong worked example (Fig. 5.5) reported; proportional allocation, for one test only (below).
- **Claims tested:** the three propositions (anarchy gives power politics; defense dominance helps;
  defensive alliances help), hegemonic takeoff, survivors mostly predators, and **ch. 5 fn 5's doubt**
  — that the alliance result depends on how resources are allocated — which Cederman never ran.
  Störmer's inertness checked under random parameters.
- **Measured beyond the paper,** in keeping with the war study: resources destroyed, wars, their
  durations and what ended them.
- **GeoSim0's GPL code** (`papers/geopolitics/cederman-icr-2004-geosim0-repast-source.zip`) is read
  for comparison only, never copied.

### 2. Nationalist mobilization (note; no model kind)

Chapter 7's chain as a tested function and a section in the docs: the printed closed forms' typos
(S(0) = 2; the m = b sign; "m − a" for "m − b"), the optimum near m ≈ 0.028 where the text says 0.04,
and the long-run horizon that decides Fig. 7.7. Kuran's feedback (fn 24) and a finite-agent version
only if a later study wants them.

### 3. GeoSim and the size of wars (large; the centerpiece)

- **Sources:** Cederman 2003, *APSR* ("Modeling the Size of Wars"), its 2002 working paper, and
  Cederman 2002, *PNAS* (endogenous boundaries); Clauset 2018 and 2019 for the method and the data.
- **The claim:** technology shocks diffusing through a conquest system give power-law war sizes,
  slopes about −0.55 against the empirical −0.41 (least squares on the cumulative distribution).
- **The test:** refit the model's output and the data by maximum likelihood with a fitted lower
  cutoff (Clauset), with the paper's fit as the literal default; check the paper's text against the
  GROWLab `geosim2` source (GPL; readings only) for code that departs from theory.
- **Also:** Cederman's 2003 finding that state sizes are log-normal, not power-law; the shrinking
  number of states against Abramson 2017's census of European polities, 1100–1790.

### 4. The democratic peace as selection (medium)

Cederman 2001, *JCR* ("Kantian selection"), Cederman & Rao 2001, and Cederman & Gleditsch 2004,
*ISQ* (conquest and regime change; e.g. final democratic share .636 with collective security, .072
without), built on phase 3's engine.

### Optional, if a later need arises

- **Growing Sovereignty** (Cederman & Girardin 2010, *ISQ*): the authors' own robustness failure
  (an exponential distance gradient instead of a logistic one loses the shift to direct rule) is a
  ready-made switch.
- **Axelrod's tribute model** (1995; "Building New Political Actors", 1997): a predecessor, small,
  with its Pascal source recovered; its question — when do many agents become one actor — also
  belongs to the AI-coordination thread.
- **Nationalist coordination (ch. 8):** dropped unless a study wants it.

### Handed on

The phase 3 engine on generated terrain serves queue items Turchin et al. 2013, Gavrilets et al.
2010 and circumscription; Artzrouni & Komlos 1996 (predatory states on a map of Europe, 500–1800)
bridges to the terrain milestone.

## Wanted for this campaign

Cederman 1995, "Competing Identities", *EJIR* 1:331–65 (ch. 8's first publication); Hannan 1979;
Bremer & Mihalka 1977; Cusack & Stoll 1990 (the book); Duffy 1993; Antunes et al. 2002 (BVG choice
in the tribute model); Cederman, Girardin & Müller-Crepon 2023, *World Politics*; Alesina & Spolaore
1997, *QJE*; Cioffi-Revilla & Midlarsky 2004; Tilly 1990, Levy 1983, Fazal 2007, Richardson 1960
(books); Abramson 2017's replication data.
