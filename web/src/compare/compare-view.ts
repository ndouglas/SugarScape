import { finishedNotice, randomSeed, type Engine } from '../engine';
import { finishesUnpredictably } from '../models';
import type { ChartsPanel } from '../ui/charts-panel';
import { CreditPanel } from '../ui/credit-panel';
import type { Display } from '../ui/display';
import { h } from '../ui/dom';
import { GridView } from '../ui/grid-view';
import { InspectPanel } from '../ui/inspect-panel';
import { showNotice } from '../ui/notice';
import { RingView } from '../ui/ring-view';
import { RulesPanel } from '../ui/rules-panel';
import type { Tabs } from '../ui/tabs';
import { followChip, replayChip, type Toolbar } from '../ui/toolbar';
import type { Tools } from '../ui/tools';
import type { WorldSlot } from '../ui/world-slot';
import { Lockstep } from './lockstep';

export type WorldName = 'A' | 'B';

/** The playground's parts that Compare extends to a second world. */
export interface Playground {
  engine: Engine;
  grid: GridView;
  toolbar: Toolbar;
  tools: Tools;
  tabs: Tabs;
  charts: ChartsPanel;
  display: Display;
  /** A's ring view (Ring World). */
  ringView: RingView;
  rules: WorldSlot<RulesPanel>;
  inspect: WorldSlot<InspectPanel>;
  credit: WorldSlot<CreditPanel>;
  /** Shows the Credit tab while either world on screen has credit on. */
  syncCreditTab: (b: Engine | null) => void;
  /** The run state changed (recording follows it). */
  onRun: () => void;
  /** Both worlds took a step: a frame for the recording. */
  onFrame: () => void;
  onCrash: () => void;
}

/** B's figure: its header, and the copy's progress until its grid (and, for Ring World, its ring) can be shown. */
export interface CompareShell { figure: HTMLElement; header: HTMLElement; progress: HTMLElement; canvas: HTMLCanvasElement; ring: HTMLCanvasElement }

/** Puts B's figure beside A's (the grid area splits at once) with "Copying A…" in it. */
export function compareShell(): CompareShell {
  const header = h('header', { class: 'world-header' });
  const progress = h('p', { class: 'hint copy-progress', role: 'status' }, 'Copying A…');
  const canvas = h('canvas', { class: 'grid-b', 'aria-label': 'Sugarscape grid B', hidden: true });
  const ring = h('canvas', { class: 'ring-view', 'aria-label': 'Ring World’s ring B', hidden: true });
  const figure = h('figure', { class: 'world', id: 'world-b' }, header, progress, ring, canvas);
  document.querySelector('#grids')!.append(figure);
  document.body.dataset.compare = 'on';
  return { figure, header, progress, canvas, ring };
}

/** Asks which world stays when leaving Compare; null (Cancel, Escape) stays in Compare. */
export function askKeep(): Promise<WorldName | null> {
  return new Promise((resolve) => {
    const dialog = h(
      'dialog',
      { class: 'keep' },
      h(
        'form',
        { method: 'dialog' },
        h('p', {}, 'Leave Compare: which world stays in the playground?'),
        h(
          'div',
          { class: 'row' },
          h('button', { value: 'A' }, 'Keep A'),
          h('button', { value: 'B' }, 'Keep B'),
          h('button', { value: '' }, 'Cancel'),
        ),
      ),
    );
    dialog.addEventListener('close', () => {
      dialog.remove();
      resolve(dialog.returnValue === 'A' || dialog.returnValue === 'B' ? dialog.returnValue : null);
    });
    document.body.append(dialog);
    dialog.showModal();
  });
}

/** Two worlds side by side, stepped in lockstep (Decisions 8–10). */
export class CompareView {
  readonly lock: Lockstep;
  readonly gridB: GridView;
  readonly ringB: RingView;
  private readonly offs: (() => void)[] = [];
  /** The headers' 🎲 buttons, disabled while leaving. */
  private readonly dice: HTMLButtonElement[] = [];
  private dirty = true;
  /** The world last clicked (Inspect, Credit and the disease picker follow it). */
  private focused: WorldName = 'A';

