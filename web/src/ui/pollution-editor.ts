import { MAX_POLLUTANTS, addPollutant, removePollutant } from '../goods';
import type { Config, Pollutant } from '../types';
import { h } from './dom';
import { bind, editor, num, type Commit, type Editor, type Sync } from './goods-editor';

/**
 * Rows per pollutant — production and consumption per good, and the goods it devalues — plus Add,
 * for `config`'s `pollutionEditorSignature`.
 */
export function pollutionEditor(config: Config, commit: Commit): Editor {
  const goods = config.goods;
  const list = config.pollution.pollutants;
  const syncs: Sync[] = [];
  const header = h(
    'tr',
    {},
    h('th', {}, ''),
    ...goods.map((g) => h('th', {}, h('span', { class: 'swatch', style: `background:${g.color}` }), ` ${g.name}`)),
  );
  const rows: HTMLElement[] = [];
  list.forEach((_, k) => {
    const edit = (f: (x: Pollutant) => void) => commit((c) => f(c.pollution.pollutants[k]), false);
    const name = h('input', { type: 'text', maxLength: 16, class: 'name' });
    bind(syncs, name, (c) => (name.value = c.pollution.pollutants[k].name));
    name.addEventListener('change', () => edit((x) => (x.name = name.value)));
    const remove = h('button', { disabled: list.length <= 1, onclick: () => commit((c) => removePollutant(c, k), true) }, 'Remove');
    rows.push(h('tr', { class: 'pollutant' }, h('th', { colSpan: goods.length + 1 }, h('span', { class: 'row' }, name, remove))));
    const coefficients = (label: string, key: 'production' | 'consumption') =>
      h(
        'tr',
        {},
        h('th', {}, label),
        ...goods.map((_, i) =>
          h('td', {}, num(syncs, (c) => c.pollution.pollutants[k][key][i], 0, 5, 0.1, (v) => edit((x) => (x[key][i] = v)))),
        ),
      );
    rows.push(coefficients('Per unit gathered', 'production'), coefficients('Per unit eaten', 'consumption'));
    rows.push(
      h('tr', {}, h('th', {}, 'Devalues'), ...goods.map((_, i) => {
        const box = h('input', { type: 'checkbox' });
        bind(syncs, box, (c) => (box.checked = c.pollution.pollutants[k].devalues[i]));
        box.addEventListener('change', () => edit((x) => (x.devalues[i] = box.checked)));
        return h('td', {}, box);
      })),
    );
  });
  const el = h(
    'div',
    { class: 'pollution-table' },
    h('table', {}, h('thead', {}, header), h('tbody', {}, ...rows)),
    h('button', { disabled: list.length >= MAX_POLLUTANTS, onclick: () => commit((c) => addPollutant(c), true) }, 'Add pollutant'),
  );
  return editor(el, syncs, config);
}
