import { COMPARE_PRESETS } from '../compare-presets';
import type { Engine } from '../engine';
import { goodsEditorSignature, pollutionEditorSignature } from '../goods';
import { groupsEditorSignature } from '../groups';
import { errorsFor, getPath, setPath } from '../paths';
import { GROUPS, type Control, type Group } from '../schema';
import { scheduleLines } from '../schedule';
import type { Config, FieldError, URange } from '../types';
import { h } from './dom';
import { goodsEditor, type Commit, type Editor } from './goods-editor';
import { groupsEditor } from './groups-editor';
import { pollutionEditor } from './pollution-editor';

type CustomEditor = NonNullable<Group['custom']>;

const EDITORS: Record<CustomEditor, { build: (c: Config, commit: Commit) => Editor; signature: (c: Config) => string; errors: string }> = {
  goods: { build: goodsEditor, signature: goodsEditorSignature, errors: 'goods' },
  pollution: { build: pollutionEditor, signature: pollutionEditorSignature, errors: 'pollution.pollutants' },
  groups: { build: groupsEditor, signature: groupsEditorSignature, errors: 'culture.groups' },
};

/** Preset picker plus one section per rule, generated from GROUPS. */
export class RulesPanel {
  readonly el = h('div', { class: 'rules' });
  private errors: FieldError[] = [];
  private syncers: (() => void)[] = [];
  /** The Goods editor's and Pollution table's syncers, which painting cannot affect. */
  private editorSyncers: (() => void)[] = [];
  private errorSlots: { path: string; el: HTMLElement; withField: boolean }[] = [];
  private general = h('div', { class: 'error' });

  /** `onCompare` (A's panel only) makes the Compare entries of the presets menu work: it gets the entry's id. */
  constructor(
    private engine: Engine,
    private onCompare?: (id: string) => void,
  ) {
    this.el.append(this.presetSection(), this.scheduleSection(), this.general, ...GROUPS.map((g) => this.groupSection(g)));
    engine.on('reset', () => this.sync());
    engine.on('config', () => this.sync());
    // Painting changes the landscape, which counts as a modification.
    engine.on('edit', () => this.sync(false));
    this.sync();
  }

  /**
   * Reset-required changes rebuild from the base setup; others edit the running world. The engine
   * applies `mutate` when the write runs, so a quick second edit builds on the first.
   */
  private async commit(mutate: (c: Config) => void, reset: boolean): Promise<void> {
    if (reset) {
      this.errors = (await this.engine.resetWith(mutate)) ?? [];
    } else {
      this.errors = (await this.engine.applyConfig(mutate)) ?? [];
    }
    if (this.errors.length > 0) this.sync();
    this.renderErrors();
  }

  private sync(editors = true): void {
    this.syncers.forEach((s) => s());
    if (editors) this.editorSyncers.forEach((s) => s());
  }

  private renderErrors(): void {
    const claimed = new Set<FieldError>();
    for (const slot of this.errorSlots) {
      const mine = errorsFor(this.errors, slot.path);
      mine.forEach((e) => claimed.add(e));
      slot.el.replaceChildren(...mine.map((e) => h('p', {}, slot.withField ? `${e.field}: ${e.message}` : e.message)));
    }
    const rest = this.errors.filter((e) => !claimed.has(e));
    this.general.replaceChildren(...rest.map((e) => h('p', {}, `${e.field}: ${e.message}`)));
  }

  private errorSlot(path: string, withField = false): HTMLElement {
    const el = h('div', { class: 'error' });
    this.errorSlots.push({ path, el, withField });
    return el;
  }

  private presetSection(): HTMLElement {
    const select = h(
      'select',
      {
        onchange: async () => {
          const compare = COMPARE_PRESETS.find((c) => `compare:${c.id}` === select.value);
          if (compare) {
            // Not a rule system of this world: the menu goes back to showing the current one.
            this.sync();
            this.onCompare?.(compare.id);
            return;
          }
          this.errors = (await this.engine.loadPreset(select.value)) ?? [];
          if (this.errors.length > 0) this.sync();
          this.renderErrors();
        },
      },
      h('option', { value: '', disabled: true }, 'Custom'),
      ...this.engine.presets.map((p) => h('option', { value: p.id }, `${p.name} — ${p.source}`)),
      this.onCompare
        ? h('optgroup', { label: 'Compare' }, ...COMPARE_PRESETS.map((c) => h('option', { value: `compare:${c.id}` }, c.label)))
        : null,
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
      const lines = scheduleLines(this.engine.config);
      section.hidden = lines.length === 0;
      // Outbreaks are listed read-only; the button clears scheduled changes.
      clear.hidden = this.engine.config.schedule.length === 0;
      list.replaceChildren(...lines.map((line) => h('li', {}, line)));
    });
    return section;
  }

  private groupSection(group: (typeof GROUPS)[number]): HTMLElement {
    const header = h('h3', {}, group.title);
    if (group.enable) {
      const path = group.enable;
      const box = h('input', {
        type: 'checkbox',
        onchange: () => this.commit((c) => setPath(c, path, box.checked), group.enableResets === true),
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
      ...(group.custom ? this.customEditor(group.custom) : []),
    );
  }

  /**
   * Rebuilds a custom editor only when its structure changes; otherwise refreshes its values in
   * place, so focus and half-typed input survive live and scheduled edits.
   */
  private customEditor(kind: CustomEditor): HTMLElement[] {
    const holder = h('div');
    const commit: Commit = (mutate, reset) => this.commit(mutate, reset);
    const { build, signature, errors } = EDITORS[kind];
    let built: string | null = null;
    let current: Editor | null = null;
    this.editorSyncers.push(() => {
      const config = this.engine.config;
      const next = signature(config);
      if (current && next === built) {
        current.sync(config);
        return;
      }
      built = next;
      current = build(config, commit);
      holder.replaceChildren(current.el);
    });
    return [holder, this.errorSlot(errors, true)];
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
        const apply = (v: string) =>
          this.commit((cfg) => {
            const before = structuredClone(cfg);
            setPath(cfg, c.path, Number(v));
            c.adjust?.(cfg, before);
          }, reset);
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
