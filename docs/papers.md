# Papers: what we have reproduced and what is next

The playground reproduces published agent-based models from their stated rules, with each
ambiguity or departure as a named switch, and reports where a paper's claims do not reproduce
(see the survey, `docs/superpowers/specs/2026-09-24-model-survey-design.md`). This file tracks the
source papers: which milestone used each, which critiques we have, and the queue.

**Where the files are.** Local copies live in `papers/<topic>/`, which is gitignored: they are
personal copies, never committed or redistributed. File names are
`<authors>-<year>[-<venue>]-<short-title>.<pdf|html>`. A paper marked *not in `papers/`* was
read online or from another copy; add it when found. Scanned PDFs (no text layer) are marked
*scan*: read them page by page as images (`pdftoppm -r 110 -png`).

## Reproduced

| Milestone | Model kind | Sources | Headline |
|---|---|---|---|
| 1–8 | `sugarscape` | `sugarscape/epstein-axtell-1996-growing-artificial-societies.pdf` | Chapters II–VI; VI-2's crash does not follow from the stated rules |
| 9 | `schelling`, `ring` | the book, Chapter VI | — |
| 10 | `anasazi` | Janssen 2009 JASSS 12(4) 13 and the CoMSES data (*not in `papers/`*); Axtell et al. 2002 PNAS (*not in `papers/`*) | the documented model does not reproduce the published fit |
| 11 | `civil` | Epstein 2002 PNAS 99 (*not in `papers/`*) | the stated arrest rule gives no rebellion; only NetLogo's rounding reproduces Model I |
| 12 | `tags` | `tags/riolo-cohen-axelrod-2001-nature-cooperation-without-reciprocity.pdf`; Edmonds & Hales 2003, Roberts & Sherratt 2002 (*not in `papers/`*) | the tables need an unstated tie rule |
| 13 | `spatial` | `spatial-games/nowak-may-1992-…` (*scan*), `huberman-glance-1993-…`, `nowak-bonhoeffer-may-1994-…` | Huberman & Glance's "always all D" holds only above b = 1.8 |
| 14 | `culture` | `culture/axelrod-1997-jcr-dissemination-of-culture.pdf` (*scan*); Axtell, Axelrod, Epstein & Cohen 1996 (*not in `papers/`*) | the docked mobility experiment's single culture does not reproduce |
| 15 | `classes` | `classes/axtell-epstein-young-2000-…`, `classes/poza-et-al-2011-…` | classes never emerge under AEY's rule at their parameters |
| 16 | `ethno` | `ethnocentrism/hammond-axelrod-2006-jcr-evolution-of-ethnocentrism.pdf` | the appendix's 5 % mutation is a slip; color-blind cooperation does not reproduce |
| 17 | `opinions` | `bounded-confidence/hegselmann-krause-2002-…` | Fig. 2b's two camps are the exception; the lattice claim holds |
| 18 | `structure` | `social-structure/cohen-riolo-axelrod-2001-role-of-social-structure.pdf` | reproduces closely; the unstated threshold is 2.3; only the Appendix's noise rule keeps FRNE above 2DK |
| 19 | `dpd` | `demographic-pd/epstein-1998-zones-of-cooperation-in-demographic-pd.pdf` (the working paper), `demographic-pd/epstein-2006-generative-social-science.pdf` (ch. 9: the published rule, Tables 9.1 and 9.3), `demographic-pd/radax-rengs-2009-mpra-replication-of-the-demographic-prisoners-dilemma.pdf` (published as JASSS 13(4) 1, 2010) | Tables 1 and 2 do not reproduce under the published rule; only unstated readings (founders with no wealth, the working paper's rule) come close; the metabolism "equivalence" holds only per game |
| 20 | `norms` | `norms/axelrod-1986-apsr-evolutionary-approach-to-norms.pdf`, `norms/galan-izquierdo-2005-jasss-appearances-can-be-deceiving.html` | Axelrod's 100-generation results reproduce; metanorms usually collapse by 10⁶ (G&I); the unstated tie and refill rules decide whether the metanorm lasts |

## Queue

In rough order: complete original-and-critique pairs first, then the large projects. Every entry is
worth doing; "size" is a guess at the milestone's scale.

| # | Model | Original | Critique or follow-up | Size | Shape |
|---|---|---|---|---|---|
| 1 | Relative agreement and extremism | `bounded-confidence/deffuant-neau-amblard-weisbuch-2000-acs-mixing-beliefs.pdf` (the pairwise original), `bounded-confidence/deffuant-amblard-weisbuch-faure-2002-jasss-how-can-extremism-prevail.html` | `bounded-confidence/amblard-deffuant-2004-network-topology-and-extremism.pdf`, `bounded-confidence/weisbuch-2003-bounded-confidence-and-social-networks.pdf` | medium | extends `opinions` (pairwise updating, uncertainty, networks) |
| 2 | Image scoring | `image-scoring/nowak-sigmund-1998-iiasa-indirect-reciprocity-by-image-scoring.pdf` | `image-scoring/leimar-hammerstein-2001-prsb-cooperation-through-indirect-reciprocity.pdf` | medium | new kind |
| 3 | El Farol and the minority game | `el-farol/arthur-1994-aer-inductive-reasoning-and-bounded-rationality.pdf` | `el-farol/challet-zhang-1997-emergence-of-cooperation-minority-game.pdf` | small | new kind (predictor pools, the memory transition) |
| 4 | Ants and recruitment (herding) | `ants/kirman-1993-qje-ants-rationality-and-recruitment.pdf` | — | small | new kind (N agents, two sources, random recruitment and switching; the bimodal regime) |
| 5 | Threshold models | `thresholds/granovetter-1978-ajs-threshold-models-of-collective-behavior.pdf` (*scan*) | — | small | new kind, or a Sugarscape rule |
| 6 | The timing of retirement | `retirement/axtell-epstein-1999-coordination-in-transient-social-networks-retirement.pdf` | — | small | new kind (age cohorts, rational and imitating agents, a social network) |
| 7 | Altruistic punishment | `punishment/boyd-gintis-bowles-richerson-2003-pnas-altruistic-punishment.pdf` | — | medium | new kind (groups, migration, conflict) |
| 8 | Zero-intelligence traders | `zi-traders/gode-sunder-1993-jpe-zero-intelligence-traders.pdf` (*scan*) | `zi-traders/cliff-1997-hp-minimal-intelligence-agents-for-bargaining.pdf` (Cliff's HP Labs report: where ZI-C fails, and ZIP traders) | medium | new kind: a double auction with values and costs (Sugarscape's `PriceRule::Random` is only the bilateral analog) |
| 9 | Bali water temples | `bali/lansing-kremer-1993-am-anthropologist-balinese-water-temples.pdf` (*scan*) | `bali/janssen-2007-agricultural-systems-coordination-in-irrigation.pdf`; github.com/mars0i/bali (NetLogo, no license: reference only); Janssen's CoMSES model 2221 for the watershed data (check its license first) | large | new kind on a watershed: subaks, dams, rain, pests |
| 10 | Emergence of firms | `firms/axtell-1999-emergence-of-firms.pdf` (108 pp) | — | large | new kind (team formation, power-law firm sizes) |
| 11 | Emergent actors in world politics | `geopolitics/cederman-1997-emergent-actors-in-world-politics.pdf` (274-page book) | — | very large | new kind (states on a grid, conquest, nationalism) |

## Wanted

Papers that would strengthen a milestone, not yet found (every queued model's sources are in hand):

- For the completed milestones' records: Epstein 2002 (civil violence), Janssen 2009 (Anasazi),
  Edmonds & Hales 2003 and Roberts & Sherratt 2002 (tags), Axtell, Axelrod, Epstein & Cohen 1996
  (docking), Lorenz 2006 (bounded confidence).
