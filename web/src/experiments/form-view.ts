import type { FieldError } from '../types';
import { h } from '../ui/dom';
import { controlFor, formToSweep, type AxisForm, type SweepForm } from './form';
import type { Metric, Sweep, SweepBase } from './types';

const KINDS: { value: Metric['kind']; label: string }[] = [
  { value: 'window_mean', label: 'Mean over a window of ticks' },
  { value: 'final', label: 'Value at the last tick' },
  { value: 'timeseries', label: 'Time series (block means)' },
];

/** Suggestions for the config-path inputs. */
const PATH_LIST = 'sweep-config-paths';

/** The editable sweep: a config path and values per axis, seeds, ticks and the metric (Decision 18). */
export class FormView {
  readonly el = h('div', { class: 'sweep-form' });
  private readonly errorEls = new Map<string, HTMLElement>();

  constructor(
    private readonly form: SweepForm,
    private readonly base: SweepBase,
    private readonly baseNote: string,
    private readonly paths: string[],
    private readonly seriesNames: string[],
    private readonly onChange: () => void,
  ) {
    this.render();
  }

  sweep(): Sweep | null {
    const { sweep, errors } = formToSweep(this.form, this.base);
    if (!sweep) this.showErrors(errors);
    return sweep;
  }

  showErrors(errors: FieldError[]): void {
    for (const el of this.errorEls.values()) el.replaceChildren();
    for (const e of errors) {
      const key = controlFor(e.field);
      const el = this.errorEls.get(key) ?? this.errorEls.get('general');
      el?.append(h('p', {}, key === e.field ? e.message : `${e.field}: ${e.message}`));
    }
  }

  private errorSlot(key: string): HTMLElement {
    const el = h('div', { class: 'error', 'aria-live': 'polite' });
    this.errorEls.set(key, el);
    return el;
  }

  /** A caption and its input in one <label> (so the input is named by the caption), then the control's errors. */
  private control(label: string, key: string, input: HTMLElement): HTMLElement {
    return h('div', { class: 'control' }, h('label', {}, h('span', {}, label), h('div', { class: 'row' }, input)), this.errorSlot(key));
  }

  private text(value: string, set: (v: string) => void, placeholder: string, list?: string): HTMLInputElement {
    const input = h('input', {
      type: 'text',
      value,
      placeholder,
      onchange: () => {
        set(input.value);
        this.onChange();
      },
    });
    // `list` is read-only as a property; it can only be set as an attribute.
    if (list) input.setAttribute('list', list);
    return input;
  }

  private number(value: number | null, set: (v: number | null) => void, props: Record<string, unknown> = {}): HTMLInputElement {
    const input = h('input', {
      type: 'number',
      class: 'num',
      value: value === null ? '' : String(value),
      ...props,
      onchange: () => {
        set(input.value === '' ? null : Number(input.value));
        this.onChange();
      },
    });
    return input;
  }

  private axis(title: string, key: 'x' | 'series', axis: AxisForm): HTMLElement {
    return h(
      'fieldset',
      {},
      h('legend', {}, title),
      this.control('Config path', `${key}.path`, this.text(axis.path, (v) => (axis.path = v), 'e.g. vision.max', PATH_LIST)),
      this.control('Values', `${key}.values`, this.text(axis.values, (v) => (axis.values = v), '1, 2, 3 or 1:6:1')),
    );
  }

  private render(): void {
    this.errorEls.clear();
    const f = this.form;
    const m = f.metric;
    const timeseries = m.kind === 'timeseries';
    const second = h('input', {
      type: 'checkbox',
      checked: f.series !== null,
      onchange: () => {
        f.series = second.checked ? { path: '', values: '' } : null;
        this.render();
        this.onChange();
      },
    });
    const kind = h(
      'select',
      {
        onchange: () => {
          m.kind = kind.value as Metric['kind'];
          this.render();
          this.onChange();
        },
      },
      ...KINDS.map((k) => h('option', { value: k.value, selected: k.value === m.kind }, k.label)),
    );
    const names = this.seriesNames.includes(m.series) ? this.seriesNames : [m.series, ...this.seriesNames];
    const statistic = h(
      'select',
      {
        onchange: () => {
          m.series = statistic.value;
          this.onChange();
        },
      },
      ...names.map((n) => h('option', { value: n, selected: n === m.series }, n)),
    );
    // `replaceChildren` (unlike `h`) rejects `null` children under TS, so nulls are filtered before the call.
    const children: (HTMLElement | null)[] = [
      h('datalist', { id: PATH_LIST }, ...this.paths.map((p) => h('option', { value: p }))),
      this.control('Name', 'name', this.text(f.name, (v) => (f.name = v), 'Untitled sweep')),
      h('p', { class: 'hint' }, `Base: ${this.baseNote}`),
      timeseries
        ? h('p', { class: 'hint' }, 'A time series is plotted against the tick; the optional axis below gives one line per value.')
        : this.axis('x axis', 'x', f.x),
      h('label', {}, second, ' Second axis: one line per value'),
      f.series ? this.axis('Lines', 'series', f.series) : null,
      this.control('Seeds', 'seeds.count', this.number(f.seeds, (v) => (f.seeds = v ?? 0), { min: 1, max: 100 })),
      this.control('Ticks', 'ticks', this.number(f.ticks, (v) => (f.ticks = v ?? 0), { min: 1, max: 100000 })),
      this.control('Metric', 'metric.kind', kind),
      this.control('Statistic', 'metric.series', statistic),
      m.kind === 'window_mean'
        ? this.control('Window from tick', 'metric.from', this.number(m.from, (v) => (m.from = v ?? 0), { min: 0 }))
        : null,
      m.kind === 'window_mean'
        ? this.control('to tick (blank: the last)', 'metric.to', this.number(m.to, (v) => (m.to = v), { min: 0 }))
        : null,
      timeseries
        ? this.control('Block length (ticks)', 'metric.every', this.number(m.every, (v) => (m.every = v ?? 0), { min: 1 }))
        : null,
      this.errorSlot('general'),
    ];
    this.el.replaceChildren(...children.filter((c): c is HTMLElement => c !== null));
  }
}
