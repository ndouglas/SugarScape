import { describe, expect, it } from 'vitest';
import { creditHeader, creditLayout, creditLevels, drawnLoans, type CreditGraph, type CreditLoan } from './credit';

const loan = (lender: number, borrower: number, due = 1): CreditLoan => ({ lender, borrower, good: 0, due });
const idsOf = (loans: CreditLoan[]) => [...new Set(loans.flatMap((l) => [l.lender, l.borrower]))].sort((a, b) => a - b);
const levels = (loans: CreditLoan[]) => Object.fromEntries(creditLevels(idsOf(loans), loans));
const graph = (loans: CreditLoan[]): CreditGraph => {
  const lenders = new Set(loans.map((l) => l.lender));
  const borrowers = new Set(loans.map((l) => l.borrower));
  return {
    agents: idsOf(loans).map((id): CreditGraph['agents'][number] => ({
      id,
      role: lenders.has(id) ? (borrowers.has(id) ? 'both' : 'lender') : 'borrower',
    })),
    loans,
  };
};

describe('creditLevels', () => {
  it('puts a chain one level per link', () => {
    expect(levels([loan(1, 2), loan(2, 3)])).toEqual({ 1: 0, 2: 1, 3: 2 });
  });

  it('takes the longest path through a diamond', () => {
    expect(levels([loan(1, 2), loan(1, 3), loan(2, 4), loan(3, 4), loan(1, 4)])).toEqual({ 1: 0, 2: 1, 3: 1, 4: 2 });
  });

  it('cuts the loans that close a cycle', () => {
    expect(levels([loan(1, 2), loan(2, 1)])).toEqual({ 1: 0, 2: 0 });
    expect(levels([loan(5, 1), loan(1, 2), loan(2, 1)])).toEqual({ 1: 1, 2: 1, 5: 0 });
    expect(levels([loan(1, 2), loan(2, 3), loan(3, 1), loan(3, 4)])).toEqual({ 1: 0, 2: 0, 3: 0, 4: 1 });
  });
});

describe('creditLayout', () => {
  it('lays out one row per level, in id order', () => {
    const layout = creditLayout(graph([loan(9, 4), loan(9, 2), loan(4, 7)]));
    expect(layout.rows).toEqual([[9], [2, 4], [7]]);
    expect(layout.roles.get(4)).toBe('both');
    expect(creditHeader(layout)).toBe('4 agents · 3 loans · 3 levels');
  });

  it('draws only the largest loans of a big graph', () => {
    const layout = creditLayout(graph(Array.from({ length: 401 }, (_, i) => loan(1000 + i, 2000 + i, i))));
    expect(layout.loans).toHaveLength(400);
    expect(layout.loans.some((l) => l.due === 0)).toBe(false);
    expect(layout.omitted).toBe(1);
    expect(creditHeader(layout)).toBe('802 agents · 401 loans · 2 levels · the 400 largest loans drawn (1 omitted)');
    expect(drawnLoans([loan(1, 2, 5), loan(3, 4, 5), loan(5, 6, 1)], 2)).toEqual([loan(1, 2, 5), loan(3, 4, 5)]);
  });

  it('says when there is nothing to draw', () => {
    const empty = creditLayout({ agents: [], loans: [] });
    expect(empty.rows).toEqual([]);
    expect(creditHeader(empty)).toBe('No outstanding loans.');
  });
});
