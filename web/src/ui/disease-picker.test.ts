import { describe, expect, it } from 'vitest';
import { diseaseOptions } from './disease-picker';

const list = [
  { id: 0, bits: '101', carriers: 4 },
  { id: 1, bits: '0011', carriers: 0 },
];

describe('diseaseOptions', () => {
  it('lists diseases by id and bits, with New disease first when allowed', () => {
    expect(diseaseOptions(list, true)).toEqual([
      ['-1', 'New disease'],
      ['0', '#0 · 101'],
      ['1', '#1 · 0011'],
    ]);
  });
  it('omits New disease for vaccination', () => {
    expect(diseaseOptions(list, false)).toEqual([
      ['0', '#0 · 101'],
      ['1', '#1 · 0011'],
    ]);
  });
});
