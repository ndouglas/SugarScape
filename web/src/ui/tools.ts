import type { Engine, PlaceOverrides } from '../engine';
import { capacitiesFromPixels } from '../image';
import type { DiseaseEntry } from '../types';
import { DiseaseListPoll, diseaseOptions, PickerWorldGate } from './disease-picker';
import { h } from './dom';
import type { CellEvent, GridView } from './grid-view';
import { readImagePixels } from './image-import';

type Tool = 'inspect' | 'paint' | 'place' | 'erase' | 'infect' | 'vaccinate';

const TOOLS: [Tool, string][] = [
  ['inspect', 'Inspect'],
  ['paint', 'Paint capacity'],
  ['place', 'Place agent'],
  ['erase', 'Erase agent'],
  ['infect', 'Infect'],
  ['vaccinate', 'Vaccinate'],
];

/** Tools that exist only while disease is on (in some world on screen). */
const DISEASE_TOOLS: Tool[] = ['infect', 'vaccinate'];

/** A world's disease list is fetched at most this often while a disease tool is open. */
const LIST_MS = 250;

/** A grid and the world its clicks edit. */
export interface ToolTarget { engine: Engine; grid: GridView }

export interface Tools {
  readonly el: HTMLElement;
  /** Routes `target.grid`'s clicks to `target.engine` (Compare's B); returns the detach. */
  attach(target: ToolTarget): () => void;
  /** The world whose diseases the picker lists and whose map an image import sets (the grid last clicked). */
  focus(engine: Engine): void;
}

/**
 * Tool picker; routes each grid's clicks/drags to the active tool on that grid's world (Decision 10).
 * Edit errors (e.g. an occupied site) are ignored. Display changes (the paint layer, the disease
 * colours) and the paint tool's goods follow `primary`, whose display Compare mirrors to B.
 */
