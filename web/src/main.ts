import './style.css';
import { askKeep, CompareView, compareShell, type Playground, type WorldName } from './compare/compare-view';
import { copyWorld } from './compare/lockstep';
import { canvasBlob, downloadBlob, downloadText } from './downloads';
import { Engine } from './engine';
import { errorMessage } from './errors';
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
import { buildRecordControl } from './ui/record-control';
import { RulesPanel } from './ui/rules-panel';
import { buildShareMenu } from './ui/share-menu';
import { Tabs } from './ui/tabs';
import { Toolbar } from './ui/toolbar';
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
    // A link's session replays its edits as the world runs (Decision 2).
    engine = token ? await Engine.create(await decodeShare(token)) : await Engine.create();
  } catch (e) {
    showBanner(`That share link could not be loaded (${errorMessage(e)}). Showing the default rule system.`);
    engine = await Engine.create();
  }
  /** Compare mode's second world and its coordinator, while Compare is on. */
  let compare: CompareView | null = null;
  // Browser checks drive the engines through this handle (7a Decision 14).
  if (new URLSearchParams(location.search).has('debug')) Object.assign(window, { sugarscape: { engine, compare: () => compare } });
  const grid = new GridView(document.querySelector<HTMLCanvasElement>('#grid')!, engine);
  const toolbar = new Toolbar(engine);
  document.querySelector('#toolbar')!.append(toolbar.el);
  document.querySelector('#display')!.append(buildDisplay(engine));
  const experiments = new ExperimentsView(engine);
  document.querySelector('#experiments')!.append(experiments.el);
  const views = { playground: 'Playground', experiments: 'Experiments' } as const;
  type View = keyof typeof views;
  const viewButtons = (Object.keys(views) as View[]).map((view) =>
    h('button', { 'data-view': view, onclick: () => showView(view) }, views[view]),
  );
  const showView = (view: View): void => {
    // The playground's worlds are kept, paused, while Experiments is shown.
    if (view === 'experiments') (compare?.lock ?? engine).setRunning(false);
    document.body.dataset.view = view;
    document.querySelector<HTMLElement>('#playground')!.hidden = view !== 'playground';
    document.querySelector<HTMLElement>('#experiments')!.hidden = view !== 'experiments';
    for (const b of viewButtons) b.setAttribute('aria-pressed', String(b.dataset.view === view));
  };
  const compareButton = h(
    'button',
    {
      class: 'compare-toggle',
      'aria-pressed': 'false',
      title: 'Run a copy of this world beside it, both stepped in lockstep',
      onclick: () => void toggleCompare(),
    },
    'Compare',
  );
  document
    .querySelector('.toolbar h1')!
    .after(h('div', { class: 'view-switch', role: 'group', 'aria-label': 'View' }, ...viewButtons), compareButton);
  showView('playground');

  const sweepToken = readSweepHash();
  if (sweepToken) {
    try {
      experiments.openSweep(await decodeSweep(sweepToken));
      showView('experiments');
    } catch (e) {
      showBanner(`That experiment link could not be loaded (${errorMessage(e)}).`);
    }
  }

  const tabs = new Tabs(document.querySelector('#tabs')!, document.querySelector('#panel-body')!);
  const rules = new RulesPanel(engine);
  tabs.add('Rules', rules.el);
  const charts = new ChartsPanel(engine);
  tabs.add('Charts', charts.el, (visible) => charts.setVisible(visible));
  const inspect = new InspectPanel(engine);
  tabs.add('Inspect', inspect.el, (visible) => inspect.setVisible(visible));
  const credit = new CreditPanel(engine, () => tabs.show('Inspect'));
  tabs.add('Credit', credit.el, (visible) => credit.setVisible(visible));
  // The Credit tab exists only while credit (L) is on in a world on screen.
  const syncCreditTab = (b: Engine | null) => tabs.setHidden('Credit', ![engine, b].some((e) => e?.config.credit.enabled));
  engine.on('reset', () => syncCreditTab(compare?.b ?? null));
  engine.on('config', () => syncCreditTab(compare?.b ?? null));
  syncCreditTab(null);
  const tools = buildTools(engine, grid, () => tabs.show('Inspect'));
  document.querySelector('#tools')!.append(tools);

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
        if (busy) throw new Error('Compare is starting or ending; try again in a moment');
        if (compare) await leaveCompare('A');
        const errors = await engine.open(opened.state);
        if (errors) throw new Error(errors.map((x) => `${x.field}: ${x.message}`).join('; '));
        // The address bar no longer describes this world.
        history.replaceState(null, '', location.pathname + location.search);
        showNotice(`Opened ${file.name}`);
      } catch (e) {
        showNotice(`${file.name} could not be opened (${errorMessage(e)})`, 10_000);
      }
    },
  });
  const record = buildRecordControl({
    grids: () => [{ canvas: grid.canvas, cells: () => engine.size() }],
    tick: () => engine.tick,
    running: () => (compare?.lock ?? engine).running,
    base: () => `sugarscape-${engine.presetId ?? 'custom'}-seed${engine.seed}`,
  });
  engine.on('run', () => record.sync());
  document.querySelector('.toolbar-end')!.append(record.el, shareMenu, exportMenu);

  const crashed = () => showBanner('The simulation crashed.', { label: 'Reload', run: () => location.reload() });
  engine.on('crash', crashed);
  // In Compare the notice names the world (B's names B: see CompareView).
  engine.on('fork', () => showNotice(`Replay ended${compare ? ' in A' : ''} — your edit starts a new branch`));

  let dirty = true;
  /** A snapshot arrived since the last frame (Compare: a lockstep pair): once drawn, the recording captures it. */
  let fresh = false;
  for (const event of ['snapshot', 'display'] as const) engine.on(event, () => (dirty = true));
  engine.on('snapshot', () => {
    if (!compare) fresh = true;
  });
  const playground: Playground = {
    engine,
    grid,
    toolbar,
    syncCreditTab,
    onRun: () => record.sync(),
    onFrame: () => (fresh = true),
    onCrash: crashed,
  };

  /** Compare is starting or ending: the toggle waits. */
  let busy = false;
  const syncCompareButton = () => {
    compareButton.setAttribute('aria-pressed', String(compare !== null));
    compareButton.disabled = busy;
  };
  /**
   * Locks A while B copies it: nothing on the page may run, step, rebuild or edit A (the toolbar
   * is held; the grid, its tools and the Rules panel are inert), or B would stop being a copy.
   * Opening a session file is refused while `busy`.
   */
  const holdA = (on: boolean): void => {
    toolbar.hold(on);
    for (const el of [grid.canvas, tools, rules.el]) el.inert = on;
  };
  /** Starts Compare with B a copy of A at its current tick (Decision 9). */
  async function enterCompare(): Promise<void> {
    if (compare || busy) return;
    busy = true;
    syncCompareButton();
    engine.setRunning(false);
    holdA(true);
    const shell = compareShell();
    let b: Engine | null = null;
    try {
      const { session, full, tick } = await engine.session();
      if (full) throw new Error('A’s edit log is full (50 000 edits), so B cannot copy it exactly');
      b = await copyWorld(session, tick, (s) => Engine.create(s), (at, of) => {
        shell.progress.textContent = `Copying A… ${at} / ${of}`;
      });
      // Belt and braces: A must still be where B copied it from, with the same edits so far.
      const [now, copy] = await Promise.all([engine.session(), b.session()]);
      const upTo = (s: typeof now) => s.session.log.filter((e) => e.tick <= s.tick).length;
      if (now.tick !== copy.tick || upTo(now) !== upTo(copy)) {
        throw new Error(`A changed while it was copied (A at t = ${now.tick}, B at t = ${copy.tick})`);
      }
      compare = new CompareView(playground, b, shell);
      // The coordinator first brings both worlds to rest at one tick; only then may they run.
      await compare.lock.settled();
    } catch (e) {
      b?.close();
      shell.figure.remove();
      delete document.body.dataset.compare;
      showNotice(`Compare could not start (${errorMessage(e)})`, 10_000);
    } finally {
      busy = false;
      holdA(false);
      syncCompareButton();
    }
  }
  async function leaveCompare(keep: WorldName): Promise<void> {
    const c = compare;
    if (!c || busy) return;
    compare = null;
    busy = true;
    syncCompareButton();
    // Nothing may step or rebuild the pair while it settles and is taken apart.
    toolbar.hold(true);
    try {
      await c.leave(keep);
    } catch (e) {
      showNotice(`B could not be kept (${errorMessage(e)}); A stays in the playground`, 10_000);
    } finally {
      toolbar.hold(false);
      busy = false;
      syncCompareButton();
    }
  }
  async function toggleCompare(): Promise<void> {
    if (busy) return;
    if (!compare) return enterCompare();
    const keep = await askKeep();
    if (keep) await leaveCompare(keep);
  }

  const loop = (now: number) => {
    try {
      (compare?.lock ?? engine).pump(now);
      if (dirty) {
        grid.draw();
        dirty = false;
      }
      compare?.draw();
      if (fresh) {
        fresh = false;
        record.capture();
      }
    } catch (e) {
      // A Rust panic in the page's WASM leaves it unusable; reloading keeps any #s= share state.
      (compare?.lock ?? engine).setRunning(false);
      console.error(e);
      crashed();
      return;
    }
    requestAnimationFrame(loop);
  };
  requestAnimationFrame(loop);
}

main().catch((e) => showBanner(`Failed to start: ${errorMessage(e)}`));
