# Behavior-tree design: sources and current controller boundaries

**Date:** 2026-10-08. Design reading only; no implementation or measurement.

## Primary sources consulted

- [Colledanchise and Ögren, Behavior Trees in Robotics and AI](https://arxiv.org/abs/1709.00084): control-flow/task switching foundation. Used to motivate explicit execution semantics, not a prediction that a tree wins this laboratory. The linked preprint is a living version of the book; no numeric result is selected for reproduction.
- [The authors' reference library](https://github.com/miccol/Behavior-Tree/blob/master/README.md): ordered fallback/sequence nodes and Success/Failure/Running actions; read-only conditions; action halt on preemption. These published concepts anchor the proposed small executor. SugarScape's one-physical-action-per-turn restriction is our explicit adaptation, not a verbatim reference-library implementation.
- [Colledanchise et al., Formalizing the Execution Context of Behavior Trees for Runtime Verification of Deliberative Policies](https://arxiv.org/abs/2106.12474): execution-context verification as a separate requirement. This proposal does not implement its concurrent/message-passing architecture or reproduce its experiments.
- [Neufeld et al., A Hybrid Approach to Planning and Execution in Dynamic Environments Through Hierarchical Task Networks and Behavior Trees](https://ojs.aaai.org/index.php/AIIDE/article/view/13044): a later integration reference that separates planning from execution. HTN is deferred here.

No source establishes that prewritten trees outperform a competent matched planner, learn routines, reproduce animal decisions or supply a population effect size for this study.

## Source inspected at b678886

Three existing patterns informed the proposal:

1. `minds/utility.rs`: common candidates, score function, nearest/random tie rule and ordinary movement. With one food, travel=1, crowding=0 and idle stay, its score is food/(distance+1).
2. `minds/goap/forage.rs`: retained plans already revalidate the next target, reject failed/unreachable candidates, bound search at 4,096 expansions and record fallback causes. Its current ecological goal is metabolism×horizon, not remaining externally assigned quota. The proposed Task GOAP adapter and unchanged Legacy GOAP reference must remain distinct.
3. `minds/deception/{runner,records}` and its survey archive: default-off research configurations, physical movement/accounting, researcher diagnostics separated from policy inputs, durable one-writer evidence and actual prospective/empirical gates. These are interface patterns; no P4 source, data or acceptance is regenerated.

`movement::arrive` gathers at the actual stopping site, including intermediate steps, uses the existing pathfinder and preserves ordinary metabolism outside the movement action. A tree cannot treat reaching a patch as its only harvest or perform several physical actions in one actor turn.

The legacy fingerprint excludes some historical observational structures, including the GOAP plan. The new tree/task state must be authoritative and covered by its own enabled extension; this is not permission to rewrite historical GOAP fingerprints. Exact matched physical/RNG projection checks accompany per-controller hashes.

## Evidence quality and scope

The sources support a formal software/control foundation. The first task laboratory is our construction, with exact software oracles where available. It selects no quantitative animal validation target. Later foraging or social validation needs its own sources, observation model and registration. Supplied routines and a map prior must not be relabelled learned behavior or discovered information.

The design proposes numerical fixtures for review. No simulation, effect estimate, favorable pilot, registered World or source-code change occurred during this reading task.
