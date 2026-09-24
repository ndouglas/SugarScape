import type { Engine, Overlay } from '../engine';
import type { ColorMode, Layer } from '../types';
import { h } from './dom';
import { layerOptions, overlayAvailable } from '../layers';

const MODES: [ColorMode, string][] = [
  ['tribe', 'Tribe'],
  ['wealth', 'Wealth'],
  ['sex', 'Sex'],
  ['age', 'Age'],
  ['vision', 'Vision'],
  ['credit', 'Credit'],
  ['disease', 'Disease'],
  ['lineage', 'Lineage'],
];

/** The overlay checkboxes, in order (Chapter VI's three networks after the others). */
const OVERLAY_LABELS: [Overlay, string][] = [
  ['trade', 'Trade network'],
  ['credit', 'Credit network'],
  ['disease', 'Disease network'],
  ['neighbors', 'Neighbor network'],
  ['friends', 'Friends network'],
  ['family', 'Family network'],
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
  /** A checkbox per overlay; Friends and Family show only while culture and sex are on (and Disease while disease is). */
  const overlay = (kind: Overlay, label: string) => {
    const box = h('input', { type: 'checkbox', onchange: () => engine.setDisplay({ overlays: { [kind]: box.checked } }) });
    const el = h('label', {}, box, ` ${label}`);
    const show = () => (el.hidden = !overlayAvailable(kind, engine.config));
    engine.on('display', () => (box.checked = engine.overlays[kind]));
    engine.on('reset', show);
    engine.on('config', show);
    show();
    return el;
  };
  return h(
    'div',
    { class: 'display-controls' },
    h('label', {}, 'Agents ', mode),
    h('label', {}, 'Landscape ', layer),
    ...OVERLAY_LABELS.map(([kind, label]) => overlay(kind, label)),
  );
}
