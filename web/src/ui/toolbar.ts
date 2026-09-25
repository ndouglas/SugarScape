import type { Lockstep } from '../compare/lockstep';
import { randomSeed, type Engine, type RunControls, type Speed } from '../engine';
import { errorMessage } from '../errors';
import { readoutText } from '../valley';
import { h } from './dom';
import { showNotice } from './notice';
import { StopControl } from './stop-control';
import { Timeline } from './timeline';

/** Below 1×, speeds are ticks a second: 1/60 of a tick per frame is one a second. */
const PER_SECOND = [1, 2, 5, 10, 20, 30];
const SPEEDS: Speed[] = [...PER_SECOND.map((n) => n / 60), 1, 2, 5, 10, 25, 100, 'max'];

function speedLabel(s: Speed): string {
  if (s === 'max') return 'Max';
  return s < 1 ? `${Math.round(s * 60)}/s` : `${s}×`;
}

/** A chip and the removal of its listeners. */
export interface Chip { el: HTMLElement; off: () => void }

/**
 * "Following #id ✕" while `engine` draws an agent's trail; † once it has died. The text span and
 * ✕ button are built once and only updated in place: rebuilding them on every sync (which fires
 * on every 'tick') would swap out the ✕ button mid-click, silently dropping the click.
 */
export function followChip(engine: Engine): Chip {
  const text = h('span', { class: 'chip-text' });
  const button = h(
    'button',
    { class: 'link', title: 'Stop following', 'aria-label': 'Stop following', onclick: () => engine.unfollow() },
    '✕',
  );
  const el = h('span', { class: 'chip' }, text, button);
  const sync = () => {
    const id = engine.followed();
    el.hidden = id === null;
    if (id === null) return;
    const alive = engine.followedAlive();
    const label = `Following #${id}${alive ? '' : ' †'}`;
    el.title = label;
    text.textContent = label;
  };
  const offs = (['follow', 'reset', 'tick', 'edit'] as const).map((event) => engine.on(event, sync));
  sync();
  return { el, off: () => offs.forEach((off) => off()) };
}

/**
 * "Replaying · N edits left ✕" while `engine` has edits to replay; ✕ keeps the world and drops the
 * rest. The text span and ✕ button are built once and only updated in place (see `followChip`):
 * during a replayed paint drag, 'replay' fires every frame, and rebuilding the button each time
 * could swap it out between a human's mousedown and mouseup, dropping the click.
 */
export function replayChip(engine: Engine): Chip {
  const text = h('span', { class: 'chip-text' });
  const endReplay = () => {
    engine.endReplay().catch((e) => showNotice(`Could not stop replaying (${errorMessage(e)})`, 10_000));
  };
  const button = h(
    'button',
    { class: 'link', title: 'Stop replaying (keep the world as it is)', 'aria-label': 'Stop replaying', onclick: endReplay },
    '✕',
  );
  const el = h('span', { class: 'chip replay-chip' }, text, button);
  const sync = () => {
    const left = engine.replayLeft;
    el.hidden = left === 0;
    if (left === 0) return;
    const label = `Replaying · ${left} edit${left === 1 ? '' : 's'} left`;
    el.title = label;
    text.textContent = label;
  };
  const offs = (['replay', 'reset'] as const).map((event) => engine.on(event, sync));
  sync();
  return { el, off: () => offs.forEach((off) => off()) };
}

/**
 * Play, Step, speed, seed, Reset and 🎲, the readout and the world's chips. In Compare (Decision 10)
 * Play/Step/speed/Reset drive the lockstep, the seed box, 🎲 and chips move to the grid headers,
 * and the readout shows both populations.
 */
export class Toolbar {
  readonly el: HTMLElement;
  private controls: RunControls;
  private lock: Lockstep | null = null;
  private b: Engine | null = null;
  private offs: (() => void)[] = [];
  private held = false;
  private readonly play: HTMLButtonElement;
  private readonly step: HTMLButtonElement;
  private readonly speed: HTMLSelectElement;
  private readonly seed: HTMLInputElement;
  private readonly resetButton: HTMLButtonElement;
  private readonly dice: HTMLButtonElement;
  private readonly readout = h('span', { class: 'readout' });
  /** The follow and replay chips: their ✕ is held too (ending a replay edits the world). */
  private readonly chips: HTMLElement;
  /** Hidden in Compare: the headers carry them. */
  private readonly singleOnly: HTMLElement[];
  private readonly timeline = new Timeline((e) => showNotice(`Could not go to that tick (${errorMessage(e)})`, 10_000));
  private readonly stopControl: StopControl;

