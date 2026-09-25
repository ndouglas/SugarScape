import type { Engine, Overlay } from '../engine';
import { layerOptions, overlayAvailableAny } from '../layers';
import { COLOR_MODES } from '../models';
import { middleSlice, sliceOptions } from '../spatial';
import type { ColorMode, Layer, ModelKind, SpatialConfig } from '../types';
import { VALLEY_OVERLAY_LABELS } from '../valley';
import { h } from './dom';

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

/**
 * The Agents and Landscape menus and the overlay checkboxes, for the model on screen (Decision 12):
 * a sugarscape offers all of them; Schelling only its three color modes; Ring World none; the
 * anasazi its three modes (as "Valley") and its water, settlement and link overlays; the spatial
 * games their three modes and, in a cube, the Slice menu.
 */
export function buildDisplay(engine: Engine): Display {
  const mode = h('select', { onchange: () => engine.setDisplay({ colorMode: mode.value as ColorMode }) });
  const layer = h('select', { onchange: () => engine.setDisplay({ layer: layer.value as Layer }) });
  const modeName = h('span', {}, 'Agents ');
  const modeLabel = h('label', {}, modeName, mode);
  const layerName = h('span', {}, 'Landscape ');
  const layerLabel = h('label', {}, layerName, layer);
  const sync = () => {
    mode.value = engine.colorMode;
    layer.value = engine.layer;
  };
  let shownModel: ModelKind | null = null;
  /** The model's color modes (when the model changed), and the goods and pollutants (and their names), which change on reset and config. */
  const refill = () => {
    const model = engine.model;
    if (model !== shownModel) {
      shownModel = model;
      const modes = COLOR_MODES[model];
      mode.replaceChildren(...modes.map(([v, l]) => h('option', { value: v }, l)));
      modeLabel.hidden = modes.length === 0;
      modeName.textContent = model === 'anasazi' ? 'Valley ' : 'Agents ';
    }
    // A cube's view is one z-slice (the Slice menu); a sugarscape's landscape layers otherwise.
    const slices = model === 'spatial' ? sliceOptions(engine.config as SpatialConfig) : [];
    layerName.textContent = slices.length > 0 ? 'Slice ' : 'Landscape ';
    layerLabel.hidden = model !== 'sugarscape' && slices.length === 0;
    const options = model === 'sugarscape' ? layerOptions(engine.sugar) : slices;
    layer.replaceChildren(...options.map(([v, l]) => h('option', { value: v }, l)));
    if (slices.length > 0 && !slices.some(([v]) => v === engine.layer)) engine.setDisplay({ layer: middleSlice(engine.config as SpatialConfig) });
    sync();
  };
  engine.on('display', sync);
  engine.on('reset', refill);
  engine.on('config', refill);
  refill();
  let b: Engine | null = null;
  const configs = () => (b ? [engine.sugar, b.sugar] : [engine.sugar]);
  /** A checkbox per overlay (sugarscape only); Friends, Family and Disease show while either world on screen allows them. */
  const overlay = (kind: Overlay, label: string) => {
    const box = h('input', { type: 'checkbox', onchange: () => engine.setDisplay({ overlays: { [kind]: box.checked } }) });
    const el = h('label', {}, box, ` ${label}`);
    const valley = VALLEY_OVERLAY_LABELS.some(([k]) => k === kind);
    const show = () =>
      (el.hidden = valley ? engine.model !== 'anasazi' : engine.model !== 'sugarscape' || !overlayAvailableAny(kind, configs()));
    engine.on('display', () => (box.checked = engine.overlays[kind]));
    engine.on('reset', show);
    engine.on('config', show);
    show();
    return { el, show };
  };
  const overlays = [...OVERLAY_LABELS, ...VALLEY_OVERLAY_LABELS].map(([kind, label]) => overlay(kind, label));
  const el = h('div', { class: 'display-controls' }, modeLabel, layerLabel, ...overlays.map((o) => o.el));
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
