<!--
A prompt from the Hornvale project (../hornvale) on answering strategies for asks between selfish
agents, kept as received on 2026-09-27. It's a reference for shaping and critiquing our plans,
not a spec. We own the knowledge representation, theory, verification, experiment design and
interpretation; Hornvale consumes the results.

The text arrived damaged by pasting. Several lines lost characters in the middle, marked [sic]:
- CheapHelpReciprocity's formula;
- the Golden Rule's "cost in A's position";
- the Brass Rule's "to";
- the Iron Rule's comparison;
- the circulation measure;
- "stable against invasion under" which settings;
- the determinism constraint.
They're left as received, not reconstructed.

Our critique so far (see the conversation that produced `../2026-09-27-minds.md`):
- Tit-for-Tat and Pavlov aren't defined when asking is one-sided. If H never asks A, there is no
  answer of A's to mirror.
- The Golden Rule needs A's need to be perceivable.
- The study needs no planner. It can start after Minds 3 (memory and belief).
-->

Prompt: response strategies for asks between selfish agents

Hornvale's agent kernel is about to add asking. One agent asks another for something the other holds, such as a tool, food, or a place at a site only one agent can occupy. The holder decides whether to agree. Hornvale's agents are selfish utility planners, so every "yes" needs a reason that comes from the agent's own interests and memory. We'd like the Sugarscape work to investigate which answering strategies produce which population-level outcomes, before Hornvale commits to any beyond a first default.

The exchange (implement it however suits your codebase):
- Ask. Agent A asks agent H for a thing H holds. A can only ask agents within some interaction range, and only individuals it can tell apart. Anonymous agents can't build a history.
- Answer. H answers yes or no using its response strategy. On yes, the thing moves from H to A. Either way, both agents update their memory of the exchange.
- Memory is per individual and fades.
  - Each agent keeps, for each other individual it knows, a few relation values: a debt (how much it owes them), a grudge (how much it resents them), and a tally of that individual's past answers to its asks.
  - All of these decay over time, and decay faster for agents with poor memory.
- Asking has a price. A's planner treats an ask as a chancy action whose success probability is A's belief about H's willingness, taken from A's tally for H, or a prior if A has no history with H. Asking costs time.
- The cost of giving is local. H can compute cost = its expected utility keeping the thing minus its expected utility without it, over its planning horizon. In Sugarscape terms that might be sugar lost from giving some away, measured against H's own metabolism and horizon.

Strategies to compare. Each is a function from (H's coy with A, H's needs, A's apparent need if visible) toyes or no:
1. CheapHelpReciprocity (Hornvale's intended default):t(H→A) − grudge(H→A). Help when it's cheap, and morereadily for those you owe.
2. Golden Rule: yes if A's gain is larger than H's coss in A's position.
3. Silver Rule: never take or harm, but no duty to help. Yes only when the cost is zero or negative.
4. Brass Rule / Tit-for-Tat: mirror A's last answer to
5. Generous Tit-for-Tat: like Tit-for-Tat, but forgive a no with some probability. That probability must come from the seed.
6. Win-Stay, Lose-Shift (Pavlov).
7. Grim Trigger: cooperate until A refuses once, then never again.
8. Iron Rule: help only those stronger or richer than  Or pick another reading of the Iron Rule and saywhich.
9. Always yes and always no, as baselines.
10. Kin preference, if lineage exists there: discount the cost by relatedness.

Measure, per strategy and for mixed populations:
- survival and population size;
- wealth inequality (Gini);
- circulation: the share of scarce, needed things thatg they sit idle while someone needs them;
- the share of asks answered yes, and how asking volume changes as memory builds;
- exploitation: can a population of always-no agents iach strategy, and can each strategy invade always-no?
- how results depend on memory decay (short versus long memory) and on interaction range.

Constraints:
- Deterministic and seed-driven: the same seed gives t
- All randomness comes from named streams derived from the world seed.
- Properties of a population belong in census studies s inside tests.

Please report back:
1. Each strategy's exact decision rule, written as a function of (cost, debt, grudge, answer tally, needs, apparent need), so Hornvale
   can transcribe it as a ResponseStrategy.
2. The measured outcomes, with the seeds and population sizes used.
3. Which strategies are stable against invasion under tings.
4. Anything the interface above lacks that a strategy needed. For example: whether "A's apparent need" has to be perceivable, whether a strategy needed reputation (second-hand informationry, or whether some strategy needed promises or loans.
