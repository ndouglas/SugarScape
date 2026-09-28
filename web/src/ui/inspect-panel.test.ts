import { describe, expect, it } from 'vitest';
import { headingText, memoryText } from './inspect-panel';

const plan = (path: [number, number][]) => ({ target_x: 3, target_y: 4, path, walked: true });
const memory = (sites: number, spots: number) => ({ remembers: true, sites, spots });

describe('headingText', () => {
  it('says "Staying" once the path is empty at the target (arrived, or nothing to plan)', () => {
    expect(headingText(plan([]), [3, 4])).toBe('Staying');
  });

  it("says it can't reach a target elsewhere when no path was found", () => {
    expect(headingText(plan([]), [1, 1])).toBe("Can't reach (3, 4)");
  });

  it('singularizes exactly one step left', () => {
    expect(headingText(plan([[3, 4]]), [2, 4])).toBe('(3, 4), 1 step left');
  });

  it('pluralizes more than one step left', () => {
    expect(headingText(plan([[1, 4], [2, 4], [3, 4]]), [0, 4])).toBe('(3, 4), 3 steps left');
  });
});

describe('memoryText', () => {
  it("says it doesn't remember for a non-rememberer", () => {
    expect(memoryText({ remembers: false, sites: 0, spots: 0 })).toBe("Doesn't remember");
  });

  it("says it doesn't remember when memory is off (null)", () => {
    expect(memoryText(null)).toBe("Doesn't remember");
  });

  it('pluralizes zero sites and zero truffle spots', () => {
    expect(memoryText(memory(0, 0))).toBe('0 sites (0 truffle spots)');
  });

  it('singularizes exactly one site and one truffle spot', () => {
    expect(memoryText(memory(1, 1))).toBe('1 site (1 truffle spot)');
  });

  it('pluralizes two sites and two truffle spots', () => {
    expect(memoryText(memory(2, 2))).toBe('2 sites (2 truffle spots)');
  });

  it('singularizes and pluralizes each noun independently', () => {
    expect(memoryText(memory(1, 2))).toBe('1 site (2 truffle spots)');
    expect(memoryText(memory(2, 1))).toBe('2 sites (1 truffle spot)');
  });
});
