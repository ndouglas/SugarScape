import { describe, expect, it } from 'vitest';
import {
  calendarYear,
  COLOR_MODES,
  finishesUnpredictably,
  isAgreementView,
  isAntsView,
  isThresholdsView,
  isFarolView,
  isCivilView,
  isClassesView,
  isOpinionsView,
  isStructureView,
  isNormsView,
  isCultureView,
  isDpdView,
  isEthnoView,
  isImageView,
  isRingView,
  isSpatialView,
  isSugar,
  isSugarView,
  isTagsView,
  isValleyView,
  MODEL_OVERLAYS,
  modelOf,
  presetGroups,
  presetOptionLabel,
  presetReference,
  ticksLeft,
} from './models';
import type { AnyInspection, Config, ModelConfig, Preset } from './types';

describe('modelOf', () => {
  it('reads a config without a model key (every config before milestone 9) as a sugarscape', () => {
    const old = { width: 50, goods: [] } as unknown as Config;
    expect(modelOf(old)).toBe('sugarscape');
    expect(isSugar(old)).toBe(true);
    expect(modelOf({ ...old, model: 'sugarscape' } as unknown as ModelConfig)).toBe('sugarscape');
  });

  it('reads the other models by their tag', () => {
    expect(modelOf({ model: 'schelling' } as ModelConfig)).toBe('schelling');
    expect(modelOf({ model: 'ring' } as ModelConfig)).toBe('ring');
    expect(isSugar({ model: 'ring' } as ModelConfig)).toBe(false);
  });

  it('tells a sugarscape inspection by its resources and Ring World’s by its sugar', () => {
    const sugar = { site: { x: 0, y: 0, resources: [1], capacities: [4], pollution: [] }, agent: null } as AnyInspection;
    const ring = { site: { x: 3, sugar: 2, capacity: 4 }, agent: null } as AnyInspection;
    const schelling = { site: { x: 1, y: 2 }, agent: null } as AnyInspection;
    expect(isSugarView(sugar)).toBe(true);
    expect(isSugarView(ring)).toBe(false);
    expect([sugar, ring, schelling].map(isRingView)).toEqual([false, true, false]);
  });

  it('offers each model its color modes, Schelling first Color, Ring World none', () => {
    expect(COLOR_MODES.sugarscape[0][0]).toBe('tribe');
    expect(COLOR_MODES.schelling.map(([m]) => m)).toEqual(['color', 'satisfaction', 'preference']);
    expect(COLOR_MODES.ring).toEqual([]);
  });
});

describe('presetGroups', () => {
  it('groups the presets menu by model in a fixed order, leaving out models without presets', () => {
    const p = (id: string, config: unknown): Preset => ({ id, title: id, name: id, source: '', description: '', config: config as ModelConfig });
    const presets = [p('ring-1', { model: 'ring' }), p('ii-2', {}), p('ii-3', {}), p('ring-2', { model: 'ring' })];
    expect(presetGroups(presets).map((g) => [g.label, g.presets.map((x) => x.id)])).toEqual([
      ['Sugarscape', ['ii-2', 'ii-3']],
      ['Ring World', ['ring-1', 'ring-2']],
    ]);
  });
});

