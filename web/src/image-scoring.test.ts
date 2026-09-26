import { describe, expect, it } from 'vitest';
import { imageRows } from './image-scoring';
import type { ImageAgentView, ImageClass, ImageInspection } from './types';

const agent = (a: Partial<ImageAgentView>): ImageAgentView => ({
  id: 57,
  group: 0,
  strategy: 'k = 0',
  class: 'k',
  cooperative: true,
  score: 3,
  standing: true,
  known: null,
  mean_view: null,
  payoff: 1.5,
  given: 2,
  received: 1,
  ...a,
});

const cell = (a: ImageAgentView | null, group: number | null = 0): ImageInspection => ({ cell: { x: 4, y: 1 }, group, agent: a });
const one = { groups: 1, strategies: ['k'] as ImageClass[] };

describe('imageRows', () => {
  it('shows the strategy, whether it helps at the start, the score, the payoff and this generation’s help (NS98, one group)', () => {
    expect(imageRows(cell(agent({})), one)).toEqual([
      ['Cell', '(4, 1)'],
      ['Agent', '#57'],
      ['Strategy', 'k = 0 · helps at a generation’s start'],
      ['Score', '3'],
      ['Payoff', '1.50'],
      ['This generation', 'helped 2 times, was helped 1 time'],
    ]);
  });

  it('names the group of an island model, and with observers how many others have seen the agent act', () => {
    const a = agent({ id: 9, group: 41, strategy: 'k = 2, h = 5 (AND)', class: 'and', cooperative: false, score: -1, known: 4, mean_view: -0.75, payoff: 0, given: 1, received: 0 });
    expect(imageRows(cell(a, 41), { groups: 100, strategies: ['and'] })).toEqual([
      ['Cell', '(4, 1)'],
      ['Group', '42 of 100'],
      ['Agent', '#9'],
      ['Strategy', 'k = 2, h = 5 (AND) · refuses at a generation’s start'],
      ['Score', '-1'],
      ['Seen by', '4 others, whose mean record is -0.75'],
      ['Payoff', '0'],
      ['This generation', 'helped 1 time, was helped 0 times'],
    ]);
    expect(imageRows(cell(agent({ known: 1, mean_view: 2 })), one)[4]).toEqual(['Seen by', '1 other, whose mean record is 2']);
    expect(imageRows(cell(agent({ known: 0, mean_view: null })), one)[4]).toEqual(['Seen by', 'nobody yet: the others record it as 0']);
  });

  it('shows standing when standing plays', () => {
    const a = agent({ strategy: 'standing', class: 'standing', standing: false, score: -1 });
    expect(imageRows(cell(a), { groups: 1, strategies: ['binary', 'standing'] })).toContainEqual(['Standing', 'bad']);
    expect(imageRows(cell(agent({})), one).map(([k]) => k)).not.toContain('Standing');
  });

  it('says when a cell is a gap between groups or an unused cell of a tile', () => {
    expect(imageRows(cell(null, null), { groups: 4, strategies: ['k'] })).toEqual([
      ['Cell', '(4, 1)'],
      ['Agent', 'none (a gap between groups)'],
    ]);
    expect(imageRows(cell(null, 2), { groups: 4, strategies: ['k'] })).toEqual([
      ['Cell', '(4, 1)'],
      ['Group', '3 of 4'],
      ['Agent', 'none (an unused cell of the group’s tile)'],
    ]);
  });
});
