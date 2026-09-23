import { describe, expect, it } from 'vitest';
import { scheduleLines } from './schedule';
import type { Config } from './types';

const config = (schedule: Config['schedule'], outbreaks: Config['disease']['outbreaks']) =>
  ({ schedule, disease: { outbreaks } }) as unknown as Config;

describe('scheduleLines', () => {
  it('lists scheduled changes and outbreaks in tick order', () => {
    const lines = scheduleLines(
      config(
        [
          { tick: 100, set: { 'diffusion.enabled': true } },
          { tick: 50, set: { 'pollution.enabled': true } },
        ],
        [
          { tick: 300, agents: 5 },
          { tick: 75, agents: 1 },
        ],
      ),
    );
    expect(lines).toEqual([
      't = 50 · pollution.enabled = true',
      't = 75 · new disease → 1 agent',
      't = 100 · diffusion.enabled = true',
      't = 300 · new disease → 5 agents',
    ]);
  });

  it('is empty without entries', () => {
    expect(scheduleLines(config([], []))).toEqual([]);
  });

  it('appends length information when set', () => {
    const lines = scheduleLines(
      config(
        [],
        [
          { tick: 100, agents: 10, length: { min: 10, max: 10 } },
          { tick: 200, agents: 5, length: { min: 3, max: 8 } },
        ],
      ),
    );
    expect(lines).toEqual([
      't = 100 · new disease (10 bits) → 10 agents',
      't = 200 · new disease (3–8 bits) → 5 agents',
    ]);
  });
});
