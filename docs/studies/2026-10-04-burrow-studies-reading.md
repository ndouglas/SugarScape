# Burrow study protocol and resource access source audit

**Date:** 2026-10-04
**Status:** source and code audit supporting proposed documents; no scientific episodes or campaign results; zero-tick constructor checks only.

## Primary sources and evidence limits

Pielström and Roces (2013), [Sequential Soil Transport and Its Influence on the Spatial Organisation of Collective Digging in Leaf-Cutting Ants](https://journals.plos.org/plosone/article?id=10.1371/journal.pone.0057040), DOI 10.1371/journal.pone.0057040. The full publisher article's methods and choice results were inspected. It supports sequential pellet transport and a preference for freshly accumulated pellets in its experiments. Old accumulations and a single fresh pellet did not produce significant preferences there; that does not demonstrate zero biological effects. The model's coefficient, freshness boundary, relay leg, grid and budgets remain supplied abstractions, and source-paper choice proportions are not a reproduction target.

Prasath and colleagues, [Dynamics of cooperative excavation in ant and robot collectives](https://pubmed.ncbi.nlm.nih.gov/36214457/), DOI 10.7554/eLife.79638. The indexed abstract describes carpenter ants excavating a confining corral, and accompanying models and robots. It supports task completion as an organizing research question. Escape from confinement is distinct from resource access or inhabited nest construction. Full publisher and PMC reads met browser challenges in this session; parameter equations, robot controller details and quantitative reproduction claims are not imported into either design. The earlier [excavation reading](2026-10-03-burrow-excavation-reading.md) retains other leads for full review.

Known goal coordinates, goal preference weights and graph-based access evaluation in the proposed Burrow 2 design are engineering benchmark assumptions. Neither source validates those particular mechanisms. Ecological psychology, food acquisition, ventilation, body-dependent accessibility, cultural practices and architectural patterns still need their own mechanism-specific source reviews before being treated as empirical implementations.

## Code and capability audit

Read the implemented `burrow/config.rs`, `fixtures.rs`, `controller.rs`, `runner.rs`, `view.rs`, replay guide and existing measured protection protocol. Read `survey/src/stats.rs::paired_summary` and a current survey manifest implementation for exact-seed matching and archive conventions.

Existing named fixtures are growing, choice and corridor. Choice supports left/right pile placement, three pile types and a terminal selection before action. Growing geometry supplies fixed left-edge staging; it has no public rotation/reflection parameter. Corridor has one available excavation cell, so it is a finite one-unit diagnostic, not a maintained production stream. Run options are fixed ticks and sample cadence; there is no matched-volume stopping rule. Per-worker work, unit histories, spatial dig records, three search-cost families, logical storage and null zero-opportunity rates already exist.

In growing direct transport there is no loose spoil: the fixture starts empty and the controller never drops. Thus direct blind/responsive equality is a structural negative control, not an empirical cue-effect comparison. The interaction contrast is redundant with the relay cue contrast when that control holds.

Resource sites, task objectives, resource-aware target preference and local task-completion memory do not exist. A resource-access extension should reuse physical transactions and scheduling, retain the old configuration and replay paths, and label supplied task information explicitly. It must not read global frontiers or hide navigation scaffolding.

## Design alternatives considered

For Burrow 1, an immediate four-condition sweep is easy but lacks a frozen archive/analysis contract. A reproduction campaign would need full source-method mapping that the current abstraction does not offer. The recommended first protocol is a prospective computational comparison with explicit primary outcomes, verification panels and a reviewed archive boundary.

For Burrow 2, structural access can be measured with existing movement and digging physics. Actual harvesting and delivery would require a second resource ledger, shared carrying semantics and additional actions. Physically emitted resource cues would require a diffusion/attenuation model and sensory evidence. The recommended first increment measures structural access and compares unguided construction with an explicitly known-goal benchmark. Harvesting and physical signals remain distinct later increments.

## Draft verification record

The candidate manifest contains 32 unique conditions and forty scientific seeds per condition: 1280 proposed episodes. Its growing panels contain 800 episodes at 4096 opportunities each, hence 3,276,800 declared action opportunities; the 480 choice records commit no actions. Eighteen separate corridor configurations at seeds 7 and 8 give 36 construction records and 18,432 opportunities if later executed. There are six primary estimates, not ten: the direct cue contrast is an exact negative control and the interaction is a derived duplicate when it holds.

All fifty fully resolved configurations were passed to the already-built baseline native CLI at seed 7 with zero requested ticks. Each constructor accepted the configuration and exported exactly matching normalized values, zero action events, zero selections and zero opportunities. Temporary outputs were removed. These checks validate the current schema and geometry, not behavior or comparative outcomes; no scientific seed was executed.

Static inspection verified condition uniqueness, panel counts, action budgets, primary contrast keys and positive sample intervals. All 23 relative links across the changed/new prose documents resolve. The candidate manifest SHA-256 is `3ef020036fa96ebc5a440480dccfcab64fd8991dfc2ff2dc4129b05466acb890`; this identifies a proposed artifact, not a registered study. No runtime files or dependencies changed. The merged engine baseline's passing regression evidence remains applicable; unchanged broad suites were not repeated for documentation.
