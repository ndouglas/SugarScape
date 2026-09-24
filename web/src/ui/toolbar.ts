import { randomSeed, type Engine, type Speed } from '../engine';
import { errorMessage } from '../errors';
import { h } from './dom';
import { showNotice } from './notice';

const SPEEDS: Speed[] = [1, 2, 5, 10, 25, 100, 'max'];

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

export function buildToolbar(engine: Engine): HTMLElement {
  const play = h('button', { class: 'primary', onclick: () => engine.setRunning(!engine.running) });
  const step = h('button', { onclick: () => void engine.advance(1), title: 'Advance one tick' }, 'Step');
  const speed = h(
    'select',
    {
      title: 'Ticks per frame; Max runs the simulation as fast as it goes and redraws about 30 times a second',
      onchange: () => engine.setSpeed(speed.value === 'max' ? 'max' : Number(speed.value)),
    },
    ...SPEEDS.map((s) => h('option', { value: String(s) }, s === 'max' ? 'Max' : `${s}×`)),
  );
  const seed = h('input', { type: 'number', min: 0, max: 4294967295, class: 'seed', title: 'Seed' });
  const reset = h(
    'button',
    {
      title: 'Rebuild this world and replay its edits; with another seed typed, build a new world',
      onclick: () => {
        const s = Number(seed.value) >>> 0;
        // The same seed rewinds and replays the session; another seed builds a new world (Decision 4).
        void (s === engine.seed ? engine.replay() : engine.reset(undefined, s));
      },
    },
    'Reset',
  );
  const dice = h(
    'button',
    { title: 'Random seed and reset', onclick: () => void engine.reset(undefined, randomSeed()) },
    '🎲',
  );
  const readout = h('span', { class: 'readout' });

  const sync = () => {
    play.textContent = engine.running ? 'Pause' : 'Play';
    step.disabled = engine.running;
    seed.value = String(engine.seed);
  };
  const tick = () => {
    readout.textContent = `t = ${engine.tick} · ${engine.population} agents`;
  };
  engine.on('run', sync);
  engine.on('reset', () => {
    sync();
    tick();
  });
  engine.on('tick', tick);
  engine.on('edit', tick);
  sync();
  tick();

  return h(
    'div',
    { class: 'toolbar' },
    h('h1', {}, 'SugarScape'),
    h('div', { class: 'group' }, play, step, speed),
    h('div', { class: 'group' }, h('label', {}, 'Seed ', seed), reset, dice),
    readout,
    followChip(engine).el,
    replayChip(engine).el,
    h('div', { class: 'toolbar-end' }),
  );
}
