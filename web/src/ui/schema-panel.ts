import { scheduleLines } from '../civil';
import type { Engine } from '../engine';
import { errorsFor } from '../paths';
import { describedBy, groupParams, paramEdit, paramInput, paramShown, type ParamInput } from '../schema-form';
import type { CivilConfig, FieldError, ModelKind, Param } from '../types';
import { h } from './dom';

/**
 * The anasazi's data credit: the valley's files are GPL-2.0 and compiled into the WASM, so the page
 * links their notice, which the build serves beside it (vite.config.ts; relative, for Pages).
 */
function valleyCredit(): HTMLElement {
  return h(
    'p',
    { class: 'hint data-credit' },
    'Valley data: Janssen, Artificial Anasazi v1.1.0 (CoMSES, doi:10.25937/krp4-g724), GPL-2.0 — ',
    h('a', { href: 'anasazi-data/NOTICE', target: '_blank', rel: 'noopener' }, 'notice'),
  );
}

/**
 * Whether the user is on `el`: a sync leaves it alone, so a ramp (which sends the config with
 * every snapshot while it runs) cannot overwrite a half-typed number or a slider being dragged.
 */
function focused(el: HTMLElement): boolean {
  return document.activeElement === el;
}

/** Numbers each panel so its controls' ids are unique on the page. */
let panels = 0;

/** A section's note: whether its fields apply to the world as it runs or rebuild it. */
function note(params: Param[]): string {
  return params.every((p) => p.apply === 'live') ? 'These apply to the running world.' : 'Changing these rebuilds the world.';
}

/**
 * The Rules panel of a model other than the sugarscape, built from its schema (Decision 8): one
 * section per group; live fields edit the running world (logged as `setConfig`), reset fields
 * rebuild it from the base config.
 */
export class SchemaPanel {
  readonly el = h('div', { class: 'schema' });
  private model: ModelKind | null = null;
  private syncers: (() => void)[] = [];
  /** Each field's error slot, and the inputs its help and errors describe. */
  private slots: { path: string; el: HTMLElement; inputs: HTMLElement[]; ids: { help: string | null; error: string } }[] = [];
  private readonly idPrefix = `schema-${++panels}`;
  private general = h('div', { class: 'error' });
  private errors: FieldError[] = [];

  constructor(private readonly engine: Engine) {}

  /** Shows the engine's model's fields (rebuilt when the model changes) with the live config's values. */
  sync(): void {
    const model = this.engine.model;
    if (model !== this.model) this.build(model);
    this.syncers.forEach((s) => s());
  }

  private build(model: ModelKind): void {
    this.model = model;
    this.syncers = [];
    this.slots = [];
    this.errors = [];
    const sections = groupParams(this.engine.schemas[model] ?? []).map(({ group, params }) => {
      const section = h('section', { class: 'group' }, h('h3', {}, group), h('p', { class: 'hint' }, note(params)), ...params.map((p) => this.control(p)));
      if (params.every((p) => p.show_if)) this.syncers.push(() => (section.hidden = !params.some((p) => paramShown(p, this.engine.config))));
      return section;
    });
    const extra = model === 'anasazi' ? [valleyCredit()] : [];
    const schedule = model === 'civil' ? [this.schedule()] : [];
    this.el.replaceChildren(...extra, this.general, ...sections, ...schedule);
    this.renderErrors();
  }

  /** Civil violence's schedule and ramps, read-only (they travel in links and sessions). */
  private schedule(): HTMLElement {
    const list = h('ul', { class: 'hint' });
    const section = h('section', { class: 'group' }, h('h3', {}, 'Schedule'), h('p', { class: 'hint' }, 'Set by the preset; these change the running world at their ticks. While a ramp runs, its field follows the ramp: a change to it (or a scheduled one) lasts only until the next tick.'), list);
    this.syncers.push(() => {
      const lines = scheduleLines(this.engine.config as CivilConfig);
      section.hidden = lines.length === 0;
      list.replaceChildren(...lines.map((l) => h('li', {}, l)));
    });
    return section;
  }

