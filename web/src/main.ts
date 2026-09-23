import './style.css';
import { downloadBlob, downloadText, canvasBlob } from './downloads';
import { Engine } from './engine';
import { ExperimentsView } from './experiments/view';
import { decodeShare, encodeShare, readHash } from './share';
import { ChartsPanel } from './ui/charts-panel';
import { buildDisplay } from './ui/display';
import { h } from './ui/dom';
import { GridView } from './ui/grid-view';
import { InspectPanel } from './ui/inspect-panel';
import { RulesPanel } from './ui/rules-panel';
import { Tabs } from './ui/tabs';
import { buildToolbar } from './ui/toolbar';
import { buildTools } from './ui/tools';

export function showBanner(message: string, action?: { label: string; run: () => void }): void {
  const banner = document.querySelector<HTMLElement>('#banner')!;
  banner.replaceChildren(
    ...[
      h('span', {}, message),
      action ? h('button', { onclick: action.run }, action.label) : null,
      h('button', { onclick: () => (banner.hidden = true), 'aria-label': 'Dismiss' }, '×'),
    ].filter((child): child is HTMLElement => child !== null),
  );
  banner.hidden = false;
}

async function main(): Promise<void> {
  let engine: Engine;
  const token = readHash();
  try {
    engine = token ? await Engine.create(await decodeShare(token)) : await Engine.create();
  } catch (e) {
    showBanner(`That share link could not be loaded (${e instanceof Error ? e.message : String(e)}). Showing the default rule system.`);
    engine = await Engine.create();
  }
  const grid = new GridView(document.querySelector<HTMLCanvasElement>('#grid')!, engine);
  document.querySelector('#toolbar')!.append(buildToolbar(engine));
  document.querySelector('#display')!.append(buildDisplay(engine));
  const experiments = new ExperimentsView();
  document.querySelector('#experiments')!.append(experiments.el);
  const views = { playground: 'Playground', experiments: 'Experiments' } as const;
  type View = keyof typeof views;
  const viewButtons = (Object.keys(views) as View[]).map((view) =>
    h('button', { 'data-view': view, onclick: () => showView(view) }, views[view]),
  );
  const showView = (view: View): void => {
    // The playground's world is kept, paused, while Experiments is shown.
    if (view === 'experiments') engine.setRunning(false);
    document.body.dataset.view = view;
    document.querySelector<HTMLElement>('#playground')!.hidden = view !== 'playground';
    document.querySelector<HTMLElement>('#experiments')!.hidden = view !== 'experiments';
    for (const b of viewButtons) b.setAttribute('aria-pressed', String(b.dataset.view === view));
  };
  document
    .querySelector('.toolbar h1')!
    .after(h('div', { class: 'view-switch', role: 'group', 'aria-label': 'View' }, ...viewButtons));
  showView('playground');

  const tabs = new Tabs(document.querySelector('#tabs')!, document.querySelector('#panel-body')!);
  tabs.add('Rules', new RulesPanel(engine).el);
  const charts = new ChartsPanel(engine);
  tabs.add('Charts', charts.el, (visible) => charts.setVisible(visible));
  const inspect = new InspectPanel(engine);
  tabs.add('Inspect', inspect.el, (visible) => inspect.setVisible(visible));
  document.querySelector('#tools')!.append(buildTools(engine, grid, () => tabs.show('Inspect')));

  const slug = () => `sugarscape-${engine.presetId ?? 'custom'}-seed${engine.seed}-t${engine.sim.tick()}`;
  const shareButton = h('button', {
    onclick: async () => {
      const token = await encodeShare({ config: engine.baseConfig, seed: engine.seed, landscapes: engine.editedLandscapes() });
      history.replaceState(null, '', `#s=${token}`);
      try {
        await navigator.clipboard.writeText(location.href);
        shareButton.textContent = 'Link copied';
      } catch {
        shareButton.textContent = 'Link in address bar';
      }
      setTimeout(() => (shareButton.textContent = 'Share'), 2000);
    },
    title: 'Copy a link to this setup (config, seed and the painted capacity of every good; hand-placed agents are not included)',
  }, 'Share');
  const menu = h(
    'details',
    { class: 'menu' },
    h('summary', {}, 'Export'),
    h('div', { class: 'menu-items' },
      h('button', { onclick: () => downloadText(`${slug()}-series.csv`, engine.sim.export_series_csv()) }, 'Statistics (CSV)'),
      h('button', { onclick: () => downloadText(`${slug()}-agents.csv`, engine.sim.export_agents_csv()) }, 'Agents (CSV)'),
      h('button', { onclick: async () => downloadBlob(`${slug()}-grid.png`, await grid.toPngBlob()) }, 'Grid (PNG)'),
      h('button', {
        onclick: async () => {
          tabs.show('Charts');
          for (const { name, canvas } of charts.canvases()) {
            downloadBlob(`${slug()}-${name.toLowerCase().replace(/\W+/g, '-')}.png`, await canvasBlob(canvas));
          }
        },
      }, 'Charts (PNG)'),
    ),
  );
  document.querySelector('.toolbar-end')!.append(shareButton, menu);

  let dirty = true;
  for (const event of ['reset', 'tick', 'config', 'display', 'select', 'edit'] as const) {
    engine.on(event, () => (dirty = true));
  }
  const loop = () => {
    try {
      if (engine.running) engine.advance();
      if (dirty) {
        engine.trackSelection();
        grid.draw();
        dirty = false;
      }
      charts.maybeRefresh(performance.now());
    } catch (e) {
      // A Rust panic leaves the WASM instance unusable; reloading keeps any #s= share state.
      engine.running = false;
      console.error(e);
      showBanner('The simulation crashed.', { label: 'Reload', run: () => location.reload() });
      return;
    }
    requestAnimationFrame(loop);
  };
  requestAnimationFrame(loop);
}

main().catch((e) => showBanner(`Failed to start: ${e instanceof Error ? e.message : String(e)}`));
