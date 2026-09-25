import { describe, expect, it } from 'vitest';
import { scheduleLines } from './civil';
import type { CivilConfig } from './types';

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
