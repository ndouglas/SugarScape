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
