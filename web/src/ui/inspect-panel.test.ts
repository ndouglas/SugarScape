import { describe, expect, it } from 'vitest';
import { headingText } from './inspect-panel';

describe('headingText', () => {
  it('says "Staying" once the path is empty (arrived, or nothing to plan)', () => {
    expect(headingText({ target_x: 3, target_y: 4, path: [] })).toBe('Staying');
  });

  it('singularizes exactly one step left', () => {
    expect(headingText({ target_x: 3, target_y: 4, path: [[3, 4]] })).toBe('(3, 4), 1 step left');
  });

  it('pluralizes more than one step left', () => {
    expect(headingText({ target_x: 3, target_y: 4, path: [[1, 4], [2, 4], [3, 4]] })).toBe('(3, 4), 3 steps left');
  });
});
