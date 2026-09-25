import type { Engine } from '../engine';
import { errorsFor } from '../paths';
import { groupParams, paramEdit, paramInput, type ParamInput } from '../schema-form';
import type { FieldError, ModelKind, Param } from '../types';
import { h } from './dom';

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
  private slots: { path: string; el: HTMLElement }[] = [];
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
    const sections = groupParams(this.engine.schemas[model] ?? []).map(({ group, params }) =>
      h('section', { class: 'group' }, h('h3', {}, group), h('p', { class: 'hint' }, note(params)), ...params.map((p) => this.control(p))),
    );
    this.el.replaceChildren(this.general, ...sections);
    this.renderErrors();
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
    }
    const rest = this.errors.filter((e) => !claimed.has(e));
    this.general.replaceChildren(...rest.map((e) => h('p', {}, `${e.field}: ${e.message}`)));
  }

  private control(p: Param): HTMLElement {
    const slot = h('div', { class: 'error' });
    this.slots.push({ path: p.path, el: slot });
    // A field's one-line explanation (the anasazi's quirks cite the extraction).
    const help = p.help ? h('p', { class: 'hint help' }, p.help) : null;
    const current = () => paramInput(p, this.engine.config);
    const bounds = { min: p.min, max: p.max, step: p.step };
    switch (p.kind) {
      case 'bool': {
        const box = h('input', { type: 'checkbox', onchange: () => void this.commit(p, box.checked) });
        this.syncers.push(() => (box.checked = current() === true));
        return h('div', { class: 'control' }, h('label', { class: 'switch' }, box, ` ${p.label}`), help, slot);
      }
      case 'choice': {
        const select = h(
          'select',
          { onchange: () => void this.commit(p, select.value) },
          ...(p.choices ?? []).map((c) => h('option', { value: c.value }, c.label)),
        );
        this.syncers.push(() => (select.value = String(current())));
        return h('div', { class: 'control' }, h('label', {}, p.label), select, help, slot);
      }
      case 'range': {
        const lo = h('input', { type: 'number', class: 'num', ...bounds });
        const hi = h('input', { type: 'number', class: 'num', ...bounds });
        const apply = (edited: 'min' | 'max') => void this.commit(p, { min: lo.value, max: hi.value, edited });
        lo.addEventListener('change', () => apply('min'));
        hi.addEventListener('change', () => apply('max'));
        this.syncers.push(() => {
          const r = current() as { min: string; max: string };
          lo.value = r.min;
          hi.value = r.max;
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
        slider.addEventListener('input', () => (num.value = slider.value));
        slider.addEventListener('change', () => void this.commit(p, slider.value));
        num.addEventListener('change', () => void this.commit(p, num.value));
        this.syncers.push(() => {
          const v = String(current());
          slider.value = v;
          num.value = v;
        });
        return h('div', { class: 'control' }, h('label', {}, p.label), h('div', { class: 'row' }, slider, num), help, slot);
      }
    }
  }
}
