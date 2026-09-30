import { describe, expect, it } from 'vitest';
import { hoardColumn, hoardStatusText, lastBout } from './hoard';

describe('the hoard status line', () => {
  it('names the bout just run, from the next one the core keeps', () => {
    expect(lastBout(1, 1, 20)).toBeNull();
    expect(lastBout(1, 2, 20)).toEqual({ day: 1, bout: 1 });
    expect(lastBout(11, 1, 20)).toEqual({ day: 10, bout: 20 });
    // A finished season's next bout is day 101, bout 1: its last was day 100's bout 20.
    expect(lastBout(101, 1, 20)).toEqual({ day: 100, bout: 20 });
  });

  it('reads generation, day and bout, and the public food', () => {
    const s = { generation: 3, day: 12, bout: 6, public: 64, season_over: false, living: 18 };
    expect(hoardStatusText(s, { bouts: 20 })).toBe('Generation 3 · day 12 · bout 5 · public food 64');
    expect(hoardStatusText({ ...s, generation: 1, day: 1, bout: 1, public: 0 }, { bouts: 20 })).toBe('Generation 1 · before the first bout · public food 0');
    expect(hoardStatusText({ ...s, day: 101, bout: 1, public: 0, season_over: true }, { bouts: 20 })).toBe(
      'Generation 3 · day 100 · bout 20 · public food 0 · season over',
    );
  });
});

describe('the hoard frame’s columns', () => {
  it('finds the clicked agent’s columns as the core does (view.rs), less the gutter', () => {
    expect(hoardColumn(0, 20, 200)).toEqual([0, 9]);
    expect(hoardColumn(35, 20, 200)).toEqual([30, 39]);
    expect(hoardColumn(199, 20, 200)).toEqual([190, 199]);
    expect(hoardColumn(199, 7, 200)).toEqual([171, 199]);
  });
});
