import type { Engine, Overlay } from '../engine';
import type { ColorMode, Layer } from '../types';
import { h } from './dom';
import { layerOptions, overlayAvailableAny } from '../layers';

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
/** The display controls' element, and (Decision 10) which worlds' configs its checkboxes offer for. */
export interface Display {
  el: HTMLElement;
  /** Compare on (a checkbox shows while either A's or B's config allows it) or off (A's alone). */
  setCompare(b: Engine | null): void;
}

export function buildDisplay(engine: Engine): Display {
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
    layer.replaceChildren(...layerOptions(engine.sugar).map(([v, l]) => h('option', { value: v }, l)));
    sync();
  };
  engine.on('display', sync);
  engine.on('reset', refill);
  engine.on('config', refill);
  refill();
  let b: Engine | null = null;
  const configs = () => (b ? [engine.sugar, b.sugar] : [engine.sugar]);
  /** A checkbox per overlay; Friends, Family and Disease show while either world on screen allows them. */
  const overlay = (kind: Overlay, label: string) => {
    const box = h('input', { type: 'checkbox', onchange: () => engine.setDisplay({ overlays: { [kind]: box.checked } }) });
    const el = h('label', {}, box, ` ${label}`);
    const show = () => (el.hidden = !overlayAvailableAny(kind, configs()));
    engine.on('display', () => (box.checked = engine.overlays[kind]));
    engine.on('reset', show);
    engine.on('config', show);
    show();
    return { el, show };
  };
  const overlays = OVERLAY_LABELS.map(([kind, label]) => overlay(kind, label));
  const el = h(
    'div',
    { class: 'display-controls' },
    h('label', {}, 'Agents ', mode),
    h('label', {}, 'Landscape ', layer),
    ...overlays.map((o) => o.el),
  );
  let offB: (() => void)[] = [];
  const setCompare = (newB: Engine | null): void => {
    for (const off of offB) off();
    offB = [];
    b = newB;
    if (b) {
      const other = b;
      const show = () => overlays.forEach((o) => o.show());
      offB = [other.on('reset', show), other.on('config', show)];
    }
    overlays.forEach((o) => o.show());
  };
  return { el, setCompare };
}
