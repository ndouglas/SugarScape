import { h } from '../ui/dom';
import { fmt } from './format';
import type { Summary } from './types';

const cells = (tag: 'th' | 'td', values: string[]): HTMLTableCellElement[] => values.map((v) => h(tag, {}, v));

/** The summary as a table; collapsed for time series, which can be long. */
export function resultsTable(summary: Summary): HTMLElement {
  const table =
    summary.kind === 'scalar'
      ? h(
          'table',
          {},
          h('thead', {}, h('tr', {}, ...cells('th', ['Line', 'x', 'n', 'mean', 'sd', 'min', 'max']))),
          h(
            'tbody',
            {},
            ...summary.rows.map((r) =>
              h('tr', {}, ...cells('td', [r.series_name, fmt(r.at), String(r.n), fmt(r.mean), fmt(r.sd), fmt(r.min), fmt(r.max)])),
            ),
          ),
        )
      : h(
          'table',
          {},
          h('thead', {}, h('tr', {}, ...cells('th', ['Line', 't', 'n', 'mean', 'sd']))),
          h(
            'tbody',
            {},
            ...summary.rows.map((r) => h('tr', {}, ...cells('td', [r.series_name, String(r.t), String(r.n), fmt(r.mean), fmt(r.sd)]))),
          ),
        );
  return h('details', { class: 'sweep-table', open: summary.kind === 'scalar' }, h('summary', {}, 'Summary table'), table);
}
