import { describe, expect, it } from 'vitest';
import { COMPARE_PRESETS, comparePresetStates } from './compare-presets';
import type { Config, Preset } from './types';

const preset = (id: string, trade: boolean): Preset => ({
  id,
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
    const valley = (id: string): Preset => ({ id, name: id, source: '', description: '', config: { model: 'anasazi' } as unknown as Config });
    const states = comparePresetStates([valley('lhv-published'), valley('lhv-documented')], lhv, 9)!;
    expect([states.aSeed, states.b.seed]).toEqual([9, 9]);
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
});
