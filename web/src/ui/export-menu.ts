import { downloadBlob, downloadText } from '../downloads';
import type { Engine } from '../engine';
import { h } from './dom';
import type { GridView } from './grid-view';

/** A world the Export menu exports from; `label` ('A', 'B') is empty outside Compare. */
export interface ExportWorld { label: string; engine: Engine; grid: GridView }

export interface ExportOptions {
  worlds: () => ExportWorld[];
  /** A world's file-name stem. */
  slug: (world: ExportWorld) => string;
  /** Charts (PNG): the charts as drawn. */
  charts: () => Promise<void>;
  /** Session (JSON): the session (or comparison) as a file. */
  session: () => Promise<void>;
}

/**
 * The Export menu. Per-world exports show one button, or in Compare a row with one button per
 * world ("Statistics (CSV) [A] [B]", Decision 12); the menu is rebuilt each time it opens.
 */
export function buildExportMenu(opts: ExportOptions): HTMLElement {
  const items = h('div', { class: 'menu-items' });
  const menu = h('details', { class: 'menu' }, h('summary', {}, 'Export'), items);
  const perWorld = (name: string, run: (world: ExportWorld) => Promise<void>): HTMLElement => {
    const worlds = opts.worlds();
    if (worlds.length === 1) return h('button', { onclick: () => void run(worlds[0]) }, name);
    return h(
      'div',
      { class: 'menu-row' },
      h('span', {}, name),
      ...worlds.map((w) => h('button', { title: `${name} of world ${w.label}`, onclick: () => void run(w) }, w.label)),
    );
  };
  const fill = (): void => {
    items.replaceChildren(
      perWorld('Statistics (CSV)', async (w) => downloadText(`${opts.slug(w)}-series.csv`, await w.engine.seriesCsv())),
      perWorld('Agents (CSV)', async (w) => downloadText(`${opts.slug(w)}-agents.csv`, await w.engine.agentsCsv())),
      perWorld('Grid (PNG)', async (w) => downloadBlob(`${opts.slug(w)}-grid.png`, await w.grid.toPngBlob())),
      h('button', { onclick: () => void opts.charts() }, 'Charts (PNG)'),
      h(
        'button',
        {
          title: 'The whole session — setup, painted maps and every edit — as a file; Share → Open session… loads it',
          onclick: () => void opts.session(),
        },
        'Session (JSON)',
      ),
    );
  };
  menu.addEventListener('toggle', () => {
    if (menu.open) fill();
  });
  fill();
  return menu;
}