describe('the anasazi model', () => {
  const valley = { model: 'anasazi', start_year: 800, end_year: 1350 } as ModelConfig;

  it('is read by its tag, and its inspections by their zone', () => {
    expect(modelOf(valley)).toBe('anasazi');
    expect(isSugar(valley)).toBe(false);
    const cell = { site: { x: 1, y: 2, zone: 'north', zone_name: 'North Valley Floor' }, agent: null } as unknown as AnyInspection;
    const schelling = { site: { x: 1, y: 2 }, agent: null } as AnyInspection;
    expect([cell, schelling].map(isValleyView)).toEqual([true, false]);
    expect(isRingView(cell) || isSugarView(cell)).toBe(false);
  });

  it('offers Occupation first, then Zones and Yield, and the valley’s three overlays', () => {
    expect(COLOR_MODES.anasazi).toEqual([
      ['occupation', 'Occupation'],
      ['zones', 'Zones'],
      ['yield', 'Yield'],
    ]);
    expect(MODEL_OVERLAYS.anasazi).toEqual(['water', 'settlements', 'links']);
    expect(MODEL_OVERLAYS.sugarscape).not.toContain('water');
    expect(MODEL_OVERLAYS.ring).toEqual([]);
  });

  it('counts years from its start year and ticks until its end year', () => {
    expect(calendarYear(valley, 342)).toBe(1142);
    expect(ticksLeft(valley, 540)).toBe(10);
    expect(ticksLeft(valley, 550)).toBe(0);
    const ring = { model: 'ring' } as ModelConfig;
    expect(calendarYear(ring, 5)).toBeNull();
    expect(ticksLeft(ring, 5)).toBe(Infinity);
  });

  it('groups its presets under Artificial Anasazi, last', () => {
    const p = (id: string, config: unknown): Preset => ({ id, title: id, name: id, source: '', description: '', config: config as ModelConfig });
    const groups = presetGroups([p('lhv', valley), p('ii-2', {}), p('vi-8', { model: 'ring' })]);
    expect(groups.map((g) => g.label)).toEqual(['Sugarscape', 'Ring World', 'Artificial Anasazi']);
  });
});

describe('the presets menu', () => {
  it('shows each preset by its plain title, with its source and rules beside the description', () => {
    const p = { id: 'ii-2-unit', title: 'Sugar grows back slowly', name: '({G₁}, {M})', source: 'Animation II-2', description: '', config: {} } as unknown as Preset;
    expect(presetOptionLabel(p)).toBe('Sugar grows back slowly');
    expect(presetReference(p)).toBe('Animation II-2 · ({G₁}, {M})');
  });
});

describe('the thresholds model', () => {
  it('is read by its tag, and its inspections by their cdf, before the ants’ and El Farol’s', () => {
    const c = { model: 'thresholds', stop_at: 0 } as unknown as ModelConfig;
    expect(modelOf(c)).toBe('thresholds');
    const cell = { site: { x: 1, y: 2 }, panel: 'actors', step: null, crowds: null, share: null, cdf: null, count: null, member: null, agent: null } as unknown as AnyInspection;
    const ants = { site: { x: 1, y: 2 }, panel: 'ants', step: null, shares: null, share: null, count: null, theory: null, member: null, agent: null } as unknown as AnyInspection;
    expect([cell, ants].map(isThresholdsView)).toEqual([true, false]);
    expect(isAntsView(cell)).toBe(false);
  });

  it('colors four ways, has no overlays, and stops predictably at its last step', () => {
    expect(COLOR_MODES.thresholds).toEqual([
      ['state', 'State'],
      ['threshold', 'Threshold'],
      ['degree', 'Degree'],
      ['crowd', 'Crowd'],
    ]);
    expect(MODEL_OVERLAYS.thresholds).toEqual([]);
    const c = (stop_at: number) => ({ model: 'thresholds', stop_at }) as unknown as ModelConfig;
    expect([finishesUnpredictably(c(50)), ticksLeft(c(50), 20), ticksLeft(c(0), 20)]).toEqual([false, 30, Infinity]);
  });
});

