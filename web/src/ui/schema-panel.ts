import { DEMOCRATIC_PEACE_RULES, democraticPeaceParams } from '../democratic-peace';
import { GEOSIM_RULES, geosimParams } from '../geosim';
import { scheduleLines } from '../civil';
import type { Engine } from '../engine';
import { errorsFor } from '../paths';
import { describedBy, groupParams, paramEdit, paramInput, paramShown, paramSlider, schemaControlLabel, type ParamInput } from '../schema-form';
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
    const params = this.engine.schemas[model] ?? [];
    const sections = groupParams(model === 'democratic_peace' ? democraticPeaceParams(params) : model === 'geosim' ? geosimParams(params) : params).map(({ group, params }) => {
      const section = h('section', { class: 'group' }, h('h3', {}, group), h('p', { class: 'hint' }, note(params)), ...params.map((p) => this.control(p)));
      if (params.every((p) => p.show_if)) this.syncers.push(() => (section.hidden = !params.some((p) => paramShown(p, this.engine.config))));
      return section;
    });
    const extra = model === 'anasazi' ? [valleyCredit()] : model === 'geosim' ? [h('p', { class: 'hint' }, GEOSIM_RULES)] : model === 'democratic_peace' ? [h('p', { class: 'hint' }, DEMOCRATIC_PEACE_RULES)] : [];
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
    const labelId = `${id}-label`;
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
        const selectId = `${id}-choice`;
        const select = h(
          'select',
          { id: selectId, 'aria-labelledby': labelId, onchange: () => void this.commit(p, select.value) },
          ...(p.choices ?? []).map((c) => h('option', { value: c.value }, c.label)),
        );
        inputs.push(select);
        this.syncers.push(() => (select.value = String(current())));
        return h('div', { class: 'control' }, h('label', { id: labelId, htmlFor: selectId }, schemaControlLabel(p.label)), select, help, slot);
      }
      case 'range': {
        const loId = `${id}-minimum`;
        const hiId = `${id}-maximum`;
        const loNameId = `${loId}-name`;
        const hiNameId = `${hiId}-name`;
        const lo = h('input', { id: loId, type: 'number', class: 'num', 'aria-labelledby': `${labelId} ${loNameId}`, ...bounds });
        const hi = h('input', { id: hiId, type: 'number', class: 'num', 'aria-labelledby': `${labelId} ${hiNameId}`, ...bounds });
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
          h('label', { id: labelId, htmlFor: loId }, schemaControlLabel(p.label)),
          h('div', { class: 'row' }, lo, h('span', { id: loNameId, hidden: true }, schemaControlLabel('', 'minimum')), h('span', { class: 'hint' }, 'to'), hi, h('span', { id: hiNameId, hidden: true }, schemaControlLabel('', 'maximum'))),
          help,
          slot,
        );
      }
      default: {
        const sliderId = `${id}-slider`;
        const numberId = `${id}-number`;
        const sliderNameId = `${sliderId}-name`;
        const numberNameId = `${numberId}-name`;
        const slider = h('input', { id: sliderId, type: 'range', 'aria-labelledby': `${labelId} ${sliderNameId}`, ...bounds });
        const num = h('input', { id: numberId, type: 'number', class: 'num', 'aria-labelledby': `${labelId} ${numberNameId}`, ...bounds });
        inputs.push(slider, num);
        slider.addEventListener('input', () => (num.value = slider.value));
        slider.addEventListener('change', () => void this.commit(p, slider.value));
        num.addEventListener('change', () => void this.commit(p, num.value));
        this.syncers.push(() => {
          if (!focused(num)) num.value = String(current());
          // A null nullable field (the ethnocentrism model's tag mutation) shows its slider at the
          // value it falls back to, not the browser's own midpoint default for an empty value.
          if (!focused(slider)) slider.value = paramSlider(p, this.engine.config);
        });
        return h(
          'div',
          { class: 'control' },
          h('label', { id: labelId, htmlFor: sliderId }, schemaControlLabel(p.label)),
          h('div', { class: 'row' }, slider, h('span', { id: sliderNameId, hidden: true }, schemaControlLabel('', 'slider')), num, h('span', { id: numberNameId, hidden: true }, schemaControlLabel('', 'number'))),
          help,
          slot,
        );
      }
    }
  }
}
