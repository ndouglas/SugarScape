import { cachingOn, COLOR_MODES, hasCaches, isSugar, MODEL_OVERLAYS, modelOf, theftOn } from './models';
import { OVERLAYS, type DisplayState, type Overlay } from './protocol';
import type { ColorMode, Config, Layer, ModelConfig } from './types';

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

/**
 * Whether an overlay can show anything in a sugarscape with `config`: disease, friends and family
 * need their rules; the anasazi's overlays never show in a sugarscape.
 */
export function overlayAvailable(kind: Overlay, config: Config): boolean {
  switch (kind) {
    case 'disease':
      return config.disease.enabled;
    case 'friends':
      return config.culture.enabled;
    case 'family':
      return config.sex.enabled;
    case 'water':
    case 'settlements':
    case 'links':
      return false;
    // Minds 5–6: caches exist under caching (a central world's larders are caches too).
    case 'caches':
      return hasCaches(config);
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
 * A sugarscape's own color mode for `config`: Strategy (hoarder or cheater) with theft on, Caching
 * rule under mixed rules, else Tribe (whose groups are random in the Minds worlds). Loading a preset
 * picks it; `clampDisplay` falls back to it.
 */
export function defaultColorMode(config: Config): ColorMode {
  if (theftOn(config)) return 'strategy';
  if (config.caching?.mixed === true) return 'caching_rule';
  return 'tribe';
}

/**
 * The display a freshly loaded `config` starts with, from `d`: in a sugarscape, the color mode moves
 * to `defaultColorMode` when it was Tribe or another default (a mode picked by hand stays), and the
 * all-caches overlay turns on where there are caches. Returns `d` itself when nothing changes.
 */
export function loadedDisplay(d: DisplayState, config: ModelConfig): DisplayState {
  if (!isSugar(config)) return d;
  const defaults: ColorMode[] = ['tribe', 'strategy', 'caching_rule'];
  const colorMode = defaults.includes(d.colorMode) ? defaultColorMode(config) : d.colorMode;
  const caches = overlayAvailable('caches', config);
  if (colorMode === d.colorMode && caches === d.overlays.caches) return d;
  return { ...d, colorMode, overlays: { ...d.overlays, caches } };
}

/** Whether sugarscape color mode `mode` means anything in `config`. */
function colorModeAvailable(mode: ColorMode, config: Config): boolean {
  switch (mode) {
    case 'disease':
      return config.disease.enabled;
    case 'strategy':
      return theftOn(config);
    case 'caching_rule':
      return cachingOn(config);
    default:
      return COLOR_MODES.sugarscape.some(([m]) => m === mode);
  }
}

/**
 * `d` kept valid for `config` (the host applies it to every snapshot). In a sugarscape: a layer the
 * world lacks falls back to good 0's level; another model's color mode, Disease with disease off,
 * Strategy with theft off or Caching rule with caching off falls back to `defaultColorMode`; an overlay the world cannot show (`overlayAvailable`) is turned off. In
 * another model: a color mode it lacks falls back to its first, an overlay it does not draw
 * (`MODEL_OVERLAYS`) is off and the layer is kept (unused). Returns `d` itself when nothing changes.
 */
export function clampDisplay(d: DisplayState, config: ModelConfig): DisplayState {
  if (!isSugar(config)) {
    const model = modelOf(config);
    const modes = COLOR_MODES[model].map(([m]) => m);
    const colorMode = modes.length === 0 || modes.includes(d.colorMode) ? d.colorMode : modes[0];
    const off = OVERLAYS.filter((k) => d.overlays[k] && !MODEL_OVERLAYS[model].includes(k));
    if (colorMode === d.colorMode && off.length === 0) return d;
    const overlays = { ...d.overlays };
    for (const k of off) overlays[k] = false;
    return { colorMode, layer: d.layer, overlays };
  }
  const layer = validLayer(d.layer, config);
  const colorMode = colorModeAvailable(d.colorMode, config) ? d.colorMode : defaultColorMode(config);
  const off = OVERLAYS.filter((k) => d.overlays[k] && !overlayAvailable(k, config));
  if (layer === d.layer && colorMode === d.colorMode && off.length === 0) return d;
  const overlays = { ...d.overlays };
  for (const k of off) overlays[k] = false;
  return { colorMode, layer, overlays };
}