describe('the ants model', () => {
  it('is read by its tag, and its inspections by their shares', () => {
    const c = { model: 'ants', stop_at: 0 } as unknown as ModelConfig;
    expect(modelOf(c)).toBe('ants');
    const cell = { site: { x: 1, y: 2 }, panel: 'ants', step: null, shares: null, share: null, count: null, theory: null, member: null, agent: null } as unknown as AnyInspection;
    const farol = { site: { x: 1, y: 2 }, panel: 'agents', round: null, attendance: null, count: null, member: null, agent: null } as unknown as AnyInspection;
    expect([cell, farol].map(isAntsView)).toEqual([true, false]);
    expect(isAgreementView(cell)).toBe(false);
  });

  it('colors three ways, has no overlays, and stops predictably at its last step', () => {
    expect(COLOR_MODES.ants).toEqual([
      ['source', 'Source'],
      ['independent', 'Independent'],
      ['degree', 'Degree'],
    ]);
    expect(MODEL_OVERLAYS.ants).toEqual([]);
    const c = (stop_at: number) => ({ model: 'ants', stop_at }) as unknown as ModelConfig;
    expect([finishesUnpredictably(c(2000)), ticksLeft(c(2000), 500), ticksLeft(c(0), 500)]).toEqual([false, 1500, Infinity]);
  });
});

describe('the El Farol and minority game model', () => {
  it('is read by its tag, and its inspections by their panel and member', () => {
    const c = { model: 'farol', stop_at: 100 } as unknown as ModelConfig;
    expect(modelOf(c)).toBe('farol');
    const cell = { site: { x: 1, y: 2 }, panel: 'agents', round: null, attendance: null, count: null, member: null, agent: null } as unknown as AnyInspection;
    const ra = { site: { x: 1, y: 2 }, panel: 'diagram', period: 0, opinion: 0.5, agents: [], agent: null } as unknown as AnyInspection;
    expect([cell, ra].map(isFarolView)).toEqual([true, false]);
    expect(isAgreementView(cell)).toBe(false);
  });

  it('colors four ways, has no overlays, and stops predictably at its last round', () => {
    expect(COLOR_MODES.farol).toEqual([
      ['choice', 'Choice'],
      ['gain', 'Gain'],
      ['strategy', 'Strategy'],
      ['memory', 'Memory'],
    ]);
    expect(MODEL_OVERLAYS.farol).toEqual([]);
    const c = (stop_at: number) => ({ model: 'farol', stop_at }) as unknown as ModelConfig;
    expect([finishesUnpredictably(c(100)), ticksLeft(c(100), 40), ticksLeft(c(0), 40)]).toEqual([false, 60, Infinity]);
  });
});

describe('the relative agreement model', () => {
  it('is read by its tag, and its inspections by their panel', () => {
    const c = { model: 'agreement', stop_at: 200 } as unknown as ModelConfig;
    expect(modelOf(c)).toBe('agreement');
    const cell = { site: { x: 1, y: 2 }, panel: 'diagram', period: 0, opinion: 0.5, agents: [], agent: null } as unknown as AnyInspection;
    const hk = { site: { x: 1, y: 2 }, period: 0, opinion: 0.5, lattice_site: null, agents: [], agent: null } as unknown as AnyInspection;
    expect([cell, hk].map(isAgreementView)).toEqual([true, false]);
    expect([isOpinionsView(cell), isNormsView(cell)]).toEqual([false, false]);
  });

  it('colors three ways, has no overlays, and stops at its period or unpredictably when stable', () => {
    expect(COLOR_MODES.agreement).toEqual([
      ['uncertainty', 'Uncertainty'],
      ['role', 'Role'],
      ['start', 'Start'],
    ]);
    expect(MODEL_OVERLAYS.agreement).toEqual([]);
    const c = (stop_when_stable: boolean, stop_at: number) => ({ model: 'agreement', stop_when_stable, stop_at }) as unknown as ModelConfig;
    expect([finishesUnpredictably(c(false, 200)), ticksLeft(c(false, 200), 40)]).toEqual([false, 160]);
    expect([finishesUnpredictably(c(true, 20000)), ticksLeft(c(true, 20000), 40), ticksLeft(c(true, 0), 40)]).toEqual([true, 19960, Infinity]);
  });
});

