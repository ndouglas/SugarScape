import { describe, expect, it } from 'vitest';
import { finishesUnpredictably, modelOf, ticksLeft } from './models';
import { defaultForm, formToSweep } from './experiments/form';
import { COMPARE_PRESETS, comparePresetStates } from './compare-presets';
import { MODEL_CHARTS } from './ui/series-data';
import type { ModelConfig, Preset } from './types';
const config = { model: 'auctions', horizon: 10001, periods_per_tick: 1000 } as unknown as ModelConfig;
describe('Q-learning auction integration', () => {
  it('counts the partial final tick and never expects early convergence stopping', () => {
    expect(modelOf(config)).toBe('auctions');
    expect([ticksLeft(config, 0), ticksLeft(config, 10), ticksLeft(config, 11), finishesUnpredictably(config)]).toEqual([11, 1, 0, false]);
  });
  it('defaults format sweeps to terminal revenue after the full economic horizon', () => {
    expect(formToSweep(defaultForm('auctions'), { config }).sweep?.set).toEqual({ auction: 'mixture' });
    expect(defaultForm('auctions')).toMatchObject({ x: { path: 'auction_alpha', values: '1:2:0.1' }, ticks: 1000, metric: { kind: 'final', series: 'terminal_revenue' } });
  });
  it('keeps custom and persistent horizons when suggesting an experiment', () => {
    expect(defaultForm('auctions', { ...config, horizon: 100_000_000 } as ModelConfig).ticks).toBe(100_000);
    expect(defaultForm('auctions', config).ticks).toBe(11);
  });
  it('compares formats and pairs rival information with all-action updating', () => {
    const preset = (id: string, fields: object): Preset => ({ id, title: id, name: id, source: '', description: '', config: { ...config, ...fields } as ModelConfig });
    const presets = [preset('auctions-first-price', { auction: 'first_price', feedback: 'outcome', update: 'chosen' }), preset('auctions-second-price', { auction: 'second_price' }), preset('auctions-feedback', { feedback: 'rival_bids', update: 'all' })];
    const formats = COMPARE_PRESETS.find((p) => p.id === 'auctions-formats')!;
    expect(formats).toBeDefined();
    expect(comparePresetStates(presets, formats, 17)?.b.config).toMatchObject({ auction: 'second_price' });
    const feedback = COMPARE_PRESETS.find((p) => p.id === 'auctions-feedback')!;
    expect(comparePresetStates(presets, feedback, 17)?.b.config).toMatchObject({ feedback: 'rival_bids', update: 'all' });
  });
  it('charts played and greedy bids separately from seller and bidder rewards', () => {
    const charts = MODEL_CHARTS['auctions'];
    expect(charts).toBeDefined();
    expect(charts.map((c) => c.title)).toEqual(['Bids', 'Seller revenue', 'Bidder rewards', 'Learning']);
    expect(charts[0].lines.map((l) => l.key)).toEqual(['bid_1', 'bid_2', 'bid_3', 'greedy_1', 'greedy_2', 'greedy_3']);
  });
});
