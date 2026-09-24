import type { DisplayState } from './protocol';
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

/**
 * `d` kept valid for `config` (the host applies it to every snapshot): a layer the world lacks falls
 * back to good 0's level; with disease off the Disease color mode and overlay fall back to Tribe and
 * hidden. Returns `d` itself when nothing changes.
 */
export function clampDisplay(d: DisplayState, config: Config): DisplayState {
  const layer = validLayer(d.layer, config);
  const off = !config.disease.enabled;
  const colorMode = off && d.colorMode === 'disease' ? 'tribe' : d.colorMode;
  const disease = off ? false : d.overlays.disease;
  if (layer === d.layer && colorMode === d.colorMode && disease === d.overlays.disease) return d;
  return { colorMode, layer, overlays: { ...d.overlays, disease } };
}