describe('the norms model', () => {
  it('is read by its tag, and its inspections by their plane level', () => {
    const c = { model: 'norms', stop_at: 100 } as unknown as ModelConfig;
    expect(modelOf(c)).toBe('norms');
    const cell = { site: { x: 1, y: 2 }, level: [3, 4], agents: [], agent: null } as unknown as AnyInspection;
    const block = { site: { x: 1, y: 2 }, block: { x: 0, y: 0 }, plane: null, agents: [], agent: null } as unknown as AnyInspection;
    expect([cell, block].map(isNormsView)).toEqual([true, false]);
    expect(isStructureView(cell)).toBe(false);
  });

  it('colors three ways, has no overlays, and stops predictably at its last generation', () => {
    expect(COLOR_MODES.norms).toEqual([
      ['agents', 'Agents'],
      ['payoff', 'Payoff'],
      ['group', 'Group'],
    ]);
    expect(MODEL_OVERLAYS.norms).toEqual([]);
    const c = (stop_at: number) => ({ model: 'norms', stop_at }) as unknown as ModelConfig;
    expect([finishesUnpredictably(c(100)), ticksLeft(c(100), 40), ticksLeft(c(0), 40)]).toEqual([false, 60, Infinity]);
  });
});

describe('the social-structure model', () => {
  it('is read by its tag, and its inspections by their block cell and plane point', () => {
    const c = { model: 'structure', stop_at: 2500 } as unknown as ModelConfig;
    expect(modelOf(c)).toBe('structure');
    const cell = { site: { x: 1, y: 2 }, block: { x: 0, y: 0 }, plane: null, agents: [], agent: null } as unknown as AnyInspection;
    const line = { site: { x: 1, y: 2 }, period: 3, opinion: 0.5, lattice_site: null, agents: [], agent: null } as unknown as AnyInspection;
    expect([cell, line].map(isStructureView)).toEqual([true, false]);
    expect(isOpinionsView(cell) || isClassesView(cell) || isCultureView(cell) || isTagsView(cell) || isSugarView(cell)).toBe(false);
  });

  it('colors agents four ways, has no overlays, and stops predictably at its last period', () => {
    expect(COLOR_MODES.structure).toEqual([
      ['friendliness', 'Friendliness'],
      ['provocability', 'Provocability'],
      ['payoff', 'Payoff'],
      ['strategy', 'Strategy'],
    ]);
    expect(MODEL_OVERLAYS.structure).toEqual([]);
    const c = (stop_at: number) => ({ model: 'structure', stop_at }) as unknown as ModelConfig;
    expect([finishesUnpredictably(c(2500)), ticksLeft(c(2500), 500), ticksLeft(c(0), 500)]).toEqual([false, 2000, Infinity]);
  });
});

describe('the bounded-confidence model', () => {
  it('is read by its tag, and its inspections by their period and lattice site', () => {
    const c = { model: 'opinions', stop_when_stable: true } as unknown as ModelConfig;
    expect(modelOf(c)).toBe('opinions');
    const cell = { site: { x: 1, y: 2 }, period: 3, opinion: 0.5, lattice_site: null, agents: [], agent: null } as unknown as AnyInspection;
    const simplex = { site: { x: 1, y: 2 }, simplex: 'one', mix: [0.2, 0.5, 0.3], best_reply: 'M', agents: [], agent: null } as unknown as AnyInspection;
    expect([cell, simplex].map(isOpinionsView)).toEqual([true, false]);
    expect(isClassesView(cell) || isCultureView(cell) || isTagsView(cell) || isSugarView(cell)).toBe(false);
  });

  it('colors lines by start or opinion, has no overlays, and stops unpredictably when asked to stop at stability', () => {
    expect(COLOR_MODES.opinions).toEqual([
      ['start', 'Start'],
      ['opinion', 'Opinion'],
    ]);
    expect(MODEL_OVERLAYS.opinions).toEqual([]);
    const c = (stop_when_stable: boolean) => ({ model: 'opinions', stop_when_stable }) as unknown as ModelConfig;
    expect([finishesUnpredictably(c(true)), finishesUnpredictably(c(false))]).toEqual([true, false]);
    expect(ticksLeft(c(true), 5)).toBe(Infinity);
  });
});

