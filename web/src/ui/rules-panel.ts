import type { Engine } from '../engine';
import { errorsFor, getPath, setPath } from '../paths';
import { GROUPS, type Control } from '../schema';
import type { Config, FieldError, URange } from '../types';
import { h } from './dom';

/** Preset picker plus one section per rule, generated from GROUPS. */
export class RulesPanel {
  readonly el = h('div', { class: 'rules' });
  private errors: FieldError[] = [];
  private syncers: (() => void)[] = [];
  private errorSlots: { path: string; el: HTMLElement }[] = [];
  private general = h('div', { class: 'error' });

  constructor(private engine: Engine) {
    this.el.append(this.presetSection(), this.scheduleSection(), this.general, ...GROUPS.map((g) => this.groupSection(g)));
    engine.on('reset', () => this.sync());
    engine.on('config', () => this.sync());
    // Painting changes the landscape, which counts as a modification.
    engine.on('edit', () => this.sync());
    this.sync();
  }

  private commit(mutate: (c: Config) => void, reset: boolean): void {
    const next = structuredClone(this.engine.config);
    mutate(next);
    this.errors = (reset ? this.engine.reset(next) : this.engine.applyConfig(next)) ?? [];
    if (this.errors.length > 0) this.sync();
    this.renderErrors();
  }

  private sync(): void {
    this.syncers.forEach((s) => s());
  }

  private renderErrors(): void {
    const claimed = new Set<FieldError>();
    for (const slot of this.errorSlots) {
      const mine = errorsFor(this.errors, slot.path);
      mine.forEach((e) => claimed.add(e));
      slot.el.replaceChildren(...mine.map((e) => h('p', {}, e.message)));
    }
    const rest = this.errors.filter((e) => !claimed.has(e));
    this.general.replaceChildren(...rest.map((e) => h('p', {}, `${e.field}: ${e.message}`)));
  }

  private errorSlot(path: string): HTMLElement {
    const el = h('div', { class: 'error' });
    this.errorSlots.push({ path, el });
    return el;
  }

  private presetSection(): HTMLElement {
    const select = h(
      'select',
      {
        onchange: () => {
          this.errors = this.engine.loadPreset(select.value) ?? [];
          if (this.errors.length > 0) this.sync();
          this.renderErrors();
        },
      },
      h('option', { value: '', disabled: true }, 'Custom'),
      ...this.engine.presets.map((p) => h('option', { value: p.id }, `${p.name} — ${p.source}`)),
    );
    const badge = h('span', { class: 'badge' }, 'modified');
    const desc = h('p', { class: 'hint' });
    this.syncers.push(() => {
      const p = this.engine.presets.find((x) => x.id === this.engine.presetId);
      select.value = p?.id ?? '';
      badge.hidden = !this.engine.isModified();
      desc.textContent = p ? p.description : 'Custom configuration.';
    });
    return h('section', { class: 'presets' }, h('label', {}, 'Rule system ', badge), select, desc);
  }

  private scheduleSection(): HTMLElement {
    const list = h('ul', { class: 'schedule' });
    const clear = h('button', { onclick: () => this.commit((c) => (c.schedule = []), false) }, 'Clear schedule');
    const section = h('section', { class: 'group' }, h('h3', {}, 'Schedule'), list, clear, this.errorSlot('schedule'));
    this.syncers.push(() => {
      const entries = [...this.engine.config.schedule].sort((a, b) => a.tick - b.tick);
      section.hidden = entries.length === 0;
      list.replaceChildren(
        ...entries.flatMap((e) =>
          Object.entries(e.set).map(([path, value]) => h('li', {}, `t = ${e.tick} · ${path} = ${JSON.stringify(value)}`)),
        ),
      );
    });
    return section;
  }

  private groupSection(group: (typeof GROUPS)[number]): HTMLElement {
    const header = h('h3', {}, group.title);
    if (group.enable) {
      const path = group.enable;
      const box = h('input', {
        type: 'checkbox',
        onchange: () => this.commit((c) => setPath(c, path, box.checked), false),
      });
      this.syncers.push(() => (box.checked = getPath(this.engine.config, path) === true));
      header.replaceChildren(h('label', { class: 'switch' }, box, ` ${group.title}`));
    }
    return h(
      'section',
      { class: 'group' },
      header,
      group.enable ? this.errorSlot(group.enable) : null,
      group.note ? h('p', { class: 'hint' }, group.note) : null,
      ...group.controls.map((c) => this.control(c)),
    );
  }

  private control(c: Control): HTMLElement {
    const reset = c.reset === true;
    const wrap = (input: HTMLElement) =>
      h('div', { class: 'control' }, h('label', {}, c.label), input, this.errorSlot(c.path));
    switch (c.kind) {
      case 'toggle': {
        const box = h('input', {
          type: 'checkbox',
          onchange: () => this.commit((cfg) => setPath(cfg, c.path, box.checked), reset),
        });
        this.syncers.push(() => (box.checked = getPath(this.engine.config, c.path) === true));
        return h('div', { class: 'control' }, h('label', { class: 'switch' }, box, ` ${c.label}`), this.errorSlot(c.path));
      }
      case 'number': {
        const slider = h('input', { type: 'range', min: c.min, max: c.max, step: c.step });
        const num = h('input', { type: 'number', min: c.min, max: c.max, step: c.step, class: 'num' });
        const apply = (v: string) => this.commit((cfg) => setPath(cfg, c.path, Number(v)), reset);
        slider.addEventListener('input', () => (num.value = slider.value));
        slider.addEventListener('change', () => apply(slider.value));
        num.addEventListener('change', () => apply(num.value));
        this.syncers.push(() => {
          const v = String(getPath(this.engine.config, c.path));
          slider.value = v;
          num.value = v;
        });
        return wrap(h('div', { class: 'row' }, slider, num));
      }
      case 'range': {
        const lo = h('input', { type: 'number', min: c.min, max: c.max, class: 'num' });
        const hi = h('input', { type: 'number', min: c.min, max: c.max, class: 'num' });
        const apply = () =>
          this.commit((cfg) => setPath(cfg, c.path, { min: Number(lo.value), max: Number(hi.value) } satisfies URange), reset);
        lo.addEventListener('change', apply);
        hi.addEventListener('change', apply);
        this.syncers.push(() => {
          const r = getPath(this.engine.config, c.path) as URange;
          lo.value = String(r.min);
          hi.value = String(r.max);
        });
        return wrap(h('div', { class: 'row' }, lo, h('span', { class: 'hint' }, 'to'), hi));
      }
      case 'select': {
        const select = h(
          'select',
          {
            onchange: () => {
              const opt = c.options.find((o) => o.value === select.value);
              if (opt) this.commit(opt.apply, reset);
            },
          },
          ...c.options.map((o) => h('option', { value: o.value }, o.label)),
        );
        this.syncers.push(() => (select.value = c.current(this.engine.config)));
        return wrap(select);
      }
    }
  }
}
