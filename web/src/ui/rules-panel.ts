import { COMPARE_PRESETS } from '../compare-presets';
import type { Engine } from '../engine';
import { goodsEditorSignature, pollutionEditorSignature } from '../goods';
import { groupsEditorSignature } from '../groups';
import { worldMenu, presetGroups, presetModel, presetOptionLabel, presetReference, presetSubgroups, type MenuKind } from '../models';
import { errorsFor, getPath, setPath } from '../paths';
import { GROUPS, type Control, type Group } from '../schema';
import { scheduleLines } from '../schedule';
import type { Config, FieldError, URange } from '../types';
import { h } from './dom';
import { goodsEditor, type Commit, type Editor } from './goods-editor';
import { groupsEditor } from './groups-editor';
import { pollutionEditor } from './pollution-editor';
import { SchemaPanel } from './schema-panel';

type CustomEditor = NonNullable<Group['custom']>;

const EDITORS: Record<CustomEditor, { build: (c: Config, commit: Commit) => Editor; signature: (c: Config) => string; errors: string }> = {
  goods: { build: goodsEditor, signature: goodsEditorSignature, errors: 'goods' },
  pollution: { build: pollutionEditor, signature: pollutionEditorSignature, errors: 'pollution.pollutants' },
  groups: { build: groupsEditor, signature: groupsEditorSignature, errors: 'culture.groups' },
};

export interface RulesOptions {
  /** A's panel: the presets menu's Compare entries work, getting the entry's id. */
  onCompare?: (id: string) => void;
  /**
   * A's panel: runs before another model's preset loads; resolving false cancels it (Compare pairs
   * one model, so it is left first — Decision 11).
   */
  beforeModelChange?: () => Promise<boolean>;
  /** B's panel in Compare: the model menu is fixed to its world's model. */
  sameModelOnly?: boolean;
}

/**
 * The preset picker (a model menu, then its presets), then the sugarscape's sections (one per rule, from GROUPS)
 * or, for another model, its schema-driven panel.
 */
export class RulesPanel {
  readonly el = h('div', { class: 'rules' });
  private errors: FieldError[] = [];
  /** The preset picker's syncer (every model). */
  private presetSync: () => void = () => {};
  /** The sugarscape sections' syncers. */
  private syncers: (() => void)[] = [];
  /** The Goods editor's and Pollution table's syncers, which painting cannot affect. */
  private editorSyncers: (() => void)[] = [];
  private errorSlots: { path: string; el: HTMLElement; withField: boolean }[] = [];
  private general = h('div', { class: 'error' });
  /** The Minds rules' sections, shown only for a Minds world. */
  private readonly mindsSections: HTMLElement[] = [];
  private readonly sugarBody: HTMLElement;
  private readonly schema: SchemaPanel;