describe('the classes model', () => {
  it('is read by its tag, and its inspections by their simplex and mix', () => {
    const c = { model: 'classes', stop_at_equity: false } as unknown as ModelConfig;
    expect(modelOf(c)).toBe('classes');
    const point = { site: { x: 1, y: 2 }, simplex: 'one', mix: [0.2, 0.5, 0.3], best_reply: 'M', agents: [], agent: null } as unknown as AnyInspection;
    const culture = { site: { x: 1, y: 2 }, kind: 'site', a: {}, b: null, shared: null, neighbors: [], agent: null } as unknown as AnyInspection;
    expect([point, culture].map(isClassesView)).toEqual([true, false]);
    expect(isCultureView(point) || isTagsView(point) || isSugarView(point)).toBe(false);
  });

  it('offers the paper’s shading and payoffs, no overlays, and stops unpredictably only at equity', () => {
    expect(COLOR_MODES.classes).toEqual([
      ['best_reply', 'Best reply'],
      ['payoff', 'Payoff'],
    ]);
    expect(MODEL_OVERLAYS.classes).toEqual([]);
    const c = (stop_at_equity: boolean) => ({ model: 'classes', stop_at_equity }) as unknown as ModelConfig;
    expect([finishesUnpredictably(c(true)), finishesUnpredictably(c(false))]).toEqual([true, false]);
    expect(ticksLeft(c(true), 5)).toBe(Infinity);
  });
});

describe('the culture model', () => {
  const culture = (stop_when_stable: boolean, drift: number) => ({ model: 'culture', stop_when_stable, drift }) as unknown as ModelConfig;

  it('is read by its tag, and its inspections by their kind and neighbors', () => {
    expect(modelOf(culture(true, 0))).toBe('culture');
    const site = { site: { x: 1, y: 2 }, kind: 'site', a: {}, b: null, shared: null, neighbors: [], agent: null } as unknown as AnyInspection;
    const tags = { site: { x: 1, y: 2 }, generation: 5, agents: [], agent: null } as unknown as AnyInspection;
    expect([site, tags].map(isCultureView)).toEqual([true, false]);
    expect(isRingView(site) || isSugarView(site) || isValleyView(site) || isCivilView(site) || isTagsView(site)).toBe(false);
  });

  it('offers its three shadings and no overlays, and stops unpredictably only without drift', () => {
    expect(COLOR_MODES.culture).toEqual([
      ['culture', 'Culture'],
      ['similarity', 'Similarity'],
      ['zones', 'Zones'],
    ]);
    expect(MODEL_OVERLAYS.culture).toEqual([]);
    expect(ticksLeft(culture(true, 0), 5)).toBe(Infinity);
    expect([finishesUnpredictably(culture(true, 0)), finishesUnpredictably(culture(true, 0.01)), finishesUnpredictably(culture(false, 0))]).toEqual([true, false, false]);
  });

  it('lets a sugarscape stop when its Axelrod cultures settle', () => {
    const sugar = (rule: string, stop: boolean) => ({ culture: { enabled: true, groups: [], rule, stop_when_settled: stop } }) as unknown as ModelConfig;
    expect([finishesUnpredictably(sugar('axelrod', true)), finishesUnpredictably(sugar('axelrod', false)), finishesUnpredictably(sugar('flip', true))]).toEqual([true, false, false]);
    expect(COLOR_MODES.sugarscape.map(([m]) => m)).toContain('culture');
  });
});

