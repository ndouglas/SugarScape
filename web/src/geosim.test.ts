import { describe, expect, it } from 'vitest';
import { modelOf, ticksLeft, COLOR_MODES, finishesUnpredictably } from './models';
import { defaultForm, formToSweep } from './experiments/form';
import { finishedNotice } from './engine';
import { readoutText } from './valley';
import { COMPARE_PRESETS, comparePresetStates } from './compare-presets';
import type { ModelConfig, ModelStats, Preset } from './types';
const config = { model: 'geosim', initialization_periods: 2, observation_periods: 13, periods_per_tick: 7 } as unknown as ModelConfig;
describe('GeoSim host contracts', () => {
  it('recognizes GeoSim and counts its partial final source tick', () => {
    expect([modelOf(config), ticksLeft(config, 0), ticksLeft(config, 2), ticksLeft(config, 3), finishesUnpredictably(config)]).toEqual(['geosim', 3, 1, 0, false]);
  });
  it('uses captured custom horizons and batching for experiment defaults', () => {
    const form = defaultForm('geosim', config);
    expect(form.ticks).toBe(3);
    expect(form.metric).toMatchObject({ kind: 'final', series: 'completed_wars' });
    expect(formToSweep(form, { config }).sweep?.base).toEqual({ config });
  });
  it('presents actual completed source periods independently from display ticks', () => {
    expect(readoutText({ config, tick: 3, population: 1, latest: { periods: 15 } } as never, null)).toBe('t = 3 · 15 periods · 1 sovereign governments');
    expect(finishedNotice(config, 3, { periods: 15, finish_reason: 'horizon', invalidity: null } as unknown as ModelStats)).toContain('period 15');
  });
  it('reports attempted invalid periods without claiming horizon completion', () => {
    const text = finishedNotice(config, 1, { periods: 4, attempted_period: 5, finish_reason: 'invalid', invalidity: 'nonfinite capacity' } as unknown as ModelStats);
    expect(text).toContain('nonfinite capacity');
    expect(text).toContain('5');
  });
  it('offers all five core canvas modes', () => {
    expect(COLOR_MODES.geosim.map(([mode]) => mode)).toEqual(['territory', 'capacity', 'technology', 'alert', 'wars']);
  });
  it('compares technology treatments using independent captured configs', () => {
    const presets = ['geosim-paper', 'geosim-no-technology'].map(id => ({ id, title: id, name: id, source: 'Cederman2003', description: '', config: { ...config, shock_shift: id === 'geosim-paper' ? 20 : 0 } })) as Preset[];
    const pair = COMPARE_PRESETS.find(p => p.id === 'geosim-technology');
    expect(pair).toBeDefined();
    expect(comparePresetStates(presets, pair!, 17)?.b.config).toMatchObject({ shock_shift: 0, periods_per_tick: 7 });
  });
});

describe('GeoSim attributed control metadata', () => {
  it('retains alternative choices and validation bounds when adding source guidance', async () => {
    const {geosimParams}=await import('./geosim');
    const params=[{path:'defender_threshold',label:'Defender threshold',kind:'choice',group:'Readings',apply:'reset',choices:[{value:'reciprocal',label:'reciprocal'},{value:'same_threshold',label:'same threshold'}]}, {path:'war_shadow',label:'War shadow',kind:'integer',group:'Model',apply:'reset',min:0,max:10000}] as import('./types').Param[];
    const decorated=geosimParams(params);
    expect(decorated[0].choices).toEqual(params[0].choices);
    expect(decorated[0].help).toContain('p.148');
    expect(decorated[1]).toMatchObject({min:0,max:10000});
    expect(decorated[1].help).toContain('source periods');
  });
});

describe('captured experiment base', () => {
  it('captures resolved custom config rather than following subsequent control edits', async () => {
    const {captureExperimentBase}=await import('./experiments/form');
    const original=structuredClone(config);
    const captured=captureExperimentBase(null,true,original);
    (original as unknown as {observation_periods:number}).observation_periods=100;
    expect(captured).toEqual({config});
    expect(captureExperimentBase('geosim-paper',false,original)).toEqual({preset:'geosim-paper'});
  });
});
