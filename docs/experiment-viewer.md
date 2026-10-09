# Experiment viewer

Open the web app, choose **Experiments**, then **Episodes**. **Sweeps** and **Playground** remain available. Returning to either pauses episode playback; returning to Episodes keeps the shown successful record.

Choose a study, adjust its controls, and select **Run episode**. The thirteen entries include Wink, noisy testimony, testimony decision game, strategic reporting, strategy-aware listeners, adversarial audit, shared surface, active surface, Burrow excavation, Burrow access, Foraging fixed, Foraging passage, and Foraging construction. Controls describe the next run. The heading and **Shown successful episode** note describe the retained record, even after you edit controls or select another study. The **Retained episode** selector keeps at most two successful records.

Wink accepts a decimal seed, policy, and mode. Seeds must contain digits only and fit an unsigned 64-bit integer, including the maximum `18446744073709551615`. Testimony selects an original fixture. Decision and reporting studies select the original environment, listener/controller, policy/witness/catalog where applicable, and a public history from 0 to 31. Surface studies select an ordered original setting and complete bit sequence from 0 to 255. Opened inputs that do not match a listed setting remain available for validation on run.

## Read the episode

Use **Previous**, **Next**, **Reset**, or the **Checkpoint** slider to inspect retained checkpoints. **Play** advances those checkpoints without rerunning the engine. With reduced motion enabled, use manual controls. Keyboard focus, arrow keys on sliders, and standard select/button controls provide the same navigation.

**Agent** perspective shows only the selected Agent's information at the current checkpoint. An observation may have an earlier delivery clock; another Agent's action does not refresh it. Unavailable state stays unavailable. **Researcher (privileged)** exposes physical state and complete results for inspection; changing perspective does not change a run or provide knowledge to its controller. Complete results also become available at the final checkpoint in Agent perspective.

Wink displays the game roster, delivered observations, submissions, and observed edges. Testimony and reporting display evidence, native beliefs, decisions, and exact conditional quantities. Shared and active surfaces display local observations, event history, belief masses, predictions, costs, and decisions. Researcher perspective separately labels physical topology and fields. Fractions and large integers remain exact text.

The shown semantics distinguish an actual episode trajectory from a **conditional case**. A selected public history conditions the decision/reporting case; private truth and signals are not sampled. Conditional expected payoff and regret are expectations, not realized reward. Predictive possibilities and hypothetical receiver responses are labeled separately from observed events or actual-policy references.

A supported but mistaken decision remains an ordinary result. **Unsupported history**, zero evidence, unavailable posterior, missing terminal outcome, and an unvisited checkpoint are explicit states; they do not imply a successful decision or an invented hidden world. Surface failures can retain costs already paid. Historical supported beliefs do not replace a currently unsupported belief.

## Compare surface controls

For a shown surface episode, **Matched policy comparison** lists only compatible original controls. **Run matched comparison** keeps the actual mechanism, role, complete sequence, IDs, and scientific settings and changes the compatible policy or pair. It runs another episode and retains the original plus its comparison.

The panes join the same public clock and stage when both records contain it. If a run ended early or skipped that stage, the comparison says **Unavailable at this public clock and stage**. It does not insert a paid wait or infer state. Use **Comparison checkpoint** to inspect its independent timeline; **Join current public clock** restores the exact join. Each pane applies the selected perspective and its own complete-result availability.

## Recorded measurements

**Load recorded study results** opens original testimony-game and strategic-reporting search measurements. Each study includes all 20 genetic-search and 20 random-search seeds, every retained curve point, frozen holdout evaluations, summaries, and paired differences. Select **Recorded study** and **Recorded search run**, expand the exact curve table or aggregate measurements, and inspect provenance and scientific settings.

These measurements were collected previously. Changing or running an episode does not train a policy, repeat a search, or execute a complete diagnostic census. A selected example illustrates a case; it is not a new aggregate performance claim. Chart coordinates are illustrative; tables and **Export complete retained measurements** preserve the exact recorded values.

## Share, export, and open

**Share input link** copies the edited input as an `#e=` link. It contains input only. Opening it selects Episodes and opens controls for editing; select **Run episode** to evaluate it. It does not automatically run or carry a claimed result. Existing Playground and sweep share formats remain supported.

**Export shown episode** exports the retained successful record after fresh engine validation. **Open episode…** accepts a compatible JSON episode file and validates it in a fresh worker before replacing the shown record. Imported claims are not trusted. A malformed link, invalid input/file, failed run, canceled request, or timeout reports an error and preserves the prior successful episode.

Wink uses fixed-width unsigned 64-bit sampling for runtime choices, preserving the original native results across native and browser targets. Its rules identity records the original measurement source and the portable viewer source separately. Recorded measurements keep their original identities.

