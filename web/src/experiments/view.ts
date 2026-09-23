import { aggregate, builtin_sweeps, config_series_names, sweep_points } from '../wasm-pkg/sugarscape.js';
import type { Engine } from '../engine';
import { parseErrors, type Config, type FieldError } from '../types';
import { h } from '../ui/dom';
import { SweepChart } from './chart';
import { chartData } from './chart-data';
import { FixedPanel } from './fixed-panel';
import { defaultForm, numericPaths, type SweepForm } from './form';
import { FormView } from './form-view';
import { baseLabel } from './labels';
import { poolSize, WorkerPool, type WorkerLike } from './pool';
import { resultsTable } from './results-table';
import type { BuiltinSweep, Point, RunResult, Summary, Sweep, SweepBase } from './types';

/** Each worker loads its own WASM instance (Decision 19). */
const createWorker = (): WorkerLike => new Worker(new URL('./worker.ts', import.meta.url), { type: 'module' });

/** Partial results are re-aggregated and redrawn at most this often. */
const REDRAW_MS = 250;

/** The sweep being edited: the form, or a fixed sweep whose seeds and ticks alone change. */
export interface SweepEditor {
  readonly el: HTMLElement;
  /** The sweep, or null (with its errors shown) when the editor cannot build one. */
  sweep(): Sweep | null;
  showErrors(errors: FieldError[]): void;
}

/** Runs on screen and the sweep they belong to (with its JSON, as sent to WASM). */
interface Shown { sweep: Sweep; spec: string; runs: RunResult[] }

// PF5: reuses `baseLabel` (Task 10, labels.ts) instead of re-implementing its preset/custom-config wording;
// `baseLabel` reads only `sweep.base`, so a base alone (no full sweep yet) is cast to satisfy its signature.
const baseNote = (base: SweepBase): string => baseLabel({ base } as unknown as Sweep);

/** The Experiments view: pick or build a sweep, run it on workers, show the results. */
export class ExperimentsView {
  readonly el: HTMLElement;
  private readonly builtins = JSON.parse(builtin_sweeps()) as BuiltinSweep[];
  private readonly picker: HTMLSelectElement;
  private readonly editorSlot = h('div', { class: 'sweep-editor' });
  private editor: SweepEditor | null = null;
  private readonly runButton = h('button', { class: 'primary', onclick: () => void this.run() }, 'Run');
  private readonly cancelButton = h('button', { disabled: true, onclick: () => this.pool?.cancel() }, 'Cancel');
  private readonly bar = h('progress', { max: 1, value: 0 });
  private readonly status = h('span', { class: 'hint', role: 'status' });
  private readonly chart = new SweepChart();
  private readonly table = h('div');
  private pool: WorkerPool | null = null;
  private shown: Shown | null = null;

  constructor(private readonly engine: Engine) {
    this.picker = h(
      'select',
      { 'aria-label': 'Sweep', onchange: () => this.pick(this.picker.value) },
      h('optgroup', { label: 'Built-in' }, ...this.builtins.map((b) => h('option', { value: `builtin:${b.id}` }, b.sweep.name))),
      h('option', { value: 'current' }, 'From current world'),
    );
    this.el = h(
      'div',
      { class: 'experiments-view' },
      h(
        'section',
        { class: 'sweep-setup' },
        h('div', { class: 'row' }, h('label', {}, 'Sweep ', this.picker)),
        this.editorSlot,
        h('div', { class: 'run-bar' }, this.runButton, this.cancelButton, this.bar, this.status),
      ),
      h('section', { class: 'sweep-results' }, this.chart.el, this.table),
    );
    this.pick(this.picker.value);
  }

