import { describe, expect, it } from 'vitest';
import { ethnoRows } from './ethno';
import type { EthnoAgentView } from './types';

const agent = (c: Partial<EthnoAgentView>): EthnoAgentView => ({
  id: 40,
  tag: 2,
  strategy: 'E',
  basis: 'tag',
  ptr: 0.14,
  given: 1,
  received: 1,
  lineage: 7,
  kin_marker: 40,
  age: 12,
  neighbors: [
    { x: 5, y: 4, tag: 2, strategy: 'H', related: true, helped: 1, helped_by: 1 },
    { x: 4, y: 5, tag: 0, strategy: 'S', related: false, helped: 0, helped_by: 0 },
  ],
  ...c,
});

describe('ethnoRows', () => {
  it('shows the traits, this period’s PTR and helps, the lineage, and each neighbor with who helped whom', () => {
    expect(ethnoRows(agent({}), false)).toEqual([
      ['Agent', '#40 · tag 2'],
      ['Strategy', 'ethnocentric: helps its own color only'],
      ['PTR', '0.140'],
      ['Helps', 'gave 1 · received 1'],
      ['Lineage', '#7'],
      ['Kin marker', '#40 (founded the family)'],
      ['Age', '12 periods'],
      ['(5, 4)', 'tag 2 · humanitarian · related · helped it, helped by it'],
      ['(4, 5)', 'tag 0 · selfish · unrelated · no help either way'],
    ]);
  });

  it('says what a kin strategist judges by, counts a pair played twice, and names a lone founder', () => {
    const a = agent({
      id: 7,
      strategy: 'kin',
      basis: 'kin',
      lineage: 7,
      kin_marker: 7,
      age: 1,
      neighbors: [{ x: 5, y: 4, tag: 1, strategy: 'nonkin', related: false, helped: 2, helped_by: 0 }],
    });
    expect(ethnoRows(a, true)).toEqual([
      ['Agent', '#7 · tag 2'],
      ['Strategy', 'kin: helps its kin only'],
      ['Judges by', 'its kin marker (kin or not)'],
      ['PTR', '0.140'],
      ['Helps', 'gave 1 · received 1'],
      ['Lineage', '#7 (founded it)'],
      ['Kin marker', '#7 (founded the family)'],
      ['Age', '1 period'],
      ['(5, 4)', 'tag 1 · non-kin · unrelated · helped it twice'],
    ]);
    expect(ethnoRows(agent({ neighbors: [] }), false).at(-1)).toEqual(['Neighbors', 'none']);
    expect(ethnoRows(agent({ basis: 'tag' }), true)[2]).toEqual(['Judges by', 'its tag (same color or not)']);
  });
});
