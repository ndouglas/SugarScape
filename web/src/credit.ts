export type CreditRole = 'lender' | 'borrower' | 'both';
export interface CreditLoan { lender: number; borrower: number; good: number; due: number }
/** `Sim.credit_graph()`: agents in some outstanding loan (by id) and the loans (in loan order). */
export interface CreditGraph { agents: { id: number; role: CreditRole }[]; loans: CreditLoan[] }

export const MAX_DRAWN_LOANS = 400;

/** The `max` loans with the largest `due` (ties: the earlier loan), in their original order (Decision 17). */
export function drawnLoans(loans: CreditLoan[], max = MAX_DRAWN_LOANS): CreditLoan[] {
  if (loans.length <= max) return loans;
  const keep = new Set(
    loans
      .map((l, i) => ({ due: l.due, i }))
      .sort((a, b) => b.due - a.due || a.i - b.i)
      .slice(0, max)
      .map(({ i }) => i),
  );
  return loans.filter((_, i) => keep.has(i));
}

/**
 * Each agent's level (Decision 16): 0 for pure lenders, otherwise 1 + the highest level among its lenders,
 * computed on strongly connected components so every loan that closes a cycle is ignored — members of a
 * cycle with no lender above it get level 0.
 */
export function creditLevels(ids: number[], loans: { lender: number; borrower: number }[]): Map<number, number> {
  const borrowersOf = new Map<number, number[]>(ids.map((id) => [id, []]));
  for (const l of loans) borrowersOf.get(l.lender)?.push(l.borrower);
  // Tarjan's strongly connected components.
  const index = new Map<number, number>();
  const low = new Map<number, number>();
  const component = new Map<number, number>();
  const stack: number[] = [];
  const onStack = new Set<number>();
  let next = 0;
  let components = 0;
  const strong = (v: number): void => {
    index.set(v, next);
    low.set(v, next);
    next++;
    stack.push(v);
    onStack.add(v);
    for (const w of borrowersOf.get(v) ?? []) {
      if (!index.has(w)) {
        strong(w);
        low.set(v, Math.min(low.get(v)!, low.get(w)!));
      } else if (onStack.has(w)) {
        low.set(v, Math.min(low.get(v)!, index.get(w)!));
      }
    }
    if (low.get(v) === index.get(v)) {
      let w: number;
      do {
        w = stack.pop()!;
        onStack.delete(w);
        component.set(w, components);
      } while (w !== v);
      components++;
    }
  };
  for (const id of ids) if (!index.has(id)) strong(id);
  // Components lending into each component (loans inside a component are cut).
  const lendersOf = new Map<number, Set<number>>();
  for (const l of loans) {
    const from = component.get(l.lender);
    const to = component.get(l.borrower);
    if (from === undefined || to === undefined || from === to) continue;
    let set = lendersOf.get(to);
    if (!set) lendersOf.set(to, (set = new Set()));
    set.add(from);
  }
  const level = new Map<number, number>();
  const levelOf = (c: number): number => {
    const known = level.get(c);
    if (known !== undefined) return known;
    let v = 0;
    for (const from of lendersOf.get(c) ?? []) v = Math.max(v, levelOf(from) + 1);
    level.set(c, v);
    return v;
  };
  return new Map(ids.map((id) => [id, levelOf(component.get(id)!)]));
}

export interface CreditLayout {
  /** Agent ids per level (row 0 = pure lenders), each row in id order. */
  rows: number[][];
  /** The loans drawn. */
  loans: CreditLoan[];
  roles: Map<number, CreditRole>;
  /** The whole graph's agents and loans, and the loans left out. */
  agents: number;
  totalLoans: number;
  omitted: number;
}

export function creditLayout(graph: CreditGraph, max = MAX_DRAWN_LOANS): CreditLayout {
  const loans = drawnLoans(graph.loans, max);
  const ids = [...new Set(loans.flatMap((l) => [l.lender, l.borrower]))].sort((a, b) => a - b);
  const levels = creditLevels(ids, loans);
  const rows: number[][] = [];
  for (const id of ids) {
    const level = levels.get(id) ?? 0;
    while (rows.length <= level) rows.push([]);
    rows[level].push(id);
  }
  return {
    rows,
    loans,
    roles: new Map(graph.agents.map((a) => [a.id, a.role])),
    agents: graph.agents.length,
    totalLoans: graph.loans.length,
    omitted: graph.loans.length - loans.length,
  };
}

/** "N agents · M loans · L levels", noting any loans left out. */
export function creditHeader(layout: CreditLayout): string {
  if (layout.totalLoans === 0) return 'No outstanding loans.';
  const count = (n: number, word: string) => `${n} ${word}${n === 1 ? '' : 's'}`;
  const text = `${count(layout.agents, 'agent')} · ${count(layout.totalLoans, 'loan')} · ${count(layout.rows.length, 'level')}`;
  return layout.omitted > 0 ? `${text} · the ${layout.loans.length} largest loans drawn (${layout.omitted} omitted)` : text;
}
