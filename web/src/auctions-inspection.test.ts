import { describe, expect, it } from 'vitest';
import { auctionRows, auctionChartCaption, auctionChartLines } from './auctions';
import type { AuctionsConfig, AuctionsInspection } from './types';
const view: AuctionsInspection = {
  x: 0, y: 12, panel: 'bids', bidder: null, action: null, count: 4, frequency: .4, q: null, chosen: null, updated: null, hypothetical_reward: null,
  tick: 2, period: 10, horizon: 11, grid: [.05, .10], greedy: [.1, .1, .05], played: [.05, .1, .1], shares: [0, .5, .5], fringe_bid: null, fringe_share: null, payment: .1, epsilon: .025, stable: 4,
  occupancy: [4, 1, 2, 3], late_occupancy: [1, 0, 1, 0], whole_count: 10, late_count: 2,
  learners: [{ q: [2, 3], greedy: 1, chosen: [3, 7], updated: [3, 7] }], equilibria: [], outcome: null, agent: null,
};
describe('auction inspection', () => {
  it('uses the selected histogram and its economic denominator', () => {
    expect(auctionRows(view, 'bids')).toContainEqual(['Pair occupancy', '4 of 10 auctions (40.000%) · whole run']);
    expect(auctionRows(view, 'late')).toContainEqual(['Pair occupancy', '1 of 2 auctions (50.000%) · final 20%']);
    expect(auctionRows(view, 'late')).toContainEqual(['View', 'Played pairs of bidders 1 and 2 · first-two-bidder projection']);
  });
  it('reports unobserved late pairs without a spurious frequency', () => {
    expect(auctionRows({ ...view, late_count: 0, late_occupancy: [0, 0, 0, 0] }, 'late')).toContainEqual(['Pair occupancy', '0 of 0 auctions (unavailable) · final 20%']);
  });
  it('shows chosen counts separately from all-action updates and hypothetical reward', () => {
    expect(auctionRows({ ...view, x: 12, y: 0, panel: 'values', bidder: 0, action: 1, q: 3, chosen: 7, updated: 10, hypothetical_reward: .45, learners: [{ q: [2, 3], greedy: 1, chosen: [3, 7], updated: [10, 10] }] }, 'values')).toContainEqual(['Action', 'Bidder 1 · bid 0.100 · Q 3.000 · chosen 7 · updated 10 · hypothetical reward 0.450']);
  });
  it('omits unavailable third bidder requests and legends for a two-bidder world', () => {
    const lines = [{ key: 'bid_1', label: 'One', color: '--c1' }, { key: 'bid_3', label: 'Three', color: '--c3' }];
    expect(auctionChartLines(lines, { bidders: 2 } as AuctionsConfig).map((l) => l.key)).toEqual(['bid_1']);
    expect(auctionChartLines(lines, { bidders: 3 } as AuctionsConfig).map((l) => l.key)).toEqual(['bid_1', 'bid_3']);
  });
  it('labels the fringe bid and share beside strategic allocation', () => {
    const rows = auctionRows({ ...view, fringe_bid: .72, fringe_share: 1 }, 'values');
    expect(rows).toContainEqual(['Fringe', 'Bid 0.720 · win share 1.000']);
    expect(rows.find(([name]) => name === 'Allocation')?.[1]).toContain('bidder 2 share 0.500');
    const withFringe = auctionRows({ ...view, greedy: [.1, .1], played: [.05, .1], shares: [0, 0, 1], fringe_bid: .72, fringe_share: 1 }, 'bids');
    expect(withFringe.find(([name]) => name === 'Allocation')?.[1]).not.toContain('bidder 3');
  });
  it('labels batching in captions separately for each comparison world', () => {
    expect(auctionChartCaption('Bids', [{ model: 'auctions', periods_per_tick: 1000 }, { model: 'auctions', periods_per_tick: 7 }] as AuctionsConfig[])).toBe('Bids · A: 1000 periods/tick · B: 7 periods/tick');
  });
});
