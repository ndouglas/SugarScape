import { describe, expect, it } from 'vitest';
import { headingText } from './inspect-panel';

const plan = (path: [number, number][]) => ({ target_x: 3, target_y: 4, path, walked: true });

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
