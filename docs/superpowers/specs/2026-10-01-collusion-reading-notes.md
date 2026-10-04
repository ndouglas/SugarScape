# Algorithmic collusion: reading notes

Research notes for the algorithmic-collusion milestone (`2026-10-01-algorithmic-collusion-design.md`).
Section 1 is Calvano, Calzolari, Denicolò & Pastorello (2020, *AER*) with its online appendix and
the authors' Fortran replication package (MIT); section 2 the follow-ups Calvano et al. (2021,
*IJIO*) and Klein (2021, *RAND*); section 3 the critiques and foundations (Asker, Fershtman & Pakes
2021–22; Waltman & Kaymak 2006–08; Sandholm & Crites 1996); section 4 the literature since; section 5
Epivent & Lambin (2023), Lambin (2024) and den Boer, Meylahn & Schinkel (2026) read against the spec. Each was
written by a separate reader on 2026-10-01. Scripts and renders mentioned were throwaway and not kept.
Local copies are in `papers/ai-coordination/`.

---------------------------------------------------------------------------------------------------

## 1. Calvano, Calzolari, Denicolò & Pastorello (2020), "Artificial Intelligence, Algorithmic Pricing, and Collusion", AER 110(10): 3267–97

Notes for a reproduction in the Rust playground. Written 2026-10-01.

### Sources in hand

- Paper: `papers/ai-coordination/calvano-calzolari-denicolo-pastorello-2020-aer-ai-algorithmic-pricing-and-collusion.pdf` (31 pp.; journal pages 3267–3297).
- Online appendix (24 pp., dated May 2020, from aeaweb.org/articles/materials/13398): `papers/ai-coordination/calvano-calzolari-denicolo-pastorello-2020-aer-online-appendix.pdf`.
- Replication package (openICPSR E119462V1; MIT license): `papers/ai-coordination/calvano-calzolari-denicolo-pastorello-2020-aer-replication-package.zip`. openICPSR is behind Cloudflare and needs a login, so I took a verbatim mirror from GitHub `gsekeres/cornell_theory_reading_group_RL/calvano_replication/calvano-et-al-replication-code`. It contains:
  - `AER_fcode/{baseline,stochasticdemand,entryexit}/*.f90`: Intel Fortran with OpenMP, plus Windows .exe files.
  - `AER_paper_Rscripts/{figure_1..11,table_I..IV}/`: an `A_InputParameters.txt`, an R script, and the **output** (`table_*.txt`, `figure_*.pdf`) for each figure and table. Large simulation outputs are not included.
  - `AER_online_appendix_Rscripts/...`: the same for the appendix.
  - `README.pdf` (code guide, July 2020), `Table replication paper.pdf`, `Table replication online appendix.pdf`.
  - A scratch copy (unzipped) is at `scratchpad/cornell/calvano_replication/calvano-et-al-replication-code/`.
- Also on disk: `calvano-...-2021-ijio-algorithmic-collusion-imperfect-monitoring.pdf`, `klein-2021-rand-...sequential-pricing.pdf`, `waltman-kaymak-2006...pdf`, `asker-fershtman-pakes...pdf`.
- Third-party ports exist: matteocourthoud/Algorithmic-Collusion-Replication (Python/Matlab/Julia) and Yusei406/calvano-qlearning (Python). I did not read them closely. The original Fortran is the authority.

**The R figures in the package are vector PDFs.** Their curves can be read exactly from the PDF content streams (`m`/`l` path coordinates plus axis-tick calibration), so they don't have to be digitized from raster images. I did this for Figures 3, 4 and 6 (values below) with `scratchpad/irdig.py`. The heat maps are lattice `levelplot`s, so their cells are vector rectangles with fill colors; mapping colors back to the colorkey would recover all 91×100 cells.

---

### 1. The spec

#### 1.1 Stage game (paper §II.A, p. 3273)

Logit demand, eq. (5), quoted exactly:

    q_{i,t} = exp((a_i − p_{i,t})/μ) / ( Σ_{j=1}^n exp((a_j − p_{j,t})/μ) + exp(a_0/μ) )

- Reward: π_{i,t} = (p_{i,t} − c_i) q_{i,t}. The code matches exactly (`PI_routines.f90::logitDemands`).
- Baseline (p. 3274): n = 2, c_i = 1, a_i − c_i = 1 (so a_i = 2), a_0 = 0, μ = 1/4, δ = 0.95, m = 15, ξ = 0.1, k = 1.
- The text says the "price-cost margin ≈ 47% in Bertrand, about twice under collusion". That is (p − c)/c: 0.473 and 0.925. The Lerner index would be 0.32 and 0.48.

#### 1.2 Benchmarks p^N and p^M

- The paper says only "we compute both the Bertrand-Nash equilibrium of the one-shot game and the monopoly prices (i.e., those that maximize aggregate profits)" (p. 3274).
- **The code does not compute them.** They are inputs (`NashP*`, `CoopP*`), "computed with a Mathematica script available upon request" (README §3.2), rounded to 5 decimals.
- Baseline inputs: p^N = 1.47293 and p^M = 1.92498. I re-derived both independently: symmetric FOC (p − c)(1 − q)/μ = 1 gives p^N = 1.472927, and the joint-profit argmax gives p^M = 1.924981. They agree.
- π^N = 0.222927 and π^M = 0.33749 (R scripts hard-code these). The code evaluates them at the **continuous** (rounded) p^N and p^M, not at grid points.
- Other values from the input files, all checked by me:
  - n = 3: p^N = 1.37016, p^M = 2.0 (exactly 2.000000).
  - n = 4 (mine): p^N = 1.331461, p^M = 2.054411.
  - μ = 0.5: 1.79947 / 2.18741. μ = 0 is a separate Bertrand routine with p^N = 1 and p^M = 2.
- Asymmetric-cost inputs, (c2: p^N_1, p^N_2, p^M_1, p^M_2):
  - c2 = .875: 1.44135, 1.39588, 1.97674, 1.85174
  - c2 = .75: 1.4142, 1.32539, 2.04051, 1.79051
  - c2 = .625: 1.39133, 1.26145, 2.11503, 1.74003
  - c2 = .5: 1.37233, 1.20377, 2.1984, 1.6984
  - c2 = .25: 1.34369, 1.10519, 2.38411, 1.63411
  - Joint maximization gives equal markups: p^M_1 − p^M_2 = c_1 − c_2.

#### 1.3 Price grid (p. 3274)

"m equally spaced points in the interval [p^N − ξ(p^M − p^N), p^M + ξ(p^M − p^N)]."

- The code (`computePIMatricesLogit`) builds a separate grid for each agent from that agent's own p^N_i and p^M_i. It sets the endpoints, step = (hi − lo)/(m − 1), and fills `P(i) = P(i−1) + step` by **cumulative addition**. For bit-exactness, do the same.
- Baseline grid (index: price): 1: 1.42773, 2: 1.46647, 3: 1.50522, 4: 1.54397, 5: 1.58271, 6: 1.62146, 7: 1.66021, 8: 1.69896, 9: 1.73770, 10: 1.77645, 11: 1.81520, 12: 1.85394, 13: 1.89269, 14: 1.93143, 15: 1.97018.
- Δ when both firms charge a symmetric grid price (my computation): idx2 −0.024, idx3 0.117, idx5 0.378, idx8 0.707, idx9 0.794, idx10 0.868, idx11 0.926, idx12 0.969, idx13 0.993, idx14 0.9997, idx15 0.987. This matches fn 21: "Δ ≈ −2% if Nash approximated by defect, 12% by excess."
- Alternative grid flag (README; Table A12 row "lowest price 0.99"): if extend1 < 0, the lower bound is c(1 + ξ1) and the upper is p^M + ξ2(p^M − p^N). The input is −0.01, giving a lowest price of 0.99.

#### 1.4 State and memory (p. 3274)

- s_t = {p_{t−1}, …, p_{t−k}}. |A| = m and |S| = m^{nk}. The baseline has 225 states and a 225×15 Q per agent, i.e. 3,375 cells.
- The state number is a mixed-radix encoding: `1 + Σ cStates·(idx − 1)`, agents' most recent prices first (`computeStateNumber`).
- The initial state s_0 is drawn uniformly at random ("drawn randomly at the beginning of each session", p. 3275). In the code it comes from one RAN2 stream seeded with idum = −1 for all sessions (`generate_uIniPrice`), with p = 1 + INT(m·u).

#### 1.5 Q-learning update, eq. (4), p. 3271

    Q_{t+1}(s,a) = (1−α) Q_t(s,a) + α [π_t + δ max_a Q_t(s′,a)]

- Only the visited cell changes. Code:

      newq = oldq + α (PI(action) + δ·maxValQ(s′) − oldq)

- Agents are updated in order 1..n. Each agent's own Q only.
- `maxValQ` and the greedy action `strategyPrime` are **maintained incrementally**:
  - if newq > maxVal, the argmax becomes the visited action;
  - if newq < maxVal and the visited action *was* the argmax, the row argmax is recomputed with **random tie-breaking** (`MaxLocBreakTies`; ties within EPSILON(1d0) ≈ 2.2e-16);
  - an exact tie with the current max does not switch the argmax.

#### 1.6 Exploration, eq. (7), p. 3274

- ε_t = e^{−βt}.
- Code: ε starts at 1.0 and, *after* each use, ε ← ε·exp(−β). So ε_t = e^{−β(t−1)} for t = 1, 2, …. Each agent has its own ε, decaying identically.
- β is entered per episode: input `Beta = β × 25,000`, and `ExplorationParameters = exp(−MExpl/25000)`.
- Each period draws 2n uniforms in the order u(1,ag1), u(1,ag2), u(2,ag1), u(2,ag2).
- Agent i explores iff u(1,i) ≤ ε_i. It then picks price 1 + INT(m·u(2,i)), uniform over **all m** prices including the greedy one. Otherwise it plays the greedy action.
- A negative input β means "no exploration" (used for the rematch in Fig. 11).
- Baseline β = 4×10⁻⁶ (input 0.1).

#### 1.7 Q-matrix initialization, eq. (8), p. 3275

    Q_{i,0}(s, a_i) = Σ_{a_{−i} ∈ A^{n−1}} π_i(a_i, a_{−i}) / ((1 − δ) |A|^{n−1})

- This is identical for every state. Code type 'O': `den = COUNT(...)*(1−δ)`, the mean over rivals' actions divided by (1 − δ).
- Baseline Q0 row (mine): 5.790, 6.008, 6.162, 6.252, **6.278** (argmax = idx 5, 1.5827), 6.244, 6.153, 6.010, 5.821, 5.593, 5.332, 5.047, 4.744, 4.430, 4.111.
- So until learning moves a row, the initial greedy action everywhere is price idx 5.
- Alternatives (`initQMatrices` and appendix A5.6):
  - F: Q^π for the rival playing a fixed price;
  - G: grim trigger (exact discounted formula);
  - T: a random trained Q from an earlier run;
  - R: U(lo, hi) per cell;
  - U: constant.

#### 1.8 Convergence (p. 3276)

- "If for each player i and each state s the action a_{i,t}(s) = argmax[Q_{i,t}(a,s)] stays constant for 100,000 repetitions … We stop … in any case after one billion repetitions."
- Code: 25,000 iterations per episode. The performance period is 4 episodes, which is 100,000 iterations.
- **The cap is maxNumEpisodes = 50,000 episodes, i.e. 1.25×10⁹ iterations**, not 10⁹.
- The counter increments when the greedy actions at the *visited* state are unchanged in that iteration and resets to 1 otherwise. Only the visited row can change, so this is equivalent to "the whole strategy is unchanged".
- Exploration and Q updates continue during the 100k window.
- The reported time-to-convergence is (iItersFix − 100,000)/25,000 episodes, i.e. it excludes the verification window.

#### 1.9 Sessions

1,000 sessions per experiment (p. 3273; `Number of sessions 1000` in every input file). Per-session RNG:
- exploration stream: RAN2 with idum = −iSession;
- tie-break and Q-init stream: a second RAN2 with idum = −iSession;
- initial prices: one stream with idum = −1 shared across sessions.

RAN2 is the L'Ecuyer generator with Bays–Durham shuffle (NR in Fortran 77, p. 272), modified to be thread-safe. It is about 40 lines and easy to port for bit-exact comparison.

#### 1.10 Outcome measured after convergence (not stated precisely in the paper)

Code (`ConvergenceResults.f90`):
1. Take the strategies at convergence (`strategyFix`) and the last state.
2. Replay **deterministically, with no exploration**, from the last state until a state repeats.
3. The **limit cycle** is the repeated portion. The per-session profit for each agent is the **average over the cycle**.

Δ (eq. 9) = (π̄ − π^N)/(π^M − π^N):
- π̄ is the mean over sessions and agents of the cycle-average profit.
- In asymmetric runs, the overall Δ uses the agent-averaged π^N and π^M. Per-agent ΔG_i are also written.
- Table I's per-session profit gain = mean over the two agents of (cycle-average profit_i − π^N_i)/(π^M_i − π^N_i).
- **Unconverged sessions are included** in every statistic except mean time-to-convergence. There is no convergence mask in ConvergenceResults, DetailedAnalysis, or the R scripts. Their strategy at the 1.25e9 cap is used.
- Stochastic demand: profits are replayed with **expected** π over the a0 values, and Δ uses the expected π^N and π^M.

#### 1.11 Equilibrium check and Q-loss (p. 3278; `EquilibriumCheck.f90`, `QGapToMaximum.f90`)

- The paper says the "true" Q is solved using eq. (3), i.e. with max over a′ in the continuation, and then compared with best responses.
- **The code instead computes Q^π (policy evaluation):** `computeQCell` plays action a in state s, then all agents (*including i*) follow their limit strategies, until a state repeats. It uses an exact geometric sum: Q = pre-cycle sum + δ^L·(cycle sum)/(1 − δ^C).
- Best response at s means π_i(s) ∈ argmax_a Q^π_i(s,a), with exact equality in EqCheck and an EPSILON tolerance in QGap.
- "Nash on path" means that both agents pass this check at every state of the limit cycle. FlagEQOnPath is the session-level 0/1. Figure 2 and Table I's "Frequency of Nash equilibria" are the session mean of this flag.
- Q-loss(s, i) = (max_a Q^π − Q^π(s, π(s)))/|max_a Q^π|.

---

### 2. Results to match

(R = numbers available exactly from the package `table_*.txt`; D = digitize.)

#### Baseline / representative experiment: α = 0.15, β = 4×10⁻⁶ (ν ≈ 20)

**Table I (R, exact; `AER_paper_Rscripts/table_I/table_I.txt`)**

| | 1-Sym | 1-Asym | 1 | 2 | ≥3 | All | Nash eq |
|---|---|---|---|---|---|---|---|
| Frequency | .277 | .366 | .643 | .238 | .119 | 1 | .505 |
| Avg Δ | .866 | .855 | .860 | .846 | .793 | .849 | .854 |
| SD Δ | .115 | .114 | .114 | .104 | .097 | .112 | .108 |
| Freq Nash eq (on path) | .686 | .661 | .672 | .294 | .025 | .505 | 1 |
| Q-loss on path (mean/SD) | .001/.002 | .001/.004 | .001/.003 | .002/.003 | .004/.006 | .002/.004 | 0/0 |
| Q-loss "all states" (mean/SD) | .018/.006 | .018/.007 | .018/.006 | .018/.006 | .018/.006 | .018/.006 | .018/.006 |

**Text claims at the representative point:**
- "In more than 95% of the cases the punishment makes the deviation unprofitable" (p. 3282). Table A5 gives IC = 0.936 for the same experiment.
- Punishment typically ends after 5–7 periods. The mean punishment length is 5.705 (A5).
- IR = −0.127, the average relative price change of the nondeviator at τ = 2 (A5).

**Figure 4 (D, exact from vector PDF).** One-period deviation to the static best response, average over 1,000 sessions. Prices for τ = 0..15, ±0.003 from marker-center calibration.
- Long-run (horizontal line) = 1.7914. Nash line 1.4729, monopoly line 1.925.
- Deviating agent: 1.792, **1.505**, 1.560, 1.579, 1.639, 1.693, 1.733, 1.762, 1.777, 1.785, 1.789, 1.790, 1.792, 1.792, 1.793, 1.791.
- Nondeviating agent: 1.795, 1.795, **1.551**, 1.592, 1.638, 1.694, 1.737, 1.765, 1.780, 1.785, 1.792, 1.794, 1.795, 1.794, 1.795, 1.794.
- Deviation price ≈ 1.505 = grid idx 3 on average. It is computed per session and per cycle state as `MINVAL(MAXLOC)` of one-period profit given the rival's strategy action in that state. Ties go to the lowest index.

**Figure 6 (D, exact).** Smaller deviation: a one-period deviation to **grid index 8 (1.699)** (`DevPrice = 8`, from `A_irToAll.txt`).
- Deviating agent: 1.792, 1.700, **1.592**, 1.598, 1.651, 1.696, 1.741, 1.763, 1.780, 1.786, 1.792, 1.788, 1.796, 1.790, 1.794, 1.789.
- Nondeviating agent: 1.795, 1.795, **1.588**, 1.603, 1.649, 1.705, 1.741, 1.768, 1.782, 1.790, 1.793, 1.793, 1.795, 1.795, 1.794, 1.794.
- The overshoot at τ = 2 is below the τ = 1 deviation price for both agents.

**Figure 5.** Box plots, τ = 0..10, of the price at τ minus the long-run price, for the deviating and nondeviating agents.
- Code (`figure_5.R`): **only sessions with symmetric, length-1 limit cycles**, deviation = static BR, *and* only the modal pre-shock price.
- Box = 25–75%. Whiskers = **2.5–97.5% quantiles**, although the caption says "ranges".
- Y range is about −0.4 to +0.1. Digitize from `figure_5.pdf`.

**Tables II & III (R, exact; `table_II_panel_*.txt`, `table_III_Panel_*.txt`; full 15-column versions = appendix A2, A3).**
- Rows: deviator's pre-shock price, idx 6..15, 1.62..1.97, sessions with cycle length 1.
- Columns: deviation price idx 1..row.
- Row frequencies: 0.01, 0.06, 0.11, 0.16, 0.19, 0.18, 0.11, 0.09, 0.05, 0.03.
- Table II A = nondeviator's relative price change at τ = 2. Examples: row 1.78 ≈ −0.13 flat; row 1.97 about −0.17 to −0.21.
- Table II B = deviator's change τ = 2 vs τ = 1: +0.06..+0.13 for large cuts, negative for small cuts.
- Table III A = mean of (Q^π(dev) − Q^π(on-path))/Q^π(on-path): −0.02 to −0.04 everywhere.
- Table III B = fraction with deviation unprofitable: 0.82–1.00. Lows: 0.82 (1.62 row, dev 1.54) and 0.88 (1.97 row, dev 1.66).

**Table A4 (punishment length, appendix p. 12).** 4.3–7.1 periods. It rises with the pre-shock price: about 5.2 at 1.62, 5.9 at 1.74, 6.2 at 1.82, 6.5–6.9 at ≥1.85. It is insensitive to the size of the cut. The diagonal is 1.00, a no-op deviation.

**Graph of strategies (§IV.C; Figs 8, 9, A10–A13).**
- Session example: mean path length to the absorbing node 6, maximum 18.
- In 92% of sessions every one of the 225 nodes returns to the long-run prices. Fig A11 shows 920 sessions with 0 non-returning states.
- In 98% of sessions, fewer than 3 nodes don't return.
- Fig A12: median return path 5, rarely >10, over 225,000 trajectories.
- Fig A13: the 5 most central nodes hold >50% of betweenness.
- Figure 9 note: "Bertrand-Nash price best approximated by the third lowest price, the monopoly price by the third highest". **This is false by distance** (see §3).

**Appendix A3/A4 extras.**
- Fig A2 is a 3-D histogram of long-run price pairs. The modal pair is (idx 10, idx 10) = 1.78 (Fig A10 text).
- Fig A3: fraction of sessions with constant (cycle-1) prices across the grid (colorbar 0.3–1.0).
- Fig A8: average IR for a 5-period deviation to the Nash price (grid idx nearest p^N = idx 2, by `MINLOC` distance).
- Fig A9: fan charts.

#### (α, β) grid: Figures 1, 2, 7, A1, A3–A7

- **Code grid (all of `figure_1/A_InputParameters.txt`, 10,000 experiments):**
  - α = 0.0025, 0.0050, …, 0.25 (100 values, step 0.0025);
  - β_j = j × 2×10⁻⁷, j = 1..100 (input 0.005..0.5 per episode), i.e. 2×10⁻⁷ … 2×10⁻⁵.
- **All plotted heat maps drop α < 0.025** (`p = p[p$alpha >= 0.025, ]`), so the figures show 91 × 100 cells.
- Fig 1, Δ: colorbar 0.70–0.90. "Over our grid Δ ranges from 70 to 90%." Highest when α and β are both low; it "backfires" at the very lowest values.
- Fig 2, fraction of sessions with Nash on path: 0 to about 0.8, low at high β. At α = .15, β = .4e-5 about one half (0.505).
- Correlation of Fig 1 with Fig 2 (Pearson): 0.12. Using "at least one agent best-responds": 0.24.
- Q-loss on path (Fig A4): "most often <0.5%, never exceeds 1.2%" (colorbar to 0.012). Q-loss off path (Fig A5): up to 0.025.
- Fig 7: computed as p_nondev(τ=2)/p_nondev(τ=1), a **price ratio** (colorbar 0.84–0.96), not a "percentage reduction". Its correlation with Δ is 76.2%.
- Fig A1, iterations to convergence: from 0.4M at the largest β to several million. The −1/x color scale saturates above about 2.5M at the left edge. At α = 0.125, β = 1e-5 the mean is 850,000.
- Fig A6 (mutual BR, all states): 0.05–0.30. Fig A7 (≥1 agent BR): 0.15–0.45.

#### Discount factor: Figure 3 (D, exact from vector PDF; 113 points)

- δ grid: 0, 0.01, …, 0.99, then 0.991…0.999, then 0.9991…0.9995. The input has 114 experiments; the plotted line has 113 points ending at 0.9994.
- Δ values (δ: Δ):

  | δ | 0 | .1 | .2 | .3 | .34 (min) | .35 | .4 | .5 | .6 | .7 | .8 | .9 | .95 | .97 | .98 | .99 | .995 | .998 | .9994 |
  |---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
  | Δ | .212 | .207 | .178 | .160 | .156 | .165 | .173 | .215 | .278 | .372 | .500 | .710 | .849 | .892 | .916 | .934 | .942 | .943 | .936 |

- Claims: "at δ = 0.35 it has dropped to 16%"; Δ turns back up as δ → 0; "eventually starts decreasing with δ" near 1. The near-1 decline is tiny (0.943 to 0.936).

#### Memoryless check (Table A1)

k = 0, δ = 0, α = 0.25, **β = 10⁻³** (input 25/episode; the appendix text says 10⁻⁴). Converges in about 5,000 periods. Δ = 0.185, equilibrium play = 0.273, Q-gap = 0.013.

#### Time scale

- **Fig 10:** Δ vs iterations, 0–1.5M, moving average over the last 100. The run records 15,000 checkpoints × 100 iterations.
  - The dashed benchmark is the Δ that would result if non-exploring play were Nash-by-defect (idx 2) and exploring play uniform. The R script computes it from PI[2,2], the row/column means, and the overall mean.
  - Claim: Δ rises above this benchmark early.