  constructor(
    private readonly p: Playground,
    readonly b: Engine,
    private readonly shell: CompareShell,
  ) {
    const a = p.engine;
    shell.progress.remove();
    shell.canvas.hidden = false;
    this.gridB = new GridView(shell.canvas, b);
    this.ringB = new RingView(shell.ring, b);
    // Both worlds run one model (Decision 11): B shows its ring exactly when A does.
    shell.ring.hidden = b.model !== 'ring';
    this.ringB.onSite = (site) => {
      void b.select(site, b.size().height - 1);
      this.focus('B');
      p.tabs.show('Inspect');
    };
    this.lock = new Lockstep([a, b], a.speed);
    try {
      this.header(document.querySelector<HTMLElement>('#world-a .world-header')!, 'A', a);
      this.header(shell.header, 'B', b);
      p.toolbar.setCompare(this.lock, b);
      p.charts.setCompare(b);
      p.display.setCompare(b);
      // Each world has its own Rules, Inspect and Credit panels; tools act on the grid clicked (Decision 10).
      this.offs.push(p.tools.attach({ engine: b, grid: this.gridB }));
      p.rules.setB(new RulesPanel(b, { sameModelOnly: true }));
      p.inspect.setB(new InspectPanel(b));
      p.credit.setB(
        new CreditPanel(b, () => {
          this.focus('B');
          p.tabs.show('Inspect');
        }),
      );
      this.listen(p.grid.canvas, 'A');
      this.listen(this.gridB.canvas, 'B');
      // One display for both worlds: A's, mirrored to B (Decision 10).
      const mirror = () => b.setDisplay({ colorMode: a.colorMode, layer: a.layer, overlays: { ...a.overlays } });
      mirror();
      this.offs.push(
        a.on('display', mirror),
        b.on('snapshot', () => (this.dirty = true)),
        b.on('display', () => (this.dirty = true)),
        b.on('reset', () => p.syncCreditTab(b)),
        b.on('config', () => p.syncCreditTab(b)),
        ...[a, b].flatMap((e) => [e.on('reset', () => this.syncCredit()), e.on('config', () => this.syncCredit())]),
        b.on('crash', p.onCrash),
        b.on('fork', () => showNotice('Replay ended in B — your edit starts a new branch')),
        // A says when it finishes (main.ts); B speaks only when it finished and A did not. Checked
        // once the pair's step has settled: the two replies land in either order.
        b.on('finished', () => {
          void this.lock.settled().then(() => {
            if (this.lock.finishedWorld() === 1) showNotice(`B: ${finishedNotice(b.config, b.tick)}`, 10_000);
          });
        }),
        // The pair has no ticks left: Play or Step would otherwise no-op silently (the world's own
        // 'finished' fired only when it first finished, not on a later press). Names that world.
        this.lock.on('finished', () =>
          showNotice(
            this.lock.finishedWorld() === 1
              ? `B: ${finishedNotice(b.config, b.tick)}`
              : `${finishesUnpredictably(a.config) ? 'A: ' : ''}${finishedNotice(a.config, a.tick)}`,
            10_000,
          ),
        ),
        // The tick rule paused the pair (Compare supports no condition rule: Decision 9).
        this.lock.on('stopped', () => showNotice(this.lock.lastStop ?? 'Stopped', 10_000)),
        this.lock.on('run', p.onRun),
        this.lock.on('tick', p.onFrame),
      );
      p.syncCreditTab(b);
      this.focus('A');
      p.onRun();
    } catch (e) {
      // Half-built: undo what is wired so far (the caller closes B and removes its figure).
      this.lock.dispose();
      for (const off of this.offs) off();
      p.toolbar.setCompare(null, null);
      p.charts.setCompare(null);
      p.display.setCompare(null);
      this.dropPanels();
      p.syncCreditTab(null);
      throw e;
    }
  }