  private pick(choice: string): void {
    if (choice === 'current') {
      const base = this.currentBase();
      const painted = 'config' in base && this.engine.editedLandscapes() !== undefined;
      const note = `${baseNote(base)} from the current world${painted ? ' (painted maps are not included)' : ''}, captured when chosen`;
      this.setEditor(this.formView(defaultForm(), base, note));
      return;
    }
    const builtin = this.builtins.find((b) => choice === `builtin:${b.id}`);
    if (builtin) this.setEditor(new FixedPanel(builtin.sweep, () => this.validate()));
  }

  /** The current world as a base: its preset when unmodified, otherwise its config (Decision 17). */
  private currentBase(): SweepBase {
    const e = this.engine;
    return e.presetId !== null && !e.isModified() ? { preset: e.presetId } : { config: structuredClone(e.baseConfig) };
  }

  private configOf(base: SweepBase): Config | null {
    return 'preset' in base ? (this.engine.presets.find((p) => p.id === base.preset)?.config ?? null) : base.config;
  }

  private formView(form: SweepForm, base: SweepBase, note: string): FormView {
    const config = this.configOf(base);
    let names: string[] = [];
    try {
      if (config) names = JSON.parse(config_series_names(JSON.stringify(config))) as string[];
    } catch {
      names = [];
    }
    return new FormView(form, base, note, config ? numericPaths(config) : [], names, () => this.validate());
  }

  private setEditor(editor: SweepEditor): void {
    this.editor = editor;
    this.editorSlot.replaceChildren(editor.el);
    this.validate();
  }

  /** Checks the editor's sweep with the core, shows its errors or size, and returns it when valid. */
  private validate(): { sweep: Sweep; spec: string; count: number } | null {
    const editor = this.editor;
    const sweep = editor?.sweep() ?? null;
    if (!editor || !sweep) {
      this.status.textContent = '';
      return null;
    }
    const spec = JSON.stringify(sweep);
    try {
      const count = (JSON.parse(sweep_points(spec)) as Point[]).length;
      editor.showErrors([]);
      this.status.textContent = `${count} runs`;
      return { sweep, spec, count };
    } catch (e) {
      editor.showErrors(parseErrors(e));
      this.status.textContent = '';
      return null;
    }
  }

  private async run(): Promise<void> {
    if (this.pool) return;
    const checked = this.validate();
    if (!checked) return;
    const { sweep, spec, count } = checked;
    const shown: Shown = { sweep, spec, runs: [] };
    this.shown = shown;
    this.showResults();
    const pool = new WorkerPool(poolSize(navigator.hardwareConcurrency), createWorker);
    this.pool = pool;
    this.setRunning(true);
    this.bar.max = count;
    this.bar.value = 0;
    let dirty = false;
    const timer = setInterval(() => {
      if (dirty) {
        dirty = false;
        this.showResults();
      }
    }, REDRAW_MS);
    try {
      const outcome = await pool.run(spec, count, (run) => {
        shown.runs.push(run);
        this.bar.value = shown.runs.length;
        this.status.textContent = `${shown.runs.length} / ${count} runs`;
        dirty = true;
      });
      this.status.textContent =
        outcome === 'done' ? `${count} runs done` : `Cancelled after ${shown.runs.length} of ${count} runs: the results are incomplete`;
    } catch (e) {
      this.status.textContent = `The sweep failed: ${e instanceof Error ? e.message : String(e)}`;
    } finally {
      clearInterval(timer);
      this.pool = null;
      this.setRunning(false);
      this.showResults();
    }
  }

  private setRunning(on: boolean): void {
    this.runButton.disabled = on;
    this.cancelButton.disabled = !on;
    this.picker.disabled = on;
  }

  /** Re-aggregates the runs on screen with the core and redraws them. */
  private showResults(): void {
    const shown = this.shown;
    if (!shown || shown.runs.length === 0) {
      this.chart.clear();
      this.table.replaceChildren();
      return;
    }
    const summary = JSON.parse(aggregate(shown.spec, JSON.stringify(shown.runs))) as Summary;
    this.chart.draw(chartData(shown.sweep, summary));
    this.table.replaceChildren(resultsTable(summary));
  }
}
