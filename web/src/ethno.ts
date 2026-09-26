// The ethnocentrism model's pure page helpers (milestone 16): Inspect's rows.
import type { EthnoAgentView, EthnoStrategy } from './types';

/** Each strategy's name and what it does. */
export const STRATEGY_TEXT: Record<EthnoStrategy, [string, string]> = {
  E: ['ethnocentric', 'helps its own color only'],
  H: ['humanitarian', 'helps everyone'],
  S: ['selfish', 'helps no one'],
  T: ['traitorous', 'helps other colors only'],
  kin: ['kin', 'helps its kin only'],
  nonkin: ['non-kin', 'helps non-kin only'],
  mixed: ['mixed', 'helps some colors and not others'],
};

/** "helped it", "helped it twice" (pair play `twice`), or nothing. */
function times(n: number, verb: string): string | null {
  return n === 0 ? null : n === 1 ? verb : n === 2 ? `${verb} twice` : `${verb} ${n} times`;
}

/**
 * An agent's Inspect rows: its tag and strategy (and, with kin strategies, what it judges same and
 * other by), this period's PTR and helps, its lineage, kin marker and age, then each occupied
 * neighbor (up, left, right, down) with its tag, strategy, kinship and who helped whom.
 */
export function ethnoRows(a: EthnoAgentView, kinStrategies: boolean): [string, string][] {
  const [name, does] = STRATEGY_TEXT[a.strategy];
  const rows: [string, string][] = [
    ['Agent', `#${a.id} · tag ${a.tag}`],
    ['Strategy', `${name}: ${does}`],
  ];
  if (kinStrategies) rows.push(['Judges by', a.basis === 'kin' ? 'its kin marker (kin or not)' : 'its tag (same color or not)']);
  rows.push(
    ['PTR', a.ptr.toFixed(3)],
    ['Helps', `gave ${a.given} · received ${a.received}`],
    ['Lineage', a.lineage === a.id ? `#${a.lineage} (founded it)` : `#${a.lineage}`],
    ['Kin marker', a.kin_marker === a.id ? `#${a.kin_marker} (founded the family)` : `#${a.kin_marker}`],
    ['Age', `${a.age} ${a.age === 1 ? 'period' : 'periods'}`],
  );
  if (a.neighbors.length === 0) return [...rows, ['Neighbors', 'none']];
  for (const n of a.neighbors) {
    const help = [times(n.helped, 'helped it'), times(n.helped_by, 'helped by it')].filter((t) => t !== null);
    const parts = [`tag ${n.tag}`, STRATEGY_TEXT[n.strategy][0], n.related ? 'related' : 'unrelated', help.length > 0 ? help.join(', ') : 'no help either way'];
    rows.push([`(${n.x}, ${n.y})`, parts.join(' · ')]);
  }
  return rows;
}