  /** Called every animation frame: redraws B's grid when it has news. */
  draw(): void {
    if (!this.dirty) return;
    this.gridB.draw();
    this.ringB.draw();
    this.dirty = false;
  }

  /**
   * Leaves Compare: `keep` becomes (or stays) the playground's world (Decision 8). Rejects if B
   * cannot be kept (it crashed); Compare is left all the same, with A in the playground.
   */
  async leave(keep: WorldName): Promise<void> {
    const { p, b } = this;
    const a = p.engine;
    // Nothing may rebuild a world while the pair settles.
    for (const d of this.dice) d.disabled = true;
    this.lock.setRunning(false);
    await this.lock.settled();
    // Before takeWorld: its 'reset' would otherwise make the coordinator rewind the worlds.
    this.lock.dispose();
    for (const off of this.offs) off();
    a.setSpeed(this.lock.speed);
    p.toolbar.setCompare(null, null);
    p.charts.setCompare(null);
    p.display.setCompare(null);
    this.dropPanels();
    this.shell.figure.remove();
    delete document.body.dataset.compare;
    try {
      if (keep === 'B') await a.takeWorld(b);
    } finally {
      // After takeWorld B holds a dead stub, so closing it is harmless either way.
      b.close();
      p.syncCreditTab(null);
      p.onRun();
    }
  }

  /** Inspect and Credit show the world last clicked; the disease picker and image import follow it. */
  focus(world: WorldName): void {
    this.focused = world;
    this.p.inspect.show(world);
    this.syncCredit();
    this.p.tools.focus(world === 'A' ? this.p.engine : this.b);
  }

  /** Credit shows the focused world, or the other one while only it has credit on (the tab is there for it). */
  private syncCredit(): void {
    const on = (w: WorldName) => {
      const e = w === 'A' ? this.p.engine : this.b;
      return e.model === 'sugarscape' && e.sugar.credit.enabled;
    };
    const other: WorldName = this.focused === 'A' ? 'B' : 'A';
    this.p.credit.show(!on(this.focused) && on(other) ? other : this.focused);
  }

  /** A pointerdown on a world's grid focuses that world. */
  private listen(canvas: HTMLCanvasElement, world: WorldName): void {
    const on = () => this.focus(world);
    canvas.addEventListener('pointerdown', on);
    this.offs.push(() => canvas.removeEventListener('pointerdown', on));
  }

  /**
   * Back to A's panels alone. B's panels listen only to B, which is closed after this (or, after
   * Keep B, holds a dead stub), so once dropped here nothing reaches them or it.
   */
  private dropPanels(): void {
    const { p } = this;
    p.rules.setB(null);
    p.inspect.setB(null);
    p.credit.setB(null);
    p.tools.focus(p.engine);
  }

  /** "A · seed n · 🎲" and the world's follow and replay chips. */
  private header(el: HTMLElement, name: WorldName, engine: Engine): void {
    const seed = h('span', { class: 'hint' });
    const syncSeed = () => (seed.textContent = `seed ${engine.seed}`);
    syncSeed();
    const follow = followChip(engine);
    const replay = replayChip(engine);
    const dice = h(
      'button',
      {
        title: `Random seed and rebuild ${name} (the other world rewinds to t = 0)`,
        onclick: () => void engine.reset(undefined, randomSeed()),
      },
      '🎲',
    );
    this.dice.push(dice);
    el.replaceChildren(
      h('strong', {}, name),
      seed,
      dice,
      follow.el,
      replay.el,
    );
    el.hidden = false;
    this.offs.push(engine.on('reset', syncSeed), follow.off, replay.off, () => {
      el.hidden = true;
      el.replaceChildren();
    });
  }
}