describe('the tags model', () => {
  const tags = (end: number) => ({ model: 'tags', end }) as unknown as ModelConfig;

  it('is read by its tag, and its inspections by their generation', () => {
    expect(modelOf(tags(30000))).toBe('tags');
    expect(isSugar(tags(30000))).toBe(false);
    const cell = { site: { x: 1, y: 2 }, generation: 5, agents: [], agent: null } as unknown as AnyInspection;
    const above = { site: { x: 1, y: 2 }, generation: null, agents: [], agent: null } as unknown as AnyInspection;
    const civil = { site: { x: 1, y: 2 }, agent: null, cop: null, jailed: [] } as unknown as AnyInspection;
    expect([cell, above, civil].map(isTagsView)).toEqual([true, true, false]);
    expect(isRingView(cell) || isSugarView(cell) || isValleyView(cell) || isCivilView(cell)).toBe(false);
  });

  it('offers its diagram’s three shadings and no overlays', () => {
    expect(COLOR_MODES.tags).toEqual([
      ['count', 'Count'],
      ['tolerance', 'Tolerance'],
      ['clones', 'Clones'],
    ]);
    expect(MODEL_OVERLAYS.tags).toEqual([]);
    expect(calendarYear(tags(30000), 5)).toBeNull();
    expect(finishesUnpredictably(tags(30000))).toBe(false);
  });

  it('counts down to its last generation, or never with none', () => {
    expect(ticksLeft(tags(30000), 29990)).toBe(10);
    expect(ticksLeft(tags(30000), 30005)).toBe(0);
    expect(ticksLeft(tags(0), 5)).toBe(Infinity);
  });
});

describe('the civil model', () => {
  it('is read by its tag, and its inspections by their jailed list', () => {
    const civil = { model: 'civil', variant: 'rebellion' } as unknown as ModelConfig;
    expect(modelOf(civil)).toBe('civil');
    expect(isSugar(civil)).toBe(false);
    const site = { site: { x: 1, y: 2 }, agent: null, cop: null, jailed: [] } as unknown as AnyInspection;
    const schelling = { site: { x: 1, y: 2 }, agent: null } as AnyInspection;
    expect([site, schelling].map(isCivilView)).toEqual([true, false]);
    expect(isRingView(site) || isSugarView(site) || isValleyView(site)).toBe(false);
  });

  it('offers the paper’s two screens and its groups, and no overlays', () => {
    expect(COLOR_MODES.civil).toEqual([
      ['action', 'Action'],
      ['grievance', 'Grievance'],
      ['group', 'Group'],
    ]);
    expect(MODEL_OVERLAYS.civil).toEqual([]);
    expect(ticksLeft({ model: 'civil' } as unknown as ModelConfig, 5)).toBe(Infinity);
    expect(calendarYear({ model: 'civil' } as unknown as ModelConfig, 5)).toBeNull();
  });

  it('can finish unpredictably only as Model II stopping at extinction', () => {
    const civil = (variant: string, stop_at_extinction: boolean) => ({ model: 'civil', variant, stop_at_extinction }) as unknown as ModelConfig;
    expect(finishesUnpredictably(civil('ethnic', true))).toBe(true);
    expect(finishesUnpredictably(civil('ethnic', false))).toBe(false);
    expect(finishesUnpredictably(civil('rebellion', true))).toBe(false);
    const valley = { model: 'anasazi', start_year: 800, end_year: 1350 } as unknown as ModelConfig;
    expect(finishesUnpredictably(valley)).toBe(false);
  });
});

describe('the spatial model', () => {
  it('is read by its tag, and its inspections by their z', () => {
    const c = { model: 'spatial', lattice: 'square' } as unknown as ModelConfig;
    expect(modelOf(c)).toBe('spatial');
    const cell = { site: { x: 1, y: 2, z: 0 }, agent: null } as unknown as AnyInspection;
    const schelling = { site: { x: 1, y: 2 }, agent: null } as AnyInspection;
    expect([cell, schelling].map(isSpatialView)).toEqual([true, false]);
  });

  it('offers the papers’ four-color change view first', () => {
    expect(COLOR_MODES.spatial).toEqual([
      ['change', 'Change'],
      ['strategy', 'Strategy'],
      ['payoff', 'Payoff'],
    ]);
    expect(MODEL_OVERLAYS.spatial).toEqual([]);
  });
});

