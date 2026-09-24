import type { Engine, PlaceOverrides } from '../engine';
import { capacitiesFromPixels } from '../image';
import { diseaseOptions } from './disease-picker';
import { h } from './dom';
import type { GridView } from './grid-view';
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

/** Tools that exist only while disease is on. */
const DISEASE_TOOLS: Tool[] = ['infect', 'vaccinate'];

/** Tool picker; routes grid clicks/drags to the active tool. Edit errors (e.g. occupied site) are ignored. */
export function buildTools(engine: Engine, grid: GridView, onInspect: () => void): HTMLElement {
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
  const brushed = () => tool === 'paint' || tool === 'vaccinate';

  const buttons = TOOLS.map(([t, label]) => h('button', { onclick: () => choose(t) }, label));
  const options = h('div', { class: 'tool-options' });
  const picker = h('select', { onchange: () => (disease = Number(picker.value)) });

  const number = (label: string, min: number, max: number, get: () => number, set: (v: number) => void) => {
    const input = h('input', { type: 'number', min, max, value: get(), class: 'num' });
    input.addEventListener('change', () => {
      set(Math.min(max, Math.max(min, Number(input.value))));
      input.value = String(get());
      if (brushed()) grid.brushRadius = radius;
    });
    return h('label', {}, `${label} `, input);
  };
  /** The paint tool's good picker, refilled only when the goods' names change. */
  const goodSelect = h('select', {
    onchange: () => {
      good = Number(goodSelect.value);
      engine.setDisplay({ layer: `capacity:${good}` });
    },
  });
  const goodLabel = h('label', {}, 'Good ', goodSelect);
  let goodNames = '';
  function refreshGoods(): void {
    const names = engine.config.goods.map((g) => g.name);
    const signature = JSON.stringify(names);
    if (good >= names.length) good = 0;
    if (signature !== goodNames) {
      goodNames = signature;
      goodSelect.replaceChildren(...names.map((name, i) => h('option', { value: String(i) }, name)));
    }
    goodSelect.value = String(good);
  }

  /** Image import (Decision 12): for the paint tool's good, max capacity 0–10, optionally inverted. */
  let importMax = 4;
  let invert = false;
  const importStatus = h('span', { class: 'hint', 'aria-live': 'polite' });
  const fileInput = h('input', { type: 'file', accept: 'image/*', hidden: true, 'aria-label': 'Image to import' });
  fileInput.addEventListener('change', async () => {
    const file = fileInput.files?.[0];
    fileInput.value = '';
    if (!file) return;
    try {
      const { width, height } = engine.size();
      const capacities = capacitiesFromPixels(await readImagePixels(file, width, height), importMax, invert);
      const errors = engine.importLandscape(good, capacities);
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

  /** Refills the picker when the disease list has grown (outbreaks, mutations, Infect), or when forced. */
  function refreshPicker(force = false): void {
    if (!DISEASE_TOOLS.includes(tool) || !engine.config.disease.enabled) return;
    const list = engine.diseaseList();
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
    grid.brushRadius = brushed() ? radius : null;
    if (tool === 'paint') {
      refreshGoods();
      engine.setDisplay({ layer: `capacity:${good}` });
    }
    if (DISEASE_TOOLS.includes(tool)) engine.setDisplay({ colorMode: 'disease' });
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
    grid.draw();
  }

  /** Hides the disease tools while disease is off (leaving them if one was active). */
  function syncAvailability(): void {
    const on = engine.config.disease.enabled;
    buttons.forEach((b, i) => {
      if (DISEASE_TOOLS.includes(TOOLS[i][0])) b.hidden = !on;
    });
    if (!on && DISEASE_TOOLS.includes(tool)) choose('inspect');
    else refreshPicker(true);
    // Keeps the paint tool's layer and inputs; only the good list follows the config.
    refreshGoods();
  }

  grid.onCell = (x, y, kind) => {
    switch (tool) {
      case 'inspect':
        if (kind === 'down') {
          engine.select(x, y);
          onInspect();
        }
        break;
      case 'paint':
        engine.paint(x, y, radius, value, good);
        break;
      case 'place':
        if (kind === 'down') {
          const o: PlaceOverrides = {};
          if (sex) o.sex = sex;
          if (tribe) o.tribe = tribe;
          engine.place(x, y, o);
        }
        break;
      case 'erase':
        engine.erase(x, y);
        break;
      case 'infect':
        if (kind === 'down') engine.infect(x, y, disease);
        break;
      case 'vaccinate':
        engine.vaccinate(x, y, radius, disease);
        break;
    }
  };

  engine.on('reset', syncAvailability);
  engine.on('config', syncAvailability);
  engine.on('tick', () => refreshPicker());
  engine.on('edit', () => refreshPicker());
  choose('inspect');
  syncAvailability();
  return h('div', { class: 'tools' }, h('div', { class: 'tool-buttons' }, ...buttons), options);
}
