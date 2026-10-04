import type { AuctionsConfig, AuctionsInspection, ColorMode } from './types';
const fmt = (v: number): string => v.toFixed(3);

/** Inspect uses the selected occupancy window; pair counts include every auction. */
export function auctionRows(view: AuctionsInspection, mode: ColorMode): [string, string][] {
  const rows: [string, string][] = [
    ['Session', `${view.period} of ${view.horizon} periods · tick ${view.tick} · ${view.stable} stable periods`],
    ['Allocation', `Payment ${fmt(view.payment)} · ${view.shares.slice(0, view.greedy.length).map((share, i) => `bidder ${i + 1} share ${fmt(share)}`).join(', ')} · epsilon ${fmt(view.epsilon)}`],
    ['Strategies', `Greedy ${view.greedy.map(fmt).join(', ')} · played ${view.played.map(fmt).join(', ')}`],
  ];
  if (view.fringe_bid !== null && view.fringe_bid !== undefined) rows.push(['Fringe', `Bid ${fmt(view.fringe_bid)} · win share ${fmt(view.fringe_share ?? 0)}`]);
  if (mode === 'values') {
    const bidder = Math.floor(view.y / 36);
    const action = Math.floor(view.x / 12);
    const learner = view.learners[bidder];
    // Core inspection includes the action's counterfactual reward when the selected value cell exists.
    if (learner && action < view.grid.length) {
      rows.push(['Action', `Bidder ${bidder + 1} · bid ${fmt(view.grid[action])} · Q ${fmt(learner.q[action])} · chosen ${learner.chosen[action]} · updated ${learner.updated[action]} · hypothetical reward ${view.hypothetical_reward === null ? 'unavailable' : fmt(view.hypothetical_reward)}`]);
    }
    rows.push(['View', 'Q values, chosen counts and update counts for each bidder']);
  } else {
    const size = view.grid.length;
    const first = Math.floor(view.x / 12);
    const second = size - 1 - Math.floor(view.y / 12);
    const index = first * size + second;
    const total = mode === 'late' ? view.late_count : view.whole_count;
    const histogram = mode === 'late' ? view.late_occupancy : view.occupancy;
    if (first >= 0 && first < size && second >= 0 && second < size) {
      const count = histogram[index];
      rows.push(['Pair bids', `Bidder 1 ${fmt(view.grid[first])} · bidder 2 ${fmt(view.grid[second])}`]);
      rows.push(['Pair occupancy', `${count} of ${total} auctions (${total ? `${fmt(100 * count / total)}%` : 'unavailable'}) · ${mode === 'late' ? 'final 20%' : 'whole run'}`]);
    }
    rows.push(['View', `Played pairs of bidders 1 and 2${view.greedy.length === 3 ? ' · first-two-bidder projection' : ''}`]);
  }
  const o = view.outcome;
  if (o) {
    rows.push(
      ['Completion', `Fixed horizon reached · final strategy window ${o.converged ? 'stable' : 'unstable'}`],
      ['Terminal policy', `Expected seller revenue ${fmt(o.terminal_revenue)} · bidder rewards ${o.terminal_profits.map(fmt).join(', ')}`],
      ['Deviation gains', o.deviation_gain.map(fmt).join(', ')],
      ['Realized revenue', `Whole run ${fmt(o.whole_revenue)} · final 20% ${fmt(o.late_revenue)}`],
      ['Policy status', `Top profile ${o.top_profile ? 'yes' : 'no'} · below top ${o.below_top ? 'yes' : 'no'} · low with profitable deviation ${o.low_non_nash ? 'yes' : 'no'}`],
    );
  } else rows.push(['Completion', 'Runs to the fixed horizon; transient strategy stability does not finish the session']);
  return rows;
}

/** Tick charts show each world's economic batching explicitly, including Compare. */
export function auctionChartCaption(title: string, configs: AuctionsConfig[]): string {
  return `${title} · ${configs.map((c, i) => `${configs.length > 1 ? `${i === 0 ? 'A' : 'B'}: ` : ''}${c.periods_per_tick} periods/tick`).join(' · ')}`;
}

export function auctionChartLines<T extends { key: string }>(lines: T[], config: AuctionsConfig): T[] {
  return config.bidders === 3 ? lines : lines.filter((line) => !['bid_3', 'greedy_3', 'profit_3'].includes(line.key));
}

/** The selected window's denominator comes from the economic clock, never display ticks. */
export function auctionLegend(config: AuctionsConfig, mode: ColorMode, periods: number): { status: string; items: import('./ui/legend').LegendItem[] } {
  const low = fmt((config.out_bids > 0 ? 1 - config.out_bids : 1) / (config.bids + 1));
  const high = fmt(config.bids / (config.bids + 1));
  const clock = `${periods} of ${config.horizon} periods`;
  if (mode === 'values') return {
    status: `${clock} · Action bids: left ${low} → right ${high} · Bidders 1–${config.bidders}: top → bottom · Rows per bidder: Q, chosen, updated`,
    items: [
      { label: 'Greedy action · gold top/left edges', mark: { kind: 'swatch', color: '#f0d246' } },
      { label: 'Played action · cyan bottom/right edges', mark: { kind: 'swatch', color: '#46dce6' } },
      { label: 'Q / max Q; chosen and updated / completed periods · low → high', mark: { kind: 'ramp', from: '#1e2864', to: '#dc9664' } },
    ],
  };
  const late = mode === 'late';
  const count = late ? Math.max(0, periods - Math.floor(.8 * config.horizon)) : periods;
  return {
    status: `${clock} · ${count} auctions counted · ${late ? 'final 20%' : 'whole run'} · Bidder 1: left ${low} → right ${high} · Bidder 2: bottom ${low} → top ${high}${config.bidders === 3 ? ' · first-two-bidder projection' : ''}`,
    items: [
      { label: 'Current played pair · gold border', mark: { kind: 'swatch', color: '#f0d246' } },
      { label: 'Played-pair counts relative to the most frequent pair · low → high', mark: { kind: 'ramp', from: '#14233c', to: '#e69b8c' } },
    ],
  };
}