export function buildTools(primary: ToolTarget, onInspect: (engine: Engine) => void): Tools {
  let tool: Tool = 'inspect';
  let radius = 1;
  let value = 4;
  let good = 0;
  let sex: '' | 'female' | 'male' = '';
  let tribe: '' | 'blue' | 'red' = '';
  /** Selected disease id; −1 is a new random disease (Infect only). */
  let disease = -1;
  /** Length of the disease list the picker was last filled from. */
  let known = -1;
  const targets: ToolTarget[] = [];
  /** Each world's latest disease list (while a disease tool is open) and when to ask again. */
  const lists = new Map<Engine, { diseases: DiseaseEntry[]; poll: DiseaseListPoll }>();
  let focused = primary.engine;
  const gate = new PickerWorldGate();
  const diseases = (): DiseaseEntry[] => lists.get(focused)?.diseases ?? [];
  const brushed = () => tool === 'paint' || tool === 'vaccinate';
  const diseaseOn = () => targets.some((t) => t.engine.model === 'sugarscape' && t.engine.sugar.disease.enabled);
  const grids = () => targets.map((t) => t.grid);

  const buttons = TOOLS.map(([t, label]) => h('button', { onclick: () => choose(t) }, label));
  const options = h('div', { class: 'tool-options' });
  const picker = h('select', { onchange: () => (disease = Number(picker.value)) });

  const number = (label: string, min: number, max: number, get: () => number, set: (v: number) => void) => {
    const input = h('input', { type: 'number', min, max, value: get(), class: 'num' });
    input.addEventListener('change', () => {
      set(Math.min(max, Math.max(min, Number(input.value))));
      input.value = String(get());
      if (brushed()) for (const g of grids()) g.brushRadius = radius;
    });
    return h('label', {}, `${label} `, input);
  };
  /** The paint tool's good picker, refilled only when the goods' names change. */
  const goodSelect = h('select', {
    onchange: () => {
      good = Number(goodSelect.value);
      primary.engine.setDisplay({ layer: `capacity:${good}` });
    },
  });
  const goodLabel = h('label', {}, 'Good ', goodSelect);
  let goodNames = '';
  function refreshGoods(): void {
    const names = primary.engine.sugar.goods.map((g) => g.name);
    const signature = JSON.stringify(names);
    if (good >= names.length) good = 0;
    if (signature !== goodNames) {
      goodNames = signature;
      goodSelect.replaceChildren(...names.map((name, i) => h('option', { value: String(i) }, name)));
    }
    goodSelect.value = String(good);
  }

  /** Image import (Decision 12 of milestone 6): for the paint tool's good, into the focused world. */
  let importMax = 4;
  let invert = false;
  const importStatus = h('span', { class: 'hint', 'aria-live': 'polite' });
  const fileInput = h('input', { type: 'file', accept: 'image/*', hidden: true, 'aria-label': 'Image to import' });
  fileInput.addEventListener('change', async () => {
    const file = fileInput.files?.[0];
    fileInput.value = '';
    if (!file) return;
    const engine = focused;
    try {
      const { width, height } = engine.size();
      const capacities = capacitiesFromPixels(await readImagePixels(file, width, height), importMax, invert);
      const errors = await engine.importLandscape(good, capacities);
      importStatus.textContent = errors ? errors.map((e) => e.message).join('; ') : `Imported ${file.name}`;
    } catch (e) {
      importStatus.textContent = `Could not read ${file.name}: ${e instanceof Error ? e.message : String(e)}`;
    }
  });
  const invertBox = h('input', { type: 'checkbox', onchange: () => (invert = invertBox.checked) });
  const importControls = h(
    'span',
    { class: 'tool-options' },
    h('button', { onclick: () => fileInput.click(), title: 'Set the good’s capacities from an image’s brightness' }, 'Import image…'),
    fileInput,
    number('Max', 0, 10, () => importMax, (v) => (importMax = Math.round(v))),
    h('label', {}, invertBox, ' Invert'),
    importStatus,
  );

  const select = <T extends string>(label: string, values: [T, string][], set: (v: T) => void) => {
    const s = h('select', {}, ...values.map(([v, l]) => h('option', { value: v }, l)));
    s.addEventListener('change', () => set(s.value as T));
    return h('label', {}, `${label} `, s);
  };

  /** Refills the picker when the focused world's disease list has grown, or when forced. */
  function refreshPicker(force = false): void {
    if (!DISEASE_TOOLS.includes(tool) || !diseaseOn()) return;
    const list = diseases();
    if (!force && list.length === known) return;
    known = list.length;
    picker.replaceChildren(...diseaseOptions(list, tool === 'infect').map(([v, l]) => h('option', { value: v }, l)));
    const values = Array.from(picker.options, (o) => Number(o.value));
    if (!values.includes(disease)) disease = values[0] ?? -1;
    picker.value = String(disease);
  }

  function choose(next: Tool): void {
    tool = next;
    buttons.forEach((b, i) => b.setAttribute('aria-pressed', String(TOOLS[i][0] === tool)));
    for (const g of grids()) g.brushRadius = brushed() ? radius : null;
    if (tool === 'paint') {
      refreshGoods();
      primary.engine.setDisplay({ layer: `capacity:${good}` });
    }
    if (DISEASE_TOOLS.includes(tool)) primary.engine.setDisplay({ colorMode: 'disease' });
    const pick = h('label', {}, 'Disease ', picker);
    options.replaceChildren(
      ...(tool === 'paint'
        ? [goodLabel, number('Radius', 0, 10, () => radius, (v) => (radius = v)), number('Capacity', 0, 10, () => value, (v) => (value = v)), importControls]
        : tool === 'place'
          ? [
              select('Sex', [['', 'Random'], ['female', 'Female'], ['male', 'Male']], (v) => (sex = v)),
              select('Tribe', [['', 'Random'], ['blue', 'Blue'], ['red', 'Red']], (v) => (tribe = v)),
            ]
          : tool === 'infect'
            ? [pick, h('span', { class: 'hint' }, 'Click an agent to infect it.')]
            : tool === 'vaccinate'
              ? [number('Radius', 0, 10, () => radius, (v) => (radius = v)), pick]
              : tool === 'inspect'
                ? [h('span', { class: 'hint' }, 'Click an agent or site.')]
                : [h('span', { class: 'hint' }, 'Click or drag over agents to remove them.')]),
    );
    refreshPicker(true);
    if (DISEASE_TOOLS.includes(tool)) {
      // Fetch the lists now, even while paused.
      for (const t of targets) {
        lists.get(t.engine)?.poll.expedite();
        void t.engine.refresh();
      }
    }
    for (const g of grids()) g.draw();
  }

  /**
   * Shows only Inspect while no world on screen is a sugarscape (the other models have no editing
   * tools, Decision 13), and hides the disease tools while no world on screen has disease (leaving
   * a tool that went away for Inspect).
   */
  function syncAvailability(): void {
    const sugar = targets.some((t) => t.engine.model === 'sugarscape');
    const on = diseaseOn();
    buttons.forEach((b, i) => {
      const t = TOOLS[i][0];
      b.hidden = t !== 'inspect' && (!sugar || (DISEASE_TOOLS.includes(t) && !on));
    });
    if (tool !== 'inspect' && (!sugar || (!on && DISEASE_TOOLS.includes(tool)))) choose('inspect');
    else refreshPicker(true);
    // Keeps the paint tool's layer and inputs; only the good list follows the config.
    refreshGoods();
  }

  /** A grid's clicks and drags, on its own world. */
  const route = (engine: Engine) => (x: number, y: number, kind: CellEvent) => {
    // The picker's disease id belongs to the focused world's list, and a grid's own pointerdown runs
    // before Compare's focus listener: a press on the other grid only switches the picker to its world.
    if (DISEASE_TOOLS.includes(tool) && !gate.allows(kind, engine === focused)) {
      if (kind === 'down') focus(engine);
      return;
    }
    switch (tool) {
      case 'inspect':
        if (kind === 'down') {
          void engine.select(x, y);
          onInspect(engine);
        }
        break;
      case 'paint':
        void engine.paint(x, y, radius, value, good);
        break;
      case 'place':
        if (kind === 'down') {
          const o: PlaceOverrides = {};
          if (sex) o.sex = sex;
          if (tribe) o.tribe = tribe;
          void engine.place(x, y, o);
        }
        break;
      case 'erase':
        void engine.erase(x, y);
        break;
      case 'infect':
        if (kind === 'down') void engine.infect(x, y, disease);
        break;
      case 'vaccinate':
        void engine.vaccinate(x, y, radius, disease);
        break;
    }
  };

  function focus(engine: Engine): void {
    if (focused === engine) return;
    focused = engine;
    refreshPicker(true);
  }

  function attach(target: ToolTarget): () => void {
    const { engine, grid } = target;
    const entry = { diseases: [] as DiseaseEntry[], poll: new DiseaseListPoll(LIST_MS) };
    targets.push(target);
    lists.set(engine, entry);
    grid.onCell = route(engine);
    grid.brushRadius = brushed() ? radius : null;
    const offs = [
      engine.on('reset', syncAvailability),
      engine.on('config', syncAvailability),
      // A reset can change which diseases exist: drop the stale list and ask for a fresh one promptly.
      engine.on('reset', () => {
        entry.poll.expedite();
        entry.diseases = [];
        if (engine === focused) refreshPicker();
      }),
      // Infect, Vaccinate and a rule change can change the list without a tick.
      engine.on('edit', () => entry.poll.invalidate()),
      engine.on('config', () => entry.poll.invalidate()),
      engine.want((now) =>
        DISEASE_TOOLS.includes(tool) && engine.model === 'sugarscape' && engine.sugar.disease.enabled && entry.poll.due(now, engine.tick)
          ? { diseaseList: true }
          : {},
      ),
      engine.on('snapshot', () => {
        const list = engine.last?.diseaseList;
        if (!list) return;
        entry.diseases = list;
        entry.poll.received(performance.now(), engine.tick);
        if (engine === focused) refreshPicker();
      }),
    ];
    syncAvailability();
    return () => {
      for (const off of offs) off();
      targets.splice(targets.indexOf(target), 1);
      lists.delete(engine);
      grid.onCell = null;
      grid.brushRadius = null;
      if (focused === engine) focus(primary.engine);
      syncAvailability();
    };
  }

  const el = h('div', { class: 'tools' }, h('div', { class: 'tool-buttons' }, ...buttons), options);
  attach(primary);
  choose('inspect');
  syncAvailability();
  return { el, attach, focus };
}
