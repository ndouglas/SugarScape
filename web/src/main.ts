import './style.css';
import { askKeep, CompareView, compareShell, type Playground, type WorldName } from './compare/compare-view';
import { copyWorld } from './compare/lockstep';
import { COMPARE_PRESETS, comparePresetStates } from './compare-presets';
import { canvasBlob, downloadBlob, downloadText } from './downloads';
import { Engine, type InitialState } from './engine';
import { errorMessage, fieldErrorsMessage } from './errors';
import { ExperimentsView } from './experiments/view';
import { compareLink, LOG_FULL_NOTICE, sessionLink, shareable } from './sessions';
import {
  decodeCompare,
  decodeShare,
  decodeSweep,
  parseSessionFile,
  readCompareHash,
  readHash,
  readSweepHash,
  sessionFileText,
  type SessionFile,
} from './share';
import { ChartsPanel } from './ui/charts-panel';
import { CreditPanel } from './ui/credit-panel';
import { buildDisplay } from './ui/display';
import { h } from './ui/dom';
import { buildExportMenu, type ExportWorld } from './ui/export-menu';
import { GridView } from './ui/grid-view';
import { InspectPanel } from './ui/inspect-panel';
import { showNotice } from './ui/notice';
import { buildRecordControl } from './ui/record-control';
import { RulesPanel } from './ui/rules-panel';
import { buildShareMenu } from './ui/share-menu';
import { Tabs } from './ui/tabs';
import { Toolbar } from './ui/toolbar';
import { buildTools } from './ui/tools';
import { WorldSlot } from './ui/world-slot';

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
  /** A `#c=` link's B: Compare starts with it once the page is built. */
  let startB: InitialState | null = null;
  const token = readHash();
  const compareToken = readCompareHash();
  try {
    if (compareToken) {
      const { a, b } = await decodeCompare(compareToken);
      engine = await Engine.create(a);
      startB = b;
    } else {
      // A link's session replays its edits as the world runs (Decision 2).
      engine = token ? await Engine.create(await decodeShare(token)) : await Engine.create();
    }
  } catch (e) {
    showBanner(`That share link could not be loaded (${errorMessage(e)}). Showing the default rule system.`);
    engine = await Engine.create();
  }
  /** Compare mode's second world and its coordinator, while Compare is on. */
  let compare: CompareView | null = null;
  // Declared here (not next to `hold`/`enterCompare` below) so the Compare button, wired to
  // `toggleCompare` well before this function finishes its startup `await`s (e.g. a `#x=` link's
  // `decodeSweep`), never reads `busy` in its temporal dead zone.
  /** Compare is starting or ending: the toggle waits. */
  let busy = false;
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
  // Each tab holds A's panel, and B's beside it in Compare (Decision 10).
  const rules = new WorldSlot(
    new RulesPanel(engine, (id) => void openComparePreset(id)),
    'switch',
    'Rules for',
  );
  tabs.add('Rules', rules.el);
  const charts = new ChartsPanel(engine);
  tabs.add('Charts', charts.el, (visible) => charts.setVisible(visible));
  const inspect = new WorldSlot(new InspectPanel(engine), 'label');
  tabs.add('Inspect', inspect.el, (visible) => inspect.setVisible(visible));
  const credit = new WorldSlot(
    new CreditPanel(engine, () => {
      compare?.focus('A');
      tabs.show('Inspect');
    }),
    'label',
  );
  tabs.add('Credit', credit.el, (visible) => credit.setVisible(visible));
  // The Credit tab exists only while credit (L) is on in a world on screen.
  const syncCreditTab = (b: Engine | null) => tabs.setHidden('Credit', ![engine, b].some((e) => e?.config.credit.enabled));
  engine.on('reset', () => syncCreditTab(compare?.b ?? null));
  engine.on('config', () => syncCreditTab(compare?.b ?? null));
  syncCreditTab(null);
  const tools = buildTools({ engine, grid }, (world) => {
    compare?.focus(world === engine ? 'A' : 'B');
    tabs.show('Inspect');
  });
  document.querySelector('#tools')!.append(tools.el);

  const slug = (e: Engine) => `sugarscape-${e.presetId ?? 'custom'}-seed${e.seed}-t${e.tick}`;
  /** File stems for what covers both worlds in Compare (charts, the session). */
  const stem = () => (compare ? `sugarscape-compare-t${engine.tick}` : slug(engine));
  const worlds = (): ExportWorld[] => {
    const c = compare;
    return c
      ? [
          { label: 'A', engine, grid },
          { label: 'B', engine: c.b, grid: c.gridB },
        ]
      : [{ label: '', engine, grid }];
  };
  const exportMenu = buildExportMenu({
    worlds,
    slug: (w) => (w.label ? `${slug(w.engine)}-${w.label}` : slug(w.engine)),
    charts: async () => {
      const name = stem();
      tabs.show('Charts');
      // Both worlds' lines are drawn from fresh snapshots (Compare overlays B's on A's).
      await Promise.all([engine.refresh(), compare?.b.refresh()]);
      // Let the panel draw the fresh snapshots before the canvases are captured.
      await new Promise((resolve) => requestAnimationFrame(resolve));
      for (const chart of charts.canvases()) {
        downloadBlob(`${name}-${chart.name.toLowerCase().replace(/\W+/g, '-')}.png`, await canvasBlob(chart.canvas));
      }
    },
    session: async () => {
      const c = compare;
      const name = stem();
      const a = await shareable(engine);
      let file: SessionFile = { kind: 'session', state: a.state };
      let full = a.full;
      if (c) {
        const b = await shareable(c.b);
        file = { kind: 'compare', state: { a: a.state, b: b.state } };
        full ||= b.full;
      }
      if (full) showNotice(LOG_FULL_NOTICE, 10_000);
      downloadText(`${name}-session.json`, sessionFileText(file), 'application/json');
    },
  });
  const shareMenu = buildShareMenu({
    link: () => (compare ? compareLink(engine, compare.b) : sessionLink(engine)),
    open: async (file) => {
      try {
        const opened = parseSessionFile(await file.text());
        if (busy) throw new Error('Compare is starting or ending; try again in a moment');
        // A single session opens in the playground: Compare ends keeping A (Decision 12).
        if (compare) await leaveCompare('A');
        if (busy || compare) throw new Error('Compare could not be left; try again in a moment');
        await openFile(opened);
        showNotice(`Opened ${file.name}`);
      } catch (e) {
        showNotice(`${file.name} could not be opened (${errorMessage(e)})`, 10_000);
      }
    },
  });
  const record = buildRecordControl({
    // In Compare each frame shows both grids side by side, tagged "A" and "B".
    grids: () => {
      const c = compare;
      return c
        ? [
            { canvas: grid.canvas, cells: () => engine.size(), label: 'A' },
            { canvas: c.gridB.canvas, cells: () => c.b.size(), label: 'B' },
          ]
        : [{ canvas: grid.canvas, cells: () => engine.size() }];
    },
    tick: () => engine.tick,
    running: () => (compare?.lock ?? engine).running,
    base: () => {
      const c = compare;
      return c ? `sugarscape-compare-seed${engine.seed}-vs-seed${c.b.seed}` : `sugarscape-${engine.presetId ?? 'custom'}-seed${engine.seed}`;
    },
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
    tools,
    tabs,
    charts,
    rules,
    inspect,
    credit,
    syncCreditTab,
    onRun: () => record.sync(),
    onFrame: () => (fresh = true),
    onCrash: crashed,
  };

  const syncCompareButton = () => {
    compareButton.setAttribute('aria-pressed', String(compare !== null));
    compareButton.disabled = busy;
  };
  /**
   * Locks the worlds while B copies A, and while Compare is left: nothing on the page may run,
   * step, rebuild or edit a world (the toolbar is held; the grids, the tools and the Rules tab —
   * both worlds' panels — are inert), or B would stop being a copy / the pair would move while it
   * settles. Opening a session file is refused while `busy`.
   */
  const hold = (on: boolean): void => {
    toolbar.hold(on);
    for (const el of [grid.canvas, compare?.gridB.canvas, tools.el, rules.el]) if (el) el.inert = on;
  };
  /** Starts Compare with B built from `bState` (a link or file), or a copy of A at its current tick (Decision 9). */
  async function enterCompare(bState?: InitialState): Promise<void> {
    if (compare || busy) return;
    busy = true;
    syncCompareButton();
    hold(true);
    try {
      await buildCompare(bState);
    } catch (e) {
      showNotice(`Compare could not start (${errorMessage(e)})`, 10_000);
    } finally {
      busy = false;
      hold(false);
      syncCompareButton();
    }
  }
  /** Enter's work, run while `busy` and held; on failure it is undone and the error rethrown. */
  async function buildCompare(bState?: InitialState): Promise<void> {
    engine.setRunning(false);
    const shell = compareShell();
    let b: Engine | null = null;
    try {
      if (bState) {
        // Both worlds start at t = 0 and replay their logs; the coordinator aligns them if A has moved.
        b = await Engine.create(bState);
      } else {
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
      }
      compare = new CompareView(playground, b, shell);
      // A stale [B] row (from before Compare started) must not survive under an open menu.
      exportMenu.open = false;
      // B's grid exists now: it stays inert with A's until the pair has settled.
      hold(true);
      // The coordinator first brings both worlds to rest at one tick; only then may they run.
      await compare.lock.settled();
    } catch (e) {
      b?.close();
      shell.figure.remove();
      delete document.body.dataset.compare;
      throw e;
    }
  }
  /**
   * Opens a session file outside Compare: A takes its (first) session; a comparison then enters
   * Compare with its B. All of it runs `busy` and held, so no enter or leave can race it.
   */
  async function openFile(opened: SessionFile): Promise<void> {
    busy = true;
    syncCompareButton();
    hold(true);
    try {
      engine.setRunning(false);
      const errors = await engine.open(opened.kind === 'session' ? opened.state : opened.state.a);
      if (errors) throw new Error(fieldErrorsMessage(errors));
      // The address bar no longer describes this world.
      history.replaceState(null, '', location.pathname + location.search);
      if (opened.kind === 'compare') {
        await buildCompare(opened.state.b).catch((e: unknown) => {
          throw new Error(`Compare could not start: ${errorMessage(e)}; world A is open alone`);
        });
      }
    } finally {
      busy = false;
      hold(false);
      syncCompareButton();
    }
  }
  async function leaveCompare(keep: WorldName): Promise<void> {
    const c = compare;
    if (!c || busy) return;
    compare = null;
    // The [B] row must not survive under an open menu once B is gone.
    exportMenu.open = false;
    busy = true;
    syncCompareButton();
    // Nothing may step, rebuild or edit the pair while it settles and is taken apart.
    hold(true);
    // (`compare` is already null, so `hold` misses B's grid; its figure goes with Compare.)
    c.gridB.canvas.inert = true;
    try {
      await c.leave(keep);
    } catch (e) {
      showNotice(`B could not be kept (${errorMessage(e)}); A stays in the playground`, 10_000);
    } finally {
      hold(false);
      busy = false;
      syncCompareButton();
    }
  }
  /**
   * A Compare entry of the presets menu: A rebuilds as the entry's first preset and B as its second,
   * both with the seed box's seed, and Compare starts at t = 0 in lockstep (as a `#c=` link does).
   * If Compare is on it is left first, keeping A.
   */
  async function openComparePreset(id: string): Promise<void> {
    const entry = COMPARE_PRESETS.find((c) => c.id === id);
    const states = entry && comparePresetStates(engine.presets, entry, toolbar.typedSeed());
    if (!entry || !states) {
      showNotice(`The comparison ${id} is not available`, 10_000);
      return;
    }
    if (busy) {
      showNotice('Compare is starting or ending; try again in a moment');
      return;
    }
    if (compare) await leaveCompare('A');
    if (busy || compare) {
      showNotice('Compare could not be left; try again in a moment');
      return;
    }
    busy = true;
    syncCompareButton();
    hold(true);
    try {
      engine.setRunning(false);
      const errors = await engine.loadPreset(entry.a, states.a.seed);
      if (errors) throw new Error(fieldErrorsMessage(errors));
      // The address bar no longer describes this world.
      history.replaceState(null, '', location.pathname + location.search);
      await buildCompare(states.b);
    } catch (e) {
      showNotice(`Compare could not start (${errorMessage(e)})`, 10_000);
    } finally {
      busy = false;
      hold(false);
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
  // A #c= link opens straight into Compare, both worlds at t = 0 replaying their logs.
  if (startB) await enterCompare(startB);
}

main().catch((e) => showBanner(`Failed to start: ${errorMessage(e)}`));
