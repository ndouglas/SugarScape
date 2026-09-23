import { MAX_GOODS, MAX_PEAKS, TRANSFORMS, addGood, defaultMap, newPeak, removeGood } from '../goods';
import type { Config, Good, GoodMap, Peak, Transform, URange } from '../types';
import { h } from './dom';

export type Commit = (mutate: (c: Config) => void, reset: boolean) => void;

const KINDS: [GoodMap['kind'], string][] = [
  ['two_peaks', 'Two peaks (50×50)'],
  ['peaks', 'Peaks'],
  ['flat', 'Flat'],
];

export function num(value: number, min: number, max: number, step: number, set: (v: number) => void): HTMLInputElement {
  const input = h('input', { type: 'number', min, max, step, value, class: 'num' });
  input.addEventListener('change', () => set(Number(input.value)));
  return input;
}

function range(r: URange, min: number, max: number, set: (r: URange) => void): HTMLElement {
  return h(
    'span',
    { class: 'row' },
    num(r.min, min, max, 1, (v) => set({ min: v, max: r.max })),
    h('span', { class: 'hint' }, 'to'),
    num(r.max, min, max, 1, (v) => set({ min: r.min, max: v })),
  );
}

/** The kind-specific part of good `i`'s map (changes rebuild the world). */
function mapDetails(map: GoodMap, i: number, commit: Commit): HTMLElement {
  const setMap = (f: (m: GoodMap) => void) => commit((c) => f(c.goods[i].map), true);
  switch (map.kind) {
    case 'two_peaks': {
      const select = h('select', {}, ...TRANSFORMS.map(([v, l]) => h('option', { value: v }, l)));
      select.value = map.transform;
      select.addEventListener('change', () =>
        setMap((m) => {
          if (m.kind === 'two_peaks') m.transform = select.value as Transform;
        }),
      );
      return select;
    }
    case 'flat':
      return num(map.capacity, 0, 10, 0.5, (v) =>
        setMap((m) => {
          if (m.kind === 'flat') m.capacity = v;
        }),
      );
    case 'peaks': {
      const rows = map.peaks.map((p, k) => {
        const field = (key: keyof Peak, min: number, max: number, step: number) =>
          h('label', {}, `${key} `, num(p[key], min, max, step, (v) =>
            setMap((m) => {
              if (m.kind === 'peaks') m.peaks[k][key] = v;
            }),
          ));
        const remove = h('button', {
          title: 'Remove this peak',
          disabled: map.peaks.length <= 1,
          onclick: () =>
            setMap((m) => {
              if (m.kind === 'peaks') m.peaks.splice(k, 1);
            }),
        }, '×');
        return h('div', { class: 'row peak' }, field('x', 0, 499, 1), field('y', 0, 499, 1), field('radius', 0.5, 250, 0.5), field('height', 0, 10, 0.5), remove);
      });
      const add = h('button', {
        disabled: map.peaks.length >= MAX_PEAKS,
        onclick: () =>
          commit((c) => {
            const m = c.goods[i].map;
            if (m.kind === 'peaks' && m.peaks.length < MAX_PEAKS) m.peaks.push(newPeak(c));
          }, true),
      }, 'Add peak');
      return h('div', { class: 'peaks' }, ...rows, add);
    }
  }
}

function goodRow(g: Good, i: number, count: number, commit: Commit): HTMLElement {
  const edit = (f: (x: Good) => void) => commit((c) => f(c.goods[i]), false);
  const name = h('input', { type: 'text', value: g.name, maxLength: 16, class: 'name', title: 'Name' });
  name.addEventListener('change', () => edit((x) => (x.name = name.value)));
  const color = h('input', { type: 'color', value: g.color, title: 'Color' });
  color.addEventListener('change', () => edit((x) => (x.color = color.value)));
  const kind = h('select', {}, ...KINDS.map(([v, l]) => h('option', { value: v }, l)));
  kind.value = g.map.kind;
  kind.addEventListener('change', () => commit((c) => (c.goods[i].map = defaultMap(c, kind.value as GoodMap['kind'])), true));
  const remove = h('button', { disabled: count <= 1, title: 'Remove this good (rebuilds the world)', onclick: () => commit((c) => removeGood(c, i), true) }, 'Remove');
  return h(
    'div',
    { class: 'good' },
    h('div', { class: 'row' }, color, name, remove),
    h('div', { class: 'control' }, h('label', {}, 'Map'), h('div', { class: 'row' }, kind, mapDetails(g.map, i, commit))),
    h('div', { class: 'control' }, h('label', {}, 'Metabolism'), range(g.metabolism, 0, 10, (r) => edit((x) => (x.metabolism = r)))),
    h('div', { class: 'control' }, h('label', {}, 'Initial endowment'), range(g.endowment, 0, 500, (r) => edit((x) => (x.endowment = r)))),
  );
}

/** One row per good plus an Add button. */
export function goodsEditor(config: Config, commit: Commit): HTMLElement {
  const n = config.goods.length;
  return h(
    'div',
    { class: 'goods' },
    ...config.goods.map((g, i) => goodRow(g, i, n, commit)),
    h('button', { disabled: n >= MAX_GOODS, onclick: () => commit((c) => addGood(c), true) }, 'Add good'),
  );
}