  private async commit(p: Param, input: ParamInput): Promise<void> {
    const edit = paramEdit(p, input);
    const errors = p.apply === 'live' ? await this.engine.applyModelConfig(edit) : await this.engine.resetModelWith(edit);
    this.errors = errors ?? [];
    if (this.errors.length > 0) this.sync();
    this.renderErrors();
  }

  private renderErrors(): void {
    const claimed = new Set<FieldError>();
    for (const slot of this.slots) {
      const mine = errorsFor(this.errors, slot.path);
      mine.forEach((e) => claimed.add(e));
      slot.el.replaceChildren(...mine.map((e) => h('p', {}, e.message)));
      const described = describedBy(slot.ids, mine.length > 0);
      for (const input of slot.inputs) {
        if (described) input.setAttribute('aria-describedby', described);
        else input.removeAttribute('aria-describedby');
      }
    }
    const rest = this.errors.filter((e) => !claimed.has(e));
    this.general.replaceChildren(...rest.map((e) => h('p', {}, `${e.field}: ${e.message}`)));
  }

  private control(p: Param): HTMLElement {
    const el = this.controlBody(p);
    if (p.show_if) this.syncers.push(() => (el.hidden = !paramShown(p, this.engine.config)));
    return el;
  }

  private controlBody(p: Param): HTMLElement {
    const id = `${this.idPrefix}-${p.path.replace(/[^A-Za-z0-9_-]/g, '-')}`;
    const ids = { help: p.help ? `${id}-help` : null, error: `${id}-error` };
    const slot = h('div', { class: 'error', id: ids.error });
    const inputs: HTMLElement[] = [];
    this.slots.push({ path: p.path, el: slot, inputs, ids });
    // A field's one-line explanation (the anasazi's quirks cite the extraction).
    const help = p.help ? h('p', { class: 'hint help', id: ids.help }, p.help) : null;
    const current = () => paramInput(p, this.engine.config);
    const bounds = { min: p.min, max: p.max, step: p.step };
    switch (p.kind) {
      case 'bool': {
        const box = h('input', { type: 'checkbox', onchange: () => void this.commit(p, box.checked) });
        inputs.push(box);
        this.syncers.push(() => (box.checked = current() === true));
        return h('div', { class: 'control' }, h('label', { class: 'switch' }, box, ` ${p.label}`), help, slot);
      }
      case 'choice': {
        const select = h(
          'select',
          { onchange: () => void this.commit(p, select.value) },
          ...(p.choices ?? []).map((c) => h('option', { value: c.value }, c.label)),
        );
        inputs.push(select);
        this.syncers.push(() => (select.value = String(current())));
        return h('div', { class: 'control' }, h('label', {}, p.label), select, help, slot);
      }
      case 'range': {
        const lo = h('input', { type: 'number', class: 'num', ...bounds });
        const hi = h('input', { type: 'number', class: 'num', ...bounds });
        const apply = (edited: 'min' | 'max') => void this.commit(p, { min: lo.value, max: hi.value, edited });
        inputs.push(lo, hi);
        lo.addEventListener('change', () => apply('min'));
        hi.addEventListener('change', () => apply('max'));
        this.syncers.push(() => {
          const r = current() as { min: string; max: string };
          if (!focused(lo)) lo.value = r.min;
          if (!focused(hi)) hi.value = r.max;
        });
        return h(
          'div',
          { class: 'control' },
          h('label', {}, p.label),
          h('div', { class: 'row' }, lo, h('span', { class: 'hint' }, 'to'), hi),
          help,
          slot,
        );
      }
      default: {
        const slider = h('input', { type: 'range', ...bounds });
        const num = h('input', { type: 'number', class: 'num', ...bounds });
        inputs.push(slider, num);
        slider.addEventListener('input', () => (num.value = slider.value));
        slider.addEventListener('change', () => void this.commit(p, slider.value));
        num.addEventListener('change', () => void this.commit(p, num.value));
        this.syncers.push(() => {
          const v = String(current());
          if (!focused(slider)) slider.value = v;
          if (!focused(num)) num.value = v;
        });
        return h('div', { class: 'control' }, h('label', {}, p.label), h('div', { class: 'row' }, slider, num), help, slot);
      }
    }
  }
}
