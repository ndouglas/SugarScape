# Prompt: response strategies for asks between selfish agents (v2)

This replaces the version from 2026-09-27, which arrived damaged by pasting. Every line marked
[sic] in that copy is restored here. This version also answers SugarScape's critique: there is a
fallback for one-sided asking, and an explicit input for apparent need.

## Why

Hornvale's agent kernel is adding *asking*. One agent asks another for something the other holds:
a tool, food, or a place at a site only one agent can occupy. The holder decides whether to agree.
Hornvale's agents are selfish utility planners, so every "yes" needs a reason that comes from the
agent's own interests and memory. We'd like SugarScape to measure which answering strategies
produce which population-level outcomes. Hornvale ships one default now, `CheapHelpReciprocity`,
behind a strategy slot, and will adopt others from your results.

The study needs no planner. "Cost" can be any local, selfish valuation SugarScape already has.

## The exchange

- **Ask.** Agent A asks agent H for a thing H holds.
  - A can only ask agents within an interaction range.
  - A can only ask individuals it can tell apart. Anonymous agents build no history.
- **Answer.** H answers yes or no using its response strategy. On yes, the thing moves from H to
  A. Either way, both agents update their memory of the exchange.
- **Memory is per individual and fades.** Each agent keeps these values for every other
  individual it knows:
  - `debt(X→Y)`: how much X owes Y. It rises by the value received when Y says yes to X.
  - `grudge(X→Y)`: how much X resents Y. It rises when Y refuses X while Y's cost was low, or
    when Y harms X.
  - `tally(X about Y)`: Y's past answers to X's asks (yes count, no count).

  Every value decays over time, and decays faster for agents with poor memory. Keep these records
  private, one per observer: your image-scoring result shows that a global score inflates
  cooperation.
- **Asking has a price.** Asking costs time. A judges its odds as its belief about H's
  willingness, taken from `tally(A about H)`, or from a prior if A has no history with H.
- **The cost of giving is local.** H computes `cost = U_H(keep) − U_H(give)` over its own
  horizon. In SugarScape terms, that is the sugar lost by giving, measured against H's
  metabolism. The cost may be zero or negative, for example when H can't use the thing before it
  spoils or before H next needs it.
- **Apparent need.** `need_A` is A's need as H can perceive it, for example A's visible
  starvation margin. Strategies that use it say so. Measure each such strategy twice: once with
  need perceivable, and once without (use `need_A = 0`).

## Strategies

Each strategy is a function from (`cost`, `debt(H→A)`, `grudge(H→A)`, `tally(H about A)`,
H's own needs, `need_A`) to yes or no.

1. **CheapHelpReciprocity** (Hornvale's default): yes if
   `cost ≤ slack + debt(H→A) − grudge(H→A)`, where `slack` is a small per-agent constant (trait).
   Help when it's cheap, and help more readily the ones you owe.
2. **Golden Rule:** yes if `gain_A ≥ cost`, where `gain_A` is A's gain as H estimates it, using
   `need_A`.
3. **Silver Rule:** never take or harm, and no duty to help. Yes only when `cost ≤ 0`.
4. **Brass Rule / Tit-for-Tat:** answer A as A last answered H. Cooperate first.
   - **One-sided fallback:** if H has never asked A, there is no answer to mirror. Use A's
     answers to others that H has witnessed (within its perception), if any. Otherwise
     cooperate.
   - Report how often each branch of the fallback fired.
5. **Generous Tit-for-Tat:** like Tit-for-Tat, but forgive a "no" with probability `g`, drawn
   from a named seed stream.
6. **Win-Stay, Lose-Shift (Pavlov):** H repeats its last answer to A if its last exchange with A
   went well for H; otherwise H switches. Use the same one-sided fallback as Tit-for-Tat.
7. **Grim Trigger:** cooperate until A refuses H once, then never again. A grudge that never
   decays.
8. **Iron Rule:** yes only if A is stronger or richer than H (A's wealth > H's wealth). Otherwise
   no. If you use a different reading, say which.
9. **Always yes** and **always no**, as baselines.
10. **Kin preference,** if lineage exists: discount the cost by relatedness `r`, so yes if
    `cost·(1 − r) ≤ slack`.

## Measurements

Measure each strategy in a pure population, and in mixed populations.

- Survival and population size.
- Wealth inequality (Gini).
- **Circulation:** the share of agent-ticks in which a scarce, needed thing sits idle (held but
  unused) while some other agent in range needs it. Also report how many times each thing
  changes hands.
- The share of asks answered yes, and how asking volume changes as memory builds.
- **Invasion:** can 10% always-no invaders grow in a population of each strategy? Can 10% of each
  strategy grow among always-no?
- Sensitivity of everything above to:
  - memory decay (short, medium and long half-lives);
  - interaction range;
  - whether `need_A` is perceivable.

## Constraints

- Deterministic and seed-driven: the same seed gives the same run on every platform.
- All randomness comes from named streams derived from the world seed.
- Population properties belong in census-style studies with bands, not in seed sweeps inside
  tests.

## Report back

1. Each strategy's exact rule as implemented, as a function of the inputs above.
2. The outcomes, with seeds and population sizes.
3. Which strategies are stable against invasion, and under which memory, range and need
   settings.
4. What the interface lacked: for example reputation (second-hand information), promises or
   loans, or an operational definition of a "harm" that raises a grudge.
