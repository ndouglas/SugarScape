# Q-learning auctions: reading notes

Research for Banchio & Skrzypacz (2022), *Artificial Intelligence and Auction Design*.
Written 2026-10-02. This is the brainstorming record, not an approved implementation spec.
No learning experiments have been run. Quantities below are source data or arithmetic on
source data, not SugarScape results. Readers independently investigated the paper, original
code/provenance, and follow-ups; the controller checked the merged pricing engine and figures.

## Intent and constraints

Reproduce the originator's stated model and test its distinction between first-price and
second-price bidding, then its feedback intervention. Findings are the deliverable: separate
the paper's words, plotted observations, reconstruction choices, and our eventual measurements.
Keep unspecified choices named and visible. Fix decision rules before running learners.
Long surveys stay outside CI; preserve existing model fingerprints; use complete sequential
auction periods inside `periods_per_tick`. The implementation plan will come from verified
scratch code, with byte-for-byte reconstruction verification, followed by subagent execution
and independent final review. Those later stages have not started.

## Sources and original-code search

- Original local PDF: `papers/multi-agent-coordination/banchio-skrzypacz-2022-arxiv-ai-and-auction-design.pdf`.
  [arXiv v1](https://arxiv.org/abs/2202.05947v1), submitted 2022-02-12.
  Printed pages are one less than PDF page numbers. The PDF has 11 figures, no result tables.
- [Original TeX archive](https://export.arxiv.org/src/2202.05947v1): inspected in full;
  `arxiv_sub.tex`, bibliography, and figure assets, with no simulator. TeX comments contain
  no numerical optimistic initialization, seeds, or implementation pointer.
- [Banchio's site](https://martinobanchio.github.io/), its complete public repository tree,
  and [public GitHub repositories](https://api.github.com/users/martinobanchio/repos?per_page=100)
  were inspected. Four repositories: academic website and three unrelated projects.
- [Skrzypacz's research site](https://web.stanford.edu/~skrz/Research.htm) and
  [paper metadata](https://web.stanford.edu/~skrz/papers.js): arXiv and publication links;
  no replication link located.
- [AEA-hosted draft](https://www.aeaweb.org/conference/2023/program/paper/r4Ddk4d9),
  dated 2022-08-31, 34 PDF pages: preserves the baseline, still no numerical Q initialization.
- ACM publication/supplement inspection was blocked by HTTP 403. This remains an unverified
  source; the search does not establish that original code was never released.
- [Gmo23/auction_thesis](https://github.com/Gmo23/auction_thesis) is a third-party follow-up,
  explicitly citing Banchio & Skrzypacz as prior work. It is not an author-code authority.

**Conclusion:** no original simulator located in these public sources. Period-for-period
author docking cannot currently be promised. An arXiv source archive is document source,
not simulation code. Do not tune a reconstruction until its output becomes the prose's .24.

## Literal baseline

Paper §§2–4, printed pp. 6–10:

| Parameter | Stated value |
|---|---|
| Strategic bidders | 2 |
| Item and valuations | One indivisible item each period; both values constant at 1 |
| Actions | 19 bids, `b[k] = k / 20`, k = 1..19: .05 to .95 |
| State | None; one Q vector per bidder |
| Learning rate | .05 |
| Discount | .99 |
| Exploration | `.025 * exp(-.0002 * t)` |
| Exploratory action | Uniform over the whole action set (§3 formal description) |
| Initial Q | Optimistic; numerical vector/scalar unstated |
| Horizon | 1,000,000 periods |
| Replications | 1,000, except specified extensions |
| Convergence | Both greedy argmax actions unchanged over the final 1,000 periods |
| Failed convergence | Discard from the paper's terminal distribution |

Use distinct names for learning rate and auction format: the paper uses alpha for both.
For auction parameter a in [1,2], the highest bidder wins and pays
`p = (2-a) * highest_bid + (a-1) * second_bid`.
First price is a=1, second price a=2. Winner reward is `1-p`, loser reward 0.
Equal-bid discussions imply fair allocation; the simulation's sampled versus expected
tie reward is unspecified. Appendix A discusses repeated-game equilibria separately from
the stateless learning simulation; do not import those strategies into the learner.

**Stage-game check, before experiments:** §2.1 p.7 asserts a unique equilibrium at the
highest bid for every auction parameter. Under fair ties, original-grid FPA also has the
weak equilibrium (.90,.90): tying gives .5*(1-.90)=.05, raising to .95 gives 1-.95=.05,
and every lower bid loses. At (.95,.95), tying gives .025 and lowering loses. Both are
Nash equilibria; only the latter is strict. SPA at (.90,.90) instead admits a strictly
profitable .95 deviation paying .90. The paper's footnote 16 itself recognizes the .90
endpoint in the feedback experiment. Treat the equilibrium discrepancy as arithmetic
on the stated game, distinct from any measured learning outcome.

Chosen-action update:
`Q_next(chosen) = (1-lr)*Q(chosen) + lr*(reward + discount*max(Q))`.
Unchosen entries do not change. Bids must be chosen from both old Q vectors before either
bidder learns from the auction. The equation's continuation maximum uses old Q.

Do not borrow Calvano's early stopping: run the full fixed horizon, then inspect its final
stability window. Do not confuse greedy stability with constant realized exploratory bids,
constant rewards, numerical Q convergence, or a Nash equilibrium. Preserve nonconverged
runs in raw output and give discard counts, even though the paper's terminal analysis excludes them.

## Feedback intervention

§5, printed pp. 17–19: reveal the highest rival bid and let each learner compute the reward
it would have earned for every candidate bid. Update every Q entry from the same old maximum:
`Q_next(b) = (1-lr)*Q(b) + lr*(hindsight_reward(b) + discount*max(Q))`.
Do not recompute the maximum during an in-place sweep.

For first price against rival c: reward `1-b` above c, zero below c, unresolved tie reward at c.
The treatment changes both available information and the use of information. A learner that
receives a rival bid but still updates only its chosen action is an information-unused control,
not the paper's synchronous treatment. No-feedback/all-action updating is unavailable unless
an explicitly labelled oracle supplies the missing counterfactuals.

Figure 5 uses **500** simulations. Footnote 16 reports **(.90,.90)**, not (.95,.95), because
of discretization. At (.90,.90), expected tie reward .05 equals the reward from raising to .95;
greedy Q ties can matter. Figure 5b is an example trajectory, not a population trajectory claim;
its time origin and sampling protocol are unspecified. At t=1,000,000 baseline exploration is
essentially zero; do not silently add exploration to imitate the pictured excursions.

## Source figure data recovered before model measurements

Machine-readable transcription: `2026-10-02-q-learning-auctions-source-figures.json`, with
the local PDF's SHA-256, source coordinates, counts, and derived revenues. Original arXiv
document source and the August draft are retained under gitignored `papers/`.

Figure 1's full-resolution embedded PNGs have readable cell annotations. FPA's nonzero entries
are diagonal; SPA includes one off-diagonal outcome. A reader independently checked this
transcription and the weighted means. The annotations, rather than color interpolation, give:

| Bid pair | FPA count | SPA count |
|---|---:|---:|
| (.10,.10) | 110 | 1 |
| (.15,.15) | 202 | 0 |
| (.20,.20) | 202 | 0 |
| (.25,.25) | 192 | 1 |
| (.30,.30) | 164 | 0 |
| (.35,.35) | 92 | 0 |
| (.40,.40) | 34 | 0 |
| (.45,.45) | 4 | 1 |
| (.65,.65) | 0 | 1 |
| (.70,.70) | 0 | 1 |
| (.65,.95), image x/y order | 0 | 1 |
| (.95,.95) | 0 | 994 |
| Total | 1,000 | 1,000 |

These imply mean terminal revenue **.2265 FPA** and **.9471 SPA** (SPA revenue is the lower
bid, including the off-diagonal pair). The text reports **.24**
and **.95**, and says SPA has no dispersion. Preserve the prose and figure as separate targets.
The SPA mean rounds to .95; its six lower outcomes qualify the no-dispersion wording, and the
off-diagonal outcome qualifies the claim that outcomes never lie outside the diagonal. The FPA
gap is not explained by rounding .2265 to two decimals. Different simulation batches are
possible; no source currently identifies the cause. This is a documentary discrepancy,
not an established simulation bug.

Figure 5a annotates **500** at (.90,.90), zero elsewhere. Its colorbar's scale does not agree
with interpreting that annotation as a raw color-normalized count; use the printed count and
state the uncertainty rather than deriving occupancy duration from its colorbar.

Figure 2, original TeX line 230, specifies interval bars with these left-endpoint coordinates:

| Auction parameter a | 1 | 1.1 | 1.2 | 1.3 | 1.4 | 1.5 | 1.6 | 1.7 | 1.8 | 1.9 | 2 |
|---|---|---|---|---|---|---|---|---|---|---|---|
| Collusive percent | 100 | 100 | 100 | 100 | 86 | 41 | 15 | 2 | 0 | 0 | 0 |

The final (2,0) closes the last interval; it does not draw an additional bar. The rendered
centers are 1.05, 1.15, etc., not the supplied coordinates. Vector PDF heights independently
recover the same percentages. The experimental mesh and operational definition of collusive
remain unstated. §4.1's preceding prose reverses the low/high-a relationship; equation,
Result 2 and the plot establish the intended direction. Preserve the inconsistency in reporting.

## Other source experiments and their definitions

| Source | Protocol | Target / unresolved details |
|---|---|---|
| Result 3, Fig.10 | 100 runs; explore immediate neighboring bids | Low FPA/high SPA; boundary rule unspecified |
| Fig.3 | One trajectory; biased toward (.4,.4); epsilon0=.25, beta=.0002 | SPA escapes low bids, FPA remains low; numeric Q bias and averaging window missing |
| Result 4, Fig.4 | Downward choice probability `.62*exp(-.002*t)`; lowest bid within .3 of max Q | Both formats lower; steady-state trigger and clock origin missing |
| Result 6, Fig.6 | Add nonparticipation bids from -.3 through 0 | FPA reaches lower positive bids; exact added mesh and all-out rules missing |
| Result 7, Fig.7 | Reserve .2 | FPA distribution shifts upward; reserve eligibility/payment rules missing |
| Result 8, Fig.11 | 3 bidders, discounts .99 and .999; 500 runs | Higher discount restores low FPA bids; plots show only bidders 1 and 2 |
| Result 9, Fig.8 | Fresh Uniform[0,1] fringe bid each period | FPA mass at .60/.65, above symmetric collusion optimum .50; SPA competitive |
| Fig.9 | One 100-million-period run per format; constant epsilon=.001 | All-period **played-pair occupancy**, not converged-terminal distribution |
| Footnote 13 | Extend baseline to 10m and 100m periods | Nearly unchanged outcomes; exact counts and quantitative tolerance absent |

Constant-exploration runs must continue to the horizon regardless of transient greedy stability.
Their statistical unit is a dependent trajectory, not 100 million independent replications.
Use whole-run occupancy to reproduce Fig.9; late windows and additional seeds are separately
labelled robustness summaries. Do not replace occupancy with a final policy pair.

## Reconstruction choices to name and freeze

1. Numerical `initial_q`. Candidate 100 at discount .99 exceeds the baseline discounted-return
   bound .95/(1-.99)=95. It is our reconstruction, not a value printed in the paper. Scale
   initialization explicitly when varying discount; distinguish stage-payoff and return optimism.
2. Actual-auction ties: sample a fair winner or supply expected fractional rewards. The latter
   changes reward noise. Extend fair ties explicitly for three bidders.
3. Greedy ties: lowest, highest, uniform random, or retain incumbent; fix comparison tolerance.
4. Hindsight ties: expected share or sampled priority, with coupling to actual tie draws explicit.
5. Exploration over all bids (formal default) versus excluding the greedy action (introductory wording).
6. Period origin t=0 or t=1; greedy observation before or after updating; exactly 1,000 final observations.
7. Seed list, PRNG, bidder/action/tie draw order, and treatment pairing. No original seeds are supplied.
8. Extension-specific initialization, local boundaries, downward trigger/clock, nonparticipation,
   reserves, and histogram definitions. Do not hide these behind generic labels such as paper mode.

## Follow-ups, critiques, and AI-safety framing

**Banchio & Mantegazza (2023), *Artificial Intelligence and Spontaneous Collusion***,
[arXiv v5](https://arxiv.org/abs/2202.05946v5), local
`papers/ai-coordination/banchio-mantegazza-2023-arxiv-ai-and-spontaneous-collusion.pdf`:

- §4, pp.22–26: unequal effective update rates retain stale estimates for neglected actions;
  correlated errors can support cooperation without a history-based punishment strategy.
  All-action feedback removes this asymmetry under their assumptions. Weak-dominance results
  require continuing exploration; these are not universal guarantees for arbitrary RL.
- §6, pp.29–32: second-price **participation** learning among eight subsets of three keywords.
  Values U[1,2], reserve 1, CTRs 1/.6/.2; participating bidders bid values. Eighteen of 50
  initialized runs split branded keywords. Figure X: discount .9, epsilon .01, learning rate .005.
  This is a different action space, not a contradiction of the original scalar-bid experiment.
- §7, pp.32–35: minimally informative counterfactual menus; feedback must be used by the learner.
  Uniform effective learning rates are sufficient, not necessary.

**Authors' overview, *Market Design for AI Algorithms***,
[SIGecom PDF](https://www.sigecom.org/exchanges/volume_20/2/BANCHIO.pdf), journal pp.64–67:
constant-exploration cycles; warns that extra information can support different outcomes when
agents condition on history. This warning is also in the original paper's conclusion, pp.23–24.

**Decarolis et al.**, [CEPR DP18009](https://cepr.org/publications/dp18009), 2023/revised 2024:
asymmetric generalized-second-price simulations vary memory/updating; less detailed rival-bid
data can improve revenue. Counterfactual feedback and history-state feedback are distinct designs.

**Rawat**, *Designing Auctions when Algorithms Learn to Bid*,
[arXiv v3, March 2026](https://arxiv.org/html/2306.09437v3): §5.2.1's constant-value factorial
has no significant main format effect on final revenue, with interactions. Bandit and budget-pacing
experiments can reverse the format ranking. It is not an exact replication of the 2022 baseline;
keep algorithm class, market constraints and paper version attached to each result.

**Xu & Zhao (2026)**, local *Memoryless Algorithmic Collusion: Sure to Fail, Slow to Fall*,
`papers/ai-coordination/xu-zhao-2026-arxiv-memoryless-algorithmic-collusion-sure-to-fail-slow-to-fall.pdf`:
[arXiv v2, 2026-08-08](https://arxiv.org/abs/2409.01147v2). The direct protocol audit finds
two transfer limits. Assumption 1(iii) requires a strictly
profitable competitive deviation at each non-Nash symmetric action; the original .90 FPA
profile fails that condition as calculated above. Example 3 assumes a sufficiently fine grid
with a unique strict top equilibrium. Its auction protocol also uses deterministic equal
splitting at ties (p.6), making the unperturbed state transition deterministic (p.7); sampled
winner rewards require a different argument. Its stochastic-stability conclusion concerns
stationary distributions as epsilon tends to zero, not literal absorption at a fixed positive
epsilon. Do not label its theorem a proof for the exact original-grid simulation. Any test
of this critique must implement its own qualifying grid and deterministic reward protocol
as named treatments. A finite 100m-period occupancy result does not settle its asymptotic claim.
Figure 11, pp.29–30, uses persistent epsilon=1e-10 and payment
`w*winning_bid + (1-w)*losing_bid` (w=1 FPA, w=0 SPA). Panel a: discount .95, learning
rate .15, random-versus-best exploration; panels b/c compare w=0/.5/.9, with learning rate
.15 in b and discount .8 in c. The auction passage does not state its bid grid. Their time
is rounds times epsilon; convergence is a Q-value condition (every non-Nash value below
discounted Nash payoff), not the original paper's final 1,000 unchanged greedy actions.
Their acceleration uses Poisson exploration events and deterministic contraction between
events (pp.18–20, 52–53), which cannot be silently used with sampled tie rewards. For a
qualifying reconstruction, state the new grid and call it ours; exact Figure 11 replication
remains underdetermined until its grid is sourced.

AI-safety interpretation: independent reward-seeking learners can create undesirable joint
outcomes through their interaction with feedback. Low bids identify an outcome; they do not
establish intent, communication, deception, credible threats or a learned punishment mechanism.
Record whose objective is harmed (seller revenue), bidder benefits, and the limited transfer
from this symmetric tabular market to advanced agents. Participation, memory and pacing are
distinct follow-up environments, not silent modifications of this reproduction.

## Architecture alternatives and recommendation

1. **New kind `auctions` (recommended).** Dedicated stateless auction learner, mechanism,
   analysis and period runner. Follow `collusion` for portability, schema/presets, tick batching,
   CLI/WASM/page/survey integration. Keep auction terminology and metrics native to the model.
2. Extend `collusion` with auction environments. This would replace its logit/payoff benchmark,
   state interpretation, convergence stop, price-grid and punishment/cycle analysis assumptions.
   Its original spec anticipated pricing environments (Klein, Calvano 2021, AFP), but the merged
   config has no demand/timing environment enums. This is more restructuring than the spec suggests.
3. Extract a generic Q-learning game framework first. Some mathematics is shared, but greedy
   ties, information sets, convergence and statistics differ; the new abstraction would have
   to accommodate unverified future models. Defer until duplication with matching semantics exists.

Evidence read: milestone 34 design including amendments, reading notes, plan introduction/
constraints/decisions and task structure; `collusion/{config,learner,world,analysis,stats,view}`;
`model.rs`, `sweep.rs`, survey collusion claims. Other convention checks: `zi/config.rs` for
mechanism/source switches and `ants/world.rs` for histogram/period view integration.
Milestone 34 source is 3,691 Rust lines across ten files (`scc`).

## Approved design scope (2026-10-02)

Make `auctions` a full playground kind. Core controls: format/mixture, feedback/update use,
optimistic initialization and tie conventions; inspect Q vectors and action update counts.
Display realized and greedy bids, seller revenue and bidder reward separately, plus a bid-pair
occupancy panel. Preserve terminal-convergence summaries separately from trajectory occupancy.
Use one engine for native and WASM; the default tick executes 1,000 sequential periods, with
economic time in periods and an exact fixed horizon (including partial final ticks).

First reproduce Results 1 and 5 and Figure 9. Then cover the paper's remaining Results 2–9
with explicitly named reconstruction choices; do not silently substitute later authors' models.
Add initialization/tie sensitivity and an information-unused control to diagnose dependence.
Participation splitting, memory-state feedback and budget pacing stay next in this kind.
If an extension cannot be specified fairly, report its unresolved source gap rather than
calling an invented protocol a replication.

Before scratch learners run, the written spec must freeze sample counts/seed lists, convergence
selection, whole-run/window statistics, figure targets, numerical acceptance tolerances, and
what counts as below-Nash bidding. Separate directional corroboration, quantitative figure fit,
prose agreement, and exact-code docking (currently unavailable). Report each choice's result;
never select the best-performing reading as the default after observing it.

Paper workloads: 1,000 baseline runs, 500 feedback runs, 100 local-exploration runs and 500
three-bidder runs where captions specify them; standalone survey may exceed the playground
sweep's 100-seed limit. Built-in sweeps remain within 100 seeds and 100,000 ticks; page within
1,000,000 ticks. Batch periods rather than raise those limits. CI tests short hand-computed
auction/update cases, final-window selection, seeded determinism and batching equivalence
for 1/7/1,000 periods per tick; native/WASM fingerprints use short runs. Full surveys are explicit.

The user approved the architecture and scope on 2026-10-02. The written design is
`2026-10-02-q-learning-auctions-design.md`; its review precedes implementation planning.
The research stage has not run a learner or claimed a completed reproduction.
