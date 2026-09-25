import { describe, expect, it } from 'vitest';
import { citizenRows, scheduleLines, shownCitizen } from './civil';
import type { CitizenView, CivilConfig, CivilInspection } from './types';

const civil = (c: Partial<CivilConfig>): CivilConfig => ({ schedule: [], ramps: [], ...c }) as CivilConfig;

describe('scheduleLines', () => {
  it('lists steps and ramps by their first tick, steps first', () => {
    const c = civil({
      schedule: [
        { tick: 77, set: { legitimacy: 0.7 } },
        { tick: 10, set: { cop_density: 0.04, 'quirks.floor_ratio': true } },
      ],
      ramps: [{ path: 'legitimacy', start: 77, end: 147, to: 0.2 }],
    });
    expect(scheduleLines(c)).toEqual([
      't = 10: cop_density → 0.04, quirks.floor_ratio → true',
      't = 77: legitimacy → 0.7',
      't = 77–147: legitimacy moves steadily to 0.2',
    ]);
  });

  it('is empty without a schedule', () => {
    expect(scheduleLines(civil({}))).toEqual([]);
  });
});

const agent = (c: Partial<CitizenView>): CitizenView => ({
  id: 1,
  state: 'quiet',
  hardship: 0.5,
  risk_aversion: 0.25,
  grievance: 0.09,
  arrest_probability: 0.9,
  net_risk: 0.225,
  jail_left: null,
  jail_life: false,
  group: null,
  age: null,
  death_age: null,
  ...c,
});

describe('citizenRows', () => {
  it('shows H, R, G and the risk it runs, and Model II’s group and age', () => {
    expect(citizenRows(agent({}))).toEqual([
      ['Agent', '#1 · quiet'],
      ['Hardship (H)', '0.50'],
      ['Risk aversion (R)', '0.25'],
      ['Grievance (G)', '0.09 = H × (1 − L)'],
      ['Arrest risk (P)', '0.90 · net risk R·P 0.23'],
    ]);
    const rows = citizenRows(agent({ group: 'green', age: 12, death_age: 150 }));
    expect(rows.slice(-2)).toEqual([
      ['Group', 'Green'],
      ['Age', '12 of 150'],
    ]);
  });

  it('says how long a jailed agent has left instead of its risk', () => {
    expect(citizenRows(agent({ state: 'jailed', jail_left: 4 }))[0]).toEqual(['Agent', '#1 · jailed (4 ticks left)']);
    const life = citizenRows(agent({ state: 'jailed', jail_life: true }));
    expect(life[0]).toEqual(['Agent', '#1 · jailed (for life)']);
    expect(life.some(([k]) => k === 'Arrest risk (P)')).toBe(false);
  });
});

describe('shownCitizen', () => {
  const free = agent({ id: 2 });
  const jailed = agent({ id: 3, state: 'jailed', jail_left: 2 });
  const view: CivilInspection = { site: { x: 0, y: 0 }, agent: free, cop: null, jailed: [jailed] };

  it('shows the followed agent while it is jailed at this site', () => {
    expect(shownCitizen(view, 3)).toBe(jailed);
    expect(shownCitizen(view, 2)).toBe(free);
  });

  it('shows the free agent otherwise', () => {
    expect(shownCitizen(view, null)).toBe(free);
    expect(shownCitizen(view, 99)).toBe(free);
    expect(shownCitizen({ ...view, agent: null }, null)).toBeNull();
  });
});
