import {
  aggregate,
  builtin_sweeps,
  config_series_names,
  parse_sweep,
  sweep_csv,
  sweep_points,
  sweep_result,
} from '../wasm-pkg/sugarscape.js';
import { canvasBlob, downloadBlob, downloadText } from '../downloads';
import type { Engine } from '../engine';
import { encodeSweep } from '../share';
import { parseErrors, type FieldError, type ModelConfig } from '../types';
import { h } from '../ui/dom';
import { SweepChart } from './chart';
import { chartData } from './chart-data';
import { readOpened, slug, type OpenCore } from './file';
import { FixedPanel } from './fixed-panel';
import { defaultForm, numericPaths, sweepToForm, type SweepForm } from './form';
import { FormView } from './form-view';
import { baseLabel } from './labels';
import { poolSize, WorkerPool, type WorkerLike } from './pool';
import { resultsTable } from './results-table';
import type { BuiltinSweep, Point, RunResult, Summary, Sweep, SweepBase, SweepResult } from './types';

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

/** The sweep behind the picker's "Opened: …" entry, with the result file's runs when one was opened. */
interface OpenedEntry { sweep: Sweep; result?: SweepResult; status?: string }

/** The core checks for opened files and links. */
const OPEN_CORE: OpenCore = { parseSweep: parse_sweep, aggregate };

// Reuses `baseLabel` (labels.ts) instead of re-implementing its preset/custom-config wording.
// `baseLabel` takes `Pick<Sweep, 'base'>`, so a base alone (no full sweep yet) needs no cast.
const baseNote = (base: SweepBase): string => baseLabel({ base });

const messages = (errors: FieldError[]): string => errors.map((e) => `${e.field}: ${e.message}`).join('; ');

/** The Experiments view: pick, build or open a sweep, run it on workers, show and export the results. */
export class ExperimentsView {
  readonly el: HTMLElement;
  private readonly builtins = JSON.parse(builtin_sweeps()) as BuiltinSweep[];
  private readonly picker: HTMLSelectElement;
  private readonly fileInput: HTMLInputElement;
  /** A fieldset so that disabling it (while running) disables every control in the editor. */
  private readonly editorSlot = h('fieldset', { class: 'sweep-editor' });
  private editor: SweepEditor | null = null;
  private readonly runButton = h('button', { class: 'primary', onclick: () => void this.run() }, 'Run');
  private readonly cancelButton = h('button', { disabled: true, onclick: () => this.pool?.cancel() }, 'Cancel');
  private readonly openButton: HTMLButtonElement;
  private readonly bar = h('progress', { max: 1, value: 0, 'aria-label': 'Sweep progress' });
  private readonly status = h('span', { class: 'hint', role: 'status' });
  private readonly chart = new SweepChart();
  private readonly table = h('div');
  private readonly outputs: HTMLButtonElement[];
  private readonly outputsEl: HTMLElement;
  private pool: WorkerPool | null = null;
  private shown: Shown | null = null;
  private opened: OpenedEntry | null = null;

