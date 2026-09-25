import { COLOR_MODES, isSugar, modelOf } from './models';
import { noOverlays, OVERLAYS, type DisplayState, type Overlay } from './protocol';
import type { Config, Layer, ModelConfig } from './types';

/** The Landscape selector: each good's level and capacity (by name), then each pollutant. */
export function layerOptions(config: Config): [Layer, string][] {
  const out: [Layer, string][] = [];
  config.goods.forEach((g, i) => out.push([`resource:${i}`, g.name], [`capacity:${i}`, `${g.name} capacity`]));
  config.pollution.pollutants.forEach((p, k) => out.push([`pollution:${k}`, p.name]));
  return out;
}

/** `layer` if this config has it, else good 0's level. */
export function validLayer(layer: Layer, config: Config): Layer {
  return layerOptions(config).some(([l]) => l === layer) ? layer : 'resource:0';
}

/** Whether an overlay can show anything in a world with `config`: disease, friends and family need their rules. */
export function overlayAvailable(kind: Overlay, config: Config): boolean {
  switch (kind) {
    case 'disease':
      return config.disease.enabled;
    case 'friends':
      return config.culture.enabled;
    case 'family':
      return config.sex.enabled;
    default:
      return true;
  }
}

/**
 * Whether an overlay's checkbox should be offered given every world on screen (Compare: A's and
 * B's `config`s) — available if any one of them would show it. Each world still only draws what
 * its own config allows (`clampDisplay` per world).
 */
export function overlayAvailableAny(kind: Overlay, configs: Config[]): boolean {
  return configs.some((c) => overlayAvailable(kind, c));
}

/**
 * `d` kept valid for `config` (the host applies it to every snapshot). In a sugarscape: a layer the
 * world lacks falls back to good 0's level; another model's color mode, or Disease with disease off,
 * falls back to Tribe; an overlay the world cannot show (`overlayAvailable`) is turned off. In
 * another model: a color mode it lacks falls back to its first, every overlay is off and the layer
 * is kept (unused). Returns `d` itself when nothing changes.
 */
export function clampDisplay(d: DisplayState, config: ModelConfig): DisplayState {
  if (!isSugar(config)) {
    const modes = COLOR_MODES[modelOf(config)].map(([m]) => m);
    const colorMode = modes.length === 0 || modes.includes(d.colorMode) ? d.colorMode : modes[0];
    if (colorMode === d.colorMode && !OVERLAYS.some((k) => d.overlays[k])) return d;
    return { colorMode, layer: d.layer, overlays: noOverlays() };
  }
  const layer = validLayer(d.layer, config);
  const sugarMode = COLOR_MODES.sugarscape.some(([m]) => m === d.colorMode);
  const colorMode = !sugarMode || (!config.disease.enabled && d.colorMode === 'disease') ? 'tribe' : d.colorMode;
  const off = OVERLAYS.filter((k) => d.overlays[k] && !overlayAvailable(k, config));
  if (layer === d.layer && colorMode === d.colorMode && off.length === 0) return d;
  const overlays = { ...d.overlays };
  for (const k of off) overlays[k] = false;
  return { colorMode, layer, overlays };
}
