import { randomSeed, type Engine } from '../engine';
import { h } from '../ui/dom';
import { GridView } from '../ui/grid-view';
import { showNotice } from '../ui/notice';
import { followChip, replayChip, type Toolbar } from '../ui/toolbar';
import { Lockstep } from './lockstep';

export type WorldName = 'A' | 'B';

/** The playground's parts that Compare extends to a second world. */
export interface Playground {
  engine: Engine;
  grid: GridView;
  toolbar: Toolbar;
  /** Shows the Credit tab while either world on screen has credit on. */
  syncCreditTab: (b: Engine | null) => void;
  /** The run state changed (recording follows it). */
  onRun: () => void;
  /** Both worlds took a step: a frame for the recording. */
  onFrame: () => void;
  onCrash: () => void;
}

/** B's figure: its header, and the copy's progress until its grid can be shown. */
export interface CompareShell { figure: HTMLElement; header: HTMLElement; progress: HTMLElement; canvas: HTMLCanvasElement }

/** Puts B's figure beside A's (the grid area splits at once) with "Copying A…" in it. */
export function compareShell(): CompareShell {
  const header = h('header', { class: 'world-header' });
  const progress = h('p', { class: 'hint copy-progress', role: 'status' }, 'Copying A…');
  const canvas = h('canvas', { class: 'grid-b', 'aria-label': 'Sugarscape grid B', hidden: true });
  const figure = h('figure', { class: 'world', id: 'world-b' }, header, progress, canvas);
  document.querySelector('#grids')!.append(figure);
  document.body.dataset.compare = 'on';
  return { figure, header, progress, canvas };
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
  private readonly offs: (() => void)[] = [];
  /** The headers' 🎲 buttons, disabled while leaving. */
  private readonly dice: HTMLButtonElement[] = [];
  private dirty = true;

  constructor(
    private readonly p: Playground,
    readonly b: Engine,
    private readonly shell: CompareShell,
  ) {
    const a = p.engine;
    shell.progress.remove();
    shell.canvas.hidden = false;
    this.gridB = new GridView(shell.canvas, b);
    this.lock = new Lockstep([a, b], a.speed);
    try {
      this.header(document.querySelector<HTMLElement>('#world-a .world-header')!, 'A', a);
      this.header(shell.header, 'B', b);
      p.toolbar.setCompare(this.lock, b);
      // One display for both worlds: A's, mirrored to B (Decision 10).
      const mirror = () => b.setDisplay({ colorMode: a.colorMode, layer: a.layer, overlays: { ...a.overlays } });
      mirror();
      this.offs.push(
        a.on('display', mirror),
        b.on('snapshot', () => (this.dirty = true)),
        b.on('display', () => (this.dirty = true)),
        b.on('reset', () => p.syncCreditTab(b)),
        b.on('config', () => p.syncCreditTab(b)),
        b.on('crash', p.onCrash),
        b.on('fork', () => showNotice('Replay ended in B — your edit starts a new branch')),
        this.lock.on('run', p.onRun),
        this.lock.on('tick', p.onFrame),
      );
      p.syncCreditTab(b);
      p.onRun();
    } catch (e) {
      // Half-built: undo what is wired so far (the caller closes B and removes its figure).
      this.lock.dispose();
      for (const off of this.offs) off();
      p.toolbar.setCompare(null, null);
      p.syncCreditTab(null);
      throw e;
    }
  }

  /** Called every animation frame: redraws B's grid when it has news. */
  draw(): void {
    if (!this.dirty) return;
    this.gridB.draw();
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
