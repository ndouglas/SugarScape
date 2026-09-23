import type { Engine, PlaceOverrides } from '../engine';
import { h } from './dom';
import type { GridView } from './grid-view';

type Tool = 'inspect' | 'paint' | 'place' | 'erase';

const TOOLS: [Tool, string][] = [
  ['inspect', 'Inspect'],
  ['paint', 'Paint capacity'],
  ['place', 'Place agent'],
  ['erase', 'Erase agent'],
];

/** Tool picker; routes grid clicks/drags to the active tool. Edit errors (e.g. occupied site) are ignored. */
export function buildTools(engine: Engine, grid: GridView, onInspect: () => void): HTMLElement {
  let tool: Tool = 'inspect';
  let radius = 1;
  let value = 4;
  let sex: '' | 'female' | 'male' = '';
  let tribe: '' | 'blue' | 'red' = '';

  const buttons = TOOLS.map(([t, label]) => h('button', { onclick: () => choose(t) }, label));
  const options = h('div', { class: 'tool-options' });

  const number = (label: string, min: number, max: number, get: () => number, set: (v: number) => void) => {
    const input = h('input', { type: 'number', min, max, value: get(), class: 'num' });
    input.addEventListener('change', () => {
      set(Math.min(max, Math.max(min, Number(input.value))));
      input.value = String(get());
      if (tool === 'paint') grid.brushRadius = radius;
    });
    return h('label', {}, `${label} `, input);
  };
  const select = <T extends string>(label: string, values: [T, string][], set: (v: T) => void) => {
    const s = h('select', {}, ...values.map(([v, l]) => h('option', { value: v }, l)));
    s.addEventListener('change', () => set(s.value as T));
    return h('label', {}, `${label} `, s);
  };

  function choose(next: Tool): void {
    tool = next;
    buttons.forEach((b, i) => b.setAttribute('aria-pressed', String(TOOLS[i][0] === tool)));
    grid.brushRadius = tool === 'paint' ? radius : null;
    if (tool === 'paint') engine.setDisplay({ layer: 'capacity' });
    options.replaceChildren(
      ...(tool === 'paint'
        ? [number('Radius', 0, 10, () => radius, (v) => (radius = v)), number('Capacity', 0, 4, () => value, (v) => (value = v))]
        : tool === 'place'
          ? [
              select('Sex', [['', 'Random'], ['female', 'Female'], ['male', 'Male']], (v) => (sex = v)),
              select('Tribe', [['', 'Random'], ['blue', 'Blue'], ['red', 'Red']], (v) => (tribe = v)),
            ]
          : tool === 'inspect'
            ? [h('span', { class: 'hint' }, 'Click an agent or site.')]
            : [h('span', { class: 'hint' }, 'Click or drag over agents to remove them.')]),
    );
    grid.draw();
  }

  grid.onCell = (x, y, kind) => {
    switch (tool) {
      case 'inspect':
        if (kind === 'down') {
          engine.select(x, y);
          onInspect();
        }
        break;
      case 'paint':
        engine.paint(x, y, radius, value);
        break;
      case 'place':
        if (kind === 'down') {
          const o: PlaceOverrides = {};
          if (sex) o.sex = sex;
          if (tribe) o.tribe = tribe;
          engine.place(x, y, o);
        }
        break;
      case 'erase':
        engine.erase(x, y);
        break;
    }
  };

  choose('inspect');
  return h('div', { class: 'tools' }, h('div', { class: 'tool-buttons' }, ...buttons), options);
}
