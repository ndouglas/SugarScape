import { MAX_POLLUTANTS, addPollutant, removePollutant } from '../goods';
import type { Config, Pollutant } from '../types';
import { h } from './dom';
import { num, type Commit } from './goods-editor';

/** Rows per pollutant — production and consumption per good, and the goods it devalues — plus Add. */
export function pollutionEditor(config: Config, commit: Commit): HTMLElement {
  const goods = config.goods;
  const list = config.pollution.pollutants;
  const header = h(
    'tr',
    {},
    h('th', {}, ''),
    ...goods.map((g) => h('th', {}, h('span', { class: 'swatch', style: `background:${g.color}` }), ` ${g.name}`)),
  );
  const rows: HTMLElement[] = [];
  list.forEach((p, k) => {
    const edit = (f: (x: Pollutant) => void) => commit((c) => f(c.pollution.pollutants[k]), false);
    const name = h('input', { type: 'text', value: p.name, maxLength: 16, class: 'name' });
    name.addEventListener('change', () => edit((x) => (x.name = name.value)));
    const remove = h('button', { disabled: list.length <= 1, onclick: () => commit((c) => removePollutant(c, k), true) }, 'Remove');
    rows.push(h('tr', { class: 'pollutant' }, h('th', { colSpan: goods.length + 1 }, h('span', { class: 'row' }, name, remove))));
    const coefficients = (label: string, key: 'production' | 'consumption') =>
      h('tr', {}, h('th', {}, label), ...goods.map((_, i) => h('td', {}, num(p[key][i], 0, 5, 0.1, (v) => edit((x) => (x[key][i] = v))))));
    rows.push(coefficients('Per unit gathered', 'production'), coefficients('Per unit eaten', 'consumption'));
    rows.push(
      h('tr', {}, h('th', {}, 'Devalues'), ...goods.map((_, i) => {
        const box = h('input', { type: 'checkbox', checked: p.devalues[i] });
        box.addEventListener('change', () => edit((x) => (x.devalues[i] = box.checked)));
        return h('td', {}, box);
      })),
    );
  });
  return h(
    'div',
    { class: 'pollution-table' },
    h('table', {}, h('thead', {}, header), h('tbody', {}, ...rows)),
    h('button', { disabled: list.length >= MAX_POLLUTANTS, onclick: () => commit((c) => addPollutant(c), true) }, 'Add pollutant'),
  );
}
