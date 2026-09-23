import type { Engine, Overlay } from '../engine';
import type { ColorMode, Layer } from '../types';
import { h } from './dom';

const MODES: [ColorMode, string][] = [
  ['tribe', 'Tribe'],
  ['wealth', 'Wealth'],
  ['sex', 'Sex'],
  ['age', 'Age'],
  ['vision', 'Vision'],
  ['credit', 'Credit'],
  ['disease', 'Disease'],
];
const LAYERS: [Layer, string][] = [
  ['sugar', 'Sugar'],
  ['capacity', 'Capacity'],
  ['pollution', 'Pollution'],
  ['spice', 'Spice'],
  ['spice_capacity', 'Spice capacity'],
];

export function buildDisplay(engine: Engine): HTMLElement {
  const mode = h(
    'select',
    { onchange: () => engine.setDisplay({ colorMode: mode.value as ColorMode }) },
    ...MODES.map(([v, l]) => h('option', { value: v }, l)),
  );
  const layer = h(
    'select',
    { onchange: () => engine.setDisplay({ layer: layer.value as Layer }) },
    ...LAYERS.map(([v, l]) => h('option', { value: v }, l)),
  );
  const sync = () => {
    mode.value = engine.colorMode;
    layer.value = engine.layer;
  };
  engine.on('display', sync);
  sync();
  const overlay = (kind: Overlay, label: string) => {
    const box = h('input', { type: 'checkbox', onchange: () => engine.setDisplay({ overlays: { [kind]: box.checked } }) });
    engine.on('display', () => (box.checked = engine.overlays[kind]));
    return h('label', {}, box, ` ${label}`);
  };
  return h(
    'div',
    { class: 'display-controls' },
    h('label', {}, 'Agents ', mode),
    h('label', {}, 'Landscape ', layer),
    overlay('trade', 'Trade network'),
    overlay('credit', 'Credit network'),
    overlay('disease', 'Disease network'),
  );
}
