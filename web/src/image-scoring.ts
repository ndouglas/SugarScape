// Image scoring's pure page helpers (milestone 21): Inspect's rows. (`image.ts` is the map import's.)
import type { ImageConfig, ImageInspection } from './types';

/** Whole numbers as they are, others to two decimals. */
const num = (n: number): string => (Number.isInteger(n) ? String(n) : n.toFixed(2));

/** "1 time", "3 times". */
const times = (n: number): string => `${n} ${n === 1 ? 'time' : 'times'}`;

/**
 * A cell's Inspect rows: the cell, its group (with more than one), then the agent of the generation
 * that last played — its strategy and whether it helps at a generation's start, its score (and, with
 * private records, how many others have seen it act and their mean record of it), its standing
 * (only when standing plays), its payoff and the help it gave and received. A gap or a tile's unused
 * cell says so.
 */
export function imageRows(view: ImageInspection, config: Pick<ImageConfig, 'groups' | 'strategies'>): [string, string][] {
  const rows: [string, string][] = [['Cell', `(${view.cell.x}, ${view.cell.y})`]];
  if (view.group === null) return [...rows, ['Agent', 'none (a gap between groups)']];
  if (config.groups > 1) rows.push(['Group', `${view.group + 1} of ${config.groups}`]);
  const a = view.agent;
  if (!a) return [...rows, ['Agent', 'none (an unused cell of the group’s tile)']];
  rows.push(
    ['Agent', `#${a.id}`],
    ['Strategy', `${a.strategy} · ${a.cooperative ? 'helps' : 'refuses'} at a generation’s start`],
    ['Score', String(a.score)],
  );
  if (a.known !== null) {
    const seen = a.known === 0 || a.mean_view === null ? 'nobody yet: the others record it as 0' : `${a.known} ${a.known === 1 ? 'other' : 'others'}, whose mean record is ${num(a.mean_view)}`;
    rows.push(['Seen by', seen]);
  }
  if (config.strategies.includes('standing')) rows.push(['Standing', a.standing ? 'good' : 'bad']);
  rows.push(['Payoff', num(a.payoff)], ['This generation', `helped ${times(a.given)}, was helped ${times(a.received)}`]);
  return rows;
}
