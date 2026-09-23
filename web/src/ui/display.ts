import type { Engine, Overlay } from '../engine';
import type { ColorMode, Layer } from '../types';
import { h } from './dom';
import { layerOptions } from '../layers';

const MODES: [ColorMode, string][] = [
  ['tribe', 'Tribe'],
  ['wealth', 'Wealth'],
  ['sex', 'Sex'],
  ['age', 'Age'],
  ['vision', 'Vision'],
  ['credit', 'Credit'],
  ['disease', 'Disease'],
];
export function buildDisplay(engine: Engine): HTMLElement {
  const mode = h(
    'select',
    { onchange: () => engine.setDisplay({ colorMode: mode.value as ColorMode }) },
    ...MODES.map(([v, l]) => h('option', { value: v }, l)),
  );
  const layer = h('select', { onchange: () => engine.setDisplay({ layer: layer.value as Layer }) });
  const sync = () => {
    mode.value = engine.colorMode;
    layer.value = engine.layer;
  };
  /** Goods and pollutants (and their names) change on reset and config. */
  const refill = () => {
    layer.replaceChildren(...layerOptions(engine.config).map(([v, l]) => h('option', { value: v }, l)));
    sync();
  };
  engine.on('display', sync);
  engine.on('reset', refill);
  engine.on('config', refill);
  refill();
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