describe('the ethnocentrism model', () => {
  const ethno = (end: number) => ({ model: 'ethno', end }) as unknown as ModelConfig;

  it('is read by its tag, and its inspections by the model (an empty site is shaped like Schelling’s)', () => {
    expect(modelOf(ethno(2000))).toBe('ethno');
    expect(isSugar(ethno(2000))).toBe(false);
    const empty = { site: { x: 1, y: 2 }, agent: null } as AnyInspection;
    const agent = { site: { x: 1, y: 2 }, agent: { id: 3, kin_marker: 3, neighbors: [] } } as unknown as AnyInspection;
    const schelling = { site: { x: 1, y: 2 }, agent: { id: 3, neighbors: 2 } } as unknown as AnyInspection;
    expect([empty, agent].map((v) => isEthnoView(v, 'ethno'))).toEqual([true, true]);
    expect([empty, agent, schelling].map((v) => isEthnoView(v, 'schelling'))).toEqual([false, false, false]);
    expect(isEthnoView(schelling, 'ethno')).toBe(false);
    expect(isTagsView(agent) || isRingView(agent) || isSugarView(agent) || isValleyView(agent) || isCivilView(agent) || isSpatialView(agent)).toBe(false);
  });

  it('offers strategy, tag, lineage and PTR colors and no overlays, and is grouped last', () => {
    expect(COLOR_MODES.ethno).toEqual([
      ['strategy', 'Strategy'],
      ['tag', 'Tag'],
      ['lineage', 'Lineage'],
      ['ptr', 'PTR'],
    ]);
    expect(MODEL_OVERLAYS.ethno).toEqual([]);
    const p = (id: string, config: object) => ({ id, name: id, source: '', description: '', config }) as unknown as Preset;
    expect(presetGroups([p('ha', { model: 'ethno' }), p('nm', { model: 'spatial' })]).map((g) => g.label)).toEqual(['Spatial Games', 'Ethnocentrism']);
  });

  it('counts down to its last period, or never with none', () => {
    expect(ticksLeft(ethno(2000), 1990)).toBe(10);
    expect(ticksLeft(ethno(2000), 2005)).toBe(0);
    expect(ticksLeft(ethno(0), 5)).toBe(Infinity);
    expect(finishesUnpredictably(ethno(2000))).toBe(false);
    expect(calendarYear(ethno(2000), 5)).toBeNull();
  });
});

describe('the demographic PD', () => {
  const dpd = (end: number) => ({ model: 'dpd', end }) as unknown as ModelConfig;

  it('is read by its tag, and its inspections by the model (an empty site is shaped like Schelling’s and ethnocentrism’s)', () => {
    expect(modelOf(dpd(0))).toBe('dpd');
    expect(isSugar(dpd(0))).toBe(false);
    const empty = { site: { x: 1, y: 2 }, agent: null } as AnyInspection;
    const agent = { site: { x: 1, y: 2 }, agent: { id: 3, strategy: 'C', surrounded: false, neighbors: [] } } as unknown as AnyInspection;
    const ethno = { site: { x: 1, y: 2 }, agent: { id: 3, kin_marker: 3, neighbors: [] } } as unknown as AnyInspection;
    expect([empty, agent].map((v) => isDpdView(v, 'dpd'))).toEqual([true, true]);
    expect([empty, agent].map((v) => isDpdView(v, 'ethno'))).toEqual([false, false]);
    expect([empty, agent].map((v) => isDpdView(v, 'schelling'))).toEqual([false, false]);
    expect(isDpdView(ethno, 'dpd')).toBe(false);
    expect([empty, agent].map((v) => isEthnoView(v, 'dpd'))).toEqual([false, false]);
    expect(isTagsView(agent) || isRingView(agent) || isSugarView(agent) || isValleyView(agent) || isCivilView(agent) || isSpatialView(agent)).toBe(false);
    expect(isClassesView(agent) || isCultureView(agent)).toBe(false);
  });

  it('offers strategy, wealth, age and surrounded colors and no overlays, and is grouped last', () => {
    expect(COLOR_MODES.dpd).toEqual([
      ['strategy', 'Strategy'],
      ['wealth', 'Wealth'],
      ['age', 'Age'],
      ['surrounded', 'Surrounded'],
    ]);
    expect(MODEL_OVERLAYS.dpd).toEqual([]);
    const p = (id: string, config: object) => ({ id, name: id, source: '', description: '', config }) as unknown as Preset;
    expect(presetGroups([p('dpd', { model: 'dpd' }), p('ha', { model: 'ethno' })]).map((g) => g.label)).toEqual(['Ethnocentrism', 'Demographic PD']);
  });

  it('counts down to its last cycle, or never with none (the default)', () => {
    expect(ticksLeft(dpd(500), 490)).toBe(10);
    expect(ticksLeft(dpd(500), 505)).toBe(0);
    expect(ticksLeft(dpd(0), 5)).toBe(Infinity);
    expect(finishesUnpredictably(dpd(500))).toBe(false);
    expect(calendarYear(dpd(500), 5)).toBeNull();
  });
});

