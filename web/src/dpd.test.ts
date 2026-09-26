import { describe, expect, it } from 'vitest';
import { dpdRows } from './dpd';
import type { DpdAgentView } from './types';

const agent = (c: Partial<DpdAgentView>): DpdAgentView => ({
  id: 57,
  strategy: 'C',
  wealth: 14,
  age: 37,
  max_age: 100,
  surrounded: false,
  income: 4,
  games: 2,
  neighbors: [
    { x: 5, y: 3, id: 12, strategy: 'C', payoff: 5, their_payoff: 5 },
    { x: 4, y: 4, id: 80, strategy: 'D', payoff: -6, their_payoff: 6 },
  ],
  ...c,
});

describe('dpdRows', () => {
  it('shows the strategy, wealth, age against the maximum, this cycle’s payoffs, and each neighbor with what a game pays', () => {
    expect(dpdRows(agent({}))).toEqual([
      ['Agent', '#57 · cooperator'],
      ['Wealth', '14'],
      ['Age', '37 cycles (maximum 100)'],
      ['Surrounded', 'no'],
      ['This cycle', '4 from 2 games'],
      ['(5, 3)', '#12 · cooperator · a game pays this agent 5 and the neighbor 5'],
      ['(4, 4)', '#80 · defector · a game pays this agent -6 and the neighbor 6'],
    ]);
  });

  it('says when a cooperator is surrounded, with no maximum age, fractional wealth and a lone agent', () => {
    const a = agent({ surrounded: true, max_age: 0, age: 1, wealth: 7.25, income: -0.5, games: 1, neighbors: [] });
    expect(dpdRows(a)).toEqual([
      ['Agent', '#57 · cooperator'],
      ['Wealth', '7.25'],
      ['Age', '1 cycle (no maximum)'],
      ['Surrounded', 'yes: all eight neighbors (Moore) cooperate'],
      ['This cycle', '-0.50 from 1 game'],
      ['Neighbors', 'none'],
    ]);
    const d = agent({ strategy: 'D', neighbors: [{ x: 0, y: 29, id: 3, strategy: 'C', payoff: 6, their_payoff: -6 }] });
    expect(dpdRows(d)[0]).toEqual(['Agent', '#57 · defector']);
    expect(dpdRows(d).at(-1)).toEqual(['(0, 29)', '#3 · cooperator · a game pays this agent 6 and the neighbor -6']);
  });
});
