import { describe, expect, it } from 'vitest';
import { COMPARE_PRESETS, comparePresetStates } from './compare-presets';
import type { Config, Preset } from './types';

const preset = (id: string, trade: boolean): Preset => ({
  id,
  title: id,
  name: id,
  source: '',
  description: '',
  config: { trade: { enabled: trade } } as unknown as Config,
});

describe('compare presets', () => {
  const entry = COMPARE_PRESETS[0];

  it('opens VI-2 as A and VI-3 as B', () => {
    expect(entry).toMatchObject({ a: 'vi-2-no-trade', b: 'vi-3-trade', label: 'Indecomposability — VI-2 vs VI-3 (Compare)' });
  });

  it('gives A its seed and builds B from a copy of its preset at the same seed', () => {
    const presets = [preset('vi-2-no-trade', false), preset('vi-3-trade', true)];
    const states = comparePresetStates(presets, entry, 42)!;
    expect(states.aSeed).toBe(42);
    expect(states.b.seed).toBe(42);
    const b = states.b.config as Config;
    expect(b.trade.enabled).toBe(true);
    b.trade.enabled = false;
    expect((presets[1].config as Config).trade.enabled).toBe(true);
  });

  it('is null when a preset is missing or the two are of different models', () => {
    expect(comparePresetStates([preset('vi-2-no-trade', false)], entry, 1)).toBeNull();
    const ring: Preset = { ...preset('vi-3-trade', true), config: { model: 'ring' } as unknown as Config };
    expect(comparePresetStates([preset('vi-2-no-trade', false), ring], entry, 1)).toBeNull();
  });

  it('opens the published Anasazi replication as A and the documented model as B', () => {
    const lhv = COMPARE_PRESETS.find((c) => c.id === 'lhv-published-vs-documented')!;
    expect(lhv).toMatchObject({ a: 'lhv-published', b: 'lhv-documented', label: 'Replication vs documented — Anasazi (Compare)' });
    const valley = (id: string): Preset => ({ id, title: id, name: id, source: '', description: '', config: { model: 'anasazi' } as unknown as Config });
    const states = comparePresetStates([valley('lhv-published'), valley('lhv-documented')], lhv, 9)!;
    expect([states.aSeed, states.b.seed]).toEqual([9, 9]);
  });

  it('pairs the budget constraint with its absence, and ZI-C with ZIP', () => {
    const ids = COMPARE_PRESETS.map((c) => [c.id, c.a, c.b, c.label]);
    expect(ids).toContainEqual(['gs-1-vs-u', 'gs-1', 'gs-1-u', 'With vs without the budget constraint — Zero-Intelligence Traders (Compare)']);
    expect(ids).toContainEqual(['zi-c-vs-zip', 'cliff-excess-demand', 'zip-excess-demand', 'ZI-C vs ZIP in a box market — Zero-Intelligence Traders (Compare)']);
  });

  it('pairs the literal reading of others’ effort with the live one', () => {
    const ids = COMPARE_PRESETS.map((c) => [c.id, c.a, c.b, c.label]);
    expect(ids).toContainEqual(['firms-last-vs-live', 'firms-base', 'firms-live', "Last period's effort vs live effort — The Emergence of Firms (Compare)"]);
  });

  it('pairs learning from the price charged with learning from every price', () => {
    const ids = COMPARE_PRESETS.map((c) => [c.id, c.a, c.b, c.label]);
    expect(ids).toContainEqual(['collusion-async-vs-sync', 'collusion-calvano', 'collusion-synchronous', 'Learning from the price charged vs every price — Algorithmic Collusion (Compare)']);
  });

  it('pairs imitation with the same plans fixed', () => {
    const ids = COMPARE_PRESETS.map((c) => [c.id, c.a, c.b, c.label]);
    expect(ids).toContainEqual(['lk-random-vs-fixed', 'lk-random', 'lk-random-fixed', 'Imitating neighbors vs fixed random plans — Balinese Water Temples (Compare)']);
  });

  it('pairs the hoard worlds either side of the threshold', () => {
    const ids = COMPARE_PRESETS.map((c) => [c.id, c.a, c.b, c.label]);
    expect(ids).toContainEqual(['hoard-scatter-vs-larder', 'hoard-scatter', 'hoard-larder', 'Scattered caches hard vs easy to find — Minds 7 (Compare)']);
  });

  it('pairs punishment with its absence', () => {
    const ids = COMPARE_PRESETS.map((c) => [c.id, c.a, c.b, c.label]);
    expect(ids).toContainEqual(['bg-base-vs-none', 'bg-base', 'bg-none', 'With vs without punishment — Altruistic Punishment (Compare)']);
  });

  it('pairs 15 % and 5 % rational retirees', () => {
    const ids = COMPARE_PRESETS.map((c) => [c.id, c.a, c.b, c.label]);
    expect(ids).toContainEqual(['ae-rapid-vs-slow', 'ae-rapid', 'ae-slow', '15 % vs 5 % rational — Retirement (Compare)']);
  });

  it('pairs Granovetter’s uniform and perturbed crowds', () => {
    const ids = COMPARE_PRESETS.map((c) => [c.id, c.a, c.b, c.label]);
    expect(ids).toContainEqual(['gr-uniform-vs-perturbed', 'gr-uniform', 'gr-perturbed', 'Uniform vs perturbed crowd — Threshold Models (Compare)']);
  });

  it('pairs Kirman’s colony with ten times the ants', () => {
    const ids = COMPARE_PRESETS.map((c) => [c.id, c.a, c.b, c.label]);
    expect(ids).toContainEqual(['ants-colony-size', 'ants-2b', 'ants-crowd', 'Colony size — Ants (Compare)']);
  });

  it('pairs Arthur’s accuracy and payoff-rated predictors', () => {
    const ids = COMPARE_PRESETS.map((c) => [c.id, c.a, c.b, c.label]);
    expect(ids).toContainEqual(['ef-accuracy-vs-payoff', 'ef-arthur', 'ef-payoff', 'Accuracy vs payoff scoring — El Farol (Compare)']);
  });

  it('pairs Meadows and Cliff’s reading and the reply', () => {
    const ids = COMPARE_PRESETS.map((c) => [c.id, c.a, c.b, c.label]);
    expect(ids).toContainEqual(['ra-meadows-cliff-vs-reply', 'ra-meadows-cliff', 'ra-deffuant-2013', 'Meadows and Cliff vs Deffuant et al.’s reply — Relative Agreement (Compare)']);
  });

  it('pairs Axelrod’s selection and a random tournament', () => {
    const ids = COMPARE_PRESETS.map((c) => [c.id, c.a, c.b, c.label]);
    expect(ids).toContainEqual(['gi-axelrod-vs-tournament', 'gi-metanorms-long', 'gi-tournament', 'Axelrod’s selection vs a random tournament — Norms and Metanorms (Compare)']);
  });

  it('pairs random mixing and fixed random neighbors', () => {
    const ids = COMPARE_PRESETS.map((c) => [c.id, c.a, c.b, c.label]);
    expect(ids).toContainEqual(['cra-rwr-vs-frn', 'cra-rwr', 'cra-frn', 'Random mixing vs fixed random neighbors — Social Structure (Compare)']);
  });

  it('pairs simultaneous and serial updating', () => {
    const ids = COMPARE_PRESETS.map((c) => [c.id, c.a, c.b, c.label]);
    expect(ids).toContainEqual(['hk-simultaneous-vs-serial', 'hk-polarisation', 'hk-serial', 'Simultaneous vs serial updating — Bounded Confidence (Compare)']);
  });

  it('pairs AEY’s rule and the mode rule with tags', () => {
    const ids = COMPARE_PRESETS.map((c) => [c.id, c.a, c.b, c.label]);
    expect(ids).toContainEqual(['aey-tags-vs-mode', 'aey-tags', 'pvplh-mode', 'AEY’s rule vs the mode rule, with tags — Emergence of Classes (Compare)']);
  });

  it('pairs the docking paper’s two activations at 20 × 20', () => {
    const ids = COMPARE_PRESETS.map((c) => [c.id, c.a, c.b, c.label]);
    expect(ids).toContainEqual([
      'ac-random-vs-sweep-20',
      'ac-random-activation-20',
      'ac-sweep-activation',
      'Literal vs Sugarscape activation, 20 × 20 — Axelrod Culture (Compare)',
    ]);
  });

  it('pairs the published and literal tie rules at two pairings', () => {
    const ids = COMPARE_PRESETS.map((c) => [c.id, c.a, c.b, c.label]);
    expect(ids).toContainEqual([
      'rca-published-vs-literal-p2',
      'rca-published-p2',
      'rca-literal-p2',
      'Published vs literal ties at P = 2 — Tag Cooperation (Compare)',
    ]);
  });

  it('pairs the civil runs the paper compares', () => {
    const ids = COMPARE_PRESETS.map((c) => [c.id, c.a, c.b, c.label]);
    expect(ids).toContainEqual(['cv-salami-vs-jump', 'cv-run-3-salami', 'cv-run-4-one-jump', 'Salami tactics vs one jump — Civil Violence (Compare)']);
    expect(ids).toContainEqual([
      'cv-cleansing-vs-peacekeepers',
      'cv-run-7-cleansing',
      'cv-safe-havens',
      'Ethnic cleansing vs peacekeepers — Civil Violence (Compare)',
    ]);
  });

  it('pairs the spatial games the debate compared', () => {
    const ids = COMPARE_PRESETS.map((c) => [c.id, c.a, c.b, c.label]);
    expect(ids).toContainEqual(['nm-sync-vs-async', 'nm-3-kaleidoscope', 'hg-async-kaleidoscope', 'Synchronous vs asynchronous — Spatial Games (Compare)']);
    expect(ids).toContainEqual(['nbm-discrete-vs-continuous', 'nbm-discrete', 'nbm-continuous', 'Discrete vs continuous time — Spatial Games (Compare)']);
  });

  it('pairs the ethnocentrism runs the sources disagree on', () => {
    const ids = COMPARE_PRESETS.map((c) => [c.id, c.a, c.b, c.label]);
    expect(ids).toContainEqual(['ha-four-vs-five', 'ha-standard', 'ha-java-five-colors', 'Four colors vs five (the Java’s draw) — Ethnocentrism (Compare)']);
    expect(ids).toContainEqual(['ha-adjacent-vs-anywhere', 'ha-standard', 'jansson-offspring-anywhere', 'Next to the parent vs anywhere — Ethnocentrism (Compare)']);
    expect(ids).toContainEqual(['ha-tags-vs-kin', 'ha-standard', 'jansson-kin', 'Tags vs kin — Ethnocentrism (Compare)']);
  });

  it('pairs the demographic PD runs the sources and readings disagree on', () => {
    const ids = COMPARE_PRESETS.filter((c) => c.id.startsWith('dpd-')).map((c) => [c.id, c.a, c.b, c.label]);
    expect(ids).toEqual([
      ['dpd-wp-vs-published', 'dpd-run-1', 'dpd-working-paper', 'Working paper vs published rule — Demographic PD (Compare)'],
      ['dpd-negative-vs-metabolism', 'dpd-run-2', 'dpd-metabolism', 'Negative payoffs vs shifted with metabolism — Demographic PD (Compare)'],
      ['dpd-space-vs-soup', 'dpd-run-1', 'dpd-soup', 'Space vs soup — Demographic PD (Compare)'],
      ['dpd-published-vs-closest', 'dpd-run-1', 'dpd-closest', 'Published rule vs closest reading — Demographic PD (Compare)'],
    ]);
  });

  it('pairs the image-scoring runs the sources and readings disagree on', () => {
    const ids = COMPARE_PRESETS.filter((c) => c.id.startsWith('image-')).map((c) => [c.id, c.a, c.b, c.label]);
    expect(ids).toEqual([
      ['image-one-vs-island', 'lh-fig-2a', 'lh-fig-2b', 'One group vs the island model — Image Scoring (Compare)'],
      ['image-scoring-vs-standing', 'lh-fig-2b', 'lh-fig-4c', 'Image scoring vs standing — Image Scoring (Compare)'],
      ['image-offset', 'ns-fig-1', 'ns-no-offset', 'With vs without the offset — Image Scoring (Compare)'],
      ['image-group-size', 'ns-fig-3-n20', 'ns-fig-3-n100', 'Small vs large groups with observers — Image Scoring (Compare)'],
    ]);
  });
});
