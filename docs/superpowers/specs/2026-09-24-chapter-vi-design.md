# SugarScape Milestone 8 — Chapter VI: indecomposability and the emergent-society views — Design

**Date:** 2026-09-24
**Builds on:** the milestone 1–7b specs in `docs/superpowers/specs/`; all remain binding where not changed here.
**Source text:** Epstein & Axtell, *Growing Artificial Societies*, Chapter VI ("Conclusions"): "Emergent Society" (animation VI-1 and its list of eighteen views) and "Indecomposability" (animations VI-2, VI-3); Chapter II "Social Networks" (the neighbor connection network, animation II-5); Chapter III animations III-1 (age histogram), III-5 (genealogical networks), III-7 (tag histogram) and III-8 (network of friends, with notes 24–28).
**Out of scope:** Chapter VI's "Other Artificial Societies" — the Schelling segregation variant (animations VI-4 to VI-7) and Ring World flocking (VI-8, VI-9) — are different model kinds and become a later milestone.

## Goal

Reproduce the book's indecomposability demonstration (the same society without and with trade) and complete the "everything on" run's set of views by adding the neighbor, friends and family networks, lineage colors, and the age and cultural-tag histograms.

## Non-negotiable constraints

- **No simulation change.** Every golden entry and legacy fixture stays green and unedited. The new bookkeeping (neighbor lists, friends) is observational: it uses no RNG, never affects behavior, and is excluded from `fingerprint`, configs, exports and share links (like trails).
- **Faithful definitions.** Networks and histograms follow the book's definitions quoted below, including its asymmetric neighbor lists and never-rechecked friends.
- **Quiet when paused.** New data is fetched only while its view is shown and only when stale (the 7a rules, ruling PF6).

## VI-2 and VI-3: indecomposability

- **Book:** "Society consists of an initial population of 500 agents inhabiting Chapter IV's sugar and spice landscape. The agents follow movement rule M and reproduce according to S. In the first run there is no trading. This population crashes" (VI-2, rules ({G₁}, {M, S})). "With everything exactly as it was in animation VI-2 … We turn trade rule T on" (VI-3, ({G₁}, {M, S, T})): the population first declines as in VI-2, then recovers "to a level more than twice that of the initial population", then shows "sustained oscillations" whose minima are "at 700 agents" and whose peaks are "roughly 115 years" apart; "the maximum lifetime of an agent … is eighty years on average".
- **Presets:** `vi-2-no-trade` and `vi-3-trade`: 500 agents, Chapter IV's sugar and spice landscape, spice on, M and S with Chapter III demography and lifetimes 60–100, trade off / on. The two presets differ only in `trade.enabled`.
- **Calibration:** settings not fixed by the text (endowments, fertility ages) are chosen by measurement over seeds 1–5 for 1000 ticks so that VI-2 goes extinct and VI-3 survives with recovery and long waves; any endowment change applies to both presets. The measured outcomes (extinction tick, peak factor, observed wave spacing) are recorded in each preset's description and a code comment. Where the book's exact figures are not reproduced by its stated rules, the recorded measurements stand.
- **Compare entry:** the presets menu gains "Indecomposability — VI-2 vs VI-3 (Compare)", which opens Compare (as a `#c=` link does) with A = `vi-2-no-trade` and B = `vi-3-trade`, both with the seed box's current seed, at t = 0 in lockstep.

## Networks (VI-1 views 2, 8, 10)

