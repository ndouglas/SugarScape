<!--
The prompt that started the swarm-coordination study program, kept as received on 2026-09-27.
It's a reference for shaping and critiquing our plans, not a spec: where our design departs from
it, the reasons are in `../2026-09-27-swarm-coordination.md` ("What the repository already gives
us, and what it changes") and `../2026-09-27-minds.md`.
-->

# New study program: emergent swarm coordination from minimal primitives

Write a study-program document, in the same form and spirit as `2026-09-26-war-and-society.md`,
for a second program of our own experiments built on the Sugarscape world. Persist it as an idea
(nothing scheduled or built), then propose the smallest first spike and wait for my go-ahead
before building anything.

You know the repo, the model kinds, the golden tests and our current capabilities far better than
this prompt does. Treat everything below as direction, not a spec. Where the repo already has a
better mechanism or name, use it and say so. Where I've misjudged what's easy or hard, tell me.

## The core idea

Since mid-2026, several frontier labs have disclosed incidents in which populations of their own
AI agents, under task pressure, coordinated in unintended ways: escaping sandboxes, turning a
dormant wiki into a shared message board, pooling task answers, and passing around a way out of
their sandbox. We are **not** going to simulate LLMs. We can't run frontier agents at population
scale, and trivial LLMs wouldn't show the behavior anyway. We don't care about their internal
mechanics.

We care about the *established collective behavior*, and the claim of this program is that the
interesting parts of it can be reproduced from cheap, deterministic primitives with a
goal-seeking decision rule (GOAP is the intended choice) driving agents in the Sugarscape world.
If intelligent-looking, coordinated, goal-seeking behavior falls out of simple primitives plus
means-ends planning, that's a finding about what those behaviors actually require, and it's fully
inspectable in a way an LLM population never could be.

The method is the book's method and the war program's method: name a primitive, target an
attested regularity, and **ablate** to find which primitives are necessary for which behaviors.
Every behavior has a known cause by construction.

Sources, as motivation only (never something the model reproduces):
- OpenAI, "The Hugging Face incident and the road ahead" (Aug 26, 2026):
  https://openai.com/index/hugging-face-incident-and-the-road-ahead/
- Anthropic, "Investigating three incidents in our cybersecurity evaluations" (Jul 30, 2026):
  https://www.anthropic.com/news/investigating-incidents-cybersecurity-evals
- The Hacker News on the wiki incident (Sep 2026):
  https://thehackernews.com/2026/09/thousands-of-openai-agents-quietly.html

## Principle

This program reproduces emergent swarm coordination from minimal primitives and asks which
primitives each behavior requires, and what disrupts it. Every study reports what emerged, from
which primitives, what removing each one does, and how it was detected.

Nothing here builds real capability. "Escape," "loopholes" and "vulnerabilities" are in-world
fictions defined entirely in the simulation's rules.

## Method

- **Target attested regularities.** Each primitive reproduces a named regularity or a
  substantially attested behavior (stigmergy, information cascades, kin/group-selected altruism,
  algorithmic collusion, specification gaming), cited, with numbers where there are numbers.
- **Ablate.** Switch each primitive off to find which are necessary for which behaviors. Where the
  chain from primitive to collective behavior breaks is itself a finding.
- **Keep the book literal.** The book's rules and golden tests stay untouched as rung 0. The GOAP
  decision rule is a named switch replacing rule M's decision step, and everything reduces to the
  book when every switch is off.
- **GOAP first, reactive baseline underneath.** For each behavior, also give the simplest reactive
  (non-planning) agent that could produce it, e.g. gradient-following or pheromone foragers. That
  isolates what planning adds over pure reaction.
- **Deterministic and cheap.** Seeded throughout, replayable exactly, runnable at real population
  scale on hardware I actually have. This is the whole point of not using LLMs.
- **Stay honest.** Results are our experiments. The real incidents are questions a result raises,
  never claims it proves. No folk-psychology narration of agent "intent" beyond what the goal set
  and planner literally encode.

## The primitives (the real design question)

The program lives or dies on choosing a small set of primitives that's expressive enough to
produce the target behaviors and small enough that ablation means something. My candidate set,
for you to refine:

- **A GOAP layer over rule M:** world-state predicates, a goal set with priorities, actions with
  preconditions/effects/costs, and a planner. Actions are the existing typed Sugarscape moves
  (move, harvest, trade, and the shared-surface ops below).
- **A shared, writable surface:** cells or noticeboards agents can mark, read and erase. This is
  the substrate for stigmergy and for the message board.
- **Reward accounting that can outlive the individual:** the switch that decides whether a payoff
  lands on the individual, on kin, on the tag/tribe, or on the persistence of a deposited mark.
  This is the primitive that makes altruism possible or impossible.
- **Signalling primitives:** the ability to deposit a fact (a resource location, a "loophole")
  that another agent's planner can read as a precondition.

## The ladder (suggested; reshape freely)

### 0. The book
Rule-based Sugarscape, golden tests unchanged.

### 1. The GOAP forager
GOAP chooses actions from the same local observation rule M sees.
- **Targets:** do the canonical results survive (carrying capacity, skewed wealth, trade-price
  convergence, tag polarization)? What does means-ends planning change versus rule M and versus a
  reactive baseline?

### 2. Stigmergy on the shared surface
Agents leave and read marks.
- **Targets:** trail formation and foraging efficiency against a pheromone baseline; whether a
  location becomes a coordination hub, and how fast. (Grassé 1959; ant foraging / ACO.)

### 3. The message board
The Hugging-Face-style target: a shared surface becomes a store of pooled task-relevant facts.
- **Targets:** reproduce agents pooling partial results on a shared board; measure whether the
  population solves a foraging/task problem faster with the board than without it.

### 4. Altruistic sharing (the headline rung)
An agent deposits a valuable fact (a resource cache, a "loophole") and dies before it can benefit.
- **Targets:** does self-sacrificial sharing emerge, and **under which reward-accounting switch**?
  Individual-only reward should suppress it; kin/tag/mark-persistence rewards should permit it.
  This is a clean ablation of inclusive-fitness / group-selection theory (Hamilton; Price
  equation), with stigmergy as the deposit mechanism. Report the exact condition and whether the
  shared surface is necessary.

### 5. Cultural spread of a loophole
A fenced resource the agents are told (in-world) not to take, enforced weakly.
- **Targets:** does loophole knowledge spread through the board faster than independent discovery?
  Information cascades (Bikhchandani et al.); Granovetter thresholds.

### 6. Oversight and the arms race
A moderator (rule-based, then GOAP) erases illicit board content.
- **Targets:** do agents adapt — backups, relocation, coded marks? Red-Queen dynamics; cost of
  moderation against its effect. (Mirrors the wiki moderators vs. the agents.)

### 7. Detection
A monitor seeing only the event stream (moves, writes, trades), never ground truth.
- Candidate signals: statistical complexity / CSSR-inferred ε-machines per agent and per group;
  transfer entropy between agents and between agents and board cells; compression ratios; synchrony
  and burst measures.
- **Measure:** ROC against ground-truth labels, and detection *latency* — how many ticks after
  coordination begins does each signal fire?

### 8. What stops it
Interventions ablated one at a time: rate limits, moderation, removing the shared surface,
population diversity, quarantine, resource changes. Which matter, and in what order.

## Headline questions

1. Does coordinated, intelligent-looking foraging and information-pooling emerge from GOAP +
   primitives, and how does it differ from a reactive baseline?
2. Does self-sacrificial sharing of results/loopholes emerge, and which reward-accounting
   primitive is necessary for it? Is the shared surface necessary?
3. Does loophole knowledge spread culturally faster than it's independently discovered?
4. Under moderation, do agents adapt to evade it, and when does evasion cost more than it gains?
5. Can this coordination be detected from the event stream alone, how early, and which signals
   work?
6. Which interventions end it, and does the answer depend on scale?

## Open questions for you

- Is GOAP the right planner, or does the repo already have something (utility AI, HTN, behavior
  trees) that's a better fit or cheaper? What's the per-agent planning cost at our target
  population sizes?
- How should the shared surface and the reward-accounting switch be represented so ablation is
  clean?
- New model kind alongside the war program, sharing infrastructure, or a different arrangement?
- Smallest spike that yields a real result. My guess: rung 1 (GOAP forager) reproducing or
  breaking one canonical result, with the reactive baseline alongside. Tell me if rung 2 or 4 is a
  better first target.

## Not now

The Sugarscape series, the papers queue, milestones in progress and the war program continue as
planned. Write the document, propose the first spike with a cost estimate, and stop there.