  constructor(private readonly engine: Engine) {
    this.picker = h(
      'select',
      { 'aria-label': 'Sweep', onchange: () => this.pick(this.picker.value) },
      h('optgroup', { label: 'Built-in' }, ...this.builtins.map((b) => h('option', { value: `builtin:${b.id}` }, b.sweep.name))),
      h('option', { value: 'current' }, 'From current world'),
    );
    this.fileInput = h('input', {
      type: 'file',
      accept: '.json,application/json',
      hidden: true,
      onchange: () => void this.openFile(),
    });
    this.openButton = h(
      'button',
      { onclick: () => this.fileInput.click(), title: 'Open a sweep or a result file (JSON)' },
      'Open file…',
    );
    this.outputs = [
      h('button', { onclick: () => this.download('result') }, 'Result (JSON)'),
      h('button', { onclick: () => this.download('runs') }, 'Runs (CSV)'),
      h('button', { onclick: () => this.download('summary') }, 'Summary (CSV)'),
      h('button', { onclick: () => void this.downloadChart() }, 'Chart (PNG)'),
    ];
    this.outputsEl = h('div', { class: 'sweep-outputs', hidden: true }, ...this.outputs);
    this.el = h(
      'div',
      { class: 'experiments-view' },
      h(
        'section',
        { class: 'sweep-setup' },
        h(
          'div',
          { class: 'row' },
          h('label', { class: 'sweep-picker' }, 'Sweep ', this.picker),
          this.openButton,
          this.fileInput,
        ),
        this.editorSlot,
        h('div', { class: 'run-bar' }, this.runButton, this.cancelButton, this.bar, this.status),
        h(
          'div',
          { class: 'row' },
          h('button', { onclick: () => void this.share(), title: 'Copy a link that opens this sweep (not its results)' }, 'Share link'),
        ),
      ),
      h('section', { class: 'sweep-results' }, this.chart.el, this.table, this.outputsEl),
    );
    this.pick(this.picker.value);
    this.showResults();
  }

  /**
   * Opens a sweep from a link for editing, without running it. The core checks
   * it first; an invalid sweep throws and leaves the view unchanged.
   */
  openSweep(sweep: Sweep): void {
    const opened = readOpened(sweep, OPEN_CORE);
    if (opened.kind === 'error') throw new Error(opened.message);
    if (opened.kind !== 'sweep') throw new Error('expected a sweep');
    this.showOpened({ sweep: opened.sweep });
  }

  /**
   * Makes `entry` the picker's "Opened: …" entry and shows it. If showing it
   * fails, the previous entry, editor and results are put back and the error rethrown.
   */
  private showOpened(entry: OpenedEntry): void {
    const before = {
      opened: this.opened,
      option: this.picker.querySelector('option[value="opened"]'),
      value: this.picker.value,
      editor: this.editor,
      shown: this.shown,
    };
    try {
      this.opened = entry;
      this.markOpened(entry.sweep.name);
      this.pick('opened');
    } catch (e) {
      this.opened = before.opened;
      this.picker.querySelector('option[value="opened"]')?.remove();
      if (before.option) this.picker.prepend(before.option);
      this.picker.value = before.value;
      this.shown = before.shown;
      if (before.editor) this.setEditor(before.editor);
      else this.editorSlot.replaceChildren();
      this.showResults();
      throw e;
    }
  }

  /** Shows the chosen sweep in the editor; results on screen are cleared (an opened result file's are restored). */
  private pick(choice: string): void {
    this.shown = null;
    if (choice === 'opened') {
      const entry = this.opened;
      if (!entry) return;
      const { sweep, result } = entry;
      if (result) this.shown = { sweep: result.sweep, spec: JSON.stringify(result.sweep), runs: result.runs };
      const form = result ? null : sweepToForm(sweep);
      this.setEditor(form ? this.formView(form, sweep.base, baseNote(sweep.base)) : new FixedPanel(sweep, () => this.validate()));
      this.showResults();
      if (entry.status) this.status.textContent = entry.status;
      return;
    }
    this.showResults();
    if (choice === 'current') {
      const base = this.currentBase();
      const painted = 'config' in base && this.engine.editedLandscapes() !== undefined;
      const note = `${baseNote(base)} from the current world${painted ? ' (painted maps are not included)' : ''}, captured when chosen`;
      this.setEditor(this.formView(defaultForm(this.engine.model), base, note));
      return;
    }
    const builtin = this.builtins.find((b) => choice === `builtin:${b.id}`);
    if (builtin) this.setEditor(new FixedPanel(builtin.sweep, () => this.validate()));
  }

  /** Adds (or replaces) the picker entry for an opened link or file, and selects it (without `pick`). */
  private markOpened(name: string): void {
    this.picker.querySelector('option[value="opened"]')?.remove();
    this.picker.prepend(h('option', { value: 'opened' }, `Opened: ${name}`));
    this.picker.value = 'opened';
  }

