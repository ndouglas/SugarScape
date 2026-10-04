import { afterEach, describe, expect, it, vi } from 'vitest';
import { auctionLegend } from './auctions';
import { GridView } from './ui/grid-view';
import { Engine } from './engine';
import { SimHost } from './sim-host';
import { InlineTransport } from './transport';
import { wasmSimModule } from './sim-module';
import type { AuctionsConfig, Preset } from './types';
import { initSync, presets_json } from './wasm-pkg/sugarscape.js';
import { readFileSync } from 'node:fs';

const config = { bids: 19, out_bids: 7, bidders: 3, horizon: 15 } as AuctionsConfig;
const context = { setTransform() {}, drawImage() {}, putImageData() {} };
class Element {
  children: (Element | string)[] = []; style = {}; hidden = false; clientWidth = 240; width = 0; height = 0;
  get textContent(): string { return this.children.map((c) => typeof c === 'string' ? c : c.textContent).join(' '); }
  append(...children: (Element | string)[]): void { this.children.push(...children); }
  prepend(...children: (Element | string)[]): void { this.children.unshift(...children); }
  replaceChildren(...children: (Element | string)[]): void { this.children = children; }
  setAttribute() {} addEventListener() {} after() {} getContext() { return context; }
}
afterEach(() => vi.unstubAllGlobals());
describe('auction map legend', () => {
  it('labels axes, out-action range, third-bidder projection and actual late denominator', () => {
    const legend = auctionLegend(config, 'late', 14);
    expect(legend.status).toContain('2 auctions counted · final 20%');
    expect(legend.status).toContain('Bidder 1: left -0.300 → right 0.950');
    expect(legend.status).toContain('Bidder 2: bottom -0.300 → top 0.950');
    expect(legend.status).toContain('first-two-bidder projection');
    expect(auctionLegend(config, 'late', 11).status).toContain('0 auctions counted');
    expect(auctionLegend(config, 'bids', 14).status).toContain('14 auctions counted · whole run');
  });
  it('labels value rows and distinct played and greedy edges', () => {
    const legend = auctionLegend(config, 'values', 14);
    expect(legend.status).toContain('Rows per bidder: Q, chosen, updated');
    expect(legend.items.map((i) => i.label)).toContain('Greedy action · gold top/left edges');
    expect(legend.items.map((i) => i.label)).toContain('Played action · cyan bottom/right edges');
  });
  it('shows the selected count and orientation in real GridView without inspecting a cell', async () => {
    const wasm = initSync({ module: readFileSync(new URL('./wasm-pkg/sugarscape_bg.wasm', import.meta.url)) });
    const presets = JSON.parse(presets_json()) as Preset[];
    const preset = presets.find((p) => p.id === 'auctions-first-price')!;
    const e = await Engine.create({ config: { ...preset.config as AuctionsConfig, horizon: 15, window: 3, periods_per_tick: 7 }, seed: 1 }, { presets, transport: new InlineTransport(new SimHost(wasmSimModule(wasm.memory))) });
    await e.advance(2);
    vi.stubGlobal('document', { createElement: () => new Element(), createElementNS: () => new Element() });
    vi.stubGlobal('getComputedStyle', () => ({ getPropertyValue: () => '' }));
    vi.stubGlobal('ImageData', class { constructor(..._args: unknown[]) {} });
    vi.stubGlobal('devicePixelRatio', 1);
    const grid = new GridView(new Element() as unknown as HTMLCanvasElement, e);
    grid.draw();
    expect(grid.legend.hidden).toBe(false);
    expect(grid.legend.textContent).toContain('14 auctions counted · whole run');
    expect(grid.legend.textContent).toContain('Bidder 2: bottom 0.050 → top 0.950');
    e.setDisplay({ colorMode: 'late' });
    grid.draw();
    expect(grid.legend.textContent).toContain('2 auctions counted · final 20%');
  });
});
