import { randomSeed, type Engine } from '../engine';
import { h } from './dom';

const SPEEDS = [1, 2, 5, 10, 25, 100];

export function buildToolbar(engine: Engine): HTMLElement {
  const play = h('button', { class: 'primary', onclick: () => engine.setRunning(!engine.running) });
  const step = h('button', { onclick: () => engine.advance(1), title: 'Advance one tick' }, 'Step');
  const speed = h(
    'select',
    { title: 'Ticks per frame', onchange: () => (engine.stepsPerFrame = Number(speed.value)) },
    ...SPEEDS.map((s) => h('option', { value: String(s) }, `${s}×`)),
  );
  const seed = h('input', { type: 'number', min: 0, max: 4294967295, class: 'seed', title: 'Seed' });
  const reset = h('button', { onclick: () => engine.reset(engine.baseConfig, Number(seed.value) >>> 0) }, 'Reset');
  const dice = h(
    'button',
    { title: 'Random seed and reset', onclick: () => engine.reset(engine.baseConfig, randomSeed()) },
    '🎲',
  );
  const readout = h('span', { class: 'readout' });

  /** "Following #id ✕" while an agent's trail is drawn; † once it has died. */
  const chip = h('span', { class: 'chip' });
  const syncFollow = () => {
    const id = engine.followed();
    chip.hidden = id === null;
    if (id === null) return;
    const alive = engine.sim.locate(id) !== undefined;
    const text = `Following #${id}${alive ? '' : ' †'}`;
    chip.title = text;
    chip.replaceChildren(
      h('span', { class: 'chip-text' }, text),
      h('button', { class: 'link', title: 'Stop following', 'aria-label': 'Stop following', onclick: () => engine.unfollow() }, '✕'),
    );
  };
  for (const event of ['follow', 'reset', 'tick', 'edit'] as const) engine.on(event, syncFollow);
  syncFollow();

  const sync = () => {
    play.textContent = engine.running ? 'Pause' : 'Play';
    step.disabled = engine.running;
    seed.value = String(engine.seed);
  };
  const tick = () => {
    readout.textContent = `t = ${engine.sim.tick()} · ${engine.sim.population()} agents`;
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
    chip,
    h('div', { class: 'toolbar-end' }),
  );
}
