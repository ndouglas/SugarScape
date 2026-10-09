# Experiment viewer

Open the web app, choose **Experiments**, then **Episodes**. **Sweeps** and **Playground** remain available. Returning to either pauses episode playback; returning to Episodes keeps the shown successful record.

Choose a study, adjust its controls, and select **Run episode**. The eight entries are Wink, noisy testimony, testimony decision game, strategic reporting, strategy-aware listeners, adversarial audit, shared surface, and active surface. Controls describe the next run. The heading and **Shown successful episode** note describe the retained record, even after you edit controls or select another study. The **Retained episode** selector keeps at most two successful records.

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
