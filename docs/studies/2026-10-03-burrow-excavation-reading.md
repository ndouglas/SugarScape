# Collective burrow excavation research and code reading

**Date:** 2026-10-03

**Status:** initial source and code review for the first excavation design; no simulated results.

## Selected anchor

Pielström and Roces (2013), *Sequential Soil Transport and Its Influence on the Spatial
Organisation of Collective Digging in Leaf-Cutting Ants*, PLOS ONE 8(2):e57040.
[Original article](https://journals.plos.org/plosone/article?id=10.1371/journal.pone.0057040).

Methods describe pellet tracking along a two-metre tube and separate digging-choice experiments.
Transport involved excavators, intermediate carriers and final carriers. Fresh accumulated pellets
biased the first digging location: 56/80 trials selected the pellet side. One-hour-old accumulations
(47/80) and a single fresh pellet (44/80) did not yield significant preferences in their tests.
Non-significance does not establish zero effect. The article offers evidence for transport and
local cue response, not a complete nest-generation algorithm or an optimal relay policy.

Our proposed response rule, age threshold, grid units and relay distance are supplied abstractions.
Implementing a preference and observing it again is verification, not independent validation.
Investigating downstream transport and growth effects is a separate computational question.

## Competing and later anchors

- Green, Bardunias, Turner, Nagpal and Werfel (2017), *Excavation and aggregation as organizing
  factors in de novo construction by mound-building termites*.
  [Original article](https://pmc.ncbi.nlm.nih.gov/articles/PMC5474063/).
  Excavation-site organization is an alternative to assuming deposition pheromones explain
  construction universally. This is a lead for later comparison; full methods/data review remains.
- Prasath et al., *Dynamics of cooperative excavation in ant and robot collectives*;
  accepted 2022, version of record 2023.
  [Publisher](https://elifesciences.org/articles/79638).
  Experimental observations, agent/continuum models and robots offer a candidate reproduction
  campaign. The escape-barrier task differs from building an inhabited burrow. Search-accessible
  text was inspected; full model extraction is deferred. Publisher/PMC follow-up reads encountered
  browser challenges.
- Khuong et al. (2016), [ant nest construction](https://pubmed.ncbi.nlm.nih.gov/26787857/),
  remains relevant to deposition and shaping. Do not substitute deposition rules for excavation
  without identifying the departure.

## Code inspected

- [Minds decision dispatch](../../crates/sugarscape-core/src/minds/mod.rs) is coupled to
  Sugarscape movement and harvest. Burrow digging is a new action/world domain.
- [Grid adapters](../../crates/sugarscape-core/src/minds/grid.rs) distinguish a four-way torus
  from an eight-way bounded benchmark map. Neither is the proposed mutable bounded four-way world.
- [Protection lab](../../crates/sugarscape-core/src/minds/protection/lab.rs) demonstrates
  checked prepared fixtures and explicit restrictions on interacting configurations.
- [Model interface](../../crates/sugarscape-core/src/model.rs) provides common hosting,
  rendering and statistics but a new model kind requires integration across its dispatches.
- [Ants world](../../crates/sugarscape-core/src/ants/world.rs) is the Kirman recruitment model,
  not an excavation simulation. Its name does not make it the right home for this campaign.
- [CLI](../../crates/sugarscape-cli/src/main.rs) uses Clap subcommands and separates I/O failures
  from configuration errors. A bounded lab subcommand can follow those conventions.
- [Seeded RNG](../../crates/sugarscape-core/src/rng.rs) supplies the existing portable generator.

Hornvale was inspected read-only. `../hornvale/crates/hv-world/src/cottage_layout.rs` and
`cottage_layout/arrangements.rs` generate and validate a bounded family of authored cottage
arrangements. Existing checks cover requirements, overlap, reachability, arrival and public
routes through sleeping areas. These are useful examples of functional validation, but are not
a general evolving excavation model. No Hornvale files were changed. Live residents are explicitly
separate from the durable construction brief, another useful distinction for later integration.

## Design recommendation

Start with a standalone bounded core lab and a CLI replay, leaving existing model dispatch and
Minds actions alone. Compare transport mode and cue response independently. Use a choice fixture
to verify the controller, then measure downstream consequences in a growing world. Keep supplied
navigation, work motivation and relay rules visible in results. Integrating planners, presentation,
inhabitation and cultural transmission follows later designs.

See the [first excavation design](../superpowers/specs/2026-10-03-burrow-1-excavation-design.md)
and the [long-term programme](2026-10-03-cultures-construction-and-underworlds.md).