  /** The current world as a base: its preset when unmodified, otherwise its config (Decision 17). */
  private currentBase(): SweepBase {
    const e = this.engine;
    return e.presetId !== null && !e.isModified() ? { preset: e.presetId } : { config: structuredClone(e.baseConfig) };
  }

  private configOf(base: SweepBase): ModelConfig | null {
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
        outcome === 'done' ? `${count} runs done` : `Canceled after ${shown.runs.length} of ${count} runs: the results are incomplete`;
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
    this.editorSlot.disabled = on;
    this.openButton.disabled = on;
  }

  /** Re-aggregates the runs on screen with the core and redraws them; exports need at least one run. */
  private showResults(): void {
    const shown = this.shown;
    const empty = !shown || shown.runs.length === 0;
    for (const b of this.outputs) b.disabled = empty;
    this.outputsEl.hidden = empty;
    if (!shown || empty) {
      this.chart.clear();
      this.table.replaceChildren();
      return;
    }
    const summary = JSON.parse(aggregate(shown.spec, JSON.stringify(shown.runs))) as Summary;
    this.chart.draw(chartData(shown.sweep, summary));
    this.table.replaceChildren(resultsTable(summary));
  }

  /**
   * Opens a sweep or a result file. The core checks it (a result's runs
   * included) before anything changes; on any error the view is left as it
   * was and the error is shown. A result's summary is recomputed from its runs (Decision 20).
   */
  private async openFile(): Promise<void> {
    const file = this.fileInput.files?.[0];
    this.fileInput.value = '';
    if (!file || this.pool) return;
    try {
      let json: unknown;
      try {
        json = JSON.parse(await file.text());
      } catch {
        this.status.textContent = `${file.name} is not JSON`;
        return;
      }
      const opened = readOpened(json, OPEN_CORE);
      if (opened.kind === 'error') {
        this.status.textContent = `${file.name}: ${opened.message}`;
        return;
      }
      if (opened.kind === 'sweep') {
        this.showOpened({ sweep: opened.sweep });
        return;
      }
      const { result } = opened;
      const status = `${file.name}: ${result.runs.length} runs${result.incomplete ? ' (incomplete)' : ''}`;
      this.showOpened({ sweep: result.sweep, result, status });
    } catch (e) {
      this.status.textContent = `${file.name} could not be opened: ${messages(parseErrors(e, 'file'))}`;
    }
  }

  private download(kind: 'result' | 'runs' | 'summary'): void {
    const shown = this.shown;
    if (!shown) return;
    const runs = JSON.stringify(shown.runs);
    const name = slug(shown.sweep.name);
    try {
      if (kind === 'result') downloadText(`${name}-result.json`, sweep_result(shown.spec, runs), 'application/json');
      else downloadText(`${name}-${kind}.csv`, sweep_csv(shown.spec, runs, kind));
    } catch (e) {
      this.status.textContent = messages(parseErrors(e));
    }
  }

  private async downloadChart(): Promise<void> {
    const canvas = this.chart.canvas();
    if (this.shown && canvas) downloadBlob(`${slug(this.shown.sweep.name)}-chart.png`, await canvasBlob(canvas));
  }

  /** Puts a `#x=` link to the edited sweep (not its results) in the address bar and the clipboard. */
  private async share(): Promise<void> {
    const sweep = this.editor?.sweep() ?? null;
    if (!sweep) {
      this.status.textContent = 'Fix the sweep before sharing it';
      return;
    }
    history.replaceState(null, '', `#x=${await encodeSweep(sweep)}`);
    try {
      await navigator.clipboard.writeText(location.href);
      this.status.textContent = 'Link copied';
    } catch {
      this.status.textContent = 'Link in the address bar';
    }
  }
}