  constructor(private readonly engine: Engine) {
    this.controls = engine;
    this.stopControl = new StopControl(engine);
    this.play = h('button', { class: 'primary', onclick: () => this.controls.setRunning(!this.controls.running) });
    this.step = h(
      'button',
      {
        onclick: () => {
          this.controls.advance(1).catch((e) => showNotice(`Could not step (${errorMessage(e)})`, 10_000));
        },
        title: 'Advance one tick',
      },
      'Step',
    );
    this.speed = h(
      'select',
      {
        title:
          'Ticks a second (n/s) or ticks per frame (n×); Max runs the simulation as fast as it goes and redraws about 30 times a second',
        onchange: () => this.controls.setSpeed(this.speed.value === 'max' ? 'max' : Number(this.speed.value)),
      },
      ...SPEEDS.map((s) => h('option', { value: String(s) }, speedLabel(s))),
    );
    this.seed = h('input', { type: 'number', min: 0, max: 4294967295, class: 'seed', title: 'Seed' });
    const seedLabel = h('label', {}, 'Seed ', this.seed);
    this.resetButton = h(
      'button',
      { title: 'Rebuild this world and replay its edits; with another seed typed, build a new world', onclick: () => this.reset() },
      'Reset',
    );
    this.dice = h('button', { title: 'Random seed and reset', onclick: () => void engine.reset(undefined, randomSeed()) }, '🎲');
    this.chips = h('span', { class: 'chips' }, followChip(engine).el, replayChip(engine).el);
    const chips = this.chips;
    this.singleOnly = [seedLabel, this.dice, chips];
    this.el = h(
      'div',
      { class: 'toolbar' },
      h('h1', {}, 'SugarScape'),
      h('div', { class: 'group' }, this.play, this.step, this.speed),
      this.timeline.el,
      this.stopControl.el,
      h('div', { class: 'group' }, seedLabel, this.resetButton, this.dice),
      this.readout,
      chips,
      h('div', { class: 'toolbar-end' }),
    );
    this.timeline.bind(engine);
    engine.on('run', () => this.sync());
    engine.on('reset', () => {
      this.sync();
      this.tick();
    });
    engine.on('tick', () => this.tick());
    engine.on('edit', () => this.tick());
    engine.on('snapshot', () => this.timeline.sync());
    this.sync();
    this.tick();
  }

  /** Compare on (`lock` and `b`) or off (nulls). */
  setCompare(lock: Lockstep | null, b: Engine | null): void {
    for (const off of this.offs) off();
    this.offs = [];
    this.lock = lock;
    this.b = b;
    this.controls = lock ?? this.engine;
    this.timeline.bind(this.controls);
    this.stopControl.bind(this.controls);
    if (lock) {
      this.offs.push(lock.on('run', () => this.sync()));
      this.offs.push(lock.on('tick', () => this.timeline.sync()));
    }
    if (b) for (const event of ['tick', 'edit', 'reset'] as const) this.offs.push(b.on(event, () => this.tick()));
    for (const el of this.singleOnly) el.hidden = lock !== null;
    this.speed.value = String(this.controls.speed);
    this.sync();
    this.tick();
  }

  /**
   * Disables the run controls, Reset, 🎲 and the chips' ✕ while Compare copies A (A must stay at
   * the tick B is copying, with the same log) and while Compare is left.
   */
  hold(on: boolean): void {
    this.held = on;
    this.chips.inert = on;
    this.sync();
  }

  /** Steps the current controls back one tick (⟲1); for shortcuts (Task 11). */
  back(): void {
    this.timeline.back();
  }

  private reset(): void {
    if (this.lock) {
      // A world that cannot rewind stops the comparison's Reset; say so rather than fail silently.
      this.lock.reset().catch((e) => showNotice(`Could not reset (${errorMessage(e)})`, 10_000));
      return;
    }
    const s = this.typedSeed();
    // The same seed rewinds and replays the session; another seed builds a new world (Decision 4).
    void (s === this.engine.seed ? this.engine.replay() : this.engine.reset(undefined, s));
  }

  /** The seed in the seed box (typed, not yet applied), as an unsigned 32-bit integer. */
  typedSeed(): number {
    return Number(this.seed.value) >>> 0;
  }

  private sync(): void {
    this.play.textContent = this.controls.running ? 'Pause' : 'Play';
    this.play.disabled = this.held;
    this.step.disabled = this.held || this.controls.running;
    this.speed.disabled = this.held;
    this.resetButton.disabled = this.held;
    this.dice.disabled = this.held;
    this.seed.value = String(this.engine.seed);
    this.timeline.held = this.held;
    this.timeline.sync();
    this.stopControl.el.inert = this.held;
  }

  private tick(): void {
    this.readout.textContent = readoutText(this.engine, this.b);
    this.timeline.sync();
  }
}