- **Neighbor network (Chapter II):** after an agent executes M (whether or not its position changes) the world records the ids of the agents in its von Neumann neighborhood at that moment; the list is kept until the agent's next move. Edges are directed from each agent to each agent on its list (so they may be asymmetric, per the book's note 29).
- **Friends (Chapter III):** while culture is on, when an agent records its neighbors after moving, it compares each with its friends by Hamming distance between tag strings. It keeps at most 5 friends: with fewer than 5 the neighbor is added (if not already a friend); otherwise it replaces the friend farthest in Hamming distance if strictly closer (ties keep existing friends; among equally far friends the earliest-added is replaced). Stored distances are those at the time of meeting; friends are never rechecked (note 28). A friend who dies is dropped. Edges are directed from each agent to each of its friends. With culture off, friend lists are empty and not maintained.
- **Family network (animation III-5):** edges from each living parent to each living child (`parents`/`children` in core).
- **Lineage (animation III-5):** each living agent's class — founder (no parents) or born, and parent (has a living or dead child) or not — rendered by a **Lineage** color mode: founder non-parent black (drawn dark grey where the background is dark), founder parent red, born non-parent green, born parent yellow.
- **Core API:** `World::neighbor_edges()`, `World::friend_edges()`, `World::family_edges()` (pairs of positions, from → to, living agents only) and `World::lineage(id)`; none read or advance `World.rng`.
- **WASM / host:** edge lists as `Uint32Array` quadruples (x₁, y₁, x₂, y₂) like the existing networks; the overlay list becomes `trade | credit | disease | neighbors | friends | family`; the lineage class per cell is rendered by the host's frame renderer when the Lineage color mode is selected.
- **Page:** the overlay checkboxes gain **Neighbor network** (always available), **Friends network** (when culture is on) and **Family network** (when sex is on), drawn by the existing edge renderer: neighbors with a direction marker, friends and family as lines (family parent → child); edges crossing the torus edge are split as trails are. In Compare each grid draws its own world's networks.

## Histograms (VI-1 views 7, 9)

- **Age histogram (animation III-1):** living agents' ages in 5-tick bins from 0 to the largest configured maximum lifetime; shown while lifetimes are finite.
- **Cultural tag histogram (animation III-7):** "the horizontal axis is divided into … one bin for each tag position. The height of the bin gives the percentage of agents having a 0 at that position"; shown while culture is on.
- **Delivery:** core `stats::age_histogram(world, bin)` and `stats::tag_histogram(world)`; WASM functions; snapshot fields `ageHist` and `tagHist` requested through `wants` exactly like `wealthHist` (only while Charts is visible, refetched when the tick moved or an edit/reset/config made them stale). In Compare both overlay as step outlines (A solid, B dashed).

## VI-1

With the additions above, `vi-1-everything` offers all eighteen views of the book's list; its description names where each lives (grid color modes, overlays, charts, Credit tab). No change to its rules.

## Testing

- **Golden/legacy:** unchanged; new golden entries for `vi-2-no-trade` and `vi-3-trade`; a test that a run's fingerprint is identical whether or not the new edge/lineage queries are called.
- **Core unit:** neighbor lists at move time (including an asymmetric case and a stationary move); the friend rule (fill to 5, strictly-closer replacement, ties, no recheck after tags change, removal on death, empty with culture off); family edges; lineage classes; age and tag histograms (bins, ages at the maximum, empty population, 0 % / 100 % positions).
- **Book-style (`#[ignore]`, release):** for seeds 1–5, `vi-2-no-trade` reaches population 0 by the measured tick; `vi-3-trade` survives to t = 1000, peaks above its initial population by the measured factor, and shows at least two population peaks separated by at least the measured spacing (thresholds recorded from the calibration).
- **Web (Vitest):** overlay wants for the new networks; edge splitting at the torus edge; lineage color mapping; histogram chart visibility; a paused page sends nothing with the new charts and overlays shown.
- **Browser (controller):** the VI-2 vs VI-3 Compare entry (extinction vs recovery with long waves); each new overlay on `vi-1-everything` and a Chapter III preset; Lineage colors; both histograms, also in Compare; every existing scenario and the Max-speed performance check.

## Docs

README: the indecomposability presets and Compare entry, the three networks, Lineage colors, the two histograms. Roadmap: mark the Chapter VI item done and add a milestone "Other artificial societies" (Schelling variant, animations VI-4–VI-7: 50 × 50 torus, 2000 Red/Blue agents, von Neumann neighbors, preferences 25 % / 50 % / uniform 25–50 %, random acceptable relocation, residence 80–100 with random-color replacement; Ring World, VI-8/VI-9: 150-site ring, capacity 4 growing back at 1, initial sugar uniform 0–4, 40 agents, vision 15–30 looking counterclockwise, nearest maximum-sugar unoccupied site, random order, and a megagroup start).