Import validation accepts equivalent JSON numeric spellings (for example, `3` and `3.0`) without rounding distinct large integers together. Only computed noisy-testimony snapshot probabilities (hypotheses, proposition truth, and speaker profiles), testimony-game posterior truth probabilities, and testimony-game conditional regret allow the original absolute `1e-12` tolerance. All other values and fields—including actions, shapes, availability, parameters, and exact integer/rational quantities—must match exactly. Successful validation returns a freshly reconstructed record.

Inputs are limited to 64 KiB, including raw input bytes before parsing and normalized input. Input links also have a 64 KiB decompressed envelope limit. A record has at most 4,096 display checkpoints and a complete compact JSON export is limited to 16 MiB; file size is checked before reading. Use the original CLI tools for work outside the viewer's supported scope.

## Computation and cancellation

The browser computes one requested episode at a time in a dedicated worker, retaining one compiled engine during that request. **Cancel** terminates incomplete work. A 60-second wall-clock limit also terminates an incomplete request, releases its WASM state, and reports a timeout. This operational limit does not change scientific clocks, costs, or planner objectives. Completed playback reads checkpoints; another run or validation starts fresh. Loading recorded measurements may briefly allocate several megabytes, and expanding full tables can be expensive on small devices.

## Spatial engineering demonstrations

Episodes also includes **Burrow excavation**, **Burrow access**, **Foraging fixed**, **Foraging passage**, and **Foraging construction**. These five entries use the original Burrow and CPFA runners. **Spatial setup** selects a frozen, source-bound engineering example; it replaces the complete edited input only when you explicitly select it. Seeds, resource/material IDs, and Burrow freshness windows use canonical unsigned 64-bit decimal text. The JSON editor preserves opened custom or invalid inputs for correction; the runner validates them before allocation.

These are engineering demonstrations, not new biological, coordination, learning, optimality, or treatment-effect results. Their illustrative horizon and sampling do not replace the separate saved scientific measurements. F5, P3/P4, raw surface discovery, new controllers, and scientific campaign collection are outside this viewer.

Playback shows retained native boundaries. Burrow preserves native frame order and distinct stages even when they share a tick; its exported ASCII grid is an overlay, not an inferred inventory or occupancy map. CPFA checkpoints describe completed ticks after sequential opportunities in ascending Agent-ID order. A sampled tick is not a per-action transcript, and CPFA has no supplied per-action log. Event milestones retain their native clocks, including unavailable censored values.

In Agent perspective, Burrow supplies only committed own actions/choices and available native completion markers. Its observations, local map, observation ages, and current holdings are unavailable. Fixed foraging supplies captured own Agent state and the supplied arena/nest. Passage and construction additionally supply that Agent's captured private topology beliefs: unknown cells remain unknown, remembered walls can be stale, and capture time is not the time each cell was learned. The viewer does not fill private state with global food, peer positions, paths, or researcher knowledge. Food and Spoil with the same numeric ID remain distinct namespaces. Researcher perspective and the final checkpoint separately gate complete native results.

Select a cell to inspect exact supplied state, use zoom or **Fit map**, and use the checkpoint range with arrow keys. Focus survives redraw and playback. Narrow screens keep readable maps inside their own scrolling pane; reduced motion disables automatic playback. Leaving Episodes pauses playback and retains its bounded successful records. Disposal clears introduced references. Keyboard verification covers standard selection, repeated range keys, focus, and redraw.

**Matched policy comparison** offers family-validated, declared one-control changes. CPFA's listed contrasts use zero and one while the JSON editor retains the original parameter domains. A comparison preserves geometry, seed, horizon, and sampling and joins actual clocks and stages. Equal seeds do not guarantee paired random draws; missing checkpoints remain unavailable.

The spatial browser profile accepts at most 4,096 grid cells, 16 Agents, and 8,192 requested opportunities, alongside the 64 KiB input/link limits, 4,096 checkpoint cap, and 16 MiB complete-record limit. Dimensions, counts, products, and retained sample budgets are checked before World allocation; capture also checks the byte budget incrementally. Native stricter limits still apply. Oversized requests are rejected without shortening their input. One worker request, a 60-second timeout/cancellation limit, and at most two successful retained records remain in force.

Spatial records authenticate the original source digest and adapter version. CPFA also binds the actual architecture/OS compilation target. Same-target fresh reconstruction must match exactly; foreign-target CPFA imports report incompatibility. Declared native/WASM fixtures verify semantic geometry, actions, identities, inventories, costs, clocks, shape, and availability while retaining floating differences for inspection. This does not establish universal portability, and no CPFA import tolerance is added. Source-bound records from an older source identity reject even when their retained native values happen to match. Existing eight study identities and original Burrow CLI exports remain compatible.
