# Generated worlds: terrain, water and climate as experiments

**Date:** 2026-09-30
**Status:** an idea, persisted for later. Nothing is scheduled or built.
**Separate from:** the reproductions (`docs/papers.md`), though several queued models (circumscription,
war and states, the wave of advance, farming and property, MERCURY, spatial dialects) are its first
consumers.

## The question

How do geography and climate shape populations — where states form, how farming spreads, where
languages split, where wars are fought — when geography is something we can vary, not a single map?
Real maps give one Earth: correlations, not experiments. Generated worlds give ensembles that differ in
one respect at a time (ruggedness, river density, coastline, how walled-in the fertile valleys are,
continental axes, rainfall), so geographic claims become causal tests.

## Principle: scientifically coherent, and checked

A world is only as good as its agreement with what real landscapes do. Before any population runs on
generated terrain, the generator is itself a study, measured against geomorphology's regularities:

- **Hack's law:** main-stream length ∝ drainage area^~0.57–0.6 (Hack 1957, `papers/terrain/`).
- **Horton's laws:** bifurcation and length ratios of stream orders (Horton 1945, wanted).
- **Optimal channel networks' scaling** (Rinaldo et al. 2014; OCNet, Carraro et al. 2020).
- Drainage density, the hypsometric curve, lake and basin statistics.
- **Climate:** rain shadows under a linear orographic model (Smith & Barstad 2004).

## A ladder, simplest first

1. **Noise, as the baseline.** Seeded fBm (Perlin 2002, Gustavson 2005) for elevation and moisture —
   reproducible and simple, and the right control: what does a population do on terrain with no
   drainage structure at all?
2. **Water on noise.** Priority-flood depression filling and watershed labels (Barnes, Lehman & Mulla
   2014, with their flat-resolution companion), D8 flow directions and accumulation (Mark 1983;
   O'Callaghan & Mark 1984 wanted) or D∞ (Tarboton 1997): rivers, lakes, basins.
3. **Erosion.** Tectonic uplift and stream-power incision (Braun & Willett 2013, wanted;
   Cordonnier et al. 2016), or hydrology-first generation (Génevaux et al. 2013): valleys, fans,
   realistic networks. Check the laws above at each rung.
4. **Climate.** Orographic precipitation and temperature by elevation and latitude; soil fertility
   from slope, water and rock.
5. **Scenarios.** Coherent worlds with histories: climate shifts (the Anasazi's droughts; Timmermann &
   Friedrich 2016, wanted), sea-level change, volcanic or plague shocks.

## Hornvale already has much of this

`~/Projects/hornvale/hornvale` (as of 2026-09-30, not yet read closely; how complete and integrated
each domain is needs checking):

- `domains/terrain/` (~30 000 lines): plates, cratons, rifts, isostasy and boundary profiles
  (`plates.rs`, `crust.rs`, `rift.rs`, `elevation.rs`); a sculpting epoch with incision, sediment
  routing, deltas and barriers (`carve.rs`); drainage, streams, channels, water table, lithology,
  strata, caves; assembled on a globe (`globe.rs`). Byte-identity disciplined, calibrated by sweeps.
- `domains/climate/` (~10 700 lines): circulation, currents, precipitation, moisture, temperature,
  snowpack, weather, biomes, crops, habitability, substrate.
- `domains/paleoclimate/`, `domains/history/`, `domains/demography/` (~7 100 lines: carrying capacity,
  niches, founders, kinship, flows), and smaller `settlement`, `religion`, `culture`, `topology`,
  `epidemiology`, `language` domains.
- `../hornvale/crates/hv-noise` (the earlier repository): seeded fBm elevation and moisture.

Options: port what's needed (a flat, bounded grid is a projection of the globe), call it as a
dependency, or export grids as data. The first task is an inventory: what exists, how it's validated,
and what it would take to produce a grid with elevation, water, climate and soil for this project.
Hornvale gains from the other direction too: validation against the laws above.

## Experiments the worlds enable

- **Circumscription** (Carneiro 1970; Williams & Mesoudi 2024–25): do states form where arable land
  is walled in? Sweep how enclosed the valleys are.
- **Continental axes** (Diamond's hypothesis): do crops and techniques spread faster along an
  east–west landmass than a north–south one?
- **War, space and states** (Turchin et al. 2013): how much of the model's fit is geography, and how
  much the steppe's position? Rerun on worlds with the steppe moved.
- **The wave of advance** (Fort 2012; Ackland et al. 2007): how terrain and rivers speed or stall
  farming's front.
- **Rivers as highways or barriers:** trade (MERCURY), dialect boundaries (Burridge 2017), war and
  raiding (the war studies; Griffin & Stanish 2007).
- **Carrying capacity and migration under climate shocks:** the Anasazi valley generalized; drought
  and migration (Kniveton et al. 2011, wanted).
- **Sugarscape itself** on generated land: sugar where water and soil are, and the book's results
  (carrying capacity, wealth, trade, war) as terrain changes.

## Not now

The model queue, milestone 33 (firms) and the other studies continue as planned. This program starts
with the Hornvale inventory whenever it's picked up.