- **Fig 11, rematch:**
  - Trained Q matrices are re-paired at random (init 'T': each agent draws a random session's Q). Exploration is off (β < 0).
  - Δ drops from 0.85 to about 0.20, then recovers within 40,000 periods ("less than one-tenth of the time"), ending somewhat lower.
  - Table A17: with exploration reactivated, Δ = 0.837.

#### Robustness (§V; α = 0.15, β = 4e-6 unless noted). Appendix tables, exact in the package.

Columns: Δ, eq-on-path, IR (nondeviator's τ = 2 relative cut), IC (fraction of deviations unprofitable), punishment length.

**A5, number of firms**

| n | Δ | eq on path | IR | IC | punishment length |
|---|---|---|---|---|---|
| 2 | .849 | .505 | −.127 | .936 | 5.705 |
| 3 | .643 | .184 | −.078 | .882 | 11.686 |
| 4 | .559 | **.717** | −.080 | .899 | 18.379 |

Text: 85 → 64 → 56%. Q has 3,375, ≈50,000, and >750,000 entries.

**A6, n = 3 with more exploration:** α = .05, β = 0.024e-5 (input 0.006) gives Δ = .750, eq .306, IR −.146, IC .908, length 7.548. Text says "close to 75%".

**Table IV, cost asymmetry (c1 = 1)**

| c2 | 1 | .875 | .75 | .625 | .5 | .25 |
|---|---|---|---|---|---|---|
| 2's Nash share | .500 | .545 | .588 | .627 | .662 | .722 |
| Δ | .849 | .841 | .812 | .781 | .759 | .713 |
| (π1/π1^N)/(π2/π2^N) | .997 | 1.050 | 1.121 | 1.193 | 1.265 | 1.442 |

**A7, demand asymmetry** (a2 = 2, 2.12, 2.25, 2.38, 2.50, 2.75)

| a2 | 2 | 2.12 | 2.25 | 2.38 | 2.50 | 2.75 |
|---|---|---|---|---|---|---|
| Δ | .85 | .84 | .81 | .78 | .76 | .71 |
| ratio | 1.00 | 1.05 | 1.12 | 1.19 | 1.27 | 1.44 |
| eq | .50 | .54 | .52 | .55 | .56 | .57 |
| IR | −.13 | −.15 | −.16 | −.17 | −.19 | −.21 |
| IC | .94 | .92 | .92 | .91 | .93 | .89 |

**A8, stochastic a0** (iid uniform on {−a0^H, 0, a0^H}, unobserved, not in the state)

| a0^H | 0 | .05 | .10 | .15 | .20 | .25 |
|---|---|---|---|---|---|---|
| Δ | .848 | .848 | .832 | .797 | .750 | .695 |
| eq | .509 | .485 | .462 | .433 | .407 | .337 |
| IC | .916 | .906 | .896 | .879 | .867 | .842 |

- Text: 85 → 80% at 0.15, 70% at 0.25.
- The grid is built from min p^N and max p^M across the three markets. Note that the a0^H = 0 row (.848) is not identical to the baseline (.849), because the RNG stream differs.

**A9, entry/exit, 3 agents** (outsider = agent 3; ρ = Pr(entry | out) = Pr(exit | in))

| ρ | Δ | eq | IR | IC | length |
|---|---|---|---|---|---|
| .001 | .583 | .048 | −.006 | .432 | 9.62 |
| .0001 | .566 | .070 | −.009 | .450 | 9.83 |

Text: "less than 60%", "eq <10%", "deviations profitable in a sizable fraction". The paper text describes running both 2↔1 and 3↔2 markets. **Only the 3-firm version exists in the package and appendix.**

**A10, μ (product substitutability)**

| μ | .50 | .45 | .40 | .35 | .30 | .25 | .20 | .15 | .10 | .05 | .01 | 0 |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| Δ | .856 | .860 | .859 | .853 | .859 | .849 | .839 | .835 | .829 | .827 | .805 | .774 |
| eq | .331 | .371 | .405 | .414 | .433 | .505 | .532 | .582 | .660 | .694 | .654 | .631 |

Text: at μ = 0, Δ ">77%".

**A11, Q-initialization**

| Init | Δ | eq | IR | IC | length |
|---|---|---|---|---|---|
| benchmark | .849 | .505 | −.127 | .936 | 5.705 |
| Nash | .731 | .647 | −.104 | .915 | 5.244 |
| Grim | .724 | .640 | −.104 | .925 | 5.350 |
| Random U(0,10) | .877 | .238 | −.113 | .853 | 5.604 |
| U Q = 5 | .793 | .621 | −.119 | .928 | 5.481 |
| U Q = 10, no exploration | **.978** | .525 | −.120 | .951 | 9.619 |

**A12, action set**

| Action set | Δ | eq | IR | IC | length |
|---|---|---|---|---|---|
| m = 15, ξ = .5 | .761 | .730 | −.125 | .947 | 4.694 |
| lowest price .99 | .612 | .871 | −.112 | .942 | 4.258 |
| m = 50 | .709 | **.154** | −.059 | .931 | 12.713 |
| m = 100 | .704 | **.684** | −.071 | .986 | 23.301 |

**A13, memory k = 2:** Δ .574, eq .371, IR −.030, IC .441, length 24.038.

**A14, linear demand** (Singh–Vives u = q1 + q2 − ½(q1² + q2²) − γq1q2; γ = .01….99)
- Δ is U-shaped: .856 at .01, .808 at .40, .770 at .50, .651 at .70, **.634 at .75** (minimum), .649 at .80, .677 at .85, .753 at .90, .854 at .95, .871 at .99.
- The appendix text gives the minimum as "63% when γ = 3/4" (a stacked fraction that pdftotext garbles), which matches the table.

**A15/Fig A15, Boltzmann exploration**
- T_t = 1000·λ1^t; grid over α and λ1 ∈ [0.999, 0.999999596].
- Single point α = .05, λ1 = .999961: Δ .856, eq .094, IR −.090, IC .841, length 6.554.
- Converges in under 400k iterations, against ">1.5M for the representative ε-greedy run".

**A16, asymmetric learning** (agent 2's α2, β2·10⁵)

| α2, β2·10⁵ | Δ | eq |
|---|---|---|
| .05, .4 | .821 | .304 |
| .30, .4 | .797 | .523 |
| .15, .2 | .771 | .423 |
| .15, .8 | .798 | .310 |

The slower updater gains more; the heavier explorer underperforms.

**Main-text summaries:**
- n = 3: 64%; n = 4: 56%.
- Demand shocks: 80%/70%.
- Entry/exit: <60%.
- μ = 0: >77%.
- Initialization: always >70%; Q̄ = 10 with no exploration is "almost perfect".
- Downward-extended grid: >60%. m = 100: 70%.

---

### 3. Ambiguities, discrepancies, unstated details

1. **Tie-breaking.** The paper (p. 3274) says "as a tie-breaking rule, they are instructed to choose the lowest price". **The code breaks argmax ties at random** (`MaxLocBreakTies`, separate RAN2 stream, tolerance EPSILON). The lowest-index rule applies only to the static BR and dynamic BR used in IR analysis (`MINVAL(MAXLOC(...))`). Proposed switch: `tie_break = {random (code default) | lowest_price (paper text)}`. Exact ties are rare after learning, but the initial uniform-constant inits (U 5, U 10) start fully tied.
2. **Iteration cap.** The paper says 10⁹ (p. 3276); the code uses 50,000 episodes × 25,000 = **1.25×10⁹**.
3. **Unconverged sessions are kept in all averages.** The paper says "nearly all sessions converged" and never gives a count; `A_res.txt numConv` has it but is not in the package.
4. **α grid.** The paper says "100 equally spaced points in [0.025, 0.25]" (p. 3275). The code uses 0.0025..0.25 step 0.0025 and plots only the 91 points ≥ 0.025.
5. **ν–β formula, fn 20 (p. 3276) as printed is wrong:** ν = (m−1)^n / (m^{kn(n+1)} [1 − e^{−β(n+1)}]).
   - At n = 2, m = 15, k = 1, β̄ = 2e-5 it gives ν = 0.29, not 4.
   - The text's numbers (ν ≈ 4 at β̄ = 2e-5, ≈ 20 at 4e-6, ≈ 450 at the lowest β = 2e-7) fit two candidates:
     - (m−1)^n/(m^{kn+n+1}(1−e^{−β(n+1)})): 4.30, 21.5, 430;
     - 1/(m^{kn+1}(1−e^{−β(n+1)})) (state uniform via joint exploration, then action uniform): 4.94, 24.7, 494.
   - Both also give ν ≈ 20 for k = 0 at β = 1e-3: 19.4 and 22.2.
   - Treat ν as descriptive only. β is the parameter.
6. **"14 percent" exploration claim (p. 3276).** With β = 10⁻⁵, e^{−βt} at t = 100,000 is e^{−1} = **36.8%**, not 14%. 14% ≈ e^{−2}, the chance that *both* agents explore. The text says "the probability of choosing an action randomly".
7. **Memoryless β.** The appendix A4.1 text says β = 10⁻⁴; the input uses 10⁻³ (25 per 25,000). Only 10⁻³ is consistent with "converges in 5,000 periods" (ε(5000) = 0.0067; at 10⁻⁴ it would be 0.61).
8. **True Q and the Nash claims.** The paper describes solving eq. (3), with max in the continuation, for the best response. The code evaluates Q^π (own continuation fixed at the limit strategy) and checks one-shot deviations.
   - "All states greedy w.r.t. Q^π" is a valid subgame-perfection test (policy improvement).
   - "Nash on path" as coded is a **one-shot-deviation check at path states only**. A multi-period deviation that exploits off-path suboptimality is not tested. So it is necessary but not sufficient for a Nash equilibrium of the repeated game.
   - The Q-loss is likewise the one-step improvement gap, not the gap to V*.
   - Proposed switch: `eq_check = {one_shot_Qpi (code) | full_best_response (value-iterate the MDP given the rival's limit strategy)}`. The second option is the stricter test the paper describes.
9. **Table I "Q-loss (all states)"** is actually `QGapNotOnPath` (off-path states only) in `table_I.R`. It is constant at 0.018 in every column.
10. **Approximations of p^N and p^M.**
    - Fig 9 note: "Nash best approximated by the third lowest, monopoly by the third highest." By distance, the nearest are idx 2 (|0.0064|) and idx 14 (|0.0064|). The appendix Fig A2 note says "between the second and third".
    - Fn 40 says the Nash init uses "the closest approximation". The appendix A5.6 text says the third lowest, "by excess". The code input uses **idx 2** (`F 2 2`). The grim init uses idx 14/2 (the closest), though the appendix text says "Bertrand by excess and monopoly by defect" (i.e. 3/13).
    - Deviation-to-Nash IRs use the nearest index (idx 2).
11. **Table A12 (ξ = 0.5) row uses a different Q init.** Its input file (`table_A10/A_InputParameters_2.txt`) has `F 3 3` instead of 'O'. That is an undocumented change confounded with ξ. Check before trusting Δ = .761.
12. **Impulse-response protocol (from code):**
    - Deviation at τ = 1 to agent i's static BR against the rival's limit action *in that state*. Ties go to the lowest index.
    - Then both agents follow their limit strategies. Every agent is used as deviator, and every state in the limit cycle is used as the pre-shock state.
    - Results are averaged over cycle states within a session, then over sessions, then over the deviating agent.
    - "Punishment length" = periods until the state first re-enters any pre-shock cycle state, counting τ = 1 (so a no-op deviation = 1). If the path enters a different cycle, the length is measured to the first repeated state.
    - "IC" = Q^π(s, dev) < Q^π(s, π(s)), i.e. one deviation then the limit strategies, δ = 0.95.
    - Figure 5 uses symmetric cycle-1 sessions at the modal price only. The paper's fn 33 says "sessions that converge to constant prices".
13. **Cycles.** Profit is the cycle average. Cycles are found by deterministic replay of greedy strategies. Strategy indices are written as integers, so this is reproducible.
14. **Internal inconsistencies to probe:**
    - Eq-on-path is n = 4 .717 vs n = 3 .184 (A5), and m = 100 .684 vs m = 50 .154 (A12). Both are non-monotone jumps.
    - IC = .936 at baseline vs the text's "more than 95%".
    - The text says "Δ from 70 to 90% over the grid", while Fig 3 points at δ = .99 reach .943. That is not a contradiction (different axis), but note the baseline is not the Δ maximum.
15. **Folder numbering.** The replication package's appendix R folders are numbered differently from the published appendix: folder table_A4 = published A6, A5 → A7, A6 → A8, …, A15 → A17. Map by contents, not names.
16. **Unstated in the paper but fixed in code:**
    - per-agent independent ε draws;
    - exploration over all m prices (the greedy action included);
    - Q update order agent 1 then 2;
    - initial state shared stream;
    - a period's state = last period's joint price indices;
    - for k ≥ 2, the state vector ordering is RESHAPE(TRANSPOSE(p)).
17. **Entry/exit implementation.**
    - The outsider is agent 3 with an extra "out" action (price index 16 → price 1000). numPrices = 16 for everyone, so states = 16³ including impossible ones.
    - When out, it enters with probability ρ. When in, it exits with probability ρ, and otherwise it explores or exploits over the 15 real prices.
    - Its argmax excludes the "out" action.

---

### 4. Claims, phrased as tests

- **C1, supracompetitive limit prices.** At the baseline (α = .15, β = 4e-6), mean Δ over 1,000 sessions ≈ 0.85 (Table I: .849, SD .112). Over the α ≥ .025 grid, Δ ∈ [0.70, 0.90].
  - Decision rule: reproduce if the session-mean Δ lies within ±0.02 of .849 (SE ≈ .0035), and the grid min and max fall in [.68, .92].
- **C2, not a failure to learn.**
  - (a) Δ is near competitive when collusion is infeasible. k = 0 with δ = 0 gives Δ ≈ .185, about 5 points above the 12% "Nash by excess" artifact. Δ(δ) bottoms out near .16 at δ ≈ .35.
  - (b) ≈ 50% of baseline sessions pass the on-path equilibrium check, and on-path Q-loss is ≈ 0.2%.
  - (c) Δ correlates positively, though weakly, with equilibrium play across the grid (r = .12).
  - Test both the code's one-shot Q^π check and a full best-response check (§3.8).
- **C3, reward–punishment.** After a forced one-period deviation to the static BR, the nondeviator cuts its price at τ = 2 (mean IR −.127; Fig 4 τ = 2 ≈ 1.551 from 1.795). The deviation is unprofitable in ≈ 94% of cases (IC .936; the text says >95%). The mean discounted loss is 3–4% (Table III A).
  - Punishment strength across the grid correlates with Δ (r = .762, Fig 7 ratio).
- **C4, finite punishment with a gradual return.**
  - Prices return to within 0.005 of the pre-deviation level by about τ = 9–10 (Fig 4). Mean punishment length is 5.7 periods (4.3–7.1 by cell).
  - The return is gradual (monotone climb from τ = 2), not grim trigger. The τ = 2 punishment stays well above p^N (≈1.55 vs 1.47).
- **C5, overshooting.** For small cuts (dev to idx 8, 1.699), both agents price *below* the deviation price at τ = 2 (≈1.59 < 1.70) (Fig 6, Table II B negative entries for small cuts).
- **C6, deviator participates.** At τ = 2 the deviator prices just above the punisher. It raises its price after big cuts and lowers it after small ones (Table II B).
- **C7, return from anywhere.** In ≥90% of sessions every one of the 225 states leads back to the limit cycle. The median return path is 5.
- **C8, robustness ordering.**
  - Δ(n): .85 > .64 > .56.
  - Δ falls with cost or demand asymmetry: .849 → .713 at c2 = .25. The split favors the less efficient firm: ratio 1.44.
  - Δ falls with demand noise (.695 at a0^H = .25), with entry/exit (≈.57), and with k = 2 (.574).
  - Δ is fairly flat in μ: .77–.86.
- **C9, pair-specific coordination.** Rematched trained agents (no exploration) fall to Δ ≈ .2, then relearn to below .85 in under 1/10 of the original time.
- **C10, early collusion.** In Fig 10, Δ exceeds the Nash-with-exploration benchmark early, well before convergence.

---

### 5. Computational cost

- Reported runtimes:
  - "the 'training' of our algorithms takes just a few seconds of CPU time in any session" (fn 41, p. 3293);
  - heat-map tasks take "up to a few days on a 40 cores workstation" and produce outputs of several GB (README §2).
- Reported convergence times:
  - 400k at the largest β, to several million at small β (A3.1, Fig A1);
  - 850k mean at α = .125, β = 1e-5 (p. 3276);
  - ">1,500,000 on average" at the representative point (appendix A5.10).
- Rule of thumb: convergence starts when ε ≈ e^{−6…−8.5}, so T_conv ≈ (6–8.5)/β, plus the 100k verification window.
  - Representative point: ≈ 1.5–2.2M iterations per session.
  - Representative experiment: 1,000 sessions ≈ 2×10⁹ iterations.
- One iteration is 4 RNG draws plus 2 Q-cell updates, with an occasional row argmax. The Q tables are 2 × 225 × 15 f64 = 54 KB, so they fit in L1/L2. Estimate 10–30 ns in Rust, so ≈ 20–60 s single-core for the whole 1,000-session baseline experiment.
- Full (α, β) grid, 100 × 100 × 1,000 = 10⁷ sessions:
  - Σ_β (8.5/β_j + 1e5) = 8.5 × 5×10⁶ × H₁₀₀ + 10⁷ ≈ 2.3×10⁸ iterations per (α, session).
  - Total ≈ 2.3×10¹³ iterations ≈ 2×10⁵ to 7×10⁵ core-seconds, i.e. 60–190 core-hours. That is a few hours on 32–64 cores.
  - The lowest-β column alone (β = 2e-7, T ≈ 35–43M per session) is about 19% of the total.
  - Only 91 α rows are plotted.
  - The equilibrium check, IR, and Q-loss post-processing cost 225 × 15 × 2 policy-evaluation walks per session (≤226 steps each), about 1.5M steps per session. That is comparable to training at high β, so it is not negligible for the grid figures (Figs 2 and 7).
- Robustness cells (about 50 experiments × 1,000 sessions at β = 4e-6) are cheap, except:
  - n = 4: 759k-cell Q, slower convergence;
  - k = 2 and m = 100: 50k–1M cells;
  - Boltzmann: exp per action.

### Files

- Notes: this file.
- Downloads: `papers/ai-coordination/calvano-calzolari-denicolo-pastorello-2020-aer-online-appendix.pdf` and `...-2020-aer-replication-package.zip` (14 MB; Fortran, R scripts, input files, table outputs, vector figure PDFs).
- Scratch checks: `scratchpad/calv_check.py` (benchmarks, grid, Q0, ν), `scratchpad/irdig.py` (vector-PDF digitizer).

---------------------------------------------------------------------------------------------------

## 2. Calvano 2020 follow-ups: Calvano et al. 2021 (IJIO) and Klein 2021 (RAND)

Sources (in `papers/ai-coordination/`):
- `calvano-calzolari-denicolo-pastorello-2021-ijio-algorithmic-collusion-imperfect-monitoring.pdf`. This is the Bologna IRIS **accepted manuscript** (dated "Janunary 2021", 21 ms pages after 2 cover pages). Page refs below are **manuscript pages** ("ms p.N"); PDF page = N+2. I could not compare it against the typeset IJIO 79:102712 version, but a postprint should match it.
- `klein-2021-rand-autonomous-algorithmic-collusion-sequential-pricing.pdf`, the publisher version (open access, CC BY-NC-ND), RAND J. Econ. 52(3):538–558. Page refs below are **journal pages** ("p.5NN"); PDF page = journal page − 536.
- NEW, downloaded: `klein-2018-tinbergen-dp18-056-autonomous-algorithmic-collusion-working-paper.pdf`, TI DP 18-056 revised Nov 2020. It is 49 pp. with the figures at the end. I grepped it for extra implementation detail (deviation price, ties, init, market-price definition) and found none. Its content is essentially the published text.
- NEW, downloaded: `ballestero-2026-ijio-sequential-pricing-stochastic-costs-replication-code.zip`. This is Julia code for Ballestero (2026, IJIO 106:103281), "Algorithmic collusion under sequential pricing and stochastic costs", a Klein follow-up. Its `qlearning_deterministic.jl` implements Klein's update exactly: `new = π[t-2,i] + δ π[t-1,i] + δ² max Q[i, a[t-1,j], :]`, with Q indexed [firm, rival price, own price]. Its other choices differ from Klein's:
  - Calvano-style exploration exp(−βt), β=4e-6.
  - α=0.15.
  - Calvano-style convergence: 1e5 stable steps, cap 1e7.
  - Ties broken by Julia `argmax`, which takes the first (lowest-price) maximum.
  - Initial state: `a[2,1]=a[1,1]`, so firm 1's price is held over into period 2.
  - Price grid `0:1/k:1`, so k+1 prices.
  It is the closest public code to Klein, but it is not his.
- NEW, downloaded: `martinez-msc-julia-calvano-2021-imperfect-monitoring-replication-code.zip`. This is a Master's-thesis Julia replication of Calvano 2021, and it has bugs:
  - Both agents share one `strategy_matrix` array.
  - Q tables are not reset between simulations.
  - Ties go to the first argmax, i.e., the *lowest* output, contrary to the paper's fn 17.
  - Q init = 0.
  It is useful only as a cross-check of the grid. Do not treat it as authoritative.
- **Klein's own code: not found.** I searched the web and GitHub repos/code ("Klein sequential Q-learning", "edgeworth cycles q-learning", "algorithmic collusion" repos, 40 listed). There is no public Klein repository. The paper has a pseudocode box (p.545) but no replication package. Calvano 2021 fn 22 (ms p.15): "code … can be obtained from the authors upon request." Unfound items to list for the user: Klein's simulation code, and Calvano et al.'s 2021 code and detailed robustness results (fn 22).
- An earlier, separate Klein version also exists: SSRN 3195812, "Assessing Autonomous Algorithmic Collusion: Q-Learning Under Short-Run Price Commitments" (2018/2019). Calvano 2021 cites that title as "Klein (2019)". I did not download it; it is likely superseded.

---

### Part A: Calvano, Calzolari, Denicolò & Pastorello (2021), "Algorithmic collusion with imperfect monitoring", IJIO 79:102712

#### A1. What differs from Calvano 2020

| Aspect | Calvano 2020 (AER) | Calvano 2021 (IJIO) |
|---|---|---|
| Stage game | Logit differentiated **Bertrand**; prices | **Cournot**, homogeneous good, quantities (Green & Porter 1984). ms p.6 |
| Demand | logit, deterministic | `(5) p_t = d_t − (q_1t + … + q_nt)`, d_t i.i.d. ms p.6 |
| Costs | c=1 | marginal cost 0, so π_it = p_t q_it. ms p.6 |
| Shock | none in the baseline | d ∈ {290, 310}, equiprobable, i.i.d. (E d = 300). ms p.8 |
| Timing | simultaneous | simultaneous: outputs are chosen **before** demand realizes. ms p.6 |
| State (baseline) | (p_1,t−1, p_2,t−1), both past prices | **s_t = p_{t−1}, the past market price only**, common to both firms. ms p.7 |
| State (benchmark "perfect monitoring") | n/a | s_t = {q_1,t−1, q_2,t−1}, the past outputs. ms p.7 |
| Mentioned alternative | — | s_it = {q_i,t−1, p_t−1} (firm-specific state). Discussed but **not used**. ms p.7, fn 14 |
| Actions | 15 prices | k=15 outputs {70, 72½, …, 102½, 105}, v=2½. ms p.8 |
| State count | 225 | 37 prices {80, 82½, …, 167½, 170}; Q is 15×37 = 555 cells. Perfect monitoring: 225 states, 3,375 cells. ms p.8 |
| Learning rule | eq. as in 2020 | same: `(3) Q_{t+1}(s,a) = (1−α)Q_t(s,a) + α[π_t + δ max_{a'} Q_t(s',a')]`. ms p.5 |
| Exploration | ε_t = e^{−βt} | same: `(4) ε_t = e^{−βt}`. ms p.6 |
| α, β, δ | 0.15, 4×10⁻⁶, 0.95 | **α=0.15, β=4×10⁻⁶, δ=0.95**, "same as Calvano 2020". ms p.8 |
| Q init | discounted payoff vs. uniformly randomizing rivals | same: "discounted payoff … if competitors randomized uniformly"; s_0 random. ms p.6 fn 12 |
| Ties | — | **"ties are broken by choosing the higher output"**. ms p.12 fn 17 |
| Convergence | greedy strategy unchanged for 100,000 consecutive periods; cap 10⁹ | same 100,000-period rule. ms p.8. **No cap stated.** |
| Runs | 1,000 sessions | 1,000 sessions per experiment. ms p.8 |
| Metric | Δ = (π̄ − π^N)/(π^M − π^N) | `(6) Δ ≡ (π̄ − π^C)/(π^M − π^C)`, π̄ = average per-firm profit upon convergence. ms p.9 |

Benchmarks at mean demand d=300 (ms p.6):
- Cournot: q^C=100 each, π^C=10,000.
- Collusive: q^M=75 each, π^M=11,250.
- Both outputs are on the grid. (100,100) is a stage Nash equilibrium on the grid: BR(100) = (300−100)/2 = 100.

Discretization rule (ms p.7):
- Actions: q^{j+1}−q^j = v.
- Demand: h equally spaced levels with spacing m·v for some integer m. The paper writes "d^{k+1}−d^k = mv", which reuses k; it is an index typo for d^{j+1}−d^j.
- Baseline: v=2½, d spacing 20, so **m=8**.
- Fn 15 measures how imperfect monitoring is as the fraction of price levels that do not reveal the rival's output (given own output): [(h−3)m + k(n−1) − n + 2] / [(h−1)m + k(n−1) − n + 2]. For h=n=2 this is (k−m)/(k+m) = 7/23 ≈ 0.30 ("roughly a third", ms p.8). I checked that the general form reduces to this.

Uncertainty does not change the equilibrium benchmarks, because outputs are chosen before d realizes and E d is constant (ms p.6).

#### A2. Results to match

| Item | Value | Ref |
|---|---|---|
| Baseline Δ (imperfect monitoring, stochastic) | "above 75%"; **76.25%** | ms p.10; Table I ms p.14 |
| Perfect monitoring, deterministic (d≡300) | **84.16%** (cf. 84.9% in Calvano 2020; fn 18) | Table I |
| Perfect monitoring, stochastic | **79.72%** | Table I |
| Imperfect monitoring (state = price), deterministic | **89.60%** | Table I |
| Diff-in-diff effect of imperfect monitoring | (76.25−89.60) − (79.72−84.16) = −13.35 + 4.44 = **−8.91 pp** (I checked the arithmetic) | ms p.15 |
| α×β grid, 100×100 "same grid as Calvano 2020" | Δ ranges **65% to over 80%**; Δ falls as β rises; low β takes longer; high β can cut convergence time to ~1/5 | ms p.15, fn 23 |
| n=3 | **72.7%** | ms p.16 |
| n=4 | **66.6%** | ms p.16 |
| 2-period memory (p_{t−1}, p_{t−2}) | **75.7%**, slower convergence | ms p.16 |
| 3-period memory | **64.5%**, much slower | ms p.16 |
| AR(1) demand, autocorrelation 0.9 | **86.7%** | ms p.16 |
| Rematching converged agents, ε=0 (learning continues) | Δ drops 75% → ~40%, then "exceeds 60% in a few thousand", reaching the new long run in 5–10% of the original convergence time | ms p.17, Fig 5 |
| Wider grid k=27, q ∈ 60..125, v=2½ | Δ "about 80%" | ms p.18 |
| h=5, d ∈ {250, 275, 300, 325, 350}, i.i.d., imperfect monitoring | **71.93%** ("72%") | ms p.18, Table II |
| h=5, perfect monitoring | **84.42%** ("84%") | Table II |
| DiD with h=5 | "about 17%". Computed from Table II: (71.93−89.60) − (84.42−84.16) = −17.93 | ms p.18 |
| Convergence | "always settled"; takes "hundreds of thousands" of periods | ms p.8–9 |

Figures to digitize (my readings from 130-dpi renders):
- **Fig 1** (ms p.9). Greedy output against iteration, 0–2,000,000, two series (one per firm, nearly coincident).
  - Starts at 105.
  - Drops to ~98 within ~50k.
  - Reaches 95 at ~200k and plateaus at ~94.2 from 300k to 500k.
  - Then declines: ~92 at 800k, ~90 at 1.0M, ~88.5 at 1.25M, ~87 at 1.5M, ~86 at 1.8–2.0M.
  - Reference lines: competitive 100, collusive 75.
- **Fig 2** (ms p.10). Δ of greedy outputs against iteration.
  - Starts at **−0.45**. This matches both firms greedily playing 105 at d̄=300: π=9,450, Δ=−0.44. That start confirms the Q-init: argmax of q(212.5−q) on the grid is 105.
  - ~0.15 at 30k, ~0.33 at 300k, plateau ~0.35 to 500k.
  - ~0.5 at 850k, ~0.6 at 1.1M, ~0.7 at 1.45M, ~0.75 at 2M.
- **Fig 3** (ms p.12). Average limit strategy, output against past price for p = 80..170 in steps of 2.5.
  - 80→92.8, 82.5→94.0, 85→94.9, 87.5→95.3, 90→95.7, 92.5→95.7, 95→96.0, 97.5→95.8, 100→95.5, 102.5→94.9, 105→94.8.
  - 107.5→93.9, 110→93.2, 112.5→92.7, 115–120→92.4, 122.5→92.1, 125→92.0, 127.5→91.1, 130–135→~90.9.
  - 137.5–140→90.4, 142.5–145→90.8, 147.5→90.6, 150→90.3, 152.5–162.5→~90.0–90.2, 165→89.7, 167.5→89.5, 170→88.8.
  - Also plotted: the firm-level demand lines q=(d−p)/2 for d=290 and 310.
- **Fig 4** (ms p.13). Forced-deviation impulse response, averaged over 1,000 sessions, with demand frozen.
  - High demand: pre-deviation output ≈86.1. At t=1 the deviator plays **105**. At t=2 both play ~92.5, at t=3 ~89.8, at t=4 ~88.8, and they settle at **~88.2 from t≈5 to t=20**.
  - Low demand: pre-deviation ≈85.2. The deviator plays **~101**. At t=2: nondeviator ~95, deviator ~94. Then 90.6, 89.2, and they settle at **~87.9**.
- **Fig 5** (ms p.17). Rematch Δ against iteration, 0–100k. Readings: ~0.41 at ~1k, 0.50 at ~3k, 0.55 at ~6k, 0.60 at ~10k, 0.65 at ~22k, 0.68 at ~42k, 0.69 at ~55k, 0.70 at 75k–100k.
- **Fig 6** (ms p.19). Average limit strategy for h=5, k=27; price axis 0–230 in steps of 2.5, U-shaped.
  - 103 at p=0, falling to ~91 by p≈12.
  - Flat ~90.8 from 12 to 45, small bump ~91.7 at 47.
  - Declines to ~87 by 75, then ~86–86.5 from 85 to 165, with a minimum ~85 at ~155.
  - Rises to ~88 at 195, ~89 at 210, ~92 at 220, ~102.5 at 230.

#### A3. Ambiguities and unstated details

1. **How π̄ "upon convergence" is measured** (ms p.9). With i.i.d. demand the converged greedy play is a Markov chain on prices, not a deterministic cycle as in Calvano 2020. The candidates are an expectation under its stationary distribution, a long simulated average with ε=0, or the average over the final window. Not stated. Recommend: compute the exact stationary distribution of the 37-state chain (it is small), and add a named switch for a simulated average.
2. **Iteration cap.** No maximum is stated (Calvano 2020 used 10⁹).
3. **What Figs 1–2 are.** Is it one session or the cross-session average? The smooth curves suggest an average. If so, the averaged greedy output is **still falling at 1.5–2M**, which means many sessions have not converged by 2M. That sits uneasily with "hundreds of thousands" (ms p.3, p.9). Check: report the distribution of convergence times.
4. **Q-init with stochastic demand.** "Discounted payoff … if competitors randomized uniformly" (fn 12). Presumably E_d E_{q_j}[π(q_i)]/(1−δ), the same for every state. The Fig 2 start (−0.45 = everyone greedy at 105) is consistent with this. For n>2, "competitors" plural means all rivals randomize uniformly.
5. **Tie-break.** Ties go to the higher output (fn 17). Note the first-argmax/lowest-index convention in common code is the opposite.
6. **What the state is.** The baseline state is p_{t−1} only; the firm does **not** condition on its own past output. Fn 15's "fully revealing" measure assumes own output is known, but the strategy cannot use it. Our switch: `state = price | (own_q, price) | (q1, q2)`.
7. **"Deterministic demand" benchmark.** Presumably d ≡ 300 (29 reachable prices, 115–185). Not stated explicitly. For "deterministic and imperfect", is the state set the 29 reachable prices or still the 37 grid? Unstated.
8. **Grid for n=3, 4.** With n=3 the Cournot benchmarks are q^C=75 and q^M=50, but **50 is outside [70,105]**. The paper does not give the grids, demand intercepts or m for n=3, 4.
9. **Memory 2/3.** The states are 37²=1,369 and 37³=50,653. Q-init and β are presumably unchanged; not stated.
10. **Correlated shocks.** "Coefficient of autocorrelation 0.9" for a two-point process presumably means a symmetric Markov chain with stay-probability 0.95. Not stated.
11. **Rematching** (ms p.16–17). Which pairs are formed (random permutation of 1,000 converged agents?), which Q's and starting state are kept, whether learning α continues (it seems so), and the horizon (100k plotted) are all unstated. "75%" is the pre-rematch Δ. The plotted curve starts at x≈1,000, so it may be a moving average.
12. **Forced deviation** (ms p.12–13). The size of the "output expansion" is not stated. The figure implies the static best response to the rival's output at the frozen demand: high demand gives (310−86)/2 = 112, capped to **105**; low demand gives (290−85)/2 ≈ 102.5, averaging ~101 across sessions. Both match the digitized deviations. Also unstated:
    - which firm deviates;
    - how the "converged outputs" starting point is chosen under frozen demand (the session's resting point after re-running greedy play with d fixed?);
    - whether there is any learning during the impulse response (presumably none).
13. **Grids for the k=27 and h=5 experiments.** Was β kept at 4×10⁻⁶ despite the bigger Q? For perfect monitoring with k=27 there are 729 states × 27 = 19,683 cells. Not stated. Which configuration gives the "about 80%" (k=27, h=2, imperfect monitoring?) is also unclear (ms p.18).
14. **Table II mixes grids.** Its deterministic column reuses the k=15 baseline numbers (84.16, 89.60), while the stochastic column uses k=27, h=5. So the "about 17%" DiD mixes grids, and it computes to 17.93, not 17.
15. **The α×β grid.** The 2021 paper only says "same grid". Calvano 2020 §A ("Parameter Grid") used 100 equally spaced α in [0.025, 0.25] and 100 equally spaced β in (0, 2×10⁻⁵]; I checked this against the 2020 PDF. Note that 2021's Q has 555 cells rather than 3,375, so the same β means more visits per cell (ms p.8 says this itself).

#### A4. Testable claims (literal default = baseline above)

- **C21-1.** In the baseline (n=2, k=15, h=2, d∈{290,310}, state=p_{t−1}, α=.15, β=4e-6, δ=.95), 1,000 sessions all converge under the 100k-stable rule, and mean Δ ≈ **0.76** (Table I).
- **C21-2.** The perfect-monitoring benchmark (state=(q1,q2)) has deterministic Δ≈0.84 and stochastic Δ≈0.80. Imperfect monitoring with deterministic demand has Δ≈0.90, which is **higher** than perfect monitoring. The DiD effect of imperfect monitoring is ≈ −9 pp.
- **C21-3.** The average limit strategy q(p_{t−1}) decreases in p over the visited range with slope |dq/dp| < ½. It is flatter than the firm-level demand, so punishments fade and the price recovers ("intensity as a clock"). Test both the average and the share of sessions where this holds individually. The paper says limit strategies vary a lot by session (ms p.11).
- **C21-4.** After a forced best-response deviation with frozen demand, both firms raise output (a price war), and the war is harsher under low demand than high demand (peak ~95 vs ~92.5). **The paper says play "gradually return[s] to … pre-deviation behavior", but its own Fig 4 plateaus ~2 units above the pre-deviation output through t=20 (88.2 vs 86.1; 87.9 vs 85.2).** Test the return to baseline explicitly.
- **C21-5.** Δ over the α×β grid lies in [0.65, 0.80+] and decreases with β.
- **C21-6.** n=3 gives Δ≈0.727 and n=4 gives Δ≈0.666 (once the grid is defined).
- **C21-7.** Memory 2 gives Δ≈0.757 and memory 3 gives Δ≈0.645, with longer convergence times.
- **C21-8.** AR(0.9) shocks give Δ≈0.867.
- **C21-9.** Rematched converged agents with ε=0: Δ falls to ≈0.40 and recovers to ≈0.70 within 100k periods, which is 5–10% of the original convergence time. The text says ">60% in a few thousand"; the figure reaches 0.60 at ~10k.
- **C21-10.** k=27 gives Δ≈0.80. h=5 gives imperfect Δ≈0.72 and perfect Δ≈0.84. The limit strategy is flat at high prices and decreasing at lower ones (ms p.18). The figure is actually U-shaped at both extremes, probably because unvisited states keep their init-argmax output.
- **C21-check.** Is the average strategy consistent with Δ? A fixed point of the Fig 3 average strategy against mean demand (q≈90.5 at p≈119) gives Δ≈0.6, not 0.76. Averaging strategies across heterogeneous sessions can explain the gap. Report per-session on-path outputs alongside the average strategy.

---

### Part B: Klein (2021), "Autonomous algorithmic collusion: Q-learning under sequential pricing", RAND 52(3):538–558

#### B1. What differs from Calvano 2020

| Aspect | Calvano 2020 | Klein 2021 |
|---|---|---|
| Timing | simultaneous | **Alternating moves (Maskin–Tirole 1988)**: firm 1 moves in odd periods and firm 2 in even periods. Prices stay fixed between own moves. p.543 |
| Demand | logit, differentiated | **Homogeneous linear Bertrand** (eq. 3, p.543): D_i = 1−p_i if p_i<p_j; 0.5(1−p_i) if p_i=p_j; 0 if p_i>p_j |
| Cost | c=1 | 0 (no marginal or fixed cost); π_i = p_i D_i (eq. 1, p.543) |
| Grid | 15 prices around [p^N, p^M] ± ξ | **P = {0, 1/k, 2/k, …, 1}** with "k equally sized intervals", **so k+1 prices** (baseline k=6 gives 7 prices). p.543 |
| Monopoly | — | p^C = 0.5, per-firm π = 0.125 (with sharing). p.543 |
| State | (p_i,t−1, p_j,t−1) | **s_t = p_{j,t−1}, the rival's current price only** (Markov assumption). Q is |P|×|S| = (k+1)². p.544 |
| Update | one-step Q-learning | **Two-period target** (eq. 5, p.545) |
| Exploration | ε=e^{−βt}, β=4e-6 | ε-greedy with **ε_t = (1−θ)^t**, θ chosen so that **ε_{0.5T} = 0.001 and ε_T = 10⁻⁶**, i.e. θ = 1 − 10^{−6/T}. That gives θ ≈ 2.76×10⁻⁵ for T=500k. p.547 |
| Ties | — | **random among argmax**. p.545 |
| Q init | discounted payoff vs. random rival | **all zeros** ("results not sensitive"). p.547 |
| Stopping | convergence rule (100k stable; cap 10⁹) | **fixed horizon T**; no convergence rule. Baseline T=500,000; Figs 1, 4 and 10 sweep T up to 500k. p.547 |
| α, δ | .15, .95 | **α=0.3, δ=0.95**. p.547 |
| Runs | 1,000 | **1,000 runs** (baseline). p.547 |
| Profit metric | Δ normalized at convergence | **Π_i** = average profit over the last 1,000 periods (eq. 7, p.546) |
| Equilibrium test | IR + best-response check | **Γ_i**, the optimality ratio (eq. 8, p.547). NE iff Γ_1=Γ_2=1 with tolerance 1e-5 |

The equations, quoted:
- (2) objective: max Σ_{s=0}^∞ δ^s π_i(p_{i,t+s}, p_{j,t+s}). p.543
- (4) MPE value condition: V_i(p_jt) = max_p [π_i(p, p_jt) + E_{p_j,t+1}[δ π_i(p, p_j,t+1) + δ² V_i(p_j,t+1)]]. p.544
- (5) Q_i(p_it, s_t) ← (1−α)·Q_i(p_it, s_t) + α·[π(p_it, s_t) + δ π(p_it, s_{t+1}) + δ² max_p Q_i(p, s_{t+1})], with α∈(0,1) and δ∈[0,1). p.545
- (6) p_it ~ U{P} with probability ε_t; = argmax_p Q_i(p, s_t) with probability 1−ε_t. p.545
- (7) Π_i = (1/1,000) Σ_{t=T−1,000}^{T} π_i(p_it, p_jt). p.546. As printed this sums 1,001 terms; treat it as the last 1,000 periods.
- (8) Γ_i(p_i, p_j) = Q_i(p_i, p_j) / max_p Q*_i(p, p_j). Q*_i is computed exactly "by keeping the competitor Q-function fixed and looping over all action-state pairs until Equation (5) converges". p.546–547
- Pseudocode (p.545):
  1. Set parameters; initialize Q.
  2. Initialize {p_1t, p_2t} for t ∈ {1,2} randomly; t=3, i=1, j=2.
  3. Loop over periods:
     - update Q_i(p_{i,t−2}, p_{j,t−2}) by (5);
     - set p_it by (6) and p_jt = p_{j,t−1};
     - t←t+1 and swap i, j;
  4. Until t=T.

  Read the update as: at time t, firm i updates the action it took at t−2. Its targets are the profit at t−2 against the rival's old price, plus δ times the profit at t−1 against the rival's new price, plus δ²·max Q at the new state p_{j,t−1}. Ballestero's code does exactly this.

Benchmarks:
- **Joint-profit benchmark 0.125.**
- **"Competitive" benchmark** (p.546): the most competitive Edgeworth-cycle MPE. Firms undercut by one increment until prices reach the lower bound; the first firm to see the lower bound resets to one increment above monopoly. Average per-firm per-period profit is **≈0.0611 for k=6, rising to ≈0.0833 as k→∞**.
  - I reproduced both numbers: the lower bound is p=0, the reset period earns 0, and the k=6 cycle has length 5.
  - **The same rule gives 0.0699 for k=12 and 0.0758 for k=24.** Klein's Figs 4, 8, 9 and 10 draw one benchmark line at ≈0.061 for every k.
- Static Nash: price at or one increment above MC; fn 13 (p.544) gives V_1(1/k) = (1+δ)/(1−δ²)·max_p π_1(p, 1/k). For k=6, one increment above MC is p=1/6, with per-firm shared profit (1/6)(5/6)/2 = **0.0694**, which is **above the 0.0611 "competitive" benchmark**.

#### B2. Results to match

All at k=6, α=.3, δ=.95, T=500k, 1,000 runs unless noted.
- **Fig 1** (p.548). Sweep over T: x ≈ {~1k?, 20k, 60k, 100k, …, 500k} in steps of 40k.
  - Average Π: ~0.090 at the first point, ~0.100 at 20k, ~0.105 at 60k, then a plateau rising slowly to **~0.108–0.110** at 500k.
  - Average Γ: ~0.23, then 0.88 at 20k, 0.91, 0.93, …, **~0.97** at 500k (text: "around 97%").
  - NE share: 0%, 20%, 33%, 42%, 46%, 54%, 56%, 57%, 61%, 61%, 65%, 66%, 66%, **67%** (text: "around 67%").
- **Fig 2** (p.549). Joint histogram of (Π_1, Π_2), 0.01-wide bins.
  - All runs: a dark cell at (0.111, 0.111) (~400+), a cell at (0.125, 0.125) (~250–300), and off-diagonal cells at ≈(0.05, 0.14) and (0.14, 0.05) (~100 each).
  - Text: **230 asymmetric runs**; **667 NE runs, of which 241 are at joint-profit max (0.125, 0.125)**, so ~426 NE runs sit at 0.111.
- **Fig 3** (p.549). Forced deviation by firm 1, averaged over the 241 JPM-NE runs.
  - Market price: 0.50 for t=−5…−1; **0.33 at t=0**, 0.16 at t=1, 0.06 at t=2, 0.03 at t=3, 0.30 at t=4, and **0.50 from t≥5**.
  - "Two-period profit": firm 1 (deviator) is 0.125, then **0.175 at t=0**, ~0.015 at t=2, ~0.05 at t=4, 0.11 at t=6, 0.125 at t≥8.
  - Firm 2: 0.125, then 0.065 at t=1, 0.02 at t=3, 0.097 at t=5, 0.122 at t=7, 0.125.
- **Fig 4** (p.550). k ∈ {6, 12, 24}, sweep over T.
  - Π at 500k ≈ 0.108 (k=6), ~0.101 (k=12), ~0.104 (k=24). All are above 0.061 and "remain[s] above competitive".
  - Γ at 500k ≈ 0.97 / ~0.89 / **~0.82**.
  - NE share at 500k ≈ 67% / **~20%** / **~0–1%** ("nearly no Nash equilibria when k=24", p.550).
- **Fig 5** (p.551), k=24.
  - Joint Π: clusters at ≈(0.09–0.10, 0.12–0.13) and its mirror (0.12–0.13, 0.08–0.09). One firm is near or above 0.125, the other lower; there are none at (0.125, 0.125).
  - Joint Γ: one firm at 1.0, the other ~0.6–0.95. The higher-profit firm is the one with Γ=1.
- **Fig 6** (p.551), k=24. Histogram of market-price changes over the last 100 periods of all runs, with bin width 1/24.
  - ≈45% at −1/24.
  - ≈16% at 0.
  - ≈13% at −2/24.
  - ~5%, 3%, 2.5%, 1% at −3 to −6 /24.
  - Positive jumps spread over +0.2…+0.55, each bin ≤2.5% and totalling ~12%.
- **Fig 7** (p.552), k=24. Market price over the last 40 periods of runs 1–3, all deterministic sawtooth cycles.
  - Run 1: cycles between ~0.17 and 0.625, period ≈8.
  - Run 2: between ~0.29 and 0.625, period ≈8.
  - Run 3: between ~0.29 and 0.75, period ≈9–10.
  - In every case the same firm always resets the price.
- **Fig 8** (p.553). α ∈ {0, 0.1, …, 1.0}, T ∈ {100k, 500k}.
  - NE share at 500k: 25, 57, 67, 67, 65, 53, 46, 34, 26, 18, 21%.
  - NE share at 100k: 11, 26, 37, 42, 40, 39, 33, 28, 21, 18, 14%.
  - Π is an inverted U, peaking at ~0.107–0.11 for α .2–.4 and ~0.09 at α=1.
  - Γ ≈ 0.9–0.97, dipping at the ends.
- **Fig 9** (p.554). δ from 0.60 to 0.98 in steps of 0.02, plus points near 1.
  - Π ≈ 0.070 for δ ≤ 0.68, then ~0.075 (0.70–0.76), 0.083 (0.78), 0.090 (0.80), 0.095 (0.82), 0.098 (0.84), 0.100 (0.86), 0.102 (0.88), 0.104 (0.90), 0.106 (0.92), 0.108 (0.94), 0.111 (0.96), 0.115 (0.98), ~0.117 (≈0.99), and a collapse to **0.082 at δ≈1**.
  - Γ ≈ 1.0, decaying to 0.96 and falling to **0.72 at δ≈1**.
  - NE share: 100% to 0.74, then 98 (0.76), 92 (0.78), 91 (0.80), **48 (0.82)**, 45, 46, 49, 52, 59, 62, 68, 70 (0.98), then ~69, 65, 62 near 0.99, and **0% at δ≈1**.
- **Fig 10** (p.555). Self-reactive state (p_i,t−1, p_j,t−1) against baseline, sweep over T.
  - Π at 500k ≈ 0.113 vs 0.108.
  - Γ ≈ 0.94 vs 0.97, and starts lower (0.65 vs 0.88 at 20k).
  - NE share **~42% vs 67%**.
- Not shown in the paper: asymmetric α gives similar results (p.552), and so do other α at different δ (p.552).

#### B3. Ambiguities and unstated details

1. **Grid size.** "k equally sized intervals" on [0,1] means **k+1 prices**, including 0 and 1. A "k=6 price levels" reading would be wrong. Keep k as intervals; Ballestero uses `0:1/k:1`.
2. **ε_t indexing.** Does t count global periods (both firms) or own moves? It is presumably global, since T is "total learning periods" and the pseudocode runs t to T. θ = 1−10^{−6/T} is derived from ε_{T/2}=1e-3 and ε_T=1e-6; the two conditions are consistent.
3. **Initialization** (pseudocode line 2). "Initialize {p_1t, p_2t} for t={1,2} randomly" could draw p_1,2 independently of p_1,1, which would break alternation. Ballestero holds p_1,2=p_1,1. Neither run updates Q until t=3.
4. **Profit flow.** Both firms earn profit every period (eq. 1). Π is averaged over all periods, not just own-move periods.
5. **What "market price" means** in Figs 3, 6 and 7. It is undefined. The data are consistent with **min(p_1, p_2)**, the transaction price:
   - in Fig 7, the low price persists for one extra period at each reset;
   - in Fig 3, the market price at t=0 (0.33) equals the deviator's price.
   Treat "market price = min" as an inferred switch.
6. **Forced-deviation price.** Not stated. Fig 3 (market price 0.33 at t=0, from 0.5) implies the **myopic best response 2/6** (π=0.222 vs 0.139 at 1/6). Also unstated: whether learning and exploration are frozen during the test (presumably ε≈0 at T and no updates).
7. **"Two-period profit"** (Fig 3). Undefined. The numbers fit **(π_{t−1}+π_t)/2**: at t=0, (0.125 + 0.222)/2 = 0.174 ≈ 0.175. The forward-looking alternatives do not fit. Each firm's value is plotted at its own move periods.
8. **Γ_i evaluation point** (eq. 8). It is evaluated "at the prevailing p_i and p_j at the end of the simulation". For cycling runs this depends on where the last period falls in the cycle. Q* is the best response to the rival's **greedy** policy derived from its frozen Q, and the rival's tie rule is unspecified. Only the current state is checked, not all on-path states.
9. **NE tolerance** is 1e-5 (p.547), applied to |Γ−1|.
10. **"Collusive equilibrium" = NE ∧ Π > competitive benchmark** (p.547). A run converging to static Nash at p=1/6 earns 0.0694 > 0.0611, so **by this definition it would count as a "collusive equilibrium"**. At δ≤0.76, Fig 9 shows ~100% NE at Π≈0.07, i.e. static Nash ("coordinate on a static Nash equilibrium outcome", p.552). Report the price level as well.
11. **Fig 2's 0.111 cell is ambiguous.** Shared profit at p=1/3 and p=2/3 is identical (both 1/9 per firm), so the ~426 non-JPM NE runs could be focal prices **below or above** monopoly. Record the converged prices.
12. **Benchmark line for k=12 and k=24.** By Klein's own definition it should be 0.0699 and 0.0758, but the figures appear to draw it at ~0.061. The conclusion survives (Π≈0.10), but the margin shrinks.
13. **Runs per point** in the sweeps (Figs 1, 4, 8, 9, 10) are not stated; presumably 1,000 each, with a fresh schedule θ(T) per T.
14. **α=0, α=1 and δ≈1** are plotted although the paper defines α∈(0,1) and δ∈[0,1). The actual endpoint values are unstated; α=0 must be some small positive value.
15. **The seed/run mapping** for "first three runs" (Fig 7) is unreproducible. Only the qualitative pattern can be matched.
16. **Self-reactive state.** (p_i,t−1, p_j,t−1), where p_i,t−1 is own current price. The Q shape becomes (k+1)×(k+1)². The update form is presumably unchanged.

#### B4. Testable claims

- **K-1.** At k=6, α=.3, δ=.95, T=500k, 1,000 runs, Q₀=0, ε_t=(1−θ)^t: mean Π ≈ **0.108**, mean Γ ≈ **0.97**, NE share ≈ **67%**. JPM-NE runs ≈ **24%**, asymmetric-profit runs ≈ **23%**.
- **K-2.** NE runs have a constant final market price (a focal price). Non-NE runs show sawtooth patterns: gradual decline, then a jump.
- **K-3.** In the JPM-NE runs, a forced best-response deviation to 1/3 is unprofitable for the deviator. Prices spiral down over ~3 periods, then a jump restores 0.5 within ~5 periods. Individual runs jump back abruptly, and the gradual average recovery is a cross-run averaging artifact (p.548–549).
- **K-4.** On grid size: at **k=6** the algorithms often reach a **fixed collusive (focal) price** (NE ≈67%). At **k=12 and k=24**, NE falls to ≈20% and then ≈0%, Γ falls (≈0.89, ≈0.82), and play converges to **deterministic asymmetric Edgeworth cycles** in which one firm always resets. **Mean profitability barely changes with k** (≈0.10–0.11 for all three) and stays above the competitive benchmark, including the corrected 0.070 and 0.076.
- **K-5.** At k=24 the distribution of market-price changes is dominated by −1/24 steps (~45%) with rare large jumps of +0.2…+0.55.
- **K-6.** Performance is inverted-U in α: NE share peaks at ≈67% for α∈[0.2, 0.4] and is ≈20–25% at the extremes. T=100k gives lower NE shares at every α.
- **K-7.** For low δ (≤0.76) the runs reach static Nash (Π≈0.07, NE≈100%). Π rises monotonically with δ up to ~0.117 at δ≈0.99. The NE share drops sharply between δ=0.80 and 0.82 (91%→48%), then recovers to ~70% at 0.96–0.98. Everything collapses as δ→1.
- **K-8.** Adding own past price to the state raises Π slightly (≈+0.005) but lowers Γ and the NE share (≈42% vs 67%), and slows learning.
- **K-9.** The results are insensitive to Q-initialization (asserted, not shown). Test Q₀=0 against Calvano-style Q₀.
- **K-10 (bridge to Calvano 2020).** Klein's policy claim (p.556) is that simultaneous moves without history conditioning would avoid collusion. That is directly testable in our base by switching timing to simultaneous with state = rival's last price only.

#### Suggested switches for our Rust base

- `timing = simultaneous | alternating`.
- `stage = logit_bertrand | homogeneous_bertrand_linear | cournot_linear`.
- `state = both_prices | rival_price | market_price | own_q_and_price | both_q`, plus `memory = 1..3`.
- `update = one_step | two_period(δ, δ²)`.
- `explore = exp_beta(β) | geometric_to_target(T, ε_mid=1e-3, ε_T=1e-6)`.
- `stop = stable_100k(cap) | fixed_T`.
- `ties = random | higher_action | first_index`.
- `q_init = zeros | uniform_rival_payoff`.
- `demand_shock = none | iid_h_levels | markov(ρ)`.
- `market_price = min | mean`, Klein's figures only.

---------------------------------------------------------------------------------------------------

## 3. Critiques of and foundations for Calvano et al. (2020): reading notes

Files are in `papers/ai-coordination/`:

- AFP21: `asker-fershtman-pakes-2021-nber-w28535-ai-and-pricing-algorithm-design.pdf` (NBER w28535, 54 pp; page refs use the printed page numbers)
- AFP22: `asker-fershtman-pakes-2022-aea-pp-ai-algorithm-design-and-pricing.pdf` (AEA P&P draft, 5 pp, printed pp. 1–5)
- WK06: `waltman-kaymak-2006-erim-theoretical-analysis-cooperative-behavior-multi-agent-q-learning.pdf` (ERIM ERS-2006-006-LIS; page refs use the report's own pp. 1–20. It was published as Waltman & Kaymak 2007, IEEE ADPRL pp. 84–91)
- WK08: `waltman-kaymak-2008-jedc-q-learning-agents-cournot-oligopoly.pdf`. **NEW, fetched via sci-hub**, DOI 10.1016/j.jedc.2008.01.003, JEDC 32(10):3275–3293 (journal pages)
- SC96: `sandholm-crites-1996-biosystems-multiagent-rl-iterated-prisoners-dilemma.pdf` (preprint, printed pp. 1–39; journal Biosystems 37:147–166)

For comparison, the Calvano 2020 baseline (AER 110(10), pp. 3274–3275): logit demand (eq. 5), n=2, c=1, a−c=1, a0=0, μ=1/4, δ=0.95, m=15 prices on [pN − ξ(pM−pN), pM + ξ(pM−pN)] with ξ=0.1, memory k=1 (state = last period's price pair), ε-greedy with ε_t = e^{−βt}, baseline α=0.15 and β=4×10⁻⁶. Updates are asynchronous (only the Q(s,a) actually played). Ties go to the lowest price. Convergence means the greedy policy is unchanged for 100,000 periods.

**Notation clash to fix in code.** In AFP, **β is the discount factor** and α is the learning rate. In Calvano, β is the exploration decay and δ is the discount. In WK08, **b is the Boltzmann temperature** and γ is the discount. In WK06, T is the temperature and ε is the ε-greedy rate. In SC96, t is the temperature and γ is the discount (the extraction garbles these). Use our own names, e.g. `discount`, `explore_decay`, `temperature`.

---

### 1. Asker, Fershtman & Pakes (2021 NBER w28535; 2022 AEA P&P)

#### 1.1 Model, and what differs from Calvano

**Stage game.** Two firms (N varied up to 10) sell a **homogeneous** good under Bertrand competition (AFP21 §2.1 p. 8; AFP22 §I p. 1). This is not logit.

- Demand to firm i (AFP21 eq. 1, p. 8): d_i = D(p_i) if p_i < p_j; D(p_i)/2 if p_i = p_j; 0 otherwise.
- D(p) = 1 for p ≤ V = 10 and 0 above (AFP21 p. 16: "equal to one for any price below or equal to ten"; AFP22 p. 1: "1 if p_i < p_j and p_i ≤ 10").
- Marginal cost c = 2.
- **Price grid: 100 points equally spaced on [0.01, 10]** (AFP21 p. 16; AFP22 p. 1; code A.1 `pL=0.01; pH=theta; dim=100; linspace`). The step is 0.10091.
- Static Nash equilibria: p = 2.0282 and p = 2.1291 (AFP22 p. 1; "2.03 / 2.13", AFP21 p. 16). I checked these against the grid. For N ≥ 5 only 2.03 is Nash, since sharing 0.1291/N falls below undercutting's 0.0282 (fn. 28, p. 26).
- Monopoly price: 10. Monopoly profit is 4 per firm when they share the market.

**Learner.** Each firm keeps a value table W_i(p | s_i).

- Baseline: the state is a **singleton** (memoryless) and the **discount β = 0** (myopic) (AFP21 p. 14 "Baseline Parameterization"; AFP22 §II).
- Action choice is **purely greedy**: p = argmax W, with **no ε-exploration** in the baseline (AFP21 p. 16; AFP22 item (2)). Exploration comes **only from optimistic initial values**: W⁰(p) ~ i.i.d. U[10, 20], above any feasible profit (max 8) (AFP21 p. 16; AFP22 p. 2).
- Learning rate α = 0.1, constant.
- General update (AFP21 eq. 2, p. 13): W^{k+1}(p|s^k) = α(k)[π_i(s^k, p) + β max_y W^k(y | s^{k+1})] + (1−α(k)) W^k(p|s^k).
- With β = 0 and a singleton state this becomes AFP21 eq. 3: **W^{k+1}(p) = α π(p, p_j^k) + (1−α) W^k(p).**

**The key definitions** (AFP21 §2.2 pp. 13–14; AFP22 §II pp. 1–2):

- **Asynchronous updating** is standard tabular Q-learning, and it is exactly what Calvano uses. Only W(p_i^k | s_i^k), the value of the price actually played, is updated, using realized profit (AFP22 (i): "Only W_i^k(p_i^k) is updated … W_i^{k+1}(p) = W_i^k(p) for all p ≠ p_i^k"). It needs only your own price and your own profit.
- **Synchronous ("perfect synchronous") updating**: at the current state, **every** p ∈ P is updated with the counterfactual profit π_i(p, p_j^k) it *would* have earned against the rival's realized price (AFP22 (ii): W^{k+1}(p) = α π^e(p, p_j^k) + (1−α) W^k(p) for all p). This requires observing p_j and knowing the demand and cost functions.
- **Synchronous using downward-sloping demand** (an intermediate case). The firm knows only its own cost, price and quantity q* = d_i(p*, p_j), plus the fact that residual demand weakly falls with own price (AFP21 eq. 4, p. 30; AFP22 p. 2):
  - p = p*: W ← α(p*−c) q* + (1−α) W(p*)  (the printed eq. 4 has the typo "(p−c)")
  - p > p*: W ← α · min((p−c) q*, W(p)) + (1−α) W(p). This is an upper bound, so W can only be revised **down**.
  - p < p*: W ← α · max((p−c) q*, W(p)) + (1−α) W(p). This is a lower bound, so W can only be revised **up**.
  - AFP22 states the same rule as: "if p > p_i^k and W^k(p) > (p−c)q … π^e = (p−c)q … if neither condition is met W^{k+1}(p) = W^k(p)."

**Rest point vs. convergence** (AFP21 Defs. 1–2, p. 11). *Convergence* means policies are unchanged for all s from k* on. A *rest point* means the prices themselves are constant from k* on. The paper's numbers are rest-point prices. Calvano instead measures 100k periods of policy stability under continuing (tiny) ε.

**Extensions in AFP21**

- **ε-experimentation** (AFP21 p. 28): the experimentation probability at iteration k is **k^{−1/θ}** (the formula in the notes to Table 2 is garbled; it is recovered from the "Pr at 10,000" column, where 10,000^{−1/θ} = 0.0001, 0.01, 0.0464, … for θ = 1, 2, 3). An experiment draws a uniformly random action. Experimentation runs for 10,000 iterations, then stops, and the algorithm runs greedily until a rest point. Note the schedule is **polynomial**, whereas Calvano's is exponential.
- **β > 0 with memory** (Fig. 5, p. 32): β = 0.95, state = both firms' prices last period (memory-1, like Calvano), **25-point grid** on [0.01, 10] (the note says 0.1, but Fig. 6's tick labels 2.09, 2.51, … 10.00 imply 0.01 + 0.41625k), W⁰ ~ U[200, 210], α = 0.1, greedy. Static Nash prices are 2.09 and 2.51; the text says "2.52" (p. 32).
- **Logit appendix A.2** (p. 46): Q_i = e^{a − b p_i} / (1 + Σ_j e^{a − b p_j}) with a = 40, b = 4. Everything else is unchanged (c = 2, the 100-point [0.01, 10] grid, U[10, 20] initial values, β = 0, singleton state). **These are not Calvano's logit parameters.**
- **Cournot appendix A.4** (p. 47): P = a − Q with a = 10, c = 2 (inherited), 150 quantities on [1.51, 3], W⁰ ~ U[25, 35].
- **Mixed pairing A.5**: one firm updates synchronously and the other asynchronously.

**Differences from Calvano, summarized**

| | Calvano 2020 | AFP baseline |
|---|---|---|
| Demand | logit, μ = 1/4 | homogeneous Bertrand, unit demand at p ≤ 10 (logit only in an appendix, with different parameters) |
| Grid | m = 15 | 100 points |
| Memory and discount | k = 1, δ = 0.95 | singleton state, β = 0 |
| Exploration | ε-greedy, exp(−βt) | none; optimistic initial values U[10, 20] |
| Initial Q | expected payoff against a uniformly randomizing rival | optimistic |
| Learning rate | α = 0.15 | α = 0.1 |
| Stopping rule | policy stable for 100k periods | rest point |

**Asynchronous updating is Calvano's update rule.** AFP's only new ingredient is the synchronous/counterfactual update (plus the downward-demand bound).

#### 1.2 Results to match

**AFP22 Fig. 1** (100 runs each; firm 1's price quantiles; notes on p. 3):

- (a) Asynchronous, after 5000 periods: min / 25th / median / 75th / max = **5.06 / 7.33 / 8.34 / 9.14 / 10**. Nothing changes after 4600 periods.
- (b) Perfect synchronous, after 500 periods: all **2.1291**, settled after about 105 iterations.
- (c) Synchronous with downward demand, after 500 periods: **2.0282 / 2.129 / 2.1291 / 2.23 / 3.84**.
- (d) A single asynchronous run converges to **8.89 at period 1,645**. The first upward revision comes at k = 1,119, and the firms match prices 22 times before converging.

**AFP21 Fig. 2 / Table 1** (Table 1: 1,000 runs; capped at 40,000 iterations; p. 27):

| Algorithm | N | min | 25th | median | 75th | max | % > static Nash | time to converge |
|---|---|---|---|---|---|---|---|---|
| Sync | 1 | 10 | 10 | 10 | 10 | 10 | – | 50 |
| Sync | 2 | 2.03 | 2.13 | 2.13 | 2.13 | 2.13 | 0 | 115 |
| Sync | 3 | 2.03 | 2.13 | 2.13 | 2.13 | 2.13 | 0 | 70 |
| Sync | 4 | 2.03 | 2.13 | 2.13 | 2.13 | 2.13 | 0 | 75 |
| Sync | 5 | 2.03 | 2.03 | 2.03 | 2.03 | 2.03 | 0 | 85 |
| Sync | 10 | 2.03 | 2.03 | 2.03 | 2.03 | 2.03 | 0 | 90 |
| Async | 1 | 10 | 10 | 10 | 10 | 10 | – | 1,235 |
| Async | 2 | 4.65 | 7.28 | 8.39 | 9.29 | 10 | 100 | 4,550 |
| Async | 3 | 2.13 | 2.13 | 2.84 | 3.95 | 10 | 71.5 | 8,530 |
| Async | 4 | 2.03 | 2.13 | 2.13 | 2.13 | 7.38 | 19.9 | 10,395 |
| Async | 5 | 2.03 | 2.03 | 2.03 | 2.03 | 3.34 | 12.1 | 10,395 |
| Async | 10 | 2.03 | 2.03 | 2.03 | 2.03 | 2.23 | 0.1 | 10,025 |

Footnote 30 gives N = 6 at 2.1% above Nash. The logit version is far stickier: 71.9 / 66.6 / 60.6% above Nash for N = 4 / 5 / 6.

**AFP21 Fig. 3** (p. 19): a synchronous example reaches 2.13 by period 70. The asynchronous example is the 8.89 run from AFP22 (d).

**AFP21 Table 2**, experimentation (async, N = 2, 100 runs; p. 29). Prices are min / 25th / median / 75th / max.

| θ | Pr at k = 10⁴ | min # plays of any action | prices |
|---|---|---|---|
| 1 | .0001 | 14 | 5.36 / 7.07 / 8.03 / 9.24 / 10 |
| 2 | .01 | 17 | 3.04 / 6.42 / 7.68 / 8.89 / 10 |
| 3 | .0464 | 20 | 3.04 / 5.46 / 7.02 / 8.54 / 10 |
| 4 | .1 | 24 | 2.73 / 4.25 / 6.27 / 7.98 / 10 |
| 5 | .1585 | 29 | 2.53 / 3.84 / 5.56 / 7.68 / 10 |
| 6 | .2154 | 33 | 2.53 / 3.54 / 5.01 / 7.28 / 10 |
| 7 | .2683 | 37 | 2.13 / 3.44 / 4.65 / 6.87 / 10 |
| 8 | .3162 | 42 | 2.13 / 3.34 / 4.40 / 6.57 / 10 |
| 9 | .3594 | 45 | 2.13 / 3.34 / 4.20 / 6.37 / 10 |
| 10 | .3981 | 48 | 2.13 / 3.34 / 4.15 / 6.06 / 10 |

The claim: experimentation lowers prices but does not remove the elevation; the max stays at 10.

**AFP21 Fig. 4**, async with downward demand: min 2.03, **max 2.23**, at least 10× faster (p. 31).

**AFP21 Fig. 5** (β = 0.95, memory-1, 25-point grid, p. 32):

- Synchronous median **6.25**; stabilizes after about 750k iterations.
- Asynchronous median **10**; stabilizes after about 4M iterations (fn. 35).

**AFP21 Fig. 6**, async rest-point price distributions (100 runs each; p. 33):

- static (singleton state, β = 0): mean / median 8.33 / 8.34
- memory-1 with β = 0: **8.73 / 8.75**
- memory-1 with β = 0.95: **9.83 / 10**

The claim: most of the elevation needs no punishment or history.

**AFP21 Table 3** (β = 0.95, 100 runs; p. 34). Columns are min / 25th / median / 75th / max.

| Algorithm | Treatment | prices |
|---|---|---|
| Sync | N = 2 | 5.01 / 5.84 / 6.25 / 6.25 / 8.34 |
| Sync | N = 2, θ = 4 | 4.17 / 4.59 / 4.59 / 5.01 / 6.67 |
| Sync | N = 3 | 2.51 / 3.34 / 3.34 / 3.34 / 5.84 |
| Async | N = 2 | 8.34 / 9.58 / 10 / 10 / 10 |
| Async | N = 2, θ = 4 | 6.25 / 9.17 / 9.58 / 10 / 10 |
| Async | N = 3 | 5.42 / 8.75 / 9.17 / 9.58 / 10 |
| Async | N = 2, downward demand | 6.25 / 7.09 / 7.50 / 8.34 / 10 |

**AFP21 Table 4**, logit appendix (p. 46). Columns are min / 25th / median / 75th / max.

| Algorithm | N | prices |
|---|---|---|
| Sync | 2 | 2.53 everywhere |
| Sync | 3 | 2.33 / 2.43 / 2.43 / 2.43 / 2.43 |
| Sync | 5 | 2.33 everywhere |
| Sync | 10 | 2.23 / 2.23 / 2.33 / 2.33 / 2.33 |
| Async | 2 | 6.17 / 8.13 / 8.59 / 9.19 / 10 |
| Async | 3 | 2.63 / 4.95 / 6.37 / 7.93 / 9.80 |
| Async | 5 | 2.23 / 2.43 / 2.43 / 2.63 / 9.90 |
| Async | 10 | 2.13 / 2.33 / 2.43 / 2.43 / 2.73 |
| Sync, downward demand | 2 | 2.53 everywhere |
| Async, downward demand | 2 | 2.33 / 2.53 / 2.73 / 3.04 / 7.88 |

At N = 2 the logit runs take 2,910 iterations (async) and 100 (sync) (fn. 43).

**AFP21 Table 5**, Cournot (reported as prices): sync 4.67 for all runs, which is the Cournot-Nash price (10 − 2·8/3). Async min / 25th / median / 75th / max = 5.24 / 5.65 / 5.84 / 6.00 / **6.42**. The max is *above* the monopoly price of 6. Times: 6,706 (async) and 346 (sync) iterations.

**AFP21 Table 6**, mixed pairing:

- Homogeneous Bertrand: both firms at **2.13 in every run**. A single synchronous firm disciplines the asynchronous one.
- Logit: the synchronous firm 2.73 / 2.94 / 3.69 / 4.45 / 7.28; the asynchronous firm 2.84 / 3.14 / 4.10 / 4.95 / 7.98. Not disciplined.

**AFP21 Fig. 7, the "algorithm coordination game"** (p. 36): async/async gives (8.12, 8.12); any pairing with sync gives (2.13, 2.13). The authors say async is weakly dominant, and strictly dominant under logit (fn. 40). The cells are labeled "payoffs" but they look like prices.

**My scratch sanity check (Python, 100 runs, 20k periods; my seeds, not theirs):**

- sync: all 2.1291
- async: 4.15 / 7.30 / **8.44** / 9.19 / 10
- downward demand: 2.03 / 2.13 / 2.13 / 2.13 / 2.43

So the baseline reading is right, and the protocol is unambiguous enough to reproduce. Script: `scratchpad/afp.py`.

#### 1.3 Ambiguities

1. **Price grid lower end.** The notes to AFP21 Fig. 1, Fig. 2 and Fig. 5 and to AFP22 Fig. 1 say "**0.1** and 10". The text (AFP21 p. 16; AFP22 p. 1) and the code (A.1) say **0.01**. The reported Nash prices 2.0282 / 2.1291, and Fig. 6's 2.09 / 2.51, only fall on the 0.01 grid. **Use 0.01.**
2. **Demand at p = 10.** Fig. notes say "Q = 1 if P < 10". The text says "≤ 10", and the code gives demand D at p = pH. Rest points at max = 10 require positive demand at 10. **Use ≤.**
3. **Downward-demand results disagree between the two papers.** The maximum rest point is 2.23 in AFP21 Fig. 4 and 3.84 in AFP22 Fig. 1(c), for apparently the same model (100 runs). My check gives 2.43, so the tail is seed-sensitive. Report the distribution, not the max.
4. **Asynchronous N = 2 numbers differ.** Fig. 2 / AFP22 (100 runs) give min 5.06 and median 8.34. Table 1 (1,000 runs) gives 4.65 and 8.39. Convergence time is "≈105" (text) vs 115 (Table 1) vs "by period 70" (Fig. 3 example). These are sampling differences.
5. **Downward-demand bound for p < c.** For p < c, (p−c)q* is not a valid lower bound, because profit is negative and more demand makes it worse. It is harmless while W stays positive. Implement it literally, and note the issue.
6. **Synchronous update with β > 0: which next state?** Eq. 2 (p. 13, partly garbled) writes max_y W(y | s^{k+1}). For a counterfactual price p ≠ p_i^k, the text (p. 25) says the continuation is "the discounted perceived value of future play were the non-optimal action played". That implies the counterfactual next state s' = (p, p_j^k), not the realized one. **Make this a switch.** Default to the counterfactual state, since that is what the paper's REBE argument needs.
7. **Tie-breaking.** Matlab `max` returns the first index, i.e. the **lowest price**, which matches Calvano. The tie-share in code A.1 is `nshares = (# rivals at the min) + 1`, an equal split.
8. **Timing.** The code computes both firms' updates from the same period-k price vector, so play is simultaneous. Only one seed (rng(2)) appears in A.1; the loop over 100 seeds is described, not shown.
9. **Experimentation schedule.** The notes formula is garbled; it is k^{−1/θ}, reconstructed from the table column. The time-to-converge column is defined as "iterations until the reported price moments cease to change". That is a population statistic, not a per-run one.
10. **Fig. 5 initial values** U[200, 210] are well above the per-firm monopoly value 4/(1−0.95) = 80, so they are optimistic. No ε is stated for Fig. 5, so it is presumably greedy ("all other respects mirror figure 2").
11. **Fig. 7 cells** are labeled payoffs but show prices (8.12 is presumably the mean async price).

#### 1.4 Testable claims, including what "synchronous" and "asynchronous" mean

- **Precise meaning.** In a period with state s, own price p_i^k and rival price p_j^k:
  - *Asynchronous* revises only W(p_i^k | s) toward realized π_i (+ β·max W at the next state). This is Calvano's rule.
  - *Synchronous* revises W(p | s) for **every** p toward π_i(p, p_j^k) (+ β·continuation), holding the rival's realized price fixed. The firm best-responds in expectation to the rival's last move, as in fictitious play / Cournot adjustment.
  - These are AFP's own words, and they flag them as "slight abuses of language" relative to the RL literature (fn. 21, p. 13).
- **C1 (proved, Prop. 1.1b, p. 21; and §4 p. 24).** With β = 0, any rest point of synchronous learning is a static Nash equilibrium, for any profit function or state space. In a homogeneous Bertrand duopoly every run should end at **2.13 (or 2.03)** within about 100 periods.
- **C2 (Prop. 1.2).** With β = 0, an asynchronous rest point needs only (i) W(p*) ≥ W(p) for all p, and (ii) π(p*, p*) ≥ W(p) for all p ≠ p*. Unplayed prices keep their (decayed) initial values forever. Any p ∈ P can be a rest point with positive probability. The median is about **8.3–8.4** and the min about 4.6–5.1, with **100% of runs above Nash**. Prices *rise* over learning. The mechanism is optimistic values decaying until the two firms happen to match. **No punishment is involved.**
- **C3.** Elevation falls with N: 71.5% above Nash at N = 3, about 20% at N = 4, about 12% at N = 5, 0.1% at N = 10. Logit is much stickier.
- **C4.** Uniform ε-experimentation only partly helps: the median falls from 8.0 to 4.2 as θ goes 1 → 10, and the max stays 10.
- **C5.** The downward-demand bound almost removes elevation (max about 2.2–3.8) and speeds convergence at least 10×.
- **C6 (β = 0.95, memory-1).** Synchronous learning also yields supra-competitive prices (median 6.25), but less than asynchronous (median 10). Under asynchronous learning the memory-1 β = 0 median (8.75) is already close to β = 0.95's 10. **This is the direct challenge to Calvano's "collusion via reward-punishment" reading.** It is testable in our model by turning δ → 0 and memory off in Calvano's logit environment.
- **C7.** One synchronous firm drags an asynchronous rival to Nash under homogeneous Bertrand, but not under logit.
- **C8.** Asynchronous learning is a weakly dominant "meta-strategy" for firms choosing algorithms.
- **Proposed named variations for our model:**
  - `update = async (Calvano default) | sync | sync_downward_demand`
  - `continuation_state = counterfactual | realized` (sync with δ > 0)
  - `exploration = eps_exp_decay (Calvano) | none_optimistic_init | eps_poly(θ)`
  - a separate companion preset for AFP's homogeneous Bertrand (100-point grid, c = 2, V = 10)

---

### 2. Waltman & Kaymak: 2006 ERIM report (IPD theory) and 2008 JEDC (Cournot)

#### 2.1 Model, and what differs from Calvano

**WK08: Cournot, the one Calvano cites and "follows" (Calvano p. 3270)**

- Inverse demand p = max(u − v Σ q_i, 0). Cost w·q_i. Profit π_i = q_i · max(u − w − v Σq, −w) (eqs. 5–7, p. 3280; the minus sign is lost in extraction).
- **u = 40, v = 1, w = 4** (p. 3282). n ∈ {2, …, 6}.
- **Actions: integer q ∈ {0, …, 40}**, i.e. 41 actions.
- Benchmarks:
  - Nash joint output n(u−w)/(v(n+1)) (eq. 8); joint profit (u−w)²n/(v(n+1)²) (eq. 9). For n = 2 that is Q = 24 and profit 288.
  - Collusive (monopoly): Q = 18, profit 324, for any n.
  - Walrasian: Q = 36, profit 0 (p. 3281).
- **Action choice is Boltzmann/logit, not ε-greedy** (eq. 1/3, p. 3278–3279): Pr(a) = exp(Q(s, a)/b) / Σ exp(Q(s, a′)/b).
  - **Temperature b(t) = 1000 · 0.99999^t** (eq. 10, p. 3282).
  - Over **10⁶ periods** per run. b(10⁶) = 1000·e^{−10.0} ≈ 0.045.
- **Q initialized to 0** (p. 3282).
- Update (eq. 2/4): standard asynchronous Q-learning. Memoryless firms use Q(a) ← (1−α)Q(a) + α π with γ = 0.
- Three firm types (p. 3282):
  - **memoryless** (γ = 0)
  - **memory, myopic** (γ = 0). The state is the firm's own last q plus the competitors' *joint* last q. Note this is not Calvano's state, which is the price vector.
  - **memory, far-sighted** (γ = 0.9)
- α ∈ {0.05, 0.25, 0.5, 1.0} for memoryless firms; α = 0.5 for firms with memory.
- 100 runs per cell. **The outcome is the mean of the last 100 periods** (p. 3282).

**WK06: IPD theory**

- Payoffs follow Table 1, p. 2: (C,C) = w, (C,D) = u for the cooperator, (D,C) = z, (D,D) = v.
  - Required ordering: **u < v < w < z** and **2v < z + u < 2w** (eqs. 1–2, p. 3).
- Assumptions (pp. 7–8):
  - A1: memoryless (single state).
  - A2: **α = 1** (Q equals the last payoff received for that action).
  - A3: exploration probability → 0, i.e. fixed T → 0⁺ (Boltzmann, eq. 9) or fixed ε → 0 (ε-greedy).
- Single-state update (eq. 7, p. 6): Q̂(a) ← (1−α)Q̂(a) + α r.
- Experiments (pp. 14–15):
  - 500,000 periods.
  - Boltzmann **T = 10 · 0.999994^t** (eq. 29). ε-greedy **ε = 0.99999^t** (eq. 30).
  - α ∈ {0.05, 0.20, 0.50, 1.00}. 50 runs per cell.
  - Outcome: both cooperating in period 500,000.
  - Initial Q values are not stated.

**Differences from Calvano**

| | Calvano 2020 | WK08 |
|---|---|---|
| Competition | Bertrand, logit | Cournot, homogeneous linear |
| Actions | 15 prices | 41 integer quantities |
| Exploration | ε-greedy, exp decay | Boltzmann, b = 1000·0.99999^t |
| Initial Q | expected payoff against a uniform rival | zero |
| Discount | δ = 0.95 | γ = 0 / 0 / 0.9 |
| Memory | last price pair | none, or (own q, rivals' total q) |
| Outcome | 100k-period policy-stability rule | last-100-period average under still-positive (tiny) temperature |

**Calvano's characterization of WK08 (p. 3269)** is that memoryless/myopic output reduction is "even larger". The tables show that is only partly true:

- At n = 2: memoryless 20.8 vs far-sighted-with-memory **19.6**, so far-sighted is *more* collusive.
- At n ≥ 3: memoryless (α = 0.5) is more collusive than myopic-with-memory.

#### 2.2 Results to match

**WK08 Table 1**, memoryless firms (mean joint quantity and profit over 100 runs, SD in parentheses; p. 3283):

| n | Nash Q / profit | α = .05 | α = .25 | α = .50 | α = 1.00 |
|---|---|---|---|---|---|
| 2 | 24.0 / 288.0 | 22.8 (1.3) / 299.1 (11.6) | 21.2 (1.4) / 312.0 (10.2) | 20.8 (1.2) / 314.7 (6.2) | 20.8 (1.4) / 314.3 (7.0) |
| 3 | 27.0 / 243.0 | 25.1 / 270.7 | 22.0 / 304.6 | 21.5 / 307.8 | 22.1 / 303.7 |
| 4 | 28.8 / 207.4 | 26.3 / 252.1 | 22.6 / 299.0 | 22.1 / 301.4 | 22.9 / 293.2 |
| 5 | 30.0 / 180.0 | 27.6 / 229.3 | 23.2 / 294.1 | 22.2 / 301.1 | 23.3 / 290.2 |
| 6 | 30.9 / 158.7 | 28.3 / 215.4 | 23.3 / 290.7 | 22.6 / 296.3 | 23.1 / 289.1 |

Every cell differs from Nash at p < 0.0001.

**WK08 Table 2**, firms with memory, α = 0.5 (p. 3284):

| n | myopic (γ = 0) Q / profit | far-sighted (γ = 0.9) Q / profit |
|---|---|---|
| 2 | 20.8 / 314.2 | 19.6 / 318.0 |
| 3 | 22.9 / 297.3 | 21.5 / 304.8 |
| 4 | 24.1 / 284.2 | 23.8 / 277.5 |
| 5 | 24.4 / 280.1 | 23.6 / 271.6 |
| 6 | 24.7 / 274.5 | 21.9 / 288.5 |

**WK06 Table 3**, Boltzmann (runs out of 50 with (C,C) at t = 500k; p. 16). Columns are α = .05 / .20 / .50 / 1.00.

| u, v, w, z | runs with (C,C) |
|---|---|
| 0, 1, 9, 10 | 48 / 50 / 50 / 50 |
| 0, 2, 9, 10 | 1 / 50 / 49 / 49 |
| 0, 2, 6, 10 | 0 / 0 / 9 / 10 |
| 0, 3, 6, 10 | all 0 |

**WK06 Table 4**, ε-greedy (same layout):

| u, v, w, z | runs with (C,C) |
|---|---|
| 0, 1, 9, 10 | 48 / 30 / 1 / 0 |
| 0, 2, 9, 10 | 13 / 2 / 0 / 0 |
| 0, 2, 6, 10 | all 0 |
| 0, 3, 6, 10 | all 0 |

WK06 Fig. 1 (p. 16): at (0, 1, 9, 10) with α = 0.05 under Boltzmann, cooperation emerges slowly over the 500k periods.

**My scratch sanity check of WK08 memoryless n = 2** (4 runs each, my own RNG): α = 0.5 gives Q = 21.0, profit 314.5; α = 0.05 gives 23.0, 298.5. This matches Table 1 (20.8 / 314.7 and 22.8 / 299.1). The spec is complete enough to reproduce. Script: `scratchpad/wk.py`.

#### 2.3 Ambiguities

1. **WK08 state for firms with memory.** It is (own last q, *competitors' joint* last q). The second component's range is 0..40(n−1), so the state space grows with n. It is unclear whether states are indexed exactly or binned; presumably exactly.
2. **WK08 outcome window.** Results are "the last 100 periods" while b ≈ 0.045 > 0. Profits are of order 100, so the logit is effectively greedy, but it is not formally converged. No convergence criterion is given.
3. **WK08 Theorem 1 initial conditions** require Q_C ∈ (π_CN, π_CC) and Q_N ∈ (π_NN, π_NC) (p. 3285). The simulations start Q at 0. The theorem is about the two-action reduced game, not the 41-action simulations.
4. **WK06 initial Q values** for the experiments are unstated.
5. **WK06 outcome.** It is a *single-period* snapshot ("both cooperated in period 500,000").
6. **WK06 result definitions.** The convergence results hold under the paper's Definitions 1–2 (lim Pr(a_t = π) = 1), with the order of limits (T → 0 after t → ∞). They "need not be valid for alternative definitions of convergence" (p. 9).
7. **Ties under ε-greedy.** WK06 says "the action (or one of the actions) with the highest Q̂", so tie-breaking is unspecified.
8. **Boundary case of Theorem 1.** It gives lim P1 = **1/3** when w − v = 2(v − u) (eq. 25, p. 13). The (0, 2, 6, 10) row is exactly that case (4 = 4): Boltzmann with α = 1 gives 10/50 = 0.2 against the theoretical 0.33. The authors don't comment.

#### 2.4 Testable claims: why memoryless Q-learners beat Nash

- **Claim (WK08 abstract, §6, §8).** Memoryless, myopic Q-learners **on average produce less than Cournot-Nash output and earn more** (Table 1). There is "no punishment mechanism and no possibility for explicit communication". Full collusion (Q = 18) usually does not emerge. The effect persists up to n = 6, which contradicts lab evidence that "four are many" (p. 3287).
- **The mechanism is exploration-driven and is a stochastic-stability argument, not an equilibrium** (WK08 §7 pp. 3284–3286; WK06 Thm. 1 pp. 9–13). Reduce the game to two actions, q_N (Nash) and q_C (more collusive), giving a prisoner's dilemma. With vanishing noise the learning process spends almost all its time in one of two absorbing regimes, a **collusive state** or a **Nash state**. The relative rates of escape decide which:
  - Leaving collusion needs **one** firm to experiment (play q_N).
  - Leaving Nash needs **both** firms to experiment *simultaneously* (one joint (q_C, q_C) play suffices if α > (π_NN − π_CN)/(π_CC − π_CN); eq. 12).
  - Under **Boltzmann**, the probability of experimenting is ≈ exp(−ΔQ/b), so it depends on the Q-gap:
    - in the collusive state ΔQ ≈ π_CC − π_NN
    - in the Nash state ΔQ ≤ π_NN − π_CN
  - When **π_CC − π_NN > 2(π_NN − π_CN)**, one firm's deviation from collusion is exponentially rarer than two simultaneous deviations from Nash. So the time fraction in collusion → 1 as b → 0. This is the same condition as WK06 eq. 11, w − v > 2(v − u). In the Cournot reduction it always holds (p. 3285).
  - With **fixed-probability experimentation (ε-greedy)** the argument flips. A single deviation (∝ ε) is always more likely than a double one (∝ ε²), so play ends at Nash (WK06 Thm. 2: p_{16→1} = p_{1→13} = ε²/4, p_{1→5} = p_{1→9} = ε/2 − ε²/4; WK08 p. 3286: "a fixed probability of experimentation typically leads to firms spending most of their time in the Nash state").
  - So **the "collusion" is a learning artifact of value-dependent (logit) exploration**, which is exactly what Calvano's footnote 7 calls a "failure to learn an optimal strategy".
- **Caveats in their own data that matter for Calvano.** With α < 1, ε-greedy *can* still cooperate when w − v is large and α small: 48/50 runs at α = 0.05 for (0, 1, 9, 10) (WK06 Table 4). Lower α makes Boltzmann cooperation harder and ε-greedy cooperation easier (p. 18). Calvano uses ε-greedy with α = 0.15. **So WK's ε-greedy prediction for Calvano is "Nash unless α is small", which is directly testable in our engine.**
- **Proposed named variations:**
  - a `memoryless` (k = 0) and `myopic` (δ = 0) switch on Calvano's model
  - `exploration = boltzmann(b0, decay)` beside `eps_greedy(exp(−βt))`
  - a WK08 Cournot companion preset (u = 40, v = 1, w = 4, q ∈ 0..40, b = 1000·0.99999^t, Q₀ = 0, 10⁶ periods)
  - a WK06 2×2 IPD preset with the four payoff rows
- **Decision rule to preregister.** Calvano-logit with memory off and δ = 0, under ε-greedy and under Boltzmann. If the profit gain Δ stays well above 0 under ε-greedy, then AFP's optimistic-asymmetric-update channel (not WK's exploration channel) is doing the work.

---

### 3. Sandholm & Crites (1996, Biosystems): Q-learning in the IPD

#### 3.1 Model, and what differs from Calvano

- **PD payoffs (Table 1, p. 8): R = 0.3, S = 0, T = 0.5, P = 0.1.** These satisfy T > R > P > S and 2R > T + S > 2P (eqs. 3–4). Both players use the same matrix.
- Infinite horizon, discounted. Q-learning (eq. 1, p. 4): Q(s,a) ← Q(s,a) + α[r + γ max_b Q(s′,b) − Q(s,a)].
- Defaults (p. 11): **α = 0.2, γ = 0.95.**
- **The state ("sensation") is the last w moves of both players.** The baseline w = 1 gives 4 states (CC, CD, DC, DD). This is structurally Calvano's memory-1 with m = 2.
- Two kinds of Q storage:
  - lookup table
  - Elman recurrent net per action: 4 unary inputs, 3 logistic hidden units, 3 context units, linear output; backprop learning rate 0.3 for hidden units (momentum 0.05) and 0.01 for the output unit (pp. 13–15)
- **Exploration is Boltzmann** (eq. 5) with **t = 5 · 0.999^n**. When t < 0.01 exploration stops and choice becomes greedy, which happens after **6,212** games (eq. 6, p. 16).
  - Longer schedules use annealing factors 0.9999 / 0.99999 / 0.999999 / 0.9999999, giving 62,143 / 621,458 / 6,214,605 / 62,146,078 exploring games (p. 23).
  - Learning **continues after exploration stops**, with greedy play.
- **Initial Q values ~ U[0, 1]** (stated in Exp. 3, p. 22; presumably global).
- Runs: 100 games per setting (35 for the slowest schedule). 300,000 iterations, extended so that about 300k follow the end of exploration. Outcome = the share of the **last 100 iterations** in CC / CD / DC / DD.

**Differences from Calvano.** A 2-action PD instead of a 15-price logit Bertrand game. Boltzmann with a hard cutoff instead of ε-greedy. α = 0.2 and γ = 0.95 (≈ Calvano's α = 0.15, δ = 0.95). Random U[0, 1] initial values. Memory w ∈ {0, 1, 2, 3}. An optional neural-net Q. **This is the closest structural precursor**: memory-1 tabular Q-learning with δ = 0.95, both players learning.

#### 3.2 Results to match

**Exp. 1, against Tit-for-Tat (p. 18).** Tabular w = 1, 100 runs × 100k iterations per γ. Optimal play is learned in **100% of runs** at every γ:

- defect for γ ≤ 0.2
- alternate C/D for 0.25–0.65
- cooperate for γ ≥ 0.7

The theoretical thresholds are 1/4 and 2/3 (pp. 10–11).

**Exp. 2, learner vs. learner**, % of 100 runs ending in each final pattern (pp. 20–21):

| Pairing | Final pattern | α = 0.2 | α = 1.0 | anneal .9999 (α = 0.2) |
|---|---|---|---|---|
| LB–LB (table vs table) | DD | 68 | 75 | 0 |
| | CC | 25 | 1 | 24 |
| | CC/CD loop | 4 | 8 | 14 |
| | CC/DC loop | 2 | 7 | 13 |
| | CD/DC loop | 1 | 2 | 49 |
| | CC/DD loop | – | 7 | – |
| LB–RB (table vs net) | DD | 86 | 98 | 86 |
| | CC/DC loop | 9 | – | 4 |
| | CC | 5 | – | 10 |
| RB–RB (net vs net) | DD | 98 | 89 | 88 |
| | CC | 2 | 11 | 11 |

**Exp. 3, no exploration (pp. 22–23).** Outcomes depend on the initial Q values. Non-exploring agents get exploited by exploring ones. Cooperation is "quite frequent" when neither side explores.

**Exp. 4, longer exploration, LB–LB** (% of runs by annealing factor; p. 24):

| Final pattern | .999 | .9999 | .99999 | .999999 | .9999999 (35 runs) |
|---|---|---|---|---|---|
| DD | 68 | 0 | 0 | 0 | 0 |
| CC | 25 | 24 | 6 | 3 | 1 run |
| CC/CD loop | 4 | 14 | 39 | 45 | 17 runs |
| CC/DC loop | 2 | 13 | 33 | 52 | 17 runs |
| CD/DC loop | 1 | 49 | 21 | 0 | 0 |
| three-way | – | – | 1 | – | – |

- As exploration lengthens, DD vanishes, but **so does CC**: play converges to CC/CD loops (50% CC, 25% CD, 25% DC).
- **Mean stage payoff rises monotonically toward about 0.275** (Fig. 5, p. 25), below full cooperation's 0.3.

**Exp. 5, memoryless (γ = 0, two Q values).** Players always learn to **defect**, against both TFT and each other (p. 26). A 2-step-return variant cooperates with TFT (expected 2-step returns: C 0.1p + 0.5, D 0.3p + 0.2) but still defects against another learner (pp. 26–28).

**Exp. 6, unequal memory lengths** (anneal .999 / .9999):

- h1–h2: DD 85% / 0%. At .9999: CD/DC 41%, the 0/50/25/25 pattern 26%, CC 10%.
- h1–h3: DD 93% / 3%. At .9999 the 0/50/25/25 pattern is 44%.
- h2–h2: DD 99% / 0%.
- The longest loop has 8 steps. Longer memory gives a slight edge, but both players having one move of memory is the most cooperative (pp. 28–31).

#### 3.3 Ambiguities

1. **Greek letters lost in extraction.** α (learning rate) and γ (discount) are inferred from context: "learning rate α = 0.2 and discount factor γ = 0.95", p. 11; table headers "(α = 0.2)", "(α = 1.0)".
2. **Initial Q values** are only stated in Exp. 3 (U[0, 1]).
3. **"Final state"** classification uses the last 100 iterations. Loops are reported by their composition. Phase/ordering conventions are implicit (e.g. 50/50/0/0 = CC/CD loop).
4. **Exp. 5 (memoryless) gives no parameters** beyond γ = 0. With these payoffs, WK06's Theorem 1 condition w − v vs 2(v − u) is **0.2 vs 0.2, exactly the knife edge**, where the predicted cooperation share is 1/3. Meanwhile the fast schedule (t ≥ 0.01, then greedy) and α = 0.2 push toward defection. So SC96's "memoryless always defect" and WK's "memoryless can cooperate" are not in direct conflict: SC96's payoffs sit exactly on WK's boundary. **This is a clean variation to run.**
5. **The hard exploration cutoff** at t < 0.01, followed by greedy learning, is a third exploration regime, distinct from both WK's and Calvano's.
6. **Asymmetric loops** (CC/CD) are reported as stable final states. A convergence criterion like Calvano's (policy unchanged for 100k periods) would count these as converged, but a "rest point" criterion like AFP's would not.

#### 3.4 Testable claims

- Tabular memory-1 Q-learners (α = 0.2, γ = 0.95) learn the optimal reply to TFT at all γ.
- Learner vs. learner: with short exploration, mostly mutual defection (68%). **Longer exploration eliminates DD**, but it produces exploitative CC/CD loops rather than stable CC. Payoff approaches about 0.275 (vs. R = 0.3).
  - This is the precursor of Calvano's "longer exploration → more collusion", and of its imperfect, oscillating outcomes.
- Memoryless learners defect (γ = 0). This contrasts with WK06/WK08; see ambiguity 4.
- Longer or asymmetric memory does not help cooperation.
- **Proposed preset:** a 2-action IPD on our Calvano engine (m = 2, memory k ∈ {0, 1, 2, 3}, Boltzmann t = 5·r^n with a cutoff at 0.01, Q₀ ~ U[0, 1], α = 0.2, γ = 0.95), sweeping r over {.999 … .9999999}. Lookup tables only; the RNN is out of scope.

---

### Cross-paper threads for the Calvano reproduction

1. **Three distinct mechanisms for supra-competitive outcomes without punishment:**
   - AFP: asynchronous updating plus optimistic/random initial values. Unplayed actions are never re-evaluated, so a matched high price becomes self-confirming (an EBE, not Nash).
   - WK: value-dependent (Boltzmann) exploration makes simultaneous co-experimentation out of Nash likelier than a unilateral exit from collusion.
   - SC96: long exploration gives loops.

   Calvano uses ε-greedy (which by WK's argument should *not* collude when memoryless) with asynchronous updating (which by AFP's argument should). **The decisive experiment is Calvano with k = 0, δ = 0.** AFP says Δ stays high; WK's ε-greedy theory says Δ → about 0 unless α is small.
2. **The best single switch to add is `update = sync`.** AFP prove that with δ = 0 it forces Nash. With δ = 0.95 and memory-1, AFP report a lower but still supra-competitive median (6.25 vs 10 in their Bertrand).
3. **Convergence criteria differ in all four papers:**
   - Calvano: 100k-period policy stability.
   - AFP: price rest point.
   - WK: last 100 periods, or the final period.
   - SC96: last 100 iterations.

   Our model should report under Calvano's criterion and also under each paper's own.

---------------------------------------------------------------------------------------------------

## 4. Literature reacting to Calvano, Calzolari, Denicolò & Pastorello (2020, AER 110(10):3267–97)

Survey date: 2026-10-01. Purpose: plan a reproduction in the playground, with a literal default, named switches, and sweeps where the critics say the result breaks.

All saved PDFs are in `papers/ai-coordination/`. "NOT OBTAINED" means SSRN/INFORMS/Elsevier only (Cloudflare 403). Sci-hub returned nothing for any post-2021 DOI. Extracted text for the PDFs I read is in `scratchpad/lit/*.txt`.

Out of scope here, covered by other agents: Calvano 2020's own appendix and code, Calvano 2021 IJIO, Klein 2021 (see `scratchpad/collusion-followups.md`), and Waltman & Kaymak 2008 (already saved).

---

### 0. Bottom line for planning

**The headline numbers replicate.** Schildknecht (2026, JCRE) is a peer-reviewed Python port of the Fortran. It gets Δ ≈ 0.86 mean (0.895 median), punishment-then-return impulse responses, and a rising Δ(δ) curve. Several GitHub ports agree.

**What is contested is the interpretation**, i.e. whether this is "collusion" (a learned reward–punishment equilibrium). The main lines of attack:
1. **Exploration artifact.** Supra-competitive prices come from too little or decaying exploration and failure to learn, not from equilibrium strategies.
   - Abada & Lambin 2023 MS; Abada, Lambin & Tchakarov 2024 EJOR; Lambin 2024 WP.
   - Asker, Fershtman & Pakes (AFP).
   - Xu & Zhao 2026: with memoryless agents and persistent ε-greedy, only Nash is stochastically stable, but convergence time is super-exponential in δ.
   - Dolgopolov 2024: in a memoryless PD, ε-greedy's only stochastically stable state is mutual defection.
2. **The impulse-response test is not diagnostic.** Epivent & Lambin 2024 report three things:
   - the same algorithms also start "price wars" after a unilateral price *increase* (an invitation to collude);
   - "punishment" appears even in sessions that converge *below* Nash;
   - so the IR signature does not identify a reward–punishment scheme.
   Lambin 2024: simultaneous experimentation plus learning inertia produce both high prices and the IR pattern without a causal link, even without memory.
3. **Timescale and equilibrium in algorithm space.** den Boer, Meylahn & Schinkel (2022 SSRN; Management Science 2026):
   - Q-learning learns collusive equilibria "only on timescales irrelevant to the firm's objective";
   - Calvano's simulations "do not give sufficient evidence for the claim that these types of Q-learning algorithms systematically learn collusive strategies" (quoted in Hartline, Long & Zhang 2024);
   - using Q-learning is not an equilibrium of the meta-game where firms choose algorithms;
   - real collusion needs an "explicit cartel agreement" on the algorithm.
   Carissimo et al. 2025 make a similar point: "algorithm orchestration".
4. **Fragility out of the training environment.** Eschenbaum, Mellgren & Zahn 2022:
   - Calvano-parameterized Q-learners trained in one context and tested in another (new rival seed, or slightly different parameters) drop to static Nash;
   - the collusion index falls by about 72% on average;
   - it does not recover without many more iterations;
   - collusion survives the transfer only when agents condition on their *own* past price.
5. **Spurious vs. genuine collusion, conceded by Calvano et al. themselves** (2023 IJIO, "Genuine or spurious?"):
   - supra-competitive prices can arise "even in markets where collusion is impossible by design or cannot be an equilibrium";
   - which kind you get depends on "the rate and mode of exploration";
   - random (ε-greedy) exploration gives genuine collusion; mechanical (optimistic-initialization) exploration, as in AFP's asynchronous learners, gives spurious collusion.
6. **Mechanism without strategies.** Banchio & Mantegazza 2023: memoryless ε-greedy Q-learners "spontaneously couple". Correlated estimation errors make them jointly cooperate and defect in stochastic cycles, so price-fixing needs no reward–punishment scheme. Algorithms with uniform learning rates across actions (counterfactual or full-feedback updates) are immune.

**Implications for our reproduction.** These are switches and sweeps that the critics say flip the result:
- (a) Exploration schedule:
  - β sweep;
  - constant-ε vs. decaying;
  - Boltzmann/logit vs. ε-greedy;
  - optimistic vs. Calvano's Q₀ initialization.
- (b) Memory: 0 (stateless) vs. 1.
- (c) Synchronous vs. asynchronous updates, i.e. AFP's counterfactual "full-feedback" updating.
- (d) Upward-deviation IR in addition to Calvano's downward IR (Epivent & Lambin).
- (e) Fraction of sessions whose converged strategies are mutual best responses or subgame-perfect.
- (f) Train/test transfer: pair agents from different sessions (Eschenbaum).
- (g) Time-to-convergence distribution.
- (h) Δ at δ=0:
  - Schildknecht's posted sweep CSV gives Δ ≈ 0.2 at δ=0;
  - myopic agents should not collude;
  - Δ>0 at δ=0 is the "spurious" signature.
- (i) Asynchronous Poisson-clock updating (Conjeaud, Abel & Kalogeratos 2026).

---

### 1. Explicit replications of Calvano 2020

| Item | Verdict | Notes |
|---|---|---|
| **Schildknecht, J. (2026). "Tacit Algorithmic Collusion. A Replication of Calvano et al. (AER 2020)." *J. Comments & Replications in Economics* 5 (2026-7). DOI 10.18718/81781.58.** Code: JCRE archive DOI 10.15456/j1.2026111.1137134085; GitHub `JacobSKN/replication_calvano_algorithmic_collusion`. **SAVED** `schildknecht-2026-jcre-tacit-algorithmic-collusion-replication-of-calvano-2020.pdf` | **Replicates the headline** | See the list below the table. |
| **Courthoud, M. (2021)** `matteocourthoud/Algorithmic-Collusion-Replication` (Python, Matlab, Julia; 22★) | Unstated in the README | The most-forked port. `teeross/...` extends it to oligopoly. The other agent has a `scratchpad/courthoud/` checkout. |
| **Lewis, J. (2024)** `jeremiahpslewis/AlgorithmicCompetition.jl` (Julia; `run_aiapc()`) | Unstated | Active (pushed 2026-10). It also has a "DDDC" experiment. No paper found. |
| **Werner, T. (2021/22)** `ToFeWe/qpricesim` and replication code | n/a | Calvano-style Q-learners in a non-elastic (homogeneous, reservation-price) demand market; used with human lab subjects. **SAVED** `werner-2022-wp-algorithmic-and-human-collusion.pdf` |
| **Yusei406** (`calvano-thesis`, `calvano-replication`, `calvano-qlearning`, `calvano2020-replication`) | Untrustworthy | Undergraduate-thesis code. One repo uses 11 actions, constant ε=0.1, α=0.1 and "μ=0.05 optimization", and reports Δ beating "paper targets". Do not use as a reference. |
| Small course ports: `Wick-7/calvano2020-qlearning-reproduction` (Chinese course project), `mountaha-ghabri/algorithmic-collusion-reinforcement` | Weak | Wick-7: one run converged at 1.52M periods; mean price ≈ 1.79 over 1,000 sessions (Nash 1.47, monopoly 1.93). Mountaha: baseline Δ = 0.991 with a price grid of [1.2, 2.0], which is not Calvano's ξ=0.1 grid. Likely misspecified. |

Schildknecht (2026) in detail:
- **Reproduced:**
  - 1,000 sessions; mean Δ = 0.863 (sd 0.123), median 0.895;
  - mean converged price 1.802 (range 1.50–1.97);
  - downward-deviation IR: punishment, then gradual return;
  - Δ rises with δ: about 0.11 at δ=0.36 and >0.8 at δ≈0.92.
- **Discrepancies:**
  - Convergence takes **about 3.76M periods** (sd 139k), not Calvano's "hundreds of thousands". He blames the RNG and implementation.
  - The (α,β) heatmap keeps its diagonal, but the best values sit at higher α and β.
  - **The "Nash region" at low α is largely absent**: Nash convergence peaks at 45%. He reads this as collusion being "more robust" than reported.
  - **Convergence cliff**: at β=1e-6, only 6% of runs converge within 5M periods; at β=2.5e-6, 100% converge.
  - Quality asymmetry breaks Δ: at a₂=2.4, Δ₁ = 9.4 because the JPM denominator goes to 0, while Calvano's values stay below 1. His explanation is that the original Fortran reads its benchmarks from a precomputed file.
- **Must flag:** fn 2 says he used "the corrected formula for the parameter ν, as pointed out by the reviewers regarding **footnote 20 in Calvano et al. (2020)**". So Calvano's printed ν formula appears to be wrong. Check against the Fortran before coding.
- **Posted CSVs show more:**
  - Δ ≈ 0.20–0.24 at δ=0.0–0.12, an "even myopic agents get Δ>0" signal;
  - the symmetric baseline row in `asymmetry_summary_results.csv` has ConvergenceRate 0.625 with a 5M cap, with mean convergence period about 2.95M.
  - Both figures are in tension with Calvano's ~100% convergence claim.
- **Other ports he lists:** Lewis (Julia), Courthoud, Werner `qpricesim`, yusei406.

No replication I found reports that the headline Δ *fails* to reproduce under Calvano's literal parameters. The failures in the literature come from changing exploration, memory, synchrony, the deployment context, or the interpretation test.

---

### 2. Core critiques

#### Abada & Lambin (2023), "Artificial intelligence: Can seemingly collusive outcomes be avoided?" *Management Science* 69(9):5042–5065. DOI 10.1287/mnsc.2022.4623. SSRN 3559308.
- **Claim.** Independent Q-learners trading a *storable good* (battery arbitrage in an electricity market) quickly reach "seemingly collusive" outcomes. This "could originate in imperfect exploration rather than excessive algorithmic sophistication". A regulator can restore competitive outcomes by "enforcing decentralized learning" or by intervening during learning.
- **Relation to Calvano.** Critique/extension. A different market (dynamic storage, not logit Bertrand), but the same mechanism question. It names the exploration-artifact hypothesis.
- **Spec.** Code is public: Google Drive folder linked from Lambin's site, https://drive.google.com/drive/folders/1KLtPQzkec9Yyj1lUsDhKHBPLn4KNY-h- . Paper numbers unseen.
- **Status.** **NOT OBTAINED** (INFORMS/SSRN 403; sci-hub not found).
- **Caution.** Hammond et al. 2025 (multi-agent risks) cites Abada & Lambin 2023 as *evidence that algorithms collude*. That misreads a paper whose point is that the collusion is "seeming".

#### Abada, Lambin & Tchakarov (2024), "Collusion by mistake: Does algorithmic sophistication drive supra-competitive profits?" *EJOR* 318(3):927–953. SSRN 4099361.
- **Claim.** In a PD with stylized Q-learners and in Bertrand simulations, seemingly-collusive Q-learners turn competitive when allowed "more thorough exploration". More sophisticated algorithms outcompete them, so sophistication may cure seeming collusion rather than cause it.
- **Relation.** Direct critique of Calvano's interpretation.
- **Status.** **NOT OBTAINED.**

#### Lambin (2024 WP, R&R), "Less than meets the eye: simultaneous experiments as a source of algorithmic seeming collusion." SSRN 4498926.
- **Claim.** Calvano-type high prices *and* the reward–punishment-looking IRs both come from simultaneous experimentation plus learning inertia, with no causal link between them.
  - It arises with memoryless (stateless) Q-learning and "much faster than previously thought".
  - The secondary summary says it also arises with myopic agents. Xu & Zhao instead cite Lambin 2024 for collusion vanishing at small δ. Check the source.
  - The model splits learning into a full-exploration phase and a no-exploration phase.
- **Relation.** The sharpest mechanism critique, set in Calvano's own environment.
- **Spec.** Conjeaud 2026 says it uses Calvano-like parameters (μ=0.25, a=2, c=1, 15 prices).
- **Status.** **NOT OBTAINED.**

#### Epivent & Lambin (2024), "On algorithmic collusion and reward–punishment schemes." *Economics Letters* 237:111661. SSRN 4227229.
- **Claim.**
  - The collusion interpretation rests on IRs: after a unilateral cut, several periods of low prices and profits follow.
  - But "simple invitations to collude such as price increments are also followed by aggressive price wars".
  - Algorithms "may converge to outcomes worse than Nash" while still punishing deviations.
  - So the IR is not a signature of collusion; the high prices may be "a failure to learn to compete".
- **Relation.** Direct critique of Calvano's §V evidence. It uses Calvano's environment, per Schildknecht, who cites it as building on Calvano's baseline.
- **Spec.** Short letter. Reproducible by adding an upward-deviation IR to a Calvano reproduction.
- **Status.** **NOT OBTAINED** (SSRN/Elsevier 403).

#### den Boer, Meylahn & Schinkel (2022 SSRN 4213600; *Management Science*, online 2026-06-09, DOI 10.1287/mnsc.2024.08557), "Artificial collusion: Examining supracompetitive pricing by Q-learning algorithms."
- **Claim.**
  - Q-learning "can learn collusive equilibria only on timescales irrelevant to the firm's objective".
  - Achieving it requires an "explicit cartel agreement" on algorithm design.
  - "There is no immediate reason for alarm."
  - Calvano's simulations "do not give sufficient evidence for the claim that these types of Q-learning algorithms systematically learn collusive strategies" (quoted by Hartline et al. 2024).
  - In the meta-game where firms choose their algorithm, Q-learning vs. Q-learning is not an equilibrium unless firms are patient or learning is fast (as summarized by Frick 2026).
  - They propose conditions for "autonomous algorithmic collusion": supracompetitive prices, learned in relevant time, by algorithms that are themselves an equilibrium choice.
- **Relation.** Critique of Calvano's design; a detailed dissection of their Q-learner.
- **Spec.** Unknown in detail (paper unseen).
- **Status.** **NOT OBTAINED.** Twente repository PDF behind Cloudflare 403: https://research.utwente.nl/files/289647191/2022_XBoerMeylahnSchinkel_SSRN_id4213600.pdf . Open it in a browser.

#### Asker, Fershtman & Pakes. 2021 NBER and 2022 AEA P&P are already saved. Published long version: "The impact of artificial intelligence design on pricing", *J. Economics & Management Strategy* 33(2), 2024 (DOI 10.1111/jems.12516). **JEMS version not obtained.**
- **Claim.** Learners that update counterfactually ("synchronous"), using knowledge of the demand curve, converge near competitive prices. Asynchronous learners with optimistic initialization stick at high prices. So the outcome depends on design.

#### Calvano, Calzolari, Denicolò & Pastorello (2023), "Algorithmic collusion: Genuine or spurious?" *IJIO* 90:102973. CEPR DP16393 (2021, titled "…Genuine and Spurious"); SSRN 3928672.
- **Claim (the authors' own reply to AFP).** "Reinforcement-learning pricing algorithms sometimes converge to supra-competitive prices even in markets where collusion is impossible by design or cannot be an equilibrium outcome. We analyze when such spurious collusion may arise, and when instead the algorithms learn genuinely collusive strategies, focusing on the role of the rate and mode of exploration." Per CEPR: AFP's asynchronous algorithms explore *mechanically* (via optimistic initialization). With *random* exploration, asynchronous algorithms "learn genuinely collusive strategies".
- **Relation.** The original authors concede that spurious collusion exists and draw the line by exploration mode.
- **Status.** **NOT OBTAINED.**
- **Important for us.** This gives a literal, named switch: exploration mode {ε-greedy random, optimistic-init mechanical} × update {asynchronous, synchronous}.

#### Banchio & Mantegazza (2023), "Adaptive Algorithms and Collusion via Coupling", EC '23 (ACM). arXiv 2202.05946, v5 Sept 2023, titled "Artificial Intelligence and Spontaneous Collusion". **SAVED** `banchio-mantegazza-2023-arxiv-ai-and-spontaneous-collusion.pdf` (61 pp).
- **Claim.**
  - In a fluid (continuous-time ODE) limit, ε-greedy Q-learners without memory fall into **spontaneous coupling**: correlated estimation errors make them cooperate and defect jointly in stochastic cycles.
  - That sustains dominated, collusive outcomes with no reward–punishment strategy.
  - The cause is non-uniform effective learning rates: rarely played actions update slowly and their estimates persist.
  - Algorithms with uniform learning rates (counterfactual or full-feedback updating) learn only undominated strategies.
  - Applications: Bertrand price-fixing in AFP's setting, market splitting in keyword auctions, and feedback design for auctions.
- **Relation.** Says Calvano's "mistakes" (failures to optimize) are sustained by coupling. Supplies the mechanism behind AFP's result.
- **Spec.**
  - The PD and Bertrand examples have explicit parameters in the paper.
  - The analysis is the α→0 limit and symmetric only, a limitation noted by Xu & Zhao.
  - Reproducible as a memoryless switch in our model.

#### Xu & Zhao (2026), "Memoryless Algorithmic Collusion: Sure to Fail, Slow to Fall." arXiv 2409.01147v2 (formerly "On Mechanism underlying Algorithmic Collusion"). **SAVED** `xu-zhao-2026-arxiv-memoryless-algorithmic-collusion-sure-to-fail-slow-to-fall.pdf`.
- **Claim.**
  - For memoryless Q-learning in Bertrand-style games (PD, Bertrand, first/second-price auction mixtures): with exploration off, any symmetric action profile is absorbing, so collusion can persist.
  - With persistent ε exploration, only the Nash state is stochastically stable. Collusion "dissolves with probability one in the long run".
  - Convergence time grows **super-exponentially in δ**. Short-run collusion survives because exploring unprofitable low actions re-inflates their Q-values (a random walk with Nash as the absorbing boundary).
  - The learning rate has a non-monotone effect.
- **Relation.** Theoretical support for the exploration-artifact view (memoryless case only). Reconciles "collusion exists" with "it's not an equilibrium".

#### Dolgopolov, A. (2024), "Reinforcement learning in a prisoner's dilemma." *GEB* 144:84–103. SSRN 4240842.
- **Claim.** Stochastic stability of memoryless Q-learners in a PD:
  - ε-greedy has mutual defection as the only stochastically stable state;
  - logit exploration permits some cooperation, depending on payoffs and α;
  - Waltman & Kaymak 2007/2008 did the α=1, undiscounted case.
- **Relation.** Theory against memoryless collusion under ε-greedy; the exploration mode matters.
- **Status.** **NOT OBTAINED.**

#### Eschenbaum, Mellgren & Zahn (2022), "Robust Algorithmic Collusion." arXiv 2201.00345. **SAVED** `eschenbaum-mellgren-zahn-2022-arxiv-robust-algorithmic-collusion.pdf` (38 pp).
- **Claim.**
  - Q-learners trained in Calvano's environment do not carry collusion into a test context: a different rival seed or slightly different parameters.
  - Play reverts to static Nash; the collusion index falls about 72% on average.
  - Re-convergence takes many iterations.
  - Restricting agents to condition only on their own past price makes collusion robust, because it limits overfitting to the rival.
  - Policy implication: coordination of algorithm *design* is the real risk.
- **Spec.** Calvano's literal parameters (α=0.15, β=4e-6, logit demand, ≤1e9 periods, >99% convergence) plus a train/test context framework. **Fully reproducible** as a "cross-session pairing" switch.

---

### 3. Extensions and supporting results

| Paper | Claim | Relation | Spec? | File |
|---|---|---|---|---|
| **Hettich, M. (2021).** "Algorithmic Collusion: Insights from Deep Learning." CQE WP 94/2021, Münster; SSRN 3785966 | DQN agents in Calvano's Bertrand collude and start raising prices after about 20k steps, far faster than tabular. Collusion vanishes by 10 firms; a modified state strengthens it. Notes Calvano needs about 850k steps, over 3 years at Amazon's 30 updates/hour. | Extends (speed) | Yes, Calvano demand | SAVED `hettich-2021-cqe-wp-…` |
| **Possnig, C. (2023).** "Reinforcement Learning and Collusion." WP, U. Waterloo | Stochastic-approximation ODE characterization of long-run policies. In repeated Cournot, Nash is learned when agents condition only on the last price. With richer state, they converge with positive probability to collusion and never to Nash. | Theory supporting Calvano (with memory) | Analytic | SAVED `possnig-2023-wp-…` |
| **Askenazi-Golan, Mergoni Cecchelli, Plumb & Possnig (2024).** "The Bounds of Algorithmic Collusion: Q-learning, Gradient Learning, and the Folk Theorem." arXiv 2411.12725 | Folk-theorem-style characterization of payoffs reachable by Q-learning and other dynamics with finite recall; claims the first convergence result for multi-agent Q-learning in repeated games | Supports possibility | Theory | SAVED |
| **Bertrand, Duque, Calvano & Gidel (2025, ICML).** "Self-Play Q-learners Can Provably Collude in the Iterated Prisoner's Dilemma." arXiv 2312.08484 | Memory-1 self-play ε-greedy Q-learners provably learn Pavlov (win-stay, lose-shift), not always-defect, under broad conditions | Calvano's coauthor; theoretical support with memory | Yes | SAVED |
| **Meylahn & Janssen (2021).** arXiv 2107.13995 | Memory-1 best-response dynamics in 2×2 games; WSLS is the only equilibrium pair in PD, stag hunt and hawk-dove | Theory | Yes | SAVED |
| **Barfuss & Meylahn (2023).** "Intrinsic fluctuations of RL promote cooperation." arXiv 2209.01013 (Sci. Reports 2023) | Memory-1 IPD: noise from the learning process doubles cooperation, up to 80%. It needs high δ, low ε and low α, and takes long times. | Mechanism | Yes | SAVED |
| **Meylahn & Schäfer (2026).** "Equilibrium stability as a driver of cooperation among Q-learners." arXiv 2607.13607 | With constant exploration, measure the time-share in cooperative strategies, not convergence. Derives a boundary predicting cooperative dominance (F1 0.85–0.95). Explicitly motivated by the Lambin critique. | Reframes | Yes | SAVED |
| **Schaefer, M. (2022).** arXiv 2211.15331 | ε-greedy memory-1 Q-learners in IPD. The replicator-potential "kinetic energy" ratio predicts the cooperation frontier; it correlates above 0.8 with human lab cooperation rates. | Mechanism | Yes | SAVED |
| **Conjeaud (2026).** "Algorithmic collusion with endogenous exploration." arXiv 2312.02644 | When firms choose ε, every equilibrium is collusive. Players over-explore in equilibrium. Uses Calvano/Lambin parameters: μ=0.25, a=2, c=1, n=15, 40×40 ε grid, 50 runs × 2M periods. | Extends | Yes | SAVED |
| **Conjeaud, Abel & Kalogeratos (2026).** "Algorithmic collusion under asynchronous price updating." arXiv 2608.01406 | Poisson-clock asynchronous updating hampers collusion, especially for stateless agents ("spurious" collusion disappears). Memory-1 collusion is more robust if rival prices are observed precisely. They detect reward–punishment schemes by comparing trained vs. untrained responses to price cuts. | Critique (synchrony) | Yes | SAVED |
| **Johnson, Rhodes & Wildenbeest (2023).** "Platform Design When Sellers Use Pricing Algorithms." *Econometrica* 91(5):1841–1879 | With Calvano-style Q-learners, simple price-directed prominence fails. Dynamic, non-neutral demand steering (rewarding past price cuts) breaks demand rotation and collusion, giving very low prices. | Extends (policy) | Yes, Calvano demand | SAVED (TSE WP 1146, July 2021) `johnson-rhodes-wildenbeest-2021-tse-wp-…` |
| **Brown & MacKay (2023).** "Competition in Pricing Algorithms." *AEJ: Micro* 15(2):109–156 | Empirical: online retailers differ in pricing frequency. Theory: with asymmetric frequency and commitment to reaction functions, Markov equilibrium prices exceed Bertrand **without collusion**. | Alternative non-collusive mechanism | Model | SAVED (published) |
| **Assad, Clark, Ershov & Xu (2024).** "Algorithmic Pricing and Competition: Empirical Evidence from the German Retail Gasoline Market." *JPE* 132(3):723–771 | Adoption identified by structural breaks, instrumented by headquarters adoption. Margins rise only in non-monopoly markets. In duopolies, margins rise only if both adopt: about +28% (per Bichler). Margins rise gradually over about a year. | Empirical support (indirect) | Data, not ABM | SAVED (Jan 2021 WP) `assad-clark-ershov-xu-2021-wp-…` |
| **Assad, Calvano, Calzolari, Clark, Denicolò, Ershov, Johnson, Pastorello, Rhodes, Xu & Wildenbeest (2021).** "Autonomous algorithmic collusion: Economic research and policy implications." *Oxford Rev. Econ. Policy* 37(3):459–478 | Survey by the pro-concern camp | Survey | — | SAVED (TSE WP 1210) |
| **Calvano et al. (2019).** "Algorithmic Pricing: What Implications for Competition Policy?" *Rev. Ind. Org.* 55:155–171 | Policy precursor | Context | — | SAVED |
| **Calvano, Calzolari, Denicolò, Harrington & Pastorello (2020).** "Protecting consumers from collusive prices due to AI." *Science* 370:1040–1042 | Policy proposal | Context | — | not fetched |
| **Abada, Harrington, Lambin & Meylahn (2024/25 WP, R&R).** "Algorithmic Collusion: Where Are We and Where Should We Be Going?" SSRN 4891033 | Stock-take with evaluation criteria (aim: prevent collusion in real markets). Per the search summary: non-mean-based bandits converge to equilibrium; collusion arises only with *symmetric* UCB or Q-learning, not with heterogeneous algorithms. | Survey, critique-leaning | — | **NOT OBTAINED** |
| **den Boer & Meylahn (2024 WP).** "A (mathematical) definition of algorithmic collusion." SSRN 5012923; also den Boer (2023) SSRN 4636488 | Formal definition | Definitions | — | **NOT OBTAINED** |
| **Meylahn & den Boer (2022).** "Learning to Collude in a Pricing Duopoly." *MSOM* 24(5):2577–2594 | A *designed* Kiefer–Wolfowitz algorithm provably converges to joint-revenue maximization if profitable for both, else competitive. Collusion is easy when the algorithm is *designed* to collude. | Contrast case | Yes | **NOT OBTAINED** (INFORMS HTML) |
| **Sanchez-Cartas & Katsamakas (2022).** "Artificial Intelligence, Algorithmic Competition and Market Structures." *IEEE Access* 10:10575–10584 | Q-learning and PSO pricing on competing two-sided platforms; algorithms internalize network effects. Related: Sanchez-Cartas & Katsamakas, "AI pricing algorithms under platform competition", *Electronic Commerce Research* (2024) | Extends (platforms) | Partly | **NOT OBTAINED** (IEEE Access is open access but I did not fetch it; easy to get) |
| **Hansen, Misra & Pai (2021).** "Frontiers: Algorithmic Collusion: Supra-competitive Prices via Independent Algorithms." *Marketing Sci.* 40(1):1–12 | Symmetric UCB bandits *without* conditioning on rivals reach supra-competitive prices via correlated experimentation. The precursor of the "no strategies needed" line. | Mechanism | Yes | SAVED (NSF PAR) |
| **Douglas, Provost & Sundararajan (2026).** "The Illusion of Collusion." arXiv 2411.16574 | Context-free bandits in a repeated PD. "Naive collusion" is driven by synchronicity. Never in the long run with persistently random policies (constant-ε); sometimes with GLIE; always with deterministic UCB. | Critique/mechanism | Yes | SAVED |
| **Xiong, Chen & Gao (2024).** "Is Thompson Sampling Susceptible to Algorithmic Collusion?" arXiv 2405.17463 | Thompson sampling converges to Nash under a mild payoff condition | Contrast | Theory | SAVED |
| **Arunachaleswaran, Collina, Kannan, Roth & Ziani (2025, ITCS).** "Algorithmic Collusion Without Threats." arXiv 2409.03956 | No-regret first mover plus an optimizing second mover gives monopoly-like prices with no threats encoded | Expands the definition | Theory | SAVED |
| **Deng, Schiffer & Bichler (2025).** arXiv 2503.11270 (also WI 2024, arXiv 2406.02437) | Tabular Q-learning colludes most and is unstable. DQN and PPO end near Nash. Heterogeneous deep-RL pairs collude less. | Robustness critique | Yes | SAVED (2025 version) |
| **Frick (2026).** "Convergence to collusion in algorithmic pricing." arXiv 2604.15825 | Modern deep RL with continuous prices colludes in a time that matches the empirical timescale (Assad's about 1 year), about 100× faster than Calvano. Includes reward–punishment checks. Answers den Boer's timescale point. | Rebuttal | Yes | SAVED |
| **Keppo, Li, Tsoukalas & Yuan (2026).** "On the Fragility of AI Agent Collusion." arXiv 2603.20281 | LLM pricing agents: heterogeneity in patience cuts the price lift from 22% to 10%, and data asymmetry to 7%. More firms or LLM-vs-Q-learning pairs break collusion. | Robustness (LLM) | Yes | SAVED |
| **Carissimo, Falniowski, Rahimi & Nax (2025).** "Algorithmic Collusion is Algorithm Orchestration." arXiv 2508.14766 | Getting collusion requires co-parametrizing the algorithms, i.e., explicit coordination by designers. A meta-game analysis. | Critique (like den Boer) | Yes | SAVED |
| **Compte (2025).** "Learned Collusion." arXiv 2304.12647 | Q-based automata with learnable cooperation biases. Stable biases foster collusion independent of initial Q. | Theory | Yes | SAVED |
| **Han (2021).** arXiv 2102.09139 | Relative-performance experience replay: agents averse to underperformance converge to Nash; tolerant agents collude | Extension | Yes | SAVED |
| **Ye (2025).** arXiv 2502.15084 | Observed demand shocks: Q-learners learn procyclical pricing at high δ and countercyclical at low δ (Rotemberg–Saloner pattern) | Extension (supports genuine strategies) | Yes | SAVED |
| **Martin, Normann, Püplichhuisen & Werner (2025).** arXiv 2501.07178 | Asymmetric Cournot: algorithms reach the Pareto frontier, best described by "equal relative gains" | Extension | Yes | SAVED |
| **Kasberger, Martin, Normann & Werner (2026).** "Algorithmic cooperation: A comparison with human play in the infinitely repeated PD." *GEB* 159:113–130 | Q-learners vs. humans in IRPD | Extension | Yes | SAVED |
| **Werner (2022 WP).** "Algorithmic and Human Collusion" | Calvano-type Q-learners vs. humans; algorithms are more collusive in duopoly | Extension | Yes | SAVED |
| **Brero, Lepore, Mibuari & Parkes (2022, NeurIPS).** arXiv 2202.07106 | Platform learns buy-box rules (Stackelberg POMDP) that stop RL-seller collusion | Policy | Yes | SAVED |
| **Hartline, Long & Zhang (2024).** "Regulation of Algorithmic Collusion." arXiv 2401.15794 (CSLAW '24) | Audit definition of "plausible non-collusion" | Policy | Theory | SAVED |
| **Eschenbaum & Meylahn (2026).** "Auditing Algorithmic Collusion from Strategy Graphs." arXiv 2608.07098 | Graph metrics on frozen policies (max betweenness, attractor in-degree) correlate with Calvano's Δ | Diagnostic usable in our reproduction | Yes | SAVED |
| **Banerjee (2026).** arXiv 2303.02576 | Interpretable deep-RL collusion; an order-book mechanism shields undercutters from punishment | Policy | Yes | SAVED |
| **Luo, Schoepflin & Wang (2026).** arXiv 2602.17203 | Test-time meta-game over pretrained RL, UCB and LLM policies. Code: github.com/chailab-rutgers/CollusionMetagame | Robustness | Yes | SAVED |
| **Bichler, Durmann & Oberlechner (2025).** "Algorithmic Pricing and Algorithmic Collusion." *BISE* (arXiv 2504.16592) | Short survey linking to learning-in-games theory | Survey | — | SAVED |
| Ballestero (2026). "Algorithmic collusion under sequential pricing and stochastic costs." *IJIO* 106:103281 | A Klein follow-up; the other agent saved its Julia code | Klein line | Yes | (other agent) |
| Klein's own follow-ups | None found after RAND 2021. The earlier SSRN 3195812 title was "Assessing Autonomous Algorithmic Collusion: Q-Learning Under Short-Run Price Commitments". | — | — | — |
| Waltman & Kaymak (2008). "Q-learning agents in a Cournot oligopoly model." *JEDC* 32(10):3275–3293 | Memoryless Q-learners in Cournot produce below Nash quantities (collusive) | Precursor | Yes | already saved |

Also noticed, not fetched:
- Colliard, Foucault & Lovo, algorithmic market makers: Q-learning market makers keep supra-competitive spreads; per Xu & Zhao (2026), the cause is failure to learn.
- Musolff (2022), Amazon repricers.
- Calder-Wang & Kim (2024/26), RealPage rents.
- Kang, Kim & Kim (2022 ICIS), "Raising skepticisms on the feasibility of algorithmic tacit collusion".
- Wieting & Sapi (2021), Bol.com buy box.
- Hanspach & Calzolari (2025, JCLE), repricing industry.

---

### 4. AI-safety framing (brief)

- **Hammond et al. (2025)**, "Multi-agent risks from advanced AI", in `papers/multi-agent-coordination/`.
  - Collusion is one of three failure modes (miscoordination, conflict, collusion).
  - Calvano 2020 and Klein 2021 are cited as proof that independent RL learns to collude.
  - Case Study 4 is Assad's German gasoline market.
  - It also lists Abada & Lambin 2023 as *evidence* of collusion, which is a misreading.
  - It notes that the line between explicit and tacit collusion may dissolve for agents who can communicate in new forms.
- **Dafoe et al. (2020)**, "Open problems in cooperative AI", in `ai-coordination/`. Frames collusion as the dark side of cooperation.
- **Motwani, …, Hammond & Schroeder de Witt (2024/25 NeurIPS)**, "Secret Collusion among AI Agents: Multi-Agent Deception via Steganography", arXiv 2402.07510. **SAVED** `motwani-et-al-2024-neurips-secret-collusion-among-ai-agents-steganography.pdf`. LLM agents can collude covertly through steganographic channels; this is the safety analogue of "tacit".
- **LLM follow-ons:**
  - Fish, Gonczarowski & Shorrer 2024 (already saved);
  - Keppo et al. 2026 (fragility under heterogeneity);
  - Luo et al. 2026;
  - COLOSSEUM (arXiv 2602.15198, auditing collusion in cooperative multi-agent systems; not fetched).
- **Takeaway for framing.** Lessons transfer from the economics debate:
  - the exploration/synchrony artifact;
  - symmetric-design (orchestration) dependence;
  - fragility under heterogeneity;
  - IR tests that don't diagnose intent.
  For safety, this means "we observed supracompetitive/colluding outcomes" is a weak claim without counterfactual tests (upward-deviation probes, cross-pairing, asynchrony). This matches the playground's findings-first stance.

---

### 5. Not obtained (list for the user)

1. Abada & Lambin (2023) MS. DOI 10.1287/mnsc.2022.4623; SSRN 3559308. Their code *is* on Google Drive (link above).
2. den Boer, Meylahn & Schinkel (2022 SSRN 4213600 / MS 2026, DOI 10.1287/mnsc.2024.08557). Twente PDF behind Cloudflare.
3. Epivent & Lambin (2024) EL 237:111661. SSRN 4227229.
4. Lambin (2024 WP), "Less than meets the eye". SSRN 4498926.
5. Abada, Lambin & Tchakarov (2024) EJOR 318(3). SSRN 4099361.
6. Abada, Harrington, Lambin & Meylahn (2025 WP). SSRN 4891033.
7. Calvano et al. (2023) IJIO 90:102973, "Genuine or spurious?". SSRN 3928672 / CEPR DP16393.
8. Dolgopolov (2024) GEB 144:84–103. SSRN 4240842.
9. Asker, Fershtman & Pakes (2024) JEMS 33(2), the published long version.
10. Meylahn & den Boer (2022) MSOM 24(5). SSRN 3741385.
11. den Boer & Meylahn, definition WPs. SSRN 5012923, 4636488.
12. Sanchez-Cartas & Katsamakas (2022) IEEE Access 10 (open access, just not fetched) and (2024) ECR.
13. Assad et al. JPE 2024 final version (the 2021 WP is saved). Brown & MacKay is saved as the published version.

Highest priority to get by hand: **#2, #3, #4, #7**. These are the papers that make specific, testable claims against Calvano's own environment.

---------------------------------------------------------------------------------------------------

## 5. Three critiques of Calvano et al. (2020): reading notes for the collusion spec

Read 2026-10-01 from the local copies in `papers/ai-coordination/`. Page numbers are the papers' printed numbers. In den Boer et al. the PDF page is the printed page + 1, because of the cover sheet. Equations were checked against page renders wherever the text extraction was garbled. "Computed here" marks numbers I recomputed with `scratchpad/lambin_thm1.py`, using the spec's demand, grid and p^N/p^M.

---

### 1. Epivent & Lambin, "On algorithmic collusion and reward-punishment schemes" (SSRN 4227229, version of 17 Feb 2023; 19 pp.; published 2024 in *Economics Letters* 237:111661)

#### 1.1 Setup relative to CCDP
- **Model:** "faithfully replicates the setting in Calvano et al." (p. 4). Logit demand with outside good, c = 1, p^N ≈ 1.47, p^M ≈ 1.92, n = 2, one-period memory (p. 5).
- **Hyperparameters, p. 5 fn 3:** α = 0.15 and ε = exp(−βt) with β = 4 × 10⁻⁶. **δ is never stated.** The text implies 0.95 ("standard hyperparameters") but does not say so.
- **Convergence:** optimal strategy unchanged for 100 000 consecutive periods for each agent; cap of one billion iterations, which is the paper's 10⁹ and not the code's 1.25 × 10⁹ (p. 5).
- **Sessions:** **10 000**, against CCDP's 1 000. The paper says this is "the only difference" (p. 5).
- **Code:** not named and not said to be CCDP's Fortran. It says "We refer the reader to Calvano et al. [2020b] for further implementation details." The plots look like seaborn/matplotlib, so this is probably a Python re-implementation. Tie-breaking, Q-initialization and the RNG are all unstated.

#### 1.2 Protocols (exact)
**P1, the impulse response in both directions (p. 5; Fig. 1, p. 7).**
- For each session, take the converged prices at τ = 0. Play the learned strategies for τ < 10.
- At τ = 10, "exogenously force one of the algorithms to lower or raise its price (by an arbitrary amount)".
- Observe both agents from τ = 11 on, and average over sessions.
- Fig. 1 does not say how big the deviation is. Read off the figure, the average deviation price is about 1.565 for the cut, which looks like CCDP's static best response, and about 1.89 for the increase.
- Restrictions (fn 4, p. 6):
  - (i) "we restrict our exercises to the simulations that converge on a pre-shock price lower than the monopoly price". On our grid that drops pre-shock prices 1.93144 and 1.97019. Yet Table 1 still has rows for 1.93 and 1.97, so the restriction probably applies only to Fig. 1.
  - (ii) Cuts are excluded when the pre-shock price is already the grid minimum.
- How a pre-shock "price" is assigned for asymmetric points and cycles is **not stated**.

**P2, the full deviation matrix (Table 1, p. 8).** This is the counterpart of CCDP's Table 2.
- Rows are the pre-shock price: 1.62, 1.66, …, 1.97 (10 rows), each with a frequency column.
- Columns are every deviation price on the grid: 1.43 … 1.97 (15 columns).
- The cells come in two panels:
  - **(a)** the *relative* price change of the non-deviating agent at τ = 11 relative to τ = 10;
  - **(b)** the deviating agent's relative change at τ = 11 relative to its τ = 10 (deviation) price.
- The upper-right triangle (increments) is the new part. **This is the protocol for an upward deviation: every grid price above the pre-shock price, held for one period, with the response measured as the τ = 11 relative change.** It is not a one-step deviation.

**P3, the "invitation to collude" (p. 10, Fig. 2).**
- At τ = 10 the deviator is forced **one grid step up**.
- From τ = 11 the non-deviator is *exogenously forced to match* that price. Per the figure, it stays matched through τ = 30.
- At τ = 16 the deviator regains control.
- Also reported: "reversion to low prices is also observed when the nondeviating agent regains control in period 12 or later" (p. 10).
- Only sessions that converged to a point (cycles excluded) are used (fn 6, p. 10). The caption says "simulation run", but the jagged post-τ-16 curve looks like an average over the point sessions.

**P4, convergence below Nash (App. C, pp. 18–19).**
- The grid is shifted to "15 actions evenly distributed between 1.25 and 1.47". The upper bound is described as the Bertrand–Nash price, and everything else is as in the main text.
- P1 is repeated on the sessions that converge to singletons strictly below Nash.

#### 1.3 Quantitative results to reproduce or digitize
- **Average converged price p̃ = 1.79** (p. 5).
- **Table 1 (p. 8)** is fully legible in the text extraction. Transcribe it, don't digitize.
  - Row frequencies: 1.62: .02, 1.66: .05, 1.70: .10, 1.74: .15, 1.78: .16, 1.82: .16, 1.85: .14, 1.89: .10, 1.93: .06, 1.97: .03 (sum .97).
  - Panel (a), the non-deviator: **every** cell is negative, cuts and increments alike. Magnitudes run from −0.05 (row 1.62, deviation 1.97) to −0.18 (row 1.97, deviation 1.43). Increment cells are about −0.05 to −0.16.
  - Panel (b), the deviator: positive after large cuts (+0.08 to +0.10 after a deviation to 1.43) and negative after small cuts and all increments (down to −0.19).
  - In levels, both agents land near 1.55–1.62 at τ = 11 whatever the deviation (App. B, Fig. 5, p. 18). That figure plots the τ = 11 prices against the deviation price for pre-shock 1.66, 1.74, 1.82 and 1.89.
- **Fig. 1 (p. 7), digitized by eye.**
  - Fig. 1a, the cut: pre-shock ≈ 1.795 for both. At τ = 10 the deviator is ≈ 1.565. At τ = 11 the non-deviator is ≈ 1.555 and the deviator ≈ 1.57. τ = 12 ≈ 1.58, τ = 13 ≈ 1.64, τ = 14 ≈ 1.70, τ = 15 ≈ 1.74. Prices are back to ≈ 1.79 by τ ≈ 18–20.
  - Fig. 1b, the increment: pre-shock ≈ 1.785. Deviator ≈ 1.89 at τ = 10. Both ≈ 1.60 at τ = 11–12, then ≈ 1.65, 1.70, 1.74, back near 1.78 by τ ≈ 18.
- **Fig. 2 (p. 10):** pre-shock ≈ 1.785. At τ = 10 the deviator is at ≈ 1.825. τ = 11–15 both ≈ 1.825. **At τ = 16 the deviator drops to ≈ 1.59** and stays at 1.57–1.59 through τ = 30, while the forced non-deviator holds 1.825.
- **App. A, Table 2 (p. 16), the cycle census of 10 000 sessions:** 1-sym 2791, 1-asym 3622, period 2: 1833, period ≥ 3: 1100, "hybrid" (cycles of varying length) 654. Points therefore make up 64.1 %. Figs. 3–4 (p. 17) show that the point and cycle subsets give the same impulse shapes.
- **App. C (shifted grid):** **53 %** converge to Nash, **9 %** to cycles and **38 %** to singletons strictly below Nash. Fig. 6 uses **3831** sessions.
  - Fig. 6a, the cut: ≈ 1.465 → 1.438 at τ = 11, recovering by τ ≈ 16.
  - Fig. 6b, the increment: ≈ 1.465 → 1.43 at τ = 11, recovering by τ ≈ 17.
  - **Inconsistency to report:** sessions "strictly below Nash" should sit at or below the second-highest price, ≈ 1.454 (step 0.0157), yet Fig. 6's pre-shock average is ≈ 1.465. Either the top grid price is 1.47 < p^N = 1.47293, so top-price sessions count as "below Nash", or the grid is something other than stated.

#### 1.4 Testable claims
- E1: increments are followed by a price war. The non-deviator cuts at τ = 11 after every upward deviation, by 5–16 %, which is similar to after cuts (−9 to −18 %).
- E2: the deviator "self-punishes" at τ = 11 after every increment and every mild cut, but not after large cuts.
- E3: in the invitation protocol (P3), the deviator abandons the matched higher price immediately on regaining control and goes to ≈ 1.58.
- E4: the responses do not depend on recurrent-class size (points vs cycles).
- E5: on the shifted grid, 53/9/38 % converge to Nash / cycle / below Nash, and upward deviations toward Nash are punished there too.
- E6: 64.1 % of sessions converge to a point (27.9 % symmetric, 36.2 % asymmetric).

---

### 2. Lambin, "Less than meets the eye: simultaneous experiments as a source of algorithmic seeming collusion" (SSRN 4498926, 12 July 2024; 38 pp.)

#### 2.1 Setup relative to CCDP
- **Model:** "faithfully" / "exact replication" of CCDP (pp. 4, 5). Section 3 uses a general symmetric discrete game, with CCDP's numbers in Section 4.
- **Hyperparameters:**
  - δ = 0.95 throughout (fn 4, p. 5).
  - α = 0.15 (fn 12, p. 33).
  - β = 4 × 10⁻⁶ (fn 9, p. 21), except the entry experiment, which uses β = 4 × 10⁻⁵.
- **Code:** not stated. There is no mention of CCDP's Fortran.
- **Sessions:**
  - Fig. 3 and Fig. 10: 1 000.
  - Fig. 5: 1 000.
  - Fig. 6: 200.
  - Fig. 1: unstated.
  - Figs. 2, 4 and 9 are single runs.
- **Horizon:** fixed-length runs (1.5 × 10⁶ periods in Figs. 3 and 10; 2 × 10⁶ in Fig. 1; 4 × 10⁶ in Fig. 6), not stopped at convergence. The outcome is the **average greedy price over time**, not the converged Δ.
- **Q-initialization:**
  - The theory section (fn 8, p. 11) says "We choose random initialization …", calling it a linear increasing transform of CCDP's.
  - **Inferred from the figures:** the numerical runs also start from a non-CCDP initialization. At t = 0, Figs. 1, 3, 6 and 10 start at an average greedy price of ≈ 1.70, which is the mean of the grid (computed here: 1.69895). That is what random or zero (tied) initialization gives. CCDP's eq. 8 would start at 1.58271.
  - This contradicts "exact replication". Report it as an inference, since the paper never states it.

#### 2.2 The "key robustness test" (pp. 4–6)
- The test is **memoryless agents (k = 0, a single state) with δ kept at 0.95.**
- CCDP-A tested k = 0 only *jointly* with δ = 0. It wrote "With memoryless algorithms (k = 0), there is no loss of generality in setting the discount factor δ to 0." Lambin says that statement is "incorrect": a positive δ changes the Q-update dynamics even with one state (fn 4, p. 5).
- Result, Fig. 1 (p. 6), the average greedy price over 0–2 × 10⁶ periods:
  - (a) k = 1: ≈ 1.70 at t = 0, a dip to ≈ 1.60, a hump ≈ 1.64 at 0.4 × 10⁶, a trough ≈ 1.61 at 0.6 × 10⁶, then a rise to ≈ 1.75–1.76 by 2 × 10⁶.
  - (b) k = 0: ≈ 1.70 at t = 0, a dip to ≈ 1.59 at 0.2 × 10⁶, a steady rise to ≈ 1.80 at 1 × 10⁶, and ≈ 1.85 at 2 × 10⁶.
  - The claim: "the memoryless version … seem[s] to collude even more effectively than algorithms with memory" (p. 6).
- The same holds in App. A for Klein (2021) and Calvano et al. (2023). In Fig. 8 (p. 32), at δ = 0.95 the memoryless final price is ≈ 1.73 against ≈ 1.56 with memory (the two curves cross near δ ≈ 0.77), under rectangular demand with WTP 2, c = 1 and 15 prices.

#### 2.3 Theory: equations (pp. 9–14)
- **Update (eq. 1):** CCDP's eq. 4, asynchronous, with ε-greedy uniform exploration.
- **Mean-field assumption (p. 9; theory only):** an agent receives the expected profit against the rival's exploration distribution.
- **Definitions:**
  - π̃(p) = E_q π(p, q), the profit against a uniformly random rival.
  - Actions are ranked so that π̃(a₁) > π̃(a₂) > … and π̃ = π̃(a₁).
- **Two-phase exploration:** ε = 1 for T₁ periods, then ε = 0 for T₂ periods.
- **Lemma 1 (ε = 1, T₁ → ∞):** Q(aᵢ) → π̃(aᵢ) + δ/(1−δ)·π̃, so the greedy action is a₁.
- **Lemma 2 (ε = 0):** Q(a₁) → π(a₁, a₁)/(1−δ), while the other Q-values are frozen at their Lemma-1 values.
- **Theorem 1:** let J be the smallest j with max_{i≤j} π(aᵢ, aᵢ) > (1−δ)π̃(a_{j+1}) + δπ̃ (eq. 5). Agents then converge to **I = argmax_{i≤J} π(aᵢ, aᵢ)** (eq. 6). Supra-competitive play needs no reward–punishment and survives δ = 0.
- **Eq. 7, the reaction function after learning:**
  - from a symmetric state below I, play the next action on the path;
  - from the symmetric state at I, play a_I;
  - from **any asymmetric state, or a symmetric state with i > I, play a₁** (the stale greedy action from the exploration phase).
  - So any unilateral deviation produces a₁, then a walk up to a_I. This is the "seeming punishment".
- **Computed here for the CCDP grid:**
  - a₁ = 1.58271 (grid index 4).
  - The π̃ ranking is grid indices 4, 3, 5, 2, 6, 7, 1, 8, ….
  - δ = 0.95: J = 8 and **I = 1.73770 (index 8), π(a_I, a_I) = 0.31389, Δ = 0.794**. This matches the paper's "p_conv = 1.74".
  - δ = 0: J = 6 and I = 1.69895 (index 7), **Δ = 0.707**.
- **App. B, eq. (9), the fumbling bound:** t̃ᵢ < ⌊ln[((1−δ)π̃(aᵢ) + δπ̃ − π(aᵢ, aᵢ)) / ((1−δ)(π(a_I, a_I) − π(aᵢ, aᵢ)))] / ln(1/α)⌋ + 1. The total is T_fum = Σ_{i≤J, i≠I} t̃ᵢ.
- **Internal inconsistency to report:** the proof of Lemma 2 writes Qᵗ = α^{t−T₁}(…) + …, and eqs. 8–9 and Theorem 2 inherit that ln(1/α). But eq. 2's own recursion contracts by (1 − α(1 − δ)) = 0.9925 per visit, not by α = 0.15. Eq. 9's denominator also carries an extra (1 − δ).
  - With the bound as printed, I get T_fum = 14 periods.
  - With the contraction factor corrected and the denominator as printed, I get ≈ 2 775.
  - With both corrected, each action needs about 1 visit.
  - The qualitative "fumbling is short" claim survives. Eq. 9 itself should not be used as a prediction; measure fumbling directly.
- **App. C, turn-based exploration (Def. 1):** agents alternate, in blocks of T periods, which one may explore with probability ε. Theorem 2: if T > ln(2η̄_ε/η_ε)/ln(1/α) (the same α issue) and play converges to a fixed point, that point is the static Nash.

#### 2.4 Numerical protocols and results
- **Fig. 2 (p. 16), stateless two-phase.**
  - The text gives ε = 1 for 1 000 steps; the caption says "switch … at time t = 10", which is a plotting offset or an inconsistency.
  - The initial Q-values match the Lemma-1 values (computed here: 1.58 → 6.2780, 1.43 → 6.2536, 1.78 → 6.2437, all as in the plot).
  - The greedy action starts at 1.58 and "after less than 30 periods" settles at **1.74**, whose Q then rises toward π(1.74, 1.74)/(1−δ) = 6.2777.
- **Fig. 3 (p. 18) and Fig. 10 (p. 38), the n-step approximations of e^{−βt}.**
  - n = 2, 3, 5, 20 and 50 steps, 1 000 sessions, 1.5 × 10⁶ periods; Fig. 3 is k = 0 and Fig. 10 is k = 1.
  - The step construction is **not stated**. Read from the plots: the horizon 1.5 × 10⁶ is cut into n equal intervals, the first at ε = 1 and interval k ≥ 2 at e^{−β·midpoint}. For n = 3 the second step is ≈ 0.05 = e^{−3}, and for n = 5 it is ≈ 0.165 = e^{−1.8}.
  - k = 0 endpoints (average greedy price at 1.5 × 10⁶): 2-step ≈ 1.88 (band 1.80–1.93); 3-step ≈ 1.83; 5-step ≈ 1.85; 20-step ≈ 1.86; 50-step ≈ 1.87.
  - k = 1 endpoints: 2-step ≈ 1.87; 3-step ≈ 1.88; 5-step ≈ 1.85; 20-step ≈ 1.80; 50-step ≈ 1.79.
  - Before the first step, every panel sits flat at ≈ 1.60–1.61.
- **Fig. 4 (p. 20), seeming punishments.**
  - One session, k = 1, δ = 0.95, T₁ = T₂ = 1 000, pre-deviation price 1.74 for both.
  - Agent 1 is forced at t = 3 to 1.54, 1.62, 1.70, 1.79, 1.85 or 1.93.
  - **Whatever the deviation, the path is identical afterward:** t = 4 both at 1.58, t = 5 both at ≈ 1.70, t = 6 both at 1.74. Agent 2 holds 1.74 at t = 3.
- **"These patterns are also observed when the discount factor δ is zero"** (p. 20). No figure is given.
- **Fig. 9 (p. 36), k = 1 two-phase:** noisy play until **t = 224**, then 1.74 (the text says "after about 250 periods").
- **§5.1 and Fig. 5 (pp. 21–22), sequential entry.**
  - The incumbent learns alone; entry comes at T = 100 000, with β = 4 × 10⁻⁵ (fn 9) and k = 1.
  - After entry the incumbent cannot explore but keeps updating.
  - Average over 1 000 sessions: ≈ 1.88–1.89 before entry, falling to ≈ 1.58 by 150 000 and ≈ 1.52 by 250 000.
  - Unstated: the pre-entry demand (a monopoly logit with the outside good?), whether the entrant's ε clock starts at entry, and what the incumbent's state is before entry.
- **§5.2 and Fig. 6 (pp. 22–23), turn-based remedy.**
  - CCDP is run for 2 × 10⁶ periods. From then on **agent 2 only** explores with constant ε_min = 0.005 (6a) or 0.05 (6b). 200 sessions, k = 1.
  - 6a: peak ≈ 1.85 at ≈ 1.2–1.5 × 10⁶, declining to ≈ 1.51 by 3.5–4 × 10⁶.
  - 6b: drops to ≈ 1.50 within about 0.1 × 10⁶ of the switch.
- **Appendix A:** Fig. 7 (Klein 2021: six prices, linear demand, homogeneous goods) shows stateless profits above k = 1. Fig. 8 (Calvano et al. 2023) shows the k = 0 price above k = 1 for δ ≳ 0.75.

#### 2.5 Testable claims
- L1: k = 0 with δ = 0.95 gives prices **at least as high as** k = 1 (Fig. 1).
- L2: under two-phase exploration, both agents converge to I from Theorem 1: 1.7377 at δ = 0.95 and 1.6990 at δ = 0.
- L3: after any unilateral deviation from a symmetric fixed point, the next play is a₁ = 1.58271 by both, then a walk back to I (Fig. 4). This holds for upward deviations and at δ = 0.
- L4: under two-phase exploration, convergence takes tens of periods (k = 0) or a few hundred (k = 1).
- L5: as the number of steps grows, the step approximation recovers CCDP's greedy-price path (Figs. 3 and 10).
- L6: sequential entry and turn-based exploration (one-sided ε floor) both bring prices close to Nash.
- L7, the overreach to check: E&L's punishment price "is always the same … 1.58" (p. 19). E&L's own Table 1 puts the non-deviator's τ = 11 price between ≈ 1.46 and ≈ 1.69 depending on the row, so that claim needs checking.

---

### 3. den Boer, Meylahn & Schinkel, "Artificial Collusion: Examining Supracompetitive Pricing by Q-learning Algorithms" (Amsterdam Law School RP 2022-25 / ACLE WP 2022-06; **version dated 19 Feb 2026**; 47 pp.)

The local filename says 2022; the docs' Wanted list says "2026". It is the same paper, and the copy we hold is the 2026 revision.

#### 3.1 Setup relative to CCDP
- **Model (pp. 6–8):**
  - Logit demand with the 1 in the denominator (a₀ = 0 is WLOG).
  - c = 1, a = 2, μ = ¼, δ = 0.95.
  - p^N = 1.4729 and p^M = 1.9250 in closed form, via the Li & Huh V(·) and Lambert-W expressions.
  - π^N = 0.2229, π^M = 0.3375.
  - Grid Aᵢ = {pᴺ − ξ(pᴹ − pᴺ) + k(1 + 2ξ)(pᴹ − pᴺ)/(m − 1)}, which for CCDP gives {1.4277, … step 0.0387 …, 1.9702}.
  - Q₀ from CCDP's eq. 8.
  - **Ties broken uniformly at random**, as the code does (p. 7).
  - ε_t = exp(−βt).
- **Grid sweep (p. 8):** α ∈ [0.025, 0.25] and β ∈ [0, 2 × 10⁻⁵], 100 × 100 points, 1 000 sessions each. Results are not shown as such.
- **Code:** their own. Validated by reproducing CCDP's Fig. 3 (Δ vs δ, 100 trajectories per δ; App. A, Fig. 5, p. 33). Nothing indicates the Fortran.
- **Convergence (fn 10, p. 9):** CCDP's criterion with t₀ ≤ 10⁹ − 10⁵.
- **π̄ (fn 11):** "the observed average profit over the time periods t₀+1, …, t₀+100 000". That is the realized profit with residual exploration, **not** CCDP's limit-cycle replay. Fig. 10 likewise computes "Δ over the last 10⁵ time periods". This is a definitional difference from CCDP and from our spec.
- **The m = 2 reduction (§4.2, pp. 12–17):** prices {D = pᴺ, C = pᴹ} (m = 2, ξ = 0), β = 10⁻⁴, α = 0.15, δ = 0.95, CCDP initialization, 1 000 sessions.
  - Payoffs: R = 0.3375, S = 0.1180, T = 0.3679, P = 0.2229 (verified here).
  - Strategies are 4-letter words over the states (CC, CD, DC, DD).

#### 3.2 Definitions and equations
- **Collusive equilibrium (fn 4, p. 3):** a strategy profile that is an equilibrium (mutual best responses) **and** contains a reward–punishment scheme.
- **Three-plus-two criteria (p. 5):**
  1. learned on timescales relevant to the firm's objective;
  2. not by mistake or incomplete learning (close to equilibrium);
  3. contains a reward–punishment scheme;
  4. performs well against reasonable alternative algorithms;
  5. needs no coordination by the firms.
- **Discounted gain from the start (p. 9):** Δ̃ = (1 − δ)E[Σ_{t≥1} δ^{t−1}π_i(t) − π^N] / (π^M − π^N). Compare CCDP's Δ = (π̄ − π^N)/(π^M − π^N).
- **Effective horizon (p. 10):** T_δ = log(π_min/(1000 π_max))/log δ. Profits after T_δ contribute < 0.1 %.
  - π_min = 0.0911, π_max = 0.4270, so **T_0.95 = 165** (computed here: 164.8).
  - T_0.99 = 842, T_0.999 = 8449, T_0.9999 = 84516 (computed: 841.0, 8447.8, 84516.0; the small mismatches are rounding).
  - With 15³ = 3 375 Q-values per firm, **at most ≈ 5 % can be updated within T_δ**.
- **Within T_δ, Q-learning ≈ uniform random pricing (p. 11):** the chance of any greedy choice in the first 165 periods is ≤ 1 − e^{−2×10⁻⁵·165} ≈ 0.0033 (for β = 4 × 10⁻⁶ it is ≈ 0.00066).
  - Uniform random pricing on CCDP's grid gives an average price of **1.69895** and profit **0.279906**, so Δ̃ = **0.497** (Table 5, p. 40; verified here).
  - On the symmetric grids: Ã (ξ = 0) gives price 1.47293, profit 0.164479, Δ̃ = −0.510. Â gives 1.47293, 0.172288, Δ̃ = −0.442.
  - Grids (App. F, p. 39): Ã = {pᴺ − (ξ+1)ζ + k·2(1+ξ)ζ/(m−1)}, k = 0..m−1, and Â = {c + k·2(pᴺ − c)/(m+1)}, k = 1..m, where ζ = pᴹ − pᴺ.
- **(δ, ε)-best response and equilibrium (eqs. 3–4, p. 12):** a Bellman equation in which the rival plays its strategy with probability 1 − ε and is uniform with probability ε. The firm's *own* exploration is excluded (fn 17).
- **Table 1 (p. 13), equilibrium conditions for m = 2.** The formulas are taken from the render and reproduce the paper's critical values here.
  - AD always.
  - GT (CDDD): [ε(S+T−R−P) + 2(R−T)] / [(ε²−3ε+2)(P−T)] < δ < [ε(S+T−R−P) + 2(P−S)] / [(1−ε)(ε(T−P) + 2(P−S))].
  - WSLS (CDDC): δ > [ε(P+R−S−T) + 2(T−R)] / [(1−ε)(ε(P+S−T−R) + 2(R−P))].
  - At δ = 0.95: **GT exists for ε < 0.515126 and WSLS for ε < 0.292042.** With β = 10⁻⁴ that means t > **6 633** and t > **12 308** (verified here: 6633.4 and 12308.6).
- **Theorem 1 (p. 14; proof in App. G):** for every δ there is ε(δ) < 1 such that, for ε above it, the only (δ, ε)-equilibrium is σ* ≡ p* = 1.58271, the best price against a uniform rival (which is CCDP's initial greedy price).
- **Reward–punishment, the operational definitions:**
  - **RP strategy (m = 2, p. 18):** plays C in (C, C) and D in (C, D).
  - **CCDP's pattern (a, b, c), as den Boer describes it (p. 18):** (a) a forced cut is followed by a rival price decrease; (b) prices return to the original level after some periods; (c) the deviator's total discounted profit is lower than without the deviation.
  - **Their systematic criterion (p. 21):** a session is supported by a reward–punishment scheme only if "**all possible unilateral deviations by both players** lead to a path through the strategy graph that contains a punishment phase and a phase in which the supra-competitive prices are reestablished (forgiveness)".
  - **§5, fourth point (p. 26):** "all possible deviations from a collusive price should be followed by a path that returns to a supra-competitive price". This excludes GT, which has no recovery.
- **Exp3 (App. H, pp. 43–45):**
  - Pₜ,ₚ ∝ exp(η Σ_{s<t} δ^s X̂_{s,p}), with X̂ = 1 − Ŷ, Ŷ = Yₜ·1{pₜ = p}/Pₜ,ₚ, and Yₜ = 1 − πᵢ(pₜ).
  - Theorem 2 gives η = √(log|A|·(1−δ)² / (δ²|A|)), with regret ≤ 2√(δ²/(1−δ²)·|A| log|A|).
  - **Inconsistency to report:** the proof's final step (log|A|/η + η|A|δ²/(1−δ²)) is minimized at η = √(log|A|·(1−δ²)/(δ²|A|)), not at (1−δ)². For |A| = 15 and δ = 0.95 the two give η = 0.0224 (theorem) against 0.1396 (proof).
  - The δ^s weights mean that late observations count exponentially less, so the policy effectively freezes after ~1/(1−δ) periods.

#### 3.3 Quantitative results
- **m = 2, Fig. 2 (p. 15), digitized by eye.** Profile fractions over t ∈ [0, 80 000], 1 000 sessions.
  - AD rises to ≈ 0.82 before t = 12 308, then falls to ≈ 0.04 by 40 000.
  - "Other" peaks at ≈ 0.62 near 20 000 and ends at ≈ 0.53.
  - WSLS is ≈ 0 before 12 308 and plateaus at ≈ 0.41 from about 50 000.
  - GT stays ≈ 0.02.
  - The (C, C) share of the last 1 000 periods rises to ≈ 0.74.
  - The right panel is player 1 only: AD ≈ 0.88 peak, ending at 0.048; other ≈ 0.48; WSLS + GT ≈ 0.47.
- **m = 2 final numbers (p. 17):**
  - Profiles: WSLS **40.9 %**, GT **2.4 %**, AD **3.6 %**, other **53.1 %**.
  - Per player: AD 4.8 %, other 48.1 %.
  - Both players RP: **46 %**.
  - (C, C) limit share: **74 %**.
  - "44.7 % of supra-competitive limit prices are not generated by collusive equilibria".
- **Table 3 (p. 36), α sweep (m = 2, Wilson intervals):**

  | α | WSLS | GT |
  |---|---|---|
  | 0.05 | 0.28 ± .03 | 0.002 ± .002 |
  | 0.10 | 0.50 | 0.03 |
  | 0.15 | 0.43 | 0.03 |
  | 0.20 | 0.39 | 0.02 |
  | 0.25 | 0.34 | 0.02 |

- **Table 4 (p. 36), β sweep:** WSLS + GT is 0.55 at β = 10⁻⁶, 0.54 at 10⁻⁵, 0.46 at 10⁻⁴, 0.16 at 10⁻³ and 0.01 at 10⁻². The corresponding T_ε₀,β are 1 230 860, 123 086, 12 309, 1 231 and 123. Figure 8 (p. 37) shows the time paths.
- **Variant (i), ε₀ e^{−βt} with ε₀ = 0.292042 (Fig. 7):** WSLS ≈ 36 %.
- **Variant (iii), constant ε = 0.1 (Fig. 9):** collusive profiles < 20 %.
- **Fig. 11 (p. 39), m = 2 fractions against δ ∈ [0.80, 1).**
  - Left, β = 10⁻⁴: AD is .94 at .80, .80 at .85, .47 at .90, .20 at .925 and ≈ .03 at .95. WSLS is ≈ .01 at .80, .16 at .90, .30 at .925, **.41 at .95**, .31 at .98 and .29 at .99, then ≈ .04 at the last point. GT peaks at ≈ .10 around .90–.925.
  - Right, β = 10⁻⁵: AD is ≈ 1.0 until .85, .86 at .90, .15 at .95 and 0 at .98. WSLS is .09 at .90, .50 at .95, **.62 at .98**, then ≈ .18 at the last point.
- **Fig. 10 (p. 38), m = 15, Δ over the last 10⁵ periods, 1 000 sessions per δ.**
  - Left, the CCDP grid: Δ ≈ 0.927 flat from δ = 0.9975 to 0.999, then 0.885 at ≈ 0.9999, ≈ 0.83 at ≈ 0.99999 and ≈ 0.78 at 0.999999 (2.4 % of sessions unconverged there).
  - Right, the Ã grid with ξ = 0, δ ∈ [0.975, 1): 0.55, 0.53 (0.99), 0.42 (0.995), 0.27 (0.999), ≈ 0.16 (last point).
  - **Note:** the left panel's ≈ 0.93 is above CCDP's ≈ 0.85–0.87 at δ = 0.95, which fits Δ being measured with residual exploration over a window.
- **Fig. 5 (p. 33):** a replication of CCDP's Fig. 3 (Δ against δ, 100 sessions per δ), with the minimum near δ ≈ 0.3 and > 0.8 at high δ.
- **Fig. 4 (p. 20), one m = 15 trajectory with β = 10⁻⁵:** the limit point is (6, 5), where index 0 is the lowest grid price (the paper calls it "the Nash price", but it is 1.4277).
  - Paths: (6,5)→(1,5)→(4,2)→(5,2)→(1,2)→(2,1)→(6,5), which punishes and forgives.
  - (6,5)→(6,2)→(9,2)→(6,5): the rival *raises* its price.
  - (6,5)→(2,5)→(1,0)→(9,1): punished, but never returns.
- **Table 2 (p. 24), Δ̃ over T_δ for row against column, m = 15, α = 0.15, β = 4 × 10⁻⁶, 1 000 sessions, 95 % CI:**

  | δ | Exp3–Exp3 | Exp3–Q | Q–Exp3 | Q–Q |
  |---|---|---|---|---|
  | 0.95 | .49 | .50 | .48 | .50 |
  | 0.99 | .47 | .52 | .46 | .50 |
  | 0.999 | .44 | .56 | .38 | .50 |
  | 0.9999 | .37 | .62 | .26 | .49 |

  - Tables 6 and 7 (pp. 40–41) give the same for the Ã and Â grids. All entries are negative; for Q–Q at δ = 0.95 they are −0.51 (Ã) and −0.45 (Â).
- **App. B, Fig. 6 (p. 35), offline-trained cross-play:** moving-average Δ over 500 periods. Exp3 uses η = √(log M/(MT)). It is a prisoner's dilemma before ≈ 13 500 and a stag hunt after. Whether learning continues during the test is not stated.
- **Counterexample (p. 18):** strategies CCDC against CDCC give pattern (a, b, c) for δ > 0.2653 without being an equilibrium or an RP profile.

#### 3.4 Testable claims
- D1: within T_δ, baseline Q-learning earns Δ̃ ≈ the uniform-random 0.497. On grid Ã it earns ≈ −0.51.
- D2 (m = 2): no GT before t = 6 633 and no WSLS before t = 12 308 (β = 10⁻⁴). The final profile shares are 40.9 / 2.4 / 3.6 / 53.1 %, with 46 % RP and 74 % (C, C).
- D3: the β, α and δ dependencies in Tables 3–4 and Fig. 11 (collusive equilibria vanish as δ → 1).
- D4 (m = 15): Δ falls as δ → 1 (Fig. 10).
- D5: a one-shot-deviation pattern (a, b, c) does not imply an RP or equilibrium profile. Many converged profiles fail "all deviations punished and forgiven".
- D6: Exp3 beats Q-learning against Q-learning for δ ≥ 0.99 (Table 2), so Q-learning is not an equilibrium of the algorithm meta-game.

---

### 4. Recommended changes to the spec

Each item quotes the current spec line and gives a replacement or addition.

#### 4.1 Source texts
- Current: "Epivent & Lambin (2024, *Economics Letters*) and Abada & Lambin (2023, *Management Science*), both known only from abstracts and citing papers until found"
  - Replace with: "Epivent & Lambin (SSRN 4227229, Feb 2023 version, the working paper of the 2024 *Economics Letters* article; local copy), Lambin (2024, SSRN 4498926; local copy), den Boer, Meylahn & Schinkel (Amsterdam LSRP 2022-25, revision of 19 Feb 2026; local copy). Abada & Lambin (2023) is still known only from abstracts."
- The Docs "Wanted" list should drop these three.

#### 4.2 Config: new fields and changed readings
- **`impulse`.** Current: "`up` (one period one grid step above the pre-deviation price: Epivent & Lambin's reading, the step is ours)".
  - Replace with: "`every_price` (CCDP-A's, and Epivent & Lambin's Table 1: one period at each grid price, below and above; their measure is each agent's relative price change at τ+1 against τ), `invitation` (Epivent & Lambin's Fig. 2: the deviator goes one grid step up for one period, the rival is forced to match from the next period on, and the deviator regains control after `invitation_hold` = 5 periods; `invitation_release` = `deviator` (the default) or `rival` (their 'period 12 or later' variant))".
  - Keep `up` only as a named convenience (one step up). It is not the paper's protocol.
  - Run `impulse` from τ = 0 with 10 periods of on-path play before the deviation (E&L's τ = 10), so that their figures align. For cycles the deviation phase is then fixed, so state it.
- **`exploration`.** Add:
  - `two_phase`: ε = 1 for `t1` periods, then 0; Lambin uses `t1` = 1 000, and T₂ = 1 000 in Fig. 4.
  - `steps`: an n-step approximation of e^{−βt} over a horizon H. Our reading of Lambin's Figs. 3 and 10, which the paper does not state: n equal intervals of [0, H], the first at ε = 1 and interval k ≥ 2 at e^{−β·midpoint}, with H = 1.5 × 10⁶ and n ∈ {2, 3, 5, 20, 50}.
- **`epsilon_floor`, `floor_firm` and `floor_from`** (Lambin §5.2, turn-based remedy): from period `floor_from` (2 × 10⁶), firm `floor_firm` (2) explores with at least ε_min (0.005 or 0.05). Optionally add `turns` = T (Lambin's Def. 1: firms take turns exploring in blocks of T periods).
- **`entry`** (Lambin §5.1): firm 2 enters at `entry_at` (100 000); after entry the incumbent does not explore.
  - Our reading of what the paper leaves unstated: before entry, the incumbent faces logit demand with only its own product and the outside good, and its state is (own price, a fixed placeholder for the absent rival). The entrant's ε clock starts at entry.
  - Lambin uses β = 4 × 10⁻⁵ here.
- **`grid`:** `calvano` (the default), `symmetric` (den Boer's Ã, ξ = 0), `cost_to_nash` (den Boer's Â) and `below_nash` (Epivent & Lambin App. C: 15 prices evenly from 1.25 to the upper bound).
  - Default the upper bound to 1.47, as their text has it; switch `below_nash_top = nash` to use 1.47293. Report which one reproduces their 53/9/38 %.
- **`q_init`:** add `random`, Lambin's theory, footnote 8. His numerical figures start at the grid mean, which is consistent with random or zero initialization (see 2.1). The range is unstated, so use `optimistic`'s [`q_low`, `q_high`] with a stated default.
- **`delta` with `memory = 0`:** state explicitly that `memory` = 0 keeps δ, which is Lambin's key test. CCDP-A's memoryless case is `memory = 0` with `delta = 0` (and β = 10⁻³ as in the code, 10⁻⁴ as in the text).
- **New statistics:**
  - `greedy_price`: the mean over firms of the greedy price at the visited state. For k = 0 there is only one state. Add the variant `greedy_price_all`, averaged over all states.
  - `discounted_gain` (den Boer's Δ̃ from t = 1, truncated at T_δ, with T_δ reported).
  - `window_gain` (den Boer's and Epivent & Lambin's reading of π̄: realized profit over the last 10⁵ periods, against our limit-cycle Δ).
  - **Learning-inertia measures:**
    - (i) `stale_share`: the share of each firm's (state, price) cells last updated while ε > ½, and while ε > 0.01, at convergence.
    - (ii) `stale_greedy`: the share of off-path states whose greedy price is still a₁ = 1.58271, the initial greedy price.
    - (iii) `q_bias`: the mean of Q(s, a) − Q^true(s, a) over non-greedy cells, where Q^true comes from value iteration against the rival's converged strategy (we already compute it for `equilibrium_check = best_response`).
    - (iv) `fumbling`: the periods between the end of exploration and the last greedy change.

#### 4.3 Survey B: protocol changes and thresholds
- **The "punishment-like response" definition.** Keep it as written as the per-deviation unit. Also report:
  - (a) Epivent & Lambin's continuous measure: the mean relative change of each agent at τ + 1, by (pre-shock price, deviation price) cell.
  - (b) den Boer's condition (c): the deviator's discounted profit over the response is lower than without the deviation.
  - (c) den Boer's session-level criterion. Add: "A session is **RP-complete** (den Boer et al., p. 21) if every unilateral one-period deviation by either firm to any other grid price, from every state of the limit cycle, gives a punishment-like response; it is **recovering** (their §5) if every such deviation is followed within 25 periods by a return to a cycle with Δ > 0."
- **B1.** Current: "| B1 | memory 0 (no state, so no punishment is possible) | Δ(k = 0) ≥ ½ Δ(k = 1) |"
  - Replace with: "| B1 | memory 0 with δ = 0.95 (Lambin's 'key robustness test'), β and α as baseline; the CCDP-A reading (memory 0, δ = 0) reported beside it | Lambin's claim holds if the mean greedy price at 1.5 × 10⁶ and 2 × 10⁶ periods **and** the converged Δ are each at least as high at k = 0 as at k = 1 (difference ≥ −2 SE); the weaker critique holds if Δ(k = 0) ≥ ½ Δ(k = 1) |"
  - Lambin's own criterion is "≥", not "≥ ½". Keep the ½ rule as the secondary verdict, since it was set before measuring.
- **B2.** Current: "| B2 | δ = 0 (no future to protect) | Δ(δ = 0) > 0.1 … |"
  - Add a row **B2b**: "δ = 0 under `exploration = two_phase` (t1 = 1 000), memory 0 and 1: Lambin's Theorem 1 predicts convergence to 1.6990 (Δ = 0.707); at δ = 0.95 to 1.7377 (Δ = 0.794). Holds if ≥ 80 % of sessions end with both firms at I [threshold ours]; Lambin's 'patterns also observed when δ is zero' (p. 20) is tested with `impulse = every_price` under the punishment-like rule."
- **B3.** Current: "| B3 | an upward deviation | punishment-like responses after an upward deviation in at least half as many sessions as after the paper's downward one |"
  - Replace with: "| B3 | Epivent & Lambin's Table 1: from each converged session, a one-period deviation to every grid price above and below the pre-shock price (sessions with pre-shock ≥ p^M excluded from the increments; cuts from the grid minimum excluded) | their claim holds if, in every (pre-shock row, upward deviation) cell with ≥ 30 sessions, the non-deviator's mean relative change at τ+1 is negative, and the mean across upward cells is at least half the mean across downward cells; the original rule (punishment-like responses after an upward deviation in at least half as many sessions as after the downward one) is reported beside it |"
  - E&L give no threshold, so "≥ 30 sessions" and "half" are ours. They are written now, before measuring.
  - Add a row **B3c**: "Epivent & Lambin's invitation (Fig. 2): `impulse = invitation` on point sessions; holds if the deviator's mean price in the period it regains control is below its pre-shock price by at least one grid step."
- **B3b.** Current: "| B3b | sessions that settle below p^N (Epivent & Lambin) | punishment-like responses in them at least half as often as in sessions above |"
  - This misreads the paper. E&L get below-Nash sessions only on a **shifted grid**; on CCDP's grid only one price (1.4277) is below p^N.
  - Replace with: "| B3b | `grid = below_nash` (15 prices from 1.25 to 1.47), 10 000 sessions as the paper | report the shares converging to the top price / cycles / singletons below it against their 53 / 9 / 38 %; in the below-Nash singletons, their claim holds if upward deviations get punishment-like responses at least half as often as downward ones |"
- **B5.** Current: "| B5 | more exploration: `constant` ε = 0.05, and β ten times lower | Δ falls by more than half under either |"
  - Add a row **B5b**: "Lambin's turn-based remedy: baseline to 2 × 10⁶, then `epsilon_floor` 0.005 or 0.05 on firm 2 only; holds if Δ at 4 × 10⁶ is less than half its value at 2 × 10⁶."
  - Add a row **B5c**: "Lambin's sequential entry (`entry_at` = 100 000, β = 4 × 10⁻⁵): holds if Δ at 250 000 is below ½ the baseline Δ."
  - Note for the write-up: den Boer's constant ε = 0.1 (m = 2) drops collusive profiles below 20 %.
- **B6.** No change to the rule. Cite den Boer's App. B, which is the same exercise with an Exp3 arm. Our reading of "greedy play from a random state" differs from their (unstated) test-time learning.
- **New B7, den Boer's timescale claim:** "Δ̃ over T_0.95 = 165 periods, baseline 1 000 sessions: holds if mean Δ̃ is within 2 SE of the uniform-random 0.497 (and of −0.510 on `grid = symmetric`)."
  - Also report the share of Q-cells updated by T_δ (their bound is ≤ 5 %).
  - Also report the period at which mean Δ̃ over a sliding T_δ window first exceeds Δ_random + 0.05.
- **New B8, den Boer's m = 2 game:** a preset `two-prices` (`prices` = 2, `xi` = 0, β = 10⁻⁴, ties random), with an analysis that classifies each firm's strategy (16 words) and the profile (AD / GT / WSLS / other / RP).
  - It reproduces if the final shares are within 2 Wilson SE of 40.9 / 2.4 / 3.6 / 53.1 %, both-RP 46 %, and (C, C) 74 %.
  - The phase-transition claim holds if the share of sessions in WSLS before t = 12 308 is < 1 % and in GT before t = 6 633 is < 1 % [thresholds ours].
  - Sweeps: Table 3 (α), Table 4 (β) and Fig. 11 (δ) as A-style comparisons.
- **New B9, den Boer's "pattern ≠ scheme":** the share of baseline sessions that pass CCDP's one-deviation test (the paper's best-response cut gives a punishment-like response) but are **not** RP-complete.
  - Holds if that share is ≥ ¼ of the sessions that pass [threshold ours].
- **A-tests from these papers.** Add as reported comparisons, not scored A claims:
  - E&L's census: 1-sym 27.9 %, 1-asym 36.2 %, period 2: 18.3 %, ≥ 3: 11.0 %, hybrid 6.5 %. Lambin cites 64 % fixed points.
  - E&L's p̃ = 1.79.
  - den Boer's Fig. 10 Δ near δ → 1, computed with `window_gain` so that it matches their measure.

#### 4.4 Items for the A6-style "against the paper" list (reported, not scored)
- **Lambin:**
  - Lemma 2, eqs. 8–9 and Theorem 2 use α^n where the recursion implies (1 − α(1 − δ))^n, and eq. 9 has an extra (1 − δ) in the denominator.
  - Fig. 2's caption says the switch is at t = 10, while the text says 1 000 steps.
  - The numerics start at the grid-mean greedy price, which implies a non-CCDP initialization in an "exact replication".
  - "The punishment price is always 1.58" does not match E&L's Table 1.
- **Epivent & Lambin:**
  - δ is never stated.
  - Fig. 1's deviation size is not stated.
  - The pre-shock restriction (fn 4) conflicts with Table 1's rows 1.93 and 1.97.
  - App. C's "strictly below Nash" average of ≈ 1.465 is inconsistent with a top price of 1.47.
- **den Boer:**
  - Exp3's η in Theorem 2 has (1−δ)², while the proof's optimum has (1−δ²).
  - π̄ is read as a window average, not CCDP's limit cycle.
  - T_δ values are off by one or two periods from the stated formula.
  - Fig. 4 calls grid index 0 "the Nash price".

### Digitizing list
- E&L: Fig. 1a/b (both agents, τ = 0–30); Figs. 3–4 (point vs set); Fig. 5a–d (τ = 11 prices against deviation price); Fig. 6a/b (shifted grid). Table 1 and Table 2 are text, so transcribe them.
- Lambin: Fig. 1a/b (greedy price, 0–2 × 10⁶); Figs. 3 and 10 (ten panels each, plus the ε steps to confirm the step construction); Fig. 2 (Q-values, periods 0–80); Fig. 4 (six paths, which are exact grid values); Fig. 5 (entry, 0–250 000); Fig. 6a/b (floor, 0–4 × 10⁶); Fig. 9b; Figs. 7–8 (if later milestones take on Klein or Calvano 2023).
- den Boer: Fig. 2 left/right (strategy fractions and the (C, C) share, 0–80 000); Figs. 7–9 (variants); Fig. 10 left/right; Fig. 11 left/right; Fig. 5 (CCDP Fig. 3 replication); App. B Fig. 6. Tables 1–7 are text.
