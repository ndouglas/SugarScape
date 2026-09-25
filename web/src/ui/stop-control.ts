import type { Engine, RunControls } from '../engine';
import type { StopRules } from '../protocol';
import { config_series_names } from '../wasm-pkg/sugarscape.js';
import { h } from './dom';

/** The rules the fields describe, or what is wrong with them. Blank fields make no rule. */
export function parseStopRules(input: { tick: string; series: string; op: string; value: string }): StopRules | string {
  const rules: StopRules = {};
  const tick = input.tick.trim();
  if (tick !== '') {
    const t = Number(tick);
    if (!Number.isInteger(t) || t < 0) return 'The tick must be a whole number, 0 or more';
    rules.tick = t;
  }
  const value = input.value.trim();
  if (value !== '') {
    const v = Number(value);
    if (!Number.isFinite(v)) return 'The value must be a number';
    rules.when = { series: input.series, op: input.op === '>' ? '>' : '<', value: v };
  }
  return rules;
}

/** "Stop at tick [ ] · when [series] [<|>] [ ]": sets the run controls' stop rules as they are edited. */
export class StopControl {
  readonly el: HTMLElement;
  private controls: RunControls;
  private readonly tick = h('input', { type: 'number', min: 0, step: 1, class: 'stop-tick', placeholder: 'tick', 'aria-label': 'Stop at tick' });
  private readonly series = h('select', { 'aria-label': 'Series' });
  private readonly op = h('select', { 'aria-label': 'Comparison' }, h('option', { value: '<' }, '<'), h('option', { value: '>' }, '>'));
  private readonly value = h('input', { type: 'number', step: 'any', class: 'stop-value', placeholder: 'value', 'aria-label': 'Value' });
  private readonly when: HTMLElement;
  private readonly error = h('span', { class: 'stop-error', role: 'alert' });
  private model: string;

  constructor(private readonly engine: Engine) {
    this.controls = engine;
    this.model = engine.model;
    this.when = h('span', { class: 'stop-when' }, ' when ', this.series, this.op, this.value);
    for (const el of [this.tick, this.value]) el.addEventListener('change', () => this.apply());
    for (const el of [this.series, this.op]) el.addEventListener('change', () => this.apply());
    this.el = h('div', { class: 'group stop-control', title: 'Pause by itself at a tick, or when a series crosses a value' }, 'Stop at ', this.tick, this.when, this.error);
    this.fillSeries();
    // A new model has other series: its rules go. A sugarscape's own series follow its config too
    // (per-good, per-pollutant, per-group), so the list is rebuilt on every reset and config change,
    // not only a model-kind change; the current selection is kept if it still exists there.
    const resync = () => {
      if (engine.model !== this.model) {
        this.model = engine.model;
        this.fillSeries();
        this.clear();
      } else {
        this.fillSeries();
      }
    };
    engine.on('reset', resync);
    engine.on('config', resync);
  }

  bind(controls: RunControls): void {
    this.controls = controls;
    const on = controls.conditionStops;
    this.series.disabled = this.op.disabled = this.value.disabled = !on;
    this.when.title = on ? '' : 'In Compare a run stops only at a tick: the two worlds could meet a condition on different ticks';
    this.apply();
  }

  clear(): void {
    this.tick.value = '';
    this.value.value = '';
    this.apply();
  }

  private apply(): void {
    const rules = parseStopRules({ tick: this.tick.value, series: this.series.value, op: this.op.value, value: this.value.value });
    this.error.textContent = typeof rules === 'string' ? rules : '';
    if (typeof rules === 'string') return;
    if (!this.controls.conditionStops) delete rules.when;
    this.controls.setStops(rules);
  }

  /** Rebuilds the series options, keeping the current selection if it still exists there; if it
   * doesn't, the condition can no longer describe it, so its value is cleared and the rules reapplied. */
  private fillSeries(): void {
    const had = this.series.value;
    const names = seriesNames(this.engine);
    this.series.replaceChildren(...names.map((n) => h('option', { value: n }, n)));
    if (!had) return;
    if (names.includes(had)) {
      this.series.value = had;
    } else {
      this.value.value = '';
      this.apply();
    }
  }
}

/** The current config's chartable series (the same list Experiments' series picker offers), population first, no duplicates. */
function seriesNames(engine: Engine): string[] {
  let names: string[];
  try {
    names = JSON.parse(config_series_names(JSON.stringify(engine.config))) as string[];
  } catch {
    names = [];
  }
  const unique = [...new Set(names)];
  return unique.includes('population') ? ['population', ...unique.filter((n) => n !== 'population')] : unique;
}
