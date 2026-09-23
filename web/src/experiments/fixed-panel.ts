import type { FieldError } from '../types';
import { h } from '../ui/dom';
import { axisLabel, baseLabel, metricLabel } from './labels';
import type { Sweep } from './types';

/** A sweep shown read-only except its seed count and ticks: the built-ins, and files the form cannot express. */
export class FixedPanel {
  readonly el: HTMLElement;
  private readonly seeds: HTMLInputElement;
  private readonly ticks: HTMLInputElement;
  private readonly errors = h('div', { class: 'error' });

  constructor(
    private readonly base: Sweep,
    onChange: () => void,
  ) {
    this.seeds = h('input', { type: 'number', class: 'num', min: 1, max: 100, value: String(base.seeds.count), onchange: onChange });
    this.ticks = h('input', { type: 'number', class: 'num', min: 1, max: 100000, value: String(base.ticks), onchange: onChange });
    const lines = base.series ? `${axisLabel(base.series)}: ${base.series.values.length} lines` : 'one line';
    this.el = h(
      'div',
      { class: 'sweep-fixed' },
      h('h2', {}, base.name),
      base.description ? h('p', { class: 'hint' }, base.description) : null,
      h(
        'dl',
        {},
        h('dt', {}, 'Base'),
        h('dd', {}, baseLabel(base)),
        h('dt', {}, 'x'),
        h('dd', {}, `${axisLabel(base.x)}: ${base.x.values.length} values`),
        h('dt', {}, 'Lines'),
        h('dd', {}, lines),
        h('dt', {}, 'Metric'),
        h('dd', {}, metricLabel(base)),
      ),
      h('div', { class: 'row' }, h('label', {}, 'Seeds ', this.seeds), h('label', {}, 'Ticks ', this.ticks)),
      this.errors,
    );
  }

  sweep(): Sweep {
    return {
      ...structuredClone(this.base),
      seeds: { ...this.base.seeds, count: Number(this.seeds.value) },
      ticks: Number(this.ticks.value),
    };
  }

  showErrors(errors: FieldError[]): void {
    this.errors.replaceChildren(...errors.map((e) => h('p', {}, `${e.field}: ${e.message}`)));
  }
}
