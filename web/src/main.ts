import './style.css';
import { canvasBlob, downloadBlob, downloadText } from './downloads';
import { Engine } from './engine';
import { ExperimentsView } from './experiments/view';
import { LOG_FULL_NOTICE, sessionLink, shareable } from './sessions';
import { decodeShare, decodeSweep, parseSessionFile, readHash, readSweepHash, sessionFileText } from './share';
import { ChartsPanel } from './ui/charts-panel';
import { CreditPanel } from './ui/credit-panel';
import { buildDisplay } from './ui/display';
import { h } from './ui/dom';
import { buildExportMenu } from './ui/export-menu';
import { GridView } from './ui/grid-view';
import { InspectPanel } from './ui/inspect-panel';
import { showNotice } from './ui/notice';
import { RulesPanel } from './ui/rules-panel';
import { buildShareMenu } from './ui/share-menu';
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

const message = (e: unknown): string => (e instanceof Error ? e.message : String(e));

async function main(): Promise<void> {
  let engine: Engine;
  const token = readHash();
  try {
    // A link's session replays its edits as the world runs (Decision 2).
    engine = token ? await Engine.create(await decodeShare(token)) : await Engine.create();
  } catch (e) {
    showBanner(`That share link could not be loaded (${message(e)}). Showing the default rule system.`);
    engine = await Engine.create();
  }
  // Browser checks drive the engine through this handle (7a Decision 14).
  if (new URLSearchParams(location.search).has('debug')) Object.assign(window, { sugarscape: { engine } });
  const grid = new GridView(document.querySelector<HTMLCanvasElement>('#grid')!, engine);
  document.querySelector('#toolbar')!.append(buildToolbar(engine));
  document.querySelector('#display')!.append(buildDisplay(engine));
  const experiments = new ExperimentsView(engine);
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

  const sweepToken = readSweepHash();
  if (sweepToken) {
    try {
      experiments.openSweep(await decodeSweep(sweepToken));
      showView('experiments');
    } catch (e) {
      showBanner(`That experiment link could not be loaded (${message(e)}).`);
    }
  }

  const tabs = new Tabs(document.querySelector('#tabs')!, document.querySelector('#panel-body')!);
  tabs.add('Rules', new RulesPanel(engine).el);
  const charts = new ChartsPanel(engine);
  tabs.add('Charts', charts.el, (visible) => charts.setVisible(visible));
  const inspect = new InspectPanel(engine);
  tabs.add('Inspect', inspect.el, (visible) => inspect.setVisible(visible));
  const credit = new CreditPanel(engine, () => tabs.show('Inspect'));
  tabs.add('Credit', credit.el, (visible) => credit.setVisible(visible));
  // The Credit tab exists only while credit (L) is on.
  const syncCreditTab = () => tabs.setHidden('Credit', !engine.config.credit.enabled);
  engine.on('reset', syncCreditTab);
  engine.on('config', syncCreditTab);
  syncCreditTab();
  document.querySelector('#tools')!.append(buildTools(engine, grid, () => tabs.show('Inspect')));

  const slug = () => `sugarscape-${engine.presetId ?? 'custom'}-seed${engine.seed}-t${engine.tick}`;
  const exportMenu = buildExportMenu({
    worlds: () => [{ label: '', engine, grid }],
    slug: () => slug(),
    charts: async () => {
      tabs.show('Charts');
      await engine.refresh();
      // Let the panel draw the fresh snapshot before the canvases are captured.
      await new Promise((resolve) => requestAnimationFrame(resolve));
      for (const { name, canvas } of charts.canvases()) {
        downloadBlob(`${slug()}-${name.toLowerCase().replace(/\W+/g, '-')}.png`, await canvasBlob(canvas));
      }
    },
    session: async () => {
      const { state, full } = await shareable(engine);
      if (full) showNotice(LOG_FULL_NOTICE, 10_000);
      downloadText(`${slug()}-session.json`, sessionFileText({ kind: 'session', state }), 'application/json');
    },
  });
  const shareMenu = buildShareMenu({
    link: () => sessionLink(engine),
    open: async (file) => {
      try {
        const opened = parseSessionFile(await file.text());
        if (opened.kind !== 'session') throw new Error('it holds a comparison, which this page cannot open yet');
        const errors = await engine.open(opened.state);
        if (errors) throw new Error(errors.map((x) => `${x.field}: ${x.message}`).join('; '));
        // The address bar no longer describes this world.
        history.replaceState(null, '', location.pathname + location.search);
        showNotice(`Opened ${file.name}`);
      } catch (e) {
        showNotice(`${file.name} could not be opened (${message(e)})`, 10_000);
      }
    },
  });
  document.querySelector('.toolbar-end')!.append(shareMenu, exportMenu);

  engine.on('crash', () => showBanner('The simulation crashed.', { label: 'Reload', run: () => location.reload() }));
  engine.on('fork', () => showNotice('Replay ended — your edit starts a new branch'));
  let dirty = true;
  for (const event of ['snapshot', 'display'] as const) engine.on(event, () => (dirty = true));
  const loop = (now: number) => {
    try {
      engine.pump(now);
      if (dirty) {
        grid.draw();
        dirty = false;
      }
    } catch (e) {
      // A Rust panic in the page's WASM leaves it unusable; reloading keeps any #s= share state.
      engine.setRunning(false);
      console.error(e);
      showBanner('The simulation crashed.', { label: 'Reload', run: () => location.reload() });
      return;
    }
    requestAnimationFrame(loop);
  };
  requestAnimationFrame(loop);
}

main().catch((e) => showBanner(`Failed to start: ${message(e)}`));
