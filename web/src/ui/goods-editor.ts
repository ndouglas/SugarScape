import { MAX_GOODS, MAX_PEAKS, TRANSFORMS, addGood, defaultMap, newPeak, removeGood } from '../goods';
import type { Config, Good, GoodMap, Peak, Transform, URange } from '../types';
import { h } from './dom';

export type Commit = (mutate: (c: Config) => void, reset: boolean) => void;

/** Shows a config's values in an already-built editor. */
export type Sync = (c: Config) => void;

/** An editor built for one structure (see the signatures in `goods.ts`) whose values `sync` refreshes. */
export interface Editor {
  el: HTMLElement;
  sync: Sync;
}

const KINDS: [GoodMap['kind'], string][] = [
  ['two_peaks', 'Two peaks (50×50)'],
  ['peaks', 'Peaks'],
  ['flat', 'Flat'],
];

/** Registers `show` to refresh `el` from each synced config, except a text-like input while it has focus. */
export function bind(syncs: Sync[], el: HTMLElement, show: Sync): void {
  const typed = el instanceof HTMLInputElement && el.type !== 'checkbox';
  syncs.push((c) => {
    if (!typed || document.activeElement !== el) show(c);
  });
}

/** Collects the syncers into one editor and shows `config`'s values. */
export function editor(el: HTMLElement, syncs: Sync[], config: Config): Editor {
  const sync: Sync = (c) => syncs.forEach((s) => s(c));
  sync(config);
  return { el, sync };
}

/** A number input showing `get(config)`; a change calls `set`. */
export function num(syncs: Sync[], get: (c: Config) => number, min: number, max: number, step: number, set: (v: number) => void): HTMLInputElement {
  const input = h('input', { type: 'number', min, max, step, class: 'num' });
  input.addEventListener('change', () => set(Number(input.value)));
  bind(syncs, input, (c) => (input.value = String(get(c))));
  return input;
}

/** Min and max inputs; each commit changes only its own end of the range. */
function range(syncs: Sync[], get: (c: Config) => URange, min: number, max: number, edit: (f: (r: URange) => void) => void): HTMLElement {
  return h(
    'span',
    { class: 'row' },
    num(syncs, (c) => get(c).min, min, max, 1, (v) => edit((r) => (r.min = v))),
    h('span', { class: 'hint' }, 'to'),
    num(syncs, (c) => get(c).max, min, max, 1, (v) => edit((r) => (r.max = v))),
  );
}

/** The kind-specific part of good `i`'s map (changes rebuild the world). */
function mapDetails(config: Config, i: number, commit: Commit, syncs: Sync[]): HTMLElement {
  const map = config.goods[i].map;
  const setMap = (f: (m: GoodMap) => void) => commit((c) => f(c.goods[i].map), true);
  switch (map.kind) {
    case 'two_peaks': {
      const select = h('select', {}, ...TRANSFORMS.map(([v, l]) => h('option', { value: v }, l)));
      bind(syncs, select, (c) => {
        const m = c.goods[i].map;
        if (m.kind === 'two_peaks') select.value = m.transform;
      });
      select.addEventListener('change', () =>
        setMap((m) => {
          if (m.kind === 'two_peaks') m.transform = select.value as Transform;
        }),
      );
      return select;
    }
    case 'flat':
      return num(
        syncs,
        (c) => {
          const m = c.goods[i].map;
          return m.kind === 'flat' ? m.capacity : 0;
        },
        0,
        10,
        0.5,
        (v) =>
          setMap((m) => {
            if (m.kind === 'flat') m.capacity = v;
          }),
      );
    case 'peaks': {
      const rows = map.peaks.map((_, k) => {
        const field = (key: keyof Peak, min: number, max: number, step: number) =>
          h('label', {}, `${key} `, num(
            syncs,
            (c) => {
              const m = c.goods[i].map;
              return m.kind === 'peaks' ? m.peaks[k][key] : 0;
            },
            min,
            max,
            step,
            (v) =>
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
        return h(
          'div',
          { class: 'row peak' },
          field('x', 0, config.width - 1, 1),
          field('y', 0, config.height - 1, 1),
          field('radius', 0.5, 250, 0.5),
          field('height', 0, 10, 0.5),
          remove,
        );
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

function goodRow(config: Config, i: number, commit: Commit, syncs: Sync[]): HTMLElement {
  const count = config.goods.length;
  const edit = (f: (x: Good) => void) => commit((c) => f(c.goods[i]), false);
  const name = h('input', { type: 'text', maxLength: 16, class: 'name', title: 'Name' });
  bind(syncs, name, (c) => (name.value = c.goods[i].name));
  name.addEventListener('change', () => edit((x) => (x.name = name.value)));
  const color = h('input', { type: 'color', title: 'Color' });
  bind(syncs, color, (c) => (color.value = c.goods[i].color));
  color.addEventListener('change', () => edit((x) => (x.color = color.value)));
  const kind = h('select', {}, ...KINDS.map(([v, l]) => h('option', { value: v }, l)));
  bind(syncs, kind, (c) => (kind.value = c.goods[i].map.kind));
  kind.addEventListener('change', () => commit((c) => (c.goods[i].map = defaultMap(c, kind.value as GoodMap['kind'])), true));
  const remove = h('button', { disabled: count <= 1, title: 'Remove this good (rebuilds the world)', onclick: () => commit((c) => removeGood(c, i), true) }, 'Remove');
  return h(
    'div',
    { class: 'good' },
    h('div', { class: 'row' }, color, name, remove),
    h('div', { class: 'control' }, h('label', {}, 'Map'), h('div', { class: 'row' }, kind, mapDetails(config, i, commit, syncs))),
    h('div', { class: 'control' }, h('label', {}, 'Metabolism'), range(syncs, (c) => c.goods[i].metabolism, 0, 10, (f) => edit((x) => f(x.metabolism)))),
    h('div', { class: 'control' }, h('label', {}, 'Initial endowment'), range(syncs, (c) => c.goods[i].endowment, 0, 500, (f) => edit((x) => f(x.endowment)))),
  );
}

/** One row per good plus an Add button, for `config`'s `goodsEditorSignature`. */
export function goodsEditor(config: Config, commit: Commit): Editor {
  const n = config.goods.length;
  const syncs: Sync[] = [];
  const el = h(
    'div',
    { class: 'goods' },
    ...config.goods.map((_, i) => goodRow(config, i, commit, syncs)),
    h('button', { disabled: n >= MAX_GOODS, onclick: () => commit((c) => addGood(c), true) }, 'Add good'),
  );
  return editor(el, syncs, config);
}
