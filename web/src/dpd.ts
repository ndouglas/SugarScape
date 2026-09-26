// The demographic Prisoner's Dilemma's pure page helpers (milestone 19): Inspect's rows.
import type { DpdAgentView } from './types';

/** A strategy's name. */
const STRATEGY: Record<'C' | 'D', string> = { C: 'cooperator', D: 'defector' };

/** Whole numbers as they are, others to two decimals; an infinite payoff serializes as null, shown as "—". */
const num = (n: number | null): string => (n === null ? '—' : Number.isInteger(n) ? String(n) : n.toFixed(2));

/** "1 game", "3 games" (and cycles alike). */
const count = (n: number, one: string): string => `${n} ${n === 1 ? one : `${one}s`}`;

/**
 * An agent's Inspect rows: its strategy, wealth and age (against its maximum), whether it is a
 * surrounded cooperator, this cycle's payoffs and games, then each occupied neighbor (up, left,
 * right, down) with what one game between them pays each under the current payoffs.
 */
export function dpdRows(a: DpdAgentView): [string, string][] {
  const rows: [string, string][] = [
    ['Agent', `#${a.id} · ${STRATEGY[a.strategy]}`],
    ['Wealth', num(a.wealth)],
    ['Age', `${count(a.age, 'cycle')} (${a.max_age === 0 ? 'no maximum' : `maximum ${a.max_age}`})`],
    ['Surrounded', a.surrounded ? 'yes: all eight neighbors (Moore) cooperate' : 'no'],
    ['This cycle', `${num(a.income)} from ${count(a.games, 'game')}`],
  ];
  if (a.neighbors.length === 0) return [...rows, ['Neighbors', 'none']];
  for (const n of a.neighbors) {
    rows.push([`(${n.x}, ${n.y})`, `#${n.id} · ${STRATEGY[n.strategy]} · a game pays this agent ${num(n.payoff)} and the neighbor ${num(n.their_payoff)}`]);
  }
  return rows;
}
