import { OVERLAYS, type DisplayState, type Overlay } from './protocol';
import type { Config, Layer } from './types';

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
 * `d` kept valid for `config` (the host applies it to every snapshot): a layer the world lacks falls
 * back to good 0's level; with disease off the Disease color mode falls back to Tribe; an overlay
 * the world cannot show (`overlayAvailable`) is turned off. Returns `d` itself when nothing changes.
 */
export function clampDisplay(d: DisplayState, config: Config): DisplayState {
  const layer = validLayer(d.layer, config);
  const colorMode = !config.disease.enabled && d.colorMode === 'disease' ? 'tribe' : d.colorMode;
  const off = OVERLAYS.filter((k) => d.overlays[k] && !overlayAvailable(k, config));
  if (layer === d.layer && colorMode === d.colorMode && off.length === 0) return d;
  const overlays = { ...d.overlays };
  for (const k of off) overlays[k] = false;
  return { colorMode, layer, overlays };
}