describe('image scoring', () => {
  const image = (end: number) => ({ model: 'image', end }) as unknown as ModelConfig;

  it('is read by its tag, and its cells by their shape, which no other model’s inspection shares', () => {
    expect(modelOf(image(0))).toBe('image');
    expect(isSugar(image(0))).toBe(false);
    const gap = { cell: { x: 10, y: 0 }, group: null, agent: null } as AnyInspection;
    const agent = { cell: { x: 1, y: 2 }, group: 0, agent: { id: 3, group: 0, strategy: 'k = 0', class: 'k', score: 1 } } as unknown as AnyInspection;
    expect([gap, agent].map(isImageView)).toEqual([true, true]);
    // The guards that read `site` check for one first (an image cell has none).
    for (const v of [gap, agent]) {
      expect([isSugarView(v), isRingView(v), isValleyView(v), isSpatialView(v)]).toEqual([false, false, false, false]);
      expect([isCivilView(v), isTagsView(v), isClassesView(v), isCultureView(v), isStructureView(v), isOpinionsView(v)]).toEqual([false, false, false, false, false, false]);
      expect([isEthnoView(v, 'image'), isDpdView(v, 'image')]).toEqual([false, false]);
    }
    const empty = { site: { x: 1, y: 2 }, agent: null } as AnyInspection;
    const sugar = { site: { x: 1, y: 2, resources: [] }, agent: null } as unknown as AnyInspection;
    expect([empty, sugar].map(isImageView)).toEqual([false, false]);
  });

  it('offers strategy, score and payoff colors and no overlays, and is grouped last', () => {
    expect(COLOR_MODES.image).toEqual([
      ['strategy', 'Strategy'],
      ['score', 'Score'],
      ['payoff', 'Payoff'],
    ]);
    expect(MODEL_OVERLAYS.image).toEqual([]);
    const p = (id: string, config: object) => ({ id, name: id, source: '', description: '', config }) as unknown as Preset;
    expect(presetGroups([p('ns', { model: 'image' }), p('dpd', { model: 'dpd' }), p('ha', { model: 'ethno' })]).map((g) => g.label)).toEqual([
      'Ethnocentrism',
      'Demographic PD',
      'Image Scoring',
    ]);
  });

  it('counts down to its last generation, or never with none (the default)', () => {
    expect(ticksLeft(image(500), 490)).toBe(10);
    expect(ticksLeft(image(500), 505)).toBe(0);
    expect(ticksLeft(image(0), 5)).toBe(Infinity);
    expect(finishesUnpredictably(image(500))).toBe(false);
    expect(calendarYear(image(500), 5)).toBeNull();
  });
});