  constructor(
    private engine: Engine,
    private opts: RulesOptions = {},
  ) {
    this.sugarBody = h(
      'div',
      {},
      this.scheduleSection(),
      this.general,
      ...GROUPS.map((g) => {
        const el = this.groupSection(g);
        if (g.minds) this.mindsSections.push(el);
        return el;
      }),
    );
    this.schema = new SchemaPanel(engine);
    this.el.append(this.presetSection(), this.sugarBody, this.schema.el);
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

  /** The model menu's entry for the running world (the Minds, or its model). */
  private menu(): MenuKind {
    return worldMenu(this.engine.config, this.engine.presets.find((p) => p.id === this.engine.presetId));
  }

  private sync(editors = true): void {
    this.presetSync();
    const sugar = this.engine.model === 'sugarscape';
    this.sugarBody.hidden = !sugar;
    this.schema.el.hidden = sugar;
    if (!sugar) {
      this.schema.sync();
      return;
    }
    // The Minds rules belong to the model menu's Minds entry, not to the book's sugarscape.
    const minds = this.menu() === 'minds';
    this.mindsSections.forEach((el) => (el.hidden = !minds));
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

  /**
   * Two menus: the model, then that model's presets (the sugarscape's grouped by chapter). The model
   * menu's last entry, Compare, lists the Compare pairs in the second menu instead. Choosing another
   * model loads its first preset.
   */
  private presetSection(): HTMLElement {
    const COMPARE = 'compare';
    const custom = () => h('option', { value: '', disabled: true }, 'Custom');
    // The model whose presets the second menu lists: the world's, or Compare while browsing its pairs.
    let listed = '';
    const load = async (id: string) => {
      const preset = this.engine.presets.find((p) => p.id === id);
      const leaving = preset !== undefined && presetModel(preset) !== this.engine.model;
      if (leaving && this.opts.beforeModelChange && !(await this.opts.beforeModelChange())) {
        this.sync();
        return;
      }
      this.errors = (await this.engine.loadPreset(id)) ?? [];
      if (this.errors.length > 0) this.sync();
      this.renderErrors();
    };
    const presetSelect = h('select', {
      onchange: async () => {
        if (listed === COMPARE) {
          const id = presetSelect.value;
          // Not a rule system of this world: the menus go back to showing the current one.
          this.sync();
          this.opts.onCompare?.(id);
          return;
        }
        await load(presetSelect.value);
      },
    });
    const list = (model: string) => {
      listed = model;
      if (model === COMPARE) {
        presetSelect.replaceChildren(
          h('option', { value: '', disabled: true }, 'Choose two worlds to compare'),
          ...COMPARE_PRESETS.map((c) => h('option', { value: c.id }, c.label.replace(/ \(Compare\)$/, ''))),
        );
        presetSelect.value = '';
        return;
      }
      presetSelect.replaceChildren(
        custom(),
        ...presetSubgroups(model as MenuKind, this.engine.presets).map((g) => {
          const options = g.presets.map((p) => h('option', { value: p.id }, presetOptionLabel(p)));
          return g.label === null ? options : [h('optgroup', { label: g.label }, ...options)];
        }).flat(),
      );
    };
    const modelSelect = h(
      'select',
      {
        onchange: async () => {
          const model = modelSelect.value;
          const current = this.menu();
          if (model === COMPARE || model === current) {
            list(model);
            if (model === current) this.sync();
            return;
          }
          const first = presetSubgroups(model as MenuKind, this.engine.presets)[0]?.presets[0];
          if (first) await load(first.id);
        },
      },
      ...presetGroups(this.engine.presets).map((g) => h('option', { value: g.model }, g.label)),
      this.opts.onCompare ? h('option', { value: COMPARE }, 'Compare two worlds…') : null,
    );
    // B's panel in Compare keeps its world's model.
    modelSelect.disabled = this.opts.sameModelOnly === true;
    const badge = h('span', { class: 'badge' }, 'modified');
    const reference = h('p', { class: 'hint preset-reference' });
    const desc = h('p', { class: 'hint' });
    this.presetSync = () => {
      const p = this.engine.presets.find((x) => x.id === this.engine.presetId);
      const menu = this.menu();
      modelSelect.value = menu;
      if (listed !== menu) list(menu);
      presetSelect.value = p?.id ?? '';
      badge.hidden = !this.engine.isModified();
      reference.textContent = p ? presetReference(p) : '';
      reference.hidden = p === undefined;
      desc.textContent = p ? p.description : 'Custom configuration.';
    };
    return h(
      'section',
      { class: 'presets' },
      h('label', {}, 'Model'),
      modelSelect,
      h('label', {}, 'Rule system ', badge),
      presetSelect,
      reference,
      desc,
    );
  }

  private scheduleSection(): HTMLElement {
    const list = h('ul', { class: 'schedule' });
    const clear = h('button', { onclick: () => this.commit((c) => (c.schedule = []), false) }, 'Clear schedule');
    const section = h('section', { class: 'group' }, h('h3', {}, 'Schedule'), list, clear, this.errorSlot('schedule'));
    this.syncers.push(() => {
      const lines = scheduleLines(this.engine.sugar);
      section.hidden = lines.length === 0;
      // Outbreaks are listed read-only; the button clears scheduled changes.
      clear.hidden = this.engine.sugar.schedule.length === 0;
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
      this.syncers.push(() => (box.checked = getPath(this.engine.sugar, path) === true));
      header.replaceChildren(h('label', { class: 'switch' }, box, ` ${group.title}`));
    }
    const extras = [group.conditionalNote, ...(group.conditionalNotes ?? [])].flatMap((extra) => {
      if (!extra) return [];
      const el = h('p', { class: 'hint' }, extra.text);
      this.syncers.push(() => (el.hidden = !extra.when(this.engine.sugar)));
      return [el];
    });
    return h(
      'section',
      { class: 'group' },
      header,
      group.enable ? this.errorSlot(group.enable) : null,
      group.note ? h('p', { class: 'hint' }, group.note) : null,
      ...extras,
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
      const config = this.engine.sugar;
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
          onchange: () =>
            this.commit((cfg) => {
              // As for a number control: adjust seeds a missing parent (central.enabled on an older config).
              try {
                setPath(cfg, c.path, box.checked);
              } catch (err) {
                if (!c.adjust) throw err;
                c.adjust(cfg, structuredClone(cfg));
                setPath(cfg, c.path, box.checked);
              }
            }, reset),
        });
        this.syncers.push(() => (box.checked = c.current ? c.current(this.engine.sugar) : getPath(this.engine.sugar, c.path) === true));
        return h('div', { class: 'control' }, h('label', { class: 'switch' }, box, ` ${c.label}`), this.errorSlot(c.path));
      }
      case 'number': {
        const slider = h('input', { type: 'range', min: c.min, max: c.max, step: c.step });
        const num = h('input', { type: 'number', min: c.min, max: c.max, step: c.step, class: 'num' });
        const apply = (v: string) =>
          this.commit((cfg) => {
            const before = structuredClone(cfg);
            // A number control's path may have a missing parent on an older config (e.g.
            // decision.travel with no decision); adjust seeds it, then the write is retried.
            try {
              setPath(cfg, c.path, Number(v));
            } catch (err) {
              if (!c.adjust) throw err;
              c.adjust(cfg, before);
              setPath(cfg, c.path, Number(v));
            }
            c.adjust?.(cfg, before);
          }, reset);
        slider.addEventListener('input', () => (num.value = slider.value));
        slider.addEventListener('change', () => apply(slider.value));
        num.addEventListener('change', () => apply(num.value));
        this.syncers.push(() => {
          const v = String(getPath(this.engine.sugar, c.path) ?? 0);
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
          const r = getPath(this.engine.sugar, c.path) as URange;
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
        this.syncers.push(() => (select.value = c.current(this.engine.sugar)));
        return wrap(select);
      }
    }
  }
}
